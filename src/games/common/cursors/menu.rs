//! App menus draw their cursor — the red corners around the chosen item — as a layout of its own: a
//! root pane whose only child is an `nw::lyt::Window` (spike 40). The window sits on the chosen item
//! at the item's size (spike 39), so the item under the cursor is the screen text inside the window's
//! rectangle, on the screen the cursor is drawn to (spikes 40 and 41). The X menu, the bag and the save
//! prompt's list all draw theirs this way.
//!
//! ```text
//! Window  +0x0c -> the layout's root    +0x48 f32 width    +0x4c f32 height
//!         +0x80 f32 global scale x      +0x8c f32 x        +0x94 f32 global scale y   +0x9c f32 y
//! root    +0x0c 0, a root has no parent                    +0x10 u32 child count, 1 for a cursor
//! ```
//!
//! Sun's name entry keyboard builds its cursor the same way, as a window alone under a Parts pane, a
//! layout built into the keys' own (spike 112).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, Panes, Rect, Said, framed, place_of};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::py::Result;

/// The windows that are an app menu's cursor, each read on every poll once found.
pub struct MenuCursors {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    names: Rc<ClassNames>,
    /// The cursors found so far.
    pub windows: Vec<i64>,
}

impl MenuCursors {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>, names: Rc<ClassNames>) -> Self {
        MenuCursors {
            layouts: panes.layouts.clone(),
            panes,
            said,
            names,
            windows: Vec::new(),
        }
    }

    /// Read the cursors among these windows from now on, as well as every one found before.
    ///
    /// Most windows are not cursors, so only those shaped as one when found are kept. A cursor
    /// once found is read on every poll after: menus are rebuilt where they were, and a sweep
    /// takes seconds to find a new one. Cursor pictures are found in the layouts themselves.
    pub fn track(&mut self, windows: impl IntoIterator<Item = i64>) -> Result<()> {
        for window in windows {
            if !self.windows.contains(&window) && self.shaped(window)? {
                self.windows.push(window);
            }
        }
        Ok(())
    }

    /// Report the item under each menu cursor on screen.
    pub fn poll(&self, on_screen: &[ScreenText], items: &mut Vec<CursorItem>) -> Result<()> {
        for &cursor in &self.windows {
            let Some(rect) = self.rect(cursor)? else {
                self.said.forget(cursor);
                continue;
            };
            // A cursor built into a layout frames that layout's words: Sun's keyboard keeps a
            // second, parked over the keys of the page drawn in front of it (spike 112).
            let own = self.built_into(cursor)?;
            let theirs: Vec<ScreenText> = on_screen
                .iter()
                .filter(|piece| own.is_none() || own == Some(piece.root))
                .cloned()
                .collect();
            let mut inside = framed(&theirs, &rect);
            if inside.is_empty() {
                inside = self.pictures_inside(&rect)?;
            }
            let at = place_of(&rect)?;
            self.said.report(cursor, inside, items, at, false);
        }
        Ok(())
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
        Ok(self.panes.cursor_panes
            && self.names.of_vtable(u32(&head, 0)?)?.as_deref() == Some(lyt::PARTS_CLASS))
    }

    /// Where the cursor is, or None when it is not shaped as a cursor or not on screen.
    fn rect(&self, cursor: i64) -> Result<Option<Rect>> {
        if !self.shaped(cursor)? {
            return Ok(None);
        }
        self.panes.rect(cursor)
    }

    /// The root of the layout a cursor window is built into, as Sun's keyboard's is under a
    /// Parts pane, or None for a cursor that is a layout of its own.
    fn built_into(&self, window: i64) -> Result<Option<i64>> {
        let parent = self.panes.parent(window)?;
        let vtable = u32(&self.layouts.pane(parent)?, 0)?;
        if self.names.of_vtable(vtable)?.as_deref() != Some(lyt::PARTS_CLASS) {
            return Ok(None);
        }
        let placement = self.layouts.placement(window)?;
        Ok(placement.map(|placement| placement.root))
    }

    /// What the game calls the pictures shown inside a cursor with no words under it: the
    /// Pokédex's language tabs, whose textures the game swaps as it runs, each drawn as two
    /// pictures that name one language (spike 147).
    fn pictures_inside(&self, rect: &Rect) -> Result<Vec<ScreenText>> {
        let Some(pictures) = self.panes.pictures.clone() else {
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
                let (x, y) = self.panes.place(pane)?;
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
                        parent: self.panes.parent(pane)?,
                        ..ScreenText::new(pane, words.clone(), false)
                    };
                    said.insert(words, piece);
                }
            }
        }
        Ok(said.into_values().collect())
    }
}
