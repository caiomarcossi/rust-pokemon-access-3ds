//! Touch mode's walk over the bottom screen's targets (`games.common.touch`).
//!
//! What a touch area is called is measured against the game (spike 171); what is tested here is
//! the walk: reading order row by row, the keys' moves, and that a tap only ever reaches the target
//! pa3ds has just named.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::touch::{self, Options, Target, TouchTargets};

fn dexnav() -> Target {
    Target::new(1, "DexNav", 285, 53, 67.0, 30.0)
}

fn area() -> Target {
    Target::new(2, "Route 101", 125, 105, 15.0, 210.0)
}

fn bar() -> Vec<Target> {
    ["POKÉMON", "POKÉDEX", "BAG"]
        .iter()
        .enumerate()
        .map(|(i, name)| Target::new(10 + i as i64, name, 60 + 40 * i as i64, 223, -103.0, 30.0))
        .collect()
}

/// Touch targets with the screen's given, and every tap kept.
///
/// Python subclasses `TouchTargets` and overrides `targets`; here the targets are handed to it
/// through its test-only `given`, and the layouts it never reads are an empty memory's.
struct Screen {
    targets: TouchTargets,
    tapped: Rc<RefCell<Vec<(i64, i64)>>>,
    /// Every slide, as Python's test keeps them by setting `_slide`.
    slid: Rc<RefCell<Vec<(Vec<(i64, i64)>, f64)>>>,
    found: Rc<RefCell<Vec<Target>>>,
}

impl Screen {
    fn new(found: Vec<Target>) -> Self {
        let tapped: Rc<RefCell<Vec<(i64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
        let slid: Rc<RefCell<Vec<(Vec<(i64, i64)>, f64)>>> = Rc::new(RefCell::new(Vec::new()));
        let kept = slid.clone();
        let found = Rc::new(RefCell::new(found));
        let memory = FakeMemory::new();
        let layouts = Rc::new(Layouts::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
            lyt::ORAS,
        ));
        let keep = tapped.clone();
        let mut targets = TouchTargets::new(
            layouts,
            Rc::new(Vec::new),
            Rc::new(|_box| Ok(String::new())),
            Rc::new(|_pane| Ok(None)),
            Rc::new(|_root| None),
            Rc::new(|_line: &str| {}),
            Options {
                tap: Some(Rc::new(move |x, y| {
                    keep.borrow_mut().push((x, y));
                    None
                })),
                slide: Some(Rc::new(move |path: &[(i64, i64)], seconds| {
                    kept.borrow_mut().push((path.to_vec(), seconds));
                    None
                })),
                ..Options::default()
            },
        );
        let given = found.clone();
        targets.given = Some(Rc::new(move || {
            let found = given.borrow();
            touch::rows(&found)
                .into_iter()
                .flatten()
                .map(|i| found[i].clone())
                .collect()
        }));
        Screen {
            targets,
            tapped,
            slid,
            found,
        }
    }

    fn start(&mut self) -> String {
        self.targets.start().unwrap()
    }

    fn press(&mut self, action: &str) -> String {
        self.targets.press(action).unwrap()
    }

    fn tapped(&self) -> Vec<(i64, i64)> {
        self.tapped.borrow().clone()
    }
}

#[test]
fn test_targets_are_walked_row_by_row_from_the_top() {
    let mut screen = Screen::new([bar(), vec![area(), dexnav()]].concat());
    assert_eq!(screen.start(), "DexNav");
    assert_eq!(
        (0..5)
            .map(|_| screen.press(touch::ITEM_NEXT))
            .collect::<Vec<_>>(),
        ["Route 101", "POKÉMON", "POKÉDEX", "BAG", "DexNav"]
    );
    assert_eq!(screen.press(touch::ITEM_PREVIOUS), "BAG");
    assert_eq!(screen.press(touch::ROW_NEXT), "DexNav");
    assert_eq!(screen.press(touch::ROW_PREVIOUS), "POKÉMON");
    assert_eq!(screen.press(touch::ITEM_READ), "POKÉMON");
}

/// With nothing chosen, and once the chosen target has left the screen, the tap key only names
/// the first: the player has not heard what it would touch.
#[test]
fn test_a_tap_reaches_only_the_target_just_named() {
    let mut screen = Screen::new([vec![dexnav()], bar()].concat());
    assert!(screen.press(touch::TAP) == "DexNav" && screen.tapped().is_empty());
    assert!(screen.press(touch::TAP).is_empty() && screen.tapped() == [(285, 53)]);
    *screen.found.borrow_mut() = bar();
    assert!(screen.press(touch::TAP) == "POKÉMON" && screen.tapped() == [(285, 53)]);
}

#[test]
fn test_nothing_on_the_bottom_screen_is_said_so() {
    let mut screen = Screen::new(Vec::new());
    let started = screen.start();
    assert!(started == touch::NOTHING && touch::NOTHING == screen.press(touch::ITEM_NEXT));
}

/// The X menu (layer 10) over the PokéNav (layer 1): its POKÉDEX button covers the DexNav, which a
/// tap would never reach; the shortcut bar, in no menu button, stays (movie21).
#[test]
fn test_a_target_under_a_layout_drawn_over_it_is_left_out() {
    let pokedex = Target {
        layer: 10,
        ..Target::new(30, "POKÉDEX", 240, 60, 60.0, 40.0)
    };
    let dexnav = Target {
        layer: 1,
        ..Target::new(1, "DexNav", 285, 53, 67.0, 30.0)
    };
    let areas = [(10, 240, 60, 160.0, 40.0), (1, 285, 53, 30.0, 30.0)];
    assert_eq!(
        touch::uncovered(
            &[vec![pokedex.clone(), dexnav.clone()], bar()].concat(),
            &areas
        ),
        [vec![pokedex], bar()].concat()
    );
    assert_eq!(
        touch::uncovered(std::slice::from_ref(&dexnav), &[(1, 240, 60, 160.0, 40.0)]),
        [dexnav]
    );
}

/// `{a, b}` of words.
fn words(items: &[&str]) -> HashSet<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// The party list's plates each hold a corner where a held item is drawn, its words all the
/// plate's, and a tap there did nothing (movie22); a button inside another with words of its own
/// stays.
#[test]
fn test_a_touch_area_inside_another_is_part_of_its_button() {
    let (plate, corner, other) = (1, 2, 3);
    let placed = IndexMap::from([
        (plate, (80, 35, 160.0, 60.0)),
        (corner, (125, 52, 30.0, 20.0)),
        (other, (240, 45, 160.0, 60.0)),
    ]);
    let mut own = IndexMap::from([
        (plate, words(&["Blaze", "L", "14"])),
        (corner, words(&["L", "14"])),
        (other, words(&["Beldum"])),
    ]);
    let none = HashSet::new();
    assert!(touch::within_another(corner, &placed, &own, &none));
    assert!(!touch::within_another(plate, &placed, &own, &none));
    own.insert(corner, words(&["ITEM"]));
    assert!(!touch::within_another(corner, &placed, &own, &none));
}

/// The BuzzNav's NEXT lies within a touch area over its whole panel, hanging from the root, whose
/// words are all of the panel's, and only the panel was offered (movie24).
#[test]
fn test_a_button_inside_the_screens_own_touch_area_is_its_own() {
    let (panel, next) = (1, 2);
    let placed = IndexMap::from([
        (panel, (125, 105, 250.0, 210.0)),
        (next, (125, 140, 150.0, 30.0)),
    ]);
    let own = IndexMap::from([
        (panel, words(&["NEXT", "Welcome to BuzzNav!"])),
        (next, words(&["NEXT"])),
    ]);
    assert!(touch::within_another(next, &placed, &own, &HashSet::new()));
    assert!(!touch::within_another(
        next,
        &placed,
        &own,
        &HashSet::from([panel])
    ));
}

/// The PC Box under "Continue operations?" drew its grid as bright as ever, and touch mode
/// offered it with YES and NO (movie22); a layout at a higher layer stays.
#[test]
fn test_while_a_list_window_is_up_only_its_buttons_are_offered() {
    let (r#box, bar, question, above) = (0x10, 0x20, 0x30, 0x40);
    let placed = |pane, name: &str, x, y, row, layer, root| Target {
        layer,
        root,
        ..Target::new(pane, name, x, y, row, 30.0)
    };
    let slot = placed(1, "BOX 1, row 1, column 1, empty", 30, 65, 55.0, 1, r#box);
    let exit = placed(2, "exit", 255, 225, -105.0, 1, bar);
    let yes = placed(3, "YES", 275, 155, -35.0, 1, question);
    let no = placed(4, "NO", 275, 185, -65.0, 1, question);
    let over = placed(5, "zoom in", 20, 20, 100.0, 2, above);
    let found = vec![slot, exit, yes.clone(), no.clone(), over.clone()];
    assert_eq!(
        touch::answered_first(&found, &words(&["YES", "NO"])),
        [yes, no, over]
    );
    assert_eq!(touch::answered_first(&found, &HashSet::new()), found);
    assert_eq!(
        touch::answered_first(&found, &words(&["Buy", "Sell"])),
        found
    );
}

/// The PlayNav's pages and the sub-screen tutorial's arrows both draw `button.bclim`, a ◀
/// mirrored by its material on the right; the same texture lower down is a plate (spike A).
#[test]
fn test_a_bare_button_in_the_top_bars_corners_is_the_page_arrows() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let mut drawn: HashSet<i64> = HashSet::new();
    for (i, (x, y)) in [(-140.0f32, 105.0f32), (140.0, 105.0), (0.0, 0.0)]
        .into_iter()
        .enumerate()
    {
        let button = group + 0x1000 * (i as i64 + 1);
        fake.pane(button, group);
        fake.pane_with_vtable(button + 0x200, button, Some(bounding));
        fake.pane_with_vtable(button + 0x400, button, Some(picture));
        memory.write(button + 0x200 + lyt::X, &x.to_le_bytes());
        memory.write(button + 0x200 + lyt::Y, &y.to_le_bytes());
        memory.write(
            button + 0x200 + lyt::WIDTH,
            &[30.0f32.to_le_bytes(), 30.0f32.to_le_bytes()].concat(),
        );
        drawn.insert(button + 0x400);
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(move |pane| {
            Ok(if drawn.contains(&pane) {
                Some("button.bclim".to_string())
            } else {
                None
            })
        }),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options::default(),
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x, t.y))
            .collect::<Vec<_>>(),
        [
            ("previous", 20, 15),
            ("next", 300, 15),
            ("button", 160, 120)
        ]
    );
}

/// The Pokémon-Amie page's PLAY is a picture of its word inside the touch area behind its
/// menu, which closes it: the backdrop took PLAY's word, and PLAY was left out (spike A).
#[test]
fn test_a_button_the_game_names_inside_the_menus_backdrop_is_its_own() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let menu = group + 0x1000;
    let (backdrop, behind, play, area) = (menu + 0x200, menu + 0x400, menu + 0x600, menu + 0x800);
    fake.pane(menu, group);
    fake.pane_with_vtable(backdrop, menu, Some(bounding));
    fake.pane_with_vtable(behind, menu, Some(picture));
    fake.pane_with_vtable(play, menu, Some(picture));
    fake.pane_with_vtable(area, play, Some(bounding));
    memory.write(
        backdrop + lyt::WIDTH,
        &[320.0f32.to_le_bytes(), 180.0f32.to_le_bytes()].concat(),
    );
    for pane in [play, area] {
        memory.write(pane + lyt::Y, &(-61.0f32).to_le_bytes());
    }
    memory.write(
        area + lyt::WIDTH,
        &[90.0f32.to_le_bytes(), 54.0f32.to_le_bytes()].concat(),
    );
    fake.track().unwrap();
    let drawn: IndexMap<i64, String> = IndexMap::from([
        (behind, "msg_win_c.bclim".to_string()),
        (play, "kwg_button01.bclim".to_string()),
    ]);
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(move |pane| Ok(drawn.get(&pane).cloned())),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            images: Some(IndexMap::from([(
                "kwg_button01.bclim".to_string(),
                "PLAY".to_string(),
            )])),
            ..Options::default()
        },
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x, t.y))
            .collect::<Vec<_>>(),
        [("close", 160, 120), ("PLAY", 160, 181)]
    );
}

/// Pokémon-Amie's Pokémon is stroked, not tapped: its target's key slides along the path
/// (spike C).
#[test]
fn test_a_stroke_slides_along_its_path_where_a_tap_would_press() {
    let stroke = Target {
        path: vec![(110, 110), (210, 110)],
        seconds: 3.0,
        ..Target::new(1, "stroke Sceptile", 160, 120, 0.0, 240.0)
    };
    let mut screen = Screen::new(vec![stroke]);
    assert_eq!(screen.press(touch::TAP), "stroke Sceptile");
    assert_eq!(screen.press(touch::TAP), "");
    assert!(
        *screen.slid.borrow() == [(vec![(110, 110), (210, 110)], 3.0)]
            && screen.tapped().is_empty()
    );
}
