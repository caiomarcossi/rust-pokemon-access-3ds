//! The field: where the player is, and what is on the map worth walking to.
//!
//! What it lists is every game's (`common.field`: `Interactable`, `Field`, `Ground`); this reads
//! gen 6's memory for them.
//!
//! Four structures, none with a class of its own, read in spike 78:
//!
//! **Zone data** — the current zone's data file as the ROM stores it, decompressed whole into the
//! heap. Magic `ZO`, a u16 count of sections (5, and 4 in X and Y) and one u32 offset more than
//! that, the first just past them. Section 0 is the zone's info (0x38 bytes), section 1 its entity
//! table, the same in both games (spike 100):
//!
//! ```text
//! +0x00 u32 size = 8 + the records      +0x04 u8 furniture, u8 NPCs, u8 warps, u8 triggers
//! +0x08 u8 areas, then the records in that order:
//! furniture 0x14: u16 script, kind, ?, placement, x, z, width, height, ?, 0
//! NPC       0x30: u16 index, model code, flag, 0, script, work, ?, 0 x5, width, height,
//!                 range x4, 0 x3, x, z, f32 y
//! warp      0x18: u16 to zone, to warp, kind, placement, s16 world x, y, z, u16 width,
//!                 height, 0 x3
//! ```
//!
//! A record whose placement is 1 is on a rail, as on Lumiose's boulevards: where the cell or the
//! world position would be it holds a u32 line id, s16 cells along it and across it, and u16 cells
//! it spans along and across (spike 102). Such a record is listed in the corridor of the rails
//! loaded (`rail`), and not at all off them.
//!
//! ```text
//! trigger   0x18: u16 script, ?, work, way, 0, 0, x, z, width, height, 0, 0; the work
//!           probably the value that arms it, 0 for none, and the way probably the way it
//!           is walked onto to fire, 5 north, 6 south, 7 west, 8 east, 0 any (spike 132)
//! area      0x18: the same shape as a trigger (six on Route 102, none in Littleroot)
//! ```
//!
//! Cells are (x, z), x east and z south; a warp's world units are 18 to a cell, the centre at
//! `cell * 18 + 9`. Two files are kept in a pair of buffers behind a two-word holder, the current
//! one first, with the current one's LZ11 original after them.
//!
//! **Move model** — `field::mmodel::MoveModel`, a pool of 32, one per person or thing on the map:
//! +0x04 zero when free, +0x0c `zone << 16 | NPC index` (0xff for the player), +0x10 `flag << 16 |
//! model code`, +0x14 `script << 16`, the cell it is in as floats at +0x94 and +0x9c. The zone is
//! the one the model was built in: walking into the next outdoor zone keeps the model.
//!
//! **Field map** — the matrix of the outdoors the player is in: a heap block of 0x9180 bytes with
//! a grid of u16 zone ids at +0x3c, one per block of 10 by 10 cells, 0xffff where there is no
//! zone, row by row. How wide the grid is depends on the matrix: +0x36 and +0x38 are its width and
//! height in chunks of four blocks, 8 by 8 for Littleroot's and Petalburg's, 10 by 10 for Route
//! 104's, 1 by 1 for the bedroom. Outdoor zones on one matrix share their cells; a building's
//! floor is a matrix of its own.
//!
//! **Zone table** — every zone's info, 0x38 bytes a row indexed by zone id, the ROM's own file
//! loaded whole, so the current zone's info is also its row and the table starts that zone's rows
//! before it (spike 100). Row +0x1a is the parent outdoor
//! zone, +0x08 an area kind — a town and its plain houses share one, and the buildings with a
//! name of their own have theirs: 0x10 a Pokémon Center, 0x45 a Poké Mart, 0x11 a Gym room — and
//! +0x1c a u16 whose low ten bits are the zone's place-name index, for when names come from the
//! ROM (spikes 85 and 86).
//!
//! **Hidden Pokémon** — the DexNav's find, whose tail shows above the tall grass: not a model in
//! the pool but a record of its own (spike 80), listed under a category of its own. ORAS alone has
//! one, and its reader is ORAS's (`games.oras.hidden`), handed to this one by the game
//! (`HiddenReader`).
//!
//! **Map block** — `field::MapBlock`, one of the four 40 by 40 chunks of the matrix the game keeps
//! loaded around the player, read in `mapblock` (spike 82). Its cell table says which cells can be
//! walked, which is how a zone's edge is narrowed to the gap in the trees, and what kind each cell
//! is, which is how a **use tile** is found: a PC, and whatever else the game runs a script for
//! when the player faces the cell and presses A, with no entity record behind it (spike 83).

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{s16, u16, u32};
use crate::core::discovery::{ALLOCATION, Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{self, Cell, EAST, NORTH, SOUTH, WEST};
use crate::games::common::field::{
    self as common, ASH, ASH_SOURCE, BERRIES, BERRY_SOURCE, BIKE, BIKE_RAILS, BIKE_SOURCE, BLOCK,
    BLOCKED, BUMPY_SLOPES, CHUNK, Chunk, EDGE_SOURCE, FURNITURE_SOURCE, Field, Floor, Grid, Ground,
    Interactable, KIND_SHIFT, NO_ZONE, Of, RotatingDoor, SECRET_BASE_SOURCE, SLOPES, TILE_SOURCE,
    TRIGGER_SOURCE, WARP_SOURCE, place_source, walkable,
};
use crate::games::gen6::{camera, mapblock, mmodel, rail};
use crate::py::{self, Error, PySet, Result};

/// What a sweep looks for. The slot's own shape is `mmodel`'s, as every structure's is.
pub const MOVE_MODEL_CLASS: &str = mmodel::CLASS;

pub const ZONE_MAGIC: &[u8] = b"ZO";
/// Five in ORAS, four in X and Y (spike 100): the info and the entity table lead in both.
pub const ZONE_SECTIONS: [i64; 2] = [5, 4];
pub const ZONE_INFO: i64 = 0;
pub const ZONE_ENTITIES: i64 = 1;
/// The zone's script, compiled Pawn (`script`, spike 107).
pub const ZONE_SCRIPT: i64 = 2;
pub const INFO_SIZE: i64 = 0x38;
/// The map matrix a zone is on, a file of `a/0/4/2` in X: Lumiose's zones 0x3f and 0x40 are both
/// South Boulevard's, 128 (spike 107).
pub const INFO_MATRIX: i64 = 0x04;
/// What a zone with no matrix of its own has at `INFO_MATRIX`: ORAS's placeholder zones 0 to 5,
/// and 39 rooms, the Trick House's among them, none of them walked between across an edge.
pub const NO_MATRIX: i64 = 0;
/// More rows than any game's zone table has: ORAS has 536, X 360.
pub const ZONE_LIMIT: i64 = 0x400;
/// An area kind: a town and its plain houses share one, and a building with a name of its own has
/// its own, the same in every town (spike 86).
pub const INFO_KIND: i64 = 0x08;
pub const INFO_PARENT: i64 = 0x1A;
pub const INFO_NAME: i64 = 0x1C;
/// The place-name index is the low ten bits of +0x1c; the bits above are flags (spike 85).
pub const NAME_MASK: i64 = 0x3FF;
pub const MAX_ZONE_SIZE: i64 = 0x20000;

pub const ENTITY_HEADER: i64 = 12;
pub const PLACEMENT: i64 = 0x06;
/// A furniture or warp record's +0x06: 0 on the grid, 1 on a rail (spike 102).
pub const ON_RAIL: i64 = 1;
pub const NPC_SPRITE: i64 = 0x02;
pub const NPC_FLAG: i64 = 0x0A;
/// An NPC record's +0x0a is 3000 plus a trainer's id when the NPC is one (spike 90).
pub const TRAINER_FLAGS: i64 = 3000;
pub const FURNITURE: i64 = 0x14;
pub const NPC: i64 = 0x30;
pub const WARP: i64 = 0x18;
pub const TRIGGER: i64 = 0x18;
pub const AREA: i64 = 0x18;
pub const TRIGGER_SCRIPT: i64 = 0x00;
pub const TRIGGER_X: i64 = 0x0C;
pub const TRIGGER_Z: i64 = 0x0E;
pub const TRIGGER_WIDTH: i64 = 0x10;
pub const TRIGGER_HEIGHT: i64 = 0x12;
/// How high in the world a trigger stands, in world units, where `TRIGGER_HEIGHT` is how many
/// cells deep it is: 108 for the top of each of Fortree City's ladders and 0 for its foot (spike
/// 160), -9 to 108 for Lavaridge Gym's (spike 157).
pub const TRIGGER_Y: i64 = 0x14;
/// World units per cell.
pub const CELL: i64 = 18;
pub const LZ11: i64 = 0x11;

/// A door in from outdoors.
pub const WARP_DOOR: i64 = 0x0301;
/// The mat out of a building.
pub const WARP_OUT: i64 = 0x0200;
/// The high byte of a warp walked into off one of Lumiose's streets, in X: into one of its
/// buildings (spike 101), or onto Centrico Plaza, another zone of its streets (spike 107).
pub const WARP_STREET: i64 = 0x07;
/// The high byte of a warp walked through: stairs between floors (low byte 1 up, 2 down), and the
/// open path between two outdoor matrices (Oldale to Route 102 is 3, back again 2).
pub const WARP_STAIRS: i64 = 0x05;
/// Model codes from here up are things, not people: a doll in the bedroom, a Poké Ball.
pub const PROP_MODEL: i64 = 0x2000;

/// The PC's cell kind, seen booted in poke_center_pc.ctm (spike 83).
pub const PC_TILE: i64 = 0xD5;

/// The cell kinds the game runs a script for when the player faces one and presses A: every kind
/// in the field's table of (kind test, script) pairs (spike 83). The PC is called what the game
/// calls it where the ROM can be read (`FieldReader`'s `words`) and "PC" where it cannot. The rest
/// are read by facing them like a prop, and are called what a prop is until each has been seen:
/// which is the trash can, a bookshelf or a vase is in the scripts, which pa3ds does not read, and
/// the game's text has only what each of them says, never a word for one.
///
/// Python's `USE_TILES` dict: `{PC_TILE: "PC", **dict.fromkeys(range(0xD7, 0xE0), "object")}`, as
/// the lookup `USE_TILES.get(kind)`.
pub fn use_tiles(kind: i64) -> Option<&'static str> {
    if kind == PC_TILE {
        Some("PC")
    } else if (0xD7..0xE0).contains(&kind) {
        Some("object")
    } else {
        None
    }
}

pub const MAP_BLOCK_SIZE: i64 = 0x9180;
pub const MAP_KIND: i64 = 0x08;
/// 32 in every matrix seen, whatever its width: it was taken for the width until Route 104's
/// matrix, 40 blocks wide, still read 32 here.
pub const MAP_MARK: i64 = 0x0C;
pub const MAP_MARKED: i64 = 32;
pub const MAP_CHUNKS_WIDE: i64 = 0x36;
/// Every matrix seen is square, so which of the two is the width is inferred from the order, not
/// measured.
pub const MAP_CHUNKS_TALL: i64 = 0x38;
pub const MAP_GRID: i64 = 0x3C;
/// Each chunk's file in the ROM's map chunks, a u16 a chunk row by row, after room for a grid of
/// 128 by 128 blocks: 619 to 660 for the part of Lumiose round South Boulevard, 7 chunks by 6, the
/// chunk each street section says it is (spike 106), and in ORAS 43 to 46 for Route 109's four
/// round the beach, each the ROM's file whose cell table the game loads (spike 136).
pub const MAP_CHUNK_FILES: i64 = 0x803C;
/// What the list has where the matrix has no chunk: most of Route 109's 12 by 10 (spike 136).
pub const NO_CHUNK_FILE: i64 = 0xFFFF;
/// How many chunks flattened at a height are kept (`FieldReader._flattened`).
pub const FLATTENED: i64 = 64;
pub const BLOCKS_A_CHUNK: i64 = CHUNK / BLOCK;
/// Sweeps a zone listed with no chunk asks for before leaving the rest to the schedule.
pub const ASKS: i64 = 4;
pub const TABLE_ASKS: i64 = 10;
/// Sweeps asked for while the zone data is in and the zone table is not, and the seconds between
/// them, before leaving the rest to the schedule. The table's row was in place 1.9 s after the
/// field started, but nothing pointed at its start yet, and it was found only by the whole sweep 30
/// s later: every exit said without its place, and the way to Oldale Town said as stairs
/// (omega_ruby_ledges_3.ctm, 2026-09-28).
pub const TABLE_EVERY: f64 = 2.0;

/// What a spot berries are planted in is called: the game says "It's soft soil." of an empty one
/// and names the Berry growing in one, but never names the spot.
pub const BERRY_SPOT: &str = "berry spot";

/// What a grey mound of volcanic ash on Route 113 is called: the game says how many grams the Soot
/// Sack has gathered, but never names the pile (spike 155).
pub const ASH_PILE: &str = "ash pile";

/// What a beam between two posts is called, which a switch swings a quarter turn: the game never
/// names one, and its move model's sprite is the invisible blocker it stands elsewhere (spike
/// 146).
pub const BARRIER: &str = "barrier";

pub const SLOPE: &str = "slope";
pub const BUMPY_SLOPE: &str = "bumpy slope";
/// What a sandy slope only the Mach Bike rides up, a rocky step only the Acro Bike hops up, and an
/// end of a rail only the Acro Bike crosses, are called: the game names none of them, and its words
/// for the bikes are their items' (spike 159).
pub const BIKE_RAIL: &str = "rail";

/// What a spot a Secret Base can be made at is called: the game draws a mark on the wall, the tree
/// or the bush, and says what it is on facing it, but never names the spot (spike 154).
pub const SECRET_BASE_SPOT: &str = "secret base spot";

/// What the player's own Secret Base's spot is called, said with the place it leads into, as an
/// exit is: "entrance to Secret Base".
pub const ENTRANCE: &str = "entrance";

/// What a Secret Base's flag beside its PC is called: the game never names it where it stands.
pub const FLAG: &str = "flag";

/// What the decorating cursor is on where nothing stands and nothing is in the way: the game says
/// nothing as it moves from one such cell to the next (the user's word, 2026-10-02).
pub const BLANK: &str = "blank";

/// What the decorating cursor is on where the cell table has the cell blocked and no decoration,
/// PC or flag stands: the base's walls and the hedge round them.
pub const WALL: &str = "wall";

pub const BASE_ASKS: i64 = 5;
/// Sweeps asked for inside a Secret Base while what stands where is not found, and the seconds
/// between them. The base's module is loaded as the player goes in, and its classes are learned a
/// look after a look that lists it, too late for the sweep that look comes with.
pub const BASE_EVERY: f64 = 2.0;

/// What a DexNav find is called: the game shows a tail and never names it (the user, 2026-09-22,
/// asking for the shaking-tail Pokémon under a category of its own).
pub const HIDDEN_POKEMON: &str = "hidden Pokémon";

/// A zone data file found in the heap, validated.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneData {
    pub address: i64,
    pub offsets: Vec<i64>,
}

impl ZoneData {
    pub fn size(&self) -> i64 {
        self.offsets[self.offsets.len() - 1]
    }

    pub fn info(&self) -> i64 {
        self.address + self.offsets[ZONE_INFO as usize]
    }

    pub fn entities(&self) -> i64 {
        self.address + self.offsets[ZONE_ENTITIES as usize]
    }
}

/// The two-word holder of the zone data buffers, current first.
struct Holder {
    address: i64,
    files: IndexMap<i64, ZoneData>,
}

/// A hidden Pokémon up in the grass, as the field lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct Hiding {
    /// Where the game keeps it, which tells it apart from one poll to the next
    /// (`Interactable.source`).
    pub record: i64,
    pub species: i64,
    pub cell: Cell,
}

/// A game's hidden Pokémon, where it has one: ORAS's DexNav (`games.oras.hidden`). Found by a
/// sweep, and read at every poll, since a tail shows and goes between sweeps.
pub trait HiddenReader {
    /// What a sweep looks for to find it by.
    fn classes(&self) -> Vec<&'static str>;

    fn track(&self, found: &Found) -> Result<()>;

    /// The one up now, or None.
    fn read(&self) -> Result<Option<Hiding>>;

    /// What its record holds before any check, for the log, and the cell it says; None before a
    /// sweep has found it.
    fn words(&self) -> Result<Option<(String, Cell)>>;
}

/// A game's ash piles, where it has them: ORAS's on Route 113 (`games.oras.ash`). Found by a sweep
/// as the route's module loads, and read at every listing, since a pile goes as the player walks
/// into it.
pub trait AshReader {
    /// What a sweep looks for to find them by.
    fn classes(&self) -> Vec<&'static str>;

    fn track(&self, found: &Found) -> Result<()>;

    /// (index, cell) of each pile in `zone` still there.
    fn piles(&self, zone: i64) -> Result<Vec<(i64, Cell)>>;
}

/// A game's rotating doors, where it has them: Fortree Gym's (`games.oras.doors`). Found by a
/// sweep as the Gym's module loads, and read whenever the ground is, since they turn as the player
/// walks.
pub trait DoorReader {
    /// What a sweep looks for to find them by.
    fn classes(&self) -> Vec<&'static str>;

    /// The sprite of the invisible models the game sets round a door for its arms.
    fn stand_in(&self) -> i64;

    fn track(&self, found: &Found) -> Result<()>;

    /// The doors in `zone` as they stand now, or none.
    fn doors(&self, zone: i64) -> Result<Vec<RotatingDoor>>;
}

/// A game's Secret Bases, where it has them: ORAS's (`games.oras.secret_base`). Every spot is the
/// game's, out of its executable; which is the player's is the save's, found by a sweep.
pub trait SecretBaseReader {
    /// What a sweep looks for to find the save by.
    fn classes(&self) -> Vec<&'static str>;

    /// (zone, cell) of every spot, by its number.
    fn spots(&self) -> Vec<(i64, Cell)>;

    /// A zone inside a base, whose place the game calls "Secret Base", as every base's is.
    fn inside(&self) -> i64;

    /// Every zone a base is built in.
    fn zones(&self) -> IndexSet<i64>;

    fn track(&self, found: &Found) -> Result<()>;

    /// The number of the player's own spot, or None.
    fn own(&self) -> Result<Option<i64>>;

    /// Whether a sweep has found what stands where in a base.
    fn furnished(&self) -> Result<bool>;

    /// The cell the decorating cursor is on, or None while it is not shown.
    fn cursor(&self) -> Result<Option<Cell>>;

    /// The decorations standing in a cell, by number.
    fn standing(&self, cell: Cell) -> Result<Vec<i64>>;

    /// The cells a decoration stands in that is walked round, not over.
    fn solid(&self) -> Result<IndexSet<Cell>>;
}

/// A chunk file's cell table out of the ROM, by its number (`archives.MapChunks`); None where
/// there is no ROM to read, and a chunk the game has not loaded is not known.
pub type FarCells = Rc<dyn Fn(i64) -> Result<Option<Vec<i64>>>>;
/// A chunk file's floors out of the ROM, by its number, where it has a deck over its ground
/// (`archives.MapChunks.floors`).
pub type FarFloors = Rc<dyn Fn(i64) -> Result<Vec<Floor>>>;
/// A chunk file's ground heights out of the ROM, by its number, which say where it drops
/// (`archives.MapChunks.heights`).
pub type FarHeights = Rc<dyn Fn(i64) -> Result<Option<Vec<Option<f64>>>>>;
/// Every line and track in a map's chunks and where its joints are, out of the ROM
/// (`archives.Streets`); None where there is no ROM to read, and only the lines the game has loaded
/// are known, with no joint to turn at.
pub type Streets = Rc<dyn Fn(&[i64]) -> Result<rail::Network>>;

/// `FieldReader`'s keyword arguments after `memory`, each as the Python defaults it: set what
/// differs with `..Options::default()`.
pub struct Options {
    /// The game's own word for a kind of use tile, where it has one: the PC's, out of the ROM
    /// (`archives.PC_LINES`), ahead of `USE_TILES`' own.
    pub words: IndexMap<i64, String>,
    /// Where the game keeps the sprite a move model is drawn with (`mmodel.drawn_as`).
    pub drawn: i64,
    pub streets: Option<Streets>,
    /// Zones walked about in as outdoors that are not their own parent, out of the ROM
    /// (`plates.outdoors`): a warp off a street into one of them is not a door.
    pub outdoors: HashSet<i64>,
    /// The furniture and warps of every zone with records on streets, out of the ROM: on a street
    /// the records of the other zones of the map are listed too, since South Boulevard is two
    /// zones and the one the player is not in had Café Soleil (spike 107).
    pub street_records: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)>,
    pub hidden: Option<Rc<dyn HiddenReader>>,
    pub clock: Rc<dyn Fn() -> f64>,
    /// (zone, a trigger's script) -> the way the conveyor belt it is runs (`Game.belts`).
    pub belts: IndexMap<(i64, i64), &'static str>,
    pub far: Option<FarCells>,
    pub far_floors: Option<FarFloors>,
    pub far_heights: Option<FarHeights>,
    /// Sprite -> the move that clears the thing drawn as it (`Game.clears`).
    pub clears: IndexMap<i64, i64>,
    /// (zone, cell) of every berry spot, out of the ROM (`archives.berry_spots`); none without a
    /// ROM, since no zone's data has them.
    pub berries: Vec<(i64, Cell)>,
    /// (zone, sprite) of the move models that are a barrier's beam (`Game.barriers`).
    pub barriers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) -> the category and kind it is listed as (`Game.triggers`).
    pub triggers: IndexMap<(i64, i64), (String, String)>,
    /// (zone, a furniture record's script) -> what it is, where it is no sign (`Game.furniture`):
    /// a Secret Base's PC and flag.
    pub furniture: IndexMap<(i64, i64), String>,
    pub secret_bases: Option<Rc<dyn SecretBaseReader>>,
    /// What the game calls each decoration, by its number (`GameText.decorations`).
    pub decorations: Vec<String>,
    pub ash: Option<Rc<dyn AshReader>>,
    pub doors: Option<Rc<dyn DoorReader>>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            words: IndexMap::new(),
            drawn: mmodel::DRAWN,
            streets: None,
            outdoors: HashSet::new(),
            street_records: IndexMap::new(),
            hidden: None,
            clock: Rc::new(py::time::monotonic),
            belts: IndexMap::new(),
            far: None,
            far_floors: None,
            far_heights: None,
            clears: IndexMap::new(),
            berries: Vec::new(),
            barriers: HashSet::new(),
            triggers: IndexMap::new(),
            furniture: IndexMap::new(),
            secret_bases: None,
            decorations: Vec::new(),
            ash: None,
            doors: None,
        }
    }
}

/// What `_bike_ways` was found for: the zone, the matrix's chunk files, and the chunks loaded.
type BikeKey = (i64, Rc<Vec<i64>>, Vec<(i64, i64)>);

/// Tracks the player's model and the current zone's data, and lists what is on the map.
///
/// Every holder shares the one reader, so it is made in an `Rc` (`FieldReader::new`) and its
/// methods take `&self`; the ground it hands out keeps the reader's `_far` and `_flattened` alive
/// through `me`.
pub struct FieldReader {
    me: Weak<FieldReader>,
    memory: Rc<dyn Memory>,
    /// The game's ash piles, or None in a game without them.
    ash: Option<Rc<dyn AshReader>>,
    /// The game's rotating doors, or None in a game without them.
    doors: Option<Rc<dyn DoorReader>>,
    /// What the game calls each decoration, by its number (`GameText.decorations`).
    decorations: Vec<String>,
    /// (zone, a furniture record's script) -> what it is, where it is no sign
    /// (`Game.furniture`): a Secret Base's PC and flag.
    furniture_kinds: IndexMap<(i64, i64), String>,
    /// The game's Secret Bases, or None in a game without them.
    secret_bases: Option<Rc<dyn SecretBaseReader>>,
    /// (zone, sprite) of the move models that are a barrier's beam (`Game.barriers`).
    barriers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) -> the category and kind it is listed as (`Game.triggers`).
    triggers: IndexMap<(i64, i64), (String, String)>,
    /// (zone, cell) of every berry spot, out of the ROM (`archives.berry_spots`); none without a
    /// ROM, since no zone's data has them.
    berries: Vec<(i64, Cell)>,
    /// Sprite -> the move that clears the thing drawn as it (`Game.clears`).
    clears: IndexMap<i64, i64>,
    /// A chunk file's cell table out of the ROM, by its number (`archives.MapChunks`); None where
    /// there is no ROM to read, and a chunk the game has not loaded is not known.
    far_cells: Option<FarCells>,
    far_floors: Option<FarFloors>,
    /// A chunk file's ground heights out of the ROM, by its number, which say where it drops
    /// (`archives.MapChunks.heights`).
    far_heights: Option<FarHeights>,
    /// (the chunk files, the height rounded) -> the drops found for them (`_cliffs`).
    drops: RefCell<HashMap<(Vec<i64>, Option<i64>), IndexSet<(Cell, Cell)>>>,
    /// The player's height in the world at the last poll, which says which floor of a chunk with a
    /// deck they are on (`mapblock.flat`).
    height: Slot<Option<f64>>,
    /// (id of a chunk with floors, the player's height rounded, or None for any floor) -> the
    /// chunk and the chunk flattened so: the ground is built again at every step guided.
    ///
    /// The id is the `Rc`'s address, which the chunk kept here holds fixed.
    flat: RefCell<HashMap<(usize, Option<i64>), (Rc<Chunk>, Rc<Chunk>)>>,
    /// The matrix's width in chunks, and each chunk's file, row by row, as the field map lists
    /// them (`MAP_CHUNK_FILES`).
    files: RefCell<(i64, Rc<Vec<i64>>)>,
    /// What the slopes and rail ends were found for, and what they are (`_bike_ways`).
    bike_ways_of: RefCell<(Option<BikeKey>, Vec<Interactable>)>,
    /// The chunk files the bumpy slopes' ways were found for, and the ways (`_hops`).
    hops_of: RefCell<(Option<Rc<Vec<i64>>>, IndexMap<Cell, &'static str>)>,
    /// The chunk files the floors' heights were read for, and the heights (`_decks`).
    deck_heights: RefCell<(Option<Rc<Vec<i64>>>, Vec<i64>)>,
    /// (column, row, file) -> the ROM's chunk there, once read.
    far_chunks: RefCell<HashMap<(i64, i64, i64), Option<Rc<Chunk>>>>,
    clock: Rc<dyn Fn() -> f64>,
    /// The furniture and warps of every zone with records on streets, out of the ROM: on a street
    /// the records of the other zones of the map are listed too, since South Boulevard is two
    /// zones and the one the player is not in had Café Soleil (spike 107).
    street_records: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)>,
    /// (zone, a trigger's script) -> the way the conveyor belt it is runs (`Game.belts`).
    belts: IndexMap<(i64, i64), &'static str>,
    /// A zone data file's info -> the zone it is, once looked up (`_home`).
    homes: RefCell<HashMap<Vec<u8>, i64>>,
    /// Zones walked about in as outdoors that are not their own parent, out of the ROM
    /// (`plates.outdoors`): a warp off a street into one of them is not a door.
    outdoors: HashSet<i64>,
    /// Every line and track in a map's chunks and where its joints are, out of the ROM
    /// (`archives.Streets`); None where there is no ROM to read, and only the lines the game has
    /// loaded are known, with no joint to turn at.
    streets: Option<Streets>,
    /// The chunk files the lines of the map were last read for.
    chunk_files: RefCell<Vec<i64>>,
    /// Where the world has each point and joint of the map's streets (`rail.Network`).
    places: RefCell<IndexMap<i64, (f64, f64)>>,
    /// Where the game keeps the sprite a move model is drawn with (`mmodel.drawn_as`).
    drawn: i64,
    /// The game's own word for a kind of use tile, where it has one: the PC's, out of the ROM
    /// (`archives.PC_LINES`), ahead of `USE_TILES`' own.
    words: IndexMap<i64, String>,
    models: RefCell<Vec<i64>>,
    player: Slot<Option<i64>>,
    holder: RefCell<Option<Holder>>,
    map: Slot<Option<i64>>,
    /// The zone table's base.
    table: Slot<Option<i64>>,
    /// The field map's grid as last read: a sweep ago, or when the player's model changed.
    grid: RefCell<Option<Grid>>,
    /// The game's hidden Pokémon, or None in a game without one.
    hidden: Option<Rc<dyn HiddenReader>>,
    /// The map blocks, once a sweep has found them.
    blocks: RefCell<Vec<i64>>,
    /// Every chunk's cell table read so far, by where its file sat: a block's file is read once,
    /// and again only when the block has loaded another chunk into it.
    chunks: RefCell<HashMap<mapblock::Loaded, Rc<Chunk>>>,
    /// The zone whose edges were listed whole because no block held a chunk. The blocks' class is
    /// in the field's DLL, which no sweep can look for until the runtime has read it, and a sweep
    /// before the blocks are built finds none; either way the listing at the door of the house
    /// sent the player at the trees (the replays of 2026-09-22), so a sweep is asked for.
    wanted: Slot<Option<i64>>,
    /// The zone a sweep was asked for since the last sweep, so a listing asks once per sweep and
    /// not every poll. Only a matrix with a neighbouring zone asks, so a building, whose matrix has
    /// none, never does.
    asked: Slot<Option<i64>>,
    /// Sweeps asked for the zone in `_wanted` so far. A sweep that still finds no chunk is
    /// followed by another, up to `ASKS`: the blocks may be built after the field's other objects,
    /// but a matrix whose blocks a sweep can never find must not keep the runtime sweeping every
    /// tick, and the scheduled whole sweep catches up in a few seconds.
    asks: Slot<i64>,
    /// Sweeps asked for the zone table so far (`TABLE_ASKS`).
    table_asks: Slot<i64>,
    /// When the next may be asked for.
    table_next: Slot<f64>,
    /// Sweeps asked for in the Secret Base the player is in so far (`BASE_ASKS`).
    base_asks: Slot<i64>,
    base_next: Slot<f64>,
    /// The player's model and the zone it was built in, as the last poll read them.
    visit: Slot<Option<(i64, i64)>>,
    /// The table of the rails loaded, found through the line the player was on (`rail`).
    rail_table: Slot<Option<i64>>,
    /// Every line read from it, by id, kept once read: a line's walls and the points it joins the
    /// others at never change, and the table misses a line it holds for a poll now and then as the
    /// game moves its entries about, which laid the corridor out afresh under a way being walked
    /// and lost the thing it led to (spike 105).
    lines: RefCell<IndexMap<i64, rail::Line>>,
    /// The corridor the last poll found the player in, or None off the rails.
    corridor: RefCell<Option<Rc<rail::Corridor>>>,
    /// The block holding the camera's heading, once a sweep has found it (`camera`).
    camera: Slot<Option<i64>>,
    /// The line the corridor is laid out from. Kept while it is loaded and joined to the player's,
    /// so the cells a way was found in stay where they were as the player walks from one line onto
    /// the next; laid afresh from the player's own line otherwise. Which way it runs does not
    /// matter: a way there is said from the way the player faces.
    anchor: Slot<Option<i64>>,
}

/// `a or b` for an address that is None or a number: 0 is as false as None.
fn keep(found: Option<i64>, held: Option<i64>) -> Option<i64> {
    if found.is_some_and(|address| address != 0) {
        found
    } else {
        held
    }
}

impl common::Corridor for rail::Corridor {
    fn open(&self, cell: Cell) -> Result<bool> {
        Ok(rail::Corridor::open(self, cell))
    }

    fn r#box(&self) -> Result<pathfind::Box> {
        rail::Corridor::r#box(self)
    }
}

impl FieldReader {
    /// `FieldReader(memory)`: every other argument left to its default.
    pub fn new(memory: Rc<dyn Memory>) -> Rc<FieldReader> {
        FieldReader::with(memory, Options::default())
    }

    pub fn with(memory: Rc<dyn Memory>, options: Options) -> Rc<FieldReader> {
        let Options {
            words,
            drawn,
            streets,
            outdoors,
            street_records,
            hidden,
            clock,
            belts,
            far,
            far_floors,
            far_heights,
            clears,
            berries,
            barriers,
            triggers,
            furniture,
            secret_bases,
            decorations,
            ash,
            doors,
        } = options;
        Rc::new_cyclic(|me| FieldReader {
            me: me.clone(),
            memory,
            ash,
            doors,
            decorations,
            furniture_kinds: furniture,
            secret_bases,
            barriers,
            triggers,
            berries,
            clears,
            far_cells: far,
            far_floors,
            far_heights,
            drops: RefCell::new(HashMap::new()),
            height: Slot::new(None),
            flat: RefCell::new(HashMap::new()),
            files: RefCell::new((0, Rc::new(Vec::new()))),
            bike_ways_of: RefCell::new((None, Vec::new())),
            hops_of: RefCell::new((None, IndexMap::new())),
            deck_heights: RefCell::new((None, Vec::new())),
            far_chunks: RefCell::new(HashMap::new()),
            clock,
            street_records,
            belts,
            homes: RefCell::new(HashMap::new()),
            outdoors,
            streets,
            chunk_files: RefCell::new(Vec::new()),
            places: RefCell::new(IndexMap::new()),
            drawn,
            words,
            models: RefCell::new(Vec::new()),
            player: Slot::new(None),
            holder: RefCell::new(None),
            map: Slot::new(None),
            table: Slot::new(None),
            grid: RefCell::new(None),
            hidden,
            blocks: RefCell::new(Vec::new()),
            chunks: RefCell::new(HashMap::new()),
            wanted: Slot::new(None),
            asked: Slot::new(None),
            asks: Slot::new(0),
            table_asks: Slot::new(0),
            table_next: Slot::new(0.0),
            base_asks: Slot::new(0),
            base_next: Slot::new(0.0),
            visit: Slot::new(None),
            rail_table: Slot::new(None),
            lines: RefCell::new(IndexMap::new()),
            corridor: RefCell::new(None),
            camera: Slot::new(None),
            anchor: Slot::new(None),
        })
    }

    /// Take in what a sweep found: the model pool, the zone data and its holder, the field map and
    /// the zone table.
    pub fn track(&self, found: &Found) -> Result<()> {
        let mut models = found.of(&[MOVE_MODEL_CLASS]);
        if !models.is_empty() {
            models.sort();
            *self.models.borrow_mut() = models;
        }
        let mut files: IndexMap<i64, ZoneData> = IndexMap::new();
        for data in zone_files(&found.pages, &*self.memory)? {
            files.insert(data.address, data);
        }
        if !files.is_empty() {
            if let Some(holder) = self.find_holder(found, &files)? {
                *self.holder.borrow_mut() = Some(holder);
            }
            if let Some(holder) = self.holder.borrow_mut().as_mut() {
                holder.files.extend(files);
            }
        }
        self.map
            .set(keep(self.find_map(&found.pages)?, self.map.get()));
        self.read_map()?;
        if self.table.get().is_none() && self.holder.borrow().is_some() {
            self.table.set(self.find_table(found)?);
        }
        if let Some(hidden) = &self.hidden {
            hidden.track(found)?;
        }
        if let Some(secret_bases) = &self.secret_bases {
            secret_bases.track(found)?;
        }
        if let Some(ash) = &self.ash {
            ash.track(found)?;
        }
        if let Some(doors) = &self.doors {
            doors.track(found)?;
        }
        self.camera
            .set(keep(camera::find(found, &*self.memory)?, self.camera.get()));
        if let Some(player) = self.find_player()? {
            let head = self.memory.read(player, mmodel::HEAD)?;
            if mmodel::on_rail(&head)? {
                let line = mmodel::rail(&head)?.0;
                self.rail_table.set(keep(
                    rail::find_table(&found.pages, &*self.memory, line)?,
                    self.rail_table.get(),
                ));
            }
        }
        let blocks = found.of(&[mapblock::CLASS]);
        if !blocks.is_empty() && found.partial {
            // A sweep of the blocks just allocated finds the new ones alone; the rest are kept
            // while they still carry a map block's vtable.
            let vtable = u32(&self.memory.read(blocks[0], 4)?, 0)?;
            let held = self.blocks.borrow().clone();
            let mut kept: Vec<i64> = Vec::new();
            for block in held {
                if u32(&self.memory.read(block, 4)?, 0)? == vtable {
                    kept.push(block);
                }
            }
            kept.extend(blocks);
            kept.sort();
            kept.dedup();
            *self.blocks.borrow_mut() = kept;
        } else if !blocks.is_empty() {
            let mut sorted = blocks;
            sorted.sort();
            *self.blocks.borrow_mut() = sorted;
        }
        if !found.partial {
            // A sweep has been; if it still found no chunk, the next listing may ask for another.
            self.asked.set(None);
        }
        Ok(())
    }

    /// The player's zone and cell, or None while nothing says where they are: before the first
    /// sweep, and on the title screen, where the pool holds the save's models with no zone data
    /// behind them.
    pub fn poll(&self) -> Result<Option<Field>> {
        let Some(player) = self.find_player()? else {
            return Ok(None);
        };
        if self.current()?.is_none() {
            return Ok(None);
        }
        let head = self.memory.read(player, mmodel::HEAD)?;
        let Some(cell) = mmodel::cell(&head)? else {
            return Ok(None);
        };
        self.visit.set(Some((player, mmodel::zone_of(&head)?)));
        self.height.set(mmodel::height(&head)?);
        let zone = match self.zone_at(cell)? {
            Some(zone) => zone,
            None => mmodel::zone_of(&head)?,
        };
        *self.corridor.borrow_mut() = None;
        let bike = mmodel::on_bike(&head)?;
        if !mmodel::on_rail(&head)? {
            return Ok(Some(Field {
                on_bike: bike,
                ..Field::new(zone, cell)
            }));
        }
        let (line, a, x) = mmodel::rail(&head)?;
        let corridor = self.read_corridor(line)?;
        if let Some(corridor) = &corridor {
            self.anchor.set(Some(corridor.laid[0].line.id));
        }
        let at = match &corridor {
            None => None,
            Some(corridor) => corridor.at(line, a, x)?,
        };
        let (Some(corridor), Some(at)) = (corridor, at) else {
            return Ok(Some(Field {
                on_rail: true,
                on_bike: bike,
                ..Field::new(zone, cell)
            }));
        };
        *self.corridor.borrow_mut() = Some(corridor.clone());
        let bearing = self.bearing(&corridor, line, &head)?;
        let heading = match bearing {
            None => None,
            Some(bearing) => Some(pathfind::nearest(bearing)?),
        };
        Ok(Some(Field {
            on_rail: true,
            heading,
            bearing,
            on_bike: bike,
            ..Field::new(zone, at)
        }))
    }

    /// Where up walks the player in the corridor: the camera's heading, turned from the world into
    /// the line's frame by the way the line runs where they stand; the way they face while no
    /// camera has been found, which is the same until left or right is tapped or down pressed.
    fn bearing(&self, corridor: &rail::Corridor, line: i64, head: &[u8]) -> Result<Option<f64>> {
        let Some(on) = corridor.line(line) else {
            return Ok(None);
        };
        let world = match self.camera.get() {
            None => None,
            Some(block) => camera::heading(&*self.memory, block)?,
        };
        let (across, along) = match world {
            None => {
                let (x, a) = mmodel::rail_facing(head)?;
                let (along, across) = on.direction(a, x);
                (across, along)
            }
            Some(world) => {
                let toward = (world - mmodel::rail_way(head)?).to_radians();
                let (across, along) = (toward.sin(), toward.cos());
                if on.track {
                    (-across, -along)
                } else {
                    (across, along)
                }
            }
        };
        Ok(corridor.bearing(line, across, along))
    }

    /// The streets joined to `line`, laid out flat, or None while the line is not known: every
    /// line and track in the map's chunks where the ROM can be read, and the lines the game has
    /// loaded.
    fn read_corridor(&self, line: i64) -> Result<Option<Rc<rail::Corridor>>> {
        self.read_streets()?;
        if let Some(table) = self.rail_table.get() {
            let read = rail::read_table(&*self.memory, table, &self.lines.borrow())?;
            self.lines.borrow_mut().extend(read);
        }
        if !self.lines.borrow().contains_key(&line) {
            return Ok(None);
        }
        if let Some(anchor) = self.anchor.get() {
            let kept = rail::lay_out(&self.lines.borrow(), anchor, Some(&self.places.borrow()))?;
            if let Some(kept) = kept
                && kept.holds(line)
            {
                return Ok(Some(Rc::new(kept)));
            }
        }
        Ok(rail::lay_out(&self.lines.borrow(), line, Some(&self.places.borrow()))?.map(Rc::new))
    }

    /// Take in every line of the map's streets from the ROM, once a map: the game loads a line
    /// only with the chunks round the player, and a door on a line further on was nowhere, the
    /// Pokémon Lab's among them (spike 106). A map numbers its lines and points afresh, the same id
    /// in two maps two different things (spike 107), so another map's are let go.
    fn read_streets(&self) -> Result<()> {
        let (_, files) = self.read_files()?;
        if files.is_empty() || files == *self.chunk_files.borrow() {
            return Ok(());
        }
        *self.chunk_files.borrow_mut() = files.clone();
        *self.lines.borrow_mut() = IndexMap::new();
        *self.places.borrow_mut() = IndexMap::new();
        self.anchor.set(None);
        if let Some(streets) = &self.streets {
            let wanted: Vec<i64> = files.iter().copied().filter(|&file| file != 0).collect();
            let network = streets(&wanted)?;
            self.lines.borrow_mut().extend(network.lines);
            self.places.borrow_mut().extend(network.places);
        }
        Ok(())
    }

    /// The player's model and the zone it was built in, as the last poll read them.
    ///
    /// Every warp builds the player a new model in the zone it arrives in, and walking across an
    /// edge keeps the old one (spike 78), so this changes at a warp and not at an edge. It may also
    /// change where the game rebuilds the field around a player standing still.
    pub fn visit(&self) -> Option<(i64, i64)> {
        self.visit.get()
    }

    /// Where the player can walk around `where`, with its listing's people standing in their
    /// cells when it has one.
    ///
    /// A matrix that puts no zone at the player's own cell is not trusted to say where the map
    /// ends: the grid was read 32 blocks wide on a matrix 40 wide until 2026-09-23, and put no
    /// zone under the player at all.
    ///
    /// A chunk with a deck over its ground is walked on the floor nearest the player's height, cell
    /// by cell (`mapblock.flat`, spike 156), or nearest `height` where one is given: one of the
    /// ground's `decks`, every other floor's height on the matrix.
    pub fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground> {
        if r#where.on_rail {
            let corridor = self
                .corridor
                .borrow()
                .clone()
                .map(|corridor| corridor as Rc<dyn common::Corridor>);
            return Ok(Ground::of(
                r#where,
                Of {
                    corridor,
                    ..Of::default()
                },
            ));
        }
        let solid: Vec<Cell> = match &self.secret_bases {
            Some(bases) if bases.zones().contains(&r#where.zone) => {
                bases.solid()?.into_iter().collect()
            }
            _ => Vec::new(),
        };
        let level = height.or(self.height.get());
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.flattened(&chunk, level, true)?);
        }
        let decks = self.decks(level)?;
        let cliffs = self.cliffs(level)?;
        let doors = match &self.doors {
            None => Vec::new(),
            Some(doors) => doors.doors(r#where.zone)?,
        };
        let hops = self.hops()?;
        if self.zone_at(r#where.cell)?.is_none() {
            let belts = self.belts_here(r#where.zone)?;
            return Ok(Ground {
                decks,
                cliffs,
                doors,
                hops,
                ..Ground::of(
                    r#where,
                    Of {
                        chunks,
                        belts,
                        clears: self.clears.clone(),
                        solid,
                        ..Of::default()
                    },
                )
            });
        }
        let me = self
            .me
            .upgrade()
            .expect("a reader is made in an Rc, which its holders keep");
        let far: crate::games::common::field::Far =
            Rc::new(move |column, row| match me.far(column, row)? {
                None => Ok(None),
                Some(chunk) => Ok(Some(me.flattened(&chunk, level, true)?)),
            });
        let grid = self.grid.borrow().clone();
        let belts = self.belts_here(r#where.zone)?;
        Ok(Ground {
            decks,
            cliffs,
            doors,
            hops,
            ..Ground::of(
                r#where,
                Of {
                    chunks,
                    grid,
                    belts,
                    far: if self.far_cells.is_none() {
                        None
                    } else {
                        Some(far)
                    },
                    clears: self.clears.clone(),
                    solid,
                    ..Of::default()
                },
            )
        })
    }

    /// Open cells side by side, each way, a drop apart (`mapblock.DROP`), but for the zone's
    /// ladders: Fortree City's platforms stand 108 over the ground beside them in one cell table,
    /// and P walked the player off one, "9 south" into nothing, where the game lets them down only
    /// by a ladder (spike 160). Read off the ROM's chunks of one floor across the whole matrix, a
    /// chunk with a deck on the floor each cell is walked on at `level`, the height the decks are
    /// walked at; none without the ROM.
    fn cliffs(&self, level: Option<f64>) -> Result<IndexSet<(Cell, Cell)>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let key = (
            (*files).clone(),
            match level {
                None => None,
                Some(level) => Some(py::round(level)?),
            },
        );
        let held = self.drops.borrow().get(&key).cloned();
        let drops = match held {
            Some(drops) => drops,
            None => {
                // ponytail: emptied whole when full; a map is walked at a few heights.
                if self.drops.borrow().len() as i64 > FLATTENED {
                    self.drops.borrow_mut().clear();
                }
                let found = self.find_drops(wide, &files, level)?;
                self.drops.borrow_mut().insert(key, found.clone());
                found
            }
        };
        let data = self.current()?;
        let Some(data) = data.filter(|_| !drops.is_empty()) else {
            return Ok(drops);
        };
        let ladders = ladders(&triggers(&*self.memory, &data)?)?;
        Ok(drops
            .into_iter()
            .filter(|pair| !ladders.contains(pair))
            .collect())
    }

    /// Each bumpy slope on the matrix with the way it is hopped up: between the two open cells on
    /// its opposite sides, towards the higher, as the ROM's heights have them. Every one of Jagged
    /// Pass's 24 has one such pair, a cell's height apart, north for most and west for three (spike
    /// 159). None without the ROM's heights.
    fn hops(&self) -> Result<IndexMap<Cell, &'static str>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        if self.hops_of.borrow().0.as_ref() == Some(&files) {
            return Ok(self.hops_of.borrow().1.clone());
        }
        let mut out: IndexMap<Cell, &'static str> = IndexMap::new();
        let mut tables: IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)> = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let chunk = self.far(py::modulo(at, wide), py::floordiv(at, wide))?;
            let heights = match &self.far_heights {
                None => None,
                Some(far_heights) => far_heights(file)?,
            };
            if let (Some(chunk), Some(heights)) = (chunk, heights) {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }

        let read = |cell: Cell| -> Result<Option<(i64, Option<f64>)>> {
            let table = tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)));
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let Some(table) = table else {
                return Ok(None);
            };
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            let height = *usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            Ok(Some((word, height)))
        };

        for (&(column, row), (words, _)) in &tables {
            for (i, &word) in words.iter().enumerate() {
                let i = i as i64;
                if !BUMPY_SLOPES.contains(&(word >> KIND_SHIFT)) {
                    continue;
                }
                let cell = (column * CHUNK + i % CHUNK, row * CHUNK + i / CHUNK);
                for (one, other) in [(NORTH, SOUTH), (WEST, EAST)] {
                    let (first, second) = {
                        let first = step_of(one);
                        let first = read((cell.0 + first.0, cell.1 + first.1))?;
                        let second = step_of(other);
                        let second = read((cell.0 + second.0, cell.1 + second.1))?;
                        (first, second)
                    };
                    let (Some(first), Some(second)) = (first, second) else {
                        continue;
                    };
                    if (first.0 | second.0) & BLOCKED != 0 {
                        continue;
                    }
                    if let (Some(a), Some(b)) = (first.1, second.1)
                        && a != b
                    {
                        out.insert(cell, if a > b { one } else { other });
                    }
                }
            }
        }
        *self.hops_of.borrow_mut() = (Some(files), out);
        Ok(self.hops_of.borrow().1.clone())
    }

    /// Every drop on the matrix with its decks walked at `level` (`_cliffs`).
    fn find_drops(
        &self,
        wide: i64,
        files: &[i64],
        level: Option<f64>,
    ) -> Result<IndexSet<(Cell, Cell)>> {
        if self.far_heights.is_none() || wide == 0 {
            return Ok(IndexSet::new());
        }
        let mut tables: IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)> = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let Some(chunk) = self.far(py::modulo(at, wide), py::floordiv(at, wide))? else {
                continue;
            };
            if !chunk.floors.is_empty() {
                let flat = self.flattened(&chunk, level, true)?;
                let heights = if flat.heights.is_empty() {
                    chunk.floors[0].heights.clone()
                } else {
                    flat.heights.clone()
                };
                tables.insert(chunk.key(), (flat.words.clone(), heights));
            } else if let Some(heights) = self.far_heights.as_ref().map_or(Ok(None), |f| f(file))? {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }

        let height = |cell: Cell| -> Result<Option<f64>> {
            let Some(table) =
                tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)))
            else {
                return Ok(None);
            };
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            if word & BLOCKED != 0 {
                return Ok(None);
            }
            Ok(*usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?)
        };

        let mut out: IndexSet<(Cell, Cell)> = IndexSet::new();
        for &(column, row) in tables.keys() {
            for z in row * CHUNK..(row + 1) * CHUNK {
                for x in column * CHUNK..(column + 1) * CHUNK {
                    let Some(here) = height((x, z))? else {
                        continue;
                    };
                    for near in [(x + 1, z), (x, z + 1)] {
                        let there = height(near)?;
                        if let Some(there) = there
                            && (here - there).abs() > mapblock::DROP
                        {
                            out.insert(((x, z), near));
                            out.insert((near, (x, z)));
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    /// Every floor's height on the matrix but the one at `level`, nearest it first: each floor of
    /// a chunk with a deck, at the middle of its open cells' heights: the loaded chunks', and with
    /// the ROM every chunk's of the matrix (spike 159).
    ///
    /// The heights are whole numbers, as `round` makes them, and so are the ground's `decks`.
    fn decks(&self, level: Option<f64>) -> Result<Vec<f64>> {
        let Some(level) = level else {
            return Ok(Vec::new());
        };
        let files = self.files.borrow().1.clone();
        if self.deck_heights.borrow().0.as_ref() != Some(&files) {
            let mut floors: Vec<Floor> = Vec::new();
            for chunk in self.chunks_loaded()? {
                floors.extend(chunk.floors.iter().cloned());
            }
            if let Some(far_floors) = &self.far_floors {
                for file in without(&PySet::of(files.iter().copied()), NO_CHUNK_FILE) {
                    floors.extend(far_floors(file)?);
                }
            }
            let mut heights: Vec<i64> = Vec::new();
            for floor in &floors {
                if floor.words.len() != floor.heights.len() {
                    return Err(Error::value(format!(
                        "zip() argument 2 is {} than argument 1",
                        if floor.heights.len() < floor.words.len() {
                            "shorter"
                        } else {
                            "longer"
                        }
                    )));
                }
                let mut open: Vec<f64> = Vec::new();
                for (&word, &height) in floor.words.iter().zip(&floor.heights) {
                    if let Some(height) = height
                        && word & BLOCKED == 0
                    {
                        open.push(height);
                    }
                }
                if !open.is_empty() {
                    heights.push(py::round(median(&mut open))?);
                }
            }
            heights.sort();
            heights.dedup();
            *self.deck_heights.borrow_mut() = (Some(files), heights);
        }
        let mut others: Vec<i64> = self
            .deck_heights
            .borrow()
            .1
            .iter()
            .copied()
            .filter(|&height| (height as f64 - level).abs() > mapblock::CLIMB)
            .collect();
        others.sort_by(|a, b| {
            let (a, b) = ((*a as f64 - level).abs(), (*b as f64 - level).abs());
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(others.into_iter().map(|height| height as f64).collect())
    }

    /// The chunk on the floor nearest `height` cell by cell, or on any floor open where
    /// `at_height` is False (`mapblock.flat`), made once a height.
    fn flattened(
        &self,
        chunk: &Rc<Chunk>,
        height: Option<f64>,
        at_height: bool,
    ) -> Result<Rc<Chunk>> {
        if chunk.floors.is_empty() || (at_height && height.is_none()) {
            return Ok(chunk.clone());
        }
        let rounded = match height {
            Some(height) if at_height => Some(py::round(height)?),
            _ => None,
        };
        let key = (Rc::as_ptr(chunk) as usize, rounded);
        if !self.flat.borrow().contains_key(&key) {
            // ponytail: emptied whole when full; a few chunks at a few heights is all a map holds.
            if self.flat.borrow().len() as i64 > FLATTENED {
                self.flat.borrow_mut().clear();
            }
            let flat = mapblock::flat(chunk, rounded.map(|height| height as f64));
            self.flat
                .borrow_mut()
                .insert(key, (chunk.clone(), Rc::new(flat)));
        }
        Ok(self.flat.borrow()[&key].1.clone())
    }

    /// The cell the player's Secret Base's decorating cursor is on and what is there, or None while
    /// it is not shown: the decorations standing in it, by the game's names for them, or the PC or
    /// the flag, or a wall, or blank. The game names a decoration on the bottom screen only, says
    /// "Select location" alike over the floor, the PC and the walls, and nothing at all moving from
    /// one empty cell to the next (omega_ruby_secret-base2.ctm).
    pub fn decorating(&self, r#where: &Field) -> Result<Option<(Cell, String)>> {
        let cell = match &self.secret_bases {
            Some(bases) if bases.zones().contains(&r#where.zone) => bases.cursor()?,
            _ => None,
        };
        let (Some(bases), Some(cell)) = (&self.secret_bases, cell) else {
            return Ok(None);
        };
        let mut names: Vec<&str> = Vec::new();
        let standing = bases.standing(cell)?;
        for number in standing {
            if number < self.decorations.len() as i64
                && !self.decorations[number as usize].is_empty()
            {
                names.push(&self.decorations[number as usize]);
            }
        }
        if !names.is_empty() {
            return Ok(Some((cell, names.join(", "))));
        }
        let data = self.current()?;
        let home = match &data {
            None => None,
            Some(data) => self.home(data, &[r#where.zone])?,
        };
        let items = match &data {
            None => Vec::new(),
            Some(data) => furniture(&*self.memory, data)?,
        };
        for item in items {
            if item.cells().contains(&cell) {
                return Ok(Some((cell, self.furniture_kind(home, &item))));
            }
        }
        let wall = walkable(&self.chunks_loaded()?, cell)? == Some(false);
        Ok(Some((cell, if wall { WALL } else { BLANK }.to_string())))
    }

    /// The cells of the zone's conveyor belts, each with the way it runs: its triggers whose
    /// scripts `Game.belts` knows as belts.
    fn belts_here(&self, zone: i64) -> Result<IndexMap<Cell, &'static str>> {
        let data = self.current()?;
        let Some(data) = data.filter(|_| !self.belts.is_empty()) else {
            return Ok(IndexMap::new());
        };
        let Some(home) = self.home(&data, &[zone])? else {
            return Ok(IndexMap::new());
        };
        let mut out: IndexMap<Cell, &'static str> = IndexMap::new();
        for record in triggers(&*self.memory, &data)? {
            let Some(&way) = self.belts.get(&(home, u16(&record, TRIGGER_SCRIPT)?)) else {
                continue;
            };
            let (x, z) = (u16(&record, TRIGGER_X)?, u16(&record, TRIGGER_Z)?);
            for dx in 0..u16(&record, TRIGGER_WIDTH)? {
                for dz in 0..u16(&record, TRIGGER_HEIGHT)? {
                    out.insert((x + dx, z + dz), way);
                }
            }
        }
        Ok(out)
    }

    /// The zone's triggers `Game.triggers` knows as something to walk to: Mauville Gym's switches,
    /// and the way between its two rooms, which share no floor (spike 146); Lavaridge Gym's
    /// trapdoors and geysers (spike 157).
    fn listed_triggers(&self, data: &ZoneData, zone: i64) -> Result<Vec<Interactable>> {
        let home = if self.triggers.is_empty() {
            None
        } else {
            self.home(data, &[zone])?
        };
        let Some(home) = home else {
            return Ok(Vec::new());
        };
        let mut out: Vec<Interactable> = Vec::new();
        for (i, record) in triggers(&*self.memory, data)?.iter().enumerate() {
            let script = u16(record, TRIGGER_SCRIPT)?;
            if let Some((category, kind)) = self.triggers.get(&(home, script)) {
                out.push(Interactable {
                    size: (u16(record, TRIGGER_WIDTH)?, u16(record, TRIGGER_HEIGHT)?),
                    code: script,
                    source: place_source(TRIGGER_SOURCE, i as i64)?,
                    home: Some(home),
                    ..Interactable::new(
                        category,
                        kind,
                        (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                    )
                });
            }
        }
        Ok(out)
    }

    /// `where` with everything worth walking to in its zone listed.
    pub fn interactables(&self, r#where: &Field) -> Result<Field> {
        let corridor = self.corridor.borrow().clone();
        if let Some(corridor) = corridor.filter(|_| r#where.on_rail) {
            return Ok(Field {
                interactables: self.on_the_rails(&corridor, r#where.zone)?,
                ..r#where.clone()
            });
        }
        let mut items: Vec<Interactable> = Vec::new();
        let data = self.current()?;
        if let Some(data) = &data {
            let home = self.home(data, &[r#where.zone])?;
            for item in furniture(&*self.memory, data)? {
                let kind = self.furniture_kind(home, &item);
                items.push(Interactable { kind, home, ..item });
            }
            for (i, record) in records_in(&*self.memory, data, WARP)?.iter().enumerate() {
                if !on_rail(record)? {
                    let warp = self.warp(i as i64, record, None)?;
                    items.push(Interactable { home, ..warp });
                }
            }
        }
        items.extend(self.people(r#where.zone)?);
        items.extend(self.edges(r#where.zone, !r#where.on_rail)?);
        items.extend(self.use_tiles(r#where.zone)?);
        items.extend(self.bike_ways(r#where.zone)?);
        if let Some(data) = &data {
            items.extend(self.listed_triggers(data, r#where.zone)?);
        }
        for (index, &(zone, cell)) in self.berries.iter().enumerate() {
            if zone == r#where.zone {
                items.push(Interactable {
                    source: place_source(BERRY_SOURCE, index as i64)?,
                    home: Some(zone),
                    ..Interactable::new(BERRIES, BERRY_SPOT, cell)
                });
            }
        }
        if let Some(ash) = &self.ash {
            for (index, cell) in ash.piles(r#where.zone)? {
                items.push(Interactable {
                    source: place_source(ASH_SOURCE, index)?,
                    home: Some(r#where.zone),
                    ..Interactable::new(ASH, ASH_PILE, cell)
                });
            }
        }
        items.extend(self.spots(r#where.zone)?);
        if let Some(hiding) = self.hidden(r#where.zone)? {
            items.push(hiding);
        }
        Ok(Field {
            interactables: items,
            ..r#where.clone()
        })
    }

    /// What a furniture record is: a sign, unless the game's table of them says otherwise. The PC
    /// in a Secret Base is called what the game calls a PC, as the Pokémon Center's is.
    fn furniture_kind(&self, home: Option<i64>, item: &Interactable) -> String {
        let kind = match home {
            Some(home) => self.furniture_kinds.get(&(home, item.code)),
            None => None,
        };
        let Some(kind) = kind else {
            return item.kind.clone();
        };
        if kind == "PC" {
            return match self.words.get(&PC_TILE) {
                Some(word) if !word.is_empty() => word.clone(),
                _ => kind.clone(),
            };
        }
        kind.clone()
    }

    /// The zone's Secret Base spots: the player's own base as its entrance, leading inside, and
    /// every other spot as a spot (spike 154). A spot is a tree, a wall or a bush, walked up to and
    /// faced as a sign is.
    fn spots(&self, zone: i64) -> Result<Vec<Interactable>> {
        let Some(reader) = &self.secret_bases else {
            return Ok(Vec::new());
        };
        let own = reader.own()?;
        let mut out: Vec<Interactable> = Vec::new();
        for (number, (home, cell)) in reader.spots().into_iter().enumerate() {
            let number = number as i64;
            if home != zone {
                continue;
            }
            out.push(if Some(number) == own {
                Interactable {
                    zone: Some(reader.inside()),
                    source: place_source(SECRET_BASE_SOURCE, number)?,
                    home: Some(home),
                    ..Interactable::new("exits", ENTRANCE, cell)
                }
            } else {
                Interactable {
                    source: place_source(SECRET_BASE_SOURCE, number)?,
                    home: Some(home),
                    ..Interactable::new("exits", SECRET_BASE_SPOT, cell)
                }
            });
        }
        Ok(out)
    }

    /// Why to sweep now rather than on schedule, or None.
    ///
    /// A zone's edges were listed whole because no block held a chunk, no sweep has been asked for
    /// since the last one, and fewer than `ASKS` have been asked for the zone. The player's own
    /// chunk is always loaded, so no chunk at all means the blocks are not known yet, or moved;
    /// only a sweep finds them.
    ///
    /// Or the zone data is in and the zone table is not, which names every place and tells a path
    /// from stairs, and only a sweep finds it (`TABLE_ASKS`).
    ///
    /// Nothing while the field is not the app `running`: a battle tears the map blocks down where
    /// they are and builds them again there, which the field reads as no chunk, and all 72 asks
    /// over the recordings of ORAS and X came in another app, 32 whole sweeps in movie4 (spike
    /// 130). The zone's asks are spent instead, so its want is not swept for the moment the field
    /// comes back.
    pub fn sweep_wanted(&self, running: bool) -> Result<Option<&'static str>> {
        if !running {
            self.asks.set(ASKS);
            return Ok(None);
        }
        if self.wanted.get().is_some()
            && self.wanted.get() != self.asked.get()
            && self.asks.get() < ASKS
        {
            self.asked.set(self.wanted.get());
            self.asks.set(self.asks.get() + 1);
            return Ok(Some("no chunk loaded, sweeping for the map blocks"));
        }
        if self.table.get().is_none()
            && self.holder.borrow().is_some()
            && self.table_asks.get() < TABLE_ASKS
        {
            let now = (self.clock)();
            if now >= self.table_next.get() {
                self.table_next.set(now + TABLE_EVERY);
                self.table_asks.set(self.table_asks.get() + 1);
                return Ok(Some("no zone table, sweeping for it"));
            }
        }
        let in_a_base = match (&self.secret_bases, self.visit.get()) {
            (Some(bases), Some(visit)) => bases.zones().contains(&visit.1),
            _ => false,
        };
        if !in_a_base {
            self.base_asks.set(0);
        } else if let Some(bases) = &self.secret_bases
            && !bases.furnished()?
            && self.base_asks.get() < BASE_ASKS
        {
            let now = (self.clock)();
            if now >= self.base_next.get() {
                self.base_next.set(now + BASE_EVERY);
                self.base_asks.set(self.base_asks.get() + 1);
                return Ok(Some(
                    "in a Secret Base with nothing found standing in it, sweeping for it",
                ));
            }
        }
        Ok(None)
    }

    /// The record's words as they are, with the zone the matrix puts its cell in, for the log; None
    /// before a sweep has found the record, and in a game without one.
    pub fn hidden_words(&self) -> Result<Option<String>> {
        let held = match &self.hidden {
            None => None,
            Some(hidden) => hidden.words()?,
        };
        let Some((words, cell)) = held else {
            return Ok(None);
        };
        let zone = self.zone_at(cell)?;
        let zone = match zone {
            None => "none".to_string(),
            Some(zone) => format!("{zone:#x}"),
        };
        Ok(Some(format!("{words} zone {zone}")))
    }

    /// The DexNav's hidden Pokémon, while one is up in `zone`: one read of its record, cheap enough
    /// for every poll, since it comes and goes between sweeps.
    ///
    /// The record outlives its zone: walking onto Route 102 listed Route 101's Poochyena at 105
    /// west until the next find replaced it (spike 80's replay). So one is listed only where the
    /// matrix puts its cell in the player's own zone.
    pub fn hidden(&self, zone: i64) -> Result<Option<Interactable>> {
        let found = match &self.hidden {
            None => None,
            Some(hidden) => hidden.read()?,
        };
        let Some(found) = found else {
            return Ok(None);
        };
        if self.zone_at(found.cell)? != Some(zone) {
            return Ok(None);
        }
        Ok(Some(Interactable {
            code: found.species,
            source: found.record,
            ..Interactable::new("pokémon", HIDDEN_POKEMON, found.cell)
        }))
    }

    /// The zone's place-name index, or None without the zone table: the same for a town and every
    /// building in it, so it changes only when the player arrives somewhere.
    pub fn place_index(&self, zone: i64) -> Result<Option<i64>> {
        match self.zone_row(zone)? {
            None => Ok(None),
            Some(row) => Ok(Some(u16(&row, INFO_NAME)? & NAME_MASK)),
        }
    }

    /// A zone's row of the zone table: its parent zone at +0x1a, and at +0x1c a u16 whose low ten
    /// bits are its place-name index, for naming exits once names come from the ROM (spike 85; the
    /// bits above are flags, and 0x400 marks the zone that carries the place's name).
    pub fn zone_row(&self, zone: i64) -> Result<Option<Vec<u8>>> {
        let Some(table) = self.table.get() else {
            return Ok(None);
        };
        Ok(Some(self.memory.read(table + zone * INFO_SIZE, INFO_SIZE)?))
    }

    /// Which zone a zone data file is: the zone whose row of the table its info is a copy of, every
    /// row different from every other in both games (spike 107). The matrix can put the player in
    /// the next zone a poll before its data is in, and a door of the town was named for the door of
    /// the next zone's with its number. The zones `likely` and the player's model's are looked at
    /// first; None without the zone table.
    fn home(&self, data: &ZoneData, likely: &[i64]) -> Result<Option<i64>> {
        let Some(table) = self.table.get() else {
            return Ok(None);
        };
        let info = self.memory.read(data.info(), INFO_SIZE)?;
        if let Some(&zone) = self.homes.borrow().get(&info) {
            return Ok(Some(zone));
        }
        let mut zones: Vec<i64> = likely.to_vec();
        if let Some(visit) = self.visit.get() {
            zones.push(visit.1);
        }
        for zone in zones {
            if self.zone_row(zone)?.as_ref() == Some(&info) {
                self.homes.borrow_mut().insert(info, zone);
                return Ok(Some(zone));
            }
        }
        let rows = self.memory.read(table, ZONE_LIMIT * INFO_SIZE)?;
        for zone in 0..ZONE_LIMIT {
            if py::slice(&rows, zone * INFO_SIZE, (zone + 1) * INFO_SIZE) == info.as_slice() {
                self.homes.borrow_mut().insert(info, zone);
                return Ok(Some(zone));
            }
        }
        Ok(None)
    }

    /// The zone's matrix, or None without the zone table or for `NO_MATRIX`.
    fn matrix(&self, zone: i64) -> Result<Option<i64>> {
        let matrix = match self.zone_row(zone)? {
            None => None,
            Some(row) => Some(u16(&row, INFO_MATRIX)?),
        };
        Ok(if matrix == Some(NO_MATRIX) {
            None
        } else {
            matrix
        })
    }

    /// Whether a zone is one of the outdoors, or None without the zone table: its own parent, or on
    /// its parent's matrix, as the foot of Route 119 is on its top's. Taking only the first, the
    /// way from Route 118 onto Route 119 was said as stairs (spike 159).
    pub fn outdoors(&self, zone: i64) -> Result<Option<bool>> {
        let Some(row) = self.zone_row(zone)? else {
            return Ok(None);
        };
        let parent = u16(&row, INFO_PARENT)?;
        let matrix = self.matrix(zone)?;
        if parent == zone {
            return Ok(Some(true));
        }
        Ok(Some(match matrix {
            None => false,
            Some(matrix) => self.matrix(parent)? == Some(matrix),
        }))
    }

    /// Whether two zones are on one matrix, walked between with no warp: Route 119's foot and its
    /// top, which the game calls by one name (spike 159).
    pub fn one_matrix(&self, here: i64, there: i64) -> Result<bool> {
        let matrix = self.matrix(here)?;
        Ok(match matrix {
            None => false,
            Some(matrix) => Some(matrix) == self.matrix(there)?,
        })
    }

    /// What is on the corridor's lines: the signs and warps placed on them, the zone's own and
    /// those of the other zones of its map out of the ROM, and the people standing on them, in the
    /// corridor's cells. Edges and the cells used by facing them are the grid's, and the chunks
    /// under the rails mark every cell a wall, so neither is.
    fn on_the_rails(&self, corridor: &rail::Corridor, zone: i64) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        let mut tables: Vec<(Option<i64>, Vec<Vec<u8>>, Vec<Vec<u8>>)> = Vec::new();
        let data = self.current()?;
        let home = match &data {
            None => None,
            Some(data) => self.home(data, &[zone])?,
        };
        if let Some(data) = &data {
            tables.push((
                home,
                records_in(&*self.memory, data, FURNITURE)?,
                records_in(&*self.memory, data, WARP)?,
            ));
        }
        let matrix = self.matrix(home.unwrap_or(zone))?;
        let mut others: Vec<(&i64, &(Vec<Vec<u8>>, Vec<Vec<u8>>))> =
            self.street_records.iter().collect();
        others.sort_by_key(|(other, _)| **other);
        for (&other, (furniture, warps)) in others {
            if Some(other) != home
                && let Some(matrix) = matrix
                && self.matrix(other)? == Some(matrix)
            {
                tables.push((Some(other), furniture.clone(), warps.clone()));
            }
        }
        for (owner, furniture, warps) in tables {
            for (i, record) in furniture.iter().enumerate() {
                if let Some(at) = placed(corridor, record)? {
                    out.push(Interactable {
                        size: at.1,
                        code: u16(record, 0)?,
                        source: place_source(FURNITURE_SOURCE, i as i64)?,
                        home: owner,
                        ..Interactable::new("signs", "sign", at.0)
                    });
                }
            }
            for (i, record) in warps.iter().enumerate() {
                if let Some(at) = placed(corridor, record)? {
                    let warp = self.warp(i as i64, record, Some(at))?;
                    out.push(Interactable {
                        home: owner,
                        ..warp
                    });
                }
            }
        }
        let models = self.models.borrow().clone();
        for model in models {
            let head = self.memory.read(model, mmodel::HEAD)?;
            if !mmodel::in_use(&head)? || mmodel::index_of(&head)? == mmodel::PLAYER_INDEX {
                continue;
            }
            if !mmodel::on_rail(&head)? {
                continue;
            }
            let (line, a, x) = mmodel::rail(&head)?;
            if let Some(cell) = corridor.at(line, a, x)? {
                out.push(self.model_item(model, &head, cell, (1, 1))?);
            }
        }
        Ok(out)
    }

    /// A warp as what the player walks through: a door, the mat out, stairs, or the path between
    /// two outdoor matrices, told by the kind and whether the zone it goes to is outdoors. The
    /// place it arrives in is said by the navigator, which has the game's words
    /// (`Navigator.destination`).
    fn warp(
        &self,
        index: i64,
        record: &[u8],
        at: Option<(Cell, (i64, i64))>,
    ) -> Result<Interactable> {
        let (to, kind) = (u16(record, 0)?, u16(record, 4)?);
        let (cell, size) = match at {
            None => (
                (from_world(s16(record, 8)?), from_world(s16(record, 12)?)),
                (u16(record, 14)?, u16(record, 16)?),
            ),
            Some(at) => at,
        };
        let what =
            if kind == WARP_DOOR || (kind >> 8 == WARP_STREET && !self.outdoors.contains(&to)) {
                "door"
            } else if kind == WARP_OUT {
                "way out"
            } else if kind >> 8 == WARP_STAIRS {
                if self.outdoors(to)? == Some(true) {
                    "path"
                } else {
                    match kind & 0xFF {
                        1 => "stairs up",
                        2 => "stairs down",
                        _ => "stairs",
                    }
                }
            } else {
                "warp"
            };
        Ok(Interactable {
            size,
            zone: Some(to),
            code: kind,
            source: place_source(WARP_SOURCE, index)?,
            ..Interactable::new("exits", what, cell)
        })
    }

    fn find_player(&self) -> Result<Option<i64>> {
        if let Some(player) = self.player.get()
            && self.is_player(player)?
        {
            return Ok(Some(player));
        }
        let mut found = None;
        let models = self.models.borrow().clone();
        for model in models {
            if self.is_player(model)? {
                found = Some(model);
                break;
            }
        }
        self.player.set(found);
        // A new model for the player means a warp, and a warp can be onto another matrix.
        self.read_map()?;
        Ok(self.player.get())
    }

    fn is_player(&self, model: i64) -> Result<bool> {
        let head = self.memory.read(model, mmodel::ZONE + 4)?;
        Ok(mmodel::in_use(&head)? && mmodel::index_of(&head)? == mmodel::PLAYER_INDEX)
    }

    /// The zone data in use now: the holder's first buffer.
    fn current(&self) -> Result<Option<ZoneData>> {
        let address = match self.holder.borrow().as_ref() {
            None => return Ok(None),
            Some(holder) => holder.address,
        };
        let address = u32(&self.memory.read(address, 4)?, 0)?;
        let known = self
            .holder
            .borrow()
            .as_ref()
            .and_then(|holder| holder.files.get(&address).cloned());
        if known.is_some() {
            return Ok(known);
        }
        let data = zone_file_in(&*self.memory, address)?;
        if let Some(data) = &data
            && let Some(holder) = self.holder.borrow_mut().as_mut()
        {
            holder.files.insert(address, data.clone());
        }
        Ok(data)
    }

    /// Every model in use but the player's: an item ball or a thing by its model code, a person
    /// otherwise. The moving truck has a person's kind of code and no script, and trainers have no
    /// script either, so it passes for a person until model codes have names. Where a zone's
    /// rotating doors are known, the invisible models the game sets round them for their arms are
    /// not anybody: five were listed as "person 1 west" round Fortree Gym's third door, and P said
    /// "person in the way" (spike 161).
    fn people(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.flattened(&chunk, None, false)?);
        }
        let stand_in = match &self.doors {
            Some(doors) if !doors.doors(zone)?.is_empty() => Some(doors.stand_in()),
            _ => None,
        };
        let models = self.models.borrow().clone();
        for model in models {
            let head = self.memory.read(model, mmodel::HEAD)?;
            if !mmodel::in_use(&head)? || mmodel::index_of(&head)? == mmodel::PLAYER_INDEX {
                continue;
            }
            let Some(cell) = mmodel::cell(&head)? else {
                continue;
            };
            if walled_in(&chunks, cell)? {
                continue;
            }
            if let Some(stand_in) = stand_in
                && mmodel::drawn_as(&*self.memory, model, &head, self.drawn)? == stand_in
            {
                continue;
            }
            let (cell, size) = mmodel::footprint(&head, cell)?;
            out.push(self.model_item(model, &head, cell, size)?);
        }
        Ok(out)
    }

    /// A model as what it is, standing in `size` cells from `cell`: an item ball or a thing by its
    /// model code, a person otherwise.
    ///
    /// Told apart by its slot and the record it was built from: walking into the next zone fills
    /// the slots of the zone left with the new zone's people, and the way to a Youngster on Route
    /// 102 became a way to Norman in Petalburg City, in the same slot (spike 131).
    fn model_item(
        &self,
        model: i64,
        head: &[u8],
        cell: Cell,
        size: (i64, i64),
    ) -> Result<Interactable> {
        let code = mmodel::drawn_as(&*self.memory, model, head, self.drawn)?;
        let source = u32(head, mmodel::ZONE)? << 32 | model;
        let (category, kind) = if code == mmodel::ITEM_BALL {
            ("items", "item ball")
        } else if code >= PROP_MODEL {
            ("signs", "object")
        } else if self.barriers.contains(&(mmodel::zone_of(head)?, code)) {
            ("signs", BARRIER)
        } else {
            ("people", "person")
        };
        Ok(Interactable {
            size,
            code,
            source,
            move_model: true,
            ..Interactable::new(category, kind, cell)
        })
    }

    /// The grid, and each chunk's file where the ROM's chunks are read, as the matrix has them now:
    /// the game rewrites the one block in place for each matrix, so the sizes are read with them.
    fn read_map(&self) -> Result<()> {
        let grid = self.read_grid()?;
        *self.grid.borrow_mut() = grid;
        let (wide, files) = if self.far_cells.is_some() {
            self.read_files()?
        } else {
            (0, Vec::new())
        };
        *self.files.borrow_mut() = (wide, Rc::new(files));
        Ok(())
    }

    /// The matrix's width in chunks and each chunk's file, row by row, as the field map lists them
    /// now; (0, ()) where there is no field map.
    fn read_files(&self) -> Result<(i64, Vec<i64>)> {
        let Some(map) = self.map.get() else {
            return Ok((0, Vec::new()));
        };
        let head = self.memory.read(map, MAP_GRID)?;
        if grid_size(&head)?.is_none() {
            return Ok((0, Vec::new()));
        }
        let wide = u16(&head, MAP_CHUNKS_WIDE)?;
        let count = wide * u16(&head, MAP_CHUNKS_TALL)?;
        let raw = self.memory.read(map + MAP_CHUNK_FILES, count * 2)?;
        let mut files: Vec<i64> = Vec::new();
        for i in 0..count {
            files.push(u16(&raw, 2 * i)?);
        }
        Ok((wide, files))
    }

    fn read_grid(&self) -> Result<Option<Grid>> {
        let Some(map) = self.map.get() else {
            return Ok(None);
        };
        let Some((width, height)) = grid_size(&self.memory.read(map, MAP_GRID)?)? else {
            return Ok(None);
        };
        let raw = self.memory.read(map + MAP_GRID, width * height * 2)?;
        let mut zones: Vec<i64> = Vec::new();
        for i in 0..width * height {
            zones.push(u16(&raw, i * 2)?);
        }
        Ok(Some(Grid {
            width,
            height,
            zones,
        }))
    }

    /// The chunk at (column, row) of the matrix as the ROM has it, by the file the field map lists
    /// there, or None off the matrix, where it lists none, or without a ROM.
    fn far(&self, column: i64, row: i64) -> Result<Option<Rc<Chunk>>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let Some(far_cells) = &self.far_cells else {
            return Ok(None);
        };
        if !(0 <= column && column < wide) || row < 0 {
            return Ok(None);
        }
        let at = row * wide + column;
        if at >= files.len() as i64 || files[at as usize] == NO_CHUNK_FILE {
            return Ok(None);
        }
        let file = files[at as usize];
        let key = (column, row, file);
        if !self.far_chunks.borrow().contains_key(&key) {
            let words = far_cells(file)?;
            let levels = match &self.far_floors {
                None => Vec::new(),
                Some(far_floors) => far_floors(file)?,
            };
            let chunk = words.map(|words| {
                Rc::new(Chunk {
                    floors: levels,
                    ..Chunk::new((column * CHUNK, row * CHUNK), words)
                })
            });
            self.far_chunks.borrow_mut().insert(key, chunk);
        }
        Ok(self.far_chunks.borrow()[&key].clone())
    }

    fn zone_at(&self, cell: (i64, i64)) -> Result<Option<i64>> {
        let grid = self.grid.borrow();
        let Some(grid) = grid.as_ref() else {
            return Ok(None);
        };
        let zone = grid.at(py::floordiv(cell.0, BLOCK), py::floordiv(cell.1, BLOCK))?;
        Ok(if zone == NO_ZONE { None } else { Some(zone) })
    }

    /// Where the zone's blocks touch another zone's: the open way into it, as each run of cells
    /// along the shared border that can be walked across.
    ///
    /// The border is a run of blocks, and most of it is trees: Littleroot's north side is 40 cells
    /// wide and the way onto Route 101 is 3 of them (the user, 2026-09-22, asking for the walkable
    /// cells rather than the whole border). A cell is a way across when neither side of it is
    /// blocked in the chunks loaded now, and each run of such cells is an exit of its own. Only the
    /// four chunks around the player are loaded, so a border can have one side known and the other
    /// not; a border with neither side known is listed whole, as it was before the chunks were
    /// read.
    fn edges(&self, zone: i64, narrow: bool) -> Result<Vec<Interactable>> {
        let Some(grid) = self.grid.borrow().clone() else {
            return Ok(Vec::new());
        };
        let mut borders: IndexMap<i64, Vec<(Cell, Cell)>> = IndexMap::new();
        for row in 0..grid.height {
            for column in 0..grid.width {
                if grid.at(column, row)? != zone {
                    continue;
                }
                for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let other = grid.at(column + dc, row + dr)?;
                    if other == zone || other == NO_ZONE {
                        continue;
                    }
                    borders
                        .entry(other)
                        .or_default()
                        .extend(border(column, row, dc, dr));
                }
            }
        }
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.flattened(&chunk, None, false)?);
        }
        let wanted = if !borders.is_empty() && chunks.is_empty() {
            Some(zone)
        } else {
            None
        };
        if !narrow {
            // On a rail the chunks mark every cell a wall, and would narrow every edge away.
            chunks = Vec::new();
        }
        if wanted != self.wanted.get() {
            self.wanted.set(wanted);
            self.asks.set(0);
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (other, pairs) in &borders {
            let mut known = false;
            'any: for pair in pairs {
                for cell in [pair.0, pair.1] {
                    if walkable(&chunks, cell)?.is_some() {
                        known = true;
                        break 'any;
                    }
                }
            }
            let mut crossings: Vec<Cell> = Vec::new();
            if known {
                for &(mine, theirs) in pairs {
                    if walkable(&chunks, mine)? != Some(false)
                        && walkable(&chunks, theirs)? != Some(false)
                    {
                        crossings.push(theirs);
                    }
                }
            } else {
                crossings.extend(pairs.iter().map(|&(_, theirs)| theirs));
            }
            for (gap, (first, size)) in runs(crossings).into_iter().enumerate() {
                out.push(Interactable {
                    size,
                    zone: Some(*other),
                    source: place_source(EDGE_SOURCE, other << 16 | gap as i64)?,
                    ..Interactable::new("exits", "edge", first)
                });
            }
        }
        Ok(out)
    }

    /// The cells of the zone the player uses by facing them, each run of one kind a thing: a PC is
    /// one cell of a Pokémon Center's counter, and a bookshelf two cells wide is one shelf, not
    /// two.
    ///
    /// They are cells of the map, not records: a Center's zone data has its bookshelf as furniture
    /// and no PC at all, and the player's bedroom has no record for its trash can (spikes 78 and
    /// 83). So only the chunks loaded now can say where they are, and outdoors a chunk holds
    /// several zones, of which the matrix says whose each cell is.
    fn use_tiles(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut found: IndexMap<i64, Vec<Cell>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            for (cell, kind) in chunk.kinds() {
                if use_tiles(kind).is_some() {
                    let at = self.zone_at(cell)?;
                    if at == Some(zone) || at.is_none() {
                        found.entry(kind).or_default().push(cell);
                    }
                }
            }
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (kind, cells) in found {
            for (first, size) in runs(cells) {
                let called = match self.words.get(&kind) {
                    Some(word) if !word.is_empty() => word.as_str(),
                    _ => use_tiles(kind).expect("a kind found is a use tile's"),
                };
                out.push(Interactable {
                    size,
                    code: kind,
                    source: place_source(TILE_SOURCE, first.0 << 12 | first.1)?,
                    ..Interactable::new("signs", called, first)
                });
            }
        }
        Ok(out)
    }

    /// The zone's slopes, each patch of one kind a thing, and each end of its rails, where the
    /// player gets on one (spike 159). Cells of the map with no record behind them, as use tiles
    /// are, but read off the whole matrix where the ROM's chunks are: the game loads the quarter
    /// round the player, and Route 119's rails are 67 cells north of where it starts. Off the
    /// chunks loaded without the ROM.
    fn bike_ways(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut loaded: IndexMap<(i64, i64), Rc<Chunk>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            loaded.insert(chunk.key(), chunk);
        }
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let key: BikeKey = (
            zone,
            files.clone(),
            if self.far_cells.is_some() {
                Vec::new()
            } else {
                let mut keys: Vec<(i64, i64)> = loaded.keys().copied().collect();
                keys.sort();
                keys
            },
        );
        if self.bike_ways_of.borrow().0.as_ref() == Some(&key) {
            return Ok(self.bike_ways_of.borrow().1.clone());
        }
        let mut chunks = loaded.clone();
        if wide != 0 {
            for at in 0..files.len() as i64 {
                let spot = (py::modulo(at, wide), py::floordiv(at, wide));
                if !chunks.contains_key(&spot)
                    && let Some(far) = self.far(spot.0, spot.1)?
                {
                    chunks.insert(spot, far);
                }
            }
        }
        let mut found: IndexMap<Cell, i64> = IndexMap::new();
        for chunk in chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if (in_slopes(kind) || BIKE_RAILS.contains(&kind) || BUMPY_SLOPES.contains(&kind))
                    && {
                        let at = self.zone_at(cell)?;
                        at == Some(zone) || at.is_none()
                    }
                {
                    found.insert(cell, kind);
                }
            }
        }
        let kind_at = |cell: Cell| -> Result<i64> {
            found
                .get(&cell)
                .copied()
                .ok_or_else(|| Error::key(format!("({}, {})", cell.0, cell.1)))
        };
        let mut out: Vec<Interactable> = Vec::new();
        for patch in patches(&found) {
            if BUMPY_SLOPES.contains(&kind_at(patch[0])?) {
                for &cell in &patch {
                    out.push(Interactable {
                        code: kind_at(cell)?,
                        source: place_source(BIKE_SOURCE, cell_number(cell))?,
                        home: Some(zone),
                        ..Interactable::new(BIKE, BUMPY_SLOPE, cell)
                    });
                }
                continue;
            }
            if in_slopes(kind_at(patch[0])?) {
                let (first, size) = spread(&patch);
                out.push(Interactable {
                    size,
                    code: kind_at(first)?,
                    source: place_source(BIKE_SOURCE, cell_number(first))?,
                    home: Some(zone),
                    ..Interactable::new(BIKE, SLOPE, first)
                });
                continue;
            }
            let held: HashSet<Cell> = patch.iter().copied().collect();
            for &cell in &patch {
                if beside(cell)
                    .iter()
                    .filter(|near| held.contains(*near))
                    .count()
                    <= 1
                {
                    out.push(Interactable {
                        code: kind_at(cell)?,
                        source: place_source(BIKE_SOURCE, cell_number(cell))?,
                        home: Some(zone),
                        ..Interactable::new(BIKE, BIKE_RAIL, cell)
                    });
                }
            }
        }
        *self.bike_ways_of.borrow_mut() = (Some(key), out.clone());
        Ok(out)
    }

    /// The cell tables of the chunks loaded now.
    pub fn chunks_loaded(&self) -> Result<Vec<Rc<Chunk>>> {
        let mut out: Vec<Rc<Chunk>> = Vec::new();
        let blocks = self.blocks.borrow().clone();
        for block in blocks {
            let Some(held) = mapblock::loaded(&*self.memory, block)? else {
                continue;
            };
            let known = self.chunks.borrow().get(&held).cloned();
            let chunk = match known {
                Some(chunk) => chunk,
                None => {
                    let Some(chunk) = mapblock::read(&*self.memory, &held)? else {
                        continue;
                    };
                    let chunk = Rc::new(chunk);
                    self.chunks.borrow_mut().insert(held, chunk.clone());
                    chunk
                }
            };
            out.push(chunk);
        }
        Ok(out)
    }

    /// The word pair pointing at the two zone data buffers, followed by an LZ11 header.
    ///
    /// The second buffer holds whatever zone was loaded before, and after a battle the same zone
    /// again; while the first is being replaced it can be anything, so only the first has to be a
    /// zone data file.
    fn find_holder(
        &self,
        found: &Found,
        files: &IndexMap<i64, ZoneData>,
    ) -> Result<Option<Holder>> {
        let targets: Vec<i64> = files.keys().copied().collect();
        for holder in found.holders(&targets)? {
            let words = self.memory.read(holder, 12)?;
            if files.contains_key(&u32(&words, 0)?)
                && within(HEAP_SPAN, u32(&words, 4)?)
                && u32(&words, 8)? & 0xFF == LZ11
            {
                return Ok(Some(Holder {
                    address: holder,
                    files: files.clone(),
                }));
            }
        }
        Ok(None)
    }

    fn find_map(&self, pages: &IndexMap<i64, Vec<u8>>) -> Result<Option<i64>> {
        let mut needle: Vec<u8> = Vec::new();
        needle.extend_from_slice(&(ALLOCATION as u32).to_le_bytes());
        needle.extend_from_slice(&(MAP_BLOCK_SIZE as u32).to_le_bytes());
        for (&base, page) in pages {
            let mut at = find(page, &needle, 0);
            while let Some(index) = at {
                let block = base + index as i64 + needle.len() as i64;
                let head = self.memory.read(block, MAP_GRID)?;
                if u32(&head, MAP_KIND)? == 4
                    && u32(&head, MAP_MARK)? == MAP_MARKED
                    && grid_size(&head)?.is_some()
                {
                    return Ok(Some(block));
                }
                at = find(page, &needle, index + 1);
            }
        }
        Ok(None)
    }

    /// The zone table, through the current zone's info: the same bytes sit in the table's row for
    /// that zone, so the table starts that many rows before them, and something in the heap points
    /// at its start.
    ///
    /// The zone the info is for is the one the player's model was built in, or the one the matrix
    /// puts them in once they have walked on. A row can run from one page into the next, as Route
    /// 4's did in X; searched a page at a time it was never found (spike 100). Which row the word
    /// before the table's pointer leads to is not the current zone's in X either (it was the zone
    /// the player had come from), so the table is found by its start.
    fn find_table(&self, found: &Found) -> Result<Option<i64>> {
        let data = self.current()?;
        let player = self.find_player()?;
        let (Some(data), Some(player)) = (data, player) else {
            return Ok(None);
        };
        let info = self.memory.read(data.info(), INFO_SIZE)?;
        if !info.iter().any(|&byte| byte != 0) {
            return Ok(None);
        }
        let head = self.memory.read(player, mmodel::HEAD)?;
        let mut zones = PySet::of([mmodel::zone_of(&head)?]);
        let cell = mmodel::cell(&head)?;
        if let Some(cell) = cell
            && let Some(walked) = self.zone_at(cell)?
        {
            zones.add(walked);
        }
        let parent = u16(&info, INFO_PARENT)?;
        for row in spanning(&found.pages, &info) {
            if row == data.info() {
                continue;
            }
            for zone in zones.iter() {
                let base = row - zone * INFO_SIZE;
                if !within(HEAP_SPAN, base) || found.holders(&[base])?.is_empty() {
                    continue;
                }
                // The info's parent is an outdoor zone, which is its own parent in the table.
                if u16(
                    &self.memory.read(base + parent * INFO_SIZE, INFO_SIZE)?,
                    INFO_PARENT,
                )? == parent
                {
                    return Ok(Some(base));
                }
            }
        }
        Ok(None)
    }
}

/// Every zone data file in these pages, validated.
pub fn zone_files(pages: &IndexMap<i64, Vec<u8>>, memory: &dyn Memory) -> Result<Vec<ZoneData>> {
    let mut out: Vec<ZoneData> = Vec::new();
    for (&base, page) in pages {
        for at in all(page, ZONE_MAGIC) {
            if at % 4 == 0
                && let Some(data) = zone_file_in(memory, base + at as i64)?
            {
                out.push(data);
            }
        }
    }
    Ok(out)
}

/// (slot, zone behind it) for every door in from the outdoors in a zone data file, given whole as
/// the ROM keeps it once decompressed (spike 86). The slot is the warp's place among the zone's
/// warps, which is what a door the navigation keys are on is renamed by.
pub fn door_warps(data: &[u8]) -> Result<Vec<(i64, i64)>> {
    Ok(warps(data)?
        .into_iter()
        .filter(|&(_, _, kind)| kind == WARP_DOOR)
        .map(|(slot, to, _)| (slot, to))
        .collect())
}

/// (slot, zone it goes to, kind) for every warp in a zone data file held whole.
pub fn warps(data: &[u8]) -> Result<Vec<(i64, i64, i64)>> {
    let memory = Bytes(data);
    let Some(zone) = zone_file_in(&memory, 0)? else {
        return Ok(Vec::new());
    };
    let mut out: Vec<(i64, i64, i64)> = Vec::new();
    for (slot, record) in records_in(&memory, &zone, WARP)?.iter().enumerate() {
        out.push((slot as i64, u16(record, 0)?, u16(record, 4)?));
    }
    Ok(out)
}

/// (sprite, trainer id) for every NPC record in a zone data file that is a trainer: +0x0a holds
/// 3000 plus the trainer's id, the flag the game sets once they are beaten (spike 90). `trainers`
/// is how many there are, so a flag past the last is not taken for one.
pub fn npc_trainers(data: &[u8], trainers: i64) -> Result<Vec<(i64, i64)>> {
    let memory = Bytes(data);
    let Some(zone) = zone_file_in(&memory, 0)? else {
        return Ok(Vec::new());
    };
    let mut found: Vec<(i64, i64)> = Vec::new();
    for record in records_in(&memory, &zone, NPC)? {
        let trainer = u16(&record, NPC_FLAG)? - TRAINER_FLAGS;
        if 0 < trainer && trainer < trainers {
            found.push((u16(&record, NPC_SPRITE)?, trainer));
        }
    }
    Ok(found)
}

/// A file held whole, read as memory from address 0.
struct Bytes<'a>(&'a [u8]);

impl Memory for Bytes<'_> {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let mut out = py::slice(self.0, address, address + size).to_vec();
        while (out.len() as i64) < size {
            out.push(0);
        }
        Ok(out)
    }
}

/// The size of a zone data file's header: the magic, the count, and an offset a section with one
/// more for the end.
pub fn zone_header(sections: i64) -> i64 {
    4 + 4 * (sections + 1)
}

/// A zone data file as the ROM keeps it once decompressed, validated, or None.
pub fn zone_file(data: &[u8]) -> Result<Option<ZoneData>> {
    zone_file_in(&Bytes(data), 0)
}

/// The records of one kind in a zone data file held whole (`_records`).
pub fn records(data: &[u8], zone: &ZoneData, kind: i64) -> Result<Vec<Vec<u8>>> {
    records_in(&Bytes(data), zone, kind)
}

/// The zone data file at `address`, if what is there is one: the magic, the game's count of
/// sections, offsets that climb from the header's end, and an entity table sized to its counts.
///
/// Python's `_zone_file`: renamed, as the public `zone_file` takes the name it would have here.
fn zone_file_in(memory: &dyn Memory, address: i64) -> Result<Option<ZoneData>> {
    let head = memory.read(address, zone_header(ZONE_SECTIONS[0]))?;
    let sections = u16(&head, 2)?;
    if py::slice(&head, 0, 2) != ZONE_MAGIC || !ZONE_SECTIONS.contains(&sections) {
        return Ok(None);
    }
    let mut offsets: Vec<i64> = Vec::new();
    for i in 0..sections + 1 {
        offsets.push(u32(&head, 4 + 4 * i)?);
    }
    if offsets[0] != zone_header(sections) || offsets[offsets.len() - 1] > MAX_ZONE_SIZE {
        return Ok(None);
    }
    if offsets.windows(2).any(|pair| pair[0] > pair[1]) {
        return Ok(None);
    }
    let data = ZoneData { address, offsets };
    let table = memory.read(data.entities(), ENTITY_HEADER)?;
    let counts: Vec<i64> = py::slice(&table, 4, 9).iter().map(|&n| n as i64).collect();
    let expected = 8
        + counts[0] * FURNITURE
        + counts[1] * NPC
        + counts[2] * WARP
        + counts[3] * TRIGGER
        + counts[4] * AREA;
    if u32(&table, 0)? != expected {
        return Ok(None);
    }
    Ok(Some(data))
}

/// The entity table's records of one kind: `FURNITURE`, `NPC` or `WARP`.
///
/// Python's `_records`: renamed, as the public `records` takes the name it would have here.
fn records_in(memory: &dyn Memory, data: &ZoneData, kind: i64) -> Result<Vec<Vec<u8>>> {
    let head = memory.read(data.entities(), ENTITY_HEADER)?;
    let (furniture, npcs, warps) = (head[4] as i64, head[5] as i64, head[6] as i64);
    let mut at = data.entities() + ENTITY_HEADER;
    let count = if kind == FURNITURE {
        furniture
    } else if kind == NPC {
        at += furniture * FURNITURE;
        npcs
    } else {
        at += furniture * FURNITURE + npcs * NPC;
        warps
    };
    let raw = memory.read(at, count * kind)?;
    Ok((0..count)
        .map(|i| py::slice(&raw, i * kind, (i + 1) * kind).to_vec())
        .collect())
}

/// The entity table's triggers, after its warps.
fn triggers(memory: &dyn Memory, data: &ZoneData) -> Result<Vec<Vec<u8>>> {
    let head = memory.read(data.entities(), ENTITY_HEADER)?;
    let (furniture, npcs, warps, triggers) = (
        head[4] as i64,
        head[5] as i64,
        head[6] as i64,
        head[7] as i64,
    );
    let at = data.entities() + ENTITY_HEADER + furniture * FURNITURE + npcs * NPC + warps * WARP;
    let raw = memory.read(at, triggers * TRIGGER)?;
    Ok((0..triggers)
        .map(|i| py::slice(&raw, i * TRIGGER, (i + 1) * TRIGGER).to_vec())
        .collect())
}

/// What is read by facing it: signs outdoors, the notebook and the clock indoors. NPCs come from
/// the models in use, since a record is where one started and half of them are hidden by the
/// story; triggers and areas are scripts, not places.
fn furniture(memory: &dyn Memory, data: &ZoneData) -> Result<Vec<Interactable>> {
    let mut out: Vec<Interactable> = Vec::new();
    for (i, record) in records_in(memory, data, FURNITURE)?.iter().enumerate() {
        if !on_rail(record)? {
            out.push(Interactable {
                size: (u16(record, 12)?, u16(record, 14)?),
                code: u16(record, 0)?,
                source: place_source(FURNITURE_SOURCE, i as i64)?,
                ..Interactable::new("signs", "sign", (u16(record, 8)?, u16(record, 10)?))
            });
        }
    }
    Ok(out)
}

/// Whether the cell and every cell round it are walls in the chunks loaded: where the game keeps a
/// move model it has built and not put on the map. A Secret Base parks the six people of the
/// player's secret team in the walls of its far corner until each is registered, and they were
/// listed as six people there with no way to them (spike 154).
fn walled_in(chunks: &[Rc<Chunk>], cell: Cell) -> Result<bool> {
    let (x, z) = cell;
    for dx in [-1, 0, 1] {
        for dz in [-1, 0, 1] {
            if walkable(chunks, (x + dx, z + dz))? != Some(false) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The field map's grid in blocks, (width, height), from its header; None where the sizes are zero
/// or the grid would not fit in the block.
fn grid_size(head: &[u8]) -> Result<Option<(i64, i64)>> {
    let width = u16(head, MAP_CHUNKS_WIDE)? * BLOCKS_A_CHUNK;
    let height = u16(head, MAP_CHUNKS_TALL)? * BLOCKS_A_CHUNK;
    if width == 0 || height == 0 || MAP_GRID + width * height * 2 > MAP_BLOCK_SIZE {
        return Ok(None);
    }
    Ok(Some((width, height)))
}

/// The cells along the side of block (`column`, `row`) that faces the block (`dc`, `dr`) away, each
/// paired with the neighbouring block's cell against it.
fn border(column: i64, row: i64, dc: i64, dr: i64) -> Vec<(Cell, Cell)> {
    let (x, z) = (column * BLOCK, row * BLOCK);
    let mut out: Vec<(Cell, Cell)> = Vec::new();
    for i in 0..BLOCK {
        if dc != 0 {
            let mine = (x + if dc > 0 { BLOCK - 1 } else { 0 }, z + i);
            out.push((mine, (mine.0 + dc, mine.1)));
        } else {
            let mine = (x + i, z + if dr > 0 { BLOCK - 1 } else { 0 });
            out.push((mine, (mine.0, mine.1 + dr)));
        }
    }
    out
}

/// The steps a ladder makes, each way: two one-cell triggers side by side whose heights (+0x14)
/// are a drop apart, its top on a platform and its foot on the ground below, as Fortree City's
/// five are, at (21, 13) over (21, 14) and so on (spike 160).
fn ladders(triggers: &[Vec<u8>]) -> Result<IndexSet<(Cell, Cell)>> {
    let mut ends: IndexMap<Cell, i64> = IndexMap::new();
    for record in triggers {
        if u16(record, TRIGGER_WIDTH)? == 1 && u16(record, TRIGGER_HEIGHT)? == 1 {
            ends.insert(
                (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                s16(record, TRIGGER_Y)?,
            );
        }
    }
    let mut out: IndexSet<(Cell, Cell)> = IndexSet::new();
    for (&cell, &high) in &ends {
        for near in beside(cell) {
            if let Some(&low) = ends.get(&near)
                && ((high - low).abs() as f64) > mapblock::DROP
            {
                out.insert((cell, near));
            }
        }
    }
    Ok(out)
}

/// Each patch of cells side by side of one kind, the rails' kinds counting as one, its first cell
/// first.
fn patches(found: &IndexMap<Cell, i64>) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    let mut left: std::collections::BTreeSet<Cell> = found.keys().copied().collect();
    while let Some(&first) = left.iter().next() {
        left.remove(&first);
        let railed = BIKE_RAILS.contains(&found[&first]);
        let mut patch = vec![first];
        let mut todo = vec![first];
        while let Some(cell) = todo.pop() {
            for near in beside(cell) {
                if left.contains(&near)
                    && if railed {
                        BIKE_RAILS.contains(&found[&near])
                    } else {
                        found[&near] == found[&first]
                    }
                {
                    left.remove(&near);
                    patch.push(near);
                    todo.push(near);
                }
            }
        }
        out.push(patch);
    }
    out
}

/// The four cells a step from this one.
fn beside(cell: Cell) -> [Cell; 4] {
    pathfind::STEPS.map(|(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
}

/// The north-west corner of the box round the cells, and its size.
fn spread(cells: &[Cell]) -> (Cell, (i64, i64)) {
    let xs = cells.iter().map(|cell| cell.0);
    let zs = cells.iter().map(|cell| cell.1);
    let (west, east) = (xs.clone().min().unwrap_or(0), xs.max().unwrap_or(0));
    let (north, south) = (zs.clone().min().unwrap_or(0), zs.max().unwrap_or(0));
    ((west, north), (east - west + 1, south - north + 1))
}

/// A cell as a place's number: `x << 12 | z`, as a use tile's is.
fn cell_number(cell: Cell) -> i64 {
    cell.0 << 12 | cell.1
}

/// Each run of cells adjacent along a row or a column, as its first cell and its size.
fn runs(cells: impl IntoIterator<Item = Cell>) -> Vec<(Cell, (i64, i64))> {
    let mut left: Vec<Cell> = cells.into_iter().collect();
    left.sort();
    left.dedup();
    let mut out: Vec<(Cell, (i64, i64))> = Vec::new();
    while !left.is_empty() {
        let first = left.remove(0);
        let mut last = first;
        for step in [(1, 0), (0, 1)] {
            while let Some(at) = left
                .iter()
                .position(|&cell| cell == (last.0 + step.0, last.1 + step.1))
            {
                last = left.remove(at);
            }
            if last != first {
                break;
            }
        }
        out.push((first, (last.0 - first.0 + 1, last.1 - first.1 + 1)));
    }
    out
}

/// Where a record placed on a rail is in the corridor, as its first corner and size, or None for
/// one on the grid or on a line the corridor does not hold.
fn placed(corridor: &rail::Corridor, record: &[u8]) -> Result<Option<(Cell, (i64, i64))>> {
    if !on_rail(record)? {
        return Ok(None);
    }
    let (line, a, x, width, height) = rail::placed(record)?;
    let cells = rail::cells_of(corridor, line, a, x, width, height)?;
    if cells.is_empty() {
        Ok(None)
    } else {
        Ok(Some(rail::spread(cells)?))
    }
}

/// Whether a furniture or warp record is placed on a rail rather than a cell.
pub fn on_rail(record: &[u8]) -> Result<bool> {
    Ok(u16(record, PLACEMENT)? == ON_RAIL)
}

/// The cell a place in world units is in, as a warp record gives its place.
pub fn from_world(units: i64) -> i64 {
    py::floordiv(units - CELL / 2, CELL)
}

/// Every address in these pages where `needle` starts, a copy that runs on into the page after
/// included.
fn spanning(pages: &IndexMap<i64, Vec<u8>>, needle: &[u8]) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    for (&base, page) in pages {
        let following: &[u8] = pages
            .get(&(base + page.len() as i64))
            .map_or(&[], Vec::as_slice);
        let mut joined = page.clone();
        joined.extend_from_slice(py::slice(following, 0, needle.len() as i64 - 1));
        for at in all(&joined, needle) {
            if at < page.len() {
                out.push(base + at as i64);
            }
        }
    }
    out
}

/// Every place `needle` starts in `page`.
fn all(page: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    let mut at = find(page, needle, 0);
    while let Some(index) = at {
        out.push(index);
        at = find(page, needle, index + 1);
    }
    out
}

/// `bytes.find(needle, start)`.
fn find(page: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    let tail = page.get(start..)?;
    let first = *needle.first()?;
    let mut from = 0;
    while let Some(skip) = tail[from..].iter().position(|&byte| byte == first) {
        let at = from + skip;
        if tail[at..].starts_with(needle) {
            return Some(start + at);
        }
        from = at + 1;
    }
    None
}

/// How many slots a set of `n` ints added one at a time has: it grows to the least power of two
/// over four times its size once it is three fifths full (`set_add_entry`).
fn slots_after(n: usize) -> usize {
    let mut slots = 8;
    for used in 1..=n {
        if used * 5 >= (slots - 1) * 3 {
            slots = 8;
            while slots <= used * 4 {
                slots <<= 1;
            }
        }
    }
    slots
}

/// The keys of `set - {out}` in the order Python iterates the difference, which is laid out afresh
/// (`set_difference` in `Objects/setobject.c`): added in the set's order into a new set, or, where
/// the set holds eight or more and the other one, copied and `out` discarded. A copy takes the
/// same slots where its table is as big as the set's and is built key by key in the set's order
/// into one sized for them (`set_merge`) where it is not.
fn without(set: &PySet, out: i64) -> Vec<i64> {
    let keys: Vec<i64> = set.iter().collect();
    if set.len() >> 2 <= 1 {
        let kept = keys.iter().copied().filter(|&key| key != out);
        return PySet::of(kept).iter().collect();
    }
    let mut slots = 8;
    while slots <= set.len() * 2 {
        slots <<= 1;
    }
    let copy = if slots == slots_after(set.len()) {
        keys
    } else {
        let mut copy = PySet::new();
        copy.update_dict(&keys);
        copy.iter().collect()
    };
    copy.into_iter().filter(|&key| key != out).collect()
}

/// `statistics.median`: the middle of the sorted values, or the mean of the two middle ones.
fn median(values: &mut [f64]) -> f64 {
    // `sorted`: a height read from a freed mesh can be NaN, which only CPython's sort places
    // as Python does.
    py::sort_by_lt(values, |a, b| a < b);
    let (n, i) = (values.len(), values.len() / 2);
    if n % 2 == 1 {
        values[i]
    } else {
        (values[i - 1] + values[i]) / 2.0
    }
}

/// `in SLOPES`, the kinds of sandy slope.
fn in_slopes(kind: i64) -> bool {
    SLOPES.iter().any(|&(known, _)| known == kind)
}

/// `pathfind.STEPS[way]`.
fn step_of(way: &str) -> Cell {
    pathfind::STEPS
        .iter()
        .find(|(known, _)| *known == way)
        .map(|&(_, step)| step)
        .expect("a way of the compass")
}
