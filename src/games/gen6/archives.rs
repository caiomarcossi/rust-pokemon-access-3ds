//! The game's own words, out of the player's ROM: place names, the rival, the PC and doors.
//!
//! The ROM keeps its text in GARC archives, one a language, and a file of lines each (findings,
//! spike 74). They name what RAM only numbers:
//!
//! - **Place names**, file 90: "Littleroot Town", indexed by the low ten bits of a zone row's +0x1c
//!   (`field.FieldReader.place_index`, spike 85). The index is in RAM; the words are here.
//! - **Trainer names**, file 22: lines 1 to 3 are the boy protagonist and 4 to 6 the girl, in all
//!   seven languages (spike 85). The rival stands in a work object, and the model says the sprite it
//!   was filled with: `0x13a SUPPORT_F_RS` for May, `0x13b SUPPORT_M_RS` for Brendan (spike 89). So
//!   the rival is named by the game's own word in the player's own language, whoever they are in
//!   that scene, and "Rival" is only what is said without a ROM.
//!
//! - **The PC**, file 92: nothing labels it where it stands, but its own lines all name it, and
//!   the one word they share is the game's word for it (`PC_LINES`).
//!
//! - **Door names**, from three things the ROM holds and nothing links: every zone's data file and
//!   its warps (`a/0/1/3`, LZ11), the zone table with each zone's area kind (the same archive's file
//!   536), and the AreaNav's list of what each town has (`a/0/6/6`, its words in text file 33). A
//!   door is named when the pairing can be *derived* (spike 86): a kind behind doors in two towns or
//!   more is the one facility every one of those towns lists — the Pokémon Center, the Poké Mart,
//!   the Contest Spectacular Hall — and a town left with one door and one facility pairs them. 38 of
//!   143 doors in towns and routes, the same in every language because the pairing is made on line
//!   numbers, never on words. A door the ROM cannot pair is left unnamed, not guessed at.
//!
//! - **The map's places**, which the Pokédex's habitat draws as touch areas of the AreaNav's map
//!   (`a/0/6/7`): the AreaNav's list gives each row's zone the index of its touch area at +0x04,
//!   and the map's archive opens with its layout's pane names, which the index is into (spike 152).
//!
//! The archives and their text files are read by `common.garc`.
//!
//! What a file number holds is a fact about ORAS, the way a struct offset is; what the lines say is
//! the ROM's, which is the point. The two are checked against each other when loaded, so a ROM that
//! is not what these numbers expect is turned away rather than read as nonsense. X and Y keep the same
//! kinds of file under other numbers, with a Town Map where ORAS has the AreaNav; a `RomLayout` says
//! where one game keeps each, and each game's package has its own (`games.oras.ROM`, `games.xy.ROM`;
//! spikes 100 and 101).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use indexmap::IndexMap;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::core::binary::{self, u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::field::{CUT, Floor, ROCK_SMASH, STRENGTH, SURF};
use crate::games::common::garc::{garc_files, garc_spans, message_lines, plain};
use crate::games::gen6::field as gen6_field;
use crate::games::gen6::{mapblock, plates, rail};
use crate::py::{self, Error, Kind, Result};

/// The protagonists' lines in the trainer name list, each three times over.
pub const BOY: i64 = 1;
pub const GIRL: i64 = 4;

/// A trainer record's battle: every trainer of a pair class — "Twins", "Young Couple" — fights
/// doubles, and no class of one person does throughout (spike 90).
pub const DOUBLE: i64 = 1;

/// The moves whose names a way is said with (`Navigator`).
pub const MOVES: [i64; 4] = [CUT, SURF, STRENGTH, ROCK_SMASH];

/// The PC's own text: booting it ("booted up the PC!") and the three PCs it offers ("Someone's
/// PC", "Lanette's PC", "Professor's PC"). Nothing labels the PC where it stands — it is a cell of
/// the Center's counter, drawn with the building, not a sprite — but each of these lines names it,
/// and the word they all share is what the game calls it: "PC" in six languages and "パソコン" in
/// Japanese, the same in both games (2026-09-24). Line 1 is left out because the Italian asks
/// which "sistema" instead.
pub const PC_LINES: [i64; 4] = [0, 2, 3, 6];

/// `struct.Struct("<HHhHH")`: a berry spot in the field DLL's table (spike 141): its cell's x and
/// z, the ground's height there, its zone, and a word every spot shares, 0x218 in ORAS.
pub struct BerrySpot;

pub const BERRY_SPOT: BerrySpot = BerrySpot;

impl BerrySpot {
    /// `BERRY_SPOT.size`.
    pub fn size(&self) -> i64 {
        10
    }

    /// `BERRY_SPOT.iter_unpack(blob)`: a buffer that is not a whole number of records is
    /// `struct.error`.
    #[track_caller]
    pub fn iter_unpack(&self, blob: &[u8]) -> Result<Vec<[i64; 5]>> {
        if blob.len() as i64 % self.size() != 0 {
            return Err(Error::unpack(format!(
                "iterative unpacking requires a buffer of a multiple of {} bytes",
                self.size()
            )));
        }
        let mut out = Vec::new();
        for at in (0..blob.len() as i64).step_by(self.size() as usize) {
            out.push([
                u16(blob, at)?,
                u16(blob, at + 2)?,
                binary::s16(blob, at + 4)?,
                u16(blob, at + 6)?,
                u16(blob, at + 8)?,
            ]);
        }
        Ok(out)
    }

    /// `BERRY_SPOT.pack(*values)`: a value out of its field's range is `struct.error`.
    #[track_caller]
    pub fn pack(&self, values: [i64; 5]) -> Result<Vec<u8>> {
        const FIELDS: [(char, i64, i64); 5] = [
            ('H', 0, 0xFFFF),
            ('H', 0, 0xFFFF),
            ('h', -0x8000, 0x7FFF),
            ('H', 0, 0xFFFF),
            ('H', 0, 0xFFFF),
        ];
        let mut out = Vec::new();
        for (value, (code, least, most)) in values.into_iter().zip(FIELDS) {
            if !(least..=most).contains(&value) {
                return Err(Error::unpack(format!(
                    "'{code}' format requires {least} <= number <= {most}"
                )));
            }
            out.extend_from_slice(&value.to_le_bytes()[..2]);
        }
        Ok(out)
    }
}

/// Fewer records of the shape than this are not the table. ORAS's has 90, and the next longest
/// run of the shape in its DLL is 26, the same table read out of step.
pub const BERRY_SPOTS_LEAST: i64 = 32;
/// More cells than any map matrix is wide or tall.
pub const MATRIX_CELLS: i64 = 0x400;

/// A game's list of what each town has, a row a town naming each facility by its line of a
/// text file: ORAS's AreaNav, X's Town Map. Both are rows of u32s, the town's outdoor zone and
/// then its facilities' lines to the end of the row, under numbers of each game's own.
#[derive(Debug, Clone, PartialEq)]
pub struct TownFacilities {
    pub archive: String,
    pub file: i64,
    /// The text file whose lines the facilities are.
    pub text: i64,
    pub row: i64,
    pub zone: i64,
    /// Where in a row the town's zone is, and where its facilities' lines begin.
    pub first: i64,
}

/// Where one game keeps what `GameText` reads: the same kinds of file in every Gen 6 game,
/// under each game's own numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct RomLayout {
    /// What the game is called in a message saying its ROM is not what was expected.
    pub name: String,
    /// The text archive a language.
    pub text: IndexMap<String, String>,
    pub places: i64,
    pub trainer_names: i64,
    pub class_names: i64,
    pub pc: i64,
    /// Zone N's data file is file N, LZ11-compressed.
    pub zones: String,
    pub zone_table: i64,
    /// One small file a trainer, by the same number as the trainer names.
    pub trainers: String,
    /// Where a trainer's record keeps its class, and the struct format it is read with.
    pub trainer_class: (i64, String),
    /// Where a trainer's record says the battle it fights, `DOUBLE` for doubles.
    pub trainer_battle: i64,
    /// The list of what each town has, which names the buildings behind doors; None in a game
    /// without one.
    pub towns: Option<TownFacilities>,
    /// Sprite -> trainer, for the people the story names (`CHARACTERS`).
    pub characters: IndexMap<i64, i64>,
    /// Whether the trainer names open with the boy three times and the girl three times, which is
    /// how the text archive is checked. Without it, the place names are checked against the zone
    /// table instead.
    pub protagonists: bool,
    /// The map chunks' files, LZ11-compressed, by the number a map's field map lists each chunk
    /// under, where a game has streets placed on rails in them (`Streets`); None in one without.
    pub chunks: Option<String>,
    /// The story text archive a language, whose file a zone-table row's +0x06 gives is what the
    /// zone's script shows, its plates' words among it (`plates.plate_names`); none in a game
    /// whose plates are not read.
    pub story: IndexMap<String, String>,
    /// Where Lumiose's street marks are, a file a zone of its streets (`plates.street_names`).
    pub street_marks: Option<String>,
    /// The text file of Lumiose's street names, two lines a name.
    pub street_names: Option<i64>,
    /// The text file of the moves' names, a line a move by its number (`SURF`).
    pub moves: Option<i64>,
    /// The map chunks' files, LZ11-compressed, by the number a map's field map lists each chunk
    /// under, whose cell tables P's ground reads where the game has not loaded them (`MapChunks`);
    /// None where they are not read.
    pub map_chunks: Option<String>,
    /// The field's DLL, whose table lists every berry spot (`berry_spots`); None in a game
    /// whose berry spots are not read.
    pub berries: Option<String>,
    /// The archives holding textures with names, which `textures.census` indexes; none in a game
    /// whose pictures are not named by their image.
    pub textures: Vec<String>,
    /// The text file whose lines name the languages, and the line they start at: in the order
    /// English, Spanish, French, German, Italian, Japanese, Korean (`textures.LANGUAGES`).
    pub languages: Option<(i64, i64)>,
    /// The text file of the types' names, a line a type by its number.
    pub types: Option<i64>,
    /// The text file naming the moves' categories and the line they start at: status, physical,
    /// special, as the category icons number them (`textures`).
    pub categories: Option<(i64, i64)>,
    /// The text file naming the status conditions and the line they start at: paralysis, asleep,
    /// frozen, burned, poisoned, fainted, Pokérus, as the status icons number them (`textures`).
    pub conditions: Option<(i64, i64)>,
    /// The map's archive and file, which opens with the names of its layout's panes, and where a
    /// row of the list of what each town has keeps the index of its zone's touch area among them
    /// (`map_places`); None in a game whose map is not read.
    pub map_panes: Option<(String, i64, i64)>,
    /// The text file of a Secret Base's decorations' names, a line a decoration by its number;
    /// None in a game without Secret Bases.
    pub decorations: Option<i64>,
    /// The text file of the items' names and the Mach Bike's line in it, which a way up a
    /// sandy slope is said with; None in a game without sandy slopes.
    pub bike: Option<(i64, i64)>,
    /// The same file and the Acro Bike's line, which a hop up a bumpy slope is said with; None in
    /// a game without bumpy slopes.
    pub acro: Option<(i64, i64)>,
}

impl RomLayout {
    /// `RomLayout(name, text, places, trainer_names, class_names, pc, zones, zone_table, trainers,
    /// trainer_class, trainer_battle)`: everything after those is at its default, and is set
    /// with `RomLayout { towns, ..RomLayout::new(...) }`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        text: &IndexMap<&str, &str>,
        places: i64,
        trainer_names: i64,
        class_names: i64,
        pc: i64,
        zones: &str,
        zone_table: i64,
        trainers: &str,
        trainer_class: (i64, &str),
        trainer_battle: i64,
    ) -> Self {
        RomLayout {
            name: name.to_string(),
            text: owned(text),
            places,
            trainer_names,
            class_names,
            pc,
            zones: zones.to_string(),
            zone_table,
            trainers: trainers.to_string(),
            trainer_class: (trainer_class.0, trainer_class.1.to_string()),
            trainer_battle,
            towns: None,
            characters: IndexMap::new(),
            protagonists: false,
            chunks: None,
            story: IndexMap::new(),
            street_marks: None,
            street_names: None,
            moves: None,
            map_chunks: None,
            berries: None,
            textures: Vec::new(),
            languages: None,
            types: None,
            categories: None,
            conditions: None,
            map_panes: None,
            decorations: None,
            bike: None,
            acro: None,
        }
    }
}

/// A table of a game's package, `str` to `str`, as a layout holds it.
pub fn owned(table: &IndexMap<&str, &str>) -> IndexMap<String, String> {
    table
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// The longest run of characters every line holds, with the spaces and punctuation around it
/// taken off, or None when that leaves less than two characters: the one word a handful of
/// lines all say, whatever language they say it in and whether it spaces its words or not.
pub fn shared_word(lines: &[String]) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let shortest: Vec<char> = lines
        .iter()
        .min_by_key(|line| line.chars().count())
        .expect("lines is not empty")
        .chars()
        .collect();
    for size in (2..=shortest.len()).rev() {
        for start in 0..=shortest.len() - size {
            let run: String = shortest[start..start + size].iter().collect();
            if lines.iter().all(|line| line.contains(&run)) {
                let word = trimmed(&run);
                if word.chars().count() >= 2 {
                    return Some(word);
                }
            }
        }
    }
    None
}

/// `run` without the spaces and punctuation at either end.
fn trimmed(run: &str) -> String {
    use GeneralCategory::*;
    fn edge(character: char) -> bool {
        matches!(
            get_general_category(character),
            ConnectorPunctuation
                | DashPunctuation
                | ClosePunctuation
                | FinalPunctuation
                | InitialPunctuation
                | OtherPunctuation
                | OpenPunctuation
                | SpaceSeparator
                | LineSeparator
                | ParagraphSeparator
                | Control
                | Format
                | Surrogate
                | PrivateUse
                | Unassigned
        )
    }

    let run: Vec<char> = run.chars().collect();
    let (mut start, mut end) = (0, run.len());
    while start < end && edge(run[start]) {
        start += 1;
    }
    while end > start && edge(run[end - 1]) {
        end -= 1;
    }
    run[start..end].iter().collect()
}

/// `items[index]`: a negative index counts from the end, and one out of range raises.
#[track_caller]
fn item<T>(items: &[T], index: i64) -> Result<&T> {
    let at = if index < 0 {
        index + items.len() as i64
    } else {
        index
    };
    if at < 0 || at >= items.len() as i64 {
        return Err(Error::index("list index out of range"));
    }
    Ok(&items[at as usize])
}

/// The lines of Lumiose's streets out of the ROM's map chunks, a chunk file at a time as a
/// map asks for them and kept once read: the game loads a line only with the chunks round the
/// player, so a door further along the street was on no line pa3ds knew (spike 106).
#[derive(Debug)]
pub struct Streets {
    path: PathBuf,
    archive: String,
    read: RefCell<IndexMap<i64, rail::Network>>,
}

impl Streets {
    /// Which of a chunk file's sections holds its streets.
    pub const SECTION: i64 = 4;

    pub fn new(path: &Path, archive: &str) -> Self {
        Streets {
            path: path.to_path_buf(),
            archive: archive.to_string(),
            read: RefCell::new(IndexMap::new()),
        }
    }

    /// Every line and track in these chunk files' streets, by id, and where their points
    /// and joints are. A file the ROM does not have, or that is not a chunk's, holds none; so
    /// does every chunk if the ROM cannot be read.
    pub fn network(&self, chunks: &[i64]) -> Result<rail::Network> {
        let mut wanted: Vec<i64> = Vec::new();
        for &chunk in chunks {
            if !self.read.borrow().contains_key(&chunk) && !wanted.contains(&chunk) {
                wanted.push(chunk);
            }
        }
        if !wanted.is_empty() {
            let files = match self.files() {
                Ok(files) => files,
                Err(error) if error.is(Kind::RomError) || error.is(Kind::OsError) => Vec::new(),
                Err(error) => return Err(error),
            };
            for chunk in wanted {
                let network = if chunk < files.len() as i64 {
                    chunk_streets(item(&files, chunk)?)?
                } else {
                    no_streets()
                };
                self.read.borrow_mut().insert(chunk, network);
            }
        }
        let mut lines: IndexMap<i64, rail::Line> = IndexMap::new();
        let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
        for chunk in chunks {
            let network = self
                .read
                .borrow()
                .get(chunk)
                .cloned()
                .unwrap_or_else(no_streets);
            for line in network.lines.values() {
                lines.entry(line.id).or_insert_with(|| line.clone());
            }
            for (&point, &place) in &network.places {
                places.entry(point).or_insert(place);
            }
        }
        Ok(rail::Network { lines, places })
    }

    fn files(&self) -> Result<Vec<Vec<u8>>> {
        let rom = Rom::new(&self.path)?;
        garc_files(&rom.read(&self.archive)?)
    }
}

/// What `MapChunks` keeps of a chunk file: its cell table, its floors and its ground heights.
type Loaded = (Option<Vec<i64>>, Vec<Floor>, Option<Vec<Option<f64>>>);

/// The cell tables of the ROM's map chunks, a file at a time as P's ground reaches into a
/// chunk the game has not loaded, and kept once read: a way found where the map is not loaded
/// took the sea for land, and was given up when the chunks with the sea in them loaded
/// (omega_ruby_surf_2.ctm, spike 136).
#[derive(Debug)]
pub struct MapChunks {
    path: PathBuf,
    archive: String,
    read: RefCell<HashMap<i64, Loaded>>,
}

impl MapChunks {
    pub fn new(path: &Path, archive: &str) -> Self {
        MapChunks {
            path: path.to_path_buf(),
            archive: archive.to_string(),
            read: RefCell::new(HashMap::new()),
        }
    }

    /// One chunk file's cell table, or None where the ROM has no such file, it is not a
    /// chunk's, or the ROM cannot be read.
    pub fn cells(&self, file: i64) -> Result<Option<Vec<i64>>> {
        Ok(self.load(file)?.0)
    }

    /// One chunk file's floors, the ground first, or none for a chunk of one floor
    /// (`mapblock.floors`).
    pub fn floors(&self, file: i64) -> Result<Vec<Floor>> {
        Ok(self.load(file)?.1)
    }

    /// One chunk file's ground heights, a cell at a time, or None
    /// (`mapblock.ground_heights`).
    pub fn heights(&self, file: i64) -> Result<Option<Vec<Option<f64>>>> {
        Ok(self.load(file)?.2)
    }

    fn load(&self, file: i64) -> Result<Loaded> {
        if !self.read.borrow().contains_key(&file) {
            let loaded = match self.read_file(file) {
                Ok(loaded) => loaded,
                Err(error)
                    if error.is(Kind::RomError)
                        || error.is(Kind::OsError)
                        || error.is(Kind::ValueError)
                        || error.is(Kind::IndexError)
                        || error.is(Kind::StructError) =>
                {
                    (None, Vec::new(), None)
                }
                Err(error) => return Err(error),
            };
            self.read.borrow_mut().insert(file, loaded);
        }
        Ok(self.read.borrow()[&file].clone())
    }

    fn read_file(&self, file: i64) -> Result<Loaded> {
        let packed = {
            let rom = Rom::new(&self.path)?;
            let (start, spans) = garc_spans(&rom, &self.archive)?;
            let (at, end) = if file < spans.len() as i64 {
                *item(&spans, file)?
            } else {
                (0, 0)
            };
            rom.read_part(&self.archive, start + at, end - at)?
        };
        let data = if py::slice(&packed, 0, 1) == [0x11] {
            lz11(&packed)?
        } else {
            packed
        };
        Ok((
            mapblock::cells(&data)?,
            mapblock::floors(&data)?,
            mapblock::ground_heights(&data)?,
        ))
    }
}

/// `NO_STREETS`: a network with no lines and no places, which a `const` cannot hold.
pub fn no_streets() -> rail::Network {
    rail::Network::default()
}

/// One chunk file's streets, or none from a file that is not a chunk's.
fn chunk_streets(packed: &[u8]) -> Result<rail::Network> {
    let data = if py::slice(packed, 0, 1) == [0x11] {
        match lz11(packed) {
            Ok(data) => data,
            Err(error)
                if error.is(Kind::RomError)
                    || error.is(Kind::ValueError)
                    || error.is(Kind::IndexError) =>
            {
                return Ok(no_streets());
            }
            Err(error) => return Err(error),
        }
    } else {
        packed.to_vec()
    };
    if py::slice(&data, 0, 2) != b"GR" || data.len() < 8 {
        return Ok(no_streets());
    }
    let sections = u16(&data, 2)?;
    if sections <= Streets::SECTION || (data.len() as i64) < 8 + 4 * sections {
        return Ok(no_streets());
    }
    let (start, end) = (
        u32(&data, 4 + 4 * Streets::SECTION)?,
        u32(&data, 4 + 4 * Streets::SECTION + 4)?,
    );
    if start < end && end <= data.len() as i64 {
        rail::section_streets(py::slice(&data, start, end))
    } else {
        Ok(no_streets())
    }
}

/// What the ROM names that RAM only numbers, in one language.
///
/// `streets`, `street_records` and `map_chunks` are left out of the comparison, as the
/// dataclass leaves them (`compare=False`).
#[derive(Debug, Clone)]
pub struct GameText {
    pub places: Vec<String>,
    pub trainers: Vec<String>,
    pub language: String,
    /// (outdoor zone, warp slot) -> what the game calls the building behind that door.
    pub doors: IndexMap<(i64, i64), String>,
    /// Sprite -> what the game calls whoever is drawn with it: a person, or a kind of trainer.
    pub sprites: IndexMap<i64, String>,
    /// What the game calls the PC, out of the PC's own lines (`PC_LINES`), or None.
    pub pc: Option<String>,
    /// Where Lumiose's streets are read from, in a game that has any (`RomLayout.chunks`).
    pub streets: Option<Rc<Streets>>,
    /// (zone, warp slot) -> the street of Lumiose's a warp between two of its zones comes out on
    /// (`plates.street_names`).
    pub street_to: IndexMap<(i64, i64), String>,
    /// The zones walked about in as outdoors (`plates.outdoors`).
    pub outdoors: HashSet<i64>,
    /// Zone -> its furniture and warps, for every zone with records on streets: listed with
    /// the zone the player is in when both are on one map (`FieldReader`, spike 107).
    pub street_records: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)>,
    /// What the game calls each of `MOVES`, by its number; none where the ROM's move names are
    /// not read.
    pub moves: IndexMap<i64, String>,
    /// Where the cell tables of chunks the game has not loaded are read from
    /// (`RomLayout.map_chunks`).
    pub map_chunks: Option<Rc<MapChunks>>,
    /// (zone, cell) of every berry spot, in the game's own order (`berry_spots`).
    pub berries: Vec<(i64, (i64, i64))>,
    /// What the game calls each language it carries, in the language choice's order
    /// (`RomLayout.languages`); none where the ROM's are not read.
    pub languages: Vec<String>,
    /// What the game calls each type, by its number; none where they are not read.
    pub types: Vec<String>,
    /// What the game calls each move category, in `RomLayout.categories`' order.
    pub categories: Vec<String>,
    /// What the game calls each status condition, in `RomLayout.conditions`' order.
    pub conditions: Vec<String>,
    /// A touch area of the map, by its pane's name -> what the game calls the place, in the
    /// order the AreaNav lists them (`map_places`); none where the map is not read.
    pub map_places: IndexMap<String, String>,
    /// What the game calls each Secret Base decoration, by its number (`RomLayout.decorations`).
    pub decorations: Vec<String>,
    /// What the game calls the Mach Bike (`RomLayout.bike`), or None.
    pub bike: Option<String>,
    /// What the game calls the Acro Bike (`RomLayout.acro`), or None.
    pub acro: Option<String>,
}

impl PartialEq for GameText {
    fn eq(&self, other: &Self) -> bool {
        self.places == other.places
            && self.trainers == other.trainers
            && self.language == other.language
            && self.doors == other.doors
            && self.sprites == other.sprites
            && self.pc == other.pc
            && self.street_to == other.street_to
            && self.outdoors == other.outdoors
            && self.moves == other.moves
            && self.berries == other.berries
            && self.languages == other.languages
            && self.types == other.types
            && self.categories == other.categories
            && self.conditions == other.conditions
            && self.map_places == other.map_places
            && self.decorations == other.decorations
            && self.bike == other.bike
            && self.acro == other.acro
    }
}

impl Default for GameText {
    /// `GameText((), ())`: nothing named, in English.
    fn default() -> Self {
        GameText {
            places: Vec::new(),
            trainers: Vec::new(),
            language: "en".to_string(),
            doors: IndexMap::new(),
            sprites: IndexMap::new(),
            pc: None,
            streets: None,
            street_to: IndexMap::new(),
            outdoors: HashSet::new(),
            street_records: IndexMap::new(),
            moves: IndexMap::new(),
            map_chunks: None,
            berries: Vec::new(),
            languages: Vec::new(),
            types: Vec::new(),
            categories: Vec::new(),
            conditions: Vec::new(),
            map_places: IndexMap::new(),
            decorations: Vec::new(),
            bike: None,
            acro: None,
        }
    }
}

impl GameText {
    /// `GameText(places, trainers)`: the rest left to its default, set with
    /// `GameText { pc, ..GameText::new(places, trainers) }`.
    pub fn new(places: Vec<String>, trainers: Vec<String>) -> Self {
        GameText {
            places,
            trainers,
            ..GameText::default()
        }
    }

    /// Read one language's names out of the ROM. A RomError when the language is not one
    /// the game carries or the files are not what the layout says they are.
    pub fn load(rom: &Rom, language: &str, layout: &RomLayout) -> Result<GameText> {
        let Some(archive) = layout.text.get(language) else {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "no text in {}; {} carries {}",
                    py::repr(language),
                    layout.name,
                    layout.text.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            ));
        };
        let files = garc_files(&rom.read(archive)?)?;
        if files.len() as i64 <= layout.places.max(layout.trainer_names) {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} holds {} files, too few to be {}'s text",
                    files.len(),
                    layout.name
                ),
            ));
        }
        let places: Vec<String> = message_lines(item(&files, layout.places)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let trainers: Vec<String> = message_lines(item(&files, layout.trainer_names)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let classes: Vec<String> = message_lines(item(&files, layout.class_names)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let zones = garc_files(&rom.read(&layout.zones)?)?;
        let table = zone_table(&zones, layout)?;
        let read = read_zones(&zones, layout)?;
        let outside = plates::outdoors(&read, &table)?;
        let mut doors: IndexMap<(i64, i64), String> = IndexMap::new();
        // Place -> what its town's row calls each facility, which a plate may hold.
        let mut listed: IndexMap<i64, Vec<String>> = IndexMap::new();
        if let Some(towns) = &layout.towns {
            let facilities: Vec<String> = message_lines(item(&files, towns.text)?)?
                .iter()
                .map(|line| plain(line))
                .collect();
            for (door, line) in door_facilities(rom, Some(&zones), layout, &outside)? {
                if line < facilities.len() as i64 && !facilities[line as usize].is_empty() {
                    doors.insert(door, facilities[line as usize].clone());
                }
            }
            let size = gen6_field::INFO_SIZE;
            for (zone, lines) in town_rows(rom, towns, table.len() as i64 / size)? {
                let place =
                    u16(&table, zone * size + gen6_field::INFO_NAME)? & gen6_field::NAME_MASK;
                listed.entry(place).or_default().extend(
                    lines
                        .iter()
                        .filter(|&&line| line < facilities.len() as i64)
                        .map(|&line| facilities[line as usize].clone()),
                );
            }
        }
        // The plates and the streets name what is otherwise said by its kind, so a ROM without
        // their files, as a version laid out otherwise might be, is read for the rest.
        if let Some(story_archive) = layout.story.get(language) {
            let story = match rom.read(story_archive).and_then(|blob| garc_files(&blob)) {
                Ok(story) => story,
                Err(error) if error.is(Kind::RomError) => Vec::new(),
                Err(error) => return Err(error),
            };
            let named =
                plates::plate_names(&read, &table, &lines_of(story), &outside, Some(&listed))?;
            for (door, words) in named {
                doors.entry(door).or_insert(words);
            }
        }
        let mut street_to: IndexMap<(i64, i64), String> = IndexMap::new();
        if let Some(street_marks) = layout
            .street_marks
            .as_ref()
            .filter(|marks| !marks.is_empty())
            && let Some(street_names) = layout.street_names
            && street_names < files.len() as i64
        {
            let loaded = (|| -> Result<(Vec<String>, Vec<Vec<u8>>)> {
                let names = message_lines(item(&files, street_names)?)?
                    .iter()
                    .map(|line| plain(line))
                    .collect();
                let marks = garc_files(&rom.read(street_marks)?)?;
                Ok((names, marks))
            })();
            let (names, marks) = match loaded {
                Ok(loaded) => loaded,
                Err(error) if error.is(Kind::RomError) => (Vec::new(), Vec::new()),
                Err(error) => return Err(error),
            };
            street_to = plates::street_names(&read, &table, &marks, &names)?;
        }
        let mut sprites: IndexMap<i64, String> = IndexMap::new();
        for (&sprite, &trainer) in &layout.characters {
            if trainer < trainers.len() as i64 && !trainers[trainer as usize].is_empty() {
                sprites.insert(sprite, trainers[trainer as usize].clone());
            }
        }
        let (class_of, pairs) = trainer_classes(rom, layout)?;
        for (sprite, words) in
            trainer_sprites(&zones, &class_of, &pairs, &classes, &trainers, layout)?
        {
            sprites.entry(sprite).or_insert(words);
        }
        let mut pc = None;
        if files.len() as i64 > layout.pc && !files[layout.pc as usize].is_empty() {
            let lines = message_lines(&files[layout.pc as usize])?;
            if lines.len() as i64 > PC_LINES[PC_LINES.len() - 1] {
                let said: Vec<String> = PC_LINES
                    .iter()
                    .map(|&line| plain(&lines[line as usize]))
                    .collect();
                pc = shared_word(&said);
            }
        }
        let mut moves: IndexMap<i64, String> = IndexMap::new();
        if let Some(file) = layout.moves
            && file < files.len() as i64
            && !files[file as usize].is_empty()
        {
            let names = message_lines(&files[file as usize])?;
            for r#move in MOVES {
                if r#move < names.len() as i64 {
                    let word = plain(&names[r#move as usize]);
                    if !word.is_empty() {
                        moves.insert(r#move, word);
                    }
                }
            }
        }
        let streets = layout
            .chunks
            .as_ref()
            .filter(|chunks| !chunks.is_empty())
            .map(|chunks| Rc::new(Streets::new(&rom.path, chunks)));
        let mut on_streets: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)> = IndexMap::new();
        for (&zone, (_data, furniture, warps)) in &read {
            let mut any = false;
            for record in furniture.iter().chain(warps) {
                if gen6_field::on_rail(record)? {
                    any = true;
                    break;
                }
            }
            if any {
                on_streets.insert(zone, (furniture.clone(), warps.clone()));
            }
        }
        let mut berries: Vec<(i64, (i64, i64))> = Vec::new();
        if let Some(file) = &layout.berries {
            berries = match rom.read(file) {
                Ok(dll) => berry_spots(&dll),
                Err(error) if error.is(Kind::RomError) => Vec::new(),
                Err(error) => return Err(error),
            };
        }
        let lines_from = |from: Option<(i64, i64)>| -> Result<Vec<String>> {
            match from {
                None => Ok(Vec::new()),
                Some((file, start)) => {
                    Ok(py::slice_from(&lines_of_file(&files, file)?, start).to_vec())
                }
            }
        };

        let types = match layout.types {
            Some(file) => lines_of_file(&files, file)?,
            None => Vec::new(),
        };
        let languages = lines_from(layout.languages)?;
        let categories = lines_from(layout.categories)?;
        let conditions = lines_from(layout.conditions)?;
        let text = GameText {
            places: places.clone(),
            trainers,
            language: language.to_string(),
            doors,
            sprites,
            pc,
            streets,
            street_to,
            outdoors: outside,
            street_records: on_streets,
            moves,
            map_chunks: layout
                .map_chunks
                .as_ref()
                .filter(|chunks| !chunks.is_empty())
                .map(|chunks| Rc::new(MapChunks::new(&rom.path, chunks))),
            berries,
            languages,
            types,
            categories,
            conditions,
            map_places: map_places_of(rom, layout, &table, &places)?,
            decorations: match layout.decorations {
                Some(file) => lines_of_file(&files, file)?,
                None => Vec::new(),
            },
            bike: lines_from(layout.bike)?
                .into_iter()
                .next()
                .filter(|line| !line.is_empty()),
            acro: lines_from(layout.acro)?
                .into_iter()
                .next()
                .filter(|line| !line.is_empty()),
        };
        if layout.protagonists && !text.protagonists_hold() {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} file {} is not the trainer names this version of pa3ds knows",
                    layout.trainer_names
                ),
            ));
        }
        if !layout.protagonists && !text.places_hold(&table)? {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} file {} is not the place names this version of pa3ds knows",
                    layout.places
                ),
            ));
        }
        Ok(text)
    }

    /// What the game calls a place, or None for the placeholder at 0 and an empty line.
    pub fn place(&self, index: i64) -> Option<String> {
        if !(0 < index && index < self.places.len() as i64) {
            return None;
        }
        Some(self.places[index as usize].clone()).filter(|place| !place.is_empty())
    }

    /// What the game calls the building behind a door in from this zone's outdoors, or None
    /// when the ROM does not say which building it is.
    pub fn door(&self, zone: i64, slot: i64) -> Option<String> {
        self.doors.get(&(zone, slot)).cloned()
    }

    /// The street of Lumiose's this zone's warp comes out on, or None.
    pub fn street(&self, zone: i64, slot: i64) -> Option<String> {
        self.street_to.get(&(zone, slot)).cloned()
    }

    /// What the game calls whoever is drawn with this sprite, or None: a person the story
    /// names, or the kind of trainer everyone drawn as it is.
    pub fn character(&self, sprite: i64) -> Option<String> {
        self.sprites.get(&sprite).cloned()
    }

    /// Is the place list what it should be: long enough for every zone's place, with a name
    /// at the place of every outdoor zone. That holds of X's list in every language it carries
    /// (spike 100), and of no other file of its archive.
    fn places_hold(&self, table: &[u8]) -> Result<bool> {
        let size = gen6_field::INFO_SIZE;
        for zone in 0..table.len() as i64 / size {
            let (parent, index) = (
                u16(table, zone * size + gen6_field::INFO_PARENT)?,
                u16(table, zone * size + gen6_field::INFO_PARENT + 2)?,
            );
            let index = index & gen6_field::NAME_MASK;
            if index >= self.places.len() as i64
                || (parent == zone && self.places[index as usize].is_empty())
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Is the trainer list what it should be: the boy three times, then the girl three
    /// times, two different words. That is true in every language ORAS carries (spike 85) and
    /// of nothing else in the archive.
    fn protagonists_hold(&self) -> bool {
        let names = &self.trainers;
        if names.len() as i64 <= GIRL + 2 {
            return false;
        }
        let boy = py::slice(names, BOY, BOY + 3);
        let girl = py::slice(names, GIRL, GIRL + 3);
        let one = |words: &[String]| words.iter().collect::<HashSet<_>>().len() == 1;
        one(boy)
            && one(girl)
            && boy[0] != girl[0]
            && boy.iter().chain(girl).all(|name| !name.is_empty())
    }
}

/// The area kind of a Gym's rooms, in ORAS and in X alike (spikes 85, 101).
pub const GYM: i64 = 0x11;

/// (outdoor zone, warp slot) -> the line of the game's list of what each town has
/// (`TownFacilities`) naming the building behind that door, for every door the ROM's own tables
/// pair with a facility (spike 86; X's Town Map, spike 101). `outside` is the zones walked about
/// in as outdoors besides the towns' own (`plates.outdoors`): Lumiose's streets, whose doors
/// count as the town's (spike 107).
///
/// 1. A door whose zone has the area kind of the town's own outdoors leads to a plain house: a
///    town and its houses share one kind. Some facilities do too (Dewford Hall, the Trainers'
///    School), and they are left unnamed with the houses.
/// 2. A kind behind doors in two towns or more is a shared facility, named for the one facility
///    every one of those towns lists that no kind in more towns has already taken — most towns
///    first, so the Pokémon Center's kind takes "Pokémon Center" before the Gyms' kind can.
/// 3. A town left with one unnamed door and one unnamed facility pairs them.
/// 4. A door off a street, or in a zone of the town that is not its own parent, is named by the
///    kind as a shared facility only where its town lists that facility: Lumiose lists
///    "Boutique Couture" where five towns list "Boutique" (spike 107). What kind is which
///    facility is worked out from the towns' own doors alone, as before.
/// 5. A warp into a Gym's room is named for the facility its town's row lists just before the
///    one the most towns share, the Pokémon Center: in every one of X's eight Gym towns the Gym
///    comes there, and in no other town does anything (spike 107). ORAS lists the Center first,
///    so nothing is named so there.
pub fn door_facilities(
    rom: &Rom,
    zones: Option<&[Vec<u8>]>,
    layout: &RomLayout,
    outside: &HashSet<i64>,
) -> Result<IndexMap<(i64, i64), i64>> {
    let Some(towns) = &layout.towns else {
        return Ok(IndexMap::new());
    };
    let read_zones;
    let zones: &[Vec<u8>] = match zones {
        Some(zones) => zones,
        None => {
            read_zones = garc_files(&rom.read(&layout.zones)?)?;
            &read_zones
        }
    };
    let table = zone_table(zones, layout)?;
    let count = table.len() as i64 / gen6_field::INFO_SIZE;

    let row = |zone: i64| -> &[u8] {
        py::slice(
            &table,
            zone * gen6_field::INFO_SIZE,
            (zone + 1) * gen6_field::INFO_SIZE,
        )
    };
    let kind = |zone: i64| -> Result<i64> { u16(row(zone), gen6_field::INFO_KIND) };
    let parent = |zone: i64| -> Result<i64> { u16(row(zone), gen6_field::INFO_PARENT) };
    let place = |zone: i64| -> Result<i64> {
        Ok(u16(row(zone), gen6_field::INFO_NAME)? & gen6_field::NAME_MASK)
    };

    let mut listed: IndexMap<i64, Vec<i64>> = IndexMap::new();
    // A row's zone -> its facilities in the row's order.
    let mut rows: IndexMap<i64, Vec<i64>> = IndexMap::new();
    for (zone, ordered) in town_rows(rom, towns, count)? {
        let lines = listed.entry(place(zone)?).or_default();
        let kept = rows.entry(zone).or_default();
        for line in ordered {
            if !lines.contains(&line) {
                lines.push(line);
            }
            if !kept.contains(&line) {
                kept.push(line);
            }
        }
    }

    // Place -> (outdoor zone, slot, zone behind) for every door in from its outdoors.
    let mut doors: IndexMap<i64, Vec<(i64, i64, i64)>> = IndexMap::new();
    // The same for doors off a street, and in a town's zones that are not their own parent.
    let mut street_doors: IndexMap<i64, Vec<(i64, i64, i64)>> = IndexMap::new();
    // (outdoor zone, slot) of every warp into a Gym's room.
    let mut gyms: Vec<(i64, i64)> = Vec::new();
    let mut own: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for zone in 0..count {
        let walked = parent(zone)? == zone || outside.contains(&zone);
        if !walked
            || !listed.contains_key(&place(zone)?)
            || zone >= zones.len() as i64
            || zones[zone as usize].is_empty()
        {
            continue;
        }
        own.entry(place(zone)?).or_default().insert(kind(zone)?);
        for (slot, to, how) in gen6_field::warps(&lz11(&zones[zone as usize])?)? {
            if to >= count || outside.contains(&to) {
                continue;
            }
            if kind(to)? == GYM {
                gyms.push((zone, slot));
            }
            if how == gen6_field::WARP_DOOR && parent(zone)? == zone {
                doors
                    .entry(place(zone)?)
                    .or_default()
                    .push((zone, slot, to));
            } else if how == gen6_field::WARP_DOOR || how >> 8 == gen6_field::WARP_STREET {
                street_doors
                    .entry(place(zone)?)
                    .or_default()
                    .push((zone, slot, to));
            }
        }
    }

    let mut behind: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for (town, entries) in &doors {
        for &(_zone, _slot, to) in entries {
            if !own[town].contains(&kind(to)?) {
                behind.entry(kind(to)?).or_default().insert(*town);
            }
        }
    }
    let mut shared: IndexMap<i64, i64> = IndexMap::new();
    let mut taken: HashSet<i64> = HashSet::new();
    // The facility the most towns share, the Pokémon Center.
    let mut centre: Option<i64> = None;
    let mut by_towns: Vec<(&i64, &HashSet<i64>)> = behind.iter().collect();
    by_towns.sort_by_key(|(_building, towns)| std::cmp::Reverse(towns.len()));
    for (&building, towns) in by_towns {
        if towns.len() < 2 {
            continue;
        }
        let first = *towns.iter().next().expect("two towns or more");
        let mut common: HashSet<i64> = listed[&first].iter().copied().collect();
        for town in towns {
            let lines: HashSet<i64> = listed[town].iter().copied().collect();
            common = common.intersection(&lines).copied().collect();
        }
        common.retain(|line| !taken.contains(line));
        if common.len() == 1 {
            let line = common.into_iter().next().expect("one facility is common");
            shared.insert(building, line);
            taken.insert(line);
            if centre.is_none() {
                centre = Some(line);
            }
        }
    }

    let mut named: IndexMap<(i64, i64), i64> = IndexMap::new();
    for (town, entries) in &doors {
        let mut left: Vec<(i64, i64)> = Vec::new();
        for &(zone, slot, to) in entries {
            if !own[town].contains(&kind(to)?) && !shared.contains_key(&kind(to)?) {
                left.push((zone, slot));
            }
        }
        for &(zone, slot, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?) {
                named.insert((zone, slot), line);
            }
        }
        let mut claimed: HashSet<i64> = HashSet::new();
        for &(_z, _s, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?) {
                claimed.insert(line);
            }
        }
        let mut unclaimed: HashSet<i64> = listed[town].iter().copied().collect();
        unclaimed.retain(|line| !claimed.contains(line));
        if left.len() == 1 && unclaimed.len() == 1 {
            named.insert(
                left[0],
                unclaimed.into_iter().next().expect("one facility is left"),
            );
        }
    }
    for (town, entries) in &street_doors {
        for &(zone, slot, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?)
                && listed[town].contains(&line)
            {
                named.insert((zone, slot), line);
            }
        }
    }
    for &(zone, slot) in &gyms {
        let ordered: Vec<i64> = match rows.get(&zone) {
            Some(ordered) if !ordered.is_empty() => ordered.clone(),
            _ => rows.get(&parent(zone)?).cloned().unwrap_or_default(),
        };
        if let Some(centre) = centre
            && let Some(at) = ordered.iter().position(|&line| line == centre)
            && at > 0
        {
            named.entry((zone, slot)).or_insert(ordered[at - 1]);
        }
    }
    Ok(named)
}

/// (outdoor zone, its facilities' lines in the row's order) of each row of the game's list of
/// what each town has, for every row whose zone is one of the zone table's `count`.
pub fn town_rows(rom: &Rom, towns: &TownFacilities, count: i64) -> Result<Vec<(i64, Vec<i64>)>> {
    let mut out: Vec<(i64, Vec<i64>)> = Vec::new();
    let files = garc_files(&rom.read(&towns.archive)?)?;
    let nav = item(&files, towns.file)?;
    let mut at = 0;
    while at < nav.len() as i64 - towns.row + 1 {
        let zone = u16(nav, at + towns.zone)?;
        if zone < count {
            let mut ordered: Vec<i64> = Vec::new();
            let mut offset = towns.first;
            while offset < towns.row {
                let line = u32(nav, at + offset)?;
                if line != 0 && !ordered.contains(&line) {
                    ordered.push(line);
                }
                offset += 4;
            }
            out.push((zone, ordered));
        }
        at += towns.row;
    }
    Ok(out)
}

/// A touch area of the map, by its pane's name -> what the game calls its place.
///
/// The map's archive opens with a u32 count of its files and their names, 64 bytes each, then a
/// u32 count of its layout's panes and their names, 32 bytes each; a row of the list of what
/// each town has keeps its zone's touch area as an index into those (spike 152). Rows sharing a
/// touch area, as a cave's floors do, name it by the first.
pub fn map_places(
    head: &[u8],
    nav: &[u8],
    towns: &TownFacilities,
    pane_at: i64,
    place: &dyn Fn(i64) -> Option<String>,
) -> Result<IndexMap<String, String>> {
    let files = u32(head, 0)?;
    let at = 4 + 64 * files;
    let count = if at + 4 <= head.len() as i64 {
        u32(head, at)?
    } else {
        0
    };
    if !(0 < count && count <= 4096) || at + 4 + 32 * count > head.len() as i64 {
        return Err(Error::new(
            Kind::RomError,
            "the map's archive does not open with its panes' names",
        ));
    }
    let names: Vec<String> = (0..count)
        .map(|i| {
            let name = py::slice(head, at + 4 + 32 * i, at + 36 + 32 * i);
            let end = name
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(name.len());
            py::decode_ascii(&name[..end])
        })
        .collect();
    let mut out: IndexMap<String, String> = IndexMap::new();
    let mut row = 0;
    while row < nav.len() as i64 - towns.row + 1 {
        let zone = u16(nav, row + towns.zone)?;
        let pane = u16(nav, row + pane_at)?;
        if pane < count
            && let Some(name) = place(zone).filter(|name| !name.is_empty())
        {
            out.entry(names[pane as usize].clone()).or_insert(name);
        }
        row += towns.row;
    }
    Ok(out)
}

/// `map_places` out of the ROM, or none where the game's map is not read.
fn map_places_of(
    rom: &Rom,
    layout: &RomLayout,
    table: &[u8],
    places: &[String],
) -> Result<IndexMap<String, String>> {
    let (Some((archive, file, pane_at)), Some(towns)) = (&layout.map_panes, &layout.towns) else {
        return Ok(IndexMap::new());
    };
    let size = gen6_field::INFO_SIZE;

    let place = |zone: i64| -> Option<String> {
        if (zone + 1) * size > table.len() as i64 {
            return None;
        }
        let index = (u16(table, zone * size + gen6_field::INFO_NAME).ok()?) & gen6_field::NAME_MASK;
        if 0 < index && index < places.len() as i64 && !places[index as usize].is_empty() {
            Some(places[index as usize].clone())
        } else {
            None
        }
    };

    let loaded = (|| -> Result<IndexMap<String, String>> {
        let map_files = garc_files(&rom.read(archive)?)?;
        let head = item(&map_files, *file)?.clone();
        let nav_files = garc_files(&rom.read(&towns.archive)?)?;
        let nav = item(&nav_files, towns.file)?.clone();
        let head = if py::slice(&head, 0, 1) == [0x11] {
            lz11(&head)?
        } else {
            head
        };
        map_places(&head, &nav, towns, *pane_at, &place)
    })();
    match loaded {
        Ok(places) => Ok(places),
        Err(error)
            if error.is(Kind::RomError)
                || error.is(Kind::IndexError)
                || error.is(Kind::StructError) =>
        {
            Ok(IndexMap::new())
        }
        Err(error) => Err(error),
    }
}

/// Every zone's data file the ROM holds, decompressed, with its furniture and its warps.
fn read_zones(zones: &[Vec<u8>], layout: &RomLayout) -> Result<IndexMap<i64, plates::Zone>> {
    let mut out: IndexMap<i64, plates::Zone> = IndexMap::new();
    for (number, packed) in zones.iter().enumerate() {
        let number = number as i64;
        if number == layout.zone_table || packed.is_empty() {
            continue;
        }
        let data = match lz11(packed) {
            Ok(data) => data,
            Err(error) if error.is(Kind::RomError) => continue,
            Err(error) => return Err(error),
        };
        let Some(zone) = gen6_field::zone_file(&data)? else {
            continue;
        };
        let furniture = gen6_field::records(&data, &zone, gen6_field::FURNITURE)?;
        let warps = gen6_field::records(&data, &zone, gen6_field::WARP)?;
        out.insert(number, (data, furniture, warps));
    }
    Ok(out)
}

/// A text file's lines, or none where the archive has no such file or it is not text: a ROM
/// laid out otherwise loses only what that file names.
fn lines_of_file(files: &[Vec<u8>], file: i64) -> Result<Vec<String>> {
    if !(file < files.len() as i64) || files[file as usize].is_empty() {
        return Ok(Vec::new());
    }
    match message_lines(&files[file as usize]) {
        Ok(lines) => Ok(lines.iter().map(|line| plain(line)).collect()),
        Err(error) if error.is(Kind::RomError) => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

/// A text archive's file as its lines, each split once when first asked for.
fn lines_of(files: Vec<Vec<u8>>) -> impl Fn(i64) -> Result<Vec<Vec<u8>>> {
    let split: RefCell<HashMap<i64, Vec<Vec<u8>>>> = RefCell::new(HashMap::new());

    move |file: i64| -> Result<Vec<Vec<u8>>> {
        if !split.borrow().contains_key(&file) {
            let lines = if file < files.len() as i64 {
                match message_lines(item(&files, file)?) {
                    Ok(lines) => lines,
                    Err(error) if error.is(Kind::RomError) => Vec::new(),
                    Err(error) => return Err(error),
                }
            } else {
                Vec::new()
            };
            split.borrow_mut().insert(file, lines);
        }
        Ok(split.borrow()[&file].clone())
    }
}

/// The zone table, checked: rows of 0x38 bytes, every parent a zone in the table, and many of
/// them their own parent, as the outdoors are (spike 85).
fn zone_table(zones: &[Vec<u8>], layout: &RomLayout) -> Result<Vec<u8>> {
    if zones.len() as i64 <= layout.zone_table {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} holds {} files, too few to hold the zone table",
                layout.zones,
                zones.len()
            ),
        ));
    }
    let table = &zones[layout.zone_table as usize];
    let size = gen6_field::INFO_SIZE;
    let count = table.len() as i64 / size;
    if count < 100 || table.len() as i64 % size != 0 {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} file {} is not the zone table",
                layout.zones, layout.zone_table
            ),
        ));
    }
    let mut parents: Vec<i64> = Vec::new();
    for zone in 0..count {
        parents.push(u16(table, zone * size + gen6_field::INFO_PARENT)?);
    }
    if parents.iter().any(|&p| p >= count)
        || (parents
            .iter()
            .enumerate()
            .filter(|&(z, &p)| p == z as i64)
            .count())
            < 20
    {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} file {} is not the zone table",
                layout.zones, layout.zone_table
            ),
        ));
    }
    Ok(table.clone())
}

/// (zone, cell) of every berry spot the field's DLL lists, in its order, or none where it
/// lists none (spike 141).
///
/// No zone's data has them: the soil on Route 102 is a blocked cell like a tree's, with no
/// record and no move model at it, so the list never had one. The DLL keeps them all in one
/// table, found here by its shape, since a DLL's layout is the build's: the longest run of
/// `BERRY_SPOT` records whose zone and cells are in range and whose last word is one value
/// throughout, at any even offset.
pub fn berry_spots(dll: &[u8]) -> Vec<(i64, (i64, i64))> {
    let mut best: Vec<(i64, (i64, i64))> = Vec::new();
    let size = BERRY_SPOT.size();
    for start in (0..size).step_by(2) {
        let mut run: Vec<(i64, (i64, i64))> = Vec::new();
        let mut shared = 0;
        let end = start + py::floordiv(dll.len() as i64 - start, size) * size;
        for [x, z, _height, zone, word] in BERRY_SPOT
            .iter_unpack(py::slice(dll, start, end))
            .expect("the slice is whole records")
        {
            if word != 0
                && 0 < zone
                && zone < gen6_field::ZONE_LIMIT
                && x < MATRIX_CELLS
                && z < MATRIX_CELLS
            {
                if !run.is_empty() && word != shared {
                    if run.len() > best.len() {
                        best = run;
                    }
                    run = Vec::new();
                }
                shared = word;
                run.push((zone, (x, z)));
            } else if !run.is_empty() {
                if run.len() > best.len() {
                    best = run;
                }
                run = Vec::new();
            }
        }
        if run.len() > best.len() {
            best = run;
        }
    }
    if best.len() as i64 >= BERRY_SPOTS_LEAST {
        best
    } else {
        Vec::new()
    }
}

/// `struct.calcsize(form)` and `struct.unpack_from(form, record, at)[0]` for the one-field
/// little-endian formats a trainer's class is read with.
fn class_form(form: &str) -> Result<(i64, bool)> {
    match form {
        "<B" | "<b" => Ok((1, form == "<b")),
        "<H" | "<h" => Ok((2, form == "<h")),
        "<I" | "<i" => Ok((4, form == "<i")),
        _ => Err(Error::unpack("bad char in struct format")),
    }
}

fn unpack_class(form: &str, record: &[u8], at: i64) -> Result<i64> {
    let (size, signed) = class_form(form)?;
    Ok(match (size, signed) {
        (1, false) => binary::take::<1>(record, at)?[0] as i64,
        (1, true) => binary::take::<1>(record, at)?[0] as i8 as i64,
        (2, false) => u16(record, at)?,
        (2, true) => binary::s16(record, at)?,
        (4, false) => u32(record, at)?,
        _ => binary::s32(record, at)?,
    })
}

/// Each trainer's class by trainer id, and the classes that are a pair of people: those whose
/// every trainer fights doubles. ORAS's `a/0/3/6` holds a small file a trainer with the class a
/// u16 at +0x02, X's `a/0/3/8` with the class a byte at +0x01 (spike 100).
pub fn trainer_classes(rom: &Rom, layout: &RomLayout) -> Result<(Vec<i64>, HashSet<i64>)> {
    let records = garc_files(&rom.read(&layout.trainers)?)?;
    let (at, form) = (layout.trainer_class.0, layout.trainer_class.1.as_str());
    let end = at + class_form(form)?.0;
    let mut classes: Vec<i64> = Vec::new();
    for record in &records {
        classes.push(if record.len() as i64 >= end {
            unpack_class(form, record, at)?
        } else {
            0
        });
    }
    let mut battles: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for (&kind, record) in classes.iter().zip(&records) {
        if record.len() as i64 > layout.trainer_battle {
            battles
                .entry(kind)
                .or_default()
                .insert(record[layout.trainer_battle as usize] as i64);
        }
    }
    Ok((
        classes,
        battles
            .into_iter()
            .filter(|(_kind, seen)| seen.len() == 1 && seen.contains(&DOUBLE))
            .map(|(kind, _seen)| kind)
            .collect(),
    ))
}

/// Sprite -> the kind of trainer it is, for every sprite the maps show two trainers or more
/// standing as, all of one class (spike 90): "Youngster", "Hiker". When they also share one name,
/// as grunts do, the name follows the class: "Team Magma Grunt", as the game says it in battle.
///
/// Left out: a sprite whose trainers are of more than one class ("Sis & Bro" beside "Swimmer",
/// which the sprite is cannot be told); a sprite of a pair class, since "Teammates" is two people
/// and the old man drawn as one of them in town is neither; and a sprite only one trainer stands
/// as, which may be a look everybody shares that happens to battle once.
pub fn trainer_sprites(
    zones: &[Vec<u8>],
    class_of: &[i64],
    pairs: &HashSet<i64>,
    classes: &[String],
    names: &[String],
    layout: &RomLayout,
) -> Result<IndexMap<i64, String>> {
    let mut standing: IndexMap<i64, Vec<i64>> = IndexMap::new();
    for data in py::slice(zones, 0, layout.zone_table) {
        if !data.is_empty() {
            for (sprite, trainer) in gen6_field::npc_trainers(&lz11(data)?, class_of.len() as i64)?
            {
                standing.entry(sprite).or_default().push(trainer);
            }
        }
    }
    let mut words: IndexMap<i64, String> = IndexMap::new();
    for (&sprite, trainers) in &standing {
        let kinds: HashSet<i64> = trainers
            .iter()
            .map(|&trainer| class_of[trainer as usize])
            .collect();
        if trainers.len() < 2 || kinds.len() != 1 {
            continue;
        }
        let kind = *kinds.iter().next().expect("one class");
        if !(0 <= kind && kind < classes.len() as i64) {
            continue;
        }
        if pairs.contains(&kind) {
            continue;
        }
        let kind = &classes[kind as usize];
        let who: HashSet<&String> = trainers
            .iter()
            .filter(|&&trainer| trainer < names.len() as i64)
            .map(|&trainer| &names[trainer as usize])
            .collect();
        if kind.is_empty() {
            continue;
        }
        words.insert(
            sprite,
            if who.len() == 1 && who.iter().all(|name| !name.is_empty()) {
                format!("{kind} {}", who.iter().next().expect("one name"))
            } else {
                kind.clone()
            },
        );
    }
    Ok(words)
}
