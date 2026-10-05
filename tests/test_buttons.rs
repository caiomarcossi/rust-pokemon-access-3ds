//! The game's button managers and touch bars: whose buttons take taps now (spike 189).

use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::FakeMemory;
use crate::games::gen6::buttons::{
    BAR_MANAGER, BUTTONS, Buttons, HANDLE, INPUT, LISTENER, LOCKED, WORK,
};

const MANAGER_VTABLE: i64 = 0x005DCFFC;
const BAR_VTABLE: i64 = 0x00705B64;
/// The party's managers: its list's, its held items' corners', and its bottom bar's.
const LIST: i64 = 0x082DFEFC;
const CORNERS: i64 = 0x082E0348;
const BAR: i64 = 0x08399548;
const PARTY_WORK: i64 = 0x0825C208;
const BAR_WORK: i64 = 0x0825C820;
const TOUCH_BAR: i64 = 0x08369574;

/// A manager whose buttons are all in the layout of this work.
fn manager(memory: &FakeMemory, at: i64, work: i64, locked: bool, buttons: i64) {
    let (table, handle) = (at + 0x100, at + 0x200);
    memory.u32(at, &[MANAGER_VTABLE]);
    memory.u32(at + BUTTONS, &[table, buttons]);
    memory.write(at + LOCKED, &[locked as u8]);
    for i in 0..buttons {
        let button = at + 0x300 + 0x60 * i;
        memory.u32(table + 4 * i, &[button]);
        memory.u32(button + HANDLE, &[handle]);
    }
    memory.u32(handle + WORK, &[work]);
}

fn touch_bar(memory: &FakeMemory, taking: bool) {
    memory.u32(TOUCH_BAR, &[BAR_VTABLE]);
    memory.u32(TOUCH_BAR + BAR_MANAGER, &[BAR]);
    memory.u32(TOUCH_BAR + LISTENER, &[TOUCH_BAR + 0x74]);
    memory.write(TOUCH_BAR + INPUT, &[taking as u8]);
}

fn party(locked: bool, taking: bool) -> Rc<FakeMemory> {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, locked, 2);
    manager(&memory, CORNERS, PARTY_WORK, true, 2);
    manager(&memory, BAR, BAR_WORK, false, 4);
    touch_bar(&memory, taking);
    memory
}

fn set(works: &[i64]) -> HashSet<i64> {
    works.iter().copied().collect()
}

/// Under "Do what with Blaze?" the party's list's managers are locked, and its touch bar takes
/// no input though its own manager is open (movie22, 131 s); at the list, only the corners'
/// manager is locked, and the list takes taps through the other (110 s).
#[test]
fn test_a_layout_whose_managers_are_all_locked_takes_no_tap() {
    let memory = party(false, true);
    let mut buttons = Buttons::new(memory.clone());
    buttons
        .track([LIST, CORNERS, BAR], [TOUCH_BAR], false)
        .unwrap();
    assert_eq!(buttons.idle().unwrap(), set(&[]));
    memory.write(LIST + LOCKED, b"\x01");
    memory.write(TOUCH_BAR + INPUT, b"\x00");
    assert_eq!(buttons.idle().unwrap(), set(&[PARTY_WORK, BAR_WORK]));
}

/// A manager whose memory holds something else now, and a word of a module's file kept in the
/// heap, which names a touch bar's vtable with code where its listener would be (spike 189).
#[test]
fn test_a_freed_manager_and_a_touch_bar_with_no_listener_say_nothing() {
    let memory = party(true, true);
    let mut buttons = Buttons::new(memory.clone());
    buttons
        .track([LIST, CORNERS, BAR], [TOUCH_BAR], false)
        .unwrap();
    assert_eq!(buttons.idle().unwrap(), set(&[PARTY_WORK]));
    memory.u32(LIST, &[0]);
    memory.u32(CORNERS, &[0]);
    assert_eq!(buttons.idle().unwrap(), set(&[]));
    memory.write(TOUCH_BAR + INPUT, b"\x00");
    memory.u32(TOUCH_BAR + LISTENER, &[0xE1A06001]);
    assert_eq!(buttons.idle().unwrap(), set(&[]));
}
