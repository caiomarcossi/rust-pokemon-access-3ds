//! Replay tapes through the Rust runtime and compare what it logged and said with what the
//! Python runtime did on the same tape (`tools/tape/README.md`).
//!
//!     cargo run --release --bin tape -- <tape>...
//!
//! Each `<name>.tape.gz` is compared with `<name>.py.log` and `<name>.py.said` beside it, the
//! Python runtime's replay of the same tape (`tools/tape/py.py replay`), and the Rust runtime's
//! own are written as `<name>.rust.log` and `<name>.rust.said`. Exits 1 if any tape differs.

use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use pa3ds::app::{self, Settings};
use pa3ds::core::rpc::Rpc;
use pa3ds::core::speech::{Backend, Speech};
use pa3ds::py::{Log, time};
use pa3ds::tape::{Stop, Tape, TapeClock, TapeRpc};

/// What would be said, kept: `!` before a line that interrupts, as the Python's `Heard` keeps it.
struct Heard(Rc<RefCell<Vec<String>>>);

impl Backend for Heard {
    fn speak(&self, text: &str, interrupt: bool) {
        self.0
            .borrow_mut()
            .push(format!("{}{text}", if interrupt { "!" } else { "" }));
    }

    fn braille(&self, _text: &str) {}
}

/// The settings `spikes/regress.py` gives a replay at real speed: the defaults, with the
/// replays' own user folder, never the player's.
fn replay_settings() -> Settings {
    let home = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|local| local.join("pa3ds").join("captures").join("replay-home"));
    Settings {
        names_home: home,
        ..Settings::default()
    }
}

/// A file's lines, CRLF or LF.
fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// Lines compared as the tape compares them: a reader's failure by its first line only, its
/// traceback naming the other language's files.
fn comparable(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_failure = false;
    for line in lines {
        if line == "a reader failed, carrying on:" {
            in_failure = true;
            out.push(line.clone());
            continue;
        }
        if in_failure
            && (line.starts_with(' ')
                || line.contains("Error")
                || line.contains("Traceback")
                || line.starts_with("panic"))
        {
            continue;
        }
        in_failure = false;
        out.push(line.clone());
    }
    out
}

/// The first difference between what Python wrote and what Rust wrote, or None.
fn first_difference(python: &[String], rust: &[String]) -> Option<String> {
    let (python, rust) = (comparable(python), comparable(rust));
    let at = python.iter().zip(&rust).position(|(a, b)| a != b);
    match at {
        Some(at) => Some(format!(
            "line {}:\n    python: {}\n    rust:   {}",
            at + 1,
            python[at],
            rust[at]
        )),
        None if python.len() != rust.len() => Some(format!(
            "python has {} lines, rust {}; the first unmatched: {}",
            python.len(),
            rust.len(),
            python
                .get(rust.len())
                .or(rust.get(python.len()))
                .cloned()
                .unwrap_or_default()
        )),
        None => None,
    }
}

fn replay(path: &Path) -> Result<bool, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".tape.gz"))
        .ok_or_else(|| format!("{} is not a .tape.gz", path.display()))?;
    let beside = |suffix: &str| path.with_file_name(format!("{name}{suffix}"));
    let tape = Tape::open(path).map_err(|error| error.to_string())?;
    let logged = Rc::new(RefCell::new(Vec::<String>::new()));
    let said = Rc::new(RefCell::new(Vec::<String>::new()));
    let log: Log = {
        let logged = logged.clone();
        Rc::new(move |line: &str| {
            logged
                .borrow_mut()
                .extend(line.split('\n').map(str::to_string))
        })
    };
    let previous = time::set_clock(Some(Box::new(TapeClock::new(tape.clone()))));
    let rpc: Rc<dyn Rpc> = Rc::new(TapeRpc::new(tape.clone()));
    let speech = Rc::new(Speech::plain(Rc::new(Heard(said.clone()))));
    let ran = panic::catch_unwind(AssertUnwindSafe(|| {
        app::run(rpc, speech, replay_settings(), &|| false, log)
    }));
    time::set_clock(previous);
    let ending = match ran {
        Err(payload) => match payload.downcast::<Stop>() {
            Ok(stop) => *stop,
            Err(payload) => panic::resume_unwind(payload),
        },
        Ok(Ok(())) => Stop::Diverged("the runtime returned before the tape ended".into()),
        Ok(Err(error)) => Stop::Diverged(format!("the runtime raised {error:?}")),
    };
    let (logged, said) = (logged.borrow().clone(), said.borrow().clone());
    let write = |suffix: &str, lines: &[String]| {
        let text: String = lines.iter().map(|line| format!("{line}\n")).collect();
        std::fs::write(beside(suffix), text).map_err(|error| error.to_string())
    };
    write(".rust.log", &logged)?;
    write(".rust.said", &said)?;
    let read = |suffix: &str| -> Result<Vec<String>, String> {
        std::fs::read_to_string(beside(suffix))
            .map(|text| lines(&text))
            .map_err(|error| format!("{}: {error}", beside(suffix).display()))
    };
    let (python_log, python_said) = (read(".py.log")?, read(".py.said")?);
    let mut same = true;
    let mut report = format!(
        "{name}: {} records, {} lines, {} said",
        tape.count(),
        logged.len(),
        said.len()
    );
    if let Stop::Diverged(why) = &ending {
        same = false;
        report += &format!("\n  DIVERGED, {why}");
    }
    if let Some(difference) = first_difference(&python_log, &logged) {
        same = false;
        report += &format!("\n  log differs at {difference}");
    }
    if let Some(difference) = first_difference(&python_said, &said) {
        same = false;
        report += &format!("\n  speech differs at {difference}");
    }
    if same {
        report += ", identical to the Python runtime's";
    }
    println!("{report}");
    Ok(same)
}

fn main() -> ExitCode {
    let tapes: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if tapes.is_empty() {
        eprintln!("usage: tape <tape.gz>...");
        return ExitCode::from(2);
    }
    let mut all = true;
    for tape in &tapes {
        match replay(tape) {
            Ok(same) => all &= same,
            Err(error) => {
                println!("{}: {error}", tape.display());
                all = false;
            }
        }
    }
    if all {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
