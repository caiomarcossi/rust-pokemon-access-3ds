//! `pa3ds` (or `python -m pa3ds`): announce what the game shows. The executable is `src/main.rs`,
//! which calls `main` here, so the tests can reach what the Python's reached.

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::app::{self, Settings};
use crate::core::rpc::{DEFAULT_PORT, Rpc, UdpRpc};
use crate::core::speech::{self, Backend, Category, Speech};
use crate::core::{config, keyfile, window};
use crate::games::common::field::keys::{DEFAULT_BINDINGS, EXPLAINED};
use crate::py::{self, Log, time};

/// Runs whose logs are kept, this one included.
pub const KEPT_LOGS: usize = 10;
const LOG_NAME: &str = "pa3ds.log";
const LOG_FOLDER: &str = "logs";

/// A backend that says nothing, for runs where the log is the output.
pub struct Silent;

impl Backend for Silent {
    fn speak(&self, _text: &str, _interrupt: bool) {}

    fn braille(&self, _text: &str) {}
}

/// The command line: a flag is only what a developer passes for one run; settings are the
/// file's (Orion, 2026-09-30).
#[derive(Default)]
struct Args {
    silent: bool,
    seconds: Option<f64>,
    port: u16,
    no_keys: bool,
    rom: Option<PathBuf>,
    no_log: bool,
    log_path: Option<PathBuf>,
}

fn usage() -> String {
    "usage: pa3ds [-h] [--silent] [--seconds SECONDS] [--port PORT] [--no-keys] [--rom PATH]\n\
     \x20            [--no-log | --log-path PATH]"
        .to_string()
}

fn help() -> Result<String, String> {
    let config = config::config_path(None).map_err(|error| error.to_string())?;
    Ok(format!(
        "{}\n\nAnnounce what the running game shows.\n\noptions:\n\
         \x20 -h, --help       show this help message and exit\n\
         \x20 --silent         log what would be said, say nothing\n\
         \x20 --seconds SECONDS\n\
         \x20                  stop after this many seconds\n\
         \x20 --port PORT      the emulator's RPC port\n\
         \x20 --no-keys        answer no keys and patch no buttons, whatever the settings say\n\
         \x20 --rom PATH       your decrypted ROM, for the game's own names of places and the\n\
         \x20                  rival; by default pa3ds finds it among the ROMs Azahar has opened\n\
         \x20 --no-log         write no log file; what is said is still printed\n\
         \x20 --log-path PATH  write the log to this file, or to pa3ds.log in this folder,\n\
         \x20                  keeping earlier runs beside it\n\n\
         Each run is logged to {} unless --no-log or --log-path says otherwise, and the {} runs \
         before it are kept beside it as pa3ds.1.log (the newest) to pa3ds.{}.log. Everything \
         else is set in {}.",
        usage(),
        default_log_path()
            .map_err(|error| error.to_string())?
            .display(),
        KEPT_LOGS - 1,
        KEPT_LOGS - 1,
        config.display(),
    ))
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        port: DEFAULT_PORT,
        ..Args::default()
    };
    let mut at = 0;
    let value = |at: &mut usize, flag: &str| -> Result<String, String> {
        *at += 1;
        argv.get(*at)
            .cloned()
            .ok_or_else(|| format!("argument {flag}: expected one argument"))
    };
    while at < argv.len() {
        let arg = argv[at].as_str();
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, inline)) if flag.starts_with("--") => (flag, Some(inline.to_string())),
            _ => (arg, None),
        };
        let take = |at: &mut usize| match &inline {
            Some(inline) => Ok(inline.clone()),
            None => value(at, flag),
        };
        match flag {
            "-h" | "--help" => return Err(help()?),
            "--silent" => args.silent = true,
            "--no-keys" => args.no_keys = true,
            "--no-log" => args.no_log = true,
            "--seconds" => {
                let text = take(&mut at)?;
                args.seconds =
                    Some(py::float_of(&text).map_err(|_| {
                        format!("argument --seconds: invalid float value: '{text}'")
                    })?);
            }
            "--port" => {
                let text = take(&mut at)?;
                args.port = text
                    .trim()
                    .parse()
                    .map_err(|_| format!("argument --port: invalid int value: '{text}'"))?;
            }
            "--rom" => args.rom = Some(PathBuf::from(take(&mut at)?)),
            "--log-path" => args.log_path = Some(PathBuf::from(take(&mut at)?)),
            other => return Err(format!("unrecognized arguments: {other}")),
        }
        at += 1;
    }
    if args.no_log && args.log_path.is_some() {
        return Err("argument --log-path: not allowed with argument --no-log".to_string());
    }
    Ok(args)
}

/// The command line `argv` (without the program's name), run: 0 when it ran, 2 when the arguments
/// were wrong, 1 when it failed.
pub fn main(argv: &[String]) -> i32 {
    main_with(argv, &Hooks::default())
}

/// What `main` hands its run to: the runtime, the backend `--silent` says nothing through, and
/// whether the game's window is in front. The Python's tests patched each of these in place.
pub struct Hooks {
    pub run:
        Box<dyn Fn(Rc<dyn Rpc>, Rc<Speech>, Settings, &dyn Fn() -> bool, Log) -> py::Result<()>>,
    pub silent: Rc<dyn Backend>,
    pub in_front: Rc<dyn Fn() -> bool>,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks {
            run: Box::new(app::run),
            silent: Rc::new(Silent),
            in_front: Rc::new(|| window::in_front(window::EMULATOR)),
        }
    }
}

/// `main`, run through `hooks`.
pub fn main_with(argv: &[String], hooks: &Hooks) -> i32 {
    let args = match parse(argv) {
        Ok(args) => args,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return 0;
        }
        Err(message) => {
            eprintln!(
                "{}
pa3ds: error: {message}",
                usage()
            );
            return 2;
        }
    };
    match run(args, hooks) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("pa3ds: {}: {}", error.kind.name(), error.message);
            1
        }
    }
}

fn run(args: Args, hooks: &Hooks) -> py::Result<()> {
    // A patch of Azahar's config is lost when an open Azahar closes, so none is written while
    // it is open.
    let running = !window::titles(window::EMULATOR).is_empty();
    let settings = config::prepare(None, None, true, running)?;
    let ui = settings.ui.clone();

    let backend: Rc<dyn Backend> = if args.silent {
        hooks.silent.clone()
    } else {
        let (backend, name) = speech::screen_reader()?;
        println!("speaking through {name}");
        backend
    };

    let log_file: Option<Rc<RefCell<File>>> = if args.no_log {
        println!("not logging to a file");
        None
    } else {
        let path = log_path(args.log_path.as_deref())?;
        println!("logging to {}", path.display());
        Some(Rc::new(RefCell::new(File::create(&path)?)))
    };
    let started = time::monotonic();
    let deadline = args.seconds.map(|seconds| started + seconds);
    let rpc: Rc<dyn Rpc> = Rc::new(UdpRpc::connect(args.port)?);

    let log: Log = {
        let log_file = log_file.clone();
        Rc::new(move |line: &str| {
            let stamped = format!("{:8.2} {line}", time::monotonic() - started);
            println!("{stamped}");
            if let Some(file) = &log_file {
                let mut file = file.borrow_mut();
                let _ = writeln!(file, "{stamped}");
                let _ = file.flush();
            }
        })
    };

    // A key's answer always interrupts (ADR-0010): `interrupt` adds menus to it. Until
    // 2026-09-24 this passed menus or nothing, and so no key ever cut speech off.
    let mut interrupts = speech::default_interrupts();
    if ui.interrupt {
        interrupts.insert(Category::Menu);
    }
    // Quiet while another window is in front, so reading it is never talked over (ADR-0021).
    let listening: Option<speech::Listening> = if ui.unfocused {
        None
    } else {
        Some(hooks.in_front.clone())
    };
    let speech = Rc::new(Speech::new(backend, interrupts, listening));
    // Before the game, not after: what a key does and which buttons it costs is about the
    // keyboard, and there is nothing to wait for. A patched binding needs the emulator
    // restarted anyway, so the player wants to hear about it before they start playing.
    for note in &settings.notes {
        log(note);
    }
    if !settings.said.is_empty() {
        speech.say(
            &settings.said,
            if settings.warning {
                Category::Warning
            } else {
                Category::System
            },
        );
    }
    let mut bindings = None;
    if settings.general.keys && !args.no_keys {
        let explained: IndexMap<String, String> = EXPLAINED
            .iter()
            .map(|(key, why)| (key.to_string(), why.to_string()))
            .collect();
        let keys = keyfile::prepare(
            &DEFAULT_BINDINGS,
            None,
            None,
            true,
            running,
            Some(&explained),
        )?;
        for note in &keys.notes {
            log(note);
        }
        if !keys.said.is_empty() {
            speech.say(&keys.said, Category::System);
        }
        bindings = Some(keys.bindings);
    }

    let until = move || deadline.is_some_and(|deadline| time::monotonic() >= deadline);
    (hooks.run)(
        rpc,
        speech,
        Settings {
            tick: 1.0 / settings.general.poll_rate,
            ui,
            navigation: settings.navigation.clone(),
            extras: settings.extras.clone(),
            keys: bindings,
            names_home: Some(keyfile::home_path()?),
            rom: args.rom,
            ..Settings::default()
        },
        &until,
        log,
    )
}

/// `<pa3ds>/user/logs/pa3ds.log`, among everything else the player owns (ADR-0032), in a folder
/// of its own so ten runs of it do not bury the keys file and the names (the user, 2026-09-24).
pub fn default_log_path() -> py::Result<PathBuf> {
    Ok(keyfile::home_path()?.join(LOG_FOLDER).join(LOG_NAME))
}

/// Where this run's log goes, with the runs before it moved aside to make room.
///
/// `where` is a file, or a folder to put `pa3ds.log` in; by default `default_log_path()`. The runs
/// before are kept beside it with a number before the extension, `pa3ds.1.log` the last to
/// `pa3ds.9.log`. A live session's log is what shows why something was said, or not said, when no
/// replay reproduces it, and a player testing from the start of the game restarts pa3ds more than
/// once.
pub fn log_path(place: Option<&Path>) -> py::Result<PathBuf> {
    let mut path = match place {
        None => default_log_path()?,
        Some(place) => place.to_path_buf(),
    };
    if path.is_dir() {
        path = path.join(LOG_NAME);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    rotate(&path)?;
    Ok(path)
}

/// `RotatingFileHandler(path, backupCount=KEPT_LOGS - 1).doRollover()`, with the number put
/// before the extension: each kept run moves one number up, the oldest is dropped, and the last
/// run becomes `.1`.
fn rotate(path: &Path) -> py::Result<()> {
    let numbered = |n: usize| -> PathBuf {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = match path.extension() {
            Some(extension) => format!("{stem}.{n}.{}", extension.to_string_lossy()),
            None => format!("{stem}.{n}"),
        };
        path.with_file_name(name)
    };
    for n in (1..KEPT_LOGS - 1).rev() {
        let (source, target) = (numbered(n), numbered(n + 1));
        if source.exists() {
            if target.exists() {
                std::fs::remove_file(&target)?;
            }
            std::fs::rename(&source, &target)?;
        }
    }
    let first = numbered(1);
    if first.exists() {
        std::fs::remove_file(&first)?;
    }
    if path.exists() {
        std::fs::rename(path, &first)?;
    }
    Ok(())
}
