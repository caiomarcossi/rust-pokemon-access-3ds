//! The AreaNav's views, which say which of its layouts it draws (spikes 181 and 183).
//!
//! The AreaNav keeps the layouts of a screen it has put away exactly as they were: the berry list
//! after its View Details, the Pokémon Distribution, the trainers' details. Its layout work, its
//! panes' flags and alpha, Game Freak's layout helper (`app::util::G2DUtil`) and its button manager
//! all read as drawn (spikes 174, 179 and 180), and a tap on a berry row moved the map's cursor
//! (Orion, movie20).
//! The game stops walking the layout altogether (spike 181, a read watchpoint on its root): each
//! screen is a view of the AreaNav's module, `app::townmap::NutsView` for the berry list, whose
//! update and draw return at once while a byte of its own is 0, and which otherwise draws the
//! layouts of the `G2DUtil` at its +0x5C (spike 183, the byte 1 while the list is up and 0 once it
//! is gone). Which byte is each class's own: the offsets in `VIEWS` are where each class's draw
//! reads it. `MapView` and `GraphicWork` draw whenever the AreaNav does, and are not listed.
//!
//! ```text
//! view     +0x5C  -> app::util::G2DUtil
//! G2DUtil  +0x1C  -> its layouts, 8 bytes each: a handle, then flags; +0x20 how many
//! handle   +0x04  the layout's work, as a layout pool's table keeps it
//! ```

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::py::Result;

/// Each view class whose draw is skipped while a byte of it is 0, by where that byte is.
pub const VIEWS: [(&str, i64); 5] = [
    ("N3app7townmap8NutsViewE", 0xAC),
    ("N3app7townmap14TrainerEyeViewE", 0xAC),
    ("N3app7townmap14SecretBaseViewE", 0xB4),
    ("N3app7townmap9ZukanViewE", 0x7C),
    ("N3app7townmap11MessageViewE", 0x7C),
];
pub const UTIL: i64 = 0x5C;
pub const UTIL_CLASS: &str = "N3app4util7G2DUtilE";
pub const LAYOUTS: i64 = 0x1C;
pub const LAYOUT_COUNT: i64 = 0x20;
pub const ENTRY: i64 = 8;
pub const WORK: i64 = 0x04;
pub const MAX_LAYOUTS: i64 = 16;

/// The AreaNav's views a sweep found, and the layouts of those it has put away.
pub struct Views {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// View -> the vtable it was found with, and where its byte saying it is drawn is.
    views: IndexMap<i64, (i64, i64)>,
}

impl Views {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Views {
            memory,
            names,
            views: IndexMap::new(),
        }
    }

    /// The views a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut views = if !adding {
            IndexMap::new()
        } else {
            self.views.clone()
        };
        for (&view, name) in found {
            if let Some(&(_, drawn_at)) = VIEWS.iter().find(|(class, _)| class == name) {
                views.insert(view, (u32(&self.memory.read(view, 4)?, 0)?, drawn_at));
            }
        }
        self.views = views;
        Ok(())
    }

    /// The works of the layouts of every view the AreaNav is not drawing now.
    pub fn put_away(&self) -> Result<HashSet<i64>> {
        let memory = &self.memory;
        let mut works: HashSet<i64> = HashSet::new();
        for (&view, &(vtable, drawn_at)) in &self.views {
            let head = memory.read(view, drawn_at.max(UTIL) + 4)?;
            // A view freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable || head[drawn_at as usize] != 0 {
                continue;
            }
            let util = u32(&head, UTIL)?;
            if util == 0
                || self
                    .names
                    .of_vtable(u32(&memory.read(util, 4)?, 0)?)?
                    .as_deref()
                    != Some(UTIL_CLASS)
            {
                continue;
            }
            let listed = memory.read(util + LAYOUTS, 8)?;
            let (table, count) = (u32(&listed, 0)?, u32(&listed, 4)?);
            if table == 0 || !(0 < count && count <= MAX_LAYOUTS) {
                continue;
            }
            let entries = memory.read(table, ENTRY * count)?;
            for i in 0..count {
                let handle = u32(&entries, ENTRY * i)?;
                if handle != 0 {
                    let work = u32(&memory.read(handle + WORK, 4)?, 0)?;
                    if work != 0 {
                        works.insert(work);
                    }
                }
            }
        }
        Ok(works)
    }
}
