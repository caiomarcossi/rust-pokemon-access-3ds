//! The Pokédex draws a sixth (spike 95): a **plate**, a window with no panes of its own that never
//! moves, while the list's rows scroll under it — three or more plain panes under one parent, each
//! holding words, all at one x and of one height. The item is the row the plate sits on.

use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::f32;
use crate::games::common::cursors::menu::MenuCursors;
use crate::games::common::cursors::shared::{Panes, class_of, near, on};
use crate::games::common::layout::MAX_DEPTH;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::py::{PySet, Result};

/// The fewest like rows that make a list a plate can sit on. The Pokédex shows five at the end of
/// its list and nine elsewhere.
pub const LIST_ROWS: i64 = 3;
/// The most rows a plate is tall. The Pokédex's is 40 units on rows of 30; a window as tall as
/// several rows is a panel's frame, not a plate on one of them.
pub const PLATE_ROWS: f64 = 2.0;

/// (root, the row it sits on, the rows of its list).
pub type Plate = (i64, i64, Vec<i64>);

/// Window -> (root, the row it sits on, the rows of its list), for every window shown with
/// no panes of its own that sits on a row of a list: the Pokédex's plate, which stays where
/// it is while the rows scroll under it (spike 95).
///
/// A window directly under its root is a frame, and one sliding between choices a choice's
/// cursor, each read as such. A window as tall as several rows frames a panel, and a window
/// inside a row is part of that row. While the rows slide, the one nearest the plate's middle
/// is the one it sits on.
pub fn plates(
    panes: &Panes,
    menus: &MenuCursors,
    on_screen: &[ScreenText],
    choices: &HashSet<i64>,
) -> Result<IndexMap<i64, Plate>> {
    let layouts = &panes.layouts;
    let mut found: IndexMap<i64, Plate> = IndexMap::new();
    let mut roots = PySet::new();
    for piece in on_screen {
        roots.add(piece.root);
    }
    for root in roots.iter().collect::<Vec<_>>() {
        let mut windows: Vec<i64> = Vec::new();
        for (&pane, name) in &layouts.tree(root) {
            if name == lyt::WINDOW_CLASS
                && !menus.windows.contains(&pane)
                && !choices.contains(&pane)
                && panes.parent(pane)? != root
                && layouts.children(root, pane).is_empty()
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
        let lists = rows(panes, root, &pieces)?;
        for &window in &windows {
            let Some(rect) = panes.rect(window)? else {
                continue;
            };
            let middle = (rect.low + rect.high) / 2.0;
            for rows in &lists {
                let height = f32(&layouts.pane(rows[0])?, layouts.shape.height)?;
                if rect.high - rect.low > PLATE_ROWS * height {
                    continue;
                }
                let mut inside = false;
                for &row in rows {
                    if panes.below(window, row)? {
                        inside = true;
                        break;
                    }
                }
                if inside {
                    continue;
                }
                let mut sat: Vec<i64> = Vec::new();
                for &row in rows {
                    if on(panes.place(row)?, &rect) {
                        sat.push(row);
                    }
                }
                if !sat.is_empty() {
                    // `min(sat, key=...)`: the first of the nearest.
                    let mut row = sat[0];
                    let mut best = (panes.place(row)?.1 - middle).abs();
                    for &other in &sat[1..] {
                        let key = (panes.place(other)?.1 - middle).abs();
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
fn rows(panes: &Panes, root: i64, pieces: &[ScreenText]) -> Result<Vec<Vec<i64>>> {
    let tree = panes.layouts.tree(root);
    let mut holding: IndexSet<i64> = IndexSet::new();
    for piece in pieces {
        let mut pane = panes.parent(piece.r#box)?;
        for _ in 0..MAX_DEPTH {
            if pane == 0 || pane == root {
                break;
            }
            if class_of(&tree, pane) == Some(lyt::PANE_CLASS) {
                holding.insert(pane);
            }
            pane = panes.parent(pane)?;
        }
    }
    let mut siblings: IndexMap<i64, Vec<i64>> = IndexMap::new();
    for &pane in &holding {
        siblings.entry(panes.parent(pane)?).or_default().push(pane);
    }
    let mut lists: Vec<Vec<i64>> = Vec::new();
    for rows in siblings.values() {
        if (rows.len() as i64) < LIST_ROWS {
            continue;
        }
        let (x, _y) = panes.place(rows[0])?;
        let height = panes.size(rows[0])?.1;
        let mut alike = true;
        for &row in rows {
            if !(near(panes.place(row)?.0, x) && near(panes.size(row)?.1, height)) {
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
