//! Who is speaking a field message.
//!
//! The game draws no name plate and keeps no name for the people on its maps, but it does say which
//! of them a message belongs to: a talk object holds the message window it is showing and the model
//! showing it (`talk`), and the model says which character it is (`mmodel`). Spike 73 read a whole
//! conversation off this — Norman, the trainer who had just battled him, and Wally, each line on the
//! right one.
//!
//! A name for a character is `pa3ds.core.names`: the shipped table, or what the player has called
//! them. A character nobody has named is left unsaid, so dialogue reads exactly as it did before.
//!
//! Nothing is read until a page is about to be announced. The actor is written as the message begins
//! and cleared between messages (spike 73), so asking at that moment is asking when it is true, and a
//! field with nobody talking costs nothing.
//!
//! Whoever spoke last is remembered, because that is who a player wants to name: they have just
//! heard a line from somebody the game does not name, and the key is pressed after the line, not
//! during it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::names::Names;
use crate::games::gen6::{mmodel, talk};
use crate::py::Result;

/// Tracks the field's talk objects and says who is speaking a window.
///
/// Shared: the profile hands `last` and `name_for` to the readers that ask, so the mutable parts
/// are in `Cell` and `RefCell` and every method takes `&self`.
pub struct SpeakerReader {
    memory: Rc<dyn Memory>,
    names: Rc<Names>,
    /// Where the game keeps the sprite a move model is drawn with (`mmodel.drawn_as`).
    drawn: i64,
    /// The character that last spoke, for a rename: a player names somebody having just
    /// heard them, which is a moment later than the page that named them.
    last: Cell<Option<i64>>,
    /// Every talk object found. Kept for good: a talk object's class is a DLL's while a talk
    /// runs, so it cannot be found again exactly when it matters.
    talking: RefCell<Vec<i64>>,
}

impl SpeakerReader {
    /// `SpeakerReader(memory, names, drawn)`.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<Names>, drawn: i64) -> Self {
        SpeakerReader {
            memory,
            names,
            drawn,
            last: Cell::new(None),
            talking: RefCell::new(Vec::new()),
        }
    }

    /// `SpeakerReader(memory, names)`: `drawn` left to its default, `mmodel.DRAWN`.
    pub fn with_default_drawn(memory: Rc<dyn Memory>, names: Rc<Names>) -> Self {
        SpeakerReader::new(memory, names, mmodel::DRAWN)
    }

    /// Keep these talk objects, as well as every one kept before.
    pub fn track(&self, objects: impl IntoIterator<Item = i64>) {
        for at in objects {
            let mut talking = self.talking.borrow_mut();
            if !talking.contains(&at) {
                talking.push(at);
            }
        }
    }

    /// Whether a talk object kept is showing this message window, with anyone speaking or not:
    /// a sign's talk has nobody, and a talk object not found yet has no say at all.
    pub fn shows(&self, window: i64) -> Result<bool> {
        for &at in self.talking.borrow().iter() {
            if u32(&self.memory.read(at + talk::WINDOW, 4)?, 0)? == window {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Which character is showing this message window, or None when none is.
    pub fn character_of(&self, window: i64) -> Result<Option<i64>> {
        for &at in self.talking.borrow().iter() {
            let head = self.memory.read(at + talk::WINDOW, talk::HEAD)?;
            if u32(&head, 0)? != window {
                continue;
            }
            let actor = u32(&head, talk::ACTOR - talk::WINDOW)?;
            if actor == 0 {
                continue;
            }
            let head = self.memory.read(actor, mmodel::HEAD)?;
            let character = mmodel::drawn_as(&*self.memory, actor, &head, self.drawn)?;
            if character != 0 {
                return Ok(Some(character));
            }
        }
        Ok(None)
    }

    /// What to call whoever is speaking this window, or None.
    ///
    /// Remembers the character either way: somebody nameless is exactly who a rename is for, so
    /// the one worth remembering is the one this returns None for.
    pub fn name_for(&self, window: i64) -> Result<Option<String>> {
        let character = self.character_of(window)?;
        if let Some(character) = character {
            self.last.set(Some(character));
        }
        Ok(character.and_then(|character| self.names.sprite(character)))
    }

    /// The character that last spoke, or None if nobody has this session.
    pub fn last(&self) -> Option<i64> {
        self.last.get()
    }
}
