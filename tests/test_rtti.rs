//! RTTI tests, against a synthetic executable image laid out the way the Itanium ABI lays it out.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::Counting;
use crate::core::rtti::{ClassNames, demangle, find_objects, find_vtables};
use crate::core::snapshot::{PAGE, Snapshot};

const BASE: i64 = 0x00100000;
const SIZE: i64 = 0x1000;
const CODE: (i64, i64) = (BASE, BASE + SIZE);

const NAME: i64 = 0x100;
const TYPEINFO: i64 = 0x200;
const TYPEINFO_KIND: i64 = 0x280;
const VTABLE: i64 = 0x30C;
const SECONDARY: i64 = 0x40C;
const FUNCTION: i64 = 0x800;

/// `struct.pack_into("<I...", out, at, *values)`; a negative value is its two's complement.
fn pack_into(out: &mut [u8], at: i64, values: &[i64]) {
    for (index, &value) in values.iter().enumerate() {
        let start = at as usize + index * 4;
        out[start..start + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
}

fn image(name: &[u8]) -> Vec<u8> {
    let mut out = vec![0; SIZE as usize];
    let at = NAME as usize;
    out[at..at + name.len()].copy_from_slice(name);
    out[at + name.len()] = 0;
    pack_into(&mut out, TYPEINFO, &[BASE + TYPEINFO_KIND, BASE + NAME]);
    pack_into(&mut out, VTABLE - 8, &[0, BASE + TYPEINFO, BASE + FUNCTION]);
    out
}

fn menu_window() -> Vec<u8> {
    image(b"N3app4tool10MenuWindowE")
}

fn memory(raw: &[u8], extra: &[(i64, &[u8])]) -> Snapshot {
    let page_size = PAGE as usize;
    let mut pages: IndexMap<i64, Vec<u8>> = IndexMap::new();
    for at in (0..SIZE).step_by(page_size) {
        let mut page = raw[at as usize..(at as usize + page_size).min(raw.len())].to_vec();
        page.resize(page_size, 0);
        pages.insert(BASE + at, page);
    }
    for &(address, data) in extra {
        let mut page = data.to_vec();
        page.resize(page_size, 0);
        pages.insert(address, page);
    }
    Snapshot { pages }
}

#[test]
fn test_find_vtables_follows_name_to_typeinfo_to_vtable() {
    assert_eq!(
        find_vtables(&menu_window(), BASE, "N3app4tool10MenuWindowE", 0, None).unwrap(),
        [BASE + VTABLE]
    );
}

/// A class with two bases has a second vtable with a negative offset-to-top.
#[test]
fn test_find_vtables_skips_a_secondary_vtable() {
    let mut raw = menu_window();
    pack_into(
        &mut raw,
        SECONDARY - 8,
        &[-8, BASE + TYPEINFO, BASE + FUNCTION],
    );
    assert_eq!(
        find_vtables(&raw, BASE, "N3app4tool10MenuWindowE", 0, None).unwrap(),
        [BASE + VTABLE]
    );
}

#[test]
fn test_find_vtables_does_not_match_a_longer_name_ending_the_same_way() {
    let raw = image(b"N3app4tool14BigMenuWindowE");
    assert_eq!(
        find_vtables(&raw, BASE, "MenuWindowE", 0, None).unwrap(),
        Vec::<i64>::new()
    );
}

#[test]
fn test_find_vtables_of_an_absent_class_is_empty() {
    assert_eq!(
        find_vtables(&menu_window(), BASE, "N2nw3lyt7TextBoxE", 0, None).unwrap(),
        Vec::<i64>::new()
    );
}

#[test]
fn test_find_objects_finds_word_aligned_vtable_pointers() {
    let heap = 0x08000000;
    let mut page = vec![0; PAGE as usize];
    pack_into(&mut page, 0x08, &[BASE + VTABLE]);
    // misaligned: not an object
    page[0x21..0x25].copy_from_slice(&((BASE + VTABLE) as u32).to_le_bytes());
    pack_into(&mut page, 0x40, &[BASE + SECONDARY]);
    let found = find_objects(
        &IndexMap::from([(heap, page)]),
        &[BASE + VTABLE, BASE + SECONDARY],
    )
    .unwrap();
    assert_eq!(
        found,
        IndexMap::from([
            (heap + 0x08, BASE + VTABLE),
            (heap + 0x40, BASE + SECONDARY)
        ])
    );
}

#[test]
fn test_of_vtable_names_the_class() {
    let names = ClassNames::new(Rc::new(memory(&menu_window(), &[])), CODE);
    assert_eq!(
        names.of_vtable(BASE + VTABLE).unwrap().as_deref(),
        Some("N3app4tool10MenuWindowE")
    );
}

#[test]
fn test_of_object_reads_the_vtable_from_the_object() {
    let heap = 0x08000000;
    let word = ((BASE + VTABLE) as u32).to_le_bytes();
    let names = ClassNames::new(Rc::new(memory(&menu_window(), &[(heap, &word)])), CODE);
    assert_eq!(
        names.of_object(heap).unwrap().as_deref(),
        Some("N3app4tool10MenuWindowE")
    );
}

#[test]
fn test_a_heap_pointer_is_not_a_vtable() {
    let names = ClassNames::new(Rc::new(memory(&menu_window(), &[])), CODE);
    assert_eq!(names.of_vtable(0x08001234).unwrap(), None);
}

#[test]
fn test_a_misaligned_word_is_not_a_vtable() {
    let names = ClassNames::new(Rc::new(memory(&menu_window(), &[])), CODE);
    assert_eq!(names.of_vtable(BASE + VTABLE + 1).unwrap(), None);
}

#[test]
fn test_a_typeinfo_pointing_at_non_name_bytes_is_not_a_class() {
    let mut raw = menu_window();
    raw[NAME as usize..NAME as usize + 4].copy_from_slice(b"\xff\xfe\x00\x00");
    assert_eq!(
        ClassNames::new(Rc::new(memory(&raw, &[])), CODE)
            .of_vtable(BASE + VTABLE)
            .unwrap(),
        None
    );
}

#[test]
fn test_names_are_cached_per_vtable() {
    let counting = Counting::new(Rc::new(memory(&menu_window(), &[])));
    let names = ClassNames::new(counting.clone(), CODE);
    names.of_vtable(BASE + VTABLE).unwrap();
    let first = counting.reads.get();
    names.of_vtable(BASE + VTABLE).unwrap();
    assert_eq!(counting.reads.get(), first);
}

#[test]
fn test_demangle_nested_name() {
    assert_eq!(demangle("N2nw3lyt4PaneE"), "nw::lyt::Pane");
}

#[test]
fn test_demangle_plain_name() {
    assert_eq!(demangle("6MsgWin"), "MsgWin");
}

#[test]
fn test_demangle_leaves_templates_alone() {
    let name = "N3gfl4base9FixedListIiEE";
    assert_eq!(demangle(name), name);
}
