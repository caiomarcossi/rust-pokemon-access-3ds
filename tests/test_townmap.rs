//! The AreaNav's views say which of its layouts it draws (spikes 181 and 183).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::gen6::screens::townmap::{self, VIEWS};

const NUTS: &str = "N3app7townmap8NutsViewE";
const VIEW: i64 = 0x08401BE4;
const UTIL: i64 = 0x08402324;
const TABLE: i64 = 0x084022E4;
const HANDLE: i64 = 0x082486EC;
const WORK: i64 = 0x0820F000;

/// The berry list's view, drawn while its byte at +0xAC is 1, its one layout's work at the
/// handle its `G2DUtil` lists.
fn berry_list(shown: u8) -> (Rc<FakeMemory>, townmap::Views) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(VIEW, &[classes.add(NUTS)]);
    let drawn_at = VIEWS.iter().find(|(class, _)| *class == NUTS).unwrap().1;
    memory.write(VIEW + drawn_at, &[shown]);
    memory.u32(VIEW + townmap::UTIL, &[UTIL]);
    memory.u32(UTIL, &[classes.add(townmap::UTIL_CLASS)]);
    memory.u32(UTIL + townmap::LAYOUTS, &[TABLE, 1]);
    memory.u32(TABLE, &[HANDLE, 0x101]);
    memory.u32(HANDLE + townmap::WORK, &[WORK]);
    let mut views = townmap::Views::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    );
    views
        .track(&IndexMap::from([(VIEW, NUTS.to_string())]), false)
        .unwrap();
    (memory, views)
}

#[test]
fn test_a_view_put_away_puts_its_layouts_away() {
    assert_eq!(berry_list(0).1.put_away().unwrap(), HashSet::from([WORK]));
}

#[test]
fn test_a_view_drawn_puts_nothing_away() {
    assert_eq!(berry_list(1).1.put_away().unwrap(), HashSet::new());
}

#[test]
fn test_a_view_whose_memory_is_taken_says_nothing() {
    let (memory, views) = berry_list(0);
    memory.u32(VIEW, &[0]);
    assert_eq!(views.put_away().unwrap(), HashSet::new());
}
