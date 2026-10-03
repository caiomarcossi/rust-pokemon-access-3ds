//! Speech: what gets said, and whether it cuts off what is already being said.
//!
//! The NVDA backend cannot report whether it is still speaking (spike 1), so pa3ds never waits for
//! speech to finish. NVDA queues on its own when `interrupt` is false; all pa3ds decides is the
//! interrupt flag, per speech category. Nothing interrupts by default: a screen reader already cuts
//! speech when a key is pressed in the game's window, and pa3ds interrupting itself cut an item under
//! the cursor off with the description the same move brought up (the user, 2026-09-14). Categories
//! can be made to interrupt, as `[pa3ds.ui] interrupt` does for menus.
//!
//! Everything a reader reports is said, identical or not. Readers report changes, not states, so
//! polling the same state again says nothing; and when the game shows the same words again — a second
//! conversation with the same person — a sighted player sees them again, so they are heard again.
//! This reverses an earlier "identical text is never repeated" rule, which silenced exactly that
//! (the user, 2026-09-13).

use std::collections::HashSet;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::rc::Rc;

use crate::py::{self, Error, Kind, Result};

/// What a piece of speech is. Decides whether it interrupts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Menu,
    /// A list opening, such as Yes/No. It queues, so it does not cut off the question before it.
    Prompt,
    /// Words a screen draws outside any message window as it comes up: a menu's labels, a question
    /// over a list. It queues, so a screen that opens after a message does not cut the message off.
    Screen,
    Dialogue,
    Battle,
    System,
    /// The answer to a key the player pressed. It interrupts: the player asked for it now, and
    /// the key never reaches the game's window, so the screen reader has not cut speech for it.
    Key,
    /// The next leg of a path, said as the player walks up to each turn. It queues: two turns a
    /// step apart are both heard, the second after the first.
    Guidance,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Menu,
        Category::Prompt,
        Category::Screen,
        Category::Dialogue,
        Category::Battle,
        Category::System,
        Category::Key,
        Category::Guidance,
    ];

    /// The category's value, as the settings file names it.
    pub fn value(self) -> &'static str {
        match self {
            Category::Menu => "menu",
            Category::Prompt => "prompt",
            Category::Screen => "screen",
            Category::Dialogue => "dialogue",
            Category::Battle => "battle",
            Category::System => "system",
            Category::Key => "key",
            Category::Guidance => "guidance",
        }
    }
}

pub fn default_interrupts() -> HashSet<Category> {
    HashSet::from([Category::Key])
}

/// Said whether or not the player is in the game's window (ADR-0021): pa3ds's own few lines
/// ("pa3ds ready", a reader failing), said while the player may still be in the window they started
/// pa3ds from, and the answer to a key they pressed, which only comes while the game is in front.
pub const HEARD_ANYWHERE: [Category; 2] = [Category::System, Category::Key];

/// The game font's private-use gender glyphs, as the Unicode signs they draw. A screen reader skips
/// a private-use character but says ♂ and ♀ in the player's own words (the user, 2026-09-14). U+E08E
/// was read beside Torchic's ♂ on the party screen (spike 32); U+E08F, the ♀ beside it in the font,
/// has not been seen in a replay yet.
pub fn symbol(c: char) -> char {
    match c {
        '\u{e08e}' => '♂',
        '\u{e08f}' => '♀',
        c => c,
    }
}

/// Where speech goes: the part of a Prism backend pa3ds uses.
pub trait Backend {
    fn speak(&self, text: &str, interrupt: bool);
    fn braille(&self, text: &str);
}

/// Whether the player is in the game's window.
pub type Listening = Rc<dyn Fn() -> bool>;

/// Says things through one backend, applying the interrupt policy.
pub struct Speech {
    backend: Rc<dyn Backend>,
    interrupts: HashSet<Category>,
    listening: Option<Listening>,
}

impl Speech {
    /// `listening` says whether the player is in the game's window; while it is false only
    /// `HEARD_ANYWHERE` is said, so reading another window is never talked over (ADR-0021).
    /// None says everything, as a replay, whose emulator is never in front, wants.
    pub fn new(
        backend: Rc<dyn Backend>,
        interrupts: HashSet<Category>,
        listening: Option<Listening>,
    ) -> Self {
        Speech {
            backend,
            interrupts,
            listening,
        }
    }

    /// Speech with the default interrupts, heard in any window.
    pub fn plain(backend: Rc<dyn Backend>) -> Self {
        Speech::new(backend, default_interrupts(), None)
    }

    /// Speak `text` unless there is nothing in it, or the player is in another window.
    /// Returns whether it was spoken.
    ///
    /// Whitespace is collapsed first: a line break in game text is where a box ran out of
    /// width, and a screen reader pauses on it as though a sentence ended.
    pub fn say(&self, text: &str, category: Category) -> bool {
        let translated: String = text.chars().map(symbol).collect();
        let spoken = py::split(&translated).join(" ");
        if spoken.is_empty() {
            return false;
        }
        if let Some(listening) = &self.listening
            && !HEARD_ANYWHERE.contains(&category)
            && !listening()
        {
            return false;
        }
        self.backend
            .speak(&spoken, self.interrupts.contains(&category));
        self.backend.braille(&spoken);
        true
    }
}

/// The best backend Prism can find on this machine, and its name.
///
/// `prism.dll` is loaded here, at the first call, rather than linked: tests, and anything else
/// that never speaks, do not need the native library, and a player without it hears why from the
/// log rather than having pa3ds refuse to start. It is looked for beside pa3ds, then wherever
/// Windows looks for a DLL.
pub fn screen_reader() -> Result<(Rc<dyn Backend>, String)> {
    let prism = Prism::load()?;
    let name = prism.name();
    Ok((Rc::new(prism), name))
}

#[repr(C)]
struct PrismConfig {
    version: u8,
    registry: *mut c_void,
    availability_callback: *mut c_void,
    availability_userdata: *mut c_void,
    availability_poll_interval_ms: u32,
    availability_debounce_samples: u32,
    availability_backoff_max_ms: u32,
    availability_auto_power_manage: bool,
    availability_baseline_callback: *mut c_void,
    /// Room for fields a newer Prism adds: `prism_config_init` returns the struct by value, and
    /// writes as much of it as its own version has.
    reserved: [u8; 64],
}

type ConfigInit = unsafe extern "C" fn() -> PrismConfig;
type Init = unsafe extern "C" fn(*mut PrismConfig) -> *mut c_void;
type AcquireBest = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type BackendName = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type BackendInitialize = unsafe extern "C" fn(*mut c_void) -> c_int;
type BackendSpeak = unsafe extern "C" fn(*mut c_void, *const c_char, bool) -> c_int;
type BackendBraille = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;

const PRISM_OK: c_int = 0;
const PRISM_ERROR_ALREADY_INITIALIZED: c_int = 15;

/// A Prism backend reached through `prism.dll`.
struct Prism {
    _library: libloading::Library,
    backend: *mut c_void,
    name: BackendName,
    speak: BackendSpeak,
    braille: BackendBraille,
}

impl Prism {
    fn load() -> Result<Self> {
        let beside = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("prism.dll")))
            .filter(|path| path.exists());
        let missing = |what: String| Error::new(Kind::OsError, what);
        // SAFETY: prism.dll is Prism's own library; loading it runs only its initialisers.
        let library = unsafe {
            match &beside {
                Some(path) => libloading::Library::new(path),
                None => libloading::Library::new("prism.dll"),
            }
        }
        .map_err(|error| missing(format!("prism.dll could not be loaded: {error}")))?;
        // SAFETY: each symbol is looked up by its name in Prism's C API, with the signature the
        // API gives it.
        unsafe {
            let symbol = |name: &[u8]| -> Result<*const c_void> {
                library
                    .get::<*const c_void>(name)
                    .map(|s| *s)
                    .map_err(|error| {
                        missing(format!(
                            "prism.dll has no {}: {error}",
                            String::from_utf8_lossy(&name[..name.len() - 1])
                        ))
                    })
            };
            let config_init: ConfigInit = std::mem::transmute(symbol(b"prism_config_init\0")?);
            let init: Init = std::mem::transmute(symbol(b"prism_init\0")?);
            let acquire_best: AcquireBest =
                std::mem::transmute(symbol(b"prism_registry_acquire_best\0")?);
            let name: BackendName = std::mem::transmute(symbol(b"prism_backend_name\0")?);
            let initialize: BackendInitialize =
                std::mem::transmute(symbol(b"prism_backend_initialize\0")?);
            let speak: BackendSpeak = std::mem::transmute(symbol(b"prism_backend_speak\0")?);
            let braille: BackendBraille = std::mem::transmute(symbol(b"prism_backend_braille\0")?);
            let mut config = config_init();
            let context = init(&mut config);
            if context.is_null() {
                return Err(Error::new(Kind::RuntimeError, "prism could not start"));
            }
            let backend = acquire_best(context);
            if backend.is_null() {
                return Err(Error::new(Kind::RuntimeError, "prism found no backend"));
            }
            let started = initialize(backend);
            if started != PRISM_OK && started != PRISM_ERROR_ALREADY_INITIALIZED {
                return Err(Error::new(
                    Kind::RuntimeError,
                    format!("prism's backend did not start: error {started}"),
                ));
            }
            Ok(Prism {
                _library: library,
                backend,
                name,
                speak,
                braille,
            })
        }
    }

    fn name(&self) -> String {
        // SAFETY: the backend is live for as long as the library is loaded, and Prism returns a
        // NUL-terminated name it owns.
        unsafe {
            let name = (self.name)(self.backend);
            if name.is_null() {
                String::new()
            } else {
                CStr::from_ptr(name).to_string_lossy().into()
            }
        }
    }
}

impl Backend for Prism {
    fn speak(&self, text: &str, interrupt: bool) {
        if let Ok(text) = CString::new(text) {
            // SAFETY: the backend is live, and the text is NUL-terminated and outlives the call.
            unsafe { (self.speak)(self.backend, text.as_ptr(), interrupt) };
        }
    }

    fn braille(&self, text: &str) {
        if let Ok(text) = CString::new(text) {
            // SAFETY: as for `speak`.
            unsafe { (self.braille)(self.backend, text.as_ptr()) };
        }
    }
}
