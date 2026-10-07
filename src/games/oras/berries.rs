//! ORAS's berry spots: the patches of soft soil a Berry is planted in (spike 141).
//!
//! No zone's data has one: the field's DLL lists all 90, which the ROM is read for
//! (`archives.berry_spots`).

use std::rc::Rc;

use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::{BERRIES, BERRY_SOURCE, Interactable, place_source};
use crate::games::gen6::field::features::{Feature, Tracked, ZoneAt};
use crate::games::gen6::rom::archives::GameText;
use crate::py::Result;

/// What a spot berries are planted in is called: the game says "It's soft soil." of an empty one
/// and names the Berry growing in one, but never names the spot.
pub const BERRY_SPOT: &str = "berry spot";

/// Every berry spot, (zone, cell) by its number, for the field to list; none without the ROM.
pub struct BerrySpots {
    spots: Vec<(i64, Cell)>,
}

impl BerrySpots {
    pub fn new(spots: Vec<(i64, Cell)>) -> Self {
        BerrySpots { spots }
    }

    /// The factory `Game.features` holds.
    pub fn of(_memory: Rc<dyn Memory>, text: Option<&GameText>) -> Rc<dyn Feature> {
        Rc::new(BerrySpots::new(
            text.map(|text| text.berries.clone()).unwrap_or_default(),
        ))
    }
}

impl Tracked for BerrySpots {
    fn classes(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn track(&self, _found: &Found) -> Result<()> {
        Ok(())
    }
}

impl Feature for BerrySpots {
    fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for (index, &(home, cell)) in self.spots.iter().enumerate() {
            if home == zone {
                out.push(Interactable {
                    source: place_source(BERRY_SOURCE, index as i64)?,
                    home: Some(zone),
                    ..Interactable::new(BERRIES, BERRY_SPOT, cell)
                });
            }
        }
        Ok(out)
    }
}
