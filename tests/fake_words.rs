//! pa3ds's own words written for a test, in a data folder of its own.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::words::{self, Words};

/// `messages`, Fluent, as a game's file under `data` in one language, and put in place.
pub fn words_in(data: &Path, messages: &str, game: &str, language: &str, file: &str) -> Rc<Words> {
    let folder = data.join(game).join(language);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join(file), messages).unwrap();
    let words = Rc::new(
        Words::new(
            language,
            Some(&data.join(game)),
            IndexMap::new(),
            Rc::new(|_line: &str| {}),
        )
        .unwrap(),
    );
    words::r#use(Some(words.clone()));
    words
}

/// `words_in` with its defaults: Omega Ruby's English `pictures.ftl`.
pub fn pictures_in(data: &Path, messages: &str) -> Rc<Words> {
    words_in(data, messages, "oras", "en", "pictures.ftl")
}
