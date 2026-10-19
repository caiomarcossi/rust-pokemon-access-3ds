//! The Bag's party is found by its shape: six PokemonParam pointers, then how many are filled.

use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::screens::{distribution, partyicons};

const PARTY: i64 = 0x08CF7200;
const PARAMS: i64 = 0x08CF723C;

#[test]
fn test_the_party_is_six_params_and_a_count() {
    let memory = FakeMemory::new();
    let param = FakeClasses::new(memory.clone()).add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        memory.u32(PARAMS + 0x1E4 * i, &[param]);
        memory.u32(PARTY + 4 * i, &[PARAMS + 0x1E4 * i]);
    }
    memory.u32(PARTY + partyicons::COUNT, &[5]);
    let names = ClassNames::new(memory.clone(), CODE);
    // A word holding a param on its own is not the party; the list is.
    assert_eq!(
        partyicons::party_of([PARTY + 4, PARTY], &*memory, &names).unwrap(),
        Some(PARTY)
    );
    memory.u32(PARTY + partyicons::COUNT, &[7]);
    assert_eq!(
        partyicons::party_of([PARTY], &*memory, &names).unwrap(),
        None
    );
}

/// Another screen's icons, 21 textures, hold an empty texture at slot 16, as a picture drawing
/// nothing does; the party's list, found in an earlier Bag, now counts past six. Its seven words
/// hold no slot 16 (a player's Bag, twice, 2026-10-18).
#[test]
fn test_a_slot_past_six_is_no_party_member() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        memory.u32(PARAMS + 0x1E4 * i, &[param]);
        memory.u32(PARTY + 4 * i, &[PARAMS + 0x1E4 * i]);
    }
    memory.u32(PARTY + partyicons::COUNT, &[5]);
    let (icon, replace, table, picture) = (0x08647000, 0x08647100, 0x08647200, 0x08648000);
    memory.u32(icon, &[classes.add(distribution::ICON_CLASS)]);
    memory.u32(icon + distribution::REPLACE, &[replace]);
    memory.u32(replace, &[classes.add(distribution::REPLACE_CLASS)]);
    memory.u32(replace + distribution::TEXTURES, &[table, 21]);
    for i in 0..21 {
        memory.u32(table + 4 * i, &[0x08649000 + 0x10 * i]);
        memory.u32(
            0x08649000 + 0x10 * i + distribution::IMAGE,
            &[if i >= 16 { 0 } else { 0x1000 + i }],
        );
    }
    memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x200]);
    let mut icons = partyicons::PartyIcons::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        vec![String::new(), "Bulbasaur".to_string()],
        Rc::new(|| Some(partyicons::BAG.to_string())),
    );
    icons
        .track(&[PARAMS], [icon], &|_params: &[i64]| Ok(vec![PARTY]), false)
        .unwrap();
    memory.u32(PARTY + partyicons::COUNT, &[0x20]);
    assert_eq!(icons.name(picture).unwrap(), None);
}
