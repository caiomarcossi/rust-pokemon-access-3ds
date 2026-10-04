//! Slots: the places a named cursor rests on where no words are, said by where they are.
//!
//! The PC Box's pointer, `cursol01.bclim`, walks its grid of Pokémon and its party as it walks its
//! menus, but a slot holds an icon and no words (spike 129). Each slot is an `nw::lyt::Bounding`
//! touch area, and the pointer comes to rest exactly on the touch area's place:
//!
//! - **A grid**, the box: touch areas of one size under the layout's root, thirty of them in six
//!   columns and five rows, 30 units apart. A slot is said by the words just above the grid, the
//!   box's name, and its row and column: "BOX 1, row 2, column 3".
//! - **A list**, the party: the touch areas a panel holds, six in two columns, the second column 20
//!   units lower than the first, numbered in reading order: "Party 2".
//! - **A mark**, of the six a Pokémon can be given: a touch area beside a picture textured
//!   `box_mark_01` to `box_mark_06`, said by the sign the picture shows and whether it is set: a
//!   mark set is drawn solid, one not set half-transparent (or/movie11_pokedex_and_box, where
//!   setting ▲ took its picture from alpha 127 to 255, and the save holds it set).
//!
//! A slot holding a Pokémon has its icon (`poke_icon.bclim`) exactly on its place; the one the
//! player carries rides 10 units above it, so an empty slot is one no icon sits exactly on. The
//! top screen's summary of what is under the pointer changes as it moves, and is read after the
//! slot as its details (the user, 2026-09-27: where, then the summary; where, then "empty").
//! While a Pokémon is carried the summary is the carried one's, whatever slot the pointer is on,
//! so a slot is said alone.

use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::lyt;
use crate::games::common::pictures::{EMPTY, OFF, ON, mark, words};
use crate::games::common::screen::ScreenText;
use crate::py::{self, Error, PySet, Result, sorted_by_lt};

pub const PARTY: &str = "Party";
pub const ROW: &str = "row";
/// pa3ds's own words for a slot: the game names neither the party's slots nor a box's rows.
pub const COLUMN: &str = "column";
/// The alpha from which a mark's picture is drawn as set: 255 set, 127 not (spike 129).
pub const SET_ALPHA: i64 = 192;
/// The texture a slot's Pokémon is drawn with.
pub const ICON: &str = "poke_icon";
/// Units within which a pointer rests on a touch area, and an icon sits on a slot: both exact.
pub const SAME_SPOT: f64 = 1.0;
/// How far above the slot the pointer rests on the icon of the Pokémon carried rides (spike 129).
pub const CARRIED: f64 = 10.0;

/// `list.index(item)`, raising `ValueError` where the item is not in the list.
#[track_caller]
fn index_of(items: &[i64], item: i64) -> Result<usize> {
    items
        .iter()
        .position(|&each| each == item)
        .ok_or_else(|| Error::value(format!("{item} is not in list")))
}

/// The touch area a cursor resting at (x, y) is on, what to say of it, and whether it is said
/// alone, the summary left unsaid: an empty slot, or any while a Pokémon is carried. None where it
/// rests on no slot, as on a lone button's touch area.
pub fn slot(
    layouts: &Layouts,
    file: Option<&LayoutFile>,
    root: i64,
    x: f64,
    y: f64,
    on_screen: &[ScreenText],
) -> Result<Option<(i64, String, bool)>> {
    let tree = layouts.tree(root);
    let mut touch: Option<i64> = None;
    for (pane, kind) in &tree {
        if kind == lyt::BOUNDING_CLASS
            && shown(layouts, *pane)?
            && on(place(layouts, *pane)?, (x, y))
        {
            touch = Some(*pane);
            break;
        }
    }
    let Some(touch) = touch else {
        return Ok(None);
    };
    let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
    if let Some(file) = file {
        let sign = mark_of(layouts, file, root, parent, touch)?;
        if let Some(sign) = sign {
            return Ok(Some((touch, sign, false)));
        }
    }
    let size = size(layouts, touch)?;
    let mut group: Vec<i64> = Vec::new();
    for pane in layouts.children(root, parent) {
        if tree.get(&pane).map(String::as_str) == Some(lyt::BOUNDING_CLASS)
            && shown(layouts, pane)?
            && on(self::size(layouts, pane)?, size)
        {
            group.push(pane);
        }
    }
    let mut r#where = in_grid(layouts, touch, &group, root, on_screen)?;
    if r#where.is_none() && parent != root && group.len() > 1 {
        let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
        for &pane in &group {
            places.insert(pane, place(layouts, pane)?);
        }
        // The key is `(-y, x)`: a tuple is smaller by its first element that differs.
        let order = sorted_by_lt(group.clone(), |a, b| {
            if -places[a].1 != -places[b].1 {
                return -places[a].1 < -places[b].1;
            }
            places[a].0 < places[b].0
        });
        r#where = Some(format!("{PARTY} {}", index_of(&order, touch)? + 1));
    }
    let Some(r#where) = r#where else {
        return Ok(None);
    };
    let place = place(layouts, touch)?;
    let empty = match file {
        None => true,
        Some(file) => !holds_icon(layouts, file, root, place)?,
    };
    let carrying = match file {
        None => false,
        Some(file) => holds_icon(layouts, file, root, (place.0, place.1 + CARRIED))?,
    };
    Ok(Some((
        touch,
        if empty {
            format!("{}, {EMPTY}", r#where)
        } else {
            r#where
        },
        empty || carrying,
    )))
}

/// A lone button of another layout on the cursor's screen whose touch area the cursor rests
/// on the middle of, what to say of it, and that it is said alone: the words of the picture
/// beside its touch area. The Box's pointer, down from its grid, rests on the bottom bar's
/// `change_box.bclim`, in `touch_bar_lower`, a layout many screens share, where the Box's own
/// leaves an empty pane (spike 143, movie12). A lone button of the cursor's own layout stays
/// unsaid: the Box's modes are pictures named `mode01` to `mode03`, and the pointer never went to
/// them.
pub fn button(
    layouts: &Layouts,
    files: &LayoutFiles,
    root: i64,
    x: f64,
    y: f64,
    top: bool,
) -> Result<Option<(i64, String, bool)>> {
    for other in layouts.roots_set().difference(&PySet::of([root])).iter() {
        let tree = layouts.tree(other);
        for (touch, kind) in &tree {
            let touch = *touch;
            if kind != lyt::BOUNDING_CLASS || !on(place(layouts, touch)?, (x, y)) {
                continue;
            }
            let placement = layouts.placement(touch)?;
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown || placement.top != top {
                continue;
            }
            let Some(file) = files.matching(&layouts.pane_names(other)) else {
                continue;
            };
            let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
            for pane in layouts.children(other, parent) {
                if tree.get(&pane).map(String::as_str) != Some(lyt::PICTURE_CLASS)
                    || !shown(layouts, pane)?
                {
                    continue;
                }
                if let Some(textures) = file.textures.get(&name(layouts, pane)?)
                    && !textures.is_empty()
                {
                    return Ok(Some((touch, words(&textures[0]), true)));
                }
            }
        }
    }
    Ok(None)
}

/// The slot's row and column, in the full grid of touch areas it shares rows and columns with,
/// after the words just above the grid; None when those make no grid two by two or more.
fn in_grid(
    layouts: &Layouts,
    touch: i64,
    group: &[i64],
    root: i64,
    on_screen: &[ScreenText],
) -> Result<Option<String>> {
    let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
    for &pane in group {
        places.insert(pane, place(layouts, pane)?);
    }
    // `grid` is a set of ints iterated below, so it is kept as the order its members were added
    // in and laid out as Python lays it out.
    let mut grid: Vec<i64> = vec![touch];
    let mut frontier: Vec<i64> = vec![touch];
    while let Some(next) = frontier.pop() {
        let here = *places
            .get(&next)
            .ok_or_else(|| Error::key(next.to_string()))?;
        for (&pane, place) in &places {
            if !grid.contains(&pane)
                && ((place.0 - here.0).abs() <= SAME_SPOT || (place.1 - here.1).abs() <= SAME_SPOT)
            {
                grid.push(pane);
                frontier.push(pane);
            }
        }
    }
    let members: Vec<i64> = PySet::of(grid.iter().copied()).iter().collect();
    let mut xs: BTreeSet<i64> = BTreeSet::new();
    for pane in &members {
        xs.insert(py::round(places[pane].0)?);
    }
    let mut ys: BTreeSet<i64> = BTreeSet::new();
    for pane in &members {
        ys.insert(py::round(places[pane].1)?);
    }
    let xs: Vec<i64> = xs.into_iter().collect();
    let ys: Vec<i64> = ys.into_iter().rev().collect();
    if xs.len() < 2 || ys.len() < 2 || xs.len() * ys.len() != grid.len() {
        return Ok(None);
    }
    let (x, y) = places[&touch];
    let row = index_of(&ys, py::round(y)?)? + 1;
    let column = index_of(&xs, py::round(x)?)? + 1;
    let width = size(layouts, touch)?.0 / 2.0;
    let mut title: Option<&ScreenText> = None;
    for piece in on_screen {
        if piece.root == root
            && piece.glyph > 0.0
            && piece.row > ys[0] as f64
            && xs[0] as f64 - width <= (piece.left + piece.right) / 2.0
            && (piece.left + piece.right) / 2.0 <= xs[xs.len() - 1] as f64 + width
            && title.is_none_or(|first| piece.row < first.row)
        {
            title = Some(piece);
        }
    }
    let title = title
        .map(|piece| piece.text.as_str())
        .filter(|text| !text.is_empty());
    Ok(Some(format!(
        "{}{ROW} {row}, {COLUMN} {column}",
        title.map_or(String::new(), |title| format!("{title}, "))
    )))
}

/// The sign of the mark whose picture lies within this touch area, beside it, if one does.
fn mark_of(
    layouts: &Layouts,
    file: &LayoutFile,
    root: i64,
    parent: i64,
    touch: i64,
) -> Result<Option<String>> {
    let (x, y) = place(layouts, touch)?;
    let (width, height) = size(layouts, touch)?;
    for pane in layouts.children(root, parent) {
        if layouts.tree(root).get(&pane).map(String::as_str) != Some(lyt::PICTURE_CLASS) {
            continue;
        }
        let (px, py) = place(layouts, pane)?;
        if (px - x).abs() > width / 2.0 || (py - y).abs() > height / 2.0 {
            continue;
        }
        let Some(textures) = file.textures.get(&name(layouts, pane)?) else {
            continue;
        };
        for texture in textures {
            if let Some(sign) = mark(texture) {
                let alpha = *layouts
                    .pane(pane)?
                    .get(layouts.shape.alpha as usize)
                    .ok_or_else(|| Error::index("index out of range"))?;
                return Ok(Some(format!(
                    "{sign}, {}",
                    if i64::from(alpha) >= SET_ALPHA {
                        ON
                    } else {
                        OFF
                    }
                )));
            }
        }
    }
    Ok(None)
}

/// Does a Pokémon's icon sit exactly on this place?
fn holds_icon(layouts: &Layouts, file: &LayoutFile, root: i64, at: (f64, f64)) -> Result<bool> {
    for (pane, kind) in &layouts.tree(root) {
        let pane = *pane;
        if kind == lyt::PICTURE_CLASS && shown(layouts, pane)? && on(place(layouts, pane)?, at) {
            let name = name(layouts, pane)?;
            if let Some(textures) = file.textures.get(&name)
                && textures.iter().any(|texture| texture.starts_with(ICON))
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn shown(layouts: &Layouts, pane: i64) -> Result<bool> {
    Ok(layouts
        .placement(pane)?
        .is_some_and(|placement| placement.shown))
}

fn place(layouts: &Layouts, pane: i64) -> Result<(f64, f64)> {
    let head = layouts.pane(pane)?;
    Ok((f32(&head, layouts.shape.x)?, f32(&head, layouts.shape.y)?))
}

fn size(layouts: &Layouts, pane: i64) -> Result<(f64, f64)> {
    let head = layouts.pane(pane)?;
    Ok((
        f32(&head, layouts.shape.width)?,
        f32(&head, layouts.shape.height)?,
    ))
}

fn on(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() <= SAME_SPOT && (a.1 - b.1).abs() <= SAME_SPOT
}

fn name(layouts: &Layouts, pane: i64) -> Result<String> {
    let raw = layouts.name(pane)?;
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    Ok(py::decode_ascii(&raw[..end]))
}
