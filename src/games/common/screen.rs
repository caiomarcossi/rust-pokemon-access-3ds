//! Screen text: announce the words a screen draws outside any message window.
//!
//! A menu's labels, the options screen's names and values, and a question such as "Would you
//! like to make these changes?" are not messages. Each is a string set straight into an
//! `nw::lyt::TextBox` pane of a NintendoWare layout (spike 28), laid out as `lyt` gives.
//!
//! Layouts hold far more text than is on screen — options this language does not have, questions
//! not asked yet, whole screens not shown yet — so a TextBox's words are heard only when code
//! wrote them (the tag processor is set) and `Layouts` says the TextBox is being drawn (spikes 29
//! to 31), or when they are a designer's separator between two written words, as Sun's "/" is
//! (`separators`).
//!
//! Words are reported once they read the same on two polls, so a string caught halfway through
//! being written is not spoken. A message window's line objects are TextBoxes too; the dialogue
//! reader owns those, and passes them in to be left out.
//!
//! Each piece of a screen is its own TextBox — the party screen draws "L", "5", "19", "/" and
//! "19" as five — so `join` puts pieces drawn side by side back together, from where the pane
//! says its words are: its global place, width and scale, its anchor, and the TextBox's glyph
//! width and text position (spike 33).

use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::text::pages;
use crate::games::common::{lyt, msgwin};
use crate::py::{self, Error, Kind, PySet, Result};

/// Halfwords. Screen text is labels and short sentences; this only rejects garbage lengths.
pub const MAX_TEXT: i64 = 0x400;

/// Left, centre, right: how far across its box an anchor or an alignment sits.
pub const FRACTION: [f64; 3] = [0.0, 0.5, 1.0];

/// An average glyph's width across, as a share of the TextBox's glyph width. An estimate:
/// "Torchic", seven glyphs at 12.5, drew about 44 units wide in spike 31's party screen picture.
pub const GLYPH_ADVANCE: f64 = 0.5;
/// Units within which two pieces with the same words are one drawn twice.
pub const SAME_PLACE: f64 = 2.0;
/// Pieces whose words come closer than this many glyph widths are one piece: "L5" and "19/19" are
/// 6 to 8 units apart at a glyph width of 16, and "PP 35/35"'s "PP" and "35" about 15 at 12.5.
pub const JOIN_GAP: f64 = 0.8;
/// What Sun's party screen writes where it draws "Lv.": a TextBox of its own holding a lone "l",
/// drawn as the level mark by the font the level's digits are in, just left of the level in the
/// same group. Read as it is, Rowlet's slot was "Rowlet 22 22 l5" (sun_9.ctm, 2026-09-27).
pub const LEVEL_GLYPH: &str = "l";
/// Polls after its layout's first words within which a piece still comes with its screen: the
/// battle bag's "LAST USED ITEM" was written a poll after the pockets beside it.
pub const ARRIVAL_POLLS: i64 = 2;
pub const SCREEN_HALF_HEIGHT: f64 = 120.0;
/// Half the bottom screen's width, and the top's. A layout's origin is the middle of its screen.
pub const SCREEN_HALF_WIDTH: [f64; 2] = [160.0, 200.0];

/// Words a screen has just shown: in a TextBox newly on screen, or newly written into one.
///
/// Python's `ScreenText(box, text, changed)` leaves the rest at their defaults: [`ScreenText::new`]
/// does, [`ScreenText::placed`] gives the place too, and the other fields are `pub` for a struct
/// update.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenText {
    pub r#box: i64,
    pub text: String,
    /// True when the TextBox was already on screen and its words changed, as a value's do.
    pub changed: bool,
    pub left: f64,
    /// Where the words are estimated to start and end across the screen.
    pub right: f64,
    pub row: f64,
    /// The glyph width; 0 when the pane says nothing usable about where it is.
    pub glyph: f64,
    /// On the top screen.
    pub top: bool,
    /// The root pane of the layout it is in.
    pub root: i64,
    /// The pane it hangs from: the group of a row, a panel, a slot.
    pub parent: i64,
    /// The pane directly under its layout's root that it hangs from. Where `parent` is the row or
    /// slot, this is the part of the screen: the battle's HP box keeps the player's level and HP in
    /// one panel and the wild Pokemon's level in another (spike 53), and words from two panels are
    /// two things to say rather than one run.
    pub panel: i64,
    /// It came on screen in the same poll as the first of its layout's pieces: part of a screen
    /// opening or coming back, not words appearing later in a screen already up.
    pub with_layout: bool,
    /// It came back on screen with the words it left with.
    pub returned: bool,
    /// Its words are what the game calls what a picture draws (`gen6.textures`): a type's icon,
    /// a status condition's.
    pub picture: bool,
    /// Read first in its row, wherever it is drawn in it (`after_the_name`).
    pub leads: bool,
    /// It moved onto its screen with the words it had just outside it: a list's row as the list
    /// scrolls, where the language screen moves its rows rather than writing new words into them.
    pub scrolled: bool,
    /// For words hanging straight from their layout's root, the `nw::lyt::Window` of the layout
    /// drawn round them, or 0. The move-forget screen hangs every word from its root, the Pokémon's
    /// stats and the chosen move's alike, and groups them only by the frames it draws: one round
    /// CATEGORY, POWER and ACCURACY, one round the description (spike 139).
    pub frame: i64,
}

impl ScreenText {
    /// `ScreenText(box, text, changed)`: every other field at its default.
    pub fn new(r#box: i64, text: impl Into<String>, changed: bool) -> Self {
        ScreenText {
            r#box,
            text: text.into(),
            changed,
            left: 0.0,
            right: 0.0,
            row: 0.0,
            glyph: 0.0,
            top: false,
            root: 0,
            parent: 0,
            panel: 0,
            with_layout: false,
            returned: false,
            picture: false,
            leads: false,
            scrolled: false,
            frame: 0,
        }
    }

    /// `ScreenText(box, text, changed, left, right, row, glyph)`: the rest at their defaults.
    pub fn placed(
        r#box: i64,
        text: impl Into<String>,
        changed: bool,
        left: f64,
        right: f64,
        row: f64,
        glyph: f64,
    ) -> Self {
        ScreenText {
            left,
            right,
            row,
            glyph,
            ..ScreenText::new(r#box, text, changed)
        }
    }

    /// The layout's root and the pane or frame its words are grouped by.
    pub fn group(&self) -> (i64, i64) {
        (
            self.root,
            if self.frame != 0 {
                self.frame
            } else {
                self.parent
            },
        )
    }
}

/// Python's `min(a, b)` on floats: `a`, unless `b < a`. A NaN is never smaller, so it stays
/// whichever side it is on, where `f64::min` would drop it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python's `max(a, b)` on floats: `a`, unless `b > a`.
fn max_of(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `sorted(items, key=...)`, where `lt(a, b)` is `key(a) < key(b)`.
///
/// Python's sort compares with `<` alone, and where a key is a NaN, which is no order at all, the
/// order it makes depends on timsort's steps: a pane whose place reads NaN is common enough in
/// freed memory. Of fewer than 64 items timsort is `count_run` then a binary insertion sort,
/// both written here as `Objects/listobject.c` has them (and checked against `sorted` over
/// random lists with NaNs). From 64 items it also merges runs, which this does not follow: a
/// stable merge sort gives the same order for any keys that are ordered, and a different one only
/// around a NaN.
pub(crate) fn sorted_by_lt<T>(mut items: Vec<T>, lt: impl Fn(&T, &T) -> bool) -> Vec<T> {
    let n = items.len();
    if n < 2 {
        return items;
    }
    if n >= 64 {
        return merge_sorted(items, &lt);
    }
    let run = count_run(&mut items, &lt);
    for start in run..n {
        let (mut low, mut high) = (0, start);
        while low < high {
            let middle = low + ((high - low) >> 1);
            if lt(&items[start], &items[middle]) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        items[low..=start].rotate_right(1);
    }
    items
}

/// The length of the run timsort takes from the start of `items`, which it leaves ascending: a
/// descending run is reversed, with each stretch of equal items in it turned back so the sort
/// stays stable.
fn count_run<T>(items: &mut [T], lt: &impl Fn(&T, &T) -> bool) -> usize {
    fn reverse_last_neq<T>(items: &mut [T], n: usize, neq: &mut usize) {
        if *neq > 0 {
            items[n - (*neq + 1)..n].reverse();
            *neq = 0;
        }
    }
    let remaining = items.len();
    let mut n = 1;
    while n < remaining {
        if lt(&items[n], &items[n - 1]) {
            break;
        }
        n += 1;
    }
    if n == remaining {
        return n;
    }
    let mut neq = 0;
    if n > 1 {
        if lt(&items[0], &items[n - 1]) {
            return n;
        }
        neq = n - 1;
    }
    reverse_last_neq(items, n, &mut neq);
    n += 1;
    while n < remaining {
        if lt(&items[n], &items[n - 1]) {
            reverse_last_neq(items, n, &mut neq);
        } else {
            if lt(&items[n - 1], &items[n]) {
                break;
            }
            neq += 1;
        }
        n += 1;
    }
    reverse_last_neq(items, n, &mut neq);
    items[..n].reverse();
    while n < remaining {
        if lt(&items[n], &items[n - 1]) {
            break;
        }
        n += 1;
    }
    n
}

/// A stable merge sort by `lt` alone, for the lists `sorted_by_lt` leaves to merging.
fn merge_sorted<T>(mut items: Vec<T>, lt: &impl Fn(&T, &T) -> bool) -> Vec<T> {
    if items.len() < 2 {
        return items;
    }
    let right = items.split_off(items.len() / 2);
    let (left, right) = (merge_sorted(items, lt), merge_sorted(right, lt));
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let (mut left, mut right) = (left.into_iter().peekable(), right.into_iter().peekable());
    while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
        merged.push(if lt(b, a) {
            right.next().expect("peeked")
        } else {
            left.next().expect("peeked")
        });
    }
    merged.extend(left);
    merged.extend(right);
    merged
}

/// `str.isalpha()` for one character: the letter categories.
fn is_alpha(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    matches!(
        unicode_general_category::get_general_category(c),
        UppercaseLetter | LowercaseLetter | TitlecaseLetter | ModifierLetter | OtherLetter
    )
}

/// `str.isdigit()` for one character: `py::is_digit`'s decimal digits, and the digits that are no
/// decimal ones (superscripts, circled numbers), which `py::is_digit` leaves out.
fn is_digit(c: char) -> bool {
    py::is_digit(c)
        || matches!(c,
            '\u{b2}'..='\u{b3}' | '\u{b9}' | '\u{1369}'..='\u{1371}' | '\u{19da}' | '\u{2070}'
            | '\u{2074}'..='\u{2079}' | '\u{2080}'..='\u{2089}' | '\u{2460}'..='\u{2468}'
            | '\u{2474}'..='\u{247c}' | '\u{2488}'..='\u{2490}' | '\u{24ea}'
            | '\u{24f5}'..='\u{24fd}' | '\u{24ff}' | '\u{2776}'..='\u{277e}'
            | '\u{2780}'..='\u{2788}' | '\u{278a}'..='\u{2792}' | '\u{10a40}'..='\u{10a43}'
            | '\u{10e60}'..='\u{10e68}' | '\u{11052}'..='\u{1105a}' | '\u{1f100}'..='\u{1f10a}')
}

/// `str.isalnum()` for one character: a letter, or any number (`isdecimal`, `isdigit` and
/// `isnumeric` together).
fn is_alnum(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    is_alpha(c)
        || is_digit(c)
        || matches!(
            unicode_general_category::get_general_category(c),
            DecimalNumber | LetterNumber | OtherNumber
        )
}

/// `text.isdigit()`: some characters, all of them digits.
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_digit)
}

/// The pieces in the order given, with each run of them on one row read left to right.
///
/// The order the game built its TextBoxes in keeps a screen's parts together — the summary's two
/// columns of stats, a party slot's name above its level — and reading the whole screen row by row
/// interleaved those columns (spike 47). Inside a row that order is not reading order: a move's
/// "/" is built before both of its PP numbers (spike 33). A run is consecutive pieces less than
/// half a glyph apart up and down; pieces whose place is unknown end a run and stay where they are.
pub fn in_rows(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let mut ordered: Vec<ScreenText> = Vec::new();
    let mut run: Vec<ScreenText> = Vec::new();
    for piece in pieces {
        if !run.is_empty()
            && (piece.glyph <= 0.0 || (run[0].row - piece.row).abs() >= piece.glyph / 2.0)
        {
            ordered.extend(sorted_across(std::mem::take(&mut run)));
        }
        if piece.glyph <= 0.0 {
            ordered.push(piece);
        } else {
            run.push(piece);
        }
    }
    ordered.extend(sorted_across(run));
    ordered
}

/// `sorted(run, key=_across)`.
fn sorted_across(run: Vec<ScreenText>) -> Vec<ScreenText> {
    // The key is `(not leads, left)`: a tuple is smaller by its first element that differs.
    sorted_by_lt(run, |a, b| {
        if a.leads != b.leads {
            return a.leads && !b.leads;
        }
        a.left < b.left
    })
}

/// The pieces row by row from the top, each row left to right: a panel of labels and values, as
/// a move's Category, Power and Accuracy, read as a table. A row is pieces less than half a glyph
/// apart up and down. Not for a whole screen, whose columns this would interleave (spike 47).
pub fn by_rows(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let placed: Vec<ScreenText> = pieces.into_iter().filter(|p| p.glyph > 0.0).collect();
    let placed = sorted_by_lt(placed, |a, b| -a.row < -b.row);
    let mut ordered: Vec<ScreenText> = Vec::new();
    let mut row: Vec<ScreenText> = Vec::new();
    for piece in placed {
        if !row.is_empty() && row[0].row - piece.row >= piece.glyph / 2.0 {
            ordered.extend(sorted_across(std::mem::take(&mut row)));
        }
        row.push(piece);
    }
    ordered.extend(sorted_across(row));
    ordered
}

/// The pieces without the same words drawn twice in one place.
///
/// The X menu draws each label in two TextBoxes a unit apart (spike 38). The same words somewhere
/// else are kept: a move's PP reads "40 / 40", and dropping the second "40" lost the PP left.
pub fn distinct(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let mut out: Vec<ScreenText> = Vec::new();
    for piece in pieces {
        if !out.iter().any(|other| {
            other.text == piece.text
                && (other.left - piece.left).abs() <= SAME_PLACE
                && (other.row - piece.row).abs() <= SAME_PLACE
        }) {
            out.push(piece);
        }
    }
    out
}

/// The pieces as one line to speak, in their order.
///
/// Pieces drawn side by side on one row are joined as drawn ("L5", "19/19"). Others are separated
/// by a space, or when asked for (the user, 2026-09-14) by a comma, or a colon after a label
/// (`labels`): "OT: Raiden, ID No.: 16285" (Orion, 2026-09-30). A label with a colon of its own
/// is followed by a space: "Badges: 0", not "Badges:, 0". A value is no label of what follows
/// it: "ITEM: Potion, Sand Attack", "Name: Raiden, ID No.: 16285".
///
/// Python's `pauses` defaults to False.
pub fn join(pieces: &[ScreenText], pauses: bool) -> String {
    let mut out = String::new();
    let mut valued = false;
    for (index, piece) in pieces.iter().enumerate() {
        let before = if index != 0 {
            Some(&pieces[index - 1])
        } else {
            None
        };
        let mut labelled = false;
        match before {
            None => out = piece.text.clone(),
            Some(before) if touching(before, piece) => out.push_str(&piece.text),
            Some(before) if !pauses || before.text.ends_with(':') => {
                out.push(' ');
                out.push_str(&piece.text);
                labelled = pauses;
            }
            Some(before) => {
                labelled = !valued && labels(before, piece);
                out.push_str(if labelled { ": " } else { ", " });
                out.push_str(&piece.text);
            }
        }
        valued = labelled;
    }
    out
}

/// The most words a label has: "To Next Lv." has three.
pub const LABEL_WORDS: usize = 3;
/// Glyph widths past a label's end a value ends at, at least: a table's values are drawn to its
/// right edge.
pub const VALUE_BEYOND: f64 = 2.0;
/// Glyph widths below a label its value may start: the summary draws Exp. Points' number on the
/// row under it.
pub const VALUE_BELOW: f64 = 2.5;

/// Is `first` a label and `second` its value, as a table draws them?
///
/// The summary names none of its panes for what they hold (spike 148: "0", "4", "1k"), so this is
/// told by place: a label is a few words with a letter and no digit, and its value starts no
/// further left, on its row, a little above it or just below, and ends well past it, as a table's
/// right-aligned column does. A value below its label starts past the label's end, as Exp.
/// Points' number does: the next row of a list starts under the one above it ("Ember", "Sand
/// Attack"). A label's words are the screen's own, so words that changed are a value's, and a
/// picture's words are what it draws, as a type's icon, "Fire", is (movie11, 2026-10-01).
fn labels(first: &ScreenText, second: &ScreenText) -> bool {
    // ponytail: judged by place alone; a screen this gets wrong wants its label panes listed.
    let glyph = min_of(first.glyph, second.glyph);
    if glyph <= 0.0
        || first.changed
        || first.picture
        || first.text.ends_with(':')
        || first.text.chars().any(is_digit)
        || !first.text.chars().any(is_alpha)
    {
        return false;
    }
    let below = first.row - second.row > glyph / 2.0;
    // A sign is no value: the party slot draws the gender sign at a place of its own, far past a
    // short name.
    py::split(&first.text).len() <= LABEL_WORDS
        && second.text.chars().any(is_alnum)
        && second.left
            >= (if below {
                first.right
            } else {
                first.left - glyph / 2.0
            })
        && -glyph <= first.row - second.row
        && first.row - second.row <= glyph * VALUE_BELOW
        && second.right >= first.right + glyph * VALUE_BEYOND
}

fn touching(first: &ScreenText, second: &ScreenText) -> bool {
    let glyph = min_of(first.glyph, second.glyph);
    if glyph <= 0.0 || (first.row - second.row).abs() >= glyph {
        return false;
    }
    -glyph < second.left - first.right && second.left - first.right <= glyph * JOIN_GAP
}

/// What `poll` keeps of each window shown in a layout: the pane, then its left, right, low and
/// high edges.
type Frame = (i64, f64, f64, f64, f64);

/// `pictures: Callable[[int], str | None]`: what the game calls what a picture draws now, by its
/// image (`gen6.textures`). It reads the emulator, so it can raise, and the Rust one returns a
/// `Result` for that.
pub type Pictures = Rc<dyn Fn(i64) -> Result<Option<String>>>;

/// A `set` of ints as Python lays it out, from the order its members were added in: the copy
/// `set(s)` makes, which is what a method returning one hands its caller to iterate.
///
/// A copy is made by `set_merge` in `Objects/setobject.c`, into a table sized for it: where that
/// is the size the original grew to (up to four members, and eight to fifteen) every member
/// keeps its slot; otherwise the members are put in again, in the order the original holds them.
fn copy_of_set(added: &[i64]) -> IndexSet<i64> {
    let original: Vec<i64> = PySet::of(added.iter().copied()).iter().collect();
    let used = original.len();
    let table = |minused: usize| {
        let mut size = 8;
        while size <= minused {
            size <<= 1;
        }
        size
    };
    let mut grown = 8;
    for members in 1..=used {
        if members * 5 >= (grown - 1) * 3 {
            grown = table(if members > 50000 {
                members * 2
            } else {
                members * 4
            });
        }
    }
    let copied = if used * 5 >= 7 * 3 {
        table(used * 2)
    } else {
        8
    };
    if copied == grown {
        original.into_iter().collect()
    } else {
        PySet::of_dict(&original).iter().collect()
    }
}

/// `dict[key]`, raising `KeyError` where the key is not there.
#[track_caller]
fn get(map: &IndexMap<i64, i64>, key: i64) -> Result<i64> {
    map.get(&key)
        .copied()
        .ok_or_else(|| Error::key(key.to_string()))
}

/// Tracks layout TextBoxes and reports the words that come on screen.
pub struct ScreenTextReader {
    level_mark: Option<String>,
    pictures: Option<Pictures>,
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    windows: Vec<String>,
    boxes: Vec<i64>,
    /// TextBox -> its words at the last poll, for TextBoxes that were on screen.
    seen: IndexMap<i64, String>,
    /// TextBox -> the words last reported, for TextBoxes still on screen.
    said: IndexMap<i64, String>,
    /// TextBox -> the piece last reported for it, to say what a word that left was.
    last_said: IndexMap<i64, ScreenText>,
    /// TextBox -> the words it left the screen with, for TextBoxes off screen and tracked.
    gone: IndexMap<i64, String>,
    /// Roots of layouts with words on screen that have not been reported yet, in the order the
    /// Python set was added to (a set of ints is iterated by its callers, see [`copy_of_set`]).
    settling: Vec<i64>,
    /// TextBox -> its piece at the last poll, for TextBoxes counted on screen then.
    last: IndexMap<i64, ScreenText>,
    /// TextBoxes counted on screen at the last poll only because they had been the poll before.
    carried: HashSet<i64>,
    on_screen: Vec<ScreenText>,
    /// The pieces that left the screen at the last poll, as they last read.
    left: Vec<ScreenText>,
    polls: i64,
    /// Layout root -> the poll its pieces came on screen at, for layouts on screen.
    root_since: IndexMap<i64, i64>,
    /// TextBox -> the poll it came on screen at, for TextBoxes on screen.
    box_since: IndexMap<i64, i64>,
    /// Tag processor -> whether it is one a message window embeds, until the next `track`.
    window_tags: IndexMap<i64, bool>,
    /// TextBoxes a message window was typing into at the last poll, in the order the Python set
    /// was added to.
    message_lines: Vec<i64>,
    /// TextBox -> its words, for TextBoxes drawn outside their screen at the last poll.
    outside: IndexMap<i64, String>,
    /// TextBox -> its words, for TextBoxes that moved onto their screen and are not reported
    /// yet: words are reported a poll after they arrive, when they are no longer outside.
    scrolled: IndexMap<i64, String>,
    /// Layout root -> (window, left, right, low, high) for each window shown in it, at this
    /// poll.
    frames: IndexMap<i64, Vec<Frame>>,
}

impl ScreenTextReader {
    /// `ScreenTextReader(memory, names, layouts)`: the message window classes of `msgwin`, no
    /// level mark and no pictures.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, layouts: Rc<Layouts>) -> Self {
        ScreenTextReader::with(
            memory,
            names,
            layouts,
            msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
            None,
            None,
        )
    }

    /// `windows` are the game's message window classes, whose lines this leaves out.
    /// `level_mark` is the game's own words for the level mark Sun draws as a glyph
    /// (`LEVEL_GLYPH`), which are said in its place. `pictures` says what the game calls what a
    /// picture draws now, by its image (`gen6.textures`), or None.
    pub fn with(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        windows: Vec<String>,
        level_mark: Option<String>,
        pictures: Option<Pictures>,
    ) -> Self {
        ScreenTextReader {
            level_mark,
            pictures,
            memory,
            names,
            layouts,
            windows,
            boxes: Vec::new(),
            seen: IndexMap::new(),
            said: IndexMap::new(),
            last_said: IndexMap::new(),
            gone: IndexMap::new(),
            settling: Vec::new(),
            last: IndexMap::new(),
            carried: HashSet::new(),
            on_screen: Vec::new(),
            left: Vec::new(),
            polls: 0,
            root_since: IndexMap::new(),
            box_since: IndexMap::new(),
            window_tags: IndexMap::new(),
            message_lines: Vec::new(),
            outside: IndexMap::new(),
            scrolled: IndexMap::new(),
            frames: IndexMap::new(),
        }
    }

    /// Every piece on screen at the last poll, reported or not, the top screen first.
    pub fn on_screen(&self) -> Vec<ScreenText> {
        self.on_screen.clone()
    }

    /// The pieces that left the screen at the last poll, as they last read: a letter deleted
    /// from a name is one of them.
    pub fn left(&self) -> Vec<ScreenText> {
        self.left.clone()
    }

    /// The TextBoxes a message window was typing into at the last poll, whether or not the
    /// dialogue reader has found that window yet: handed to it, they lead to the window the moment
    /// it types, where a sweep took up to 2 s (spike 60).
    ///
    /// Python returns a `set`; the `IndexSet` here is in the order Python iterates it.
    pub fn message_lines(&self) -> IndexSet<i64> {
        copy_of_set(&self.message_lines)
    }

    /// The roots of layouts whose words on screen at the last poll were not all reported,
    /// because some had not yet read the same for a poll: words still being written.
    ///
    /// Python returns a `set`; the `IndexSet` here is in the order Python iterates it.
    pub fn settling(&self) -> IndexSet<i64> {
        copy_of_set(&self.settling)
    }

    /// Read these TextBoxes from now on.
    ///
    /// Each sweep replaces the last, except that whatever is on screen stays tracked: a sweep taken
    /// while a scene loads can miss objects that are back a moment later, and losing them would
    /// announce a whole screen again.
    pub fn track(&mut self, boxes: impl IntoIterator<Item = i64>) {
        let every: BTreeSet<i64> = boxes.into_iter().chain(self.said.keys().copied()).collect();
        self.boxes = every.into_iter().collect();
        let tracked: HashSet<i64> = self
            .boxes
            .iter()
            .copied()
            .chain(self.layouts.panes(lyt::TEXTBOX_CLASS))
            .collect();
        self.gone.retain(|r#box, _| tracked.contains(r#box));
        self.window_tags.clear();
    }

    /// What has come on screen since the last poll, the top screen first.
    ///
    /// `exclude` is the TextBoxes message windows lay their pages out in. With `hold`, words are
    /// reported only once they read the same on two polls in a row; without it, at once.
    ///
    /// Python's `exclude` defaults to no TextBoxes and `hold` to True.
    pub fn poll(&mut self, exclude: &HashSet<i64>, hold: bool) -> Result<Vec<ScreenText>> {
        self.layouts.refresh();
        self.polls += 1;
        self.message_lines.clear();
        self.frames.clear();
        let outside = std::mem::take(&mut self.outside);
        let mut found: IndexMap<i64, ScreenText> = IndexMap::new();
        let mut designed: IndexMap<i64, ScreenText> = IndexMap::new();
        // A layout's own tree finds its TextBoxes as soon as its work is known, where a sweep of
        // the linear heap took seconds: AreaNav's "Littleroot Town" (spike 49).
        let every: BTreeSet<i64> = self
            .boxes
            .iter()
            .copied()
            .chain(self.layouts.panes(lyt::TEXTBOX_CLASS))
            .collect();
        for r#box in every {
            if !exclude.contains(&r#box) {
                let (piece, written) = self.shown(r#box)?;
                if let Some(piece) = piece {
                    if written {
                        found.insert(r#box, piece);
                    } else {
                        designed.insert(r#box, piece);
                    }
                }
            }
        }
        for (r#box, piece) in separators(&designed, &found) {
            found.insert(r#box, piece);
        }
        // `{p.root for p in (*found.values(), *designed.values())}`, a set iterated in its order.
        let roots: Vec<i64> = PySet::of(found.values().chain(designed.values()).map(|p| p.root))
            .iter()
            .collect();
        for (r#box, piece) in self.named_pictures(&roots)? {
            found.insert(r#box, piece);
        }
        found = after_the_name(&found);
        if let Some(level_mark) = &self.level_mark {
            found = level_marks(&found, level_mark);
        }
        // A pane read while the game rewrites it can sit 1,000 units off for one poll (spike 50),
        // and the move list read "Growl PP 40/" and then "3535" as its pieces came back. Words
        // missing for a single poll are taken as still there.
        let carried: IndexMap<i64, ScreenText> = self
            .last
            .iter()
            .filter(|&(r#box, _)| {
                !found.contains_key(r#box)
                    && !exclude.contains(r#box)
                    && !self.carried.contains(r#box)
            })
            .map(|(&r#box, piece)| (r#box, piece.clone()))
            .collect();
        self.carried = carried.keys().copied().collect();
        let mut now: IndexMap<i64, ScreenText> = found.clone();
        now.extend(carried);
        self.last = now.clone();
        self.on_screen = now.values().cloned().collect();
        self.on_screen.sort_by_key(reading_order);
        let mut root_since: IndexMap<i64, i64> = IndexMap::new();
        for p in now.values() {
            let since = self.root_since.get(&p.root).copied().unwrap_or(self.polls);
            root_since.insert(p.root, since);
        }
        self.root_since = root_since;
        let mut box_since: IndexMap<i64, i64> = IndexMap::new();
        for r#box in now.keys() {
            let since = self.box_since.get(r#box).copied().unwrap_or(self.polls);
            box_since.insert(*r#box, since);
        }
        self.box_since = box_since;
        let mut scrolled = std::mem::take(&mut self.scrolled);
        scrolled.extend(outside);
        self.scrolled = scrolled
            .into_iter()
            .filter(|(r#box, text)| found.get(r#box).is_some_and(|piece| &piece.text == text))
            .collect();

        let mut out: Vec<ScreenText> = Vec::new();
        for (r#box, piece) in &found {
            if self.said.get(r#box) == Some(&piece.text)
                || (hold && self.seen.get(r#box) != Some(&piece.text))
            {
                continue;
            }
            let with_layout =
                get(&self.box_since, *r#box)? - get(&self.root_since, piece.root)? <= ARRIVAL_POLLS;
            let returned = !self.said.contains_key(r#box)
                && self.gone.shift_remove(r#box).as_ref() == Some(&piece.text);
            let scrolled = self.scrolled.shift_remove(r#box).is_some();
            out.push(ScreenText {
                changed: self.said.contains_key(r#box),
                with_layout,
                returned,
                scrolled,
                ..piece.clone()
            });
            self.said.insert(*r#box, piece.text.clone());
            self.last_said.insert(*r#box, out[out.len() - 1].clone());
        }
        let going: Vec<i64> = self
            .said
            .keys()
            .copied()
            .filter(|r#box| !now.contains_key(r#box))
            .collect();
        self.left = going
            .iter()
            .filter_map(|r#box| self.last_said.get(r#box).cloned())
            .collect();
        for r#box in going {
            let words = self
                .said
                .shift_remove(&r#box)
                .ok_or_else(|| Error::key(r#box.to_string()))?;
            self.gone.insert(r#box, words);
            self.last_said.shift_remove(&r#box);
        }
        self.settling = Vec::new();
        for (r#box, p) in &now {
            if self.said.get(r#box) != Some(&p.text) && !self.settling.contains(&p.root) {
                self.settling.push(p.root);
            }
        }
        self.seen = now
            .iter()
            .map(|(&r#box, piece)| (r#box, piece.text.clone()))
            .collect();
        out.sort_by_key(reading_order);
        Ok(out)
    }

    /// The words of a TextBox being drawn and where they are, or None; and whether code wrote
    /// them, which a designer's words left as the layout file has them were not.
    fn shown(&mut self, r#box: i64) -> Result<(Option<ScreenText>, bool)> {
        if self
            .names
            .of_vtable(u32(&self.layouts.pane(r#box)?, 0)?)?
            .as_deref()
            != Some(lyt::TEXTBOX_CLASS)
        {
            return Ok((None, false));
        }
        let shape = self.layouts.shape;
        let head = self.layouts.textbox(r#box)?;
        let tag = u32(&head, shape.tag_processor)?;
        if tag != 0 && self.is_window_tag(tag)? {
            if !self.message_lines.contains(&r#box) {
                self.message_lines.push(r#box);
            }
            return Ok((None, false));
        }
        let placement = self.layouts.placement(r#box)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            return Ok((None, false));
        };
        let text = self.text(&head)?;
        if text.is_empty() {
            return Ok((None, false));
        }
        let placed = placed(r#box, &text, &head, &shape)?;
        let parent = u32(&head, lyt::PARENT)?;
        let mut piece = ScreenText {
            top: placement.top,
            root: placement.root,
            panel: placement.panel,
            parent,
            ..placed
        };
        let root = self.layouts.pane(placement.root)?;
        if off_screen(&piece, f32(&root, shape.x)?, f32(&root, shape.y)?) {
            self.outside.insert(r#box, text);
            return Ok((None, false));
        }
        if piece.parent == piece.root && piece.glyph > 0.0 {
            piece.frame = self.frame_of(&piece)?;
        }
        Ok((Some(piece), tag != 0))
    }

    /// Picture -> what the game calls what it draws, as a piece where the picture is, for the
    /// pictures shown in layouts with words on screen: the Pokédex entry's type, drawn as an icon
    /// beside the word TYPE (spike 147; Orion, 2026-09-29), "Male TYPE Fire".
    fn named_pictures(&self, roots: &[i64]) -> Result<IndexMap<i64, ScreenText>> {
        let Some(pictures) = &self.pictures else {
            return Ok(IndexMap::new());
        };
        let shape = self.layouts.shape;
        let mut out: IndexMap<i64, ScreenText> = IndexMap::new();
        for &root in roots {
            let root_head = self.layouts.pane(root)?;
            for (pane, kind) in self.layouts.tree(root) {
                let placement = self.layouts.placement(pane)?;
                let Some(placement) = placement.filter(|placement| placement.shown) else {
                    continue;
                };
                if kind != lyt::PICTURE_CLASS {
                    continue;
                }
                let head = self.layouts.pane(pane)?;
                let width = f32(&head, shape.width)? * f32(&head, shape.scale_x)?;
                let height = f32(&head, shape.height)? * f32(&head, shape.scale_y)?;
                if width <= 0.0 || height <= 0.0 {
                    continue;
                }
                let Some(words) = pictures(pane)?.filter(|words| !words.is_empty()) else {
                    continue;
                };
                let (x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
                // Wholly off its screen is not on it: the summary keeps a fifth move's type icon
                // 140 above the bottom screen's middle, 130 to 150, its edge at 120 (movie7).
                let across = (x - f32(&root_head, shape.x)?).abs() - width / 2.0;
                let up = (y - f32(&root_head, shape.y)?).abs() - height / 2.0;
                if up >= SCREEN_HALF_HEIGHT
                    || across >= SCREEN_HALF_WIDTH[usize::from(placement.top)]
                {
                    continue;
                }
                // Its height stands in for a glyph's width: what keeps it in its row.
                out.insert(
                    pane,
                    ScreenText {
                        top: placement.top,
                        root: placement.root,
                        panel: placement.panel,
                        parent: u32(&head, lyt::PARENT)?,
                        picture: true,
                        ..ScreenText::placed(
                            pane,
                            words,
                            false,
                            x - width / 2.0,
                            x + width / 2.0,
                            y,
                            height,
                        )
                    },
                );
            }
        }
        Ok(side_by_side(&out))
    }

    /// The window shown in the piece's layout whose rectangle holds the words' middle, or 0.
    fn frame_of(&mut self, piece: &ScreenText) -> Result<i64> {
        if !self.frames.contains_key(&piece.root) {
            let shape = self.layouts.shape;
            let mut frames: Vec<Frame> = Vec::new();
            for (pane, kind) in self.layouts.tree(piece.root) {
                let placement = self.layouts.placement(pane)?;
                let Some(_) = placement.filter(|placement| placement.shown) else {
                    continue;
                };
                if kind != lyt::WINDOW_CLASS {
                    continue;
                }
                let head = self.layouts.pane(pane)?;
                let half_width = f32(&head, shape.width)? * f32(&head, shape.scale_x)? / 2.0;
                let half_height = f32(&head, shape.height)? * f32(&head, shape.scale_y)? / 2.0;
                let (x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
                frames.push((
                    pane,
                    x - half_width,
                    x + half_width,
                    y - half_height,
                    y + half_height,
                ));
            }
            self.frames.insert(piece.root, frames);
        }
        let middle = (piece.left + piece.right) / 2.0;
        Ok(self.frames[&piece.root]
            .iter()
            .find(|&&(_, left, right, low, high)| {
                left <= middle && middle <= right && low <= piece.row && piece.row <= high
            })
            .map_or(0, |frame| frame.0))
    }

    /// Is this the tag processor a message window embeds, so the line is a message's?
    ///
    /// A message window's line objects are left to the dialogue reader, which says a page at a
    /// time. `exclude` names them once that reader has found the window, but whatever else finds
    /// TextBoxes can find a window's lines first: the linear heap's rolling sweep did, and Birch's
    /// "Whew..." was heard twice (spike 46). The window's own tag processor names them from the
    /// start.
    fn is_window_tag(&mut self, tag: i64) -> Result<bool> {
        if !self.window_tags.contains_key(&tag) {
            let classes: Vec<&str> = self.windows.iter().map(String::as_str).collect();
            let is_window = msgwin::window_of(&self.names, tag, &classes)?.is_some();
            self.window_tags.insert(tag, is_window);
        }
        Ok(self.window_tags[&tag])
    }

    /// The TextBox's words, control codes dropped and whitespace collapsed.
    fn text(&self, head: &[u8]) -> Result<String> {
        let shape = self.layouts.shape;
        let (capacity, length) = (u16(head, shape.capacity)?, u16(head, shape.length)?);
        if !(0 < length && length <= capacity.min(MAX_TEXT)) {
            return Ok(String::new());
        }
        let spoken = (|| {
            let raw = self.memory.read(u32(head, shape.text)?, length * 2)?;
            pages(&raw)
        })();
        let spoken = match spoken {
            Ok(spoken) => spoken,
            Err(error) if error.is(Kind::ValueError) => return Ok(String::new()),
            Err(error) => return Err(error),
        };
        let pages: Vec<&str> = spoken.iter().map(|page| page.text.as_str()).collect();
        Ok(py::split(&pages.join(" ")).join(" "))
    }
}

/// Is the piece wholly outside its screen, judged from its layout's root at (root_x, root_y)?
///
/// The X menu, closed, keeps its labels visible and drawn 189 units below the middle of the bottom
/// screen, whose edge is 120 below it (spike 48). Counted as shown, they were read whenever the
/// menu's layout was built again, as the summary, the bag or the trainer card closed to the field.
///
/// A layout's global places include a view offset from its work (+0x74): 290 on the field's bottom
/// screen, 1,000 in battle, where the top screen's roots come out at (0, 1000) and its words about
/// 1,000 up. Judged from (0, 0), every battle HP box and the level up's stats were left out
/// (battles.ctm, spikes 47 and 48); the X menu's root stays at (0, 0) as it parks its labels.
/// A piece whose place is unknown is kept.
///
/// Python's `root_x` and `root_y` default to 0.0.
fn off_screen(piece: &ScreenText, root_x: f64, root_y: f64) -> bool {
    if piece.glyph <= 0.0 {
        return false;
    }
    let half_width = SCREEN_HALF_WIDTH[usize::from(piece.top)];
    (piece.row - root_y).abs() > SCREEN_HALF_HEIGHT + piece.glyph
        || piece.right - root_x < -half_width
        || piece.left - root_x > half_width
}

/// The designer's pieces holding no letter or digit that touch a written piece of their panel
/// on each side: the "/" Sun's party screen and HP box leave as the layout file has it between
/// the current HP and the maximum, which code writes. Read as written words only, the party slot
/// ran the two numbers together (sun_9.ctm). The designer's other words are placeholders, "999"
/// and fullwidth "123", in panes built for what this screen does not show, and are left out.
/// Of the panel, not the layout: as the HP box comes on, all its panes sit at one place, and a
/// double battle's panel's "/" touched the player's numbers.
fn separators(
    designed: &IndexMap<i64, ScreenText>,
    written: &IndexMap<i64, ScreenText>,
) -> IndexMap<i64, ScreenText> {
    let beside = |piece: &ScreenText, left: bool| -> bool {
        // A piece hanging straight from the root is a panel of its own, so the summary's "/",
        // between HP's numbers, was of nobody's panel and "HP 30 30" was said (spike 148).
        written.values().any(|other| {
            (other.panel == piece.panel || other.parent == piece.parent)
                && (if left {
                    touching(other, piece)
                } else {
                    touching(piece, other)
                })
        })
    };
    designed
        .iter()
        .filter(|(_, piece)| {
            !piece.text.chars().any(is_alnum) && beside(piece, true) && beside(piece, false)
        })
        .map(|(&r#box, piece)| (r#box, piece.clone()))
        .collect()
}

/// These pieces, with each lone `LEVEL_GLYPH` just left of a number in its own group, on its
/// row, read as `words`.
fn level_marks(found: &IndexMap<i64, ScreenText>, words: &str) -> IndexMap<i64, ScreenText> {
    let mut out = found.clone();
    for (&r#box, piece) in found {
        if piece.text == LEVEL_GLYPH
            && found.values().any(|other| {
                other.parent == piece.parent
                    && is_digits(&other.text)
                    && other.left > piece.left
                    && (other.row - piece.row).abs() < max_of(piece.glyph, other.glyph) / 2.0
            })
        {
            out.insert(
                r#box,
                ScreenText {
                    text: words.to_string(),
                    ..piece.clone()
                },
            );
        }
    }
    out
}

/// A group's one picture read straight after the one piece above it, as a move's type after
/// its name. The battle's move list mirrors its two columns, the type icon left of the PP on one
/// and right of it on the other, and read as drawn the right one's was "Growl PP 40/40 Normal";
/// where the mirroring cannot be seen it only sounds lopsided (Orion, 2026-09-29), so it is
/// "Growl Normal PP 40/40". Only a group of two rows with one piece on the upper is shaped so:
/// the Pokédex entry's TYPE is one of three rows, and stays "TYPE Fire".
pub fn after_the_name(found: &IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
    let mut groups: IndexMap<i64, Vec<&ScreenText>> = IndexMap::new();
    for piece in found.values() {
        groups.entry(piece.parent).or_default().push(piece);
    }
    let mut out = found.clone();
    for pieces in groups.values() {
        let pictures: Vec<&ScreenText> = pieces.iter().copied().filter(|p| p.picture).collect();
        if pictures.len() != 1 {
            continue;
        }
        let icon = pictures[0];
        let words: Vec<&ScreenText> = pieces.iter().copied().filter(|p| !p.picture).collect();
        let above: Vec<&ScreenText> = words
            .iter()
            .copied()
            .filter(|p| p.row - icon.row >= icon.glyph / 2.0)
            .collect();
        let beside: Vec<&ScreenText> = words
            .iter()
            .copied()
            .filter(|p| (p.row - icon.row).abs() < icon.glyph / 2.0)
            .collect();
        if above.len() == 1 && !beside.is_empty() && above.len() + beside.len() == words.len() {
            out.insert(
                icon.r#box,
                ScreenText {
                    leads: true,
                    ..icon.clone()
                },
            );
        }
    }
    out
}

/// Pictures drawn side by side as one piece, their words between slashes: a Pokémon of two
/// types is "Bug/Water" (Orion, 2026-09-29), where joined as drawn they read "BugWater". A
/// picture saying what the one beside it says is one thing drawn twice, as each Pokédex language
/// tab is.
fn side_by_side(pictures: &IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
    let mut out: IndexMap<i64, ScreenText> = IndexMap::new();
    let mut last: Option<ScreenText> = None;
    // The key is `(not top, root, -row, left)`: a tuple is smaller by its first element that
    // differs.
    let ordered = sorted_by_lt(
        pictures.values().collect(),
        |a: &&ScreenText, b: &&ScreenText| {
            if a.top != b.top {
                return a.top && !b.top;
            }
            if a.root != b.root {
                return a.root < b.root;
            }
            if -a.row != -b.row {
                return -a.row < -b.row;
            }
            a.left < b.left
        },
    );
    for piece in ordered {
        if let Some(before) = last.clone()
            && before.root == piece.root
            && touching(&before, piece)
        {
            if !before.text.split('/').any(|part| part == piece.text) {
                let joined = ScreenText {
                    text: format!("{}/{}", before.text, piece.text),
                    right: piece.right,
                    ..before
                };
                out.insert(joined.r#box, joined.clone());
                last = Some(joined);
            }
            continue;
        }
        out.insert(piece.r#box, piece.clone());
        last = Some(piece.clone());
    }
    out
}

/// The top screen first, then the order the layouts built their panes in.
pub fn reading_order(piece: &ScreenText) -> (bool, i64) {
    (!piece.top, piece.r#box)
}

/// The words with where they are estimated to be drawn. A pane's global matrix is filled in when
/// it is drawn, so a scale of 0 means nothing about its place can be believed yet.
fn placed(r#box: i64, text: &str, head: &[u8], shape: &lyt::PaneShape) -> Result<ScreenText> {
    let scale = f32(head, shape.scale_x)?;
    if scale <= 0.0 {
        return Ok(ScreenText::new(r#box, text, false));
    }
    let width = f32(head, shape.width)? * scale;
    let glyph = f32(head, shape.glyph_width)? * scale;
    let box_left = f32(head, shape.x)? - FRACTION[shape.anchored(head)? as usize] * width;
    let words = min_of(width, text.chars().count() as f64 * glyph * GLYPH_ADVANCE);
    let left = box_left + (width - words) * FRACTION[shape.aligned(head)? as usize];
    Ok(ScreenText::placed(
        r#box,
        text,
        false,
        left,
        left + words,
        f32(head, shape.y)?,
        glyph,
    ))
}
