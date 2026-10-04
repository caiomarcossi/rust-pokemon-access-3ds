//! Navigation tests: the keys walking a fake map's list, and what each press says.

use std::cell::{Cell as Slot, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::Navigation;
use crate::core::keyfile;
use crate::core::keys::key_name;
use crate::core::names::Names;
use crate::core::pathfind::{self, Cell, nearest};
use crate::core::speech::{Backend, Category, Speech, default_interrupts};
use crate::games::common::field::{
    BLOCKED, CHUNK, CUT, Chunk, FURNITURE_SOURCE, Field, Ground, Interactable, Of, ROCK_SMASH,
    RotatingDoor, STRENGTH, SURF, WARP_SOURCE, place_source,
};
use crate::games::common::navigate::{
    CATEGORY_NEXT, CATEGORY_PREVIOUS, DEFAULT_BINDINGS, DIRECTION, EAST, ENEMY_SIDE, FieldSource,
    Guidance, ITEM_NEXT, ITEM_PREVIOUS, ITEM_READ, LIST, LeadsTo, MODE_TOGGLE, NORTH, Navigator,
    OUR_SIDE, Options, PATHFIND, PATHFIND_HM, RENAME, RENAME_SPEAKER, SOUTH, Sides, Street, WEST,
    bound_keys,
};
use crate::games::oras::{self, BIG_BOULDER, BUSH, CRACKED_BOULDER};
use crate::py::{self, Result};

/// Python's `Recorder`: what it was given to say, and whether each was said interrupting. The
/// shared `fake_speech::Recorder` keeps the text alone.
#[derive(Default)]
struct Recorder {
    spoken: RefCell<Vec<(String, bool)>>,
}

impl Backend for Recorder {
    fn speak(&self, text: &str, interrupt: bool) {
        self.spoken.borrow_mut().push((text.to_string(), interrupt));
    }

    fn braille(&self, _text: &str) {}
}

/// Petalburg Woods' ledges, jumped south in omega_ruby_ledges.ctm (spike 126).
const SOUTH_LEDGE: i64 = 0x75020021;
/// Jagged Pass's bumpy slopes, hopped up on the Acro Bike (omega_ruby_bike-jagged2.ctm,
/// spike 159).
const BUMPY: i64 = 0x6F000021;
/// Route 109's sea (omega_ruby_surf.ctm, spike 136).
const SEA: i64 = 0x3D1A0006;
/// Route 122's strip of shore, jumped over getting on the water and off (spike 162).
const SHORE: i64 = 0x40180023;
/// Jagged Pass's sandy slopes, ridden up north on the Mach Bike (omega_ruby_bike.ctm, spike 158).
const SOUTH_SLOPE: i64 = 0x6C000020;

/// What `FakeField.ground` is run through before it is handed over: Python's tests replace the
/// method itself, which Rust cannot do to a value behind a trait.
type Edit = Rc<dyn Fn(Ground) -> Ground>;

/// Stands in for the field reader: a fixed list, or none while off the map, over one loaded
/// chunk with `blocked` cells in it, `ledges` dropping south, `sea`, and `slopes` going down
/// south.
struct FakeField {
    r#where: RefCell<Option<Field>>,
    blocked: RefCell<HashSet<Cell>>,
    ledges: HashSet<Cell>,
    sea: HashSet<Cell>,
    slopes: HashSet<Cell>,
    /// Cells of the Acro Bike's bumpy slopes, each hopped up north.
    bumpy: RefCell<HashSet<Cell>>,
    shore: RefCell<HashSet<Cell>>,
    /// A floor's height -> the blocked cells its deck opens, walked at that height alone. A
    /// list, since a float is no key in Rust.
    decks: RefCell<Vec<(f64, HashSet<Cell>)>>,
    visit: Slot<Option<(i64, i64)>>,
    edit: RefCell<Option<Edit>>,
}

impl FakeField {
    fn new(
        r#where: Option<Field>,
        blocked: &[Cell],
        ledges: &[Cell],
        sea: &[Cell],
        slopes: &[Cell],
    ) -> Self {
        FakeField {
            r#where: RefCell::new(r#where),
            blocked: RefCell::new(blocked.iter().copied().collect()),
            ledges: ledges.iter().copied().collect(),
            sea: sea.iter().copied().collect(),
            slopes: slopes.iter().copied().collect(),
            bumpy: RefCell::new(HashSet::new()),
            shore: RefCell::new(HashSet::new()),
            decks: RefCell::new(Vec::new()),
            visit: Slot::new(Some((0x08D209B8, 0x6))),
            edit: RefCell::new(None),
        }
    }
}

impl FieldSource for FakeField {
    fn visit(&self) -> Option<(i64, i64)> {
        self.visit.get()
    }

    fn poll(&self) -> Result<Option<Field>> {
        let r#where = self.r#where.borrow();
        let Some(r#where) = r#where.as_ref() else {
            return Ok(None);
        };
        Ok(Some(Field {
            interactables: Vec::new(),
            ..r#where.clone()
        }))
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        let here = self.r#where.borrow();
        let here = here.as_ref().unwrap();
        Ok(Field {
            interactables: here.interactables.clone(),
            ..r#where.clone()
        })
    }

    fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground> {
        let origin = (80, 160);
        let cells: Vec<Cell> = (0..CHUNK * CHUNK)
            .map(|i| (origin.0 + i % CHUNK, origin.1 + i / CHUNK))
            .collect();
        let decks = self.decks.borrow();
        let opened: HashSet<Cell> = match height {
            Some(height) => decks
                .iter()
                .find(|(known, _)| *known == height)
                .map(|(_, opened)| opened.clone())
                .unwrap_or_default(),
            None => HashSet::new(),
        };
        let blocked = self.blocked.borrow();
        let bumpy = self.bumpy.borrow();
        let shore = self.shore.borrow();
        let words: Vec<i64> = cells
            .iter()
            .map(|cell| {
                if self.ledges.contains(cell) {
                    SOUTH_LEDGE
                } else if self.sea.contains(cell) {
                    SEA
                } else if self.slopes.contains(cell) {
                    SOUTH_SLOPE
                } else if bumpy.contains(cell) {
                    BUMPY
                } else if shore.contains(cell) {
                    SHORE
                } else if blocked.contains(cell) && !opened.contains(cell) {
                    BLOCKED
                } else {
                    0
                }
            })
            .collect();
        let mut ground = Ground::of(
            r#where,
            Of {
                chunks: vec![Rc::new(Chunk::new(origin, words))],
                clears: oras::game().clears.clone(),
                ..Of::default()
            },
        );
        ground.decks = decks
            .iter()
            .map(|(known, _)| *known)
            .filter(|known| Some(*known) != height)
            .collect();
        ground.hops = bumpy
            .iter()
            .map(|&cell| (cell, pathfind::NORTH))
            .collect::<IndexMap<_, _>>();
        let edit = self.edit.borrow().clone();
        Ok(match edit {
            Some(edit) => edit(ground),
            None => ground,
        })
    }
}

/// The town: a door and an edge to the north, a sign, and two people.
fn town() -> Field {
    Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: 0,
                ..Interactable::new("exits", "door", (94, 171))
            },
            Interactable {
                size: (40, 1),
                zone: Some(0x17),
                source: 0x17,
                ..Interactable::new("exits", "edge", (80, 159))
            },
            Interactable {
                source: 0,
                ..Interactable::new("signs", "sign", (97, 171))
            },
            Interactable {
                source: 0x08D1E9A8,
                ..Interactable::new("people", "person", (104, 179))
            },
            Interactable {
                source: 0x08D1F458,
                ..Interactable::new("people", "person", (90, 160))
            },
        ],
        ..Field::new(0x6, (94, 172))
    }
}

/// `navigator`'s arguments, each as the Python defaults it.
struct Want {
    r#where: Option<Field>,
    exits: bool,
    direction: bool,
    blocked: Vec<Cell>,
    ledges: Vec<Cell>,
    sea: Vec<Cell>,
    slopes: Vec<Cell>,
    names: Option<Rc<Names>>,
    leads_to: Option<LeadsTo>,
    street: Option<Street>,
    clock: Rc<dyn Fn() -> f64>,
    navigation: Option<Navigation>,
}

impl Default for Want {
    fn default() -> Self {
        Want {
            r#where: Some(town()),
            exits: false,
            direction: false,
            blocked: Vec::new(),
            ledges: Vec::new(),
            sea: Vec::new(),
            slopes: Vec::new(),
            names: None,
            leads_to: None,
            street: None,
            clock: Rc::new(|| 0.0),
            navigation: None,
        }
    }
}

/// A `Want` on the field the test names.
fn on(r#where: Field) -> Want {
    Want {
        r#where: Some(r#where),
        ..Want::default()
    }
}

/// What `navigator` hands back: the navigator, what it said, what it logged, and the field it
/// reads, which Python's tests reach through `nav._field`.
struct Rig {
    nav: Navigator,
    backend: Rc<Recorder>,
    log: Rc<RefCell<Vec<String>>>,
    field: Rc<FakeField>,
}

impl Rig {
    fn press(&mut self, action: &str) {
        self.nav.press(action).unwrap();
    }

    fn press_key(&mut self, key: &str) {
        self.nav.press_key(key).unwrap();
    }
}

/// A navigator on `where`, moved on to the exits category or into direction mode first if
/// asked, with what those presses said forgotten.
fn navigator(want: Want) -> Rig {
    let backend = Rc::new(Recorder::default());
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let field = Rc::new(FakeField::new(
        want.r#where,
        &want.blocked,
        &want.ledges,
        &want.sea,
        &want.slopes,
    ));
    let lines = log.clone();
    let nav = Navigator::new(
        field.clone(),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(move |line: &str| lines.borrow_mut().push(line.to_string())),
        Options {
            names: want.names,
            leads_to: want.leads_to,
            street: want.street,
            clock: Some(want.clock),
            navigation: want.navigation,
            moves: Some(IndexMap::from([
                (SURF, "Surf".to_string()),
                (CUT, "Cut".to_string()),
                (STRENGTH, "Strength".to_string()),
                (ROCK_SMASH, "Rock Smash".to_string()),
            ])),
            bike: Some("Mach Bike".to_string()),
            ..Options::default()
        },
    );
    let mut rig = Rig {
        nav,
        backend,
        log,
        field,
    };
    if want.exits {
        rig.press(CATEGORY_NEXT);
    }
    if want.direction {
        rig.press(MODE_TOGGLE);
    }
    rig.backend.spoken.borrow_mut().clear();
    rig.log.borrow_mut().clear();
    rig
}

fn said(backend: &Recorder) -> Vec<String> {
    backend
        .spoken
        .borrow()
        .iter()
        .map(|(text, _)| text.clone())
        .collect()
}

/// `said(backend)[-1]`.
fn last(backend: &Recorder) -> String {
    said(backend).pop().unwrap()
}

/// `said(backend)[-count:]`.
fn tail(backend: &Recorder, count: usize) -> Vec<String> {
    let said = said(backend);
    said[said.len() - count..].to_vec()
}

/// `said(backend)[start:]`.
fn from(backend: &Recorder, start: usize) -> Vec<String> {
    said(backend)[start..].to_vec()
}

/// `backend.spoken`, to compare with what was said and how.
fn heard(backend: &Recorder) -> Vec<(String, bool)> {
    backend.spoken.borrow().clone()
}

fn walk_to(rig: &Rig, cell: Cell) {
    let mut slot = rig.field.r#where.borrow_mut();
    let was = slot.as_ref().unwrap();
    *slot = Some(Field {
        interactables: was.interactables.clone(),
        ..Field::new(was.zone, cell)
    });
}

/// Walk the player through `cells`, the runtime following each step as it polls.
fn step(rig: &mut Rig, cells: &[Cell]) {
    for &cell in cells {
        walk_to(rig, cell);
        let r#where = rig.field.poll().unwrap().unwrap();
        rig.nav.follow(&r#where).unwrap();
    }
}

fn guidance(rig: &Rig) -> Option<Rc<RefCell<Guidance>>> {
    rig.nav.guidance.clone()
}

/// The cells of the way being walked.
fn path_of(way: &Option<Rc<RefCell<Guidance>>>) -> Vec<Cell> {
    way.as_ref().unwrap().borrow().path.clone()
}

fn cells(xs: std::ops::Range<i64>, z: i64) -> Vec<Cell> {
    xs.map(|x| (x, z)).collect()
}

#[test]
fn test_both_modes_start_on_everything_nearest_first() {
    let mut rig = navigator(Want::default());
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        ["door, 1 of 5, 1 north", "sign, 2 of 5, 3 east 1 north"]
    );
}

/// Orion, 2026-09-30: "Nurse, 2 west 1 north, 1 of 9", and a setting for each.
#[test]
fn test_the_place_in_the_list_may_follow_the_way_or_be_left_out() {
    let mut after = navigator(Want {
        navigation: Some(Navigation {
            position_first: false,
            ..Navigation::default()
        }),
        ..Want::default()
    });
    after.press(ITEM_READ);
    let mut without = navigator(Want {
        navigation: Some(Navigation {
            positions: false,
            ..Navigation::default()
        }),
        ..Want::default()
    });
    without.press(ITEM_READ);
    let mut both = said(&after.backend);
    both.extend(said(&without.backend));
    assert_eq!(both, ["door, 1 north, 1 of 5", "door, 1 north"]);
}

#[test]
fn test_a_category_is_said_with_how_many_things_are_in_it() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    assert_eq!(said(&rig.backend), ["exits, 2", "signs, 1"]);
}

#[test]
fn test_empty_categories_are_skipped_either_way() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_PREVIOUS);
    assert_eq!(said(&rig.backend), ["people, 2"]);
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    assert_eq!(tail(&rig.backend, 2), ["all, 5", "exits, 2"]);
}

#[test]
fn test_items_are_said_with_their_place_nearest_first_and_wrap() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_NEXT);
    rig.press(ITEM_NEXT);
    rig.press(ITEM_PREVIOUS);
    assert_eq!(
        said(&rig.backend),
        [
            "edge, 2 of 2, 13 north",
            "door, 1 of 2, 1 north",
            "edge, 2 of 2, 13 north"
        ]
    );
}

#[test]
fn test_reading_the_item_again_uses_the_bearing_from_where_the_player_stands() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    walk_to(&rig, (94, 180));
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        ["door, 1 of 2, 1 north", "door, 1 of 2, 9 north"]
    );
}

#[test]
fn test_with_nothing_on_the_map_every_key_says_so() {
    let here = town();
    let nothing = Field::new(here.zone, here.cell);
    let mut rig = navigator(on(nothing));
    rig.press(CATEGORY_NEXT);
    rig.press(ITEM_NEXT);
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        ["nothing here", "nothing here", "nothing north"]
    );
}

#[test]
fn test_off_the_map_there_is_no_map() {
    let mut rig = navigator(Want {
        r#where: None,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(NORTH);
    assert_eq!(said(&rig.backend), ["no map", "no map"]);
    assert_eq!(*rig.log.borrow(), ["key: no map", "key: no map"]);
}

#[test]
fn test_a_key_answer_interrupts() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    assert_eq!(
        heard(&rig.backend),
        [("door, 1 of 2, 1 north".to_string(), true)]
    );
    // `Speech(backend)._interrupts` is private to Speech here: a new Speech takes these.
    assert!(default_interrupts().contains(&Category::Key));
}

/// K tracked one person and then, a few steps on, another (the user, 2026-09-21).
#[test]
fn test_the_chosen_person_stays_chosen_as_walking_reorders_the_list() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_NEXT);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 10 east 7 south");
    // Walking towards them makes them the nearest; the list re-sorts, the choice does not move.
    walk_to(&rig, (104, 176));
    rig.press(ITEM_READ);
    assert_eq!(last(&rig.backend), "person, 1 of 2, 3 south");
    rig.press(ITEM_NEXT);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 14 west 16 north");
}

#[test]
fn test_the_selection_is_kept_within_a_list_that_shrank() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_NEXT);
    let here = town();
    walk_to(&rig, here.cell);
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    });
    rig.press(ITEM_READ);
    assert_eq!(last(&rig.backend), "door, 1 of 1, 1 north");
}

// Where an exit leads.

/// The town's place and Route 101's, as the profile has them from the zone table and the
/// ROM: the edge leads onto the route, the door into one of the town's own houses.
fn route_101() -> LeadsTo {
    Rc::new(|here: i64, there: i64| {
        assert_eq!(here, town().zone);
        Ok(if there == 0x17 {
            Some("Route 101".to_string())
        } else {
            None
        })
    })
}

/// The user, 2026-09-24: nothing said that the gap north of Littleroot leads to Route 101.
/// The door into a house of the town leads nowhere new, and says nothing more.
#[test]
fn test_an_exit_into_another_place_says_the_place() {
    let mut rig = navigator(Want {
        exits: true,
        leads_to: Some(route_101()),
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "door, 1 of 2, 1 north",
            "edge to Route 101, 2 of 2, 13 north"
        ]
    );
}

/// The place is the game's own word, so a name of the player's comes before it rather than
/// instead of it; a name that already says it is left alone.
#[test]
fn test_a_named_exit_keeps_the_place_after_its_name() {
    let names = Rc::new(Names::default());
    names
        .name_place(town().zone, 0x17, "the gap in the trees")
        .unwrap();
    let mut rig = navigator(Want {
        exits: true,
        names: Some(names.clone()),
        leads_to: Some(route_101()),
        ..Want::default()
    });
    rig.press(ITEM_PREVIOUS);
    names
        .name_place(town().zone, 0x17, "the way to Route 101")
        .unwrap();
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        [
            "the gap in the trees to Route 101, 2 of 2, 13 north",
            "the way to Route 101, 2 of 2, 13 north"
        ]
    );
}

/// Spike 107: Lumiose's zones are all "Lumiose City", so the place says nothing of where a
/// warp between two of them comes out; the street it comes out on does.
#[test]
fn test_an_exit_within_one_place_says_the_street_it_comes_out_on() {
    let mut rig = navigator(Want {
        exits: true,
        street: Some(Rc::new(|_zone: i64, source: i64| {
            Ok(if source == 0 {
                Some("Vernal Avenue".to_string())
            } else {
                None
            })
        })),
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "door to Vernal Avenue, 1 of 2, 1 north",
            "edge, 2 of 2, 13 north"
        ]
    );
}

#[test]
fn test_without_the_games_words_an_exit_is_what_it_is() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_PREVIOUS);
    assert_eq!(said(&rig.backend), ["edge, 2 of 2, 13 north"]);
}

// Direction mode.

/// The user's example (2026-09-21): M into direction mode, then O for north.
#[test]
fn test_standing_south_of_a_door_o_says_door_1_north() {
    let mut rig = navigator(Want::default());
    rig.press_key("'");
    rig.press_key("o");
    assert_eq!(
        said(&rig.backend),
        ["direction mode", "door, 1 of 2, 1 north"]
    );
    assert_eq!(rig.nav.mode(), DIRECTION);
}

#[test]
fn test_a_direction_cycles_its_side_nearest_first_and_wraps() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    for _ in 0..3 {
        rig.press(NORTH);
    }
    assert_eq!(
        said(&rig.backend),
        [
            "door, 1 of 2, 1 north",
            "sign, 2 of 2, 3 east 1 north",
            "door, 1 of 2, 1 north"
        ]
    );
}

/// The edge 13 north and the person 12 north are beyond it (the user, 2026-09-21: 10).
#[test]
fn test_a_direction_looks_ten_cells_each_way_and_no_further() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    walk_to(&rig, (94, 169));
    rig.press(NORTH);
    rig.press(NORTH);
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        [
            "edge, 1 of 2, 10 north",
            "person, 2 of 2, 4 west 9 north",
            "edge, 1 of 2, 10 north"
        ]
    );
}

#[test]
fn test_a_thing_north_east_is_heard_under_north_and_under_east() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(EAST);
    rig.press(EAST);
    rig.press(SOUTH);
    rig.press(WEST);
    assert_eq!(
        said(&rig.backend),
        [
            "sign, 1 of 2, 3 east 1 north",
            "person, 2 of 2, 10 east 7 south",
            "person, 1 of 1, 10 east 7 south",
            "nothing west"
        ]
    );
}

#[test]
fn test_another_direction_starts_again_from_its_nearest() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(NORTH);
    rig.press(NORTH);
    rig.press(EAST);
    rig.press(NORTH);
    assert_eq!(
        tail(&rig.backend, 2),
        ["sign, 1 of 2, 3 east 1 north", "door, 1 of 2, 1 north"]
    );
}

#[test]
fn test_a_direction_with_nothing_that_way_says_so_by_category() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    rig.press(SOUTH);
    rig.press(EAST);
    let here = town();
    let only_door = Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    };
    let mut rig2 = navigator(Want {
        direction: true,
        ..on(only_door)
    });
    rig2.press(SOUTH);
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "signs, 1",
            "no signs south",
            "sign, 1 of 1, 3 east 1 north"
        ]
    );
    assert_eq!(said(&rig2.backend), ["nothing south"]);
}

#[test]
fn test_walking_keeps_the_chosen_and_steps_on_from_it() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(NORTH);
    rig.press(NORTH);
    assert_eq!(last(&rig.backend), "sign, 2 of 2, 3 east 1 north");
    // Beside the sign, it is the nearest thing north; the next press steps on past it.
    walk_to(&rig, (97, 172));
    rig.press(NORTH);
    assert_eq!(last(&rig.backend), "door, 2 of 2, 3 west 1 north");
}

#[test]
fn test_the_category_keys_work_in_both_modes_and_forget_the_direction() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press_key("]");
    rig.press(NORTH);
    rig.press_key("[");
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "door, 1 of 1, 1 north",
            "all, 5",
            "door, 1 of 2, 1 north"
        ]
    );
}

#[test]
fn test_the_toggle_switches_mode_and_a_key_means_what_its_mode_says() {
    let mut rig = navigator(Want::default());
    rig.press_key(";");
    rig.press_key("o");
    rig.press_key("'");
    rig.press_key("o");
    rig.press_key("'");
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "direction mode",
            "door, 1 of 1, 1 north",
            "list mode"
        ]
    );
    assert_eq!(rig.nav.mode(), LIST);
}

/// The user's ask (2026-09-22): the shaking-tail Pokémon under a category of its own.
#[test]
fn test_a_hidden_pokemon_is_its_own_category_after_items() {
    let grass = Field {
        interactables: vec![
            Interactable {
                code: 0x105,
                source: 0x08D374E4,
                ..Interactable::new("pokémon", "hidden Pokémon", (94, 129))
            },
            Interactable {
                source: 0x08D1F458,
                ..Interactable::new("people", "person", (99, 129))
            },
        ],
        ..Field::new(0x17, (99, 130))
    };
    let mut rig = navigator(on(grass));
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_READ);
    rig.press(CATEGORY_PREVIOUS);
    assert_eq!(
        said(&rig.backend),
        [
            "pokémon, 1",
            "hidden Pokémon, 1 of 1, 5 west 1 north",
            "people, 1"
        ]
    );
    rig.press(MODE_TOGGLE);
    rig.press(CATEGORY_NEXT);
    rig.press(WEST);
    rig.press(SOUTH);
    assert_eq!(
        tail(&rig.backend, 3),
        [
            "pokémon, 1",
            "hidden Pokémon, 1 of 1, 5 west 1 north",
            "no pokémon south"
        ]
    );
}

// The way there.

/// A house's front wall, its door at (95, 172) blocked in it as spike 82 found doors.
fn house_front() -> Vec<Cell> {
    cells(88..101, 172)
}

fn street() -> Field {
    Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: place_source(WARP_SOURCE, 0).unwrap(),
                ..Interactable::new("exits", "door", (95, 172))
            },
            Interactable {
                source: 0x08D1E9A8,
                move_model: true,
                ..Interactable::new("people", "person", (100, 180))
            },
        ],
        ..Field::new(0x6, (90, 175))
    }
}

/// The user's example (2026-09-24): "5 east, 2 north", and at the end of the 5 east, north.
/// The door is solid, so the way ends in front of it, and says it there.
#[test]
fn test_p_says_the_way_leg_by_leg_and_the_next_leg_at_each_turn() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press_key("p");
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175), (94, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(95, 175)]);
    assert_eq!(last(&rig.backend), "2 north");
    step(&mut rig, &[(95, 174), (95, 173)]);
    assert_eq!(from(&rig.backend, 1), ["2 north", "door, 1 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_without_turn_by_turn_the_way_is_said_once_and_p_says_it_again() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        navigation: Some(Navigation {
            turn_by_turn: false,
            ..Navigation::default()
        }),
        ..on(street())
    });
    rig.press_key("p");
    assert!(!rig.nav.guiding());
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    rig.press_key("p");
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
}

#[test]
fn test_a_turn_is_said_without_interrupting_and_logged_as_the_path() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    assert_eq!(
        heard(&rig.backend),
        [
            ("5 east, 2 north".to_string(), true),
            ("2 north".to_string(), false)
        ]
    );
    assert_eq!(*rig.log.borrow(), ["key: 5 east, 2 north", "path: 2 north"]);
}

#[test]
fn test_p_goes_to_whatever_the_navigation_keys_are_on() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        direction: true,
        ..on(street())
    });
    rig.press(SOUTH);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["person, 1 of 1, 10 east 5 south", "10 east, 4 south"]
    );
}

#[test]
fn test_a_step_the_wrong_way_says_the_new_way_from_there() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(89, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "6 east, 2 north"]);
}

/// North first where the path went east first heard the whole way again at every step
/// (poke_center_pc.ctm, 2026-09-24): going on the way the player walks is taken quietly.
#[test]
fn test_a_way_of_the_players_own_as_short_is_followed_and_its_turn_said() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175), (93, 174)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(93, 173)]);
    assert_eq!(last(&rig.backend), "2 east");
    step(&mut rig, &[(94, 173), (95, 173)]);
    assert_eq!(from(&rig.backend, 1), ["2 east", "door, 1 north"]);
}

#[test]
fn test_a_wall_in_a_chunk_loaded_since_finds_the_path_again() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    rig.field
        .blocked
        .borrow_mut()
        .extend((92..96).map(|x| (x, 174)));
    step(&mut rig, &[(91, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north, 4 east"]);
}

/// Issue 3: a ledge was a wall, walked round. The player's cell goes from before the ledge
/// to after it in one move (omega_ruby_ledges.ctm), which is still on the way.
#[test]
fn test_a_ledge_on_the_way_is_jumped_and_the_jump_followed() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ledges: cells(80..120, 177),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["people, 1", "10 east, 4 south"]);
    let way = guidance(&rig);
    let mut walked = cells(91..101, 175);
    walked.extend([(100, 176), (100, 178)]);
    step(&mut rig, &walked);
    assert!(
        Rc::ptr_eq(way.as_ref().unwrap(), rig.nav.guidance.as_ref().unwrap()),
        "the way was found again"
    );
    step(&mut rig, &[(100, 179)]);
    assert_eq!(from(&rig.backend, 2), ["4 south", "person, 1 south"]);
}

fn sea_band() -> Vec<Cell> {
    (80..120)
        .flat_map(|x| [177, 178].into_iter().map(move |z| (x, z)))
        .collect()
}

/// Route 109: P walked the player into the sea, where the blocked bit is clear
/// (omega_ruby_surf.ctm, the user, 2026-09-28). P goes round the water; Shift and P crosses it,
/// and says the move as the leg on the water begins.
#[test]
fn test_p_keeps_to_land_and_shift_p_says_the_leg_to_surf_when_the_player_reaches_it() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea_band(),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.iter().any(|cell| sea_band().contains(cell)));
    assert!(!last(&rig.backend).contains("Surf"));
    rig.press_key("shift+p");
    assert_eq!(
        last(&rig.backend),
        "10 east, 1 south, Surf 2 south, 1 south"
    );
    let mut walked = cells(91..101, 175);
    walked.push((100, 176));
    step(&mut rig, &walked);
    assert_eq!(tail(&rig.backend, 2), ["1 south", "Surf 2 south"]);
    step(&mut rig, &[(100, 177), (100, 178), (100, 179)]);
    assert_eq!(tail(&rig.backend, 2), ["1 south", "person, 1 south"]);
}

#[test]
fn test_p_from_the_water_crosses_it_without_saying_so() {
    let here = street();
    let water = Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 177))
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea_band(),
        ..on(water)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "2 south");
}

/// Route 122: the shore along the water is blocked, and Surf jumps the player over it, from
/// (22, 8) to (22, 10) and off from (54, 58) to (54, 56); P walled the water off at both
/// (omega_ruby_surf3.ctm, spike 162). Shift and P jumps it both ways, saying the
/// jump with the side it lands on, and P from the water lands.
#[test]
fn test_shift_p_jumps_the_shore_onto_the_water_and_off_it() {
    let here = street();
    let across = Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 175))
    };
    let sea: Vec<Cell> = cells(80..120, 177);
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea.clone(),
        ..on(across)
    });
    let shore: HashSet<Cell> = (80..120)
        .flat_map(|x| [176, 178].into_iter().map(move |z| (x, z)))
        .collect();
    *rig.field.shore.borrow_mut() = shore.clone();
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(
        !way.iter()
            .any(|cell| sea.contains(cell) || shore.contains(cell))
    );
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "Surf 2 south, 2 south");
    step(&mut rig, &[(100, 177)]);
    assert_eq!(last(&rig.backend), "2 south");
    step(&mut rig, &[(100, 179)]);
    assert_eq!(last(&rig.backend), "person, 1 south");
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 177))
    });
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "2 south");
}

/// A sandy slope going down south, the one way up through a cliff across the chunk at z 170.
fn slope() -> Vec<Cell> {
    (168..173).map(|z| (95, z)).collect()
}

fn cliff() -> Vec<Cell> {
    let mut out: Vec<Cell> = (80..120).filter(|&x| x != 95).map(|x| (x, 170)).collect();
    for x in [94, 96] {
        out.extend((168..173).map(|z| (x, z)));
    }
    out
}

/// Jagged Pass: the Hiker up the slope from the player (omega_ruby_bike.ctm).
fn pass() -> Field {
    Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (95, 160))
        }],
        ..Field::new(0x54, (95, 175))
    }
}

/// Jagged Pass: P to the Hiker up the sandy slope (the user, 2026-10-03). On foot P goes round
/// it; on the bike it rides straight up; Shift and P on foot says the leg up it with the bike, as
/// the player reaches the slope.
#[test]
fn test_p_rides_up_a_slope_only_on_the_bike_and_shift_p_says_the_bike_on_foot() {
    let mut rig = navigator(Want {
        blocked: cliff(),
        slopes: slope(),
        ..on(pass())
    });
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.iter().any(|cell| slope().contains(cell)));
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "2 north, Mach Bike 5 north, 7 north");
    step(&mut rig, &[(95, 174), (95, 173)]);
    assert_eq!(last(&rig.backend), "Mach Bike 5 north");
    *rig.field.r#where.borrow_mut() = Some(Field {
        on_bike: true,
        ..pass()
    });
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "14 north");
}

/// Jagged Pass: the player hopped north up the bumpy slope at (49, 30) on the Acro Bike, from
/// (49, 31) to (49, 29) (the user, 2026-10-03). On foot P goes round it; Shift and P hops it.
#[test]
fn test_shift_p_hops_up_a_bumpy_slope_and_says_the_bike() {
    let mut rig = navigator(Want {
        blocked: cliff(),
        ..on(pass())
    });
    *rig.field.bumpy.borrow_mut() = HashSet::from([(95, 170)]);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.contains(&(95, 170)));
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "4 north, bumpy slope 2 north, 8 north");
}

/// A wall across the chunk, its one gap at (95, 177) with a bush or a rock in it.
fn hedge() -> Vec<Cell> {
    (80..120).filter(|&x| x != 95).map(|x| (x, 177)).collect()
}

/// Route 104's bush stands in the way to an item ball (omega_ruby_bush.ctm), Rusturf
/// Tunnel's rock in the way on (omega_ruby_smash.ctm, spike 137). P walks round either as round
/// anything standing; Shift and P clears it, and says the move as the player reaches it. One
/// chosen itself is walked up to, with HMs or not.
#[test]
fn test_p_goes_round_a_bush_or_a_rock_and_shift_p_clears_it() {
    for (sprite, r#move) in [(BUSH, "Cut"), (CRACKED_BOULDER, "Rock Smash")] {
        let here = street();
        let mut interactables = here.interactables.clone();
        interactables.push(Interactable {
            code: sprite,
            source: 0x08D1F458,
            move_model: true,
            ..Interactable::new("signs", "object", (95, 177))
        });
        let blocked = Field {
            interactables,
            ..Field::new(here.zone, here.cell)
        };
        let mut walls = house_front();
        walls.extend(hedge());
        let mut rig = navigator(Want {
            blocked: walls,
            ..on(blocked)
        });
        rig.press(CATEGORY_PREVIOUS);
        rig.press(PATHFIND);
        let way = path_of(&guidance(&rig));
        assert!(!way.contains(&(95, 177)), "{move}");
        rig.press_key("shift+p");
        assert_eq!(
            last(&rig.backend),
            format!("5 east, 1 south, {move} 1 south, 3 south, 4 east"),
            "{move}"
        );
        let mut walked = cells(91..96, 175);
        walked.push((95, 176));
        step(&mut rig, &walked);
        assert_eq!(
            tail(&rig.backend, 2),
            ["1 south".to_string(), format!("{move} 1 south")],
            "{move}"
        );
        rig.press(CATEGORY_NEXT);
        rig.press(ITEM_READ);
        rig.press_key("shift+p");
        assert_eq!(last(&rig.backend), "object, 1 south", "{move}");
    }
}

/// Fiery Path's boulder fills a path two cells wide (omega_ruby_strength-boulders.ctm): P
/// walks round it, and Shift and P goes through both its rows, said with Strength.
#[test]
fn test_p_goes_round_a_big_boulder_and_shift_p_pushes_through_all_of_it() {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        size: (2, 2),
        code: BIG_BOULDER,
        source: 0x08D1F458,
        move_model: true,
        ..Interactable::new("signs", "object", (95, 177))
    });
    let blocked = Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    };
    let hedge: Vec<Cell> = (80..120)
        .filter(|x| ![95, 96].contains(x))
        .map(|x| (x, 177))
        .collect();
    let mut walls = house_front();
    walls.extend(hedge);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(blocked)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(
        !way.iter()
            .any(|cell| [(95, 177), (96, 177), (95, 178), (96, 178)].contains(cell))
    );
    rig.press_key("shift+p");
    assert!(last(&rig.backend).contains("Strength 2 south"));
}

/// Three of Rusturf Tunnel's rocks have a sign in their cell and a script of their own, and
/// stand across the tunnel between its halves: they are the rocks Rock Smash breaks to join
/// them, and Shift and P found no way to Verdanturf Town while they were walls (2026-09-29).
#[test]
fn test_a_rock_with_a_sign_under_it_is_smashed_all_the_same() {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        code: CRACKED_BOULDER,
        source: 0x08D1F458,
        move_model: true,
        ..Interactable::new("signs", "object", (95, 177))
    });
    interactables.push(Interactable {
        source: place_source(FURNITURE_SOURCE, 0).unwrap(),
        ..Interactable::new("signs", "sign", (95, 177))
    });
    let scripted = Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    };
    let mut walls = house_front();
    walls.extend(hedge());
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(scripted)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press_key("shift+p");
    assert_eq!(
        last(&rig.backend),
        "5 east, 1 south, Rock Smash 1 south, 3 south, 4 east"
    );
}

/// The game puts the player in a door's own cell before the warp, and the Pokémon Center's
/// door said a path to the Poké Mart a moment before it (poke_center_pc.ctm, 2026-09-24).
#[test]
fn test_a_door_walked_into_on_the_way_is_left_to_its_warp() {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        zone: Some(0xE1),
        source: place_source(WARP_SOURCE, 1).unwrap(),
        ..Interactable::new("exits", "door", (99, 172))
    });
    let street = Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street)
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(99, 172)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    assert!(rig.nav.guiding());
    rig.field.visit.set(Some((0x08D1FF08, 0xE1)));
    step(&mut rig, &[(4, 9)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_a_warp_ends_the_path_quietly_and_a_field_rebuilt_in_place_does_not() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    rig.field.visit.set(Some((0x08D1FF08, 0x6)));
    step(&mut rig, &[(90, 175)]);
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
    rig.field.visit.set(Some((0x08D1FF08, 0xDF)));
    step(&mut rig, &[(16, 18)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_beside_the_thing_p_says_where_it_is() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    walk_to(&rig, (95, 173));
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["door, 1 north"]);
    assert!(!rig.nav.guiding());
}

/// The user, 2026-10-03: "find a path to the closest it can be before saying can't
/// continue", where P on Route 119 said "no path to edge" and nothing more.
#[test]
fn test_with_no_way_there_p_goes_as_near_as_it_can_and_says_it_can_t_continue() {
    let ring: Vec<Cell> = (98..103)
        .flat_map(|x| (178..183).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 2)
        .collect();
    let mut walls = house_front();
    walls.extend(ring);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["people, 1", "4 south, 7 east, then can't continue"]
    );
    let mut walked: Vec<Cell> = (176..180).map(|z| (90, z)).collect();
    walked.extend((91..98).map(|x| (x, 179)));
    step(&mut rig, &walked);
    assert_eq!(
        from(&rig.backend, 2),
        ["7 east", "can't continue, person, 3 east 1 south"]
    );
    assert!(!rig.nav.guiding());
}

/// Spike 159: from the foot of Route 119 the way north crosses a bridge high over the path,
/// and its decks walked at the player's height cut it. The way found at the bridge's height is
/// followed on that ground: at the player's own the bridge is a wall, and every step would
/// find the way again.
#[test]
fn test_where_no_way_is_found_at_the_players_height_the_decks_are_walked_at_another() {
    let wall: Vec<Cell> = (93..108)
        .flat_map(|x| (173..188).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 7)
        .collect();
    let mut walls = house_front();
    walls.extend(wall);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    *rig.field.decks.borrow_mut() = vec![(99.0, HashSet::from([(93, 177)]))];
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["people, 1", "2 south, 10 east, 2 south"]
    );
    step(&mut rig, &[(90, 176), (90, 177), (91, 177), (91, 178)]);
    assert_eq!(
        from(&rig.backend, 2),
        ["10 east", "1 north, 9 east, 2 south"]
    );
    let way = guidance(&rig);
    assert!(way.is_some() && way.unwrap().borrow().height == Some(99.0));
}

/// Spike 160: Fortree City's invisible Kecleon stands in the one way to the Gym, and the way as
/// near as the player could get as the crow flies ended behind a wall from it, on a walkway 7
/// cells south of the door. The way goes up to whoever stands in it instead, and says them.
#[test]
fn test_a_person_in_the_one_way_is_walked_up_to_and_said() {
    let ring: Vec<Cell> = (98..103)
        .flat_map(|x| (178..183).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 2 && (x, z) != (98, 180))
        .collect();
    let guard = Interactable {
        source: 0x08D1F000,
        move_model: true,
        ..Interactable::new("people", "person", (98, 180))
    };
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(guard);
    let street = Field {
        interactables,
        ..here
    };
    let mut walls = house_front();
    walls.extend(ring);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_NEXT);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "5 south, 7 east, then can't continue, person in the way"
    );
}

/// Spike 161: Fortree Gym's doors stand across its ways, and are walked through by walking into
/// an arm, which turns the door. A wall with one gap, a door's arm standing across it.
#[test]
fn test_p_walks_through_a_rotating_door_and_says_the_push() {
    let wall: Vec<Cell> = (80..120).filter(|&x| x != 94).map(|x| (x, 177)).collect();
    let mut walls = house_front();
    walls.extend(wall);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    let door = RotatingDoor::new((95, 178), IndexMap::from([(pathfind::WEST, 1)]));
    *rig.field.edit.borrow_mut() = Some(Rc::new(move |ground: Ground| Ground {
        doors: vec![door.clone()],
        ..ground
    }));
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert!(last(&rig.backend).contains("push 1 south"));
}

#[test]
fn test_with_nowhere_nearer_p_says_there_is_no_path() {
    let mut walls = house_front();
    walls.extend([(89, 175), (91, 175), (90, 174), (90, 176)]);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["people, 1", "no path to person"]);
    assert!(!rig.nav.guiding());
}

/// omega_ruby_movil.ctm (spike 142): P again halfway up Route 110 found no new way, said "no
/// path to path", and left nothing to follow. The way being walked stays, and P says what is
/// left of it.
#[test]
fn test_p_again_that_finds_no_way_keeps_the_way_being_walked_and_says_the_rest() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175)]);
    let held = guidance(&rig).unwrap();
    // Python makes `pathfind.find` answer None. `find` is called by name here, so the door's
    // approaches are made cells a way only goes round (`stepped`, which `open` ignores): the door
    // has no goal left to walk to, and `find` answers None for want of one, with every cell of
    // the way still open for `follow`.
    *rig.field.edit.borrow_mut() = Some(Rc::new(|mut ground: Ground| {
        ground.stepped.extend([(95, 173), (95, 171)]);
        ground
    }));
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 east, 2 north"]);
    assert!(rig.nav.guiding());
    // Not in the Python: with the real `find` P again finds the same way anew and says the same
    // words, so only the way kept tells the branch the test is for from the one beside it.
    assert!(
        Rc::ptr_eq(&held, rig.nav.guidance.as_ref().unwrap()),
        "the way being walked is kept"
    );
    step(&mut rig, &[(94, 175), (95, 175)]);
    assert_eq!(last(&rig.backend), "2 north");
}

#[test]
fn test_a_thing_gone_while_the_player_was_off_the_path_is_said_gone() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let here = street();
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    });
    step(&mut rig, &[(90, 174)]);
    assert_eq!(last(&rig.backend), "person gone");
    assert!(!rig.nav.guiding());
}

/// `DEFAULT_BINDINGS[mode][key]`.
fn bound(mode: &str, key: &str) -> &'static str {
    // The bindings live as long as the program; a `LazyLock` hands out `&'static` to them.
    let keys = &DEFAULT_BINDINGS[mode];
    keys[key].as_str()
}

#[test]
fn test_the_users_keys_map_to_the_actions() {
    assert_eq!(
        "iojlk"
            .chars()
            .map(|key| bound(LIST, &key.to_string()))
            .collect::<Vec<_>>(),
        [
            CATEGORY_PREVIOUS,
            CATEGORY_NEXT,
            ITEM_PREVIOUS,
            ITEM_NEXT,
            ITEM_READ
        ]
    );
    assert_eq!(
        "o;lk"
            .chars()
            .map(|key| bound(DIRECTION, &key.to_string()))
            .collect::<Vec<_>>(),
        [NORTH, EAST, SOUTH, WEST]
    );
    for mode in [LIST, DIRECTION] {
        assert_eq!(
            "[]'p"
                .chars()
                .map(|key| bound(mode, &key.to_string()))
                .collect::<Vec<_>>(),
            [CATEGORY_PREVIOUS, CATEGORY_NEXT, MODE_TOGGLE, PATHFIND],
            "{mode}"
        );
    }
    let mut expected: HashSet<String> = "iojlk[];'pr".chars().map(|key| key.to_string()).collect();
    expected.extend(["shift+k", "shift+j", "shift+p", "shift+r"].map(str::to_string));
    assert_eq!(bound_keys(&DEFAULT_BINDINGS), expected);
    for mode in [LIST, DIRECTION] {
        assert_eq!(bound(mode, "shift+k"), RENAME, "{mode}");
        assert_eq!(bound(mode, "shift+j"), RENAME_SPEAKER, "{mode}");
        assert_eq!(bound(mode, "shift+p"), PATHFIND_HM, "{mode}");
        assert_eq!(bound(mode, "r"), OUR_SIDE, "{mode}");
        assert_eq!(bound(mode, "shift+r"), ENEMY_SIDE, "{mode}");
    }
}

/// M was the mode toggle for a day, and M is this machine's Start (`pa3ds.core.keyfile`): a key
/// pa3ds takes is a button the game never gets. The C-stick, the debug button and GPIO 14 are
/// the ones ORAS does not use, so those are the only overlaps the defaults are allowed: P, the
/// way there, is GPIO 14 here.
#[test]
fn test_no_key_of_the_default_bindings_is_a_button_the_player_presses() {
    // `read_text` reads a file with universal newlines.
    let config = include_str!("fake_qt_config.ini")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let taken = keyfile::taken(&DEFAULT_BINDINGS, &keyfile::emulator_keys(&config).unwrap());
    let controls: HashSet<String> = taken.into_iter().map(|(control, _, _)| control).collect();
    assert_eq!(
        controls,
        HashSet::from([
            "the C-stick".to_string(),
            "the debug button".to_string(),
            "the GPIO 14 button".to_string()
        ])
    );
}

#[test]
fn test_virtual_keys_are_named_as_their_letters_and_us_punctuation() {
    assert!(key_name(0x4B) == "k" && key_name(0x31) == "1");
    assert_eq!(
        [0xBA, 0xDB, 0xDD].map(key_name),
        [";".to_string(), "[".to_string(), "]".to_string()]
    );
    assert_eq!(key_name(0x70), "vk0x70");
}

/// Spike 102: on Lumiose's boulevards the chunks mark every cell a wall, so a way cannot be
/// found; the thing and its bearing still can, and the player hears why there is no more.
#[test]
fn test_p_on_a_rail_says_where_the_thing_is_and_that_no_route_is_known() {
    let here = street();
    let rail = Field {
        on_rail: true,
        ..here
    };
    let mut rig = navigator(on(rail));
    rig.press_key("p");
    assert_eq!(
        last(&rig.backend),
        "door, 5 east 3 north, no route on these streets"
    );
    assert!(!rig.nav.guiding());
}

/// Spike 105: on Lumiose's boulevards up walks the way the camera looks and a held left or
/// right turns it, so P says the holds and the steps, and the list says where things are from
/// the way the camera looks.
#[test]
fn test_on_a_rail_the_way_and_the_bearing_are_said_from_the_camera() {
    let rail = Field {
        heading: Some("east"),
        bearing: Some(92.0),
        ..street()
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(rail)
    });
    rig.press_key("p");
    assert_eq!(last(&rig.backend), "5 forward, hold left, 2 forward");
    rig.press(ITEM_PREVIOUS);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 10 ahead 5 right");
}

/// A tap of left or right leaves the camera between two ways, and up walks the player across
/// the street at its angle (spike 105).
#[test]
fn test_on_a_rail_a_camera_left_at_an_angle_is_straightened_first() {
    let rail = Field {
        heading: Some("east"),
        bearing: Some(60.0),
        ..street()
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(rail)
    });
    rig.press_key("p");
    assert_eq!(
        last(&rig.backend),
        "camera angled, hold left, hold right, 5 forward, hold left, 2 forward"
    );
}

/// A hold turns the camera and a tap leaves it partway: once it has stopped, the player
/// standing still hears the way from where it looks now, and nothing while it still turns
/// (spike 105).
#[test]
fn test_the_way_is_said_again_once_the_camera_stops_somewhere_new() {
    let now = Rc::new(Slot::new(0.0));
    let rail = Field {
        heading: Some("east"),
        bearing: Some(90.0),
        ..street()
    };
    let clock = now.clone();
    let mut rig = navigator(Want {
        blocked: house_front(),
        clock: Rc::new(move || clock.get()),
        ..on(rail.clone())
    });
    rig.press_key("p");

    /// The camera at `bearing` for `seconds`, the runtime polling every twentieth.
    fn camera(rig: &mut Rig, now: &Slot<f64>, rail: &Field, bearing: f64, seconds: f64) {
        *rig.field.r#where.borrow_mut() = Some(Field {
            heading: Some(nearest(bearing).unwrap()),
            bearing: Some(bearing),
            ..rail.clone()
        });
        for _ in 0..py::round(seconds * 20.0).unwrap() {
            now.set(now.get() + 0.05);
            let r#where = rig.field.poll().unwrap().unwrap();
            rig.nav.follow(&r#where).unwrap();
        }
    }

    camera(&mut rig, &now, &rail, 89.0, 1.0);
    assert_eq!(said(&rig.backend), ["5 forward, hold left, 2 forward"]);
    for bearing in [60.0, 30.0, 5.0] {
        camera(&mut rig, &now, &rail, bearing, 0.1);
    }
    assert_eq!(said(&rig.backend).len(), 1);
    camera(&mut rig, &now, &rail, 1.0, 0.5);
    assert_eq!(
        last(&rig.backend),
        "hold right, 5 forward, hold left, 2 forward"
    );
    camera(&mut rig, &now, &rail, 65.0, 0.5);
    assert_eq!(
        last(&rig.backend),
        "camera angled, hold left, hold right, 5 forward, hold left, 2 forward"
    );
    assert_eq!(said(&rig.backend).len(), 3);
}

/// For when the line was missed, and while the HP box is off the screen (ADR-0015; the
/// user's keys, 2026-09-29). A key's answer interrupts.
#[test]
fn test_r_says_our_side_again_and_shift_r_the_enemys() {
    let backend = Rc::new(Recorder::default());
    let sides: Sides = Rc::new(|ours: bool| {
        Some(
            if ours {
                "Blaze♂ L10 20/30"
            } else {
                "Zigzagoon♂ L2 50%"
            }
            .to_string(),
        )
    });
    let mut nav = Navigator::new(
        Rc::new(FakeField::new(Some(town()), &[], &[], &[], &[])),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_line: &str| {}),
        Options {
            sides: Some(sides),
            ..Options::default()
        },
    );
    nav.press(OUR_SIDE).unwrap();
    nav.press(ENEMY_SIDE).unwrap();
    assert_eq!(
        heard(&backend),
        [
            ("Blaze♂ L10 20/30".to_string(), true),
            ("Zigzagoon♂ L2 50%".to_string(), true)
        ]
    );
}

/// A press is never silent: the player cannot tell a key that did nothing from one that was
/// not heard.
#[test]
fn test_the_sides_keys_say_no_battle_out_of_one() {
    let mut rig = navigator(Want {
        r#where: None,
        ..Want::default()
    });
    rig.press(OUR_SIDE);
    rig.press(ENEMY_SIDE);
    assert_eq!(said(&rig.backend), ["no battle", "no battle"]);
}
