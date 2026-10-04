//! The map as every game lists it: what is on it worth walking to, and the ground under it.
//!
//! Gen 6 reads these out of its zone files, move models and chunks (`gen6.field`), Sun out of its
//! actors and its walls (`sm.field`); what a player walks the list of, and where pathfinding may
//! walk, is the same in both:
//!
//! - **`Interactable`** — one thing worth walking to, and **`Field`**, where the player is with
//!   everything around them; `describe` puts a listing in the log.
//! - A place's **source**: which table it was read from, over its own number (`SOURCE_KIND`).
//! - **`Ground`** — where the player can walk: the cells of the **chunks** loaded (`Chunk`, the
//!   cell table gen 6's chunks keep and Sun's walls are laid out as, spike 114), less where the
//!   matrix has no zone (gen 6's `Grid`), or a rail's corridor instead (X's Lumiose, `gen6.rail`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Box, Cell, EAST, NORTH, SOUTH, WEST};
use crate::py::{self, Error, Result};

/// What the ground asks of a rails' corridor (`gen6.rail.Corridor`): its own walls are the only
/// ones there, and it is the whole of where a way may be found.
pub trait Corridor {
    fn open(&self, cell: Cell) -> Result<bool>;

    fn r#box(&self) -> Result<Box>;
}

/// Cells along a chunk's side.
pub const CHUNK: i64 = 40;

/// The bit of a cell's word that is set where the player cannot walk.
pub const BLOCKED: i64 = 0x1;

/// The bit of a cell's word that is set on water, where the blocked bit is clear and only a player
/// surfing goes. Route 109's sea is `0x3d1a0006`, and the player could not walk onto it
/// (omega_ruby_surf.ctm, spike 136). Across every chunk in both ROMs the bit is set on every word
/// of nine kinds and no other: the sea `0x3d`, `0x41` in 3-by-3 patches inside it (Dive's, by the
/// look), `0x42` between the sea and walls, the four `0x94` to `0x97` (currents, one a way, as
/// ledges are), and `0x3f` and `0x40`, which are blocked as well (`SHORE`). X has `0x3d`, `0x3f`
/// and `0x40`.
pub const WATER: i64 = 0x2;

/// The kind of the strip of shore between land and the sea, one cell wide: water and blocked.
/// Surf jumps the player over it, never onto it, getting on the water and off: on Route 122 from
/// (22, 8) to (22, 10) over (22, 9), and from (54, 58) to (54, 56) over (54, 57), and P walled the
/// water off at both (omega_ruby_surf3.ctm, spike 162). Of its 1,528 cells in ORAS's chunks,
/// nearly all have land on one side and the sea on the other. `0x3f`, the other blocked water,
/// has water on both sides and no land, and is not read.
pub const SHORE: i64 = 0x40;

pub const CUT: i64 = 15;
pub const SURF: i64 = 57;
pub const STRENGTH: i64 = 70;
/// The moves that open a way, by their numbers, the same in every game and each its line in the
/// move names: "Surf", "なみのり" in both ORAS and X (spike 136).
pub const ROCK_SMASH: i64 = 249;

/// What going onto water costs a path on top of the step, five steps' worth: the player faces the
/// water, presses A, and answers the game's question before the step is taken. So a way surfing
/// is taken over one on foot only when it saves more than that. Cutting a bush down or smashing a
/// rock is the same face, A and answer, and costs the same.
pub const BOARD: i64 = 10;

/// Where a cell's kind sits in its word: the top byte.
pub const KIND_SHIFT: i64 = 24;

/// A ledge's kind -> the way it is jumped down (spike 126). The game's own function at
/// `0x00402e74` turns these kinds into 0, 1, 2 and 3, the engine's north, south, west and east as a
/// rail's record numbers them (spike 107). omega_ruby_ledges.ctm jumped 0x75 south three times,
/// omega_ruby_ledges_2.ctm 0x73 west and omega_ruby_ledges_3.ctm 0x72 east; ORAS has no ledge that
/// drops north, so 0x74 rests on the numbering.
/// A ledge is blocked, and the player's cell goes from before it to after it in one move.
pub const LEDGES: [(i64, &str); 4] = [(0x74, NORTH), (0x75, SOUTH), (0x73, WEST), (0x72, EAST)];

/// A counter's kind: the Pokémon Center's, the Poké Mart's. The game talks across one, to the
/// thing in the cell beyond it: the field DLL's function at `0x007175a8` reads the kind of the cell
/// the player faces, and where the executable's test at `0x00403004` says it is this one, looks two
/// cells ahead instead of one. No other kind is tested there. In Petalburg's Pokémon Center the
/// Nurse stands at (19, 15) with the counter at (19, 16) and every cell beside her a counter or a
/// wall, and P said "no path to Nurse" (poke_center_pc.ctm, 2026-10-01).
pub const COUNTER: i64 = 0xD4;

/// A sandy slope's kind -> the way it goes down, in the order the ledges' kinds go. Its cells are
/// open, but on foot the game never lets the player onto one going up: in Granite Cave 1F
/// (omega_ruby_cave.ctm) the player at (10, 29) faced west onto `0x69000020` at (9, 29) and (8, 29)
/// and did not move. Only the Mach Bike rides up one, as the player did up Jagged Pass's `0x6c`
/// slopes, north from (28, 80), sliding back down the first time (omega_ruby_bike.ctm, spike 158).
/// Across ORAS's map chunks the ground's mesh falls the same way across every cell of a kind: all
/// 25 of 0x69 east, all 44 of 0x6a west, 166 of 0x6c's 188 south and the rest level along its foot
/// or top. No chunk has 0x6b, so north rests on the numbering. On foot a slope is walked down only,
/// and on the bike either way (`Ground.bike`).
pub const SLOPES: [(i64, &str); 4] = [(0x69, EAST), (0x6A, WEST), (0x6B, NORTH), (0x6C, SOUTH)];

/// A bumpy slope's kind, the rocky step only the Acro Bike's bunny hop gets up: walled to the
/// player, and in Jagged Pass each one stands in a row of ledges dropping south, where a ledge
/// would be, five of them up a column at x 50 from z 86 to 73, or alone across a narrow way, as at
/// x 25. Only in 13 of the ROM's chunk files, the Safari Zone's among them, which has bumpy slopes
/// too (spike 159, 2026-10-03). Read off the map, not seen hopped.
pub const BUMPY_SLOPES: [i64; 1] = [0x6F];

/// The Acro Bike's rails, by their kinds, not Lumiose's (`gen6.rail`): walled to the player, and
/// crossed only on the Acro Bike, hopping from rail to rail. Both are only in Route 119's map
/// files, the Safari Zone's, which has rails too, and one other, and on Route 119 each runs in a
/// line from open ground to open ground: 0x6e at (107, 219) to (114, 219), (114, 222) and (121,
/// 222), beside the path north up the route, and 0x6d up and down the lines of 0x6e in the route's
/// top (spike B). Read off the map, not seen ridden.
pub const BIKE_RAILS: [i64; 2] = [0x6D, 0x6E];

/// `{n:#x}`, which writes a negative number as Python does, with its sign before the prefix.
fn hex(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    format!("{sign}{:#x}", n.unsigned_abs())
}

/// `table.get(kind)`, for a table in the order its Python dict was written in.
fn way_of(table: &[(i64, &'static str)], kind: i64) -> Option<&'static str> {
    table
        .iter()
        .find(|(known, _)| *known == kind)
        .map(|&(_, way)| way)
}

/// One floor of a chunk that has more than one: its own cell table, and the height of the
/// floor under each cell, None where none is.
#[derive(Debug, Clone, PartialEq)]
pub struct Floor {
    pub words: Vec<i64>,
    pub heights: Vec<Option<f64>>,
}

/// One chunk's cell table, at the cells it covers.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chunk {
    /// The north-west cell.
    pub origin: (i64, i64),
    /// One a cell, `CHUNK` to a row.
    pub words: Vec<i64>,
    /// Every floor of a chunk with a deck over its ground, the ground first, or none: Mt.
    /// Chimney's catwalk over the crater is a floor of its own, open where the ground's table has
    /// the crater as a wall (spike 156). `words` is always the ground's.
    pub floors: Vec<Floor>,
    /// For a chunk flattened onto one floor a cell at a height, the height of each cell's floor,
    /// None where none is open (`mapblock.flat`); none otherwise.
    pub heights: Vec<Option<f64>>,
}

impl Chunk {
    /// A chunk with no floors and no heights, which `Chunk(origin, words)` is; the rest is set
    /// with `..Chunk::new(...)`.
    pub fn new(origin: (i64, i64), words: Vec<i64>) -> Self {
        Chunk {
            origin,
            words,
            floors: Vec::new(),
            heights: Vec::new(),
        }
    }

    /// Which chunk of the matrix this is, as (column, row).
    pub fn key(&self) -> (i64, i64) {
        (
            py::floordiv(self.origin.0, CHUNK),
            py::floordiv(self.origin.1, CHUNK),
        )
    }

    pub fn holds(&self, cell: (i64, i64)) -> bool {
        0 <= cell.0 - self.origin.0
            && cell.0 - self.origin.0 < CHUNK
            && 0 <= cell.1 - self.origin.1
            && cell.1 - self.origin.1 < CHUNK
    }

    /// `self.words[z * CHUNK + x]` for the cell, which counts from the end for a negative index
    /// as Python's tuples do, and raises past either end. Raised for the caller, as the Python
    /// raised it in each of the readers below.
    #[track_caller]
    fn word(&self, cell: (i64, i64)) -> Result<i64> {
        let (x, z) = (cell.0 - self.origin.0, cell.1 - self.origin.1);
        let len = self.words.len() as i64;
        let at = z * CHUNK + x;
        let at = if at < 0 { at + len } else { at };
        match usize::try_from(at).ok().and_then(|at| self.words.get(at)) {
            Some(&word) => Ok(word),
            None => Err(Error::index("tuple index out of range")),
        }
    }

    pub fn walkable(&self, cell: (i64, i64)) -> Result<bool> {
        Ok(self.word(cell)? & BLOCKED == 0)
    }

    pub fn water(&self, cell: (i64, i64)) -> Result<bool> {
        Ok(self.word(cell)? & WATER != 0)
    }

    /// Which way the cell goes down, a ledge jumped or a slope walked, or None where it is
    /// neither.
    pub fn ledge(&self, cell: (i64, i64)) -> Result<Option<&'static str>> {
        match way_of(&LEDGES, self.kind(cell)?) {
            Some(way) => Ok(Some(way)),
            None => self.slope(cell),
        }
    }

    /// Which way the cell goes down where it is a slope, or None.
    pub fn slope(&self, cell: (i64, i64)) -> Result<Option<&'static str>> {
        Ok(way_of(&SLOPES, self.kind(cell)?))
    }

    pub fn kind(&self, cell: (i64, i64)) -> Result<i64> {
        Ok(self.word(cell)? >> KIND_SHIFT)
    }

    /// Every cell of the chunk with its kind.
    pub fn kinds(&self) -> impl Iterator<Item = ((i64, i64), i64)> + '_ {
        self.words.iter().enumerate().map(|(i, &word)| {
            let i = i as i64;
            (
                (self.origin.0 + i % CHUNK, self.origin.1 + i / CHUNK),
                word >> KIND_SHIFT,
            )
        })
    }
}

/// The bits of a place's `source` that say which table or encoding the rest of it is in. Places
/// are keyed by (map, source), and each kind numbers its own from zero — a sign by its index in the
/// furniture table, a warp by its index in the warp table, a use tile by its first cell, an edge by
/// the zone it leads to — so without a tag, two places of different kinds on one map share a name
/// whenever their numbers happen to coincide: sign 1 and door 1 did until 2026-09-23 (renaming the
/// sign renamed the door, and the ROM's name for the door was said for the sign), and an edge's
/// `zone << 16` reached the bit a use tile was marked with, `1 << 24`, for any zone from `0x100` up
/// (the same day). The tag sits above 32 bits because every number under it is built from u16s the
/// game holds — a zone shifted by 16 at most — so nothing read from the game can reach it.
pub const SOURCE_KIND: i64 = 0xF << 32;

/// A furniture record's source is its bare index: the tag the first names were keyed under.
pub const FURNITURE_SOURCE: i64 = 0 << 32;

/// A warp's source: `WARP_SOURCE | index` in the warp table.
pub const WARP_SOURCE: i64 = 1 << 32;

/// A use tile's source: `TILE_SOURCE | x << 12 | z`, its first cell.
pub const TILE_SOURCE: i64 = 2 << 32;

/// An edge's source: `EDGE_SOURCE | zone << 16 | gap`, the zone it leads to and the gap's place
/// along the border.
pub const EDGE_SOURCE: i64 = 3 << 32;

/// A berry spot's source: `BERRY_SOURCE | index` in the game's table of them (spike 141).
pub const BERRY_SOURCE: i64 = 4 << 32;

/// A trigger's source, one a game knows as something to walk to: `TRIGGER_SOURCE | index` in the
/// zone's table of triggers (spike 146).
pub const TRIGGER_SOURCE: i64 = 5 << 32;

/// A Secret Base spot's source: `SECRET_BASE_SOURCE | number`, the spot's number in the game's
/// table of them (spike 154).
pub const SECRET_BASE_SOURCE: i64 = 6 << 32;

/// An ash pile's source: `ASH_SOURCE | index` in the zone's list of them (spike 155).
pub const ASH_SOURCE: i64 = 7 << 32;

/// A slope's or a rail end's source: `BIKE_SOURCE | x << 12 | z`, its first cell.
pub const BIKE_SOURCE: i64 = 8 << 32;

/// A place's `source` from its kind's tag and its own number. The number must leave the tag's
/// bits clear, and no u16 the game holds can reach them, so one that does is a new encoding
/// written wider than the tag allows for: a programmer's error, refused here rather than keyed as
/// another kind's place.
pub fn place_source(kind: i64, payload: i64) -> Result<i64> {
    if payload & SOURCE_KIND != 0 {
        return Err(Error::value(format!(
            "place payload {} reaches the kind bits",
            hex(payload)
        )));
    }
    Ok(kind | payload)
}

/// Which kind of place a source is: one of the `*_SOURCE` tags.
pub fn source_kind(source: i64) -> i64 {
    source & SOURCE_KIND
}

/// A place's own number under its tag: a warp's index, an edge's `zone << 16 | gap`.
pub fn source_number(source: i64) -> i64 {
    source & !SOURCE_KIND
}

/// Cells to a block of the matrix.
pub const BLOCK: i64 = 10;

pub const NO_ZONE: i64 = 0xFFFF;

/// Cells past the loaded chunks, the player and the goal that a path may go round through,
/// none of them known.
pub const SEARCH_MARGIN: i64 = 10;

/// The category that is every category: what `Field.of` gives the whole list for.
pub const ALL: &str = "all";

pub const SPRITE: &str = "sprite";

/// The two name spaces of `pa3ds.core.names`, as `Interactable.naming` chooses between them.
pub const PLACE: &str = "place";

pub const POKEMON: &str = "pokémon";

/// The soil berries are planted in, a category of its own (the user, 2026-09-29).
pub const BERRIES: &str = "berries";

/// Route 113's ash piles, a category of its own (the user, 2026-10-02).
pub const ASH: &str = "ash";

/// The ground only a bike gets up or across, the Mach Bike's slopes and the Acro Bike's rails and
/// bumpy slopes, a category of its own: the game names none of them, and the player could not find
/// them to record them (the user, 2026-10-03).
pub const BIKE: &str = "bike";

/// In the order they are walked and logged.
pub const CATEGORIES: [&str; 8] = [
    "exits", "signs", "people", "items", BERRIES, ASH, BIKE, POKEMON,
];

/// One thing worth walking to, in the player's cells.
#[derive(Debug, Clone, PartialEq)]
pub struct Interactable {
    /// One of `CATEGORIES`: `exits`, `signs`, `people`, `items`, `berries`, `ash`, `bike` or
    /// `pokémon`.
    pub category: String,
    /// What it is, in pa3ds's own words: `door`, `sign`, `PC`, `person`, `item ball`, `edge`,
    /// `berry spot`, `ash pile`, `hidden Pokémon`, `barrier`, `switch`, `passage`, `trapdoor`,
    /// `geyser`, `lift`, `teleporter`, `slope`, `bumpy slope`, `rail`.
    pub kind: String,
    pub cell: (i64, i64),
    pub size: (i64, i64),
    /// For an exit, the zone it leads to.
    pub zone: Option<i64>,
    /// The record's model code or script, a use tile's kind, or a hidden Pokémon's species, for
    /// the log and for names later.
    pub code: i64,
    /// What this was read from: a model's address for a move model, with the zone and index of the
    /// record it was built from above it in gen 6, since a slot is filled with the next zone's
    /// people as the player walks into it (spike 131); and for a place its kind's tag
    /// (`SOURCE_KIND`) over its own number — a furniture record's index, a warp's index, an edge's
    /// zone with the gap's place along the border after it (`zone << 16 | gap`), a use tile's first
    /// cell (`x << 12 | z`). With the kind, it tells the same thing apart from one press to the
    /// next while the list is re-sorted around a moving player.
    pub source: i64,
    /// Whether this is a move model — someone or something standing on the map now — rather than
    /// a record in the zone's own table. It decides which name a rename gives it (`naming`), and it
    /// is why `source` is an address here and an index there.
    pub move_model: bool,
    /// For a record of a zone's table, that zone, where it is known: the one the player is in,
    /// or on Lumiose's streets another zone of the same street (spike 107). A place is named by
    /// it, and told apart by it from the same slot of the other zone.
    pub home: Option<i64>,
}

impl Interactable {
    /// An interactable of one cell with the Python's defaults for the rest, which are set with
    /// `..Interactable::new(...)`.
    pub fn new(category: &str, kind: &str, cell: (i64, i64)) -> Self {
        Interactable {
            category: category.to_string(),
            kind: kind.to_string(),
            cell,
            size: (1, 1),
            zone: None,
            code: 0,
            source: 0,
            move_model: false,
            home: None,
        }
    }

    pub fn key(&self) -> (String, String, i64, Option<i64>) {
        (
            self.category.clone(),
            self.kind.clone(),
            self.source,
            self.home,
        )
    }

    /// Which name belongs to this, as (what, map, slot), or None for a thing with no name of
    /// its own.
    ///
    /// A **move model is a character**, so naming the person standing in the Gym names Norman
    /// everywhere, and names him as a speaker too: it is the same number either way
    /// (`mmodel.character`). Everything else on the map is a **place**, a slot on one map, so a
    /// sign named in Petalburg says nothing about a sign in Rustboro.
    ///
    /// The cost of that is a sprite everybody nameless shares: naming "the man in the Mart"
    /// names every townsperson drawn as him. That is already how the shipped table works —
    /// `0x128` is "Woman", Mrs. Birch and 27 zones of others — so it is the behaviour being
    /// kept, not a new one.
    ///
    /// **A hidden Pokémon has no name.** It is a different Pokémon each time and gone in a
    /// minute, its `source` is the record's address rather than a slot in any table, and the
    /// game names the species itself once the DexNav has registered it — so a name here would be
    /// a key that never matches again and would replace something the game says.
    pub fn naming(&self, zone: i64) -> Option<(&'static str, i64, i64)> {
        if self.category == POKEMON {
            return None;
        }
        if self.move_model {
            return Some((SPRITE, self.code, 0));
        }
        Some((PLACE, self.home.unwrap_or(zone), self.source))
    }

    /// Direction and distance from `origin`, as "3 east 2 north", or "here"; for a player
    /// heading one way on a rail, from where they face: "9 ahead 8 left".
    pub fn bearing(&self, origin: (i64, i64), heading: Option<&str>) -> Result<String> {
        if let Some(heading) = heading {
            return pathfind::facing(origin, self.nearest(origin), heading);
        }
        Ok(bearing(origin, self.nearest(origin)))
    }

    /// The cell of this that is closest to `origin`: an exit can be several cells wide.
    pub fn nearest(&self, origin: (i64, i64)) -> (i64, i64) {
        let x = origin.0.max(self.cell.0).min(self.cell.0 + self.size.0 - 1);
        let z = origin.1.max(self.cell.1).min(self.cell.1 + self.size.1 - 1);
        (x, z)
    }

    /// Every cell this covers.
    pub fn cells(&self) -> Vec<Cell> {
        let mut out = Vec::new();
        for dz in 0..self.size.1 {
            for dx in 0..self.size.0 {
                out.push((self.cell.0 + dx, self.cell.1 + dz));
            }
        }
        out
    }
}

/// Where the player is, and what is around them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Field {
    pub zone: i64,
    pub cell: (i64, i64),
    pub interactables: Vec<Interactable>,
    /// Whether the player is on a rail, as on Lumiose's boulevards, where the game keeps the
    /// streets' walls in a map of the rail's own and the chunks mark every cell a wall, the
    /// player's own included (spike 102). With the rails loaded, `cell` and every cell listed are
    /// the corridor's (`Corridor`).
    pub on_rail: bool,
    /// On a rail, the way of the corridor up walks the player, nearest `bearing`, which is the
    /// way their keys work from: the camera is behind them there, so up walks the way it looks,
    /// down back from it, and left and right turn the player a quarter, the camera following only
    /// while the key is held (spike 105).
    pub heading: Option<&'static str>,
    /// On a rail, the way up walks the player in the corridor, in degrees clockwise from its
    /// north: the camera's heading, which a tap of left or right leaves anywhere (spike 105).
    pub bearing: Option<f64>,
    /// Whether the player is riding the Mach Bike, which rides up a sandy slope (spike 158).
    pub on_bike: bool,
}

impl Field {
    /// A field with the Python's defaults for all but where the player is, the rest set with
    /// `..Field::new(...)`.
    pub fn new(zone: i64, cell: (i64, i64)) -> Self {
        Field {
            zone,
            cell,
            ..Field::default()
        }
    }

    /// This category's interactables (every one for `ALL`), nearest first.
    pub fn of(&self, category: &str) -> Vec<Interactable> {
        let mut items: Vec<Interactable> = self
            .interactables
            .iter()
            .filter(|item| category == ALL || item.category == category)
            .cloned()
            .collect();
        items.sort_by_key(|item| distance(self.cell, item.nearest(self.cell)));
        items
    }
}

/// The ways round, a quarter turn anticlockwise from each to the next.
pub const ANTICLOCKWISE: [&str; 4] = [NORTH, WEST, SOUTH, EAST];

/// `ANTICLOCKWISE.index(way)`, which raises where `way` is none of the four.
#[track_caller]
fn anticlockwise_index(way: &str) -> Result<i64> {
    match ANTICLOCKWISE.iter().position(|&known| known == way) {
        Some(at) => Ok(at as i64),
        None => Err(Error::value("tuple.index(x): x not in tuple")),
    }
}

/// One of Fortree Gym's rotating doors: wooden arms standing on the lines between cells, out
/// from a pivot where four cells meet, turned a quarter about it as one is walked into (spike
/// D). An arm stops a step across the line it stands on; a step into one turns the door the way
/// the step goes and is taken, but only where every cell the arms sweep through is open, an arm
/// two cells long sweeping the whole quarter between where it is and where it goes, one cell long
/// the cell beside the pivot.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RotatingDoor {
    /// The corner the arms turn about: the cell south-east of it is this cell.
    pub pivot: Cell,
    /// Each arm's way out from the pivot at no turn, and how many cells long it is.
    pub arms: IndexMap<&'static str, i64>,
    /// How many quarter turns anticlockwise it stands at now.
    pub turn: i64,
}

impl RotatingDoor {
    /// A door at no turn; `turn` is set with `..RotatingDoor::new(...)`.
    pub fn new(pivot: Cell, arms: IndexMap<&'static str, i64>) -> Self {
        RotatingDoor {
            pivot,
            arms,
            turn: 0,
        }
    }

    /// The arms as they stand `turn` quarters anticlockwise round.
    pub fn at(&self, turn: i64) -> Result<IndexMap<&'static str, i64>> {
        let mut out = IndexMap::new();
        for (&way, &length) in &self.arms {
            let round = py::modulo(anticlockwise_index(way)? + turn, 4);
            out.insert(ANTICLOCKWISE[round as usize], length);
        }
        Ok(out)
    }

    /// Which arm a step from `cell` to `step` walks into with the door at `turn`, as its way
    /// and the quarter turns the door takes, 1 anticlockwise or -1, or None for no arm.
    pub fn pushed(&self, turn: i64, cell: Cell, step: Cell) -> Result<Option<(&'static str, i64)>> {
        let ((px, pz), arms) = (self.pivot, self.at(turn)?);
        let (dx, dz) = (step.0 - cell.0, step.1 - cell.1);
        let (way, out, push);
        if dx != 0 && cell.0.max(step.0) == px {
            (way, out) = if cell.1 < pz {
                (NORTH, pz - cell.1)
            } else {
                (SOUTH, cell.1 - pz + 1)
            };
            push = if dx > 0 { EAST } else { WEST };
        } else if dz != 0 && cell.1.max(step.1) == pz {
            (way, out) = if cell.0 < px {
                (WEST, px - cell.0)
            } else {
                (EAST, cell.0 - px + 1)
            };
            push = if dz > 0 { SOUTH } else { NORTH };
        } else {
            return Ok(None);
        }
        if arms.get(way).copied().unwrap_or(0) < out {
            return Ok(None);
        }
        let ahead = ANTICLOCKWISE[py::modulo(anticlockwise_index(way)? + 1, 4) as usize];
        Ok(Some((way, if push == ahead { 1 } else { -1 })))
    }

    /// The cells the arms sweep through turning a quarter `by` (1 anticlockwise, -1 not).
    pub fn swept(&self, turn: i64, by: i64) -> Result<Vec<Cell>> {
        let (px, pz) = self.pivot;
        let mut out: Vec<Cell> = Vec::new();
        for (way, length) in self.at(turn)? {
            let goes = ANTICLOCKWISE[py::modulo(anticlockwise_index(way)? + by, 4) as usize];
            let east = way == EAST || goes == EAST;
            let south = way == SOUTH || goes == SOUTH;
            let xs = if east { [px, px + 1] } else { [px - 1, px - 2] };
            let zs = if south {
                [pz, pz + 1]
            } else {
                [pz - 1, pz - 2]
            };
            for &x in py::slice(&xs, 0, length) {
                for &z in py::slice(&zs, 0, length) {
                    out.push((x, z));
                }
            }
        }
        Ok(out)
    }
}

type Crossings = HashMap<(Cell, Cell), i64>;

thread_local! {
    static CROSSINGS: RefCell<HashMap<Vec<Cell>, Rc<Crossings>>> = RefCell::new(HashMap::new());
}

/// Every step, each way, across the lines out from each rotating door's pivot that an arm
/// two cells long can stand on, the only steps a door can stop or be turned by, with the door's
/// place in `pivots`. Asked of every step a search takes, so worked out once a set of doors.
fn crossings_of(pivots: &[Cell]) -> Rc<Crossings> {
    if let Some(known) = CROSSINGS.with(|cache| cache.borrow().get(pivots).cloned()) {
        return known;
    }
    let mut out: Crossings = HashMap::new();
    for (i, &(px, pz)) in pivots.iter().enumerate() {
        for k in [1, 2] {
            for (a, b) in [
                ((px - 1, pz - k), (px, pz - k)),
                ((px - 1, pz + k - 1), (px, pz + k - 1)),
                ((px - k, pz - 1), (px - k, pz)),
                ((px + k - 1, pz - 1), (px + k - 1, pz)),
            ] {
                out.insert((a, b), i as i64);
                out.insert((b, a), i as i64);
            }
        }
    }
    let out = Rc::new(out);
    CROSSINGS.with(|cache| cache.borrow_mut().insert(pivots.to_vec(), out.clone()));
    out
}

/// The field map's zone ids, `width` blocks to a row.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub width: i64,
    pub height: i64,
    pub zones: Vec<i64>,
}

impl Grid {
    /// The zone id of a block, `NO_ZONE` off the grid.
    pub fn at(&self, column: i64, row: i64) -> Result<i64> {
        if !(0 <= column && column < self.width) || !(0 <= row && row < self.height) {
            return Ok(NO_ZONE);
        }
        match usize::try_from(row * self.width + column)
            .ok()
            .and_then(|at| self.zones.get(at))
        {
            Some(&zone) => Ok(zone),
            None => Err(Error::index("tuple index out of range")),
        }
    }
}

/// The chunk at a (column, row) of the matrix as the ROM has it, or None: read where no
/// loaded chunk holds a cell, so a way to something far off is not a guess. P's way to a
/// Fisherman across the sea on Route 109 walked where the sea was not loaded, and was given up
/// once it was (omega_ruby_surf_2.ctm, spike 136). None without the ROM's chunks to read.
pub type Far = Rc<dyn Fn(i64, i64) -> Result<Option<Rc<Chunk>>>>;

/// Where the player can walk, as far as the map loaded now says: what pathfinding walks on.
///
/// A cell cannot be walked where its chunk's blocked bit is set — a tree, a wall, a door, a sign,
/// a ledge — where the matrix has no zone, or where a move model stands. A ledge is jumped the
/// way it drops, from the cell before it to the cell after, and a slope, which is open, walked
/// only the way it goes down, as a conveyor belt is the way it runs (`ledge`). Only the four
/// chunks around the player are loaded; the rest of the matrix is the ROM's (`far`). A cell
/// neither holds is taken to be open, and a path that turns out to cross a wall is found again
/// once the chunk with the wall in it loads.
/// Water is a wall too, unless the ground is one to `surf`, and a bush or a rock a move clears
/// unless it is one to clear them (`hm`).
#[derive(Clone)]
pub struct Ground {
    pub chunks: Vec<Rc<Chunk>>,
    pub grid: Option<Grid>,
    /// Cells somebody or something stands in: a person, an item ball, a prop.
    pub standing: IndexSet<Cell>,
    /// Cells that do something when walked onto: a warp takes the player off the map, and a
    /// hidden Pokémon jumps out. Walked round, unless one is where the player is going.
    pub stepped: IndexSet<Cell>,
    /// The rails' corridor the player is in, whose own walls are the only ones there: a cell
    /// no line of it holds is not somewhere on these streets.
    pub corridor: Option<Rc<dyn Corridor>>,
    /// How many cells out from a solid thing a path may end, looking out from each side until
    /// one is open. One in ORAS, where a thing is its tiles. Sun's props are wider than the records
    /// their things are read by: the cells beside a flower pot's record were inside the pot, and P
    /// never said the player had arrived (sun_5.ctm, 2026-09-26).
    pub reach: i64,
    /// Cells of a conveyor belt, each with the way it runs: open, and walked only that way, as a
    /// slope is. The cell table has plain floor there; the belt is a trigger (spike 132).
    pub belts: IndexMap<Cell, &'static str>,
    /// Whether water is crossed, as Surf crosses it: Shift and P's ground, and P's for a player
    /// already on the water. P walked into the sea on Route 109 while the blocked bit was all it
    /// read (omega_ruby_surf.ctm, spike 136).
    pub surf: bool,
    /// The chunk at a (column, row) of the matrix as the ROM has it, or None: read where no
    /// loaded chunk holds a cell, so a way to something far off is not a guess. P's way to a
    /// Fisherman across the sea on Route 109 walked where the sea was not loaded, and was given up
    /// once it was (omega_ruby_surf_2.ctm, spike 136). None without the ROM's chunks to read.
    pub far: Option<Far>,
    /// Cells a thing a move clears stands in, each with the move: a bush Cut clears, a rock Rock
    /// Smash does, a boulder Strength pushes. Stood in like any other thing, and walked through
    /// where the ground is one to `hm`.
    pub cleared: IndexMap<Cell, i64>,
    /// Whether the way goes through what a move clears, and up a slope: Shift and P's ground.
    pub hm: bool,
    /// Whether the player is riding the Mach Bike, which rides up a slope as well as down: P's
    /// ground goes up one only then (spike 158).
    pub bike: bool,
    /// Cells side by side, each way, that can both be stood in and are not walked between: a
    /// platform in Fortree City's trees and the ground beside it, 108 below, but for its ladder
    /// (`joined`, spike 160).
    pub cliffs: IndexSet<(Cell, Cell)>,
    /// The bumpy slopes, each with the way it is hopped up on the Acro Bike, from the cell before
    /// it to the cell after, as a ledge is jumped down: walls on foot (spike 159).
    pub hops: IndexMap<Cell, &'static str>,
    /// Fortree Gym's rotating doors, as they stand now: walked through by turning them, a step
    /// into an arm turning its door (`through`, spike 161).
    pub doors: Vec<RotatingDoor>,
    /// The heights of the map's floors other than the player's, nearest theirs first, at which
    /// the ground can be walked instead (`FieldSource.ground`). A chunk with a deck is walked on
    /// the floor nearest one height, and from the foot of Route 119 that kept the way under its
    /// bridge and cut the bridge, the only way north (spike 159).
    pub decks: Vec<f64>,
}

impl Default for Ground {
    fn default() -> Self {
        Ground {
            chunks: Vec::new(),
            grid: None,
            standing: IndexSet::new(),
            stepped: IndexSet::new(),
            corridor: None,
            reach: 1,
            belts: IndexMap::new(),
            surf: false,
            far: None,
            cleared: IndexMap::new(),
            hm: false,
            bike: false,
            cliffs: IndexSet::new(),
            hops: IndexMap::new(),
            doors: Vec::new(),
            decks: Vec::new(),
        }
    }
}

impl fmt::Debug for Ground {
    /// The ground without its two callables, which have nothing to show.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ground")
            .field("chunks", &self.chunks)
            .field("grid", &self.grid)
            .field("standing", &self.standing)
            .field("stepped", &self.stepped)
            .field("corridor", &self.corridor.is_some())
            .field("reach", &self.reach)
            .field("belts", &self.belts)
            .field("surf", &self.surf)
            .field("far", &self.far.is_some())
            .field("cleared", &self.cleared)
            .field("hm", &self.hm)
            .field("bike", &self.bike)
            .field("cliffs", &self.cliffs)
            .field("hops", &self.hops)
            .field("doors", &self.doors)
            .field("decks", &self.decks)
            .finish()
    }
}

/// `Ground.of`'s arguments after `where`, each as the Python defaults it: set what differs with
/// `..Of::default()`.
#[derive(Default)]
pub struct Of {
    pub chunks: Vec<Rc<Chunk>>,
    pub grid: Option<Grid>,
    pub corridor: Option<Rc<dyn Corridor>>,
    pub belts: IndexMap<Cell, &'static str>,
    pub far: Option<Far>,
    pub clears: IndexMap<i64, i64>,
    pub solid: Vec<Cell>,
}

impl Ground {
    /// The ground under `where`'s listing: its move models stand in their cells, all of a
    /// barrier's beam, and its warps, triggers and hidden Pokémon are stepped round: a way to
    /// Wattson over one of his Gym's switches would swing a barrier shut behind the player
    /// (spike 146). A move model drawn as a sprite `clears` has is cleared by that move, a script
    /// of its own or not: three of Rusturf Tunnel's ten rocks
    /// have a sign under them and a script, and are the rocks Rock Smash breaks to join the
    /// tunnel's halves, the only way through it to Verdanturf Town (spike 137). `solid` is
    /// cells something stands in that is not listed: a Secret Base's desks and chairs, which the
    /// cell table has as open floor (spike 154).
    pub fn of(r#where: &Field, of: Of) -> Ground {
        let Of {
            chunks,
            grid,
            corridor,
            belts,
            far,
            clears,
            solid,
        } = of;
        let mut standing: IndexSet<Cell> = solid.into_iter().collect();
        for item in &r#where.interactables {
            if item.move_model {
                standing.extend(item.cells());
            }
        }
        let mut stepped: IndexSet<Cell> = IndexSet::new();
        for item in &r#where.interactables {
            let source = source_kind(item.source);
            if item.category == POKEMON
                || (!item.move_model && (source == WARP_SOURCE || source == TRIGGER_SOURCE))
            {
                stepped.extend(item.cells());
            }
        }
        let mut cleared: IndexMap<Cell, i64> = IndexMap::new();
        for item in &r#where.interactables {
            if item.move_model
                && let Some(&clear) = clears.get(&item.code)
            {
                for cell in item.cells() {
                    cleared.insert(cell, clear);
                }
            }
        }
        Ground {
            chunks,
            grid,
            standing,
            stepped,
            corridor,
            belts,
            far,
            cleared,
            ..Ground::default()
        }
    }

    /// Whether the player can stand in the cell.
    pub fn open(&self, cell: Cell) -> Result<bool> {
        if let Some(corridor) = &self.corridor {
            return Ok(!self.standing.contains(&cell) && corridor.open(cell)?);
        }
        if let Some(grid) = &self.grid
            && grid.at(py::floordiv(cell.0, BLOCK), py::floordiv(cell.1, BLOCK))? == NO_ZONE
        {
            return Ok(false);
        }
        if self.standing.contains(&cell) && !(self.hm && self.cleared.contains_key(&cell)) {
            return Ok(false);
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(true);
        };
        Ok(chunk.walkable(cell)? && (self.surf || !chunk.water(cell)?))
    }

    /// Whether the cell is water, as far as a loaded chunk or the ROM's says.
    pub fn water(&self, cell: Cell) -> Result<bool> {
        if self.corridor.is_some() {
            return Ok(false);
        }
        match self.chunk(cell)? {
            Some(chunk) => chunk.water(cell),
            None => Ok(false),
        }
    }

    /// Whether the cell is shore the ground jumps over, getting on the water or off it: only
    /// a ground to `surf` does (`SHORE`).
    pub fn shore(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() || !self.surf {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => Ok(chunk.kind(cell)? == SHORE),
            None => Ok(false),
        }
    }

    /// What a move from `cell` to `step` costs on top of its steps: going onto water,
    /// through what a move clears, or up a slope from the foot of it on foot, where the player
    /// gets on the bike first.
    pub fn toll(&self, cell: Cell, step: Cell) -> Result<i64> {
        if self.cleared.contains_key(&step) || (self.water(step)? && !self.water(cell)?) {
            return Ok(BOARD);
        }
        if !self.bike && self.hopping(cell, step) {
            return Ok(BOARD);
        }
        Ok(
            if !self.bike && self.climbs(cell, step)? && self.slope(cell)?.is_none() {
                BOARD
            } else {
                0
            },
        )
    }

    /// Which way the cell goes down, a ledge jumped, a slope walked or a belt ridden, or
    /// None where it is none of them or no loaded chunk holds it. A slope is None too where
    /// the ground rides up it (`bike`, `hm`).
    pub fn ledge(&self, cell: Cell) -> Result<Option<&'static str>> {
        if self.corridor.is_some() {
            return Ok(None);
        }
        if let Some(&way) = self.belts.get(&cell) {
            return Ok(Some(way));
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(None);
        };
        if (self.bike || self.hm) && chunk.slope(cell)?.is_some() {
            return Ok(None);
        }
        if let Some(&way) = self.hops.get(&cell) {
            // Hopped over on the Acro Bike as a ledge is jumped, but up: from (49, 31) through
            // (49, 30) to (49, 29) in Jagged Pass (omega_ruby_bike-jagged2.ctm, spike 159).
            return Ok(if self.bike || self.hm {
                Some(way)
            } else {
                None
            });
        }
        chunk.ledge(cell)
    }

    /// Which way the cell goes down where it is a slope, or None.
    pub fn slope(&self, cell: Cell) -> Result<Option<&'static str>> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => chunk.slope(cell),
            None => Ok(None),
        }
    }

    /// Whether the move from `cell` to `step` is onto, off or over a bumpy slope, hopped up
    /// on the Acro Bike: a search's move over one lands two cells on.
    pub fn hopping(&self, cell: Cell, step: Cell) -> bool {
        let over = (
            py::floordiv(cell.0 + step.0, 2),
            py::floordiv(cell.1 + step.1, 2),
        );
        [cell, step, over]
            .iter()
            .any(|one| self.hops.contains_key(one))
    }

    /// Whether the move from `cell` to `step` goes up a slope, which only the Mach Bike
    /// does: onto it or along it any way but the way it goes down.
    pub fn climbs(&self, cell: Cell, step: Cell) -> Result<bool> {
        let down = self.slope(step)?;
        Ok(down.is_some_and(|down| down != pathfind::towards(cell, step)))
    }

    /// The loaded chunk holding the cell, or else the ROM's.
    fn chunk(&self, cell: Cell) -> Result<Option<Rc<Chunk>>> {
        let loaded = self.chunks.iter().find(|chunk| chunk.holds(cell)).cloned();
        let Some(far) = &self.far else {
            return Ok(loaded);
        };
        if loaded.is_some() {
            return Ok(loaded);
        }
        far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))
    }

    /// Whether the cell is a counter, which the player talks across.
    pub fn counter(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => Ok(chunk.kind(cell)? == COUNTER),
            None => Ok(false),
        }
    }

    /// Whether a step from `cell` to the cell beside it can be taken: not off a drop.
    pub fn joined(&self, cell: Cell, step: Cell) -> bool {
        !self.cliffs.contains(&(cell, step))
    }

    /// How far each door stands turned now, in `doors`' order: where a search starts.
    pub fn turns(&self) -> Vec<i64> {
        self.doors.iter().map(|door| door.turn).collect()
    }

    /// Every step a door's arm can stand across, with the door's place in `doors`: for
    /// `through`, which a search asks of every step it takes.
    pub fn crossings(&self) -> Rc<HashMap<(Cell, Cell), i64>> {
        let pivots: Vec<Cell> = self.doors.iter().map(|door| door.pivot).collect();
        crossings_of(&pivots)
    }

    /// The doors' turns after a step from `cell` to `step` with them at `turns`: as they were
    /// where it walks into no arm, the door turned a quarter where it walks into one and every
    /// cell its arms sweep is open, and None where they are not, and the step is not taken.
    /// `crossings` is `crossings()`, worked out once a search.
    pub fn through(
        &self,
        turns: &[i64],
        cell: Cell,
        step: Cell,
        crossings: Option<&HashMap<(Cell, Cell), i64>>,
    ) -> Result<Option<Vec<i64>>> {
        let worked;
        let crossings = match crossings {
            Some(crossings) => crossings,
            None => {
                worked = self.crossings();
                &*worked
            }
        };
        let Some(&i) = crossings.get(&(cell, step)) else {
            return Ok(Some(turns.to_vec()));
        };
        let door = usize::try_from(i)
            .ok()
            .and_then(|i| self.doors.get(i))
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let at = i as usize;
        let turned = turns
            .get(at)
            .copied()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let Some(pushed) = door.pushed(turned, cell, step)? else {
            return Ok(Some(turns.to_vec()));
        };
        for swept in door.swept(turned, pushed.1)? {
            if !(self.open(swept)? || swept == cell) {
                return Ok(None);
            }
        }
        let mut after = turns[..at].to_vec();
        after.push(py::modulo(turned + pushed.1, 4));
        after.extend_from_slice(py::slice_from(turns, i + 1));
        Ok(Some(after))
    }

    /// Where along `path` a step walks into a door's arm and turns it, by the index of the
    /// cell it steps to.
    pub fn pushes(&self, path: &[Cell]) -> Result<IndexSet<i64>> {
        let mut out: IndexSet<i64> = IndexSet::new();
        let mut turns = self.turns();
        for i in 1..path.len() {
            let after = self.through(&turns, path[i - 1], path[i], None)?;
            if let Some(after) = after
                && after != turns
            {
                out.insert(i as i64);
                turns = after;
            }
        }
        Ok(out)
    }

    /// Whether a path may go through the cell on its way somewhere else.
    pub fn passable(&self, cell: Cell) -> Result<bool> {
        Ok(!self.stepped.contains(&cell) && self.open(cell)?)
    }

    /// Where a path to `item` ends, each with the way to face there, or None for any way.
    ///
    /// A thing with any cell the player can stand in is walked onto: an edge's gap, the mat out
    /// of a building. A thing that is solid all over — a door, a sign, a person, a PC — is
    /// walked up to, to an open cell beside it, facing it, since that is where the player
    /// presses A or walks in from (the user, 2026-09-24); or across a counter from it, as the
    /// Nurse and a Poké Mart's clerk are talked to (`COUNTER`).
    pub fn goals(&self, item: &Interactable) -> Result<IndexMap<Cell, Option<&'static str>>> {
        let cells = item.cells();
        // A bush or a rock is walked up to and cleared from there, with HMs or not.
        let mut onto: IndexMap<Cell, Option<&'static str>> = IndexMap::new();
        for &cell in &cells {
            if self.open(cell)? && !self.cleared.contains_key(&cell) {
                onto.insert(cell, None);
            }
        }
        if !onto.is_empty() {
            return Ok(onto);
        }
        let mut beside: IndexMap<Cell, Option<&'static str>> = IndexMap::new();
        for out in 1..=self.reach {
            for &cell in &cells {
                for (direction, (dx, dz)) in pathfind::STEPS {
                    let near = (cell.0 - dx * out, cell.1 - dz * out);
                    if !cells.contains(&near) && self.passable(near)? {
                        beside.insert(near, Some(direction));
                    } else if out == 1 && self.counter(near)? {
                        let across = (near.0 - dx, near.1 - dz);
                        if !cells.contains(&across) && self.passable(across)? {
                            beside.insert(across, Some(direction));
                        }
                    }
                }
            }
            if !beside.is_empty() {
                break;
            }
        }
        Ok(beside)
    }

    /// The cells a search from `start` may walk: the whole matrix where the ROM's chunks are
    /// read; otherwise round the loaded chunks, the start and the goals, `SEARCH_MARGIN`
    /// further, and never off the matrix; in a corridor, the corridor.
    ///
    /// A cell no chunk holds is taken to be open, so without the ROM's chunks a search kept to
    /// what is loaded, and P crossed Route 109's sea as land (spike 136). With them every cell
    /// is known, and keeping to the loaded chunks lost the way to Mauville City halfway up
    /// Route 110: it turns 38 east at x 56, and once the chunks east of x 40 unloaded no way was
    /// found inside x 49 (omega_ruby_movil.ctm, spike 142).
    pub fn r#box(&self, start: Cell, goals: impl IntoIterator<Item = Cell>) -> Result<Box> {
        if let Some(corridor) = &self.corridor {
            return corridor.r#box();
        }
        let mut cells: Vec<Cell> = vec![start];
        cells.extend(goals);
        for chunk in &self.chunks {
            cells.push(chunk.origin);
            cells.push((chunk.origin.0 + CHUNK - 1, chunk.origin.1 + CHUNK - 1));
        }
        let r#box = Box::around(cells, SEARCH_MARGIN)?;
        let Some(grid) = &self.grid else {
            return Ok(r#box);
        };
        let matrix = Box {
            west: 0,
            north: 0,
            east: grid.width * BLOCK - 1,
            south: grid.height * BLOCK - 1,
        };
        Ok(if self.far.is_some() {
            matrix
        } else {
            r#box.within(matrix)
        })
    }
}

/// One line a category, for the log: what is there and where, nearest first, what each is
/// `called` when it has a name, and the place an exit `leads` to when that is somewhere else. A
/// replay presses no navigation keys, so this is where it shows which door the game's own words
/// named (spike 86) and where each exit goes.
///
/// The lines are made as they are asked for, as the Python's generator makes them, so a
/// `called` or `leads` that is never reached is never run; one that fails ends the lines.
pub fn describe<'a>(
    r#where: &'a Field,
    called: Option<&'a dyn Fn(&Interactable) -> Result<Option<String>>>,
    leads: Option<&'a dyn Fn(&Interactable) -> Result<Option<String>>>,
) -> impl Iterator<Item = Result<String>> + 'a {
    let mut categories = CATEGORIES.iter();
    let mut failed = false;
    std::iter::from_fn(move || {
        if failed {
            return None;
        }
        for category in categories.by_ref() {
            let items = r#where.of(category);
            if items.is_empty() {
                continue;
            }
            let line = (|| -> Result<String> {
                let mut said: Vec<String> = Vec::new();
                for item in &items {
                    let place = match leads {
                        Some(leads) => leads(item)?,
                        None => None,
                    };
                    let detail = detail(item, place.as_deref());
                    let name = called_name(item, called)?;
                    let bearing = item.bearing(r#where.cell, None)?;
                    said.push(format!("{}{detail}{name} {bearing}", item.kind));
                }
                Ok(format!("{category}: {}", said.join("; ")))
            })();
            failed = line.is_err();
            return Some(line);
        }
        None
    })
}

fn called_name(
    item: &Interactable,
    called: Option<&dyn Fn(&Interactable) -> Result<Option<String>>>,
) -> Result<String> {
    let name = match called {
        Some(called) => called(item)?,
        None => None,
    };
    Ok(match name {
        None => String::new(),
        Some(name) => format!(" \"{name}\""),
    })
}

fn detail(item: &Interactable, place: Option<&str>) -> String {
    if let Some(zone) = item.zone {
        return format!(" to zone {}", hex(zone))
            + &match place {
                None => String::new(),
                Some(place) => format!(" ({place})"),
            };
    }
    if item.category == "pokémon" {
        return format!(" species {}", item.code);
    }
    String::new()
}

/// Direction and distance from `origin` to `cell`, as "3 east 2 north", or "here".
pub fn bearing(origin: (i64, i64), cell: (i64, i64)) -> String {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    let mut parts: Vec<String> = Vec::new();
    if dx != 0 {
        parts.push(format!(
            "{} {}",
            dx.abs(),
            if dx > 0 { "east" } else { "west" }
        ));
    }
    if dz != 0 {
        parts.push(format!(
            "{} {}",
            dz.abs(),
            if dz > 0 { "south" } else { "north" }
        ));
    }
    if parts.is_empty() {
        "here".to_string()
    } else {
        parts.join(" ")
    }
}

/// Whether the cell can be walked, or None when no loaded chunk holds it.
pub fn walkable(chunks: &[Rc<Chunk>], cell: (i64, i64)) -> Result<Option<bool>> {
    for chunk in chunks {
        if chunk.holds(cell) {
            return Ok(Some(chunk.walkable(cell)?));
        }
    }
    Ok(None)
}

fn distance(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}
