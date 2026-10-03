//! GDB stub client tests, against a fake stub speaking the real protocol.

use std::collections::HashSet;

use indexmap::IndexMap;

use super::fake_gdb::{DEFAULT_REGISTERS, FakeGdbStub};
use crate::debug::gdb::{Gdb, Watch};
use crate::py::Kind;

fn no_memory() -> IndexMap<i64, Vec<u8>> {
    IndexMap::new()
}

#[test]
fn test_attach_reports_where_the_game_halted() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    let stop = gdb.attach().unwrap();
    assert_eq!(stop.pc, Some(DEFAULT_REGISTERS[15]));
    assert_eq!(stop.sp, Some(DEFAULT_REGISTERS[13]));
    assert_eq!(stop.lr, Some(DEFAULT_REGISTERS[14]));
    assert_eq!(stop.thread, Some(0x2F));
    assert!(!stub.running());
}

#[test]
fn test_registers_decode_the_stubs_little_endian_hex() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    let regs = gdb.registers().unwrap();
    assert_eq!(regs.r, DEFAULT_REGISTERS);
    assert_eq!(
        regs.args(),
        (
            DEFAULT_REGISTERS[0],
            DEFAULT_REGISTERS[1],
            DEFAULT_REGISTERS[2],
            DEFAULT_REGISTERS[3]
        )
    );
    assert_eq!(regs.pc(), DEFAULT_REGISTERS[15]);
    assert!(!regs.thumb());
}

#[test]
fn test_read_returns_the_bytes_at_that_address() {
    let text: Vec<u8> = "Would you like to save the game?"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let stub = FakeGdbStub::new(IndexMap::from([(0x08608810, text.clone())]));
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    assert_eq!(gdb.read(0x08608810, text.len() as i64).unwrap(), text);
}

/// The RPC answers zeros for anything unmapped; the stub answers nothing at all.
#[test]
fn test_read_of_unmapped_memory_is_an_error_not_zeros() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    let error = gdb.read(0x20000000, 16).unwrap_err();
    assert!(
        error.is(Kind::GdbError) && error.message.contains("nothing mapped"),
        "{error:?}"
    );
}

#[test]
fn test_read_refuses_sizes_the_stub_would_garble() {
    for size in [0, -1, 4097, 65536] {
        let stub = FakeGdbStub::new(no_memory());
        let mut gdb = Gdb::connect(stub.port()).unwrap();
        let error = gdb.read(0x08608810, size).unwrap_err();
        assert!(error.is(Kind::ValueError), "{size}: {error:?}");
    }
}

/// `?` is what tells the stub which process it is debugging. Without it, `E01`.
#[test]
fn test_a_watchpoint_before_attach_is_refused_by_the_stub() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    let error = gdb.watch_writes(0x08608810, 64).unwrap_err();
    assert!(
        error.is(Kind::GdbError) && error.message.contains("E01"),
        "{error:?}"
    );
    assert_eq!(stub.watchpoints(), HashSet::new());
}

#[test]
fn test_watch_writes_registers_the_range_with_the_stub() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.watch_writes(0x08608810, 64).unwrap();
    assert_eq!(
        stub.watchpoints(),
        HashSet::from([(Watch::Write.value(), 0x08608810, 64)])
    );
}

#[test]
fn test_watching_writes_takes_the_watchpoint_back_off() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.watching_writes(0x08608810, 64, |_| {
        assert!(!stub.watchpoints().is_empty());
        Ok(())
    })
    .unwrap();
    assert_eq!(stub.watchpoints(), HashSet::new());
}

/// A client that walks away while the game is halted leaves it halted.
#[test]
fn test_closing_removes_every_watchpoint_and_detaches() {
    let stub = FakeGdbStub::new(no_memory());
    {
        let mut gdb = Gdb::connect(stub.port()).unwrap();
        gdb.attach().unwrap();
        gdb.watch_writes(0x08608810, 64).unwrap();
        gdb.watch_reads(0x1585C2E0, 8).unwrap();
    }
    assert_eq!(stub.watchpoints(), HashSet::new());
    assert!(stub.detached());
    assert!(stub.running());
}

/// Azahar's `D` clears the process it would have continued, so `D` alone freezes the game.
#[test]
fn test_closing_resumes_before_it_detaches() {
    let stub = FakeGdbStub::new(no_memory());
    {
        let mut gdb = Gdb::connect(stub.port()).unwrap();
        gdb.attach().unwrap();
    }
    let commands = stub.commands();
    let position = |wanted: &str| commands.iter().position(|command| command == wanted);
    assert!(commands.iter().any(|command| command == "c"));
    assert!(position("c").unwrap() < position("D").unwrap());
}

#[test]
fn test_wait_returns_the_stop_the_game_produced() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.watch_writes(0x08608810, 64).unwrap();
    gdb.resume().unwrap();
    stub.wait_for("c");
    assert!(stub.running());
    stub.fire(5, 0x004C1234, 0x2F);
    let stop = gdb.wait(Some(2.0)).unwrap();
    assert_eq!(stop.signal, 5);
    assert_eq!(stop.pc, Some(0x004C1234));
}

#[test]
fn test_wait_gives_up_rather_than_hanging_when_the_game_never_stops() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.resume().unwrap();
    let error = gdb.wait(Some(0.2)).unwrap_err();
    assert!(error.is(Kind::GdbTimeoutError), "{error:?}");
}

#[test]
fn test_poll_is_empty_until_something_stops() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.resume().unwrap();
    assert_eq!(gdb.poll().unwrap(), None);
    stub.fire_at(0x004C1234);
    let stop = (0..100).find_map(|_| gdb.poll().unwrap());
    let stop = stop.expect("a stop within 100 polls");
    assert_eq!(stop.pc, Some(0x004C1234));
}

#[test]
fn test_interrupt_halts_a_running_game() {
    let stub = FakeGdbStub::new(no_memory());
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.resume().unwrap();
    let stop = gdb.interrupt(Some(2.0)).unwrap();
    assert_eq!(stop.signal, 5);
    assert!(!stub.running());
}

/// The game can halt at any moment, including while a read is in flight.
#[test]
fn test_a_stop_arriving_mid_command_is_not_mistaken_for_the_reply() {
    let text = b"ab".repeat(8);
    let stub = FakeGdbStub::new(IndexMap::from([(0x08608810, text.clone())]));
    let mut gdb = Gdb::connect(stub.port()).unwrap();
    gdb.attach().unwrap();
    gdb.resume().unwrap();
    stub.fire_at(0x004C1234);
    assert_eq!(gdb.read(0x08608810, text.len() as i64).unwrap(), text);
    let stop = gdb.wait(Some(2.0)).unwrap();
    assert_eq!(stop.pc, Some(0x004C1234));
}
