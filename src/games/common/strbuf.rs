//! `gfl::str::StrBuf`: the game's string buffer, which holds a message, a label or a battler's name.
//!
//! ```text
//! +0x04 -> UTF-16LE text, control sequences inline (`text`)    +0x0a u16 length in halfwords
//! ```
//!
//! Read in spikes 8, 18 and 57. The dialogue reader parses a message's pages out of one, the list and
//! choice box readers an option's label, the battle reader a name ending in a gender sign.

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::games::common::text::pages;
use crate::py::{Kind, Result};

pub const CLASS: &str = "N3gfl3str6StrBufE";
pub const TEXT: i64 = 0x04;
pub const LENGTH: i64 = 0x0A;
pub const HEAD: i64 = 0x0C;

/// The buffer's text as UTF-16LE bytes, or b"" when its length is 0 or over `limit` halfwords.
///
/// `limit` only rejects garbage lengths: a read of a buffer that is not one, or not yet one.
pub fn raw(memory: &dyn Memory, buffer: i64, limit: i64) -> Result<Vec<u8>> {
    let head = memory.read(buffer, HEAD)?;
    let length = u16(&head, LENGTH)?;
    if !(0 < length && length <= limit) {
        return Ok(Vec::new());
    }
    memory.read(u32(&head, TEXT)?, length * 2)
}

/// The words in a buffer holding one short label, control codes dropped, or None.
///
/// `label(memory, buffer)` is the call with the default limit of 256 halfwords.
pub fn label(memory: &dyn Memory, buffer: i64) -> Result<Option<String>> {
    label_with(memory, buffer, 256)
}

/// `label(memory, buffer, limit)`.
pub fn label_with(memory: &dyn Memory, buffer: i64, limit: i64) -> Result<Option<String>> {
    let text = raw(memory, buffer, limit)?;
    if text.is_empty() {
        return Ok(None);
    }
    let spoken = match pages(&text) {
        Ok(spoken) => spoken,
        Err(error) if error.is(Kind::ValueError) => return Ok(None),
        Err(error) => return Err(error),
    };
    let words: Vec<&str> = spoken
        .iter()
        .filter(|page| !page.text.is_empty())
        .map(|page| page.text.as_str())
        .collect();
    let joined = words.join(" ");
    Ok(if joined.is_empty() {
        None
    } else {
        Some(joined)
    })
}
