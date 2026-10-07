//! Menu cursors: say the item under a menu's cursor, as the menu opens and as the cursor moves.
//!
//! A screen draws its cursor in one of nine ways, a module each, and `reader` reads them all: the
//! red corners of an app menu (`menu`), the battle's picture (`picture`), the name entry's key
//! cursor (`key`), the main menu's frame (`frame`), the opening's choice frame (`choice`), the
//! Pokédex's plate (`plate`), a cursor named by its texture (`named`), Wally's pointer (`pointer`)
//! and Sun's cursor pane (`pane`); what they share is in `shared`.

pub mod choice;
pub mod frame;
pub mod key;
pub mod menu;
pub mod named;
pub mod pane;
pub mod picture;
pub mod plate;
pub mod pointer;
pub mod reader;
pub mod shared;
