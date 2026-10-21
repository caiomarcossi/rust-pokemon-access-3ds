//! Pokémon Sun and Moon: another engine, gfl2, read by ORAS's dialogue reader where the two agree
//! and by readers of its own where they do not (spike 111).
//!
//! What agrees: the ROM's text files, a message window's fields, the string buffer's and the
//! control sequences that break a message into pages. What does not: the classes' names, a
//! TextBox's fields, where the heap is, and a speaker the game names itself. Moon shares Sun's
//! definitions. Ultra shares the readers with the field differences described by `ultra` below.

use std::collections::HashSet;
use std::sync::OnceLock;

pub const SUN: i64 = 0x0004000000164800;
pub const MOON: i64 = 0x0004000000175E00;
pub const ULTRA_SUN: i64 = 0x00040000001B5000;
pub const ULTRA_MOON: i64 = 0x00040000001B5100;

/// Sun or Moon, as the catalog tells it from the title running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SunMoon {
    /// What the log calls it.
    pub name: String,
    /// The title ids it runs as, one a version.
    pub titles: HashSet<i64>,
}

/// `GAME`: a function, since a `String` and a `HashSet` cannot be built in a constant.
pub fn game() -> &'static SunMoon {
    static GAME: OnceLock<SunMoon> = OnceLock::new();
    GAME.get_or_init(|| SunMoon {
        name: "Sun and Moon".into(),
        titles: HashSet::from([SUN, MOON]),
    })
}

/// Shares Sun's readers, with Ultra Sun's measured field offsets and place-name file. Ultra Moon
/// provisionally uses the same definitions at the fork owner's request (2026-10-21); its field
/// layout has not been independently measured. Other structures still use Sun's definitions.
///
/// `ULTRA`: a function for the same reason as [`game`].
pub fn ultra() -> &'static SunMoon {
    static ULTRA: OnceLock<SunMoon> = OnceLock::new();
    ULTRA.get_or_init(|| SunMoon {
        name: "Ultra Sun and Ultra Moon".into(),
        titles: HashSet::from([ULTRA_SUN, ULTRA_MOON]),
    })
}

pub mod apps;
pub mod battle;
pub mod camera;
pub mod field;
pub mod heaps;
pub mod langselect;
pub mod lyt;
pub mod msgwin;
pub mod naming;
pub mod profile;
pub mod session;
pub mod talk;
pub mod text;
pub mod walls;
pub mod zones;
