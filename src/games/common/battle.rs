//! A battle as every game's battle reader says it: a side (`Battler`), the signs its name ends
//! in, and what a gauge leads to.
//!
//! Gen 6 and Sun draw their HP boxes otherwise and find their gauges otherwise (`gen6.screens.battle`,
//! `sm.battle`); what a gauge holds of the Pokémon it shows is the same in both, the battle record
//! at +0x0c and its `PokemonParam` at the record's +0x00 (spikes 66 and 124).
//!
//! Bound by:
//! - ADR-0015: each side said whole as its HP changes, the enemy's as a percentage, and again on R
//!   and shift and R.

pub const MALE: &str = "♂";
pub const FEMALE: &str = "♀";
/// The official games' battles write the real signs, which a screen reader says. The Chinese fan
/// Omega Ruby's writes the font's private-use glyphs, U+E08E for ♂, which `speech::symbol` maps
/// (movie2_route103 on it, 2026-10-18).
pub const SIGNS: [&str; 2] = [MALE, FEMALE];

/// Asks, one a screen poll, between sweeps asked for while a gauge is still missing: a second.
pub const REASK: i64 = 5;
/// Sweeps asked for a box with no gauge before giving up on it. Asked for as long as the box was
/// up, a battle whose gauge class was never learned cost 75 whole walks, 31.8 s of 171, where the
/// scheduled ones were 10 (sun_9.ctm, before `Discovery.learn` read the battle's module).
pub const MAX_ASKS: i64 = 3;

/// -> the battle record of the Pokémon a gauge shows, whose +0x00 is its PokemonParam (spike 66).
pub const GAUGE_RECORD: i64 = 0x0C;
pub const RECORD_PARAM: i64 = 0x00;

/// One side of the battle as it should be said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battler {
    pub panel: i64,
    pub text: String,
    /// True for the player's side, which is the panel that shows a number over a number.
    pub player: bool,
}
