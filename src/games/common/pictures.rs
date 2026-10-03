//! Words for what the game draws without any: buttons, icons and signs that are pictures alone.
//!
//! A picture's words are in its texture's name, `fight_button.bclim`, or in its pane's name when
//! the layout file is not read, Sun's `Btn_Fight` (spike 124); a sign the game never names, a Box
//! mark, is said as the Unicode sign it draws, which the screen reader names. Every such name is
//! here, so a screen's picture is said the same way wherever a reader meets it.
//!
//! **English only, for now** (Orion, 2026-09-29). A texture's name is an English word whatever the
//! ROM's language, and "empty", "on" and "off" are pa3ds's own, so a player in French hears "fight"
//! and "female" as written here. A table per language, in code or in a data file beside the names
//! (`pa3ds.core.names`), is where they would go when someone asks; the signs need none.

use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::py;

/// The word a texture name ends in to say its picture is a button. Left unsaid.
pub const BUTTON: &str = "button";
/// Words a texture name starts with to say what its picture is or belongs to, left unsaid: an
/// icon, a move's ("waza", Japanese for move) or a contest's. The name entry's confirm key is
/// `icon_return`, and reads "return" (opening.ctm, the user, 2026-09-18); the move-forget screen's
/// `waza_con_change` reads "change" (Orion, 2026-10-01), not "waza con change".
pub const KINDS: [&str; 3] = ["icon", "waza", "con"];
/// The word a designer starts a button's pane name with, as the battle's `Btn_Fight` (spike 124):
/// what the button is, not what it says.
pub const BUTTON_PREFIX: &str = "btn";

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

/// A texture's name as words: `fight_button.bclim` is "fight".
///
/// A trailing "button", or a leading "icon" (`KINDS`), is the designer saying what the picture is,
/// not part of the item's name. The battle's command menu reads "fight bag pokémon run" as any
/// menu would (the user, 2026-09-16), and the name entry's confirm key `icon_return.bclim` reads
/// "return". A texture called only `button` or `icon` keeps the word: dropping it leaves nothing.
pub fn words(texture: &str) -> String {
    let stem = match texture.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => texture,
    };
    let replaced = stem.replace('_', " ");
    let mut said = py::split(&replaced);
    if said.len() > 1 && said[said.len() - 1] == BUTTON {
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
