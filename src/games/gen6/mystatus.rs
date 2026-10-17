//! The save's trainer record, `savedata::MyStatus`: which language the game is in (spike 226).
//!
//! ```text
//! savedata::MyStatus   (in the heap from before the title screen)
//!     +0x08  the save's trainer record, whole (PKHeX's format):
//!            +0x04 u8 the version, 0x1b Omega Ruby, 0x18 X
//!            +0x2d u8 the language, so the object's +0x35
//!            +0x48 the player's name, UTF-16
//! ```
//!
//! The language is the one the player chose as their game began, or on a new game the one it starts
//! with: 1 Japanese, 2 English, 3 French, 4 Italian, 5 German, 7 Spanish, 8 Korean, and in Sun, whose
//! record pa3ds cannot reach, 9 and 10 Chinese. A fan translation keeps another language's number:
//! the Chinese Omega Ruby says Japanese, whose text it replaced.

use crate::core::discovery::Found;

pub const CLASS: &str = "N8savedata8MyStatusE";
pub const LANGUAGE: i64 = 0x35;
/// The record's number for a language -> its code, the name of its folders under `data/` and its
/// key in a game's text archives.
pub const LANGUAGES: [(u8, &str); 9] = [
    (1, "ja"),
    (2, "en"),
    (3, "fr"),
    (4, "it"),
    (5, "de"),
    (7, "es"),
    (8, "ko"),
    (9, "zh-CN"),
    (10, "zh-TW"),
];

/// The language the save says, out of the pages a sweep has already read, or None where the sweep
/// found no record or it holds no language pa3ds knows.
pub fn language(found: &Found) -> Option<&'static str> {
    for address in found.of(&[CLASS]) {
        let Some(byte) = found.read(address + LANGUAGE, 1) else {
            continue;
        };
        if let Some((_, code)) = LANGUAGES.iter().find(|(number, _)| *number == byte[0]) {
            return Some(code);
        }
    }
    None
}
