//! Dowsing Machine tests, against the field's work and record as omega_ruby_item-finder.ctm read
//! them (2026-09-29).

use std::rc::Rc;

use super::fake_memory::FakeMemory;
use crate::games::gen6::field::dowsing::{LEVEL, Lights, PARENT, WORK};

const PROCESS: i64 = 0x08C65224;
const FIELD_WORK: i64 = 0x08248DD4;
const RECORD: i64 = 0x0828B748;
const OFFSET: i64 = 0x1D0;

fn lights(level: i64) -> (Rc<FakeMemory>, Lights) {
    lights_under(level, FIELD_WORK)
}

fn lights_under(level: i64, parent: i64) -> (Rc<FakeMemory>, Lights) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + WORK, &[FIELD_WORK]);
    memory.u32(FIELD_WORK + OFFSET, &[RECORD]);
    memory.u32(RECORD + PARENT, &[parent]);
    memory.u32(RECORD + LEVEL, &[level]);
    let lights = Lights::new(memory.clone(), OFFSET);
    (memory, lights)
}

#[test]
fn test_a_colour_is_said_once_two_polls_have_read_it() {
    let (_, mut reader) = lights(2);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    assert_eq!(
        reader.poll(PROCESS).unwrap().as_deref(),
        Some("dowsing green")
    );
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
}

/// Turning, the level read 2 and then 3 within a frame or two before going off.
#[test]
fn test_a_level_read_for_one_poll_is_not_said() {
    let (memory, mut reader) = lights(0);
    reader.poll(PROCESS).unwrap();
    memory.u32(RECORD + LEVEL, &[3]);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    memory.u32(RECORD + LEVEL, &[0]);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
}

#[test]
fn test_lights_going_off_are_said_and_off_at_first_is_not() {
    let (memory, mut reader) = lights(0);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, None]
    );
    memory.u32(RECORD + LEVEL, &[4]);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, Some("dowsing red".to_string())]
    );
    memory.u32(RECORD + LEVEL, &[0]);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, Some("dowsing nothing".to_string())]
    );
}

#[test]
fn test_a_record_not_pointing_back_at_the_work_is_not_read() {
    let (_, mut reader) = lights_under(1, 0x08000000);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, None]
    );
}
