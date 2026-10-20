//! `gfl::heap::CTRExpHeap`: Game Freak's heap, the SDK's expanded heap inside it, and what of a
//! block it has just allocated is worth sweeping (spike 110).
//!
//! ```text
//! CTRExpHeap  +0x00 vtable    +0x04 the heap's id    +0x08 blocks in use
//!             +0x38 the SDK's head, which `pa3ds.core.heaps` reads
//! block memory +0x00 the heap's id    +0x04 this header's length, 0x20    then the object
//! ```
//!
//! Every message window, name buffer, layout work, heap and Pokémon record in spike 99's captures
//! starts right after the game's header, at its block's +0x30. What sits deeper is in a block holding
//! several: the move models' pool of 88 KB, the talk windows' pool, the camera's block, and name
//! buffers up to 0x1348 into a block of 256 KB.
//!
//! Bound by:
//! - ADR-0034: a new block is swept as it appears and 0.5 s and 2 s on, up to 128 KiB of it.

use crate::core::heaps::Block;

pub const CLASS: &str = "N3gfl4heap10CTRExpHeapE";
/// Where the SDK's head is in the heap object.
pub const HEAD: i64 = 0x38;
/// The game's header in a block, after the SDK's and before the object.
pub const HEADER: i64 = 0x20;
/// The heap's count of blocks in use: it moves when a block is freed and another taken where it
/// was, which leaves the SDK's head as it was.
pub const COUNT: i64 = 0x08;

/// A new block this size or smaller is swept whole: over movies 4 and 5 that read 40 and 89 MB,
/// against 137 and 315 MB for every block whole, and found everything a whole sweep did.
pub const WHOLE: i64 = 0x20000;
/// Of a larger block, only this much is swept: its object and whatever the game keeps near the
/// start of a big buffer. The rest of such blocks is textures, files and models.
pub const FIRST: i64 = 0x2000;

/// The part of a new block worth sweeping for the objects the readers want.
pub fn to_sweep(block: Block) -> (i64, i64) {
    let (start, end) = block.memory();
    (
        start,
        if block.size <= WHOLE {
            end
        } else {
            start + FIRST
        },
    )
}
