# pa3ds in a page

pa3ds speaks 3DS Pokémon games to blind players. It attaches to a running Azahar, reads the game's
own memory over the emulator's RPC, and sends what it finds to the player's screen reader. It never
presses a button and never writes to the game.

This file is the **vocabulary**: the words pa3ds uses, in code and in conversation. If a concept is
not here, it does not have a name yet — add it here before inventing one in code. Where a decision
is *why* something is the way it is, that is an [ADR](docs/adr/README.md); where a fact was
measured, that is a [finding](docs/findings/README.md).

## Reaching the game

**Emulator** — Azahar, running a 3DS game. pa3ds attaches to a running instance; it does not
launch it and does not modify it.

**RPC** — Azahar's built-in UDP request interface on port 45987. Supports ReadMemory, WriteMemory,
ProcessList, SetGetProcess. pa3ds only ever reads.

**Fingerprint** — a hash identifying exactly which game and version is loaded, derived from the
executable image. Everything cached is cached against a fingerprint.

**Anchor** — a structure in RAM that pa3ds can find without knowing its address, by scanning for a
recognisable pattern and confirming it with a **validator**. Anchors are the root of every read;
addresses are derived from them, never written down.

**Validator** — the check that proves a scan hit the real structure rather than a coincidence. An
anchor without a validator is a guess.

**Class name** — the mangled C++ name an object's vtable leads to through the executable's RTTI
(`N5print6MsgWinE`). The cheapest anchor there is: scan for the class's vtable, no guessed shape
needed. A name says what an object was built as, never that it is still alive. A class in a DLL
(the map blocks', the battle's gauges') has no vtable until the game loads it. A **module** (a
CRO) is loaded after whatever is loaded beneath it, so where it lands depends on that (spike 140);
the loaded ones are listed by their headers at every app start, each read the first time it is
seen and found again by where it lands. Gen 6's class names
are read in the code there was as pa3ds started, and cannot name a DLL's class: an object of one
is checked by the vtable it had when a sweep found it (spike 138).

**Heap** — one of the game's allocators, `gfl::heap::CTRExpHeap`, around the SDK's expanded heap.
Its head keeps a list of its **blocks** in use, a new one at the list's end (or its front, taken
from the top of the heap); heaps link to their siblings and children, so one found leads to all
(spike 110).

**Sweep** — reading memory for the vtables of the classes the readers want. A **whole** sweep
reads all of the heap, at the start, after an app starts, when a reader is waiting, and every 30 s
as a net. Otherwise only the **new blocks** the heaps have just
allocated are swept, a tenth of a second after they appear: a **partial** find, which says what is
new and nothing of what is gone, so readers add it to what they know (ADR-0034). In Sun a whole
sweep is a walk of the **heap table**'s small allocations, and only a new block of 4 KB or less is
swept, to the byte (ADR-0035).

**Watch region** — a byte range polled every tick. Regions are batched into as few RPC reads as
the packet size allows: each poll reads in a **phase** of the page cache, which fetches whole pages
pipelined and starts with the pages the same phase read last time.

**App** — one of the game's screens as the game runs it: the field, the Bag, the Pokédex, the
party, a battle, the title menu. Each is a process object that `gamesystem::GameProcManager`,
which lives all session, points into while it runs, and its class names it (`app::bag::Proc`,
`btl::BattleMainProc`). One read a poll says when another app starts, which is when a sweep has
something new to find (spike 99). The X menu and the Poké Mart are the field's, not apps. Sun's
manager is `GameSys::GameProcManager`, which points at a record of the process in heap 6, a heap
of two small blocks the walk reads for it; the record leads to the process, `Field::FieldProc` on
the field (spike 116, 2026-09-27).

## Turning memory into speech

**Tick** — one poll of every watch region. Rate is configurable (30 or 60 Hz) and rises
automatically when the emulator is running fast-forward.

**Event** — a change found by diffing this tick's regions against the last. Events, not raw memory,
are what the rest of the system consumes.

**Reader** — code that turns events into things worth saying: the dialogue reader, the menu reader,
the battle reader, the keyboard reader. A reader owns one kind of screen or state.

**Message window** — a box of game text: `print::MsgWin`, or its base `gfl::str::StrWin`; in Sun
`print::MessageWindow` and `gfl2::str::StrWin`, the same fields under other names. It holds
a whole message and lays out one page of it at a time. Signs, the TV, the opening and battle text
all use them.

**Page** — what a message window shows between two waits for A. Dialogue is spoken a page at a
time, the moment the page starts to type.

**Line object** — the `nw::lyt::TextBox` a message window types one line of a page into. It holds
the line as far as the typewriter has got, and names the tag processor laying it out.

**Tag processor** — the `print::XYTagProc` that lays a line object's text out. A message window
embeds its own; a screen that writes into a line object directly uses one of its own.

**Live** — a message window whose line objects are all still TextBoxes, none of them taken by
another window's tag processor. Windows keep their text long after they close, and hand their line
objects on to the next window; only a live one is being shown.

**Written text** — text a screen writes straight into a window's line objects under its own tag
processor, leaving the window's message alone, as the options screen does with each description.
Spoken whenever it changes.

**Layout** — a NintendoWare `nw::lyt::Layout`: the tree of panes one screen draws, on the top screen
or the bottom. Screens build their layouts long before showing them.

**Pane** — a node of a layout (`nw::lyt::Pane` and the classes derived from it). It is on screen only
if it and every pane above it are visible. Sun's NintendoWare keeps a pane's size, place, flags and
name, and a TextBox's fields, in other places than ORAS's, and the smaller objects end sooner: a
game's **pane shape** (`lyt.PaneShape`, `sm.lyt.SHAPE`) says where, and every layout reader reads
panes through it (spike 112).

**Layout work** — Game Freak's record for one layout, which says whether the layout is being drawn.
It has no class of its own; the `nw::lyt::DrawInfo` inside it has.

**Layout pool** — where a screen's layout works are built: a header pointing at itself, a table of
(pool, work) entries, then slots of 0x104 bytes, and the pool's own work after the last. Every
work's first word leads to its pool, and a watched pool's count of slots in use changes when a
layout is built into it, which is taken in at the next screen poll without a sweep (spikes 64
and 99).

**Screen text** — words a screen sets straight into a layout's TextBox rather than shows in a message
window: menu labels, option names and values, a question above a list. Heard when it comes on
screen and when it changes.

**Menu cursor** — the red corners around a menu's chosen item: a layout whose root holds a single
`nw::lyt::Window`, placed over the item. The item under the cursor is the screen text inside it.
The corners hold nothing: a window alone under its root that holds panes of its own is a box its
words are drawn in, as the Poké Mart's Yes and No are (spike 98).
The main menu draws the same corners as a window directly under its own layout's root, known as a
cursor once it has moved from one resting place to another while its sibling windows stayed.
Sun's name entry keyboard draws its cursor as a window alone under a Parts pane, a layout built
into the keys' own, moved and sized onto the chosen key (spike 112).

**Picture cursor** — the battle's cursor: a layout whose root holds a single small
`nw::lyt::Picture`, placed exactly on an empty pane in the chosen button's group. The item is that
group: its screen text, or the words of its first picture's texture name when it has none.

**Choice frame** — the opening's boy-or-girl cursor: an `nw::lyt::Window` holding no panes of its
own that the game slides from one choice's place onto another's. The choices are the plain panes
beside it, all one size and each in a place of its own; the choice is the one the frame sits
exactly on. It is said by the screen text under it, or by the words of the textures the other
choices do not have, since every choice is drawn on the same backing.

**Layout file** — the CLYT a layout was built from, kept in memory. It lists the layout's panes in
the order they are built and names the textures its pictures use, which is how a button drawn as a
picture gets a name ("fight_button.bclim" is said "fight": a trailing "button" is left out, and an
accent a file name cannot carry is put back, so `pokemon_button` is "pokémon").

**View offset** — a translation a layout work applies to its whole layout (+0x74): 290 on the
field's bottom screen, 1,000 in battle. It is in every pane's global place, so a word's place on
screen is judged from its layout's root.

**List window** — `app::tool::MenuWindow`: labelled items and a cursor, as in a Yes/No prompt.
Its cursor graphic is a separate object, `app::tool::MenuCursor`.

**Choice box** — the small box of options the field puts beside a speaker once A is pressed on a
question: "Yes" and "No" for Birch's questions. Not a list window, and without a class of its own;
found through the input device manager it holds. The Mart's Buy, Sell and No thanks is one too; a
purchase's Yes and No, drawn the same way, is not, and only its cursor picture reads it (spike 98).

**Fallback scrape** — what happens on a screen no reader claims: pull whatever readable text is
there, speak it with a marker saying it is a fallback, and capture a snapshot.

**Profile** — everything game-specific: fingerprints, anchors, struct layouts, archive ids, readers.
The core (`pa3ds.core`) knows nothing about Pokemon. There are two: gen 6's (`games.gen6.profile`),
which reads ORAS and X and Y, and Sun and Moon's (`games.sm.profile`), which reads dialogue with the
common dialogue reader given Sun's **shapes** — its message windows' class names and where its
TextBoxes keep their fields — menus and screens with the common layout, screen text and cursor
readers given Sun's pane shape, and the language choice a new game opens with, in the words of the
player's ROM. A profile says the classes a sweep looks for, which reader each object found goes to,
and the order readers are polled; the profile, and how the game's heaps are read, reach the runtime
as a **session**, which the engine's package puts together (`games.common.session`). A **game
package** (`pa3ds.games.<game>`) holds what one game has of its own: its numbers, its `Game` and
`data/<language>/` (`oras`, `xy`), or a profile and readers of its own (`sm`). What games share has
a package of its own: `games.common` what every game is read the same way in, and `games.gen6` the
engine of X, Y and ORAS.

**Game** — what the running title id says is running (`games.catalog`): ORAS, X and Y, or Sun and
Moon. ORAS and X are one engine, gen 6, read by one profile and one set of readers
(`games.gen6`), and each game's package says what it keeps elsewhere — where its ROM files are,
where a move model keeps its sprite — and what it alone has, ORAS's DexNav, in a `Game`
(`games.gen6.game`, spike 100). Sun and Moon are another engine, gfl2, with a profile of their
own (spike 111).

**Heap table** — Sun's list of its heaps, at the start of the application heap: a heap an id,
each a `gfl2::heap::CTRExpHeap`. Sun keeps nearly everything in linear memory with the GPU's
buffers among it, and reading one of those takes the emulator down, so pa3ds walks the heaps
under the dialogue's and the field's from this table, block by block, and reads only the **small
allocations**, 4 KB or less, each to the byte, reading each with the walk (ADR-0035). The field's
own heap, 0x21, is walked for the heaps inside it and not for its own blocks, the rendering
engine's (spike 116). Between walks the same heaps are watched, and what may be read is every small
block the last walk found and every one allocated since, less any a newer block lies over (spike
123).

**Key cursor** — the name entry keyboard's cursor: a picture the game moves into the chosen key's
`nw::lyt::Bounding` touch area. The item is the words of the key holding that touch area. Where
its layout's file is known, only a picture drawn as a cursor is one, as the name entry's
`cursor.bclim` is: the PC Box keeps its party's icons in touch areas too (spike 127).

**Plate** — the Pokédex list's cursor: an `nw::lyt::Window` holding no panes of its own that never
moves, sitting on one row of a **list** while the rows scroll under it. A list is three or more
plain panes under one parent, each holding words, all at one x and of one height. The item is the
row the plate sits on, said once it has read the same for two polls.

**Named cursor** — a picture or window drawn with a texture its designer named a cursor, as the
Poké Mart's `shop_cursor.bclim`, the PC Box's `cursol01.bclim` (the designers' spelling) and the
main menu's frame `cursor_lt.bclim` are: a cursor from the poll it is seen on, over the words
inside it, or inside the pane that holds the picture. A plate and a named picture know the list
they walk, and own only that: the rest of their layout — the Mart's money and the chosen item's
description — is screen text, not the menu's. A named picture's list is the row it is over and the
rows drawn like it, its parent's children of its class, x and size; over rows each drawn on a pane
of its own, as the Box's menu is, its item is held until it has read the same for two polls, since
it glides through the rows between (spike 127). A named window owns its layout whole, as any menu's
cursor does. A named picture over no words points at them from beside, unless it has been over
words: the item is the words on its row, to its right, as in the Poké Mart's boxes of choices,
which draw `cursor.bclim` left of Buy, Sell and No thanks, and of a purchase's Yes and No (spike 98).
A named cursor is followed in its layout while the layout lasts, its words all hidden or not: under
the Box's box list it says only another layout's button it rests on (spike 143).
A named picture holding words of its own says those: the Bag's `select_bar.bclim`, the plate
the chosen Pokémon is drawn on as a TM is taught, carries its name, level and ABLE!, said once
they have read the same for two polls; parked off its screen, as while the player walks the
Bag's items, it is no cursor (spike 139).
A Secret Base's menus draw `s_b_cursor.bclim` in a pane 30 wide, alone in a pane the game moves
onto the chosen row at the row's size: that pane is the cursor's place. A pane of cursor pictures
taller than they are, as the YES and NO's is, is as tall as its pictures; and words rewritten
under a cursor at rest are said once they read the same on two polls (spike 154).

**Slot** — a place a named cursor rests on where no words are, found as the touch area
(`nw::lyt::Bounding`) it rests exactly on: in the PC Box, a box's grid, the party, and a
Pokémon's marks (spike 129). A slot in a full grid of touch areas is said by the words above the grid
and its row and column ("BOX 1, row 2, column 3"), one in a panel's list by its number ("Party
2"), and either as "empty" where no Pokémon's icon sits exactly on it; a mark by the sign its
picture shows (●, ▲, ■, ♥, ★, ◆) and "on" where it is drawn solid, "off" half-transparent. A lone
button's touch area is not a slot. Where the cursor rests on no slot of its own layout, a lone
button of another layout on its screen whose touch area it rests on the middle of is said by the
words of its picture: the bottom bar's `change_box.bclim`, "change box" (spike 143). A cursor walking slots or rows each drawn apart glides through
them, and says what it is over once it has stood there three ticks, sampled every tick. Each group of slots is
a cursor of its own, whose details are the top screen's summary, left unsaid for an empty slot and
while a Pokémon is carried.

**Habitat map** — the Pokédex's habitat page's bottom screen: the AreaNav's map, each place a
touch area, a place the Pokémon lives in drawing a `poke_point.bclim` picture over it. The ROM
names each touch area, through the AreaNav's list of what each town has (`archives.map_places`);
the top screen names only the place under the map's cursor (spike 152).

**Pointer** — the hand that shows a tap in a battle the player watches: Wally's catching tutorial,
the one battle fought for the player. A small picture alone in a layout but for the pane that
places it, which the game puts on the button about to be tapped. Its item is that button, found
by the touch area (`nw::lyt::Bounding`) under that place.

**Cursor pane** — Sun's menu cursor, in every menu read so far: a plain pane alone under the root of
a layout of its own, holding only the arrow's pictures, moved and sized onto the chosen item. Each
button keeps an empty pane of exactly that place and size, and the item is that button, all its
words; where the buttons' own panes are hidden and never placed, as the Yes and No's are, the item
is the words inside the cursor pane. A button with no words on screen is said as "empty": the X
menu's slots for what the story has yet to give (spike 112). The battle's cursor pane is 8 wider
than the `Null_Cur` it lands on, so a cursor pane fits a pane up to 8 smaller, the nearest in size
first; and a button holding no TextBox at all is said by its pane's name, `Btn_Fight` as "fight",
as ORAS says a picture button by its texture's (spike 124).

**Battler** — one side of a battle as it is said: name, gender sign, level, and either current
over maximum HP (the player's) or a percentage (the enemy's, because a bar is all the game shows).
The name and sign are the **name plate** (a `gfl::str::StrBuf` whose text ends in ♂ or ♀, one for
each Pokémon in the battle, which exist only while a battle is up) of the Pokémon its gauge shows.
Sun writes the name, level and HP into its panel as screen text and draws the sign as a picture,
so the name is the panel's own and the sign is the Pokémon record's (spike 124).

**Gauge** — the battle's object behind one Pokémon's HP reading (`btl::BtlvGaugeSelf` for each of the
player's, `btl::BtlvGaugeBase` for the rest): the battle record of the Pokémon it shows, the
**group** of panes its reading hangs from — level TextBox, HP, bar, status icon — and the HP it
shows beside the HP it is counting towards. A double battle draws both Pokémon of a side in one
panel, a group each, so a side is read a gauge at a time, never a panel at a time (spike 150). Sun's is `btl::BtlvGaugeBase` alone, holding its panel and its side
outright, with the same record and the same two HPs (spike 124). The **fill** is the picture of a
gauge whose width is the HP: 48 wide when full in ORAS, 104.8 and anchored left in Sun, where it
keeps a sliver at 0 HP, so Sun's enemy share is the gauge's HP over the record's maximum instead.

**Pokémon record** — one Pokémon as the save stores it: 0xE8 bytes, encrypted and checked by a
16-bit sum, and for a party member 0x1C bytes of battle stats after it (PKHeX's format). The game
keeps its live Pokémon as `pml::pokepara::PokemonParam` objects holding one each, the wild one in a
battle included.

**Extra** — information the game does not show on screen (IVs, EVs, exact HP, hidden items, shiny
status, encounter slots). Each is separately toggleable and ships off.

## Speaking

**Speech category** — what a piece of speech is (menu, dialogue, battle, system, a key's answer,
guidance along a path), which decides whether it interrupts or queues. A key's answer
interrupts, since the key never reached the game's window for the screen reader to cut speech
on; everything else queues, menus too under `[pa3ds.ui] interrupt` (ADR-0010). While another window
is in front only pa3ds's own lines and a key's answer are said (ADR-0021).

**Speech formatting** — pronunciation applied on the way out: "Cap O" for a single capital letter,
symbol names, gender signs. Applied to speech only. Braille receives the raw string.

**Backend** — where speech goes: the player's screen reader through Prism, a dedicated voice, or
SAPI5 as fallback. Selectable, including off.

## Moving around

**Interactable** — anything worth walking to: an NPC, a ground item, a door or warp, a sign, a PC.

**Source** — what an interactable was read from, which tells it apart from the rest of its kind
from one press to the next: a move model's address, and in gen 6 the zone and index of the record
it was built from, since walking into the next zone fills the slots of the zone left with its
people (spike 131); or for a place its **kind tag** over its own number. Each kind of place numbers its own from zero (a sign by its index in the furniture table,
a warp by its index in the warp table, a use tile by its first cell, an edge by the zone it leads
to), and the tag (`field.SOURCE_KIND`) keeps sign 1 and door 1 on one map from being one name.
A record is also known by the zone whose table it is in, its **home**: on a street the records of
the map's other zones are listed too, and a name is the home's (spike 107).

**Zone** — one map as the game loads it: a town or route of the outdoors, or one floor of a
building. Identified by a number (`0xe0` is the player's bedroom, and which house is whose swaps
with the player's gender). The **zone table** holds every zone's info: its parent outdoor zone and
its place-name index.

**Place name** — what the game calls where a zone is: "Littleroot Town", "Route 101". A town and
every building in it share one, so a place changes when the player arrives somewhere, not at every
door. The index is in RAM (the low ten bits of a zone row's +0x1c) and the words are the **game
text**'s; pa3ds says it the moment the place changes, and AreaNav's words saying it a moment later
are left unsaid (spike 87). Not to be confused with a *place* in the names, which is a slot on one
map.

**Game text** — the game's own words out of the player's ROM, in their language: place names,
the protagonists' names and the buildings behind doors (`games.gen6.archives`, ADR-0008). The
ROM is found among the ones Azahar has opened, by the running game's title id; only a decrypted
one can be read, and without one pa3ds says what it said before.

**Matrix** — the grid of zones the player walks across without a warp: Littleroot, Route 101,
Oldale and Route 103 are one, in blocks of 10 by 10 cells. The **field map** is the matrix in
RAM, and where the player's zone is read from. A building's floor is a matrix of its own.

**Zone data** — the zone's data file as the ROM stores it, decompressed whole into the heap
(magic `ZO`). Its **entity table** lists the zone's furniture, NPCs, warps and triggers with their
cells. The source of interactables and their coordinates, and it holds people who are not standing
there yet — Wally has a record in Petalburg City long before he walks into the Gym. A stale one
lingers after a zone change; the live one is the one the move models in use agree with.

**Area** — Sun's word, in pa3ds, for the zones the game loads together, as the player's house's
four rooms are: one area's events in the ROM are eleven files of `a/0/8/2`, the people, warps
and things read by facing them of each of its zones, and the field keeps the number of the area
it has loaded. Sun has no zone data in the memory pa3ds may read; its **actors** stand in for
the entity table (spike 113). The game builds actors for the zone the player is in only, and an
outdoor area's zones are one ground, Route 1's two with a line between them nobody can see:
pa3ds lists the whole area, the ROM's records standing in for zones without actors (sun_7.ctm).
Indoors an area's zones are rooms, the player's house's four laid out far apart on one set of
coordinates, and only the player's own zone is listed (2026-09-27).

**Actor** — one of Sun's objects for a thing the field has placed: a warp
(`Field::FieldMapJumpActor`), a thing read by facing it (`FieldBgEventActor`), a ball still on
the ground (`FieldItemActor`), a person's (`FieldMoveModelActor`). What each is, a door or a
signpost, is its record's type in the ROM (spike 115). An actor holds its record's place and exists only while its zone is
loaded, so the actors are what is on the map now. Sun has no cells: a **cell** there is 50 of
its world units, and its walls are meshes, the **map blocks**'.

**Map block** — Sun's ground as the game walks it: a square of 4,200 world units, 84 cells a
side, whose floor and walls are triangle meshes in the ROM (`a/0/8/6`). An area's **block
table** lists the blocks the game keeps loaded while the player is in the area, its own and its
neighbours', so pa3ds reads an area's ground whole from the ROM, where ORAS's chunks come from
RAM (`games.sm.walls`, spike 114).

**Floor** — one height a Sun cell can be stood at: the range of heights of the meshes' floor
under its points. A porch's deck over the ground is one floor, the ground under it covered; a
town's edge and the route below it, side by side in one cell, are two. Two cells' floors are
walked between where they come within 50 units (spike 120). An ORAS chunk with a **deck** over
its ground — Mt. Chimney's catwalk over the crater, a bridge over a path — keeps a floor a cell
table and a mesh: the ground's, and one more for each deck. P walks each cell on the open floor
nearest the player's height, and two floors join where they come within a cell's height, 18
units; where they do not, the cell further from the player's is closed (spike 156). Where that
finds no way, P walks the decks at each other floor's height on the matrix in turn, the ground's
**decks**: Route 119's way north crosses a bridge high over the path below it (spike 159).

**Pocket** — open floors of one area joined side to side, all of it walked to from any of it.
Floor runs on under a bed, and under a house seen from outdoors, so their middles are pockets
of their own; the ground P walks in Sun closes every pocket but the player's, chosen by the
player's height. A **seam** is two cells side by side whose floors do not join, as along a drop
with no wall: where both are in the player's pockets, P's ground closes the one further from
the player's height, as ORAS's walks round a ledge.

**Prop** — something standing still on Sun's field with a body of its own: a flower pot, a
pillar. Its collision is a `StaticActor` holding a box or a cylinder, in the small allocations,
and it is wider than the record its thing is read by, so a way to a thing ends outside the prop,
up to three cells out (spike 118). Only the collision's bodies' group is solid: its other groups
are the ranges people are talked to from, warps' lines, the camera's boxes and event triggers.

**Cell** — a grid square of the map, (x, z) with x east and z south. Coordinates are cells;
the game's world units are 18 to a cell, a cell's centre at `cell × 18 + 9`.

**Move model** — `field::mmodel::MoveModel`, a person or thing on the map, built from an NPC
record and holding the cell it is in now, and how many cells it stands in: one for a person,
two by two for a big boulder, a barrier's three across the way it faces. The player is the one
with entity index `0xff`.

**Entity list** — the pool of 32 move models, 0xab0 apart, which the game fills as a zone loads
and empties as it leaves: what is *standing* on the map now, as against the zone data's whole cast.

**Character** — which person a move model is, as the game counts them, at +0x10: Norman `0x13f`
wherever he stands, Professor Birch `0x13e`, Mom `0x118`, and the player `0xac` as the boy and
`0xab` as the girl. The ROM lists them all by the game's own ids (`MAMA_RS`, `LEADER5A_RS`), which
name them for pa3ds but are not words for a player. One character can be two people — `0x7a` is
a **work object** a script fills, May to a boy and Brendan to a girl — and one can be shared by
everybody nameless, which is why a shared one is named for what it is ("Nurse") rather than who.
There are sixteen work objects (`0x7a` to `0x89`); `+0x10` keeps the work object's id and
`+0x898` who it was filled with, so a work object is read as the character drawn — the rival as
`0x13a` May or `0x13b` Brendan — and a name given to them is theirs, not the slot's (spike 89).
The game's own word for a character comes from its **game text**: the person, for the story's
people by a table of ours from sprite to trainer, or the kind of trainer everyone drawn as it is
("Youngster"), read off the trainers the maps stand as it (spike 90).

**Name of our own** — a word pa3ds has for something the game never names: a character, or a place
(a slot on a map). Shipped ones live beside the game's code, a file a language; the player's live
in the **player's folder**, win over the shipped, and are shared by sending the file, which opens
by saying which game and language it is for. Where the game text has a word for a character after
all — the rival — **the game's own word** comes between the two: the player's name, then the
game's, then the shipped one.

**Player's folder** — `<pa3ds>/user`, beside pa3ds itself and never in `%APPDATA%` or Program
Files (ADR-0032): the keys file, the names and the log, so the whole thing copies to another
machine with the emulator beside it. Everything in it a person reads or writes is TOML
(ADR-0031). `PA3DS_HOME` moves it, which is how a test and an agent's session stay off the files
somebody is really playing with.

**Game title** — what the emulator says it is running, read from its window title, which Azahar
took from the ROM's own SMDH (`pa3ds.core.window`). Wanted in exactly one place: the comment at
the top of a player's names file, so a file they send somebody says what it is for. Never spoken,
and pa3ds keeps no table of game names — a ROM nothing names is called what the console calls it,
`sango-1`. Never a
replacement for anything the game says (`pa3ds.core.names`). Which of the two a thing gets is the
thing's own business: a **move model** is a character, so naming it names that person everywhere
and in dialogue too, and everything else is a place, a slot on one map.

**Rename** — giving something a name of our own while playing, in a **dialog**: a native Win32
box on a thread of its own, so the game keeps being read while it is open, with the answer taken
at a later poll. One key renames what the navigation keys are on, another whoever last spoke. The
new name is said back, because a box closing is not evidence to a blind player that anything was
saved.

**Speaker** — the character showing the message a window is displaying, from the talk object the
field hands it to. Said when it changes. It survives a map change and a save, and in ORAS and X
**nothing in the game turns it into a name** (spikes 52, 73, 75, 76 and 77). Sun names its
speakers itself, on a plate above the box: the **talk window** (`App::Tool::TalkWindow`) holds
the message and the name on the plate, and a message window is paired with the talk window
holding the same words (spike 111). The talk window is built after its first page has begun, so
a first page with no name yet waits a fifth of a second for one (`announce.SPEAKER_POLLS`).

**Warp** — an entity that sends the player to a cell of another zone: a door, the mat out of a
building, stairs, or the path between two matrices. An **edge** is where a zone's blocks touch
the next zone's on one matrix, walked across without a warp: listed as each run of cells along
that border that is open on both sides in the chunks loaded, and as the whole border while no
chunk of either side is. A door in from outdoors is called what the game calls the building
behind it where the ROM's own tables pair the two — the Pokémon Center, the Poké Mart — or what
its **plate** says, the sign beside it the zone's script shows in a plate's style ("Café
Soleil"; a plate of several lines holding a facility its town lists, by that facility alone,
since ORAS's put a slogan on a line of its own), a Gym for its town's Town Map, and "door"
where none of them says (spikes 86, 107, 109).
An exit into another **place** says it after its name, "edge to Route 101"; a door into one of
the town's own houses leads nowhere new and does not (spike 93). An edge onto another zone of
the player's own matrix says the place too, though it is the player's own: Route 119, Route 120
and Route 123's hill are each more than one zone, and their edges were said bare (spike 159). Lumiose's zones are all one
place, so a warp between two of them says the street it comes out on instead, by the game's
**street marks** ("warp to Vernal Avenue", spike 107).

**Use tile** — a cell the player faces and presses A at, which the game runs a script for with
no entity record behind it: the PC, and the trash can, shelves and the like. Found by its kind in
the cell table, from the field's own table of such kinds (spike 83), and listed under **signs**,
one thing a run of cells of one kind. The PC is called what its own lines in the game text call
it — "PC", "パソコン" (spike 93); the rest are "object" until each has been seen, since the game
text has only what each of them says.

**Bearing** — how far something is from the player in cells, said as direction and distance:
"3 east 2 north". North is the way up on the pad walks: the world's north in ORAS, whose camera
always looks north, and in Sun the quarter the camera looks nearest, which on a porch can be the
world's west, kept until the camera looks ten degrees past the quarter's edge so a camera swinging
about it does not swing the compass (`games.sm.camera`, spike 119).

**Pathfinding** — the way on foot from the player to whatever the navigation keys are on, round what
cannot be walked, found on **P** in either navigation mode (`pa3ds.core.pathfind`, the core's
search; `common.field.Ground`, the game's cells). Said as its **legs**, each a straight run, with
commas between them where a bearing has none: "5 east, 2 north". A thing solid all over — a door, a
sign, a person, a PC — is walked up to, to an open cell beside it, facing it, or across a
**counter** from it, the one kind of cell the game talks across (the Nurse, a Poké Mart's Clerk,
spike 151); a thing with a cell
the player can stand in — an edge's gap, the mat out — is walked onto. The **ground** it is found on
is the cells the chunks leave open, less those a move model stands in, with warps and the
hidden Pokémon walked round unless one is the goal: the loaded chunks, and for the rest of the
matrix the ROM's, by the file the field map lists for each (spike 136). With the ROM's chunks
the search may go anywhere on the matrix; without them, only round the loaded chunks, since a
cell no chunk holds is taken for open (spike 142). P again on the thing being guided to keeps the
way held where it finds no new one, and says what is left of it. Where no way reaches the thing,
P goes as near as it can, "8 north, then can't continue", and arriving there says "can't
continue" before the thing and where it is; "no path" is for nowhere nearer (spike 159). Where
somebody stands in the one way, as Fortree's invisible Kecleon does in the Gym's, it goes up to
them and says them: "then can't continue, person in the way" (spike 160). A **drop**, two open
cells side by side more than two cells' height apart, is not walked between, but for a
**ladder**, two one-cell triggers a drop apart, its top on a platform and its foot on the ground
below: Fortree City's platforms in the trees are in the ground's own cell table (spike 160). A **ledge** is jumped down the way it drops and never climbed, a **slope** walked down
and never up but on the **Mach Bike**, which the player's move model says they are riding (spike
A), and a **conveyor belt** ridden the way it runs and never walked against: a
trigger over plain floor, known a game at a time by its script, as Dewford Gym's three are
(spike 132). A turn costs as much as a step. **Water** is a wall to P, unless the player is on it
already; **Shift and P** finds the way **with HMs**, across water too, going onto it costing five
steps more than a step, and says each leg on the water with the game's word for the move:
"3 south, Surf 33 south" (spike 136). A **bush**, the thing Cut clears, and a **cracked boulder**,
the rock Rock Smash breaks, stand in the way of P as anything standing does, and Shift and P goes
through them at the same five steps' cost, "Cut 1 east, 2 east", the three of Rusturf Tunnel's
rocks with a script of their own too, the only way through it to Verdanturf Town (spike 137). A
**big boulder**, the one Strength pushes, is the same, "Strength 2 west", and stands in four
cells, as its move model says (Fiery Path, 2026-10-01). Shift and P rides up a slope too, saying
the leg with the game's word for the bike where the player is not on it: "Mach Bike 6 north"
(spike 158).
Not to be confused with a *path*, the warp between two outdoor matrices.

**Guiding** — following the player along the way P found: the next leg said the moment they
reach each turn ("2 north") or the water ("Surf 33 south"), and the thing itself when they
arrive ("door, 1 north"). A player
who leaves the way is given a new one from where they stand, said when it turns them and taken
quietly when it goes on the way they are walking. A warp ends it without a word.

**Hidden Pokémon** — the DexNav's find: a wild Pokémon hiding in the tall grass with its tail
shaking above it, to be sneaked up on. Not a move model: the game keeps one **hidden record** for
it, without a class of its own, holding the species, the level and the world position, and
zeroed while nothing is hiding but kept across a zone change (`games.oras.hidden`). The player's
work (`field::PlayerWork`, one all session) holds its address, which is how it is found. Listed in
its own zone under the **pokémon** category as "hidden Pokémon", since the game shows a tail and
never names it.

**Secret Base** — a room the player makes at one of 85 **spots** with Secret Power: a mark on a
wall, a tree or a bush. No zone's data has a spot; the executable lists all 85 by number, and the
save keeps the number of the player's own (`games.oras.secret_base`, spike 154). The player's own
spot is listed under **exits** as "entrance to Secret Base", the place the game calls each of
the 52 zones a base is built in, and every other as "secret base spot", walked up to and faced as
a sign is. Inside, a trigger on the cell the player arrives in is the **way out**, walked onto
going south, and the PC and its **flag** stand side by side. The six people of the player's
**secret team** are kept in the walls of the far corner while their places are empty, and a move
model walled in on every side is not listed. A **decoration** is what the player sets in their base,
called by the game's own word for it; the base's **mapper** says which stands in each cell, which
the cell table never does, and P walks round them but for the mats. The **decorating cursor** is
the square the player moves over the floor to set one down: each cell it goes to is said with what
is there (a decoration, the PC, the flag, a wall, or "blank") and where it is from the player.

**Berry spot** — a patch of soil a Berry is planted in, and the tree that grows there. Not a move
model and in no zone's data: a blocked cell like a tree's, which the game knows from a table of
its field DLL, 90 of them in ORAS (spike 141). Listed in its own zone under the **berries**
category as "berry spot", since the game says "It's soft soil." or names the Berry growing, but
never names the spot.

**Ash pile** — a grey mound of volcanic ash on Route 113, which the Soot Sack collects from as
the player walks into it. In no zone's data and no move model: the route's own module
(`DllUSR113.cro`) writes its seven into a list, and the save's gimmick work keeps a byte a pile,
3 the moment it is walked into and 1 once collected (spike 155). Listed under the **ash**
category as "ash pile" until then, since the game says the grams gathered but never names the
pile.

**Bike rail** — a line of cells only the Acro Bike crosses, hopping from rail to rail: walled
to the player, two kinds of cell (`0x6d`, `0x6e`), on Route 119 and in the Safari Zone. Not
Lumiose's **rail**. A **bumpy slope** (`0x6f`) is a rocky step only the Acro Bike hops up, standing
in a row of ledges where a ledge would be, as Jagged Pass's 24 do. Listed with the Mach Bike's
**slopes** under the **bike** category, a slope as each patch of one kind, a bumpy slope a cell at
a time, and a bike rail at each of its ends, where the player gets on, as "slope", "bumpy slope"
and "rail", since the game names none of them; read off the whole matrix from the ROM (spike 159).
A bumpy slope is hopped as a ledge is jumped but up, towards the higher of the two open cells
beside it, by P on the bike and Shift and P on foot: "Acro Bike 2 north". The player's model is
the same on either bike, so the leg says which.

**Rotating door** — one of Fortree Gym's seven wooden doors: arms out from a **pivot** where four
cells meet, each on a line between cells, one or two cells long. An arm stops a step across its
line; a step into it turns the door a quarter the way the step goes and is taken, where every
cell the arms sweep through is open. The Gym's module keeps each door's pattern of arms and how
far it has turned, and P searches the doors' turns with the cells, saying a step into an arm as
"push 1 north" (`games.oras.doors`, spike 161). The invisible models the game sets round a door for
its arms are its **stand-ins**, and nobody.

**Barrier** — one of Mauville Gym's electric beams, running between two posts four cells apart.
Its move model stands at the beam's middle, drawn as `STOPPER3`, a sprite the game stands as an
invisible blocker of one cell everywhere else, so a barrier is known by its zone and sprite
(`oras.BARRIERS`). It is listed under **signs** as "barrier", three cells across the way its
model faces, and P's ground is shut in all three. A **switch**, a floor trigger listed under
signs, turns two barriers a quarter about the post between them; a way elsewhere keeps off the
switches. The gym is two rooms with no floor between them, joined by a trigger at the end of
each, listed under **exits** as a **passage** (spike 146).

**Trapdoor** — an opening in the floor of Lavaridge Gym's entrance room, which drops the player
into the hot spring below; a **geyser** is under a raft in that hot spring, and shoots it back up
through the trapdoor over it. Twelve of each, every one a one-cell trigger the game names
nowhere, so both words are pa3ds's. Listed under **exits**, walked onto when one is the goal and
walked round otherwise (spike 157).

**Dowsing Machine** — the key item the player holds out on the field to find hidden items, drawn
in their hands, not on the bottom screen. Its two **lights** come on only while the player faces a
hidden item near enough, coloured by how near: blue, green, yellow, and red with the item in the
cell in front. The game keeps what they show in a record without a class of its own, reached from
the field's process through the field's work (`games.gen6.dowsing`, spike 145). pa3ds says the
colour once it holds, "dowsing green", and "dowsing nothing" as the lights go off; never where the
item is, which the game does not show (an **Extra**).

**Navigation keys** — the keys that walk the map's list, answered only while the emulator's
window is in front and kept from reaching the game. Their answer interrupts. They work in one of
two **navigation modes**, toggled by one key: **list mode** walks the whole list by category and
item, nearest first; **direction mode** has a key a compass point, each cycling the things in
the **half-box** on that side of the player: everything with a smaller z is north, a thing
north-east is under north and under east both, and nothing further than the **range** of 10
cells either way is in. The **category** is one setting under both, and **all** is a category of
its own, the one both start on. **P** finds the way to whatever either mode is on
(**pathfinding**), and **Shift and P** the way with HMs.

**Keys file** — `<pa3ds>\user\keys.toml` (ADR-0032), written with the defaults on the first run and
never overwritten. It holds both halves of one question, because **a key pa3ds takes is a button
the player has lost**. `[pa3ds.navigation]` is which keys pa3ds answers, with a table per
navigation mode under it for the keys that differ between them; `[azahar]` is what the emulator's
buttons and **sticks** are bound to, which pa3ds patches into Azahar's `qt-config.ini`. A key's
name may carry **modifiers** — `shift+k` — and pa3ds says at startup which of its keys are sitting
on a button (`pa3ds.core.keyfile`).

**Settings file** — `<pa3ds>\user\config.toml` (ADR-0026): every setting a player has, in
tables — `[pa3ds]`, `[pa3ds.ui]` for menus, lists and screens, `[pa3ds.navigation]` for the map's
list and the way to a thing — and `[azahar]`, the emulator's own settings pa3ds wants, its RPC
server above all, patched into `qt-config.ini` only while Azahar is closed, since it saves its
own over the file as it closes (`pa3ds.core.config`). Written once, never overwritten, as the
keys file is. Only what a developer passes for one run is a flag.

**Stick** — the circle pad or the C-stick, as the emulator binds them: four directions, a
**modifier** key that moves the stick halfway while it is held, and the scale that halving uses.
The modifier is how ORAS is walked rather than run, and it belongs to one stick — both sticks
carrying the same key is not a conflict. Separate from the **D-pad**, which is four buttons and
has no halfway.

**Camera** — a cursor the player moves around the map without moving their character, with a snap
back to the character. The *light camera* reports entities and coordinates; the *full camera*
also describes terrain, and needs collision data.

**Chunk** — the outdoors as the game draws it: 40 by 40 cells, four blocks of the matrix each
way, kept as a `field::MapBlock` while it is loaded. Four are loaded at a time, the player's
and the three nearest it, so a map's far half is unknown until the player walks towards it. A chunk
is read only once its block says the load is done, since the file holds the chunk before it
until then.

**Rail** — how Lumiose City's streets place things in X, and a zone each of Coumarine City, Anistar
City, Route 7 and Parfum Palace (ORAS places nothing on one): a **line**'s id and two numbers, where
everywhere else has a cell, which the line's **way** of measuring, a way of the compass, turns into
cells along it and an offset across, rising to its left (spike 107). Each line keeps its own walls;
lines meet at points, and short **tracks** meet at **joints** where streets cross (spikes 102, 103
and 107). Every line joined to the player's is laid flat into a **corridor**, a grid with the first
line running north, a line going straight on at a point and turned the quarter nearest its way in
the world at a joint: the player, the people, the signs and the doors on the rails get its cells,
its walls are the ground, and a way is found there as anywhere else. The camera on a rail is behind
the player and the D-pad works from it: up walks the way it looks, down walks back without turning
it, and left and right turn the player a quarter, the camera following only while the key is held.
So a way there is said from the player's **heading**, the way the camera looks, with the keys that
walk it: "3 back, hold right, 29 forward"; a tap leaves the camera **angled**, between two ways, and
a hold each way straightens it (spike 105). The game loads a line only with the map chunks it runs
through, so the lines are read from the ROM for the whole map, the **streets** of each chunk the
field map lists (spike 106).

**Cell table** — section 0 of a chunk's file: one word a cell, 40 to a row from the chunk's
north-west corner. **Bit 0 is set where the player cannot walk** — a tree, a wall, a sign —
which is all the **collision** pa3ds reads so far (`games.gen6.mapblock`). **The top byte is
the cell's kind**: a Pokémon Center's counter `0xd4`, its PC `0xd5`. A **ledge** is one of
four kinds, one a way it drops: `0x74` north, `0x75` south, `0x73` west, `0x72` east (spike
126). A **slope**, the sand the Mach Bike rides up, is open but walked down only on foot, four
kinds a way it goes down, as ledges are: `0x69` east (Granite Cave 1F), `0x6a` west, `0x6b`
north and `0x6c` south (Jagged Pass), read off the ground's mesh (spike 158). **Bit 1 is set on water**, the sea `0x3d` and eight
other kinds, where the blocked bit is clear and only a player surfing goes (spike 136). Which of
the other bits is tall grass is unread, so pathfinding goes through it. The game loads the
quarter of the map round the player; the field map lists every chunk's file in the ROM's map
chunks, and pathfinding reads the rest of the matrix from there, a chunk once. A cell neither
holds is open, a guess checked again at each step as the chunks load. A chunk with a **deck**
has a cell table of the deck's own as well, and a **mesh** each, the floor's height under each
cell (**Floor**, spike 156).

## Testing

**Movie** — an Azahar input recording (.ctm). Replays deterministically, including RNG, so any game
state can be reached again without the user playing.

**Pins** — what the emulator charged the game for each file load in one replay that followed its
movie to the end, kept beside the movie and charged again in every later replay. The host's own
time reaches the game only through those charges, so a pinned movie replays the same way on any
machine, at any speed and whatever pa3ds reads (spike 121).

**Snapshot** — captured watch regions at one moment, replayed through readers in tests.
*Raw* snapshots are complete, contain save data, and stay outside the checkout. *Scrubbed* snapshots keep only
the byte ranges a reader actually touched and are the only ones committed.
