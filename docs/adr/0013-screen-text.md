# 13. Screen text is heard when it comes on screen and when it changes

- **Status:** Accepted
- **Date:** 2026-09-14, revised the same day after the user tried it
- **Deciders:** Orion

## Context

Menus, the summary, the options screen and the battle's boxes draw their words straight into
layout TextBoxes rather than into a message window. There is no "screen opened" event to hang
speech on, and a layout keeps its words long after it leaves the screen.

## Decision

Words a screen draws in its own layouts are heard **when they come on screen and when they
change**. Everything that comes on screen in one poll is said together and queued, top screen
first, then in the order the layout built its panes. A value that changes in place ("FAST")
interrupts, like a menu. A list's own labels are left to the list readers, which say them under
the cursor.

The rules under it, all the user's, each from live play:

- **No commas.** Pieces are joined with spaces; "Torchic, ♂, L, 5, 19, /, 19" is clumsy through a
  screen reader. `[pa3ds.ui] pauses` puts them back — a colon after a label, a comma elsewhere
  (Orion, 2026-09-30: "commas" until the colons came) — and should put them only between groups.
  Pieces drawn side by side belong together ("L5", "19/19"), which needs each piece's place on screen.
- **Signs, not words.** The font's private-use gender glyphs become ♂ and ♀, which screen readers
  say in the player's own settings; they skip the private-use characters.
- **Menus never list their items.** Only the item under the cursor, on opening and as it moves. An
  app menu's cursor is a window alone in a layout of its own, and the item is the screen text
  inside it ([findings, spikes 39 to 41](../findings/screens-and-cursors.md)), read in place: rows
  top down, each left to right, and the same words dropped only where they are drawn twice in one
  spot ("PP 40/40", not "PP / 40").
- **A button drawn as a picture is said without "button"** (2026-09-16). Its name comes from its
  texture, `fight_button.bclim`, and the trailing word is the designer saying what the picture is.
  A texture called only `button` keeps the word. A file name is ASCII, so `pokemon_button.bclim`
  loses the accent the game writes everywhere else, and `intro_famale.bclim` is misspelt; both are
  put right through one spelling table, because a screen reader says the two differently. That
  restores a word's own spelling and is not a name pa3ds invented (ADR-0023 covers names it does).
- **A menu's layout belongs to its cursor.** A layout a cursor has been over is a menu: what comes
  on screen with it, its rows as a list scrolls, and whatever a cursor is over, are left to the
  cursor. Words that come later in a menu already up, such as "Would you like to make these
  changes?", are said. (A 0.8 s wait for the cursor, tried first, lagged and still leaked labels —
  [findings, spike 47](../findings/screens-and-cursors.md).)
  **Narrowed 2026-09-24 for a cursor that knows its list** (a plate or a named cursor, spike 95):
  it owns the list and no more of its layout. The Poké Mart draws its list, the money, the count
  in the bag and the chosen item's description in one layout, and left to the cursor whole, the
  description would never be heard as the Mart opens nor the count as it changes. The rest of such
  a layout is heard as any screen's words are, and what changes as the cursor moves is said as
  its details. It is still a menu for coming back: words that come back to it come alone.
- **A screen coming back is read like a screen opening**, in full, unless it is a menu. Words that
  come back to a layout which kept its other words on screen bring those with them. Words new to a
  screen already up, as a question is, come alone.
- **A changed value is said with its row**, from its own group of panes: "PP 40/40", "TEXT SPEED
  FAST", not "40", and not a neighbouring panel's words on the same line.
- **Details follow the cursor, whole and in one order.** What changes after a cursor moves is
  gathered until it has settled, then every panel that cursor's moves have changed is read, row by
  row, in the order the screen built them: "The user growls… Category Power — Accuracy 100".
  Saying each change as a poll caught it put the description before the panel or after it. A
  cursor coming on screen brings no details. **Narrowed 2026-09-28 for words hanging straight
  from a layout's root:** the move-forget screen groups nothing, and its "panel" was the whole
  top screen, so every move read the Pokémon's number, name and six stats before its power. Such
  words are grouped by the window drawn round them, one round Category, Power and Accuracy and
  one round the description; words in no frame stay one group (spike 139).
- **A written text covers the one before it.** A description written over a window waits one whole
  screen poll, so it follows the cursor item it describes; then only the last one written is said.
- **Order within one poll:** the item under a cursor, then changed rows, then what came on screen
  (`src/pa3ds/games/common/announce.py`).

## Consequences

This is the largest single body of behaviour in pa3ds and the one most likely to need retuning; it
is also the one with the most live-play evidence behind each clause. Changing a clause without a
replay to compare against is how a regression gets in.
