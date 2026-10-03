//! Snapshot tests. No emulator involved except through the fake.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use super::fake_emulator::FakeEmulator;
use crate::core::memory::Memory;
use crate::core::rpc::{Rpc, UdpRpc};
use crate::core::snapshot::{
    CHUNK_PAGES, CHUNK_PAUSE, PAGE, Snapshot, churn, merge, regions, sweep,
};
use crate::py::{Kind, Result};

const P: usize = PAGE as usize;

fn snap(pages: &[(i64, &[u8])]) -> Snapshot {
    Snapshot {
        pages: pages
            .iter()
            .map(|&(base, data)| {
                let mut page = data.to_vec();
                page.resize(P, 0);
                (base, page)
            })
            .collect(),
    }
}

fn pages_of(pages: &[(i64, Vec<u8>)]) -> Snapshot {
    Snapshot {
        pages: pages.iter().cloned().collect(),
    }
}

/// pytest's `tmp_path`: a directory of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("pa3ds-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rpc(emu: &FakeEmulator) -> UdpRpc {
    UdpRpc::connect(emu.port()).unwrap()
}

#[test]
fn test_read_returns_bytes_from_the_captured_page() {
    let mut data = vec![0; 0x10];
    data.extend_from_slice(b"hello");
    let s = snap(&[(0x08608800, &data)]);
    assert_eq!(s.read(0x08608810, 5).unwrap(), b"hello");
}

#[test]
fn test_read_spans_page_boundaries() {
    let s = pages_of(&[(0x1000, vec![b'a'; P]), (0x1400, vec![b'b'; P])]);
    assert_eq!(s.read(0x1000 + PAGE - 2, 4).unwrap(), b"aabb");
}

#[test]
fn test_absent_pages_read_as_zeros_not_errors() {
    assert_eq!(
        snap(&[(0x1000, b"x")]).read(0x9999_0000, 8).unwrap(),
        vec![0; 8]
    );
}

#[test]
fn test_read_rejects_a_negative_size() {
    let error = snap(&[(0x1000, b"x")]).read(0x1000, -1).unwrap_err();
    assert!(
        error.is(Kind::ValueError) && error.message.contains("negative"),
        "{error:?}"
    );
}

#[test]
fn test_ranges_merges_adjacent_pages_and_splits_at_gaps() {
    let s = pages_of(&[
        (0x1000, vec![0; P]),
        (0x1400, vec![0; P]),
        (0x2000, vec![0; P]),
    ]);
    assert_eq!(s.ranges(), vec![(0x1000, 0x1800), (0x2000, 0x2400)]);
}

/// The heap is pitted with unused pages; a map that lists each hole cannot narrow a sweep.
#[test]
fn test_merge_joins_regions_separated_by_less_than_the_gap() {
    assert_eq!(
        merge(&[(0x1000, 0x2000), (0x2400, 0x3000)], 0x800),
        vec![(0x1000, 0x3000)]
    );
}

#[test]
fn test_merge_keeps_regions_a_real_distance_apart() {
    assert_eq!(
        merge(
            &[(0x08000000, 0x09000000), (0x14000000, 0x15000000)],
            0x100000
        ),
        vec![(0x08000000, 0x09000000), (0x14000000, 0x15000000)]
    );
}

#[test]
fn test_merge_sorts_before_joining() {
    assert_eq!(
        merge(&[(0x2000, 0x3000), (0x1000, 0x2000)], 0),
        vec![(0x1000, 0x3000)]
    );
}

#[test]
fn test_merge_of_nothing_is_nothing() {
    assert_eq!(merge(&[], 0x1000), vec![]);
}

#[test]
fn test_diff_reports_address_and_both_values() {
    let mut a = vec![1, 2];
    a.resize(P, 0);
    let mut b = vec![1, 9];
    b.resize(P, 0);
    let (a, b) = (pages_of(&[(0x1000, a)]), pages_of(&[(0x1000, b)]));
    assert_eq!(
        a.diff(&b).collect::<Result<Vec<_>>>().unwrap(),
        vec![(0x1001, 0x02, 0x09)]
    );
}

#[test]
fn test_diff_treats_a_page_missing_on_one_side_as_zeros() {
    let mut a = vec![7];
    a.resize(P, 0);
    let a = pages_of(&[(0x1000, a)]);
    let empty = Snapshot {
        pages: IndexMap::new(),
    };
    assert_eq!(
        a.diff(&empty).collect::<Result<Vec<_>>>().unwrap(),
        vec![(0x1000, 0x07, 0x00)]
    );
}

#[test]
fn test_identical_snapshots_diff_to_nothing() {
    let a = snap(&[(0x1000, b"same")]);
    assert_eq!(
        a.diff(&snap(&[(0x1000, b"same")]))
            .collect::<Result<Vec<_>>>()
            .unwrap(),
        vec![]
    );
}

#[test]
fn test_save_then_load_round_trips() {
    let tmp = TmpPath::new("test_save_then_load_round_trips");
    let original = pages_of(&[(0x1000, vec![b'a'; P]), (0x8608800, vec![b'b'; P])]);
    let loaded = Snapshot::load(&original.save("test", Some(tmp.path())).unwrap()).unwrap();
    assert_eq!(loaded.pages, original.pages);
    assert_eq!(loaded.nbytes(), 2 * PAGE);
}

#[test]
fn test_load_rejects_a_truncated_data_file() {
    let tmp = TmpPath::new("test_load_rejects_a_truncated_data_file");
    let idx = snap(&[(0x1000, b"x")])
        .save("test", Some(tmp.path()))
        .unwrap();
    std::fs::write(idx.with_extension("dat"), b"too short").unwrap();
    let error = Snapshot::load(&idx).unwrap_err();
    assert!(
        error.is(Kind::ValueError) && error.message.contains("expected"),
        "{error:?}"
    );
}

#[test]
fn test_sweep_keeps_pages_with_data_and_drops_empty_ones() {
    let memory = HashMap::from([
        (0x1000, vec![1; P]),
        (0x1400, vec![0; P]),
        (0x1800, vec![2; P]),
    ]);
    let emu = FakeEmulator::new(memory);
    let rpc = rpc(&emu);
    let s = sweep(
        &rpc,
        &[(0x1000, 0x1C00)],
        false,
        CHUNK_PAGES,
        CHUNK_PAUSE,
        None,
    )
    .unwrap();
    let mut found: Vec<i64> = s.pages.keys().copied().collect();
    found.sort();
    assert_eq!(found, vec![0x1000, 0x1800]);
}

#[test]
fn test_sweep_can_keep_empty_pages_when_asked() {
    let emu = FakeEmulator::new(HashMap::from([(0x1000, vec![0; P])]));
    let rpc = rpc(&emu);
    let s = sweep(
        &rpc,
        &[(0x1000, 0x1400)],
        true,
        CHUNK_PAGES,
        CHUNK_PAUSE,
        None,
    )
    .unwrap();
    let mut found: Vec<i64> = s.pages.keys().copied().collect();
    found.sort();
    assert_eq!(found, vec![0x1000]);
}

/// A heap sweep held every other poll up for 300 ms (spike 60): polling carries on between its
/// chunks instead.
#[test]
fn test_sweep_hands_control_back_between_chunks() {
    let memory = HashMap::from([
        (0x1000, vec![1; P]),
        (0x1400, vec![2; P]),
        (0x1800, vec![3; P]),
    ]);
    let calls: RefCell<Vec<usize>> = RefCell::new(Vec::new());
    let emu = FakeEmulator::new(memory);
    let rpc = rpc(&emu);
    let between = || {
        let n = calls.borrow().len();
        calls.borrow_mut().push(n);
    };
    let s = sweep(&rpc, &[(0x1000, 0x1C00)], false, 1, 0.0, Some(&between)).unwrap();
    let mut found: Vec<i64> = s.pages.keys().copied().collect();
    found.sort();
    assert_eq!(found, vec![0x1000, 0x1400, 0x1800]);
    assert_eq!(calls.borrow().len(), 2);
}

#[test]
fn test_regions_hands_control_back_between_chunks() {
    let memory = HashMap::from([(0x1000, vec![1; P]), (0x3000, vec![2; P])]);
    let calls: RefCell<Vec<usize>> = RefCell::new(Vec::new());
    let emu = FakeEmulator::new(memory);
    let rpc = rpc(&emu);
    let between = || {
        let n = calls.borrow().len();
        calls.borrow_mut().push(n);
    };
    let found = regions(&rpc, (0x1000, 0x4000), 0x1000, 0, Some(1), Some(&between)).unwrap();
    assert_eq!(found, vec![(0x1000, 0x2000), (0x3000, 0x4000)]);
    assert_eq!(calls.borrow().len(), 2);
}

#[test]
fn test_churn_is_zero_when_memory_holds_still() {
    let emu = FakeEmulator::new(HashMap::from([(0x1000, vec![5; P])]));
    assert_eq!(churn(&rpc(&emu), &[0x1000], 0.0).unwrap(), 0);
}

#[test]
fn test_churn_counts_bytes_that_moved() {
    let emu = FakeEmulator::new(HashMap::from([(0x1000, vec![0; P])]));
    let rpc = rpc(&emu);
    let first = rpc.read_many(&[0x1000], PAGE).unwrap();
    let mut moved_page = vec![0xff; 3];
    moved_page.resize(P, 0);
    emu.set_page(0x1000, moved_page);
    let second = rpc.read_many(&[0x1000], PAGE).unwrap();
    let moved = first[&0x1000]
        .iter()
        .zip(&second[&0x1000])
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(moved, 3);
}
