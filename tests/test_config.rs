//! The settings file: pa3ds's flags as a file, and Azahar's own settings patched in (ADR-0026).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::config::{self, General, Navigation, Table, Ui, Value};

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-config-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for TmpPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// A fresh Azahar's settings, the ones pa3ds ships a value for, CRLF as Azahar writes them.
fn emulator_text() -> String {
    [
        "[Debugging]",
        "enable_rpc_server\\default=true",
        "enable_rpc_server=false",
        "[Miscellaneous]",
        "log_filter\\default=true",
        "log_filter=*:Info",
        "[Renderer]",
        "graphics_api\\default=true",
        "graphics_api=1",
        "async_shader_compilation\\default=true",
        "async_shader_compilation=false",
        "shaders_accurate_mul\\default=true",
        "shaders_accurate_mul=true",
        "use_vsync\\default=true",
        "use_vsync=true",
        "",
    ]
    .join("\r\n")
}

fn emulator(tmp_path: &Path) -> PathBuf {
    let path = tmp_path.join("qt-config.ini");
    fs::write(&path, emulator_text()).unwrap();
    path
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn prepare(home: &Path, emulator: &Path, running: bool) -> config::Config {
    config::prepare(Some(home), Some(emulator), true, running).unwrap()
}

/// `dataclasses.asdict(kind())`, as the table a parser reads it back as.
fn as_table<T: Table>() -> toml::Value {
    toml::Value::Table(
        T::default()
            .fields()
            .into_iter()
            .map(|(name, value)| {
                let value = match value {
                    Value::Bool(value) => toml::Value::Boolean(value),
                    Value::Int(value) => toml::Value::Integer(value),
                    Value::Float(value) => toml::Value::Float(value),
                    Value::Str(value) => toml::Value::String(value),
                };
                (name.to_string(), value)
            })
            .collect(),
    )
}

#[test]
fn test_the_first_run_writes_every_default_and_reads_them_back() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let none = tmp_path.join("none.ini");
    let first = prepare(&home, &none, false);
    let written: toml::Table = read(&home.join("config.toml")).parse().unwrap();
    let ours = written["pa3ds"].as_table().unwrap();
    assert_eq!(ours["keys"], toml::Value::Boolean(true));
    assert_eq!(ours["poll_rate"], toml::Value::Integer(60));
    assert_eq!(ours["ui"], as_table::<Ui>());
    assert_eq!(ours["navigation"], as_table::<Navigation>());
    let shipped: toml::Table = config::azahar()
        .into_iter()
        .map(|(name, (default, _))| {
            let default = match default {
                Value::Bool(default) => toml::Value::Boolean(default),
                Value::Int(default) => toml::Value::Integer(default),
                Value::Float(default) => toml::Value::Float(default),
                Value::Str(default) => toml::Value::String(default),
            };
            (name.to_string(), default)
        })
        .collect();
    assert_eq!(written["azahar"], toml::Value::Table(shipped));
    let again = prepare(&home, &none, false);
    assert_eq!(
        (again.general, again.ui, again.navigation),
        (first.general, first.ui, first.navigation)
    );
    assert!(
        !again
            .notes
            .iter()
            .any(|note| note.contains("no setting") || note.contains("no ["))
    );
}

/// The RPC server above all: Azahar ships it off, and without it pa3ds hears nothing.
#[test]
fn test_a_fresh_azahar_gets_pa3ds_s_settings_with_their_flags_off() {
    let tmp_path = TmpPath::new();
    let ini = emulator(&tmp_path);
    let settings = prepare(&tmp_path.join("home"), &ini, false);
    let after = read(&ini);
    assert!(after.contains("enable_rpc_server\\default=false\r\nenable_rpc_server=true\r\n"));
    assert!(after.contains("graphics_api=2\r\n"));
    assert!(after.contains("log_filter=\"*:Info RPC_Server:Error\"\r\n"));
    assert_eq!(
        after.matches("\r\n").count(),
        emulator_text().matches("\r\n").count()
    );
    assert_eq!(settings.said, "Azahar settings applied");
    assert_eq!(prepare(&tmp_path.join("home"), &ini, false).said, "");
}

#[test]
fn test_azahar_open_is_left_alone() {
    let tmp_path = TmpPath::new();
    let ini = emulator(&tmp_path);
    let settings = prepare(&tmp_path.join("home"), &ini, true);
    assert_eq!(read(&ini), emulator_text());
    assert!(settings.said.contains("close Azahar"));
}

#[test]
fn test_a_setting_of_the_wrong_kind_is_logged_and_left_at_its_default() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[pa3ds]\npoll_rate = 30\n\n[pa3ds.ui]\ninterrupt = true\npauses = \"yes\"\nwibble = 1\n\
         \n[pa3ds.navigation]\nturn_by_turn = 0\n",
    )
    .unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.general.poll_rate, 30.0);
    assert_eq!(
        settings.ui,
        Ui {
            interrupt: true,
            ..Ui::default()
        }
    );
    assert_eq!(settings.navigation, Navigation::default());
    assert!(settings.notes.iter().any(|note| note.contains("pauses")));
    assert!(settings.notes.iter().any(|note| note.contains("wibble")));
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("turn_by_turn"))
    );
}

/// The first file was flat, `positions = true` at the top; it is never overwritten.
#[test]
fn test_a_file_from_before_the_tables_says_what_it_has_lost() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "positions = true\n").unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.ui, Ui::default());
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("no [positions] table"))
    );
}

#[test]
fn test_a_line_taken_out_of_azahar_s_table_leaves_that_setting_alone() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "[azahar]\nuse_vsync = false\n").unwrap();
    let ini = emulator(&tmp_path);
    prepare(&home, &ini, false);
    let after = read(&ini);
    assert!(after.contains("use_vsync=false"));
    assert!(after.contains("graphics_api=1\r\n"));
}

#[test]
fn test_a_name_every_table_has_and_no_setting_is_logged() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "[pa3ds]\n__doc__ = \"x\"\n").unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.general, General::default());
    assert!(settings.notes.iter().any(|note| note.contains("__doc__")));
}

#[test]
fn test_files_that_are_not_utf_8_read_as_nothing() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), b"[pa3ds]\nkeys = false # \xff\n").unwrap();
    let ini = tmp_path.join("qt-config.ini");
    fs::write(&ini, b"enable_rpc_server=\xff\n").unwrap();
    let settings = config::prepare(Some(&home), Some(&ini), false, false).unwrap();
    assert_eq!(settings.general, General::default());
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("could not be read"))
    );
}
