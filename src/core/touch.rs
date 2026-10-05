//! Touching the bottom screen, as a cemuhook (DSU) server Azahar reads a touchpad from.
//!
//! pa3ds announces and never acts, with one exception: a tap on the touch target pa3ds has just
//! named, when the player presses the key that asks for it (ADR-0009, amended 2026-10-01). A
//! pointer on the screen would have to land on a spot of a window whose size, layout and full
//! screen are the player's; this is the touch screen's own pixels instead.
//!
//! Azahar reads its touch screen from the mouse and, while the mouse is not pressing, from a
//! second device, `controller_touch_device`, which can be the cemuhook UDP client it keeps from
//! Citra (`input_common/udp`). That client asks a server on this machine, port 26760 by default,
//! for its first pad every three seconds, and takes each pad state the server sends back: a
//! touchpad pressed or not, at an x and a y it scales by the setting's calibration. With the
//! calibration 0 to 320 and 0 to 240 (`DEVICE`), pa3ds's x and y are the touch screen's pixels,
//! from its top left. The protocol is a public one, which DS4Windows and others speak; nothing of
//! Azahar's is used here.
//!
//! A tap is a press held `HOLD` seconds and a release: the game reads the touch screen once a
//! frame, so a press is a run of frames, and it is the change from released to pressed that a
//! button takes (Amethyst's touch module for Black and White does the same through BizHawk).
//! Azahar keeps the last state it was sent, so a finger is never left down: every tap ends in a
//! release, and closing the server sends one.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use flate2::Crc;
use indexmap::IndexMap;

use crate::py::{Error, Kind, Result, time};

/// Azahar's `udp_input_port` as it ships.
pub const PORT: i64 = 26760;
pub const HOST: &str = "127.0.0.1";
/// Azahar's `controller_touch_device` for pa3ds's taps: the cemuhook client, calibrated so an x
/// and a y are the touch screen's own pixels (Azahar multiplies the share by 320 and 240).
pub const DEVICE: &str = "engine:cemuhookudp,max_x:320,max_y:240,min_x:0,min_y:0";
pub const WIDTH: i64 = 320;
pub const HEIGHT: i64 = 240;
/// How long a tap presses: six frames, a press a button takes as one.
pub const HOLD: f64 = 0.1;
/// How often the pad's state is sent while a client listens: twice a frame.
pub const TICK: f64 = 1.0 / 120.0;
/// How long a client is sent states after its last request; it asks every three seconds.
pub const LISTENING: f64 = 6.0;

pub const NOT_LISTENING: &str = "Azahar is not taking touches from pa3ds";
pub const CLIENT: &[u8; 4] = b"DSUC";
pub const SERVER: &[u8; 4] = b"DSUS";
pub const VERSION: i64 = 1001;
pub const VERSION_TYPE: i64 = 0x100000;
pub const PORT_INFO: i64 = 0x100001;
pub const PAD_DATA: i64 = 0x100002;
/// The one pad this server has, as a port's info: slot 0, connected, full gyro, USB, a MAC of its
/// own, battery full, active.
pub const PAD: [u8; 12] = [0, 2, 2, 1, 0, 0, 0, 0, 0, 1, 5, 1];

/// Where the server's thread logs: the thread is the server's own, so the log is shared with it.
pub type SharedLog = Arc<dyn Fn(&str) + Send + Sync>;

/// `struct.pack("<I", value)`.
#[track_caller]
fn pack_u32(value: i64) -> Result<[u8; 4]> {
    u32::try_from(value)
        .map(u32::to_le_bytes)
        .map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))
}

/// `struct.pack("<H", value)`.
#[track_caller]
fn pack_u16(value: i64) -> Result<[u8; 2]> {
    u16::try_from(value)
        .map(u16::to_le_bytes)
        .map_err(|_| Error::unpack("'H' format requires 0 <= number <= 65535"))
}

/// A server's packet: its header, the type and the payload, with the CRC-32 of all three.
pub fn message(kind: i64, payload: &[u8], server: i64) -> Result<Vec<u8>> {
    let mut body: Vec<u8> = pack_u32(kind)?.to_vec();
    body.extend_from_slice(payload);
    let mut head: Vec<u8> = SERVER.to_vec();
    head.extend_from_slice(&pack_u16(VERSION)?);
    head.extend_from_slice(&pack_u16(body.len() as i64)?);
    head.extend_from_slice(&pack_u32(0)?);
    head.extend_from_slice(&pack_u32(server)?);
    let mut crc = Crc::new();
    crc.update(&head);
    crc.update(&body);
    let mut out: Vec<u8> = head[..8].to_vec();
    out.extend_from_slice(&crc.sum().to_le_bytes());
    out.extend_from_slice(&head[12..]);
    out.extend_from_slice(&body);
    Ok(out)
}

/// The pad's state, 80 bytes: nothing held, the sticks centred, the first touch at `touch` or
/// none, and the pad lying still.
pub fn pad_state(counter: i64, touch: Option<(i64, i64)>) -> Result<Vec<u8>> {
    let (x, y) = touch.unwrap_or((0, 0));
    let mut out: Vec<u8> = PAD.to_vec();
    out.extend_from_slice(&pack_u32(counter)?);
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&[128, 128, 128, 128]);
    out.extend_from_slice(&[0; 12]);
    out.extend_from_slice(&[u8::from(touch.is_some()), 0]);
    out.extend_from_slice(&pack_u16(x)?);
    out.extend_from_slice(&pack_u16(y)?);
    out.extend_from_slice(&[0; 6]);
    out.extend_from_slice(&0u64.to_le_bytes());
    for value in [0.0f32, 0.0, 1.0, 0.0, 0.0, 0.0] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    Ok(out)
}

/// The type of a client's request, or None for anything that is not one.
pub fn request(packet: &[u8]) -> Option<i64> {
    if packet.len() < 20 || &packet[..4] != CLIENT {
        return None;
    }
    let word: [u8; 4] = packet[16..20].try_into().expect("four bytes");
    Some(i64::from(u32::from_le_bytes(word)))
}

/// What an `OSError` out of a socket says as `str(error)`: Windows' error and its text, the text
/// without its closing full stop.
fn os_error(error: &io::Error) -> String {
    let Some(code) = error.raw_os_error() else {
        return error.to_string();
    };
    let words = error.to_string();
    let text = words
        .strip_suffix(&format!(" (os error {code})"))
        .unwrap_or(&words);
    if cfg!(windows) {
        let text = text.trim_end_matches(|c: char| c == '.' || c.is_whitespace());
        format!("[WinError {code}] {text}")
    } else {
        format!("[Errno {code}] {text}")
    }
}

/// What `_lock` guards.
struct State {
    touch: Option<(i64, i64)>,
    until: f64,
    counter: i64,
    clients: IndexMap<SocketAddr, f64>,
}

/// What the server's thread and its caller's share.
struct Inner {
    log: SharedLog,
    id: i64,
    state: Mutex<State>,
    stop: AtomicBool,
    socket: UdpSocket,
}

/// The DSU server Azahar's touch device reads, on a thread of its own.
pub struct TouchServer {
    inner: Arc<Inner>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl TouchServer {
    /// A server on `port` (Python's default is `PORT`), or the `OSError` that says why not.
    ///
    /// `os.urandom(4)` is a `RandomState`'s key hashed, which std seeds from the operating
    /// system's random numbers.
    pub fn new(log: SharedLog, port: i64) -> Result<Self> {
        use std::hash::BuildHasher;
        let id = i64::from(std::collections::hash_map::RandomState::new().hash_one(0u8) as u32);
        let Ok(bound) = u16::try_from(port) else {
            return Err(Error::new(
                Kind::OverflowError,
                "bind(): port must be 0-65535.",
            ));
        };
        let socket = match UdpSocket::bind((HOST, bound)) {
            Ok(socket) => socket,
            Err(error) => {
                return Err(Error::new(
                    Kind::OsError,
                    format!(
                        "port {port} is taken, so pa3ds cannot touch: {}",
                        os_error(&error)
                    ),
                ));
            }
        };
        socket.set_read_timeout(Some(Duration::from_secs_f64(TICK)))?;
        let inner = Arc::new(Inner {
            log,
            id,
            state: Mutex::new(State {
                touch: None,
                until: 0.0,
                counter: 0,
                clients: IndexMap::new(),
            }),
            stop: AtomicBool::new(false),
            socket,
        });
        let running = inner.clone();
        let thread = std::thread::Builder::new()
            .name("pa3ds touch".to_string())
            .spawn(move || running.run())
            .map_err(|_| Error::new(Kind::RuntimeError, "can't start new thread"))?;
        Ok(TouchServer {
            inner,
            thread: Mutex::new(Some(thread)),
        })
    }

    /// The port the server took: `PORT`, or whichever was free when it was asked for 0.
    pub fn port(&self) -> Result<i64> {
        Ok(i64::from(self.inner.socket.local_addr()?.port()))
    }

    /// Whether Azahar has asked for the pad lately: a tap reaches nobody otherwise.
    pub fn listened(&self) -> bool {
        let now = time::monotonic();
        let state = self.inner.locked();
        state.clients.values().any(|&when| now - when < LISTENING)
    }

    /// Press the touch screen at (x, y), its pixels from the top left, and let go. Says why not
    /// where Azahar has not asked for the pad lately, which is Azahar without pa3ds's touch
    /// settings (`config.AZAHAR`): closed and opened again after pa3ds patched them, it asks.
    pub fn tap(&self, x: i64, y: i64) -> Option<String> {
        if !self.listened() {
            return Some(NOT_LISTENING.to_string());
        }
        let (x, y) = (x.clamp(0, WIDTH - 1), y.clamp(0, HEIGHT - 1));
        let mut state = self.inner.locked();
        state.touch = Some((x, y));
        state.until = time::monotonic() + HOLD;
        None
    }

    pub fn close(&self) {
        self.inner.locked().touch = None;
        self.inner.send();
        self.inner.stop.store(true, Ordering::SeqCst);
        let taken = self
            .thread
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(thread) = taken {
            // `thread.join(timeout=1)`: a thread that has not stopped in a second is left behind.
            let waited = Instant::now();
            while !thread.is_finished() && waited.elapsed() < Duration::from_secs(1) {
                std::thread::sleep(Duration::from_millis(1));
            }
            if thread.is_finished() {
                let _ = thread.join();
            }
        }
        // `socket.close()` is the socket dropping with the server.
    }
}

impl Inner {
    fn locked(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn run(&self) {
        let mut buffer = [0u8; 100];
        while !self.stop.load(Ordering::SeqCst) {
            let (packet, client) = match self.socket.recv_from(&mut buffer) {
                Ok((size, client)) => (buffer[..size].to_vec(), Some(client)),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    (Vec::new(), None)
                }
                Err(_) => {
                    if self.stop.load(Ordering::SeqCst) {
                        return;
                    }
                    continue;
                }
            };
            if let Some(client) = client {
                // A send that fails here ends the thread, as the exception ended Python's.
                if self.answer(&packet, client).is_err() {
                    return;
                }
            }
            {
                let mut state = self.locked();
                if state.touch.is_some() && time::monotonic() >= state.until {
                    state.touch = None;
                }
            }
            self.send();
        }
    }

    fn answer(&self, packet: &[u8], client: SocketAddr) -> io::Result<()> {
        let kind = request(packet);
        if kind == Some(VERSION_TYPE) {
            let reply = message(VERSION_TYPE, &pack_u16(VERSION).expect("in range"), self.id)
                .expect("in range");
            self.socket.send_to(&reply, client)?;
        } else if kind == Some(PORT_INFO) {
            let reply = message(PORT_INFO, &PAD, self.id).expect("in range");
            self.socket.send_to(&reply, client)?;
        } else if kind == Some(PAD_DATA) {
            let mut state = self.locked();
            if !state.clients.contains_key(&client) {
                (self.log)(&format!(
                    "touch: Azahar is listening, at {}:{}",
                    client.ip(),
                    client.port()
                ));
            }
            state.clients.insert(client, time::monotonic());
        }
        Ok(())
    }

    fn send(&self) {
        let now = time::monotonic();
        let (state, clients) = {
            let mut held = self.locked();
            held.clients.retain(|_, when| now - *when < LISTENING);
            if held.clients.is_empty() {
                return;
            }
            held.counter += 1;
            let state = message(
                PAD_DATA,
                &pad_state(held.counter, held.touch).expect("in range"),
                self.id,
            )
            .expect("in range");
            (state, held.clients.keys().copied().collect::<Vec<_>>())
        };
        for client in clients {
            let _ = self.socket.send_to(&state, client);
        }
    }
}
