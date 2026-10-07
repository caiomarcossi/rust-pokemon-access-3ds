//! Choice box tests, against a fake field Yes/No laid out as spike 26 read it.

use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::common::strbuf;
use crate::games::gen6::screens::choices::{
    BOX_COUNT, BOX_CURSOR, BOX_DEVICE_MANAGER, BOX_ENTRIES, BOX_STATE, ChoiceReader,
    DEVICE_MANAGER_CLASS, ENTRY,
};

const BOX: i64 = 0x08004680;
const MANAGER: i64 = 0x08008000;
const TABLE: i64 = 0x08004744;
const STRBUFS: i64 = 0x08005000;
const TEXTS: i64 = 0x08006000;

const OPENING: i64 = 0;
const UP: i64 = 3;
const ANSWERED: i64 = 4;

struct ChoiceBox {
    memory: Rc<FakeMemory>,
    at: i64,
    other: i64,
    reader: ChoiceReader,
}

impl ChoiceBox {
    fn new() -> ChoiceBox {
        ChoiceBox::of(&["Yes", "No"])
    }

    fn of(labels: &[&str]) -> ChoiceBox {
        ChoiceBox::at(labels, BOX)
    }

    fn at(labels: &[&str], at: i64) -> ChoiceBox {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        memory.u32(MANAGER, &[classes.add(DEVICE_MANAGER_CLASS)]);
        let strbuf_class = classes.add(strbuf::CLASS);
        let other = classes.add("N3gfl4heap11NwAllocatorE");
        memory.u32(at + BOX_DEVICE_MANAGER, &[MANAGER]);
        memory.u32(at + BOX_ENTRIES, &[TABLE]);
        memory.u32(at + BOX_COUNT, &[labels.len() as i64]);
        for (i, label) in labels.iter().enumerate() {
            let i = i as i64;
            let (buffer, text) = (STRBUFS + i * 0x10, TEXTS + i * 0x100);
            let raw: Vec<u8> = label.encode_utf16().flat_map(u16::to_le_bytes).collect();
            let mut written = raw.clone();
            written.extend([0, 0]);
            memory.write(text, &written);
            memory.u32(buffer, &[strbuf_class, text]);
            memory.u16(buffer + strbuf::LENGTH, (raw.len() / 2) as i64);
            memory.u32(TABLE + i * ENTRY, &[labels.len() as i64 - 1 - i, buffer]);
        }
        let mut r#box = ChoiceBox {
            memory: memory.clone(),
            at,
            other,
            reader: ChoiceReader::new(
                memory.clone(),
                Rc::new(ClassNames::new(memory.clone(), CODE)),
            ),
        };
        r#box.state(OPENING);
        r#box.reader.track([at]).unwrap();
        r#box
    }

    fn state(&self, value: i64) {
        self.memory.u16(self.at + BOX_STATE, value);
    }

    fn cursor(&self, row: i64) {
        self.cursor_from(row, 0);
    }

    fn cursor_from(&self, row: i64, first: i64) {
        self.memory.u32(self.at + BOX_CURSOR, &[row, first]);
    }

    fn heard(&mut self) -> Vec<(String, bool)> {
        self.reader
            .poll()
            .unwrap()
            .into_iter()
            .map(|focus| (focus.label, focus.opened))
            .collect()
    }
}

fn heard_of(items: &[(&str, bool)]) -> Vec<(String, bool)> {
    items
        .iter()
        .map(|&(label, opened)| (label.to_string(), opened))
        .collect()
}

fn focus(label: &str, index: i64, count: i64) -> Focus {
    Focus {
        source: BOX,
        label: label.to_string(),
        index,
        count,
        opened: true,
    }
}

#[test]
fn test_a_box_still_opening_says_nothing() {
    assert_eq!(ChoiceBox::new().heard(), heard_of(&[]));
}

#[test]
fn test_a_box_coming_up_announces_the_option_under_the_cursor() {
    let mut r#box = ChoiceBox::new();
    r#box.state(UP);
    assert_eq!(r#box.reader.poll().unwrap(), vec![focus("Yes", 0, 2)]);
}

#[test]
fn test_a_box_polled_again_says_nothing() {
    let mut r#box = ChoiceBox::new();
    r#box.state(UP);
    r#box.heard();
    assert_eq!(r#box.heard(), heard_of(&[]));
}

#[test]
fn test_moving_the_cursor_announces_the_new_option() {
    let mut r#box = ChoiceBox::new();
    r#box.state(UP);
    r#box.heard();
    r#box.cursor(1);
    assert_eq!(r#box.heard(), heard_of(&[("No", false)]));
    r#box.cursor(0);
    assert_eq!(r#box.heard(), heard_of(&[("Yes", false)]));
}

#[test]
fn test_answering_says_nothing() {
    let mut r#box = ChoiceBox::new();
    r#box.state(UP);
    r#box.cursor(1);
    r#box.heard();
    r#box.state(ANSWERED);
    assert_eq!(r#box.heard(), heard_of(&[]));
}

#[test]
fn test_a_box_built_again_where_the_last_one_was_is_announced_as_opening() {
    let mut r#box = ChoiceBox::new();
    r#box.state(UP);
    r#box.heard();
    // freed and reused for something else
    r#box.memory.u32(BOX + BOX_DEVICE_MANAGER, &[0x0000C2C]);
    assert_eq!(r#box.heard(), heard_of(&[]));
    r#box.memory.u32(BOX + BOX_DEVICE_MANAGER, &[MANAGER]);
    assert_eq!(r#box.heard(), heard_of(&[("Yes", true)]));
}

#[test]
fn test_the_labels_of_a_box_are_known_only_while_it_is_up() {
    let mut r#box = ChoiceBox::new();
    r#box.heard();
    assert_eq!(r#box.reader.labels(), HashSet::new());
    r#box.state(UP);
    r#box.heard();
    assert_eq!(
        r#box.reader.labels(),
        HashSet::from(["Yes".to_string(), "No".to_string()])
    );
    r#box.state(ANSWERED);
    r#box.heard();
    assert_eq!(r#box.reader.labels(), HashSet::new());
}

#[test]
fn test_the_option_counts_from_the_first_row_shown() {
    let mut r#box = ChoiceBox::of(&["One", "Two", "Three", "Four", "Five", "Six"]);
    r#box.cursor_from(1, 2);
    r#box.state(UP);
    assert_eq!(r#box.reader.poll().unwrap(), vec![focus("Four", 3, 6)]);
}

#[test]
fn test_a_cursor_below_the_last_row_shown_is_not_believed() {
    let mut r#box = ChoiceBox::of(&["One", "Two", "Three", "Four", "Five", "Six"]);
    r#box.state(UP);
    r#box.cursor(4);
    assert_eq!(r#box.heard(), heard_of(&[]));
}

#[test]
fn test_a_label_that_is_not_a_strbuf_is_not_believed() {
    let mut r#box = ChoiceBox::new();
    r#box.memory.u32(STRBUFS + 0x10, &[r#box.other]);
    r#box.state(UP);
    assert_eq!(r#box.heard(), heard_of(&[]));
}

/// Most objects holding the device manager are something else, and polling them all costs.
#[test]
fn test_a_candidate_not_laid_out_as_a_box_when_found_is_never_read() {
    let mut r#box = ChoiceBox::new();
    r#box.reader = ChoiceReader::new(
        r#box.memory.clone(),
        Rc::new(ClassNames::new(r#box.memory.clone(), CODE)),
    );
    r#box.memory.u32(BOX + BOX_COUNT, &[0]);
    r#box.reader.track([BOX]).unwrap();
    r#box.memory.u32(BOX + BOX_COUNT, &[2]);
    r#box.state(UP);
    assert_eq!(r#box.heard(), heard_of(&[]));
}

#[test]
fn test_a_box_missing_from_a_later_sweep_is_still_read() {
    let mut r#box = ChoiceBox::new();
    r#box.reader.track([]).unwrap();
    r#box.state(UP);
    assert_eq!(r#box.heard(), heard_of(&[("Yes", true)]));
}
