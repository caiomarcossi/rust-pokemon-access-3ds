//! The DexNav's hidden Pokémon: the one whose tail shows above the tall grass, rustling, to be
//! sneaked up on. Spike 80 took the game's "kakuremino" classes, a hiding cloak, for its mechanism;
//! they are the move models' event types for trainers (`field::mmodel::EvTypeTrKakuremino` beside
//! `EvTypeTrMirror` and `EvTypeTrDiving`), alike with a tail up or not, and none points at the
//! record (spike 135).
//!
//! It is not a move model: the pool holds nothing for it while it is up (spike 80). The game keeps
//! one record for it, without a class of its own, which reads zero while nothing is hiding:
//!
//! ```text
//! +0x00 ptr  the record's own `gfl::heap::CTRExpHeap`, whatever the record holds
//! +0x04 u16  species, the national number (0x105 Poochyena, 0x11b Surskit); 0 while empty
//! +0x06 u16  not understood: 0 in the first finds, 0x97 beside a Wurmple and 0x5e beside a
//!            Surskit later on Route 102, where reading the species as a word hid them both
//! +0x08 u32  level
//! +0x34 u32  a counter stepping 0 to 19 as the player walks through grass, reset by a find
//! +0x48 u32  1 once a record has been set; a battle does not clear it
//! +0x8c f32  world x, +0x90 y (2.0 on grass), +0x94 z, the cell's centre at `cell * 18 + 9`
//! +0x98 u32  1 once a record has been set, as +0x48
//! +0xa0 ptr  the party's records while the tail is up, cleared by its battle
//! +0xa4 ptr  a `pml::pokepara::CoreParam`, the record's Pokémon, held even while empty
//! +0xae u8   a small number while the tail is up (1, 4, 5 seen), 0xff otherwise: while the
//!            record is empty, after the battle, and while a zone change has left it stale
//! ```
//!
//! The record is found through the one `field::PlayerWork`, which holds its address at +0x6c, told
//! by the heap at its +0x00 (spike 135: all three captures of movie3, before the lesson, with its
//! Poochyena's tail up and after its battle). It was found as what holds a CoreParam at +0xa4 until
//! then, searching every page for every CoreParam at every whole sweep. Its address was the same in
//! seven captures over three movies, and it is the same record for the lesson's Poochyena on Route
//! 101 and a Surskit found on Route 102.
//!
//! A battle leaves the species, level and place in the record (the user, 2026-09-22: the entries
//! did not clean up after a battle), and so does walking to another zone; the byte at +0xae is what
//! says the tail is up, in the nine captures over four movies it was read in.
//!
//! The species is said in the log and not aloud: the game shows a tail, and the DexNav names the
//! Pokémon only once the species is registered, so a name would be an Extra (ADR-0017), and the
//! names are in the ROM's text in any case (names later, the user, 2026-09-21).

use std::cell::Cell as Slot;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

use crate::core::binary::{f32, u16, u32};
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::gen6::field::{HiddenReader, Hiding};
use crate::games::gen6::gflheap;
use crate::py::{self, Result};

/// What the record holds at +0xa4, which a sweep no longer looks for.
pub const CORE_PARAM_CLASS: &str = "N3pml8pokepara9CoreParamE";
pub const HEAP_CLASS: &str = gflheap::CLASS;
pub const PLAYER_WORK_CLASS: &str = "N5field10PlayerWorkE";
/// Where the player's work holds the record's address.
pub const WORK_RECORD: i64 = 0x6C;
/// What a sweep looks for, to find the record by.
pub const CLASSES: [&str; 2] = [PLAYER_WORK_CLASS, HEAP_CLASS];

pub const HEAP: i64 = 0x00;
pub const SPECIES: i64 = 0x04;
pub const LEVEL: i64 = 0x08;
pub const SET: i64 = 0x48;
pub const WORLD_X: i64 = 0x8C;
pub const WORLD_Z: i64 = 0x94;
pub const CORE: i64 = 0xA4;
/// Which is a guess at its meaning; what is measured is that it reads `NONE` while no tail is
/// up and a small number while one is.
pub const SLOT: i64 = 0xAE;
pub const NONE: i64 = 0xFF;
/// Enough of the record to hold everything above.
pub const SIZE: i64 = 0xB0;

/// World units per cell, the centre at `cell * 18 + 9`.
pub const CELL: i64 = 18;
/// The national Pokédex's last number in this generation.
pub const MAX_SPECIES: i64 = 721;
pub const MAX_LEVEL: i64 = 100;

/// One hiding in the grass now.
#[derive(Debug, Clone, PartialEq)]
pub struct HiddenPokemon {
    pub species: i64,
    pub level: i64,
    pub cell: (i64, i64),
}

/// The record's address, or None when a sweep found nothing to reach it by: what the player's
/// work points at, holding one of the heaps the sweep found first.
pub fn find(found: &Found, memory: &dyn Memory) -> Result<Option<i64>> {
    let heaps: HashSet<i64> = found.of(&[HEAP_CLASS]).into_iter().collect();
    for work in found.of(&[PLAYER_WORK_CLASS]) {
        let record = u32(&memory.read(work + WORK_RECORD, 4)?, 0)?;
        if record != 0 && heaps.contains(&u32(&memory.read(record + HEAP, 4)?, 0)?) {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// The record's words as they are, before any check, for the log: what the game held when a
/// rustle went unlisted (the user, 2026-09-22).
#[derive(Debug, Clone, PartialEq)]
pub struct Words {
    pub species: i64,
    pub level: i64,
    pub slot: i64,
    pub cell: (i64, i64),
}

impl fmt::Display for Words {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "species {} level {} slot {:#x} at ({}, {})",
            self.species, self.level, self.slot, self.cell.0, self.cell.1
        )
    }
}

pub fn words(memory: &dyn Memory, record: i64) -> Result<Words> {
    let head = memory.read(record, SIZE)?;
    Ok(Words {
        species: u16(&head, SPECIES)?,
        level: u32(&head, LEVEL)?,
        slot: head[SLOT as usize] as i64,
        cell: (cell(f32(&head, WORLD_X)?)?, cell(f32(&head, WORLD_Z)?)?),
    })
}

/// What the record holds, or None while no tail is up, or the bytes are not a record's: a
/// species out of the Pokédex or a level over 100.
pub fn read(memory: &dyn Memory, record: i64) -> Result<Option<HiddenPokemon>> {
    let held = words(memory, record)?;
    if !(1 <= held.species && held.species <= MAX_SPECIES)
        || !(1 <= held.level && held.level <= MAX_LEVEL)
    {
        return Ok(None);
    }
    if held.slot == NONE {
        return Ok(None);
    }
    Ok(Some(HiddenPokemon {
        species: held.species,
        level: held.level,
        cell: held.cell,
    }))
}

/// The record, for the gen 6 field to list (`gen6.field.HiddenReader`): found by a sweep and
/// kept, since it lives all session.
pub struct DexNav {
    memory: Rc<dyn Memory>,
    record: Slot<Option<i64>>,
}

impl DexNav {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        DexNav {
            memory,
            record: Slot::new(None),
        }
    }

    pub fn classes(&self) -> Vec<&'static str> {
        CLASSES.to_vec()
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        self.record
            .set(find(found, &*self.memory)?.or(self.record.get()));
        Ok(())
    }

    pub fn read(&self) -> Result<Option<Hiding>> {
        let Some(record) = self.record.get() else {
            return Ok(None);
        };
        let held = read(&*self.memory, record)?;
        Ok(held.map(|held| Hiding {
            record,
            species: held.species,
            cell: held.cell,
        }))
    }

    pub fn words(&self) -> Result<Option<(String, Cell)>> {
        let Some(record) = self.record.get() else {
            return Ok(None);
        };
        let held = words(&*self.memory, record)?;
        Ok(Some((held.to_string(), held.cell)))
    }
}

impl HiddenReader for DexNav {
    fn classes(&self) -> Vec<&'static str> {
        DexNav::classes(self)
    }

    fn track(&self, found: &Found) -> Result<()> {
        DexNav::track(self, found)
    }

    fn read(&self) -> Result<Option<Hiding>> {
        DexNav::read(self)
    }

    fn words(&self) -> Result<Option<(String, Cell)>> {
        DexNav::words(self)
    }
}

fn cell(units: f64) -> Result<i64> {
    py::round((units - CELL as f64 / 2.0) / CELL as f64)
}
