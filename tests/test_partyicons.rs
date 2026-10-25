//! The Bag's party is found by its shape: six PokemonParam pointers, then how many are filled.

use std::cell::RefCell;
use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_pokemon::{WURMPLE, second_wurmple};
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
    // A byte: the Battle Maison sets the one after it (or_multi_battle_partners).
    memory.u32(PARTY + partyicons::COUNT, &[0x206]);
    assert_eq!(
        partyicons::party_of([PARTY], &*memory, &names).unwrap(),
        Some(PARTY)
    );
}

/// Choosing the Battle Box or the party, the top screen draws both from twelve textures, the
/// Battle Box's six first: "L 73 L 71" said no species (or_multi_battle_partners).
#[test]
fn test_the_maisons_group_screen_draws_the_party_after_the_battle_box() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        let at = PARAMS + 0x1E4 * i;
        memory.u32(at, &[param, at + pokemon::RECORD_IN_PARAM]);
        memory.u32(PARTY + 4 * i, &[at]);
    }
    memory.write(PARAMS + 0x1E4 + pokemon::RECORD_IN_PARAM, &second_wurmple());
    memory.u32(PARTY + partyicons::COUNT, &[0x206]);
    let (icon, replace, table, picture) = (0x08647000, 0x08647100, 0x08647200, 0x08648000);
    memory.u32(icon, &[classes.add(distribution::ICON_CLASS)]);
    memory.u32(icon + distribution::REPLACE, &[replace]);
    memory.u32(replace, &[classes.add(distribution::REPLACE_CLASS)]);
    memory.u32(replace + distribution::TEXTURES, &[table, 12]);
    for i in 0..12 {
        memory.u32(table + 4 * i, &[0x08649000 + 0x10 * i]);
        memory.u32(
            0x08649000 + 0x10 * i + distribution::IMAGE,
            &[if i < 6 { 0 } else { 0x1000 + i }],
        );
    }
    memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x200]);
    memory.u32(picture + 0x200 + lyt::MATERIAL_TEXTURE, &[0x1007]);
    let running = Rc::new(RefCell::new(partyicons::GROUPS.to_string()));
    let mut species = vec![String::new(); WURMPLE as usize];
    species.push("Wurmple".to_string());
    let mut icons = partyicons::PartyIcons::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species,
        {
            let running = running.clone();
            Rc::new(move || Some(running.borrow().clone()))
        },
    );
    icons
        .track(&[PARAMS], [icon], &|_params: &[i64]| Ok(vec![PARTY]), false)
        .unwrap();
    assert_eq!(icons.name(picture).unwrap().as_deref(), Some("Wurmple"));
    // In the Bag the seventh texture is no party member's.
    *running.borrow_mut() = partyicons::BAG.to_string();
    icons
        .track(&[PARAMS], [icon], &|_params: &[i64]| Ok(vec![PARTY]), false)
        .unwrap();
    assert_eq!(icons.name(picture).unwrap(), None);
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
