//! A tape's answers, for checking the port against the Python runtime (`tools/tape/README.md`).
//!
//! A tape is what the emulator and the clock answered the Python runtime through one replay of a
//! movie, in the order it asked. [`TapeRpc`] and [`TapeClock`] give the Rust runtime the same
//! answers and check that it asks the same questions; the first it asks differently stops the
//! replay with where and what.
//!
//! Running out of tape, or asking something it does not hold, unwinds with a [`Stop`]: the Python
//! raised a `BaseException` there, which no reader's `except Exception` takes for a failure, and
//! the runtime passes a `Stop` on in the same way (`is_stop`).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::rpc::{Process, Rpc};
use crate::py::{Error, Kind, Result, time::Clock};

const MAGIC: &[u8] = b"PA3DSTAPE1\n";

const CLOCK: u8 = 1;
const SLEEP: u8 = 2;
const READ: u8 = 3;
const READ_EACH: u8 = 4;
const UNSURE: u8 = 5;
const REPLY_SECONDS: u8 = 6;
const CONFIRM: u8 = 7;
const ATTACH: u8 = 8;

fn name(kind: u8) -> String {
    match kind {
        CLOCK => "clock".into(),
        SLEEP => "sleep".into(),
        READ => "read".into(),
        READ_EACH => "read_each".into(),
        UNSURE => "unsure".into(),
        REPLY_SECONDS => "reply_seconds".into(),
        CONFIRM => "confirm".into(),
        ATTACH => "attach".into(),
        other => format!("record kind {other}"),
    }
}

/// Why a replay stopped: the payload of the unwind that stops it.
#[derive(Debug, Clone, PartialEq)]
pub enum Stop {
    /// Every answer on the tape was asked for: the replay ran as far as the recording did.
    End,
    /// The runtime asked something other than what the tape holds next.
    Diverged(String),
}

/// Whether a caught panic is a tape stopping the replay, which must unwind on past whatever
/// caught it.
pub fn is_stop(payload: &(dyn Any + Send)) -> bool {
    payload.is::<Stop>()
}

/// One tape, read from the front.
pub struct Tape {
    data: Vec<u8>,
    at: Cell<usize>,
    count: Cell<usize>,
    stopped: RefCell<Option<Stop>>,
}

impl Tape {
    pub fn open(path: &Path) -> Result<Rc<Tape>> {
        let mut data = Vec::new();
        flate2::read::GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut data)?;
        if !data.starts_with(MAGIC) {
            return Err(Error::value(format!("{} is not a tape", path.display())));
        }
        Ok(Rc::new(Tape {
            at: Cell::new(MAGIC.len()),
            data,
            count: Cell::new(0),
            stopped: RefCell::new(None),
        }))
    }

    /// Records the replay has asked for.
    pub fn count(&self) -> usize {
        self.count.get()
    }

    /// Why the replay stopped, once it has.
    pub fn stopped(&self) -> Option<Stop> {
        self.stopped.borrow().clone()
    }

    fn stop(&self, why: Stop) -> ! {
        *self.stopped.borrow_mut() = Some(why.clone());
        std::panic::panic_any(why)
    }

    fn diverge(&self, what: String) -> ! {
        self.stop(Stop::Diverged(format!("record {}: {what}", self.count.get())))
    }

    fn take(&self, n: usize) -> &[u8] {
        let at = self.at.get();
        if at + n > self.data.len() {
            self.stop(Stop::End);
        }
        self.at.set(at + n);
        &self.data[at..at + n]
    }

    fn u8(&self) -> u8 {
        self.take(1)[0]
    }

    fn u32(&self) -> i64 {
        u32::from_le_bytes(self.take(4).try_into().expect("4 bytes")) as i64
    }

    fn u64(&self) -> i64 {
        u64::from_le_bytes(self.take(8).try_into().expect("8 bytes")) as i64
    }

    fn f64(&self) -> f64 {
        f64::from_le_bytes(self.take(8).try_into().expect("8 bytes"))
    }

    fn blob(&self) -> Vec<u8> {
        let n = self.u32() as usize;
        self.take(n).to_vec()
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.blob()).into_owned()
    }

    fn expect(&self, kind: u8, what: &str) {
        if self.at.get() >= self.data.len() {
            self.stop(Stop::End);
        }
        let got = self.u8();
        self.count.set(self.count.get() + 1);
        if got != kind {
            self.diverge(format!("the runtime asked {what}, the tape has {}", name(got)));
        }
    }

    fn error(&self) -> Option<Error> {
        let kind = match self.u8() {
            0 => return None,
            1 => Kind::RpcError,
            2 => Kind::RpcTimeoutError,
            3 => Kind::RpcRefusedError,
            4 => Kind::RpcLostError,
            5 => Kind::ValueError,
            other => self.diverge(format!("an error of kind {other}")),
        };
        Some(Error::new(kind, self.text()))
    }
}

/// The emulator, as the tape remembers it answering.
pub struct TapeRpc {
    tape: Rc<Tape>,
}

impl TapeRpc {
    pub fn new(tape: Rc<Tape>) -> Self {
        TapeRpc { tape }
    }
}

impl Rpc for TapeRpc {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let tape = &self.tape;
        tape.expect(READ, &format!("read {address:#x} {size}"));
        let recorded = (tape.u32(), tape.u32());
        if recorded != (address, size) {
            tape.diverge(format!(
                "read {address:#x} {size}, the tape has read {:#x} {}",
                recorded.0, recorded.1
            ));
        }
        match tape.error() {
            Some(error) => Err(error),
            None => Ok(tape.blob()),
        }
    }

    /// The batch must hold the same reads as the recorded one, in any order: the Python sends
    /// some batches in the order of a `set`, which Rust does not reproduce, and the order of a
    /// batch changes nothing but the order its replies arrive in.
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let tape = &self.tape;
        tape.expect(READ_EACH, &format!("read_each of {}", reads.len()));
        let count = tape.u32() as usize;
        let mut recorded: Vec<(i64, i64)> = (0..count).map(|_| (tape.u32(), tape.u32())).collect();
        let mut asked = reads.to_vec();
        recorded.sort_unstable();
        asked.sort_unstable();
        if recorded != asked {
            let first = recorded.iter().zip(&asked).position(|(a, b)| a != b);
            tape.diverge(format!(
                "read_each of {}, the tape has {}; sorted, first differing at {}: {:x?} against \
                 the tape's {:x?}",
                asked.len(),
                recorded.len(),
                first.unwrap_or(recorded.len().min(asked.len())),
                first.map(|i| asked[i]),
                first.map(|i| recorded[i]),
            ));
        }
        if let Some(error) = tape.error() {
            return Err(error);
        }
        let replies = tape.u32() as usize;
        Ok((0..replies).map(|_| (tape.u32(), tape.blob())).collect())
    }

    fn reply_seconds(&self) -> f64 {
        self.tape.expect(REPLY_SECONDS, "reply_seconds");
        self.tape.f64()
    }

    fn unsure(&self) -> bool {
        self.tape.expect(UNSURE, "unsure");
        self.tape.u8() != 0
    }

    fn confirm(&self) -> Result<bool> {
        self.tape.expect(CONFIRM, "confirm");
        match self.tape.error() {
            Some(error) => Err(error),
            None => Ok(self.tape.u8() != 0),
        }
    }

    fn attach(&self) -> Result<Process> {
        let tape = &self.tape;
        tape.expect(ATTACH, "attach");
        if let Some(error) = tape.error() {
            return Err(error);
        }
        Ok(Process { pid: tape.u32(), title_id: tape.u64(), name: tape.text() })
    }
}

/// The clock, as the tape remembers it reading.
pub struct TapeClock {
    tape: Rc<Tape>,
}

impl TapeClock {
    pub fn new(tape: Rc<Tape>) -> Self {
        TapeClock { tape }
    }
}

impl Clock for TapeClock {
    fn monotonic(&mut self) -> f64 {
        self.tape.expect(CLOCK, "clock");
        self.tape.f64()
    }

    fn sleep(&mut self, seconds: f64) {
        self.tape.expect(SLEEP, &format!("sleep {}", crate::py::float_repr(seconds)));
        let recorded = self.tape.f64();
        if recorded != seconds {
            self.tape.diverge(format!(
                "sleep {}, the tape has {}",
                crate::py::float_repr(seconds),
                crate::py::float_repr(recorded)
            ));
        }
    }
}
