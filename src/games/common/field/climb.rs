//! Climbing walls: X's Cyllage Gym's rock faces, climbed from the blue spots at their foot, along
//! the row of cells in front of each, and off onto a ledge at a face's side (spike A).
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's kinds; `gen6.game.Climbing` carries them.
//! - ADR-0024: a climb is said as legs, each covering cells.

use indexmap::IndexMap;

use crate::core::pathfind::Cell;

/// A handhold, in world units: a cell's height. Up the wall from (25, 32) at 3 the player rose to
/// 13, 38 and 57 for the ledge at 54, three holds (spike A).
pub const HOLD: f64 = 18.0;

/// What a player on a face's line may be doing, a search's state there: walking its floor, on the
/// wall, or either. Off every line it is `FOOT`. Bits, so `state & WALL` asks one.
pub const FOOT: i64 = 1;
pub const WALL: i64 = 2;
pub const EITHER: i64 = 3;

/// One stretch of a way climbed, by index along the path: got on at `path[on]`, off at
/// `path[off]`, every step between on the wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Climb {
    pub on: i64,
    pub off: i64,
    /// Handholds climbed in place where it gets on, up positive, down negative. A climb that ends
    /// stepping sideways onto a ledge must be at the ledge's height before it, so its whole change
    /// is here. Python's default is 0.
    pub first: i64,
    /// Handholds climbed down in place where it gets off at a blue spot on the floor. Python's
    /// default is 0.
    pub last: i64,
}

/// The matrix's climbing walls, as the search and the list ask them: none on most maps
/// (`Walls::default()`, Python's `NO_WALLS`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Walls {
    /// Each cell of a face's line, with the face's number: the open cells in front of the face,
    /// from its outermost blue spot to its other, which the climber stands in (`mmodel.cell` gave
    /// (25, 32) to (31, 32) all the way along).
    pub lines: IndexMap<Cell, i64>,
    /// Each face's foot, by its number: the height of its line's floor.
    pub feet: Vec<f64>,
    /// Each blue spot, with its height: the face's foot, or a ledge's at the face's side.
    pub pads: IndexMap<Cell, f64>,
    /// Each ledge's blue spot, with the cell of the line beside it that the climber steps off the
    /// face from onto it, sideways, and back on: a cliff on foot.
    pub ledges: IndexMap<Cell, Cell>,
}

impl Walls {
    pub fn found(
        _cells: &IndexMap<Cell, (i64, Option<f64>)>,
        _face: i64,
        _sideways: &IndexMap<i64, &'static str>,
    ) -> Walls {
        unimplemented!()
    }

    pub fn sideways(&self, _cell: Cell, _step: Cell) -> bool {
        unimplemented!()
    }

    pub fn toll(&self, _cell: Cell, _step: Cell) -> i64 {
        unimplemented!()
    }

    pub fn start(&self, _cell: Cell, _level: Option<f64>) -> i64 {
        unimplemented!()
    }

    pub fn hold(&self, _state: i64, _cell: Cell, _step: Cell) -> Option<i64> {
        unimplemented!()
    }

    pub fn climbs(&self, _path: &[Cell], _level: Option<f64>) -> Vec<Climb> {
        unimplemented!()
    }

    pub fn spots(&self) -> Vec<(Vec<Cell>, bool)> {
        unimplemented!()
    }
}
