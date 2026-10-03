//! Route 113's ash piles: the grey mounds the Soot Sack collects ash from (spike 155).
//!
//! No zone's data has a pile, and no move model stands in one: the pile below the Glass Workshop's
//! sign at (252, 92) is an open cell like the grass round it. Route 113 loads a module of its own,
//! `DllUSR113.cro`, the only `DllUSR` module in the ROM, whose `field::FieldUniqueSequenceOchiba`
//! writes the route's seven piles into a list as it starts, from constants in its code:
//!
//! ```text
//! +0x14 -> the pile's states, a byte a pile, in the save's `savedata::GimmickWork`
//! +0x2c u32  how many piles, 7
//! +0x30 -> the piles, 12 bytes each: f32 x, y, z in world units, a cell's centre at
//!            `cell * 18 + 9`
//! +0x50 u16  the zone it was started in, 0x28
//! ```
//!
//! A pile's state is 0 until the player first comes near it, 2 once it is set out, 3 the moment the
//! player walks into it, and 1 once its ash is collected. The game puts every pile but the
//! collected ones back to 0 as the sequence ends, and draws none at 1 or 3: walked into on the
//! replay, the pile at (252, 92) went from 2 to 3 and was gone from the screen
//! (omega_ruby_ashes.ctm).

use std::cell::Cell as Slot;
use std::rc::Rc;

use crate::core::binary::{self, u16, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::py::{self, Error, Result};

pub const CLASS: &str = "N5field25FieldUniqueSequenceOchibaE";
pub const STATES: i64 = 0x14;
pub const COUNT: i64 = 0x2C;
pub const PILES: i64 = 0x30;
pub const ZONE: i64 = 0x50;
pub const HEAD: i64 = 0x54;

/// `struct.Struct("<3f")`: a pile's x, y and z.
pub struct Pile;

pub const PILE: Pile = Pile;

impl Pile {
    /// `PILE.size`.
    pub fn size(&self) -> i64 {
        12
    }

    /// `PILE.iter_unpack(blob)`: a buffer that is not a whole number of piles is `struct.error`.
    #[track_caller]
    pub fn iter_unpack(&self, blob: &[u8]) -> Result<Vec<[f64; 3]>> {
        if blob.len() as i64 % self.size() != 0 {
            return Err(Error::unpack(format!(
                "iterative unpacking requires a buffer of a multiple of {} bytes",
                self.size()
            )));
        }
        let mut out = Vec::new();
        for at in (0..blob.len() as i64).step_by(self.size() as usize) {
            out.push([
                binary::f32(blob, at)?,
                binary::f32(blob, at + 4)?,
                binary::f32(blob, at + 8)?,
            ]);
        }
        Ok(out)
    }
}

/// The states the game copies at a time: no more piles than this.
pub const MOST: i64 = 0x20;
pub const CELL: i64 = 18;
/// Collected, and walked into this moment.
pub const GONE: [i64; 2] = [1, 3];

/// The ash piles of the zone the sequence was started in, found by a sweep as its module
/// loads.
pub struct AshPiles {
    pub classes: Vec<&'static str>,
    memory: Rc<dyn Memory>,
    sequence: Slot<Option<i64>>,
    vtable: Slot<i64>,
}

impl AshPiles {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        AshPiles {
            classes: vec![CLASS],
            memory,
            sequence: Slot::new(None),
            vtable: Slot::new(0),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        // The module's image is copied into the heap as it loads, vtables and all, and a sweep
        // finds the class's vtable in its code too: a find is taken only in the sequence's shape.
        let mut sequences: Vec<i64> = Vec::new();
        for at in found.of(&[CLASS]) {
            if self.head(at)?.is_some() {
                sequences.push(at);
            }
        }
        if let Some(&last) = sequences.last() {
            self.sequence.set(Some(last));
            self.vtable.set(u32(&self.memory.read(last, 4)?, 0)?);
        }
        Ok(())
    }

    /// (index, cell) of each pile in `zone` whose ash is still there.
    pub fn piles(&self, zone: i64) -> Result<Vec<(i64, Cell)>> {
        let head = match self.sequence.get() {
            None => None,
            Some(sequence) => self.head(sequence)?,
        };
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        if u32(&head, 0)? != self.vtable.get() || u16(&head, ZONE)? != zone {
            return Ok(Vec::new());
        }
        let count = u32(&head, COUNT)?;
        let states = self.memory.read(u32(&head, STATES)?, count)?;
        let places = self.memory.read(u32(&head, PILES)?, count * PILE.size())?;
        let mut out = Vec::new();
        for (index, [x, _y, z]) in PILE.iter_unpack(&places)?.into_iter().enumerate() {
            let state = match states.get(index) {
                Some(&state) => state as i64,
                None => return Err(Error::index("index out of range")),
            };
            if !GONE.contains(&state) {
                out.push((
                    index as i64,
                    (
                        py::floordiv(py::int(x)?, CELL),
                        py::floordiv(py::int(z)?, CELL),
                    ),
                ));
            }
        }
        Ok(out)
    }

    fn head(&self, at: i64) -> Result<Option<Vec<u8>>> {
        let head = self.memory.read(at, HEAD)?;
        let count = u32(&head, COUNT)?;
        if !(0 < count && count <= MOST) {
            return Ok(None);
        }
        if !within(HEAP_SPAN, u32(&head, STATES)?) || !within(HEAP_SPAN, u32(&head, PILES)?) {
            return Ok(None);
        }
        Ok(Some(head))
    }
}
