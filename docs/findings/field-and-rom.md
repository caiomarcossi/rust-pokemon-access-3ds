# The field, and the ROM

Who is speaking, what is standing on a map, what a zone says about it, and what can be read out of
the player's own ROM.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 51 — who is speaking a field message

2026-09-16 (`spikes/51_speaker.py`, movie3 slot 4, Birch's conversation on Route 101). The user
asked for the speaker's name in dialogue: in the Petalburg Gym scene Wally and Norman talk back and
forth and nothing says which is which. ORAS draws no name plate, so if a speaker is anywhere it is
on the actor. The spike tracked every `field::mmodel::MoveModel`, `field::script::*` and
`app::tool::TalkWindow` object and reported the words of each that changed, with the runtime's own
dialogue pages in the same stream.

- **`app::tool::TalkWindow` names the speaker.** +0x1c is the `gfl::str::StrWin` whose pages are
  being shown, and +0x78 is the `field::mmodel::MoveModel` showing them. Both were written the
  moment each of Birch's two messages began (116.00 s and 120.90 s) and cleared the moment each
  ended (119.21 s and 125.72 s), and +0x1c held exactly `0x082882d8`, the window the dialogue
  reader announced those pages from. `app::tool::TalkWindowGra` is embedded at +0x20 and mirrors
  the same two fields at its own +0x58. So the message and the actor are tied together by the game
  itself; nothing has to be inferred from who is animating.
- **The talk object cannot be found by its class while a talk runs.** Its vtable is
  `app::tool::TalkWindow`'s (`0x005dce94`) between talks and `0x007fa200` during one — a subclass
  in a DLL. Spike 9 measured the executable as `0x00100000`-`0x00728000`; `code.bin` is all zeros
  past that, and the runtime logged its code region as `0x00100000`-`0x00818000` in this same
  replay, so **the readable code span grows when a DLL loads** and pa3ds reads it once at startup.
  A reader must keep the address once found, as the choice box reader already does (spike 26), or
  the object disappears exactly when it matters.
- Six talk objects sit in a pool, 0xf4 apart, at `0x08287a6c` onwards; one was in use. +0x60 and
  +0x18 count pages and states as the message advances, and +0x88/+0x8c/+0x94/+0x98 are floats
  that drift while a page is on screen — the box following the speaker.
- Three `field::script::RegularObject` are live on the map, and the first runs a script: +0x1c
  steps like a program counter (`0x1c64` -> `0x1c90` -> `0x1cbc`, then `0x28` <-> `0x634` across
  each page turn) with +0x20 and +0x2c moving with it. Not needed for the speaker, but this is
  where a field script can be read live if it is ever wanted.

## Spike 52 — what names the speaking actor

2026-09-16 (`spikes/52_speaker_name.py`, the same state). It dumped the speaking model whole at
each talk and followed every word of it that points into a heap, naming what it found by RTTI and
reading strings out of it. **No name is there.** Birch's model `0x08d1e9a8` and the model beside
it, `0x08d1f458`, differ only in small paired 16-bit fields:

    +0x04  0x00090003 / 0x08010003     +0x0c  0x00170005 / 0x0017000b   (0x17 both, the map)
    +0x10  0x000000b3 / 0x00000119     +0x14  0x025a0000 / 0x02260000
    +0x18  0x000307d0 / 0x000007d0     +0x50  0xffff0000 / 0x00170000

The rest is a transform: scale, rotation and two positions, with the same pair repeated at +0x90
and +0xc0. The only pointers out are to `nn::fs::CTR::MPCore::detail::RomFsArchive::File`, and only
on the *other* model. So an actor is an id — 0xb3 against 0x119 at +0x10, 602 against 550 at
+0x14's top half — and the game holds no English word for it anywhere near the model.

**That leaves the naming open, and it runs into a hard constraint.** Nothing is hardcoded in
pa3ds, so a name has to come from the ROM or RAM. Whether ORAS has a text archive that maps one of
these ids to "Prof. Birch" is unknown and needs the static-data extractor, which does not exist
yet.

**Deferred to a two-speaker scene** (the user, 2026-09-16). Every movie reaches at most one talking
NPC, and Birch alone cannot show what distinguishes one speaker from another. The next step is a
recording of the Wally and Norman conversation in Petalburg Gym, where two named characters talk
back and forth, read the same way. If that still shows no name, the fallback is the user's to pick:
build the extractor and look for an id-to-name archive, or hardcode a table and reopen the "no
hardcoded game text" constraint.

## Spike 73 — who is speaking, with three characters in one conversation

2026-09-21 (`spikes/76_two_speakers.py`, wally.ctm: Petalburg City, the Gym, Wally's Ralts on Route
102, and back). Spikes 51 and 52 read Birch, who talks alone, and were deferred for want of a scene
where named characters talk back and forth.

- **The speaker is at `app::tool::TalkWindowGra` +0x58, and the message at +0x54.** Six talk
  objects sit in a pool 0xf4 apart, and a sweep finds them by the Gra's vtable, not the
  TalkWindow's, so spike 51's +0x78 of a TalkWindow is +0x58 from where a sweep lands. The offset
  was searched for rather than assumed: every word of the object's head that points at a
  `field::mmodel::MoveModel` is reported with its offset, and only one ever did.
- **It attributes a whole conversation correctly.** Every line of the Petalburg scene landed on
  the right character — "Not bad... It was a good battle." and "Why, if it isn't wee!" on one
  model, "Th-thank you, sir!" and "Thank you for the battle, Mr. Norman!" on another, and Wally's
  lines on a third. The actor is written as each message begins and cleared between messages.
- **`MoveModel` +0x10 is the character, and it is stable.** Norman `0x13f` outside the Gym and
  again inside it, in a different model object; Wally `0xb4` in the Gym, again on Route 102 in
  another object, and again back in the Gym; the trainer `0xdd`; the Pokémon Center's nurse
  `0x12a`, the same value she had in a run on another save entirely. +0x0c is the map and the slot
  in it — `0x000d0006` for Norman in Petalburg City, `0x01af0001` inside the Gym, `0x00180002` for
  Wally on Route 102 — and +0x04, +0x14 and +0x18 all change with the map, so +0x10 is the one
  word that says who.
- **No name is near the actor, again.** Sweeping for every word that holds a speaking model found
  7 to 10 of them: the talk object itself, two or three transform records, the model's own list
  nodes, and a string object — whose text is the message being spoken ("Not bad...",
  "Th-thank you, sir!"), not a name. Nothing readable sits within 0x200 of any of them. This is
  spike 52's finding from the other side: the game holds no word for its characters where they
  are.
- **So the naming still needs a table, and spike 74 says the ROM has the words.** What is missing
  is the index: `0x13f` and `0xb4` are not line numbers in the name lists (line 319 of the trainer
  names is "Violet", line 180 "Grunt"). The likeliest read is that +0x10 is the overworld sprite,
  which would make it a stable handle for a story character and useless for a generic one. The
  next thing to try is the zone's own table of NPCs: +0x0c gives the map and the slot, so a zone
  entry for (`0x1af`, 1) would say what the game knows Norman by.

## Spike 74 — the game's own text, out of the user's ROM

2026-09-21 (`spikes/77_rom_text.py`). The architecture has always called for a headless extractor
(now ADR-0008) and spike 52 left the speaker's name waiting for one. **It works, in
about 200 lines of Python and with no keys.**

- **The ROM is decrypted.** `CTR-P-ECRA`, crypto method 0 with the no-crypto flag set, so every
  partition reads as it lies. An encrypted dump would need keys pa3ds will not ship; the extractor
  says so and stops.
- **RomFS.** The level 3 file system is at the IVFC header plus `align(0x60 + master hash, block)`
  — `0x1000` here. The level offsets in the IVFC header describe the hash tree and are not where
  the files are, which cost an hour. 655 files: the `.cro` DLLs and the 299 GARC archives under
  `a/`.
- **GARC.** Version 4 on this ROM: the data offset is at +0x10 and the file size at +0x14 (version
  6 has a 0x24-byte header and puts them at +0x18 and +0x1c). Then FATO's offsets into FATB, and
  each FATB entry a bitmask with (start, end, length) for each bit set.
- **Text.** A line is UTF-16 XORed with a key stream: the first line's key is `0x7c89`, each line
  after adds `0x2983`, and inside a line the key rotates left three bits a character. Checked
  against the Pokédex categories, which read "Seed Pokémon", "Lizard Pokémon" and so on.
- **The names are there.** `a/0/7/1` and `a/0/7/2` are Japanese, `a/0/7/3` English, then French,
  Italian, German, Spanish and Korean — the extractor is language-agnostic for free. File 22 of
  the English archive is the trainer name list, 950 lines: "Wally" at 518, "Norman" at 570,
  "Wattson", "Flannery", "Winona", "Wallace" around them.
- **What is still missing is the link.** A trainer name is indexed by trainer id, and a field actor
  is a `MoveModel` with ids of its own (+0x10 `0x12a` for the nurse, `0xe7` for the PC NPC). Whether
  a story character's model id reaches one of these lists, through the zone's own NPC table or
  otherwise, is the next thing to find out — and it wants the Gym scene.

## Spike 75 — the overworld's own array of models

2026-09-21 (`spikes/78_entity_table.py`, wally.ctm: Petalburg City while the player walks up to
Norman, then the Gym). Looking for the table a zone keeps its NPCs in, for a speaker's name and for
the interactables the field camera will want.

- **There is no table of pointers. The models *are* the table.** Sweeping for
  `field::mmodel::MoveModel` finds **32 of them, 0xab0 apart**, at every moment in every map: a
  fixed pool the game fills and empties as maps load. The words that hold one are its own list
  nodes, transforms, a talk object and a string — never an array of entries, which is why spike
  73's holder sweep turned up nothing table-shaped. An anchor for it needs no address: the class
  is the anchor, as everywhere else in pa3ds.
- **A slot in use says which map, which slot on it, and which character.** +0x0c is
  `(map << 16) | slot`, +0x10 the character. In Petalburg City (`0x000d`) slots 0, 2, 3, 4, 5, 6
  and 9 were filled, with Norman `0x13f` in slot 6 and the trainer `0xdd` in slot 9; in the Gym
  (`0x01af`) only Norman in slot 1, Wally `0xb4` in slot 2 and the player. The player is slot
  `0xff` of the map he is on, with `0x100ac` at +0x10. An empty slot reads zero at both.
- **The place is at +0x94, +0x98 and +0x9c**, three floats in the map's own units, copied again
  at +0xc0. Walking into Petalburg the player moved from (105, 0, 135) to (97, 0, 135); Norman
  stood still at (98, 0, 123) with the trainer he had just battled three tiles from him at
  (98, 0, 126), and the other five townsfolk each at a place of their own. +0xe4 and +0xec hold
  the same place in another unit, about eighteen times bigger, which is probably the world's.
- **Still no name.** Nothing in the 0x100 bytes of a slot is a string or an index into one: the
  character id is all there is. So the name has to come from the zone's static data, which is the
  ROM's, and spike 74 can read it.
- **What this is worth beyond the speaker.** Every NPC on the map, with its character, its slot
  and its place, from one sweep of one class — the entity list the domain model has been promising
  for interactables, and the same reading a light camera would report.

## Spike 76 — the zone's own list of what is on a map

2026-09-21 (`spikes/78_entity_table.py --at 27`, wally.ctm in Petalburg City). Spike 75 found the
models but no name, so this went after the list the models are built from, looking for the name and
for the interactables a camera will want. **Found, read, and it has no name in it either.**

- **Where it is.** Searching the heap for several of the map's characters within 0x200 bytes of
  each other lands on it at once: the characters sit **0x30 bytes apart, in slot order**, so entry
  N is slot N — Norman at base + 6 × 0x30 and the trainer at + 9 × 0x30, exactly the slots their
  models carry at +0x0c. Nothing points at the list from a model, and it has no class of its own,
  so an anchor for it is still wanted; the models themselves carry everything a camera needs.
- **An entry, as this spike read it** — and it read it two bytes late, which spike 78 puts right
  a few hours later. Aligning the entries on the character put the boundary in the middle of the
  record: what looked like "index + 1" at the end of one entry is the *next* entry's index at its
  start, and the script and the flag sit two bytes further in than they seemed. The record is
  `u16 index, character, flag, 0, script, work, ...` with the cell near the end
  (spike 78, `games.gen6.field`); what this spike is still good for is that the character is at a
  fixed place in a record of 0x30, and the records are in index order.

- **It lists NPCs that are not on the map yet.** Wally's sprite `0xb4` has an entry in Petalburg
  City, at (88, 127), long before he walks into the Gym; the model pool holds nothing for him
  until he does. So the list is the map's cast, not what is standing there now — which is what a
  camera wants to know the difference between.
- **No name, and the script is not an index into one.** Norman's script is `0x306`, and line
  0x306 of the English trainer names is "Timin"; in the other archives of that file it is a
  weight, a height and a Pokémon. Checked across all 175 files. The entry holds a character, a
  script, a flag and a place, and nothing else.

**So ORAS does not know its characters by name, and neither can pa3ds from the field.** The names
are in the ROM (spike 74) in lists of their own — the Gym Leaders, the story characters, the
trainer names — and nothing in the map data or the models reaches them. What is left is the user's
to choose: name a handful of story characters from a table pa3ds carries, which reopens the "no
hardcoded game text" constraint; or leave speakers unnamed and say only that the speaker changed,
which he has already declined; or look next at the trainer data, on the chance that a trainer entry
carries the overworld sprite its character walks around as, which would name the Gym Leaders and
the rivals without a table.

## Spike 77 — the trainer table, and whether it reaches an overworld sprite

2026-09-21 (`spikes/77_rom_text.py` against the user's ROM). The user's reading, and a fair one:
the people who wrote the scripts had to know who was who, so the link must be somewhere. It is —
but not between the field and a name.

- **The trainer table is `a/0/3/6`: 950 records of 24 bytes, one per trainer, and the trainer
  name list (file 22 of the English text) is indexed by the same number.** Record +0x02 is the
  trainer class, indexing file 21, 280 classes long. Trainer 570 — "Norman" in the name list — has
  class 204, "Leader". So a trainer id names a trainer and says what to call them, in every
  language the ROM carries.
- **The record holds no overworld sprite.** Norman's 24 bytes are
  `01 00 cc 00 00 00 00 03 19 00 00 00 00 00 00 00 07 00 00 00 00 1e 00 00`: flags, class, items,
  AI and party, and `0x13f` appears nowhere in it. Searching every archive for the sprite within
  0x20 bytes of Norman's name index, his Gym Leader index or his script id turns up only floats
  and compressed data — `3f 01` is the tail of `0x3f800000`, which is everywhere.
- **So the link runs through the script, not through the data.** A zone entry gives a script
  (spike 76); the script is what starts a trainer battle with a trainer id, and what tells an NPC
  which message to speak. The scripts are in the ROM, LZ11-compressed (`a/0/8/8` and `a/0/9/1`
  begin `11 ..`), and reading them means decompressing and parsing the bytecode — a job of its
  own, whose reward is a name for characters the player *battles* and nothing for the rest.
- **What this does unlock: battles.** A trainer id in RAM during a battle would give "Leader
  Norman" properly, in the player's language, out of the ROM. Worth its own spike when trainer
  battles are read.

## Spike 78 — the map's interactables and the player's cell

2026-09-21 (`spikes/79_field_entities.py`, interactable_objects.ctm: CONTINUE outside the player's
house in Littleroot Town, in through the door, up the stairs, and the bedroom's notebook, clock and
trash can read). The user asked for a list of everything on the map worth walking to, by category,
with the player's coordinates, towards finding a path to each. The runtime says nothing here beyond
the three messages; this spike is what RAM holds.

- **The zone's data file is in the heap whole.** A `field::*` census (540 classes named, most
  with no live instance) found no class for an entity table, because it is not an object: it is
  the zone's data file as the ROM stores it, decompressed into the heap, magic `ZO`, a u16 count
  of 5 sections and six u32 offsets, the first always `0x1c`. Section 0 is the zone's info, 0x38
  bytes; section 1 is its entities: a u32 size, then one u8 count each of furniture, NPCs, warps
  and triggers, a u32 zero, then the records in that order, 0x14, 0x30, 0x18 and 0x18 bytes
  each, followed by a u32 size and the zone's script. The size was `8 + counts × sizes` for all
  three zones seen (the town, the house's ground floor, the bedroom), which is the validator. A
  two-word holder at `0x08d0b930` keeps the current file first and the previous zone's second,
  with the current one's LZ11 original after them; the bedroom's file sat at `0x08d06ed8` with the
  ground floor's still whole at `0x08d02480`, so a stale file is as findable as the live one, and
  the live one is the one the models in use agree with (below).
- **Coordinates are grid cells, and world units are 18 to a cell.** NPCs, furniture and
  triggers hold their cell as a u16 pair, x east and z south; warps hold world units, the cell's
  centre being `cell × 18 + 9`. The town's door at world (1701, 3087) is cell (94, 171), and the
  player stood at (94, 172) facing it and was inside the next moment. Outdoors the cells are the
  whole overworld's (Littleroot at 80 to 112 by 160 to 186); indoors each zone's own (the bedroom
  6 to 16 by 6 to 15). Zone 6 is the town, `0xdf` the ground floor, `0xe0` the bedroom, `0xe1`
  and `0xe3` the two other doors' houses.
- **A warp is a door or a stair, and says where it goes.** `to zone, to warp, kind, 0, x, y, z,
  width, height`, kind `0x0301` a door in from outdoors, `0x0200` the mat out, `0x0501` stairs
  up, `0x0502` stairs down; the ground floor's exit is 3 cells wide and its stairs 2. The
  bedroom's stairs are at (14, 7), height 2, and the player arrived at (14, 8). A zone's edge is
  not a warp: the town's way north to Route 101 is three triggers 3 cells wide at (99, 162) and
  (99, 164), scripts `0xa` to `0xc`, and the connection itself is somewhere in the zone info or
  the map matrix, unread.
- **Furniture is what is read by facing it, when it is an entity at all.** `script, kind, ?, 0,
  x, z, width, height, ?, 0`. The bedroom's notebook is script `0x2963` at (8, 7), read from
  (8, 8) facing north; the two signs by the doors are `0x12` at (103, 171) and `0x13` at
  (97, 171), 3 cells from each door with kind 3. **The trash can is not there**: nothing in the
  table is near (7, 15), where "The trash can is empty." was read. It must be a tile attribute
  in the map's own data, which is the collision question, so a listing from this table misses
  it and the bookshelves like it.
- **An NPC record is what a `field::mmodel::MoveModel` was built from, and the model is the
  live one.** `index, model code, flag, 0, script, ?, ?, ..., width, height, range × 4, 0 × 3, x,
  z, y as a float`. The pool holds 32 models, `+0x04` zero when free; a model in use holds
  `zone << 16 | index` at `+0x0c`, `flag << 16 | model code` at `+0x10`, `script << 16` at
  `+0x14`, its cell as floats at `+0x94` and `+0x9c`, the cell it is stepping to at `+0xc0` and
  `+0xc8`, its world position at `+0xec` and `+0xf4`, and a 3 by 3 facing matrix at `+0x1c`.
  Of the town's 11 records, 3 were in use, all three with flag `0x2f`; the rest are hidden by the
  story. One of the three had walked from (91, 175) to (109, 176), so the model's cell is the one
  to say and the record's is where it started. Spike 52's "0xb3 against 0x119" were two of these
  codes, Birch's and a townsperson's, and its "602 against 550" two scripts.
- **The player is the model with index `0xff`**, code `0xac`, and gets a new model on each zone
  change (`0x08d209b8` in town, `0x08d1ff08` downstairs, `0x08d1f458` upstairs). Its cell
  followed every step of the movie: (94, 172), then (16, 18) to (16, 9) across the ground floor,
  then (14, 8) to (8, 8) for the notebook. `field::PlayerGrid` and `field::PlayerWork` exist, one
  live each, and hold the arrival cell and the step in progress; the model is enough.
- **Nothing here is a name.** A model code says what a person looks like, a script number what
  they do, a zone id where a door goes, and the zone info holds a place-name index (`0xaa` for all
  three Littleroot zones, the same byte in each). The words for any of them are in the ROM's text
  archives, which is the static extractor, not built. Item balls were not seen: none is in
  Littleroot, so which model code an item ball uses, and where its item id sits, is still open.
- **Second recording, the same day** (interactable_objects_2.ctm: Route 101 with May's DexNav
  lesson, through Oldale onto Route 102, three trainers, a wild Ralts, and a Potion picked up;
  replayed at 300% with `timeline`, faithfully, in a third of the time). Four more findings:
  - **Walking into the next outdoor zone keeps the player's model, and its zone with it.** The
    model built in Littleroot read zone 6 all the way up Route 101 and across Oldale; only the
    warp out of Oldale built a new one, reading `0x18` (Route 102) at (198, 140). So the model's
    zone is where it was built, and the current zone is read elsewhere.
  - **The field map is a matrix of zones.** A heap block of 0x9180 bytes (tag `0x5544`, +0x08 = 4,
    +0x0c = 32) holds at +0x3c a 32 by 32 grid of u16 zone ids, one per block of 10 by 10 cells,
    0xffff where there is nothing: Littleroot in blocks 8-11 by 16-19 (cells 80-119 by 160-199),
    Route 101 above it, Oldale above that and Route 103 above that, 12 blocks wide. Route 102 is
    another matrix, Petalburg 4 blocks wide beside it, and the bedroom a matrix of one row of 16
    blocks with the old outdoor grid left underneath. (Corrected 2026-09-23, below: the grid is
    not always 32 wide, and the bedroom's is 4 by 4, read here as one row.) The zone the player is in is the grid at
    their cell, and a neighbouring zone's blocks along the current one's give the open edge:
    Route 101 is the run of cells at z = 159 above the town. Oldale to Route 102 is a warp of
    kind `0x0503` three cells tall at (80, 100), the way back `0x0502`, so the stairs kinds are
    not stairs but the way a warp is walked through, and a warp between two outdoor zones is a
    path.
  - **Every zone's info is in a table**, 536 rows of 0x38 bytes indexed by zone id, with the
    current zone's row pointed at from the word before the table's own pointer (`0x0864f754`,
    `0x0864f758`). A row's +0x1a is the parent outdoor zone (a house's floors say Littleroot's 6;
    an outdoor zone says itself) and +0x1c the place-name index: 170 for the three Littleroot
    zones, 172 Oldale, 204 Route 101, 206 Route 102, which is the location list a Pokémon's
    met-place uses, so names are one text archive away. Two zone data buffers sit behind the
    holder at `0x08d0b930`: after the wild Ralts both held Route 102, so the second is not
    reliably the previous zone, and the first is the current one.
  - **An item ball is an NPC record of model code `0x2000`** with a script of its own (`0x51a`)
    and a flag in the 7000s (`0x1b58`), at (138, 148), and its model is in use until it is picked
    up. Which item it is lives in the script, not the record: names later. Route 102's table
    also holds a fifth kind of record, six of them, shaped like a trigger (`0x40c0` flags, an
    area 7 by 4), counted by the u8 after the four counts, which the size check has to include.
- **Built** (`gen6.field`, 2026-09-21): the zone data file found by its magic and validated by
  its entity size; the holder found as the word pair pointing at it with the LZ11 tag after; the
  field map by its allocation and grid; the zone table by the current zone's info bytes and the
  pointer pair; the player as the model in use with index 0xff. `FieldReader.interactables`
  lists exits (warps named door, way out, stairs or path by kind and destination, and edges from
  the matrix), signs (furniture, and things by a model code of `0x2000` and up), people (models
  in use) and items (model `0x2000`), each with its cells and a bearing from the player, nearest
  first. Logged on each zone change and after each sweep as `field: zone ... at (x, z)` and a
  line a category, and read out under the navigation keys (Behaviour, Keys); zone numbers stay
  in the log until names come from the ROM (the user, 2026-09-21: names later). Over the five
  captures it lists the town's three doors, two signs and the way north; the bedroom's five
  things and its stairs; Route 102's Potion 5 east 2 north. The keys were pressed from the rig
  (`76_field_entities.py keys`, the hook told the emulator is in front) at the town door and
  through the house: "way out, here", "signs, sign, 8 west 1 north", "stairs down, 7 east 1
  north" as the recording walked on.
  Replayed through the runtime, both movies list each zone as it is entered, Route 101 and Oldale
  included as the edges are crossed, May as she arrives for her scene, and the Potion until it is
  picked up. Before and after over both (`regress.py`, no out-of-sync lines): nothing else said
  changed except the battle's known races, and one run's wild Ralts was the other's Wurmple, the
  encounter drift of spike 25.
- **Left open:** tile-attribute interactables (the trash can); the item in an item ball's
  script; which model codes are people (the moving truck, 4 by 2 with no script, passes for one;
  the townsperson record 3 by 2 walks, so the record's width and height are not a shape); what
  the furniture's `kind`, the trigger's flags and the fifth record mean; whether a matrix's
  block is always 10 cells.

## Spike 80 — the DexNav's hidden Pokémon, and the record the game keeps for it

2026-09-22 (`spikes/80_hidden_pokemon.py` over captures made with spike 79's commands;
sneak_catch_pokemon.ctm: CONTINUE into the Wurmple battle on Route 101, away from it and from
a Poochyena, May's DexNav lesson, and the player at (99, 130) as she says "Slowly...slooowly...",
with the tail in the grass to the west). The user asked for the "shaking tail" Pokémon in the
field's list, under a category of its own.

- **The movie ends 34.7 s in, and the fork takes no save request after that.** `timeline` at
  300% logged the lesson's pages and the player's cells, then nothing; the emulator's log says
  "Playback finished" at 34.7 s with no out-of-sync line, and the game ran on with no input.
  States were asked for every 10 s from 20 s: only slots 1 and 2 were written, the ones asked
  for before playback finished. Slot 2, saved at 30 s during "Slowly...slooowly...", boots to
  just after the movie's end, since a state boot resumes about 19 s after its save, with the tail
  still up: it stays for minutes while the player stands (screenshots to 210 s).
- **The Pokémon is not a move model.** The pool at slot 2 held May at (99, 129), two trainers
  and the player at (99, 130), and 28 free slots. The class census names the mechanism, though:
  `field::mmodel::MoveCodeKakuremino`, `EvTypeTrKakuremino`, `ActionCmdStayJumpKakuremino` and
  `EffectKakureminoBase`, one live each ("kakuremino", a hiding cloak), and all four are live in
  the middle of a battle too, so they are the handlers and not the thing.
- **Four words in the heap hold the tail's cell as world units** (1701, 2, 2331), which is cell
  (94, 129), 5 west 1 north of the player, where the screenshot has the tail; as cell floats or
  cell shorts, nothing holds it. Two are `xy_system::H3dShaderModel`s (the tail drawn; one reads
  2331.04, animating), one is a block at `0x082c8904` that holds the player's world place at
  +0x90, both shader models, and at +0x28 a pointer into the fourth: **a record at
  `0x08d374e4`, just past the model pool, with no class of its own.**
- **The record**, at the same address in seven captures over three movies:
  `+0x00` a pointer to a `gfl::heap::CTRExpHeap` of its own; `+0x04` the species as the
  national number (`0x105` Poochyena); `+0x08` the level (5); `+0x48` and `+0x98` 1 while one
  is up; `+0x8c`, `+0x90`, `+0x94` the world x, y (2.0) and z; `+0xa4` a pointer to a
  `pml::pokepara::CoreParam`, kept even while the record is empty (the captures hold one to
  eight objects with that vtable, the party's records carrying it until they are built into
  `PokemonParam`s, and only the record's has a heap pointer 0xa4 before the word holding it); `+0x9c` and `+0xa0` pointers
  set only in the sneak capture, to a (species, level) pair and to the party's six records.
  Empty, in Littleroot and in the bedroom, everything but the two pointers reads zero. The
  interactable_objects_2 captures hold the **same record with the same Poochyena at the same
  cell** during the lesson at 21 s, and at 104 s on Route 102 **a Surskit (`0x11b`), level 5,
  at (152, 139)**: a hidden Pokémon was up by the Potion that nobody noticed. Words not
  understood: `+0x20` 9 in every live one, `+0x24` 0x15 on Route 101 and 0x11 on Route 102,
  `+0x28` `0x20004004`, `+0x2c` to `+0x38` differing between the two Poochyena captures
  (perhaps the rare move and the potential), `+0x3c` a float (91.78, 457.40, 64.68).
- **Built** (`games.oras.hidden`, `gen6.field`): the sweep looks for the two classes, and the
  record is the word holding a CoreParam at +0xa4 whose +0x00 is a CTRExpHeap. Read every poll,
  since it appears and goes between sweeps: a species in 1 to 721, a level in 1 to 100 and both
  flags up, or nothing. Listed as "hidden Pokémon" under a new **pokémon** category, after
  items, with the species in the log (`field pokémon: hidden Pokémon species 261 5 west 1
  north`) and not aloud (ADR-0024). The listing is made again when one comes or goes.
  Pressed from the rig at slot 2 (`79_field_entities.py keys`): the runtime listed Route 101
  with the Pokémon under "pokémon: hidden Pokémon species 261 5 west 1 north"; `[` from all
  said "pokémon, 1" and K "hidden Pokémon, 1 of 1, 5 west 1 north"; in direction mode west
  and north both found it (it is one cell north as well), south and east said "no pokémon
  south" and "no pokémon east"; back in list mode, "all, 7".
- **The record outlives its zone.** Before and after over the four replays (`regress.py`, no
  out-of-sync lines): the main menu and the first interactables movie said the same; the second
  listed the lesson's Poochyena from (105, 130) and again from Oldale, 11 south of the edge, then
  **on arriving at Route 102 at "105 west 11 north"**: the record still held Route 101's
  Pokémon, on another matrix, until the route's own finds replaced it (a Seedot, 273, after
  Calvin's battle, then a Surskit, 283, twice more as the walk went on). So a hidden Pokémon
  is listed only where the matrix puts its cell in the player's own zone, which also leaves
  Route 101's out of Oldale's list. The other differences were the known races: a cursor item
  opened in one run and moved in the other, the level-up stats caught while revealed, the
  encounter drift of spike 25 (one run's wild Ralts the other's Wurmple), and the field lines
  a poll earlier or later, since a change in the record now makes the listing again.
- **A battle does not clear the record; a byte says the tail is down** (the same day,
  sneak_catch_pokemon_2.ctm, recorded in the user's own Azahar on a save in Petalburg City, kept
  as `user/save_checkpoint_2`: east onto Route 102, a hidden Surskit at (132, 144), its battle,
  and on). The user heard the entries stay after a battle. Replayed with the runtime, the record
  held the Surskit from 48 s, the battle ran 107 to 125 s, and nothing in it changed until a
  Zigzagoon replaced it at 140 s. Captures before the battle (slot 7) and after it (slot 10)
  differ in four words: `+0x34`, `+0xa0` the party pointer to 0, `+0xa8` 0 to 1, and
  **the byte at `+0xae` from 4 to 0xff**. (`+0x34` is a counter: the runtime's raw log showed
  it stepping 0 to 19 as the player walked through grass and starting over, reset by each find,
  so it is left out of the log line.) Over all nine captures that byte is 0xff exactly when
  no tail is showing: the empty records, the Poochyena carried into Oldale and onto Route 102
  (both screenshots without a tail), and the Surskit after its battle; and 1, 4 or 5 with a
  tail up. `+0x48` and `+0x98` stay 1 after the battle, so they say a record was set, not that
  one is up. Built: the byte is the check, `hidden.SLOT`, and the raw log line shows it. Replayed
  with the runtime: the Surskit listed from 46 s, the listing remade without it at 123 s as
  the battle's experience line was said, and the Zigzagoon listed fresh at 133 s.
- **The two classes cost the sweep 41%** (2026-09-22, on the merge into main;
  `spikes/60_poll_timing.py signs.ctm --seconds 140` run on the commit before and after). The
  heap sweep's mean went from **152.65 ms to 214.88 ms**, and its total over the movie from 10.84
  to 15.26 s across the same 71 sweeps. `find_objects` grew by 1.03 s of that, so about three
  quarters of the rest is the scan recording more objects — `gfl::heap::CTRExpHeap` is the
  suspect, since a heap is a common thing to be and every one is now tracked on every sweep, to
  check one pointer.

  What a player feels is much smaller, because a sweep is read in chunks with dialogue polled
  between them: polls went 57.6 to 57.3 a second and the median gap between them did not move
  (17.1 to 17.0 ms). The tail did: gaps over 50 ms went 36 to 56 and over 100 ms 10 to 15, which
  is how long a line can sit on screen unseen.

  **Worth trying if it is ever wanted back:** drop `CTRExpHeap` from what the heap is swept for
  and check the pointer at `+0x00` by reading its vtable instead, which is one read against a
  known address rather than a class tracked everywhere. `hidden.find` also runs on every sweep
  though the record's address was the same in seven captures; stopping once found saves the rest.
- **Left open:** what the byte at `+0xae` counts (a grass slot, an encounter slot?) and what
  the counter at `+0x34` is for; when the Pokémon flees (no movie reaches it); whether more than
  one can be up at once (the game shows one); the species' name, which is the ROM's text; the
  level, which the DexNav shows once the species is registered.

## Spike 82 — the map's chunks, and which cells can be walked

2026-09-22 (`spikes/82_map_collision.py`; interactable_objects.ctm at slot 5, the player at the
door of their house in Littleroot). The user asked that the edge north out of Littleroot be the
walkable cells and not the whole border: the listing said "edge to zone 0x17 13 north" from the
door, which is straight into the trees, since the border was the run of blocks the matrix has
Route 101 above the town, 40 cells wide.

- **The game draws the outdoors in chunks of 40 by 40 cells, four loaded at a time.** The
  class census has `field::MapBlock`, and spike 79's Littleroot capture holds six: two dead in
  memory that reads as code, four live. Beside the live `field::MapFileSimple` (`0x0828d134`) is
  a list of (index, block, 0) triples: `0x19` and `0x21` with no block, `0x1a` with `0x0824c060`
  and `0x22` with `0x08252118`. The indices are cells of an 8 by 8 grid of chunks, 40 cells each,
  so the 32 by 32 matrix of 10-cell blocks is four to a chunk each way: `0x22` is column 2 row
  4, cells 80 to 119 by 160 to 199, Littleroot; `0x1a` is the chunk above it, Route 101; `0x19`
  and `0x21` the column to the west, which the matrix has nothing in, so their blocks are empty
  (+0x18 null, +0x90 zero). With the player at (94, 172), in the north-west quarter of their
  chunk, the four are the player's and the three towards that corner.
- **A block says where its chunk is and where its file sits.** `+0x0c` and `+0x14` are the
  chunk's centre in world units, (1800, 3240) for Littleroot's, cells (100, 180), 20 in from
  the corner; `+0x18` a `gfl::fs::ArcFile`; `+0x24` a pointer into the **linear heap**
  (`0x14af3780`, and `0x148f3580` for Route 101's, the four buffers `0x100100` apart), which no
  heap capture holds; `+0x90` a size, `0x242e8` and `0x2baf8`, smaller than the file's own
  section table says the file is, so likely the packed size. The state boot lands on the same
  heap, so `dump` reads the blocks at their captured addresses. **The field is torn down at
  44 s:** the first dump read the blocks as garbage under a door fade, and the screenshot showed
  the player walking indoors; the second polls for the block's centre from 39 s and reads the
  files the moment it is there (40.3 s).
- **The file is `GR` with a section table like the zone's**, u16 7 sections and eight u32
  offsets from 0x80: sizes 0x1980, 0x5d800, 0x980, 0x100, 0x80, 0x80, 0x80 for Littleroot's, and
  a sixth section of 0x55780 bytes in Route 101's. **Section 0 is a cell table**: u16 width and
  height, both 40, then one u32 a cell, row by row from the north-west corner, 31 words of
  padding after. Drawn as a map (`analyse`), it is Littleroot: the two houses and the lab as
  blocks of `0x01000021`, the paths `0x20`, the border of trees all round, the sign beside each
  house door as a blocked cell where the entity table has it ((103, 171) and (97, 171)), the
  house door itself blocked at (94, 171) with the player on the path at (94, 172) — and the way
  north **three cells wide at x 99 to 101**, open from z 162 up through the border at 160 and
  159 into Route 101, which the town's three triggers at (99, 162) and (99, 164) sit across.
  Route 101's chunk adds `0x20004004` for its tall grass and `0x75000021` for a ledge. **Bit 0
  of the word is set where the player cannot walk**, in every cell of both chunks checked
  against the screenshots and the entity tables; what the other bits mean is not read.
- **Built** (`games.gen6.mapblock`, `gen6.field`, 2026-09-22): the sweep looks for the class;
  a block is read when the edges are listed, its chunk's cell table once per load (by the file's
  address and size, since the game reuses the four buffers). An edge's border is paired cell by
  cell across the zone line, a cell is a way across when neither side is blocked in a loaded
  chunk, and each run of them is an exit of its own, `source` the zone and the gap's number.
  A border with no chunk loaded on either side is listed whole, as it was. `replay` runs the
  field reader over the slot 5 capture with the chunk files put back at their addresses: "edge
  to zone 0x17 5 east 13 north", at (99, 159), 3 wide, and the rest of the listing unchanged.
- **The blocks' class is in the field's DLL, which the runtime had never read.** The first
  before-and-after replays said "13 north" at the door in both runs, and a longer replay swept
  three times a second across all of Route 101 without ever finding a block, while `live`, the
  runtime's own sweep and reader booted the same way, found six and listed the gap. The
  difference was when the executable was read: `find_vtables` puts `field::MapBlock`'s vtable
  at `0x007fd7c8`, and the runtime reads the code on the title screen, when it ends at
  `0x00698000` — the DLL loads with the field (spike 9 saw the span grow the same way for the
  talk window's subclass). The sweeps found every other object on the field, 265 of the 271,
  and the block's class was simply not among the names a sweep looked for. Built: a whole
  sweep first measures the code again from where the image ends and reads what has appeared
  beside it (`Discovery.learn`), so a class in a DLL is found once the game has loaded it; and
  a listing with no chunk at all asks for a whole sweep, a few times per zone
  (`FieldReader.sweep_wanted`, as the battle's gauges do), since the player's own chunk is
  always loaded. `live --slot -1` boots the movie from its first frame, as the replays do.
- **Left open:** the meaning of the other bits (a ledge is one-way; tall grass and water are
  worth saying); the six other sections (0x5d800 bytes is the terrain, probably); reading a chunk
  the game has not loaded, which pathfinding across a zone needs, and which is the ROM's map
  archive (ADR-0008); whether a building's floor has a chunk of its own (its matrix is one
  row of blocks, so a chunk of 40 would cover it).

## Spike 83 — the Pokémon Center's PC, and the cells the player uses by facing them

2026-09-23 (`spikes/83_pokemon_center_pc.py`; poke_center_pc.ctm on checkpoint 3: CONTINUE in
Petalburg City, into the Pokémon Center (zone `0x103`), up the counter and the PC booted at about
33 s). The user: the list does not name the PC; find what kind of thing it is, and make whatever
else is the same kind trackable too.

- **The PC is a cell, not a record.** The Center's entity table holds one piece of furniture (the
  bookshelf at (14, 14), two cells wide, already listed as "sign"), ten NPCs of which five are in
  use, and the way out; nothing at the PC. The player booted it from (21, 17) facing north, and
  the building's one chunk (origin (0, 0); a building's floor does have a chunk, which spike 82
  left open) has at (21, 16) the only cell of its kind: `0xd50e0001`. The counter around it is
  `0xd40e0001`, and so is the blue table on the right; the stools and sofas are `0xe2`, `0xe7`,
  `0xea`, `0xec` and `0xef`, a byte each, one or two cells a piece (screenshots at the door and at
  the counter).
- **The top byte of a cell's word is its kind, and the game says so.** The executable has one
  small function a kind, `lsr r0, r0, #0x18; cmp r0, #N; moveq r0, #1`: `0x00403c1c` tests
  `0xd5`, `0x00403004` `0xd4`, and `0x0040302c` `0xe2` or `0xfb`. Nothing calls them directly.
- **The field keeps a table of the kinds it runs a script for.** At `0x007e6680`, in the field's
  DLL, twelve pairs of (kind test, `script << 16 | argument`), a terminator after them:

  | kind | script | argument |
  |---|---|---|
  | `0xd5` | `0x07d1` | 0 |
  | `0xd7` | `0x299a` | 0, 5 and 4: three entries |
  | `0xdd`, `0xd8`, `0xd9`, `0xda`, `0xdb`, `0xdc`, `0xde`, `0xdf` | `0x09c4` to `0x09cb`, in that order | 9 |

  `0xd5` is the PC: seen booted. The rest are the same kind of thing — a cell with a script on A
  and no record — which is what spike 78's trash can was.
- **Every one of them is in the ROM's maps, a few to a building.** The chunks are archive
  `a/0/3/9`, 819 files, each LZ11-compressed `GR`. Scanned whole: `0xd5` in 11 chunks, one cell
  each (the Centers' two at (21, 16)); `0xdd` 74 cells in 58 chunks, mostly one to a chunk;
  `0xd9` 82 cells in 35 chunks and `0xda` 96 in 40, in runs of two to four along a row, which is
  a shelf's shape; `0xdb` 36 cells in 3 chunks; `0xdc` one cell in the whole game.
- **`0xdd` is very likely the trash can.** Spike 78 read "The trash can is empty." with the
  player at (7, 15) in the bedroom, and chunk 507 has exactly one use tile, `0xdd` at (6, 15),
  beside it. That chunk being the bedroom is inferred from the cells, not seen, so it is not
  named yet.
- **The messages do not say which is which.** English file 155 (`a/0/7/3`) holds the shelf and
  bin lines together — knickknacks, Pokémon books, "The trash can is empty.", a vase, a
  blueprint — fourteen lines for eight scripts, so the link is in the scripts, which pa3ds does
  not read.
- **Built** (`games.gen6.mapblock`, `gen6.field`, 2026-09-23): a chunk gives each cell's kind;
  the listing takes every cell of the loaded chunks whose kind is in the table, in the player's
  zone where the matrix says whose the cell is (it says nothing for a building's), and lists
  each run of one kind as one thing under **signs**: "PC" for `0xd5` and "object", the word for
  a prop, for the rest. A tile's `source` is its first cell under a tag of its own
  (`TILE_SOURCE`, above 32 bits since 2026-09-23), so a rename keeps a slot of its own.
  Replayed (`regress.py`, pc, out of sync 0): "signs: PC 2 east 7 north; sign 4 west 9 north"
  inside the door, "PC 1 north" at the counter; nothing else said changed. The chunk is unloaded
  while the PC's screen is up and loaded again after, as the field is.
- **Left open:** which of `0xd7` to `0xdf` is which (the scripts, or a recording at each); the
  `0xd7` table's three arguments; what the bits between the kind and the blocked bit mean.

## The field map's width, read on Route 104

2026-09-23 (hidden_tail.ctm, recorded in the user's own Azahar on `user/save_checkpoint_3`:
Petalburg City west onto Route 104, where a tail showed in the grass and the list did not have
it; replayed with the runtime and a scratch hook dumping the field map on every sweep).

- **The record was right and the zone was not.** The runtime's raw line read `species 278 level
  3 slot 0x3 at (140, 119) zone none`: a Taillow with its tail up, which `hidden` drops unless
  the matrix puts its cell in the player's zone. The grid put *no* zone there, and none at the
  player's own cell either; the player's zone was right only through the model's fallback.
- **The grid is as wide as its matrix, and the header says how wide.** There is one field map
  block (`0x08cf7dac` in every capture) and the game rewrites it for each matrix. On Route 104,
  read 32 to a row, zone `0x1b` came in runs of 8 that stepped 8 columns right every row; read
  40 to a row it is whole rectangles: `0x1b` blocks 8-15 by 8-15, `0x1c` by 16-23, `0x1d` 12
  wide by 24-27, `0x08` by 28-31, and both cells in `0x1b`. The halfwords at `+0x36` and `+0x38`
  are 10 and 10 there, 8 and 8 in every capture of Littleroot's and Petalburg's matrices (32
  blocks, as before), and 1 and 1 in the bedroom, whose grid is 4 by 4 — the "one row of 16"
  above. So the grid is `+0x36` chunks of four blocks wide and `+0x38` tall. Every matrix seen
  is square, so which of the two is the width is taken from their order, not measured. The word
  at `+0x0c`, taken for the width, is 32 in all of them, Route 104's included.
- **Built** (`gen6.field`): the grid is read at the size its header gives, and the zone lookup
  and the edges walk it at that width. Replayed before and after (`regress.py`, no out-of-sync
  lines): on Route 104 the listing now has `pokémon: hidden Pokémon species 278 3 east 7 north`,
  and the edge south into `0x1c`, which the 32-wide reading never found; the Pokémon Center
  movie lists the same inside the centre, the rest poll drift.

## The hidden Pokémon's species is a halfword

2026-09-23 (hidden_tail_3.ctm, recorded in the user's own Azahar on the save of 20:39, kept as
`user/save_checkpoint_4`: Petalburg City east onto Route 102, where a tail showed in the grass
and the list did not have it; replayed with the runtime and a scratch hook logging the record's
raw bytes).

- **The record was right, in the right zone, and read wrong.** The raw line read `species
  9896201 level 2 slot 0x5 at (135, 133) zone 0x18`. As bytes, `+0x04` is `09 01 97 00`: a
  Wurmple, 265, in the low halfword and 0x97 in the high one, so the word was off the Pokédex
  and `hidden.read` took it for no record. The user's own log of a session without a movie had
  the same thing: a Surskit (283) listed first, then two more finds reading `6160667`, which is
  0x5e above 283, and neither was listed. Every record captured before held 0 in that halfword,
  which is why the first finds were heard.
- **What the halfword at `+0x06` is was not found.** It holds from the find until the record
  changes, and the level word after it read a clean 2.
- **Built**: the species is read as a u16. Replayed before and after (`regress.py`, no
  out-of-sync lines): on Route 102 the listing now has `pokémon: hidden Pokémon species 265 8
  east 2 north`; hidden_tail.ctm says the same as before, Route 104's Taillow included.

## Spike 85 — what the ROM already names: the rival, the zones, and what is in a town

2026-09-23 (`spikes/85_rom_names.py`, against the user's own ROM; no emulator). The user asked two
questions: can the rival be called what the game calls him in any language, and do zone and door
names have to be typed in by hand. Both are about static data, so both were asked of the ROM with
spike 74's extractor.

- **The rival's name is in the ROM in every language, and so is the boy's.** Trainer name list
  (file 22 of each language archive, 950 lines) lines 1 to 3 are the boy and 4 to 6 the girl:
  `Brendan`/`May`, `ユウキ`/`ハルカ`, `Brice`/`Flora`, `Brendon`/`Vera`, `Brix`/`Maike`,
  `Bruno`/`Aura`, `휘웅`/`봄이`. The seven archives are parallel — the same line number is the same
  person in each — so the *index* is a fact about ORAS and the *word* is the ROM's, in whichever
  language the ROM is played in. What is still missing is the half that is in RAM: **which of the
  two the player is**. The two protagonists are sprites `0xab HEROINE_RS` and `0xac HERO_RS` (see
  below), so the player's own move model should say it, and that wants a movie.
- **The overworld sprite list is in the ROM, and a move model's `+0x10` indexes it.** File 94, 544
  lines, and it lines up with everything already measured: `0x118 MAMA_RS` (Mom), `0x12a
  PCWOMAN_RS` (the nurse), `0x13e HAKASE2_RS` (Birch), `0x13f LEADER5A_RS` (Norman, the fifth
  Gym). **Untranslated** — the Japanese archive's file 94 is byte for byte the English one's — so
  these are the game's own ids, not words for a player. Two of them answer old questions:
  `0x7a` is `WKOBJCODE00`, a *work object*, which is why the character pa3ds ships as "Rival" is
  May to a boy and Brendan to a girl — it is not a character at all but a slot a script fills;
  and `0x162 MONSBALL`, `0x163 MONSBALL_RARE`, `0x170 KINOMI` name the item balls and berries.
- **The zone table is in the ROM too**, `a/0/1/3` file 536, 536 rows of 0x38 bytes — the same rows
  pa3ds already finds in RAM (`games.gen6.field`). Found by its own validator, not by the path:
  an outdoor zone is its own parent at `+0x1a`, and 193 of the 536 are.
- **A zone's name is the low ten bits of `+0x1c`, into the place-name list** (file 90, 356 lines).
  Checked against every zone pa3ds already knows by number: `0x006` and its five interiors —
  `0xdf`, `0xe0`, `0xe1`, `0xe2` (the two houses and their bedrooms) and `0xe3` (the lab) — all
  read "Littleroot Town"; `0x00d` and its sixteen interiors read "Petalburg City"; and the
  Japanese archive gives "ミシロタウン" for the same index. **Zone names are solved**, and the
  index is already in RAM: only the string needs the ROM.
- **`+0x1c` bit 10 says the zone carries the place's name.** Set on `0x006` and `0x00d`, clear on
  every interior under them. The rest of the bits above the name are unread.
- **`+0x08` is a building kind, shared across the game.** `0x10` is the Pokémon Center's ground
  floor and there are exactly 16 of them; `0x11` is a Gym room, 17 of them (eight Gyms, and
  Petalburg's maze is nine rooms on its own); `0x45` is a Center's second floor; `0x12` is
  Birch's lab, alone. A town and its plain houses share one value, so the field is really an area
  group — but it does separate the buildings that have names from the ones that do not.
- **The AreaNav names every facility, in the player's language**: `a/0/6/6` file 0, 168 rows of 60
  bytes, the outdoor zone at `+0x00`, a description at `+0x20` and the facilities' name indices
  from `+0x24` until a zero, all into file 33. Littleroot has `['Pokémon Lab']`; Petalburg
  `['Pokémon Center', 'Poké Mart', 'Petalburg City Gym']`; Rustboro adds `'Devon Corporation'` and
  `'Trainers’ School'`; Mossdeep in Japanese reads
  `['ポケモンセンター', 'フレンドリィショップ', 'トクサネシティジム', 'トクサネ宇宙センター']`.
- **What is still missing is the link from one door to one facility.** The AreaNav lists what a
  town has, not where; and a Poké Mart in ORAS is a counter inside the Pokémon Center, so it has
  no zone at all. A door's warp record gives the zone behind it, and that zone's name is the
  *town's*, so "door to Petalburg City" is all the zone table alone can say. The kind at `+0x08`
  is the bridge worth testing: the towns holding a kind-`0x10` zone ought to be exactly the towns
  whose AreaNav list has "Pokémon Center", which would pair a kind with a name **by derivation
  from the ROM rather than by a word of ours**. Not measured.
- **And the rival's model says nothing about which of the two he is** (the same day, from spike
  76's logs of `opening.ctm` and `opening_as_may.ctm`, kept under `captures/`). Playing as a boy
  and playing as a girl, the rival speaks as character **`0x7a` in both**, so the work object is
  filled with one slot whichever way round the story runs. The houses do swap, exactly as ADR-0023
  says: Mom (`0x118 MAMA_RS`) speaks in zone `0xdf` slot 7 in the boy's game and in `0xe1` slot 7
  in the girl's, with the other house's mother (`0x128 WOMAN_RS`) at slot 9 of the other zone in
  each. That reads the player's gender at the start of the game and nowhere else, since either
  house can be walked into later — so what is still wanted is the **player's own** move model,
  entity `0xff`, whose character ought to be `0xab HEROINE_RS` or `0xac HERO_RS`. No log of a
  replay has it, because the player never speaks and only speakers were logged.
- **Left open:** what `0x1000` means in a character. Two models in the boy's house read
  `+0x10 0x0000103f`, and `0x3f` alone is a sprite while `0x103f` is past the end of the list —
  the same shape as `0x2000` for a Poké Ball on the ground (spike 78), so the top bits of the low
  word look like a kind rather than part of the number.

## Spike 86 — naming a door from the ROM alone

2026-09-23 (`spikes/86_door_kinds.py`, the user's ROM, no emulator). Spike 85 left doors half
answered and got two things wrong, which the user caught: **a Poké Mart in ORAS is a building of
its own**, not a counter in the Pokémon Center, and so **kind `0x45` is the Poké Mart**, not a
Center's second floor — ORAS's Centers have one floor. Spike 85's entry stands as it was written.

- **Every zone's data file is in the ROM**: `a/0/1/3` file N is zone N's `ZO` file,
  LZ11-compressed (538 files: the zones, the zone table as file 536, and one more). Decompressed,
  it is byte for byte the shape pa3ds finds in the heap, and the spike reads its warps with the
  runtime's own parser (`field._zone_file`, `field._records`), so every door in the game can be
  listed without playing to it. LZ11 is written from the format, not taken from anyone's code.
- **A town and its plain houses share an area kind** (+0x08): Littleroot `0x05`, Oldale `0x06`,
  Petalburg `0x08`, Rustboro `0x09`. A building with a name of its own has a kind of its own.
- **The pairing is derived, not typed.** A kind behind doors in two towns or more is named for
  the one AreaNav facility every one of those towns lists that no more widespread kind has taken
  already; what is left in a town — one door and one facility — is paired by elimination. Both
  steps work on line numbers of the AreaNav's text, never on words, so they give the same answer
  in every language. Three kinds come out shared: **`0x10` Pokémon Center, `0x45` Poké Mart,
  `0xb7` Contest Spectacular Hall**, and elimination adds the Pokémon Lab, Petalburg City Gym,
  the Seashore House, the Glass Workshop, the Pokémon Day Care, the Treasure Hunter's House and
  the League's Pokémon Center.
- **38 of the 143 doors into AreaNav places are named this way, 84 share their town's area kind,
  and 21 are left.** The Japanese archive gives the same pairing word for word in its own
  language — `フレンドリィショップ` behind Petalburg's Mart door, `トウカシティジム` behind its Gym —
  because the pairing never looked at a word.
- **What the ROM cannot tell apart is a facility that shares its town's area kind.** Dewford
  Hall, the Trainers' School, the Lilycove Museum and the Name Rater's house sit among the plain
  houses by kind, and several Gyms are left with two candidates because their town has a second
  facility of that sort (Dewford's Gym and Dewford Hall; Mossdeep's Gym and the Space Center).
  Some facilities are not buildings at all but people — the Move Tutor, the Move Deleter, the Name
  Rater — so the AreaNav lists more facilities than a town has doors for.
- **What would settle the rest is in the world, not the tables**: the sign beside a door, whose
  message is the building's name, reached through its script. That is spike 77's script
  question again, and it would name the houses the player cares about as well.
- **Built** the same day, on the user's word (`archives.door_facilities`, `field.door_warps`): the
  same derivation at every start, about 0.1 s, the name keyed by the door's own slot on its map,
  which is what a rename is keyed by, so the player's name still wins.
- **And a bug it brought to light, fixed the same day.** A place's key was (map, index), and a
  sign's index and a warp's are each into a table of their own, so sign 1 and door 1 on one map
  were one key: the wally.ctm replay after the rebase read Petalburg's three signs as its Gym, Mart
  and Center, and renaming a sign had renamed the door of its number since renaming was built. A
  warp's slot now carries `field.WARP_SOURCE`, as the use tiles carry `TILE_SOURCE`, and the ROM's
  door names answer only a slot that does. Replayed again: the signs are plain signs, the doors
  keep their names, and the speakers and places are unchanged.
- **And the same fault between edges and use tiles, closed the same day (issue 1).** An edge's
  slot was `zone << 16 | gap` and a tile's `1 << 24 | x << 12 | z`, and a zone from `0x100` up
  puts its bit 8 on bit 24: an edge into zone `0x100 + k` with gap `g` and a tile whose first
  cell is (16k, g) were one key. Not reachable in ORAS — the smallest such zone, the Safari Zone,
  wants a tile at x = 2272 on a matrix at most 400 wide — but nothing kept it so. Every place's
  slot now carries a four-bit kind above 32 bits (`field.SOURCE_KIND`: furniture 0, warp 1, use
  tile 2, edge 3), where no number built from the game's u16s can reach — a zone shifted by 16
  is the widest — and the ROM's door names answer the warp tag rather than one bit. Replayed
  (`regress.py`, pc, out of sync 0): the exits and signs lines unchanged, one wandering
  townsperson a cell apart.

## Spike 87 — the player's own model, and a zone's name sooner than its banner

2026-09-23 (`spikes/87_player_model.py`, opening.ctm as the boy and opening_as_may.ctm as the
girl, each from its first frame on `user-saves-opening` with the runtime reading silently and
`FieldReader.poll` wrapped; no out-of-sync lines). The user suggested the two openings, since one
is played as each.

- **The player's own model says which protagonist they are.** Entity `0xff`'s `+0x10` read
  `0x000100ac` in every zone of the boy's game — the truck, Littleroot, both houses, both
  bedrooms, Route 101 — and `0x000100ab` in every zone of the girl's: spike 85's
  `0xac HERO_RS` and `0xab HEROINE_RS`, with a `0x0001` flag above them that `mmodel.character`
  already masks. It follows the player, not the house: the boy reads `0xac` in the girl's house
  and the girl `0xab` in the boy's. The rival stands in `0x7a` in both games, as spike 85 read off
  the old logs, and introduces herself as May to the boy. **So the rival's name is reachable in
  every language**: the player's sprite says which of trainer lines 1 and 4 is the rival.
- **A zone change is in RAM 0.6 to 0.8 s before the game's own place name is read.** Littleroot on
  arrival: zone `0x006` first polled at 207.75 s, "screen shown: Littleroot Town" at 208.36 s (the
  girl's run: 161.69 and 162.51). Route 101: 485.72 and 486.48. Polls are 0.2 s apart, so the lead
  is real but it is under a second: AreaNav's words change after the screen has faded up.
- **The game shows a place's name when the place changes, not the zone.** Out of the player's house
  and out of the neighbour's, back into Littleroot, nothing was shown: the interiors carry the
  town's own place name, so walking out is not arriving anywhere.
- **What pa3ds says for a place today is AreaNav's, not the sign's.** Every one of those lines is
  layout `0x15a5ccdc`, the bottom screen's AreaNav (`AREA_NAV` in `tests/test_announce.py`); the
  top screen's location sign follows it and is left unsaid as "just said elsewhere". Both runs
  also said "Littleroot Town" 0.8 s after the player walked into the rival's bedroom (the boy into
  `0x0e2` at 430.29 s, the girl into `0x0e0` at 332.33 s), just before the rival's scene: that is
  AreaNav coming back on screen, the same layout, not a banner for a place that did not change.
- **`0x1000` in a character marks a Pokémon.** Spike 85 left `0x103f` open, `0x3f` a sprite and
  `0x103f` past the end of the list; the shipped table has read it as the movers' Machoke since
  both openings were first logged. Like `0x2000` for a Poké Ball on the ground (spike 78), the top
  bits of the low word say what kind of thing stands there.

## Spike 88 — where the game places its work objects

2026-09-23 (`spikes/86_door_kinds.py stands`, the user's ROM, no emulator). The user asked how
far a name given to a character can be trusted, now that the rival turned out to be a work object:
does renaming a youngster name every youngster, and could a work object stand in for somebody
else? Every zone's data file lists its cast, each NPC record with the model code a move model
built from it holds at `+0x10`, so the question can be put to the whole game at once.

- **Most characters are fixed sprites, and a name given to one names everyone drawn with it.**
  `0xda TANPAN_RS`, the youngster, is 35 records in 31 zones; `0xdb MINI_RS` 28 in 26;
  `0x12a PCWOMAN_RS`, the nurse, 20 in 20 — one in every Pokémon Center. Renaming a youngster
  names all 35, which is what ADR-0023 chose for a shared sprite. The story's people are nearly
  their own: Norman's `0x13f LEADER5A_RS` is 5 records in Littleroot, Petalburg and Slateport,
  Wally's `0xb4 RIVAL_RS` 4 in Petalburg, Route 102 and Mauville.
- **Sixteen sprite ids are work objects, `0x7a WKOBJCODE00` to `0x89 WKOBJCODE15`, and thirteen
  are used.** A script decides who fills one, and the model holds the work object's id, not the
  sprite it was filled with: the rival read `0x7a` as May and as Brendan (spikes 85 and 87).
  `0x7a` is 27 records in 27 zones — Littleroot, Oldale, Mauville, Lilycove, Mossdeep, but also
  Meteor Falls and Mt. Chimney, where the rival may not be the one it holds. `0x7b` to `0x80` are
  59 to 105 records each, across Mt. Chimney, Mt. Pyre, Meteor Falls, Southern Island, the Battle
  Resort and the Secret Bases; `0x81` to `0x85` five to nine each, in the Contest Halls' towns and
  Mt. Chimney. Those look like the characters a version or the player's choices decide: Team
  Magma's or Team Aqua's people, the Contest's rivals, the Secret Base's visitors. **Not
  measured**: who fills any of them, which is the script's business.
- **So a work object's name is a slot's, not a person's.** A name given to one — the player's or
  pa3ds's own — goes to whoever the script puts there next, in every scene that uses it; "May"
  for `0x7a` is right in the opening and not yet checked anywhere else. The shipped "Rival" had
  the same exposure. What would settle it is the sprite the work object was filled with, which a
  move model must keep somewhere to draw it. The logged words of the rival's model do not hold it
  (`+0x14` is the script, `+0x18` changes with the scene), so it wants a spike: the rival's whole
  model in both openings, searched for `0xab` in the boy's game and `0xac` in the girl's.

## Spike 89 — the sprite a work object was filled with

2026-09-23 (`spikes/89_work_objects.py`, opening.ctm as the boy and opening_as_may.ctm as the girl,
each from its first frame on `user-saves-opening`; no out-of-sync lines). Spike 88 found a work
object's name belongs to a slot, and the user asked for renames to be targeted at the person
instead, so a name given to the rival is not given to whoever else the story puts in `0x7a`. The
game must keep the sprite it filled the slot with, to draw it; this looked for where.

- **The experiment.** Both runs were captured three seconds after the rival's model came into
  use in their bedroom — the heap, the linear heap and the code, 39,000 pages each, in two
  seconds — with the rival's model at `0x08d1e9a8` in both. The rival is drawn as May to the boy
  and Brendan to the girl, so the first guess was an address holding `0xab HEROINE_RS` in one
  and `0xac HERO_RS` in the other. **Nothing did.** The control, the player's own sprite the
  other way round, came up at once: `+0x10`, `+0x884` and `+0x898` of the player's model, and
  one `gfl::grp::g3d::H3dResource` at `+0x260`.
- **`+0x898` of a move model is the sprite it is drawn with.** Read in the rival's model:

      boy's game   +0x10 0x0000007a  +0x884 0x0101007a  +0x898 0x0000013a  SUPPORT_F_RS
      girl's game  +0x10 0x0000007a  +0x884 0x0101007a  +0x898 0x0000013b  SUPPORT_M_RS
      the player   +0x10 0x000100ac  +0x884 0x010100ac  +0x898 0x000000ac  (0xab as the girl)

  `+0x10` and `+0x884` keep the work object's id, and `+0x898` holds who was drawn. The rival
  is not drawn as the other protagonist at all but with sprites of their own, `SUPPORT_F_RS`
  and `SUPPORT_M_RS` — "support" being the game's own word for the rival — which is why the
  first guess found nothing.
- **Built** (`mmodel.drawn_as`): a model whose `+0x10` is a work object is read as its `+0x898`
  when that holds a sprite, by the speaker, the people in the navigation list and a rename; every
  other model is its `+0x10`, exactly as before, since only two models were in use in the scene
  and nothing else has been checked. A name given to May is now May's, and the slot is nobody.
  The rival's name comes from the ROM by the sprite alone (`SUPPORT_F_RS` and its bike to the
  girl protagonist's line, `SUPPORT_M_RS` and its bike to the boy's), so the player's own
  character, which spike 87 read for it, is no longer needed.
- **Replayed after** (both openings on the built code, no out-of-sync lines): the boy's game said
  all 16 of the rival's lines as May and the girl's all 15 as Brendan, each listed by name in the
  field's log, and the boy's run differed from the run before only by the usual timing noise.
- **Left open:** whether `+0x898` equals `+0x10` for every model that is not a work object (it
  is not read for them, so nothing depends on it); when `+0x898` is written as a work object
  comes into use (three seconds after was enough; a model read before it falls back to its
  slot); who fills the other twelve work objects in use (spike 88), which a replay reaching Mt.
  Chimney or a Contest would show.

## Spike 90 — the people a sprite is, through the trainers who stand as it

2026-09-23 (`spikes/90_sprite_trainers.py`, the user's ROM, no emulator). The sprite list names a
look by the developers' shorthand — `TANPAN_RS`, `MINI_RS`, `LEADER5A_RS` — which is no word for a
player, and the shipped names were filled in by hand from replays. The user asked whether the rest
could come out of the ROM the way the rival's name did, with the shipped table as the fallback.

- **A trainer's NPC record holds 3000 plus the trainer's id at +0x0a**, the flag the game sets once
  they are beaten. Read off Route 102: the Lass holds 3010 and trainer 10 is Tiana, the Bug
  Catcher 3008 and trainer 8 is Rick, a Youngster 3007 and trainer 7 is Calvin. Everyone else
  holds something below 3000 (`0x7d0`, most of them).
- **The trainer's own record gives their class** — `a/0/3/6` is a small file a trainer, 16 or 24
  bytes, the class at +0x02 into the 280 class names of text file 21 — and **+0x06 their battle**,
  1 for a double.
- **Across all 536 zones, 389 records are trainers, standing as 71 sprites, and 66 of those
  sprites have trainers of one class only**: all 11 as `TANPAN_RS` are Youngsters, all 22 as
  `FISHING2_RS` Fishermen, all 18 as `MAGMA_M_RS` grunts of Team Magma — who share the trainer
  name "Grunt" as well. The five that are split are each a single class beside a pair class using
  the same sprite for one of its two ("Swimmer ♀" and "Sis & Bro", "Black Belt" and "Brains &
  Brawn").
- **A pair class is found as the class whose every trainer fights doubles**: exactly Twins, Young
  Couple, Teammates, Sis & Bro, Old Couple, Ace Duo, Brains & Brawn, Mysterious Sisters,
  Interviewers and Leaders (Tate and Liza). Some single trainers fight doubles too, but never a
  whole class of them. A pair sprite is left unnamed: `ELDER_RS` is "Teammates" to its trainers
  and stands as five people who are not trainers at all.
- **A sprite one trainer stands as proves nothing**: `SCHOOL_M_RS` has one, Jerry, and is a look
  schoolboys share. And the story's people are not found this way: a Gym Leader's record holds no
  trainer flag, since their battle is a script's.
- **So the story's people are named by a table of ours, and the words are the ROM's.** Each
  `LEADERn_RS`, `BIGFOURn_RS`, `CHAMPION_RS` and the teams' bosses and admins stand where their
  person does — `LEADER1_RS` only in Rustboro, `LEADER3_RS` only in Mauville, `CHAMPION_RS` in
  Granite Cave, Meteor Falls and Mossdeep, `MAGMABOSS_RS` in Team Magma's hideout — and each of
  those people is a trainer in the ROM's own list: Roxanne 561, Brawly 563, Wattson 567, Flannery
  569, Norman 570, Winona 571, Wallace 572, Sidney 553 to Drake 556, Steven 557, Maxie 235, Archie
  178, and so on. The table maps sprite to trainer, and the name is read in whichever language:
  `LEADER5_RS` is Norman and センリ.
- **Built** (`archives.CHARACTERS`, `archives.trainer_sprites`): 30 sprites are named as people and
  51 as a kind of trainer, the class alone ("Youngster") or with the name when every trainer
  shares it ("Team Magma Grunt"), read at every start in about a third of a second. The shipped
  English table is the fallback, and now holds the ROM's own English for all 81, May and Brendan
  for the rival's sprites, and the people the ROM never lists as trainers — Mom, Professor Birch,
  Mr. Briney, Captain Stern, Professor Cozmo, Lanette, Mr. Stone, Looker, Tate, Liza — by the
  sprite's id and where it stands.
- **Replayed after** (both openings and wally.ctm, no out-of-sync lines): the boy's opening tagged
  24 of Professor Birch's lines where it had tagged 2, since his sprite on Route 101 and in the
  lab (`0xb3 HAKASE_RS`) is named now; wally.ctm's 35 lines of Norman, 5 of the Nurse and 34 of
  Wally were unchanged, and so was the girl's opening. The rest differed only as runs of one
  commit differ.
- **Left open:** personal names for trainers — each record says which trainer it is, so the
  Youngster on Route 102 could be "Youngster Calvin", but a name is keyed by sprite, and a rename
  of him would name every Youngster; the sprites `DANPEI_RS` (in all eight Gym towns: the Gym
  guide, most likely), `KIZUNA_RS`, `DIRECTOR_RS` and `CONTEST_RS`, whose people are not certain.

## Spike 91 — the people the ROM never lists, in every language it carries

2026-09-23 (`spikes/91_names_in_languages.py`, the user's ROM, no emulator). The shipped names are
fallbacks, heard only when pa3ds cannot read the ROM, and the user asked for a set in every
language the game carries, leaving out only what cannot be named.

- **The story's text is parallel across languages, like the system text**: `a/0/7/9` is the
  Japanese in kana, `a/0/8/0` in kanji, then `a/0/8/1` to `a/0/8/6` English, French, Italian,
  German, Spanish and Korean, 637 files each, the same sentence at the same file and line.
- **So a person the ROM never lists stands in a sentence that every language has.** None of Mom,
  Professor Birch, Mr. Briney, Captain Stern, Professor Cozmo, Lanette, Mr. Stone, Looker, Tate
  or Liza is a line of their own anywhere in the English text; each is inside one — "Mom: Norman?",
  "Did you get to meet Professor Birch?", "Mr. Briney's Cottage", the "Looker Ticket" — and that
  line printed in all seven languages gives each one's name as the ROM spells it: Mr. Briney is
  M. Marco, Signor Marino, Mr. Brack, Sr. Arenque, ハギろうじん and 하기노인; Mom is ママ, Maman,
  Mamma, Mutti, Mamá and 엄마, the game's own speaker label for her.
- **Tate and Liza are told apart by order**: "Liza and Tate" in English, and the boy first
  everywhere else — Lévy & Tatia, Tell e Pat, Ben & Svenja, Vito y Leti, フウとラン, 풍&란.
- **Machoke is in the species list** (text file 98, line 67) in every language.
- **The Nurse, the Townsperson and Mrs. Birch have no word in the ROM in any language**: they are
  pa3ds's own English, for sprites the game never names. They stay in English only.
- **Built**: `data/<language>/sprites.toml` for Japanese, French, Italian, German, Spanish and
  Korean, 93 entries each — the 81 sprites `archives` names out of the ROM, copied by `table`, and
  the twelve read off these lines — beside the English table's 96.

## Spike 92 — the way there, over the game's own collision

2026-09-24 (`spikes/92_pathfind.py`: `replay` over spike 82's Littleroot capture, and `live`, the
runtime over poke_center_pc.ctm from its first frame on `user/save_checkpoint_3`, P pressed by a
script in place of the keyboard). The user asked for P to find the way to whatever the
navigation keys are on, said as its legs ("5 east, 2 north"), to an open cell beside the thing
when the thing is solid, and for the next leg as each turn is reached.

- **The blocked bit is enough to find a way round a town.** Over Littleroot's two chunks, from
  the player's door at (94, 172): the far house's door at (96, 183) is "2 west, 12 south, 4
  east", round the house's west side to the cell in front of the door, facing it; the way north
  is "5 east, 13 north", through the three-cell gap; each person is reached at a cell beside
  them. Every path took under a millisecond.
- **The game puts the player in a door's own cell before the warp.** Walking north into the
  Pokémon Center, the player's cell read (106, 134), the door, which the cell table blocks, for a
  poll before the model inside was built. A path to the Poké Mart was found again from there and
  said a moment before the warp. Built: a player standing on a warp that is not the way's end is
  left to it, and nothing is said.
- **Right after a zone loads there can be no chunk at all**, and a door is then an open cell
  like any other. The first press said "no path to Poké Mart", because the search would not step
  onto a warp even as its goal. A goal is always walkable now, and the way given, "9 north, 7
  east", ran through the Center's walls, which no chunk had shown yet: a path over cells nobody
  has read is a guess, and it is checked again at each step against the chunks loaded by then.
- **A player walking a way of their own, as short as ours, heard the whole way again at every
  step.** P on the Center's mat said "2 east, 6 north"; the movie walked north first, and each
  step north was off the path: "2 east, 3 north", "2 east, 2 north", and on. Built: a player
  already walking one way pays a turn for a path that starts another way, and a new path that
  starts the way they are walking is taken without a word. The same replay then said "7 north,
  2 east" (the mat's other cells are the warp, so east was not the first step), "2 east" at the
  turn, and "PC, 1 north" at the counter.
- **A warp ends the guiding quietly**: the player's model inside is a new one, built in the
  Center's zone (spike 78), so the model and its zone, which the field reads each poll, both
  change. Walking across an edge changes neither.
- **Left open:** which side of a sign it can be read from — the path goes to the nearest open
  side, and Littleroot's sign at (100, 183) was reached from the north, which may be its back;
  which way a ledge drops (`0x75000021` carries the blocked bit, so a ledge is walked round;
  read in spike 126); water; tall grass, which a path goes through; and reading a chunk the game has not loaded, out
  of the ROM's `a/0/3/9`, so that a far way is not a guess.

## Spike 93 — the PC's own word, the old tile keys, and where an exit leads

2026-09-24 (`spikes/93_pc_word.py`, both of the user's ROMs, no emulator; the user's own names
file for Alpha Sapphire; replays of signs.ctm, opening.ctm and opening_as_may.ctm). The user had
played Alpha Sapphire from the opening to Oldale and back, and asked three things: whether a use
tile has a name in the ROM, since "PC" was pa3ds's own word; whether renames of use tiles, keyed
like `0xe7:16818180`, were heap addresses that change every run and should be refused; and why
no exit said that the gap north of Littleroot leads onto Route 101.

- **Nothing names the PC where it stands.** The overworld sprite list (file 94) has no PC —
  `PCWOMAN1` and `PCWOMAN_RS` are nurses — and the Center's zone data has no record at it (spike
  83). It is a cell of the counter, drawn with the building.
- **But the PC's own text names it, in every language.** File 92 of each system text archive is
  the PC's: line 0 "booted up the PC!" (after the player's name), 1 "Which PC should be
  accessed?", 2 "Someone's PC", 3 "Lanette's PC", 6 "Professor's PC", then the storage and
  records menus. Lines 0, 2, 3 and 6 share one word in every language: "PC" in English, French,
  Italian, German, Spanish and Korean, "パソコン" in Japanese. Line 1 is left out, since the
  Italian asks "A quale sistema vuoi accedere?". Both games have the file line for line.
- **Built** (`archives.PC_LINES`, `archives.shared_word`, the field reader's `words`): the
  longest run of characters all four lines hold, with the spaces and punctuation at its ends
  taken off, is what the PC is called; right in all fourteen archives of the two ROMs. "PC"
  stays as the word without a ROM.
- **The other use tiles have no word of their own in the game's text.** English file 155 holds
  what they say — "The trash can is empty.", "There's a set of Pokémon picture books." — fourteen
  sentences for eight scripts, and which kind shows which is in the scripts (spike 83). A
  sentence is not a name, so they stay "object".
- **The keys taken for heap addresses are use tiles, under the tag they were written with.**
  `0xe7:16818180` is `0x0100a004`: `1 << 24`, the use tile's tag until 9c8ff8c, over
  `10 << 12 | 4`, the cell (10, 4). All thirteen such keys in the user's `sango-2.toml` are
  cells inside their rooms, and `0xe0:16801807` is (6, 15) in the bedroom — the one `0xdd` spike
  83 found in the bedroom's chunk and took for the trash can, which the user named "trash can".
  A tile's slot is the first cell of a run of one kind in the building's one chunk, the same every
  run. They stopped answering because 9c8ff8c moved the tag above 32 bits (`2 << 32`) the same
  evening: it was pulled at 23:37 on 2026-09-23, and every tile key in the file carries the old
  tag, so all were written by a pa3ds from before the pull. Re-keyed in the user's file by hand,
  cell kept and tag moved; no code carries it.
- **So no rename is refused for lying outside the zone's table.** A use tile is never in the
  table — the PC included — so that guard would refuse every one of them. The only thing listed
  whose `source` is not a slot is the hidden Pokémon, refused already (`Interactable.naming`).
- **Where an exit leads had never been built**, only what kind of warp it is: `_warp` said
  "told by the kind and where it goes", meaning whether the zone it goes to is outdoors.
  **Built**: an exit whose zone is in another place than the player's says it after its name —
  "edge to Route 101" — the place indices the zone table's in RAM and the words the ROM's, as for
  arriving (spike 87). A place and not a zone, so a door into one of the town's own houses says
  nothing more, and nor do the stairs between two floors.
- **Replayed before and after** (`regress.py`, out of sync 0 throughout, against a run of the
  same code as a control): the exits lines carry "(Route 101)" on Littleroot's gap in 18
  listings, and "(Littleroot Town)" and "(Oldale Town)" on Route 101's two ends; no door, way out
  or stairs of the town or its houses changed. With the places taken out, the diffs are 13, 91
  and 35 lines against the control's 5, 74 and 23, of no kind the control lacks, and the speaker
  tags held at 6, 80 and 53. One listing on arriving on Route 101 has Littleroot's doors in it,
  now "(Littleroot Town)": the first poll there reads Littleroot's zone data before Route 101's
  has replaced it, as earlier runs did, and the next poll has only Route 101's.
- **A drifted replay that reported no drift.** The first "before" run of opening_as_may read Mom
  asking seven times whether the player wanted to see the new room, which the movie had gone
  upstairs to see, at out of sync 0 — the trap [the rig runbook](../runbooks/rig.md) warns of:
  a desynced replay looks normal. Run again, it followed the movie.
- **Left open: the trash can has a word after all, as a Secret Base decoration.** Files 54 to 57
  are the decorations' names and descriptions, and line 64 of file 56 is "Trash Can" ("ゴミばこ"
  in Japanese), beside a toy "TV"; there is no bookshelf, shelf or vase among them. The user's
  names put a trash can at (6, 15) in `0xe0`, (17, 15) in `0xe2`, (14, 4) in `0xe6` and (6, 4)
  in `0xe7`; if all four are kind `0xdd`, as the bedroom's is, a table of ours from that kind to
  that line — as `archives.CHARACTERS` goes from a sprite to a trainer — would call it what the
  game calls a trash can. Reading those cells' kinds wants a movie in those houses, or the zones'
  chunks out of the ROM.
- **Left open:** an edge's slot is `zone << 16 | gap`, and the gap is its place among the runs of
  crossings the loaded chunks show, so a name given to an edge can move to a neighbouring gap when
  other chunks are loaded. Nobody has named an edge yet.

## Spike 94 — a chunk read while it is still loading

2026-09-24 (`spikes/94_chunk_cache.py`: the runtime over map_chunk.ctm from its first frame on
`user/save_checkpoint_5`, P pressed by a script as the user pressed it). The user walked from
Route 116 into Rustboro, into the Gym and out, and pressed P for the Pokémon Center; halfway down
the city it said "no path to Pokémon Center", and said it again from every cell round the door
for a minute. The live log had the city's south edge read as wall along the whole chunk holding
the door, where a later visit read three open ways onto Route 104.

- **The table pa3ds kept was not the one the game had.** At the moment the way was lost, from
  (131, 194), the cached tables of the two chunks just loaded differed from the files' by then in
  1276 and 890 of 1600 cells. Drawn, every cell south of row 200 round the door was wall, and the
  door had no open cell beside it to walk to.
- **A block names its new chunk before the chunk is in it.** A reload keeps the block's own
  buffer: the centre and the file change at once, the size reads `0x1000`, and the file still
  holds the chunk the block had before, a valid `GR` file that passes every check `read` makes.
  pa3ds read it then and kept it under the block's file, size and origin, which a later look at
  the same chunk never changes. At 53 s the block went back to (80, 160): the stale copy matched
  the file for one poll, then the real table came in and the copy stayed.
- **+0xa8 is where the load has got to**, and all six loads in the run went the same way: 2 as
  the load starts (+0x78 is 1 and +0x7c points elsewhere), 3 the moment the cell table changes,
  and 4 one poll later, which it stays until the next load. The size at +0x90 takes the file's own
  value only seconds after, so it says nothing about the table. **Built**: a block is read only at
  4 (`mapblock.READY`); before that its chunk is not loaded, and its cells are open, as any cell
  no chunk holds is, until the next step checks the way again.
- **Replayed after the fix**, the same press said "9 south, 4 west" where it had said "no path",
  and guided on round the door. Before and after with `regress.py` (pc, hidden_tail,
  hidden_tail_3, map_chunk; out of sync 0 throughout): the only change of kind is one more "no
  chunk loaded, sweeping" on arriving, since the blocks are still loading at the first poll; the
  rest is the player's cell drifting between polls. Timed on the same movie with the check and
  without it (`60_poll_timing.py` and `64_runtime_audit.py`, which take `--saves` for a movie on
  a checkpoint of its own now): 56.2 polls a second against 56.0, the median gap 17.1 ms in both,
  and the reads per reader the same to within a poll.
- **The first recording would not replay.** Recorded with "Force deterministic async operations"
  off, it stayed on the title screen in both file-read modes, with out of sync 0 and "Playback
  finished" after its 53 seconds; recorded again with it ticked, it followed the user's run.

## Spike 100 — Pokémon X's field, and what it keeps where ORAS does not

2026-09-24 (`spikes/100_x_field.py`, movie_x_map.ctm on the user's X save, Route 4 by Lumiose
Gate). Every navigation key said "no map" in X (the user's log of the same day), and the log said
the game's own words were off because `a/0/1/3` held "171 files, too few to hold the zone table".
X was being read with ORAS's anchors, as it always had been. The spike captured the heap on
Route 4, read the chunks live, and compared X's ROM with Omega Ruby's, file by file.

- **X is ORAS's engine.** Every class the field reads is in X's code: 32 move models (0xaa8 apart,
  eight bytes smaller than ORAS's), six `field::MapBlock`s of which four hold chunks, the field
  map with its grid (20 by 21 chunks, which reads as clean rectangles of zones only with +0x36 as
  the width: the first matrix seen that is not square, and so the first proof of which is which).
- **The zone file has four sections, not five, and that alone was "no map".** Its header is
  `ZO`, u16 4, five offsets from 0x18; the info (0x38 bytes) and the entity table are sections 0
  and 1 as in ORAS, with the same record sizes (1 sign, 19 NPCs, 1 warp and 2 triggers make
  Route 4's 0x3f4 exactly), and the holder is ORAS's shape. The validator wanted exactly five, so no
  file was the zone's and `poll` was None. With four accepted, all 360 of X's zone files pass it,
  and the runtime listed Route 4 live.
- **The chunk file has six sections, not seven.** Its cell table is ORAS's: 40 by 40, bit 0
  blocked, the kind in the top byte. Route 4's hedge maze reads as a symmetric maze with hedges
  north and east of the player, as the screenshot has them. X's tall grass is kinds 0x21 and 0x23.
  The PC's kind 0xd5 is in 7 of X's 1,030 chunk files and the counter's 0xd4 in 69, which is how
  rare they are in ORAS; no movie of X has stood by a PC yet.
- **The zone table is in the heap, and was not found for two reasons.** It is the ROM's own file,
  byte for byte, at `0x08cfb6e4`. Route 4's row ran from one 1 KiB page into the next, and the
  search went a page at a time; and the word before the table's pointer led to Lumiose Gate's row,
  the zone the player had come from, where ORAS's reader took it for the current zone's. The table
  is now found by its start: the current zone's info sits `zone` rows in, and a word points there.
- **A move model's drawn sprite is at +0x890.** +0x898 holds a pointer in X. The work objects are
  the same sixteen ids, `WKOBJCODE00` to `15` at 0x7a in X's sprite list (text file 76), and an
  item ball is 0x2000 at +0x10 with MONSBALL (0xab) at +0x890.
- **The ROM keeps the same files under other numbers.** Zone files are `a/0/1/2` with the table
  file 360 (360 rows); text archives are `a/0/7/2` (kana) to `/9`, English `/4`; place names are
  file 72 ("Route 4" at Route 4's index 0x14, "Lumiose Gate" at Lumiose Gate's 0x78), trainer
  classes 20, trainer names 21 (614, not opening with the protagonists), and the PC's lines 74 in
  ORAS's order in all eight archives. Trainer records are `a/0/3/8`, the class a byte at +0x01 and
  the battle at +0x02; an NPC's +0x0a is 3000 plus its trainer as in ORAS, and Route 4's two
  sprites come out as Roller Skaters. There is no AreaNav, so no door is named by its building.
- **Warps**: 0x0301 is a door in, 0x0200 the way out and 0x05xx stairs, as in ORAS. X walks
  into a gate from a route by 0x0501, which is said "stairs up"; ORAS has 48 warps of that shape
  too, so it is a naming question for both games. X also has kinds ORAS never had (0x07xx in
  Lumiose, 0x10xx, 0x38xx, 0x39xx), said "warp".

**Built** (the user, 2026-09-24: an X package, sharing everything that is easy to share):
`games.xy` is X's numbers in a `Game` — title ids, ROM layout, the drawn offset — and ORAS's
profile and readers run it, the readers taking either game's section counts. `games.catalog`
picks the game by the running title id. Replayed with `regress.py` before and after, with a
same-code control (out of sync 0 throughout): pc, hidden_tail, hidden_tail_3 and map_chunk
changed no more than the control does, save one listing of Rustboro with Route 116's warps in it,
taken while the zone file was still changing over — which the old code did too, twice in the
user's own log of 2026-09-24. movie_x_map.ctm now says the place, "Route 4", and lists its edges
by where they lead and its people by their class.

## Spike 101 — X's Pokémon Center door, named by its Town Map

2026-09-25 (`spikes/101_x_town_map.py`, movie_x_city.ctm on the X save of that morning, into
Santalune's Pokémon Center and out, then onto Route 3). The user asked whether the Center's door
could be named at all, or had to be renamed by hand. Spike 100 had it unnamed because X has no
AreaNav; that was right about the AreaNav and wrong about the door.

- **X's Town Map is its AreaNav.** Text file 31 of each archive is the Town Map's: "Fly to
  where?", a description a town and route, then the facilities' names — "Santalune City Gym" at
  67, "Pokémon Center" at 94, "Boutique" at 99. `a/0/6/8` file 0 lists what each town has, a row
  of 0x40 bytes a town: its outdoor zone at +0x00, its description's line at +0x24 (Santalune's is
  11, "Many beginning Trainers gather…") and its facilities' lines from +0x28 (Santalune's: 67,
  94, 68 Trainers' School, 99). Camphrier's row lists Shabboneau Castle, the Name Rater and Hotel
  Camphrier, as the game has them. It is ORAS's AreaNav in another shape, and ORAS's derivation
  (spike 86) runs on it unchanged.
- **20 of X's 109 doors in from outdoors are named**: the Pokémon Center in thirteen towns and
  cities (Santalune's is slot 0, the door into zone 0x36 the replay walked through), the Boutique
  in five, the Scary House on Route 14, and Hotel Marine Snow in Geosenge, which is paired by the
  last rule (one door and one facility left) and has not been seen in play. The same in all seven
  languages, since the pairing is made on line numbers: "Centre Pokémon", "ポケモンセンター".
- **Every Gym is entered by a warp of its own, never a door.** 0x1001 in Santalune, Laverre,
  Anistar and Snowbelle, 0x3701 in Cyllage and Coumarine, 0x1601 in Shalour, 0x3800 into Prism
  Tower; each leads to a zone of area kind 0x11, ORAS's kind for a Gym room. Counted as doors,
  they name only Anistar's, since a town's Gym and its other facilities of one are all unique to
  it and nothing says which line is the Gym. So the Gym stays "warp" for now.
- **The PC reads in X as in ORAS**: kind 0xd5 inside Santalune's Center, called "PC".
- **The people in the Center are unnamed.** The nurse is sprite 0x3b, `PCWOMAN1` in both games'
  sprite lists. X's list is ORAS's first 171 ids under the same names, but ORAS's shipped names
  are all on its own `_RS` sprites (0x12a `PCWOMAN_RS` is its nurse), so none carries over.

**Built**: `TownFacilities` gives a game's list of what each town has its own row shape, ORAS's
AreaNav and X's Town Map both. Replayed, movie_x_city.ctm says "door to zone 0x36 "Pokémon
Center" 1 north" and "door to zone 0x39 "Boutique"", out of sync 0; ORAS's 38 door names are
the same pairs as before, compared whole.

## Spike 102 — Lumiose City's boulevards are rails, not cells

2026-09-25 (`spikes/102_lumiose.py`, movie_x_lumius.ctm on the X save of that morning, along a
boulevard and into a building). In the user's own session the runtime put the player at x = 0,
signs 44,000 cells east and warps 900 west, and P walked the player into another building's warp.

- **Lumiose's boulevards place everything on rails.** A furniture or warp record's +0x06, 0
  everywhere else, is 1 there, and where the cell or world position would be the record holds a
  u32 rail id (`0x01049366`) and two small numbers; an NPC record holds the rail id at +0x28, the
  distance along at +0x2c (the model's +0x9c agrees) and a byte that looks like the offset across
  at +0x2e. 382 of X's 1,604 furniture and warp records are on rails, in the seven Lumiose zones
  (0x3f to 0x45) and four more (0xbf, 0xd7, 0x108, 0x11f); none of ORAS's 1,743 are.
- **A move model on a rail keeps its rail coordinates where its cell would be**: +0x94 is the
  rail's id, +0x9c the distance along it, and +0xec..+0xf4 the rail's own coordinates, 0 across
  and the distance times 18. **+0x114..+0x11c is where it is in the world**, on a rail and on the
  grid alike: (3847, 3, 3372) on the boulevard, which is cell (213, 187), inside the chunks the
  game had loaded at that moment, and moving with the walk; on the grid it equals +0xec even
  mid-step (every model in 28 captures of both games), so the two differing is what says a model
  is on a rail. The `Move3DModel` at +0x97c holds the same at +0x154.
- **The chunks there mark every cell a wall**, the player's own included: the streets' walls are
  kept in a map of each rail's own. The rail system keeps a hash table of the rails loaded (in the
  heap, an entry an id and a pointer into the linear heap), and each rail there has its id, its
  group (`0x0504`), the rails before and after it, a name (`line@normal_7@5043ef6`) and offsets
  into a block that starts with the cell size, 18.0, and ends in a cell table of walls. The
  curve itself is not read: the `line@normal` names are in the chunk files' 3D models.
- **Built**: a model's cell is its +0x94 where that agrees with where it is in the world (the
  cell a step heads for, which guiding keeps its timing by) and the world's cell where it does
  not; a model not yet placed is not listed; signs and warps on rails are left out, keeping
  their slots; and a player on a rail is told the thing and its bearing on P with "no route on
  these streets", since no way can be found over walls the chunks do not have, and edges are
  not narrowed there. Replayed, movie_x_lumius.ctm lists the player at (213, 187) in zone 0x3f
  and the people at sensible distances; pc, hidden_tail, hidden_tail_3 and map_chunk changed no
  more than a same-code control does (out of sync 0 throughout).
- **The rig's emulator crashed twice** with a read of 0x120f8 in Azahar: the user's own pa3ds,
  left running after its emulator closed, kept retrying its reads, and when the rig started an
  emulator on the same port it read before any process was selected there, which Azahar does
  not survive ("No target process selected" is the last line of its log). The runtime now reads
  nothing after the emulator stops answering until it says the game is still the one selected,
  and starts over, attaching afresh, when it is not.

## Spike 103 — Lumiose's rails, their walls, and which way the stick walks them

2026-09-25 (`spikes/103_rail_walk.py`, movie_x_lumius.ctm on save_checkpoint_7, the player's model
read every 50 ms and lined up with the movie's own stick input). The user: things in Lumiose City
could not be walked to at all.

- **A position on a rail is three fields of the move model**: +0x94 the line's id, +0x9c the cells
  along it, and +0xf8 the offset across it in world units (+0xa0 in whole cells). With them the
  player's world position is a smooth function of the rail position to 0.02 units over 278
  samples. The people standing on rails agree with their records exactly: +0xf8 / 18 is the
  signed byte at the record's +0x2e (10, −8, 7, 6, 0).
- **The rails the game has loaded are listed in one table** in the heap, an allocation of its own
  (`0x08247214`, 0xa34 bytes, in this replay): entries of 16 bytes, an owner, the line's id, 0 and
  the line's record in the linear heap, which starts with the same id. The owner holds the file
  the lines are in and the rails' offset in the world, (3960, 3240) for Lumiose. The entries move
  within the table and the records to new addresses as lines load and unload with the walk.
- **A line's record** has its start and end points at +0x08 and +0x0c, its name at +0x1c, its
  cells along (48) and across (67) at +0x3c, its file at +0x44, and at +0x54 where in the file its
  walls are: 49 rows along by 67 columns across, the same words as a chunk's cells. Drawn, they
  are a street 18 cells wide with lamp posts in it and buildings either side. The doors in the
  zone's records sit at ±10 across, at the street's edges. Lines meet at points, two a point, so
  the lines loaded make chains.
- **In the world a line runs straight** from its start point to its end, the cells along a
  fraction of its length (48 × 18 units), and across runs along the side vectors its two points
  hold, blended: that fits every sample to 0.22 units. It is not needed to walk there.
- **The stick walks the player relative to the line**: up went along it (n = 6, east-north-east,
  which is the line's way) and right across it the way the offset falls (n = 17), with down and
  left the reverse; on the other line only down was held, and it went back along. The alignment
  is of a movie to a replay by movement, 80 % agreement, so this is the likeliest reading rather
  than a proof; it wants a movie holding each direction on a boulevard.
- **Built**: `rail` reads the table and the lines, and lays a chain end to end into a corridor
  of cells with along as north and a falling offset as east, so the words a way is said in are
  the stick directions that walk it. On a rail, the player, the people and the signs and warps
  placed on rails are in the corridor, the corridor's walls are the ground, and pathfinding and
  guiding run over it unchanged. The corridor stays laid out from one line while that line is
  loaded and the player's runs the same way, so a way found holds from one line onto the next.
  X's warps of kind 0x07xx, all from Lumiose's streets into its buildings, are called "door".
  Replayed, the door into zone 0x56 is listed 6 north 7 east, P finds "6 north, 6 east" to it,
  ending facing east, and the player's replay walks into it from 10 west of it; the other six
  replays changed no more than a same-code control.

## Spike 104 — on Lumiose's rails the D-pad turns the player; spike 103's stick reading was skating

2026-09-25 (`spikes/103_rail_walk.py` with the pad read from HID shared memory at each sample;
movie_x_walking_to_a_door_in_lumius.ctm, its D-pad version, and three test movies made from the
first 19.5 s of movie_x_lumius.ctm with D-pad presses appended, kept in the captures). P said
"3 south, 29 east" to Lumiose Gate's stairs, and following it the user went further away.

- **Spike 103's rule was wrong.** Its movie was driven with the circle pad, which in X is the
  roller skates: momentum, curving turns, a third of a second of coasting after release. Held
  right, the heading turned steadily through 100° in half a second; no cell-by-cell direction can
  be followed on skates, on a rail or off one. The user's Azahar puts the circle pad on the arrow
  keys and the D-pad on T, F, G and H.
- **The pad as the game reads it** is at 0x10000000, HID shared memory: the latest of eight
  entries at +0x10, and from +0x28 entries of 16 bytes, the buttons held first (D-pad right 0x10,
  left 0x20, up 0x40, down 0x80; circle pad right bit 28, down bit 31) and the circle pad's x and
  y at +0x0c. Read beside the model at every sample, it needs no lining up with the movie.
- **With the D-pad on a rail, right and left never walk the player**, even held three seconds:
  they turn them. Up walks the way they face, and down turns them round and, held, walks that way.
  The game names it: `field::mmodel::ActionCmdDirRotateSideForCameraBehind` and
  `...RotateBackForCameraBehind`, beside `field::RailSystemCameraBehind`. Taps and holds, in the
  movies made for it, mostly agreed; where they did not, up came soon after a turn, while the
  turn was still being animated. The camera's own heading was looked for in the heap and not
  found (nothing there settles to one value a heading).
- **A model's +0x1c and +0x24 are the sine and cosine of the way it faces**, in the line's own
  frame on a rail (across, along): (0, 1) along the line, (−0.96, 0.29) after turning right.
- **Built**: on a rail the field gives the player's heading, the way they face as a way of the
  corridor, and P and guiding say a way from it — "turn right, 5 forward, turn left, 3 forward",
  "turn around, 4 forward" — and the list says where a thing is from it, "9 ahead 8 left". Off
  the rails nothing changes. From the D-pad movie's start, facing along the boulevard, the way to
  the stairs P said as "3 south, 29 east" is now "turn around, 3 forward, turn left, 29 forward".

## Spike 105 — on Lumiose's rails up walks where the camera looks, and a tap leaves it anywhere

2026-09-25 (`spikes/103_rail_walk.py` `sample` and `snap`, `spikes/105_rail_camera.py`;
movie_x_walking_to_a_door_in_lumius_direction_pad_version_2.ctm on save_checkpoint_7, and two
test movies made from it). The user: "really struggling to use the system. and pressing down
dont turn my camera just character." In their session P said "turn around, 3 forward, turn
left, 29 forward"; after down, up walked them the old way, and the directions changed as they
walked straight.

- **Up walks the way the camera looks, not the way the player faces.** Down turns the player
  round and walks them back, and leaves the camera; the next up walked the old way (the user's
  movie, 22.9 s and 26.7 s). Left and right turn the player at once to the way of the street
  nearest a quarter from the camera, and turn the camera after them only while the key is held,
  about 150° a second, easing to a stop: a hold of a second or more (0.7 s, 1.5 s and 3 s in a
  test movie) settles it on the player's way to within a degree and no further, and a tap
  leaves it where it stopped. Five taps of left left it at 132° with the player facing 92°, and
  up then walked the player diagonally across the street until a wall stopped them; two taps
  left it 7° off with the player facing a quarter away.
- **The camera's heading is in the heap**, found by snapshotting the heap and the linear heap
  at seven moments the player walked up, each a different way, and looking for anything that
  pointed the way they walked at all seven (`105_rail_camera.py search`). The linear heap holds
  copies of the view matrix; the heap holds a float in radians, x over z as a model's world
  facing is, equal to the walk to a tenth of a degree, at +0xebc in a block of its own (tag
  `0x5544`, 0x1468 bytes) that `xy_system::CCameraULCD`'s +0x64 points at. The block stayed at
  one address through the walk.
- **A move model on a rail keeps the line's own way in the world** at +0xa4 and +0xac, the sine
  and cosine, where it stands: the player's facing on the line is its world facing less this,
  whenever the model is not mid-turn. The ways the game turns the player to are a line's own and
  are not quite this: 2.4° off it on one line, 15° on the next.
- **The table of lines loaded misses one for a single read now and then**, as the game moves its
  entries: the line the player had just left was not in it for one poll, the corridor was laid
  out afresh from the new one, and guiding said "stairs gone" and stopped.
- **Built**: on a rail the player's heading is the camera's, turned into the corridor through
  the line's way where they stand, and the way is said from it with the keys that walk it: "3
  back, hold right, 29 forward". A camera more than 20° off a way of the street is said to be
  angled, and a way ahead or behind then starts "camera angled, hold left, hold right", since a
  hold always settles it a quarter from the way it is nearest. While a way is being guided, the
  player standing still hears the way again once the camera has stopped somewhere new: the
  words for it once a hold has turned it, and the straightening once a tap has left it angled.
  Every line read is kept. With the model's facing where no camera is found, as before. Over a
  test movie that does what P says — down held 0.95 s, right 1.4 s, up 8 s — pa3ds said "3 back,
  hold right, 29 forward", "hold right, 29 forward", "29 forward", "stairs, 1 ahead", and the
  player went up the stairs into Lumiose Gate; over the user's own movie, guiding held the stairs
  from the first press to the end (out of sync 0 both).

## Spike 106 — Lumiose's streets load with their chunks, and the ROM has them all

2026-09-25 (`spikes/106_rail_rom.py`, `spikes/103_rail_walk.py snap`;
movie_x_path_to_lab.ctm on the X save of that afternoon, kept as save_checkpoint_8). The user:
"not sure if every path findable thing shows in the tracker? i can't seem to find my way to the
lumius lab." The list had 6 exits; zone 0x3f has 15 warps.

- **The Pokémon Lab is zone 0x53**, behind a door on line `0x0305feb7`, the line after the one
  the player's boulevard starts on: its people are the three friends who wait there and one more,
  and Lumiose's Town Map row for zone 0x3f lists the Lab among five. The Pokémon Center, zone
  0x4b (the nurse's sprite, `0x3b`), is on the line after that, `0x0205ded0`. Nothing on either
  line was listed: the rail table held three lines, `0x0305ac3b`, `0x0504bef6` and `0x0504d991`.
- **A line loads with the map chunks it runs through.** Its record is in section 4 of a chunk's
  file (`a/0/4/1`, LZ11), the section starting with the chunk's column << 8 | its row and 18.0;
  the loaded file the lines point at is that section. Each line is whole in every chunk it
  crosses. A point record in the same section joins two lines: `0x03052c3b`, the start of
  `0x0305ac3b`, lists `0x0305feb7` beside it. So the lab's line exists only once the player is
  within a chunk of its chunks, 40 cells or so down the boulevard from where the Lab person
  stands.
- **The field map lists each chunk's file**: a u16 a chunk at +0x803c of its block, after room for
  128 by 128 blocks of zones, row by row. Round South Boulevard it is 619 to 660, 7 chunks by 6,
  and each street section's own chunk number agrees with where the list has it. Reading those 42
  files out of the ROM takes 0.3 s.
- **North of the blackout the numbers are signed.** Zones 0x41 to 0x45 place nearly every record
  back from a line's far end with negative sizes, read unsigned as 65,530 cells each way, which
  would have had pa3ds walk billions of cells to place one door. Line `0x0004f00d`, which a door
  of zone 0x41 sits on at −46, is 47 cells long: the door is by the line's start, where zone 0x41
  meets zone 0x40. Read from the ROM only; nobody has walked there yet.
- **Lumiose's other parts branch.** Their sections hold junction lines (`line@connectT...`)
  beside the plain ones, which the chain of lines pa3ds lays out does not follow. South
  Boulevard, from Lumiose Gate to the Lab and the Pokémon Center, is plain lines end to end.
- **Built**: on a rail pa3ds reads the field map's list of chunk files and takes in every line of
  those chunks' streets from the ROM, once a map, besides the lines the game has loaded
  (`archives.Streets`, `FieldReader._read_streets`); a record's place on a line is signed, a
  negative distance counting from the line's far end and a negative size spanning back. Over the
  user's movie the first listing had 15 exits, the Lab's door among them, and P to it said "hold
  left, 2 forward, hold right, 70 forward, hold left, 1 forward". Over a test movie doing what
  pa3ds said — the first hold of left, with the camera 45° off after loading, turned the player
  the other way, and pa3ds said the way again from there, so a second — the player walked the
  70 cells and the game said "This building is the Sycamore Pokémon Lab! Let's go inside!" (out
  of sync 0).

## Spike 107 — north of the blackout: how a line is measured, its tracks and joints, and the names on the doors

2026-09-25 (`spikes/107_lumiose_north.py`; movie_x_lumious_other_section_1.ctm on the X save of
15:54 that day, kept as save_checkpoint_9; the rest from the ROM alone). The user walked through
a warp from South Boulevard into zone 0x44 and P said "no path" to all nine exits listed there,
and asked whether other warps do the same and whether a warp's destination could be named. Then:
"fix the tracker", and name every door and warp the game gives a name to, in every town.

- **The player was placed off every street.** Through the warp the player stood at (-5, 8) on
  line `0x0305c50a`, and pa3ds laid them 5 cells before the line's start, in no line's cells, so
  every way began in a wall; the exits were measured from there too ("20 north" for the warp just
  walked through, 1 cell behind). Spike 106 had read a negative distance as counted back from the
  line's far end, and **that was wrong**.
- **A line says how the game measures places on it**: +0x10 of its record is a way of the
  compass, 0 north, 1 south, 2 west, 3 east, and a place (a, x) is (row, across) as if the line
  ran that way of the world's (x, z): (a, x) on a line measured south, (-a, -x) north, (-x, a)
  west, (x, -a) east, across rising to the line's left. The line above is measured north: its
  start is at world z 4311, the player at 4221, 5 cells on, and 8 cells to the +x side of its
  middle at x 2511, which is its right. So measured, 328 of Lumiose's 329 records sit inside
  their lines and all 202 warps on a wall cell beside an open one; the three warps into zone 0x40
  that looked turned a quarter are on a line measured east. South Boulevard's lines are measured
  south, which is why spikes 102 to 106 never saw it. **People and the player are placed the same
  way**: 20 readings of models on seven lines and a track fit the world within 15 units, most
  within 3, the worst mid-step on a curve.
- **Tracks and joints.** Section 4's header has five counts at +0x08 and five offsets of tables
  at +0x14, for lines, points, tracks, joints and cameras. A track is a short line of its own
  table, +0x40 its file and +0x44 its walls, from a point to a **joint**, whose +0x08 lists up to
  eight tracks meeting there and +0x28 is where it is; a point's place is at +0x10. Both are
  placed from the middle of their chunk, 720 world units a side. The game measures a track from
  its joint, back along its rows, and a model on one heads away from the joint. A line's id is
  its start point's with bit 15 set, a track's never, which tells them apart in RAM too. Ids are
  a map's own: the same point id is in two other maps.
- **Zone 0x44's map is seventeen streets apart**, joined to each other only by warps: its 294
  lines and tracks meet at 311 points and joints, and 52 of its 53 dead ends are joints of one
  track. The player's piece has 12 warps; pa3ds listed 9, as the three into Centrico Plaza are
  past a joint. The others are reached through other zones. Laid out flat, turning at each joint
  by the quarter nearest its way in the world, no piece of any of Lumiose's maps lays two
  unjoined lines over one another.
- **The blackout stops the player's street at its end** in this save: "I'm sorry. There's a
  blackout. You can't go any farther until it's fixed." So no joint has been walked, and what
  the D-pad does at one is not known.
- **Built**: a line's way, and the track flag, turn a record's or a model's place into its row
  and offset (`rail.Line.upright`); the ROM's tracks, points and joints are read with its lines
  (`rail.section_streets`), and a map's lines are let go when the field map lists another map's
  chunks; the corridor is every line joined to the player's, laid flat, a line going on straight
  at a point and turned at a joint (`rail.lay_out`). Over the user's movie: "warp to zone 0x3f
  here" where it said "20 north", twelve exits, and P to the door into zone 0x80 "11 forward,
  hold left, 17 forward" (out of sync 0). North Lumiose is laid out as the game places it, but
  not walked.
- **A plate names the door beside it.** A zone-table row's +0x06 is the zone's file in the story
  text archive (`a/0/8/2` in English, `a/0/8/0` to `a/0/8/7` the seven languages). Section 2 of a
  zone file is its script, compiled Pawn (magic 0xf1e0, the code compact-encoded); its entry
  point switches on a furniture record's number to a function that shows the sign by one call
  with two constants, the line and a style: 5 a town's or a street's sign, 6 a building's
  plate, 7 and 8 tips, 9 a route's name. A plate names the nearest door into a building of the
  town's own place within 6 cells, or 12 for a plate of one line, with the next door at least
  half as far again; on a street, a plate and its door on one line. 45 doors, the same in all
  seven languages, their first page run together since the Japanese and Korean give a name two
  lines. Left out: the player's house, whose plate says whose it is; the neighbour's, which says
  Serena's or Calem's; Restaurant Le Wow, whose plate is on the next line; and Coumarine's and
  Anistar's Gym plates, 23 and 12 cells from their doors. Laying out a street to measure on it
  would name Le Wow, but reads 170 chunk files, 2 s of starting up with the LZ11 as it is.
- **A Gym is the facility just before the Pokémon Center** in its town's Town Map row, in all
  eight of X's Gym towns, and before the Center in no other; a warp into a Gym's room (area kind
  0x11) is named so, seven Gyms, Prism Tower's being entered through its lobby.
- **Lumiose's street names.** `a/2/5/5` holds a file of marks a zone of its streets, 0x18 bytes
  each: +0x02 the street, +0x0c a line, +0x10 its stretch as a record's. Street 1 to 12 is
  lines 2(n-1) and 2n-1 of text file 71, "South" "Boulevard" to "Centrico Plaza", and 13 no
  street; a boulevard's marks cover it, an avenue's and a plaza's are at the alleys' mouths. A
  warp record's +0x02 is the warp it comes out at in the zone it goes to, so a warp between two of
  Lumiose's zones is named for the mark nearest where it comes out, on that line, or for the one
  street all its zone's marks name: 165 warps, "warp to Vernal Avenue" into zone 0x44.
- **Built**: X's doors, 20 named before, are 72 (`plates.plate_names`, the Gym rule in
  `archives.door_facilities`), and a warp within Lumiose says its street
  (`plates.street_names`). A zone is walked as outdoors when it is its own parent, or when it is
  of its parent's place and area kind and its records are on streets or a building's way out
  comes out there; a warp off a street into one is a warp, not a door. Reading it all takes
  0.45 s of starting up. X has streets outside Lumiose too, in zones 0xbf, 0xd7, 0x108 and 0x11f;
  0xbf and 0xd7 are Coumarine's and Anistar's Gyms, whose puzzles are on rails.
- **Two mistakes the replays caught.** Paired with the Town Map like the towns' own doors,
  Lumiose's doors off its streets took the Boutique's name from all five towns that have one,
  since Lumiose lists "Boutique Couture" and no facility was then common to every town with a
  boutique: what kind is which facility is worked out from the towns' own doors alone again, and
  a door off a street takes a kind's name only where its town lists it, as its Pokémon Centers
  do. And without the area kind, the two Gyms on rails were taken for outdoors, and their warps
  for no door. Replayed after both (`regress.py`, all eleven movies, out of sync 0): what is
  said differs by the new names and by polls landing a moment apart, nothing else. Polling over
  the user's movie is as it was (`spikes/60_poll_timing.py --rom`): gaps between dialogue polls
  17.1 ms median and 39 ms at the 99th percentile, against 17.0 and 40.
- **Café Soleil was on the same street, in the other zone.** The user could not find it near the
  Lab: South Boulevard is zones 0x3f and 0x40, one street of 17 lines on map 128 (a zone-table
  row's +0x04), and the game moves the player from one to the other without a warp, so only the
  zone they stood in was listed. Café Soleil is 98 cells on from the Lab, past the Pokémon
  Center, on the Center's side. **Built**: on a street the ROM's records of every other zone of
  the same map are listed with the zone's own, each told apart and named by its zone
  (`Interactable.home`). From beside the Lab, P to Café Soleil: "hold right, 15 forward, hold
  left, 168 forward, hold right, 2 forward" (movie_x_path_to_lab.ctm, out of sync 0).
- **And a door was named for another zone's door of its number.** On the first poll in Lumiose
  the matrix put the player in zone 0x40 while zone 0x3f's data was in, and the door into zone
  0x56 was called "Café Soleil", zone 0x40's slot of the same number. A zone data file holds its
  own row of the zone table: all 360 of X's and all 536 of ORAS's, every row different from
  every other. So a record is named by the zone whose row its file holds (`FieldReader._home`),
  whatever zone the player is taken to be in. Replayed before and after (all eleven movies, out
  of sync 0): the Lumiose movies list the far half of the boulevard, and otherwise a zone's
  listing is caught a moment sooner or later.

## Spike 108 — X's people, named by sprite

2026-09-25 (`spikes/108_x_people.py`, X's ROM alone; no emulator). The user asked for as many of
X's people named as can be named confidently. X named 32 sprites, every one a kind of trainer
(spike 90's derivation), and nobody by name. ORAS's table of story people is keyed by ORAS's
sprites, and X shipped no fallback names at all: its `DATA` folder did not exist. A sprite was
taken only where three readings agree. Its id in X's sprite list (text file 76, the same in every
language), the zones it stands in, and a line of the story naming the person.

- **A zone's own story file says who stands there.** The row's +0x06 is the zone's file in the
  story text (spike 107), so counting the trainer names each zone's lines use, and reading who
  introduces themselves there, says who the one sprite standing in all of them is. The story's
  people are sprites 0x03 to 0x21 and 0x93 to 0xa0: `FRIEND1` to `3`, `HAKASE1`, `MAMA`,
  `CHAMPION`, `BIGFOUR1` to `4`, `LEADER1` to `8`, `FLAREBOSS`, `AZ`, `HANDSOME`, `RIVALM` and
  `RIVALW`, and further on `MEGAG`, `DIRECTOR`, `LEADER1_SIS` and `LEADER5_SIS`.
- **The Gym Leaders stand in their Gyms.** Each `LEADERn` stands in one zone of area kind 0x11,
  and that zone's lines name one Leader: Viola in Santalune's, Grant in Cyllage's, Korrina in
  Shalour's, then Ramos, Clemont in Prism Tower, Valerie, Olympia and Wulfric. They are trainers
  6, 76 and 21 to 26. `CHAMPION` stands in the Radiant Chamber and in Lumiose 0x7d, whose lines
  name Diantha eight times: trainer 276.
- **Two of the Elite Four name themselves, and a chef tells the other two apart.** Wikstrom names
  himself in the Ironworks Chamber, the one zone `BIGFOUR1` stands in, and Malva in the Blazing
  Chamber, `BIGFOUR2`'s. Nobody names themselves in the Dragonmark or Flood Chambers. But the
  Flood Chamber's member talks as a Chef in all seven languages ("It is the duty of both Trainers
  and Chefs...", story 106:6), Lumiose 0x7e's lines say one of the Elite Four is a famous chef,
  "he" (story 348:1), and the Battle Chateau's classes make Siebold a Duke and Drasna a Duchess
  (trainers 275 and 274). So `BIGFOUR4` is Siebold, 271, and `BIGFOUR3` is Drasna, 270. The
  sprites run in the trainer list's order too: 187, 269, 270, 271.
- **The three friends.** `FRIEND1` stands without the other two in Vaniville, on Route 2, in
  Lumiose 0x45 and in the Flare HQ, and those zones' lines name Shauna and never Tierno or
  Trevor. `FRIEND3` stands alone of the three in Frost Cavern, where "My name is Trevor." In the
  Tower of Mastery `FRIEND2` and `FRIEND3` stand together where "Uh, my name is Tierno." and "My
  name is Trevor." are both said, so `FRIEND2` is Tierno. Trainers 321, 324 and 325, the ids'
  order again.
- **Sycamore, Lysandre and AZ.** "I'm Professor Sycamore!" is said in the Lumiose lab `HAKASE1`
  stands in. `FLAREBOSS` stands in Lumiose and Lysandre Labs where Lysandre is named, and
  `FLAREBOSS2` in the Flare HQ alone. "That is AZ." is said in Lysandre Labs, where `AZ` stands.
  Trainers 327, 303 and 602.
- **The rival stands in no zone file.** A script draws them into a work object (+0x890, spike
  100). The ids say `RIVALM` and `RIVALW`, and the neighbour's lines say whose child is whose in
  every language: "We have a daughter... our Serena", "a son... our Calem" (story 53:0 and 53:1).
  So `RIVALW` and `RIVALW_EX` are Serena, 575, and `RIVALM` and `RIVALM_EX` are Calem, 130, each
  language's trainer line being the word its own story uses (Kalem, Kalm, カルム). No replay has
  shown them drawn yet.
- **Six people the ROM never lists as trainers**, each read off one line in all seven languages.
  Mom is `MAMA`, alone in the player's house ("Mom's words echoed...", system 140:43: ママ, Maman,
  mamma, Mama, 엄마). Looker is `HANDSOME`, which is his Japanese name, ハンサム ("It's me!
  Looker!", system 102:16). Alexa is `LEADER1_SIS` in Santalune: "I'm Alexa. The Gym Leader you
  just faced, Viola, is my younger sister." (story 421:10). Bonnie is `LEADER5_SIS` in Prism
  Tower: "You can call me Bonnie. I'm this Gym Leader's younger sister." (story 404:0). The Mega
  Evolution guru is `MEGAG` in the Tower of Mastery (story 281:6), and the Game Director is
  `DIRECTOR` in Coumarine (story 272:0). **The Spanish text never gives the player's mother a word
  of her own.** Every line about her says "tu madre", so Spanish has no Mom.
- **The nurse** is `PCWOMAN1`, at the counter of all sixteen Pokémon Centers (area kind 0x10), and
  also the Battle Maison's receptionist and one in the Pokémon League. ORAS's `PCWOMAN_RS` stands
  the same way, in sixteen Centers, the League and the Battle Resort, and ships as "Nurse". X's
  does too, pa3ds's own English as ORAS's is.
- **Left out.** `FLAREKANBU1` and `2` have five and four records in the Flare HQ, a look the
  admins share. Aliana introduces herself in the Kalos Power Plant, where `FLAREKANBU1`,
  `SCIENTIST5` and `MYSTERYW` all stand. `SCIENTIST1` to `5`, `MYSTERYM` and `MYSTERYW` are never
  named where one stands alone; Celosia is named in a zone `SCIENTIST3` and `4` share. `C02GIRL`
  and `C02GIRL_EX` stand where Looker does, and the Looker Bureau's lines name Emma and make her
  Essentia in the Expansion Suit (story 136:44), but that file is no zone's, and the zones the two
  sprites stand in never name her. `DANPEI` stands in all eight Gyms, and no line there gives him
  a word. `SHOPM1`, the clerk in all sixteen Centers, also stands in Aquacorde's outdoors and
  eleven other zones. `GIANTMAN02`, `KIZUNA`, `DELIVER`, `BUTLER` and `MAID` have no word, and
  `HERO` and `HEROINE` are the player.
- **Built**: `xy.CHARACTERS`, 24 sprites as 21 people, passed to X's `RomLayout`, so X names 56
  sprites where it named 32, read at every start as ORAS's are. And
  `games/xy/data/<language>/sprites.toml` in X's seven languages: the six people above, the
  Nurse in English only, no Mom in Spanish, and the 56 the ROM names, copied by `table`.
  Replayed with spike 109's change (`regress.py`, all eleven movies before and after, out of sync
  0): movie_x_city.ctm now names the Nurse and Tierno, and movie_x_lumious_other_section_1.ctm
  Shauna and Trevor, where each was a bare "person"; no movie reaches a Gym Leader or the rival.

## Spike 109 — ORAS's plates, and the town's own word for a plate with a slogan

2026-09-25 (`spikes/109_oras_plates.py`). The user asked whether X's doors named by their plates
(spike 107) are named so in ORAS too, and then to label as much as can be labelled confidently.
They were not: ORAS's `RomLayout` had no story archive, so its plates were never read, and
ORAS named 38 doors by its AreaNav alone.

- **ORAS has X's plates.** Its zone scripts show signs by the same call, style 6 a building's
  plate, and the story archives are `a/0/7/9` (kana) and `a/0/8/1` to `a/0/8/6`, as spike 91
  found them. X's pairing, unchanged, names 66 of ORAS's doors, 57 of them doors the AreaNav
  does not: every Gym but Petalburg's (which the AreaNav names), the Trainers' School, Mauville's
  shops and apartments, Lilycove's Department Store and Cove Lily Motel, the eight cabins of the
  S.S. Tidal, the houses of Wanda, Cutter, Lanette and Mr. Briney. 95 doors in six languages,
  94 in Spanish.
- **Where both name a door they agree.** Nine doors are named by the AreaNav and by a plate, and
  in all nine the plate is the building's: "Pokémon Lab" and "Professor Birch's Pokémon Lab",
  "Contest Spectacular Hall" three times beside "Pokémon Contest Spectacular! Registrations
  accepted inside.", "Petalburg City Gym", the Seashore House, the Glass Workshop, the Day
  Care and the Treasure Hunter's House. The AreaNav's word is kept where it has one.
- **ORAS's plates carry slogans.** X's are mostly the name alone; ORAS's put a line before or
  after it: "Dewford Hall / Everyone exchanges information here!", "“Ride in Gravel and Shake Up
  Your Soul!” / Rydel's Cycles", "Our family's hearts beat as one! / The Winstrates' House". Route
  110's plate by the Trick House is directions to it, and in English the Oceanic Museum's first
  page is its motto, the name on the next page (French and Spanish put the name first).
- **So a plate of more than one line is named by what its town's list calls it**, where it holds
  exactly one facility of that list, spaces left out of the match: "Dewford Hall", "Rydel's
  Cycles", "Trick House", "Mauville Food Court", "Mossdeep Space Center". A first page of one line
  is kept whole, since it is the name alone and may say more than the list ("Move Tutor House",
  "Pokémon Trainers' School"). The match is made in the player's language and holds in some
  languages where it does not in others: the Winstrates' is matched in Japanese and not in
  English, whose list says "The Winstrate House".
- **What that changes in X**: six plates lose their slogan in English ("Pokémon Day Care", "The
  Fishing Shack", "Fossil Lab", "Trainers' School", "Hillcrest Station"), five to six in the
  other languages, and Sycamore's lab, whose English and Japanese plates are "Sycamore / Pokémon
  Lab" on two lines, is "Pokémon Lab" as the Town Map says. X still names 72 doors in all seven.
- **Left as they are.** Plates with a slogan and no facility of the list keep their whole first
  page ("Move Maniac's House Let me teach moves to Pokémon.", Mauville's shops but Rydel's and
  the Food Court, the Old Lady's Rest Stop, the Fossil Maniac's). The English Oceanic Museum door
  is "The endless sea sustains all life.", and Route 121's gate "Welcome to the Safari Zone!":
  what the plate says, not the building's name, and the one place a rename is the fix.
- **Built**: ORAS's `RomLayout` reads its story archives, and `plates.plate_names` takes each
  place's facility words (`archives.town_rows`, the town list's rows read once for the doors and
  the plates). Starting up reads ORAS's text in 0.50 s where it took 0.39 s.
- **Replayed** with spike 108's change, all eleven movies before and after, out of sync 0 in every
  one. What is said differs by the new names and by polls landing a moment apart, nothing else:
  map_chunk.ctm names Roxanne's Gym, the Pokémon Trainers' School, Cutter's House and the
  Tunnelers' Rest House, hidden_tail.ctm Mr. Briney's Cottage, and the X movies in Lumiose call
  Sycamore's lab "Pokémon Lab".

## Spike 113 — Sun's field

2026-09-26 (`spikes/113_sun_field.py`). The user: make the map usable in Sun as it is in ORAS and
X. Every navigation key said "Not read in this game yet". sun_3.ctm stands in the player's
bedroom and never walks, so the walking came from sun_1.ctm, from 340 s: the player gets out of
bed and walks across the room to the desk, where sun_3's save was made. Both replays ran on
Vulkan with no out-of-sync line.

- **The field is objects in the small allocations**, as the dialogue is (ADR-0035), found by
  RTTI: `Field::MoveModel::FieldMoveModelManager` (0x24c), 32 `FieldMoveModel` (0x494) of which
  the player's is `FieldMoveModelPlayer` (0x780), `Field::Fieldmap::State`, and an actor for each
  thing on the map: `FieldBgEventActor` (nine in the bedroom), `FieldMapJumpActor` (its door),
  `FieldMoveModelActor` (its three people) and `FieldStaticModelActor` (two props), 0x24 to 0xec
  bytes each. gfl2 puts its own 0x20 bytes before every object in a block.
- **Where each is.** The heap tree, walked by the heads: heap 1 (41 MB from `0x30000000`) holds
  heap 5, which holds 0x20 (the field's states), 0x21 (the field) and inside it 0x45 (the
  actors) and 0x44 (the area's files, 11.5 MB). Heap 3 (27 MB, `0x328690a4` to `0x342dc000`)
  holds the move models, their manager and the area loaded among its own blocks, and heaps 6,
  7 and 0x0b under it, 0x0b being the dialogue's. So the models need heap 3 walked; the rest was
  already under heap 5.
- **The manager lists the models**: +0x0c the player's, +0x10 to +0x88 the other 31. A model
  in use has its scene node at +0x20, zero in a free slot, and its model code at +0x6c: 0x77,
  0x04 and 0x21 for the bedroom's three, as their records in the ROM have them.
- **A model's place is +0x3a4, not +0x30.** +0x30 (x, y, z) is where the save put it: zero for
  every model all through sun_1, and the player's saved place in sun_3. +0x3a4, +0x3a8 and +0x3ac
  walked with the player, from (2411, 2418) by the bed to (2126, 1978) at the desk, about 400
  world units a second, with the place a frame before at +0x3b0. The people's +0x3a4 is their
  record's place in sun_3, where nobody moves; walking people have not been watched.
- **The field has no class of its own.** Every `Fieldmap::State` points at it (+0x14), a 0x1a0
  block at the same address in both replays: +0x7c u32 the zone, 0x1d in the living room at
  300 s of sun_1 and 0x1a in the bedroom from 392 s, and in sun_3; +0x08 the zone again beside
  0x10, with the place the field was entered after it (not where the player is); +0x104 the
  player's place again, a frame ahead of the model's. +0x78 points at the **area** loaded, which
  has no class either: +0x00 u32 21, then eleven pointers to that area's files in heap 0x44.
- **The actors hold their records' places.** +0x14, +0x18 and +0x1c are x, y and z, the same
  floats, to the bit, as the record in the ROM each was built from; +0x20 points at that record
  in the area's event data, a block of 8.4 KB in heap 0x44 that is not read.
- **The ROM.** The zone table is `a/0/7/7` file 0, LZ11, 0x54 bytes a zone and 295 zones, the
  same bytes as a block the game loads; +0x1c is the zone's line of system text file 67, the
  place names ("Route 1" for the house's four zones, 0x1a to 0x1d). `a/0/8/2` is eleven files an
  area, the area's events first (magic 'ED', then 'BG', 'TR', 'AC', 'AS', a table of letters,
  'AE', 'ZS', 'ZI', 'EA', 'BG'), 240 areas; the area in RAM is the file's number over eleven.
  An 'ED' is sixteen sections, each a part a zone of the area in the area's order; section 1
  the people, 2 the warps (0x58 a record), 3 the things read by facing them (0x3c), each part a
  count, its records and then the shapes they point at: a box, a circle or a line. A warp's
  +0x34 is its zone and +0x36 the zone it comes out in: the bedroom's door to 0x1b, the living
  room's front door to zone 0, which is Route 1. Its +0x2c and +0x30 number it and the warp it
  comes out at within its area only: 13 numbers serve the whole game. Reading the archive and
  the 240 areas takes 0.3 s.
- **Walls are meshes.** `CollisionMeshResourceNode`s are small, the meshes they point at are
  not, and nothing says which cells are open: P says there is no route in Sun.
- **Walking heap 3 costs a third more**: 36 heads and 2,533 small blocks (597 KiB) at 80 s of
  sun_3, beside heap 5's 133 heads and 5,718 blocks (993 KiB); 169 heads and 8,251 blocks
  (1.6 MB) together. Spike 112's capture had read every small block of every heap every 2 s for
  100 s, and nothing went down; neither did 33 walks of heaps 3, 5 and 0x0b over sun_1, nor two
  runtime replays of sun_3.
- **Two faults found on the way.** With 0x0b a wanted heap and under heap 3, the walker met it
  first as a root and never followed its siblings: 34 of heap 3's heaps were lost, 6,316 blocks
  walked of 8,251. And a sweep between whole sweeps reads only the blocks near what was found
  before, so a reader that may read only what a sweep read lost the field and the area loaded:
  the keys said "no map" in the bedroom, now and then. Both fixed (`heaps`, `field`).
- **Built**: `games.sm.field` (the player's zone and cell, the people, the things and the doors,
  a cell being 50 world units), `games.sm.zones` (the ROM's zone table, place names and every
  area's warps and things), heap 3 walked beside the dialogue's, and the keys answered by ORAS's
  navigator, P saying "no route in this game yet". Over sun_3 the runtime logs `field place:
  Route 1`, the bedroom's nine things, its door and three people, and each key pressed answers:
  "sign, 1 of 13, here", "exits, 1", "warp, 1 of 1, 2 east 5 south", in direction mode "person,
  1 of 9, 2 east 2 north" north and "nothing west" against the wall.
- **Not read**: items on the ground (`FieldItemActor`, which no capture has held alive), trainers,
  anyone walking, an outdoor zone and where one zone of an area meets the next, the way a model
  faces, and names for people and things: nothing names them yet, and a player's own names are
  kept as in ORAS.

## Spike 114 — Sun's walls, out of the ROM

2026-09-26 (`spikes/114_sun_walls.py`). Spike 113 left P saying "no route in this game yet". The
ROM alone was read, and checked against ground the player is known to have stood on: the walk
from the bed to the desk in sun_1.ctm, outside the house on Route 1 at cells (561, 362) and
(561, 363), and Iki Town at (368, 286).

- **The walls are in `a/0/8/6`**, 1,300 files, a 'BG' pack a **map block** of 4,200 world units
  square (84 cells): entry 6 the floor mesh, 7 a pack of four wall meshes. A mesh is `00 05 12
  14`, a u32 count, and triangles of 0x48 bytes: three corners, the normal, the ground's kind
  (tall grass 31, the open sea 8, 0xffffffff on a wall) and flags. Wall mesh 0 is every wall,
  fence and cliff; 1 runs along the sea's edge; 2 is short boxes with lids, one across a way in
  Iki Town with a person standing in it (a barrier the story lifts, taken as a wall); 3 was empty.
- **An area's third file of `a/0/8/2` ('TR') is its block table**: width and height in blocks,
  the block's side, and a 12-byte record a block whose first u16 is its file in `a/0/8/6`. An
  area lists its own blocks and its neighbours', so an area's ground is read whole: Route 1
  beside the house is block (6, 4) of area 0's table, Iki Town (4, 3) of area 1's.
- **A cell is open** where floor lies under any of nine points of it and no wall comes within
  half a cell of its centre; a wall exactly on the line between two cells closes the one behind
  it, or a way stepped through the bed's side. Floor runs on under the bed and under houses, so
  P's ground closes every **pocket** of open cells but the one the player stands in. The walls'
  normals face the walkable side indoors and not always outdoors, so they are not used.
- **Against the known ground**: 9 of the bedroom walk's 11 places are open, the other two the
  player lying in bed; Route 1's two and Iki Town's are open. From the desk P reached 13 of the
  bedroom's 13 records, from outside the house 50 of Route 1's 56 in 0.64 s at worst (a way of
  600 cells), from Iki Town 47 of 48. Two of Route 1's misses are things just behind a fence,
  more than a cell from open ground; Iki Town's is the person in the barrier.
- **Cost**: an area reads in 0.25 s (the house) to 0.8 s (Route 1) and holds 0.4 to 1 MB; its
  chunks take 0.04 to 0.13 s the first time. The runtime reads an area's walls on a thread of
  its own as the player arrives (`SunWalls.prefetch`): Route 1's first P had taken 2 s.
- **Replayed** on sun_4.ctm's save states, P said "9 south" to Iki Town's way back to Route 1,
  "4 west" to a thing and "16 east, 32 south, 32 west, 4 south, 2 west" to an item ball across
  Route 1, and guided along each.
- **Not read**: heights (where one floor passes over another both count, and every wall of
  either), and which of a block's alternates the story loads (the first is read).

## Spike 115 — What the things on Sun's field are

2026-09-26 (`spikes/115_sun_things.py`, the ROM alone). Replaying the user's walk from the house
to Iki Town (sun_4.ctm), pa3ds said every warp "warp", every thing "sign" and every model
"person", and in Iki Town "warp to zone 0x1 (Route 1) 4 north" and "sign 3 west 5 north" twice.

- **Every record opens with a u32 saying what it is**, then x, y, z and a quaternion; +0x2c
  numbers it within its area. People are 0x78 bytes, not 0x58, their model code at +0x34.
- **A warp's type is its +0x38.** Warps pair by the warp they come out at: a door (1) and the mat
  out (9) 100 times; an opening with no door (8), a cave's mouth or the way onto Mahalo Trail,
  194 times; a line walked across into the next area (0), Route 1 into Iki Town, 84 times; doors
  between rooms (1 and 1) 40; up and down (3 and 4) twice. pa3ds says them in ORAS's words:
  door, way out, path, stairs up and down (`sm.field`).
- **Iki Town's "4 north" was right.** Route 1 and Iki Town share one world, and the zone had
  changed while the player was still between Route 1's line and Iki Town's, during Kukui's scene.
- **A thing with no script and kind (+0x34) 0x0e, 0x12 or 0x15 is a signpost**: Route 1's seven
  are the seven the game's sign text has for it. Anything else is an object. Iki Town builds
  five of its things twice, for two scripts, and pa3ds lists a place once.
- **Items are section 10**, 0x40 bytes: +0x30 the item, +0x38 how it lies (2 a ball on the
  ground, 0 hidden). The field builds a `FieldItemActor` for each ball still there, its place at
  +0x14 and +0x1c as the other actors': seven on Route 1's second zone, each at its record's
  place to the bit.
- **The zone table's +0x3a is the kind of place** (3 outdoors, 0 indoors, 2 a cave, trail or
  ruin), and an area is named by its warps' zones, not by the table.
- **The field's zone is a u16.** On Route 1's second zone +0x7c read 0x10001, which pa3ds took for
  no zone, and every key said "no map" from there to Iki Town.
- **Not read**: trainers (section 7's +0x30 is not the trainer's number), what a sign says (its
  script chooses the line), hidden items (behind parity, and not listed), and warp type 7.

## Spike 117 — People and items on Sun's field that nobody sees

2026-09-26 (`spikes/117_sun_unseen.py`). The user, playing their own save: P led them to a person
in the house where nobody was, and outdoors to an item ball where there was none. sun_4.ctm was
replayed to the living room (27 s) and the porch outside (36 s), every model read with a
screenshot at each.

- **The manager lists models that are never drawn.** The living room listed seven people, and the
  screenshot shows two, Mom (code 0x21) and Meowth (0x77). The other five: one in the player's
  own cell (code 5), one by the counter (4), and three in a row 25 to 33 cells east (0x8c to
  0x8e), which spike 114 had found standing where no floor is. On the porch four models stood
  within 8 cells, and the screenshot shows nobody.
- **+0x308 is 1 on every model drawn and 0 on every one that is not**, over both captures, 32
  models: the player, Mom, Meowth and the route's people far off. The undrawn also hold 0xffff
  at +0x30c and +0x310 where the drawn hold a small number, 0 at +0x318, and 100,000.0 at
  +0x3c8: their model is not loaded at all. It is not the camera's: people 40 cells away read 1.
  pa3ds lists a person only while +0x308 is 1, and reads a model's head and place in one read.
- **The ball was a hidden item.** The field builds a `FieldItemActor` for a hidden item as for a
  ball: three of the seven beside the house were hidden (spike 115's records, kind 0), among them
  the one P led to, beside the house. Only balls are listed.
- **Replayed**, the living room lists Mom and Meowth, and the porch nobody near and four balls.

## Spike 118 — Sun's props, and a way that ended inside a flower pot

2026-09-26 (`spikes/118_sun_props.py`). The user, with sun_5.ctm: P to a thing on the porch
outside the living room "goes weird", and they wondered whether a step is not what it is in ORAS
and X. Replayed with their keys (from their log), the player's place logged at every change and a
screenshot every half second.

- **The way was right, and ended where no one can stand.** "9 south, 2 west" took the player to
  two flower pots at the porch's end. The thing read there is a record a unit wide; the pot round
  it is a `StaticActor` holding a `BoxShape` 104 units a side, the player's body a cylinder of
  37 units round. The way ended at the cell beside the record, inside the pot. The player stopped
  42 units from its box, P said "2 south, 2 west" and never that they had arrived, and found a
  new way at every step they took after: "5 south", "12 south".
- **The step is not what went wrong, though it is not ORAS's either.** A cell is 50 world units;
  the player walks about 210 a second, so a cell is about a quarter of a second, as a step is in
  ORAS. But the player's body is about a cell and a half across, and a thing is as wide as its
  prop, not its tile.
- **Props are in the small allocations.** A `StaticActor` holds at +0x28 its shape and at +0x30
  and +0x38 its place: a `BoxShape` its offset at +0x04 and +0x0c, its two axes on the ground at
  +0x10 and +0x1c, and half its width, depth and height from +0x28; a `CylinderShape` its offset
  and its radius at +0x10. The walls are `MeshShape`s at the world's corner, the ROM's
  (spike 114); the door's warp is a `LineShape`.
- **A prop is solid only while its +0x0c is set**, the collision scene's entry it is in. The two
  cylinders of 40 and 80 units round two people nobody sees (spike 117) had none, and the player
  walked through one; the pots' had theirs.
- **Built**: the ground P walks closes every cell within half a cell of a prop in the collision
  scene, as the walls' are closed, and a way to a solid thing ends up to three cells out from
  it, the first open cell from each side (`Ground.reach`, 1 in ORAS). Replayed, P said "2 west,
  7 south" to the pot, round the other one, and "object, 2 south" on arriving. In Iki Town the
  ways ended a cell short of what they had walked into before: "3 west" to a thing 4 west.
- **Not known**: whether an event's trigger is ever a prop in the collision scene, which would
  close cells a player can walk through.

**Again over sun_6.ctm** (the same save; the first of the user's movies recorded with
deterministic async operations, and it replays only so: without, the player never left the
living room). The user walked out of the house, through Kukui's scene, and tried to get home with
P. Replayed with their keys, P's ground dumped at each press:

- **Every collision group had been taken as solid, and was not.** A prop's +0x0c is its group's
  first entry, and the groups are: the bodies of people and props (radius 40 round a person, the
  flower pots); the ranges they are talked to from (80 to 160); warps' lines; boxes as big as a
  hill (the camera's); and five shapes that include a box 950 units long across Route 1, where
  the user had walked into Kukui's scene. Taken as solid, a box round the player at their own
  door left them nowhere to start from, and P said "no path" to everything on Route 1; the long
  box sent the way home round Route 1, 17 legs for a house 31 cells east. Only the bodies' group
  is solid now, found as the group of the smallest cylinder where a drawn person stands (the
  same object from one area to the next), and no prop round the player's own cell. P said "1
  east, 17 south, 32 west, 7 south, 14 west" to the door it had no path to, and "6 east, 11
  south, 28 east, 13 north, 1 west" home, the way the scene had walked the player.
- **A zone is not a warp.** Route 1's two zones are one area, and the user walked across the line
  between them over and over: each time the way was left as though through a door, and the
  thing it led to was "gone", its key holding the player's zone. A warp is now a new area or a
  new model for the player, and a thing's zone its record's.
- **The field keeps only the zone's actors the player is in**, and changes them over a sweep
  before its own zone word changes. The things of the area's other zones are kept as last seen
  until the area changes, and one is gone only when a thing of its own zone is there and it is
  not (a ball picked up). P home held across the line between Route 1's zones.

**Again over sun_7.ctm** (the user's save of 16:12, kept as `user/save_sun_7`, recorded with
deterministic async operations): the user stood on the line between Route 1's two zones and
walked over it and back, asking for the two to be one map, "cause there is no way for me to know
where is the invisible line".

- **An area is one map now.** Its exits and things are listed whichever zone the player is in:
  a zone's own actors where the game has built them, and the ROM's records for the rest (item
  balls only from their actors, since one may have been picked up). From the second zone the
  exits were Route 1's nine, in the same order as from the first, and P led to the house's door
  across the line.
- **The game keeps some of a zone's actors and not others**: back in the first zone its things
  were there and the second zone's paths not. A zone's exits, things and balls are each taken as
  loaded only while any of that kind is.
- **Places by name.** Route 1's zones have place lines of their own, both "Route 1": pa3ds said
  "Route 1" at every step over the line, and the house's door from the second zone was "door to
  Route 1". The game's own banner still says "Route 1 Hau'oli Outskirts" and "Route 1" as the
  player crosses, as it shows them.
- **Indoors an area's zones are rooms, not one ground** (the user, 2026-09-27, in the porch,
  zone 0x1b): the house's four rooms, zones 0x1a to 0x1d, are area 21 laid out far apart on one
  set of coordinates, and listing the whole area gave the porch seven exits, its own four and
  the other rooms' mats back to it 157, 174 and 330 cells away. The area is folded into one map
  only where the player's zone is outdoors (zone table kind 3); indoors only the player's own
  zone is listed. Caves and trails (kind 2) are unmeasured and take the indoor rule.
- **The player drifts off a way made of cells.** Their body is a cell and a half across and the
  stick walks them anywhere, so every step a cell off a straight leg found a new way and said it,
  several times a second. A player within two cells of the way is walking it
  (`Navigator.slack`, none in ORAS).
- **After a door, the model stands where it was.** For about two seconds the area loaded is the
  new one and the player's model still where it stood in the old one; the field is not up until
  it has moved.

## Spike 119 — Which way up walks in Sun

2026-09-26 (`spikes/119_sun_camera.py`). The user, with sun_5.ctm: on the porch outside the
living room the door back in was straight up from them, and pa3ds said it was "2 west". It was
west, in the world: pa3ds's compass is x east and z south, and in ORAS, where the camera always
looks north, north is up. Every camera object in the small allocations was read in the living
room and on the porch, with screenshots.

- **The porch's camera looks west.** The house's wall is its west side, x 10,900, across the top
  of the screen; the railing its east, x 11,105, where the player stopped at 11,068, their body
  37 units round; the door its north-west corner, at the screen's top right; the flower pots its
  south, at the screen's left.
- **`Field::Camera::UiDeviceTranslator` +0x08 is the angle the pad's up is turned by**: up walks
  the world's (sin a, cos a). pi in the living room, north; -1.934 on the porch, west-north-west.
  The camera's follow controller turns by the same 69 degrees at its +0x14, and the camera's
  quaternion (`poke_3d::model::BaseCamera` +0x94) with them.
- **Elsewhere**: pi in Iki Town; on Route 1, as Kukui walks the player, between -158 and -179
  degrees, the camera easing round with them.
- **Built**: pa3ds turns its compass to the quarter the camera looks nearest (`games.sm.camera`):
  north is the way up walks, east the way right does, as in ORAS, and every cell the field says is
  turned by it, the player's, everything around them, the walls and the props P walks round.
  Things are still named and renamed by their places in the world. Replayed, the door back in is
  "path, 1 of 1, 2 north", and the flower pots "10 west", at the screen's left.
- **Not handled**: a camera between two quarters, as Route 1's at 20 degrees, walks the player a
  little off each leg, and a player who leaves the way is given a new one; a camera swinging past
  45 degrees turns the whole compass.

## Spike 126 — which way a ledge drops

2026-09-27 (`spikes/126_ledges.py`, issue 3). The user recorded omega_ruby_ledges.ctm going down
Petalburg Woods' ledges, on his save of 2026-09-26 18:21, kept as the rig's
`user/save_checkpoint_10`. The spike runs the runtime over it and logs every change of the
player's cell with the words of the cells it crossed. The replay followed, 0 out of sync.

- **A jump moves the player's cell two on in one poll, never onto the ledge.** Three jumps, all
  south, (28, 31) to (28, 33), (28, 42) to (28, 44) and (28, 53) to (28, 55), each over a cell of
  `0x75020021`: blocked, kind 0x75, as Route 101's `0x75000021` is.
- **Four kinds are ledges, and the game says which way each drops.** The function at
  `0x00402e74`, reached through a table at `0x006f8bbc`, turns a cell's word into a way:
  `0x74` 0, `0x75` 1, `0x73` 2, `0x72` 3, then `0x7c`, `0x7a`, `0x7d` and `0x7b` 4 to 7, and 9
  for anything else. 0 to 3 is the engine's north, south, west and east, the order a rail's
  record numbers its way in (spike 107), and the movie's 0x75 is 1, south. So `0x74` drops
  north, `0x75` south, `0x73` west and `0x72` east; 4 to 7 are presumably the diagonals.
- **The ROM agrees with the shapes.** Across all 819 chunk files of `a/0/3/9`, kind 0x75 (927
  cells: `0x75020021`, `0x75640021`, `0x75000021`) only ever runs east to west with open cells
  north and south; 0x72 (135) and 0x73 (220) only north to south with open cells east and west.
  No chunk has 0x74 or any of 0x7a to 0x7d.
- **West confirmed by a second movie.** omega_ruby_ledges_2.ctm, on the user's save of 23:29 in
  the pocket on Route 104 below Petalburg Woods' east exit (the rig's `save_checkpoint_11`),
  steps (143, 113) to (142, 113) and jumps to (140, 113) over `0x73020021`: west, as the
  numbering says. It jumps within seconds of the field loading, before a sweep at full speed
  finds the player, so it is replayed at a quarter speed (`--frame-limit 25`).
- **East confirmed by a third.** omega_ruby_ledges_3.ctm, on the user's save of 23:48 on Route
  102 (the rig's `save_checkpoint_12`), jumps (174, 133) to (176, 133) over `0x72020021`: east.
  Every way a ledge drops in ORAS has now been jumped; north, `0x74`, is never used in ORAS.
- **Built** (`mapblock.LEDGES`, `field.Ground.ledge`, `pathfind.find`): a path may cross a
  ledge the way it drops, from the cell before it to the cell after, which must be open. From
  the movie's first cell, (28, 20), to the woods' south exit, P said "7 south, 11 west, 7 south,
  22 east, 15 south, 22 west, 10 south, 11 east, 4 south" in the user's own session, and now
  says "43 south", the way the movie walks. From that pocket on Route 104 to Mr. Briney's
  Cottage it says "3 south, 14 west", over the ledge, where with ledges as walls there was no
  path at all. The guiding does not count a ledge on the way as a wall that has appeared since.
- **A sandy slope is open, and cannot be walked up** (2026-09-28). The user was stopped going
  west in Granite Cave 1F while P said "3 west, 5 south, 14 east, 1 south" to the warp to zone
  0x4f, and recorded omega_ruby_cave.ctm on the save of 14:23, kept as the rig's
  `save_checkpoint_13`. Replayed with 0 out of sync, by a copy of this spike that also took a
  picture every half second (not kept): the player at (10, 29) turns to face west and their place
  in the world never changes. The two cells west, (9, 29) and (8, 29), are `0x69000020`, bit 0
  clear, the only cells of kind 0x69 in the cave; the picture has a sandy slope there, rising
  west from the player's floor onto the ground the warp is on. On foot only the Mach Bike rides
  up one; the player has one badge. The movie then meets a wild Zubat and ends.
- **Built** (`field.SLOPES`, `pathfind.find`): a slope is stepped onto, along and off only the
  way it goes down, 0x69 east. Over the replay's cells, P from (10, 29) to that warp says "no
  path" where it said the recording's "3 west, 5 south, 14 east, 1 south"; to Route 106 and the
  stairs up it says what it did, and from the ground above, (5, 33), it walks down the slope,
  "4 north, 5 east". **Not known**: the kinds of slopes going down any other way, whether a
  slope is walked down on foot or slid, and whether the player is on the Mach Bike.

## Spike 132 — Dewford Gym's conveyor belts

2026-09-28 (`spikes/132_belts.py`). The user was sent "26 north" to Brawly and could not go
north: omega_ruby_fighting_gym.ctm, on the save of 15:01 in Dewford Gym (zone `0x1ac`, the rig's
`save_checkpoint_14`), replayed with 0 out of sync.

- **A belt carries the player back.** From (17, 30) the player walks north onto (17, 29), and
  their place in the world runs back to (17, 30) with them still facing north, every time. The
  picture has a dark strip across the lit corridor there. ORAS's Dewford Gym is a dark exercise
  gym whose central hallway has conveyor belts that stop the player going on, and whose way on is
  a maze of dark side passages that floor switches light up (Bulbapedia, "Dewford Gym").
- **The cell table does not say so.** Every cell of the belt is `0x00000020`, plain floor.
- **The zone's triggers do.** The entity table has seven: three cells wide across the corridor
  at z 29, 22 and 14 (scripts 9, 0xa and 0xb), and at z 10 (script 8), and one cell wide at
  (17, 31), (17, 25) and (17, 18) (scripts 5, 6 and 7), where the picture has red floor
  switches.
- **Nothing in memory holds a belt's cells.** The code past the executable, read once the gym is
  up, has `field::FieldUniqueSequenceGymFighting` (the gym's own object; other zones' slots hold
  `FieldUniqueSequenceNone`), `field::EventGymFightingMove` and
  `field::mmodel::ActionCmdWalk_BeltBack`, which by their names are the event a belt runs and the
  walk back it gives the player. The one live gym object holds material and model animations and
  no cells. The event's class shows up on the heap only as code, where a DLL is loaded.
- **A trigger's fourth u16 is probably the way it is walked onto.** Across all 361 triggers in
  `a/0/1/3` it is 0 (235), 5 (28), 6 (93), 7 (2) or 8 (3); every pair of trigger cells stacked
  north and south is 6 over 5 (14 of 14), and every pair side by side 8 then 7 (2 of 2). So 5
  north, 6 south, 7 west and 8 east, each cell firing for a player coming in from outside the
  pair, and 0 any way. The belts are 0.
- **Its third u16 is probably a work value that arms it.** The belts have 0; the switches have
  `0x40b3` and the trigger at z 10 `0x40b2`. 134 triggers have none and 227 have one. Which value
  of the work arms a trigger, and where the save keeps the works, is not read.
- **What P needs.** No way to Brawly crosses a belt: the walkthrough's way through the side
  passages ("up twice, left to the wall, down twice, left twice, up to the wall and right to the
  light") is P's own with the three belts walked round and the rest crossed, "7 west, 5 north,
  11 east, 2 north, 3 east, 5 north, 7 west, 3 north, 5 west, 2 south, 2 west, 5 north, 7 east,
  8 north". A price of 1,000 on each cell of a trigger with no work value, where P can go round,
  finds the same way with nothing known about the gym; the same price on every trigger steps
  round the switches too, in four more legs. With the belts as walls and the trigger at z 10 as
  one as well there is no path at all.
- **Not known**: whether a belt carries a player walking south along it, whether 0 in the third
  field means a trigger fires every time, and what the scripts run.
- **Built** (`oras.BELTS`, `gen6.field` reading the zone's triggers, `Ground.belts`), taking it
  that a belt lets a player walking south onto it through (the user, 2026-09-28): a belt is
  walked only the way it runs, as a slope is. Nothing in the record says a trigger is a belt,
  so the game knows each by its zone and script. In the replay the field reads the nine cells
  of the three belts from the zone's triggers, and P from (17, 30) to Brawly says the
  walkthrough's way above, and from Brawly back to the way out "39 south", down the belts.

## Spike 136 — water, and the way across it with Surf

2026-09-28 (`spikes/136_surf.py`, and both ROMs' map chunks and text). The user, on Route 109
(zone `0x20`), pressed P to a Fisherman across the sea: "71 south, 50 west", then "The water is
deep, deep blue." at the shore, and the guiding kept sending them south into the sea once all
four chunks had loaded. omega_ruby_surf.ctm, on the save of 15:54 (the rig's
`save_checkpoint_surf`), replayed with 0 out of sync.

- **The sea is not blocked.** Route 109's sea is `0x3d1a0006`, 2,688 of the cells loaded, its
  blocked bit clear, so P took it for floor. The beach is `0x02` (dry sand, `0x025a8020`) and
  `0x07` (the wet strip along the water, `0x075aa020`), the only kinds the player stood on.
- **Bit 1 of a cell's word is water.** Across every chunk in both ROMs (`a/0/3/9`, `a/0/4/1`) it
  is set on every word of nine kinds and on no other. ORAS: `0x3d` (78,256 cells, the sea),
  `0x41` (8,630, in 3-by-3 patches inside the sea, Dive's by the look), `0x42` (2,875, bordering
  only the sea and walls), `0x94` to `0x97` (four kinds, one a way as ledges are, so currents by
  the look: 11,729 cells in all, 0x95 most), and `0x3f` and `0x40`, which carry the blocked bit
  as well. X: `0x3d`, `0x3f` and `0x40`. Only `0x3d` has been seen in the game.
- **The move names** are text file 14 in ORAS (15 holds the same lines) and 13 in X (and 14),
  a line a move by its number: Surf is 57, "Surf" in English and French, "なみのり" in Japanese.
- **Built** (`field.WATER`, `Ground.surf`, `Navigator` with `PATHFIND_HM`): P's ground has water
  as a wall, unless the player stands on water already; Shift and P's crosses it. Going onto
  water costs five steps over the step (`field.BOARD`), and a leg ends where the water starts or
  stops, the legs on it said with the game's word for Surf: from the beach at (339, 203), with all
  four chunks loaded, P says "no path to Fisherman" and Shift and P "10 west, 3 south, Surf 33
  south, 15 south, 27 west". The last two legs run past the chunks loaded, which is why the water
  is not said there: a cell no chunk holds is open and not water.
- **A way into chunks the game has not loaded was a guess, and was given up.**
  omega_ruby_surf_2.ctm (the same save; 0 out of sync): with only the player's chunk loaded, P
  from (352, 183) said "51 west, 70 south", over the sea as land, and "no path to Fisherman" at
  32.9 s, when walking south-west loaded the chunks with the sea in them.
- **The field map lists each chunk's file in ORAS too**: `MAP_CHUNK_FILES`, as spike 106 read
  it in X, `0xffff` where the matrix has no chunk. Route 109's matrix is 12 by 10 chunks, and
  for all four loaded the file listed (43 to 46 of `a/0/3/9`) holds the cell table the game
  loaded, word for word.
- **Built** (`archives.MapChunks`, `Ground.far`): P's ground reads every chunk of the matrix the
  game has not loaded from the ROM, a chunk file on its own (30 ms, `garc_spans` and
  `read_part`) and each once. From (352, 183) with one chunk loaded, P says "no path to
  Fisherman" in 0.31 s and Shift and P "6 south, Surf 65 south, Surf 40 west, 10 west" in 0.18 s.
  X's list is read the same way and has not been tried for this.
- **Not known**: whether the currents carry a player and which way each runs (a way over them
  may go against one), whether `0x42` is surfed like the sea, and what `0x3f` and `0x40` are.
  Cut's trees, Rock Smash's rocks and Strength's boulders are not read at all (spike 137 reads
  Cut's).

## Spike 137 — the bush Cut clears and the rock Rock Smash breaks, and the way through them

2026-09-28 (`spikes/137_bush.py`, and the ROM's zone files). The user, on Route 104 (zone `0x1a`),
heard an "object 1 east", and walking into it the game said "You can't pass because of prickly
thorns, but a Pokémon may be able to cut the tree." omega_ruby_bush.ctm, on the save of 20:25
(the rig's `save_checkpoint_bush`), replayed with 0 out of sync; the picture has the bush right of
the player and an item ball behind it.

- **The bush is a move model**, sprite `0x200b`, standing at (149, 274): record 0 of the zone's
  NPCs, with no script (+0x08 is 0) and 10052 at +0x0a. Across ORAS's zone files the ROM stands
  42 of them in 16 zones, every one without a script. The sprite list (text file 94, 544 lines)
  stops short of the things' sprites, so the ROM has no id for it either.
- **The rock is `0x200c`.** omega_ruby_smash.ctm, on the save of 21:35 (the rig's
  `save_checkpoint_smash`), 0 out of sync: the player starts at (40, 12) in Rusturf Tunnel (zone
  `0x4b`), faces east and hears "It's a cracked boulder. A Pokémon may be able to break it." The
  picture has a brown boulder right of the player among grey rock walls. The ROM stands 108 of
  them in 20 zones, ten in Rusturf Tunnel.
- **Three of Rusturf Tunnel's rocks are the story's.** (50, 10), (51, 11) and (50, 12) have
  script `0x2a4` and 2000 at +0x0a, where the other seven have no script and 10050, and a sign
  record lies in each of their cells, so facing one runs the zone's script and not Rock Smash's
  question. They stand across the tunnel, between the player and its east half. What facing one
  says is not known; that Rock Smash breaks them is the game's story (Bulbapedia's Rusturf
  Tunnel: they join the tunnel's halves once the player has the Dynamo Badge), and left as walls
  they gave Shift and P no way to Verdanturf Town (the user, 2026-09-29).
- **Strength's boulder is `0x2009`** (2026-10-01). omega_ruby_strength-boulders.ctm, on the save
  of 14:12 (the rig's `save_checkpoint_strength`), 0 out of sync: the player starts at (37, 44)
  on Fiery Path (zone `0x55`), faces west and hears "It's a big boulder, but a Pokémon may be able
  to push it aside.", steps to (37, 45) and hears it again facing west. The model reads (35, 44),
  but the picture has the boulder against the player, twice their width. It stands in four cells,
  (35–36, 44–45), filling a path two rows tall whose cells are all open floor (`0x24000024`);
  with only its model's cell standing, P would have walked the player into its other three. The
  ROM stands 43 of them in 13 zones, with no script.
- **Built** (`oras.BUSH` and `CRACKED_BOULDER`, `Game.clears`, `Ground.cleared`, `Navigator`'s
  `moves`): the bush is named "bush" in the English table of names of our own, and the rock
  "cracked boulder", the game's own words copied from its sentence, everywhere in the game. P
  walks round either as round anything standing; Shift and P's ground has it open, going through
  it costing five steps over the step (`field.BOARD`, as for going onto water), and the step
  through it said with the game's word for the move, Cut 15 or Rock Smash 249. One chosen itself
  is walked up to, with or without HMs. From (148, 274), to the item ball at (152, 274), P says
  "no path to item ball" and Shift and P "Cut 1 east, 2 east". In Rusturf Tunnel, Shift and P to
  the tunnel's east half goes "2 north, 9 east, Rock Smash 1 east, ..." through the story's
  three. For a day they were left to the zone as walls, one with something read by facing it in
  its cell, until the user found no way to Verdanturf Town (2026-09-29).
- **A move model says its own size**, at +0x150: a u8 of cells east and west and one north and
  south, as it faces south. Both boulders read 2 and 2; the bush-sized 0x2001, the item ball and
  the player 1 and 1. The slot copies it from the ROM's record for its sprite, at +0x884: a table
  of 338 records of 0x18 bytes, found by searching the ROM image for the boulder's (which of
  the ROM's files holds it is unread), the sprite, flags, the two sizes at
  +0x0a, and the model's id at +0x14. 297 are 1 by 1; the rest are 2 by 2 (the boulders `0x2009`,
  `0x200a`, `0x200d` among them), 3 by 3, 5 by 5, 2 by 3, 1 by 2 (`0x158`, a Fisherman) and 3 by
  1 (`0x138`, Mauville Gym's barrier). The barriers read 3 and 1 whichever way they face, so the
  game turns the size a quarter for a model facing east or west: spike 146's beams, one cell
  either side of the model, across the way it faces. An odd side is centred on the model's cell
  and an even one runs on east and south of it, as the boulder does. X's models keep the size
  at +0x150 as well (movie_x_city.ctm, every model 1 and 1, the only such pair from +0x100 to
  +0x200). The zone file's NPC record has a width and height too, but they are 1 and 1 on every
  boulder, and vary on people, by the look the range they walk in.
- **Built for the boulder** (`oras.BIG_BOULDER`, `mmodel.footprint`): named "big boulder" from
  the game's sentence, listed in the cells its model says, P going round all four cells and
  Shift and P through them with Strength (move 70). From (37, 44) it is "big boulder, 1 west",
  where it was 2 west; P says "no path" to everything past it, and Shift and P to the second
  boulder, at (27, 37), "Strength 2 west, 5 west, 2 south, 4 west, 3 north, 2 east, 4 north".
  The way is said as if the boulder were gone: where a push leaves it, and whether that shuts
  the way, is not modelled.
- **Not known**: whether a bush cut or a rock smashed stays so, and whether the player needs the
  move in the party or only the badge. The way is said all the same.

## Spike 151 — talking across a counter

2026-10-01 (`spikes/151_counter.py`; the code dump, `spikes/disassemble.py 0x0071760c`; the ROM's
chunks). The user: P finds no way to people talked to across a counter, the Nurse and a Poké
Mart's Clerk, and Devon Corporation's president. Their log has "no path to Clerk" in zone
`0x117`, a Poké Mart off zone `0xf`, from (15, 24) with two Clerks at (13, 18) and (13, 16).

- **The game talks across one kind of cell, the counter `0xd4`.** The field DLL's function at
  `0x007175a8` takes the cell one ahead of the player (`0x004dce98` with 1), reads its kind
  (`0x004dbec8`) and calls the executable's `0x00403004`, which is true for `0xd4` alone (spike
  83); where it is, it takes the cell two ahead instead, for any of the four ways. Nothing else
  is tested there, so a counter is crossed one cell deep and no other kind is.
- **The Nurse.** poke_center_pc.ctm, 0 out of sync: in Petalburg's Pokémon Center (zone `0x103`)
  she stands at (19, 15) with the counter `0xd40e0001` at (19, 16) and a counter or a wall on
  every other side, and P from (19, 19) said "no path to Nurse".
- **The Clerks.** Of the ROM's 819 chunk files, one, 512, has the Mart's shelves where the list
  put them and open floor under the Clerks and the player: its counter `0xd40e0001` runs down x
  14 from z 14 to 19 and along z 19 from x 11, and the Clerks stand behind it in floor nobody
  can walk into. 142 chunk files have the kind.
- **Mr. Stone is not behind a counter.** His one record, in zone `0x120`, is at (30, 20), and no
  chunk file has a counter beside that cell with the zone's three people on open floor. What
  stops P there is not known; a movie would show it.
- **Built** (`field.COUNTER`, `Ground.goals`): a solid thing with a counter beside it may be
  walked up to the far side of the counter, facing it. P to the Nurse from (19, 19) says "2
  north"; to everybody else in the Center it says what it did.

## Spike 120 — Heights on Sun's ground

2026-09-26 (`spikes/120_sun_heights.py`, the ROM alone). Spike 114 laid Sun's collision meshes
flat: where one floor passes over another both count, and every wall of either. Measured on the
house (area 21), Route 1 (0) and Iki Town (1), where the player has been, and on Hau'oli City,
Route 2, Route 5, Ten Carat Hill, Verdant Cavern and Brooklet Hill.

- **Walls carry no heights outdoors.** Wall meshes 0 and 1 run from y -1,600 to 1,700 (3,376 of
  Route 1's 3,400 triangles), indoors from 0 to 400 or 600, and mesh 2's boxes sit on their own
  floor. No wall of one level leaves another open anywhere measured.
- **Floors overlap in four ways, and no bridge was found.** Two floors more than 40 apart over one
  point: 2 cells in the house, 1,649 on Route 1, 1,917 in Iki Town, 649 to 1,168 in Hau'oli
  City and Route 2, 12 on Route 5, none in the caves. The sea (kind 8, y -1,320) runs on under the
  land. The ground runs on under decks and porches (kind 12) 41 to 76 below them: the player's
  cell outside the house, (561, 362), is on the porch's deck at -959 over ground at -1,000.
  Floors lie far below the ground: kind 0 at -1,200 under Route 1's at -30. And Route 2's ground
  runs a few cells under its cliff, 333 below, cut off by the cliff's wall.
- **Stairs are floor**: ramps of 34 to 36 degrees, whose cells' floors come within 20 of their
  neighbours'. Iki Town's down to Route 1 fall from 197 to 8, walled both sides.
- **A flat plan joins drops with no wall along them.** Of two open cells side by side in one of
  spike 114's pockets, Route 1 has 219,149 pairs less than 10 apart and 133 from 10 to 50; 163
  cells have a neighbour more than 50 away: Iki Town's wooden platforms' edges (59), the town's
  edge over Route 1 (189), the hill outside the house's yard (72 to 250), beach over the sea
  (92), and the floors below the ground (1,170).
- **Built** (`walls`): a cell keeps its **floors**, the ranges of heights under its nine points;
  floors under a point less than 50 apart are one, a floor less than 100 under another is covered,
  and a wall closes the floors it rises above and starts less than 100 over. Neighbouring floors
  join within 50, which cuts every drop measured and keeps every stair. A pocket is floors joined;
  P's ground keeps those of the player's height (`chunks(height=...)`), and where two cells side
  by side hold floors of those pockets that do not join, a **seam**, closes the one further from
  the player's height.
- **Against the known places** nothing a player would notice changes: the house the same (13
  of 13 records from the desk), Route 1 from (561, 362), (561, 363) and (525, 365) 50 of 56
  records either way with P's ground 7 cells smaller (a strip by the sea at -1,263), Iki Town from
  (368, 286) 47 of 48 and the same 16,173 cells, its drops in strips already walled off. From below
  Route 1's hill, (540, 366), 16 cells along its edge close, where the flat plan walked up it.
- **Cost**: Route 1 reads in 1.3 s (0.79 s flat), Iki Town 0.74 s (0.47 s), the house 0.27 s
  (0.25 s); Route 1 holds 2.2 MB and 1,411 seams.
- **Not known**: whether the player can step up 59 units (the platforms' edges, closed), whether
  a drop is a ledge walked down, and the player's own y at the known places, taken to be the
  floor's top. The runtime does not pass its height yet.

## Spike 131 — what a battle does to the thing being walked to

2026-09-28 (`spikes/131_battle_teardown.py`, movies 2 to 5 replayed with the runtime reading them,
all pinned and followed). Orion: the B/W accessibility layer lost track of what the player was
following when its map came back after a battle; does pa3ds? The navigator keeps the chosen
item's key and the way being walked, and finds the item again in a fresh listing at every key
press and every step off the way. A place's key is its kind and number. A person's, an item
ball's and a thing's was its move model's address, and the player's visit is its model's
address and zone. On the field, whenever no way was being walked, the spike chose the farthest
person and pressed P, and compared the listing before each battle with the one 0.5, 2 and 5 s
after it.

- **A battle keeps everything.** Over 15 battles, Wally's catch, two Bags, the Pokédex and a
  nickname: every listed thing came back under its key (5, 6, 11, 12 and 14 of as many), the
  chosen person was still chosen, and the way was still being walked. During a battle the field
  goes on saying the player is in the cell they left, so the way is never taken for a warp. The
  only thing gone after a battle was the hidden Pokémon the player had just fought.
- **Walking into the next zone did not.** It empties the models of the zone left and fills their
  slots with the new zone's people. In movie4 the player walked from Route 102 into Petalburg
  City while a way to a Youngster was being walked, and his slot came back holding Norman: the
  way became a way to Norman ("16 west, 12 north"), and nothing said the Youngster had gone. Four
  of Route 102's eleven keys were in Petalburg's listing, each another person or an item ball.
- **Built**: a gen 6 move model is told by its slot and the record it was built from, the zone
  and index at its +0x0c (`gen6.field` `_model_item`). Replayed, the same walk says "Youngster
  gone" and ends the way, and every battle keeps every key as before. Names are untouched: a
  person is named by sprite, never by key.
- **Left open**: Sun's actors are keyed by address too, and an actor's memory may be another
  actor's after a zone change; not measured. A person the game unloads is gone to pa3ds, though
  they still stand in their zone; walking back finds them again under the same key.

## Spike 135 — what the "kakuremino" classes are, and what holds the DexNav's record

2026-09-28 (`spikes/135_kakuremino.py`: `capture` saved the code and the heap three times over
movie3, pinned and followed: before May's lesson with the record empty, with the Poochyena's tail
up, and back on the field after its battle; `census` reads them offline). Spike 80 took four
"kakuremino" classes, one object each and alive in battle too, for the handlers of the hidden
Pokémon, and found the record as the word holding a `pml::pokepara::CoreParam` at +0xa4. Orion
asked whether they, or something other than CoreParams, lead to it, and whether Pokémon hiding
elsewhere than grass work the same way ([the plan](../plans/README.md), now landed).

- **"Kakuremino" is a trainer's event type, not the DexNav.** Fourteen class names in the code
  loaded on the field hold "Kakuremino" or "EvType", all `field::mmodel::`: `EvTypeNone`,
  `EvTypeClash`, `EvTypeEyeTarget`, and the trainers' `EvTypeTrMirror`, `EvTypeTrDoubleMirror`,
  `EvTypeTrDiving`, `EvTypeTrFighting`, `EvTypeTrFixEyeWayLeft`, `EvTypeTrFixEyeWayRight` and
  `EvTypeTrKakuremino`, with `MoveCodeKakuremino`, `ActionCmdStayJumpKakuremino` ("stay, then
  jump") and `EffectKakureminoBase` beside them. The objects of them are the same with a tail up as
  without, in tables of event types, and none holds the record's address, its CoreParam or any
  word inside it. By their family they are most likely the trainers who hide under a cloak and
  jump out at the player; not checked against one.
- **The one `field::PlayerWork` holds the record's address at +0x6c**, in all three captures, at
  the same place each time; its vtable is in the executable, so any sweep finds it by class. The
  only other word holding the record is in memory spike 80 already saw point at it, of no class.
- **Built** (`games.oras.hidden`): the record is what the player's work points at, holding one of
  the heaps the sweep found first. No page is searched for what holds a CoreParam, and CoreParams
  are no longer swept for. Replayed over the eleven ORAS movies, all followed: the same dialogue
  pages and battle lines, the lesson's Poochyena in movie3 and movie4's Seedots and Surskits at
  the same cells and moments, and the net's log naming a CoreParam in no movie, where the run
  before named them 25 times. Finding things cost 144.8 s against 145.6 s: spike 134's check of
  the record found had already taken most of the search away.
- **Not known**: whether a Pokémon hiding elsewhere than the grass is kept in the same record. The
  census names nothing for other kinds of hiding, and a movie with one up is what would tell
  ([roadmap](../roadmap.md)).

## Spike 141 — the berry spots, which no zone's data has

2026-09-29 (`spikes/141_berry.py`, and the ROM's field DLL). The user, on Route 102 (zone `0x18`),
talked to soil and heard "This is where you planted the Pecha Berry!" and "The soil is dry...",
harvested an Oran Berry tree and planted an Oran Berry in soft soil, and none of the spots was in
the list. omega_ruby_berry.ctm, on the save of 13:12 (the rig's `save_checkpoint_berry`), follows
its recording through the soil and the harvest and drifts at the Bag, whose sensor readings fall
out of step (8978 out of sync); what was measured is all before it.

- **A berry spot is nothing the field reader looked at.** The player stands at (156, 130) and
  talks north to soil at (156, 129), then from (155, 130) to the Oran Berry tree at (155, 129).
  Both are blocked cells reading `0x01000021`, the same word as a tree or a wall, so the cell
  table cannot tell soil from a hedge. No move model stands in either: the pool holds the same
  eight models, the player and Route 102's people, before and after the harvest. Route 102's zone
  file has no record near them either: its two signs are at (186, 139) and (148, 129), its
  triggers and areas are the route's, and its sections 3 and 4 are the wild encounters and
  twelve bytes of zeros.
- **The field's DLL lists every one.** `DllField.cro` holds a table of 90 records of 10 bytes: u16
  x and z in cells, an s16 height, the zone, and `0x218` in every record. Its first two are
  (155, 129) and (156, 129) in zone `0x18`, the two soil cells, and the rest stand in 21 zones
  more, two to six to a zone, all of them routes from 103 to 123: Route 123's field alone is
  30, six in each of five zones. The records are no zone's and no model's, so the game draws its berry trees from
  this table and its save (`savedata::KinomiData`).
- **The table is found by its shape**, since a DLL's layout is its build's: the longest run of
  records of that shape whose zone and cells are in range and whose last word is one value
  throughout, at any even offset. In ORAS's DLL that is the 90 and nothing else comes near; the
  next longest run is 26, the same table read out of step. It takes 0.1 s.
- **Built** (`archives.berry_spots`, `GameText.berries`, `oras.BERRY_SPOTS`, the **berries**
  category): the player's zone's spots are listed as "berry spot", a place of their own by their
  place in the table, walked up to as a sign is. Replayed, Route 102's listing at (156, 130) says
  "berries: berry spot 1 north; berry spot 1 west 1 north".
- **Left open**: what grows in a spot. The game says it on facing one ("The Oran Berry tree has 2
  Berries!"), and a sighted player sees a tree, but the list says "berry spot" for soil and tree
  alike; `savedata::KinomiData` is where the game keeps each spot's berry and how far it has
  grown, not yet read. X and Y's berry fields are not looked at.

## Spike 142 — the way that went quiet halfway up Route 110

2026-09-29 (`spikes/142_movil.py`, which finds the way from the script, since a movie carries
none of pa3ds's keys, and logs at every step whether guiding holds a way and whether a fresh P
finds one). The user, on Route 110 at (24, 128), pressed P on the path to Mauville City and heard
"6 west, 26 north, 38 east, 39 north, 45 west, 34 north, 14 east, 19 north, 2 east", then "26
north" at the first turn, and a second P four cells short of the next said "no path to path".
omega_ruby_movil.ctm, on the save of 14:03 (the rig's `save_checkpoint_movil`), replayed with 0
out of sync.

- **Guiding never lost the way; a second P threw it away.** At (18, 107), where the movie ends,
  guiding still held all 224 cells and stood at the 27th. P finds the way afresh, and cleared the
  way being walked before it searched, so a search that found nothing left nothing to follow.
- **The search was kept round the loaded chunks.** A search may walk the loaded chunks, the
  player and the goal, ten cells further, and no more (`SEARCH_MARGIN`), which kept it off
  Route 109's sea as land while cells no chunk held were taken for open (spike 136). The way
  north turns 38 east at z 103, out to x 56. At (24, 128) the chunks at x 40 to 79 were loaded
  and the search reached x 79; one step west, at (18, 128), they unloaded, the search stopped at
  x 49, and a fresh P found nothing from there on, at every one of the 22 cells walked.
- **With the ROM's chunks read, the whole matrix is known**, and a search over it found the way
  from all 28 cells walked, in 32 to 42 ms.
- **Built** (`Ground.box`, `Ground.far`, `Navigator._guide`): where the ROM's chunks are read,
  a search may walk the whole matrix; without them it keeps to the loaded chunks as before. P
  again on the thing being guided to, where no new way is found and the player is on the way
  held, keeps that way and says what is left of it.
- **What it costs.** Replayed before and after, with 0 out of sync in each: spike 136's P and
  Shift and P to the Fisherman on Route 109 say the same words, "no path to Fisherman" and "10
  west, 3 south, Surf 48 south, Surf 17 west, 10 west" in 0.13 and 0.10 s where they took 0.09
  and 0.06 s, and from (352, 183) with one chunk loaded, "no path to Fisherman" and "6 south, Surf
  65 south, Surf 40 west, 10 west" in 0.20 and 0.12 s where they took 0.19 and 0.11 s. The worst
  a search can do is walk all it may and find nothing: to a goal off the matrix, Shift and P over
  Route 109's whole matrix (480 by 400 cells, the sea included) gave up in 0.78 s, reading the
  chunks not loaded from the ROM as it went, and P in 0.18 s; on Route 110, 0.27 s. No cap on a
  search was added for that.
- **Not known**: whether a matrix bigger than Route 109's makes a search that finds nothing
  slower still. A cell of the matrix whose chunk the field map lists as none is still taken for
  open.

## Spike 145 — the Dowsing Machine's lights, and the record behind them

2026-09-29 (`spikes/145_dowsing.py`, and scratch captures). The user used the Dowsing Machine on
Route 110 (zone `0x21`) and found a hidden Poké Ball, and pa3ds said nothing while the machine was
out. omega_ruby_item-finder.ctm, on the save of 17:37 (the rig's `save_checkpoint_dowsing`),
follows its recording to the ball with nothing out of sync; it ends on no save.

- **What the game shows is on the top screen.** The player holds the machine out from the
  field's first frame to the end, and the bottom screen stays on the AreaNav. Two lights on the
  machine flash blue, green, yellow or red, or stay dark. The ball was at (10, 59): the lights
  were red at (9, 59) facing east, the cell in front, and A there found it.
- **They light only while the player faces the item's way.** At (9, 64) the lights were green
  facing north and dark facing west, in the same cell; at (9, 59) red facing east and dark facing
  north. Which cells count as the item's way was not worked out: blue at (15, 63) facing north,
  with the item 5 west and 4 north, but dark running west past (13, 65), with it 3 west and 6
  north.
- **The colour is how near**, counting the cells east or west and north or south together: blue
  9 to 18 away (18 from (22, 65), the furthest lit; 20 from (24, 65) was dark), green 3 to 8,
  yellow 2, red 1. That is what was seen, not a rule pa3ds keeps: the game works it out.
- **Nothing of it has a class.** The field's classes hold only `field::EventPlayerDowsingOn` and
  `Off`, in `DllFieldEventPlayer.cro`, the events that take the machine out and put it away.
- **Grouping captures by the colour found nothing.** 103 captures of the heap half a second apart,
  and 980 reads of the code's data and the linear heap, were searched for a value the same within
  each colour and different between them, with the colour as a picture showed it or as where the
  player stood and faced said it. The record below was in those captures and was missed: a
  picture and a capture were a moment apart, and a turn or a step changes the level between.
- **The record holds the item's cell.** The heap held (10, 59) as two words side by side in one
  place alone, in a block of 0x20 bytes, and read beside the player's cell and facing the words
  around it are the lights:

      +0x00 -> the field's work    +0x08 u32 1 with the item in the cell in front
      +0x0c u32 a timer            +0x10 u32 the lights: 0 off, 1 blue, 2 green, 3 yellow, 4 red
      +0x14 s32 x, +0x18 s32 z: the item the lights show; -1 while they are off

  A picture a third, two thirds and a whole second after each change saw level 1 blue, 2 green, 3
  yellow and 4 red, every time.
- **It is reached from the field's process.** `field::FieldmapProc` holds the field's work at
  +0x24, a block of 0x21c bytes without a class that the PlayerGrid, the reverb manager and the
  subscreen's menu also point at, and the work holds the record at +0x1d0. The record points back
  at the work, which is its validator.
- **The level jumps for a frame or two as the player turns or steps**: 2, then 3 at 55.23 and
  55.26 s, then off at 55.53 s.
- **Built** (`gen6.dowsing`, `oras.DOWSING`, `RunningApp.process`): each field poll follows the
  chain from the running field's process, and a level read the same on two polls in a row is
  said, "dowsing green", and "dowsing nothing" when the lights go off after being on. Replayed,
  the log has blue at 18.9 s, green and nothing in turn as the player turns at (9, 64), and
  "dowsing red" at 63.4 s, 1.5 s before "Leos found a Poké Ball!". A yellow held for one poll
  between two steps is not said. Where the item is, which the record holds, is left unsaid: the
  game shows a colour and the way the player faces, never a place.
- **Left open**: what the record holds while the machine is put away, which no movie has; X and
  Y's machine, not looked at.

## Spike 146 — Mauville Gym's barriers, switches and two rooms

2026-09-29 (`spikes/146_movil_gym.py`, and the ROM's zone file and chunks). The user, in Mauville
Gym (zone `0x1b8`), heard every barrier listed as "person" and no switch at all.
omega_ruby_movil-gym.ctm, on the save of 18:40 (the rig's `save_checkpoint_movil_gym`), replayed
with 0 out of sync; it ends on no save.

- **A barrier is an electric beam between two posts four cells apart**, and its move model stands
  at the beam's middle: ten NPC records drawn as `0x138`, the ROM's `STOPPER3`. The posts are
  blocked cells of the chunk; the three cells of the beam between them are plain floor, `0x20`.
- **`STOPPER3` is not a beam anywhere else.** Eight other zones stand it as an invisible blocker
  of one cell, several side by side where a way is to be shut (eight in a row in zone `0x63`).
- **A switch turns two beams a quarter about the post between them.** Stepping onto (58, 38)
  moved the red beams' models between (60, 34) and (58, 36), and (60, 38) and (62, 36), about the
  post at (60, 36), and turned their facing, `+0x1c` and `+0x24`, from east to south: a beam faces
  south lying east and west, east lying north and south. Nothing in the cell table changed. Every
  barrier's `+0x08` goes from `0x804006` to `0x80400e` while the player stands on a switch.
- **A beam stands in all three of its cells.** Read off the gym's map with the posts as walls,
  one cell a beam leaves the way on open in every state of the beams, in both rooms; three cells
  shut it in all but one, both of the entrance room's beams north and south, and in Wattson's
  room all three, with the switches reachable from a changing set of states. The game lets the
  player through only with the beams one way, so the beam is three cells. Not seen in a replay:
  no movie walks into a beam's end.
- **The switches are triggers**, one cell each, scripts 4 to 8, all armed by work `0x40f8`: the
  entrance room's at (58, 38) and (66, 38), Wattson's at (5, 40), (13, 36) and (13, 44).
- **The gym is two rooms with no floor between them.** The entrance room (x 55 to 68) and
  Wattson's (x 3 to 15) are joined by two triggers three cells wide, script 9 at the top of the
  entrance room's way north, (61, 31), and script `0xa` at the bottom of Wattson's, (8, 49). That
  each takes the player to the other is read off the map, not seen: no movie walks onto one.
- **Built** (`oras.BARRIERS`, `oras.TRIGGERS`, `gen6.field`, `Ground.of`): a `STOPPER3` model in
  the gym is listed under signs as "barrier", three cells across the way it faces, and P's ground
  has all three shut; the switches are listed under signs and the two ends of the way between the
  rooms under exits as "passage", each walked onto when it is the goal and walked round
  otherwise, since a way over a switch would turn a beam behind the player. Replayed, from the
  entrance, P says "no path" to the passage north and to Wattson, "3 south, 7 east, 3 north" to
  the far switch round the red beam's post, and "7 east" once the near switch has turned it.
- **Not known**: which beams each of Wattson's three switches turns, and whether a beam's end
  cells are shut in the game, which a movie walking into one would say.

## Spike 154 — Secret Bases: every spot, the way out, and who stands in the walls

2026-10-01 (`spikes/154_secret_base.py`; the code dump; heap dumps on Route 111 and in the base).
The user, on Route 111, climbed into their Secret Base in the tree at (381, 116) and decorated it:
the list had no way in on the route and no way out in the base, and the PC's menus could not be
followed. omega_ruby_secret-base.ctm, on the save of 16:12 (the rig's
`save_checkpoint_secret_base`), follows its recording with 0 out of sync.

- **No zone's data has a spot.** The player stands at (381, 117) facing north and presses A: the
  tree's cell is `0x01000021`, a tree like every other, with no furniture record, no warp and no
  move model. Route 111's records hold Aarune (`0xd3`) at (381, 117), hidden by his flag.
- **The field draws each spot from a list of its own.** On Route 111 the heap holds the zone,
  then six entries of a state, a model and a word: `ffff0008` with the vine's
  `xy_system::H3dShaderModel` at world (6867, 0, 2097), the cell (381, 116); `ffff0009` with a
  model at (423, 91), 36 units up; four empty.
- **The executable lists every spot.** At `0x00582018`, 85 rows of 20 bytes, numbered 0 to 84:
  the number, a kind (0 for 60 rows, 1 and 2 for 12 each, 3 for one; the tree is 2), a byte 0 to
  50 for the shape inside, the cell as two s32s, its height in world units, the zone, and `0x218`
  in 83 rows. Spot 8 is zone `0x24` at (381, 116), spot 9 the same zone at (423, 91). Found by
  its shape, rows numbered from 0 with their zones and cells in range, in 0.06 s.
- **The save says which is the player's.** `savedata::SecretBase`, one all session in the save's
  block, holds the spot's number at +0x32e: 8, on the route and in the base. PKHeX's format puts
  it two bytes into the player's base, 0 for none.
- **Inside is one of 52 zones, all "Secret Base".** `0x190` to `0x1ab` and `0x1df` to `0x1f6` in
  the zone table; spot 8's shape byte is 8 and its base is `0x198`. Each of the 51 with records
  has the same three: a trigger, script `0x2a71`, on the cell the player arrives in, (18, 18) in
  `0x198`, walked onto going south, which is the way out; furniture `0x2a72`, the PC, booted
  from in front of (12, 12); and `0x2a73` beside it, the base's flag.
- **The six people in the corner are the secret team's slots.** NPC records of work objects
  `0x7b` to `0x80`, with flag `0x3d`, built into the pool at (32, 36) to (36, 35), in the walls
  of the base's far corner: every cell round them is wall. They were listed as six people and P
  found no way to any of them.
- **Built** (`oras.secret_base`, `field.SecretBaseReader`, `Game.furniture`): the player's own
  spot is listed under exits as "entrance to Secret Base", the rest as "secret base spot", each
  walked up to as a sign is; inside, "way out", "PC" in the game's word and "flag". A move model
  whose cell and every cell round it are walls in the loaded chunks is not listed.
  Replayed, Route 111 says "entrance to Secret Base 1 north" and "secret base spot 42 east 26
  north", and the base "way out here", "flag 5 west 6 north", "PC 6 west 6 north" and no people.
- **Left open**: what each kind of spot is (a wall, a tree, a bush) in the game's words, which the
  ROM has only as what each says when faced; other players' bases, which the save keeps after the
  player's and which would be entrances too, are called spots.

### Decorating: the cursor on the floor, and what stands where

2026-10-02 (`spikes/154_secret_base.py --decor`, heap dumps at 35 s and 120 s). The user, on the
same save, decorated again: the cursor walked the floor, the PC and the hedge, a Small Chair was
set down, and the desk picked up and put back. Nothing said where the cursor was or what was under
it, and they wondered whether P walked through the furniture. omega_ruby_secret-base2.ctm follows
its recording with 0 out of sync (rig replay `secret_base2`).

- **The base's module, `DllUSSecretBase`, has 53 classes**, loaded as the player goes in. A sweep
  of the whole heap finds every one of them in the module's own image, copied into the heap as it
  loaded (vtables followed by ARM code), as well as the live objects; a find is only taken in its
  object's shape.
- **The cursor is `field::secretBase::SecretBaseCursorModel`**: +0x04 its mode, 3 as the base is
  built, 0 walking the floor, 1 with an item chosen, 2 holding it; +0x08 1 while shown, 0 while the
  item list is up; +0x30 u16 x and +0x32 u16 z, its cell, which its world place (+0x0c, +0x14,
  18 units a cell) agrees with. It starts on the player's cell, (12, 13) in front of the PC.
- **`field::secretBase::SecretBaseMapper` says what stands where.** +0x04 and +0x08 are 40 and 40;
  +0x18, +0x1c and +0x20 each lead to a layer, (cells, 40, 40), 8 bytes a cell: the decoration's
  model and its row in the save. +0x1c is the floor: Your Location (row 0) at (18, 12), the five
  Welcome Mats beside it, the Small Desk (row 6) at (14, 13), and the chair (row 7) at (14, 12)
  from the moment it was set down. **Cell (x, z) is the z × 39 + x'th**: 486, 487 to 491, 521 and
  482, which no other row length fits. The two other layers were empty throughout and are taken
  for the posters on the wall and what stands on a desk.
- **A decoration's model** keeps the decoration's number at +0x04, as the save does (PKHeX's
  format: 28 rows of 12 bytes from the base's +0x04, a number, x and z as u16s, a turn), and its
  class says what kind of thing it is: the desk the plain `SecretBaseGoodsModel`, the mats
  `SecretBaseGoodsModelOmanekiMat`, Your Location `…JibunNoIbasyo`, with `…NotHit`, `…Isu` and
  32 others besides.
- **The game names a decoration from text file 56**, a line a number: Small Desk 0, Small Chair
  10, Welcome Mat 0x13 to 0x17, Your Location 0xad. The bottom screen names the one under the
  cursor ("Select item", "Small Desk"), and over the floor, the PC, the flag and the hedge alike
  says "Select location", and nothing more as the cursor goes from one such cell to the next.
- **The cell table knows nothing of the decorations**: the desk's cell is open floor at 22 s and,
  read again, every half second after; the chair's too once it stands there. P's way from the way
  in to the PC was "5 north, 6 west", through the desk; from (14, 12) "1 south, 2 west", into it.
  The player stood 2.8 s at (14, 12) and 1.8 s at (15, 13) and went round by row 14. They had
  walked onto Your Location at (18, 12) on the way.
- **Built** (`oras.secret_base`, `FieldReader.decorating`, `Ground.of`'s `solid`): while the cursor
  is shown, each cell it moves to is said with what stands there, by the game's names, or "PC",
  "flag", "wall" where the cell table blocks it, or "blank", and its bearing from the player:
  "PC, 1 north", "Small Chair, 2 east 1 north", "blank, here". The floor's decorations stand in
  P's ground, but for the classes stood on (`WALKED_ON`). In a base with no mapper found yet a
  sweep is asked for every 2 s, up to five times, since the module's classes are learned a look
  after the sweep that comes with entering. Replayed, the cursor is said from the first moment of
  decorating, 102 lines; the rest of the log against the commit before differs by 9 lines where
  two runs of the new code differ by 14.
- **Left open**: a decoration bigger than one cell, which none here is, so neither the layer's
  row length past rows 12 and 13 nor the cells such a one fills are measured; which classes are
  walked over, measured for the desk and Your Location alone, the rest of `WALKED_ON` going by
  their names (mats); the cursor turns red where a held item cannot be set down, which is drawn
  and not read; the two other layers.

## Spike 155 — Route 113's ash piles, which no zone's data has

2026-10-02 (`spikes/155_ash.py`, heap captures, and the ROM's `DllUSR113.cro` disassembled). The
user asked for the ash on Route 113 to be listed under a category of its own, unsure how the game
shows it. omega_ruby_ashes.ctm, on the save of 08:04 (the rig's `save_checkpoint_ash`), follows
its recording into the Glass Workshop, where the Big man wants 206 more grams for a Blue Flute.
After the first visit the replay leaves the recording with nothing out of sync: every replay but
the first walked west along z 94 and into the pile at (252, 92) instead of back to the door.

- **ORAS shows the ash as grey mounds on the ground**, not as ash on the grass: one below the
  workshop's sign at (252, 92), the grass round it green. The pile is an open cell
  (`0x00000020`), as the grass is `0x1f004004`: Route 113's two loaded chunks hold five words and
  no other. No move model stands in it, and the zone's records near it are its sign at (251, 91)
  and an item ball at (252, 92) whose flag is set; the route's five item balls, model codes
  `0x2000` and `0x2001`, are the ordinary kind, 221 and 39 of them across the ROM.
- **The route has a module of its own**, `DllUSR113.cro`, the only `DllUSR` module in the ROM,
  holding `field::FieldUniqueSequence` and `field::FieldUniqueSequenceOchiba`. Its code writes the
  route's seven piles from constants of its own into a list in the heap, 12 bytes a pile, f32 x,
  y and z: (250, 105), (301, 93), (304, 91), (320, 100), (239, 93), (252, 92) and (318, 92). The
  sequence holds +0x14, its states; +0x2c, the count, 7; +0x30, the list; +0x44, the player's
  world place; and +0x50, the zone it was started in, 0x28, read from the field each update.
- **Each pile near the player is drawn by an object of no class**, in a block of slots 0x15c
  apart: +0x18 the pile's place in the list, five pointers into the linear heap, and its world
  place at +0x34. Only the piles in the loaded chunks had one, (250, 105), (239, 93) and
  (252, 92) with the chunks at x 200 to 279; walked into, the pile's object stayed as it was.
- **A pile's state is a byte in `savedata::GimmickWork`**, a class of the executable living all
  session, from its +0x08, the byte the pile's place in the list. The module's code (offsets in
  the CRO file) sets it: 0 until the player first comes near, 2 as it is set out (`0x0878`), 3 as
  the player walks into it (`0x110c`, from 2 alone), and 1 once its ash is collected (`0x0568`,
  beside a random amount). As the sequence ends (`0x0eac`) every pile but those at 1 goes back
  to 0.
  Captured at 34 s and 57 s of one replay, the bytes read `02 00 00 00 02 02` and then
  `02 00 00 00 02 03`: the pile at (252, 92), the sixth, at 3, and gone from the screen. The code
  also looks for an item ball (`0x2000` or `0x2001`) in a pile's cell and sets or clears two of
  its flags, 4 and 0x100, which reads as hiding a ball under a pile; no ball still there stood
  under one here, so it is not seen.
- **Built** (`oras.ash`, `FieldReader`, the **ash** category): the sequence is swept for by its
  class, taken only in its shape (its count, and its list and states in the heap, since the
  module's image carries the vtable too) and only while its vtable and zone hold; the zone's
  piles are listed as "ash pile", a place of their own by their place in the list, but for those
  at 1 or 3. Replayed, the route's listing says "ash: ash pile 2 west 3 south; ash pile 15 west 4
  south; …", seven of them, and six after the pile at (252, 92) is walked into. Against the
  commit before, the replay differs by the ash lines and the class learned; strength.ctm, off the
  route, by the order of two lines at startup.
- **Left open**: the movie never collects a pile whole, so 3 going to 1 is read from the code, not
  seen; nor is a pile coming back, which the code says happens as the sequence ends, presumably
  on leaving the route. Jagged Pass has no module of its own and was not looked at.

## Spike 156 — Mt. Chimney's catwalk, a floor of its own

2026-10-02 (`spikes/156_floors.py`, and the ROM's map chunks read whole). On Mt. Chimney's summit
P to Maxie said "20 north, 9 east", and north was the crater's wall (omega_ruby_mt-chimney.ctm,
the user's log). Re-recorded under the same name on the save of 19:51 (the rig's
`save_checkpoint_chimney`), the movie walks from (23, 33) west and up the summit's corridor to
(14, 18), where the picture shows a metal catwalk running east over the lava, Tabitha on it.

- **The summit's two northern chunks, files 287 and 288, have nine sections**, where every chunk
  read before had seven, or six in X. `mapblock` turned them away whole, from RAM and from the
  ROM alike: the field listed two chunks of the four loaded, and P took every cell north of z 40
  for open ground, as a cell no chunk holds is.
- **A chunk with a deck adds two sections a deck**, nine for one and eleven for two: from section
  7 a cell table for each deck, the same 40 by 40 words as section 0's, then a `coll` mesh for
  each. Section 2 is the ground's mesh, in every chunk. The ground's table has the crater as a
  wall; the deck's is wall everywhere but the catwalk, x 16 to 37 and z 15 to 19, where Maxie
  (39, 17) and Tabitha (19, 17) stand.
- **A `coll` mesh is triangles in a tree of boxes**: the magic, its size after the first 16
  bytes, two words unread, then the tree's depth and how many ways a box splits, across, up and
  along (3, and 2, 1, 2); the boxes, two corners each; a leaf's start and count into the
  vertices; and the vertices, x, y and z in world units from the chunk's centre and a fourth
  float, a triangle every three, repeated in each leaf it crosses. The catwalk's is one level
  rectangle at 108 over x 16 to 39 and z 14 to 19; the summit's ground is 108 where it is walked
  and lower in the crater. The player stood at 110 the whole way.
- **A deck meets the ground where their heights come together.** Across ORAS's 38 chunks with
  decks (34 of nine sections, 4 of eleven), a deck's open cell beside a ground's open cell that
  the deck walls is 0 to 14 units from it 90 times, and 54 to 432 units the 654 others: a
  bridge's side over the path below it. The catwalk joins the corridor at x 15 to 16, both 108.
- **Built**: `mapblock` reads every floor, its table and its mesh's height under each cell
  centre, from RAM and from the ROM. P's ground takes each cell of such a chunk on the open floor
  nearest the player's height, read from the model's +0x118, and where two cells side by side are
  on floors more than 18 units apart closes the one further from the player, as Sun's seams are
  (spike 120). The listing takes a cell on any floor open there, which kept Maxie and Tabitha
  listed: with the deck's chunks read and the ground's table alone, they stood walled in and
  were left out. Replayed, P to Maxie from (23, 33) is "4 south, 8 west, 19 north, 24 east", and
  "25 east" from (14, 18). Against the commit before, the replay differs by the chunks listed;
  omega_ruby_movil.ctm, omega_ruby_movil-gym.ctm and omega_ruby_ashes.ctm by no more than a run
  of the same code against itself.
- **Left open**: the movie never steps onto the catwalk, so the join is read from the heights,
  not walked. Seams are looked for inside a chunk only, and which places the other 36 chunks are
  was not looked at. A move model on a deck stands in its cell on every floor.

## Spike 157 — Lavaridge Gym's trapdoors and geysers

2026-10-03 (`spikes/157_fire_gym.py`, and the ROM's zone file). The user, in Lavaridge Gym (zone
`0x1ad`), heard the gym's people and its two signs and nothing of its puzzle, and P said "no
path" to the trainers past it. omega_ruby_fire-gym.ctm, on the save of 03:08 (the rig's
`save_checkpoint_fire_gym`), replayed with 0 out of sync; it ends on no save.

- **The puzzle is 24 one-cell triggers**, scripts 4 to `0x1b`, in two grids of three columns. The
  room the player comes in by (x 12, 21 and 30; rows z 53, 43, 33 and 23, the middle column's
  last at 20) has a **trapdoor** in its floor at each: an opening framed in wood. The other (x
  90, 96 and 102; rows z 50, 40, 30 and 20) is a hot spring below it, a wooden raft floating at
  each, over a **geyser**. Each trigger keeps a height at +0x14: -9 to 63 for the trapdoors, 27 to
  108 for the geysers.
- **A trapdoor drops the player into the hot spring, and a geyser shoots its raft back up.**
  The player walked north onto (21, 53), fell (the model's height -12, then -228) and came out
  standing on the raft at (96, 50), height 34. They waded east across the spring at height 16,
  climbing onto each raft they crossed and down again, then stood on the raft at (102, 50): its
  height rose to 123, and the player came out at (30, 53), on the trapdoor in the same column of
  the grid above. So each trapdoor is over the geyser in its place in the other grid. Only those
  two were seen; the other 22 are read off the grids.
- **The game names neither.** Its text has the town's hot springs and trainers hiding in steam,
  and nothing for a trapdoor, a raft or a geyser, so "trapdoor" and "geyser" are pa3ds's.
- **Built** (`oras.TRIGGERS`): the 24 are listed under exits, each walked onto when it is the goal
  and walked round otherwise, as Mauville Gym's switches and passages are. Replayed, from the
  entrance P says "16 north" to the trapdoor at (21, 53); from the raft it lands on, "6 west" and
  "6 east" to the geysers beside it; from (30, 53), "1 east, 10 north, 1 west" to the trapdoor
  beyond the wall north of it.
- **Left open**: P finds no way from one room to the other, since nothing tells it where each
  trapdoor and geyser leads: "no path" to the trainers past them stays. Which trapdoor the player
  needs to reach Flannery was not looked at.

## Spike 158 — the Mach Bike, and which way each sandy slope goes

2026-10-03 (`spikes/158_bike.py`). The user, in Jagged Pass (zone `0x54`) at (28, 80), pressed P to
the Hiker 4 east 15 north and heard "no path to Hiker" from wherever they stood, got on the Mach
Bike, rode up the pass, got off, walked, and rode again: omega_ruby_bike.ctm, on the save of
02:26, kept as the rig's `save_checkpoint_bike`, replayed with 0 out of sync. They asked whether
pa3ds can tell they are on the bike, for P to ride up a slope then, and Shift and P to say the
bike is needed.

- **The player's move model says it.** Its whole slot and the one `field::PlayerWork` were kept
  every fifth of a second with a picture every second. +0x06 read 0x01 on foot and 0x21 from
  26.5 s to 40.2 s and from 77.9 s to 80.5 s; the pictures at 27.1 s and 79.6 s show the player
  in the riding pose and at 82.8 s standing, and the cells climbed three a fifth of a second
  between. +0x07 went 0 to 3 and +0xa2c 1 to 2 at the same moments; the player's work did not
  change. Over omega_ruby_surf.ctm, 130 s of walking on Route 109, +0x06 stayed 0x01. Whether
  the Acro Bike sets the same bit is not known: the player has the Mach Bike alone.
- **0x69 is not the only slope, and its way is not every slope's.** Jagged Pass's three slopes
  below the corridor at z 69 to 73, x 26 to 28, 31 to 33 and 36 to 38 from z 74, are
  `0x6c000020`. The player rode up the first north from (28, 80), sliding back to (28, 80) the
  first time, walked down it south at x 26 at 73.8 s, and rode up it again.
  pa3ds knew 0x69 alone, so P walked the player up these on foot: "9 north, 10 east" from (28,
  80) into the corridor above.
- **The ground's mesh says which way each kind goes down.** `slopes` reads every chunk of the
  ROM's map chunks and, for each kind of open cell, counts the cells where the mesh falls six
  units or more one way and rises as much the other: 0x69 all 25 east (Granite Cave's among
  them), 0x6a all 44 west, 0x6c 166 of 188 south, the rest level along a slope's foot or top.
  Plain floor, 0x00, is on such ground 5,397 times in 153,899, every way, and no other kind is
  steep nearly all over. That is the ledges' order, 0x72 east, 0x73 west, 0x74 north, 0x75
  south; no chunk has 0x6b, so north rests on the numbering, as 0x74's does.
- **The Hiker is behind a ledge, not a slope.** Row z 68 is 0x75 ledges from x 29 to 39, and
  the terrace the Hiker stands on at (32, 65) is reached only from further north, which nothing
  the player reached in the movie leads to, on foot, on the bike or with HMs; nor the path up to
  Mt. Chimney. Searched over the matrix's six chunks from the ROM, kept by `p`. Jagged Pass is
  climbed over its ledges, which in the earlier games the Acro Bike hops (not read here).
- **The ROM names the bike.** The items' names are text file 114 of each text archive, a line an
  item: line 718 is "Mach Bike" and "マッハじてんしゃ", 719 the Acro Bike (file 116 holds the same).
- **Built** (`mmodel.on_bike`, `field.SLOPES`, `Ground.bike`, `Ground.climbs`, `oras.MACH_BIKE`):
  P rides up a slope while the player is on the bike, and Shift and P goes up one on foot too,
  saying the leg with the game's word for the bike, at a toll of five steps as boarding water
  costs. The log says "field: on the bike" and "off the bike" as it changes. Replayed: the log
  says "on" at 25.4 s and 76.9 s and "off" at 39.3 s and 79.4 s; from (28, 80) to (38, 71) in
  the corridor above, P on foot finds no way, on the bike "9 north, 10 east", and Shift and P
  "Mach Bike 6 north, 3 north, 10 east"; back down on foot "10 west, 9 south".
- **Left open**: the Acro Bike and its ledges; whether a slope is ridden across as well as up
  and down, which P allows on the bike; and whether a slope needs a run-up, which the first ride
  up suggests.

## Spike 159 — Route 119's bridge, and the zones a place is split into

2026-10-03 (`spikes/159_route_119.py`, and the ROM's zone table). The user, at the foot of
Route 119 at (119, 289), heard the exit north said as "edge, 90 north", with no place after it, and
P on it said "no path to edge" four times. They asked whether much of east Hoenn goes unnamed,
whether the way was blocked by people, and for P to go as near as it can rather than say "no
path". omega_ruby_route-119.ctm, on the save of 10:52 (the rig's `save_checkpoint_route119`),
replayed with 0 out of sync; it ends on no save, and the player never moves.

- **The way north is over a bridge, and P's ground had cut it.** Route 119's matrix is 2 chunks
  by 6 (x 80 to 159, z 80 to 319), its foot zone `0x2f` and its top `0x2e`; the edge between
  them was listed whole, 80 cells along z 199, since no loaded chunk touched it, 13 of them open.
  From the player, the ground at their height (11) reached 1,834 cells and came nearest at
  (122, 203), four south of the border and a cliff below it: the floor there is at 45 and the
  cells along the cliff's top at 140. The chunks at z 200 to 239 (files 213 and 214) have a
  deck, a bridge at height 99 along z 209 to 211 from x 86 to 144, crossing over the path at 45
  that runs north into that dead end. Walked at the player's height, every cell where the two
  cross took the path's floor and closed the bridge's (`mapblock.flat`); at 99, the bridge is
  whole and 2,957 cells are reached, the 13 on the border among them. With either floor open
  everywhere, 3,139. The way is east to the bridge's end beside the Black Belt and the Psychic
  at (139, 208) and (140, 208), west across it, and north up the far bank. Nobody stands on it.
- **Built**: where no way is found at the player's height, P walks the decks at each other
  floor's height on the matrix in turn, each floor's the middle of its open cells' heights
  (`Ground.decks`), and guides on the ground it found the way on. From (119, 289), "2 east, 14
  north, 10 east, 54 north, 8 east, 12 north, 48 west, 10 north", the 48 west the bridge, in 410
  ms with the ROM's chunks read for the first time, and 169 ms with HMs after. Not seen walked:
  the movie never moves.
- **Where nothing reaches the thing, P goes as near as it can** (the user's ask): the search
  keeps the cell it reached nearest the goals' box, cheapest first, and the way to it is said
  with ", then can't continue", and on arriving, "can't continue, edge, 4 north". "no path" is
  left for a player with nowhere nearer than where they stand.
- **East Hoenn's unnamed exits are the zones its places are split into.** A ROM-wide audit
  (`audit`) of every outdoor zone: doors with no building named are in every town, west and east
  alike (Rustboro 6, Mossdeep 7, Sootopolis 8), the plain houses the game names nowhere. What is
  east Hoenn's own is places of two zones or more on one matrix: Route 119 (`0x2e`, `0x2f`),
  Route 120 (`0x30`, `0x31`), and Route 123's hill (`0x1dd`, `0x211` to `0x213`, on matrix 427
  apart from the route's 9), with the Safari Zone (`0xdb` to `0xde`) the only such place in the
  west. Every edge between two of them was said bare, since the place is the player's own.
  Such an edge now says the place, "edge to Route 119": the game's word, and the same one, since
  the game calls both zones Route 119.
- **A zone on its parent's matrix is outdoors.** `0x2f` is not its own parent (`0x2e` is), so
  the way onto it from Route 118 was said "stairs up to Route 119", where Route 123's onto Route
  122, the same warp kind `0x0501`, is "path". Exactly six zones of the table share their
  parent's matrix: Route 119's foot, Route 120's `0x31`, the Safari Zone's three, and `0x217`,
  soaring in the sky. 45 zones have matrix 0, the placeholder zones 0 to 5 and rooms (the Trick
  House's among them), and 0 is no matrix. A zone row's first word is 0 for most outdoor zones
  and 3 for most buildings, but Fortree and Slateport are 3 and every Gym 0, so it says nothing.
- **People named nowhere stand everywhere**, not in the east above all: `NONDRAW` (`0x0`) on
  Route 119's top, Route 120 and Fortree, the Pokémon sprites from `0x1000` up past the ROM's
  list of sprite ids, `ELDER_RS` and `JUNIOR_RS`, whom the shipped table leaves out, and
  `FAIRYTALE_RS`. Left as they are.
- **The Acro Bike's rails are two cell kinds, `0x6d` and `0x6e`** (`kinds`), asked for by the
  user, who could not find where the Acro Bike is used to record it. Both are walled, and both
  are only in Route 119's chunk files (207, 213 and 214), the Safari Zone's (502 to 504), which has
  rails too, and file 757. On the foot of the route `0x6e` runs from (107, 219) east to (114,
  219), south to (114, 222) and east to (121, 222), from the open ground south of the bridge's
  west end to the path north up the middle, 2 east and 67 north of (119, 289); on its top, lines
  of `0x6e` across z 100 to 119 and of `0x6d` down x 86 and 88. Jagged Pass has one rare open
  kind, `0x1f`, in four patches and in only eight chunk files, which may be its bumpy slopes;
  not looked at further. Nothing here was ridden.
- **Built**: the Mach Bike's slopes and the Acro Bike's rails are listed under a category of
  their own, **bike**: a slope as each patch of one slope kind, a rail at each end of a line of
  rail cells, read off the whole matrix from the ROM's chunks. Replayed from (119, 289), "bike:
  rail 2 east 67 north; rail 12 west 70 north", and P to the first, "2 east, 14 north, 9 east, 41
  north, 6 west, 12 north, 2 west", ending beside the rail facing it.
- **Jagged Pass's bumpy slopes are `0x6f`, not `0x1f`** (the user, the same day, finding only
  slopes under bike there; omega_ruby_bike-jagged.ctm, on the save of 15:21, the rig's
  `save_checkpoint_jagged`). The ground's mesh is level over every patch of `0x1f`, at 360, 378,
  270, 261 and 216: terraces, no slope. `0x6f`, a walled kind in 13 chunk files, the Safari
  Zone's among them, stands 24 times in Jagged Pass, each where a ledge dropping south would be in
  its row, a column of five at x 50 from z 86 to 73 and pairs at x 34 and 46, or alone across a
  narrow way at x 25: the steps the Acro Bike hops up. Listed under bike as "bumpy slope" now, each
  cell a thing; replayed from (39, 6), "bumpy slope 10 east 24 south" and on. Not seen hopped, and
  P does not hop them.
- **A bumpy slope is hopped as a ledge is jumped, but up** (the user, the same day,
  omega_ruby_bike-jagged2.ctm, on the same save, the rig's `hop`): P to the one at (49, 30), then
  round by the ledge at (46, 30) to (49, 31), onto the bike, and north through (49, 30) to (49, 29)
  and on; the model's bike bytes, +0x06 0x21 and +0x07 3, are what the Mach Bike sets, so nothing
  read says which bike is ridden. Each of the 24 has one pair of open cells on opposite sides,
  the ground's mesh a cell's height, 18, higher on one: north of 21, west of (51, 35), (53, 35) and
  (53, 54). Built: the field works each one's way out of the ROM's heights (`Ground.hops`), and
  on the bike P, and Shift and P on foot, hop it from the cell before to the cell after, said
  with the game's word for the Acro Bike, "1 north, Acro Bike 2 north, 3 north" from (49, 32).
  Over the pass's whole ground, the way from its foot at (34, 96) to its top at (39, 7) is 13
  hops, two west in a row, and the Hiker at (32, 64), which nothing reached before, is reached;
  on foot, neither.
- **Left open**: a way that goes under a deck and later onto it, or onto it and later under, is
  not found, since one height is walked for the whole map at a time; a search over each cell's
  floors would find it. Whether `0x34`'s way up onto Route 123's hill is stairs, as it is said,
  was not looked at. The Weather Institute's door, if `0x18a` is it, is said as "door": the game
  names the building nowhere beside it.

## Spike 160 — Fortree City's platforms, ladders, and the Kecleon at the Gym

2026-10-03 (`spikes/160_fortree.py`, and the ROM's chunks and zone file). The user, P on Winona's
Gym from (11, 14), heard "2 south, 10 east, 3 north, 14 east, 9 south, 4 east", climbed a ladder
on the 3 north and walked the platforms in the trees east, and the 9 south from (35, 13) went
nowhere. omega_ruby_tree-city.ctm, on the save of 13:32 (the rig's `save_checkpoint_fortree`),
replayed with 0 out of sync; it ends on no save.

- **The platforms are in the ground's own cell table.** Fortree is two chunks, files 154 and 155;
  154 has one floor, and its platforms (`00006020`) and rope bridges (`00007020`) are open cells
  beside open ground (`00000020`), the ground's mesh at 103 to 108 over them and 0 under the
  ground. 155's deck is only the bridge at (41, 13) to (42, 13), over a path on the ground at x
  42; its platforms too are in its ground's table. The player's height went from 2 to 110 as they
  stepped north from (21, 14) to (21, 13), and the picture shows a ladder there; at (35, 13), the
  edge of a platform 108 over the ground, the game did not let them down. The words of the two
  pairs are the same, and the mesh is 25 flat triangles, no wall among them.
- **A ladder is two one-cell triggers.** Fortree's zone has five, each a top on the platform with
  its height at +0x14 108 and a foot on the ground below at 0: (21, 13) over (21, 14), scripts
  0x2908 and 0x2906; (45, 13) over (45, 14), (57, 13) over (57, 14), (25, 29) over (25, 30) and
  (69, 29) over (69, 30), scripts 0xc and 0xb. The nibble at bits 12 to 15 of a cell's word, 6
  on a platform and 7 on a bridge, is no height: across the ROM 517 of the 525 places where an
  open cell with it 0 meets one with it 6 are level.
- **A drop is rare, and two cells' height or more.** Across the ROM's chunks of one floor, two
  open cells side by side step no more than 18, a cell's height, but in 42 files, and in all but
  eight of those the step is 18 and a fraction, the stairs. The eight (154, 395 to 399, 469, 704
  and 818) drop 40 to 200.
- **Built**: open cells side by side more than two cells' height apart (`mapblock.DROP`) are not
  walked between (`Ground.cliffs`, `pathfind.find`'s `joined`), read off every chunk of the
  matrix from the ROM, a chunk with a deck on the floors walked at the height P walks it, and a
  ladder's two cells are (`_ladders`). Replayed, P from (11, 14) no longer walks off a platform.
- **The Gym is behind a Kecleon nobody can see.** Its door at (39, 21) is in a pocket of ground
  whose one way in from the rest of the city is the path below the ladder at x 45, and at (45, 17)
  in it stands a model drawn as `NONDRAW`, listed as "person 34 east 3 south": the invisible
  Kecleon the Devon Scope shows. With it standing, the way as near as the player could get as the
  crow flies ended on the walkway 7 south of the door, behind a wall from it.
- **Built**: where no way reaches the thing, P finds one as if nobody stood anywhere, and where
  that one does, goes as far as the first who stands in it and says them: "2 south, 10 east, 3
  north, 24 east, 3 south, then can't continue, person in the way", up the ladder, along the
  walkways and over the bridge at its height (103), down the ladder at x 45 to (45, 16). From
  (35, 13), "10 east, 3 south, then can't continue, person in the way".
- **Left open**: the eight files' other drops (395 to 399, 469, 704 and 818) are not placed; a
  drop between a chunk of one floor and one with a deck is measured on the floor P walks there.
  Without the ROM no drop is known. The Kecleon is "person": `NONDRAW` stands for every
  invisible thing, and pa3ds has no word for it.

## Spike 161 — Fortree Gym's rotating doors

2026-10-03 (`spikes/161_tree_gym.py`, and the Gym's module disassembled, its tables read from
memory). The user, P on Winona from the Gym's door, was walked into a door's arm and heard "no
path to Winona, person in the way" at (23, 22), five unnamed models beside them, and asked for P
to know how the doors turn. omega_ruby_tree-gym.ctm, on the save of 14:22 (the rig's
`save_checkpoint_tree_gym`), replayed with 0 out of sync; it ends on no save.

- **The models beside the player are no door.** The Gym's records stand nine models drawn as
  sprite `0x8a` in a column at (36, 31) to (36, 39), with no cell while nobody is near; the game
  sets a few of them, a poll at a time, in the cells round whichever door the player is at, and
  takes them up again. They stand in for the arms, and were listed as people.
- **The Gym has a module of its own**, `DllUSGymFlight.cro`, holding
  `field::FieldUniqueSequenceGymFlight`. Its code builds seven doors, each from a world place of
  its own in the code: pivots at the corners (21, 32), (27, 28), (27, 14), (23, 23), (23, 12),
  (8, 22) and (16, 13), on the floors at heights 2, 56 and 110. The sequence holds +0x1c, the
  seven doors' objects, each with its pivot as f32 x, y, z at +0x130; and from +0x48 a record
  each of 16 bytes: the door's pattern, a word, how many quarter turns anticlockwise it stands
  at (-1 while the player is at it), and the turn it stood at then. They stood at 3, 0, 0, 1, 0,
  3 and 1 as the player came in; door 0 went to 2 as the player walked east into its north arm
  from (20, 31), and door 1 to 1 as they walked north into its east arm from (27, 28). +0xb8
  points at the record of the door the player is at.
- **The module's tables say every door's arms.** Its code reads two tables through pointers the
  loader patches: one gives each door its 16 cells, the 4 by 4 round its pivot, a quarter at a
  time; the other, for each pattern and turn, for each of those cells and each way the player
  steps (0 north, 1 south, 2 west, 3 east; 4 to 7 repeat them), the cells to be checked before
  the step turns the door, or nothing where the step walks into no arm. A step into an arm is
  exactly a step across a line out from the pivot an arm stands on, from either side, so the
  table is the arms: pattern 0 has arms west, east and south a cell long; 1 north and east two
  cells long; 2 north two long, east and south one; 3 north and east one long, south two. Door 3,
  where the player was turned away, is pattern 1 at turn 1: an L of arms north and west.
- **A door turns where its arms sweep only open cells.** The cells checked are those the arms
  sweep through, a whole quarter for an arm two cells long and the cell beside the pivot for one
  a cell long, in every entry tried of patterns 1 and 2; in pattern 0 one entry checks the cell
  beside the pivot across from the one the arm sweeps. Round door 3 the quarter south-west of
  the pivot is walled at (21, 23) and (21, 24), and the player's step west from (23, 22) would
  swing its west arm through it: the game did not turn the door, and P had walked them there.
- **Built** (`oras.doors`, `RotatingDoor`, `Ground.through`, `pathfind.find`'s `turns`): the
  sequence is swept for by its class, taken in its shape; the Gym's doors are read as they stand
  whenever the ground is, the stand-ins are no longer listed or stood in, and P searches the
  cells and the doors' turns together, a step into an arm turning its door where the cells it
  sweeps are open, said "push 1 north". Replayed, the reader follows doors 0 and 1 turning as the
  player walks them, and P from (16, 38) inside the door finds a way to Winona of 45 legs, eight
  of them pushes, in 0.4 to 0.6 s. The way is not yet walked.
- **Left open**: what slots 4 to 7 of the table are for; whether the game also checks people
  where the arms sweep; whether the player always steps on with the door, as they did both times
  here; and pattern 0's one entry that checks another cell than the sweep.

## Spike 162 — the shore Surf jumps, and surfing told from the bike

2026-10-04 (`spikes/162_shore.py`, and the ROM's map chunks). The user, at (22, 8) on Route 122
(zone `0x33`), pressed Shift and P to the path to Mt. Pyre on its island: "1 north, 2 east, then
can't continue". They surfed south and east anyway, and from the water P said "48 south, 30 east,
then can't continue" until they had landed; the log said "field: on the bike" all the while they
were on the water. omega_ruby_surf3.ctm, on the save of 11:42 (the rig's `save_checkpoint_surf3`),
replayed with 0 out of sync; it ends on no save.

- **Between the land and the sea is a strip of shore one cell wide, and P walled it.** Route 122's
  shore is `0x40180023`, kind `0x40`: the water bit and the blocked bit both set, between land and
  the sea's `0x3d`. Spike 136 left `0x40` unknown, and P's ground walls every blocked cell, the
  ground to surf included, so the water was cut off from the land on both sides.
- **Surf jumps the player over it, never onto it.** Getting on, the player went from (22, 8) to
  (22, 10) in one move, over the shore at (22, 9); getting off, from (54, 58) to (54, 56) over
  (54, 57). Neither shore cell was ever the player's. That is a ledge's jump, but either way.
- **`0x40` is the shore all over ORAS.** Its 1,528 cells are in 87 chunk files, in six words
  differing only in their second byte. Of those inside a chunk, about 1,090 have land on one side
  and the sea on the other; nearly all the rest are the strip's corners or run between the sea and
  a cliff. `0x3f`, the other blocked water kind (`0x3f180003`, 72 cells in 8 files), is a line
  with water on both sides and never land, and is still not known.
- **Surfing sets the move model's riding bit as the bike does.** +0x06 read 0x01 on foot and 0x21
  from the moment the player was on the water to the moment they landed, as on the Mach Bike
  (spike 158). +0x07 tells the two apart: 0x13 on the water, going 0x00, 0x10, 0x13 as the
  player got on, where omega_ruby_bike.ctm replayed reads 0x03 on the bike in all 78 polls it
  rode and 0x00 in the 829 on foot.
- **Built** (`field.SHORE`, `Ground.shore`, `pathfind.find`'s `over`, `mmodel.on_bike`): a ground
  to surf jumps the shore either way, from the cell before it to the cell after, and the jump is
  said with the side it lands on, so landing is a step on foot. Replayed, Shift and P at (22, 8)
  says "Surf 48 south, Surf 22 east, 8 east, 2 north", P there still "1 north, 2 east, then can't
  continue", and P from the water at (22, 34) "22 south, 30 east, 2 north"; guiding follows the
  player over the water and their jump off at (54, 57). The log no longer says the player is on
  the bike while they surf.
- **Left open**: what `0x3f` is; whether the game lets the player jump the shore where land is
  on both sides of it, which P allows with a ground to surf; and whether a shore between the sea
  and a cliff is ever crossed.

## Spike 163 — Mossdeep Gym's lifts, teleporter and switches

2026-10-04 (`spikes/163_psychic_gym.py`, and the ROM's zone file and script). The user, in
Mossdeep Gym (zone `0x1bb`, Liza and Tate already beaten), walked into "a thingy" that took them
to Liza and Tate, and a "teleport thing" there that took them back to the entrance; neither was
listed, and four of the list's six signs were what "changes the first thingy that lift you
around". omega_ruby_psychic-gym.ctm, on the save of 14:03 (the rig's
`save_checkpoint_psychic_gym`), replayed with 0 out of sync; it ends on no save.

- **The Gym is seven platforms with no floor between them.** The cell table has the entrance's
  (x 16 to 22, z 32 to 39), Liza and Tate's (x 16 to 22, z 3 to 9), two middle platforms a side
  (x 9 to 15 and 23 to 29 at z 23 to 29; x 5 to 11 and 27 to 33 at z 11 to 17) and two small
  pillars (x 14 to 16 and 22 to 24, z 18 to 20), and kind `0x01`, blocked, everywhere between.
  The pictures show tracks of white lights joining them.
- **A lift carries the player along a track.** The player walked north onto the trigger at
  (19, 32), script 5, and was carried north along x 19 at height 2, rising from z 14 to height
  146, and set down at (19, 10) on Liza and Tate's platform; their cell is in no platform between.
- **All ten triggers from script 5 to `0xe` are lifts.** The zone's script runs each of them as
  one call to the same native, 60, with a track's number last: 0 for script 5, and 1 to 9 for
  scripts 6 to `0xe`. They stand at the edges of the platforms: (19, 32) on the entrance's; (12,
  29); (26, 29) and (29, 26); (27, 12) and (27, 14); (11, 16) and (6, 17); and one on each pillar,
  (14, 19) and (24, 19). Only the first was ridden; where each other track ends is in the Gym's
  module, `DllUSGymEsper.cro`, which was not read.
- **Script `0xf` is the teleporter.** The trigger at (21, 6), beside Liza and Tate, calls native
  61 with nothing: the player stepped on it at height 146 and stood at (19, 35), the entrance, at
  the next poll.
- **The four other furniture records are switches.** Scripts 1 to 4, at (11, 25), (28, 27),
  (32, 13) and (6, 13), one on each middle platform, are pressed by facing them; their messages
  in the Gym's text (story file 507, lines 4 to 21) are all empty. Switch *n* sets work value
  `0x413b + n` to 1 and calls the module's natives 58 and 59. The two other records, script 18
  at (17, 33) and (21, 33), are the statues.
- **A trigger is armed while a work value holds a number.** The lifts on each middle platform
  carry their switch's work value at +0x04 and 1 at +0x02, and the teleporter `0x409a` and 1;
  the entrance's lift and the pillars' carry 0 and 0. Script 5 reads the four switches' values
  and, when all four are set, sets `0x409a` to 1 and rides track 0 with six positions of its own:
  the way to the leaders. So the switches arm their platforms' lifts, and all four together send
  the entrance's lift to the leaders and arm the teleporter. Spike 132's guess that +0x04 was
  the value that arms a trigger was half right.
- **The game names none of them.** The Gym's text has the guide's advice on types and the
  leaders' lines, so "lift", "teleporter" and "switch" are pa3ds's.
- **Built** (`oras.TRIGGERS`, `oras.FURNITURE`): the ten lifts and the teleporter are listed under
  exits, walked onto when one is the goal and walked round otherwise, and the four switches under
  signs as "switch". Replayed, from the entrance the list says "lift 3 north", "teleporter 2 east
  29 north", and "switch 9 east 8 north" and three more; P says "3 north" to the entrance's lift.
- **Left open**: where tracks 1 to 9 lead, and where track 0 goes before all four switches are
  pressed, so P finds no way from one platform to another and says "no path" to everything off
  the player's own; and whether pa3ds should say a lift is not armed, which needs the work
  values read from the save.

## Spike 164 — Route 128's dive spots

2026-10-04 (`spikes/164_dive.py`). The user, surfing on Route 128 (zone `0x39`, 7 badges), could
not find its dive spots in the list. omega_ruby_route-128-dive.ctm, on the save of 17:13 (the
rig's `save_checkpoint_dive`), replayed with 0 out of sync; it ends on no save.

- **A dive spot is kind `0x41`.** Spike 136 found it as water in patches inside the sea and
  guessed Dive's. The executable has a kind test for it alone at `0x00403bcc` (`lsr r0, r0,
  #0x18; cmp r0, #0x41; moveq r0, #1`, reached through a table at `0x006f469c`), and the field
  code at `0x003d3454` reads the kind of the cell under the player while their state is `0x12`
  or `0x13` and sets bit `0x400` of a flags word where it is `0x41`. The player surfing on
  Route 128 had +0x07 `0x13` throughout.
- **It is the darker water.** The movie starts with the player on `0x41` at (314, 166); the
  picture as they left it, a cell south, shows them at the edge of a band of dark water running
  diagonally from the north-west, the sea round it lighter.
- **Route 128 has three patches in its own zone:** 3-by-3 squares at (307, 175) and (339, 175),
  and a band of 296 cells whose box is 37 by 36 from (296, 160). Across ORAS's chunk files
  `0x41` is 8,630 cells in 66 files, words `0x411a0006` but for 143; 45 of its patches in one
  chunk are 3-by-3 squares, and the rest bands and blobs up to a chunk wide.
- **The game names none.** It draws the water darker and asks about diving there, so "dive
  spot" is pa3ds's.
- **Built** (`field.DIVE`, `FieldReader._ways`): each patch of `0x41` in the zone is listed under
  exits as "dive spot", read off the whole matrix as the bike's ways are, at the patch's cell
  nearest the player. A box would not do: the band's is mostly plain sea, and the way to it
  would end there. Replayed, the list at the start says "dive spot here", "dive spot 5 west 9
  south" and "dive spot 25 east 9 south", and P from the water says "9 south, 5 west" and "24
  east, 9 south, 1 east" to the two squares.
- **Under the water, Seafloor Cavern's way in is a trigger.** omega_ruby_route-128-sea-floor.ctm,
  on the save of 17:46 underwater (zone `0x44`, the rig's `save_checkpoint_seafloor`), replayed
  with 0 out of sync. Zone `0x44` has no warp record, and no zone outside Seafloor Cavern (`0x62`
  to `0x6f`) has one into it. Its only trigger, script 5, seven cells wide at (305, 184), is the
  gap in the south wall of a ring of rock about x 302 to 314 and z 169 to 184, round the light
  under the surface's dive square at (307, 175). Case 5 of the zone's script calls a function that
  ends in its one call with `0x62` among the arguments: (`0x62`, 21, -69, 35, 0, 1), Seafloor
  Cavern's first room at (21, 35), whose own trigger is an 11-cell line at (16, 36). Diving keeps
  the cell: the player under the water at (340, 175) had dived at the surface's square at (339,
  175). That square's light is in a pocket of 159 cells walled off from the rest (seaweed, `0x42`,
  and the Free Diver), from which nothing joins the gap: the player swam in it for the whole
  movie. Built (`oras.TRIGGERS`, its third item the zone an exit leads to): listed as "entrance to
  Seafloor Cavern", 29 west 9 south from (340, 175).
- **Swum through.** omega_ruby_route-128-sea-floor2.ctm, on the same save, the player's own log:
  diving at (307, 175), inside the ring, the list said "entrance to Seafloor Cavern 9 south" and P
  "can't continue, entrance, 5 south"; they surfaced, dived again at (307, 190), south of the
  ring, where the list said "6 north", P said "entrance, here", and the next poll had them in
  Seafloor Cavern, zone `0x62` at (21, 35), as the script says. Why the inside of the ring does
  not reach the gap was not looked at.
- **Under the water `0x41` is where the player surfaces**: "Light is filtering down from above.
  Would you like to return to the surface?" at (340, 175). The list still calls it a dive spot.
  The player's +0x07 is `0x03` under the water, against `0x13` surfing, and the log said "field:
  on the bike" throughout.
- **Left open**: where each spot leads (the underwater zone and the cell there), which the
  game's scripts or its Dive code hold and were not read; what `0x42`, tested at `0x00403434`
  and `0x003f9968`, is; and whether `0x41` in the chunk files with no sea on Route 128's matrix
  (237 to 259, 810 to 831) is the light the player surfaces through underwater, where "dive spot"
  would be the wrong word. No dive was recorded.

## Spike 165 — the way into Sootopolis City under Route 126

2026-10-04 (`spikes/165_sootopolis.py`). The user, under the water on Route 126 (zone `0x42`, 7
badges), could not find the way into Sootopolis City: the list had seven dive spots and nothing
else under exits, and P to most of them said "11 south, then can't continue".
omega_ruby_route-126-dive.ctm, on the save of 20:37 (the rig's `save_checkpoint_route126`),
replayed with 0 out of sync; the player stays at (13, 10) throughout, and it ends on no save.

- **The way in is a trigger, as Seafloor Cavern's is.** Zone `0x42` has no warp and one
  trigger, script 2, five cells wide at (55, 95); its case ends in the zone's one call with
  another zone among its arguments, (`0x4c`, 20, 29, 36, 0, 1). `0x4c` is the water under the
  city, which the game calls Sootopolis City: one trigger, script 2, seven wide at (17, 37),
  back to `0x42` at (57, 96), and 131 cells of deep water from (12, 29) to (28, 33), the light
  into the city's lake, 3 north of where the player arrives. Built (`oras.TRIGGERS`):
  "entrance to Sootopolis City", and back, "way out to Route 126". Replayed, the list said
  "entrance to zone 0x4c (Sootopolis City) 42 east 85 south".
- **Five such ways in ORAS** (`rom`): these two, Route 128's into Seafloor Cavern and back (`0x62`'s
  script 3, 11 wide at (16, 36), to `0x44` at (308, 185)), and Route 125's one-cell trigger at
  (240, 103) into Shoal Cave, `0x80` or `0x84` by the tide, and back. Shoal Cave's two and
  Seafloor Cavern's way out are not listed.
- **The water under the sea is pockets.** Drawn from the ROM's chunks (`map_42.txt`), rock (kind
  `0x01`, blocked) walls it into pockets. The player's, at the top, holds two lights, (11..13,
  10..12) and (60..62, 4..6), and no way to the entrance, whose pocket is the big one from z 25
  south, the entrance opening onto its south side at z 96. The south-west corner is a third,
  with two lights more. P's "can't continue" was true.
- **The matrices are `a/0/4/0`**, file N the matrix a zone-table row's +0x04 names: `MM`, a u16
  count of sections and their u32 offsets, the first section a u16 0 or 1, a u16 0, u16 width
  and height in chunks, and each chunk's file. Under Route 126 is 31, 3 by 3, files 251 to 259
  as the field map has them; Route 126 is 28, 8 by 8, files 237 to 245 in chunks 2 to 4 each way.
- **Every zone under the sea is area kind `0xc2`** (+0x08), and no other: under Routes 107, 124,
  126 to 130 and 134, under Sootopolis City, and Seafloor Cavern's first room.
- **Deep water lies under deep water, moved.** Under each but Seafloor Cavern, the sea above is
  the zone of the same place-name that is its own parent and not under the water, and its deep
  water is the underwater zone's, every cell, moved: under Route 126 by (80, 80), so (13, 10)
  rose at (93, 90) in the recording's log; under 128 by nothing; under Sootopolis by (41, 77);
  under 107 by (120, 240) and 124 by (80, 120). Routes 127 and 128 share one underwater matrix,
  129 and 130 another, and all four one sea, so the corners do not line up: the move is the one
  that lands every cell. Under Route 134's second zone a lone 3-by-3 lands in two places.
- **The sea above is pockets too.** The light at (13, 10) rises into a pond, rows 89 to 100 round
  (93, 90), walled by rock round a beach: in the recording's log P from there stopped short of
  everything. The light at (60, 6) rises at (140, 86) into a sea of 471 cells in the middle of
  the route, walled from the rest by a line of rock through the water (word `0x1000021`), whose
  other light, (122..124, 106..108), is over (42..44, 26..28) in the entrance's pocket.
- **Built** (`FieldReader._surfaces`, `navigate._dive_again`): under the water each light is
  joined to every light the sea above reaches from it, out of the ROM's matrix and chunks, and
  nothing where the sea is not found or lands in more than one place. Where P's way stops
  short in another pocket, it goes to the nearest light whose sea reaches a dive spot over the
  thing's pocket, and says that spot from the light, where the list puts it once the player has
  risen. Replayed from (13, 10), P to the entrance said "47 east, 4 north, then can't continue,
  surface and go to the dive spot 16 west 20 south", in 0.07 s with the sea read. Off the ROM's
  chunks, the way from (44, 26) to the entrance is "6 east, 27 south, 19 west, 36 south, 2 east,
  7 south, 22 east, 1 north". None of it was swum: the movie never moves.
- **Before and after**, dive, seafloor, seafloor2 and route126 replayed with 0 out of sync: the
  only change said is the entrance. seafloor2's 59 lines are a hidden Pokémon that appeared in
  one run and not the other; two runs of the new code differed by 82.
- **Left open**: a way that rises and dives more than once — P to the south-west pocket from the
  player's still says "can't continue"; the sea's own way between two lights, said as the crow
  flies; Shoal Cave's and Seafloor Cavern's ways out; and the light under the water still listed
  as "dive spot".

## Spike 167 — Sootopolis Gym: thin ice over a lower floor, its stairs, and invisible blockers

2026-10-05 (`spikes/167_ice_gym.py`, and the ROM's zone file and chunks). The user, in
Sootopolis Gym (zone `0x1bc`, 7 badges), asked for the way back up to be listed, for P to work
through the ice, and for some things not to be called people. omega_ruby_gym-8.ctm, on the save
of 09:00 (the rig's `save_checkpoint_gym8`), replayed with 0 out of sync; it ends on no save.

- **What the movie does.** The player walks north from the door at (18, 72) into the first room
  of ice, at height 254, steps back onto (18, 51) and drops through to (18, 51) at 38, where a
  Beauty battles them. After it, P to the way out said "1 east, 19 south", then "way out, 1
  south" at (18, 72): on the lower floor, 216 under the way out. The player walked west to
  (14, 72), beside a stair, and the movie ends.
- **Two floors in each chunk.** Both of the Gym's chunks, files 706 and 707 (matrix 346, 1 by
  2), have a deck. The floor above is the entrance's room (x 11 to 25, z 56 to 63) and corridor
  (x 17 to 19, z 64 to 73) at 198, three rooms of ice up corridors from it, at 252 (x 16 to 20,
  z 48 to 53), 306 (x 15 to 21, z 36 to 43) and 360 (x 11 to 25, z 24 to 30), and Wallace's
  platform at 414. The ground's floor lies under it all: a room under each room and a corridor
  under each corridor, its bottom room under the entrance's at -18, of kind `0x0a`, which
  Lavaridge Gym's hot spring has too and no other zone. The pictures show shallow water. Six of
  the eight trainers stand on it.
- **The ice has no kind.** The rooms above are kind `0x00`, as the corridors are. Which tiles
  are cracked is the Gym's module's, `DllUSGymWater.cro` (`--rooms`, and the module's file out
  of the ROM, disassembled): its `field::FieldUniqueSequenceGymWater` checks the player's cell
  against three rooms at every step (file offset `0x1ce0`). The module is mapped whole, at
  `0x00838000` in the replay, the sequence's vtable at +0x3310; two words of its code (+0x230c,
  +0x2310) hold where its 36 cells that are no ice are, in its own image, and where its data is,
  whose first three words point to the rooms, the top first. A room's record is u8 width, height,
  x and z of its corner, i32 its floor's height, u16 the work value (`0x4046` to `0x4048`) that
  is 1 once it is cleared, a pointer to a byte a tile and one to its three exits, and at +0x18 a
  byte set as it is cleared. The rooms are 15 by 7 from (11, 24) at 362, 7 by 7 from (15, 36) at
  308 and 5 by 5 from (16, 48) at 254; the cells that are no ice are three blocks of nine in the
  top room and one in the middle, and their bytes start at 1.
- **A step onto a whole tile cracks it; onto a cracked one, the player falls.** In the replay
  (18, 52), (18, 51) and (18, 50) went from 0 to 1 as the player stepped onto each; stepping
  back onto (18, 51) set the sequence's +0x84 to 135, that tile's place among every room's ice,
  and the player fell, and the room's bytes were all 0 again. The code: a whole tile is set to 1
  and a sound played; a cracked one starts `EventFallFromIceFloor`; once a step leaves every
  tile of the room at 1, `EventIceFloorClear` starts, and where the player stands on one of the
  exits, the top row's middle three, the room is cleared, and the code passes over it from then
  on. So a room is crossed by stepping on every tile once, last on an exit. The blocks of floor
  are skipped by the check, and walked on freely.
- **Two stairs are the way back up.** The lower floor climbs at x 11 and 12, and at x 24 and 25,
  from 0 at z 72 to 189 at z 59, 18 or 19 a cell, and steps onto the entrance's room at 198 at
  z 58. Nowhere else does either floor come within 18 of the other beside it. They are no warp
  or trigger: the zone has one warp, the way out, and no trigger. The top steps' inner cells are
  kinds `0x51` and `0x52`, which the League has side by side on an ordinary step, so they say
  nothing of stairs.
- **The nameless people are invisible blockers.** Nine NPC records drawn as `0x8a`, the ROM's
  `STOPPER`, sit in a square at (36, 70), off the map; in the replay they stood in rows of three
  across the three corridors up from the rooms, at z 45 (height 290), 34 (317) and 23 (371), on
  the floor above. `STOPPER` stands in three zones: this one, Fortree Gym, where they are the
  doors' stand-ins (spike 161), and Seafloor Cavern, six.
- **P walked one floor a cell**, the one nearest the player's height (spike 156), and where two
  floors overlap that broke it four ways. The Lass pacing the bottom room stood in the corridor
  above her: from the door, P to Wallace said "3 north, then can't continue, Lass in the way".
  The blockers above stood in the lower floor's corridors. A thing on the other floor was
  reached on this one: the way out from below, the Beauty below from the room over her. And the
  fallback of walking the whole map at another floor's height (spike 159) never goes from one
  floor to the other by the stairs.
- **Built: a stacked ground** (`Ground.stacked`, `Ground.climb`, `pathfind.find`'s `arrives`). A
  matrix with a deck is walked a floor at a time, the floor part of where the search stands. A
  step goes on along its floor, or onto another within 18 of the one it leaves, the nearest.
  Whoever stands in a cell stands on its floor nearest their height, and move models, warps and
  triggers now carry theirs. A thing is reached on its cell's floor within 36 of its height, and
  a way that falls short ends on the thing's own floor first. A cell with no height, as the
  bottom room's edges are in the lower floor's table, keeps the floor it is walked on.
- **Built: invisible blockers and stairs** (`oras.STOPPER`, `Game.blocker`, `Game.stairs`,
  `mapblock.joins`). A blocker is never listed, and stands in the way but round a rotating door;
  the doors' own copy of the sprite is gone. A way it shuts ends with "then can't continue",
  naming nobody. In the Gym, each run of the ground's cells beside a floor over it within 18 is
  listed under exits as "stairs": from the door, "stairs 6 west 13 north; stairs 6 east 13
  north".
- **Checked** over the ROM's chunks with everyone where the replay had them: from (14, 72) the
  way out is "2 west, 13 north, 2 east, 4 south, 3 east, 10 south", up the west stair, through the
  entrance's room and down its corridor; from the door, the Beauty below is "12 north, 4 west, 1
  north, 2 west, 11 south, 5 east, 16 north"; Wallace from anywhere goes up to the blockers and
  says "then can't continue". Route 119's way north from its foot (spike 159's capture) is the
  same as the one-height fallback found, "2 east, 14 north, 10 east, 54 north, 8 east, 12 north,
  48 west, 10 north", in 0.11 s. Replayed with the runtime, the list has no nameless people, and
  P from (19, 59) after the battle to the way out says "11 south, 5 east, 11 north, 2 west, 4
  south, 3 west, 10 south", up the east stair.
- **Built: P over the ice** (`oras.ice`, `IceRoom`, `pathfind.cover`, `navigate._over_ice`).
  The rooms are read whenever the ground is, and their cracked tiles stand in the way on their
  floor while the room is not cleared. Where a way falls short within three cells of a room's
  exits, or the player is in the room, it goes onto the room and over every whole tile once,
  ending on an exit, said with "and the ice is cleared", and "ice cleared" on the last tile; a
  way that crosses the floor among the ice twice is followed from the leg being walked. The way
  over a room is found by a search in depth taking the tile furthest from the exits first, then
  the one with fewest ways on: Warnsdorff's rule alone did not cross the top room in 400,000
  tiles tried, and furthest first crosses each room from each way onto it in under 20 ms.
  Replayed, P to Wallace from the first room's corridor says "2 north, 2 west, 4 north, 1 east,
  3 south, 2 east, 1 south, 1 east, 2 north, 2 west, 1 north, 2 east, 1 north, 2 west, and the
  ice is cleared", and from the lower floor goes up a stair first. Over the ROM's chunks, with
  the first room cleared and its blockers gone the way goes over the middle room, and with all
  three, straight to Wallace.
- **Left open**: whether the blockers leave the corridor once its room is cleared, and the work
  values read from the save on a later visit, neither seen (no movie clears a room); a room
  cleared is taken to be one whose record says so or that keeps no tiles; whether the stair's top step joins the room sideways as well as north, which
  P takes; and the one-height fallback, now only for a ground not stacked (Sun's), to delete once
  this is confirmed.

## Spike 166 — soaring on Latios: where the player is over Hoenn, and the places they can land at

2026-10-05 (`spikes/166_soaring.py`, and the ROM's `a/2/5/1` to `a/2/5/4`). The user, with eight
badges, used the Eon Flute and flew over Littleroot Town: "I don't think the map tracker works
there", and asked what the D-pad, the Circle Pad, X and Y do while soaring. omega_ruby_aeon-flute.ctm,
on the save of 13:30 in Littleroot Town (the rig's `save_checkpoint_soar`), replayed with 0 out of
sync three times; it ends on no save.

- **What the movie does.** The Eon Flute from the Y button; Latios climbs out of the town and
  hovers, facing north, 13 s; the Circle Pad up, down, left, right, down and up; Y held 2 s and X
  4 s; B, A and B; the D-pad up, left, down and right; B. The player never lands.
- **The sky is an app of its own**, `app::skyTrip::MainProc`, from the module `DllSkyTrip`
  (`0x00722000`, 0x12000 long in the replay, beside `DllSequence` and `DllBackGround`). The
  field's lists stayed as they were in the town all the while, which is what the user heard.
- **The camera holds the player.** The process's +0xb4 leads to an `app::skyTrip::Camera`,
  rebuilt once the climb is done; before that every word read is 0. Its +0x0c is the heading in
  radians, +0x4c, +0x50 and +0x54 the target's x, height and z: the player, at (-3896, 1900,
  9522) over Littleroot Town, heading pi. Its eye is 150 behind and 90 above.
- **What the controls do**, read at ten polls a second. The Circle Pad's up flies forward along
  (sin, cos) of the heading in (x, z), about 1,200 a second once up to speed, so pi is north;
  down flies back, slower; left turns on the spot, the heading rising about 1.2 a second, right
  the other way, with Latios banking as it turns. Y pitched Latios down and X up (+0x44 from 90
  to 148 and to 12) with the height left at 1900 while it hovered; the D-pad and B moved nothing.
  The height never left 1900 in the movie.
- **The bottom screen** is one picture, `map.bclim`, 320 by 240, and a pane over it holding
  `hero.bclim`, the player's arrow, turned with the heading. The arrow stands at
  `320 * (x + 14773) / 43698 - 160` to a tenth of a pixel, and its y as z over 136.6: the map is
  the world from (-14773, -11408) to (28925, 17074), letterboxed, a literal of `DllSkyTrip`'s
  (`0x0072ebb0` in the replay, beside 43698 and 28482).
- **The places are a table of the ROM's**, `a/2/5/2`'s one file, a two-letter archive "TM" of
  one file, "SLAN": 117 rows of 0x34 bytes from +0x28, each a zone (and up to three more of the
  same place), +0x10 a number from `0xb11` up, +0x14 a mirage spot's number, +0x18 0 for a place
  always there and 1 to 6 for the mirage spots and Latios's islands, +0x1c the colour a debug
  map draws it (white the 17 towns and the League, orange routes over land, cyan over the sea,
  tomato the caves and the rest), +0x20 1 but for one of Route 111's, and +0x24 x, z, width and
  depth as fractions of the world. A place in more than one rectangle has a row each: Route 104
  three, Route 126 three. 69 places are always there. Its neighbours are the sky's models and
  textures, "TM" archives of `BCH` and `CGFX` files, `a/2/5/1` holding a `SCOL`, not read.
- **+0x10 is not a flag of having been there.** At `field::EventWork`+0x300 the save's event
  flags sit in RAM as PKHeX lays them out, and `0xb11` (Littleroot Town) to `0xb21` were clear
  but for two, in a game with eight badges. Which places a player may land at is not read.
- **The place the game says is a little ahead of the player.** The top screen says the place as
  screen text, with "Land" beside it where the player can land: "Littleroot Town" hovering at
  z 9522, though its rectangle ends at 8831. Taking the first row whose rectangle holds a point
  ahead of the player, every one of the eight changes of the words fits a point 1,400 to 1,600
  ahead; whether the words stay put in between fits 1,150 to 1,350 flying north and more than
  1,400 turning on the spot. Neither the camera's eye nor its look at the ground explains it.
- **Built: the sky's places in the navigation keys** (`gen6.sky`, `RomLayout.sky`,
  `GameText.sky`, `SkySource`). While the sky runs, the keys walk its places under three new
  categories, towns, routes and landmarks, by the debug colours, said by the game's own place
  names and never renamed. Each is said from a point 1,300 ahead of the player to its nearest
  edge, in the player's own frame, a step 100 of the world's units: "Oldale Town, 2 of 17, 21
  ahead", "Slateport City, 42 right". Direction mode looks 100 each way, its keys ahead, right,
  behind and left. P says the smallest turn on the spot that brings the place to where the game
  looks, or the turn and how far to fly: "turn right 90 degrees, 29 ahead", "turn left 10
  degrees", "here". The way the player faces is said as a compass point once a turn stops,
  "facing south-west", as the arrow shows it.
- **Checked** replaying the movie with the keys pressed by the spike: over Littleroot Town at
  the start, "towns, 17", "Littleroot Town, 1 of 17, here", "Oldale Town, 2 of 17, 21 ahead";
  flying north, the game said Route 101 and Oldale Town about when the list's distances ran out;
  "facing north" once the camera was placed, and nothing while it read 0.
- **Left open**: which places the player may land at; the mirage spots and Latios's islands,
  there only some days, which are left out; the flocks of birds the player can fly into
  (`app::skyTrip::BMEncount`); the height, and whether X and Y change it while flying; the
  point the game looks at, measured only from the words.
- **Later the same day: the mirage spots, the flocks, and +0x10 after all.** The user asked
  for "the secret spots or the flying Pokémon", having flown to an island with a Hoopa ring. In
  their session of 14:18 the game said "Mirage Forest" and "Land" near the Pokémon League, at
  (25302, 2378) facing north, which the list had nothing for, and the League's "Land" never
  came; the height rose to 2190 flying east with X or Y held.
- **+0x10 is the flag of having been there, read elsewhere.** `DllSkyTrip`'s `0x007231dc`
  goes over the rows at the sky's start: a row with +0x20 0 is skipped; one whose +0x10 is a
  flag is kept only where the flag is set, by the executable's `0x0012fd48`, which reads flag n
  below 0xd00 as bit n & 7 of byte n / 8 from +0x2f4 of `field::EventWork`, itself +0x13140 into
  `savedata::SaveData` (the 0x300 above was 0xc off); kinds 0 and 1 then stay, and 2 to 6 only
  where +0x14 is today's mirage spot (`0x004608fc`, from the date, mod 33) or one of up to ten
  the save was given (`savedata::SangoNetworkSavedata` +0x51dc, u16 every 4 bytes). It writes
  the answer as a byte a row at the places' record's +0x40, the process's +0x154, and today's
  number at its +0x44: 2, Mirage Forest at (24780, 383). In the movie's save the League, Ever
  Grande City and Pacifidlog Town were not shown, nor Routes 127 to 134, and of the mirage
  spots a flag opens, Pathless Plain and Nameless Cavern were: 59 rows of 117.
- **The sky's models are a list.** The process's +0x160 leads to a manager whose +0xa0 holds
  the model built last, linked to the others at +0x04 and +0x08, each with its place at +0x20
  and its class saying what it is: `BMHikyo` the mirage spot at (24780, 0, 383), two
  `BMEncount`, the flocks, at heights 1700 and 1500, three `BMLegendPoint`, the red beacons, 600
  over Pathless Plain, Nameless Cavern and the Mirage Forest, and `BMShip`, the ship, at 90. The
  flocks and the ship move, the flocks about 300 a second; no model is a place, and the beacons
  stand over places the table has.
- **Built: what the game shows, the mirage spots and the flocks** (`sky.shown`, `sky.flocks`,
  `sky.chase`, `field.MIRAGE`). The places listed are the rows the game shows, so a place not yet
  visited is left out, as the game leaves it unnamed; the mirage spots shown are a category of
  their own, "mirage spots", named by the game ("Pathless Plain"); and each flock is listed under
  pokémon as "flock", pa3ds's word, said from the player rather than where the game looks, since
  flying into one is what battles it. P on a flock says the turn, the distance and how far
  above or below it flies: "turn right 45 degrees, 38 ahead, 2 below". Checked on the capture
  at 20 s (14 towns, 24 routes, 9 landmarks, 3 mirage spots, 2 flocks) and replayed with the keys,
  "flock, 1 of 2, 26 ahead 28 right" moving to "11 ahead 21 right" four seconds on.
- **Left open now**: whether diving to a flock's height is needed to battle it; the ship and
  the beacons, not listed; the point the game looks at, measured only from the words.

## Spike 170 — Ever Grande City's waterfall

2026-10-05 (`spikes/170_waterfall.py`, and the ROM's map chunks). The user, surfing at (51, 52) at
the foot of Ever Grande City's waterfall (zone `0x15`), pressed P to the Pokémon Center: "no path
to Pokémon Center". They used Waterfall, P from the top gave a way, and they went back down,
where P said "no path" again. P is to stay off the waterfall, and Shift and P to climb it (the
user, 2026-10-05). omega_ruby_waterfall.ctm, on the save of 18:27 (the rig's
`save_checkpoint_waterfall`), replayed twice with 0 out of sync; it ends on no save.

- **The waterfall is kind `0x3f`, a row of blocked water one cell deep.** Ever Grande City's is
  `0x3f180003` from x 36 to 54 at z 51, the water bit and the blocked bit both set, with the sea
  `0x3d` on both sides: the sea below at height 0 by the ROM's ground heights, the water above at
  234. Spike 162 left the kind unknown.
- **Waterfall carries the player over it, as Surf jumps the shore.** The climb took the player
  from (51, 52) to (51, 50), polled at (51, 51) on the way, after "It's a large waterfall. Would
  you like to use Waterfall?" and Yes. Going back down from (51, 50) to (51, 52), the game asked
  nothing.
- **Every waterfall in ORAS goes up north.** All 72 cells of `0x3f` in the ROM's chunks, in 8
  files, are the one word. The 60 with water on both sides inside a chunk are all higher to the
  north; 9 have rock north and south, the row's ends against a cliff, and 3 sit on a chunk's
  edge.
- **Built** (`field.FALLS`, `Ground.falls`, `Ground.over`, `Ground.climbing`, `FieldReader._rises`,
  which also gives the bumpy slopes their way up): a ground with HMs jumps a waterfall either
  way, from the cell before it to the cell after, going up at the five steps' cost of going onto
  water and said with the game's name for the move, and down with no cost and no word; any
  other ground walls it. Guiding waits while the player is on a waterfall's cell, between its
  two sides. Replayed, Shift and P at the foot says "20 west, Waterfall 2 north, 15 north, 10
  east, 7 north, 10 east, 5 north" and P there "no path to Pokémon Center"; from the top both
  find the way on.
- **Left open**: X's waterfalls, which the same height reading should place but no movie has
  checked; and whether the game lets a player without Waterfall go down one.
