//! `App::Tool::TalkWindow`: Sun's box for a conversation, and who is speaking in it.
//!
//! ```text
//! TalkWindow  +0x60 -> its `app::util::G2DUtil`, whose +0x34 -> its message window
//!             +0xa8 -> `gfl2::str::StrBuf`, the name on the plate above the box, empty for none
//! ```
//!
//! Read in spike 111, at four moments of the movie: each talk window led to the message window
//! showing its words, and lasted from one message to the next. Its plate said "Mom" over her lines,
//! and nothing through Kukui's video call, whose box has none. Unlike ORAS, Sun names its speakers
//! itself, so the name is the game's own words.
//!
//! A `G2DUtil` outlives the conversations it shows: the field's was there after Mom's window had
//! closed, its +0x34 back to zero. So a new message window is found the poll its util points at it,
//! not at the next sweep, which reached Mom's first window six seconds after its first page.

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::strbuf;
use crate::py::Result;

pub const CLASS: &str = "N3App4Tool10TalkWindowE";
pub const UTIL: i64 = 0x60;
pub const SPEAKER: i64 = 0xA8;
pub const UTIL_CLASS: &str = "N3app4util7G2DUtilE";
pub const UTIL_WINDOW: i64 = 0x34;

/// Who is saying a message window's message: the name on its talk window's plate.
///
/// Shared: the announcements hold `name_for` while the profile tracks the talk windows, so the
/// list is in a `RefCell` and every method takes `&self`.
pub struct Speakers {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    talk: RefCell<Vec<i64>>,
}

impl Speakers {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Speakers {
            memory,
            names,
            talk: RefCell::new(Vec::new()),
        }
    }

    /// Read these talk windows from now on: in place of those read so far, or, `adding` those
    /// a sweep of the blocks just allocated found, beside them.
    pub fn track(&self, windows: impl IntoIterator<Item = i64>, adding: bool) {
        let windows: Vec<i64> = windows.into_iter().collect();
        let mut talk = self.talk.borrow_mut();
        if adding {
            // Each is tested against the windows read so far, not against those added before it.
            let fresh: Vec<i64> = windows
                .into_iter()
                .filter(|window| !talk.contains(window))
                .collect();
            talk.extend(fresh);
        } else {
            *talk = windows;
        }
    }

    /// The name on the plate over this message window, or None for no plate or no talk
    /// window showing it.
    pub fn name_for(&self, window: i64) -> Result<Option<String>> {
        let talks = self.talk.borrow().clone();
        for talk in talks {
            if self.names.of_object(talk)?.as_deref() != Some(CLASS) {
                continue;
            }
            let util = u32(&self.memory.read(talk + UTIL, 4)?, 0)?;
            if self.names.of_object(util)?.as_deref() != Some(UTIL_CLASS) {
                continue;
            }
            if u32(&self.memory.read(util + UTIL_WINDOW, 4)?, 0)? != window {
                continue;
            }
            let buffer = u32(&self.memory.read(talk + SPEAKER, 4)?, 0)?;
            return strbuf::label(&*self.memory, buffer);
        }
        Ok(None)
    }
}

/// The message windows of these classes that these `G2DUtil`s point at now.
pub fn windows(
    memory: &dyn Memory,
    names: &ClassNames,
    utils: impl IntoIterator<Item = i64>,
    classes: &[&str],
) -> Result<Vec<i64>> {
    let mut found: Vec<i64> = Vec::new();
    for util in utils {
        let head = memory.read(util, UTIL_WINDOW + 4)?;
        if names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(UTIL_CLASS) {
            continue;
        }
        let window = u32(&head, UTIL_WINDOW)?;
        if window != 0
            && names
                .of_object(window)?
                .is_some_and(|name| classes.contains(&name.as_str()))
        {
            found.push(window);
        }
    }
    Ok(found)
}
