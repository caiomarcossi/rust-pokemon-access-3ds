//! The Bag's party slots, named by the species whose icon each draws (spike 184).
//!
//! The Bag lists the party beside its items, each slot an icon, a level and an HP bar, and no name:
//! touch mode said "L 14" (Orion, movie21). The icons come from a `common::resource::PokeIcon`, as
//! the AreaNav's Pokémon Distribution's do (`gen6.screens.distribution`): its `LytTexReplaceSystem` holds a
//! texture a party member, in the party's order, and each slot's picture draws its member's. The
//! party is a list of six `pml::pokepara::PokemonParam` pointers and then how many are in it, with
//! no class of its own:
//!
//! ```text
//! party     +0x00  six pointers to PokemonParams    +0x18 u32 how many are in the party
//! PokeIcon  +0x0C  -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture   +0x0C  the image's physical address, as a picture's material keeps it
//! ```
//!
//! Only while the Bag runs: another screen's icons are not the party's in its order.

use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::screens::distribution as d;
use crate::py::{PySet, Result};

pub const BAG: &str = "N3app3bag4ProcE";
pub const ICON_CLASS: &str = d::ICON_CLASS;
pub const SLOTS: i64 = 6;
pub const COUNT: i64 = 0x18;

/// The party's list among the addresses holding a PokemonParam: six pointers to them, the
/// first `COUNT` of them filled, then the count.
pub fn party_of(
    holders: impl IntoIterator<Item = i64>,
    memory: &dyn Memory,
    names: &ClassNames,
) -> Result<Option<i64>> {
    for at in holders {
        let head = memory.read(at, COUNT + 4)?;
        let count = u32(&head, COUNT)?;
        if !(0 < count && count <= SLOTS) {
            continue;
        }
        let mut all = true;
        for i in 0..SLOTS {
            if names.of_object(u32(&head, 4 * i)?)?.as_deref() != Some(pokemon::PARAM_CLASS) {
                all = false;
                break;
            }
        }
        if all {
            return Ok(Some(at));
        }
    }
    Ok(None)
}

/// Names a picture of the Bag's party by the species of the member whose icon it draws.
pub struct PartyIcons {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    running: Rc<dyn Fn() -> Option<String>>,
    party: Option<i64>,
    icons: PySet,
}

impl PartyIcons {
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
    ) -> Self {
        PartyIcons {
            memory,
            names,
            species,
            running,
            party: None,
            icons: PySet::new(),
        }
    }

    /// What a sweep found: the PokemonParams, to find the party by, and the PokeIcons.
    pub fn track(
        &mut self,
        params: &[i64],
        icons: impl IntoIterator<Item = i64>,
        holders: &dyn Fn(&[i64]) -> Result<Vec<i64>>,
        adding: bool,
    ) -> Result<()> {
        self.icons = if adding {
            self.icons.union(&PySet::of(icons))
        } else {
            PySet::of(icons)
        };
        if !params.is_empty() && (self.party.is_none() || !adding) {
            let found = party_of(holders(params)?, &*self.memory, &self.names)?;
            self.party = match found {
                Some(party) if party != 0 => Some(party),
                _ => self.party,
            };
        }
        Ok(())
    }

    /// The species of the party member a picture draws the icon of, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        if (self.running)().as_deref() != Some(BAG) || self.species.is_empty() {
            return Ok(None);
        }
        let Some(party_at) = self.party else {
            return Ok(None);
        };
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        if material == 0 {
            return Ok(None);
        }
        let image = u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?;
        for icon in self.icons.iter() {
            if self.names.of_object(icon)?.as_deref() != Some(ICON_CLASS) {
                continue;
            }
            let replace = u32(&memory.read(icon + d::REPLACE, 4)?, 0)?;
            if replace == 0 || self.names.of_object(replace)?.as_deref() != Some(d::REPLACE_CLASS) {
                continue;
            }
            let head = memory.read(replace + d::TEXTURES, 8)?;
            let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
            if listed == 0 || !(0 < count && count <= d::MAX_TEXTURES) {
                continue;
            }
            let table = memory.read(listed, 4 * count)?;
            let mut slot = None;
            for i in 0..count {
                if u32(&memory.read(u32(&table, 4 * i)? + d::IMAGE, 4)?, 0)? == image {
                    slot = Some(i);
                    break;
                }
            }
            let Some(slot) = slot else {
                continue;
            };
            let party = memory.read(party_at, COUNT + 4)?;
            if slot >= u32(&party, COUNT)? {
                return Ok(None);
            }
            let found = pokemon::read(
                &**memory,
                &self.names,
                u32(&party, 4 * slot)?,
                pokemon::SPECIES_MAX,
            )?;
            let Some(found) = found else {
                return Ok(None);
            };
            if found.species >= self.species.len() as i64 {
                return Ok(None);
            }
            return Ok(
                Some(self.species[found.species as usize].clone()).filter(|name| !name.is_empty())
            );
        }
        Ok(None)
    }
}
