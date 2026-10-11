//! The Pokémon-Amie page's Pokémon is named by its species, through the icon it is drawn with.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::lyt;
use crate::games::gen6::screens::amie;
use crate::games::gen6::screens::distribution as d;

const PAGE: i64 = 0x08A00000;
const HOLDER: i64 = 0x08A01000;
const ICON: i64 = 0x08A02000;
const REPLACE: i64 = 0x08A03000;
const TABLE: i64 = 0x08A04000;
const TEXTURE: i64 = 0x08A05000;
const WALKER: i64 = 0x08A06000;
const OTHER: i64 = 0x08A07000;
const IMAGE: i64 = 0x18200000;

fn species() -> Vec<String> {
    let mut names = vec![String::new(); 254];
    names.push("Sceptile".to_string());
    names
}

fn page() -> (amie::AmiePage, Rc<FakeMemory>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PAGE, &[classes.add(amie::CLASS)]);
    memory.u32(PAGE + amie::NUMBER, &[254]);
    memory.u32(PAGE + amie::HOLDER, &[HOLDER]);
    memory.u32(HOLDER, &[ICON]);
    memory.u32(ICON, &[classes.add(d::ICON_CLASS)]);
    memory.u32(ICON + d::REPLACE, &[REPLACE]);
    memory.u32(REPLACE, &[classes.add(d::REPLACE_CLASS)]);
    memory.u32(REPLACE + d::TEXTURES, &[TABLE, 1]);
    memory.u32(TABLE, &[TEXTURE]);
    memory.u32(TEXTURE + d::IMAGE, &[IMAGE]);
    for (picture, image) in [(WALKER, IMAGE), (OTHER, IMAGE + 0x1000)] {
        let material = picture + 0x400;
        memory.u32(picture + lyt::PICTURE_MATERIAL, &[material]);
        memory.u32(material + lyt::MATERIAL_TEXTURE, &[image]);
    }
    let mut found = amie::AmiePage::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species(),
    );
    found
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    (found, memory)
}

#[test]
fn test_the_picture_drawing_the_icons_first_texture_is_named_by_its_species() {
    assert_eq!(page().0.name(WALKER).unwrap().as_deref(), Some("Sceptile"));
}

#[test]
fn test_a_picture_drawing_another_image_is_not_named() {
    assert_eq!(page().0.name(OTHER).unwrap(), None);
}

#[test]
fn test_amie_itself_offers_its_pokemon_to_stroke_by_the_species_the_page_showed() {
    let (mut found, memory) = page();
    memory.u32(PAGE, &[0]); // the page freed as Amie runs
    assert!(
        found
            .strokes(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
    let strokes = found.strokes(Some(amie::AMIE)).unwrap();
    let [stroke] = strokes.as_slice() else {
        panic!("one stroke: {strokes:?}");
    };
    assert_eq!(
        (stroke.name.as_str(), stroke.path.clone(), stroke.seconds),
        (
            "stroke Sceptile",
            amie::STROKE.to_vec(),
            amie::STROKE_SECONDS
        )
    );
}
