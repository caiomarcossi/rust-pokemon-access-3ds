//! Client for Azahar's GDB stub: watchpoints, so text can be found by who writes it.
//!
//! Written from the stub's own `src/core/gdbstub/gdbstub.cpp`, not from GDB. It speaks the Remote
//! Serial Protocol only as far as this stub implements it.
//!
//! ```text
//! $<payload>#<checksum>   both directions, checksum = sum of payload bytes mod 256
//! +                       the stub acks every well-formed command before answering it
//! ```
//!
//! **Why this exists alongside `rpc`.** The RPC finds text by scanning for something that looks like
//! prose and then polling it. That is a guess confirmed after the fact. A write watchpoint is the
//! opposite: the emulator stops on the instruction that wrote the bytes and hands over the program
//! counter, so the answer comes from the game's own code rather than from a pattern that matched.
//!
//! **Watchpoints work under the JIT.** Azahar's `MemorySystem::RegisterWatchpoint` nulls the watched
//! page's entry in the page table dynarmic reads, which pushes every access to that page back through
//! `Memory::Write<T>`, where the stub checks its breakpoint list. Only the watched pages pay for it.
//! This is Azahar's own addition; Citra could not do it.
//!
//! Three facts that will otherwise cost an afternoon:
//!
//! - **The stub is served from the CPU loop** (`System::RunLoop`), so a *paused* emulator never
//!   answers and never even accepts the connection. This is the exact opposite of snapshot capture,
//!   which must be paused. Never hold both at once.
//! - **`attach()` halts the game.** The stub stops the process to report why it stopped. Nothing runs
//!   again until `resume()`, and a client that disconnects while halted leaves the game frozen, so
//!   every path out of here detaches.
//! - **There is no single step.** The stub answers `s` with `E5F`. Execute breakpoints are therefore
//!   awkward to resume past, and watchpoints are the tool of choice.
//!
//! Under the JIT a watchpoint's reported program counter is approximate — dynarmic does not write the
//! PC back per instruction, and `ARM_Dynarmic::HasSingleInstructionBreakAccuracy` says so. It lands in
//! the right function, which is already enough to replace a scan. For the exact instruction, run the
//! emulator with `use_cpu_jit=false`, where the interpreter is accurate.
//!
//! Bound by:
//! - ADR-0002: a tool for finding things out; nothing a player runs may use it.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::time::Duration;

use indexmap::{IndexMap, IndexSet};

use crate::py::{self, Error, Kind, Result, time};

// `GdbError` ("The stub did not answer as expected.") and `GdbTimeoutError` ("Nothing arrived in
// time. A paused emulator looks exactly like this.") are `py::Kind::GdbError` and
// `py::Kind::GdbTimeoutError`.

pub const DEFAULT_HOST: &str = "127.0.0.1";
/// `gdbstub_port` in `qt-config.ini`, which must also carry `use_gdbstub=true`.
pub const DEFAULT_PORT: u16 = 24689;

/// The reply buffer holds 9996 hex characters, and the stub garbles a read that overflows it.
pub const MAX_READ: i64 = 4096;

// The bytes of the wire stay `u8`: they are what a socket carries, not numbers pa3ds computes with.
const PACKET_START: u8 = b'$';
const PACKET_END: u8 = b'#';
const ACK: u8 = b'+';
const NACK: u8 = b'-';
const INTERRUPT: &[u8] = b"\x03";

const STOP_PREFIXES: [char; 4] = ['T', 'S', 'W', 'X'];

const SP: i64 = 13;
const LR: i64 = 14;
const PC: i64 = 15;
const THUMB_BIT: i64 = 1 << 5;

/// `Z`/`z` packet kinds, as the stub decodes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Watch {
    Execute = 1,
    Write = 2,
    Read = 3,
    Access = 4,
}

impl Watch {
    pub fn value(self) -> i64 {
        self as i64
    }
}

/// Why the game halted, from a `T` stop reply.
///
/// `pc`, `sp` and `lr` are absent when the stub had no thread to report them from.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stop {
    pub signal: i64,
    pub pc: Option<i64>,
    pub sp: Option<i64>,
    pub lr: Option<i64>,
    pub thread: Option<i64>,
}

/// `r0` through `r15` and the CPSR, as the halted thread saw them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    pub r: Vec<i64>,
    pub cpsr: i64,
}

impl Registers {
    pub fn sp(&self) -> i64 {
        self.r[SP as usize]
    }

    pub fn lr(&self) -> i64 {
        self.r[LR as usize]
    }

    pub fn pc(&self) -> i64 {
        self.r[PC as usize]
    }

    pub fn thumb(&self) -> bool {
        self.cpsr & THUMB_BIT != 0
    }

    /// `r0`-`r3`: the first four arguments under the ARM calling convention.
    pub fn args(&self) -> (i64, i64, i64, i64) {
        (self.r[0], self.r[1], self.r[2], self.r[3])
    }
}

/// `f"{value:x}"`: Python writes a negative number with a sign, Rust in two's complement.
fn hex(value: i64) -> String {
    if value < 0 {
        format!("-{:x}", value.unsigned_abs())
    } else {
        format!("{value:x}")
    }
}

/// `f"{value:#010x}"`.
fn hex10(value: i64) -> String {
    if value < 0 {
        format!("-{:#09x}", value.unsigned_abs())
    } else {
        format!("{value:#010x}")
    }
}

#[track_caller]
fn checksum(payload: &str) -> Result<i64> {
    Ok(py::encode_ascii(payload)?
        .iter()
        .map(|&b| i64::from(b))
        .sum::<i64>()
        & 0xFF)
}

/// The stub writes register values as little-endian bytes in hex.
#[track_caller]
fn from_hex_le(text: &str) -> Result<i64> {
    let bytes = py::fromhex(text)?;
    Ok(bytes
        .iter()
        .rev()
        .fold(0i64, |acc, &byte| acc.wrapping_shl(8) | i64::from(byte)))
}

/// `text[start:end]` by character, clamped.
fn chars_slice(text: &str, start: i64, end: i64) -> String {
    let chars: Vec<char> = text.chars().collect();
    py::slice(&chars, start, end).iter().collect()
}

fn is_stop(reply: &str) -> bool {
    reply
        .chars()
        .next()
        .is_some_and(|first| STOP_PREFIXES.contains(&first))
}

/// Decode `T05 0f:<pc>;0d:<sp>;0e:<lr>;thread:<id>;`, any field of which may be missing.
fn parse_stop(reply: &str) -> Result<Stop> {
    if !is_stop(reply) {
        return Err(Error::new(
            Kind::GdbError,
            format!("not a stop reply: {}", py::repr(reply)),
        ));
    }
    if reply.starts_with(['W', 'X']) {
        let digits = chars_slice(reply, 1, 3);
        return Ok(Stop {
            signal: py::int_base16(if digits.is_empty() { "0" } else { &digits })?,
            ..Stop::default()
        });
    }
    let signal = py::int_base16(&chars_slice(reply, 1, 3))?;
    let mut fields: IndexMap<String, String> = IndexMap::new();
    for item in chars_slice(reply, 3, reply.chars().count() as i64).split(';') {
        let (key, value) = item.split_once(':').unwrap_or((item, ""));
        if !value.is_empty() {
            fields.insert(key.to_lowercase(), value.to_string());
        }
    }
    let thread = fields.get("thread");
    let pc = match fields.get("0f") {
        Some(text) => Some(from_hex_le(text)?),
        None => None,
    };
    let sp = match fields.get("0d") {
        Some(text) => Some(from_hex_le(text)?),
        None => None,
    };
    let lr = match fields.get("0e") {
        Some(text) => Some(from_hex_le(text)?),
        None => None,
    };
    let thread = match thread {
        Some(text) if !text.is_empty() => Some(py::int_base16(text)?),
        _ => None,
    };
    Ok(Stop {
        signal,
        pc,
        sp,
        lr,
        thread,
    })
}

/// A connection to one emulator's GDB stub.
///
/// Use it as a context manager. Leaving removes every breakpoint this client set and detaches,
/// which is not politeness: a halted game that loses its client stays halted.
///
/// In Rust, leaving is `close` or dropping the client; a drop swallows what `close` would return.
pub struct Gdb {
    timeout: f64,
    buffer: VecDeque<u8>,
    stops: Vec<Stop>,
    watches: IndexSet<(Watch, i64, i64)>,
    attached: bool,
    sock: TcpStream,
    closed: bool,
}

impl Gdb {
    /// The stub on this machine, at `port`, with the default timeout.
    pub fn connect(port: u16) -> Result<Self> {
        Gdb::new(DEFAULT_HOST, port, 2.0)
    }

    pub fn new(host: &str, port: u16, timeout: f64) -> Result<Self> {
        // `socket.create_connection`: each address the host resolves to in turn, the last
        // refusal raised if none answers.
        let mut last: Option<Error> = None;
        let mut sock: Option<TcpStream> = None;
        for addr in (host, port).to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, Duration::from_secs_f64(timeout)) {
                Ok(stream) => {
                    sock = Some(stream);
                    break;
                }
                Err(error) => last = Some(Error::from(error)),
            }
        }
        let Some(sock) = sock else {
            return Err(last.unwrap_or_else(|| {
                Error::new(Kind::OsError, "getaddrinfo returns an empty list")
            }));
        };
        sock.set_nodelay(true)?;
        Ok(Gdb {
            timeout,
            buffer: VecDeque::new(),
            stops: Vec::new(),
            watches: IndexSet::new(),
            attached: false,
            sock,
            closed: false,
        })
    }

    // -- lifecycle ----------------------------------------------------------------

    /// Undo everything and let the game run.
    ///
    /// **Resume before detaching, always.** Azahar's `D` handler shuts the server down before it
    /// continues the process, and shutting down clears the `current_process` that `Continue()`
    /// needs, so `Continue()` returns having done nothing. Detaching from a halted game leaves it
    /// halted for good — no window, no error, just a frozen picture. Measured on 2126.0: after a
    /// bare `D` the game stopped advancing entirely, and a fresh connection sending `c` restarted
    /// it. So `c` goes first and `D` only tidies up the stub afterwards.
    ///
    /// Errors are swallowed here: the connection may already be gone, and raising would hide
    /// whatever sent us here.
    pub fn close(&mut self) -> Result<()> {
        let undone = self.undo();
        self.attached = false;
        let _ = self.sock.shutdown(Shutdown::Both);
        self.closed = true;
        undone
    }

    fn undo(&mut self) -> Result<()> {
        let mut watches: Vec<(Watch, i64, i64)> = self.watches.iter().copied().collect();
        watches.sort();
        for (kind, address, size) in watches {
            let command = format!("z{},{},{}", kind.value(), hex(address), hex(size));
            suppress_gdb(self.command(&command, None, false).map(|_| ()))?;
        }
        self.watches.clear();
        if self.attached {
            suppress_gdb(self.resume())?;
            suppress_gdb(self.command("D", None, false).map(|_| ()))?;
        }
        Ok(())
    }

    // -- framing ------------------------------------------------------------------

    fn fill(&mut self, deadline: f64) -> Result<()> {
        let remaining = deadline - time::monotonic();
        if remaining <= 0.0 {
            return Err(Error::new(
                Kind::GdbTimeoutError,
                "the stub did not answer; is emulation paused?",
            ));
        }
        // A timeout of zero is refused by Rust and not by Python, where `remaining` can be as
        // small as a float goes.
        self.sock.set_read_timeout(Some(
            Duration::from_secs_f64(remaining).max(Duration::from_nanos(1)),
        ))?;
        let mut chunk = [0u8; 4096];
        let got = match self.sock.read(&mut chunk) {
            Ok(got) => got,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                return Err(Error::new(
                    Kind::GdbTimeoutError,
                    "the stub did not answer; is emulation paused?",
                ));
            }
            Err(error) => return Err(Error::from(error)),
        };
        if got == 0 {
            return Err(Error::new(
                Kind::GdbError,
                "the emulator closed the connection",
            ));
        }
        self.buffer.extend(&chunk[..got]);
        Ok(())
    }

    fn byte(&mut self, deadline: f64) -> Result<u8> {
        while self.buffer.is_empty() {
            self.fill(deadline)?;
        }
        Ok(self.buffer.pop_front().expect("filled"))
    }

    /// Read one `$...#cc` and ack it. Stray acks ahead of it are skipped.
    fn packet(&mut self, deadline: f64) -> Result<String> {
        loop {
            let first = self.byte(deadline)?;
            if first != PACKET_START {
                continue;
            }
            let mut body: Vec<u8> = Vec::new();
            loop {
                let char = self.byte(deadline)?;
                if char == PACKET_END {
                    break;
                }
                body.push(char);
            }
            let got: String = [
                char::from(self.byte(deadline)?),
                char::from(self.byte(deadline)?),
            ]
            .iter()
            .collect();
            let payload = py::decode_ascii(&body);
            if py::int_base16(&got)? != checksum(&payload)? {
                self.sock.write_all(&[NACK])?;
                return Err(Error::new(
                    Kind::GdbError,
                    format!("checksum mismatch on reply {}", py::repr(&payload)),
                ));
            }
            self.sock.write_all(&[ACK])?;
            return Ok(payload);
        }
    }

    fn await_ack(&mut self, deadline: f64) -> Result<()> {
        loop {
            let char = self.byte(deadline)?;
            if char == ACK {
                return Ok(());
            }
            if char == NACK {
                return Err(Error::new(Kind::GdbError, "the stub rejected the command"));
            }
            if char == PACKET_START {
                self.buffer.push_front(char);
                return Ok(());
            }
        }
    }

    fn send(&mut self, payload: &str) -> Result<()> {
        let framed = format!("${payload}#{:02x}", checksum(payload)?);
        self.sock.write_all(framed.as_bytes())?;
        Ok(())
    }

    /// Send a command and return its reply.
    ///
    /// The game can halt at any moment, so a stop reply may arrive between the command and its
    /// answer. Those are put aside for `wait()` rather than mistaken for the answer — except for
    /// `?`, whose whole answer *is* a stop reply, which is what `stop_is_reply` says.
    ///
    /// Python's defaults are no `timeout` and `stop_is_reply` false.
    fn command(
        &mut self,
        payload: &str,
        timeout: Option<f64>,
        stop_is_reply: bool,
    ) -> Result<String> {
        let deadline = time::monotonic() + timeout.unwrap_or(self.timeout);
        self.send(payload)?;
        self.await_ack(deadline)?;
        loop {
            let reply = self.packet(deadline)?;
            if is_stop(&reply) && !stop_is_reply {
                let stop = parse_stop(&reply)?;
                self.stops.push(stop);
                continue;
            }
            if reply.chars().count() == 3 && reply.starts_with('E') {
                return Err(Error::new(
                    Kind::GdbError,
                    format!("{} refused with {reply}", py::repr(payload)),
                ));
            }
            return Ok(reply);
        }
    }

    /// For `c`, which is acked and then answers nothing until the game stops.
    fn send_only(&mut self, payload: &str) -> Result<()> {
        let deadline = time::monotonic() + self.timeout;
        self.send(payload)?;
        self.await_ack(deadline)
    }

    // -- control ------------------------------------------------------------------

    /// Bind to the running title and halt it.
    ///
    /// This must come first: the stub refuses breakpoints with `E01` until a `?` has told it
    /// which process it is debugging. The game does not run again until `resume()`.
    pub fn attach(&mut self) -> Result<Stop> {
        let reply = self.command("?", None, true)?;
        let stop = parse_stop(&reply)?;
        self.attached = true;
        Ok(stop)
    }

    pub fn resume(&mut self) -> Result<()> {
        self.send_only("c")
    }

    /// Halt a running game the way Ctrl-C does in GDB: a bare `0x03`, with no packet framing.
    pub fn interrupt(&mut self, timeout: Option<f64>) -> Result<Stop> {
        self.sock.write_all(INTERRUPT)?;
        self.wait(timeout)
    }

    /// Block until the game stops. Raises `GdbTimeoutError` if it never does.
    pub fn wait(&mut self, timeout: Option<f64>) -> Result<Stop> {
        if !self.stops.is_empty() {
            return Ok(self.stops.remove(0));
        }
        let deadline = time::monotonic() + timeout.unwrap_or(self.timeout);
        loop {
            let reply = self.packet(deadline)?;
            if is_stop(&reply) {
                return parse_stop(&reply);
            }
        }
    }

    /// The stop that has already arrived, or None. Never blocks for long.
    pub fn poll(&mut self) -> Result<Option<Stop>> {
        if !self.stops.is_empty() {
            return Ok(Some(self.stops.remove(0)));
        }
        match self.wait(Some(0.01)) {
            Ok(stop) => Ok(Some(stop)),
            Err(error) if error.is(Kind::GdbTimeoutError) => Ok(None),
            Err(error) => Err(error),
        }
    }

    // -- state --------------------------------------------------------------------

    /// `r0`-`r15` and the CPSR of the halted thread.
    pub fn registers(&mut self) -> Result<Registers> {
        let raw = self.command("g", None, false)?;
        let length = raw.chars().count();
        if length < 17 * 8 {
            return Err(Error::new(
                Kind::GdbError,
                format!("register reply too short: {length} characters"),
            ));
        }
        let mut r = Vec::new();
        for i in 0..16 {
            r.push(from_hex_le(&chars_slice(&raw, i * 8, i * 8 + 8))?);
        }
        let cpsr = from_hex_le(&chars_slice(&raw, 16 * 8, 17 * 8))?;
        Ok(Registers { r, cpsr })
    }

    /// Read emulated memory. Unmapped addresses come back as an empty reply, not an error.
    pub fn read(&mut self, address: i64, size: i64) -> Result<Vec<u8>> {
        if !(0 < size && size <= MAX_READ) {
            return Err(Error::value(format!(
                "size must be 1..{MAX_READ}, got {size}"
            )));
        }
        let reply = self.command(&format!("m{},{}", hex(address), hex(size)), None, false)?;
        if reply.is_empty() {
            return Err(Error::new(
                Kind::GdbError,
                format!("nothing mapped at {}", hex10(address)),
            ));
        }
        py::fromhex(&reply)
    }

    // -- breakpoints --------------------------------------------------------------

    /// Stop the game when anything writes into `[address, address + size)`.
    ///
    /// Nothing in the game's memory changes; the emulator only marks the page slow. Safe to leave
    /// set while the player plays, apart from the halt itself.
    pub fn watch_writes(&mut self, address: i64, size: i64) -> Result<()> {
        self.set(Watch::Write, address, size)
    }

    /// Stop the game when anything reads the range — how to find a string's consumer.
    pub fn watch_reads(&mut self, address: i64, size: i64) -> Result<()> {
        self.set(Watch::Read, address, size)
    }

    /// Stop the game when it executes `address`.
    ///
    /// Unlike a watchpoint this **writes a BKPT into the game's code**, which is exactly what
    /// pa3ds itself is forbidden to do. It is a reverse-engineering instrument, not something a
    /// reader may use. The stub also has no single step, so the only way past the breakpoint is to
    /// remove it first, which makes it effectively one-shot.
    ///
    /// Python's default for `thumb` is false.
    pub fn break_at(&mut self, address: i64, thumb: bool) -> Result<()> {
        self.set(Watch::Execute, address, if thumb { 2 } else { 4 })
    }

    pub fn clear(&mut self, kind: Watch, address: i64, size: i64) -> Result<()> {
        self.command(
            &format!("z{},{},{}", kind.value(), hex(address), hex(size)),
            None,
            false,
        )?;
        self.watches.shift_remove(&(kind, address, size));
        Ok(())
    }

    fn set(&mut self, kind: Watch, address: i64, size: i64) -> Result<()> {
        if size <= 0 {
            return Err(Error::value(format!("size must be positive, got {size}")));
        }
        self.command(
            &format!("Z{},{},{}", kind.value(), hex(address), hex(size)),
            None,
            false,
        )?;
        self.watches.insert((kind, address, size));
        Ok(())
    }

    /// Hold a write watchpoint for the duration of a block, then take it back off.
    ///
    /// The block is given the client, as Python's `with` body reaches it by name.
    pub fn watching_writes<R>(
        &mut self,
        address: i64,
        size: i64,
        body: impl FnOnce(&mut Gdb) -> Result<R>,
    ) -> Result<R> {
        self.watch_writes(address, size)?;
        let result = body(self);
        self.clear(Watch::Write, address, size)?;
        result
    }
}

impl Drop for Gdb {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.close();
        }
    }
}

/// `with suppress(GdbError): ...`
fn suppress_gdb(result: Result<()>) -> Result<()> {
    match result {
        Err(error) if error.is(Kind::GdbError) => Ok(()),
        other => other,
    }
}
