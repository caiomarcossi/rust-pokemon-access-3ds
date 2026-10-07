//! The AreaNav's Pokémon Distribution: each Pokémon's picture, named by its species.
//!
//! The Pokémon tab's View Details draws the place as a scene, with each Pokémon seen there walking
//! in it as its icon, a 40 by 30 picture and a touch area each. The icons are stored with a palette
//! in the ROM and expanded as they load, so no file holds the image a picture draws and the
//! texture index cannot name it (spikes 176 and 177). The AreaNav's navigator,
//! `app::tool::ZukanNavi`, lists the place's species, and loads the n-th one's icon into the n-th
//! texture of its icon loader (spike 178, on Route 101, 102, 103, 104 and Petalburg Woods, each
//! icon checked by eye):
//!
//! ```text
//! ZukanNavi +0x44     -> common::resource::PokeIcon
//!           +0x48     a record of 0x14 bytes a species: the national number, a u16, first
//!           +0x1D8    how many records there are
//! PokeIcon  +0x0C     -> app::tool::LytTexReplaceSystem
//! LytTexReplaceSystem +0x04 -> its textures, a pointer each; +0x08 how many
//! app::tool::LytTexData +0x0C   the image's physical address, as a picture's material keeps it
//! ```
//!
//! A picture drawing the image of the n-th texture shows the n-th record's species. The game draws
//! no picture for a species the player has not seen, so a species named is one the game shows; and
//! while a place's icons load, every picture draws the grey shape of the last texture, past the
//! records, which names none.

use std::rc::Rc;

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::lyt;
use crate::py::{PySet, Result};

pub const CLASS: &str = "N3app4tool9ZukanNaviE";
pub const ICONS: i64 = 0x44;
pub const ICON_CLASS: &str = "N6common8resource8PokeIconE";
pub const REPLACE: i64 = 0x0C;
pub const REPLACE_CLASS: &str = "N3app4tool19LytTexReplaceSystemE";
pub const TEXTURES: i64 = 0x04;
pub const TEXTURE_COUNT: i64 = 0x08;
pub const IMAGE: i64 = 0x0C;
pub const RECORDS: i64 = 0x48;
pub const RECORD: i64 = 0x14;
pub const STATE: i64 = 0x0C;
/// A record's byte `STATE`: 2 for a species caught, 3 seen, 6 not seen, which the DexNav draws no
/// icon for (spike B, movie27: Zigzagoon, Poochyena, Surskit, Masquerain and Magikarp 2, as the
/// save's Pokédex has them caught). The byte after it is the way they are met: 0 in the grass, 1 on
/// the water, 2 fishing, 3 the DexNav's alone.
pub const CAUGHT: i64 = 2;
pub const RECORD_COUNT: i64 = 0x1D8;
/// Records the navigator has room for, from +0x48 to its count.
pub const MAX_RECORDS: i64 = 20;
pub const MAX_TEXTURES: i64 = 32;

/// Names a picture the Pokémon Distribution draws a Pokémon's icon with.
pub struct Distribution {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    navis: PySet,
}

impl Distribution {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, species: Vec<String>) -> Self {
        Distribution {
            memory,
            names,
            species,
            navis: PySet::new(),
        }
    }

    /// The navigators a sweep found: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: impl IntoIterator<Item = i64>, adding: bool) {
        self.navis = if adding {
            self.navis.union(&PySet::of(found))
        } else {
            PySet::of(found)
        };
    }

    /// The species a picture draws the icon of, in the game's words, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let record = if self.species.is_empty() {
            None
        } else {
            self.record(picture)?
        };
        let Some(record) = record else {
            return Ok(None);
        };
        let number = u16(&record, 0)?;
        if !(0 < number && number < self.species.len() as i64) {
            return Ok(None);
        }
        Ok(Some(self.species[number as usize].clone()).filter(|name| !name.is_empty()))
    }

    /// Whether the species a picture draws the icon of is caught: the DexNav draws it in
    /// colour, and its silhouette while it is only seen (`STATE`).
    pub fn caught(&self, picture: i64) -> Result<bool> {
        let record = self.record(picture)?;
        Ok(record.is_some_and(|record| record[STATE as usize] as i64 == CAUGHT))
    }

    /// The navigator's record of the species a picture draws the icon of, or None.
    fn record(&self, picture: i64) -> Result<Option<Vec<u8>>> {
        if self.navis.is_empty() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        if material == 0 {
            return Ok(None);
        }
        let image = u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?;
        for navi in self.navis.iter() {
            let textures = self.textures(navi)?;
            let mut slot = None;
            for (i, texture) in textures.iter().enumerate() {
                if u32(&memory.read(texture + IMAGE, 4)?, 0)? == image {
                    slot = Some(i as i64);
                    break;
                }
            }
            let Some(slot) = slot else {
                continue;
            };
            let count = u32(&memory.read(navi + RECORD_COUNT, 4)?, 0)?;
            if slot >= count.min(MAX_RECORDS) {
                return Ok(None);
            }
            return Ok(Some(memory.read(navi + RECORDS + slot * RECORD, RECORD)?));
        }
        Ok(None)
    }

    /// The navigator's icon textures in order, or none where a link of the chain is not the
    /// class it should be: a navigator gone, its memory taken by something else.
    fn textures(&self, navi: i64) -> Result<Vec<i64>> {
        let (memory, names) = (&self.memory, &self.names);
        if names.of_vtable(u32(&memory.read(navi, 4)?, 0)?)?.as_deref() != Some(CLASS) {
            return Ok(Vec::new());
        }
        let icons = u32(&memory.read(navi + ICONS, 4)?, 0)?;
        if icons == 0
            || names
                .of_vtable(u32(&memory.read(icons, 4)?, 0)?)?
                .as_deref()
                != Some(ICON_CLASS)
        {
            return Ok(Vec::new());
        }
        let replace = u32(&memory.read(icons + REPLACE, 4)?, 0)?;
        if replace == 0
            || names
                .of_vtable(u32(&memory.read(replace, 4)?, 0)?)?
                .as_deref()
                != Some(REPLACE_CLASS)
        {
            return Ok(Vec::new());
        }
        let head = memory.read(replace + TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= MAX_TEXTURES) {
            return Ok(Vec::new());
        }
        let table = memory.read(listed, 4 * count)?;
        (0..count).map(|i| u32(&table, 4 * i)).collect()
    }
}
