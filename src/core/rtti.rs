//! Name C++ objects from the RTTI the game ships with.
//!
//! The executable is built with RTTI, so a polymorphic object names its own class: its first word
//! is its vtable, the vtable's word at -4 is its typeinfo, and the typeinfo's word at +4 points at
//! the mangled class name. That makes a class an anchor. Scan the heap for the vtable of the class
//! you want, instead of for a struct shape that has to be guessed and then validated.
//!
//! A name says what an object was *built as*, not that it is still alive. Freed memory keeps its
//! vtable, and a destructor leaves its base class's vtable behind: a destroyed `nw::lyt::TextBox`
//! reads as `nw::lyt::Pane` until something reuses the memory (spike 13). Liveness is a separate
//! check, owned by whoever knows what the object is for.
//!
//! This is the Itanium C++ ABI layout, which ARMCC follows. Nothing here is specific to Pokemon.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::bytes::Regex;

use crate::core::binary;
use crate::core::memory::Memory;
use crate::py::{self, Error, Result};

/// Longest mangled name read. Template-heavy names run long; anything past this is not a name.
pub const NAME_LIMIT: i64 = 256;

static MANGLED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A[A-Za-z0-9_]{2,255}\z").expect("a valid pattern"));

/// `_U32.pack(value)`, which raises `struct.error` outside 0 to 2**32 - 1.
#[track_caller]
fn pack_u32(value: i64) -> Result<[u8; 4]> {
    u32::try_from(value)
        .map(u32::to_le_bytes)
        .map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))
}

/// Every word-aligned address in `pages` that holds one of `vtables`, mapped to which one.
///
/// Objects start on a word boundary, so a vtable pointer always sits wholly inside one page.
/// Dead objects are found as readily as live ones; see the module docstring.
pub fn find_objects(pages: &IndexMap<i64, Vec<u8>>, vtables: &[i64]) -> Result<IndexMap<i64, i64>> {
    let mut found: IndexMap<i64, i64> = IndexMap::new();
    for &vtable in vtables {
        let needle = pack_u32(vtable)?;
        for (&base, page) in pages {
            for at in aligned(page, &needle, 4, 0) {
                found.insert(base + at, vtable);
            }
        }
    }
    Ok(found)
}

/// Every primary vtable of the class named `mangled`, found in the executable image, whose
/// name, typeinfo and vtable lie at or past `start` in it.
///
/// Name, then the typeinfo whose +4 points at it, then the vtable whose -4 points at the
/// typeinfo. A primary vtable has an offset-to-top of zero at -8; the secondary vtables of a
/// class with more than one base do not, and neither does a subclass typeinfo that merely names
/// this one as its base. Its first slot points into the image, or into `code` where given: a
/// module's class may keep a method it inherited from the executable.
///
/// Python's defaults are a `start` of 0 and no `code`.
pub fn find_vtables(
    image: &[u8],
    base: i64,
    mangled: &str,
    start: i64,
    code: Option<(i64, i64)>,
) -> Result<Vec<i64>> {
    let (low, high) = match code {
        Some(code) => code,
        None => (base, base + image.len() as i64),
    };
    let mut out: Vec<i64> = Vec::new();
    // Python's `encode("ascii")` raises UnicodeEncodeError, a ValueError, for anything else.
    if let Some((position, c)) = mangled.chars().enumerate().find(|(_, c)| !c.is_ascii()) {
        return Err(Error::value(format!(
            "'ascii' codec can't encode character {} in position {position}: ordinal not in \
             range(128)",
            py::repr(&c.to_string())
        )));
    }
    let mut wanted = mangled.as_bytes().to_vec();
    wanted.push(0);
    for name in aligned(image, &wanted, 1, start) {
        for name_ref in aligned(image, &pack_u32(base + name)?, 4, start) {
            let typeinfo = base + name_ref - 4;
            for typeinfo_ref in aligned(image, &pack_u32(typeinfo)?, 4, start) {
                if typeinfo_ref < 4 || typeinfo_ref + 8 > image.len() as i64 {
                    continue;
                }
                let offset_to_top = i32::from_le_bytes(binary::take::<4>(image, typeinfo_ref - 4)?);
                let first_slot = binary::u32(image, typeinfo_ref + 4)?;
                if offset_to_top == 0 && low <= first_slot && first_slot < high {
                    out.push(base + typeinfo_ref + 4);
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

/// `image.find(needle, start)`: the first index at or past `start`, or None where Python says -1.
fn find(image: &[u8], needle: &[u8], start: i64) -> Option<usize> {
    let len = image.len() as i64;
    let from = if start < 0 {
        (len + start).max(0)
    } else {
        start
    };
    if from > len {
        return None;
    }
    let from = from as usize;
    let Some(&first) = needle.first() else {
        return Some(from);
    };
    let last = image.len().checked_sub(needle.len())?;
    let mut at = from;
    while at <= last {
        at += image[at..=last].iter().position(|&b| b == first)?;
        if &image[at..at + needle.len()] == needle {
            return Some(at);
        }
        at += 1;
    }
    None
}

fn aligned(image: &[u8], needle: &[u8], align: i64, start: i64) -> Vec<i64> {
    let mut hits: Vec<i64> = Vec::new();
    let mut at = find(image, needle, start).map(|at| at as i64);
    while let Some(here) = at {
        if here % align == 0 {
            hits.push(here);
        }
        at = find(image, needle, here + 1).map(|at| at as i64);
    }
    hits
}

/// Class names by vtable, read from live memory and cached for the life of the process.
///
/// `code` is the executable's span. A vtable, a typeinfo and a name all live inside it, and
/// anything pointing elsewhere is not one of them — which is what keeps a heap word that
/// happens to look like a pointer from being named.
pub struct ClassNames {
    memory: Rc<dyn Memory>,
    code: (i64, i64),
    cache: RefCell<IndexMap<i64, Option<String>>>,
}

impl ClassNames {
    pub fn new(memory: Rc<dyn Memory>, code: (i64, i64)) -> Self {
        ClassNames {
            memory,
            code,
            cache: RefCell::new(IndexMap::new()),
        }
    }

    /// The mangled name of the class this is the vtable of, or None if it is not a vtable.
    pub fn of_vtable(&self, vtable: i64) -> Result<Option<String>> {
        if !self.cache.borrow().contains_key(&vtable) {
            let resolved = self.resolve(vtable)?;
            self.cache.borrow_mut().insert(vtable, resolved);
        }
        Ok(self.cache.borrow()[&vtable].clone())
    }

    /// The class `address` was built as. Says nothing about whether it is still alive.
    pub fn of_object(&self, address: i64) -> Result<Option<String>> {
        self.of_vtable(self.u32(address)?)
    }

    fn resolve(&self, vtable: i64) -> Result<Option<String>> {
        if vtable & 3 != 0 || !self.in_code(vtable - 4) {
            return Ok(None);
        }
        let typeinfo = self.u32(vtable - 4)?;
        if typeinfo & 3 != 0 || !self.in_code(typeinfo) {
            return Ok(None);
        }
        let pointer = self.u32(typeinfo + 4)?;
        if !self.in_code(pointer) {
            return Ok(None);
        }
        let raw = self.memory.read(pointer, NAME_LIMIT)?;
        let Some(end) = raw.iter().position(|&b| b == 0) else {
            return Ok(None);
        };
        if !MANGLED.is_match(&raw[..end]) {
            return Ok(None);
        }
        Ok(Some(py::decode_ascii(&raw[..end])))
    }

    fn in_code(&self, address: i64) -> bool {
        let (low, high) = self.code;
        low <= address && address < high
    }

    fn u32(&self, address: i64) -> Result<i64> {
        let data = self.memory.read(address, 4)?;
        match <[u8; 4]>::try_from(data.as_slice()) {
            Ok(word) => Ok(u32::from_le_bytes(word) as i64),
            Err(_) => Err(Error::unpack("unpack requires a buffer of 4 bytes")),
        }
    }
}

/// `N2nw3lyt4PaneE` as `nw::lyt::Pane`. Plain nested names only; anything else unchanged.
///
/// For logs and diagnostics. Matching is always on the mangled name, which is exact.
///
/// Python's `\d` also reads the Unicode digits and `int` their value; a mangled name is ASCII,
/// which a name read by [`ClassNames`] always is, so only 0 to 9 are read here.
pub fn demangle(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut body: &[char] = if name.starts_with('N') && name.ends_with('E') {
        py::slice(&chars, 1, -1)
    } else {
        &chars
    };
    let mut parts: Vec<String> = Vec::new();
    while !body.is_empty() {
        let start = body.iter().take_while(|c| c.is_ascii_digit()).count();
        if start == 0 {
            return name.to_string();
        }
        let size = body[..start].iter().fold(0i64, |size, digit| {
            size.saturating_mul(10)
                .saturating_add(digit.to_digit(10).map_or(0, i64::from))
        });
        if size == 0 || size > body.len() as i64 - start as i64 {
            return name.to_string();
        }
        let size = size as usize;
        parts.push(body[start..start + size].iter().collect());
        body = &body[start + size..];
    }
    if parts.is_empty() {
        name.to_string()
    } else {
        parts.join("::")
    }
}
