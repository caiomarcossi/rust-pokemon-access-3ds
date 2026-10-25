//! The map as every game lists it: what is on it worth walking to, and the ground under it.
//!
//! Gen 6 reads these out of its zone files, move models and chunks (`gen6.field`), Sun out of its
//! actors and its walls (`sm.field`); what a player walks the list of, and where pathfinding may
//! walk, is the same in both:
//!
//! - `things`: **`Interactable`**, one thing worth walking to, and **`Field`**, where the player
//!   is with everything around them; a place's **source**, which table it was read from, over its
//!   own number (`SOURCE_KIND`); the categories; `describe` puts a listing in the log.
//! - `chunk`: a **chunk**'s cell table (`Chunk`, the one gen 6's chunks keep and Sun's walls are
//!   laid out as, spike 114), its floors, and what each kind of cell is.
//! - `ground`: **`Ground`**, where the player can walk: the cells of the chunks loaded, less where
//!   the matrix has no zone (gen 6's `Grid`), or a rail's corridor instead (X's Lumiose,
//!   `gen6.field.rail`); rooms of thin ice and rotating doors.
//! - `navigate` answers the navigation keys (`keys`), and `guide` finds the way and follows the
//!   player along it.

pub mod chunk;
pub mod climb;
pub mod ground;
pub mod guide;
pub mod keys;
pub mod navigate;
pub mod pad;
pub mod things;
