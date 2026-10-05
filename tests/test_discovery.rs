//! Discovery tests: objects found by class in a fake emulator's heap.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_emulator::FakeEmulator;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::discovery::{Discovery, Found, Walk};
use crate::core::memory::Memory;
use crate::core::rpc::{MAX_READ, Rpc, UdpRpc};
use crate::core::snapshot::PROBE_STEP;

const WINDOW: &str = "N5print6MsgWinE";
const BUFFER: &str = "N3gfl3str6StrBufE";
const HEAP: i64 = 0x08000000;

/// `struct.pack("<I", value)` after `skip` bytes, `ljust`ed to `size` with `fill`.
fn word(skip: usize, value: i64, size: usize, fill: u8) -> Vec<u8> {
    let mut out = vec![0; skip];
    out.extend_from_slice(&(value as u32).to_le_bytes());
    out.resize(size, fill);
    out
}

/// A page whose word at `skip` is `value`, and the rest zeros.
fn page(skip: usize, value: i64) -> Vec<u8> {
    word(skip, value, MAX_READ as usize, 0)
}

fn connect(emu: &FakeEmulator) -> Rc<dyn Rpc> {
    Rc::new(UdpRpc::connect(emu.port()).unwrap())
}

fn nothing() -> Rc<dyn Fn()> {
    Rc::new(|| {})
}

/// A code image naming two classes, and their vtables.
fn executable() -> (Vec<u8>, HashMap<&'static str, i64>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let vtables = HashMap::from([(WINDOW, classes.add(WINDOW)), (BUFFER, classes.add(BUFFER))]);
    (memory.read(CODE.0, 0x2000).unwrap(), vtables)
}

fn discovery(rpc: &Rc<dyn Rpc>, image: &[u8]) -> Discovery {
    let mut found = Discovery::with(
        rpc.clone(),
        image,
        CODE.0,
        &[WINDOW, BUFFER],
        nothing(),
        1,
        None,
        &[],
    )
    .unwrap();
    found.heap = vec![(HEAP, HEAP + 2 * MAX_READ)];
    found
}

fn sorted(pages: &IndexMap<i64, Vec<u8>>) -> Vec<i64> {
    let mut keys: Vec<i64> = pages.keys().copied().collect();
    keys.sort();
    keys
}

#[test]
fn test_a_sweep_finds_objects_of_the_classes_asked_for_by_name() {
    let (image, vtables) = executable();
    let heap = HashMap::from([
        (HEAP, page(4, vtables[WINDOW])),
        (HEAP + MAX_READ, page(0, vtables[BUFFER])),
    ]);
    let emu = FakeEmulator::new(heap);
    let rpc = connect(&emu);
    let found = discovery(&rpc, &image).sweep().unwrap();
    assert_eq!(found.of(&[WINDOW]), [HEAP + 4]);
    assert_eq!(found.of(&[BUFFER]), [HEAP + MAX_READ]);
    assert_eq!(sorted(&found.pages), [HEAP, HEAP + MAX_READ]);
}

/// The bottom of the fake game's modules' memory, past its executable.
const MODULES: i64 = CODE.0 + 0x10000;

/// A loaded module at `base`, as `modules` reads one: its header, its name, and these
/// classes' vtables inside it, each keeping a first slot in the executable. Returns them.
fn module(
    memory: &Rc<FakeMemory>,
    base: i64,
    name: &str,
    classes: &[&str],
    size: i64,
    following: i64,
    previous: Option<i64>,
) -> HashMap<String, i64> {
    memory.write(base + 0x80, b"FIXD");
    memory.u32(base + 0x88, &[following, previous.unwrap_or(base), size]);
    memory.u32(base + 0xC0, &[base + 0x100, name.len() as i64 + 1]);
    let mut text = name.as_bytes().to_vec();
    text.push(0);
    memory.write(base + 0x100, &text);
    let inside = FakeClasses::starting(memory.clone(), base + 0x200);
    classes
        .iter()
        .map(|name| (name.to_string(), inside.add(name)))
        .collect()
}

/// What a fake emulator serves of `memory` from `low` to `high`.
fn served(memory: &FakeMemory, low: i64, high: i64) -> HashMap<i64, Vec<u8>> {
    (low..high)
        .step_by(MAX_READ as usize)
        .map(|at| (at, memory.read(at, MAX_READ).unwrap()))
        .collect()
}

/// `emu.memory.update(pages)`.
fn update(emu: &FakeEmulator, pages: HashMap<i64, Vec<u8>>) {
    for (at, data) in pages {
        emu.set_page(at, data);
    }
}

/// `field::MapBlock`'s vtable is in the field's DLL, past the code the runtime read on the
/// title screen, so its sweeps found every other object on the field and never a block
/// (2026-09-23).
#[test]
fn test_a_class_in_a_dll_is_learned_once_the_game_has_loaded_it() {
    let memory = FakeMemory::new();
    FakeClasses::new(memory.clone()).add(WINDOW);
    let image = memory.read(CODE.0, PROBE_STEP).unwrap();
    let dll = FakeMemory::new();
    let buffer = module(&dll, MODULES, "DllField", &[BUFFER], 0x2000, 0, None)[BUFFER];
    let heap = HashMap::from([(HEAP, page(0, buffer))]);
    let emu = FakeEmulator::new(heap);
    let rpc = connect(&emu);
    let mut found = discovery(&rpc, &image);
    assert_eq!(found.missing(&[WINDOW, BUFFER]), [BUFFER]);
    // No module loaded yet: nothing to learn from.
    assert_eq!(found.learn(CODE).unwrap(), Vec::<String>::new());
    assert_eq!(found.sweep().unwrap().of(&[BUFFER]), Vec::<i64>::new());
    update(&emu, served(&dll, MODULES, MODULES + 0x2000));
    // Read a look after it first shows, once it has settled (movie5, 2026-09-30).
    assert_eq!(found.learn(CODE).unwrap(), Vec::<String>::new());
    assert!(found.unsettled);
    assert_eq!(found.learn(CODE).unwrap(), [BUFFER]);
    assert!(!found.unsettled);
    assert_eq!(found.missing(&[WINDOW, BUFFER]), Vec::<String>::new());
    assert_eq!(found.sweep().unwrap().of(&[BUFFER]), [HEAP]);
}

/// A module is loaded after whatever is loaded beneath it: the Box's went in after the PC
/// menu's, and the name entry's keyboard was seen at three addresses (spike 140). Its classes sit
/// at the same offsets in it wherever it lands, so it is read the first time only.
#[test]
fn test_a_module_is_read_once_and_found_again_wherever_it_loads() {
    let memory = FakeMemory::new();
    FakeClasses::new(memory.clone()).add(WINDOW);
    let image = memory.read(CODE.0, PROBE_STEP).unwrap();
    let alone = FakeMemory::new();
    module(&alone, MODULES, "DllBox", &[BUFFER], 0x2000, 0, None);
    let emu = FakeEmulator::new(served(&alone, MODULES, MODULES + 0x2000));
    let rpc = connect(&emu);
    let mut found = discovery(&rpc, &image);
    found.learn(CODE).unwrap();
    assert_eq!(found.learn(CODE).unwrap(), [BUFFER]);
    let stacked = FakeMemory::new();
    module(
        &stacked,
        MODULES,
        "DllPCMenu",
        &[],
        0x1000,
        MODULES + 0x1000,
        Some(MODULES + 0x1000),
    );
    let box_ = module(
        &stacked,
        MODULES + 0x1000,
        "DllBox",
        &[BUFFER],
        0x2000,
        0,
        Some(MODULES),
    )[BUFFER];
    emu.clear_pages();
    update(&emu, served(&stacked, MODULES, MODULES + 0x3000));
    emu.set_page(HEAP, page(0, box_));
    let reads = emu.reads();
    assert_eq!(found.learn(CODE).unwrap(), [BUFFER]);
    // The headers, the names and the PC menu's 4 KB; never the Box's 8 KB again.
    assert!(emu.reads() - reads < 16);
    assert_eq!(found.sweep().unwrap().of(&[BUFFER]), [HEAP]);
}

/// Started on the field, the code measured as pa3ds attached ran on through the field's module;
/// a battle's, loaded where the field's had been, lay below where that code ended, and the
/// code past it was all that was read, so the battle's gauges were never found.
#[test]
fn test_a_module_below_where_the_code_ended_at_attach_is_learned() {
    let memory = FakeMemory::new();
    FakeClasses::new(memory.clone()).add(WINDOW);
    module(&memory, MODULES, "DllField", &[], 0x2000, 0, None);
    let image = memory.read(CODE.0, MODULES + 0x2000 - CODE.0).unwrap();
    let battle = FakeMemory::new();
    module(&battle, MODULES, "DllBattle", &[BUFFER], 0x2000, 0, None);
    let emu = FakeEmulator::new(served(&battle, MODULES, MODULES + 0x2000));
    let rpc = connect(&emu);
    let mut found = discovery(&rpc, &image);
    found.learn(CODE).unwrap();
    assert_eq!(found.learn(CODE).unwrap(), [BUFFER]);
}

/// Read as its header first appeared, a battle's module was read as Azahar freed the part of
/// it past what it keeps, and Azahar crashed (movie5, 2026-09-30). A module whose header changes
/// from one look to the next, 'CRO0' to 'FIXD' as it is fixed, is read at the look after.
#[test]
fn test_a_module_still_loading_is_not_read_until_it_has_settled() {
    let memory = FakeMemory::new();
    FakeClasses::new(memory.clone()).add(WINDOW);
    let image = memory.read(CODE.0, PROBE_STEP).unwrap();
    let battle = FakeMemory::new();
    module(&battle, MODULES, "DllBattle", &[BUFFER], 0x2000, 0, None);
    let pages = served(&battle, MODULES, MODULES + 0x2000);
    let mut first = pages[&MODULES][..0x80].to_vec();
    first.extend_from_slice(b"CRO0");
    first.extend_from_slice(&pages[&MODULES][0x84..]);
    let mut loading = pages.clone();
    loading.insert(MODULES, first);
    let emu = FakeEmulator::new(loading);
    let rpc = connect(&emu);
    let mut found = discovery(&rpc, &image);
    assert_eq!(found.learn(CODE).unwrap(), Vec::<String>::new());
    update(&emu, pages);
    assert_eq!(found.learn(CODE).unwrap(), Vec::<String>::new());
    assert_eq!(found.learn(CODE).unwrap(), [BUFFER]);
}

/// A battle grows the heap and builds its message window in the new memory, which no sweep has
/// been near (battles.ctm replay, 2026-09-18).
#[test]
fn test_measuring_the_heap_says_whether_it_changed() {
    let (image, vtables) = executable();
    let window = page(0, vtables[WINDOW]);
    let emu = FakeEmulator::new(HashMap::from([(HEAP, window.clone())]));
    let rpc = connect(&emu);
    let mut found = Discovery::with(
        rpc,
        &image,
        CODE.0,
        &[WINDOW, BUFFER],
        nothing(),
        1,
        None,
        &[],
    )
    .unwrap();
    assert!(found.measure_heap().unwrap());
    assert!(!found.measure_heap().unwrap());
    emu.set_page(HEAP + 0x100000, window);
    assert!(found.measure_heap().unwrap());
}

#[test]
fn test_classes_with_no_vtable_are_reported_missing() {
    let (image, _vtables) = executable();
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    let found = Discovery::new(rpc, &image, CODE.0, &[WINDOW, "N5print4NoneE"], nothing()).unwrap();
    assert_eq!(found.missing(&[WINDOW, "N5print4NoneE"]), ["N5print4NoneE"]);
}

#[test]
fn test_holders_are_the_words_that_point_at_an_object() {
    let mut bytes = vec![0; 8];
    bytes.extend_from_slice(&0x08001234u32.to_le_bytes());
    bytes.resize(MAX_READ as usize, 0);
    let found = Found {
        pages: IndexMap::from([(HEAP, bytes)]),
        ..Found::default()
    };
    assert_eq!(found.holders(&[0x08001234]).unwrap(), [HEAP + 8]);
}

/// The main menu's works sat in the block the title screen's resource had been in; the moment
/// that was freed the block's first page read blank, the block dropped out of the measurement
/// and the menu was never swept (main_menu.ctm, spike 72).
#[test]
fn test_a_span_measured_once_is_kept_and_reaches_a_block_either_side() {
    let (image, vtables) = executable();
    let window = page(0, vtables[WINDOW]);
    let emu = FakeEmulator::new(HashMap::from([(HEAP + PROBE_STEP, window)]));
    let rpc = connect(&emu);
    let mut found = Discovery::with(
        rpc,
        &image,
        CODE.0,
        &[WINDOW, BUFFER],
        nothing(),
        1,
        None,
        &[],
    )
    .unwrap();
    assert!(found.measure_heap().unwrap());
    assert_eq!(found.heap, [(HEAP, HEAP + 3 * PROBE_STEP)]);
    emu.remove_page(HEAP + PROBE_STEP);
    assert!(!found.measure_heap().unwrap());
    assert_eq!(found.heap, [(HEAP, HEAP + 3 * PROBE_STEP)]);
}

/// The memory the game has just allocated, where its new objects are (spike 110).
#[test]
fn test_a_sweep_of_spans_reads_them_alone_and_says_it_is_partial() {
    let (image, vtables) = executable();
    let far = HEAP + 0x40000;
    let emu = FakeEmulator::new(HashMap::from([
        (HEAP, page(0, vtables[WINDOW])),
        (far, page(0, vtables[BUFFER])),
    ]));
    let rpc = connect(&emu);
    let found = discovery(&rpc, &image)
        .sweep_spans(&[(far + 0x10, far + 0x40)])
        .unwrap();
    assert!(found.partial);
    assert_eq!(found.of(&[BUFFER]), [far]);
    assert_eq!(found.of(&[WINDOW]), Vec::<i64>::new());
    assert_eq!(sorted(&found.pages), [far]);
}

/// Joined by `merge`'s megabyte, the new blocks of movie4 held 138 MB and their sweeps read
/// 951 MB, 33 s of the replay, for nothing the watch needed (spike 133).
#[test]
fn test_spans_apart_are_read_apart_and_not_the_memory_between() {
    let (image, vtables) = executable();
    let (between, far) = (HEAP + 0x20000, HEAP + 0x40000);
    let emu = FakeEmulator::new(HashMap::from([
        (HEAP, page(0, vtables[BUFFER])),
        (between, page(0, vtables[WINDOW])),
        (far, page(0, vtables[BUFFER])),
    ]));
    let rpc = connect(&emu);
    let found = discovery(&rpc, &image)
        .sweep_spans(&[(HEAP + 0x10, HEAP + 0x40), (far + 0x10, far + 0x40)])
        .unwrap();
    assert_eq!(found.of(&[BUFFER]), [HEAP, far]);
    assert_eq!(found.of(&[WINDOW]), Vec::<i64>::new());
    assert_eq!(sorted(&found.pages), [HEAP, far]);
}

#[test]
fn test_a_sweep_of_no_spans_reads_nothing() {
    let (image, _vtables) = executable();
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    let found = discovery(&rpc, &image).sweep_spans(&[]).unwrap();
    assert_eq!(emu.reads(), 0);
    assert!(found.partial && found.heap.is_empty());
}

/// Sun's GPU buffers sit beside the blocks it sweeps, and reading one took the emulator down
/// (spike 111): the profile's walk reads the heap, exactly, and every sweep is one walk, with
/// polling carried on through it.
#[test]
fn test_a_heap_the_profile_walks_is_what_every_sweep_reads() {
    let (image, vtables) = executable();
    let inside = HEAP + PROBE_STEP;
    let window = word(0, vtables[WINDOW], 0x20, 0);
    let calls: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    let walk: Walk = {
        let calls = calls.clone();
        let window = window.clone();
        Rc::new(move |between: &dyn Fn()| {
            between();
            calls.borrow_mut().push("walk");
            Ok(IndexMap::from([(inside, window.clone())]))
        })
    };
    let between: Rc<dyn Fn()> = {
        let calls = calls.clone();
        Rc::new(move || calls.borrow_mut().push("poll"))
    };

    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    let mut found = Discovery::with(
        rpc,
        &image,
        CODE.0,
        &[WINDOW, BUFFER],
        between,
        512,
        Some(walk),
        &[],
    )
    .unwrap();
    assert!(!found.measure_heap().unwrap());
    let (first, second) = (found.sweep().unwrap(), found.sweep().unwrap());
    let reads = emu.reads();
    assert_eq!(first.of(&[WINDOW]), [inside]);
    assert_eq!(second.of(&[WINDOW]), [inside]);
    assert_eq!(found.heap, [(inside, inside + 0x20)]);
    assert_eq!(*calls.borrow(), ["poll", "walk", "poll", "walk"]);
    assert_eq!(reads, 0);
}

/// Sun's new blocks are swept as the walk reads them, and never a page around them: the byte
/// after one may be the GPU's (spike 123).
#[test]
fn test_a_sweep_of_spans_for_a_walked_heap_reads_them_to_the_byte() {
    let (image, vtables) = executable();
    let inside = HEAP + PROBE_STEP + 0x10;
    let mut bytes = vec![0; 0x10];
    bytes.extend(word(0, vtables[WINDOW], 0x20, 0));
    bytes.resize(MAX_READ as usize, 1);
    let emu = FakeEmulator::new(HashMap::from([(HEAP + PROBE_STEP, bytes)]));
    let rpc = connect(&emu);
    let walk: Walk = Rc::new(|_between: &dyn Fn()| Ok(IndexMap::new()));
    let found = Discovery::with(
        rpc,
        &image,
        CODE.0,
        &[WINDOW, BUFFER],
        nothing(),
        512,
        Some(walk),
        &[],
    )
    .unwrap()
    .sweep_spans(&[(inside, inside + 0x20)])
    .unwrap();
    assert!(found.partial);
    assert_eq!(found.of(&[WINDOW]), [inside]);
    let lengths: IndexMap<i64, usize> = found
        .pages
        .iter()
        .map(|(&at, data)| (at, data.len()))
        .collect();
    assert_eq!(lengths, IndexMap::from([(inside, 0x20)]));
}
