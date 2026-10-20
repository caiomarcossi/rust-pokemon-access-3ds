//! Little-endian numbers out of bytes read from the 3DS, which is little-endian throughout.

use std::panic::Location;

use crate::py::{Error, Result};

// A short buffer is raised from here, as Python's `unpack_from` raised it inside each of these:
// a fault is keyed by where it was raised, so every reader's short read is one fault. The line
// that called each is kept beside it for the log, as Python's traceback names the reader (#18):
// each reads in a closure, which is no `#[track_caller]`, so `take` still says it was raised here.

#[track_caller]
pub(crate) fn take<const N: usize>(blob: &[u8], offset: i64) -> Result<[u8; N]> {
    let start = usize::try_from(offset).ok();
    match start.and_then(|s| blob.get(s..s.checked_add(N)?)) {
        Some(bytes) => Ok(bytes.try_into().expect("N bytes")),
        None => Err(Error::unpack(format!(
            "unpack_from requires a buffer of at least {} bytes for unpacking {N} bytes at offset \
             {offset} (actual buffer size is {})",
            offset + N as i64,
            blob.len()
        ))),
    }
}

/// What `read` gives, or the error it raised with the line that called for it.
fn called_from<T>(
    caller: &'static Location<'static>,
    read: impl FnOnce() -> Result<T>,
) -> Result<T> {
    read().map_err(|error| error.called_from(caller))
}

#[track_caller]
pub fn u16(blob: &[u8], offset: i64) -> Result<i64> {
    called_from(Location::caller(), || {
        Ok(u16::from_le_bytes(take(blob, offset)?) as i64)
    })
}

#[track_caller]
pub fn s16(blob: &[u8], offset: i64) -> Result<i64> {
    called_from(Location::caller(), || {
        Ok(i16::from_le_bytes(take(blob, offset)?) as i64)
    })
}

#[track_caller]
pub fn u32(blob: &[u8], offset: i64) -> Result<i64> {
    called_from(Location::caller(), || {
        Ok(u32::from_le_bytes(take(blob, offset)?) as i64)
    })
}

/// `struct.unpack_from("<i", blob, offset)`: the four bytes `u32` reads, as a signed number.
#[track_caller]
pub fn s32(blob: &[u8], offset: i64) -> Result<i64> {
    called_from(Location::caller(), || {
        Ok(i32::from_le_bytes(take(blob, offset)?) as i64)
    })
}

#[track_caller]
pub fn f32(blob: &[u8], offset: i64) -> Result<f64> {
    called_from(Location::caller(), || {
        Ok(f32::from_le_bytes(take(blob, offset)?) as f64)
    })
}

#[cfg(test)]
mod checks {
    use super::*;
    use crate::py::Kind;

    #[test]
    fn s32_is_u32_read_as_signed() {
        let blob = [
            0xfe, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x80, 0x2a, 0, 0, 0,
        ];
        assert_eq!(s32(&blob, 0).unwrap(), -2);
        assert_eq!(s32(&blob, 4).unwrap(), -2_147_483_648);
        assert_eq!(s32(&blob, 8).unwrap(), 42);
        assert_eq!(u32(&blob, 0).unwrap(), 0xffff_fffe);
        assert!(s32(&blob, 9).unwrap_err().is(Kind::StructError));
    }
}
