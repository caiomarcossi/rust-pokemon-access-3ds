//! The field camera's heading: the way up walks the player on Lumiose's boulevards (spike 105).
//!
//! On a rail the camera is behind the player and the D-pad works from it: up walks the way the camera
//! looks, down walks back from it without turning it, and left and right turn the player a quarter,
//! the camera turning after them only while the key is held. So the camera, not the way the player
//! faces, is what a way there has to be said from, and a tap leaves it pointing anywhere.
//!
//! ```text
//! xy_system::CCameraULCD  +0x64 ptr a block of the heap's own, tagged and sized as every
//!                         block is (`discovery.ALLOCATION`, `BLOCK_SIZE`)
//! that block              +0xebc f32 the camera's heading in the world, in radians, the way a
//!                         model's world facing is: x over z, 0 along z
//! ```
//!
//! Found in seven snapshots taken while the player walked up, each a different way, where it was the
//! way they walked to a tenth of a degree (spike 105). The block is the upper screen camera's and is
//! there on the grid too; only the rails need it.

use std::f64::consts::PI;

use crate::core::binary::{f32, u32};
use crate::core::discovery::{ALLOCATION, Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::py::Result;

pub const CLASS: &str = "N9xy_system11CCameraULCDE";
/// Where the camera object points at the block holding its heading.
pub const BLOCK: i64 = 0x64;
pub const BLOCK_SIZE: i64 = 0x1468;
pub const YAW: i64 = 0xEBC;

/// The block holding the camera's heading, through whichever camera object points at one;
/// None when no sweep has found one.
pub fn find(found: &Found, memory: &dyn Memory) -> Result<Option<i64>> {
    let mut cameras = found.of(&[CLASS]);
    cameras.sort();
    for camera in cameras {
        let block = u32(&memory.read(camera + BLOCK, 4)?, 0)?;
        if within(HEAP_SPAN, block) && is_block(memory, block)? {
            return Ok(Some(block));
        }
    }
    Ok(None)
}

/// Whether `block` still starts the camera's block: freed, the heap reuses it.
pub fn is_block(memory: &dyn Memory, block: i64) -> Result<bool> {
    let head = memory.read(block, 8)?;
    Ok(u32(&head, 0)? == ALLOCATION && u32(&head, 4)? == BLOCK_SIZE)
}

/// The camera's heading in the world, in degrees, or None when the block is not the
/// camera's any more or holds no angle.
pub fn heading(memory: &dyn Memory, block: i64) -> Result<Option<f64>> {
    if !is_block(memory, block)? {
        return Ok(None);
    }
    let yaw = f32(&memory.read(block + YAW, 4)?, 0)?;
    if !yaw.is_finite() || yaw.abs() > PI + 0.01 {
        return Ok(None);
    }
    Ok(Some(yaw.to_degrees()))
}
