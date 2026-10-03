//! Message windows: `print::MsgWin` and its base `gfl::str::StrWin`, which game text is typed in.
//!
//! ```text
//! +0x04 -> line records, 0x68 bytes each, the first word of each the line's `nw::lyt::TextBox`
//! +0x08 -> the message's `gfl::str::StrBuf`
//! +0x14 -> its layout's entry in a layout pool's table (spike 64)
//! +0x20 u8 line count (the byte after it is set in live windows, so read it as a byte)
//! +0x9c the `print::XYTagProc` the window embeds and lays its own lines out with
//! ```
//!
//! The setter is at 0x003a6f10 (spike 8), the line appender at 0x003a65d0 (spike 10). Windows hand
//! their line objects on: in the opening, a closed window's records still point at TextBoxes a new
//! window is typing into (spike 17), and only whose tag processor they use tells the two apart.

use crate::core::rtti::ClassNames;
use crate::py::Result;

pub const CLASSES: [&str; 2] = ["N5print6MsgWinE", "N3gfl3str6StrWinE"];
pub const LINES: i64 = 0x04;
pub const MESSAGE: i64 = 0x08;
pub const LAYOUT_HANDLE: i64 = 0x14;
pub const LINE_COUNT: i64 = 0x20;
pub const HEAD: i64 = 0x24;
pub const LINE_RECORD: i64 = 0x68;
pub const MAX_LINES: i64 = 8;
pub const TAG_PROCESSOR: i64 = 0x9C;

/// The message window whose embedded tag processor this is, or None.
///
/// A line object names its tag processor, so this is how a TextBox leads to the window typing into
/// it. A tag below 0x9c is garbage from a torn read; taken as a window's, it would be an address
/// below zero (spike 60). `classes` are the game's message window classes: Sun's have other names
/// and embed theirs at the same place (spike 111). Python's default for `classes` is [`CLASSES`].
pub fn window_of(names: &ClassNames, tag: i64, classes: &[&str]) -> Result<Option<i64>> {
    if tag < TAG_PROCESSOR {
        return Ok(None);
    }
    let window = tag - TAG_PROCESSOR;
    let class = names.of_object(window)?;
    Ok(class
        .is_some_and(|class| classes.contains(&class.as_str()))
        .then_some(window))
}
