//! Touch mode's walk over the bottom screen's targets (`games.common.touch`).
//!
//! What a touch area is called is measured against the game (spike 171); what is tested here is
//! the walk: reading order row by row, the keys' moves, and that a tap only ever reaches the target
//! pa3ds has just named.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeMemory};
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
    found: Rc<RefCell<Vec<Target>>>,
}

impl Screen {
    fn new(found: Vec<Target>) -> Self {
        let tapped: Rc<RefCell<Vec<(i64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
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
