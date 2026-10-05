//! The game's button managers, which say whose buttons take taps now (spike 189).
//!
//! An app's buttons are answered through `app::tool::ButtonManager`s, each holding buttons in a
//! layout or two. A manager's update returns at once while its byte +0x24 is set, and the game sets
//! it to lock a screen's buttons under a menu: the party's list under "Do what with Blaze?"
//! (movie22). A layout whose every manager is locked takes no tap, however bright it is drawn.
//!
//! An app's bottom bar is a **touch bar** instead, a view of a class of the app's module's own
//! (`app::pokelist::TouchBar`, `app::bag::TouchBar`) built on the executable's
//! `app::ui::UITouchBarView`: it updates the manager at its +0x10 only while its byte +0x2d is set,
//! which the party clears under its menu too, the manager's own lock left open. The field's PokéNav
//! has no manager at all, and the PC Box keeps its grid's open under "Continue operations?": a list
//! window is answered first (`touch.answered_first`).
//!
//! ```text
//! manager    +0x10  its buttons, a pointer each; +0x14 how many
//!            +0x24  locked
//! button     +0x08  a handle, whose +0x04 is its layout's work
//! touch bar  +0x10  its manager
//!            +0x14  its input listener, inside the touch bar itself
//!            +0x2d  it takes input
//! ```

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::py::{self, PySet, Result};

pub const MANAGER_CLASS: &str = "N3app4tool13ButtonManagerE";
/// The class every app's touch bar is built on, found with every class built on it.
pub const BAR_CLASS: &str = "N3app2ui14UITouchBarViewE";
pub const BUTTONS: i64 = 0x10;
pub const COUNT: i64 = 0x14;
pub const LOCKED: i64 = 0x24;
pub const HANDLE: i64 = 0x08;
pub const WORK: i64 = 0x04;
pub const BAR_MANAGER: i64 = 0x10;
pub const LISTENER: i64 = 0x14;
pub const INPUT: i64 = 0x2D;
/// How far into a touch bar its listener is: a word of a module's file kept in the heap, its
/// vtable's address among its own, has no listener inside it.
pub const BAR_SIZE: i64 = 0x100;
pub const MAX_BUTTONS: i64 = 64;

/// The button managers and touch bars a sweep found, and the layouts they lock.
pub struct Buttons {
    memory: Rc<dyn Memory>,
    /// Manager -> the vtable it was found with.
    managers: IndexMap<i64, i64>,
    /// Touch bar -> the vtable it was found with.
    bars: IndexMap<i64, i64>,
    /// Manager -> its list of buttons as last read, and the works of the layouts they are in.
    works: IndexMap<i64, (Vec<u8>, PySet)>,
}

impl Buttons {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Buttons {
            memory,
            managers: IndexMap::new(),
            bars: IndexMap::new(),
            works: IndexMap::new(),
        }
    }

    /// The managers and touch bars a sweep found: all of them, or with `adding` the new
    /// ones.
    pub fn track(
        &mut self,
        managers: impl IntoIterator<Item = i64>,
        bars: impl IntoIterator<Item = i64>,
        adding: bool,
    ) -> Result<()> {
        let read = |at: i64| -> Result<i64> { u32(&self.memory.read(at, 4)?, 0) };
        let mut found: IndexMap<i64, i64> = IndexMap::new();
        for at in managers {
            found.insert(at, read(at)?);
        }
        let mut merged = if adding {
            self.managers.clone()
        } else {
            IndexMap::new()
        };
        merged.extend(found);
        self.managers = merged;
        let read = |at: i64| -> Result<i64> { u32(&self.memory.read(at, 4)?, 0) };
        let mut found: IndexMap<i64, i64> = IndexMap::new();
        for at in bars {
            found.insert(at, read(at)?);
        }
        let mut merged = if adding {
            self.bars.clone()
        } else {
            IndexMap::new()
        };
        merged.extend(found);
        self.bars = merged;
        let managers = &self.managers;
        self.works.retain(|at, _| managers.contains_key(at));
        Ok(())
    }

    /// The works of the layouts whose buttons take no tap now: every manager with buttons in
    /// them locked, or held by a touch bar taking no input. A layout no manager has buttons in is
    /// not among them.
    pub fn idle(&mut self) -> Result<HashSet<i64>> {
        let memory = self.memory.clone();
        let mut unfed: HashSet<i64> = HashSet::new();
        for (&bar, &vtable) in &self.bars {
            let head = memory.read(bar, INPUT + 1)?;
            if u32(&head, 0)? == vtable
                && 0 < u32(&head, LISTENER)? - bar
                && u32(&head, LISTENER)? - bar < BAR_SIZE
                && head[INPUT as usize] == 0
            {
                unfed.insert(u32(&head, BAR_MANAGER)?);
            }
        }
        let mut taking: IndexMap<i64, bool> = IndexMap::new();
        let managers: Vec<(i64, i64)> = self.managers.iter().map(|(&m, &v)| (m, v)).collect();
        for (manager, vtable) in managers {
            let head = memory.read(manager, LOCKED + 1)?;
            if u32(&head, 0)? != vtable {
                continue;
            }
            let open = head[LOCKED as usize] == 0 && !unfed.contains(&manager);
            for work in self.works_of(manager, &head)?.iter() {
                let before = taking.get(&work).copied().unwrap_or(false);
                taking.insert(work, before || open);
            }
        }
        Ok(taking
            .into_iter()
            .filter(|&(_, open)| !open)
            .map(|(work, _)| work)
            .collect())
    }

    /// The works of the layouts a manager's buttons are in, read again when its list of
    /// buttons is another than last read.
    fn works_of(&mut self, manager: i64, head: &[u8]) -> Result<PySet> {
        let listed = py::slice(head, BUTTONS, COUNT + 4).to_vec();
        if let Some(known) = self.works.get(&manager)
            && known.0 == listed
        {
            return Ok(known.1.clone());
        }
        let (table, count) = (u32(&listed, 0)?, u32(&listed, 4)?);
        let mut works = PySet::new();
        if table != 0 && 0 < count && count <= MAX_BUTTONS {
            let read = |at: i64, size: i64| self.memory.read(at, size);
            let entries = read(table, 4 * count)?;
            // Only where the heap is: a battle keeps much of itself in memory the GPU may hold,
            // where a read crashes players' Azahar (azahar-emu/azahar#2619).
            for i in 0..count {
                let button = u32(&entries, 4 * i)?;
                let handle = if within(HEAP_SPAN, button) {
                    u32(&read(button + HANDLE, 4)?, 0)?
                } else {
                    0
                };
                if within(HEAP_SPAN, handle) {
                    works.add(u32(&read(handle + WORK, 4)?, 0)?);
                }
            }
        }
        self.works.insert(manager, (listed, works));
        Ok(self.works[&manager].1.clone())
    }
}
