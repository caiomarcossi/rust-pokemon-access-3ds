//! Sounds out of a 3DS game's sound archive, a NintendoWare `.bcsar`, decoded to samples.
//!
//! Only what a footstep needs (spike 227, docs/findings/footsteps.md): a sequence sound that plays
//! one note on a bank with one wave. The archive's STRG block names every sound; its INFO block
//! holds a record a sound, a bank, a wave archive and a file; its FILE block holds the files. A
//! sound's record keeps its sequence's file, its volume at +8 and, through its sequence's record,
//! its bank; the bank's file names the wave archive and the wave; the wave archive's file holds the
//! wave, a CWAV, mono DSP-ADPCM in every one of Omega Ruby's footsteps. The sequence plays its note
//! at a random volume, `a0 c1` and the lowest and highest, which `Sound.quietest` keeps.
//!
//! Every offset is read from the file's own headers and records: the archive is the player's own,
//! read out of their ROM (ADR-0008), and nothing of it is shipped.

use indexmap::IndexMap;

use crate::core::binary::{s32, take, u16, u32};
use crate::core::rom::Rom;
use crate::py::{self, Error, Kind, Result};

pub const STRG: i64 = 0x2000;
pub const INFO: i64 = 0x2001;
pub const FILE: i64 = 0x2002;
pub const SOUNDS: i64 = 0x2100;
pub const BANKS: i64 = 0x2101;
pub const WAVE_ARCHIVES: i64 = 0x2103;
pub const FILES: i64 = 0x2106;
/// A sound's item id is `0x01 << 24 | index`.
pub const SOUND_ITEM: i64 = 0x01;
/// A sequence's command: a volume chosen at random between two big-endian numbers.
pub const RANDOM_VOLUME: &[u8] = b"\xa0\xc1";
pub const LOUDEST: i64 = 127;
pub const DSP_ADPCM: i64 = 2;
pub const PCM16: i64 = 1;

/// One sound as the game plays it: the wave's samples at its rate, the sound's own volume, and
/// the quietest the sequence's random volume can make it, both as shares of the loudest.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    pub rate: i64,
    pub samples: Vec<i16>,
    pub volume: f64,
    pub quietest: f64,
}

/// The sounds of `names` the archive has, by name. A RomError where it is not an archive.
pub fn sounds<'a>(
    rom: &Rom,
    archive: &str,
    names: impl IntoIterator<Item = &'a str>,
) -> Result<IndexMap<String, Sound>> {
    let reader = Archive::new(rom, archive)?;
    let items = reader.names()?;
    let mut out: IndexMap<String, Sound> = IndexMap::new();
    for name in names {
        if let Some(&item) = items.get(name)
            && item >> 24 == SOUND_ITEM
        {
            out.insert(name.to_string(), reader.sound(item & 0xFFFFFF)?);
        }
    }
    Ok(out)
}

struct Archive<'a> {
    rom: &'a Rom,
    path: String,
    blocks: IndexMap<i64, (i64, i64)>,
    info: Vec<u8>,
    tables: IndexMap<i64, Vec<i64>>,
}

impl<'a> Archive<'a> {
    fn new(rom: &'a Rom, path: &str) -> Result<Self> {
        let head = rom.read_part(path, 0, 0x40)?;
        if py::slice(&head, 0, 4) != b"CSAR" {
            return Err(Error::new(
                Kind::RomError,
                format!("{path} is not a sound archive"),
            ));
        }
        let mut blocks: IndexMap<i64, (i64, i64)> = IndexMap::new();
        for i in 0..u16(&head, 0x10)? {
            let at = 0x14 + i * 12;
            blocks.insert(u16(&head, at)?, (u32(&head, at + 4)?, u32(&head, at + 8)?));
        }
        let (at, size) = block(&blocks, INFO)?;
        let info = rom.read_part(path, at, size)?;
        let mut tables: IndexMap<i64, Vec<i64>> = IndexMap::new();
        for i in 0..8 {
            let (kind, offset) = (u16(&info, 8 + i * 8)?, s32(&info, 8 + i * 8 + 4)?);
            if [SOUNDS, BANKS, WAVE_ARCHIVES, FILES].contains(&kind) {
                let table = 8 + offset;
                let mut entries: Vec<i64> = Vec::new();
                for j in 0..u32(&info, table)? {
                    entries.push(table + s32(&info, table + 8 + j * 8)?);
                }
                tables.insert(kind, entries);
            }
        }
        Ok(Archive {
            rom,
            path: path.to_string(),
            blocks,
            info,
            tables,
        })
    }

    /// Every name in the STRG block, with its item id: a string table, and a tree whose leaves
    /// pair a string with an item.
    fn names(&self) -> Result<IndexMap<String, i64>> {
        let (at, size) = block(&self.blocks, STRG)?;
        let data = self.rom.read_part(&self.path, at, size)?;
        let (strings, tree) = (8 + u32(&data, 12)?, 8 + u32(&data, 20)?);
        let mut names: Vec<String> = Vec::new();
        for i in 0..u32(&data, strings)? {
            let at = strings + 4 + i * 12;
            let (offset, length) = (u32(&data, at + 4)?, u32(&data, at + 8)?);
            let raw = py::slice(&data, strings + offset, strings + offset + length);
            let end = raw.iter().rposition(|&b| b != 0).map_or(0, |last| last + 1);
            names.push(py::decode_ascii(&raw[..end]));
        }
        let mut out: IndexMap<String, i64> = IndexMap::new();
        for i in 0..u32(&data, tree + 4)? {
            let at = tree + 8 + i * 20;
            let (flags, string, item) =
                (u16(&data, at)?, u32(&data, at + 12)?, u32(&data, at + 16)?);
            if flags & 1 != 0 && string < names.len() as i64 {
                out.insert(names[string as usize].clone(), item);
            }
        }
        Ok(out)
    }

    fn sound(&self, index: i64) -> Result<Sound> {
        let info = &self.info;
        let record = entry(&self.tables, SOUNDS, index)?;
        let volume = byte(info, record + 8)?;
        let sequence = record + s32(info, record + 0x10)?;
        let banks = sequence + s32(info, sequence + 4)?;
        let bank = u32(info, banks + 4)? & 0xFFFFFF;
        let cseq = self.file(u32(info, record)?)?;
        let cbnk = self.file(u32(info, entry(&self.tables, BANKS, bank)?)?)?;
        let mut waves = u32(&cbnk, 0x18)? + 8;
        waves += u32(&cbnk, waves + 4)?;
        let (archive, wave) = (u32(&cbnk, waves + 4)? & 0xFFFFFF, u32(&cbnk, waves + 8)?);
        let cwar = self.file(u32(info, entry(&self.tables, WAVE_ARCHIVES, archive)?)?)?;
        let at = u32(&cwar, 0x18)? + 8 + 4 + wave * 12;
        let start = u32(&cwar, 0x24)? + 8 + u32(&cwar, at + 4)?;
        let (rate, samples) = decode(py::slice(&cwar, start, start + u32(&cwar, at + 8)?))?;
        let mut quietest = LOUDEST;
        if let Some(found) = cseq
            .windows(RANDOM_VOLUME.len())
            .position(|window| window == RANDOM_VOLUME)
        {
            quietest = i16::from_be_bytes(take::<2>(&cseq, found as i64 + 2)?) as i64;
        }
        Ok(Sound {
            rate,
            samples,
            volume: volume as f64 / LOUDEST as f64,
            quietest: quietest as f64 / LOUDEST as f64,
        })
    }

    fn file(&self, number: i64) -> Result<Vec<u8>> {
        let record = entry(&self.tables, FILES, number)?;
        let place = record + s32(&self.info, record + 4)?;
        let (offset, size) = (u32(&self.info, place + 4)?, u32(&self.info, place + 8)?);
        self.rom
            .read_part(&self.path, block(&self.blocks, FILE)?.0 + 8 + offset, size)
    }
}

/// `self._blocks[kind]`: a KeyError where the archive has no such block.
#[track_caller]
fn block(blocks: &IndexMap<i64, (i64, i64)>, kind: i64) -> Result<(i64, i64)> {
    blocks
        .get(&kind)
        .copied()
        .ok_or_else(|| Error::key(kind.to_string()))
}

/// `self._tables[kind][index]`: a KeyError where there is no such table, an IndexError past its
/// end.
#[track_caller]
fn entry(tables: &IndexMap<i64, Vec<i64>>, kind: i64, index: i64) -> Result<i64> {
    let table = tables
        .get(&kind)
        .ok_or_else(|| Error::key(kind.to_string()))?;
    usize::try_from(index)
        .ok()
        .and_then(|at| table.get(at))
        .copied()
        .ok_or_else(|| Error::index("list index out of range"))
}

/// `data[at]`: an IndexError past its end.
#[track_caller]
fn byte(data: &[u8], at: i64) -> Result<i64> {
    usize::try_from(at)
        .ok()
        .and_then(|at| data.get(at))
        .map(|&b| b as i64)
        .ok_or_else(|| Error::index("index out of range"))
}

/// A mono CWAV's rate and samples, 16-bit PCM as it is or DSP-ADPCM decoded: frames of 8
/// bytes, a header byte (the coefficient pair in its high nibble, the scale's power of two in its
/// low) and 14 signed nibbles, each `(nibble * scale << 11) + 1024 + c1 * h1 + c2 * h2 >> 11`.
pub fn decode(cwav: &[u8]) -> Result<(i64, Vec<i16>)> {
    let info = u32(cwav, 0x18)? + 8;
    let data = u32(cwav, 0x24)? + 8;
    let (encoding, rate, count) = (
        byte(cwav, info)?,
        u32(cwav, info + 4)?,
        u32(cwav, info + 0xC)?,
    );
    let channels = info + 0x14;
    let channel = channels + s32(cwav, channels + 8)?;
    let mut at = data + s32(cwav, channel + 4)?;
    if encoding == PCM16 {
        let raw = py::slice(cwav, at, at + count * 2);
        if raw.len() % 2 != 0 {
            return Err(Error::new(
                Kind::ValueError,
                "bytes length not a multiple of item size",
            ));
        }
        let samples = raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| i16::from_le_bytes(pair))
            .collect();
        return Ok((rate, samples));
    }
    if encoding != DSP_ADPCM {
        return Err(Error::new(
            Kind::RomError,
            format!("a wave encoded as {encoding}, not DSP-ADPCM"),
        ));
    }
    let table = channel + s32(cwav, channel + 12)?;
    let mut coefficients: Vec<i64> = Vec::new();
    for i in 0..16 {
        coefficients.push(i16::from_le_bytes(take::<2>(cwav, table + i * 2)?) as i64);
    }
    let coefficient = |at: i64| {
        coefficients
            .get(at as usize)
            .copied()
            .ok_or_else(|| Error::index("tuple index out of range"))
    };
    let mut out: Vec<i16> = Vec::new();
    let (mut newer, mut older) = (0i64, 0i64);
    while (out.len() as i64) < count {
        let header = byte(cwav, at)?;
        let (scale, pair) = (1i64 << (header & 0xF), header >> 4);
        let (first, second) = (coefficient(pair * 2)?, coefficient(pair * 2 + 1)?);
        for &byte in py::slice(cwav, at + 1, at + 8) {
            for nibble in [(byte >> 4) as i64, (byte & 0xF) as i64] {
                let nibble = nibble - if nibble >= 8 { 16 } else { 0 };
                let value =
                    (((nibble * scale) << 11) + 1024 + first * newer + second * older) >> 11;
                (older, newer) = (newer, value.clamp(-32768, 32767));
                out.push(newer as i16);
            }
        }
        at += 8;
    }
    out.truncate(count as usize);
    Ok((rate, out))
}
