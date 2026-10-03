//! The code modules the game has loaded: the 3DS's CROs, its DLLs, found by their headers.
//!
//! A game loads a module for each app (the field's, a battle's, the Bag's) into memory it keeps for
//! them, one after another from the bottom of it, and the emulator maps each where the game's own
//! request says (Azahar's `RO::LoadCRO` never picks an address). So where a module lands depends on
//! what is loaded beneath it: the field's and a battle's were always at the bottom, alone, and the name
//! entry's keyboard was seen at three addresses (spike 140). Each carries a header 0x80 into it, and
//! the loaded ones are linked in the order they were loaded:
//!
//! ```text
//! +0x80 'CRO0', or 'FIXD' once fixed where it is    +0x88 -> the next loaded, or 0
//! +0x8c -> the previous; the first's is the last    +0x90 u32 its size
//! +0xc0 -> its name    +0xc4 u32 the name's size
//! ```
//!
//! The bottom of their memory is found once, as the lowest page holding a header, and every later
//! look starts there: a few reads an app start where reading the code past the executable was a
//! thousand.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary;
use crate::core::rpc::Rpc;
use crate::py::{self, Result};

pub const HEADER: i64 = 0x80;
pub const MAGICS: [&[u8]; 2] = [b"CRO0", b"FIXD"];
/// The header's words read, from +0x80 to the name's size at +0xc4.
pub const FIELDS: i64 = 0x48;
/// Offsets into those words.
pub const NEXT: i64 = 0x08;
/// Offsets into those words.
pub const PREVIOUS: i64 = 0x0C;
/// Offsets into those words.
pub const SIZE: i64 = 0x10;
/// Offsets into those words.
pub const NAME: i64 = 0x40;
/// Offsets into those words.
pub const NAME_SIZE: i64 = 0x44;
/// A module is mapped a page at a time, so its header is 0x80 into a page.
pub const PAGE: i64 = 0x1000;
/// Pages whose headers are asked for together while looking up from the bottom.
pub const BATCH: i64 = 64;
pub const MAX_NAME: i64 = 64;
/// More loaded at once than this is a list gone wrong, not a game's.
pub const MAX_MODULES: i64 = 32;

/// What the modules are read through: anything with `read_each`, as an `Rpc` or a cache has.
pub trait Reader {
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>>;
}

impl<T: Rpc + ?Sized> Reader for T {
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        Rpc::read_each(self, reads)
    }
}

/// A holder keeps its emulator as an `Rc<dyn Rpc>`, which is not itself an `Rpc`.
impl Reader for Rc<dyn Rpc> {
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        Rpc::read_each(&**self, reads)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub base: i64,
    pub size: i64,
    /// 'CRO0' while it loads, and 'FIXD' once fixed where it is, if the game asks for that.
    pub magic: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Head {
    magic: Vec<u8>,
    next: i64,
    previous: i64,
    size: i64,
    name: i64,
    name_size: i64,
}

/// The modules loaded now, in the order they were loaded.
pub struct Modules {
    reader: Rc<dyn Reader>,
    span: (i64, i64),
    /// The lowest page a header has been seen on: where a module loaded alone goes.
    bottom: Option<i64>,
}

impl Modules {
    /// `span` is where code can be: the bottom of the modules' memory is looked for in it.
    pub fn new(reader: Rc<dyn Reader>, span: (i64, i64)) -> Self {
        Modules {
            reader,
            span,
            bottom: None,
        }
    }

    pub fn loaded(&mut self) -> Result<Vec<Module>> {
        let Some((mut at, mut head)) = self.first()? else {
            return Ok(Vec::new());
        };
        // A header found from the bottom up is the lowest loaded, which need not be the first
        // loaded: the first's previous is the last, whose next is 0.
        for _ in 0..MAX_MODULES {
            if head.previous == at {
                break;
            }
            let earlier = self.head(head.previous)?;
            let Some(earlier) = earlier.filter(|earlier| earlier.next != 0) else {
                break;
            };
            (at, head) = (head.previous, earlier);
        }
        let mut out: Vec<Module> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();
        let mut current = at;
        while current != 0 && !seen.contains(&current) && (out.len() as i64) < MAX_MODULES {
            seen.insert(current);
            let Some(head) = self.head(current)? else {
                break;
            };
            out.push(Module {
                name: self.name(current, &head)?,
                base: current,
                size: head.size,
                magic: head.magic.clone(),
            });
            current = head.next;
        }
        Ok(out)
    }

    /// The lowest page holding a header, from the bottom known or from the start of the span:
    /// the module at the bottom may have gone while one above it stayed.
    fn first(&mut self) -> Result<Option<(i64, Head)>> {
        let (low, high) = self.span;
        let start = self.bottom.unwrap_or(low);
        let mut at: Option<i64> = None;
        // The bottom alone first, as nearly every look finds it there, then upward a batch at a
        // time, stopping at the first header.
        let batches = std::iter::once(pages(start, (start + PAGE).min(high))).chain(
            (start + PAGE..high)
                .step_by((BATCH * PAGE) as usize)
                .map(|page| pages(page, (page + BATCH * PAGE).min(high))),
        );
        for batch in batches {
            let reads: Vec<(i64, i64)> = batch.iter().map(|&page| (page + HEADER, 4)).collect();
            let magics = self.reader.read_each(&reads)?;
            at = batch.iter().copied().find(|page| {
                magics
                    .get(&(page + HEADER))
                    .is_some_and(|magic| MAGICS.contains(&magic.as_slice()))
            });
            if at.is_some() {
                break;
            }
        }
        let Some(at) = at else {
            return Ok(None);
        };
        let Some(head) = self.head(at)? else {
            return Ok(None);
        };
        if self.bottom.is_none_or(|bottom| at < bottom) {
            self.bottom = Some(at);
        }
        Ok(Some((at, head)))
    }

    fn head(&self, at: i64) -> Result<Option<Head>> {
        let replies = self.reader.read_each(&[(at + HEADER, FIELDS)])?;
        let data = replies
            .get(&(at + HEADER))
            .map_or(&[][..], |data| data.as_slice());
        if (data.len() as i64) < FIELDS || !MAGICS.contains(&py::slice(data, 0, 4)) {
            return Ok(None);
        }
        let (next, previous, size) = (
            binary::u32(data, NEXT)?,
            binary::u32(data, NEXT + 4)?,
            binary::u32(data, NEXT + 8)?,
        );
        let (name, name_size) = (binary::u32(data, NAME)?, binary::u32(data, NAME + 4)?);
        Ok(Some(Head {
            magic: data[..4].to_vec(),
            next,
            previous,
            size,
            name,
            name_size,
        }))
    }

    fn name(&self, at: i64, head: &Head) -> Result<String> {
        let place = if head.name >= at {
            head.name
        } else {
            at + head.name
        };
        let length = head.name_size.min(MAX_NAME);
        if length <= 0 {
            return Ok(String::new());
        }
        let replies = self.reader.read_each(&[(place, length)])?;
        let raw = replies.get(&place).map_or(&[][..], |raw| raw.as_slice());
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        Ok(py::decode_ascii(&raw[..end]))
    }
}

/// `range(low, high, PAGE)`.
fn pages(low: i64, high: i64) -> Vec<i64> {
    (low..high).step_by(PAGE as usize).collect()
}
