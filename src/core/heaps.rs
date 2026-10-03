//! Heaps: the blocks the game has just allocated, read from the heaps it allocates them from.
//!
//! A 3DS game allocates through the SDK's expanded heap, and every object a sweep finds was
//! allocated that way. Each heap's head keeps its blocks in use in a list, and a new block goes on
//! the list's end, or on its front when it is taken from the top of the heap (spike 110). So one
//! read of a heap's head says whether it has allocated since the last look, and a walk from where
//! the list stopped says what: the handful of new blocks, where a sweep reads the whole heap to
//! find them.
//!
//! ```text
//! head  +0x00 'EXPH'    +0x04 -> the previous sibling's head    +0x08 -> the next sibling's
//!       +0x0c -> the first child's head    +0x10 -> the last child's    +0x14 u16 children
//!       +0x18 start    +0x1c end    +0x2c -> the first block in use    +0x30 -> the last
//! block +0x00 u16 'UD' in use    +0x04 size    +0x08 -> previous    +0x0c -> next
//!       and the memory the game asked for from +0x10
//! ```
//!
//! Heaps link to their siblings and children, so one found leads to every other. A heap made since
//! the last look is in its parent's list of children, and everything it already holds is new; a
//! heap torn down and made again where it was between two looks leaves its parent's count and last
//! child as they were, so a heap whose blocks have changed has its children listed again for a few
//! looks after.
//!
//! A block freed and taken again where it was looks exactly as it did, and the game does that all
//! the time: the save's layouts were built into the very blocks the X menu's had just given up,
//! and the walk stopped at the first of them as a block it knew (movie4, spike 110). So the game's
//! own count of blocks in use is read with the head, and whenever a heap changes, the few blocks at
//! either end of its list are looked at too, and handed over where their memory no longer reads as
//! it did: a new object's pointers lead elsewhere, and a block nothing has touched reads the same.
//! Handing every end over whatever it held swept a third of the heap again.
//!
//! Nothing here knows the game: where in its heap object a game keeps the SDK's head is the
//! profile's, and so is which memory may be read. Sun's large blocks may be the GPU's, and reading
//! one takes the emulator down (ADR-0035), so a game can have its large blocks told apart by their
//! headers alone, keep a heap's own blocks unwalked while watching the heaps inside it, and watch
//! only the heaps under the ones it adopts rather than their siblings too (spike 123).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::discovery::within;
use crate::core::memory::Memory;
use crate::py::{self, Error, Kind, Result, time};

/// 'EXPH', where the head starts.
pub const SIGNATURE: i64 = 0x45585048;
pub const LINK_PREV: i64 = 0x04;
pub const LINK_NEXT: i64 = 0x08;
pub const CHILD_FIRST: i64 = 0x0C;
pub const CHILD_LAST: i64 = 0x10;
pub const CHILD_COUNT: i64 = 0x14;
pub const START: i64 = 0x18;
pub const END: i64 = 0x1C;
pub const USED_FIRST: i64 = 0x2C;
pub const USED_LAST: i64 = 0x30;
pub const HEAD_SIZE: i64 = 0x34;

/// A block's header, before the memory the game asked for.
pub const BLOCK: i64 = 0x10;
/// 'UD', a block in use.
pub const IN_USE: i64 = 0x5544;
/// Blocks one walk goes through at most: the busiest heap in the application heap held 2,600 over
/// movie5 (spike 110), and a list read while the game rewrites it can lead anywhere.
pub const MAX_WALK: i64 = 8_192;
/// Looks after a heap's blocks change to keep listing its children: a child heap is built a moment
/// after its block is taken, up to 0.6 s over movie4 (spike 110).
pub const RECHECK_LOOKS: i64 = 5;
/// Blocks at either end of a list looked at whenever the heap changes: where a block taken again at
/// the address of one just freed sits, looking like the block it replaced.
pub const ENDS: i64 = 16;
/// How much of a block's memory is remembered to tell a block taken again for a new object from the
/// block that was there. A layout's work built again for another layout differs 0xbc into it, at
/// the resource it was built from, and not before: the battle's slot the level up's panel was built
/// into read the same for its first 0x40 bytes (movie2, 2026-09-26).
pub const MARK: i64 = 0x100;
/// Block headers a look reads before calling back, so polling goes on, and marks it reads in one
/// burst: Sun's area loads allocate thousands of blocks at once, read a header at a time, and a
/// look took 0.75 s (sun_1.ctm, spike 123).
pub const BETWEEN_READS: i64 = 256;
/// The longest a look reads before calling back, however few headers that is: as an area loaded,
/// the emulator answered a read in 5 ms, and 256 headers held polling up 1.3 s (sun_1.ctm, spike
/// 60, 2026-09-27).
pub const BETWEEN_SECONDS: f64 = 0.05;
/// Blocks remembered for one heap before they are read afresh from its list: every block seen is
/// remembered, freed or not, and a busy heap allocates a few hundred a second.
pub const KNOWN_LIMIT: i64 = 60_000;

/// A block a heap has just allocated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    /// The heap object it came from.
    pub heap: i64,
    /// Its header.
    pub address: i64,
    /// The memory the game asked for, which follows the header.
    pub size: i64,
}

impl Block {
    /// Where the game's memory in it starts and ends.
    pub fn memory(&self) -> (i64, i64) {
        let start = self.address + BLOCK;
        (start, start + self.size)
    }
}

/// A heap watched. Python hands the one object to the dict and to the loop over a copy of its
/// items, so each is shared here (`Heaps::heaps`).
struct Heap {
    head: i64,
    // Nothing reads these two, in Python either.
    #[allow(dead_code)]
    start: i64,
    #[allow(dead_code)]
    end: i64,
    first: i64,
    last: i64,
    children: (i64, i64),
    /// Every block seen in its list, freed or not, with its size: where a walk stops.
    known: IndexMap<i64, i64>,
    /// The game's count of blocks in use, where it keeps one, as of the last look.
    count: i64,
    /// Block -> the first `MARK` bytes of its memory when last looked at, for the blocks at the
    /// ends of the list.
    marks: IndexMap<i64, Vec<u8>>,
    recheck: i64,
    /// Whether its own blocks go unwalked, and only the heaps inside it are watched.
    unread: bool,
}

/// `read_each`: reads many (address, size) in one pipelined burst, keyed by address and leaving
/// out any that failed.
pub type ReadEach = Rc<dyn Fn(&[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>>>;

/// Watches every heap linked to the ones adopted, and says what each has just allocated.
pub struct Heaps {
    memory: Rc<dyn Memory>,
    read_each: Option<ReadEach>,
    head: i64,
    count: Option<i64>,
    span: (i64, i64),
    largest: Option<i64>,
    unread: Rc<dyn Fn(i64) -> bool>,
    heaps: IndexMap<i64, Rc<RefCell<Heap>>>,
    /// The heaps whose siblings have been adopted.
    siblings: HashSet<i64>,
    between: Option<Rc<dyn Fn()>>,
    reads: i64,
    called: f64,
}

/// `dict(pairs)`: a later pair with the same key replaces the value and keeps the place.
fn dict(pairs: &[(i64, i64)]) -> IndexMap<i64, i64> {
    pairs.iter().copied().collect()
}

impl Heaps {
    /// Python's defaults are no `count`, no `largest`, an `unread` that says no to every heap and
    /// no `read_each`.
    pub fn new(memory: Rc<dyn Memory>, head: i64, span: (i64, i64)) -> Self {
        Heaps::with(memory, head, span, None, None, Rc::new(|_heap| false), None)
    }

    /// `head` is where the SDK's head is in the game's heap object, and `count` where the game
    /// keeps its count of blocks in use, if it does. Only heaps whose objects lie in `span` are
    /// watched: the ones whose memory a sweep would read. A block larger than `largest` is never
    /// read, only its header, and told from the block that was there by its size; a heap object
    /// `unread` says yes to as it is adopted has its own blocks left unwalked, and the heaps
    /// inside it watched. `read_each` reads many (address, size) in one pipelined burst, keyed by
    /// address and leaving out any that failed: the marks of the blocks an area load allocates,
    /// thousands at once, which read one at a time held polling up (issue 4).
    pub fn with(
        memory: Rc<dyn Memory>,
        head: i64,
        span: (i64, i64),
        count: Option<i64>,
        largest: Option<i64>,
        unread: Rc<dyn Fn(i64) -> bool>,
        read_each: Option<ReadEach>,
    ) -> Self {
        Heaps {
            memory,
            read_each,
            head,
            count,
            span,
            largest,
            unread,
            heaps: IndexMap::new(),
            siblings: HashSet::new(),
            between: None,
            reads: 0,
            called: 0.0,
        }
    }

    /// `len(heaps)`.
    pub fn len(&self) -> usize {
        self.heaps.len()
    }

    /// `bool(heaps)`, which Python answers from `__len__`.
    pub fn is_empty(&self) -> bool {
        self.heaps.is_empty()
    }

    /// `heap in heaps`.
    pub fn contains(&self, heap: i64) -> bool {
        self.heaps.contains_key(&heap)
    }

    /// Watch these heap objects and every heap linked to them, or, `under`, only the heaps
    /// inside them and not their siblings. What a heap holds when it is adopted is taken as
    /// known, not new.
    pub fn adopt(&mut self, heaps: &[i64], under: bool) -> Result<Vec<Block>> {
        self._adopt(heaps, false, under)
    }

    /// Forget these blocks, by their headers: gone, another block taken over them, so that one
    /// taken again where it was, as it was, is handed over as new. Sun's save prompt built its
    /// cursor again into the very block it had had, and a watch that remembered the block never
    /// said so (sun_3.ctm, spike 123).
    pub fn forget(&mut self, blocks: &[i64]) {
        let gone: HashSet<i64> = blocks.iter().copied().collect();
        for heap in self.heaps.values() {
            let mut heap = heap.borrow_mut();
            let held: Vec<i64> = heap
                .known
                .keys()
                .copied()
                .filter(|block| gone.contains(block))
                .collect();
            for block in held {
                heap.known.shift_remove(&block);
                heap.marks.shift_remove(&block);
            }
        }
    }

    /// The blocks the watched heaps have allocated since the last look, with the few at either
    /// end of the list of any heap that changed. A heap whose head no longer reads as one is
    /// forgotten. `between` is called every `BETWEEN_READS` block headers read or marks burst,
    /// and whenever `BETWEEN_SECONDS` have passed without a call.
    pub fn poll(&mut self, between: Option<Rc<dyn Fn()>>) -> Result<Vec<Block>> {
        (self.between, self.reads, self.called) = (between, 0, time::monotonic());
        // Python's `finally`: the callback is let go whether the look returns, raises or panics.
        let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self._poll()));
        self.between = None;
        match polled {
            Ok(polled) => polled,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    // Python's `_poll` and `_adopt` sit beside a public `poll` and `adopt`, so they keep their
    // underscore.
    fn _poll(&mut self) -> Result<Vec<Block>> {
        let mut out: Vec<Block> = Vec::new();
        let mut parents: Vec<i64> = Vec::new();
        let items: Vec<(i64, Rc<RefCell<Heap>>)> = self
            .heaps
            .iter()
            .map(|(&at, heap)| (at, heap.clone()))
            .collect();
        for (at, heap) in items {
            let raw = match self.memory.read(at, self.head + HEAD_SIZE) {
                Ok(raw) => raw,
                Err(error) if error.is(Kind::RpcError) => {
                    vec![0; (self.head + HEAD_SIZE) as usize]
                }
                Err(error) => return Err(error),
            };
            let head = py::slice_from(&raw, self.head);
            if u32(head, 0)? != SIGNATURE {
                self.heaps.shift_remove(&at);
                self.siblings.remove(&at);
                continue;
            }
            let mut heap = heap.borrow_mut();
            let children = (u32(head, CHILD_LAST)?, u16(head, CHILD_COUNT)?);
            if children != heap.children || heap.recheck != 0 {
                heap.children = children;
                heap.recheck = (heap.recheck - 1).max(0);
                parents.push(at);
            }
            if heap.unread {
                continue;
            }
            let (first, last) = (u32(head, USED_FIRST)?, u32(head, USED_LAST)?);
            let count = match self.count {
                Some(offset) => u32(&raw, offset)?,
                None => -1,
            };
            let recounted = count != heap.count;
            let grew = if self.count.is_some() && heap.count >= 0 {
                count - heap.count
            } else {
                0
            };
            heap.count = count;
            let mut new: Vec<(i64, i64)> = Vec::new();
            let mut changed = recounted;
            if first != heap.first {
                heap.first = first;
                heap.recheck = RECHECK_LOOKS;
                if first != 0 && !heap.known.contains_key(&first) {
                    new.extend(self.walk(first, &heap.known, true, None, MAX_WALK)?);
                }
                changed = true;
            }
            if last != heap.last {
                heap.recheck = RECHECK_LOOKS;
                let appended = self.appended(&heap, last)?;
                if let Some(appended) = appended {
                    new.extend(appended);
                } else if !heap.known.contains_key(&last) {
                    let mut walked = self.walk(last, &heap.known, false, None, MAX_WALK)?;
                    walked.reverse();
                    new.extend(walked);
                }
                heap.last = last;
                changed = true;
            }
            let walked = dict(&new).len() as i64;
            if changed {
                let mut ends = self.walk(first, &IndexMap::new(), true, None, ENDS)?;
                ends.extend(self.walk(last, &IndexMap::new(), false, None, ENDS)?);
                new.extend(self.changed(&mut heap, &ends)?);
            }
            if grew > walked {
                // More blocks in use than were found new: blocks taken again where others were
                // freed, beyond the ends, stopped the walk at one it knew. The actors' heap took
                // 86 so as Route 1's second zone loaded, and none was handed over (sun_4.ctm,
                // spike 123). The whole list is looked over.
                let taken = dict(&new);
                let listed = self.walk(first, &IndexMap::new(), true, None, MAX_WALK)?;
                let rest: Vec<(i64, i64)> = listed
                    .into_iter()
                    .filter(|(block, _size)| !taken.contains_key(block))
                    .collect();
                new.extend(self.changed(&mut heap, &rest)?);
            }
            let fresh = dict(&new);
            heap.known
                .extend(fresh.iter().map(|(&block, &size)| (block, size)));
            let unmarked: Vec<(i64, i64)> = fresh
                .iter()
                .filter(|(block, _size)| !heap.marks.contains_key(*block))
                .map(|(&block, &size)| (block, size))
                .collect();
            self.changed(&mut heap, &unmarked)?;
            out.extend(fresh.iter().map(|(&block, &size)| Block {
                heap: at,
                address: block,
                size,
            }));
            if heap.known.len() as i64 > KNOWN_LIMIT {
                let listed = self.walk(heap.last, &IndexMap::new(), false, None, MAX_WALK)?;
                heap.known = dict(&listed);
                heap.marks.clear();
            }
        }
        for at in parents {
            if self.heaps.contains_key(&at) {
                out.extend(self.new_children(at)?);
            }
        }
        Ok(out)
    }

    /// Those of these blocks whose memory is not what it was when last looked at, or that were
    /// never looked at: a block taken again where one was freed.
    fn changed(&mut self, heap: &mut Heap, blocks: &[(i64, i64)]) -> Result<Vec<(i64, i64)>> {
        let mut marks: IndexMap<i64, Vec<u8>> = IndexMap::new();
        let mut reads: IndexMap<i64, (i64, i64)> = IndexMap::new();
        for &(block, size) in blocks {
            if self.largest.is_some_and(|largest| size > largest) {
                let word = u32::try_from(size)
                    .map_err(|_| Error::new(Kind::OverflowError, "int too big to convert"))?;
                marks.insert(block, word.to_le_bytes().to_vec());
            } else {
                reads.insert(block, (block + BLOCK, size.min(MARK)));
            }
        }
        if !reads.is_empty()
            && let Some(read_each) = self.read_each.clone()
        {
            // A burst at a time, polling between: the marks of an area load's 3,000 blocks read
            // in one burst held polling up 150 ms, twice over sun_1.ctm (spike 60, 2026-09-27).
            let wanted: Vec<(i64, i64)> = reads.values().copied().collect();
            let mut replies: IndexMap<i64, Vec<u8>> = IndexMap::new();
            for at in (0..wanted.len() as i64).step_by(BETWEEN_READS as usize) {
                if at != 0 {
                    self.call_back();
                }
                replies.extend(read_each(py::slice(&wanted, at, at + BETWEEN_READS))?);
            }
            for (&block, &(at, _size)) in &reads {
                if let Some(data) = replies.get(&at) {
                    marks.insert(block, data.clone());
                }
            }
        } else {
            for (&block, &(at, size)) in &reads {
                match self.memory.read(at, size) {
                    Ok(data) => {
                        marks.insert(block, data);
                    }
                    Err(error) if error.is(Kind::RpcError) => continue,
                    Err(error) => return Err(error),
                }
            }
        }
        let mut out: Vec<(i64, i64)> = Vec::new();
        for &(block, size) in blocks {
            if let Some(mark) = marks.get(&block)
                && heap.marks.get(&block) != Some(mark)
            {
                heap.marks.insert(block, mark.clone());
                out.push((block, size));
            }
        }
        Ok(out)
    }

    fn call_back(&mut self) {
        if let Some(between) = self.between.clone() {
            between();
            self.called = time::monotonic();
        }
    }

    /// The blocks put on a heap's list after its old last block, oldest first, walked to from
    /// it; or None where that cannot be done: the old last has been freed, or the list shrank.
    fn appended(&mut self, heap: &Heap, last: i64) -> Result<Option<Vec<(i64, i64)>>> {
        if heap.last == 0 {
            return Ok(None);
        }
        let header = match self.memory.read(heap.last, BLOCK) {
            Ok(header) => header,
            Err(error) if error.is(Kind::RpcError) => return Ok(None),
            Err(error) => return Err(error),
        };
        if u16(&header, 0)? != IN_USE {
            return Ok(None);
        }
        let ahead = u32(&header, 0x0C)?;
        if ahead == 0 {
            return Ok(None);
        }
        let walked = self.walk(ahead, &IndexMap::new(), true, Some(last), MAX_WALK)?;
        Ok(
            if walked.last().is_some_and(|&(block, _size)| block == last) {
                Some(walked)
            } else {
                None
            },
        )
    }

    /// (block, size) from `start` along a used list, up to and including `until`, or up to the
    /// first block in `known`, or to the list's end, and no more than `most` of them.
    fn walk(
        &mut self,
        start: i64,
        known: &IndexMap<i64, i64>,
        forward: bool,
        until: Option<i64>,
        most: i64,
    ) -> Result<Vec<(i64, i64)>> {
        let mut out: Vec<(i64, i64)> = Vec::new();
        let mut passed: HashSet<i64> = HashSet::new();
        let mut block = start;
        while block != 0
            && !known.contains_key(&block)
            && !passed.contains(&block)
            && (out.len() as i64) < most
        {
            let header = match self.memory.read(block, BLOCK) {
                Ok(header) => header,
                Err(error) if error.is(Kind::RpcError) => break,
                Err(error) => return Err(error),
            };
            self.reads += 1;
            if self.reads % BETWEEN_READS == 0 || time::monotonic() - self.called >= BETWEEN_SECONDS
            {
                self.call_back();
            }
            if u16(&header, 0)? != IN_USE {
                break;
            }
            out.push((block, u32(&header, 4)?));
            passed.insert(block);
            if Some(block) == until {
                break;
            }
            block = u32(&header, if forward { 0x0C } else { 0x08 })?;
        }
        Ok(out)
    }

    /// Adopt the children of a heap not yet watched: heaps made since the last look, all they
    /// hold new.
    fn new_children(&mut self, at: i64) -> Result<Vec<Block>> {
        let at_head = self.heaps[&at].borrow().head;
        let head = self.memory.read(at_head, HEAD_SIZE)?;
        let mut child = u32(&head, CHILD_FIRST)?;
        let mut fresh: Vec<i64> = Vec::new();
        for _ in 0..u16(&head, CHILD_COUNT)? {
            if child == 0 {
                break;
            }
            if !self.heaps.contains_key(&(child - self.head)) {
                fresh.push(child - self.head);
            }
            child = u32(&self.memory.read(child + LINK_NEXT, 4)?, 0)?;
        }
        if fresh.is_empty() {
            Ok(Vec::new())
        } else {
            self._adopt(&fresh, true, false)
        }
    }

    /// Watch these heaps and those linked to them. A `root`'s siblings are left alone, and it
    /// leads only to the heaps inside it; a heap met as a root and then as another's child has
    /// its siblings adopted then: Sun's heap 0x0b is both (spike 113).
    fn _adopt(&mut self, heaps: &[i64], fresh: bool, root: bool) -> Result<Vec<Block>> {
        let mut out: Vec<Block> = Vec::new();
        let mut todo: Vec<(i64, bool, bool)> = heaps.iter().map(|&at| (at, fresh, root)).collect();
        while let Some((at, new, alone)) = todo.pop() {
            let known = self.heaps.contains_key(&at);
            if (known && (alone || self.siblings.contains(&at))) || !within(self.span, at) {
                continue;
            }
            let head = match self.memory.read(at + self.head, HEAD_SIZE) {
                Ok(head) => head,
                Err(error) if error.is(Kind::RpcError) => continue,
                Err(error) => return Err(error),
            };
            if u32(&head, 0)? != SIGNATURE {
                continue;
            }
            let mut links: Vec<(i64, bool)> = Vec::new();
            if !known {
                let unread = (self.unread)(at);
                let last = u32(&head, USED_LAST)?;
                let held: Vec<(i64, i64)> = if unread {
                    Vec::new()
                } else {
                    let mut listed = self.walk(last, &IndexMap::new(), false, None, MAX_WALK)?;
                    listed.reverse();
                    listed
                };
                let mut count = -1;
                if let Some(offset) = self.count
                    && !unread
                {
                    count = u32(&self.memory.read(at + offset, 4)?, 0)?;
                }
                self.heaps.insert(
                    at,
                    Rc::new(RefCell::new(Heap {
                        head: at + self.head,
                        start: u32(&head, START)?,
                        end: u32(&head, END)?,
                        first: u32(&head, USED_FIRST)?,
                        last,
                        children: (u32(&head, CHILD_LAST)?, u16(&head, CHILD_COUNT)?),
                        known: dict(&held),
                        count,
                        marks: IndexMap::new(),
                        recheck: 0,
                        unread,
                    })),
                );
                if new {
                    out.extend(held.iter().map(|&(block, size)| Block {
                        heap: at,
                        address: block,
                        size,
                    }));
                }
                links.push((CHILD_FIRST, new));
                links.push((CHILD_LAST, new));
            }
            if !alone {
                self.siblings.insert(at);
                // A new heap's siblings are its parent's other children, as new as it is: taken
                // as known, a second heap made between two looks had none of its blocks handed
                // over (sun_4.ctm, spike 123).
                links.push((LINK_PREV, new));
                links.push((LINK_NEXT, new));
            }
            for (link, inherits) in links {
                let target = u32(&head, link)?;
                if target != 0 {
                    todo.push((target - self.head, inherits, false));
                }
            }
        }
        Ok(out)
    }
}
