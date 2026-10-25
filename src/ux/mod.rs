//! The windows pa3ds opens for the player: `window`, pa3ds's own, opened when it is
//! double-clicked, `dialog`, the box asking for a name, and `folder`, the folder picker.
//!
//! Native Windows controls only, which a screen reader reads as it reads any program's (spike 81),
//! each window on a thread of its own so the poll loop never waits on one.

pub mod dialog;
pub mod folder;
pub mod settings;
pub mod window;
