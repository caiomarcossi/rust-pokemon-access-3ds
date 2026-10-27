//! pa3ds's own words: a message in the player's language, then English's, a game's before every
//! game's (ADR-0040).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fluent_syntax::ast;
use indexmap::IndexMap;
use regex::Regex;

use super::fake_words::words_in;
use crate::core::keyfile::data_path;
use crate::core::words::{self, ENGLISH, Words, own};

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-words-{}-{}",
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

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn write(folder: &Path, file: &str, messages: &str) {
    fs::create_dir_all(folder).unwrap();
    fs::write(folder.join(file), messages).unwrap();
}

fn words(language: &str, game: Option<&Path>) -> Words {
    Words::new(language, game, IndexMap::new(), Rc::new(|_line: &str| {})).unwrap()
}

#[test]
fn test_a_message_missing_in_the_language_is_said_in_english() {
    let tmp_path = TmpPath::new();
    write(
        &tmp_path.join("oras").join("en"),
        "words.ftl",
        "north = north\nsouth = south\n",
    );
    write(
        &tmp_path.join("oras").join("fr"),
        "words.ftl",
        "north = nord\n",
    );
    let french = words("fr", Some(&tmp_path.join("oras")));
    assert_eq!(
        (french.say("north", &[]), french.say("south", &[])),
        ("nord".to_string(), "south".to_string())
    );
}

#[test]
fn test_a_games_message_wins_over_every_games() {
    let tmp_path = TmpPath::new();
    write(
        &tmp_path.join("common").join("en"),
        "words.ftl",
        "empty = empty\nstart = start\n",
    );
    write(
        &tmp_path.join("oras").join("en"),
        "words.ftl",
        "start = begin\n",
    );
    let found = words(ENGLISH, Some(&tmp_path.join("oras")));
    assert_eq!(
        (found.say("empty", &[]), found.say("start", &[])),
        ("empty".to_string(), "begin".to_string())
    );
}

#[test]
fn test_a_message_no_language_has_is_none_and_said_as_its_id() {
    let tmp_path = TmpPath::new();
    let found = words(ENGLISH, Some(&tmp_path.join("oras")));
    assert_eq!(found.get("nowhere", &[]), None);
    assert_eq!(found.say("nowhere", &[]), "nowhere");
}

/// Python's library marks each value inserted and groups digits by the language; Rust's never
/// groups, and the port says what this says.
#[test]
fn test_values_are_inserted_bare_and_numbers_ungrouped() {
    let tmp_path = TmpPath::new();
    let found = words_in(
        &tmp_path,
        "leg = { $steps } { $way }\n",
        "oras",
        "fr",
        "words.ftl",
    );
    assert_eq!(
        found.say("leg", &[("steps", &1234_i64), ("way", &"nord")]),
        "1234 nord"
    );
}

#[test]
fn test_a_plural_follows_the_languages_own_rules() {
    let tmp_path = TmpPath::new();
    let star = "stars = { $count ->\n    [one] { $count } star\n   *[other] { $count } stars\n}\n";
    write(&tmp_path.join("oras").join("fr"), "words.ftl", star);
    write(&tmp_path.join("oras").join("en"), "words.ftl", star);
    assert_eq!(
        words("fr", Some(&tmp_path.join("oras"))).say("stars", &[("count", &0_i64)]),
        "0 star"
    );
    assert_eq!(
        words(ENGLISH, Some(&tmp_path.join("oras"))).say("stars", &[("count", &0_i64)]),
        "0 stars"
    );
}

#[test]
fn test_a_word_made_from_a_name_is_logged_once() {
    let tmp_path = TmpPath::new();
    let logged: Rc<std::cell::RefCell<Vec<String>>> = Rc::default();
    let log = {
        let logged = logged.clone();
        Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
    };
    let found = Words::new(ENGLISH, Some(&tmp_path.join("oras")), IndexMap::new(), log).unwrap();
    found.made("texture fight_button", "fight");
    found.made("texture fight_button", "fight");
    assert_eq!(
        *logged.borrow(),
        vec!["no word in data for texture fight_button; said 'fight'".to_string()]
    );
}

#[test]
fn test_with_none_in_place_the_words_are_every_games_english() {
    words::r#use(None);
    assert_eq!(own("picture-empty", &[]), "empty");
}

/// A line Fluent cannot read is dropped from the file silently, and its message said as its
/// id: every file of every language reads whole.
#[test]
fn test_every_file_parses() {
    for path in ftl_files(&data_path().unwrap()) {
        let text = fs::read_to_string(&path).unwrap();
        let resource = match fluent_syntax::parser::parse(text.as_str()) {
            Ok(resource) => resource,
            Err((resource, _errors)) => resource,
        };
        let junk: Vec<&str> = resource
            .body
            .iter()
            .filter_map(|entry| match entry {
                ast::Entry::Junk { content } => Some(*content),
                _ => None,
            })
            .collect();
        assert!(junk.is_empty(), "{}: {junk:?}", path.display());
    }
}

/// A translation is of the English: a message only another language has is left over from an
/// English one renamed or taken out.
#[test]
fn test_no_language_has_a_message_english_lacks() {
    for folder in folders(&data_path().unwrap()) {
        let english = ids(&folder.join(ENGLISH));
        for language in folders(&folder) {
            let theirs = ids(&language);
            let extra: Vec<&String> = theirs.difference(&english).collect();
            assert!(extra.is_empty(), "{}: {extra:?}", language.display());
        }
    }
}

/// Brazilian Portuguese is a complete translation, not a folder that silently falls back to
/// English. Every English file and message is present, and every Fluent value, function and
/// selector remains attached to the same message.
#[test]
fn test_brazilian_portuguese_matches_every_english_message_and_placeable() {
    const PORTUGUESE: &str = "pt-BR";
    let data = data_path().unwrap();
    let token =
        Regex::new(r"\{[^{}\n]*\}|\{\s*\$[A-Za-z][A-Za-z0-9_-]*\s*->|\*?\[[A-Za-z0-9_-]+\]")
            .unwrap();
    for folder in folders(&data) {
        let english = folder.join(ENGLISH);
        let portuguese = folder.join(PORTUGUESE);
        let english_files = file_names(&english);
        assert_eq!(
            file_names(&portuguese),
            english_files,
            "{} does not mirror English's files",
            portuguese.display()
        );
        for file in english_files {
            let english_path = english.join(&file);
            let portuguese_path = portuguese.join(&file);
            let english_text = fs::read_to_string(&english_path).unwrap();
            let portuguese_text = fs::read_to_string(&portuguese_path).unwrap();
            assert!(
                !portuguese_text.contains("__PA3DS") && !portuguese_text.contains("__ZXQ"),
                "{} contains a translation-work marker",
                portuguese_path.display()
            );
            let english_messages = message_blocks(&english_text);
            let portuguese_messages = message_blocks(&portuguese_text);
            assert_eq!(
                portuguese_messages.keys().collect::<Vec<_>>(),
                english_messages.keys().collect::<Vec<_>>(),
                "{} does not contain every English message",
                portuguese_path.display()
            );
            for (id, english_block) in english_messages {
                let portuguese_block = &portuguese_messages[&id];
                let tokens = |block: &str| {
                    token
                        .find_iter(block)
                        .map(|found| found.as_str().to_string())
                        .collect::<Vec<_>>()
                };
                assert_eq!(
                    tokens(portuguese_block),
                    tokens(&english_block),
                    "{} changes Fluent syntax in {id}",
                    portuguese_path.display()
                );
            }
        }
        assert_eq!(
            words(PORTUGUESE, Some(&folder)).language,
            PORTUGUESE,
            "{}",
            folder.display()
        );
    }
}

/// A string in the code shaped as an id, beginning as some English message does, is one:
/// `picture-empty`. Said as its id, a message the data lacks would go unnoticed.
#[test]
fn test_every_message_the_code_names_is_in_english() {
    let data = data_path().unwrap();
    let english: BTreeSet<String> = folders(&data)
        .iter()
        .flat_map(|folder| ids(&folder.join(ENGLISH)))
        .collect();
    let prefixes: BTreeSet<&str> = english
        .iter()
        .map(|message| message.split('-').next().unwrap())
        .collect();
    let literal = Regex::new(r#""([a-z][a-z0-9]*(?:-[a-z0-9_]+)+)""#).unwrap();
    let mut named = BTreeSet::new();
    for path in rs_files(&source()) {
        let text = fs::read_to_string(&path).unwrap();
        for found in literal.captures_iter(&text) {
            let id = found[1].to_string();
            if prefixes.contains(id.split('-').next().unwrap()) {
                named.insert(id);
            }
        }
    }
    assert!(!named.is_empty());
    let missing: Vec<&String> = named.difference(&english).collect();
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn test_each_games_folder_reads() {
    for game in ["oras", "xy", "sm"] {
        let found = words(ENGLISH, Some(&data_path().unwrap().join(game)));
        assert_eq!(found.language, ENGLISH, "{game}");
    }
}

fn ids(folder: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Ok(entries) = fs::read_dir(folder) else {
        return out;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "ftl") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let resource = match fluent_syntax::parser::parse(text.as_str()) {
            Ok(resource) => resource,
            Err((resource, _errors)) => resource,
        };
        for entry in resource.body {
            if let ast::Entry::Message(message) = entry {
                out.insert(message.id.name.to_string());
            }
        }
    }
    out
}

fn folders(at: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(at)
        .into_iter()
        .flatten()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    out.sort();
    out
}

fn file_names(at: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(at)
        .into_iter()
        .flatten()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ftl"))
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

fn message_blocks(text: &str) -> BTreeMap<String, String> {
    let start = Regex::new(r"(?m)^([A-Za-z0-9][A-Za-z0-9_-]*)\s*=").unwrap();
    let found: Vec<_> = start.captures_iter(text).collect();
    let mut out = BTreeMap::new();
    for (index, captures) in found.iter().enumerate() {
        let whole = captures.get(0).unwrap();
        let end = found
            .get(index + 1)
            .map(|next| next.get(0).unwrap().start())
            .unwrap_or(text.len());
        out.insert(
            captures[1].to_string(),
            text[whole.start()..end].to_string(),
        );
    }
    out
}

fn ftl_files(at: &Path) -> Vec<PathBuf> {
    walk(at, "ftl")
}

fn rs_files(at: &Path) -> Vec<PathBuf> {
    walk(at, "rs")
}

fn walk(at: &Path, extension: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut pending = vec![at.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|found| found == extension) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}
