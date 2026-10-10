//! Words for what the game draws without any: buttons, icons and signs that are pictures alone.
//!
//! A picture's words are in its texture's name, `fight_button.bclim`, or in its pane's name when
//! the layout file is not read, Sun's `Btn_Fight` (spike 124); a sign the game never names, a Box
//! mark, is said as the Unicode sign it draws, which the screen reader names. Every such name is
//! here, so a screen's picture is said the same way wherever a reader meets it.
//!
//! **English only, for now** (Orion, 2026-09-29; on 2026-10-06 he was "increasingly convinced we
//! need to start making per-language tables"). A texture's name is an English word whatever the
//! ROM's language, and "empty", "on" and "off" are pa3ds's own, so a player in French hears "fight"
//! and "female" as written here. A table per language, in code or in a data file beside the names
//! (`pa3ds.core.names`), is where they would go when someone asks; the signs need none.

use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::py;

/// The word a texture name ends in to say its picture is a button, or `btn` short for it, as the
/// PlayNav's `return_btn` and the Pokédex's `up_btn` (movie24). Left unsaid.
pub const BUTTON: &str = "button";
pub const ICON: &str = "icon";
/// Words a texture name starts with to say what its picture is or belongs to, left unsaid: an
/// icon, a move's ("waza", Japanese for move) or a contest's. The name entry's confirm key is
/// `icon_return`, and reads "return" (opening.ctm, the user, 2026-09-18); the move-forget screen's
/// `waza_con_change` reads "change" (Orion, 2026-10-01), not "waza con change".
pub const KINDS: [&str; 3] = [ICON, "waza", "con"];
/// The word a designer starts a button's pane name with, as the battle's `Btn_Fight` (spike 124):
/// what the button is, not what it says.
pub const BUTTON_PREFIX: &str = "btn";

/// Pictures whose texture's name says what they look like, not what they do, or says it in
/// Japanese, and that the game never names, by pa3ds's words for what they do (Orion, 2026-10-01):
/// the AreaNav's map draws `map_big` on the button that makes the map bigger, and `map_small` on
/// the one that takes it back (movie17); the Bag's `rireki_icon` ("rireki", history) lists the last
/// items used (Orion, 2026-10-02, movie21); the PC Box's three modes, `mode01` to `mode03`, are its
/// red tab (a Poké Ball), its blue (two balls and an arrow both ways) and its green (a grid and an
/// arrow), which as Gen 6's PC is known pick a Pokémon up as usual, move one with a press, and
/// select a group (Orion, 2026-10-02, movie22). The DexNav's crown for an area's Pokémon caught,
/// `complete0` to `complete3`, is each by its colour, decoded from the ROM: brown, grey, yellow and
/// a paler grey, bronze to platinum; tapped, it says "Good job! You've caught them all!" (Orion,
/// movie28: the gold for Route 101, all of whose Pokémon are in the grass).
pub fn own_words() -> &'static IndexMap<&'static str, &'static str> {
    static OWN_WORDS: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    OWN_WORDS.get_or_init(|| {
        IndexMap::from([
            ("map_big", "zoom in"),
            ("map_small", "zoom out"),
            ("rireki_icon", "history"),
            ("mode01", "normal mode"),
            ("mode02", "swap mode"),
            ("mode03", "multi-select mode"),
            ("complete0", "bronze crown"),
            ("complete1", "silver crown"),
            ("complete2", "gold crown"),
            ("complete3", "platinum crown"),
        ])
    })
}

/// Words a texture name spells otherwise. A file name is ASCII, so `pokemon_button.bclim` loses the
/// accent the game itself writes everywhere else ("Choose a Pokémon."), and a screen reader says
/// the two differently (the user, 2026-09-16); `intro_famale.bclim`, which names the girl on the
/// opening's choice of boy or girl, is Game Freak's own misspelling, and a screen reader reads it
/// aloud as written (the user, 2026-09-21). Each is the word the picture is of, spelt as it is
/// spelt elsewhere; none is a name pa3ds invented.
pub fn spellings() -> &'static IndexMap<&'static str, &'static str> {
    static SPELLINGS: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    SPELLINGS.get_or_init(|| IndexMap::from([("pokemon", "pokémon"), ("famale", "female")]))
}

/// Said of a button that is only the plate other buttons' words are written on, and of a slot of
/// the PC Box holding no Pokémon.
pub const EMPTY: &str = "empty";

/// The sign each mark's picture shows, which a screen reader names in the player's own words, as
/// it does the gender signs. The game draws the marks and never names them.
pub fn marks() -> &'static IndexMap<&'static str, &'static str> {
    static MARKS: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    MARKS.get_or_init(|| {
        IndexMap::from([
            ("box_mark_01", "●"),
            ("box_mark_02", "▲"),
            ("box_mark_03", "■"),
            ("box_mark_04", "♥"),
            ("box_mark_05", "★"),
            ("box_mark_06", "◆"),
        ])
    })
}
pub const ON: &str = "on";
/// A mark set or not: the game says so only by drawing it solid.
pub const OFF: &str = "off";

pub const STAR: &str = "★";
pub const NO_STAR: &str = "☆";
pub const SPECIAL: &str = "!";
pub const HEART: &str = "♥";
/// A heart hidden under the words of what it counts: one point of it not had (`screen`).
pub const NO_HEART: &str = "♡";
/// What a picture shows, where the DexNav draws it in place of words: a star of Potential filled
/// or not, three in a row, and the red mark beside what is better than usual, its level, its first
/// move, its ability (spike 195); and where a contest does, a heart of a move's appeal, one a
/// point (spike 196). Signs for the screen text reader to say in words of its own (`stars`,
/// `hearts`, the `SPECIAL` marks), never said as signs.
pub const DRAWN: [(&str, &str); 4] = [
    ("star01", STAR),
    ("star02", NO_STAR),
    ("sp_mark", SPECIAL),
    ("heart", HEART),
];
/// A contest's places, which its results draw as pictures beside each entrant, `one` to `four`
/// (spike 196), in words of pa3ds's own, said after the entrant: "Raiden & Metagross third
/// place" (Orion, 2026-10-09).
pub const PLACES: [(&str, &str); 4] = [
    ("one", "first place"),
    ("two", "second place"),
    ("three", "third place"),
    ("four", "fourth place"),
];
pub const HIGHER_LEVEL: &str = "higher level";
pub const EGG_MOVE: &str = "egg move";
/// What the DexNav's mark says beside a level above the area's, a first move that is an egg move,
/// and the hidden ability, as the guides read it (docs/plans/dexnav.md); the game says none of it.
pub const HIDDEN_ABILITY: &str = "hidden ability";
/// Said after a Pokémon the DexNav shows in colour, caught, where the rest are its silhouettes:
/// only the caught are said so (Orion, 2026-10-06).
pub const CAUGHT: &str = "caught";

/// `DRAWN.get(texture)`.
pub fn drawn(texture: &str) -> Option<&'static str> {
    DRAWN
        .iter()
        .find(|(held, _)| *held == texture)
        .map(|(_, sign)| *sign)
}

/// `PLACES.get(texture)`.
pub fn place(texture: &str) -> Option<&'static str> {
    PLACES
        .iter()
        .find(|(held, _)| *held == texture)
        .map(|(_, words)| *words)
}

/// Potential's stars as words: "2 stars", "1 star", "0 stars".
pub fn stars(filled: i64) -> String {
    if filled == 1 {
        format!("{filled} star")
    } else {
        format!("{filled} stars")
    }
}

/// A contest move's hearts as words: "4 hearts", "1 heart".
pub fn hearts(count: i64) -> String {
    if count == 1 {
        format!("{count} heart")
    } else {
        format!("{count} hearts")
    }
}

/// A texture's name as words: `fight_button.bclim` is "fight".
///
/// A trailing "button", or a leading or trailing "icon" (`KINDS`), is the designer saying what
/// the picture is, not part of the item's name. The battle's command menu reads "fight bag
/// pokémon run" as any menu would (the user, 2026-09-16), the name entry's confirm key
/// `icon_return.bclim` reads "return", and so does the AreaNav's `return_icon.bclim` (movie17).
/// A texture called only `button` or `icon` keeps the word: dropping it leaves nothing.
pub fn words(texture: &str) -> String {
    let stem = match texture.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => texture,
    };
    if let Some(own) = own_words().get(stem) {
        return own.to_string();
    }
    let replaced = stem.replace('_', " ");
    let mut said = py::split(&replaced);
    if said.len() > 1 && [BUTTON, BUTTON_PREFIX, ICON].contains(&said[said.len() - 1]) {
        said.pop();
    }
    while said.len() > 1 && KINDS.contains(&said[0]) {
        said.remove(0);
    }
    said.iter()
        .map(|word| spellings().get(*word).copied().unwrap_or(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A button's pane name as words: `Btn_Fight` is "fight", `Btn_Pokemon_00` "pokémon". None
/// for a name with no words left.
pub fn pane_words(name: &str) -> Option<String> {
    let mut parts: Vec<String> = name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_lowercase())
        .collect();
    if !parts.is_empty() && parts[0] == BUTTON_PREFIX {
        parts.remove(0);
    }
    while parts
        .last()
        .is_some_and(|last| last.chars().all(py::is_digit))
    {
        parts.pop();
    }
    if parts.is_empty() {
        None
    } else {
        Some(words(&parts.join(" ")))
    }
}

/// The sign of a Box mark's texture, or None for any other texture.
pub fn mark(texture: &str) -> Option<&'static str> {
    let stem = match texture.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => texture,
    };
    marks().get(stem).copied()
}
