//! The code modules loaded: found by their headers and listed in the order they were loaded.

use std::collections::HashMap;
use std::rc::Rc;

use super::fake_emulator::FakeEmulator;
use super::fake_memory::{CODE, FakeMemory};
use crate::core::memory::Memory;
use crate::core::modules::{Module, Modules};
use crate::core::rpc::{MAX_READ, UdpRpc};

const BOTTOM: i64 = CODE.0 + 0x10000;

fn header(memory: &FakeMemory, base: i64, name: &str, size: i64, following: i64, previous: i64) {
    memory.write(base + 0x80, b"FIXD");
    memory.u32(base + 0x88, &[following, previous, size]);
    memory.u32(base + 0xC0, &[base + 0x100, name.len() as i64 + 1]);
    let mut text = name.as_bytes().to_vec();
    text.push(0);
    memory.write(base + 0x100, &text);
}

fn served(memory: &FakeMemory) -> HashMap<i64, Vec<u8>> {
    (BOTTOM..BOTTOM + 0x4000)
        .step_by(MAX_READ as usize)
        .map(|at| (at, memory.read(at, MAX_READ).unwrap()))
        .collect()
}

fn module(name: &str, base: i64, size: i64, magic: &[u8]) -> Module {
    Module {
        name: name.to_string(),
        base,
        size,
        magic: magic.to_vec(),
    }
}

#[test]
fn test_nothing_is_listed_before_a_module_is_loaded() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = UdpRpc::connect(emu.port()).unwrap();
    assert_eq!(Modules::new(Rc::new(rpc), CODE).loaded().unwrap(), vec![]);
}

/// The first loaded is the lowest while it stays, but not once it has gone and another has taken
/// the bottom: its previous is the last loaded, whose next is 0.
#[test]
fn test_modules_are_listed_in_the_order_they_were_loaded() {
    let memory = FakeMemory::new();
    header(
        &memory,
        BOTTOM + 0x2000,
        "DllPCMenu",
        0x1000,
        BOTTOM,
        BOTTOM,
    );
    header(&memory, BOTTOM, "DllBox", 0x1000, 0, BOTTOM + 0x2000);
    let emu = FakeEmulator::new(served(&memory));
    let rpc = UdpRpc::connect(emu.port()).unwrap();
    assert_eq!(
        Modules::new(Rc::new(rpc), CODE).loaded().unwrap(),
        vec![
            module("DllPCMenu", BOTTOM + 0x2000, 0x1000, b"FIXD"),
            module("DllBox", BOTTOM, 0x1000, b"FIXD"),
        ]
    );
}

/// A few reads a look, where finding the bottom took one a page from the start of the code.
#[test]
fn test_the_bottom_is_looked_at_first_once_known() {
    let memory = FakeMemory::new();
    header(&memory, BOTTOM, "DllField", 0x1000, 0, BOTTOM);
    let emu = FakeEmulator::new(served(&memory));
    let rpc = UdpRpc::connect(emu.port()).unwrap();
    let mut modules = Modules::new(Rc::new(rpc), CODE);
    let names = |found: Vec<Module>| -> Vec<String> { found.into_iter().map(|m| m.name).collect() };
    assert_eq!(names(modules.loaded().unwrap()), ["DllField"]);
    let reads = emu.reads();
    assert_eq!(names(modules.loaded().unwrap()), ["DllField"]);
    assert!(emu.reads() - reads <= 4);
}
