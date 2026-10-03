# Screens and cursors

Layouts, panes, screen text and the four kinds of cursor: menus, the battle's buttons, the
keyboard's keys and a frame that slides onto a choice.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 18 — the cursor moves

2026-09-13 (`spikes/13_menu_cursor.py --movie movie2.ctm --from-start`). The user recorded movie2
from a deleted NAND and SD card, arrowing up and down through the Yes/No prompts.

- **`MenuWindow` +0x2c is the cursor index.** On the second list window the replay builds, +0x2c
  was written 0, 1, 0, 1, 0, 1 with its labels reading `YES` and `NO`. The writes came from
  `MenuWindowController` code (`0x003553ac`, `0x003554c4`), beside the setter's inlined copies.
  Spike 14's reading of the code was right.
- Three list windows in the whole replay, all at `0x086ff288`: one built 26 s in and freed at
  125 s with its cursor never moved (the early one spike 17 saw), the one whose cursor moved, built
  at 126.5 s, and one built at 303.9 s whose cursor stayed at 0, freed at 340 s. Each new one was
  built where the last was freed, which is what the constructor breakpoint's `this` said every time.
- **That run played at real speed, not at 10x, and so never reached the later prompts.** The rig
  had set `frame_limit=100` for an earlier run; 100 is Azahar's built-in default, so Azahar
  re-saved its config at startup with `frame_limit\default=true`, and from then on ignored the
  stored value — the `\default` trap from the verified facts, sprung by the rig itself. Restoring
  only the value line left the file saying 1000 while every later run went at 100%. Twelve
  minutes of wall time covered the opening and not the nickname or May prompts. `rig.py` now
  writes the flag with the value.
- Rerun at true 10x with each hit labelled by the message on screen: the window whose cursor went
  0, 1, 0, 1, 0, 1 is the name confirmation, "So you're wee?", answered NO; Birch asks for the name
  again, a new list window is built, and that one is answered YES without moving. Both are the
  opening's; neither is the early, hidden one.
- **Then no list window was caught for the rest of the replay** — not at the starter choice, the
  nickname question or the Route 103 question, all of which the same replay reaches (spike 19).
- **They are list windows, found another way.** `spikes/16_list_shown.py` swept for the whole
  family at 10x, labelled by the message on screen. At 181 s an `AppMenuDrawer` and a
  `MenuWindowController` appear, and the controller stays until half a second after "why not give
  a nickname to that Treecko?". Its +0x0c and the drawer's +0x08 both point at a `MenuWindow` at
  `0x084fb398` — the address of movie1's starter-choice window. No sweep found that window and the
  breakpoint never stopped in its constructor; why is not established. `regions()` samples one page
  per 16 KiB and can miss a block whose first page is blank, which would explain the sweep but not
  the breakpoint.
- So a list window is reached through its controller: `MenuWindowController` +0x08 is the drawer
  and +0x0c the window, and `AppMenuDrawer` +0x08 and +0x18 point at the same window, +0x0c back at
  the controller. Being findable still says nothing about being shown: the opening's controller
  stayed findable, vtable and links intact, for three minutes after its prompts had gone.

## Spike 27 — the options screen's descriptions

2026-09-13 (`spikes/21_options_text.py`, movie3 slot 0, saved as Options opened, real speed).

- **The description box is an ordinary message window whose message never changes.** It holds the
  text speed description throughout. Moving to another option writes that option's whole
  description, newline included, straight into the window's first line object. Checked against the
  picture: BATTLE STYLE highlighted, and its description in that line.
- **What the reader called a line's owner link is its tag processor** (`print::XYTagProc`). A window
  embeds its own at +0x9c, which points back at the window. The options screen writes with one of
  its own, a static object at `0x00630d80`. The reader took that for another window's and treated
  the window as gone, so no description after the first was heard, and neither was text speed's
  when the cursor came back.
- **Text speed's description is retyped** under the screen's tag processor when the cursor returns,
  and again each time the speed changes, as a demonstration, sometimes before the last typing
  finished. Those are the repeats heard earlier over every speed choice.
- **The reader now:** a window is gone only when a line uses another *window's* tag processor. Text
  under any other tag processor that matches no line of the message is written text, spoken when it
  changes, interrupting. The window's own message coming back over written text is a new showing,
  and typing that starts over under a tag processor not the window's own is not.
- **"Would you like to make these changes?" is a list window**, labels "Yes" and "No", read as one.
  Its question was in no message window, like "Choose this Pokémon?". The option names and values
  (TEXT SPEED, SLOW) are not read by anything yet.
- **Run live from slot 0 at real speed, the runtime spoke every description as the cursor reached
  it,** text speed's again when the cursor came back ("again after text was written over it"),
  nothing while the speed demonstration retyped, and the confirmation's Yes and No.

## Spike 28 — the text no message window holds

2026-09-14 (`spikes/23_layout_text.py`, movie3 slot 0, real speed). From here on states load into
any build of the fork (spike 25's note).

- **It is in layout TextBoxes, in the heap.** A sweep for every `nw::lyt::TextBox` on the options
  screen found 37, from `0x0822a614` to `0x082adcec`: TEXT SPEED and SLOW and every other name and
  value, OPTIONS, CONFIRM, QUIT, "Would you like to make these changes?", the list window's Yes and
  No, and the description window's two line objects. None of it is a StrBuf or a message window's
  message, which is why spikes 23 and 27 found nothing. The linear heap held no TextBoxes at all.
- Read like a line object: +0xd4 the text, +0xf8 capacity, +0xfa length.
- **Code sets a tag processor; the designer's placeholders have none.** Every string the screen
  wrote has its static tag processor, `0x00630d80`, at +0xf4. The resident placeholders
  (`１２３４５６７８`, `１２３４５６７８９０１２３４５６７８`, fifteen rows of dashes) have 0.
- Values change in place: SLOW became FAST, DEFAULT became NO L/R BUTTON, in the same TextBox.
- Text that is not on screen is resident too: CHARACTERS and Hiragana, a Japanese-only option, and
  the confirmation question twenty seconds before it was asked.

## Spike 29 — which of them are on screen, inside one layout

(`spikes/24_layout_visible.py`):

    pane  +0x0c -> parent, 0 at the root     +0xb4 u8, +0xb5 u8, both 0xff here
          +0xb7 u8 flags, bit 0 visible      +0xb8 char[17] name ("4", "5g", "1v", "RootPane")

- Panes form a tree through +0x0c, ending at a pane named `RootPane` whose parent is 0.
- **Bit 0 of +0xb7 is visible, and a pane is shown only if it and every ancestor are.** CHARACTERS'
  own flag is set and its parent `1g`'s is clear; the picture has no CHARACTERS row. The
  confirmation question's parent `1z` was clear until the question appeared 23.65 s in, then set,
  and the description window's pane `7` went clear as the confirmation covered it. The window's
  second line object turns its own flag on and off as descriptions need one line or two.
- **Flags are not enough across layouts.** The list window's Yes and No had every flag set, root
  included, through the 23 s their prompt was not on screen.
- A screen closes by destroying its panes: at 33.7 s every options TextBox lost its vtable at once.
- The bag's census found its pocket name, "Nothing", the party sidebar's "L" and "5", and stat
  labels (MAX HP, ATTACK) under a hidden parent.

## Spike 30 — what says a layout is drawn

(`spikes/25_layout_drawn.py`, `spikes/26_layout_works.py`):

    Layout  +0x10 -> root pane    +0x18 f32 width: 320 the bottom screen, 400 the top
    work    +0x04 u8 alive        +0x07 u8 drawn    +0x10 -> Layout    +0x14 nw::lyt::DrawInfo

- `nw::lyt::Layout` holds nothing that says it is drawn, and Game Freak's wrapper has no RTTI: no
  class name in the executable contains `LytWk`. It was found by what points at a Layout. The only
  heap pointers to the options screen's layouts were at +0x10 of 0xf4-byte works, 0x104 apart,
  listed by an owner at `0x0821bb30` that holds a count, 8, and the eight work addresses.
- **Byte +0x07 of a work is drawn.** The options and description layouts read 1 throughout. The
  list window's layout read 0 until the confirmation appeared at 23.7 s and 1 after, as did a
  top-screen layout that appeared with it. At 33.5 s +0x04 went to 0 on every work, and at 33.7 s
  the works were freed.
- **The embedded DrawInfo is the anchor.** A sweep for its vtable finds every work, and the Layout
  pointer four bytes before it is the validator. Its matrices fill in the first time it is drawn.
- Untested: a layout hidden without being freed, and whether +0x07 goes back to 0 then.

## Spike 31 — a prototype layout reader, checked against pictures

(`spikes/27_layout_reader.py`, slot 0, 150 s at real speed). A TextBox counts as shown when its tag
processor is set, every pane to the root is visible, and the root's layout has a work reading alive
and drawn.

- **It read what the pictures show:** the options screen, every value change (SLOW, NORMAL, FAST;
  OFF, ON; SET, SWITCH), the description box hiding and "Would you like to make these changes?"
  with Yes and No appearing at 23.62 s; the X menu's POKÉMON, BAG, wee, SAVE and OPTIONS, each in
  two panes; "Torchic" and its gender sign on the party screen; "TMs & HMs" and "Nothing" in the
  bag. So **the in-game menu's labels are layout text** and spike 3's question about them is
  answered. Which item the cursor is on is not.
- **Misses.** The X menu read as shown while the picture still had it sliding in, so opening
  animations run ahead of the drawing. The party screen showed "Lv. 5" and "19 / 19", and neither
  was read. The bag read an "L" and a "/" the picture does not obviously hold, and not the
  sidebar's "5" that spike 29's census found. Each screen was up for one to two seconds.
- The description window's line objects are layout TextBoxes too, typed a character at a time under
  the screen's tag processor. A layout reader leaves message windows' line objects to the dialogue
  reader, or everything typed is heard twice.

## Spike 32 — screen text in the runtime

2026-09-14 (`src/pa3ds/games/common/screen.py`, run by `spikes/verify_runtime.py` over movie3 slots 0
and 4 at real speed, silent).

- **Options:** "TEXT SPEED, SLOW, BATTLE EFFECTS, ON, ..., OPTIONS, CONFIRM, QUIT" as one line when
  reading started, every value change on its own ("OFF", "L=A", "FAST"), and "Would you like to
  make these changes?" before the list reader's "Yes", with Yes and No heard only from the list.
  OPTIONS comes after the rows, because panes are said in the order the layout built them.
- **X menu:** "POKÉMON, BAG, wee, SAVE, OPTIONS" each time it came back, its doubled panes said once.
- **Party screen:** "Torchic, ♂, L, 5, 19, /, 19". The level and HP were read this time. "Lv." is a
  single "L" in a picture font, and each number is its own pane. The ♂ is U+E08E, which a screen
  reader says nothing for; speech now says "male".
- **Bag:** "TMs & HMs, L, 5, L, /, Nothing". The second "L" and the "/" are still not placed.
- **Starter choice:** "Choose a Pokémon.", "The Fire-type Pokémon Torchic", each ball's description
  as the cursor moved between them, then "Choose this Pokémon?" before YES. That is the question
  spike 23 could not find, and the one live play found silent. "You chose Torchic!" followed.
- **The Poochyena battle gave no screen text at all.** Its messages came through the dialogue reader
  as before, but the command menu and the move list said nothing. Where the battle's layouts live
  is not known; its message window is built past the heap's first measurement (spike 19), and its
  layouts may be too, or outside the heap.

## Spike 33 — where each piece of screen text is

2026-09-14 (`spikes/29_pane_geometry.py`, movie3 slot 1, the party menu, then the summary screen).
The user asked for "Torchic ♂ L5 19/19" rather than a comma between every TextBox.

    pane     +0x48 f32 width, +0x4c f32 height    +0x80 3x4 global matrix: x at +0x8c, y at +0x9c
             +0xb6 u8 anchor, bits 2-3: 0 left edge, 1 centre (0x10 and 0x14 seen)
    TextBox  +0xd8, +0xdc colours    +0xe0 -> font    +0xe4 f32 glyph width, +0xe8 f32 glyph height
             +0xfc u8 text position: % 3 is 0 left, 1 centred, 2 right; / 3 is the row

- **Boxes of pieces meant as one touch exactly.** The party slot's "L" box ends at -96 and its "5"
  box starts at -96; "19", "/" and "19" meet at -100 and -90. The summary's "Lv." (left-anchored,
  0x10) ends at -115 where its "5" begins.
- **Touching boxes are not enough.** A move's "PP" box touches its "35" box too, and the picture of
  the move list reads "PP 35/35". Where the words sit inside the box separates them: "PP" is
  centred in 35 units and "35" in 20, leaving about 15 units between the words, against 6 to 8 for
  "L5" and "19/19".
- Glyph widths are not known, so the words' width is estimated at half a glyph width per character.
  `screen.join` joins pieces on one row whose estimated words come within 0.8 glyph widths. The
  pictures of this run failed (below), so the estimate is checked against spike 31's party picture
  only, and "PP 35/35" against the move numbers alone.
- The summary's pages share one screen: stats, moves and contest stats were all read as shown at
  once, from different layouts and parent panes. Whether all three are really drawn together has
  not been checked against a picture.
- **Every screenshot failed in this run and most of the next**, with "no visible window with a
  client area" for the fork's own process, where every earlier run had pictures. The rig now lists
  the process's windows when that happens.

**Repeated dialogue, reversed again** (the user, 2026-09-16). Typing that starts over is the game
saying the message again, whether or not the box left the screen in between. The 2026-09-14 rule
that it counts only after the box goes off screen was added for the options screen, which retypes
the text speed description as a demonstration in a box that never leaves. It cost more than it
saved: reading a signpost twice went silent about half the time, because the box closes and reopens
inside a 30 Hz poll and is never caught off screen. The user's reasoning: **silence is the worse
fault**, because with nothing said there is no telling pa3ds having hung from the NPC having
finished. Only the text speed option is known to retype itself, and it is tolerable now that
nothing interrupts by default — the description follows the value rather than cutting it off.

## Spike 35 — the summary screen reads pages that are not on screen

2026-09-14 (`spikes/28_button_cursor.py` and spike 33's dump, movie3 slots 10 and 1).

- **Screen text said the summary's stats, moves and contest pages were all shown** when the reader
  started 0.9 s after slot 10, and the picture 0.6 s later shows the memo page (Nature, Date Met,
  Where) and Torchic's panel only. Visible bits, drawn works and positions inside the screen do not
  separate them.
- **Some of it is alpha.** A pane's +0xb5 is its global alpha. The move detail labels (Category,
  Power, Accuracy) and the contest labels (Effect, Appeal, Jamming) read 0 there with every visible
  bit set. `Layouts` now counts a pane with a global alpha of 0 as hidden.
- **The stats page is not.** HP, Attack and the rest read alpha 255, in their own top-screen layout,
  at positions inside the screen, while the memo page is what the picture holds. How the summary
  keeps a page drawn but out of sight is not known: covered by the memo page's box, drawn to a
  texture, or a layout that is drawn but whose work reads the same either way. Checking needs
  positions and pictures at the moment the page turns.
- Button menus: a byte diff of all 40-odd `app::tool::ButtonManager`s and their buttons across the
  X menu was noise, because freed and reused memory changes as much as a cursor. Their constructors
  give the shape to narrow with (manager +0x10 the button array, +0x18 the count; a 0x60-byte button
  keeps two values its builder passed at +0x04 and +0x08), which `spikes/32_xmenu_cursor.py` uses.

## Spike 37 — button managers are touch buttons

2026-09-14 (`spikes/32_xmenu_cursor.py`, slot 10, and the call sites of the button constructor at
`0x0034cb3c`).

    ButtonManager  +0x10 -> button array   +0x18 u32 count   +0x20 s32 an ID (0x23, 0x25, 0xad, -1)
    Button (0x60)  +0x04 u32 ID   +0x08 -> its builder's table entry   +0x0c..+0x28 small indices
                   +0x2c u32 flags   +0x30 -> listener (app::status::Touchbar, app::bag::TouchBar)

- **No button points at a pane.** Every caller loads +0x04 from a table of 28-byte entries (an ID
  followed by a rectangle of floats) and passes a hit area on the stack, so a button is a touch
  area. The indices at +0x0c to +0x28 (0x22, 0x30, 0x3e, 0x4c, 0x5a) step evenly and are most likely
  animation numbers in the button's layout, played on touch and release.
- So no manager could be matched to the X menu's labels, and the D-pad cursor is most likely kept by
  the menu's own code. The next attempt watches the menu layout's panes instead
  (`spikes/33_menu_panes.py`): the red corners around the chosen item are panes, and whatever moves
  with them names the item.

## Spike 38 — the chosen X menu button pulses

2026-09-14 (`spikes/33_menu_panes.py`, slot 10, the 111 panes under the X menu's root read every
0.1 s, rounded to two places).

- The X menu's buttons are panes at global (-80, 60), (80, 60), (-80, 0), (80, 0) and (-80, -60),
  each with two label TextBoxes a pixel apart (the doubled labels of spike 31) near its left edge.
- **One button pane at a time changes its vertical scale, 1.0 against 0.99, and it is the chosen
  one.** BAG's ("2n") at 25.55 s, when the picture has the red corners on BAG; POKÉMON's ("27") at
  27.65 s, and the replay opened the party two seconds later; wee's ("2v") at 30.90 s, before the
  trainer card. Nothing else among the 111 panes changed while the menu was steady.
- When the menu closed, every pane's global matrix read 0 for a poll and came back as it reopened.
- `savedata::FieldMenu` (`0x08c81460`) did not change in its first 0x100 bytes while the cursor
  moved, and no `field::EventSubScreen::EventFieldMenu` was found in the heap.
- Rounding hid the pulse most of the time; whether the chosen button's scale is ever exactly 1.0 is
  what `spikes/34_menu_pulse.py` measures at full precision.
- **Measured at full precision it is not a pulse but a one-off dip** (`spikes/34_menu_pulse.py`,
  every 0.05 s, ranges per half second). BAG's pane dipped to 0.985 at 24.6 s; POKÉMON's bounced
  (x 0.9 to 1.1, y 0.8 to 1.0) at 26.7 s, a press, a second before the party opened; POKÉMON's
  dipped again as the menu came back at 29.1 s; wee's dipped and bounced at 30.2 s, before the
  trainer card. At rest every button reads exactly 1.0. So a dip marks the moment the cursor lands
  on a button, and nothing in the menu's own layout says where the cursor rests. The red corners
  must be another layout's panes (`spikes/35_cursor_layout.py`).

## Spike 39 — the X menu cursor is a window in a layout of its own

2026-09-14 (`spikes/35_cursor_layout.py`, slot 10, every layout pane's global position compared
sweep to sweep, one second apart).

- **An `nw::lyt::Window` pane named "0", alone under root `0x086ca000`, jumps to the chosen button's
  position.** (80, 60), BAG's, at 7 s, two seconds before the bag opened; (-80, 60), POKÉMON's, at
  26 s, before the party; (-80, 0), wee's, at 31 s, before the trainer card. Those are the button
  panes of spike 38, exactly.
- Nothing else in the heap moved with the cursor. The other movers were screens sliding in: the
  bag's party sidebar, a bag item row, the trainer card's rows.
- So the item under the cursor is whatever shown text sits inside that window's rectangle. The same
  red corners appear on the party screen's chosen slot and on list windows, so this may be the
  shared cursor for app menus; `spikes/36_cursor_rect.py` checks that across the X menu, bag and
  party, and what tells the cursor layout apart from other small layouts.

## Spike 40 — what a menu cursor layout looks like, and which screen a layout is on

2026-09-14 (`spikes/36_cursor_rect.py`, slot 10, every shown layout of at most 12 panes each
second).

- **Cursor layouts have one shape:** a 400x240 `RootPane` whose only child is an `nw::lyt::Window`
  named "0". Six were in the heap: 180x40, 40x40 and 180x40 ones hidden under the summary screen,
  the X menu's 150x40, and the bag's 150x40 at (77, 90), shown with the bag. No other small layout
  has that shape; the rest hold pictures (a 12x12 icon, a 400x240 dimmer).
- **A layout's width does not say which screen it is on.** Both shown cursors are 400 wide, drawn
  on the bottom screen, so their rectangles were compared with the top screen's text and held the
  wrong words or none. The layout work says it instead: +0x28 in its DrawInfo's projection read
  `0xbbcccccd` for every layout spike 26 knew was drawn below — options, its confirmation list,
  and two more — and `0xbba3d70a` for the description box on top. It is 0 until the layout is
  first drawn. `Layouts` now takes the screen from it, and from the width only before that.
  (Those words were first decoded as -0.025 and -0.02, taken for -8 over the screen's width, and
  every layout then read as the top screen; they are -0.00625 and -0.005, -2 over the width. See
  spike 41.)
- **Rerun with that, the cursor rule held on three menus, and the screen did not.** The X menu's
  cursor held POKÉMON, then BAG, SAVE and wee, each label twice (spike 38's doubled panes). The save
  prompt's YES/NO list is drawn with the same kind of cursor, a 90x40 window, holding "YES". The
  bag's held "Potion" and also "MEDICINE", its pocket name, because every layout on these screens
  still read as the top screen: +0x28 did not separate them there. `spikes/37_work_screens.py` dumps
  the works of screens the pictures place.
- Every cursor layout's root holds a child count of 1 at +0x10.

## Spike 41 — the projection, decoded right

2026-09-14 (`spikes/37_work_screens.py`, slot 10: the summary screen, the X menu and the bag, every
layout work dumped as floats beside the words its shown TextBoxes hold).

- **Work +0x28 is -2 over the width of the screen the layout is drawn to.** -0.005 for the
  summary's stats, memo and name layouts and the bag's pocket name, all on the top screen in the
  pictures; -0.00625 for the summary's Pokémon panel and moves, the X menu, its cursor, the bag's
  item list and its cursor, all below. +0x1c is 2/240, 0.008333, for both. The misread in spike 40
  was the decoding, not the field.
- Six works at the X menu point at Layouts in the linear heap, whose roots the heap sweep cannot
  read; `Layouts` reads them through the RPC, so they are placed all the same.

## Spike 42 — menus heard under the cursor, in the runtime

2026-09-14 (`src/pa3ds/games/common/cursor.py`, run by `spikes/verify_runtime.py` over movie3 slot 10
for 150 s, silent).

- **X menu:** "cursor opened: POKÉMON", "moved: BAG", and "opened: BAG" when it came back, with no
  list of labels at any opening. Later "moved: SAVE" and back to BAG.
- **Bag:** "cursor opened: Potion", then the top screen's "MEDICINE Potion ×1 A spray-type medicine
  for treating wounds..." queued after it. The bag list's own pieces, including spike 32's stray
  "L" and "/", were left unsaid as the list's opening. "Potion selected." and the list reader's
  "USE" followed, then "Torchic 5 19 Use on which Pokémon?".
- The replay left the recording after the bag (a gap of 110 s with nothing on screen changing), so
  the party screen, trainer card and options were not reached in this run. The summary screen at
  the start still read every page as shown (spike 35).

## Spike 43 — the user's first live test, and the fixes

2026-09-14 (movie3 slot 1 for 90 s, silent). Live, the user heard the move list as "Growl PP / 40"
cut off by the move's description, a lone "40" or "35" now and then, and the whole X menu again on
backing out of the bag.

- The cursor's pieces were in build order, which puts a move's "/" before both PP numbers, and a
  rule meant for the X menu's doubled labels dropped the second "40" as a repeat. Pieces under a
  cursor are now read in place, and only the same words in the same spot count as a repeat.
- The description interrupted the item; nothing interrupts by default now. A changed value is said
  with its row. A layout just come on screen waits 0.8 s for a cursor.
- **Replayed:** "Do what with Torchic?" then "cursor opened: SUMMARY"; the party screen's slot as
  one item, "L5 Torchic ♂ 19/19" (the level sits above the name); the X menu, bag and pocket names
  as before; the trainer card, and its clock as "screen changed: Play time: 0:14"; and the options
  screen entirely under its cursor, "TEXT SPEED NORMAL", "TEXT SPEED FAST" as the value changed,
  "BATTLE EFFECTS ON", with no listing and no repeats.
- Still wrong: the options screen says an option's description just before its row, because the
  dialogue reader polls every frame and screen text every 0.2 s. The summary screen still reads
  every page. The move list was not in this replay.
- **Fixed after, replayed from slot 0:** written text now waits for the next screen text poll, so
  "BATTLE STYLE SET" comes before "You can choose whether to be asked about switching Pokémon...".
  The same run caught the options confirmation's Yes/No list drawing the same kind of cursor, whose
  rectangle held the option values behind the list and read "SLOW ON" and "ON SET" as the list's
  cursor moved. A cursor over any list or choice box label now belongs to that list and says nothing.

## Spike 44 — the summary's pages, watched for a page turn

2026-09-14 (`spikes/38_summary_pages.py`, slot 10, the root and every child of the root of each
layout holding shown text, every 0.25 s).

- **Nothing separated the pages.** For the 2.7 s the summary was up, the stats page's layout, the
  memo page's and the name panel's all had their roots visible at alpha 255, scale 1 and position
  0, and their children at the same local and global positions throughout. The picture holds the
  memo page's box, over the middle of the top screen, where the stats page would be.
- At 2.74 s the replay closed the summary and every watched pane read zeros and garbage: freed.
- So the stats page may be drawn and covered by the memo box drawn after it. Nothing in a pane
  says it is covered; telling that needs the draw order of layouts and the extent of what is opaque
  in each. Left open: the summary reads every page. A replay that turns pages while watching is
  what would settle it, and movie3 does not turn them.

## Spike 47 — the user's second live test

2026-09-14. No replay reproduced either fault, so the causes below are read from the code against
what was heard.

- **Backing out of any menu read "POKÉMON BAG wee SAVE OPTIONS", sometimes with "Littleroot Town",
  sometimes only that, sometimes nothing.** Two causes. A menu counted as one only while its cursor
  was up, and a closing menu's cursor can hide a frame before its labels do. And a screen coming
  back — the X menu when the bag closes, the field's AreaNav when the X menu closes — counted as new
  words, since its TextBoxes had been off screen. Which frame the reader caught decided what was
  heard. The 0.8 s wait for a cursor hid part of the first cause and made speech lag.
- **Fixes.** A layout a cursor has been over stays a menu for as long as the layout lives, and
  the wait is gone. A TextBox whose layout stopped being drawn while the TextBox stayed visible,
  alive, and comes back with the same words, is a screen returning and is not said again; a layout
  freed and built again, or words that changed while covered, still are, and so is a pane its
  screen hides and shows itself, such as a question.
- **Scratch's move power read "Power 40 Sp. Def 11"**, but only after arrowing away and back: a
  changed value took every piece on its row in its layout, and the summary's stats page shares the
  move panel's layout and rows (spike 44's page problem). A changed value now takes its row from the
  pieces that hang from its own parent pane, and the whole row only when it is alone in its group.
- The move panel's "Category" is followed by an icon, which pa3ds cannot read.
- **Runs now log to `<pa3ds>\user\pa3ds.log`** (ADR-0032; the runs before kept as `pa3ds.1.log` to
  `pa3ds.9.log`), and what
  is said of screens names its layout, so a live session can be read afterwards when no replay
  reproduces it.

## Spike 48 — menus.ctm, the user's third live test replayed

2026-09-14 (`spikes/verify_movie.py` from boot, and `spikes/39_menu_rebuild.py` from its states).
The user recorded the party screen, the summary with its page turns and move list, the bag's
pockets, the trainer card, the options screen and a save, with deterministic reads on their own
save. Replayed with the runtime, it said what their live log said, about 3.5 s earlier.

- **"POKÉMON BAG wee SAVE OPTIONS" came from labels parked off the screen.** A closed X menu keeps
  its labels visible, drawn, alive and at alpha 180, 189 units below the middle of the bottom
  screen, whose edge is 120 below it; opening the menu moves them to 61 in one poll, with its
  cursor. The summary, the bag and the trainer card each free the X menu, and closing them builds it
  again, closed: its labels came on screen as far as the reader knew, with no cursor to make them a
  menu's. Words wholly outside their screen are now not on screen. Remembering menus across
  rebuilds, planned first, was not needed.
- **Move details are said whole.** Arrowing between moves read "Category Power — Accuracy 100"
  and "Category Power 40 Accuracy 100" every time, where it had read "Power —" and "Power 40".
- **"Would you like to make these changes?" is said again**, before Yes, all three times.
- **Two faults of the details rule, found in the replay and fixed.** The bag's cursor comes back
  with the pocket name when a pocket with items is chosen, and "MEDICINE" was read with the item
  panel beside it; a cursor coming on screen now brings no details, only a move does. The panel then
  held the Potion's words as they came on screen, which were said again after it; a changed value's
  row or panel is now what was already on screen. And the list's "Potion" settled a poll after its
  cursor said it, and was said again; whatever a cursor is over is left to it.
- **Descriptions out of step with fast moves.** Moving from BATTLE EFFECTS to BATTLE STYLE within
  one screen poll read both descriptions after BATTLE STYLE. Saying only the last text held made it
  worse: BATTLE STYLE came with BATTLE EFFECTS' description, because BATTLE STYLE's own was written
  just after the poll, and a move on dropped it. A written text now waits one whole poll, only the
  last one written over a window is said, and a message page drops what was written over its window
  before it. Replayed, every description followed its own option through three passes of the
  options; on the fastest pass BUTTON MODE came once with BATTLE STYLE's, when a sweep stretched
  the poll to 0.8 s.
- **Final replay with every fix:** no X menu labels anywhere; the summary read whole on coming back
  from the move list; the trainer card, "Would you like to make these changes?" three times, and
  the save prompt's panel ("Littleroot Town Gym Badges: 0 Time: 0:20 Last saved on …"), which the
  rolling sweep had not found in time in earlier runs.
- **Returning from the move list read part of the summary**, "OT wee ID No. … Blaze …", where
  opening it read the whole page. The pictures show why: the move list keeps the stats and the
  name panel on screen and swaps only the panel below them. Words coming back to a layout now bring
  what stayed in it.
- **The summary's pages are hidden by being drawn over** (`spikes/40_summary_text_colours.py`,
  `41_summary_works.py`, slot 9, pictures). The summary opens on its blue page. The pink contest
  page and the yellow memo page are other layouts, drawn over it with opaque backgrounds, and the
  blue page's layouts stay alive, drawn and visible with opaque text underneath; turning back to
  the blue page draws it over them. Across four page turns nothing changed in any TextBox, its
  text colours or material, any work's first 0x80 bytes or any root pane's first 0x100. Work +0x0a
  looks like a draw priority (the X menu 0x0a, its cursor 0x0b, the field's banners 1 to 3) and is 1
  for every summary layout. Every work of one screen starts with the same pointer, probably its
  layout manager, which is where the page order must be. So turning back to the blue page reads
  only "Check Moves", and the first read of a summary already on a later page reads every page.
  The move list's cursor opened as "Scratch PP 35/35 Scratch" in two replays of four, then moved to
  "Scratch PP 35/35" a poll later: the contest page's list, under the blue one, has a "Scratch" in
  the same slot.
- **The object a screen's works share is its drawing setup, not a list of pages**
  (`spikes/43_layout_managers.py`, slot 9). Its first 0x400 bytes hold the top screen's extent
  (-200 to 200 by -120 to 120, near 0.1, far 800, projection -0.005) twice, offset -290 and 290
  once each, the bottom screen's (-160 to 160, projection -0.00625), and pointers at +0x3b8 on.
  Not a word of it changed through the same page turns. The page order is somewhere else: a draw
  list reached from the screen's own object, or the order works are drawn in. Open.
- The bag says "Nothing" when an empty pocket follows MEDICINE, and not between two empty pockets,
  whose "Nothing" does not change.
- The trainer card read "Play time: 0: … Adventure started on: 9/13/2026", then "18", in one replay
  and "Play time: 0:18" in the others. The game writes the minutes a poll after the rest of the
  card; the card does not slide in, and nothing on it is off screen (`spikes/42_trace_words.py`).

## Spike 49 — AreaNav, late

2026-09-14 (`spikes/44_areanav.py`, menus.ctm from boot). The user's fourth live test: "Littleroot
Town" was sometimes not said until they opened another screen, up to 4 s after the field was back,
and after loading their save they had opened the X menu before it was said. They asked for it first
thing after Continue, and whenever the area changes.

- **AreaNav is on screen with its words at 18.9 s**, as the save loads. The runtime said it at
  30.5 s. Its TextBox and panes are in the linear heap, swept a slice at a time, a pass in 3.5 s
  once measured, and measured every 10 s; its layout work (0x85fc8a8) is in the heap, swept whole in
  0.03 s at that point.
- **A layout's tree leads to its TextBoxes.** Every pane keeps its children in a list: the first
  child's node at +0x14 (the list itself when empty), each node 4 bytes into its child and pointing
  at the next, the last back at the list. Walking AreaNav's 38 panes took 5 ms and found "Littleroot
  Town"; the biggest layout at load, 410 panes, took 60 ms. `Layouts` now walks each layout new
  since the last sweep, and every layout every fifth sweep, and the screen text reader reads every
  TextBox the walks found beside the sweeps'.
- **The field's works come back where they were.** Closing a screen opened from the X menu builds
  the field's layouts again at the same works (spikes 41 and 49), so works that stop being works are
  kept, up to 64, and looked at again every screen poll: a layout built again is walked and read at
  the next poll, not the next sweep.
- **Move details, gathered.** A move's description came a poll after its panel once, and was read
  after it; Ember to Scratch changes only the description, and the panel went unsaid. What changes
  after a move is now gathered until nothing in it is still being written (at least two polls, at
  most eight), then every panel that cursor's moves have changed is read, in the order the screen
  built them.
- **Replayed** (menus.ctm from boot): "Littleroot Town" at 20.5 s, the first thing said after the
  save loaded, where it had come at 30.5 s; back to the field after the summary, the bag, the
  trainer card and the options 1.3 to 2 s after the X menu came back, where it had taken 4 to 5 s;
  every move read its description and then "Category Power … Accuracy …". One fault came with
  revival: the save prompt's list layout, built at a work an earlier layout had freed, was taken
  back at once and its "YES NO" read before the sweep found the list that owns them. Only a work
  that comes back with the root it had is revived now.
- **The area name follows the player** (movie3 slot 4, the rig's own saves): "Route 101" as the
  battle returned to the field, and "Littleroot Town" as the player walked into town, said as a
  change of AreaNav's words.
- **Said twice, live:** walking between areas said the name once from AreaNav and again from the
  top screen's location sign, a separate layout with the same words a poll later. Words another
  layout said within five polls are no longer said again (confirmed live by the user).

## Spike 68 — where words and cursors are

2026-09-17 (`spikes/70_screen_geometry.py`, battles2.ctm slots 1 and 9 and opening.ctm from its
first frame, every piece on screen and every cursor dumped at chosen moments). Three faults from
the user's playthrough, and a fourth that was never built:

- **"Littleroot Town OPTIONS".** With the Pokédex in the X menu, OPTIONS moves to the bottom right,
  over AreaNav's location name. The cursor ran from x 1 to 159, OPTIONS from 46 to 90, and
  "Littleroot Town" from -24 to 74: its middle under the cursor, its start well outside. It was
  intermittent because AreaNav's words leave the screen while the options screen is up and come
  back a few seconds after it closes. Worse than the words: AreaNav's layout became a menu, and a
  menu's changed words are left to its cursor, so a location name changing after that could go
  unsaid. **Built:** when a layout has words wholly under a cursor (half a glyph of slack), words
  of other layouts only partly under it are left out.
- **"Spanish" then "Japanese".** The language list moves its rows rather than writing new words into
  them: five rows 30 units apart fit the top screen, and moving the cursor slides every row, so
  "Japanese" moved up onto the screen from below it and was taken for words new to the menu.
  **Built:** words that move onto their screen with the words they had just outside it are marked
  as scrolled in, and in a menu they are left to the cursor, as rewritten rows already were.
  Replayed, the language screen says one language a move. One race is left: moving to "Begin game"
  said "Spanish French Begin game" as the cursor slid past.
- **The level up, read in pieces.** Its panel is one group per stat, label then value, revealed a
  row at a time over half a second with each value written a poll after its label, and then the
  gains are written over with the totals when the player presses A. It was said as "HP", "+ 2
  ATTACK + 0 DEFENSE SP. ATK SP. DEF", "+ 1 + 1 SPEED + 1". **Built:** what comes on screen in a
  layout's first second waits while more of its words are still being written, up to a second,
  and is said at once, as it came, the moment any of it changes or leaves; and a value arriving in
  a group beside one label at least two glyphs away is said with that label. Replayed: Emrys's
  level up is "HP + 2 ATTACK + 0 DEFENSE + 0 SP. ATK + 1 SP. DEF + 1 SPEED + 1", then each total
  with its label; Blaze's, where the movie pressed A within a second, is "HP 23 ATTACK 15 DEFENSE
  11 SP. ATK 13 SP. DEF 12 SPEED 12". Three first attempts each broke something the replays caught:
  dropping words that changed while waiting lost Blaze's gains entirely; taking every word beside
  an arriving one for its label said the typed name again with each letter, "R a i d"; and holding
  words in a screen already up said two quickly typed letters as "w e e".
- **The name entry's keyboard has a cursor of a third kind.** Every key is a 20 by 30
  `nw::lyt::Picture` holding its letter's TextBox and an `nw::lyt::Bounding` touch area, all in one
  layout, and the cursor is a 20 by 20 picture the game moves from one key's touch area to the
  next: under "!" as the keyboard opened, then R, then Q. The typed name beside it keeps pictures in
  a touch area too, under a plain pane. **Built:** a picture shown in a touch area whose parent is a
  picture or a window is a key cursor while it is the only one in its layout, or once it has moved
  between keys, and its item is that key's words.
- **The key that confirms the name has no words** (the user, 2026-09-18), as Delete and Space have.
  Its texture names it, as a battle button's does, but its layout's file was never found: a file is
  matched to a layout by its panes' names in the order the layout walks them, and moving the cursor
  from key to key changes that order. **Built:** a file whose pane names are the same set is taken
  when it is the only one, and a leading "icon" is left unsaid as a trailing "button" is, so the
  key reads "return" (`icon_return.bclim`).

## Spike 71 — the opening's choice of boy or girl

2026-09-19 (`spikes/75_gender_screen.py`, opening.ctm from its first frame, every layout drawn
dumped a second at a time from 100 s to 120 s with a screenshot beside each dump). Birch asks "Are
you a boy? Or are you a girl?", the player arrows between two portraits for about 13 s, and the
runtime said nothing at all.

- **The screen is one layout, and it holds no words.** `0x826504c`, on the top screen. Under the
  holder `k` is a plain pane for each choice, `2k` at (-80, 0) and `2n` at (80, 0), both 30 by 40,
  and each holds a white backing (`1pix_wht.bclim`), two glows (`intro_bg_eff01.bclim`), the
  portrait (`intro_male.bclim`, `intro_famale.bclim`) and an `nw::lyt::Window` of its own at its
  own place. There is no TextBox anywhere in the layout, so the screen text reader had nothing to
  say and the cursor readers had no cursor to find.
- **The cursor is a window beside the choices, not one of theirs.** `2h`, a window under `k`, 94
  by 96 at scale 1.8: at (-80, 0) on the boy, at (-16, 0) one poll later, at (80, 0) on the girl a
  second after that, and back. It is never resized, so the main menu's rule — a frame that comes
  to rest somewhere new at a new size (spike 72) — is not what tells it apart here. Where it comes
  to rest is: on a pane beside it, not on one of its own.
- **Built:** a window holding no panes of its own, beside two or more plain panes that are one
  size and each in a place of its own, is a cursor over the pane it sits exactly on, from the poll
  it is first seen on rather than from its first move. The frames round each choice stand among
  their own choice's pictures with no panes beside them, and a message window's box holds the
  panes of the screen it frames, so neither is taken for one. The choice is said by the screen
  text under it, and, where a choice has none, by the words of the textures the other choices do
  not have: both portraits sit on the same backing under the same glow, so the first picture's
  texture, which names a battle's buttons, would say "1pix wht" for both. What is left is "male"
  and "famale" — the texture's own spelling, which a screen reader reads out as written. So
  `famale` joins `pokemon` in the spelling table `words` applies (`oras/layout_file.py`), and the
  girl is said "female" (the user, 2026-09-21). Both entries are the word the picture is of, spelt
  as the game spells it elsewhere; neither is a name pa3ds invented.
- **Replayed through the runtime** (`verify_movie.py`, opening.ctm to 125 s, no out-of-sync
  lines): "male" as the screen opens at 108.1 s, then "female", "male", "female", "male",
  "female", "male" as the player arrows across and back, each within a fifth of a second of the
  press, and "Will you give me your name, too?" after. Before the change those 13 seconds were
  silent.
- **Before and after over signs, battles, menus and battles2** (`regress.py`, all four with no
  out-of-sync lines, this change and spike 70's together): signs is identical line for line, and
  the other three differ only where they race by design — a cursor item "opened" in one run and
  "moved" in the next, a battle's HP line landing either side of "What will Emrys do?", a save
  prompt's YES caught in one run and not the other. No screen gained a cursor it did not have.
  One run of signs said *nothing at all*, with no out-of-sync line, and read a code span of
  `0x00100000-0x00630000` where the others read `0x00100000-0x00698000`; the same replay a minute
  later was identical to the before run, line for line. The short span was a symptom, not the
  cause: every class pa3ds names by RTTI has its vtable between `0x005dbc30` and `0x005df748`
  (checked against `code.bin`), inside even that span, and opening.ctm reads `0x00630000` every
  run and speaks throughout. What the span says is which DLLs the game had mapped, so a silent
  run with a short one was somewhere else entirely — the zero-count desync spike 5 warned about.
- Timed over signs.ctm for 150 s at 60 Hz (`spikes/60_poll_timing.py`, both runs on this
  machine within the hour, against HEAD): 58.1 polls a second either way, 99th percentile gap
  31.0 ms against 31.4, the heap's probe 2.28 s against `regions`' 2.33, sweeping 10.45 s against
  10.03 — the cost of reading memory new to the heap at once — and `cursors.poll` 0.52 ms a call
  against 0.40, for one more walk of each root's panes a poll. (The 8.8 s quoted for sweeping in
  spike 69 was another day's machine; measure both sides the same afternoon or the numbers say
  nothing.)

## Spike 72 — the main menu, and what comes before it

2026-09-21 (`spikes/72_main_menu.py`, main_menu.ctm from its first frame: the logos, the opening,
the title screen, the main menu with the cursor arrowed up and down it, and CONTINUE into Route
103). The user asked for the main menu, and whether the title screen and the opening had anything
to read. Replayed at HEAD, the runtime said nothing until "Route 103" at 104 s.

- **The opening is a video, and the logos and the title screen are pictures.** Between the Game
  Freak logo (`gf_logo.bclim`, `presented_by.bclim`, 22 to 28 s) and the title screen (68 s) no
  layout shows anything. The title screen's layout holds no TextBox at all: its logo is
  `title_logo_ruby01.bclim` and its neighbours, and "PRESS START" is a picture textured
  `push_start.bclim`, which appears seven seconds after the logo and then stays. Nothing here is
  said. A rule that spoke `push_start` would have to pick it out of `start_belt`, `lineA` and
  `logo_eff03` in the same layout, and the only thing that singles it out is that it is the one a
  player waits for, which is not a rule. Left as it is, with the user to decide.
- **The main menu is ordinary screen text, and it was never swept.** Its words are TextBoxes under
  the screen's tag processor (`0x00630d80`) in a layout on the top screen: CONTINUE, the player's
  name, place, "Badges:" and its count, "Time:" and the hours, the colon and the minutes as three
  pieces, MYSTERY GIFT, LIVE COMPETITION, and POKÉMON LINK and "Pokédex:" kept off screen. Its two
  works sat at `0x081ff594` and `0x081ff698`, in the 16 KiB block the title screen had kept its
  layout resource in at `0x081fc000`. `regions` reads the first page of each block, and the moment
  the title screen freed that resource the page read blank, the block dropped out of the heap
  measurement, and with it out of every sweep, hot spans included. On the title screen the
  measurement read `0x081fc000`-`0x08328000`; on the main menu `0x08200000`-`0x08328000`.
  **Built:** a measurement only ever adds to the spans measured before, and each span reaches one
  block further either side.
- **Its cursor is a fourth kind.** The same red corners as an app menu's, but an `nw::lyt::Window`
  in the menu's own layout, directly under its root among the item plates and two hidden frames,
  336 by 140 over CONTINUE's panel and 336 by 43 over each row below. It is not alone in a layout
  of its own, so the cursor reader passed it by. Nothing about the window itself says it is a
  cursor rather than a panel's border, and the hidden frames beside it are the borders of whatever
  the menu can put up, so a rule that took a lone frame for a cursor would take those too and
  silence the screen they frame. **Built:** a window directly under a root is a cursor once it has
  moved from one resting place to another while its sibling windows stayed, as the keyboard's
  cursor is a cursor once it has moved between keys. A rest is two screen polls in one place at
  one size, so the item is heard a poll after the cursor lands, the first time only. Frames that
  move together are a list scrolling (spike 68's language screen), a frame somewhere new every
  poll is a screen sliding in or out, and both stay what they were. A window missing for one poll
  is kept, as a picture cursor is.
- **Replayed through the runtime** (`verify_movie.py`): at 88 s, as the menu opens, "CONTINUE Leos
  Route 103 Badges: 0 Time: 0:35", "MYSTERY GIFT", "LIVE COMPETITION", one panel each as any
  screen is read; then "MYSTERY GIFT" as the cursor first comes to rest on it, and every move
  after: "LIVE COMPETITION", "MYSTERY GIFT", "CONTINUE Leos Route 103 Badges: 0 Time: 0:35". The
  menu's layout is found by the 2 s sweep, as any new screen's is, so in one replay the menu was
  read as it opened and in another two seconds later, with the cursor already on MYSTERY GIFT and
  LIVE COMPETITION the first item said. A move the recording made inside one screen poll was
  caught in one replay and not the other, as before.
- **Left open: the item under the cursor as the menu opens is not said as such.** It is CONTINUE,
  and it is heard, first, with the rest of the screen; but a player who wants only the item under
  the cursor at every menu gets the whole menu here, once, until the cursor moves. Saying the
  item alone would need a way to know the frame is a cursor before it moves.
- Timed over main_menu.ctm for 112 s at 60 Hz (`spikes/60_poll_timing.py`): polls 58.4 a second,
  99th percentile gap 30.5 ms, cursors.poll 0.08 ms a call. Only main_menu.ctm is on this machine,
  so the replay set of spike 69 was not rerun.

## Spike 95 — the Pokédex list, the Poké Mart, and Wally's tutorial

2026-09-24 (`spikes/95_list_cursors.py`: a movie replayed with the runtime reading silently, and
after every screen poll each pane of the layouts whose words match `--words`, and of every small
layout, compared with the poll before; the first time a watched layout is shown, its tree and the
materials its layout file gives each window). Three recordings by the user, each with his log:
pokedex.ctm and shop.ctm from his Petalburg save, wally_again.ctm from before_wally. The
Pokédex's list was read whole, every row as it scrolled; the Mart's list likewise, and the whole
screen again each time the cursor came back from Quit; Wally's moves and bag were read whole too.
None of the three said which item was chosen.

- **The Pokédex list's cursor is a plate that never moves.** Layout `0x82ec748`, bottom screen:
  nine rows directly under the root (`f`, `o`, `v`, `12`, `19`, `1g`, `1n`, `1u`, `21`), plain
  panes at x 60 and 30 apart from y 125 down, each holding a window (`list_01_*.bclim`) with the
  number, the name and a Poké Ball for one caught. The red plate is `8f`, a 240 by 40 window with
  no panes of its own two panes below the root, drawn with `list_00_*.bclim`, fixed on the row at
  y 5. Nothing names it a cursor. **The rows move, the plate does not:** a step up or down slides
  every row 27 units and back in two polls while their words are rewritten a row on, and a page
  left or right rewrites them all seven entries on. At the end of the list the rows past the last
  entry are hidden, and in the poll a scroll starts they are shown with the words they were left
  with, under the plate.
- **Dimmed rows are unseen entries, not rows off the cursor.** At the end of the list the chosen
  row, Magnemite, was at alpha 255 and the rest at 100, which looked like the cursor; after one
  step every row was at 100, and a picture taken mid-scroll showed Torchic bright two rows from
  the plate. The Pokédex dims what the player has not seen.
- **Built: a plate.** A window shown with no panes of its own, not directly under its root, no
  taller than two rows, sitting on a row of a list — three or more plain panes under one parent,
  each holding words on screen, at one x and of one height — is a cursor over that row, the row
  nearest its middle while they slide. Its item is held until it has read the same for two polls,
  which rides out the stale words, and a plate lost for one poll is kept.
- **The Mart's cursor is named.** Layout `0x82f58f4`, top screen: its words are TextBoxes
  directly under a window, item names at x -55 and prices at x 137, 20 apart from y 90.1 down.
  The cursor is `3z`, a 280 by 20 plain pane holding two pictures textured `shop_cursor.bclim`,
  which pulses its alpha and jumps whole from row to row (90.1 to 30.1 when the player pressed up
  from Poké Ball onto Quit). Quit hides the count in the bag and the item's icon, and coming back
  from it brought "In Bag" back, which as words returning to a layout that was not a menu read the
  whole screen again: the list the user heard over and over. **Built: a named cursor.** A picture
  whose texture has `cursor` as a word of its name is a cursor from the poll it is seen on, over
  the words inside the pane holding it when that pane holds nothing else. Other layouts carry the
  word too — the main menu's frame is `cursor_lt.bclim`, the Buy/Sell box's arrow `cursor.bclim`
  — but in a window's material or pointing from beside the words, and neither is read by it.
- **A cursor that knows its list owns only the list.** The Mart's money, count and description
  share the list's layout, so left to the cursor as a menu's layout is (ADR-0013), none would be
  heard as it opens and the count never as it changes. The plate's rows and the words beside
  those the named cursor is over are the cursor's; the rest of the layout is screen text, and
  changes after a move are that item's details. It stays a menu for words coming back: "In Bag"
  returning from Quit comes alone. ADR-0013 is narrowed to say so.
- **Replayed through the runtime** (`regress.py`, both with no out-of-sync lines through the part
  studied): the Pokédex says "084 Magnemite" as the list opens, then "077 ?????", "070 ?????" a
  page at a time, "001 ?????" to "004 Torchic" a step at a time, and "029 Ralts" coming back from
  its entry — each once, where the rows had been read nine at a time. The Mart says "Poké Ball
  $ 200", "Money $ 3,440", "In Bag: 10" and the description as it opens, then "Quit", "Potion
  $ 300" and so on a move each, with "In Bag: 9" and the description as details after the item.
  Holding the Mart's item for two polls, as the plate's is, put the count and description before
  it, so it is not held.
- **Before and after over all ten replays on this machine** (`regress.py`, `fd82c7e` against the
  change). The first "after" found a fault: the name entry's key cursor is textured as a cursor
  too, the named rule read it as well as the key rule, the two disagreed about where it was, and
  the key under it was said again every poll with the keyboard read whole (+414 cursor lines on
  opening.ctm). **Built:** a picture another kind already reads — a picture cursor, a picture in
  a key's touch area — is left to it. Rerun with that: signs identical; battles 35, menus 71,
  wally 129, wally_again 28, opening 134, opening_as_may 72 and battles2 168 changed lines, every
  one of a kind the runbook lists as racing by design (bearings, "opened" against "moved", a
  keyboard letter caught a poll early, HP caught mid-slide, the level up revealed); the Pokédex
  and the Mart differ where they were meant to. battles2's first "before" desynced (178,750
  out-of-sync lines, the machine idle) and was run again. shop.ctm reads 1,395 out-of-sync lines
  on every run, from the Sell part on; the Buy part it was recorded for matches the user's log.
  The Pokémon Center's Yes/No after Wally was missed by the choice reader in three of four runs,
  before the change as after it, where the user's own session heard it.
- Timed over menus.ctm for 150 s at 60 Hz (`spikes/60_poll_timing.py`, both sides the same
  hour): `cursors.poll` 0.76 ms a call against 0.63, p90 1.42 against 1.09, for the walk of each
  layout's windows and pictures a poll; dialogue polls 55.9 a second either way, p99 gap 36.6 ms
  against 37.5.
- **Wally's tutorial has no battle cursor at all.** Wally's choices are shown by a picture
  textured `gf_logo.bclim` in a three-pane layout of its own (`0x8231c28`) that slides onto the
  button he "taps" — Tackle at (-50, 65), scaled to 2 for the tap and hidden a poll later, the
  move list gone the poll after — and later onto the bag's Poké Ball, whose button swells to 1.1
  as it is tapped, and onto USE. The move list was up for about a second. Not built: a pointer of
  an eighth kind, saying the button it taps, would need to be known as the list appears, before
  any tap, or the list is read whole first anyway. Left with the user, who asked for it last.
- **Left open.** The Pokédex list does not say whether an entry is caught, which a Poké Ball
  beside the number shows (`ball_00.bclim`). The Mart's Buy/Sell box is a frame directly under
  its root that opens in another place from one time to the next, and a replay takes it for a
  cursor that has moved and reads "Buy Sell No thanks!" whenever that lands a poll before the
  choice reader claims those words — before this change as after it.

## Spike 96 — cursors by their names, through the whole ROM; the main menu, Wally's pointer and the Pokédex entry

2026-09-24 (`spikes/96_rom_cursors.py` over the user's ROM, then `spikes/95_list_cursors.py` and
`regress.py` over pokedex2.ctm and wally_again.ctm). The user, after spike 95: the Mart's cursor is
literally `shop_cursor.bclim`, so look through the whole ROM for cursors named that way. His
pokedex2.ctm also recorded the main menu reading CONTINUE, MYSTERY GIFT and LIVE COMPETITION
together as it opened (the exception ADR-0012 kept since spike 72), and a Pokédex entry that "reads
a little off"; and he asked for Wally's taps after all.

- **What the ROM names a cursor.** Every RomFS file and every file of every GARC, LZ11
  decompressed where it is that, raw as well (a code module can start with 0x11): each `darc`
  archive found by its header and its files named from its own table, each `.bclyt` read for its
  textures, materials, and the materials each picture and window is drawn with. 379 layouts in
  98 s; 12 have a texture with `cursor` in its name:

  | Layout | Texture | Drawn by |
  |---|---|---|
  | `title_upper` (the main menu) | `cursor_lt` | window `1u`, the menu's frame |
  | `shop_lower` (the Mart) | `shop_cursor`, `shop_mark_cursor` | the list's cursor; two arrows |
  | `list_menu` (a choice box) | `cursor`, `cursor_c` | the arrow beside the choices |
  | `nameinput1_lower`, `nameinput2_lower` | `cursor` | the keyboard's key cursor |
  | `number_input_lower` | `cursor` | |
  | `waza`, `waza_contest` | `cursor1` | a picture under the root |
  | `contest_battle_lower` | `cursor` | |
  | `secret_base_upper` | `mark_cursor`, `s_b_cursor` | |
  | `pmn` | `distance_cursor` | |
  | `a/1/1/4` `lower` | `left_cursor_c`, `right_cursor_c` | |

  The X menu's cursor is `cur.bclim`, which the word does not match; the Pokédex's layouts are not
  among the 379 at all. They are in no `darc` in the RomFS, bare, LZ11 or LZ10; the two archives
  left unread are `a/0/0/8` (1 GB, 8,064 LZ11 files of `PBJ` models) and `a/0/3/9` (857 `GR`
  map files). The Pokédex has code modules of its own (`DllSangoZukan.cro`, `DllZkUtil.cro`);
  where its layouts come from is open.
- **The main menu's frame is named.** Its window `1u` is drawn with `cursor_lt.bclim`, the frame
  spike 72 could only know once it had moved. **Built:** layout files say what each window's
  content and frames are drawn with (`LayoutFile.frames`), and a window drawn with a texture named
  a cursor is a named cursor from the poll it is seen on. Unlike a named picture it knows no list:
  its layout is the menu's whole, so MYSTERY GIFT and LIVE COMPETITION are left to it as the menu
  opens. Replayed: "CONTINUE wee Petalburg City Badges: 0 Pokédex: 9 Time: 0:50" alone as the
  menu opens, then one item a move.
- **Wally's pointer.** Layout `0x8231c28` is three panes: the root, a plain pane `2`, and a 60-unit
  picture `0` textured `gf_logo.bclim` — a placeholder the game draws its hand over. For each tap
  the game puts pane `2` on the button: (0, 65) FIGHT, (-50, 65) Tackle, (-115, -72) BAG,
  (80, 70) POKÉ BALLS, (-80, 100) the Poké Ball, (35, -80) USE; the picture appears 20 units above
  it, rises, drops onto it at twice its width for the tap, and hides about a second after
  appearing. Each place lies inside the touch area (`nw::lyt::Bounding`) of the button tapped.
  **Built:** a pointer — a small picture alone in a layout but for the one plain pane that places
  it — says the button whose touch area holds that place, any layout shown on that screen, words
  or none (the command menu is all pictures), once it has held for two polls; meanwhile the
  button's layout is the pointer's, so its other buttons are not read out. Where another cursor
  claims the layout, the pointer says nothing, so a battle the player plays is untouched. The
  first replay said "LAST USED ITEM" for the tap on USE: the bag's categories were still up behind
  the USE screen. Of two touch areas, the one whose layout came on screen last is taken. Replayed:
  "fight", "Tackle PP 35/35", "fight", "Tackle PP 34/35", "bag", "POKÉ BALLS", "Poké Ball × 15",
  "USE", each before the tap, where the moves and the bag had been read whole.
- **The Pokédex entry is one panel built out of order.** Layout `0x83017f8`, bottom screen, all
  under `1y`: the species (`q`, y 70) and gender (`s`, 50) under a pane `2`; a table under a pane
  `15` built as a column of labels (TYPE 30, HEIGHT 10, WEIGHT -10) and then a column of values
  (1'04" at 10, 5.5 lbs. at -10); the times encountered (-35); and the number and name at the top
  (100) built last, on a plate of their own. Read in build order: "Chick Pokémon Female TYPE HEIGHT
  WEIGHT 1'04" 5.5 lbs. Times encountered: 1 004 Torchic". **Built:** a panel that is one column —
  no word of it beside another group's word on a row, and no two of its words beside each other
  off one line — is read top to bottom, as is its changes' order: "004 Torchic Chick Pokémon
  Female TYPE HEIGHT 1'04" WEIGHT 5.5 lbs. Times encountered: 1", and "007 Mudkip" first when
  the entry turns. Columns side by side keep their build order (spike 47). A rule that took a gap
  of four glyphs on a row for two columns was tried first and read the entry as columns: the
  value column starts far enough right. What tells a table from two columns is the line: a
  label and its value are drawn on one, two columns only come near.
- **TYPE is a picture the game retextures.** Its file gives it `type_icon_00_normal.bclim`
  whatever the Pokémon, so the type cannot be read from the layout; it would come from the
  species' own data in the ROM and the type names in the game text. Not built.
- **The habitat.** The top screen says "Torchic's Habitat" and "Petalburg City" side by side,
  over three map icons (`map_ico_00` to `02`, unread), and the bottom "Habitat Unknown". What
  "Petalburg City" names — the player's place, or the one under a map cursor — is not known yet:
  every picture of this run failed, the fork's windows reporting themselves hidden throughout.
  Left with the user.
- **Before and after over all eleven replays** (`regress.py`, `24ca8cb` against the change). The
  main menu opens on its item in every replay that starts from a save; Wally's battle says his
  taps where it read the moves and the bag whole; the Pokédex entry reads top to bottom. One
  replay of the opening found a fault: the name entry keeps a second picture textured as a cursor,
  and as the keyboard opened it said "“", the key it lay on, beside the key cursor's "!".
  **Built:** a named cursor is not looked for in a layout with a key cursor. Rerun, every other
  difference is of a kind the runbook lists as racing (a keyboard letter a poll apart, "opened"
  against "moved", the level up revealed, damage rolled differently). battles2 desynced once more
  (178,750 out-of-sync lines, on the "after" side this time) and was run again.
- Timed over menus.ctm for 150 s at 60 Hz (`spikes/60_poll_timing.py`, both sides the same
  hour, `24ca8cb` against the change): `cursors.poll` 0.85 ms a call against 0.71, p90 1.44
  against 1.18, for the pointer's walk of each layout and the windows' names; dialogue polls
  56.3 a second against 56.4, p99 gap 35.3 ms against 35.4.
- **Layout files as a way to know screens.** Each layout's file has a name in its archive
  (`shop_lower.bclyt`, `title_upper.bclyt`), and a layout matched to its file (`LayoutFiles`)
  could be known by that name. That would let a rule say which screen it is for; it would not
  make pa3ds faster, whose time goes to sweeping the heap for objects (spike 95's timing:
  `find_objects` 4.4 s and the heap probe 2.4 s of 150, against 0.8 ms a poll for every cursor).

## Spike 98 — the Poké Mart's Yes and No, and a window that holds panes

2026-09-25 (`spikes/98_mart_yes_no.py`, and `spikes/95_list_cursors.py` for the layout), on
movie5_menus_and_shops.ctm, which replays faithfully up to the Sell (spike 97).

- **Buying Poké Balls, "Is that OK?" said "Yes No" once and never the moves** (the user). The
  layout (`0x8340a14`, nine panes): its root holds one Window, framed with `menu_win_lu.bclim`,
  which holds the TextBoxes "Yes" (x 139, y 10) and "No" (x 139, y -10) and a Picture textured
  `cursor.bclim` at x 113. The picture went from y 10 to -10 and back as the player moved; nothing
  else did.
- **The box was taken for an app menu's cursor**, being a window alone under its root, so the
  item was everything inside it, and a box does not move. The corners of a menu's cursor hold no
  panes, so a window holding panes is now a box, never a cursor. The same false item said "Buy
  Sell No thanks!" over the Mart's first box, "Yes No" over both of Birch's questions in movie1,
  and was the starter choice's description box, whose words changed as the player moved from ball
  to ball.
- **The real cursor points from beside its words**, over none of them, so the named-cursor reading
  found nothing under it. Such a picture now says its layout's words on its row, to its right.
- **It is not a choice box.** At 88 s, before it opened, and at 93 and 95 s, while it was up, the
  same two objects held the input device manager, and neither reads as a box: the shop draws it
  itself. The Buy, Sell and No thanks box is a choice box, which the choice reader finds once a
  sweep reaches it, so the cursor beside its words can say it first; a choice opening on words a
  cursor said a moment ago is logged and not said again (`ScreenAnnouncer.said_by_cursor`).
- **Replayed before and after, with a control of the old code** over movies 1, 2 and 5: "Yes",
  "No", "Yes" as the cursor moved; the false items gone; the starter's descriptions the same words
  at the same moments, as screen text; the rest within the control's noise (movie2's diff 109
  lines against the control's 142). The save prompt's YES in movie1 was missed after the change
  and in the user's own run of the old code alike: the list reader's race, not this.

## Spike 112 — Pokémon Sun's menus: ORAS's layouts, another pane shape, and a cursor pane

2026-09-26 (`spikes/112_sun_menus.py`). The user recorded sun_3.ctm in their own Azahar on their Sun
save of 08:14 (the rig's `user/save_sun_3`, their whole NAND and SD card): Continue from the
title's menu, the X menu, the options and the save prompt's Yes and No, then checking things in
the bedroom, 63 s of input. It replays with no out-of-sync line. `capture` walked every heap the
heap table lists every 2 s for 100 s on Vulkan and kept every used block of 4 KB or less, read to
the byte, with a list of every block of every heap beside it; the emulator stayed up.

- **Sun's layouts are ORAS's.** A layout work is found by the `nw::lyt::DrawInfo` it embeds at
  +0x14, and reads alive at +0x04, drawn at +0x07, its Layout at +0x10 and the DrawInfo's
  projection at +0x28: -0.005 on the top screen, -0.00625 on the bottom, as in ORAS. The talk
  window's layout and the Yes/No box's, built and not shown, read not drawn. A work is a block of
  its own, 0x134 bytes; a Layout's root is at +0x10 and its width at +0x18.
- **A pane is laid out otherwise** (`sm.lyt`): the tree links where ORAS has them, then width and
  height at +0x3c and +0x40, flags at +0x44 (bit 0 visible), alpha +0x45, global alpha +0x46, the
  anchor at +0x47, the global matrix from +0x48 (x +0x54, y +0x64), the name at +0x80. A TextBox
  has its text at +0xa4, glyph width +0xb8, tag processor +0xc8, capacity and length +0xcc and
  +0xce, and text position +0xd1. **Alignment is NintendoWare's own**, the low two bits of the
  anchor and of the text position, 0 centre, 1 left, 2 right: the title's "Continue" is centred on
  x 0 (0x00, 0x04), "rewq" set from x -36 (0x01, 0x05), the play time's hours right-aligned
  (0x12, 0x06). A Pane or Bounding is 0xa4 bytes, a Picture 0xc0, a Window 0xd8, a Parts 0xb0 and
  a TextBox 0x100, each 0x20 into a block of its own.
- **Every screen in the movie is TextBoxes**, visible and written, each checked against a picture:
  the title's "Continue", the name, place and play time, "Mystery Gift" and "Live Competition";
  the X menu's SAVE and OPTIONS, the only two the story has given so far (BAG, POKÉMON, QUICK LINK
  and FESTIVAL PLAZA are written into hidden TextBoxes on empty plates); the options' five names
  and values, "Confirm" and the chosen row's help; the save prompt's date, place, play time, last
  save, and Yes and No.
- **The cursor is a pane of its own** in every one of those menus: a layout whose root holds one
  plain pane, `Cur_Main`, holding only the arrow's two pictures, moved and sized onto the chosen
  item. Each button of the menu (a Parts pane) keeps an empty `Null_Cur` of exactly that place
  and size: the title's Continue at (5, 32), 312 by 20, over its whole panel rather than its word;
  the X menu's icons at 124 by 20, empty slots included; the options' rows at 364 by 20. The save
  prompt's Yes and No keep theirs hidden and never placed, and the cursor pane lies over the
  words: at (97, 26), 118 by 20, over "No" at (131, 26).
- **The name keyboard's cursor is a window alone under a Parts pane** (sun_1.ctm, 108 to 128 s,
  captured every second): `Cursor_00` holding `Window_00`, 38 by 48 on "!" and 68 by 50 on
  "Delete", moved and sized from key to key inside the keys' own layout. The keyboard's other
  page keeps its own parked, visible, over the page in front's "!" and "1". A green key drawn
  with a picture alone has no words.
- **Built:** ORAS's `Layouts`, screen text and cursor readers take a game's **pane shape**
  (`lyt.PaneShape`, ORAS's constants gathered as `lyt.ORAS`, Sun's `sm.lyt.SHAPE`), and every
  read of a pane stops at the smallest pane's end, a TextBox's at its text position. Sun's profile
  runs them as ORAS's does, the cursor reader with **cursor panes**: the item is the button whose
  empty pane the cursor pane fits, all its words, or "empty" for a button whose words are hidden,
  or else the words inside the cursor pane; and a window alone under a Parts pane is a cursor
  whose item is its own layout's words. The layout readers read only inside the small blocks the
  last walk found and read anything else as zeros (`sm.heaps.InBlocks`, ADR-0035); over sun_3 the
  reads refused were the works of layouts since freed, which `Layouts` checks again as ORAS's
  does. The language choice keeps its screen: its buttons are pictures, and the generic readers
  said "empty" and its help twice. The name on the talk window's plate is left to the dialogue.
  And the language choice's confirmation said no instructions: they are file 51's line 3, not 2.
- **Replayed** (`spikes/verify_movie.py`, silent, no out-of-sync line in any run). sun_3: the
  title's menu item by item, "Continue rewq Route 1 Play time: 0:08", "Mystery Gift", "Live
  Competition"; the X menu's SAVE, six "empty" slots and OPTIONS; the options' rows with their
  values, "Battle Effects On", each with its help after it; the save prompt's question, "Yes",
  "No", then its date, place and play time. sun_1: the language choice said once, now with the
  confirmation's instructions; "Yes" under "You OK with the one you chose, yeah?" and "So, you go
  by !?"; the keyboard's "!" and "Delete" as its cursor moved, and the name as it was typed. A
  button whose words are on screen but not yet read the same twice waits for them: said as
  "empty" at once, SAVE was "empty" as the X menu opened. ORAS and X (`regress.py`, pc and
  x_city against the morning's run of the code before): the same lines, but for field listings
  a step apart and a sign said a poll earlier, as any two runs differ.
- **Not read:** the Trainer Passport's photo choice and the call's buttons, which are pictures;
  the keyboard's picture keys; and every menu no movie reaches: the Bag, the party, a summary,
  battles.
- **The talk window's answers point as the Poké Mart's do** (2026-09-27, sun_8.ctm, recorded on
  the rig's `user/save_sun_7` with asynchronous reads, 52 s of input: Route 1 to Kukui's "You're
  lovin' Alola already, yeah?", the player moving between the answers). The answers' layout holds
  one Window, `win_00`, holding the TextBoxes "Yeah!" and "...I guess" at x 144.5 and y -17 and
  -37, four `CursorBG` pictures, one on each row, the unused rows hidden, and `Null_arrow`, a plain
  pane holding only the picture `arrow_00`, at x 107, 24 by 20, moved from one row's y to the
  other's. Read as screen text, the box was said whole, "Yeah! ...I guess", and never a move. No
  other picture in the sun_1, sun_3 and sun_8 captures stands alone in a plain pane in a window
  among words, so the cursor reader takes that for a pointer where ORAS has the texture's name
  (`cursor.bclim`, spike 98), and says the words on its row: "Yeah!", then each answer as it moved.
  A heap capture every second and a screenshot with it took the replay elsewhere (the title menu
  at 46 s); without screenshots it followed. sun_3 said the same after the change.

## Spike 125 — Sun's picture-only buttons, and what their panes are called

2026-09-27 (`spikes/125_sun_picture_buttons.py` over the captures of sun_1.ctm and sun_2.ctm, the
runtime's layout, screen text and cursor readers rebuilt over each snapshot; sun_1's 76 to 100 s
captured for it with spike 112). Issue 4: the Trainer Passport's photo choice, the buttons on
Kukui's video call and the keyboard's picture-only keys say nothing, and since spike 124 a button
under a cursor pane that holds no TextBox is said by its pane's name (`Btn_Fight` as "fight").

- **Sun names its buttons by number, not by what they do.** The language choice's cursor pane
  sits on `Null_Cur` in `Button_06`, which the runtime would say as "button" (the language
  choice has its own reader, so it never does); its layout's panes are `Button_00` to
  `Button_08`, `Button_Yes_00`, `Button_No_00` and `intro_icon_00` to `03`. The keyboard's
  layouts (374 and 283 panes) have `Button_00` to `Button_03` and `icon_return`, four times over,
  and its cursor is a key cursor, which says a key's words and so nothing for a picture key. The
  passport's photo choice is a layout of 26 panes, `HeroSel_Chara_00` to `HeroSel_Chara_07` and
  `HeroSel_Chara_en`, with no TextBox; the video call's buttons were not told apart from its
  `intro_icon` panes. The battle's `Btn_Fight` is the exception, not the rule.
- **So no words are there to say.** What could be said is a place: "photo 3 of 8", "key 2 of
  4", from a pane's number, which is what the game shows a sighted player as a picture in a row.
  Whether to, and what to call a photo of a character the game never names, is a question for
  the user under the names rule (AGENTS.md): a name of pa3ds's own lives in a data file, never
  in code.
- **A fresh cursor reader over a snapshot fits a cursor pane the runtime does not**: the call's
  `cross_fade` pane, under a root of its own, was fitted and said as "rootpane" by the spike;
  the runtime, which knows a cursor pane by its moving, never said it over sun_1.ctm.
## Spike 127 — the PC Box's menu for a chosen Pokémon, and a cursor spelt `cursol`

2026-09-27, on or/movie6_pc, pinned (`spikes/95_list_cursors.py` for the layout,
`spikes/96_rom_cursors.py --pattern "curs?o[lr]|cur[0-9._]"` for the ROM, then `regress.py`).

- **Choosing a Pokémon in the Box said the menu whole, and never its cursor** (the user): "Umbra
  is selected." with DEPOSIT, SUMMARY, MARKING, RELEASE and CANCEL all at once, then nothing for
  the 14 s the cursor moved.
- **The cursor is a picture textured `cursol01.bclim`**, the designers' spelling, in a plain pane of
  its own (`9t`) under the root of `box_lower` (398 panes). The game moves the pane onto the chosen
  row's place exactly, (105, 55) on DEPOSIT, while the picture bobs inside it. Each row is a
  picture of its own, `bar1.bclim`, 110 by 30 at x 105 and 30 apart, holding its TextBox. After
  DEPOSIT the same pane walks the grid of Pokémon.
- **The ROM spells a cursor several ways.** Of its 379 layouts, `cursol01` is drawn in three
  (`box_lower`, `trade_lower`, `memory_lower`), `cursol` in six (the field's bottom-screen buttons
  `benri_button`, the Town Map's `map_lower`, three of a minigame's `*_bound`), `cursor1` in the
  move lists (`waza`, `waza_contest`), and `cur` in eight. Only `cursol01` is taken as a cursor so
  far: the field's buttons are on the bottom screen through most of every movie, and taking them
  for cursors unheard could change what the field says everywhere.
- **The Box was taken for a keyboard.** Its party panel is a picture holding a touch area a slot,
  each holding its Pokémon's icon, as a key holds the name entry's cursor, and no named cursor is
  looked for in a keyboard's layout. With one Pokémon in the party its icon, alone, would have
  been a key cursor itself, "poke icon". The name entry's key cursor is drawn with `cursor.bclim`,
  so where a layout's file is known only a picture drawn as a cursor is a key cursor now.
- **Each row being a picture of its own**, where the Mart's rows share one window, the list the
  cursor walks was its own row alone, and the other four were still said as the menu opened. A
  named picture's list is now its row and its parent's children drawn like it: class, x and size.
- **The cursor glides**, over several frames and through each row's own place on its way, so a
  poll on the way said the row it was leaving or passing: DEPOSIT again, SUMMARY or RELEASE from
  CANCEL round to DEPOSIT. Saying a row only where the cursor rested exactly on it still said
  RELEASE in one run of two; holding the item until it read the same on two polls, as the
  Pokédex's plate is, said neither in two runs. The Mart's cursor jumps, and is not held.
- **It leaves for the grid a poll before the menu's rows go**, and a picture over no words points
  at the words on its row to its right (the Mart's boxes, spike 98), so it said "SUMMARY". A named
  cursor that has been over words no longer points beside it.
- **Replayed before and after**, all pinned and following: movie6 said "DEPOSIT", "Umbra is
  selected.", then SUMMARY, MARKING, RELEASE, CANCEL and DEPOSIT as the cursor went, alike in two
  runs, with a diff of 36 lines against the old code where the two runs of the new code differed
  by 76. movie1's keyboards differed from the old code by 106 lines against a control's 138, and
  movie8's Mart only in the field's bearings. Over the grid the cursor says "BOX 1" on the Box's
  name and nothing over a Pokémon, whose summary the top screen shows as before.

## Spike 128 — what a picture under a cursor draws, where no words are

2026-09-27, on or/movie7_menus, pinned (`spikes/128_pictures_under_cursors.py`, and
`spikes/95_list_cursors.py` for pictures of the screens), after the user heard the Pokédex's More
list open two screens that said nothing, and CANCEL say "Habitat" too.

- **CANCEL said "Habitat" from a tie.** The list window's corners (`cur.bclim`, a layout of their
  own) sat centred on CANCEL, whose row lies over the page's Habitat button, and both words were on
  the cursor's row. Of two panels under a cursor the one nearest its middle is taken, and the tie
  went to whichever came first. It now goes to the one nearer across; CANCEL is the list's own
  label, so the cursor says nothing there, as on the list's other rows. Replayed before, after, and
  again after: the two "Habitat"s gone, and the rest within what the two runs of the new code
  differed by (125 lines against 119).
- **Two races showed in one run of three**, the old code's and the new code's alike in kind: the
  list's corners came on a poll before its words and said "HEIGHT WEIGHT" from the entry behind,
  after which the entry was a menu's and its changes went unsaid as the player went from Pokémon
  to Pokémon; and the title menu was read whole, its words a poll before its cursor. Neither came
  in 16 other replays of movies 1, 6, 7 and 8 that day.
- **APPEARANCE/CRY adds no words.** The entry's panel stays as it was; the top screen shows the
  model, and the bottom a strip of buttons drawn only as pictures: arrows round a small icon, up
  and down, Y, X, pause and return.
- **OTHER LANGUAGES draws its tabs as pictures** in a layout whose file is none in its archive
  (built of parts), under the page's own cursor, the red corners, which moved onto ENG: a picture
  (`68`, `word_win.bclim`) holding two more (`55`, `58`). Their file draws them with
  `p_zkn_word_button_jpn.bclim` and `..._jpn2.bclim`, and English shows ENG there, so the game
  swaps each tab's texture as it runs.
- **What a picture draws now is at its material.** A picture holds its `nw::lyt::Material` at
  +0x13C, and the material the physical address of the image it draws at +0x84. For a texture of
  the layout's own, that is inside the layout's archive, in FCRAM (physical 0x20000000 onwards,
  the linear heap's 0x14000000 as the game sees it), and names its `.bclim` there. The tabs' two
  pictures draw from memory in no archive within 4 MB before it, and no archive the runtime has
  read holds the same image. On the entry page, some pictures draw from video memory (physical
  0x18000000 onwards), the one beside TYPE among them, and one draws nothing. Comparing the first
  1 KB of an image names nothing either: a blank start matches the title logo's pieces.
- **So naming a picture the game fills in needs its image matched to the ROM's textures**: the
  tabs, the entry's type, and likely every icon a screen chooses as it runs.

## Spike 129 — the PC Box's slots: its grid, its party and a Pokémon's marks

2026-09-27, on or/movie9_pc_organize_screens and or/movie10_pc_markings, both pinned
(`spikes/95_list_cursors.py`, with pictures), after the user tried ORGANIZE BOXES and could not
tell how to move a Pokémon into a box or the party, and found a Pokémon's marks unreadable.

- **The Box's pointer said nothing over a slot.** Over the party it said nothing and the top
  screen's summary of the Pokémon under it changed, fourteen lines at a move; carrying a Pokémon
  after MOVE, only the box's name was heard, where the pointer rests on it.
- **Every slot is a touch area the pointer rests exactly on.** The box's thirty are
  `nw::lyt::Bounding`s under the layout's root, 40 by 30, in six columns 30 apart (x -130 to 20)
  and five rows (y 55 to -65); the party's six are under its panel's picture (`box1.bclim`),
  50 by 40, in two columns, the second 20 units lower (80, 55), (130, 35), (80, 15) and so on; the
  box's name, its two arrows and the three buttons above it have touch areas of their own. The
  pointer (`9t`, holding `cursol01.bclim`) comes to rest on a touch area's place to the unit,
  gliding in between.
- **A Pokémon in a slot has its icon exactly on it.** Icons (`poke_icon.bclim`) are a pool of 97
  pictures under the root placed as needed; an empty slot has none. The one carried after MOVE
  rides 10 units above the slot the pointer rests on, with a half-transparent shadow beside it. The
  next box's thirty wait shown off the screen, at x 390 to 540.
- **The marks are pictures beside touch areas.** CHOOSE A MARK shows six pictures,
  `box_mark_01` to `box_mark_06` (●, ▲, ■, ♥, ★, ◆, two to a row), each two units left of a
  touch area of its own, which the pointer rests on; all six were drawn half-transparent (alpha
  127) on a Pokémon with none set. No movie sets one, so what a set mark looks like is not known.
- **Built:** a slot in a full grid of touch areas is said by the words above the grid, its row and
  its column, "BOX 1, row 2, column 3"; one in a panel's list of them by its number, "Party 2";
  either followed by "empty" where no icon sits exactly on it; a mark by its sign; and the summary
  after, as its details (the user's choices). A lone button's touch area says nothing, as before.
- **The summary, as details, needed four rules** found in replays one after another. A cursor's
  details are every panel its moves have ever changed, and the pointer walks the Box's menu as
  well as its grid, so the summary was read again after every row of the menu: each group of
  slots is now a cursor of its own, still a move from where the pointer was. An empty slot's
  summary is taken in unsaid. While a Pokémon is carried, the top screen shows it over every
  slot, and carrying Umbra over Blaze's slot said Umbra's summary: a slot is said alone while an
  icon rides above it. And the pointer is held two polls on a slot before it is said (it glides
  through slots' places as through the menu's rows), so Party 1's summary was read once the top
  screen had moved on to Party 2's Pokémon: a cursor held on its way elsewhere drops the details
  it was gathering. Coming back to the slot a group last said went unsaid until moving to another
  group forgot what the others last said.
- **Replayed** movies 6, 9 and 10, pinned, each writing its end save: every slot the pointer
  rested on said where it was, Party 1 and Party 2 each with its own Pokémon's summary, the empty
  slots and the ones passed while carrying Umbra alone, the marks as ★ ■ ● ▲ ♥ ■ ★ ◆ as the
  pointer went, and the menus without a summary after each row. BOX 1 said empty in movie6's
  Withdraw is empty: the picture shows it so, Umbra never having been deposited. Two runs of the
  first version differed by 81, 110 and 6 lines.
- **or/movie11_pokedex_and_box** (the user's, pinned at the first `check`) set a mark, carried an
  item, and moved quickly. **A mark set is drawn solid**: setting ▲ took its picture from alpha
  127 to 255 with a pulse of its scale, it stayed so when the marks were opened again, and the
  save after the movie holds ▲ set in Umbra's record (the byte at 0x2A, one bit a mark) where the
  save before held none. A mark is now said with "on" or "off", and setting one is heard as the
  change, "▲, off" then "▲, on". ORGANIZE ITEMS walks the same slots, and carrying an item needs
  nothing more: giving it to Blaze was heard as his summary's "ITEM Potion".
- **Holding a slot for two screen polls lost quick moves** (the user: "Party n" not always said
  when moving quickly). Two presses 0.4 s apart passed ITEM unsaid, and the summary, changing as
  the pointer landed, was read before the slot said a poll later. Every tick now samples where a
  named cursor stands, one small read, and a screen poll says what it is over once it has stood
  there the last three ticks: a glide moves it every frame. Replayed, the quick presses said
  MOVE, SUMMARY, ITEM, MARKING, ITEM, SUMMARY, MOVE in full, and every slot came before its
  summary.

## Spike 139 — the Bag's party list as a TM is taught, and the move-forget screen

2026-09-28 (`spikes/139_teach_screens.py`). omega_ruby_hm-tm.ctm, on the save of 21:35 (the rig's
`save_checkpoint_smash`): TM51 Steel Wing from the Bag, taught to Wingull, who knew four moves,
over Growl. The user: the party list was not read, and forgetting a move read the Pokémon's
summary at every move. Replayed with 0 out of sync, saying what the recording's log said.

- **The party list names only the chosen Pokémon.** Each of the Bag's party slots is a plain pane
  under `bag_pokebg.bclim` holding an icon and ABLE!, UNABLE! or LEARNED; nothing else. The chosen
  one is drawn on `select_bar.bclim`, a 160 by 40 picture directly under the root, which the game
  moves onto the chosen slot and writes the Pokémon's name, gender, level and ABLE! into, its own
  TextBoxes. The layout also holds the Bag's item list, so it was a menu, and the bar's words
  changing were left to a cursor there was none of: only its level and ABLE! were heard, as they
  came back on screen ("L16 UNABLE!"). The slot's own ABLE! lies under the bar, nearer its middle
  than the name.
- **The bar writes its name a poll before its level**: moving from Ralts to Wingull read "Wingull
  L14 ABLE!" for one poll, then "Wingull ABLE!" (Wingull, paralysed, shows no level). Held until
  it reads the same for two polls, each move said one Pokémon.
- **While the player walks the Bag's items the bar is parked at x -320**, left of the bottom
  screen and still shown; taken for a cursor there it said "L14" and "UNABLE! Bulk Up", and the
  panels those moves changed came back as the details of every move on the party list.
- **"select" is no cursor's word**: across the ROM's 379 layouts, `select_bar.bclim` is only the
  Bag's lower screen's, while the PC Box draws 77 pictures with `select_box01.bclim` and others use
  `select_bg1`, `select_bg2`, `select_bg_line`, `select_win` and `select_battlebox01`.
- **The move-forget screen (`app::wazaoboe::Process`) hangs every word of its top screen from the
  root**: DEX NO., NAME, TYPE, the six stats, CATEGORY, POWER, ACCURACY, their values and the
  description, 25 TextBoxes and no group. A cursor's details are the groups its moves changed, and
  the one group was the whole screen. Its only grouping is drawn: a `nw::lyt::Window` 172 by 85
  round CATEGORY, POWER and ACCURACY, one 395 by 79 round the description, and the stats in
  neither. The moves are the bottom screen's, each a `waza_plate.bclim` with its name and PP, and
  the cursor a window alone in a layout of its own, read already.
- **Built:** the select bar is a named cursor, said by its own words once they hold still, and no
  cursor while wholly off its screen. Words hanging straight from their layout's root are grouped
  for details by the window drawn round them, so a move says "CATEGORY POWER 40 ACCURACY 100" and
  its description. HP reads "HP 0 34": Wingull's 0 of 34, the "/" between them a TextBox with no
  words, left as it is.
- **Replayed** hm_tm, pc, x_lab and sun_9 before and after, and after again as a control: the
  party list said "Grovyle L16 UNABLE!", "Wingull ABLE!", "Ralts L14 UNABLE!", "Frogadier L16
  UNABLE!" as the player went, and nothing else changed but bearings a cell apart and the order of
  sweeps, which the control's runs differ by too. sun_9 went its own way in the control.

## Spike 143 — the PC Box's buttons drawn as pictures alone

2026-09-29 (`spikes/143_box_buttons.py`: `record` replays movies with a line logged for every touch
area in the Box's layout and for each place the pointer stops on no slot; `rom` lists a layout's
panes as the ROM's file lays them out). Step 1 of the
plan (since landed; git history has it): what the buttons the pointer says nothing on are.

- **The pointer never went to them in or/movie6, 9, 10 and 11.** Over the four, it stopped on no
  slot at 53 places, every one on its way between slots, rows or marks, or on the box's name, which
  it says already ("BOX 1", spike 129). A recording of the pointer going onto each is what the
  build needs next.
- **Read live, a pane the game has not placed is at (0, 0).** `box_lower` holds 140 touch areas;
  as the grid was up, the modes' and the arrows' were hidden, at (0, 0), and read as holding
  everything else placed there. So the layout was read from the ROM instead: file 0 of `a/1/0/3`,
  398 panes, as the live layout has.
- **Above the box, three pictures, each holding a touch area of 60 by 20**: `mode01.bclim` at
  (-115, 120), `mode02.bclim` at (-55, 120) and `mode03.bclim` at (5, 120). Their names are not
  words.
- **The box's arrows are pictures `left_button.bclim` at (-160, 90) and `right_button.bclim` at
  (34, 90)**, each holding a touch area of 25 by 25 beside the box's name: "left" and "right", as
  `layout_file.words` says a picture button.
- **The buttons below the box are not the Box's layout.** `box_lower` keeps only empty panes
  along its bottom (`h3` to `h7`, at y -105). The buttons are `touch_bar_lower` (file 0 of
  `a/1/1/0`), a bar shared by many screens, which shows a group of its buttons at a time: the
  Box's are `return_icon.bclim`, `exit_icon.bclim` and `search_icon.bclim`, each a picture and a
  touch area of 30 by 30. Its other groups include `change_box`, three `horogram_btn`s,
  `poke_button`, `item_button`, `params_icon`, `contest_icon`, `memo_icon`, `map_big`,
  `map_small`, `left_btn`, `right_btn`, `monster_icon`, `waza_icon`, `trade_icon`,
  `rireki_icon`, `sort_icon`, `up_btn`, `down_btn`, and `makimodosi` (Japanese, "rewind").
  `layout_file.words` drops a trailing "button", not a trailing "icon": these would be said
  "return icon", "exit icon", "search icon". And the slots are looked for in the pointer's own
  layout, so a button of another layout is not found at all.
- **or/movie12_box_buttons, where the user went looking for them** (2026-09-29): only one was
  reached. Down from the grid's bottom row the pointer rests at (-110, -105), on the empty pane
  `h7` the Box leaves, and under it in `touch_bar_lower` is touch area `r`, 80 by 30, beside
  picture `o`, `change_box.bclim`, its middle exactly where the pointer rests. The bar's group
  in the Box is not return, exit and search, as guessed from the file: the pointer never rested
  on those. Up from that button the pointer wraps round to the box's name, and down from the
  name back to the button; up from the name, and left and right on the grid's edges, it moved
  nowhere else, so the modes and the arrows are for touching.
- **Built** (`slots.button`): a cursor resting on no slot of its own layout looks for a touch
  area of another layout on its screen whose middle it rests on, and says the words of the
  picture beside it, as the battle's picture buttons are said. Replayed, movie12 said "change
  box" each of the 24 times the pointer came to rest there, and nowhere else. The Box's own
  lone buttons stay unsaid.
- **From the box list the same button was said as nothing** (the user, 2026-09-29). Screenshots of
  the replay (`--shots`) show the list as eight folders on the bottom screen, the chosen box's
  name on the top, and the bottom bar's buttons unchanged: change box, then search, close and
  return; the pointer is still the Box's `cursol01` in `box_lower`, and rests on "change box" as
  before. But the list hides every word of `box_lower`, and named cursors were looked for only
  in layouts with words on screen: from 85 s to 101 s the cursor reader never looked at the
  pointer. Looked for there too, the list's folders read as the Box's slots, "row 2, column 1,
  empty", and the box's name the top screen says as the pointer moves became their unsaid
  details. So a layout a named cursor has been found in is looked in while it lasts, and a cursor
  in a layout whose words are all hidden looks only for another layout's button: replayed, the
  list said "BOX 5", "BOX 4" from the top screen as before, and "change box" each time the
  pointer came back to it.

## Spike 147 — a picture's image read whole, and named by the ROM texture it is

2026-09-29, on or/movie7_menus, pinned (`spikes/147_runtime_textures.py`), steps 1 and 2 of the plan that became
[ADR-0036](../adr/0036-a-picture-is-named-by-the-image-it-draws.md): the Pokédex's OTHER
LANGUAGES tabs and the entry's type, which spike 128 found drawn from memory no layout file names.

- **A material's texture map says the whole image.** From a picture's `nw::lyt::Material`
  (+0x13C): +0x84 the image's physical address, +0x88 its width and height (u16 each, 160 and 30
  for `touch_bar.bclim`), +0x8C the size it is stored at, each rounded up to a power of two (256
  and 32), and +0x9C the GPU's texture format (RGBA8 0, RGB565 3, RGBA4 4, IA8 5, ETC1A4 0xD).
  Checked against every picture whose texture is a file of its layout's own archive, whose
  `.bclim` footer gives the same width, height and format. The stored size and the format's bits
  a pixel give the image's length: 4,096 bytes for a tab, 64 by 32 in IA8.
- **Video memory reads.** Physical 0x18000000 onwards is the game's 0x1F000000, and pictures that
  draw from there read back what they draw; FCRAM is the linear heap, as spike 128 found.
- **The image in memory is the file's image, byte for byte.** Every `.bclim` in the ROM, through
  GARCs, LZ11, and the `darc`s in a layout pack (after a table of names, 34 KB in for `a/1/0/3`),
  hashed over its image data: 9,974 textures, 8,156 distinct images, in 554 s. Hashed the same,
  the tabs are `p_zkn_word_button_eng.bclim` to `_spa` (and the `2` beside each) in `a/2/3/5`, the
  entry's type `type_icon_09_honoo.bclim` in `a/2/1/2`, the summary's marks `box_mark_01` to `06`
  and its move categories, and the Pokédex's Pokémon a file of `a/2/6/3` each.
- **A GARC entry holds a variant a language.** Its FATB bits say which, and `garc.garc_files`
  keeps the first. The English ROM draws variant 2: ENG, ITA, JPN and SPA are the same in every
  variant, and FRE, GER and KOR matched only once every variant was indexed (55 to 235 bytes off
  the first's). Anything read from a GARC for its words may want the player's variant. **The bit
  is the game's language number**, the one the text archives are laid out by (`games.oras`): 1
  Japanese, 2 English, 3 French, 4 Italian, 5 German, 7 Spanish, 8 Korean; no entry has bit 1,
  so bit 0 is the copy Japanese and any language without its own falls back on. The English tab
  in bits 4 and 7 is one image, Italian and Spanish both saying "ING".
- **The game names the languages in its own text, in every language.** Text file 103 of each
  language's archive is the language choice's (`app::lang_select`): lines 6 to 12 "English",
  "Spanish", "French", "German", "Italian", "Japanese", "Korean" in English, "Anglais" ...
  "Coréen" in French. File 123 lines 8 to 14, and again 23 to 29, are the tabs' own abbreviations
  in the tabs' order, "JPN ENG FRE GER ITA KOR SPA" in English, "JAP. ANGL. FRA." in French, the
  same order as the textures' `jpn eng fra ger ita kor spa`.
- **The type's texture carries the game's own index.** `type_icon_09_honoo` is Fire, and 09 is
  Fire in the game's order of types (06 mushi Bug, 12 denki Electric, 13 esper Psychic), so the
  type can be said from the game's text of type names, not the texture's Japanese.
- **A locked language tab is drawn faint, and the cursor never goes to it** (or/movie14, the
  user pressing left and right for 19 s, 2026-09-29). Torchic's entry has only its English: the
  six other tabs are shown at alpha 100, ENG at 255, and the page's cursor stayed on ENG through
  every press (screenshots every 2 s, `watch`). So left and right said nothing because nothing
  moved; a Pokémon's entry in another language comes with a Pokémon from a game in it.
- **With the index, screen text says the type icons** (Orion, 2026-09-29): the Pokédex entry
  "Male TYPE Fire" and "TYPE Dark" as it goes to Poochyena, the summary's "Type Fire" and each
  move's type, "Ember Fire PP 25/25", "Ice PP 15/15" for Ice Fang (movie7 replayed before and
  after). The summary parks a fifth move's type icon at 140 above the bottom screen's middle,
  130 to 150 against its edge at 120, which a picture wholly off its screen no longer says.
- **Status conditions are icons, and the game names them.** `sick_icon_01_mahi` to `_05_doku`
  (paralysis, sleep, freeze, burn, poison) and `_06_dokudoku`, badly poisoned, and a
  `pokerus_icon`, in `a/0/0/4`, `a/0/0/5` and `a/2/1/2`. Every layout drawing one names
  paralysis's in its file and swaps it as it runs: the battle's HP boxes (`btl_up01` to
  `btl_up05`), its party list (`poke_list`), its command menu, its move list and its status page.
  Text file 123 lines 50 to 56 are PARALYSIS, ASLEEP, FROZEN, BURNED, POISONED, FAINTED and
  POKÉRUS, in the icons' order; badly poisoned has no word of its own. Not seen on screen: no
  movie has a Pokémon with a condition. The move categories' icons, `damage_type_icon_00_henka`,
  `_01_buturi` and `_02_tokusyu`, are file 150's STATUS, PHYSICAL and SPECIAL, lines 17 to 19,
  heard on movie7's move details as "Category PHYSICAL Power 40 Accuracy 100".
- **The battle's move list mirrors its two columns**: the left buttons draw the type icon at
  their left end, the right ones at their right, on the PP's row, so read left to right a move
  is "Tackle Normal PP 35/35" on the left and "Growl PP 40/40 Normal" on the right (movie5).
- **What is left unmatched is not a layout's picture.** 31 of 107 images: ETC1A4 in video memory
  (512 by 128, 128 by 128, ...), likely models' and backgrounds' textures and not `.bclim`s, and
  six of the Pokédex's own archive the census does not reach, which its layout file names anyway.

## Spike 148 — the summary's labels, and a colon told by place

2026-09-30 and 2026-10-01, on or/movie7_menus and or/movie11_pokedex_and_box, pinned
(`spikes/148_summary_labels.py`, which logs every pane of the layouts showing some words, with its
name and place). Orion asked for "OT: Raiden, ID No.: 16285" and "HP: 23/23" with `pauses` on.

- **The summary names no pane for what it holds**: its panes are "0", "4", "1k", labels and
  values alike, so a label is told from its value by place alone (`screen._labels`).
- **HP's "/" is the layout file's own**, between two written numbers, and every piece of that
  panel hangs from the layout's root, each a panel of its own: a separator is now one of its
  numbers' parent too, and "HP 30 30" is "HP 30/30".
- **Replayed with `pauses` on (2026-10-01)**, movie7, movie11 and x/movie2_pokedex, every line
  given a colon was read. Right from the first: the summary's "OT: Raiden, ID No.: 16285, Exp.
  Points: 147", "HP: 19/19", a move's "PP: 15/15", its details' "Category: PHYSICAL, Power: 40",
  the Pokédex entry's "TYPE: Fire, HEIGHT: 1'04", WEIGHT: 5.5 lbs.", and X's "CENTRAL KALOS
  POKÉDEX: 1" and "SEEN: 2, OBTAINED: 1". Wrong, each with what measured it:
  - A label with its own colon took a comma: "Badges:, 0", "Time:, 0:40", "Number caught:, 2",
    the trainer card's "Money:, $ 3,440". It is followed by a space now, "Badges: 0".
  - A value became the next piece's label: "ITEM: Potion: Sand Attack", "NATURE: Bold: Run
    Away", "Name: Raiden: ID No.:". What was just taken as a value, or follows a label's own
    colon, labels nothing now.
  - A list's next row read as a value: the Box's summary draws its moves at x 50, Ember at row
    -80 and Sand Attack at -100, so "Ember: Sand Attack", and the main menu "Raiden: Petalburg
    City", both at x -60. Exp. Points' number, the one value measured on a row under its label,
    starts past the label's end (-51 against -126): a value below its label must now.
  - A type's icon labelled the heading beside it, "Fire: MOVES LEARNED": a picture's words are
    no label now. Nor are words that changed, which a label's never do: switching Pokémon changed
    ABILITY's "Blaze" and a move's "Ember" together, "Blaze: Ember".
  - ABILITY's value is drawn 10 above it (-80 against -90), so "ABILITY, Run Away": a value may
    start a glyph above its label now. That made the gender sign, a glyph of the game's own at
    row 110, the label of "Lv.5" at 120 ("Umbra, : Lv.5"): a label has a letter now.
- **Still wrong**: the Box's summary, as its details are gathered, reads its panel out of row
  order — "NATURE: Careful, Blaze: Ember, ABILITY" — the same out-of-step reading as "147 Speed
  9" on the summary; "Blaze" beside "Ember" on one row is a table's label and value by place.
  "CONTINUE: Raiden" is a heading over the save's name, which place cannot tell from a label.
  Sun was not replayed: no Sun movie has a `[[replay]]` on this machine.

## Spike 152 — the Pokédex's habitat: which places on its map are the Pokémon's

2026-10-01, on or/movie7_menus, pinned (`spikes/152_habitat.py`, dumping every pane of the
habitat page's layouts at moments of the replay), after Orion asked for the habitat page to say
the Pokémon's places, and whether the map's cursor is on one.

- **The top screen is words over a background.** Layout `0x82ea47c`: two TextBoxes, the title
  ("Poochyena’s Habitat") and the place under the map's cursor ("Route 102"), over three map
  icons (`map_ico_00` to `02`) that stay as they are while the cursor moves. A layout behind it
  is hexagons and stripes. Nothing on the top screen says which places are the Pokémon's.
- **The bottom screen is the AreaNav's map**, `map_lower` (`a/0/6/7`): `poke_navi_map.bclim`
  under a pane of touch areas (`nw::lyt::Bounding`), one a place, a town's holding its icon, and
  a few holding plain panes that carry the shape on. Each route's touch area holds two
  `poke_point.bclim` pictures, and **a place the Pokémon lives in has one of them drawn**:
  Poochyena's `m`, `1p` and `1q`, the rest none. Which of the two is drawn, and why there are
  two, is not known. The cursor is `cursol.bclim` in a pane of its own, placed on the touch
  area of the place the top screen names (`1f`, Petalburg City).
- **The ROM names every touch area.** The map's archive opens with a u32 count of its files and
  their names, 64 bytes each, and a u32 count of its layout's panes and their names, 32 bytes
  each (681, sorted, a few repeated); the darc follows. The AreaNav's list of what each town has
  (`a/0/6/6`, 60-byte rows) keeps at +0x04 the index of its zone's touch area among those
  names: row 0, Littleroot Town, is `l`; Route 101 `m`, Route 102 `1p`, Route 103 `1q`. Several
  rows share a touch area where a cave's floors or a route's halves are one place on the map.
  72 touch areas named, in every language the ROM carries (`archives.map_places`).
- **The game draws every place a Pokémon lives, not only where the player has been**:
  Zigzagoon's eight include Route 110, 116 and 117, far ahead of this save. Torchic's none, and
  the bottom screen says "Habitat Unknown". Mudkip, never seen, has none drawn either.
- **Built** (`gen6.habitat`): the page opens on "Poochyena’s Habitat Route 101, Route 102,
  Route 103" (a colon after Habitat with pauses on), then the place under the cursor; each move
  says the place alone, "Found in Route 102" where it is drawn so (Orion's words, after "Route
  102 is a known habitat" was too long).
  Read once the title and the places have read the same for two polls. The top screen's two
  words are the reader's while the map is up, as the HP box is the battle reader's: left to
  screen text, every move said "Poochyena’s Habitat" again.
- **Before and after** over movies 7, 11 and 14: only movie7 opens the page; every other
  difference is a race of the kinds the runbook lists. Movie14 went its own way on both sides.
- **Seen in pictures of the replay** (2026-10-01): a habitat is a row of small white squares
  with yellow dots laid along the route, not a Poké Ball; the two `poke_point` pictures are
  likely its animation's two frames. Towns are pink squares, other places green, land routes
  yellow and sea routes light blue, the cursor a red arrow. **Over a habitat the top screen
  also shows how the Pokémon is found**: three icons under the title, tall grass, water and a
  fishing rod, the ones that apply in colour and the rest grey (Route 101 and 102: grass alone).
  The picture without them was taken with the More/Habitat bar still on Habitat, before the map
  opens (spike 153).
- **Left**: X's habitat, drawn on Kalos's map, was not looked at (`map_panes` is ORAS's alone).
  "——————", the place with the cursor over no place, is said as the game draws it.

## Spike 153 — the habitat's icons, the moment its map opens, and the Pokédex's delay

2026-10-01, on or/movie7_menus, pinned (`spikes/153_habitat_icons_and_delay.py`, and
`spikes/128_pictures_under_cursors.py --words Habitat --materials` for what each icon draws),
after Orion heard the place under the map's cursor said before the map was open, asked for how
a Pokémon is found, and found the Pokédex a second slow.

- **The map is open when the icons are on the top screen.** With the More/Habitat bar on
  Habitat, the panel under the title (`9`) is below the screen, its icons' middle at y -165
  (Torchic) or -136.5 (sliding up as A opens the map); open, at -70, over every place, a habitat
  or not. The bar's words leave the bottom screen at the same time.
- **An icon in colour draws the image its layout file gives it; in grey, the game's other.**
  Each picture's material points into its layout's own archive, loaded whole in memory: the
  grass icon draws `map_ico_00.bclim` over Route 101 and 102 and `map_ico_00_1.bclim` over
  Oldale Town; water and the rod drew `map_ico_01_1` and `map_ico_02_1` throughout. The
  images are not in the texture census, which does not reach the Pokédex's archive (spike 147),
  and at 8 KB or more they are past what `textures.drawn` reads: they are named by where they
  start, among the archive's files (`LayoutFiles.image_at`), never read.
- **Built:** the place is said once the map is open, and over one of the Pokémon's places with
  the icons in colour: "Found in Route 102 grass" (Orion's words: grass, water, fishing).
  Water and fishing in colour have not been seen.
- **The Pokédex's delay is not in finding its words.** From the poll a Pokémon's words are first
  read to the line saying them is one poll, 0.2 s, the hold every screen's words wait; a poll is
  late by 0.3 to 0.6 s about every ten seconds, when a sweep runs. But a Pokémon's entry is said
  in the order it changed, "010 Poochyena", "Bite Pokémon", "TYPE Dark", "HEIGHT 1'08"",
  "WEIGHT 30.0 lbs.", "Times encountered: 4", and the description after them all, a second or
  two of speech later.

## Spike 154 — a Secret Base's menus, and their cursor a row too small or two rows too tall

2026-10-01 (`spikes/154_secret_base.py --layouts`, and `spikes/139_teach_screens.py`, over
omega_ruby_secret-base.ctm). The user booted their Secret Base's PC, chose Decorate your base,
opened the item list and walked its categories and its one item: the PC's menu was said whole as
a screen at each move, the categories were said again as screen text every time the list
scrolled, the item's cursor said only "× 1", and the closing YES and NO said both.

- **Every list is one layout, its cursor `s_b_cursor.bclim`.** The PC's menu, the categories and
  the items are rows of 210 by 20, each a plain pane holding its TextBoxes ("Red Flower" and "×
  1" in two), and the cursor two pictures of `s_b_cursor.bclim`, 120 by 20, in a pane 30 by 40
  at the row's middle, alone in a pane `3i` the game moves onto the chosen row at the row's own
  place and size. The green bar is drawn under the row.
- **The cursor's pane is narrower than its words.** A named cursor's box was the pane holding its
  pictures, 30 wide at x -85, and the row's words start at x -190: none was under it, and the
  count, right of the middle, was the only word ever said. `3i` is the row exactly.
- **The YES and NO's pane is two rows tall.** Its two halves are 50 by 20 at YES's row, in a pane
  110 by 40 centred there, which reaches NO's row 20 below: "YES NO".
- **Opening a category writes its items into the rows a poll before the categories' words go**,
  and the cursor, at rest, said "Plants Red Flower × 1" from one row; so did "Chairs Back" going
  back.
- **The list scrolls under a cursor at the top or bottom**, wrapping from Desks to Back; the
  screenshots every 0.4 s agree with every row the cursor said.
- **Built** (`cursor._named`, `_named_rect`): a named cursor's pane alone in a plain pane is
  that pane, a plain pane of cursor pictures is as tall as its pictures, and words rewritten
  under a cursor at rest are said once they hold for a second poll. Replayed: "Decorate your
  base", "Desks", "Plants", "Back", "Red Flower × 1" with its description after, "YES".
- **Left open**: the categories with nothing in them are drawn grey (Desks, Dolls, Cushions,
  Tricks) and are said as the others. The cursor on the base's floor is read by the field since
  (field-and-rom.md, spike 154, decorating).
