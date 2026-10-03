//! A TCP server speaking Azahar's GDB stub protocol, so client tests need no emulator.
//!
//! It mirrors the stub in the emulator's `src/core/gdbstub/gdbstub.cpp`, quirks included: the `+`
//! ack before every reply, little-endian hex for register values, `?` halting the process before it
//! answers, `s` refused with `E5F`, an empty reply for anything unknown, and `E01` for a breakpoint
//! set before `?` has chosen a process.
//!
//! `fire()` is the part no real stub has: it pushes a stop reply at a moment the test chooses, which
//! is how a watchpoint hit is staged without a game to write the memory.

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use indexmap::IndexMap;

use crate::py::{self, time};

pub const ACK: &[u8] = b"+";
pub const INTERRUPT: u8 = 0x03;
pub const DEFAULT_REGISTERS: [i64; 16] = [
    0x08608810, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x0FFFFF00, 0x0012C000, 0x00123456,
];

fn le_hex(value: i64) -> String {
    u32::try_from(value)
        .expect("a 32-bit register")
        .to_le_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn frame(payload: &str) -> Vec<u8> {
    let checksum = payload.bytes().map(u32::from).sum::<u32>() & 0xFF;
    format!("${payload}#{checksum:02x}").into_bytes()
}

struct State {
    memory: IndexMap<i64, Vec<u8>>,
    registers: Vec<i64>,
    cpsr: i64,
    commands: Vec<String>,
    watchpoints: HashSet<(i64, i64, i64)>,
    running: bool,
    detached: bool,
    process_selected: bool,
    outbox: Vec<String>,
}

/// Serves `memory`, a mapping of base address to blob, and answers from `registers`.
///
/// A range that no single blob covers gets an empty reply, which is how the real stub reports an
/// address that is not mapped.
///
/// Python starts the server's thread on `__enter__`; here it starts at construction, and dropping
/// the stub is `__exit__`.
pub struct FakeGdbStub {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    port: u16,
}

impl FakeGdbStub {
    pub fn new(memory: IndexMap<i64, Vec<u8>>) -> Self {
        FakeGdbStub::with(memory, DEFAULT_REGISTERS.to_vec(), 0x60000010)
    }

    pub fn with(memory: IndexMap<i64, Vec<u8>>, registers: Vec<i64>, cpsr: i64) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let state = Arc::new(Mutex::new(State {
            memory,
            registers,
            cpsr,
            commands: Vec::new(),
            watchpoints: HashSet::new(),
            running: false,
            detached: false,
            process_selected: false,
            outbox: Vec::new(),
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let (state, stop) = (state.clone(), stop.clone());
            std::thread::spawn(move || serve(listener, &state, &stop))
        };
        FakeGdbStub {
            state,
            stop,
            thread: Some(thread),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn commands(&self) -> Vec<String> {
        self.state.lock().unwrap().commands.clone()
    }

    pub fn watchpoints(&self) -> HashSet<(i64, i64, i64)> {
        self.state.lock().unwrap().watchpoints.clone()
    }

    pub fn running(&self) -> bool {
        self.state.lock().unwrap().running
    }

    pub fn detached(&self) -> bool {
        self.state.lock().unwrap().detached
    }

    /// Block until the stub has handled `command`.
    ///
    /// A client's `resume()` returns as soon as `c` is written, so asking the stub what it thinks
    /// the moment after is a race — one this suite lost whenever the machine was busy.
    pub fn wait_for(&self, command: &str) {
        self.wait_for_within(command, 2.0);
    }

    pub fn wait_for_within(&self, command: &str, timeout: f64) {
        let deadline = time::monotonic() + timeout;
        while time::monotonic() < deadline {
            if self.commands().iter().any(|seen| seen == command) {
                return;
            }
            time::sleep(0.005);
        }
        let seen: Vec<String> = self.commands().iter().map(|seen| py::repr(seen)).collect();
        panic!(
            "the stub never handled {}; it saw [{}]",
            py::repr(command),
            seen.join(", ")
        );
    }

    /// Stage a stop reply, as a watchpoint hit would produce.
    pub fn fire(&self, signal: i64, pc: i64, thread: i64) {
        fire_in(&mut self.state.lock().unwrap(), signal, pc, thread);
    }

    /// `fire(pc=...)`, with the signal and thread it defaults to.
    pub fn fire_at(&self, pc: i64) {
        self.fire(5, pc, 0x2F);
    }
}

impl Drop for FakeGdbStub {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn fire_in(state: &mut State, signal: i64, pc: i64, thread: i64) {
    let reply = format!(
        "T{signal:02x}0f:{};0d:{};0e:{};thread:{thread:x};",
        le_hex(pc),
        le_hex(state.registers[13]),
        le_hex(state.registers[14]),
    );
    state.outbox.push(reply);
    state.running = false;
}

fn bytes_at(state: &State, address: i64, size: i64) -> Option<Vec<u8>> {
    for (&base, blob) in &state.memory {
        if base <= address && address + size <= base + blob.len() as i64 {
            let start = (address - base) as usize;
            return Some(blob[start..start + size as usize].to_vec());
        }
    }
    None
}

/// The stub's answer, or None when the command is acked and answered later.
fn reply_to(state: &mut State, command: &str) -> Option<String> {
    let kind = command.chars().next().expect("a command");
    match kind {
        '?' => {
            state.process_selected = true;
            state.running = false;
            Some(format!(
                "T000f:{};0d:{};0e:{};thread:2f;",
                le_hex(state.registers[15]),
                le_hex(state.registers[13]),
                le_hex(state.registers[14]),
            ))
        }
        'g' => Some(
            state
                .registers
                .iter()
                .chain([&state.cpsr])
                .map(|&value| le_hex(value))
                .collect(),
        ),
        'm' => {
            let (address, size) = command[1..].split_once(',').unwrap_or((&command[1..], ""));
            let data = bytes_at(
                state,
                i64::from_str_radix(address, 16).expect("hex"),
                i64::from_str_radix(size, 16).expect("hex"),
            );
            Some(data.map_or(String::new(), |data| {
                data.iter().map(|byte| format!("{byte:02x}")).collect()
            }))
        }
        'Z' | 'z' => {
            if !state.process_selected {
                return Some("E01".to_string());
            }
            let (wanted, rest) = command[1..].split_once(',').unwrap_or((&command[1..], ""));
            let (address, size) = rest.split_once(',').unwrap_or((rest, ""));
            let entry = (
                wanted.parse().expect("a kind"),
                i64::from_str_radix(address, 16).expect("hex"),
                i64::from_str_radix(size, 16).expect("hex"),
            );
            if kind == 'Z' {
                state.watchpoints.insert(entry);
            } else {
                state.watchpoints.remove(&entry);
            }
            Some("OK".to_string())
        }
        'c' => {
            state.running = true;
            None
        }
        's' => Some("E5F".to_string()),
        'D' => {
            state.detached = true;
            state.running = true;
            Some("OK".to_string())
        }
        _ => Some(String::new()),
    }
}

fn serve(listener: TcpListener, state: &Mutex<State>, stop: &AtomicBool) {
    // Python waits once, up to two seconds, for the connection. A Rust thread cannot be joined
    // with a timeout, so the wait also ends when the stub is dropped.
    listener.set_nonblocking(true).expect("nonblocking");
    let waiting = time::monotonic() + 2.0;
    let conn: TcpStream = loop {
        match listener.accept() {
            Ok((conn, _)) => break conn,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if stop.load(Ordering::SeqCst) || time::monotonic() >= waiting {
                    return;
                }
                time::sleep(0.005);
            }
            Err(_) => return,
        }
    };
    conn.set_nonblocking(false).expect("blocking");
    conn.set_read_timeout(Some(Duration::from_millis(20)))
        .expect("timeout");
    let mut conn = conn;
    let mut buffer: Vec<u8> = Vec::new();
    while !stop.load(Ordering::SeqCst) {
        let pending = std::mem::take(&mut state.lock().unwrap().outbox);
        for stop_reply in pending {
            if conn.write_all(&frame(&stop_reply)).is_err() {
                return;
            }
        }

        let mut chunk = [0u8; 4096];
        match conn.read(&mut chunk) {
            Ok(0) => return,
            Ok(got) => buffer.extend_from_slice(&chunk[..got]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                continue;
            }
            Err(_) => return,
        }

        while !buffer.is_empty() {
            if buffer[0] == INTERRUPT {
                buffer.remove(0);
                let mut state = state.lock().unwrap();
                let pc = state.registers[15];
                fire_in(&mut state, 5, pc, 0x2F);
                continue;
            }
            if buffer[0] == b'+' || buffer[0] == b'-' {
                buffer.remove(0);
                continue;
            }
            let end = buffer.iter().position(|&byte| byte == b'#');
            let Some(end) = end.filter(|&end| buffer[0] == b'$' && buffer.len() >= end + 3) else {
                break;
            };
            let command = String::from_utf8(buffer[1..end].to_vec()).expect("ascii");
            buffer.drain(..end + 3);
            if conn.write_all(ACK).is_err() {
                return;
            }
            let reply = {
                let mut state = state.lock().unwrap();
                let reply = reply_to(&mut state, &command);
                // Recorded after handling, not before, so `wait_for` means "this has taken
                // effect" rather than "this has been read off the socket".
                state.commands.push(command);
                reply
            };
            if let Some(reply) = reply
                && conn.write_all(&frame(&reply)).is_err()
            {
                return;
            }
        }
    }
}
