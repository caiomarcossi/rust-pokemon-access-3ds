//! What to call the things the game itself never names.
//!
//! ORAS holds no name for the people on its maps. A speaker is an overworld sprite — Norman `0x13f`
//! wherever he stands, Wally `0xb4` — and nothing in the game turns one into a word (findings, spikes
//! 52, 73, 75, 76 and 77). Signs and doors are the same: the game knows a slot on a map, not "the
//! Mart's sign".
//!
//! So pa3ds carries names of its own, under two rules that keep the "no hardcoded game text"
//! constraint meaning what it was written to mean (AGENTS.md, reopened by the user 2026-09-21):
//!
//! - **A name pa3ds invents never replaces one the game says.** Anything the game shows is read from
//!   the game, as it always was. These names fill silence, nothing else.
//! - **The player's own names win, and the player can always add one.** A name given while playing is
//!   written to the player's own file and takes precedence over the one that shipped.
//!
//! Between the two sits **the game's own word**, when the game has one after all but RAM does not
//! hold it: the rival is a work object in RAM, and the ROM names them in the player's language
//! (`games.gen6.archives`, spike 87); a door's slot is a warp in RAM, and the ROM says which
//! building some of them lead to (spike 86). So a thing is called, in order, what the player has
//! called it, what the game calls it, and what pa3ds shipped — "Rival" is only heard without a ROM.
//!
//! Two things get named, and they are keyed differently because they are different. A **sprite** is
//! one character everywhere in the game, so naming Norman once names him in every scene; the shipped
//! table is per language, beside the game's code. A **place** is a slot on one map — the zone list's
//! (map, slot) — so a sign named in Petalburg says nothing about a sign in Rustboro.
//!
//! The player's names live in `names/<language>/<rom>.toml` under their own folder, laid out the same
//! way as the shipped table beside the game's code: **TOML, per language, one file a ROM**
//! (ADR-0031). A name is a word in a language, so English names for a ROM played in French belong to
//! neither.
//!
//! The file opens with a comment saying which game, which ROM and which language it is for, because
//! `sango-1` says nothing to somebody who did not choose it and sharing names is a matter of sending
//! the file to somebody who then has to know what they have been sent. The game's own name comes from
//! the emulator's window (`pa3ds.core.window`), which reads it from the ROM, so pa3ds holds no table of
//! game names at all.
//!
//! Nothing here ever raises at a player: a names file that is missing, unreadable or malformed reads
//! as no names at all. Being told the wrong name is a nuisance; a reader that stops mid-game is not.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

use crate::core::toml;
use crate::py::{self, Result};

pub const SPRITES: &str = "sprites";
pub const PLACES: &str = "places";
pub const SECTIONS: [&str; 2] = [SPRITES, PLACES];

/// What the game itself calls a character, where RAM does not say but the ROM does.
pub type GameSprite = Rc<dyn Fn(i64) -> Option<String>>;
/// What the game itself calls a slot on a map: the building behind a door.
pub type GamePlace = Rc<dyn Fn(i64, i64) -> Option<String>>;

/// `f"{number:#x}"`: Python writes a negative number's sign before the `0x`.
fn hex(number: i64) -> String {
    if number < 0 {
        format!("-{:#x}", number.unsigned_abs())
    } else {
        format!("{number:#x}")
    }
}

/// How a character is written in a names file: `0x13f`.
pub fn sprite_key(character: i64) -> String {
    hex(character)
}

/// What each kind of slot is written as in a names file, by its tag (`things.SOURCE_KIND`): a
/// warp's slot 3 is `warp:3`. A word, so a person editing the file can tell a door from a sign
/// (Orion, 2026-10-11); a kind renamed in code keeps its word here, or its names stop matching.
pub const KINDS: [&str; 15] = [
    "furniture",
    "warp",
    "tile",
    "edge",
    "berry",
    "trigger",
    "base",
    "ash",
    "bike",
    "dive",
    "stairs",
    "app",
    "decoration",
    "model",
    "current",
];

/// A slot as names files wrote it until 2026-10-11, its kind's tag and its number one sum:
/// `0x10:4294967299`, Rustboro's warp 3.
static OLD_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(0x[0-9a-f]+):(\d+)$").expect("a fixed pattern"));

/// How a slot on a map is written in a names file: `0x10:warp:3`, map, kind and number. A slot
/// is never negative, so Python's `KINDS[kind]` never counts from the end here.
pub fn place_key(map_id: i64, slot: i64) -> String {
    let (kind, number) = (slot >> 32, slot & 0xFFFFFFFF);
    let word = match usize::try_from(kind).ok().and_then(|at| KINDS.get(at)) {
        Some(word) => word.to_string(),
        None => kind.to_string(),
    };
    format!("{}:{word}:{number}", hex(map_id))
}

/// A key in the form names files use now, from either form.
fn readable(key: &str) -> Result<String> {
    let Some(old) = OLD_KEY.captures(key) else {
        return Ok(key.to_string());
    };
    Ok(place_key(
        py::int_base16(&old[1])?,
        py::int_base10(&old[2])?,
    ))
}

/// `first or second()`: a name that is there but empty is no name at all.
fn either(first: Option<String>, second: impl FnOnce() -> Option<String>) -> Option<String> {
    match first {
        Some(first) if !first.is_empty() => Some(first),
        _ => second(),
    }
}

/// The names pa3ds has for what the game leaves unnamed, the player's ahead of the shipped.
pub struct Names {
    shipped: IndexMap<String, String>,
    path: Option<PathBuf>,
    /// What to write at the top of the player's file: the game and the language, which the
    /// file cannot work out for itself and a person reading it cannot either.
    header: IndexMap<String, String>,
    game_sprite: RefCell<GameSprite>,
    game_place: RefCell<GamePlace>,
    mine: RefCell<IndexMap<String, IndexMap<String, String>>>,
}

impl Default for Names {
    fn default() -> Self {
        Names::empty()
    }
}

impl Names {
    pub fn new(
        shipped: Option<&IndexMap<String, String>>,
        mine_path: Option<&Path>,
        header: Option<&IndexMap<String, String>>,
    ) -> Result<Self> {
        let mut mine: IndexMap<String, IndexMap<String, String>> = SECTIONS
            .iter()
            .map(|section| (section.to_string(), IndexMap::new()))
            .collect();
        let mut migrated = false;
        if let Some(mine_path) = mine_path {
            let found = toml::read(mine_path);
            mine = SECTIONS
                .iter()
                .map(|section| (section.to_string(), toml::strings(found.get(*section))))
                .collect();
            let mut places: IndexMap<String, String> = IndexMap::new();
            for (key, said) in &mine[PLACES] {
                places.insert(readable(key)?, said.clone());
            }
            // An IndexMap's `==` ignores order, as a dict's does.
            if places != mine[PLACES] {
                // Written the old way: kept the new way from now on, read once a game is played.
                mine[PLACES] = places;
                migrated = true;
            }
        }
        let names = Names {
            shipped: shipped.cloned().unwrap_or_default(),
            path: mine_path.map(Path::to_path_buf),
            header: header.cloned().unwrap_or_default(),
            game_sprite: RefCell::new(Rc::new(|_character| None)),
            game_place: RefCell::new(Rc::new(|_map, _slot| None)),
            mine: RefCell::new(mine),
        };
        if migrated {
            names.write()?;
        }
        Ok(names)
    }

    /// `Names()`: no shipped names, no file, no header.
    pub fn empty() -> Self {
        Names::new(None, None, None).expect("nothing is read without a file")
    }

    /// The names for one game, one language and one player.
    ///
    /// `data/<language>/sprites.toml` from the game's own package, and the player's
    /// `names/<language>/<fingerprint>.toml` under `home`. `title` is what the emulator says it
    /// is running, for the comment at the top of that file; without it the ROM names itself.
    pub fn load(
        data: &Path,
        language: &str,
        fingerprint: &str,
        home: Option<&Path>,
        title: &str,
    ) -> Result<Names> {
        let shipped =
            toml::strings(toml::read(&data.join(language).join("sprites.toml")).get(SPRITES));
        let mine = home.map(|home| {
            home.join("names")
                .join(language)
                .join(format!("{fingerprint}.toml"))
        });
        let header = IndexMap::from([
            (
                "game".to_string(),
                if title.is_empty() { fingerprint } else { title }.to_string(),
            ),
            ("rom".to_string(), fingerprint.to_string()),
            ("language".to_string(), language.to_string()),
        ]);
        Names::new(Some(&shipped), mine.as_deref(), Some(&header))
    }

    /// Ask the game for its own word for a character and for a slot on a map, ahead of the
    /// shipped table and behind the player's own names. Set once the game's text has been
    /// read.
    pub fn use_game(&self, sprite: Option<GameSprite>, place: Option<GamePlace>) {
        if let Some(sprite) = sprite {
            *self.game_sprite.borrow_mut() = sprite;
        }
        if let Some(place) = place {
            *self.game_place.borrow_mut() = place;
        }
    }

    /// What to call this character, or None when nobody has named it: the player's name, the
    /// game's own word, or the shipped table's, in that order.
    pub fn sprite(&self, character: i64) -> Option<String> {
        let key = sprite_key(character);
        let mine = self.mine.borrow()[SPRITES].get(&key).cloned();
        either(mine, || {
            let game_sprite = self.game_sprite.borrow().clone();
            either(game_sprite(character), || self.shipped.get(&key).cloned())
        })
    }

    /// What to call this slot on this map, or None: the player's name, then the game's.
    pub fn place(&self, map_id: i64, slot: i64) -> Option<String> {
        let mine = self.mine.borrow()[PLACES]
            .get(&place_key(map_id, slot))
            .cloned();
        either(mine, || {
            let game_place = self.game_place.borrow().clone();
            game_place(map_id, slot)
        })
    }

    /// Give this character a name, and keep it. An empty name gives it back to the game's own
    /// word, the shipped table, or silence.
    pub fn name_sprite(&self, character: i64, said: &str) -> Result<()> {
        self.keep(SPRITES, &sprite_key(character), said)
    }

    /// Give this slot on this map a name, and keep it. An empty name gives it back to the
    /// game's own word, or to silence.
    pub fn name_place(&self, map_id: i64, slot: i64, said: &str) -> Result<()> {
        self.keep(PLACES, &place_key(map_id, slot), said)
    }

    /// The player's own names, as they would be written.
    pub fn mine(&self) -> IndexMap<String, IndexMap<String, String>> {
        self.mine.borrow().clone()
    }

    fn keep(&self, section: &str, key: &str, said: &str) -> Result<()> {
        let said = crate::py::strip(said);
        {
            let mut mine = self.mine.borrow_mut();
            if !said.is_empty() {
                mine[section].insert(key.to_string(), said.to_string());
            } else {
                mine[section].shift_remove(key);
            }
        }
        self.write()
    }

    /// The player's names as the file they are written to, comment and all.
    pub fn render(&self) -> String {
        let header = |key: &str| self.header.get(key).map(String::as_str).unwrap_or("");
        let rom = header("rom");
        let language = header("language");
        let game = header("game");
        let mut lines = toml::comment([
            format!(
                "Names you have given in {}.",
                if game.is_empty() { "this game" } else { game }
            ),
            [
                if rom.is_empty() {
                    String::new()
                } else {
                    format!("ROM {rom}.")
                },
                if language.is_empty() {
                    String::new()
                } else {
                    format!("Language {language}.")
                },
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
            String::new(),
            "pa3ds writes this file when you rename something while playing. You can edit it by"
                .to_string(),
            "hand too, and sending it to somebody else gives them your names.".to_string(),
            String::new(),
            "[sprites] is a character: everyone drawn alike everywhere in the game, by the number"
                .to_string(),
            "the game counts them with. [places] is a slot on one map, written map:kind:number,"
                .to_string(),
            "so a sign named here says nothing about a sign on another map; one person or"
                .to_string(),
            "boulder named alone is a slot of its map too, of the kind model.".to_string(),
        ]);
        for section in SECTIONS {
            lines.push(String::new());
            lines.push(format!("[{section}]"));
            lines.extend(toml::pairs(&self.mine.borrow()[section]));
        }
        lines.join("\n") + "\n"
    }

    fn write(&self) -> Result<()> {
        if let Some(path) = &self.path {
            toml::write(path, &self.render())?;
        }
        Ok(())
    }
}
