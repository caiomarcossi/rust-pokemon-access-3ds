//! Menu cursors: say the item under a menu's cursor, as the menu opens and as the cursor moves.
//!
//! App menus draw their cursor — the red corners around the chosen item — as a layout of its own: a
//! root pane whose only child is an `nw::lyt::Window` (spike 40). The window sits on the chosen item
//! at the item's size (spike 39), so the item under the cursor is the screen text inside the window's
//! rectangle, on the screen the cursor is drawn to (spikes 40 and 41). The X menu, the bag and the save
//! prompt's list all draw theirs this way.
//!
//! ```text
//! Window  +0x0c -> the layout's root    +0x48 f32 width    +0x4c f32 height
//!         +0x80 f32 global scale x    +0x8c f32 x        +0x94 f32 global scale y   +0x9c f32 y
//! root    +0x0c 0, a root has no parent                    +0x10 u32 child count, 1 for a cursor
//! ```
//!
//! The battle draws its cursor otherwise (spike 50): a small `nw::lyt::Picture` alone in a layout of
//! its own, placed exactly on an empty pane in the chosen button's group. The item is that group: the
//! screen text under it, as a move's "Scratch PP 35/35", or, where the button is all pictures, as the
//! command menu's FIGHT, the words of the name of its first picture's texture, "fight", from the
//! layout file the button's layout was built from.
//!
//! The name entry's keyboard draws a third kind (spike 68): each key is a picture holding its letter
//! and an `nw::lyt::Bounding` touch area, and the cursor is a picture the game moves into the chosen
//! key's touch area. The item is the key's words, or, for a key drawn without any — the one that
//! confirms the name — the words of its picture's texture, as a battle button's are.
//!
//! The main menu draws a fourth (spike 72): the same red corners as an app menu's, but as a window in
//! the menu's own layout, directly under its root among the item plates, moved and resized onto the
//! chosen item. A frame directly under a root is taken for a cursor once it has moved on its own, its
//! sibling windows staying where they were, from one resting place to another: a list that scrolls
//! moves its rows together, a frame sliding in is somewhere new every poll, and a window that never
//! moves is a panel's border. Until it moves, the screen is read as any screen is.
//!
//! The opening's choice of boy or girl draws a fifth (spike 71): a window the game slides from one
//! choice's place onto another's inside its own layout, each choice a plain pane beside it holding
//! that choice's pictures. A window with no panes of its own, beside two or more plain panes of one
//! size in places of their own, is a cursor from the poll it is seen on, and the choice is the pane
//! it sits exactly on: the frame round each choice sits on its own pictures, not on a pane beside it.
//! The choice is said by its own words, or, having none, by the words of the textures the other
//! choices do not share — both portraits sit on the same white backing, and only `intro_male.bclim`
//! and `intro_famale.bclim` tell them apart.
//!
//! The Pokédex draws a sixth (spike 95): a **plate**, a window with no panes of its own that never
//! moves, while the list's rows scroll under it — three or more plain panes under one parent, each
//! holding words, all at one x and of one height. The item is the row the plate sits on.
//!
//! The Poké Mart draws a seventh (spike 95): two pictures textured `shop_cursor.bclim` in a pane of
//! their own, moved onto the chosen row. A picture whose texture the designer named a cursor is one
//! from the poll it is seen on, and the item is the words inside the pane holding it. Its boxes of
//! choices point one, `cursor.bclim`, at their words from beside them (spike 98): Buy, Sell and No
//! thanks, and a purchase's Yes and No, in a window alone under their root. Over no words, such a
//! picture says the words on its row to its right; and a window holding panes of its own is a box,
//! not the corners of an app menu's cursor.
//!
//! The PC Box draws one with `cursol01.bclim`, as its designers spelt it there (spike 127): a
//! picture in a plain pane the game moves onto the chosen row of its menu for a chosen Pokémon, where
//! each row is a picture of its own. A named picture's list is the row it is over and the rows drawn
//! like it; over rows drawn apart like these it glides through each on its way, so its item is held
//! until it has read the same on two polls, as a plate's is. Once it has been over words it no longer
//! points at the words beside it: the Box's leaves for the grid of Pokémon a poll before its menu's
//! rows go. Over no words at all it may rest on a slot, a touch area of the Box's grid, its party or
//! a Pokémon's marks, said by where it is (`slots`, spike 129); each group of slots is reported as a
//! cursor of its own, so that the summary its moves change is not taken for the menu's details.
//!
//! The main menu's frame is a named cursor too (spike 96): a window drawn with `cursor_lt.bclim`, so
//! known from the poll it is seen on, where a frame directly under a root is otherwise known only
//! once it has moved.
//!
//! Wally's catching tutorial draws an eighth (spike 96): a **pointer**, a small picture alone in a
//! layout but for the one plain pane that places it, which the game puts on the button about to be
//! tapped. The item is the button whose touch area holds that place, in whatever layout came on that
//! screen last; where another cursor claims that layout, the pointer says nothing.
//!
//! The plate and a named picture know the list they walk: the plate's rows, and the words beside
//! those the named picture is over. What else their layout shows — the Mart's money and the chosen
//! item's description — is the screen's own, not the menu's (`lists`). A plate says an item only
//! once it has read the same on two polls, because a list scrolling under it slides its rows a row's
//! height and back while their words are rewritten, and for a poll the row under the plate holds a
//! hidden row's stale words.
//!
//! Sun draws a ninth (spike 112), for every menu read so far — the title's, the X menu, the options
//! and the save prompt's Yes and No: a **cursor pane**, a plain pane alone under the root of a layout
//! of its own holding only the arrow's pictures, which the game moves and sizes onto the chosen item.
//! Each of the menu's buttons keeps an empty pane of exactly that place and size, and the item is
//! that button: all its words, as the title's Continue says the name, place and play time beside it.
//! Where the buttons' own panes are hidden and never placed, as the Yes and No's are, the item is
//! the words inside the cursor pane, as an app menu's cursor window's is. The talk window's answers
//! are pointed at as the Poké Mart's boxes' are, by a picture beside the words on its row, known by
//! where it stands, alone in a plain pane in the answers' window, since Sun's layout files are not
//! read to name its texture.

use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::{Layouts, MAX_DEPTH};
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::pictures::{EMPTY, pane_words, words};
use crate::games::common::screen::{
    Pictures, SCREEN_HALF_HEIGHT, SCREEN_HALF_WIDTH, ScreenText, distinct, in_rows,
};
use crate::games::common::{lyt, slots};
use crate::py::{self, Error, PySet, Result};

/// The biggest side of a picture taken for a cursor. The battle's is 20 by 20; a lone picture the
/// size of a screen is a dimmer.
pub const MARKER_SIZE: f64 = 64.0;
/// Ticks a named cursor must have stood where a screen poll finds it to be at rest there: a glide
/// moves it every frame, so it never stands on a slot or a row it passes for two (spike 129).
pub const REST_TICKS: i64 = 3;
/// Units within which a cursor picture sits on a pane. The battle's lands exactly.
pub const SAME_SPOT: f64 = 1.0;
/// Units a Sun cursor pane may be larger than the pane it is fitted to, each way. The battle's
/// command menu draws its corners in a pane 64 wide over a `Null_Cur` of 56 (spike 124); every
/// other menu's fits exactly, and among several panes in one place the nearest in size is taken.
pub const CURSOR_SLACK: f64 = 8.0;
/// Polls a window directly under a root holds one place and size for it to count as resting there.
/// A cursor jumps from rest to rest; a frame sliding onto the screen is somewhere new every poll until
/// it stops, and taking that for a cursor would make a menu of the screen it frames.
pub const FRAME_REST: i64 = 2;
/// The fewest like rows that make a list a plate can sit on. The Pokédex shows five at the end of
/// its list and nine elsewhere.
pub const LIST_ROWS: i64 = 3;
/// The most rows a plate is tall. The Pokédex's is 40 units on rows of 30; a window as tall as
/// several rows is a panel's frame, not a plate on one of them.
pub const PLATE_ROWS: f64 = 2.0;
/// The words a texture's name carries when the designer drew a cursor with it, or its whole name:
/// the Poké Mart's `shop_cursor.bclim` (spike 95), the PC Box's `cursol01.bclim`, as they spelt it
/// there (2026-09-27), and the Bag's `select_bar.bclim`, the plate the chosen Pokémon is drawn on as
/// a TM is taught (spike 139). "select" alone is no cursor: the PC Box draws its grid with
/// `select_box01.bclim`. The ROM's other spellings wait for a movie that reaches them: `cursol` on the
/// field's bottom-screen buttons, the Town Map and a minigame, and `cursor1` in the move lists.
pub const CURSOR_TEXTURES: [&str; 3] = ["cursor", "cursol01", "select_bar"];

/// The pieces of screen text a cursor is over, when it came on screen or moved.
#[derive(Debug, Clone, PartialEq)]
pub struct CursorItem {
    pub cursor: i64,
    pub pieces: Vec<ScreenText>,
    /// True for the first item said since the cursor came on screen; false when it moved.
    pub opened: bool,
    /// No details follow it: an empty slot, over which the top screen's summary keeps showing
    /// the Pokémon carried or the one last passed (spike 129).
    pub quiet: bool,
}

impl CursorItem {
    /// `CursorItem(cursor, pieces, opened)`: `quiet` left to its default, False.
    pub fn new(cursor: i64, pieces: Vec<ScreenText>, opened: bool) -> Self {
        CursorItem {
            cursor,
            pieces,
            opened,
            quiet: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    top: bool,
    left: f64,
    right: f64,
    low: f64,
    high: f64,
}

/// Where a cursor picture was, and the pane it sat on.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Spot {
    x: f64,
    y: f64,
    anchor: i64,
    root: i64,
}

/// What the Python keeps as `where: object`: where a cursor was when it last said an item, a place
/// to the unit or the pane it sat on, which is only ever compared for equality. `Nothing` is the
/// `None` a said item is reset to.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Where {
    Nothing,
    Pane(i64),
    Place(i64, i64),
}

/// `tuple[tuple[str, ...], object]`: the words last reported and where they were.
type Said = (Vec<String>, Where);
type Choice = (i64, Option<i64>, Vec<i64>);
type Plate = (i64, i64, Vec<i64>);

/// Tracks menu cursors and reports the screen text under each.
pub struct CursorReader {
    layouts: Rc<Layouts>,
    names: Rc<ClassNames>,
    files: Option<Rc<LayoutFiles>>,
    pictures: Option<Pictures>,
    cursor_panes: bool,
    /// Cursor pane on screen -> where it was at the last poll and the pane it was fitted to,
    /// an anchor of 0 where none was and the words inside it were its item.
    fits: IndexMap<i64, Spot>,
    cursors: Vec<i64>,
    /// Cursor -> the pieces under it at the last poll, for cursors on screen.
    under: IndexMap<i64, Vec<ScreenText>>,
    /// Cursor -> the words last reported and where they were, for cursors on screen.
    said: IndexMap<i64, Said>,
    /// Cursor picture -> where it sat at the last poll it was found on a pane.
    spots: IndexMap<i64, Spot>,
    /// Cursor picture -> polls in a row it has not been found on a pane.
    missed: IndexMap<i64, i64>,
    /// (root, group) -> the words of the group's first picture's texture, once known.
    picture_words: IndexMap<(i64, i64), Option<String>>,
    /// Picture in a key's touch area -> the key it was in at the last poll it was seen.
    key_plates: IndexMap<i64, i64>,
    /// Pictures in a key's touch area that have moved from one key to another.
    moved_keys: HashSet<i64>,
    /// Window directly under a root -> (its place and size at the last poll it was shown, the
    /// polls in a row it has been there).
    frames: IndexMap<i64, ((i64, i64, i64, i64), i64)>,
    /// Such a window -> polls in a row it has not been shown.
    frames_missed: IndexMap<i64, i64>,
    /// Such a window -> where it has just moved to from a rest, a cursor once it rests there.
    pending_frames: IndexMap<i64, (i64, i64, i64, i64)>,
    /// Windows directly under a root that have moved from one rest to another while their
    /// sibling windows stayed.
    moved_frames: HashSet<i64>,
    /// Windows reported as the cursor over one of the choices beside them.
    choices_seen: IndexSet<i64>,
    /// (root, choice) -> the words of the textures of the pictures under it, once known.
    choice_words: IndexMap<(i64, i64), Vec<String>>,
    /// Plate or named cursor on screen -> (its layout's root, the TextBoxes of the list it
    /// walks).
    listing: IndexMap<i64, (i64, HashSet<i64>)>,
    /// Plate or named cursor -> polls in a row it has not been found.
    listing_missed: IndexMap<i64, i64>,
    /// Plate or named cursor -> the words under it and where, at the last poll.
    steady: IndexMap<i64, Said>,
    /// (root, picture) -> whether its texture is named a cursor, once known.
    cursor_textures: IndexMap<(i64, i64), bool>,
    /// Named cursors that have been over words, and so never point at words beside them.
    over: HashSet<i64>,
    /// Layouts a named cursor has been found in, looked in for it while their words are all
    /// off screen: the Box's box list hides the Box's every word, its pointer walks the list and
    /// the bottom bar, and on "change box" it said nothing (movie12, spike 143).
    named_roots: PySet,
    /// Cursors on their way, at this poll, to an item not said yet (`moving`).
    moving: HashSet<i64>,
    /// Named cursors on screen that walk rows or slots drawn apart, whose places each tick
    /// samples (`sample`): the Poké Mart's and the main menu's never glide, and are not.
    sampled: PySet,
    /// Named cursor -> where it stood at the last few ticks, oldest first.
    samples: IndexMap<i64, VecDeque<(f64, f64)>>,
    /// Named cursor -> the groups of slots it has been on, each reported as a cursor of its
    /// own, so that what the grid's moves change is not taken for the details of the menu's.
    slot_groups: IndexMap<i64, HashSet<i64>>,
    /// Pointers on screen at the last poll.
    pointed: IndexSet<i64>,
    polls: i64,
    /// Layout root -> the poll it came on screen at, for layouts shown now.
    shown_since: IndexMap<i64, i64>,
}

impl CursorReader {
    /// `CursorReader(layouts, names)`: no layout files, no cursor panes, no picture names.
    pub fn new(layouts: Rc<Layouts>, names: Rc<ClassNames>) -> Self {
        CursorReader::with(layouts, names, None, false, None)
    }

    /// `CursorReader(layouts, names, files)`.
    pub fn with_files(layouts: Rc<Layouts>, names: Rc<ClassNames>, files: Rc<LayoutFiles>) -> Self {
        CursorReader::with(layouts, names, Some(files), false, None)
    }

    /// `cursor_panes` reads Sun's cursor pane and its boxes' pointers, which ORAS and X have
    /// none of. `pictures` says what the game calls what a picture draws now, by its image
    /// (`gen6.textures`), or None.
    pub fn with(
        layouts: Rc<Layouts>,
        names: Rc<ClassNames>,
        files: Option<Rc<LayoutFiles>>,
        cursor_panes: bool,
        pictures: Option<Pictures>,
    ) -> Self {
        CursorReader {
            layouts,
            names,
            files,
            pictures,
            cursor_panes,
            fits: IndexMap::new(),
            cursors: Vec::new(),
            under: IndexMap::new(),
            said: IndexMap::new(),
            spots: IndexMap::new(),
            missed: IndexMap::new(),
            picture_words: IndexMap::new(),
            key_plates: IndexMap::new(),
            moved_keys: HashSet::new(),
            frames: IndexMap::new(),
            frames_missed: IndexMap::new(),
            pending_frames: IndexMap::new(),
            moved_frames: HashSet::new(),
            choices_seen: IndexSet::new(),
            choice_words: IndexMap::new(),
            listing: IndexMap::new(),
            listing_missed: IndexMap::new(),
            steady: IndexMap::new(),
            cursor_textures: IndexMap::new(),
            over: HashSet::new(),
            named_roots: PySet::new(),
            moving: HashSet::new(),
            sampled: PySet::new(),
            samples: IndexMap::new(),
            slot_groups: IndexMap::new(),
            pointed: IndexSet::new(),
            polls: 0,
            shown_since: IndexMap::new(),
        }
    }

    /// Read the cursors among these windows from now on, as well as every one found before.
    ///
    /// Most windows are not cursors, so only those shaped as one when found are kept. A cursor
    /// once found is read on every poll after: menus are rebuilt where they were, and a sweep
    /// takes seconds to find a new one. Cursor pictures are found in the layouts themselves.
    pub fn track(&mut self, windows: impl IntoIterator<Item = i64>) -> Result<()> {
        for window in windows {
            if !self.cursors.contains(&window) && self.shaped(window)? {
                self.cursors.push(window);
            }
        }
        Ok(())
    }

    /// The pieces under each cursor on screen, as of the last poll.
    pub fn under(&self) -> Vec<Vec<ScreenText>> {
        self.under.values().cloned().collect()
    }

    /// Is there a named cursor on screen whose place each tick should sample?
    pub fn sampling(&self) -> bool {
        !self.sampled.is_empty()
    }

    /// Note where each named cursor stands now. Called every tick, so that a screen poll can
    /// tell a cursor at rest from one gliding past a slot or a row: the Box's pointer glides
    /// through each on its way, and holding what it was over for two screen polls instead left
    /// out a row or a slot moved past within half a second, and read a slot's summary before the
    /// slot (or/movie11_pokedex_and_box, spike 129).
    pub fn sample(&mut self, memory: &dyn Memory) -> Result<()> {
        for cursor in self.sampled.iter().collect::<Vec<_>>() {
            let head = memory.read(cursor + lyt::X, lyt::Y - lyt::X + 4)?;
            let place = (f32(&head, 0)?, f32(&head, lyt::Y - lyt::X)?);
            let samples = self.samples.entry(cursor).or_default();
            // `deque(maxlen=REST_TICKS)` drops its oldest when full.
            if samples.len() as i64 >= REST_TICKS {
                samples.pop_front();
            }
            samples.push_back(place);
        }
        Ok(())
    }

    /// Has this cursor stood where it is for the last ticks sampled? One not sampled yet, as
    /// the moment it comes on screen, has nothing to say it is moving.
    fn at_rest(&self, cursor: i64, place: (f64, f64)) -> bool {
        self.samples.get(&cursor).is_none_or(|samples| {
            samples
                .iter()
                .all(|&(x, y)| near(x, place.0) && near(y, place.1))
        })
    }

    /// The cursors on their way to an item they have not said yet, as of the last poll, with
    /// every group of slots theirs: the details gathered for where one was are no longer its
    /// (spike 129).
    pub fn moving(&self) -> HashSet<i64> {
        let mut out = self.moving.clone();
        for cursor in &self.moving {
            if let Some(groups) = self.slot_groups.get(cursor) {
                out.extend(groups);
            }
        }
        out
    }

    /// Layout root -> the TextBoxes of the list a cursor walks, as of the last poll, for the
    /// cursors that know theirs: a plate's rows, and the words beside those a named cursor is
    /// over. The rest of such a layout is not the menu's.
    pub fn lists(&self) -> IndexMap<i64, HashSet<i64>> {
        let mut out: IndexMap<i64, HashSet<i64>> = IndexMap::new();
        for (root, boxes) in self.listing.values() {
            // A cursor over no words, and pointing at none beside it, knows no list.
            if !boxes.is_empty() {
                let mut joined = out.get(root).cloned().unwrap_or_default();
                joined.extend(boxes);
                out.insert(*root, joined);
            }
        }
        out
    }

    /// What each cursor on screen is over, whenever that changed.
    ///
    /// `on_screen` is every piece of screen text on screen, from a screen text poll in the same
    /// pass: this reads panes through the same `Layouts`, so the two agree on the frame.
    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<Vec<CursorItem>> {
        let mut items: Vec<CursorItem> = Vec::new();
        self.moving = HashSet::new();
        for cursor in self.cursors.clone() {
            let Some(rect) = self.rect(cursor)? else {
                self.forget(cursor);
                continue;
            };
            // A cursor built into a layout frames that layout's words: Sun's keyboard keeps a
            // second, parked over the keys of the page drawn in front of it (spike 112).
            let own = self.built_into(cursor)?;
            let under = in_front(
                on_screen
                    .iter()
                    .filter(|piece| {
                        within(piece, &rect) && (own.is_none() || own == Some(piece.root))
                    })
                    .cloned()
                    .collect(),
                &rect,
            );
            let mut inside = distinct(in_rows(nearest_panel(under, &rect)));
            if inside.is_empty() {
                inside = self.pictures_inside(&rect)?;
            }
            let at = place_of(&rect)?;
            self.report(cursor, inside, &mut items, at, false);
        }
        let markers = self.markers()?;
        let gone: Vec<i64> = self
            .spots
            .keys()
            .copied()
            .filter(|marker| !markers.contains_key(marker))
            .collect();
        for marker in gone {
            self.spots.shift_remove(&marker);
            self.missed.shift_remove(&marker);
            self.forget(marker);
        }
        for (&marker, &top) in &markers {
            let Some(spot) = self.spot(marker, top)? else {
                // A pane read mid-update can sit 1,000 units off for one poll (spike 50): a
                // cursor lost for one poll is not taken for one gone, and said again.
                let missed = self.missed.get(&marker).copied().unwrap_or(0) + 1;
                self.missed.insert(marker, missed);
                if missed > 1 {
                    self.forget(marker);
                }
                continue;
            };
            self.missed.insert(marker, 0);
            let item = self.item(&spot, top, on_screen)?;
            self.report(marker, item, &mut items, Where::Pane(spot.anchor), false);
        }
        self.poll_cursor_panes(on_screen, &mut items)?;
        let keys = self.keys(on_screen)?;
        let gone: Vec<i64> = self
            .key_plates
            .keys()
            .copied()
            .filter(|picture| !keys.contains_key(picture))
            .collect();
        for picture in gone {
            self.key_plates.shift_remove(&picture);
            self.moved_keys.remove(&picture);
            self.forget(picture);
        }
        let mut alone: HashMap<i64, i64> = HashMap::new();
        for &(root, _plate) in keys.values() {
            *alone.entry(root).or_insert(0) += 1;
        }
        for (&picture, &(root, plate)) in &keys {
            if alone[&root] == 1 || self.moved_keys.contains(&picture) {
                let item = self.key_item(root, plate, picture, on_screen)?;
                self.report(picture, item, &mut items, Where::Pane(plate), false);
            }
        }
        for (window, rect) in self.moved_frame_rects()? {
            let under = in_front(
                on_screen
                    .iter()
                    .filter(|piece| within(piece, &rect))
                    .cloned()
                    .collect(),
                &rect,
            );
            let inside = distinct(in_rows(nearest_panel(under, &rect)));
            let at = place_of(&rect)?;
            self.report(window, inside, &mut items, at, false);
        }
        let choices = self.choices()?;
        let gone: Vec<i64> = self
            .choices_seen
            .iter()
            .copied()
            .filter(|window| !choices.contains_key(window))
            .collect();
        for window in gone {
            self.choices_seen.shift_remove(&window);
            self.forget(window);
        }
        for (&window, (root, on, others)) in &choices {
            let Some(on) = *on else {
                continue;
            };
            self.choices_seen.insert(window);
            let item = self.choice_item(*root, on, others, on_screen)?;
            self.report(window, item, &mut items, Where::Pane(on), false);
        }
        let plates = self.plates(on_screen, &choices.keys().copied().collect())?;
        // A picture another kind already reads is that kind's: the name entry's key cursor is
        // textured as a cursor too, and read twice it was said again every poll (opening.ctm).
        // So is a keyboard's whole layout: its second cursor picture said "“" as it opened.
        let taken: HashSet<i64> = markers.keys().chain(keys.keys()).copied().collect();
        let keyboards = PySet::of(keys.values().map(|&(root, _key)| root));
        let named = self.named(on_screen, &taken, &keyboards)?;
        let named_keys: Vec<i64> = named.keys().copied().collect();
        self.sampled
            .intersection_update(&PySet::of_dict(&named_keys));
        self.samples.retain(|cursor, _| named.contains_key(cursor));
        // Lost for one poll is not gone: rows sliding past can leave none under the plate.
        let lost: Vec<i64> = self
            .listing
            .keys()
            .copied()
            .filter(|cursor| !plates.contains_key(cursor) && !named.contains_key(cursor))
            .collect();
        for cursor in lost {
            let missed = self.listing_missed.get(&cursor).copied().unwrap_or(0) + 1;
            self.listing_missed.insert(cursor, missed);
            if missed > 1 {
                self.listing.shift_remove(&cursor);
                self.listing_missed.shift_remove(&cursor);
                self.steady.shift_remove(&cursor);
                self.over.remove(&cursor);
                for group in self.slot_groups.shift_remove(&cursor).unwrap_or_default() {
                    self.steady.shift_remove(&group);
                    self.forget(group);
                }
                self.forget(cursor);
            }
        }
        for cursor in plates.keys().chain(named.keys()) {
            self.listing_missed.shift_remove(cursor);
        }
        for (&window, (root, row, rows)) in &plates {
            let pieces: Vec<&ScreenText> = on_screen
                .iter()
                .filter(|piece| piece.root == *root)
                .collect();
            let mut boxes: HashSet<i64> = HashSet::new();
            for piece in &pieces {
                let mut any = false;
                for &r in rows {
                    if self.below(piece.r#box, r)? {
                        any = true;
                        break;
                    }
                }
                if any {
                    boxes.insert(piece.r#box);
                }
            }
            self.listing.insert(window, (*root, boxes));
            let mut chosen: Vec<ScreenText> = Vec::new();
            for piece in &pieces {
                if self.below(piece.r#box, *row)? {
                    chosen.push((*piece).clone());
                }
            }
            let item = distinct(in_rows(chosen));
            self.report_steady(window, item, &mut items, Where::Pane(*row), false);
        }
        let shown_roots: HashSet<i64> = on_screen.iter().map(|piece| piece.root).collect();
        for (&holder, &(root, walks)) in &named {
            let Some(rect) = self.named_rect(holder, root)? else {
                continue;
            };
            let mut own: Vec<ScreenText> = Vec::new();
            for piece in on_screen {
                if piece.root == root && self.below(piece.r#box, holder)? {
                    own.push(piece.clone());
                }
            }
            let mut inside: Vec<ScreenText>;
            if !own.is_empty() {
                // The Bag's select bar carries the chosen Pokémon's name, level and ABLE! over
                // the slot's own ABLE!, which lies nearer its middle (spike 139).
                inside = distinct(in_rows(own.clone()));
            } else {
                let under = in_front(
                    on_screen
                        .iter()
                        .filter(|piece| within(piece, &rect))
                        .cloned()
                        .collect(),
                    &rect,
                );
                inside = distinct(in_rows(nearest_panel(under, &rect)));
            }
            if !inside.is_empty() {
                self.over.insert(holder);
            } else if walks && !self.over.contains(&holder) {
                // The Mart's boxes point at their words from beside them. The PC Box's cursor is
                // over its menu's words, and leaves for the grid of Pokémon a poll before they go,
                // where the word on its row to its right was said (or/movie6_pc, 2026-09-27).
                inside = distinct(in_rows(beside(on_screen, &rect, root)));
            }
            let (mut at_slot, mut alone, mut reporter) = (false, false, holder);
            if inside.is_empty()
                && walks
                && let Some(files) = self.files.clone()
            {
                // The PC Box's pointer over its grid of Pokémon and its party, where a slot holds
                // an icon and no words: said by where it is (spike 129).
                let file = files.matching(&self.layouts.pane_names(root));
                let (x, y) = ((rect.left + rect.right) / 2.0, (rect.low + rect.high) / 2.0);
                // With its layout's words all hidden, as under the Box's box list, a cursor looks
                // only for another layout's button: the list's folders read as the Box's slots,
                // "row 2, column 1, empty", and left the box's name above them unsaid (movie12).
                let mut found = if !shown_roots.contains(&root) {
                    None
                } else {
                    slots::slot(&self.layouts, file.as_deref(), root, x, y, on_screen)?
                };
                if found.is_none() {
                    found = slots::button(&self.layouts, &files, root, x, y, rect.top)?;
                }
                if let Some((touch, said, was_alone)) = found {
                    alone = was_alone;
                    inside = vec![ScreenText {
                        top: rect.top,
                        root,
                        parent: touch,
                        ..ScreenText::new(touch, said, false)
                    }];
                    at_slot = true;
                    reporter = u32(&self.layouts.pane(touch)?, lyt::PARENT)?;
                    self.slot_groups.entry(holder).or_default();
                    // The same pointer going on from the party to the box has moved, and brings
                    // the summary of what it moved to, where an item opening brings none.
                    if !self.said.contains_key(&reporter)
                        && (self.said.contains_key(&holder)
                            || self.slot_groups[&holder]
                                .iter()
                                .any(|group| self.said.contains_key(group)))
                    {
                        self.said.insert(reporter, (Vec::new(), Where::Nothing));
                    }
                    self.slot_groups.entry(holder).or_default().insert(reporter);
                }
            }
            // Where the pointer went on to another group, what each other one last said is
            // forgotten, so that coming back to the slot it left says it again.
            let mut others: HashSet<i64> = HashSet::from([holder]);
            if let Some(groups) = self.slot_groups.get(&holder) {
                others.extend(groups);
            }
            others.remove(&reporter);
            for other in others {
                if self.said.contains_key(&other) {
                    self.said.insert(other, (Vec::new(), Where::Nothing));
                }
                self.steady.shift_remove(&other);
            }
            let mut rows: HashSet<(i64, i64)> = HashSet::new();
            if walks && !at_slot {
                for piece in &inside {
                    for row in self.alike(piece.root, piece.parent)? {
                        rows.insert((piece.root, row));
                    }
                }
            }
            self.listing.insert(
                holder,
                (
                    root,
                    on_screen
                        .iter()
                        .filter(|piece| rows.contains(&(piece.root, piece.parent)))
                        .map(|piece| piece.r#box)
                        .collect(),
                ),
            );
            let at = place_of(&rect)?;
            if rows.len() > 1 || at_slot {
                self.sampled.add(holder);
                // The PC Box's cursor glides from row to row and slot to slot over several frames,
                // through each one's own place, and a poll on the way said the rows it was leaving
                // and passing (or/movie6_pc, 2026-09-27): what it is over is said once it is at
                // rest, as the ticks between polls say (`sample`).
                if !self.at_rest(holder, centre(&rect)) {
                    self.moving.insert(holder);
                    continue;
                }
                let held = self.said.get(&reporter).map(|said| said.1);
                if held == Some(at) {
                    // Words rewritten under a cursor that has not moved: a Secret Base's list
                    // writes the next list's words into its rows a poll before it hides the last
                    // one's, and the category and the item were said together (spike 154).
                    self.report_steady(reporter, inside, &mut items, at, alone);
                } else {
                    self.report(reporter, inside, &mut items, at, alone);
                }
            } else if !own.is_empty() {
                // The select bar's name is written a poll before its level, and moving from Ralts
                // said "Wingull L14 ABLE!" with Ralts's level (spike 139).
                self.report_steady(holder, inside, &mut items, at, false);
            } else {
                // Not held: the Mart's cursor jumps whole from row to row, and holding it said the
                // new item's count and description before the item.
                self.report(holder, inside, &mut items, at, false);
            }
        }
        self.point(on_screen, &mut items)?;
        Ok(items)
    }

    /// Say the button a pointer is about to tap, where no other cursor claims its layout.
    ///
    /// Wally's catching tutorial is the one battle the player watches rather than plays, and has
    /// no battle cursor: the game shows his taps with a picture in a layout of its own, under a
    /// pane it places on the button he is about to tap, slides the picture onto it and grows it
    /// for the tap (spike 96). Its item is the button whose touch area holds that place, said
    /// once it has read the same for two polls, and meanwhile the button's layout is the
    /// pointer's, so the four moves are not read out whole.
    fn point(&mut self, on_screen: &[ScreenText], items: &mut Vec<CursorItem>) -> Result<()> {
        self.note_shown()?;
        let pointers = self.pointers()?;
        let gone: Vec<i64> = self
            .pointed
            .iter()
            .copied()
            .filter(|pointer| !pointers.contains_key(pointer))
            .collect();
        for pointer in gone {
            self.pointed.shift_remove(&pointer);
            self.steady.shift_remove(&pointer);
            self.forget(pointer);
        }
        if pointers.is_empty() {
            return Ok(());
        }
        let claimed: HashSet<i64> = self
            .under
            .iter()
            .filter(|(cursor, _pieces)| !pointers.contains_key(*cursor))
            .flat_map(|(_cursor, pieces)| pieces.iter().map(|piece| piece.root))
            .collect();
        for (&pointer, &(x, y, top)) in &pointers {
            self.pointed.insert(pointer);
            let found = self.touched(x, y, top)?;
            let Some((root, button)) = found.filter(|found| !claimed.contains(&found.0)) else {
                self.under.shift_remove(&pointer);
                self.steady.shift_remove(&pointer);
                continue;
            };
            let mut near: Vec<ScreenText> = Vec::new();
            for piece in on_screen {
                if self.below(piece.r#box, button)? {
                    near.push(piece.clone());
                }
            }
            let mut inside = distinct(in_rows(near));
            if inside.is_empty() {
                let Some(said) = self.pictured(root, button, on_screen, None)? else {
                    continue;
                };
                inside = vec![ScreenText {
                    top,
                    root,
                    parent: button,
                    ..ScreenText::new(button, said, false)
                }];
            }
            self.report_steady(pointer, inside, items, Where::Pane(button), false);
        }
        Ok(())
    }

    /// Picture -> (x, y, on the top screen) of the place it points at, for every small picture
    /// shown alone in a layout but for the one plain pane that places it: Wally's pointer, whose
    /// pane stands where he is about to tap while the picture slides onto it.
    fn pointers(&self) -> Result<IndexMap<i64, (f64, f64, bool)>> {
        let mut found: IndexMap<i64, (f64, f64, bool)> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            if tree.len() != 3 {
                continue;
            }
            let holders: Vec<i64> = tree
                .iter()
                .filter(|&(&pane, name)| name == lyt::PANE_CLASS && pane != root)
                .map(|(&pane, _name)| pane)
                .collect();
            let pictures: Vec<i64> = tree
                .iter()
                .filter(|&(_pane, name)| name == lyt::PICTURE_CLASS)
                .map(|(&pane, _name)| pane)
                .collect();
            if holders.len() != 1 || pictures.len() != 1 {
                continue;
            }
            let (holder, picture) = (holders[0], pictures[0]);
            if self.parent(picture)? != holder {
                continue;
            }
            let (width, height) = self.size(picture)?;
            if !(0.0 < width && width <= MARKER_SIZE && 0.0 < height && height <= MARKER_SIZE) {
                continue;
            }
            let placement = self.layouts.placement(picture)?;
            let Some(placement) = placement.filter(|placement| placement.shown) else {
                continue;
            };
            let (x, y) = self.place(holder)?;
            found.insert(picture, (x, y, placement.top));
        }
        Ok(found)
    }

    /// (root, button) for the touch area shown at this place on this screen, where a button
    /// is what holds the touch area, as a move's plate holds its own.
    ///
    /// Any layout shown, words or none: the battle's command menu is all pictures. Of two, the
    /// one that came on screen last, as the one before can still be up behind it: the bag's
    /// LAST USED ITEM stood under Wally's tap on USE (wally_again.ctm replay); then the smaller.
    fn touched(&self, x: f64, y: f64, top: bool) -> Result<Option<(i64, i64)>> {
        let mut found: Vec<(i64, f64, i64, i64)> = Vec::new();
        for (&root, &since) in &self.shown_since {
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::BOUNDING_CLASS {
                    continue;
                }
                let rect = self.window_rect(pane)?;
                let Some(rect) = rect.filter(|rect| rect.top == top && on((x, y), rect)) else {
                    continue;
                };
                let area = (rect.right - rect.left) * (rect.high - rect.low);
                found.push((-since, area, root, self.parent(pane)?));
            }
        }
        let Some(&first) = found.first() else {
            return Ok(None);
        };
        // `min` keeps the first of the least, and a tuple with a NaN in it is less than none.
        let mut best = first;
        for &item in &found[1..] {
            if found_lt(&item, &best) {
                best = item;
            }
        }
        Ok(Some((best.2, best.3)))
    }

    /// Remember the poll each layout came on screen at, for layouts shown now.
    fn note_shown(&mut self) -> Result<()> {
        self.polls += 1;
        let mut shown = PySet::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if placement.is_some_and(|placement| placement.shown) {
                shown.add(root);
            }
        }
        self.shown_since = shown
            .iter()
            .map(|root| {
                (
                    root,
                    self.shown_since.get(&root).copied().unwrap_or(self.polls),
                )
            })
            .collect();
        Ok(())
    }

    /// Report what a cursor is over once it has read the same on two polls in a row.
    ///
    /// The Pokédex slides its rows a row's height and back as it scrolls, rewriting their words
    /// in between, and for that poll the row under its plate is one the list had hidden, with
    /// whatever words it was left with. What is under meanwhile is still the cursor's.
    fn report_steady(
        &mut self,
        cursor: i64,
        inside: Vec<ScreenText>,
        items: &mut Vec<CursorItem>,
        r#where: Where,
        quiet: bool,
    ) {
        self.under.insert(cursor, inside.clone());
        let seen = (
            inside.iter().map(|piece| piece.text.clone()).collect(),
            r#where,
        );
        if self.steady.get(&cursor) != Some(&seen) {
            self.steady.insert(cursor, seen);
            return;
        }
        self.report(cursor, inside, items, r#where, quiet);
    }

    /// Window -> (root, the row it sits on, the rows of its list), for every window shown with
    /// no panes of its own that sits on a row of a list: the Pokédex's plate, which stays where
    /// it is while the rows scroll under it (spike 95).
    ///
    /// A window directly under its root is a frame, and one sliding between choices a choice's
    /// cursor, each read as such. A window as tall as several rows frames a panel, and a window
    /// inside a row is part of that row. While the rows slide, the one nearest the plate's middle
    /// is the one it sits on.
    fn plates(
        &self,
        on_screen: &[ScreenText],
        choices: &HashSet<i64>,
    ) -> Result<IndexMap<i64, Plate>> {
        let mut found: IndexMap<i64, Plate> = IndexMap::new();
        let mut roots = PySet::new();
        for piece in on_screen {
            roots.add(piece.root);
        }
        for root in roots.iter().collect::<Vec<_>>() {
            let mut windows: Vec<i64> = Vec::new();
            for (&pane, name) in &self.layouts.tree(root) {
                if name == lyt::WINDOW_CLASS
                    && !self.cursors.contains(&pane)
                    && !choices.contains(&pane)
                    && self.parent(pane)? != root
                    && self.layouts.children(root, pane).is_empty()
                {
                    windows.push(pane);
                }
            }
            if windows.is_empty() {
                continue;
            }
            let pieces: Vec<ScreenText> = on_screen
                .iter()
                .filter(|piece| piece.root == root)
                .cloned()
                .collect();
            let lists = self.rows(root, &pieces)?;
            for &window in &windows {
                let Some(rect) = self.window_rect(window)? else {
                    continue;
                };
                let middle = (rect.low + rect.high) / 2.0;
                for rows in &lists {
                    let height = f32(&self.layouts.pane(rows[0])?, self.layouts.shape.height)?;
                    if rect.high - rect.low > PLATE_ROWS * height {
                        continue;
                    }
                    let mut inside = false;
                    for &row in rows {
                        if self.below(window, row)? {
                            inside = true;
                            break;
                        }
                    }
                    if inside {
                        continue;
                    }
                    let mut on_it: Vec<i64> = Vec::new();
                    for &row in rows {
                        if on(self.place(row)?, &rect) {
                            on_it.push(row);
                        }
                    }
                    if !on_it.is_empty() {
                        // `min(on, key=...)`: the first of the nearest.
                        let mut row = on_it[0];
                        let mut best = (self.place(row)?.1 - middle).abs();
                        for &other in &on_it[1..] {
                            let key = (self.place(other)?.1 - middle).abs();
                            if key < best {
                                row = other;
                                best = key;
                            }
                        }
                        found.insert(window, (root, row, rows.clone()));
                        break;
                    }
                }
            }
        }
        Ok(found)
    }

    /// The lists in a layout: three or more plain panes under one parent, each holding words
    /// on screen, all at one x and of one height, as the Pokédex draws its rows.
    fn rows(&self, root: i64, pieces: &[ScreenText]) -> Result<Vec<Vec<i64>>> {
        let tree = self.layouts.tree(root);
        let mut holding: IndexSet<i64> = IndexSet::new();
        for piece in pieces {
            let mut pane = self.parent(piece.r#box)?;
            for _ in 0..MAX_DEPTH {
                if pane == 0 || pane == root {
                    break;
                }
                if class_of(&tree, pane) == Some(lyt::PANE_CLASS) {
                    holding.insert(pane);
                }
                pane = self.parent(pane)?;
            }
        }
        let mut siblings: IndexMap<i64, Vec<i64>> = IndexMap::new();
        for &pane in &holding {
            siblings.entry(self.parent(pane)?).or_default().push(pane);
        }
        let mut lists: Vec<Vec<i64>> = Vec::new();
        for rows in siblings.values() {
            if (rows.len() as i64) < LIST_ROWS {
                continue;
            }
            let (x, _y) = self.place(rows[0])?;
            let height = self.size(rows[0])?.1;
            let mut alike = true;
            for &row in rows {
                if !(near(self.place(row)?.0, x) && near(self.size(row)?.1, height)) {
                    alike = false;
                    break;
                }
            }
            if alike {
                lists.push(rows.clone());
            }
        }
        Ok(lists)
    }

    /// Pane -> (root, whether it walks a list), for every pane shown that is drawn as a
    /// cursor, by the name its designer gave the texture.
    ///
    /// A picture marks a row among the words beside it, which are its list: the picture, or the
    /// plain pane holding it when that holds nothing else, as the Poké Mart keeps the two halves
    /// of `shop_cursor.bclim` (spike 95). A window frames its item as an app menu's cursor does,
    /// and its layout is the menu's whole, as the main menu's `cursor_lt.bclim` frame's is: known
    /// from the poll it is seen on, where a frame is otherwise known only once it has moved
    /// (spike 96). Pictures `taken` by another kind of cursor are left to it, and the layouts of
    /// `keyboards` to their key cursors.
    fn named(
        &mut self,
        on_screen: &[ScreenText],
        taken: &HashSet<i64>,
        keyboards: &PySet,
    ) -> Result<IndexMap<i64, (i64, bool)>> {
        let mut found: IndexMap<i64, (i64, bool)> = IndexMap::new();
        if self.files.is_none() && !self.cursor_panes {
            return Ok(found);
        }
        self.named_roots
            .intersection_update(&self.layouts.roots_set());
        let mut shown = PySet::new();
        for piece in on_screen {
            shown.add(piece.root);
        }
        let walked = shown.union(&self.named_roots).difference(keyboards);
        for root in walked.iter().collect::<Vec<_>>() {
            let file = match &self.files {
                None => None,
                Some(files) => files.matching(&self.layouts.pane_names(root)),
            };
            if file.is_none() && !self.cursor_panes {
                continue;
            }
            let tree = self.layouts.tree(root);
            for (&pane, name) in &tree {
                if name != lyt::PICTURE_CLASS && name != lyt::WINDOW_CLASS {
                    continue;
                }
                if taken.contains(&pane) || self.cursors.contains(&pane) {
                    continue;
                }
                if !self.drawn_as_cursor(root, pane, file.as_deref())? {
                    continue;
                }
                let mut holder = pane;
                let parent = self.parent(pane)?;
                if name == lyt::PICTURE_CLASS
                    && parent != root
                    && class_of(&tree, parent) == Some(lyt::PANE_CLASS)
                {
                    let mut all = true;
                    for child in self.layouts.children(root, parent) {
                        if !self.drawn_as_cursor(root, child, file.as_deref())? {
                            all = false;
                            break;
                        }
                    }
                    if all {
                        holder = parent;
                        let above = self.parent(holder)?;
                        if above != root
                            && class_of(&tree, above) == Some(lyt::PANE_CLASS)
                            && self.layouts.children(root, above) == [holder]
                        {
                            // A Secret Base's lists keep their cursor's pane, 30 by 40, alone in a
                            // pane the game moves onto the chosen row at the row's own size, and the
                            // row's words reach out past the smaller one (spike 154).
                            holder = above;
                        }
                    }
                }
                let placement = self.layouts.placement(holder)?;
                if placement.is_some_and(|placement| placement.shown)
                    && !self.parked(holder, root)?
                {
                    found.insert(holder, (root, name == lyt::PICTURE_CLASS));
                    self.named_roots.add(root);
                }
            }
        }
        Ok(found)
    }

    /// Where a named cursor is drawn: its holder's place, a row of its pictures tall where a
    /// plain pane holds pictures shorter than itself. A Secret Base's YES and NO keep their
    /// cursor's two halves, 20 tall, in a pane of 40, two rows, which said "YES NO" (spike 154).
    fn named_rect(&self, holder: i64, root: i64) -> Result<Option<Rect>> {
        let rect = self.window_rect(holder)?;
        let tree = self.layouts.tree(root);
        let Some(rect) = rect else {
            return Ok(None);
        };
        if class_of(&tree, holder) != Some(lyt::PANE_CLASS) {
            return Ok(Some(rect));
        }
        let children = self.layouts.children(root, holder);
        if children.is_empty()
            || children
                .iter()
                .any(|&child| class_of(&tree, child) != Some(lyt::PICTURE_CLASS))
        {
            return Ok(Some(rect));
        }
        let mut drawn: Vec<Rect> = Vec::new();
        for &child in &children {
            if let Some(found) = self.window_rect(child)? {
                drawn.push(found);
            }
        }
        // `max(..., default=0.0)`: the first of the greatest, so a NaN first stays.
        let mut tall = 0.0;
        for (index, found) in drawn.iter().enumerate() {
            let height = found.high - found.low;
            if index == 0 || height > tall {
                tall = height;
            }
        }
        let middle = (rect.low + rect.high) / 2.0;
        if !(0.0 < tall && tall < rect.high - rect.low) {
            return Ok(Some(rect));
        }
        Ok(Some(Rect {
            top: rect.top,
            left: rect.left,
            right: rect.right,
            low: middle - tall / 2.0,
            high: middle + tall / 2.0,
        }))
    }

    /// Is the pane drawn wholly off its screen, judged from its layout's root? The Bag keeps
    /// its select bar at x -320 while the player walks its items, left of the bottom screen, and
    /// taken for a cursor there it said the words nearest it (spike 139).
    fn parked(&self, pane: i64, root: i64) -> Result<bool> {
        let Some(rect) = self.window_rect(pane)? else {
            return Ok(false);
        };
        let (x, y) = self.place(root)?;
        let half_width = SCREEN_HALF_WIDTH[usize::from(rect.top)];
        Ok(rect.right - x < -half_width
            || rect.left - x > half_width
            || rect.high - y < -SCREEN_HALF_HEIGHT
            || rect.low - y > SCREEN_HALF_HEIGHT)
    }

    /// Is this picture or window drawn with a texture its designer named a cursor? A pane
    /// that is neither has no texture in its layout's file, and is not. Without the file, as in
    /// Sun, a picture is a cursor by its place: a box's pointer.
    fn drawn_as_cursor(&mut self, root: i64, pane: i64, file: Option<&LayoutFile>) -> Result<bool> {
        if !self.cursor_textures.contains_key(&(root, pane)) {
            let Some(file) = file else {
                let drawn = self.points_in_box(root, pane)?;
                self.cursor_textures.insert((root, pane), drawn);
                return Ok(drawn);
            };
            let name = self.pane_name(pane)?;
            let textures = match file.textures.get(&name) {
                Some(textures) if !textures.is_empty() => Some(textures),
                _ => file
                    .frames
                    .get(&name)
                    .filter(|textures| !textures.is_empty()),
            };
            let drawn = textures.is_some_and(|textures| {
                textures.iter().any(|texture| {
                    let stem = match texture.rsplit_once('.') {
                        Some((stem, _)) => stem,
                        None => texture,
                    };
                    CURSOR_TEXTURES.contains(&stem)
                        || stem.split('_').any(|part| CURSOR_TEXTURES.contains(&part))
                })
            });
            self.cursor_textures.insert((root, pane), drawn);
        }
        Ok(self.cursor_textures[&(root, pane)])
    }

    /// Is this picture alone in a plain pane that stands in a window among its words?
    ///
    /// Sun's talk window points at the player's answers so: `arrow_00` in `Null_arrow`, beside
    /// "Yeah!" and "...I guess" in `win_00`, moved from one's row to the other's, and read as
    /// screen text the box was said whole and never the move (sun_8.ctm). No other picture in
    /// Sun's menus so far stands so; its layout files, which would name its texture, are not
    /// read.
    fn points_in_box(&self, root: i64, pane: i64) -> Result<bool> {
        let tree = self.layouts.tree(root);
        let holder = self.parent(pane)?;
        let r#box = self.parent(holder)?;
        Ok(class_of(&tree, pane) == Some(lyt::PICTURE_CLASS)
            && class_of(&tree, holder) == Some(lyt::PANE_CLASS)
            && self.layouts.children(root, holder) == [pane]
            && class_of(&tree, r#box) == Some(lyt::WINDOW_CLASS)
            && self
                .layouts
                .children(root, r#box)
                .iter()
                .any(|&child| class_of(&tree, child) == Some(lyt::TEXTBOX_CLASS)))
    }

    /// Window -> (root, the choice it sits on, the other choices), for every window shown
    /// beside two or more plain panes of one size in places of their own: the opening's choice of
    /// boy or girl, whose cursor is a window the game slides from one choice's place to another's.
    ///
    /// The choice is None while the window is between two, where there is nothing new to say. A
    /// window directly under its layout's root is a frame, read as one; a window holding panes of
    /// its own is a box the screen is drawn in, as a message window is, not a cursor; and the
    /// frame round a choice of its own stands among that choice's pictures, with no panes beside
    /// it to sit on.
    fn choices(&self) -> Result<IndexMap<i64, Choice>> {
        let mut found: IndexMap<i64, Choice> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            for (&pane, name) in &tree {
                if name != lyt::WINDOW_CLASS || self.cursors.contains(&pane) {
                    continue;
                }
                let parent = self.parent(pane)?;
                if parent == 0 || parent == root || !self.layouts.children(root, pane).is_empty() {
                    continue;
                }
                let choices: Vec<i64> = self
                    .layouts
                    .children(root, parent)
                    .into_iter()
                    .filter(|&child| class_of(&tree, child) == Some(lyt::PANE_CLASS))
                    .collect();
                if choices.len() < 2 {
                    continue;
                }
                let mut sizes: Vec<(f64, f64)> = Vec::new();
                for &choice in &choices {
                    sizes.push(self.size(choice)?);
                }
                if distinct_count(&sizes) != 1 {
                    continue;
                }
                let mut places: Vec<(f64, f64)> = Vec::new();
                for &choice in &choices {
                    places.push(self.place(choice)?);
                }
                if distinct_count(&places) != places.len() {
                    continue;
                }
                let placement = self.layouts.placement(pane)?;
                if !placement.is_some_and(|placement| placement.shown) {
                    continue;
                }
                let head = self.layouts.pane(pane)?;
                let (x, y) = (
                    f32(&head, self.layouts.shape.x)?,
                    f32(&head, self.layouts.shape.y)?,
                );
                let on_it: Vec<i64> = choices
                    .iter()
                    .zip(&places)
                    .filter(|&(_choice, &(at_x, at_y))| near(at_x, x) && near(at_y, y))
                    .map(|(&choice, _place)| choice)
                    .collect();
                if on_it.len() > 1 {
                    continue;
                }
                found.insert(
                    pane,
                    (
                        root,
                        on_it.first().copied(),
                        choices
                            .iter()
                            .copied()
                            .filter(|choice| !on_it.contains(choice))
                            .collect(),
                    ),
                );
            }
        }
        Ok(found)
    }

    /// What a choice under a sliding window is: the words under it, or the words of the
    /// textures the other choices do not have.
    ///
    /// The first picture's texture, which names a battle's buttons, would say the white backing
    /// both portraits sit on. What tells the choices apart is what one has and the others do not.
    fn choice_item(
        &mut self,
        root: i64,
        choice: i64,
        others: &[i64],
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let mut near: Vec<ScreenText> = Vec::new();
        for piece in on_screen {
            if self.below(piece.r#box, choice)? {
                near.push(piece.clone());
            }
        }
        let inside = distinct(in_rows(near));
        if !inside.is_empty() {
            return Ok(inside);
        }
        let mut theirs: HashSet<String> = HashSet::new();
        for &other in others {
            theirs.extend(self.words_under(root, other)?);
        }
        let mut said: Vec<String> = Vec::new();
        for word in self.words_under(root, choice)? {
            if !theirs.contains(&word) && !said.contains(&word) {
                said.push(word);
            }
        }
        if said.is_empty() {
            return Ok(Vec::new());
        }
        let placement = self.layouts.placement(choice)?;
        Ok(vec![ScreenText {
            top: placement.is_some_and(|placement| placement.top),
            root,
            parent: choice,
            ..ScreenText::new(choice, said.join(" "), false)
        }])
    }

    /// The words of the textures of every picture under a choice, from its layout's file.
    fn words_under(&mut self, root: i64, choice: i64) -> Result<Vec<String>> {
        if let Some(known) = self.choice_words.get(&(root, choice)) {
            return Ok(known.clone());
        }
        let file = self
            .files
            .as_ref()
            .and_then(|files| files.matching(&self.layouts.pane_names(root)));
        let Some(file) = file else {
            return Ok(Vec::new());
        };
        let mut said: Vec<String> = Vec::new();
        for (&pane, name) in &self.layouts.tree(root) {
            if name != lyt::PICTURE_CLASS || !self.below(pane, choice)? {
                continue;
            }
            if let Some(texture) = self.texture(pane, &file)? {
                said.extend(py::split(&words(&texture)).into_iter().map(String::from));
            }
        }
        self.choice_words.insert((root, choice), said.clone());
        Ok(said)
    }

    /// A row of a list and the rows drawn like it: its parent's children of its class, at its
    /// x and of its size. The Poké Mart's rows share one window; each row of the PC Box's menu for
    /// a chosen Pokémon is a picture of its own (or/movie6_pc, 2026-09-27).
    fn alike(&self, root: i64, row: i64) -> Result<HashSet<i64>> {
        let tree = self.layouts.tree(root);
        let parent = self.parent(row)?;
        let ((x, _y), (width, height)) = (self.place(row)?, self.size(row)?);
        let mut alike: HashSet<i64> = HashSet::from([row]);
        for pane in self.layouts.children(root, parent) {
            if class_of(&tree, pane) == class_of(&tree, row)
                && near(self.place(pane)?.0, x)
                && near(self.size(pane)?.0, width)
                && near(self.size(pane)?.1, height)
            {
                alike.insert(pane);
            }
        }
        Ok(alike)
    }

    /// Where a pane is placed in its parent.
    fn place(&self, pane: i64) -> Result<(f64, f64)> {
        let head = self.layouts.pane(pane)?;
        Ok((
            f32(&head, self.layouts.shape.x)?,
            f32(&head, self.layouts.shape.y)?,
        ))
    }

    fn size(&self, pane: i64) -> Result<(f64, f64)> {
        let head = self.layouts.pane(pane)?;
        Ok((
            f32(&head, self.layouts.shape.width)?,
            f32(&head, self.layouts.shape.height)?,
        ))
    }

    /// `u32(self._layouts.pane(pane), lyt.PARENT)`.
    fn parent(&self, pane: i64) -> Result<i64> {
        u32(&self.layouts.pane(pane)?, lyt::PARENT)
    }

    /// A pane's name as its layout file gives it: the bytes up to the first NUL, decoded as ASCII.
    fn pane_name(&self, pane: i64) -> Result<String> {
        let raw = self.layouts.name(pane)?;
        let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        Ok(py::decode_ascii(&raw[..end]))
    }

    /// Window -> its rectangle, for every window directly under a root that is shown now and
    /// has moved on its own from one resting place to another: the main menu's cursor, and not
    /// the border of a panel, the rows of a list scrolling together, or a frame sliding in or
    /// out, whose place changes every poll and rests only where it stops.
    ///
    /// A window lost for one poll is not taken for one gone: a pane read mid-update can sit off
    /// for a poll (spike 50), and a cursor forgotten then would have to move again to be heard.
    fn moved_frame_rects(&mut self) -> Result<IndexMap<i64, Rect>> {
        let frames = self.frames_shown()?;
        let gone: Vec<i64> = self
            .frames
            .keys()
            .copied()
            .filter(|window| !frames.contains_key(window))
            .collect();
        for window in gone {
            let missed = self.frames_missed.get(&window).copied().unwrap_or(0) + 1;
            self.frames_missed.insert(window, missed);
            if missed > 1 {
                self.frames.shift_remove(&window);
                self.frames_missed.shift_remove(&window);
                self.pending_frames.shift_remove(&window);
                self.moved_frames.remove(&window);
                self.forget(window);
            }
        }
        let mut moved: IndexMap<i64, i64> = IndexMap::new();
        for (&window, (root, rect)) in &frames {
            let spot = spot_of(rect)?;
            let last = self.frames.get(&window).copied();
            self.frames_missed.insert(window, 0);
            let Some(last) = last.filter(|last| last.0 == spot) else {
                self.frames.insert(window, (spot, 1));
                if let Some(last) = last {
                    moved.insert(window, *root);
                    if last.1 >= FRAME_REST {
                        self.pending_frames.insert(window, spot);
                    } else {
                        self.pending_frames.shift_remove(&window);
                    }
                }
                continue;
            };
            self.frames.insert(window, (spot, last.1 + 1));
            if last.1 + 1 >= FRAME_REST && self.pending_frames.get(&window) == Some(&spot) {
                self.moved_frames.insert(window);
                self.pending_frames.shift_remove(&window);
            }
        }
        let mut together: HashMap<i64, i64> = HashMap::new();
        for &root in moved.values() {
            *together.entry(root).or_insert(0) += 1;
        }
        for (window, root) in &moved {
            if together[root] > 1 {
                self.pending_frames.shift_remove(window);
            }
        }
        Ok(frames
            .iter()
            .filter(|(window, _frame)| self.moved_frames.contains(window))
            .map(|(&window, &(_root, rect))| (window, rect))
            .collect())
    }

    /// Window -> (root, rectangle) for every window shown directly under a tracked root, other
    /// than one alone in its layout, which is a cursor of the first kind.
    fn frames_shown(&self) -> Result<IndexMap<i64, (i64, Rect)>> {
        let mut frames: IndexMap<i64, (i64, Rect)> = IndexMap::new();
        for root in self.layouts.roots() {
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::WINDOW_CLASS || self.cursors.contains(&pane) {
                    continue;
                }
                if self.parent(pane)? != root {
                    continue;
                }
                if let Some(rect) = self.window_rect(pane)? {
                    frames.insert(pane, (root, rect));
                }
            }
        }
        Ok(frames)
    }

    /// The words of the key a cursor is in: its own, or those of its picture's texture.
    fn key_item(
        &mut self,
        root: i64,
        plate: i64,
        cursor: i64,
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let inside = distinct(in_rows(
            on_screen
                .iter()
                .filter(|piece| piece.root == root && piece.parent == plate)
                .cloned(),
        ));
        if !inside.is_empty() {
            return Ok(inside);
        }
        let Some(said) = self.pictured(root, plate, on_screen, Some(cursor))? else {
            return Ok(Vec::new());
        };
        Ok(vec![ScreenText {
            root,
            parent: plate,
            ..ScreenText::new(plate, said, false)
        }])
    }

    /// Picture -> (root, key) for every picture shown in a key's touch area, where a key is a
    /// picture or a window holding a touch area, as the name entry's keyboard draws its keys.
    ///
    /// One of these is taken for a cursor while it is alone in its layout, or once it has moved
    /// from key to key: an icon standing in a button's touch area stays where it is. Where the
    /// layout's file is known, only a picture drawn as a cursor is one, as the name entry's is:
    /// the PC Box keeps each party Pokémon's icon in its slot's touch area, which made it a
    /// keyboard where no named cursor is looked for, and a lone Pokémon's icon a key cursor
    /// said as "poke icon" (or/movie6_pc, 2026-09-27).
    fn keys(&mut self, on_screen: &[ScreenText]) -> Result<IndexMap<i64, (i64, i64)>> {
        let mut found: IndexMap<i64, (i64, i64)> = IndexMap::new();
        let mut roots = PySet::new();
        for piece in on_screen {
            roots.add(piece.root);
        }
        for root in roots.iter().collect::<Vec<_>>() {
            let tree = self.layouts.tree(root);
            let file = self
                .files
                .as_ref()
                .and_then(|files| files.matching(&self.layouts.pane_names(root)));
            for (&pane, name) in &tree {
                if name != lyt::PICTURE_CLASS {
                    continue;
                }
                if file.is_some() && !self.drawn_as_cursor(root, pane, file.as_deref())? {
                    continue;
                }
                let touch = self.parent(pane)?;
                if class_of(&tree, touch) != Some(lyt::BOUNDING_CLASS) {
                    continue;
                }
                let plate = self.parent(touch)?;
                if !matches!(
                    class_of(&tree, plate),
                    Some(lyt::PICTURE_CLASS | lyt::WINDOW_CLASS)
                ) {
                    continue;
                }
                let placement = self.layouts.placement(pane)?;
                if !placement.is_some_and(|placement| placement.shown) {
                    continue;
                }
                if self.key_plates.get(&pane).copied().unwrap_or(plate) != plate {
                    self.moved_keys.insert(pane);
                }
                self.key_plates.insert(pane, plate);
                found.insert(pane, (root, plate));
            }
        }
        Ok(found)
    }

    /// Report what the cursor is over, whenever the words or the place they are in changed.
    ///
    /// The place matters as much as the words. A party screen with five empty slots gives every
    /// one of them the same name, so comparing words alone said it once and then nothing however
    /// far the player arrowed, until they landed on a Pokemon and came back (the user, 2026-09-16).
    /// A cursor that has moved to another item has news even when the item reads the same.
    fn report(
        &mut self,
        cursor: i64,
        inside: Vec<ScreenText>,
        items: &mut Vec<CursorItem>,
        r#where: Where,
        quiet: bool,
    ) {
        self.under.insert(cursor, inside.clone());
        if inside.is_empty() {
            return;
        }
        let said = (
            inside.iter().map(|piece| piece.text.clone()).collect(),
            r#where,
        );
        if self.said.get(&cursor) != Some(&said) {
            items.push(CursorItem {
                cursor,
                pieces: inside,
                opened: !self.said.contains_key(&cursor),
                quiet,
            });
            self.said.insert(cursor, said);
        }
    }

    fn forget(&mut self, cursor: i64) {
        self.under.shift_remove(&cursor);
        self.said.shift_remove(&cursor);
    }

    /// Is this a window alone under the root of its layout, holding nothing of its own? In
    /// Sun, alone under a Parts pane, the root of a layout built into another, is alone too.
    ///
    /// A window holding panes is a box its words are drawn in: the Poké Mart's Yes and No are
    /// two TextBoxes in one, alone under its root, and read as a cursor it said "Yes No" and
    /// never moved (spike 98). The corners of an app menu's cursor hold nothing.
    fn shaped(&self, window: i64) -> Result<bool> {
        if self.names.of_object(window)?.as_deref() != Some(lyt::WINDOW_CLASS) {
            return Ok(false);
        }
        let head = self.layouts.pane(window)?;
        if u32(&head, lyt::CHILD_COUNT)? != 0 {
            return Ok(false);
        }
        let root = u32(&head, lyt::PARENT)?;
        if root == 0 {
            return Ok(false);
        }
        let head = self.layouts.pane(root)?;
        if u32(&head, lyt::CHILD_COUNT)? != 1 {
            return Ok(false);
        }
        if u32(&head, lyt::PARENT)? == 0 {
            return Ok(true);
        }
        // Sun's keyboard builds its cursor as a layout of its own inside the keys' layout: the
        // window alone under a Parts pane, moved and sized onto the chosen key (spike 112).
        Ok(self.cursor_panes
            && self.names.of_vtable(u32(&head, 0)?)?.as_deref() == Some(lyt::PARTS_CLASS))
    }

    /// Where the cursor is, or None when it is not shaped as a cursor or not on screen.
    fn rect(&self, cursor: i64) -> Result<Option<Rect>> {
        if !self.shaped(cursor)? {
            return Ok(None);
        }
        self.window_rect(cursor)
    }

    /// Where a window is drawn, or None when it is not on screen or has no size yet.
    fn window_rect(&self, window: i64) -> Result<Option<Rect>> {
        let placement = self.layouts.placement(window)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            return Ok(None);
        };
        let head = self.layouts.pane(window)?;
        let shape = self.layouts.shape;
        let half_width = f32(&head, shape.width)? * f32(&head, shape.scale_x)? / 2.0;
        let half_height = f32(&head, shape.height)? * f32(&head, shape.scale_y)? / 2.0;
        if half_width <= 0.0 || half_height <= 0.0 {
            return Ok(None);
        }
        let (x, y) = (
            f32(&head, self.layouts.shape.x)?,
            f32(&head, self.layouts.shape.y)?,
        );
        Ok(Some(Rect {
            top: placement.top,
            left: x - half_width,
            right: x + half_width,
            low: y - half_height,
            high: y + half_height,
        }))
    }

    /// Report the item under each of Sun's cursor panes on screen.
    fn poll_cursor_panes(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
    ) -> Result<()> {
        if !self.cursor_panes {
            return Ok(());
        }
        let panes = self.panes_as_cursors()?;
        let gone: Vec<i64> = self
            .fits
            .keys()
            .copied()
            .filter(|pane| !panes.contains_key(pane))
            .collect();
        for pane in gone {
            self.fits.shift_remove(&pane);
            self.forget(pane);
        }
        for (&pane, &top) in &panes {
            let spot = self.fitted(pane, top)?;
            self.fits.insert(pane, spot);
            if spot.anchor != 0 {
                let mut item = self.item(&spot, top, on_screen)?;
                let group = self.parent(spot.anchor)?;
                if item.is_empty() && self.words_hidden(spot.root, group)? {
                    // The X menu keeps a slot for every entry the story has yet to give, its words
                    // hidden, drawn as an empty plate the cursor still stops on (spike 112). A
                    // button of pictures alone, as a video call's, has no words to hide.
                    item = vec![ScreenText {
                        top,
                        root: spot.root,
                        parent: group,
                        ..ScreenText::new(group, EMPTY, false)
                    }];
                } else if item.is_empty()
                    && let Some(named) = self.button_name(spot.root, group)?
                {
                    // A button drawn as pictures alone is said by its pane's name, as ORAS says one
                    // by its texture's: the battle's `Btn_Fight`, `Btn_Bag`, `Btn_Pokemon_00` and
                    // `Btn_Run`, whose layout file Sun's readers do not have (spike 124).
                    item = vec![ScreenText {
                        top,
                        root: spot.root,
                        parent: group,
                        ..ScreenText::new(group, named, false)
                    }];
                }
                self.report(pane, item, items, Where::Pane(spot.anchor), false);
                continue;
            }
            let Some(rect) = self.window_rect(pane)? else {
                continue;
            };
            let under = in_front(
                on_screen
                    .iter()
                    .filter(|piece| within(piece, &rect))
                    .cloned()
                    .collect(),
                &rect,
            );
            let inside = distinct(in_rows(nearest_panel(under, &rect)));
            let at = place_of(&rect)?;
            self.report(pane, inside, items, at, false);
        }
        Ok(())
    }

    /// The words of a group's pane name, for a group holding no TextBox at all: `Btn_Fight` is
    /// "fight", `Btn_Pokemon_00` "pokémon". None for a group with words, or a name without any.
    fn button_name(&self, root: i64, group: i64) -> Result<Option<String>> {
        for (&pane, name) in &self.layouts.tree(root) {
            if name == lyt::TEXTBOX_CLASS && self.below(pane, group)? {
                return Ok(None);
            }
        }
        Ok(pane_words(&self.pane_name(group)?))
    }

    /// What the game calls the pictures shown inside a cursor with no words under it: the
    /// Pokédex's language tabs, whose textures the game swaps as it runs, each drawn as two
    /// pictures that name one language (spike 147).
    fn pictures_inside(&self, rect: &Rect) -> Result<Vec<ScreenText>> {
        let Some(pictures) = self.pictures.clone() else {
            return Ok(Vec::new());
        };
        let mut said: IndexMap<String, ScreenText> = IndexMap::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown && placement.top == rect.top) {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PICTURE_CLASS {
                    continue;
                }
                let (x, y) = self.place(pane)?;
                let shown = self.layouts.placement(pane)?;
                let inside = rect.left <= x && x <= rect.right && rect.low <= y && y <= rect.high;
                if !inside || !shown.is_some_and(|shown| shown.shown) {
                    continue;
                }
                let words = pictures(pane)?;
                if let Some(words) = words
                    && !words.is_empty()
                    && !said.contains_key(&words)
                {
                    let piece = ScreenText {
                        top: rect.top,
                        root,
                        parent: self.parent(pane)?,
                        ..ScreenText::new(pane, words.clone(), false)
                    };
                    said.insert(words, piece);
                }
            }
        }
        Ok(said.into_values().collect())
    }

    /// Does the group hold TextBoxes, none of them shown? Words shown but not yet read the same
    /// twice are words to wait for: said as empty, the X menu's SAVE was "empty" as it opened.
    fn words_hidden(&self, root: i64, group: i64) -> Result<bool> {
        let mut boxes: Vec<i64> = Vec::new();
        for (&pane, name) in &self.layouts.tree(root) {
            if name == lyt::TEXTBOX_CLASS && self.below(pane, group)? {
                boxes.push(pane);
            }
        }
        if boxes.is_empty() {
            return Ok(false);
        }
        for &r#box in &boxes {
            let placement = self.layouts.placement(r#box)?;
            if placement.is_some_and(|placement| placement.shown) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The root of the layout a cursor window is built into, as Sun's keyboard's is under a
    /// Parts pane, or None for a cursor that is a layout of its own.
    fn built_into(&self, window: i64) -> Result<Option<i64>> {
        let parent = self.parent(window)?;
        let vtable = u32(&self.layouts.pane(parent)?, 0)?;
        if self.names.of_vtable(vtable)?.as_deref() != Some(lyt::PARTS_CLASS) {
            return Ok(None);
        }
        let placement = self.layouts.placement(window)?;
        Ok(placement.map(|placement| placement.root))
    }

    /// Cursor pane -> on the top screen, for every plain pane shown alone under its layout's
    /// root with nothing but pictures under it.
    fn panes_as_cursors(&self) -> Result<IndexMap<i64, bool>> {
        let mut found: IndexMap<i64, bool> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            let under = self.layouts.children(root, root);
            if under.len() != 1 || class_of(&tree, under[0]) != Some(lyt::PANE_CLASS) {
                continue;
            }
            let pictures: Vec<i64> = tree
                .keys()
                .copied()
                .filter(|&pane| pane != root && pane != under[0])
                .collect();
            if pictures.is_empty() || pictures.iter().any(|pane| tree[pane] != lyt::PICTURE_CLASS) {
                continue;
            }
            let placement = self.layouts.placement(under[0])?;
            if let Some(placement) = placement.filter(|placement| placement.shown) {
                found.insert(under[0], placement.top);
            }
        }
        Ok(found)
    }

    /// Where the cursor pane is, and the plain pane shown in another layout on the same screen
    /// with its place and size, or an anchor of 0 for none. Searched again only when the cursor
    /// has moved: a screen's every pane is read in the search.
    fn fitted(&self, cursor: i64, top: bool) -> Result<Spot> {
        let (x, y) = self.place(cursor)?;
        if let Some(last) = self.fits.get(&cursor)
            && near(last.x, x)
            && near(last.y, y)
        {
            return Ok(*last);
        }
        let (width, height) = self.size(cursor)?;
        let own = self.parent(cursor)?;
        let mut fits: Vec<(f64, Spot)> = Vec::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if root == own
                || !placement.is_some_and(|placement| placement.shown && placement.top == top)
            {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PANE_CLASS || pane == root {
                    continue;
                }
                let (at, size) = (self.place(pane)?, self.size(pane)?);
                let slack = max_of(width - size.0, height - size.1);
                if !(near(at.0, x)
                    && near(at.1, y)
                    && -SAME_SPOT <= width - size.0
                    && width - size.0 <= CURSOR_SLACK
                    && -SAME_SPOT <= height - size.1
                    && height - size.1 <= CURSOR_SLACK)
                {
                    continue;
                }
                let shown = self.layouts.placement(pane)?;
                if shown.is_some_and(|shown| shown.shown) {
                    fits.push((
                        slack,
                        Spot {
                            x,
                            y,
                            anchor: pane,
                            root,
                        },
                    ));
                }
            }
        }
        let Some(&first) = fits.first() else {
            return Ok(Spot {
                x,
                y,
                anchor: 0,
                root: 0,
            });
        };
        // `min(fits, key=...)`: the first of the least.
        let mut best = first;
        for &fit in &fits[1..] {
            if fit.0 < best.0 {
                best = fit;
            }
        }
        Ok(best.1)
    }

    /// Cursor picture -> on the top screen, for every small picture alone in a layout shown.
    fn markers(&self) -> Result<IndexMap<i64, bool>> {
        let mut markers: IndexMap<i64, bool> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            if tree.len() != 2 {
                continue;
            }
            let others: Vec<(i64, &String)> = tree
                .iter()
                .filter(|&(&pane, _name)| pane != root)
                .map(|(&pane, name)| (pane, name))
                .collect();
            let &[(marker, name)] = others.as_slice() else {
                return Err(Error::value(if others.is_empty() {
                    "not enough values to unpack (expected 1, got 0)".to_string()
                } else {
                    format!(
                        "too many values to unpack (expected 1, got {})",
                        others.len()
                    )
                }));
            };
            if name != lyt::PICTURE_CLASS {
                continue;
            }
            let head = self.layouts.pane(marker)?;
            let width = f32(&head, self.layouts.shape.width)?;
            if !(0.0 < width && width <= MARKER_SIZE) {
                continue;
            }
            let height = f32(&head, self.layouts.shape.height)?;
            if !(0.0 < height && height <= MARKER_SIZE) {
                continue;
            }
            let placement = self.layouts.placement(marker)?;
            if let Some(placement) = placement.filter(|placement| placement.shown) {
                markers.insert(marker, placement.top);
            }
        }
        Ok(markers)
    }

    /// The pane the cursor picture sits on, or None. Searched again only when it moved.
    fn spot(&mut self, marker: i64, top: bool) -> Result<Option<Spot>> {
        let shape = self.layouts.shape;
        let head = self.layouts.pane(marker)?;
        let (x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
        if let Some(last) = self.spots.get(&marker).copied()
            && near(last.x, x)
            && near(last.y, y)
        {
            let anchor = self.layouts.pane(last.anchor)?;
            let placement = self.layouts.placement(last.anchor)?;
            if near(f32(&anchor, shape.x)?, x)
                && near(f32(&anchor, shape.y)?, y)
                && placement.is_some_and(|placement| placement.shown)
            {
                return Ok(Some(last));
            }
        }
        let marker_root = u32(&head, lyt::PARENT)?;
        for root in self.layouts.roots() {
            if root == marker_root {
                continue;
            }
            let placement = self.layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown && placement.top == top) {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PANE_CLASS || pane == root {
                    continue;
                }
                let pane_head = self.layouts.pane(pane)?;
                if !(near(f32(&pane_head, shape.x)?, x) && near(f32(&pane_head, shape.y)?, y)) {
                    continue;
                }
                let pane_placement = self.layouts.placement(pane)?;
                if pane_placement.is_some_and(|placement| placement.shown) {
                    let spot = Spot {
                        x,
                        y,
                        anchor: pane,
                        root,
                    };
                    self.spots.insert(marker, spot);
                    return Ok(Some(spot));
                }
            }
        }
        self.spots.shift_remove(&marker);
        Ok(None)
    }

    /// The item a cursor picture's pane belongs to: the words under its group, or the words
    /// of the group's first picture's texture.
    fn item(
        &mut self,
        spot: &Spot,
        top: bool,
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let group = self.parent(spot.anchor)?;
        let mut near: Vec<ScreenText> = Vec::new();
        for piece in on_screen {
            if self.below(piece.r#box, group)? {
                near.push(piece.clone());
            }
        }
        let inside = distinct(in_rows(near));
        if !inside.is_empty() {
            return Ok(inside);
        }
        let Some(said) = self.pictured(spot.root, group, on_screen, None)? else {
            return Ok(Vec::new());
        };
        Ok(vec![ScreenText {
            top,
            root: spot.root,
            parent: group,
            ..ScreenText::new(group, said, false)
        }])
    }

    /// The words of the texture of the first picture in a group, from its layout's file.
    ///
    /// A texture that also lies under words is a plate the words are written on, not a label:
    /// the move list's empty fourth slot is the plate every move is written on, and read as
    /// "waza plate 00" (battles.ctm replay). Such a group is said as empty. `skip` is a picture to
    /// pass over: a cursor standing in the group is not what the group is.
    fn pictured(
        &mut self,
        root: i64,
        group: i64,
        on_screen: &[ScreenText],
        skip: Option<i64>,
    ) -> Result<Option<String>> {
        if let Some(known) = self.picture_words.get(&(root, group)) {
            return Ok(known.clone());
        }
        let file = self
            .files
            .as_ref()
            .and_then(|files| files.matching(&self.layouts.pane_names(root)));
        let Some(file) = file else {
            return Ok(None);
        };
        let tree = self.layouts.tree(root);
        let mut texture: Option<String> = None;
        for (&pane, name) in &tree {
            if name == lyt::PICTURE_CLASS && Some(pane) != skip && self.below(pane, group)? {
                texture = self.texture(pane, &file)?;
                if texture.is_some() {
                    break;
                }
            }
        }
        let mut said: Option<String> = texture.as_deref().map(words);
        if let Some(texture) = &texture {
            for piece in on_screen {
                if piece.root != root {
                    continue;
                }
                let mut pane = self.parent(piece.r#box)?;
                for _ in 0..MAX_DEPTH {
                    if pane == 0 || pane == root {
                        break;
                    }
                    if class_of(&tree, pane) == Some(lyt::PICTURE_CLASS)
                        && self.texture(pane, &file)?.as_ref() == Some(texture)
                    {
                        said = Some(EMPTY.to_string());
                        break;
                    }
                    pane = self.parent(pane)?;
                }
                if said.as_deref() == Some(EMPTY) {
                    break;
                }
            }
        }
        self.picture_words.insert((root, group), said.clone());
        Ok(said)
    }

    /// The first texture a picture pane's layout file gives it, or None.
    fn texture(&self, pane: i64, file: &LayoutFile) -> Result<Option<String>> {
        let name = self.pane_name(pane)?;
        Ok(file
            .textures
            .get(&name)
            .and_then(|textures| textures.first().cloned()))
    }

    /// Is `pane` the group or somewhere under it?
    fn below(&self, mut pane: i64, group: i64) -> Result<bool> {
        for _ in 0..MAX_DEPTH {
            if pane == group {
                return Ok(true);
            }
            if pane == 0 {
                return Ok(false);
            }
            pane = self.parent(pane)?;
        }
        Ok(false)
    }
}

fn centre(rect: &Rect) -> (f64, f64) {
    ((rect.left + rect.right) / 2.0, (rect.low + rect.high) / 2.0)
}

fn within(piece: &ScreenText, rect: &Rect) -> bool {
    if piece.glyph <= 0.0 || piece.top != rect.top {
        return false;
    }
    let centre = (piece.left + piece.right) / 2.0;
    rect.left <= centre && centre <= rect.right && rect.low <= piece.row && piece.row <= rect.high
}

/// The words a cursor picture points at from beside them: its layout's, on its row, to its
/// right. The Poké Mart's boxes draw `cursor.bclim` left of their words rather than over them
/// (Buy, Sell, No thanks; a purchase's Yes and No), and move it from one row's height to the
/// next (spike 98).
fn beside(pieces: &[ScreenText], rect: &Rect, root: i64) -> Vec<ScreenText> {
    pieces
        .iter()
        .filter(|piece| {
            piece.root == root
                && piece.glyph > 0.0
                && piece.top == rect.top
                && rect.low <= piece.row
                && piece.row <= rect.high
                && piece.left >= rect.right - piece.glyph
        })
        .cloned()
        .collect()
}

/// The pieces of the one panel under the cursor, when several are: the panel holding the piece
/// nearest the middle of the cursor.
///
/// A screen can keep the panel it is leaving on screen for a poll: confirming a language, the list
/// was still there behind the confirmation, and the cursor over "Begin game" said "Spanish French
/// Begin game" (opening.ctm, the user, 2026-09-18). An item can still span rows, as a party slot's
/// name and level do, because a panel is kept whole. Of pieces on one row, the nearest across wins:
/// the Pokédex's More list put CANCEL over the page's Habitat button, the cursor centred on CANCEL
/// and Habitat at its edge, and the cursor said "Habitat" (or/movie7_menus, 2026-09-27).
fn nearest_panel(pieces: Vec<ScreenText>, rect: &Rect) -> Vec<ScreenText> {
    let panels: HashSet<(i64, i64)> = pieces
        .iter()
        .map(|piece| (piece.root, piece.panel))
        .collect();
    if panels.len() < 2 {
        return pieces;
    }
    let middle = (rect.low + rect.high) / 2.0;
    let centre = (rect.left + rect.right) / 2.0;
    let key = |piece: &ScreenText| {
        (
            (piece.row - middle).abs(),
            ((piece.left + piece.right) / 2.0 - centre).abs(),
        )
    };
    // `min(pieces, key=...)`: the first of the least, by a tuple of floats, which a NaN never
    // makes less.
    let mut nearest = &pieces[0];
    let mut best = key(nearest);
    for piece in &pieces[1..] {
        let candidate = key(piece);
        let less = if candidate.0 != best.0 {
            candidate.0 < best.0
        } else {
            candidate.1 < best.1
        };
        if less {
            nearest = piece;
            best = candidate;
        }
    }
    let wanted = (nearest.root, nearest.panel);
    pieces
        .iter()
        .filter(|piece| (piece.root, piece.panel) == wanted)
        .cloned()
        .collect()
}

/// The pieces of the layouts with words wholly under the cursor, when any layout has some.
///
/// A layout drawn behind the menu can have words whose middle is under the cursor and whose ends
/// reach out past it: with the Pokédex in the X menu, OPTIONS sits over AreaNav's "Littleroot
/// Town", which starts well to the left of the button, and the item was said as "Littleroot Town
/// OPTIONS" (battles2.ctm, spike 68). A piece's ends are estimated, so half a glyph either way
/// still counts as under, and when no layout's words are wholly under the cursor, all are kept.
fn in_front(pieces: Vec<ScreenText>, rect: &Rect) -> Vec<ScreenText> {
    let whole: HashSet<i64> = pieces
        .iter()
        .filter(|piece| {
            rect.left - piece.glyph / 2.0 <= piece.left
                && piece.right <= rect.right + piece.glyph / 2.0
        })
        .map(|piece| piece.root)
        .collect();
    if whole.is_empty() {
        pieces
    } else {
        pieces
            .into_iter()
            .filter(|piece| whole.contains(&piece.root))
            .collect()
    }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= SAME_SPOT
}

/// Is this place inside the rectangle?
fn on(place: (f64, f64), rect: &Rect) -> bool {
    let (x, y) = place;
    rect.left <= x && x <= rect.right && rect.low <= y && y <= rect.high
}

/// A window's place and size to the unit, to say whether it has moved or been resized: the
/// main menu's cursor is 336 by 140 on CONTINUE's panel and 336 by 43 on the rows below it.
fn spot_of(rect: &Rect) -> Result<(i64, i64, i64, i64)> {
    Ok((
        py::round(rect.left)?,
        py::round(rect.low)?,
        py::round(rect.right - rect.left)?,
        py::round(rect.high - rect.low)?,
    ))
}

/// `(round(rect.left), round(rect.low))`: where a window is, to the unit. Written out at each of
/// its four places in the Python, so a NaN is raised from the line that asked, as it is there.
#[track_caller]
fn place_of(rect: &Rect) -> Result<Where> {
    Ok(Where::Place(py::round(rect.left)?, py::round(rect.low)?))
}

/// The pane's class, or None for a pane not in the tree: `tree.get(pane)`.
fn class_of(tree: &IndexMap<i64, String>, pane: i64) -> Option<&str> {
    tree.get(&pane).map(String::as_str)
}

/// Python's `max(a, b)` on floats: `a`, unless `b > a`, so a NaN stays where it is.
fn max_of(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `len(set(pairs))` of floats: a pair is the same as another by `==`, which a NaN never is.
fn distinct_count(pairs: &[(f64, f64)]) -> usize {
    let mut seen: Vec<(f64, f64)> = Vec::new();
    for pair in pairs {
        if !seen.contains(pair) {
            seen.push(*pair);
        }
    }
    seen.len()
}

/// `a < b` for the tuples `(-since, area, root, parent)`: the first place they differ decides,
/// and a NaN differs from everything, itself included, and is less than nothing.
fn found_lt(a: &(i64, f64, i64, i64), b: &(i64, f64, i64, i64)) -> bool {
    if a.0 != b.0 {
        return a.0 < b.0;
    }
    if a.1 != b.1 {
        return a.1 < b.1;
    }
    if a.2 != b.2 {
        return a.2 < b.2;
    }
    a.3 < b.3
}
