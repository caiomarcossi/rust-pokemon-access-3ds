//! Navigation: the keys that walk the list of what is on the map, in one of two modes.
//!
//! **List mode** walks the whole map. Two keys step between categories and two between a
//! category's items, nearest first; a fifth says the chosen item again. **Direction mode** looks
//! one way at a time: four keys, one a compass point, each cycling through the category's items on
//! that side of the player within `RANGE` cells each way (a half-box: north is everything with a
//! smaller z, so a thing north-east is heard under north and under east both), nearest first. A
//! press of another direction starts again from the nearest that way. The apostrophe toggles the
//! mode (the user, 2026-09-21; moved off M on 2026-09-22, which is the emulator's Start). The
//! category is one setting under both modes, and "all" is a category of its own, the one both
//! start on.
//!
//! Each press reads the map afresh, so the bearing is from where the player stands now and a
//! person is where they have walked to. The chosen item is kept by what it is, not by its place in
//! the list: walking re-sorts the list, and keeping a place meant K tracked one person and then, a
//! few steps on, another (the user, 2026-09-21). An item that has gone — a ball picked up — gives
//! way to the nearest.
//!
//! Spoken as "exits, 3" on reaching a category and "door, 1 of 3, 1 north" on an item (the user,
//! 2026-09-21: the count on a category, the place on an item), the place counted within the
//! direction in direction mode. A category with nothing in it is skipped over, "nothing here"
//! when the map is empty, "nothing north" or "no exits north" when a direction is, and "no map"
//! off the field. An exit into another place says it, in the game's own words out of the ROM:
//! "edge to Route 101, 2 of 3, 5 north" (the user, 2026-09-24). A door into a town's own house
//! leads nowhere new, as the game has it, and says nothing more; zone numbers stay in the log.
//!
//! Each thing is said by the name it has been given, and by what it is when it has none: "the
//! Mart's sign, 1 of 3, 1 north" rather than "sign, 1 of 3, 1 north". A name is the player's or
//! the shipped table's (`pa3ds.core.names`), and **one key renames whatever is chosen**, through
//! a dialog — the player is the one standing in front of the thing, and nobody else is in a
//! position to name it.
//!
//! Keys come in as names and are looked up in the mode's bindings here, since one key means one
//! thing in list mode and another in direction mode: O steps the category in one and looks north
//! in the other. A key bound in either mode is kept from the emulator in both.
//!
//! **P finds the way to whatever is chosen**, in either mode (the user, 2026-09-24), and says it
//! as its legs: "5 east, 2 north". Then it **guides**: as the player walks the path, the next leg
//! is said the moment they reach each turn ("2 north"), and the thing itself when they get there
//! ("door, 1 north"). A player who steps off the path is given a new one from where they stand,
//! said when it turns them and taken quietly when it goes on the way they are walking; a warp ends
//! the guiding, since the path's cells belong to the map the player has left.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::config::Navigation;
use crate::core::dialog::Asker;
use crate::core::names::Names;
use crate::core::pathfind::{self, Cell, Leg};
use crate::core::speech::{Category, Speech};
use crate::games::common::field::{
    self, ALL, CUT, Field, Ground, Interactable, Of, ROCK_SMASH, SPRITE, STRENGTH, SURF,
};
use crate::py::{self, Error, Log, Result};

/// `(ALL, *field.CATEGORIES)`.
pub const CATEGORIES: [&str; 9] = {
    let mut out = [ALL; 9];
    let mut at = 0;
    while at < field::CATEGORIES.len() {
        out[at + 1] = field::CATEGORIES[at];
        at += 1;
    }
    out
};

pub const LIST: &str = "list";
pub const DIRECTION: &str = "direction";

pub const CATEGORY_PREVIOUS: &str = "category previous";
pub const CATEGORY_NEXT: &str = "category next";
pub const ITEM_PREVIOUS: &str = "item previous";
pub const ITEM_NEXT: &str = "item next";
pub const ITEM_READ: &str = "item read";
pub const NORTH: &str = "north";
pub const EAST: &str = "east";
pub const SOUTH: &str = "south";
pub const WEST: &str = "west";
pub const DIRECTIONS: [&str; 4] = [NORTH, EAST, SOUTH, WEST];
pub const MODE_TOGGLE: &str = "mode";
/// Name whatever is chosen. One key for both name spaces, because which one a thing belongs to is
/// the thing's business and not the player's (`Interactable.naming`).
pub const RENAME: &str = "rename";

/// Name whoever last spoke. pa3ds's own, not Pokemon Access's, because pa3ds knows who is
/// speaking and Pokemon Access does not — and because the moment a player knows who somebody is
/// is the moment they have just heard them, not the moment they find them in a list.
pub const RENAME_SPEAKER: &str = "rename speaker";

/// Find the way on foot to whatever is chosen, say it, and guide the player along it.
pub const PATHFIND: &str = "pathfind";

/// The same, over water and through bushes, rocks and boulders too, each leg on the water or
/// through one said with the move that crosses it: "Surf 12 south", "Cut 1 east", "Rock Smash 1
/// east", "Strength 2 west". P alone keeps to land and goes round a bush, a rock or a boulder,
/// since a way across water is no way to a player who cannot surf yet (the user, 2026-09-28).
/// Up a sandy slope too, said with the bike that rides up it, "Mach Bike 6 north", where the
/// player is not riding it already; P goes up one only on the bike (the user, 2026-10-03). Up a
/// bumpy slope as well, the Acro Bike's, said with it, "Acro Bike 2 north", by P too on the bike,
/// since the player's model is the same on either (spike 159).
pub const PATHFIND_HM: &str = "pathfind with HMs";

pub const OUR_SIDE: &str = "our side";
/// Say a side of the battle again, as it was last said: "Blaze♂ L10 20/30", "Zigzagoon♂ L2 50%".
/// For when the line was missed, and while the HP box is off the screen, in the move list or the
/// Bag (ADR-0015). "no battle" outside one.
pub const ENEMY_SIDE: &str = "enemy side";

/// What the keys file says above an action's key, where its name does not say it all. The
/// horde's order is Orion's, heard by fainting the bottom left Zigzagoon and the fifth going to 0%
/// (movie16, 2026-10-01): the enemy side is said by battle position (`gen6.battle`).
pub const EXPLAINED: [(&str, &str); 1] = [(
    ENEMY_SIDE,
    "The enemy's Pokémon are said in the order the battle places them. In a horde of five that is \
     round a half circle from the bottom right: bottom right, top right, top middle, top left, \
     bottom left.",
)];

/// What a step into one of Fortree Gym's rotating doors is said with, which turns it a quarter as
/// the player walks on: "push 1 west" (spike 161). The game names no such thing.
pub const PUSH: &str = "push";

/// What a leg is said with where the ROM's move names cannot be read: pa3ds's own word for where
/// the leg goes.
pub const OWN_MOVE_WORDS: [(i64, &str); 4] = [
    (SURF, "water"),
    (CUT, "bush"),
    (STRENGTH, "boulder"),
    (ROCK_SMASH, "rock"),
];
/// What a leg up a sandy slope is said with where the ROM's name for the Mach Bike cannot be
/// read.
pub const OWN_SLOPE_WORD: &str = "slope";
/// What a hop up a bumpy slope is said with where the ROM's name for the Acro Bike cannot be
/// read.
pub const OWN_HOP_WORD: &str = "bumpy slope";

/// What a dialog's answer is for, as `dialog.Answer` carries it.
pub const RENAMING: &str = "interactable";
pub const SPEAKING: &str = "speaker";

/// Mode -> key -> action. Keys are single characters, as `pa3ds.core.keys.key_name` gives them.
pub type Bindings = IndexMap<String, IndexMap<String, String>>;

fn keys(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, action)| (key.to_string(), action.to_string()))
        .collect()
}

/// The user's choice of keys: I, O, J, L and K for the list (2026-09-21); O, semicolon, L and K
/// laid out as a compass for the directions; the brackets for the category, and the apostrophe for
/// the mode, beside the semicolon that is already east; P for the way there in both (2026-09-24),
/// Pokemon Access's mnemonic. The mode was M until 2026-09-22, when `pa3ds.core.keyfile` reported
/// what M costs: it is the emulator's Start button, and a key pa3ds takes is a button the player
/// has lost.
///
/// Shift and K renames what K reads, as Pokemon Access does it, and shift and J renames whoever
/// last spoke. Shift and P is the way there with HMs (2026-09-28). R says the player's side of a
/// battle again and shift and R the enemy's (2026-09-29). All of these are the player's to change
/// in `<pa3ds>\user\keys.toml`.
pub static DEFAULT_BINDINGS: LazyLock<Bindings> = LazyLock::new(|| {
    IndexMap::from([
        (
            LIST.to_string(),
            keys(&[
                ("i", CATEGORY_PREVIOUS),
                ("o", CATEGORY_NEXT),
                ("j", ITEM_PREVIOUS),
                ("l", ITEM_NEXT),
                ("k", ITEM_READ),
                ("[", CATEGORY_PREVIOUS),
                ("]", CATEGORY_NEXT),
                ("'", MODE_TOGGLE),
                ("shift+k", RENAME),
                ("shift+j", RENAME_SPEAKER),
                ("p", PATHFIND),
                ("shift+p", PATHFIND_HM),
                ("r", OUR_SIDE),
                ("shift+r", ENEMY_SIDE),
            ]),
        ),
        (
            DIRECTION.to_string(),
            keys(&[
                ("o", NORTH),
                (";", EAST),
                ("l", SOUTH),
                ("k", WEST),
                ("[", CATEGORY_PREVIOUS),
                ("]", CATEGORY_NEXT),
                ("'", MODE_TOGGLE),
                ("shift+k", RENAME),
                ("shift+j", RENAME_SPEAKER),
                ("p", PATHFIND),
                ("shift+p", PATHFIND_HM),
                ("r", OUR_SIDE),
                ("shift+r", ENEMY_SIDE),
            ]),
        ),
    ])
});

/// Degrees the camera may be turned off a way of Lumiose's streets and still be said to look
/// along it. A tap of left or right leaves it 6 to 70 degrees round, and up then walks the player
/// across the street at that angle; a hold settles it within a degree of the way the player was
/// turned to. The corridor's frame is the line's where the player stands, which on a curving line
/// is up to 15 degrees from the ways the game turns the player to, so less would call a settled
/// camera angled (spike 105).
pub const ANGLED: f64 = 20.0;
/// Degrees the camera may drift and still be the same: walking a street eases it by a few.
pub const STILL: f64 = 0.5;
/// Seconds the camera must stay still to have stopped turning. A hold turns it about 150 degrees
/// a second and eases it to a stop over a fifth of a second (spike 105).
pub const SETTLE: f64 = 0.3;
/// Degrees the camera must have turned since the way was last said for it to be said again.
pub const MOVED: f64 = 3.0;

/// Every key bound in any mode: what the hook keeps from the emulator.
pub fn bound_keys(bindings: &Bindings) -> HashSet<String> {
    bindings
        .values()
        .flat_map(|keys| keys.keys().cloned())
        .collect()
}

/// How far direction mode looks, in cells each way: a box, so a thing 10 north and 10 east is
/// in. The top screen shows about 13 cells across and 9 deep (measured from a rig screenshot,
/// 2026-09-21), so this is a little over a screen sideways and two screens ahead; the user's
/// choice, so the keys do not cycle the whole map.
pub const RANGE: i64 = 10;

/// Whether `cell` is on `direction`'s side of `origin`, no more than `within` cells away
/// either way.
pub fn towards(direction: &str, origin: (i64, i64), cell: (i64, i64), within: i64) -> bool {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    if dx.abs() > within || dz.abs() > within {
        return false;
    }
    if direction == NORTH {
        return dz < 0;
    }
    if direction == SOUTH {
        return dz > 0;
    }
    if direction == EAST {
        return dx > 0;
    }
    dx < 0
}

/// What an item is kept by: `Interactable::key`.
pub type Key = (String, String, i64, Option<i64>);

/// A path being walked, and how far along it the player has got.
///
/// Shared between the navigator and the call that is holding it (`Navigator::guide`'s `held`), as
/// one object in Python, so it is an `Rc<RefCell<Guidance>>` where it is passed about.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Guidance {
    /// What the path leads to, by which it is found again when the path has to be.
    pub key: Key,
    pub name: String,
    /// The field's `visit` when the path was found: a warp changes it.
    pub visit: Option<(i64, i64)>,
    pub path: Vec<Cell>,
    pub legs: Vec<Leg>,
    /// Each cell of the path and how far along it is.
    pub along: HashMap<Cell, i64>,
    /// Where the player was last seen.
    pub cell: Cell,
    /// The leg last said: the whole path is said first, so the first leg is said already.
    /// Python's default is 0.
    pub leg: i64,
    /// On a rail, where the camera looked when the way was last said (`Field.bearing`). Python's
    /// default is None.
    pub camera: Option<f64>,
    /// Where it looked when it last moved, and `since` when that was: it has stopped turning
    /// once it has stayed there `SETTLE` seconds. Python's default is None.
    pub seen: Option<f64>,
    /// Python's default is 0.0.
    pub since: f64,
    /// Whether the way was found with HMs, as it is found again whenever it has to be. Python's
    /// default is False.
    pub hm: bool,
    /// The height its decks were walked at (`Ground.decks`), or None at the player's own. Python's
    /// default is None.
    pub height: Option<f64>,
    /// Whether it ends short of the thing, as near as the player can get, where they hear that
    /// they can't continue. Python's default is False.
    pub short: bool,
    /// Who stands in the way where it ends short, said after "can't continue": ", Team Magma
    /// Grunt in the way"; "" where nobody does. Python's default is "".
    pub blocker: String,
}

/// What the navigator reads the map through: ORAS's `field.FieldReader`, or Sun's reader.
pub trait FieldSource {
    /// `visit`, a property in Python.
    fn visit(&self) -> Option<(i64, i64)>;

    fn poll(&self) -> Result<Option<Field>>;

    fn interactables(&self, r#where: &Field) -> Result<Field>;

    /// The ground under `where`, its decks walked at `height`, or at the player's own.
    fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground>;
}

/// `Callable[[int, int], str | None]`: what the game calls the place an exit from one zone into
/// another arrives in. It reads the game's words, so it can raise.
pub type LeadsTo = Rc<dyn Fn(i64, i64) -> Result<Option<String>>>;
/// `Callable[[int, int], str | None]`: the street an exit comes out on.
pub type Street = Rc<dyn Fn(i64, i64) -> Result<Option<String>>>;
/// `Callable[[], int | None]`: the character that last spoke.
pub type Speaker = Rc<dyn Fn() -> Option<i64>>;
/// `Callable[[bool], str | None]`: the player's side of the battle as last said, or the enemy's.
pub type Sides = Rc<dyn Fn(bool) -> Option<String>>;
/// `Callable[[], float]`: `time.monotonic` unless a test or a tape says otherwise.
pub type Clock = Rc<dyn Fn() -> f64>;

/// `Navigator.__init__`'s arguments after `field`, `speech` and `log`, each as the Python
/// defaults it: set what differs with `..Options::default()`.
pub struct Options {
    /// None is `DEFAULT_BINDINGS`.
    pub bindings: Option<Bindings>,
    pub names: Option<Rc<Names>>,
    pub asker: Option<Asker>,
    pub navigation: Option<Navigation>,
    pub speaker: Option<Speaker>,
    pub leads_to: Option<LeadsTo>,
    pub street: Option<Street>,
    /// None is `py::time::monotonic`.
    pub clock: Option<Clock>,
    pub routes: bool,
    pub slack: i64,
    pub moves: Option<IndexMap<i64, String>>,
    pub bike: Option<String>,
    pub acro: Option<String>,
    pub sides: Option<Sides>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            bindings: None,
            names: None,
            asker: None,
            navigation: None,
            speaker: None,
            leads_to: None,
            street: None,
            clock: None,
            routes: true,
            slack: 0,
            moves: None,
            bike: None,
            acro: None,
            sides: None,
        }
    }
}

/// Answers the navigation keys from the field reader's list.
pub struct Navigator {
    pub(crate) field: Rc<dyn FieldSource>,
    /// How an item and the way to it are said (`config.toml`'s `[pa3ds.navigation]`).
    wording: Navigation,
    /// What a leg on water, through a bush or through a rock is said with, by the move's
    /// number: the game's name for the move, out of the ROM's move names, or `OWN_MOVE_WORDS`
    /// when there is no ROM to read it from.
    moves: IndexMap<i64, String>,
    /// What a leg up a sandy slope is said with: the game's name for the Mach Bike.
    bike: String,
    /// What a hop up a bumpy slope is said with: the game's name for the Acro Bike, said on
    /// the bike too, since nothing read says which of the two the player is riding.
    acro: String,
    /// Whether the field's walls are read, so P can find a way. Sun keeps its walls as
    /// meshes pa3ds does not read, and a way found without them would lead through them.
    routes: bool,
    /// Cells either way a player may be off the path and still be walking it. None in ORAS,
    /// where every step lands in a cell of the grid. Sun's player walks anywhere, a body a cell
    /// and a half across, and drifts a cell off a straight leg as a matter of course: counted
    /// as leaving the path, every such step found a new one and said it, several times a second
    /// (sun_6.ctm, 2026-09-26).
    slack: i64,
    clock: Clock,
    speech: Rc<Speech>,
    log: Log,
    bindings: Bindings,
    names: Rc<Names>,
    pub(crate) asker: Asker,
    speaker: Speaker,
    /// The player's side of the battle as last said, or the enemy's, or None out of one.
    sides: Sides,
    /// What the game calls the place an exit from one zone into another arrives in, or None
    /// where it arrives in the player's own place, as every door of a town does.
    leads_to: LeadsTo,
    /// The street an exit from a zone, by its source, comes out on in the player's own
    /// place, as Lumiose's zones all are one.
    street: Street,
    /// What the open dialog is renaming and what it was called, since the player may have
    /// walked on by the time they press OK.
    renaming: Option<((&'static str, i64, i64), String)>,
    mode: &'static str,
    category: i64,
    /// The key of the item chosen, or None for the nearest.
    chosen: Option<Key>,
    /// The direction the chosen item was found in, so the same key steps on from it and
    /// another starts afresh.
    facing: Option<String>,
    pub(crate) guidance: Option<Rc<RefCell<Guidance>>>,
}

impl Navigator {
    pub fn new(field: Rc<dyn FieldSource>, speech: Rc<Speech>, log: Log, options: Options) -> Self {
        let Options {
            bindings,
            names,
            asker,
            navigation,
            speaker,
            leads_to,
            street,
            clock,
            routes,
            slack,
            moves,
            bike,
            acro,
            sides,
        } = options;
        let mut own: IndexMap<i64, String> = OWN_MOVE_WORDS
            .iter()
            .map(|&(clear, word)| (clear, word.to_string()))
            .collect();
        own.extend(moves.unwrap_or_default());
        Navigator {
            field,
            wording: navigation.unwrap_or_default(),
            moves: own,
            bike: match bike {
                Some(bike) if !bike.is_empty() => bike,
                _ => OWN_SLOPE_WORD.to_string(),
            },
            acro: match acro {
                Some(acro) if !acro.is_empty() => acro,
                _ => OWN_HOP_WORD.to_string(),
            },
            routes,
            slack,
            clock: clock.unwrap_or_else(|| Rc::new(py::time::monotonic)),
            speech,
            log,
            bindings: bindings.unwrap_or_else(|| DEFAULT_BINDINGS.clone()),
            names: names.unwrap_or_else(|| Rc::new(Names::default())),
            asker: asker.unwrap_or_default(),
            speaker: speaker.unwrap_or_else(|| Rc::new(|| None)),
            sides: sides.unwrap_or_else(|| Rc::new(|_player| None)),
            leads_to: leads_to.unwrap_or_else(|| Rc::new(|_here, _there| Ok(None))),
            street: street.unwrap_or_else(|| Rc::new(|_zone, _slot| Ok(None))),
            renaming: None,
            mode: LIST,
            category: 0,
            chosen: None,
            facing: None,
            guidance: None,
        }
    }

    pub fn mode(&self) -> &str {
        self.mode
    }

    /// Whether a path is being walked, and the player's every step wants following.
    pub fn guiding(&self) -> bool {
        self.guidance.is_some()
    }

    /// Answer a key by what it means in the current mode; a key unbound there is quiet.
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        let action = self
            .bindings
            .get(self.mode)
            .and_then(|keys| keys.get(&key.to_lowercase()))
            .cloned();
        if let Some(action) = action {
            self.press(&action)?;
        }
        Ok(())
    }

    pub fn press(&mut self, action: &str) -> Result<()> {
        // Naming a speaker asks the field nothing: they may have walked off, or the conversation
        // may be over, and the player still knows who they were talking to.
        if action == RENAME_SPEAKER {
            return self.rename_speaker();
        }
        // A battle's sides are the battle's, whatever the field says.
        if action == OUR_SIDE || action == ENEMY_SIDE {
            let text = match (self.sides)(action == OUR_SIDE) {
                Some(text) if !text.is_empty() => text,
                _ => "no battle".to_string(),
            };
            self.say(&text);
            return Ok(());
        }
        if action == MODE_TOGGLE {
            self.mode = if self.mode == LIST { DIRECTION } else { LIST };
            self.chosen = None;
            self.facing = None;
            self.say(&format!("{} mode", self.mode));
            return Ok(());
        }
        let Some(r#where) = self.field.poll()? else {
            self.say("no map");
            return Ok(());
        };
        let r#where = self.field.interactables(&r#where)?;
        if action == RENAME {
            return self.rename(&r#where);
        }
        if action == PATHFIND || action == PATHFIND_HM {
            let Some(item) = self.chosen_item(&r#where) else {
                self.say("nothing here");
                return Ok(());
            };
            let held = self.guidance.clone();
            return self.guide(
                &r#where,
                &item,
                Navigator::say,
                "",
                action == PATHFIND_HM,
                held,
                None,
            );
        }
        if action == CATEGORY_PREVIOUS || action == CATEGORY_NEXT {
            let step = if action == CATEGORY_PREVIOUS { -1 } else { 1 };
            self.chosen = None;
            self.facing = None;
            let was = self.category;
            for _ in 0..CATEGORIES.len() - 1 {
                self.category = py::modulo(self.category + step, CATEGORIES.len() as i64);
                let items = self.items(&r#where);
                if !items.is_empty() {
                    self.chosen = Some(items[0].key());
                    self.say(&format!(
                        "{}, {}",
                        CATEGORIES[self.category as usize],
                        items.len()
                    ));
                    return Ok(());
                }
            }
            // Nowhere to go: the category stays, so an empty map's "nothing north" does not
            // turn into "no items north" a press later.
            self.category = was;
            self.say("nothing here");
            return Ok(());
        }
        if DIRECTIONS.contains(&action) {
            return self.look(&r#where, action);
        }
        let items = self.items(&r#where);
        if items.is_empty() {
            self.say("nothing here");
            return Ok(());
        }
        let keys: Vec<Key> = items.iter().map(Interactable::key).collect();
        let mut at = match &self.chosen {
            Some(chosen) => keys.iter().position(|key| key == chosen).unwrap_or(0) as i64,
            None => 0,
        };
        if action == ITEM_PREVIOUS || action == ITEM_NEXT {
            let step = if action == ITEM_PREVIOUS { -1 } else { 1 };
            at = py::modulo(at + step, items.len() as i64);
        }
        self.chosen = Some(keys[at as usize].clone());
        self.said(&r#where, &items, at)
    }

    /// Say the next item on `direction`'s side of the player, nearest first.
    fn look(&mut self, r#where: &Field, direction: &str) -> Result<()> {
        let items: Vec<Interactable> = self
            .items(r#where)
            .into_iter()
            .filter(|item| towards(direction, r#where.cell, item.nearest(r#where.cell), RANGE))
            .collect();
        if items.is_empty() {
            self.chosen = None;
            self.facing = None;
            let category = CATEGORIES[self.category as usize];
            self.say(&if category == ALL {
                format!("nothing {direction}")
            } else {
                format!("no {category} {direction}")
            });
            return Ok(());
        }
        let keys: Vec<Key> = items.iter().map(Interactable::key).collect();
        let mut at = 0;
        let kept = self
            .chosen
            .as_ref()
            .and_then(|chosen| keys.iter().position(|key| key == chosen));
        if self.facing.as_deref() == Some(direction)
            && let Some(kept) = kept
        {
            at = py::modulo(kept as i64 + 1, items.len() as i64);
        }
        self.chosen = Some(keys[at as usize].clone());
        self.facing = Some(direction.to_string());
        self.said(r#where, &items, at)
    }

    fn items(&self, r#where: &Field) -> Vec<Interactable> {
        r#where.of(CATEGORIES[self.category as usize])
    }

    fn said(&self, r#where: &Field, items: &[Interactable], at: i64) -> Result<()> {
        let item = &items[at as usize];
        let mut parts = vec![
            self.label(item, r#where.zone)?,
            item.bearing(r#where.cell, r#where.heading)?,
        ];
        if self.wording.positions {
            parts.insert(
                if self.wording.position_first { 1 } else { 2 },
                format!("{} of {}", at + 1, items.len()),
            );
        }
        self.say(&parts.join(", "));
        Ok(())
    }

    /// What to call this: the name it has been given, or what it is when it has none.
    pub fn name_of(&self, item: &Interactable, zone: i64) -> String {
        match self.name(item, zone) {
            Some(name) if !name.is_empty() => name,
            _ => item.kind.clone(),
        }
    }

    /// What this is said as: what it is called, and for an exit into another place, the
    /// place: "edge to Route 101", "path to Route 102", "the north gap to Route 101".
    ///
    /// The place is the game's own word, so a name the player gives an exit comes before it
    /// rather than instead of it (ADR-0023: nothing of ours replaces what the game says); a name
    /// that already says the place is left as it is.
    pub fn label(&self, item: &Interactable, zone: i64) -> Result<String> {
        let called = self.name_of(item, zone);
        Ok(leading(&called, self.destination(item, zone)?.as_deref()))
    }

    /// What the game calls the place this exit from `zone` leads to, or None: for anything that
    /// is not an exit, for an exit into the player's own place — a house's door, the mat out of
    /// it, the stairs between two floors, though not the edge onto another zone of the same
    /// outdoors, as Route 119's two are — and without the game's text.
    pub fn destination(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        let Some(there) = item.zone else {
            return Ok(None);
        };
        let home = item.home.unwrap_or(zone);
        // The street first: Lumiose's zones are one place, and a warp between two of them on one
        // matrix would say that place where the street says more.
        if let Some(street) = (self.street)(home, item.source)?
            && !street.is_empty()
        {
            return Ok(Some(street));
        }
        (self.leads_to)(zone, there)
    }

    /// The name this has been given, by the player or the game, or None.
    pub fn called(&self, item: &Interactable, zone: i64) -> Option<String> {
        self.name(item, zone)
    }

    fn name(&self, item: &Interactable, zone: i64) -> Option<String> {
        let (what, first, second) = item.naming(zone)?;
        if what == SPRITE {
            self.names.sprite(first)
        } else {
            self.names.place(first, second)
        }
    }

    /// The thing the navigation keys are on now, or None when they are on nothing.
    fn chosen_item(&self, r#where: &Field) -> Option<Interactable> {
        let mut items = self.items(r#where);
        if items.is_empty() {
            return None;
        }
        let at = match &self.chosen {
            Some(chosen) => items
                .iter()
                .position(|item| &item.key() == chosen)
                .unwrap_or(0),
            None => 0,
        };
        Some(items.swap_remove(at))
    }

    /// Find the way from where the player stands to `item`, say it, and follow the player
    /// along it; or say why there is nothing to follow. `hm` finds it over water too.
    ///
    /// Where the decks at the player's height lead nowhere, they are walked at each other
    /// floor's height in turn: from the foot of Route 119 the way north is over a bridge high
    /// above the path it crosses (spike 159). Where nothing leads there, the way goes as near as
    /// the player can get, said with ", then can't continue", and "no path" only where nowhere
    /// is nearer than where they stand (the user, 2026-10-03). `height` is where to walk them
    /// first, the height a way being found again was walked at: at the player's own, every
    /// step off a way over a bridge searched the whole map before finding it again.
    ///
    /// `heading` is the way a player who has left the path was walking. A new path that starts
    /// that way is taken quietly, and its next turn said when they reach it: a player who took
    /// a way of their own as short as ours heard the whole way again at every step, north
    /// first where ours went east first (poke_center_pc.ctm, 2026-09-24).
    ///
    /// `held` is the way being walked when P is pressed again. Where no new way is found and the
    /// player is on that one to the same thing, it is kept and what is left of it said: P again
    /// halfway up Route 110 said "no path to path" and left nothing to follow, while the way
    /// being walked still went on (omega_ruby_movil.ctm, spike 142).
    ///
    /// `say` is `Navigator::say` or `Navigator::tell`, which Python passes as bound methods.
    fn guide(
        &mut self,
        r#where: &Field,
        item: &Interactable,
        say: fn(&Navigator, &str),
        heading: &str,
        hm: bool,
        held: Option<Rc<RefCell<Guidance>>>,
        height: Option<f64>,
    ) -> Result<()> {
        self.guidance = None;
        let name = self.name_of(item, r#where.zone);
        if !self.routes {
            let text = format!(
                "{}, {}, no route in this game yet",
                self.label(item, r#where.zone)?,
                item.bearing(r#where.cell, r#where.heading)?
            );
            say(self, &text);
            return Ok(());
        }
        let mut height = height;
        let mut ground = self.ground(r#where, hm, height)?;
        if r#where.on_rail && ground.corridor.is_none() {
            // Off the rails the game has loaded, the walls are not known, and the chunks there
            // mark every cell a wall: a way found on them would be nonsense.
            let text = format!(
                "{name}, {}, no route on these streets",
                item.bearing(r#where.cell, r#where.heading)?
            );
            say(self, &text);
            return Ok(());
        }
        let mut goals = ground.goals(item)?;
        let mut path = search(&ground, r#where.cell, &goals, heading, true)?;
        // ponytail: a whole map's decks walked at one height, so a way that goes under a deck and
        // later onto it is not found; a search over each cell's floors would find it.
        let levels: Vec<Option<f64>> = std::iter::once(height)
            .chain(ground.decks.iter().map(|&deck| Some(deck)))
            .collect();
        let decks = if ends_short(&path, &goals)? {
            ground.decks.clone()
        } else {
            Vec::new()
        };
        for deck in decks {
            let other = self.ground(r#where, hm, Some(deck))?;
            let aim = other.goals(item)?;
            let found = search(&other, r#where.cell, &aim, heading, false)?;
            if let Some(found) = found {
                ground = other;
                goals = aim;
                path = Some(found);
                height = Some(deck);
                break;
            }
        }
        let short = ends_short(&path, &goals)?;
        if short {
            let at = match &held {
                Some(held) if held.borrow().key == item.key() => {
                    held.borrow().along.get(&r#where.cell).copied()
                }
                _ => None,
            };
            if let Some(held) = &held
                && let Some(at) = at
                && at < held.borrow().path.len() as i64 - 1
            {
                self.guidance = Some(held.clone());
                let mut text = left(&held.borrow(), at, r#where)?;
                if held.borrow().short {
                    text += ", then can't continue";
                }
                say(self, &text);
                return Ok(());
            }
        }
        let mut blocker = String::new();
        if short && let Some(past) = self.up_to(r#where, item, hm, heading, &levels)? {
            // Python also rebinds `goals` here, and never reads it again.
            let (past_ground, _goals, way, past_height, stood) = past;
            ground = past_ground;
            path = Some(way);
            height = past_height;
            blocker = match stood {
                None => String::new(),
                Some(stood) => {
                    format!(", {} in the way", self.label(&stood, r#where.zone)?)
                }
            };
        }
        let path = match path {
            Some(path) if !(short && path.len() == 1) => path,
            _ => {
                say(self, &format!("no path to {name}{blocker}"));
                return Ok(());
            }
        };
        if path.len() == 1 {
            let text = format!("{name}, {}", item.bearing(r#where.cell, r#where.heading)?);
            say(self, &text);
            return Ok(());
        }
        // A player already surfing needs no telling that the water is crossed with Surf.
        let surfing = ground.water(r#where.cell)?;
        let mut pushed: HashSet<(Cell, Cell)> = HashSet::new();
        for i in ground.pushes(&path)? {
            pushed.insert((path[i as usize - 1], path[i as usize]));
        }
        let how = |cell: Cell, step: Cell| -> Result<String> {
            if pushed.contains(&(cell, step)) {
                return Ok(PUSH.to_string());
            }
            if ground.hopping(cell, step) {
                return Ok(self.acro.clone());
            }
            if let Some(clear) = ground.cleared.get(&step) {
                return self
                    .moves
                    .get(clear)
                    .cloned()
                    .ok_or_else(|| Error::key(clear.to_string()));
            }
            if !surfing && ground.water(step)? {
                return self
                    .moves
                    .get(&SURF)
                    .cloned()
                    .ok_or_else(|| Error::key(SURF.to_string()));
            }
            if !ground.bike && ground.climbs(cell, step)? {
                return Ok(self.bike.clone());
            }
            Ok(String::new())
        };
        let legs = pathfind::legs(&path, Some(&how))?;
        let visit = self.field.visit();
        let along: HashMap<Cell, i64> = path
            .iter()
            .enumerate()
            .map(|(i, &cell)| (cell, i as i64))
            .collect();
        let since = (self.clock)();
        let first = legs.first().map(|leg| leg.direction);
        self.guidance = Some(Rc::new(RefCell::new(Guidance {
            key: item.key(),
            name,
            visit,
            path,
            legs: legs.clone(),
            along,
            cell: r#where.cell,
            leg: 0,
            camera: r#where.bearing,
            seen: r#where.bearing,
            since,
            hm,
            height,
            short,
            blocker: blocker.clone(),
        })));
        let first = first.ok_or_else(|| Error::index("list index out of range"))?;
        if first != heading {
            // Short of the thing, the way goes as near as it can, which the player hears the
            // end of: they asked for that over "no path" (the user, 2026-10-03).
            let mut text = spoken(&legs, r#where)?;
            if short {
                text += &format!(", then can't continue{blocker}");
            }
            say(self, &text);
        }
        if !self.wording.turn_by_turn {
            // Said once and not followed: P again says it from wherever the player has got to.
            self.guidance = None;
        }
        Ok(())
    }

    /// Keep up with the player along the path being walked: the next leg as they reach each
    /// turn, the thing itself when they arrive, and a new path when they leave this one.
    pub fn follow(&mut self, r#where: &Field) -> Result<()> {
        let Some(guidance) = self.guidance.clone() else {
            return Ok(());
        };
        let visit = self.field.visit();
        let settled = self.settled(r#where, &mut guidance.borrow_mut());
        let seen = guidance.borrow().cell;
        if r#where.cell == seen {
            // The field rebuilt around a player standing still is not a warp: nothing moved.
            guidance.borrow_mut().visit = visit;
            if settled && let Some(bearing) = r#where.bearing {
                self.turned(r#where, &guidance, bearing)?;
            }
            return Ok(());
        }
        if visit != guidance.borrow().visit {
            self.guidance = None;
            (self.log)(&format!(
                "path: a warp, the way to {} is left",
                guidance.borrow().name
            ));
            return Ok(());
        }
        let was = std::mem::replace(&mut guidance.borrow_mut().cell, r#where.cell);
        let heading = pathfind::towards(was, r#where.cell);
        let mut at = guidance.borrow().along.get(&r#where.cell).copied();
        if at.is_none() && self.slack != 0 {
            at = near(&guidance.borrow().path, r#where.cell, self.slack);
        }
        let Some(at) = at else {
            return self.again(r#where, &guidance, heading);
        };
        let last = guidance.borrow().path.len() as i64 - 1;
        if at == last {
            self.guidance = None;
            let listed = self.field.interactables(r#where)?;
            let item = self.find(&listed, &guidance.borrow());
            if let Some(item) = item {
                let text = format!(
                    "{}{}, {}",
                    if guidance.borrow().short {
                        format!("can't continue{}, ", guidance.borrow().blocker)
                    } else {
                        String::new()
                    },
                    guidance.borrow().name,
                    item.bearing(r#where.cell, r#where.heading)?
                );
                self.tell(&text);
            }
            return Ok(());
        }
        // A chunk loaded since the path was found can show a wall where nothing was known. A
        // ledge on the way is not one: it is jumped.
        let (hm, height) = {
            let guidance = guidance.borrow();
            (guidance.hm, guidance.height)
        };
        let ground = self.ground(r#where, hm, height)?;
        let mut all = true;
        for &cell in py::slice_from(&guidance.borrow().path, at + 1) {
            if !(ground.open(cell)? || ground.ledge(cell)?.is_some()) {
                all = false;
                break;
            }
        }
        if !all {
            return self.again(r#where, &guidance, heading);
        }
        let leg = guidance
            .borrow()
            .legs
            .iter()
            .position(|leg| leg.end > at)
            .expect("a leg ends past the cell") as i64;
        if leg != guidance.borrow().leg {
            let now = {
                let mut guidance = guidance.borrow_mut();
                guidance.leg = leg;
                guidance.camera = r#where.bearing;
                guidance.legs[leg as usize].clone()
            };
            let text = spoken(
                &[Leg {
                    steps: now.end - at,
                    ..now
                }],
                r#where,
            )?;
            self.tell(&text);
        }
        Ok(())
    }

    /// Where somebody stands in the way: the way there as if nobody stood anywhere, as far
    /// as the first who does, with its ground, goals, path, the height its decks are walked at,
    /// and who it is; or None where nobody is what stops it. In Fortree City the invisible
    /// Kecleon stands in the one way to the Gym until the Devon Scope shows it, and the way as
    /// near as the player could get as the crow flies ended behind a wall from the Gym, on a
    /// walkway 7 cells south of its door (spike 160).
    fn up_to(
        &self,
        r#where: &Field,
        item: &Interactable,
        hm: bool,
        heading: &str,
        levels: &[Option<f64>],
    ) -> Result<Option<Past>> {
        let own: HashSet<Cell> = item.cells().into_iter().collect();
        for &level in levels {
            let ground = self.ground(r#where, hm, level)?;
            let mut clear = ground.clone();
            clear.standing = ground
                .standing
                .iter()
                .filter(|cell| own.contains(cell))
                .copied()
                .collect();
            let goals = clear.goals(item)?;
            let way = search(&clear, r#where.cell, &goals, heading, false)?;
            let Some(way) = way else {
                continue;
            };
            let Some(at) = way.iter().position(|cell| ground.standing.contains(cell)) else {
                continue;
            };
            let stood = r#where
                .interactables
                .iter()
                .find(|thing| thing.move_model && thing.cells().contains(&way[at]))
                .cloned();
            return Ok(Some((ground, goals, way[..at].to_vec(), level, stood)));
        }
        Ok(None)
    }

    /// Whether the camera has stopped turning: it has stayed within `STILL` of one bearing
    /// for `SETTLE` seconds. False off the rails, where there is no camera to turn.
    fn settled(&self, r#where: &Field, guidance: &mut Guidance) -> bool {
        let Some(bearing) = r#where.bearing else {
            guidance.seen = None;
            return false;
        };
        let now = (self.clock)();
        if guidance
            .seen
            .is_none_or(|seen| apart(bearing, seen) > STILL)
        {
            guidance.seen = Some(bearing);
            guidance.since = now;
            return false;
        }
        now - guidance.since >= SETTLE
    }

    /// The way on from here again, when the camera has stopped somewhere new with the
    /// player standing still. On a rail the way is said from the camera, which left and right
    /// turn only while held: a tap turns the player and leaves the camera where it was, or
    /// between two ways, and the way said before is wrong for it, and a hold makes a turn said
    /// done, which is worth hearing (spike 105).
    fn turned(
        &self,
        r#where: &Field,
        guidance: &Rc<RefCell<Guidance>>,
        bearing: f64,
    ) -> Result<()> {
        if guidance
            .borrow()
            .camera
            .is_some_and(|camera| apart(bearing, camera) < MOVED)
        {
            return Ok(());
        }
        let at = guidance.borrow().along.get(&r#where.cell).copied();
        let last = guidance.borrow().path.len() as i64 - 1;
        let Some(at) = at else {
            return Ok(());
        };
        if at == last {
            return Ok(());
        }
        guidance.borrow_mut().camera = Some(bearing);
        let text = left(&guidance.borrow(), at, r#where)?;
        self.tell(&text);
        Ok(())
    }

    /// Find the way again from where the player has got to, walking `heading`.
    fn again(
        &mut self,
        r#where: &Field,
        guidance: &Rc<RefCell<Guidance>>,
        heading: &str,
    ) -> Result<()> {
        let listed = self.field.interactables(r#where)?;
        if Ground::of(&listed, Of::default())
            .stepped
            .contains(&r#where.cell)
        {
            // On a door that is not the way's end, or the hidden Pokémon: what the cell does is
            // about to happen, and a path said now would be for a map the player is leaving. The
            // game walks the player into a door's own cell before the warp (poke_center_pc.ctm).
            return Ok(());
        }
        let Some(item) = self.find(&listed, &guidance.borrow()) else {
            self.guidance = None;
            return Ok(());
        };
        let (hm, height) = {
            let guidance = guidance.borrow();
            (guidance.hm, guidance.height)
        };
        self.guide(&listed, &item, Navigator::tell, heading, hm, None, height)
    }

    /// The field's ground, its decks walked at `height` or the player's own, crossing water
    /// for a way with HMs and for a player already on it: P from the water keeps to it, or
    /// there is no first step to take. Only a way with HMs cuts through a bush. A slope is
    /// ridden up with HMs and by a player on the bike.
    fn ground(&self, r#where: &Field, hm: bool, height: Option<f64>) -> Result<Ground> {
        let mut ground = self.field.ground(r#where, height)?;
        ground.surf = hm || ground.water(r#where.cell)?;
        ground.hm = hm;
        ground.bike = r#where.on_bike;
        Ok(ground)
    }

    /// What the path leads to, where it is now, or None, said, when it has gone: a person
    /// walks, and a ball is picked up.
    fn find(&self, listed: &Field, guidance: &Guidance) -> Option<Interactable> {
        let item = listed
            .interactables
            .iter()
            .find(|item| item.key() == guidance.key)
            .cloned();
        if item.is_none() {
            self.tell(&format!("{} gone", guidance.name));
        }
        item
    }

    /// Put the rename dialog up for whatever is chosen.
    ///
    /// Nothing chosen is nothing to rename, and no dialog opens: that is the common case for a
    /// mistyped key, and it must not leave a window in front of the game.
    fn rename(&mut self, r#where: &Field) -> Result<()> {
        let Some(item) = self.chosen_item(r#where) else {
            self.say("nothing to rename");
            return Ok(());
        };
        let Some(naming) = item.naming(r#where.zone) else {
            // A hidden Pokémon: a different one each time, and the game names the species itself.
            self.say(&format!("{} cannot be renamed", item.kind));
            return Ok(());
        };
        if self.asker.open() {
            self.say("already renaming");
            return Ok(());
        }
        let called = self.name(&item, r#where.zone);
        self.renaming = Some((naming, self.label(&item, r#where.zone)?));
        let label = format!(
            "{}, {}. ",
            leading(
                &item.kind,
                self.destination(&item, r#where.zone)?.as_deref()
            ),
            item.bearing(r#where.cell, r#where.heading)?
        ) + &match called.as_deref() {
            Some(called) if !called.is_empty() => format!("Called: {called}"),
            _ => "Not named yet".to_string(),
        };
        self.asker.ask(
            RENAMING,
            "Rename",
            &label,
            called.as_deref().unwrap_or_default(),
        )?;
        Ok(())
    }

    /// Put the dialog up for whoever last spoke.
    fn rename_speaker(&mut self) -> Result<()> {
        let Some(character) = (self.speaker)() else {
            self.say("nobody has spoken");
            return Ok(());
        };
        if self.asker.open() {
            self.say("already renaming");
            return Ok(());
        }
        let called = self.names.sprite(character);
        let called = called.filter(|called| !called.is_empty());
        self.renaming = Some((
            (SPRITE, character, 0),
            called
                .clone()
                .unwrap_or_else(|| "the last speaker".to_string()),
        ));
        let label = "The last speaker. ".to_string()
            + &match &called {
                Some(called) => format!("Called: {called}"),
                None => "Not named yet".to_string(),
            };
        self.asker.ask(
            SPEAKING,
            "Rename",
            &label,
            called.as_deref().unwrap_or_default(),
        )?;
        Ok(())
    }

    /// Keep any name given since the last poll, and say it back.
    ///
    /// Said back because the dialog closing is not evidence to a blind player that anything was
    /// saved. An empty name clears theirs and gives the thing back to the shipped table, so what
    /// is said afterwards is read from the store rather than assumed.
    pub fn poll_renames(&mut self) -> Result<()> {
        for answer in self.asker.answers() {
            if (answer.what != RENAMING && answer.what != SPEAKING) || self.renaming.is_none() {
                continue;
            }
            let Some(((what, first, second), was)) = self.renaming.take() else {
                continue;
            };
            let now = if what == SPRITE {
                self.names.name_sprite(first, &answer.said)?;
                self.names.sprite(first)
            } else {
                self.names.name_place(first, second, &answer.said)?;
                self.names.place(first, second)
            };
            match now {
                Some(now) if !now.is_empty() => self.say(&format!("{was} is now {now}")),
                _ => self.say(&format!("{was} has no name")),
            }
        }
        Ok(())
    }

    fn say(&self, text: &str) {
        (self.log)(&format!("key: {text}"));
        self.speech.say(text, Category::Key);
    }

    /// Say something about the path unasked, as the player walks it.
    fn tell(&self, text: &str) {
        (self.log)(&format!("path: {text}"));
        self.speech.say(text, Category::Guidance);
    }
}

/// `up_to`'s answer: the ground, goals, path, the height its decks are walked at, and who stands
/// in the way.
type Past = (
    Ground,
    IndexMap<Cell, Option<&'static str>>,
    Vec<Cell>,
    Option<f64>,
    Option<Interactable>,
);

/// `path is None or path[-1] not in goals`, which raises on an empty path as Python's index does.
fn ends_short(
    path: &Option<Vec<Cell>>,
    goals: &IndexMap<Cell, Option<&'static str>>,
) -> Result<bool> {
    let Some(path) = path else {
        return Ok(true);
    };
    let end = path
        .last()
        .ok_or_else(|| Error::index("tuple index out of range"))?;
    Ok(!goals.contains_key(end))
}

/// `pathfind.find(...)` over `ground` with `**_doors(ground)`, which each of Python's three
/// calls spells out in full.
fn search(
    ground: &Ground,
    start: Cell,
    goals: &IndexMap<Cell, Option<&'static str>>,
    heading: &str,
    closest: bool,
) -> Result<Option<Vec<Cell>>> {
    let r#box = ground.r#box(start, goals.keys().copied())?;
    let walkable = |cell: Cell| ground.passable(cell);
    let ledge = |cell: Cell| ground.ledge(cell);
    let toll = |cell: Cell, step: Cell| ground.toll(cell, step);
    let joined = |cell: Cell, step: Cell| Ok(ground.joined(cell, step));
    let (turns, crossings) = doors(ground);
    let through = |turns: &[i64], cell: Cell, step: Cell| {
        ground.through(turns, cell, step, crossings.as_deref())
    };
    pathfind::find(
        start,
        goals,
        &walkable,
        r#box,
        &pathfind::Options {
            heading,
            ledge: Some(&ledge),
            toll: Some(&toll),
            closest,
            joined: Some(&joined),
            turns,
            through: if crossings.is_some() {
                Some(&through)
            } else {
                None
            },
        },
    )
}

/// How far along `path` the cell nearest `cell` is, when one is within `slack` cells of it
/// either way, the furthest along of the nearest; or None.
fn near(path: &[Cell], cell: Cell, slack: i64) -> Option<i64> {
    let best = path
        .iter()
        .enumerate()
        .map(|(at, &(x, z))| ((x - cell.0).abs().max((z - cell.1).abs()), -(at as i64)))
        .min();
    match best {
        Some(best) if best.0 <= slack => Some(-best.1),
        _ => None,
    }
}

/// What a search is given to walk through the ground's rotating doors: where they stand now,
/// and what each step does to them (`Ground.through`, over the returned crossings); nothing where
/// there are none.
fn doors(ground: &Ground) -> (pathfind::Turns, Option<Rc<HashMap<(Cell, Cell), i64>>>) {
    if ground.doors.is_empty() {
        return (Vec::new(), None);
    }
    // ponytail: every way the doors can stand is searched with the cells, 0.4 to 0.6 s across
    // Fortree Gym's seven; a search over the doors' turns alone, then the cells, if it grows.
    (ground.turns(), Some(ground.crossings()))
}

/// `called` with the place it leads to after it, unless it already says it.
fn leading(called: &str, place: Option<&str>) -> String {
    match place {
        Some(place) if !called.contains(place) => format!("{called} to {place}"),
        _ => called.to_string(),
    }
}

/// What is left of the way from its `at`th cell, said from where the player stands.
fn left(guidance: &Guidance, at: i64, r#where: &Field) -> Result<String> {
    let leg = guidance
        .legs
        .iter()
        .position(|leg| leg.end > at)
        .expect("a leg ends past the cell");
    let now = &guidance.legs[leg];
    let mut legs = vec![Leg {
        steps: now.end - at,
        ..now.clone()
    }];
    legs.extend_from_slice(&guidance.legs[leg + 1..]);
    spoken(&legs, r#where)
}

/// Legs as the player's keys walk them: by the compass on the grid, and on a rail, where the
/// keys work from the camera behind the player, from the way it looks (spike 105).
fn spoken(legs: &[Leg], r#where: &Field) -> Result<String> {
    if let Some(heading) = r#where.heading {
        let angled = match r#where.bearing {
            Some(bearing) => pathfind::off(bearing, heading)?.abs() > ANGLED,
            None => false,
        };
        return pathfind::turned(legs, heading, angled);
    }
    Ok(pathfind::spoken(legs))
}

/// Degrees between two bearings, the short way round.
fn apart(one: f64, other: f64) -> f64 {
    (py::modulo_f(one - other + 180.0, 360.0) - 180.0).abs()
}
