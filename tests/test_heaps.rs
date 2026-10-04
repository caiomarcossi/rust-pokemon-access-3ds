//! Heap tests: the blocks a heap has just allocated, found from its head (spike 110).

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::heaps::{self as heaps, Block, Heaps, ReadEach};
use crate::core::memory::Memory;
use crate::py::Result;

/// Where the SDK's head sits in the game's heap object, as in ORAS's.
const HEAD: i64 = 0x38;
const SPAN: (i64, i64) = (0x08000000, 0x09000000);
const ROOT: i64 = 0x08001000;
const CHILD: i64 = 0x08100000;

/// A heap object at `at` whose used list holds these (block, size), first to last, linked to
/// the heaps before and after it and to its children.
fn lay_out(
    memory: &FakeMemory,
    at: i64,
    blocks: &[(i64, i64)],
    prev: i64,
    after: i64,
    children: &[i64],
) {
    let head = at + HEAD;
    let link = |heap: i64| if heap != 0 { heap + HEAD } else { 0 };
    memory.u32(head, &[heaps::SIGNATURE, link(prev), link(after)]);
    memory.u32(
        head + heaps::CHILD_FIRST,
        &[
            children.first().map_or(0, |child| child + HEAD),
            children.last().map_or(0, |child| child + HEAD),
        ],
    );
    memory.u16(head + heaps::CHILD_COUNT, children.len() as i64);
    memory.u32(head + heaps::START, &[at + 0x100, at + 0x100000]);
    for (index, &(block, size)) in blocks.iter().enumerate() {
        let before = if index != 0 { blocks[index - 1].0 } else { 0 };
        let ahead = if index + 1 < blocks.len() {
            blocks[index + 1].0
        } else {
            0
        };
        memory.u16(block, heaps::IN_USE);
        memory.u32(block + 4, &[size, before, ahead]);
    }
    memory.u32(
        head + heaps::USED_FIRST,
        &[
            blocks.first().map_or(0, |block| block.0),
            blocks.last().map_or(0, |block| block.0),
        ],
    );
}

/// `lay_out` of a heap with no neighbours and no children.
fn alone(memory: &FakeMemory, at: i64, blocks: &[(i64, i64)]) {
    lay_out(memory, at, blocks, 0, 0, &[]);
}

fn block(heap: i64, index: i64) -> (i64, i64) {
    (heap + 0x100 + index * 0x100, 0x40)
}

fn at(heap: i64, (address, size): (i64, i64)) -> Block {
    Block {
        heap,
        address,
        size,
    }
}

/// `expected <= set(found)`.
fn assert_all_in(expected: &[Block], found: &[Block]) {
    let found: HashSet<Block> = found.iter().copied().collect();
    for block in expected {
        assert!(found.contains(block), "{block:?} not in {found:?}");
    }
}

#[test]
fn test_what_a_heap_holds_when_adopted_is_known_not_new() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0), block(ROOT, 1)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    assert_eq!(watch.adopt(&[ROOT], false).unwrap(), []);
    assert_eq!(watch.poll(None).unwrap(), []);
}

#[test]
fn test_a_block_put_on_the_end_of_the_list_is_new_at_the_next_look() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    alone(
        &memory,
        ROOT,
        &[block(ROOT, 0), block(ROOT, 1), block(ROOT, 2)],
    );
    assert_all_in(
        &[at(ROOT, block(ROOT, 1)), at(ROOT, block(ROOT, 2))],
        &watch.poll(None).unwrap(),
    );
    assert_eq!(watch.poll(None).unwrap(), []);
}

/// The Pokémon record at the very end of its heap went unseen until the front of the list was
/// watched as well (movie5, spike 110).
#[test]
fn test_a_block_taken_from_the_top_of_the_heap_goes_on_the_front_and_is_new_too() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0), block(ROOT, 1)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    let top = (ROOT + 0xFFF00, 0x80);
    alone(&memory, ROOT, &[top, block(ROOT, 0), block(ROOT, 1)]);
    assert_all_in(&[at(ROOT, top)], &watch.poll(None).unwrap());
}

#[test]
fn test_once_the_old_last_block_is_freed_the_new_ones_are_walked_back_to_one_known() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0), block(ROOT, 1)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    memory.u16(block(ROOT, 1).0, 0x4652); // 'FR': freed
    alone(
        &memory,
        ROOT,
        &[block(ROOT, 0), block(ROOT, 2), block(ROOT, 3)],
    );
    memory.u16(block(ROOT, 1).0, 0x4652);
    let new = watch.poll(None).unwrap();
    assert_all_in(&[at(ROOT, block(ROOT, 2)), at(ROOT, block(ROOT, 3))], &new);
    assert!(!new.contains(&at(ROOT, block(ROOT, 1))));
}

#[test]
fn test_a_block_freed_from_the_end_hands_over_only_what_is_left_at_the_ends() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0), block(ROOT, 1)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    alone(&memory, ROOT, &[block(ROOT, 0)]);
    memory.u16(block(ROOT, 1).0, 0x4652);
    assert_eq!(watch.poll(None).unwrap(), [at(ROOT, block(ROOT, 0))]);
}

/// It looks exactly like the block it replaced: the save's layouts were built into the blocks
/// the X menu's had just given up (movie4, spike 110). The game's count of blocks in use moved.
#[test]
fn test_a_block_taken_again_where_one_was_freed_is_handed_over_as_the_count_changes() {
    let memory = FakeMemory::new();
    let count = 0x08;
    alone(
        &memory,
        ROOT,
        &[block(ROOT, 0), block(ROOT, 1), block(ROOT, 2)],
    );
    memory.u32(ROOT + count, &[3]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        Some(count),
        None,
        Rc::new(|_heap| false),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    memory.u32(ROOT + count, &[2]);
    assert_all_in(&[at(ROOT, block(ROOT, 2))], &watch.poll(None).unwrap());
    assert_eq!(watch.poll(None).unwrap(), []);
}

#[test]
fn test_one_heap_leads_to_its_siblings_and_children() {
    let memory = FakeMemory::new();
    let sibling = 0x08200000;
    lay_out(&memory, ROOT, &[], 0, sibling, &[CHILD]);
    lay_out(&memory, sibling, &[], ROOT, 0, &[]);
    alone(&memory, CHILD, &[block(CHILD, 0)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    assert_eq!(watch.len(), 3);
    alone(&memory, CHILD, &[block(CHILD, 0), block(CHILD, 1)]);
    assert_all_in(&[at(CHILD, block(CHILD, 1))], &watch.poll(None).unwrap());
}

#[test]
fn test_a_heap_made_since_the_last_look_is_watched_and_all_it_holds_is_new() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    lay_out(
        &memory,
        ROOT,
        &[block(ROOT, 0), block(ROOT, 1)],
        0,
        0,
        &[CHILD],
    );
    alone(&memory, CHILD, &[block(CHILD, 0), block(CHILD, 1)]);
    assert_all_in(
        &[
            at(ROOT, block(ROOT, 1)),
            at(CHILD, block(CHILD, 0)),
            at(CHILD, block(CHILD, 1)),
        ],
        &watch.poll(None).unwrap(),
    );
    assert!(watch.contains(CHILD));
}

/// Its parent's count of children and its last child read as they were: the field's heaps,
/// rebuilt as the Bag closed, went unseen (movie5, spike 110).
#[test]
fn test_a_heap_torn_down_and_made_again_where_it_was_is_watched_again() {
    let memory = FakeMemory::new();
    lay_out(&memory, ROOT, &[block(ROOT, 0)], 0, 0, &[CHILD]);
    alone(&memory, CHILD, &[block(CHILD, 0)]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    memory.u32(CHILD + HEAD, &[0]); // torn down
    lay_out(
        &memory,
        ROOT,
        &[block(ROOT, 0), block(ROOT, 1)],
        0,
        0,
        &[CHILD],
    );
    assert_all_in(&[at(ROOT, block(ROOT, 1))], &watch.poll(None).unwrap());
    assert!(!watch.contains(CHILD));
    alone(&memory, CHILD, &[block(CHILD, 2)]); // made again
    assert_all_in(&[at(CHILD, block(CHILD, 2))], &watch.poll(None).unwrap());
}

/// The linear heap's trees hang off the application heap's root; nothing sweeps them.
#[test]
fn test_a_heap_outside_the_span_is_not_watched() {
    let memory = FakeMemory::new();
    let linear = 0x14000000;
    lay_out(&memory, ROOT, &[], 0, linear, &[]);
    lay_out(&memory, linear, &[block(linear, 0)], ROOT, 0, &[]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    assert_eq!(watch.len(), 1);
}

#[test]
fn test_a_blocks_memory_follows_its_header() {
    assert_eq!(
        Block {
            heap: ROOT,
            address: 0x08002000,
            size: 0x40
        }
        .memory(),
        (0x08002010, 0x08002050)
    );
}

/// Handed over whether or not it had changed, a busy heap's ends swept a sixth of the heap
/// again; a block taken again for a new object reads differently (spike 110).
#[test]
fn test_an_end_block_whose_memory_is_as_it_was_is_not_handed_over_again() {
    let memory = FakeMemory::new();
    let count = 0x08;
    alone(&memory, ROOT, &[block(ROOT, 0), block(ROOT, 1)]);
    memory.u32(ROOT + count, &[2]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        Some(count),
        None,
        Rc::new(|_heap| false),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    memory.u32(ROOT + count, &[3]);
    watch.poll(None).unwrap();
    memory.u32(ROOT + count, &[2]);
    assert_eq!(watch.poll(None).unwrap(), []);
    memory.u32(block(ROOT, 1).0 + heaps::BLOCK, &[0x12345678]);
    memory.u32(ROOT + count, &[3]);
    assert_eq!(watch.poll(None).unwrap(), [at(ROOT, block(ROOT, 1))]);
}

/// Memory that fails the test if anything reads inside these spans.
struct Guarded {
    inner: Rc<FakeMemory>,
    spans: Vec<(i64, i64)>,
}

impl Guarded {
    fn new(spans: &[(i64, i64)]) -> Rc<Self> {
        Rc::new(Guarded {
            inner: FakeMemory::new(),
            spans: spans.to_vec(),
        })
    }
}

impl Memory for Guarded {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        for &(low, high) in &self.spans {
            assert!(
                !(address < high && low < address + size),
                "read {address:#x}"
            );
        }
        self.inner.read(address, size)
    }
}

/// Sun's large blocks may be the GPU's, and reading one takes the emulator down (spike 123):
/// a large block taken where a small one was freed is new by its header alone.
#[test]
fn test_a_block_larger_than_the_largest_read_is_told_by_its_size_and_never_read() {
    let (small, large) = (block(ROOT, 0).0, (block(ROOT, 0).0, 0x20000));
    let memory = Guarded::new(&[(small + heaps::BLOCK, small + heaps::BLOCK + large.1)]);
    let count = 0x08;
    alone(&memory.inner, ROOT, &[large]);
    memory.inner.u32(ROOT + count, &[1]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        Some(count),
        Some(0x1000),
        Rc::new(|_heap| false),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    memory.inner.u32(ROOT + count, &[2]);
    assert_eq!(watch.poll(None).unwrap(), [at(ROOT, large)]);
    memory.inner.u32(ROOT + count, &[1]);
    assert_eq!(watch.poll(None).unwrap(), []);
    alone(&memory.inner, ROOT, &[(small, 0x40000)]);
    memory.inner.u32(ROOT + count, &[2]);
    assert_eq!(watch.poll(None).unwrap(), [at(ROOT, (small, 0x40000))]);
}

/// Sun's field heap holds the rendering engine's blocks and the heaps its actors are in.
#[test]
fn test_an_unread_heaps_own_blocks_are_not_walked_and_the_heaps_inside_it_are() {
    let own = block(ROOT, 0);
    let memory = Guarded::new(&[(own.0, own.0 + own.1)]);
    lay_out(&memory.inner, ROOT, &[], 0, 0, &[CHILD]);
    memory
        .inner
        .u32(ROOT + HEAD + heaps::USED_FIRST, &[own.0, own.0]);
    alone(&memory.inner, CHILD, &[block(CHILD, 0)]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        None,
        None,
        Rc::new(|heap| heap == ROOT),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    assert!(watch.contains(ROOT) && watch.contains(CHILD));
    alone(&memory.inner, CHILD, &[block(CHILD, 0), block(CHILD, 1)]);
    assert_all_in(&[at(CHILD, block(CHILD, 1))], &watch.poll(None).unwrap());
}

/// Sun's other top-level heaps are the GPU's and the sound's, not the dialogue's; a root
/// that is another root's child has its siblings watched as that one's (spike 113).
#[test]
fn test_a_heap_adopted_as_a_root_leads_to_the_heaps_inside_it_and_not_its_siblings() {
    let (beside, inner, second) = (0x08200000, 0x08300000, 0x08400000);
    let memory = FakeMemory::new();
    lay_out(&memory, ROOT, &[], 0, beside, &[CHILD]);
    lay_out(&memory, beside, &[], ROOT, 0, &[]);
    lay_out(&memory, CHILD, &[], 0, inner, &[second]);
    lay_out(&memory, inner, &[], CHILD, 0, &[]);
    alone(&memory, second, &[]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[CHILD, ROOT], true).unwrap();
    let watched: HashSet<i64> = [ROOT, beside, CHILD, inner, second]
        .into_iter()
        .filter(|&heap| watch.contains(heap))
        .collect();
    assert_eq!(watched, HashSet::from([ROOT, CHILD, inner, second]));
}

/// Taken as known through its sibling, the second heap an area made had none of its blocks
/// handed over: Sun's actors, as the field loaded (sun_4.ctm, spike 123).
#[test]
fn test_heaps_made_together_between_two_looks_are_all_new() {
    let second = 0x08200000;
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[]);
    let mut watch = Heaps::new(memory.clone(), HEAD, SPAN);
    watch.adopt(&[ROOT], false).unwrap();
    lay_out(&memory, ROOT, &[], 0, 0, &[CHILD, second]);
    lay_out(&memory, CHILD, &[block(CHILD, 0)], 0, second, &[]);
    lay_out(&memory, second, &[block(second, 0)], CHILD, 0, &[]);
    assert_all_in(
        &[at(CHILD, block(CHILD, 0)), at(second, block(second, 0))],
        &watch.poll(None).unwrap(),
    );
}

/// Blocks taken again where others were freed stopped the walk at one it knew, and 86 of
/// Sun's actors went unseen as an area's second zone loaded (sun_4.ctm, spike 123).
#[test]
fn test_a_count_grown_past_what_was_found_has_the_whole_list_looked_over() {
    let count = 0x08;
    let memory = FakeMemory::new();
    let blocks: Vec<(i64, i64)> = (0..40).map(|index| block(ROOT, index)).collect();
    alone(&memory, ROOT, &blocks);
    memory.u32(ROOT + count, &[40]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        Some(count),
        None,
        Rc::new(|_heap| false),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    alone(&memory, ROOT, &blocks[..10]);
    memory.u32(ROOT + count, &[10]);
    watch.poll(None).unwrap();
    // The last block freed, and the rest taken again where they were, built into anew.
    let mut again: Vec<(i64, i64)> = blocks[..9].to_vec();
    again.extend_from_slice(&blocks[10..]);
    alone(&memory, ROOT, &again);
    memory.u16(blocks[9].0, 0x4652);
    for &(address, _size) in &blocks[10..] {
        memory.u32(address + heaps::BLOCK, &[0xB0B]);
    }
    memory.u32(ROOT + count, &[39]);
    let expected: Vec<Block> = blocks[10..].iter().map(|&b| at(ROOT, b)).collect();
    assert_all_in(&expected, &watch.poll(None).unwrap());
}

/// Sun's save prompt built its cursor again into the very block it had had, and a watch that
/// remembered the block never said so (sun_3.ctm, spike 123).
#[test]
fn test_a_block_forgotten_and_taken_again_as_it_was_is_new() {
    let count = 0x08;
    let memory = FakeMemory::new();
    let blocks: Vec<(i64, i64)> = (0..3).map(|index| block(ROOT, index)).collect();
    alone(&memory, ROOT, &blocks);
    memory.u32(ROOT + count, &[3]);
    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        Some(count),
        None,
        Rc::new(|_heap| false),
        None,
    );
    watch.adopt(&[ROOT], false).unwrap();
    memory.u32(ROOT + count, &[4]);
    watch.poll(None).unwrap();
    alone(&memory, ROOT, &blocks[..2]);
    memory.u32(ROOT + count, &[2]);
    watch.poll(None).unwrap();
    // Taken over by another block meanwhile, then the old last block freed and the block taken
    // again as it was: remembered, it was handed over neither by a walk nor by the ends.
    watch.forget(&[blocks[2].0]);
    alone(&memory, ROOT, &[blocks[0], blocks[2]]);
    memory.u16(blocks[1].0, 0x4652);
    assert_all_in(&[at(ROOT, blocks[2])], &watch.poll(None).unwrap());
}

/// An area load's thousands of marks, read in one burst, held polling up (sun_1.ctm).
#[test]
fn test_the_marks_of_many_new_blocks_are_read_a_burst_at_a_time_polling_between() {
    let memory = FakeMemory::new();
    alone(&memory, ROOT, &[block(ROOT, 0)]);
    let bursts: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));

    let read_each: ReadEach = {
        let (bursts, memory) = (bursts.clone(), memory.clone());
        Rc::new(move |reads: &[(i64, i64)]| {
            bursts.borrow_mut().push(reads.len());
            let mut replies: IndexMap<i64, Vec<u8>> = IndexMap::new();
            for &(at, size) in reads {
                replies.insert(at, memory.read(at, size)?);
            }
            Ok(replies)
        })
    };

    let mut watch = Heaps::with(
        memory.clone(),
        HEAD,
        SPAN,
        None,
        None,
        Rc::new(|_heap| false),
        Some(read_each),
    );
    watch.adopt(&[ROOT], false).unwrap();
    let many: Vec<(i64, i64)> = (0..heaps::BETWEEN_READS + 20)
        .map(|index| block(ROOT, index))
        .collect();
    alone(&memory, ROOT, &many);
    let polled = Rc::new(Cell::new(0));
    let new = {
        let polled = polled.clone();
        watch
            .poll(Some(Rc::new(move || polled.set(polled.get() + 1))))
            .unwrap()
    };
    assert!(new.len() as i64 >= heaps::BETWEEN_READS + 19);
    assert!(*bursts.borrow().iter().max().unwrap() as i64 <= heaps::BETWEEN_READS);
    assert!(polled.get() > 0);
}
