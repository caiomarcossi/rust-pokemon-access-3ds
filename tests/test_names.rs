//! Names for what the game does not name: the shipped table, the player's own, and precedence.

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use indexmap::IndexMap;

use crate::core::names::Names;

const NORMAN: i64 = 0x13F;
const WALLY: i64 = 0xB4;
const GYM: i64 = 0x1AF;

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-names-{}-{}",
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

/// `Path.read_text(encoding="utf-8")`, whose universal newlines read a Windows file's CRLF as LF.
fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn shipped(tmp_path: &Path, body: &str) -> PathBuf {
    let data = tmp_path.join("data").join("en");
    fs::create_dir_all(&data).unwrap();
    fs::write(data.join("sprites.toml"), body).unwrap();
    tmp_path.join("data")
}

fn load(data: &Path, language: &str, fingerprint: &str, home: &Path) -> Names {
    Names::load(data, language, fingerprint, Some(home), "").unwrap()
}

/// A sprite is one character everywhere in the game: Norman named once is Norman in the Gym
/// and in the street (spike 75).
#[test]
fn test_a_shipped_sprite_is_said_for_every_map_it_walks_on() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "[sprites]\n\"0x13f\" = \"Norman\"\n");
    let names = load(&data, "en", "sango-1", &tmp_path.join("home"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    assert_eq!(names.sprite(WALLY), None);
}

#[test]
fn test_a_name_the_player_gives_wins_and_is_kept() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "[sprites]\n\"0x13f\" = \"Norman\"\n");
    let home = tmp_path.join("home");
    let names = load(&data, "en", "sango-1", &home);
    names.name_sprite(NORMAN, "Dad").unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Dad"));
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
    let again = load(&data, "en", "sango-1", &home);
    assert_eq!(again.sprite(NORMAN).as_deref(), Some("Dad"));
    assert_eq!(again.sprite(WALLY).as_deref(), Some("Wally"));
}

#[test]
fn test_an_empty_name_gives_the_character_back_to_the_shipped_table() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "[sprites]\n\"0x13f\" = \"Norman\"\n");
    let names = load(&data, "en", "sango-1", &tmp_path.join("home"));
    names.name_sprite(NORMAN, "Dad").unwrap();
    names.name_sprite(NORMAN, "  ").unwrap();
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}

/// May's sprite, which the rival's work object is filled with in the boy's game (spike 89).
const RIVAL: i64 = 0x13A;

/// The rival is "Rival" in the shipped table and May in the ROM: a word pa3ds invented never
/// replaces one the game says (ADR-0023).
#[test]
fn test_the_games_own_word_comes_before_the_shipped_one() {
    let tmp_path = TmpPath::new();
    let data = shipped(
        &tmp_path,
        "[sprites]\n\"0x13a\" = \"Rival\"\n\"0x13f\" = \"Norman\"\n",
    );
    let names = load(&data, "en", "sango-1", &tmp_path.join("home"));
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("Rival"));
    names.use_game(
        Some(Rc::new(|character| {
            (character == RIVAL).then(|| "May".to_string())
        })),
        None,
    );
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("May"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}

#[test]
fn test_the_players_own_name_comes_before_the_games() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "[sprites]\n\"0x13a\" = \"Rival\"\n");
    let names = load(&data, "en", "sango-1", &tmp_path.join("home"));
    names.use_game(
        Some(Rc::new(|character| {
            (character == RIVAL).then(|| "May".to_string())
        })),
        None,
    );
    names.name_sprite(RIVAL, "Neighbour").unwrap();
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("Neighbour"));
    names.name_sprite(RIVAL, "").unwrap();
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("May"));
}

/// A sign is a slot on one map, so naming it says nothing about the sign in the next town.
#[test]
fn test_a_place_is_named_on_its_own_map() {
    let tmp_path = TmpPath::new();
    let names = load(
        &shipped(&tmp_path, ""),
        "en",
        "sango-1",
        &tmp_path.join("home"),
    );
    names.name_place(GYM, 1, "the Gym's sign").unwrap();
    assert_eq!(names.place(GYM, 1).as_deref(), Some("the Gym's sign"));
    assert_eq!(names.place(GYM, 2), None);
    assert_eq!(names.place(0x000D, 1), None);
}

/// One player, two games: a sprite in one is nobody in the other.
#[test]
fn test_names_are_kept_apart_by_fingerprint() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "");
    let home = tmp_path.join("home");
    load(&data, "en", "sango-1", &home)
        .name_sprite(NORMAN, "Norman")
        .unwrap();
    assert_eq!(load(&data, "en", "sapphire-1", &home).sprite(NORMAN), None);
}

/// Being told the wrong name is a nuisance; a reader that stops mid-game is not.
#[test]
fn test_a_names_file_that_cannot_be_read_is_no_names_at_all() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir_all(home.join("names").join("en")).unwrap();
    fs::write(
        home.join("names").join("en").join("sango-1.toml"),
        "{not toml",
    )
    .unwrap();
    let data = shipped(&tmp_path, "[sprites]\n\"0x13f\" = \"Norman\"\n");
    let names = load(&data, "en", "sango-1", &home);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
}

#[test]
fn test_a_shipped_table_that_is_not_there_is_no_names_at_all() {
    let tmp_path = TmpPath::new();
    let names = load(
        &tmp_path.join("nowhere"),
        "en",
        "sango-1",
        &tmp_path.join("home"),
    );
    assert_eq!(names.sprite(NORMAN), None);
}

/// Sharing names is copying the file, so it is written sorted and as words, not as a dump,
/// and it opens by saying which game and language it is for — `sango-1` says nothing to somebody
/// who has just been sent one.
#[test]
fn test_the_players_names_are_a_file_worth_sending_to_someone() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let names = Names::load(
        &shipped(&tmp_path, ""),
        "en",
        "sango-1",
        Some(&home),
        "Pokémon Omega Ruby",
    )
    .unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    names.name_sprite(NORMAN, "Norman").unwrap();
    let body = read_text(&home.join("names").join("en").join("sango-1.toml"));
    assert!(
        body.starts_with(
            "# Names you have given in Pokémon Omega Ruby.\n# ROM sango-1. Language en."
        ),
        "{body}"
    );
    let loaded: ::toml::Table = body.parse().unwrap();
    let sprites = loaded["sprites"].as_table().unwrap();
    assert_eq!(sprites.len(), 2);
    assert_eq!(sprites["0x13f"].as_str(), Some("Norman"));
    assert_eq!(sprites["0xb4"].as_str(), Some("Wally"));
    assert!(body.find("\"0x13f\"").unwrap() < body.find("\"0xb4\"").unwrap());
}

/// The title comes from the emulator's own window, which is not up in a test and will not be
/// up if somebody opens a names file with the game closed. pa3ds holds no table of game names to
/// fall back on, by design (ADR-0031), so the ROM says what it is called.
#[test]
fn test_a_rom_the_emulator_does_not_name_names_itself() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let names = load(&shipped(&tmp_path, ""), "en", "sango-1", &home);
    names.name_sprite(NORMAN, "Norman").unwrap();
    let body = read_text(&home.join("names").join("en").join("sango-1.toml"));
    assert!(
        body.starts_with("# Names you have given in sango-1.\n# ROM sango-1. Language en."),
        "{body}"
    );
}

/// English names for a ROM played in French belong to neither, and the shipped table has been
/// laid out this way all along.
#[test]
fn test_a_name_is_kept_per_language_because_a_name_is_a_word_in_one() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "");
    let home = tmp_path.join("home");
    load(&data, "en", "sango-1", &home)
        .name_sprite(NORMAN, "Dad")
        .unwrap();
    assert_eq!(load(&data, "fr", "sango-1", &home).sprite(NORMAN), None);
    assert_eq!(
        load(&data, "en", "sango-1", &home)
            .sprite(NORMAN)
            .as_deref(),
        Some("Dad")
    );
}

/// A quotation mark or a backslash written into the file unescaped would take every name
/// after it with the line it broke.
#[test]
fn test_a_name_may_hold_anything_a_player_can_type() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let awkward = "the \"old\" man \\ upstairs";
    let names = load(&shipped(&tmp_path, ""), "en", "sango-1", &home);
    names.name_sprite(NORMAN, awkward).unwrap();
    names
        .name_place(GYM, 2, "caf\u{e9}\u{3001} \u{30dd}\u{30b1}\u{30e2}\u{30f3}")
        .unwrap();
    let again = load(&shipped(&tmp_path, ""), "en", "sango-1", &home);
    assert_eq!(again.sprite(NORMAN).as_deref(), Some(awkward));
    assert_eq!(
        again.place(GYM, 2).as_deref(),
        Some("caf\u{e9}\u{3001} \u{30dd}\u{30b1}\u{30e2}\u{30f3}")
    );
}

/// `--no-log` style: a run that writes nothing still answers while it lasts.
#[test]
fn test_names_without_a_home_are_kept_for_this_run_only() {
    let shipped = IndexMap::from([("0x13f".to_string(), "Norman".to_string())]);
    let names = Names::new(Some(&shipped), None, None).unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}
