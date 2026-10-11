//! The PlayNav's Pokémon-Amie page, whose Pokémon is a picture named by its species (spike 198).
//!
//! After the tutorial the page shows one Pokémon of the player's walking in its ring, a 40 by 30
//! touch area on a picture of its own, and the game names neither: touch mode said "touch: 2 of
//! layout 0x15a024bc has no name" and left it out, so the player could not open Amie at all
//! (omega_ruby_bottom-screen1). The picture draws texture 0 of a `common::resource::PokeIcon`, the
//! class the AreaNav's Pokémon Distribution and the Bag draw their icons with
//! (`screens.distribution`, `screens.partyicons`). The page's object is a class of its module, Game
//! Freak's "kawaigari" (Amie), `field::subscreen::kawaigari::FM_Kawaigari`:
//!
//! ```text
//! FM_Kawaigari  +0x14C  the national number of the Pokémon the page shows (254, Sceptile, in the
//!                       replay; a second 0x18-byte record at +0x164 held the same 254)
//!               +0x194  -> a record whose +0x00 -> common::resource::PokeIcon
//! PokeIcon      +0x0C   -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture 0     +0x0C   the image's physical address, as a picture's material keeps it
//! ```
//!
//! A picture drawing that image is the Pokémon's. Measured with one Pokémon on one replay: that
//! +0x14C holds the number for any other species is taken from the two records agreeing.
//!
//! PLAY opens Amie itself, `kawaigari::MainProc`: the Pokémon's model on the bottom screen and no
//! button drawn, its Poké Puffs, minigames and way back hidden (omega_ruby_bottom-screen4). Amie
//! strokes the Pokémon by a slide over it, and shows its hand where the stylus is (spike 200, live in
//! the rig). So while it runs touch mode offers the Pokémon, "stroke Sceptile", whose key slides
//! back and forth across the middle of the screen, where Amie stands the Pokémon: an exploration
//! past ADR-0009's tap, which Khai asked for (2026-10-08).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, touch};
use crate::games::gen6::screens::distribution as d;
use crate::py::Result;

pub const CLASS: &str = "N5field9subscreen9kawaigari12FM_KawaigariE";
/// Pokémon-Amie itself, the app PLAY starts.
pub const AMIE: &str = "N9kawaigari8MainProcE";
/// Back and forth across the middle of the screen, over the Pokémon's body: Sceptile stood from
/// about 110 to 200 across and 60 to 170 down, and the hand followed every pass (spike 200).
pub const STROKE: [(i64, i64); 6] = [
    (110, 110),
    (210, 110),
    (110, 130),
    (210, 130),
    (110, 110),
    (210, 110),
];
pub const STROKE_SECONDS: f64 = 3.0;
/// The pane a stroke stands in for in touch mode, which no pane is at.
pub const STROKE_TARGET: i64 = 1;
/// pa3ds's word for the slide, the manual's: "you can stroke your Pokémon".
pub const STROKE_WORD: &str = "stroke";
pub const NUMBER: i64 = 0x14C;
pub const HOLDER: i64 = 0x194;
pub const HEAD: i64 = HOLDER + 4;

/// Names the picture of the Pokémon the Pokémon-Amie page shows by its species.
pub struct AmiePage {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    /// Page object -> the vtable it was found with.
    pages: IndexMap<i64, i64>,
    /// The species the page last showed, which Amie opens on.
    last: Option<String>,
}

impl AmiePage {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, species: Vec<String>) -> Self {
        AmiePage {
            memory,
            names,
            species,
            pages: IndexMap::new(),
            last: None,
        }
    }

    /// The pages a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut pages = if adding {
            self.pages.clone()
        } else {
            IndexMap::new()
        };
        for (&page, name) in found {
            if name == CLASS {
                pages.insert(page, u32(&self.memory.read(page, 4)?, 0)?);
            }
        }
        self.pages = pages;
        self.species()?;
        Ok(())
    }

    /// The species a picture draws the Amie page's Pokémon as, in the game's words, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        if self.pages.is_empty() || self.species.is_empty() {
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
        for (&page, &vtable) in &self.pages {
            let head = memory.read(page, HEAD)?;
            // A page freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                continue;
            }
            let (number, holder) = (u32(&head, NUMBER)?, u32(&head, HOLDER)?);
            if holder == 0 || !(0 < number && number < self.species.len() as i64) {
                continue;
            }
            if self.first_image(u32(&memory.read(holder, 4)?, 0)?)? == Some(image) {
                let name = &self.species[number as usize];
                return Ok(if name.is_empty() {
                    None
                } else {
                    Some(name.clone())
                });
            }
        }
        Ok(None)
    }

    /// The species the page shows, or the one it last showed once it has gone, as it may
    /// while Amie runs.
    pub fn species(&mut self) -> Result<Option<String>> {
        for (&page, &vtable) in &self.pages {
            let head = self.memory.read(page, HEAD)?;
            let number = u32(&head, NUMBER)?;
            if u32(&head, 0)? == vtable
                && 0 < number
                && number < self.species.len() as i64
                && !self.species[number as usize].is_empty()
            {
                self.last = Some(self.species[number as usize].clone());
                break;
            }
        }
        Ok(self.last.clone())
    }

    /// The stroke touch mode offers while Amie runs, by the Pokémon's species.
    pub fn strokes(&mut self, running: Option<&str>) -> Result<Vec<Target>> {
        let species = if running == Some(AMIE) {
            self.species()?
        } else {
            None
        };
        let Some(species) = species else {
            return Ok(Vec::new());
        };
        Ok(vec![Target {
            path: STROKE.to_vec(),
            seconds: STROKE_SECONDS,
            ..Target::new(
                STROKE_TARGET,
                &format!("{STROKE_WORD} {species}"),
                160,
                120,
                0.0,
                touch::HEIGHT as f64,
            )
        }])
    }

    /// The image of texture 0 of a PokeIcon, or None where a link of the chain is not the
    /// class it should be.
    fn first_image(&self, icons: i64) -> Result<Option<i64>> {
        let (memory, names) = (&self.memory, &self.names);
        if icons == 0 || names.of_object(icons)?.as_deref() != Some(d::ICON_CLASS) {
            return Ok(None);
        }
        let replace = u32(&memory.read(icons + d::REPLACE, 4)?, 0)?;
        if replace == 0 || names.of_object(replace)?.as_deref() != Some(d::REPLACE_CLASS) {
            return Ok(None);
        }
        let head = memory.read(replace + d::TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= d::MAX_TEXTURES) {
            return Ok(None);
        }
        let texture = u32(&memory.read(listed, 4)?, 0)?;
        Ok(if texture != 0 {
            Some(u32(&memory.read(texture + d::IMAGE, 4)?, 0)?)
        } else {
            None
        })
    }
}
