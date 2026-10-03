//! Which way Sun's pad walks the player, and pa3ds's compass turned to match (spike 119).
//!
//! Sun's camera is not always behind the player looking north. Up on the pad walks the player the
//! way the camera looks, whichever that is: in the living room north, and on the porch outside it
//! west, a little north, so the door back in was "2 west" to pa3ds and straight up to the player
//! (sun_5.ctm, 2026-09-26). ORAS's camera always looks north, and there north is up.
//!
//! ```text
//! Field::Camera::UiDeviceTranslator  (a small allocation, found by a sweep)
//!     +0x08 f32 the angle up is turned by, in radians: up walks the world's (sin a, cos a),
//!           x east and z south. pi in the living room and in Iki Town, -1.934 on the porch,
//!           between -2.76 and -3.12 on Route 1 as Kukui walks the player
//!     +0x14 u32 0 while the translator is built and has not been given the camera's angle,
//!           which reads 0.0 then, the world's south; 1 once it has
//! ```
//!
//! So pa3ds turns its compass by the quarter the camera looks nearest: north is the way up walks,
//! east the way right does, as in ORAS. Every cell pa3ds says is turned by it, the player's and
//! everything around them, and the ground P walks. A camera between two quarters walks the player a
//! little off each leg, and a player who leaves the way is given a new one. A camera swinging about
//! a quarter's edge would turn the whole compass back and forth with it, so the compass keeps its
//! quarter until the camera looks `HYSTERESIS` degrees past the edge (2026-09-27).

use crate::py::{self, Result};

pub type Cell = (i64, i64);

pub const CLASS: &str = "N5Field6Camera18UiDeviceTranslatorE";
pub const ANGLE: i64 = 0x08;
/// 0 until the translator has been given the camera's angle. The field builds it again through a
/// door and after a battle, and for a poll or two its angle is 0.0, south: back from Kukui's
/// catching tutorial on Route 1, which looks north, the field was listed turned half round, and
/// 0.2 s later right (sun_10.ctm, 2026-09-27); through the front door onto the porch, the same
/// (sun_5.ctm).
pub const SET: i64 = 0x14;
pub const HEAD: i64 = SET + 4;
pub const QUARTERS: i64 = 4;
/// Degrees past a quarter's edge (45 from its centre) the camera looks before the compass turns.
pub const HYSTERESIS: f64 = 10.0;

/// The quarter turns clockwise from north of the way up walks, nearest: 0 north, 1 east, 2
/// south, 3 west. With `current`, the quarter last taken, that one until the camera looks
/// `HYSTERESIS` past its edge. North for an angle that is not a number, or `current`.
pub fn quarter(angle: f64, current: Option<i64>) -> Result<i64> {
    if !angle.is_finite() {
        return Ok(current.unwrap_or(0));
    }
    let bearing = angle.sin().atan2(-angle.cos()).to_degrees();
    if let Some(current) = current {
        let off = py::modulo_f(bearing - (current * 90) as f64 + 180.0, 360.0) - 180.0;
        if off.abs() <= 45.0 + HYSTERESIS {
            return Ok(current);
        }
    }
    Ok(py::modulo(py::round(bearing / 90.0)?, QUARTERS))
}

/// A cell of the world as the pad walks it: north the way up walks when the camera looks
/// `turn` quarters clockwise from north. Every cell is one cell still, and its neighbours its
/// neighbours.
pub fn turned(cell: Cell, turn: i64) -> Cell {
    let (mut x, mut z) = cell;
    for _ in 0..py::modulo(turn, QUARTERS) {
        (x, z) = (z, -x - 1);
    }
    (x, z)
}

/// The world's cell that `turned` makes this one.
pub fn unturned(cell: Cell, turn: i64) -> Cell {
    turned(cell, -turn)
}

/// A box of cells, its first cell and its size, turned: its first cell is its north-west
/// one again.
pub fn turned_box(first: Cell, size: (i64, i64), turn: i64) -> (Cell, (i64, i64)) {
    let a = turned(first, turn);
    let b = turned((first.0 + size.0 - 1, first.1 + size.1 - 1), turn);
    (
        (a.0.min(b.0), a.1.min(b.1)),
        ((a.0 - b.0).abs() + 1, (a.1 - b.1).abs() + 1),
    )
}
