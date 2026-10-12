//! TOML, read and written: the one format pa3ds keeps settings and names in.
//!
//! The player's files are TOML and nothing else (ADR-0031). `tomllib` reads it and the standard
//! library has no writer, so the writing is here — it is a few lines for the shape these files have,
//! which is tables of strings, and hand-writing it is what lets a file carry **comments**. A names
//! file that cannot say which game it belongs to is a file nobody can pick up a year later.
//!
//! Reading never raises at a player. A file that is missing, unreadable or malformed reads as
//! nothing at all: being told the wrong name is a nuisance, and a reader that stops mid-game is not.
//! A single line that is the wrong shape loses itself rather than the file, because these are
//! written by hand as often as by pa3ds.

use std::ffi::OsString;
use std::path::{Component, Path};

use indexmap::IndexMap;
use toml::Value;

use crate::py::{self, Error, Kind, Result};

/// What a basic string may not hold as itself. Every other control character goes as `\uXXXX`.
pub const ESCAPES: [(char, &str); 7] = [
    ('\\', "\\\\"),
    ('"', "\\\""),
    ('\u{8}', "\\b"),
    ('\u{c}', "\\f"),
    ('\n', "\\n"),
    ('\r', "\\r"),
    ('\t', "\\t"),
];

/// A TOML file as its top-level tables, or empty when there is nothing readable there.
pub fn read(path: &Path) -> IndexMap<String, Value> {
    let Ok(bytes) = std::fs::read(path) else {
        return IndexMap::new();
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return IndexMap::new();
    };
    read_text(Some(&text))
}

/// `read` of a file the binary carries, by its text: None where the game has no such file, as a
/// missing file reads as nothing.
pub fn read_text(text: Option<&str>) -> IndexMap<String, Value> {
    let Some(text) = text else {
        return IndexMap::new();
    };
    // `tomllib` takes U+FEFF for an invalid statement where the `toml` crate skips it.
    if text.starts_with('\u{feff}') {
        return IndexMap::new();
    }
    match text.parse::<toml::Table>() {
        Ok(loaded) => table(Some(&Value::Table(loaded))),
        Err(_) => IndexMap::new(),
    }
}

/// Write a file whole and move it into place; False when it could not be written.
///
/// Whole and then moved because a rename lands mid-game: a half-written file would be read back
/// as no names at all the next time pa3ds started.
pub fn write(path: &Path, text: &str) -> Result<bool> {
    let made = std::fs::create_dir_all(path.parent().unwrap_or(path));
    if made.is_err() {
        return Ok(false);
    }
    let name = name_of(path);
    if name.is_empty() {
        return Err(Error::value(format!("{} has an empty name", repr_of(path))));
    }
    let mut beside_name = name;
    beside_name.push(".new");
    let beside = path.with_file_name(beside_name);
    // `Path.write_text` opens the file in text mode, which writes each "\n" as the platform's line
    // ending: a file pa3ds writes on Windows has CRLF lines, as the Python's did.
    let body = if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text.to_string()
    };
    if std::fs::write(&beside, body).is_err() || std::fs::rename(&beside, path).is_err() {
        return Ok(false);
    }
    Ok(true)
}

/// Whatever a parser handed back, as a table of named things and nothing else.
pub fn table(value: Option<&Value>) -> IndexMap<String, Value> {
    match value {
        Some(Value::Table(held)) => held
            .iter()
            .map(|(key, held)| (key.clone(), held.clone()))
            .collect(),
        _ => IndexMap::new(),
    }
}

/// The named strings in a table, and nothing else: a number or a table where a name should be
/// is a typo, not a reason to stop reading the rest of the file.
pub fn strings(value: Option<&Value>) -> IndexMap<String, String> {
    table(value)
        .into_iter()
        .filter_map(|(key, held)| match held {
            Value::String(held) if !held.is_empty() => Some((key, held)),
            _ => None,
        })
        .collect()
}

/// One TOML basic string, escaped. A player may call something anything at all, quotation
/// marks and backslashes included, and a name that broke the file would take the rest with it.
pub fn quote(text: &str) -> String {
    let out: String = text
        .chars()
        .map(|character| {
            ESCAPES
                .iter()
                .find(|(escaped, _)| *escaped == character)
                .map(|(_, escape)| escape.to_string())
                .unwrap_or_else(|| escaped(character))
        })
        .collect();
    format!("\"{out}\"")
}

fn escaped(character: char) -> String {
    if (character as u32) < 0x20 || character as u32 == 0x7F {
        format!("\\u{:04x}", character as u32)
    } else {
        character.to_string()
    }
}

/// A table's lines, sorted, so a file written twice from the same names is the same file.
pub fn pairs(named: &IndexMap<String, String>) -> Vec<String> {
    let mut items: Vec<(&String, &String)> = named.iter().collect();
    items.sort();
    items
        .into_iter()
        .map(|(key, value)| format!("{} = {}", quote(key), quote(value)))
        .collect()
}

/// Lines as a comment block. A line of its own for each, since a file is read in a terminal
/// as often as in an editor and neither wraps kindly.
pub fn comment<S: AsRef<str>>(lines: impl IntoIterator<Item = S>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| py::rstrip(&format!("# {}", line.as_ref())).to_string())
        .collect()
}

/// `path.name`: Rust's `file_name` leaves out `..` and Python's does not.
fn name_of(path: &Path) -> OsString {
    match path.components().next_back() {
        Some(Component::Normal(name)) => name.to_os_string(),
        Some(Component::ParentDir) => OsString::from(".."),
        _ => OsString::new(),
    }
}

/// `repr(path)`, as `with_suffix` words an empty name.
fn repr_of(path: &Path) -> String {
    let class = if cfg!(windows) {
        "WindowsPath"
    } else {
        "PosixPath"
    };
    let shown = path.to_string_lossy().replace('\\', "/");
    let shown = if shown.is_empty() { "." } else { &shown };
    format!("{class}({})", py::repr(shown))
}

/// The `UnicodeDecodeError` that `bytes.decode()` raises for these bytes.
pub(crate) fn decode_error(bytes: &[u8], error: std::str::Utf8Error) -> Error {
    let start = error.valid_up_to();
    let length = error.error_len().unwrap_or(bytes.len() - start);
    let end = start + length;
    let reason = match error.error_len() {
        None => "unexpected end of data",
        Some(_) if length == 1 && !(0xC2..=0xF4).contains(&bytes[start]) => "invalid start byte",
        Some(_) => "invalid continuation byte",
    };
    let message = if length > 1 {
        format!(
            "'utf-8' codec can't decode bytes in position {start}-{}: {reason}",
            end - 1
        )
    } else {
        format!(
            "'utf-8' codec can't decode byte {:#04x} in position {start}: {reason}",
            bytes[start]
        )
    };
    Error::new(Kind::UnicodeDecodeError, message)
}
