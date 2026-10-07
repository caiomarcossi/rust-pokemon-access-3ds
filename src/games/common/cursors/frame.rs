//! The main menu draws a fourth (spike 72): the same red corners as an app menu's, but as a window
//! in the menu's own layout, directly under its root among the item plates, moved and resized onto the
//! chosen item. A frame directly under a root is taken for a cursor once it has moved on its own, its
//! sibling windows staying where they were, from one resting place to another: a list that scrolls
//! moves its rows together, a frame sliding in is somewhere new every poll, and a window that never
//! moves is a panel's border. Until it moves, the screen is read as any screen is.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::cursors::menu::MenuCursors;
use crate::games::common::cursors::shared::{
    CursorItem, Panes, Rect, Said, framed, place_of, spot_of,
};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::py::Result;

/// Polls a window directly under a root holds one place and size for it to count as resting there.
/// A cursor jumps from rest to rest; a frame sliding onto the screen is somewhere new every poll until
/// it stops, and taking that for a cursor would make a menu of the screen it frames.
pub const FRAME_REST: i64 = 2;

/// Windows directly under a layout's root that have moved on their own from rest to rest.
///
/// Python's keeps the menu cursors (`menus`), which are cursors of their own kind; here they are
/// handed to `poll`, the reader holding both.
pub struct MovedFrames {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
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
}

impl MovedFrames {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        MovedFrames {
            layouts: panes.layouts.clone(),
            panes,
            said,
            frames: IndexMap::new(),
            frames_missed: IndexMap::new(),
            pending_frames: IndexMap::new(),
            moved_frames: HashSet::new(),
        }
    }

    /// Report what each frame that has moved is over.
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
        menus: &MenuCursors,
    ) -> Result<()> {
        for (window, rect) in self.moved_frame_rects(menus)? {
            let inside = framed(on_screen, &rect);
            let at = place_of(&rect)?;
            self.said.report(window, inside, items, at, false);
        }
        Ok(())
    }

    /// Window -> its rectangle, for every window directly under a root that is shown now and
    /// has moved on its own from one resting place to another: the main menu's cursor, and not
    /// the border of a panel, the rows of a list scrolling together, or a frame sliding in or
    /// out, whose place changes every poll and rests only where it stops.
    ///
    /// A window lost for one poll is not taken for one gone: a pane read mid-update can sit off
    /// for a poll (spike 50), and a cursor forgotten then would have to move again to be heard.
    fn moved_frame_rects(&mut self, menus: &MenuCursors) -> Result<IndexMap<i64, Rect>> {
        let frames = self.frames_shown(menus)?;
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
                self.said.forget(window);
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
    fn frames_shown(&self, menus: &MenuCursors) -> Result<IndexMap<i64, (i64, Rect)>> {
        let mut frames: IndexMap<i64, (i64, Rect)> = IndexMap::new();
        for root in self.layouts.roots() {
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::WINDOW_CLASS || menus.windows.contains(&pane) {
                    continue;
                }
                if self.panes.parent(pane)? != root {
                    continue;
                }
                if let Some(rect) = self.panes.rect(pane)? {
                    frames.insert(pane, (root, rect));
                }
            }
        }
        Ok(frames)
    }
}
