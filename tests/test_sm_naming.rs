use indexmap::IndexMap;

use crate::games::gen6::rom::script::{self, Script};
use crate::games::sm::naming::{interaction_line, label};

fn interaction() -> Script {
    let instructions = IndexMap::from([
        (0, (script::SWITCH, vec![40])),
        (10, (script::CASETBL, vec![1, 0, 3, 28])),
        (20, (script::CALL, vec![40])),
        (22, (script::RETN, vec![])),
        (30, (script::PROC, vec![])),
        (31, (script::PUSH_CONSTANT, vec![1])),
        (32, (script::PUSH_CONSTANT, vec![0])),
        (33, (script::PUSH_CONSTANT, vec![5])),
        (34, (script::PUSH_CONSTANT, vec![12])),
        (35, (script::CALL, vec![60])),
        (37, (script::RETN, vec![])),
        (50, (script::PROC, vec![])),
        (51, (190, vec![20])),
        (52, (190, vec![16])),
        (53, (190, vec![12])),
        (54, (script::PUSH_CONSTANT, vec![12])),
        (55, (script::CALL, vec![60])),
        (57, (script::RETN, vec![])),
        (70, (script::PROC, vec![])),
        (71, (script::NATIVE, vec![0, 44])),
        (74, (script::RETN, vec![])),
    ]);
    Script {
        entry: 0,
        order: instructions.keys().copied().collect(),
        instructions,
    }
}

#[test]
fn test_object_callbacks_require_the_verified_message_native() {
    let script = interaction();
    assert_eq!(interaction_line(&script, &[0xFD1CF919], 3), Some(5));
    assert_eq!(interaction_line(&script, &[0x12345678], 3), None);
    assert_eq!(interaction_line(&script, &[0xFD1CF919], 4), None);
}

#[test]
fn test_conditional_or_non_message_callbacks_are_not_used_as_names() {
    let mut script = interaction();
    script.instructions.insert(31, (53, vec![24]));
    assert_eq!(interaction_line(&script, &[0xFD1CF919], 3), None);
    let mut script = interaction();
    script
        .instructions
        .insert(51, (script::PUSH_CONSTANT, vec![20]));
    assert_eq!(interaction_line(&script, &[0xFD1CF919], 3), None);
}

#[test]
fn test_short_rom_descriptions_preserve_accents_and_non_latin_text() {
    assert_eq!(
        label("É um console\nde videogame!"),
        Some("É um console de videogame!".into())
    );
    assert_eq!(label("ポケモン"), Some("ポケモン".into()));
    assert_eq!(label("\0damaged"), None);
    assert_eq!(label("\u{1}"), None);
    assert_eq!(label(" \n "), None);
    assert_eq!(label(&"x".repeat(181)), None);
}

#[test]
fn test_player_names_override_automatic_names_and_empty_names_restore_them() {
    use crate::core::names::Names;
    use std::rc::Rc;

    let names = Names::empty();
    names.use_game(
        Some(Rc::new(|code| (code == 119).then(|| "ROM species".into()))),
        Some(Rc::new(|zone, source| {
            (zone == 35 && source == 42).then(|| "ROM description".into())
        })),
    );
    names.name_sprite(119, "My companion").unwrap();
    names.name_place(35, 42, "My console").unwrap();
    assert_eq!(names.sprite(119).as_deref(), Some("My companion"));
    assert_eq!(names.place(35, 42).as_deref(), Some("My console"));
    names.name_sprite(119, "").unwrap();
    names.name_place(35, 42, "").unwrap();
    assert_eq!(names.sprite(119).as_deref(), Some("ROM species"));
    assert_eq!(names.place(35, 42).as_deref(), Some("ROM description"));
    assert_eq!(names.sprite(0), None);
    assert_eq!(names.place(36, 42), None);
}
