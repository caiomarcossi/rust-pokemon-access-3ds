//! A UDP server speaking Azahar's RPC protocol, so transport tests need no emulator.
//!
//! It implements the wire format from the emulator's `src/core/rpc/packet.h`: the same 16-byte
//! header, the same request payloads, the same 1024-byte cap, and the same habit of answering
//! oversized reads with an empty reply rather than an error.

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::core::rpc::{HEADER_SIZE, MAX_READ, PROTOCOL_VERSIONS, PacketType};

pub const NO_PROCESS: u32 = 0xFFFF_FFFF;

struct State {
    memory: HashMap<i64, Vec<u8>>,
    processes: Vec<(u32, u64, String)>,
    selected: u32,
    reads: i64,
    unselected_reads: i64,
    drop: i64,
    drop_every: i64,
    /// The protocol version served; a packet of any other is answered with nothing.
    version: u32,
}

/// Serves `memory` page by page. Addresses with no data read back as zeros.
pub struct FakeEmulator {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    port: u16,
}

pub fn sango() -> Vec<(u32, u64, String)> {
    vec![(11, 0x0004_0000_0011_C400, "sango-1".to_string())]
}

impl FakeEmulator {
    pub fn new(memory: HashMap<i64, Vec<u8>>) -> Self {
        FakeEmulator::with(memory, sango(), 0, 0)
    }

    /// `drop_first` replies go unsent, then every `drop_every`th: drops spread through a long
    /// run, not bunched at the start, are what a sweep of the whole address space actually
    /// meets, and the case the client got wrong.
    pub fn with(
        memory: HashMap<i64, Vec<u8>>,
        processes: Vec<(u32, u64, String)>,
        drop_first: i64,
        drop_every: i64,
    ) -> Self {
        FakeEmulator::with_version(
            memory,
            processes,
            drop_first,
            drop_every,
            PROTOCOL_VERSIONS[0],
        )
    }

    /// `with`, serving protocol `version`.
    pub fn with_version(
        memory: HashMap<i64, Vec<u8>>,
        processes: Vec<(u32, u64, String)>,
        drop_first: i64,
        drop_every: i64,
        version: u32,
    ) -> Self {
        let sock = UdpSocket::bind("127.0.0.1:0").expect("bind");
        sock.set_read_timeout(Some(Duration::from_millis(200)))
            .expect("timeout");
        let port = sock.local_addr().expect("addr").port();
        let state = Arc::new(Mutex::new(State {
            memory,
            processes,
            selected: NO_PROCESS,
            reads: 0,
            unselected_reads: 0,
            drop: drop_first,
            drop_every,
            version,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let (state, stop) = (state.clone(), stop.clone());
            std::thread::spawn(move || serve(&sock, &state, &stop))
        };
        FakeEmulator {
            state,
            stop,
            thread: Some(thread),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn reads(&self) -> i64 {
        self.state.lock().unwrap().reads
    }

    /// Reads answered with no process selected, which crash the real emulator.
    pub fn unselected_reads(&self) -> i64 {
        self.state.lock().unwrap().unselected_reads
    }

    /// `emu.memory[base] = data`: the game writing a page while a client holds the connection.
    pub fn set_page(&self, base: i64, data: Vec<u8>) {
        self.state.lock().unwrap().memory.insert(base, data);
    }

    /// `emu.memory.clear()`.
    pub fn clear_pages(&self) {
        self.state.lock().unwrap().memory.clear();
    }

    /// `del emu.memory[base]`.
    pub fn remove_page(&self, base: i64) {
        self.state.lock().unwrap().memory.remove(&base);
    }

    /// `emu.processes = ...`: a game started, or ended, while a client waits.
    pub fn set_processes(&self, processes: Vec<(u32, u64, String)>) {
        self.state.lock().unwrap().processes = processes;
    }

    /// The pid reads are served from, or `NO_PROCESS`.
    pub fn selected(&self) -> u32 {
        self.state.lock().unwrap().selected
    }

    /// Be the emulator after the game restarts itself: its RPC server is a new one, which
    /// drops the next `silent_reads` reads while it is down and then has no process selected.
    pub fn restart(&self, silent_reads: i64) {
        let mut state = self.state.lock().unwrap();
        state.drop = silent_reads;
        state.selected = NO_PROCESS;
    }
}

impl Drop for FakeEmulator {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn bytes_at(memory: &HashMap<i64, Vec<u8>>, address: i64, size: i64) -> Vec<u8> {
    (0..size)
        .map(|offset| {
            let a = address + offset;
            memory
                .get(&(a & !0x3FF))
                .map_or(0, |page| page[(a & 0x3FF) as usize])
        })
        .collect()
}

fn serve(sock: &UdpSocket, state: &Mutex<State>, stop: &AtomicBool) {
    let mut raw = vec![0u8; HEADER_SIZE + MAX_READ as usize];
    while !stop.load(Ordering::SeqCst) {
        let Ok((got, peer)) = sock.recv_from(&mut raw) else {
            continue;
        };
        let word = |at: usize| u32::from_le_bytes(raw[at..at + 4].try_into().unwrap());
        let (version, request_id, packet_type, size) =
            (word(0), word(4), word(8), word(12) as usize);
        let payload = &raw[HEADER_SIZE..(HEADER_SIZE + size).min(got)];
        let (arg1, arg2) = if payload.len() >= 8 {
            (
                u32::from_le_bytes(payload[..4].try_into().unwrap()),
                u32::from_le_bytes(payload[4..8].try_into().unwrap()),
            )
        } else {
            (0, 0)
        };
        let mut reply = Vec::new();
        {
            let mut s = state.lock().unwrap();
            if version != s.version {
            } else if packet_type == PacketType::ReadMemory as u32 {
                s.reads += 1;
                if s.drop > 0 {
                    s.drop -= 1;
                    continue;
                }
                if s.drop_every != 0 && s.reads % s.drop_every == 0 {
                    continue;
                }
                if s.selected == NO_PROCESS {
                    s.unselected_reads += 1;
                }
                if 0 < arg2 && arg2 as i64 <= MAX_READ {
                    reply = bytes_at(&s.memory, arg1 as i64, arg2 as i64);
                }
            } else if packet_type == PacketType::ProcessList as u32 {
                let entries: Vec<_> = s.processes.iter().take(arg2 as usize).collect();
                reply.extend_from_slice(&(entries.len() as u32).to_le_bytes());
                for (pid, tid, name) in entries {
                    reply.extend_from_slice(&pid.to_le_bytes());
                    reply.extend_from_slice(&tid.to_le_bytes());
                    let mut padded = name.as_bytes()[..name.len().min(8)].to_vec();
                    padded.resize(8, 0);
                    reply.extend_from_slice(&padded);
                }
            } else if packet_type == PacketType::SetGetProcess as u32 {
                if arg1 == 0 {
                    reply.extend_from_slice(&s.selected.to_le_bytes());
                } else {
                    s.selected = arg2;
                }
            }
        }
        let mut packet = Vec::with_capacity(HEADER_SIZE + reply.len());
        for w in [version, request_id, packet_type, reply.len() as u32] {
            packet.extend_from_slice(&w.to_le_bytes());
        }
        packet.extend_from_slice(&reply);
        let _ = sock.send_to(&packet, peer);
    }
}
