//! The Battle Maison's partners for a Multi Battle, named on the buttons the player picks one from
//! (spike 242, or_multi_battle_partners).
//!
//! After "Please choose a Trainer to serve as your partner." the bottom screen shows a button a
//! partner, May, Wally, Maxie and Steven, each a portrait over the partner's name, and a menu
//! cursor, a window alone in a layout of its own, moved onto the one chosen. The game draws the
//! names with no TextBox, and the only copy of each is in the screen's object, so the cursor said
//! nothing and touch mode said "win trainer" four times, by the button's picture. The object is a
//! class of the Maison's module, `app::battle_house::Select1View`; each portrait draws a texture
//! of its own, which the view keeps in the partners' order:
//!
//! ```text
//! Select1View  +0xBC             how many partners
//!              +0x180 + 4 * i    -> partner i's portrait's texture: +0x18 the image's physical
//!                                   address, as a picture's material keeps it
//!              +0x1D0 + 0x28 * i    partner i: a u32, then the name, UTF-16, ended by a 0
//! ```
//!
//! Read in Omega Ruby (2026-10-23): the four textures' images were the four portraits', May's
//! first at (56, 35), and the view's sixteen slots past the count held 0xffff.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::games::common::lyt;
use crate::py::{self, Result};

pub const CLASS: &str = "N3app12battle_house11Select1ViewE";
pub const COUNT: i64 = 0xBC;
pub const TEXTURES: i64 = 0x180;
pub const IMAGE: i64 = 0x18;
pub const ENTRIES: i64 = 0x1D0;
pub const ENTRY: i64 = 0x28;
pub const NAME: i64 = 0x04;
/// The view's slots for partners: a count beyond is not a view's.
pub const MOST: i64 = 16;

/// Names a picture of the Maison's partner buttons by the partner whose portrait it draws.
pub struct Partners {
    memory: Rc<dyn Memory>,
    /// View -> its vtable, to tell it from what takes its memory once it is freed.
    views: IndexMap<i64, i64>,
    /// Image -> the partner it draws, until the next sweep: screen text asks of every picture
    /// at every poll.
    named: RefCell<IndexMap<i64, Option<String>>>,
}

impl Partners {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Partners {
            memory,
            views: IndexMap::new(),
            named: RefCell::new(IndexMap::new()),
        }
    }

    /// The views a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut views = if adding {
            self.views.clone()
        } else {
            IndexMap::new()
        };
        for (&view, name) in found {
            if name == CLASS {
                views.insert(view, u32(&self.memory.read(view, 4)?, 0)?);
            }
        }
        self.views = views;
        self.named.borrow_mut().clear();
        Ok(())
    }

    /// The partner whose portrait a picture draws, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        if self.views.is_empty() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        let image = if material != 0 {
            u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?
        } else {
            0
        };
        if image == 0 {
            return Ok(None);
        }
        if !self.named.borrow().contains_key(&image) {
            let drawn = self.drawn(image)?;
            self.named.borrow_mut().insert(image, drawn);
        }
        Ok(self.named.borrow()[&image].clone())
    }

    fn drawn(&self, image: i64) -> Result<Option<String>> {
        let memory = &self.memory;
        for (&view, &vtable) in &self.views {
            let head = memory.read(view, ENTRIES + ENTRY * MOST)?;
            let count = u32(&head, COUNT)?;
            if u32(&head, 0)? != vtable || !(0 < count && count <= MOST) {
                continue;
            }
            for i in 0..count {
                let texture = u32(&head, TEXTURES + 4 * i)?;
                if texture != 0 && u32(&memory.read(texture + IMAGE, 4)?, 0)? == image {
                    let at = ENTRIES + ENTRY * i + NAME;
                    let raw = py::slice(&head, at, at + ENTRY - NAME);
                    let ended = (0..raw.len())
                        .step_by(2)
                        .find(|&j| py::slice(raw, j as i64, j as i64 + 2) == b"\0\0")
                        .unwrap_or(raw.len());
                    let name = py::decode_utf16le(&raw[..ended]);
                    return Ok((!name.is_empty()).then_some(name));
                }
            }
        }
        Ok(None)
    }
}
