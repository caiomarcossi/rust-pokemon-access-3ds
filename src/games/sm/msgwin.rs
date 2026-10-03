//! Sun's message windows and the TextBoxes they type into, as far as they differ from ORAS's.
//!
//! A message window is `print::MessageWindow`, and a closed one drops to its base
//! `gfl2::str::StrWin`, as ORAS's `print::MsgWin` drops to `gfl::str::StrWin`. Every field the
//! dialogue reader reads of a window is where ORAS keeps it (`games.common.msgwin`): the line
//! records at +0x04, 0x68 apart, the message at +0x08, the line count at +0x20 and the tag
//! processor it embeds at +0x9c. A TextBox is still `nw::lyt::TextBox`, and holds each field 0x30
//! before ORAS's does (`sm.lyt`):
//!
//! ```text
//! TextBox   +0xa4 -> text    +0xc8 -> tag processor    +0xcc u16 capacity    +0xce u16 length
//! ```
//!
//! Read in spike 111, over Kukui's call and Mom's first lines: "It's so warm and bright here!"
//! typed into the first of Mom's two TextBoxes, 33 halfwords with its closing control sequence, 55
//! room.

use crate::games::common::dialogue::Shapes;
use crate::games::sm::lyt::SHAPE;

pub const CLASSES: [&str; 2] = ["N5print13MessageWindowE", "N4gfl23str6StrWinE"];
pub const TEXT: i64 = SHAPE.text;
pub const TAG_PROCESSOR: i64 = SHAPE.tag_processor;
pub const CAPACITY: i64 = SHAPE.capacity;
pub const LENGTH: i64 = SHAPE.length;

pub const SHAPES: Shapes = Shapes {
    windows: &CLASSES,
    text: TEXT,
    tag_processor: TAG_PROCESSOR,
    capacity: CAPACITY,
    length: LENGTH,
};
