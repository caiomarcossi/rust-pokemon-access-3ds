//! A Pokémon's contest conditions as they are said, an Extra (movie33's Pikachu).

use crate::games::gen6::screens::conditions::said;

fn words() -> Vec<String> {
    ["Coolness", "Beauty", "Cuteness", "Cleverness", "Toughness"]
        .iter()
        .map(|word| word.to_string())
        .collect()
}

const PIKACHU: [i64; 5] = [116, 70, 70, 70, 70];

#[test]
fn test_each_condition_is_said_after_the_games_word_for_it() {
    assert_eq!(
        said(Some("Conditions"), &words(), &PIKACHU, false),
        "Conditions Coolness 116 Beauty 70 Cuteness 70 Cleverness 70 Toughness 70"
    );
}

#[test]
fn test_pauses_put_a_colon_after_the_heading_and_commas_between() {
    assert_eq!(
        said(Some("Conditions"), &words(), &PIKACHU, true),
        "Conditions: Coolness 116, Beauty 70, Cuteness 70, Cleverness 70, Toughness 70"
    );
}

#[test]
fn test_without_the_games_heading_the_conditions_are_said_alone() {
    assert_eq!(
        said(None, &words(), &[0, 0, 0, 0, 0], false),
        "Coolness 0 Beauty 0 Cuteness 0 Cleverness 0 Toughness 0"
    );
}
