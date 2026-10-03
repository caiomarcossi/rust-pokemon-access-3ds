# 12. A menu says the item under its cursor, and never lists itself

- **Status:** Accepted
- **Date:** 2026-09-13
- **Deciders:** Orion

## Decision

- **On open:** the title, then the focused item. **On move:** the focused item only.
- **No separate review mode** — the menus are small and moving the game's own cursor is review
  enough.
- **No position** ("Bag, 2 of 5"): ORAS menus wrap around, so the position says little, and the
  user prefers it off (2026-09-13). It was said on opening with `--positions`, and for a while
  on 2026-09-30 with two settings, taken out that day since only lists knew their length
  (ADR-0026). A list's place is logged.
- A **choice box** in the field follows the same rule as a list window: the option under the
  cursor when it opens, then each option moved to.
- Text a screen writes **over** a message window, such as the options screen's description,
  follows the cursor too, so it interrupts like a menu.

## Consequences

**The main menu was the one exception, until 2026-09-24.** Its cursor is a frame in the menu's
own layout, known as a cursor only once it had moved from one rest to another
([findings, spike 72](../findings/screens-and-cursors.md)), so the menu was read whole as it
opened — CONTINUE's panel first — and one item a move after that. The frame is drawn with
`cursor_lt.bclim`, and a window its designer textured as a cursor is now one from the first poll
(spike 96): the main menu says CONTINUE's panel alone as it opens.

**Amended 2026-09-29 ([spike 143](../findings/screens-and-cursors.md)): a button of another
layout.** Where a cursor rests on no item of its own layout, a lone button of another layout on
its screen, whose touch area it rests on the middle of, is its item, said by the words of the
picture beside that touch area, as a picture button is said: the Box's pointer, down from its
grid, rests on the bottom bar's `change_box.bclim` in `touch_bar_lower`, a layout many screens
share, and says "change box". A lone button of the cursor's own layout stays unsaid, and so do
buttons only touch reaches: the Box's modes and its arrows. A cursor is followed in its layout
while the layout lasts, its words all hidden or not; hidden, as under the Box's box list, it says
only such a button, and the words the screen above says as it moves (the box's name) stay
screen text's (the user heard it right, 2026-09-29).
