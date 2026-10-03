//! The games: what to look for in memory, how to read it, and what each game leaves unnamed.
//!
//! The core (`pa3ds.core`) knows nothing about Pokémon; everything that does is here, a package for
//! each thing the games share and a package a game (ADR-0006):
//!
//! - `common`: what every game is read the same way in, given its game's shapes.
//! - `gen6`: the engine X, Y and ORAS share, its profile and readers, run for all three and told
//!   apart by a `gen6.game.Game` (spike 100).
//! - `oras` and `xy`: each game's own numbers, its `Game`, what it alone has, and its `data/`, the
//!   words it never says itself.
//! - `sm`: Sun and Moon, another engine with a profile of its own (spike 111).
//!
//! `catalog` says which is running.

pub mod catalog;
pub mod common;
pub mod gen6;
pub mod oras;
pub mod sm;
pub mod xy;
