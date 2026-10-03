//! `app::tool::TalkWindow`: the field's message box, and who is speaking through it.
//!
//! ```text
//! TalkWindow  +0x1c -> the `gfl::str::StrWin` whose pages are being shown
//!             +0x20    `app::tool::TalkWindowGra`, embedded
//!             +0x78 -> the `field::mmodel::MoveModel` showing them
//! ```
//!
//! Six sit in a pool 0xf4 apart. A sweep finds them by the Gra's vtable, not the TalkWindow's, so
//! an address a sweep hands over is `GRA` into the object: the window is at `GRA - 0x20 + 0x1c` and
//! the actor at `GRA + 0x58` (spikes 51 and 73).
//!
//! A talk object cannot be found by class while a talk runs: its vtable swaps to a subclass in a
//! DLL the executable read at startup does not carry (spike 51). Every one found is kept, as the
//! choice box reader keeps its device manager.
//!
//! The actor is a slot of the entity list, which says which character it is (`mmodel`).

/// What a sweep looks for. `app::tool::TalkWindow` itself is only its base between talks.
pub const CLASSES: [&str; 1] = ["N3app4tool13TalkWindowGraE"];

/// Where the Gra is embedded in the TalkWindow, and so what to subtract from a swept address.
pub const GRA: i64 = 0x20;
/// The message window being shown, from the swept address: 4 bytes before it.
pub const WINDOW: i64 = 0x1C - GRA;
/// The model speaking, from the swept address.
pub const ACTOR: i64 = 0x78 - GRA;
/// Enough of the object to hold both, read from `WINDOW`.
pub const HEAD: i64 = 0x60;
