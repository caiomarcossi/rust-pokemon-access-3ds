//! Automatic Ultra map names from the player's ROM: model species, interaction text and exits.
//!
//! Field model file N is move-model code N. Its pmNNNN tag is resolved through the battle
//! model table, not assumed to be a national number (Ultra's later species use different tags).
//! An object's code selects a case of its zone's ZS script. Only constant message calls through
//! the verified message wrapper supply its label; cutscenes and ambiguous callbacks are left out.
//! Player names retain priority through `core::names::Names::use_game`.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::rom::{Rom, lz11_limit};
use crate::core::words::{current, own};
use crate::games::common::field::things::{FURNITURE_SOURCE, WARP_SOURCE, place_source};
use crate::games::common::garc::{garc_spans, message_lines, plain};
use crate::games::gen6::rom::people::{MODEL_NAMES, SPECIES_TAG};
use crate::games::gen6::rom::script::{self, Script};
use crate::games::sm::field::CELL;
use crate::games::sm::text::SunText;
use crate::games::sm::zones::{self, SunZones};
use crate::py::{self, Result};

const FIELD_MODELS: &str = "a/2/0/0";
const BATTLE_MODELS: &str = "a/0/9/4";
const SPECIES: i64 = 60;
const BATTLE_FILES: i64 = 9;
// Hashed native identifier in Ultra's AMX native table, measured from the ROM's message wrapper.
const SHOW_MESSAGE: i64 = 0xFD1CF919;
const PUSH_STACK: i64 = 190;

#[derive(Default)]
pub struct AutomaticNames {
    pub sprites: IndexMap<i64, String>,
    pub places: IndexMap<(i64, i64), String>,
}

impl AutomaticNames {
    pub fn load(rom: &Rom, language: &str, zones: Option<&SunZones>) -> Result<Self> {
        if ![super::ULTRA_SUN, super::ULTRA_MOON].contains(&rom.title_id) {
            return Ok(Self::default());
        }
        let text = SunText::load(rom, language, &[SPECIES])?;
        let species = &text.files[&SPECIES];
        let mut out = Self {
            sprites: pokemon_names(rom, species)?,
            ..Self::default()
        };
        if let Some(zones) = zones {
            out.interactions(rom, language, zones)?;
        }
        Ok(out)
    }

    fn interactions(&mut self, rom: &Rom, language: &str, zones: &SunZones) -> Result<()> {
        let Some((_, text_archive)) = crate::games::sm::text::TEXT_ARCHIVES
            .iter()
            .find(|(lang, _)| *lang == language)
        else {
            return Ok(());
        };
        let story_archive = text_archive.replace("/3/", "/4/");
        let (story_start, story_spans) = garc_spans(rom, &story_archive)?;
        let (event_start, event_spans) = garc_spans(rom, zones::EVENTS)?;
        let rooms = current().prefixed("ultra-zone-");
        let mut proposed: HashMap<(i64, i64), HashSet<String>> = HashMap::new();
        for (&area, records) in &zones.areas {
            // The eighth area file is ZS, one Pawn script per zone in the events' part order.
            let index = area * zones::AREA_FILES + 7;
            let scripts = (|| -> Result<Vec<Vec<u8>>> {
                let Some(&(begin, end)) = event_spans.get(index as usize) else {
                    return Ok(Vec::new());
                };
                let bytes = zones::unpacked(&rom.read_part(
                    zones::EVENTS,
                    event_start + begin,
                    end - begin,
                )?)?;
                zones::pack(&bytes, b"ZS")
            })();
            let scripts = scripts.unwrap_or_default();
            for (part, warps) in records.warps.iter().enumerate() {
                let Some(zone) = warps.first().and_then(|r| r.zone) else {
                    continue;
                };
                for warp in warps {
                    let Some(to) = warp.leads_to else {
                        continue;
                    };
                    let place = rooms.get(&format!("{to:x}")).cloned().or(zones.place(to)?);
                    if let Some(place) = place.filter(|p| !p.is_empty()) {
                        proposed
                            .entry((zone, source(WARP_SOURCE, warp.x, warp.z)?))
                            .or_default()
                            .insert(own("ultra-exit-to", &[("place", &place)]));
                    }
                }
                let Some(body) = scripts.get(part) else {
                    continue;
                };
                let Ok(Some(script)) = script::read(body) else {
                    continue;
                };
                let Ok(hashes) = native_hashes(body) else {
                    continue;
                };
                let file = u16(&zones.rows, zone * zones::ZONE_ROW + 0x0C)?;
                let Some(&(begin, end)) = story_spans.get(file as usize) else {
                    continue;
                };
                let Ok(lines) = message_lines(&rom.read_part(
                    &story_archive,
                    story_start + begin,
                    end - begin,
                )?) else {
                    continue;
                };
                let Some(things) = records.things.get(part) else {
                    continue;
                };
                for thing in things {
                    if let Some(line) = interaction_line(&script, &hashes, thing.code)
                        && let Some(raw) = lines.get(line as usize)
                        && let Some(label) = label(&plain(raw))
                    {
                        proposed
                            .entry((zone, source(FURNITURE_SOURCE, thing.x, thing.z)?))
                            .or_default()
                            .insert(label);
                    }
                }
            }
        }
        for (key, names) in proposed {
            if names.len() == 1 {
                self.places
                    .insert(key, names.into_iter().next().expect("one name"));
            }
        }
        Ok(())
    }
}

fn source(kind: i64, x: f64, z: f64) -> Result<i64> {
    let key = (py::floor(x / CELL)? & 0xFFFF) << 16 | (py::floor(z / CELL)? & 0xFFFF);
    place_source(kind, key)
}

pub(crate) fn label(text: &str) -> Option<String> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // A short interaction is a useful description; a long conversation or control-only line
    // is not a name. Keep the game's words rather than guessing a noun in the player's language.
    (!text.is_empty() && text.chars().count() <= 180 && !text.chars().any(char::is_control))
        .then_some(text)
}

fn model_tags(rom: &Rom, archive: &str, start: i64, span: (i64, i64)) -> Result<HashSet<i64>> {
    let (begin, end) = span;
    let prefix = rom.read_part(archive, start + begin, (end - begin).min(MODEL_NAMES))?;
    let bytes = if prefix.first() == Some(&0x11) {
        let packed = rom.read_part(archive, start + begin, end - begin)?;
        lz11_limit(&packed, Some(MODEL_NAMES))?
    } else {
        prefix
    };
    Ok(SPECIES_TAG
        .captures_iter(&bytes)
        .map(|tag| {
            tag[1]
                .iter()
                .fold(0, |n, digit| n * 10 + i64::from(digit - b'0'))
        })
        .collect())
}

fn pokemon_names(rom: &Rom, names: &[String]) -> Result<IndexMap<i64, String>> {
    let (start, spans) = garc_spans(rom, BATTLE_MODELS)?;
    let Some(&(begin, end)) = spans.first() else {
        return Ok(IndexMap::new());
    };
    let table = rom.read_part(BATTLE_MODELS, start + begin, end - begin)?;
    let mut species: HashMap<i64, HashSet<usize>> = HashMap::new();
    for number in 1..names.len() {
        let row = (number as i64 - 1) * 4;
        if row + 4 > table.len() as i64 {
            break;
        }
        let file = 1 + u16(&table, row)? * BATTLE_FILES;
        if let Some(&span) = spans.get(file as usize)
            && let Ok(tags) = model_tags(rom, BATTLE_MODELS, start, span)
            && tags.len() == 1
        {
            species
                .entry(*tags.iter().next().expect("one tag"))
                .or_default()
                .insert(number);
        }
    }
    let (start, spans) = garc_spans(rom, FIELD_MODELS)?;
    let mut out = IndexMap::new();
    for (code, span) in spans.into_iter().enumerate() {
        let Ok(tags) = model_tags(rom, FIELD_MODELS, start, span) else {
            continue;
        };
        if tags.len() != 1 {
            continue;
        }
        let Some(numbers) = species.get(tags.iter().next().expect("one tag")) else {
            continue;
        };
        if numbers.len() == 1 {
            let number = *numbers.iter().next().expect("one species");
            if let Some(name) = label(&names[number]) {
                out.insert(code as i64, name);
            }
        }
    }
    Ok(out)
}

fn native_hashes(blob: &[u8]) -> Result<Vec<i64>> {
    let (start, end) = (u32(blob, 36)?, u32(blob, 40)?);
    if start > end || end > blob.len() as i64 || (end - start) % 8 != 0 {
        return Ok(Vec::new());
    }
    (start..end)
        .step_by(8)
        .map(|at| u32(blob, at + 4))
        .collect()
}

fn body(script: &Script, start: i64) -> Vec<i64> {
    let Some(index) = script.order.iter().position(|&cell| cell == start) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for &cell in script.order.iter().skip(index).take(128) {
        out.push(cell);
        if [script::RET, script::RETN].contains(&script.instructions[&cell].0) {
            break;
        }
    }
    out
}

fn reaches_message(script: &Script, hashes: &[i64], start: i64, depth: usize) -> bool {
    if depth == 0 {
        return false;
    }
    for cell in body(script, start) {
        let (op, args) = &script.instructions[&cell];
        if *op == script::NATIVE
            && args.len() == 2
            && args[1] == 44
            && usize::try_from(args[0]).ok().and_then(|n| hashes.get(n)) == Some(&SHOW_MESSAGE)
        {
            return true;
        }
        if *op == script::CALL
            && args.len() == 1
            && reaches_message(script, hashes, cell + args[0] / 4, depth - 1)
        {
            return true;
        }
    }
    false
}

fn message_wrapper(script: &Script, hashes: &[i64], target: i64) -> bool {
    let cells = body(script, target);
    cells.windows(5).any(|run| {
        [20, 16, 12]
            .iter()
            .zip(run)
            .all(|(&offset, cell)| script.instructions[cell] == (PUSH_STACK, vec![offset]))
            && script.instructions[&run[3]] == (script::PUSH_CONSTANT, vec![12])
            && script.instructions[&run[4]].0 == script::CALL
    }) && reaches_message(script, hashes, target, 4)
}

pub(crate) fn interaction_line(script: &Script, hashes: &[i64], code: i64) -> Option<i64> {
    let case = script.cases().ok()?.get(&code).copied()?;
    let callback = script.called(case).ok()??;
    let cells = body(script, callback);
    if cells.iter().any(|cell| {
        let op = script.instructions[cell].0;
        (script::JUMP..=65).contains(&op) || op == script::SWITCH
    }) {
        return None;
    }
    let mut messages = Vec::new();
    for (index, &cell) in cells.iter().enumerate() {
        let (op, args) = &script.instructions[&cell];
        if *op != script::CALL || args.len() != 1 || index < 4 {
            continue;
        }
        if script.instructions[&cells[index - 1]] != (script::PUSH_CONSTANT, vec![12]) {
            continue;
        }
        let pushed: Vec<_> = (2..=4)
            .map(|n| &script.instructions[&cells[index - n]])
            .collect();
        if pushed
            .iter()
            .any(|(op, values)| *op != script::PUSH_CONSTANT || values.len() != 1)
        {
            continue;
        }
        let constants: Vec<_> = pushed.iter().map(|(_, values)| values[0]).collect();
        if constants[0] >= 0
            && (0..=3).contains(&constants[1])
            && (0..=3).contains(&constants[2])
            && message_wrapper(script, hashes, cell + args[0] / 4)
        {
            messages.push(constants[0]);
        }
    }
    // Several unrelated messages usually belong to a story sequence, not one object's label.
    (messages.len() <= 2)
        .then(|| messages.first().copied())
        .flatten()
}
