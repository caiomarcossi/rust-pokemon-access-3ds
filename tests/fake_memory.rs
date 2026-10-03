//! Writable memory for reader tests, with C++ classes laid out the way the game's RTTI is.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::py::Result;

pub const CODE: (i64, i64) = (0x00100000, 0x00200000);
pub const PAGE: i64 = 0x1000;

/// Sparse bytes by address. Unwritten memory reads as zeros, as the emulator's does.
///
/// Written through `&self`, so a test can change the game's memory while a reader holds it.
#[derive(Default)]
pub struct FakeMemory {
    pages: RefCell<IndexMap<i64, Vec<u8>>>,
}

impl FakeMemory {
    pub fn new() -> Rc<Self> {
        Rc::new(FakeMemory::default())
    }

    /// Everything written, as a sweep would find it, in the order the pages were first written,
    /// as the Python fake's dict keeps them.
    pub fn pages(&self) -> IndexMap<i64, Vec<u8>> {
        self.pages.borrow().clone()
    }

    pub fn write(&self, address: i64, data: &[u8]) {
        let mut pages = self.pages.borrow_mut();
        for (offset, &byte) in data.iter().enumerate() {
            let at = address + offset as i64;
            let page = pages
                .entry(at & !(PAGE - 1))
                .or_insert_with(|| vec![0; PAGE as usize]);
            page[(at & (PAGE - 1)) as usize] = byte;
        }
    }

    pub fn u32(&self, address: i64, values: &[i64]) {
        let data: Vec<u8> = values
            .iter()
            .flat_map(|&v| (v as u32).to_le_bytes())
            .collect();
        self.write(address, &data);
    }

    pub fn u16(&self, address: i64, value: i64) {
        self.write(address, &(value as u16).to_le_bytes());
    }
}

impl Memory for FakeMemory {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let pages = self.pages.borrow();
        Ok((address..address + size)
            .map(|at| {
                pages
                    .get(&(at & !(PAGE - 1)))
                    .map_or(0, |page| page[(at & (PAGE - 1)) as usize])
            })
            .collect())
    }
}

/// Memory that counts the reads made of it.
pub struct Counting {
    pub memory: Rc<dyn Memory>,
    pub reads: Cell<i64>,
}

impl Counting {
    pub fn new(memory: Rc<dyn Memory>) -> Rc<Self> {
        Rc::new(Counting {
            memory,
            reads: Cell::new(0),
        })
    }
}

impl Memory for Counting {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        self.memory.read(address, size)
    }
}

/// Writes a name, a typeinfo and a vtable into the fake executable for each class added.
pub struct FakeClasses {
    memory: Rc<FakeMemory>,
    next: Cell<i64>,
}

impl FakeClasses {
    pub fn new(memory: Rc<FakeMemory>) -> Self {
        FakeClasses::starting(memory, CODE.0 + 0x1000)
    }

    /// `start` is where the first class goes: past the executable, for a DLL's.
    pub fn starting(memory: Rc<FakeMemory>, start: i64) -> Self {
        FakeClasses {
            memory,
            next: Cell::new(start),
        }
    }

    pub fn add(&self, mangled: &str) -> i64 {
        let name = self.take(mangled.len() as i64 + 1);
        let mut text = mangled.as_bytes().to_vec();
        text.push(0);
        self.memory.write(name, &text);
        let typeinfo = self.take(8);
        self.memory.u32(typeinfo, &[CODE.0, name]);
        let vtable = self.take(12) + 8;
        self.memory.u32(vtable - 8, &[0, typeinfo, CODE.0 + 0x10]);
        vtable
    }

    fn take(&self, size: i64) -> i64 {
        let at = self.next.get();
        self.next.set((at + size + 3) & !3);
        at
    }
}
