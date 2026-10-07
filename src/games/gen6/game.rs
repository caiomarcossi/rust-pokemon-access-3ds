//! What one game keeps somewhere else than the others, for the games the gen 6 readers read.
//!
//! X and Y are the same engine as ORAS: the same classes, the same message windows and layouts, the
//! same move models and zone files (spike 100). What differs is a handful of numbers — where the ROM
//! keeps each kind of file, and one offset in a move model — and what one game has alone, as ORAS's
//! DexNav; a game package says its own in a `Game`. Everything else is shared.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::core::speech::Speech;
use crate::core::toml;
use crate::games::common::field::things::Field;
use crate::games::gen6::field::features::{
    DoorReader, Feature, HiddenReader, IceReader, SecretBaseReader,
};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::layout::RomLayout;
use crate::py::{Log, Result};

/// An app only one game has, read by that game's own package: ORAS's soaring (`oras.sky`). While
/// it runs, the navigation keys walk what it lists instead of the field's, and it is polled in
/// the field's place.
pub trait OwnApp {
    /// The class its process is (`apps.RunningApp`).
    fn app(&self) -> &'static str;

    /// What the log and the poll's phase call it.
    fn name(&self) -> &'static str;

    /// The navigation keys' categories it lists things in, after the field's.
    fn categories(&self) -> Vec<&'static str>;

    /// Another app has started, this one or any other.
    fn started(&self);

    /// What the navigation keys walk while it runs, read through its process; None while it
    /// says nothing.
    fn listed(&self, process: i64) -> Result<Option<Field>>;

    /// Say what it says of itself, every field poll while it runs.
    fn poll(&self, process: i64) -> Result<()>;
}

/// A game's `data` folder as the binary carries it: each language's `sprites.toml`, by the
/// language's code. Python's is a `Path` to the folder and `pa3ds.core.names` reads
/// `<language>/sprites.toml` from it; the Rust binary has no folder beside it, so the files are
/// `include_str!`ed.
pub type Data = &'static [(&'static str, &'static str)];

/// `Callable[[Memory], HiddenReader]`: a plain `fn`, since a `Game` is built once and kept in a
/// `static`, where an `Rc<dyn Fn>` cannot be.
pub type HiddenFactory = fn(Rc<dyn Memory>) -> Rc<dyn HiddenReader>;
/// `Callable[[Memory, bytes], SecretBaseReader]`.
pub type SecretBaseFactory = fn(Rc<dyn Memory>, &[u8]) -> Rc<dyn SecretBaseReader>;
/// `Callable[[Memory, GameText | None], Feature]`.
pub type FeatureFactory = fn(Rc<dyn Memory>, Option<&GameText>) -> Rc<dyn Feature>;
/// `Callable[[Memory, GameText | None, Speech, Callable[[str], None]], OwnApp | None]`.
pub type AppFactory =
    fn(Rc<dyn Memory>, Option<&GameText>, Rc<Speech>, Log) -> Result<Option<Rc<dyn OwnApp>>>;
/// `Callable[[Memory], DoorReader]`.
pub type DoorFactory = fn(Rc<dyn Memory>) -> Rc<dyn DoorReader>;
/// `Callable[[Memory], IceReader]`.
pub type IceFactory = fn(Rc<dyn Memory>) -> Rc<dyn IceReader>;

/// One game as the gen 6 readers read it.
///
/// Not `PartialEq`: the dataclass compares its callables by identity, which a `fn` pointer does
/// not do reliably.
#[derive(Debug, Clone)]
pub struct Game {
    /// What the log calls it.
    pub name: String,
    /// The title ids it runs as, one a version.
    pub titles: HashSet<i64>,
    /// Its words for what the game never names, a folder a language (`pa3ds.core.names`).
    pub data: Data,
    /// Where its ROM keeps the game's own words (`archives`).
    pub rom: RomLayout,
    /// Where a move model keeps the sprite it is drawn with (`mmodel.drawn_as`).
    pub drawn: i64,
    /// The reader of its hidden Pokémon, for the field to list, or None in a game with none: ORAS's
    /// DexNav (`games.oras.hidden`).
    pub hidden: Option<HiddenFactory>,
    /// (zone, a trigger's script) -> the way the conveyor belt that trigger is runs (`field`,
    /// spike 132). Nothing in a trigger's record says it is a belt, so each is known one by one.
    pub belts: IndexMap<(i64, i64), &'static str>,
    /// Sprite -> the move that clears the thing drawn as it, which Shift and P's way goes through:
    /// a bush Cut clears, a rock Rock Smash does, a boulder Strength pushes.
    pub clears: IndexMap<i64, i64>,
    /// Where the field's work holds the Dowsing Machine's record (`dowsing`), or None where it has
    /// not been measured, as in X and Y.
    pub dowsing: Option<i64>,
    /// (zone, sprite) of the move models that are a barrier's beam, three cells across the way the
    /// model faces (`field`, spike 146). The sprite is an invisible blocker of one cell elsewhere, so
    /// each zone is known one by one.
    pub barriers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) -> the category and kind it is listed as: a switch, the way
    /// between two rooms that share no floor, a trapdoor or a geyser between a floor and the hot
    /// spring under it, a lift or a teleporter between platforms, a Secret Base's way out, or the way
    /// into Seafloor Cavern, with the zone it leads to. As with the belts, nothing in the record says
    /// which.
    ///
    /// Python's value is `tuple[str, str] | tuple[str, str, int]`: the zone is the third item, None
    /// where the tuple has two.
    pub triggers: IndexMap<(i64, i64), (String, String, Option<i64>)>,
    /// (zone, a furniture record's script) -> what it is, where it is not a sign: a Secret Base's
    /// PC and flag, Mossdeep Gym's switches.
    pub furniture: IndexMap<(i64, i64), String>,
    /// The reader of its Secret Bases, given the executable's image to find every spot in, or None
    /// in a game with none: ORAS's (`games.oras.secret_base`).
    pub secret_bases: Option<SecretBaseFactory>,
    /// The readers of what else it has on its field to list, each in its own words: ORAS's berry
    /// spots and Route 113's ash piles (`games.oras.berries`, `games.oras.ash`).
    pub features: Vec<FeatureFactory>,
    /// The reader of its rotating doors, or None in a game with none: ORAS's, in Fortree Gym
    /// (`games.oras.doors`).
    pub doors: Option<DoorFactory>,
    /// The reader of its rooms of thin ice, or None in a game with none: ORAS's, in Sootopolis
    /// Gym (`games.oras.ice`).
    pub ice: Option<IceFactory>,
    /// The sprite of its invisible blocker, a move model drawn as nothing that stands in the way:
    /// listed as nobody, and solid but where it stands in for a rotating door's arm.
    pub blocker: Option<i64>,
    /// Zones whose chunks' floors are joined by stairs, each listed under exits where it meets
    /// the floor above: Sootopolis Gym's two, the way back up from its lower floor.
    pub stairs: HashSet<i64>,
    /// Its own apps' readers, each built from the memory, the ROM's words, and where to say and
    /// log, or None where what it needs is not read: ORAS's soaring (`oras.sky`).
    pub apps: Vec<AppFactory>,
    /// Warp kind -> what it is, where the kind says more than a door or stairs: ladders, a
    /// teleporter pad, the way into one building alone. A kind not here is "warp".
    pub warps: IndexMap<i64, String>,
}

impl Game {
    /// `Game(name, titles, data, rom, drawn)`: every other field at its default, set with
    /// `Game { hidden, ..Game::new(...) }`.
    pub fn new(
        name: &str,
        titles: impl IntoIterator<Item = i64>,
        data: Data,
        rom: RomLayout,
        drawn: i64,
    ) -> Self {
        Game {
            name: name.to_string(),
            titles: titles.into_iter().collect(),
            data,
            rom,
            drawn,
            hidden: None,
            belts: IndexMap::new(),
            clears: IndexMap::new(),
            dowsing: None,
            barriers: HashSet::new(),
            triggers: IndexMap::new(),
            furniture: IndexMap::new(),
            secret_bases: None,
            features: Vec::new(),
            doors: None,
            ice: None,
            blocker: None,
            stairs: HashSet::new(),
            apps: Vec::new(),
            warps: IndexMap::new(),
        }
    }

    /// The names the game's package ships for a language, read as `Names.load` reads
    /// `data / language / "sprites.toml"`: a language the package has no file for, or a file that
    /// does not parse, is no names at all. Written for the runtime's `Names.load`, which reads a
    /// folder the Rust binary does not have.
    pub fn sprites(&self, language: &str) -> IndexMap<String, String> {
        let Some((_, text)) = self.data.iter().find(|(held, _)| *held == language) else {
            return IndexMap::new();
        };
        let found = if text.starts_with('\u{feff}') {
            IndexMap::new()
        } else {
            match text.parse::<::toml::Table>() {
                Ok(loaded) => toml::table(Some(&::toml::Value::Table(loaded))),
                Err(_) => IndexMap::new(),
            }
        };
        toml::strings(found.get(crate::core::names::SPRITES))
    }
}
