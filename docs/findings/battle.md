# Battle

The HP box, the battlers' names, the gauges behind each panel, and the Pokémon records the game
keeps them in.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 36 — the battle's layouts are in the linear heap

2026-09-14 (`spikes/31_battle_layouts.py`, movie3 slot 4, censuses 40, 44, 50 and 57 s after
reading started, heap and linear heap swept).

- **Everything the battle draws in 2D is in the linear heap.** 11 to 13 layouts there against the
  field's 16 in the heap; 8 to 10 layout works there against 1 in the heap; and every one of the 14
  to 36 TextBoxes. A sweep of both took 2.8 s.
- With works from both heaps tracked, the reader placed and read "What will Torchic do?" and the
  battle messages (the message box's layout, "unknown" in spike 34), Torchic's level "5" and HP
  "19/ 19" (one TextBox, not three as on the party screen), and a "3", the Poochyena's level.
- **The command menu said nothing.** The picture at 44 s shows FIGHT, BAG, RUN and POKÉMON on the
  bottom screen. At that census 22 TextBoxes led to no tracked layout. Whether they are the menu,
  and why their roots are unowned, is the next question; the spike now prints their words and
  chains.
- The "3" was reported shown at 40 s, when the picture is the send-out flash with no HP box yet.

## Spike 45 — the battle's move list, and how to reach it

2026-09-14 (`spikes/31_battle_layouts.py`, slot 4, censuses of heap and linear heap at 42 and 45
s).

- **The move list is screen text in the linear heap.** At 42 s, with the moves up, the shown
  TextBoxes were "Scratch", "PP", "35", "/", "35", "Growl", "PP", "40", "/", "40", "Ember", "PP",
  "25", "/", "25", beside Torchic's "5" and "19/ 19", the Poochyena's "3" and "What will Torchic
  do?". All 36 TextBoxes and 10 of the 11 layout works were in the linear heap.
- The command menu (FIGHT, BAG, RUN, POKÉMON, spike 36's picture) holds no TextBox. Its words are
  most likely drawn in its pictures, and would need a different anchor.
- **Nothing in the heap points at the linear heap's layout works or Layouts**, so the runtime cannot
  follow a pointer to them from what it already sweeps. It has to sweep the linear heap, 28,000
  pages against the heap's 10,000; a whole sweep of both took 2.9 s.

## Spike 50 — battles.ctm, the battle menus

2026-09-14. The user recorded leaving the house for Route 101 and two Wurmple battles, arrowing
round the command menu, the move list, the bag and the party screen, with a level up in the first.
Replayed from boot on a copy of their save, the runtime said:

- "Route 101" once, the location sign's copy left out; "A wild Wurmple appeared!" and every battle
  message; the move list whole each time it came up, seven times in the first battle, once as
  "Scratch PP /35"; the party screen as "Choose a Pokémon. Torchic L6 22/22"; the bag's pocket
  names whole each time, six times.
- **Nothing** for the command menu (67.8 s to 87.5 s of arrowing), for any move under the cursor,
  for the bag's or party screen's cursor, or for the level up's stats after "Torchic grew to Lv. 6!".
- States saved every 20 s from 40 s: slot 2 at 60 s is before the first battle, 3 at 80 s in its
  command menu, 4 at 100 s in its move list, 5 at 120 s before the level up, 7 at 160 s in the
  second battle's command menu, 8 at 180 s before its bag, 9 at 200 s before its last move.
- **The command menu's cursor is a Picture alone in a layout of its own** (`spikes/45_battle_menu.py`,
  slot 2): root 0x14ba71e0, one Picture at alpha 210, which jumped as the player arrowed round to
  (39, 12), (128, -69), (0, -87) and (-128, -70), the global places of four empty panes of the
  command menu's layout, "2j", "2l", "2m" and "2k", one in each button's group. Then it pulses in
  scale where it sits. Nothing else in the command menu's layout changed.
- **The move list** (root 0x14728514) has four slots, each a group holding the move's TextBoxes
  ("Scratch", "PP", "35", "/", "35"), a picture, a Bounding, and an empty pane at the top of the
  button, placed like the command menu's.
- **The layout files name their textures, and the names are words** (`spikes/46_layout_resources.py`,
  slot 3). The command menu's CLYT is resident at 0x145efeb0: its 66 panes in the order the runtime
  built them, and a texture list with `fight_button.bclim`, `fight_text.bclim`, `bag_button.bclim`,
  `pokemon_button.bclim` and `run_button.bclim`. Group "23" holds "2j" and pictures textured
  `fight_button` and `fight_text`; "20" holds "2k" and `bag_button`; "1z" holds "2l" and
  `pokemon_button`; "1w" holds "2m" and `run_button`. A picture's material is named as its pane,
  so the pane leads to its texture through the file's `mat1`.
- **The cursor is 20 by 20, on 10 by 10 or 20 by 20 empty panes** (spike 45 again, slot 3, with
  sizes and parents). In the move list it went to "30" under "7", "34" under "m", "32" under "11",
  "37" under "1g" (the empty fourth slot, at alpha 100) and "3i" at (140, -85). The move list's file
  (0x14650ce0, 81 panes, slot 4) makes "7", "m", "11" and "1g" the move plates, textured
  `waza_plate_00.bclim`, each holding its move's TextBoxes; "3i" hangs from "27" beside a picture
  textured `return_button.bclim`. Panes and the cursor's own root sometimes read 1,000 units off
  for a single poll.
- **The level up's stats were left out as off screen** (`spikes/47_all_text.py`, slot 5). Layout
  0x14d470e8 showed "HP + 2", "ATTACK + 1", "DEFENSE + 1", "SP. ATK + 2", "SP. DEF + 1", "SPEED + 1",
  sliding in, then each value became the new stat ("22", "13", "10", "13", "12", "11"). Every piece
  read about 1,000 units up, at y 977 to 1,074, as do all the battle's top-screen layouts: roots at
  (0, 1000), "Torchic grew to Lv. 6!" at y 913, Torchic's HP box at y 906. The off-screen rule of
  spike 48 left them all out, which is also why this replay never read the battle's HP. The
  battle's layouts also read as drawn to the bottom screen, though these are on the top.
- **The 1,000 is a view offset** (`spikes/48_view_offset.py`, battles.ctm slot 3 and menus.ctm
  slot 4). Every layout work on the bottom screen holds a float at +0x74, 1,000 in battle and 290 on
  the field, and it is in the roots' global places: the battle's bottom-screen roots (the command
  menu, the move list, the cursor) sit at local (0, -1000) and global (0, 0); its top-screen roots
  (the HP boxes, the message box) at local (0, 0) and global (0, 1000); the X menu and AreaNav at
  local (0, -290) and global (0, 0). The X menu's root stayed at (0, 0) as it parked its labels.
  Words are now judged off screen from their layout's root. The battle's top-screen works read the
  bottom screen's projection (-0.00625), so they were taken for the bottom screen, 320 wide, and
  the level up's values at x 160 to 306 read as past its side; replayed, the level up said only
  "HP ATTACK DEFENSE SP. ATK SP. DEF SPEED". A root 500 or more from the middle is now taken for
  the top screen's, whatever the projection says.
- **Replayed with picture cursors and these fixes** (battles.ctm from boot): the command menu as
  "fight button", "pokemon button", "run button", "bag button" as the player arrowed; the move list
  as "Scratch PP 35/35", "Growl PP 40/40", "Ember PP 25/25", "empty" and "return button"; the
  party screen as "Torchic L6 22/22"; the battle bag's pockets one at a time under the cursor;
  Torchic's HP box as "5 20/ 20 3" (the Wurmple's level beside it); and the level up as "HP + 2
  ATTACK + 1 DEFENSE + 1 SP. ATK + 2 SP. DEF + 1 SPEED + 1", then "HP 22", "ATTACK 13" and the rest.
- Still rough: the HP box is read in pieces three times as it slides in for the second battle; the
  party screen's "Choose a Pokémon." is left to its cursor with the rest of the layout; and the
  first battle's first cursor item came about 2 s late in two replays of three, missing "fight
  button", most likely while the command menu's layout file waited for the rolling sweep.
- **No layout points at its file.** None of the 11 layout objects' first 0x80 bytes lands inside
  any of the 33 CLYT files swept. But the runtime built the command menu's panes in the order its
  file lists them, so a layout's pane names, walked depth first, pick out its file.

## Spike 53 — the battle's HP box, and what a panel is

2026-09-16 (`spikes/54_hp_box.py`, battles.ctm slot 6, the second battle's box arriving). The user,
on their first full play: "HP in battles being slightly odd". The battles.ctm replay said what they
heard, all from one layout, `0x14d48708`:

    69.93  5 20/ 20 3      147.77  6 22/ 22 2
   119.61  5 20/ 20        148.00  22/ 22
   130.89  6 22/ 22        150.72  6 22/ 22 2
   140.04  6

- **Two Pokemon in one utterance.** The layout holds both boxes, and the game keeps them apart by
  the pane each hangs from directly under the root: `'6'` and `'22/ 22'` under `0x14d4b614` at
  (-76, 906) and (-84, 888), and the wild Pokemon's `'2'` under `0x14d4dcd4` at (174, 1108). The
  announcer joined a whole root into one run, so Torchic's level and HP ran together with somebody
  else's level. Reading a **panel** at a time fixes it, and panels come free: `Layouts.placement`
  already walks every pane to its root, so the last pane before the root is one assignment away.
- **The same rule reaches two more screens.** The party screen keeps `"Choose a Pokémon."` under
  `0x1460e0cc` and its slots under `0x14611fc0`, so the title can be told from the rows it was
  being read with; the battle bag gives each pocket name its own panel (`0x1460ff18`,
  `0x14613fe8`, `0x14616794`, `0x1461a864`, `0x1461d010`).
- **Layouts were being joined too.** The old code gathered every layout that arrived in one poll
  and spoke them as a single run, which is how a move list and a level from a different layout
  ended up in one breath. Each panel is now its own utterance, queued in build order.
- **`'22/ 22'` is one TextBox**, not the three the party screen uses. The pieces are not the fault.
- **The root's y alternates 1000 and 0 between polls**, which is the double buffering AGENTS.md
  warns of rather than the box sliding. And a poll caught the party screen mid-rebuild, reading its
  root at `(nan, 7.4e28)` with pane addresses like `0xea4100ff` — a torn read, ridden out because
  nothing is believed from one poll alone.
- **Left open:** what the HP box should say at all. the behaviour ADR (now ADR-0015) says HP is
  on-demand, with automatic announcements only on faint and on dropping into the red; what the
  replay shows is the general screen text reader picking the box up because it came on screen. The
  numbers also carry no labels, and a lone "2" breaks the user's own rule that a bare number is
  useless. That is the user's call, not a bug to fix quietly.

## Spike 54 — what the battle's HP box holds

2026-09-16 (`spikes/55_battle_hp.py`, battles.ctm slots 3, 4 and 5). Towards the user's HP spec.
The box is one layout, `0x14d48708`, with a panel each:

    player  0x14d4b614   ally_gauge.bclim 128x32
                         black.bclim  w=48.0 h=4.0  at (-124, 897)   the HP bar
                         TextBox '5'                                 the level
                         TextBox '20/ 20'                            the HP
                         name_blank.bclim 47x14 at (-104, 909)       a blank plate
    enemy   0x14d4dcd4   enemy_gauge.bclim 128x32
                         black.bclim  w=48.0 h=4.0  at (131, 1099)   the HP bar
                         TextBox '3'                                 the level
                         name_blank.bclim 47x14 at (146, 1111)       a blank plate

- **The bar is the pane's own width, and 48.0 is full.** Watched through the first Wurmple taking
  Ember (slot 3), the enemy's `2g` read 48.0, then 30.0, then 27.0, then 0.0 as it fainted. Scale
  stays 1.0 throughout and the pane does not move, so the fraction is the width alone: 30 of 48 is
  62%. That is the enemy percentage the user asked for, with nothing to infer.
- **There is no name anywhere in the layout, and no gender sign.** The only TextBoxes in the two
  panels are the two levels and the player's HP; the names are drawn onto `name_blank` as images.
  So "Torchic ♀" cannot come from the layout at all. Spike 53's "no names" was not a reader fault.
- **What that means for the spec.** The names, the gender and the enemy's exact level have to come
  from the battle's own Pokemon structures. Worth doing rather than worked around: ORAS keeps a
  Pokemon's nickname as UTF-16 in its struct and an un-nicknamed one carries its species name
  there, so "Torchic" and "Wurmple" come straight out of RAM with no ROM extraction; and the same
  anchor gives level, current and maximum HP directly, which is steadier than scraping `'20/ 20'`
  off a TextBox and is what the on-demand keys would read too. Finding and validating those
  structures is the next spike, and the movies already here reach every state it needs.
- The player's bar never moved in any of the three states — Torchic was never hit in battles.ctm —
  so the ally bar's full width is 48.0 by symmetry with the enemy's, not by measurement. A movie
  where the player takes damage would settle it, and would also show whether the ally bar is read
  the same way.

## Spike 55 — the battlers' names and genders are in RAM after all

2026-09-16 (`spikes/56_battle_mons.py`, battles.ctm slots 3 and 5 compared). Spike 54 said the HP
box holds no name and no gender, so the search moved to the game's own data. Searching the heap for
"Torchic" as UTF-16 and diffing the two states found them:

    0x08554050  'Torchic♀'        0x08554138  'Wurmple♀'        0x08554220  'Wurmple♀'

- **Three buffers, 0xe8 apart, at the same addresses in both states.** Torchic first, then two
  Wurmples — more slots than the battle has participants, so the array is longer than what is in
  use and a slot has to be checked before it is believed.
- **The gender sign is inside the string**, right after the name, as U+2640. Not a neighbouring
  field: the halfwords run 84 111 114 99 104 105 99 9792 — "Torchic" then ♀. So "Torchic ♀" is one
  read, and it is already the sign a screen reader says rather than the font's private-use glyph
  (U+E08E/E08F) that the party screen uses and `speech.SYMBOLS` maps.
- **This is a string cache, not the Pokemon struct.** What follows each name is pointers
  (`0x0855xxxx`), and nothing near it tracked Torchic's level from 5 to 6 or its HP from 20 to 22.
  The level and the player's HP are already readable as the HP box's TextBoxes ('5', '20/ 20'), so
  between the two, everything the user's spec needs is in hand **except the enemy's percentage**.
- **What is still assumed:** that the first buffer is the player's side and the later ones the
  enemy's, which is the array's order and not proof; and that the sign varies, since all three
  Pokemon here are female. A movie with a male battler would settle both. The rival battle in
  movie3 would do it.
- **Method note.** The first pass looked for halfwords near the name matching a value the state was
  known to show, and that was the wrong instrument: 20 and 2 are common numbers and it produced
  noise on every hit. What worked was dumping a fixed span in two states and diffing it, which is
  also how the 0xe8 stride showed up.

## Spike 56 — the name array holds for another battle, and where the real HP is

2026-09-16 (`spikes/57_hp_fields.py`, movie3 slot 4, the Route 101 Poochyena). Two assumptions from
spike 55 needed a battle battles.ctm cannot give: a male battler, and the player taking damage.

- **The gender sign varies, so it is data.** movie3 reads `'Torchic♂'` and `'Poochyena♂'` against
  battles.ctm's three `♀`. Same format: name then sign, one UTF-16 string.
- **The slot order holds.** The player's buffer is at the region base +0x050 and the first enemy's
  at +0x138 in *both* movies — `0x08554050`/`0x08554138` in battles.ctm, `0x0855b050`/`0x0855b138`
  in movie3. So the first buffer is the player's side, which spike 55 could only assume.
- **The buffers are packed and variable length.** The third sits at +0x220 in battles.ctm and +0x224
  in movie3, exactly the 4 bytes by which `'Poochyena♂'` is longer than `'Wurmple♀'`. So a fixed
  stride cannot be used to step the array; each name has to be read and its length taken.
- **The real current HP is two halfwords, found by narrowing against the screen.** Sweeping the heap
  for every address holding the HP the box draws, then keeping only those that still agree, went
  1,398 -> 2 the moment the Tackle landed: `0x0855b5b0` and `0x0855b5b4`, both 19 and then both 16.
  Two copies four bytes apart. Reproduced on a second run.
- **Maximum HP is not in that region at all** — no halfword equal to 19 anywhere in the 2,560 bytes
  around it. So these two are a display or battle-parameter struct, not the Pokemon's own record,
  and the on-demand keys will need the real struct found separately.
- **The spec does not need it.** Everything it asks for is already in hand: the name and gender from
  the buffer, the level and the player's `'16/ 19'` from the HP box's own TextBoxes, and the enemy's
  percentage from the bar's settled width over 48 (spike 54) — which is also the parity-correct
  source, since the bar is what the game shows for the enemy.
- **What is still missing is an anchor.** The buffers are at a different base in each playthrough,
  so they cannot be reached by address, and nothing yet ties them to the battle. The pointers
  massed just before the array (`0x0855xxxx` words, spike 55's diff) are the first place to look.

**Method.** The narrowing only works on a *settled* reading: the HP box counts down as the bar
drains while the struct jumps straight to its final value, so screen and RAM disagree for about a
second after a hit and intersecting then would discard the very address being sought. The first run
got away with it by luck of timing; the spike now waits for the number to hold still.

## Spike 57 — the anchor for the battlers

2026-09-16 (`spikes/58_name_anchor.py`, battles.ctm slot 3 and movie3 slot 4). Spike 56 left the
names readable but unreachable: the array sits at a different base in each playthrough, and
"anchors, not addresses" forbids either address. This found what to scan for.

- **A UTF-16 run ending in ♂ or ♀ is almost the anchor by itself.** Across a 24 MB heap, during a
  battle, there are **exactly three** — the battle roster and nothing else. In movie3 the heap holds
  fifteen buffers reading "Torchic" (party, boxes, elsewhere) and only three carry a sign, so the
  sign is what makes it specific. The party screen does not: it draws gender as a separate TextBox
  in the font's private-use glyph, so only the battle's own buffers embed U+2640/U+2642.
- **Each buffer is owned by a `gfl::str::StrBuf`** (`N3gfl3str6StrBufE`), pointing at its characters
  from +0x04 — the class pa3ds already reads for dialogue, where +0x04 is the text and +0x0a the
  length. So the scan is the ordinary one: `find_vtables` for the class, `find_objects` for its
  instances, then keep those whose text ends in a sign.

      battles.ctm  StrBuf 0x08554010 -> 'Torchic♀' 0x08554050   (text at +0x40)
      movie3       StrBuf 0x0855b010 -> 'Torchic♂' 0x0855b050   (text at +0x40)

- **It holds across playthroughs, species and genders.** Different heap base, different Pokemon,
  ♂ where the other movie had ♀, and the same shape both times: three strings, lowest address the
  player's side (spike 56).
- **It is battle-scoped for free.** Swept before the battle started, movie3 gave *zero* gendered
  strings. So "none found" means "no battle" rather than an error, and a reader can scan for them
  continuously without a special case for being out of one.
- **Do not use the 0xe8 stride.** The owners are 0xe8 apart in battles.ctm, but the text is packed
  variable length (spike 56), so each StrBuf's own +0x04 is the only correct way to its characters.
  The stride is an observation, not a rule.

**What this completes.** Everything the user's battle HP spec needs is now reachable by anchor:
name and gender from the StrBuf, level and the player's current and maximum HP from the HP box's
TextBoxes (spike 54), and the enemy's percentage from the bar's settled width over 48 — the
parity-correct source, since a bar is all the game shows for the enemy. The real Pokemon record is
still unfound and still wanted, but only for the on-demand keys, which have to answer when the box
is not up.

**A measuring lesson.** The first movie3 run swept 8 s after attaching and found nothing, because
that state does not reach Route 101 for another 37 s. It was measuring the wrong moment, not the
wrong thing. The spike now waits for a gendered string to appear instead of guessing a delay, which
is also what will make it re-run cleanly on a trainer battle.

## Spike 65 — the save's own Pokémon records are in RAM

2026-09-17 (`spikes/67_save_in_ram.py`, battles.ctm slots 3 and 6, the heap and linear heap swept
whole, against the rig's save file). The user pointed at PKHeX (https://github.com/kwsch/PKHeX,
GPL-3), a save editor, and asked whether what it knows about ORAS saves helps. Its formats were
read; none of its code was used.

- **What PKHeX documents.** An ORAS save is 0x76000 bytes of blocks at fixed offsets, with a table
  of CRC16s at the end. Each Pokémon is a 0xE8-byte **Pokémon record**: an encryption constant, a
  16-bit sum of the halfwords from 0x08 to 0xE8, then four 56-byte blocks shuffled by the constant's
  bits 13-17 (mod 24) and XORed with an LCRNG (`* 0x41C64E6D + 0x6073`, high halfword) seeded by the
  constant. A party member adds 0x1C bytes of battle stats (status, level, current and maximum HP,
  five stats) XORed with the same keystream started over. Decoded from the rig's two saves: a level
  5 Torchic, ♀ 20/20 in the one battles.ctm plays on and ♂ 19/19 in movie3's, the genders the
  battle's own strings showed (spikes 55 and 56).
- **The sum is a validator.** Every 4-byte-aligned place in 36,779 pages was tried as a record. Eight
  encrypted ones checked out with a species from 1 to 721, and every one was a Pokémon of that
  battle: five Torchic and three Wurmple. Tried as already decrypted, one or two places passed with
  a species and no names at all, which a name check rejects.
- **`pml::pokepara::PokemonParam` is the live Pokémon.** Its vtable is in the executable, so it is
  found by class. +0x04 points at its record, which sits at +0x40, and +0x08 at its battle stats. At
  slot 3 there were seven, four Torchic and three of the wild Wurmple, reading level 5 20/20 and
  level 3 16/16 as the screen did. At slot 6, the next battle, the same addresses read Torchic level
  6 22/22 and the new Wurmple level 2 13/13, again as the screen showed. The wild Pokémon's record
  carries its name in the nickname field ("Wurmple", written by the game in its own language), its
  gender, level and held item (151 at slot 6).
- **`savedata::PokePartySave` is the party as last saved, not as it is.** Its +0x04 is the save
  file's party block byte for byte, and at slot 6 it still said level 5 20/20 where the screen said
  level 6 22/22. `savedata::MyStatus` +0x08 holds the trainer's ID and name; there were two.
- **The save is not one block in RAM.** Each save block is an object of its own, so PKHeX's block
  offsets do not carry over, only what is inside each block: `MyStatus`'s data ends 0x17c before the
  party's where the file puts them 0x200 apart.
- **The battle keeps a record per battler** (slot 3), 0x244 apart, with no vtable. +0x00 is the battler's
  `PokemonParam`, +0x08 its experience (159, as the save), +0x0c its species, then 20 twice, its
  ability at +0x16 (Blaze and Shield Dust), its level at +0x18, and further on its moves with their
  PP (Scratch 35/35, Growl 40/40, Ember 25/25). There were four: Torchic, Wurmple, Torchic, Wurmple.
  Which of the two 20s is the current HP needs a movie where the player's side takes damage.
- **What it offers.** The on-demand battle keys needed the Pokémon's own record (spike 56): this is
  it, live, with level, HP, maximum HP, stats and moves. It also bears on a fault the user heard in
  play: with a second Pokémon in the party, the enemy's HP was said under that Pokémon's name. The
  battle reader takes the second gendered name as the first enemy's (`_enemy_name`), and spike 57's
  "lowest address is the player's" said nothing about a party of two. Which battle record belongs
  to which side is the question a movie with a party of two, and a trainer battle, would answer.
- **Method note.** The first decoder had two entries of the block order swapped. A sum does not care
  about order, so those records still passed the check and decoded to nonsense: slot 3's Wurmple
  read right and slot 6's did not. Check a decode by its name, not by its sum alone.

## Spike 66 — which Pokémon each HP panel shows

2026-09-17 (`spikes/68_battle_sides.py`, battles2.ctm slot 1, code, heap and linear heap swept).
The user's second playthrough had a Ralts, Emrys, added with PKHeX beside Torchic, Blaze, and the
battle reader named the wild Poochyena "Blaze♀", then said Blaze's HP under Emrys's name after a
switch. The reader took the gendered StrBufs in address order, the player's first (spike 57), which
only held while each side had one Pokémon.

- **The name plates are one per Pokémon in the battle, not one per side.** Four this time, 0xe4
  apart: "Emrys♂", "Blaze♀", "Poochyena♂" twice. The party comes first, so the second is the party's
  second Pokémon whether or not it is out. Each is held by a 0x54-byte entry of a list, beside a
  47 by 20 slot 20 units below the one before: plates rendered for the HP box's name picture.
- **The battle keeps a record per Pokémon, 0x244 apart**, for the whole of both parties (spike 65's
  "four" in battles.ctm was two Pokémon twice): Emrys, Blaze, Poochyena, and the three again.
- **A gauge says which one a panel shows.** `btl::BtlvGaugeSelf` (the player's) and
  `btl::BtlvGaugeBase` (the enemy's), in the heap, each hold:

      +0x04 -> btl::BattleViewSystem      +0x0c -> the battle record of the Pokémon shown
      +0x14 u32 0 the player's side, 1 the enemy's
      +0x18 -> a pane of its panel         +0x2c -> its level TextBox   (+0x5c its HP, the player's)
      +0x3c u32 the HP shown               +0x40 u32 the HP it is counting towards

  With Blaze out, the player's gauge held Blaze's record; the enemy's, Poochyena's.
- **Their classes are in the battle's DLL**, loaded after pa3ds reads the executable, so no sweep
  can look for them by class. The gauge holds its panel's level TextBox, which the battle reader
  has in hand, so a gauge is found as what holds that TextBox, and checked by the Pokémon its record
  leads to (`pokemon.read`).

## Spike 67 — what a gauge's numbers do

2026-09-17 (`spikes/69_gauge_watch.py`, battles2.ctm slot 2, every gauge read every 30 ms through
May's battle). The user heard "Emrys♂ L5 9/19" and then "19/19" when a Potion took Emrys from 3/19.

- **+0x3c is the HP the box shows and +0x40 the HP it is going to.** On a hit +0x40 drops at once and
  +0x3c follows it a point a frame (33 ms), the HP TextBox reading the same as +0x3c throughout:
  16, 15, … 9 over a quarter of a second. The Potion set +0x40 to 19 while +0x3c still read 3,
  then +0x3c counted 4 to 19 over half a second. The enemy's gauge does the same.
- **The battle record's +0x10 is the real current HP and +0x0e the maximum**, changed at the moment
  of the hit or the Potion. The PokemonParam's own stats stay at the battle's start until it ends.
- **+0x54 and +0x58, the player's only, fill as experience is gained**: 42, then 44 to 100 as Emrys
  levelled, then 0.
- **So a reading has settled when +0x3c equals +0x40.** Holding for a poll was not enough, because a
  poll can land twice inside a count.

**Built 2026-09-17** (`battle.Gauges`, `pokemon`): the reader names each panel from its gauge's
Pokémon, the name plate whose text is its nickname and a sign (the gender picks between two alike,
and a Pokémon with no gender is said by its nickname), and says nothing while a gauge is counting.
A sweep is asked for as soon as a panel's gauge is wanted, since only a sweep finds one. Replayed
from battles2.ctm's first frame: "Emrys♂ L5 19/19", "Poochyena♂ L3 100%", then after the switch
"Blaze♀ L5 21/21", "Blaze♀ L5 18/21", "Poochyena♂ L3 42%", "Blaze♀ L6 17/23"; May's "Mudkip♀ L5
100%"; and the Potion once, "Emrys♂ L5 19/19".

## Spike 124 — Sun's first trainer battle: the HP box, its gauges, and the command menu

2026-09-27 (`spikes/124_sun_battle.py`, sun_9.ctm: the user's recording of Hau's battle in Iki
Town, Popplio against Rowlet, won with a level up, on his save of 09:29 kept as `user/save_sun_9`;
replayed from its first frame with asynchronous reads, then from states saved at 65 s, the fight
menu). The user's own log of the session had already shown what Sun draws: screen text said
"Popplio Rowlet 225２２" as the box arrived, "Rowlet 5 22２２" and then "21２２", "20２２", "19２２" as
the count ran, "Popplio 5" each time the enemy's plate came back after an attack, and nothing at all
for the Fight, Pokémon, Bag and Run buttons.

- **The HP box is one layout, a panel a side, and every word of it is screen text.** Root
  `RootPane`, 385 panes; the player's panel `HPndBall_LB`, the enemy's `HPndBall_RT`, with `_LT`,
  `_RB` and four `_Roya` panels hidden for double battles and the Battle Royal. In each panel's
  `PokePlate` Parts: `TextBox_00` the name ("Rowlet", "Popplio"), `TextBox_01` the level ("5") on
  the name's row, and in the player's `Gauge_00` two more on a row of their own: `TextBox_00` the
  current HP "22" and `TextBox_02` the maximum "２２", **in fullwidth digits**, with a 10-wide
  `TextBox_01` between them reading nothing (the slash, drawn otherwise). ORAS drew the names as
  pictures (spike 54); Sun writes them, so no name plate StrBuf is needed, and none ends in a
  sign: there were no gendered `gfl2::str::StrBuf`s in any heap.
- **The gender is a picture**, `Icon_Gender`, 12 by 12, in a `Parts_Gender`; which sign it shows
  is in its texture, which Sun's readers have no layout file to read. The screenshot at 12 s shows
  "Popplio ♂ Lv.5", and Popplio's record reads gender 0, male; Rowlet's 1, female.
- **The gauge is three pictures under `Battle_Hpgauge`**, all 16 tall: the frame and the backing
  112 wide and centred, and between them **the fill, 104.8 wide when full and anchored left**
  (+0x47 low bits 1), the one pane of the three whose width moves. Popplio's read 104.8 at full,
  then 97.9, 91.9, 85.0, 79.0, 73.1 and 65.2 a screen poll apart after Leafage, and stayed at 65.2:
  62% of a full fill, for 12 of 20 HP. Rowlet's went 104.8, 102.5, 99.8 … 86.8 for 22 to 18. The
  whole gauge hides during an attack's animation (93.6 s to 98.2 s) and comes back, which is why
  screen text said "Popplio 5" again and again.
- **`btl::BtlvGaugeBase` is Sun's gauge, one a panel, and its class is found the ordinary way**:
  the battle's module brings it, and `Discovery.learn` picks it up as the battle app starts, where
  ORAS's had to be found through the level TextBox it held (spike 66). Read every 20 ms through
  Pound and Leafage (a block of 0xc0):

      +0x0c -> the battle record of the Pokémon shown    +0x1c -> its PokePlate Parts pane
      +0x20 -> its panel (HPndBall_LB, HPndBall_RT)       +0x24 u32 0 the player's, 1 the enemy's
      +0x28 u32 the HP shown, a point or two a poll       +0x2c u32 the HP it is counting towards
      +0x30 u32 the maximum HP, 0 until the first count   +0xa4 the count's frame, 0 to 0x28

  A hit sets +0x2c at once (0x16 to 0x12; 0x14 to 0x0c) and +0x28 follows it down over about 1.3 s,
  the fill with it, so **a reading has settled when +0x28 equals +0x2c**, as in ORAS (spike 67).
- **The battle record is laid out as ORAS's** (spike 65): +0x00 its `pml::pokepara::PokemonParam`,
  +0x0c u16 the species (722 Rowlet, 728 Popplio), +0x0e u16 the maximum HP, +0x10 the current HP.
  It has no class. **The PokemonParam decodes with ORAS's code**: the same 0xE8 record at +0x40,
  the same encryption and checksum, the battle stats at +0x08 (level 5, 22/22 and 20/20), with the
  species past ORAS's 721. Sixty of them in the battle's heap, most empty.
- **The battle's heaps are under heap 5, which the runtime walks already**: 5 → 0x42 → 0x44 (the
  battle's core, `btl::BattleAi`, `ServerCommandCalculator`, the 60 PokemonParams, 1.3 MB) and
  0x45 → 0x48 → 0x49 (the view: `BtlvUi`, `BattleViewSystem` beside it in 0x45, the gauges, 8 MB),
  0x4a (the UI frames' draws, 5 MB) and a heap a model (0x4b–0x4e, the trainers 0x4f–0x52, the
  `PokeModelAllocator`s 0x5b–0x5e at 3 MB each). All told, 87 heads and 12,463 small blocks, and
  the walk over them all three times took nothing down. Heap 0x44 was the field's area files'
  heap in spike 116: ids are reused as heaps come and go.
- **The command menu is `Btn_Fight`, `Btn_Pokemon_00`, `Btn_Bag`, `Btn_Run` and a hidden
  `Btn_Back`**, each a Parts pane of pictures (`BG_Thick`, `BG_Main`), a `Bound_00` touch area and
  an empty `Null_Cur` of 56 by 28, in a layout of 119 panes on the bottom screen. The cursor is a
  cursor pane as every Sun menu's (spike 112), `Cur_Main` in a layout of its own, 64 by 28 with
  `Common_Cursor_L` and `_R` inside it, moved onto the chosen `Null_Cur`: **8 wider than the pane
  it lands on**, where every earlier menu's fitted exactly, so it was fitted to nothing and read
  the words inside it, of which the command menu has none. There is no word in the layout for any
  button; the pane's name is all there is.
- **Built** (`sm.battle`, `common.cursor`): the profile tracks `btl::BtlvGaugeBase` and says each
  side from its panel's own words, the sign from the record's gender, the enemy's as the fill's
  width over 104.8, once the gauge has counted to its target and the reading has held a poll;
  the box's panels are left out of screen text while a gauge shows them. A cursor pane fits a
  pane up to 8 units smaller, nearest in size first, and a button holding no TextBox at all is
  said by its pane's name as ORAS says one by its texture's ("fight", "pokémon", "bag", "run").
  `pokemon.read` takes the game's last species. And `Discovery.learn` reads the code past the
  executable's end again each time rather than on from where its image last ended: replayed from
  the title, the runtime learned the field's classes at 10 s and never the battle's, since the
  battle's module loads over the field's, below the end the field had grown the image to.
- **What it costs.** Nothing beyond the watch: with the class learned, the runtime made 9 or 10
  whole walks over the 171 s replay, the scheduled ones, and the gauges came from the sweep 1.5 s
  after the battle app started. Before the learn fix the reader had asked for a walk a second the
  whole battle long, 75 walks in 31.8 s, so it now asks three times at most for a box with no
  gauge. Per screen poll it reads each gauge's head and its fill's pane; the fill is found once
  in the box's 385 panes and the Pokémon decrypted once a record.
- **Not done:** the fight menu's second cursor line, "40 40", from the command menu's cursor pane
  left shown for a poll over the move list's PP; the enemy's 0%, said by "The opposing Popplio
  fainted!" as in ORAS.
- **Two follow-ups, 2026-09-27, from sun_9.ctm again.** The enemy's side was said as
  "Popplio♂ L5 100%" at 28 s, before "You are challenged by Pokémon Trainer Hau!", and the
  screenshots show no box until Rowlet is out, at 44 s. Through the intro both panels read as
  shown, their words and gauges full and their panes unmoved; what changed as the box came on was
  its name TextBox, whose flags were 0x11 and its place (0, 0) until then, 0x01 and (106, 108)
  after, fading in from alpha 102. A side is now read only once its name has lost that 0x10
  (`sm.battle.UNPLACED`): both sides are said together at 43.6 s. And the in-battle party
  screen's "Rowlet 22２２ l5": the "l" is a TextBox of its own holding a lone "l", in the font the
  level's digits use (not the title menu's), which draws it as the level mark. Sun's system text
  file 21 holds one line, "Lv.", and a lone "l" just left of a number in its own group is now said
  as that line (`screen.LEVEL_GLYPH`): "Rowlet 22２２ Lv.5", and "Lv.5 Empty" on the screen
  the Bag opened, in layout 0x309ac1a4.
- **Two more, 2026-09-27, from issue 4, over sun_9.ctm and sun_10.ctm.** The enemy's percentage
  was the fill's width over 104.8, and read 62% for 12 of 20 HP. The fill is not proportional to
  the HP: all three readings fit **5.8 + 99 × HP / maximum** (104.8 at 20/20, 65.2 at 12/20, and
  Rowlet's 86.8 at 18/22), a sliver of 5.8 left at 0 HP, which is why a fainted Popplio's fill read
  5%. The enemy's share is now the HP its gauge shows (+0x28) over its battle record's maximum
  (+0x0e; the gauge's own +0x30 is 0 until its first count), once the count has settled as before:
  "Popplio♂ L5 60%" for 12 of 20, and sun_10's Pikipek 20% where its fill had read 24%. The fill
  is no longer read. And the party slot's "22２２": the 10-wide `TextBox_01` between the current HP
  and the maximum holds "/" in a small block, but with no tag processor, since it is the layout
  file's own text, which code never writes; screen text had left every such TextBox out, as it
  does the placeholders beside it ("999", "９９９", "ああああああ", "１２３４５６"). The move list's
  PP ("PP35 35") and the HP box have the same "/"; the HP box's "l" beside the level is the same
  again, the box's "Lv.". A designer's TextBox with no letter or digit, touching a written piece of
  its own panel on each side, is now read as a separator (`screen._separators`): "Rowlet 22/２２
  Lv.5", "Tackle PP35/35". Of its panel, not its layout: as the HP box comes on its panes all sit
  at (0, 0), and a double battle's panel's "/" touched the player's numbers and was said alone.

## Spike 138 — why a battle's first HP line came late

2026-09-29 (`spikes/138_battle_hp.py`: `record` replays movies with the runtime reading them and
logs, among what is said, every change in what the battle reader knows of each gauge and each
name plate, every panel's words, every whole sweep and why, and what each sweep found of plates
and gauges). The user heard the HP after "fight", now and then.

- **Since the heaps' watch (ADR-0034, 2026-09-26) a third of battles said their first reading
  1.45 to 1.9 s after "What will Blaze do?"**, after "fight" and often after the move list's first
  move; before it, 0.1 to 0.3 s after. Measured in every replay log kept since 2026-09-17.
- **Why.** A gauge was found as what held its panel's level TextBox (spike 66), so it could be
  looked for only once the panels were on screen: 0.6 s before "What will Blaze do?". The battle
  then asked for a whole sweep, 0.46 s. About 0.2 s after "What will Blaze do?" the panels leave
  what is read on screen for 1.45 s while the command menu comes up, and a reading is said only
  once it has held for a poll. A sweep that ended in time said both sides before the menu; one
  that ended late held them until the panels came back. In movie2 and movie4 the gauges and the
  names were known and settled at 252.39 s and 184.45 s, and the sides said at 254.04 s and
  186.10 s.
- **Two more, from the same logs.** The reader forgot its asks, what it had said and what was
  pending only when no battle was up, which it judged by the name plates; a plate keeps its text
  after its battle, so none of it was forgotten. The next battle's boxes, at the same addresses,
  waited out the last battle's count of asks (140.96 s to 143.49 s), and a side that read as it had
  at the end of the last battle was never said: movie4's "Blaze♂ L10 20/30" as its last battle
  began, the open point of ADR-0015.
- **The gauges' classes are learned from the battle's DLL as the battle starts**
  (`Discovery.learn`, since 2026-09-23): "classes loaded since the executable was read:
  N3btl13BtlvGaugeSelfE, N3btl13BtlvGaugeBaseE", 0.07 s after `btl::BattleMainProc` in movie2,
  movie4 and X's opening. Swept for by class, both gauges were in the new blocks 1.3 s into the
  battle, 8 s before the panels showed. The battle's DLL loaded at the same address for both of
  movie2's battles and all eight of movie4's.
- **Gen 6's class names cannot name them.** The profile's `ClassNames` reads names inside the code
  span measured as pa3ds started, and the battle's DLL loads past it; checked by name, every gauge
  was refused and no side was said at all. A gauge is now checked by the vtable it had when a
  sweep found it, as the map blocks are.
- **Built** (`gen6.battle`): the gauges are swept for by class; a box with no gauge asks for a
  whole sweep three times at most, as Sun's does; and the reader starts afresh when the battle app
  starts (`Gen6Profile.poll_app`). Over movie2, movie4 and X's opening, all eleven battles said
  both sides 0.2 to 0.5 s before "What will Blaze do?". The battle asked for 5 whole sweeps where
  it asked for 9, each of them now as a battle ends, when the gauge is freed a moment before its
  panel leaves the screen: whole sweeps fell from 24 to 23 over movie2 and from 34 to 30 over
  movie4. What else was said differently is what races from run to run.
- **Replayed before and after over all eleven ORAS movies**, pinned and following in both
  (2026-09-29, `hp-before` at `0f10bbe` and `hp-after` at `41f68d6`). Sweeping cost 113.0 s
  before and 114.6 s after. In the six movies with no battle the gauges' classes are never
  found, so `Discovery.learn` reads the code past the executable at every app start and every
  whole sweep: 7 to 11% more pages there (movie11 330,010 to 365,955), under a second a movie.
  What was said differed only in what races: movies 7 and 9 differed from the old code by 90
  and 91 lines, and from a second run of the new code by 101 and 58, the summary's and the
  Box's details among both.
- **After a battle nothing the runtime wants is missing.** Every class gen 6 wants is in the
  executable (`code.bin`, 2026-09-29) but the map block's and the two gauges', so once a
  battle has been seen `learn` stops looking, as it did for the map blocks alone after the
  first field until now. A gauge found by class relies on the battle's DLL coming back where
  it first loaded, which it did in every battle replayed; a gauge found by what held its
  level TextBox did not rely on it.

## Spike 144 — the HP box while a hit lands, and while a Pokémon faints

2026-09-29 (`spikes/144_hits.py`: `record` replays movies and logs every change in what the screen
reader has in each panel of a layout known to be an HP box, and in each gauge's HP shown and HP
counted towards). Orion, playing on the HP keys' branch: the HP is said only once both sides have
had their turn, where Sun says it the moment a hit lands; and shift and R said a Wurmple at 29% on
the victory screen, after it had fainted.

- **The player's panel leaves the screen while the player's Pokémon attacks; the enemy's stays.**
  Over Ember in movie2 the enemy's panel alone was on screen from 62.48 s to 67.42 s, its gauge
  counting 11, 5 and 2 and settled at 64.33 s. The reader took a box's panels only in a poll that
  held the player's panel, so the enemy's 12% waited for the Wurmple's own turn and was said at
  70.31 s, just before "What will Blaze do?".
- **The drop to 0% went unsaid for the same reason, not because the panel left.** The Wurmple's
  gauge read 0 at 77.08 s and its panel stayed until 79.55 s, 2.5 s, with the player's panel away
  throughout. ADR-0015 had put it down to the panel leaving before the reading held. The keys
  say the last line said, so shift and R said 29%.
- **Built** (`gen6.battle`): a box once known is read without the player's panel. Replayed, the
  12% came at 64.67 s, before "It's super effective!", and "Wurmple♀ L3 0%" at 77.41 s, before
  "The wild Wurmple fainted!" at 78.41 s; over movies 2 and 4, eight enemies said at 0% as they
  fainted, and every enemy's line right after the move that hit it.
- **The player's panel can show its level and not its HP**, as it slides in or levels up, and
  read so it is an enemy's panel: read without the player's HP beside it, "Blaze♂ L9 75%" was said
  five times over movie4. A gauge holds its side at +0x14 (spike 66), 0 the player's, 1 the
  enemy's, and a panel is now taken only for the side its gauge says.
- **At a battle's start the enemy came first**, 0.2 s before the player's side, once the box's
  layout was known from an earlier battle: its panel comes on screen first (movie4 at 409 s), and
  heard first in one battle and second in the next it sounded like a race (Orion). An enemy's
  first reading of a battle now waits for the player's side, 2 s at most (`OURS_FIRST`): all
  eight battles of movie4 said the player's side, then the enemy's, before "What will Blaze do?".

## Spike 150 — a double battle: four gauges, two panels, a group each

2026-10-01, on or/movie15_double_battle (Orion's recording of the Twins on Route 104, Blaze and
Beldum against Lotad and Seedot) and or/movie2_route103's wild Wurmple for a single battle
(`spikes/150_double_battle_gauges.py`, which logs every gauge, every level and HP piece on screen,
and every picture of the HP box's layout with its parents and its layout file's textures).

- **Heard in the recording:** "Blaze♂ L13 26/26" and later "Blaze♂ L13 29/36", Beldum never, the
  levels as screen text, "13, 9", the enemies only once their HP fell, and the key for the enemy
  side "no battle" until then.
- **A gauge a Pokémon, one class for the player's.** Four gauges, side by side in the heap: the
  player's two `btl::BtlvGaugeSelf` (vtable 0x007d8324), the enemy's two `btl::BtlvGaugeBase`
  (0x007d82f0), the same two vtables as the single battle's. A whole sweep for the two classes also
  found two words equal to them where no gauge is, whose fields are code; their level never comes
  on screen.
- **+0x14 is the battle position**, not a side: 0 Blaze, 1 Lotad, 2 Beldum, 3 Seedot, so the
  player's are even. The reader took only 0 as the player's, and Beldum's gauge read as an enemy's.
- **A double battle draws a side to a panel.** One layout, as a single battle's; the panel under
  its root holding the player's side holds both of the player's Pokémon, and the enemy's both of
  the enemy's. Read a panel at a time, the first level found took the last HP found: Blaze's 13
  with Beldum's 26/26. The enemy's panel held two bars, and a panel's bar was looked for as its one
  bar, so neither was read until one had gone.
- **Each Pokémon is a group of its own under the panel, and the gauge holds it.** +0x18 -> the
  group: Blaze's level ('13'), HP ('29/ 36') and bar hang from 0x15265ff8, Beldum's from
  0x152685b4, Lotad's from 0x1526ac74. The single battle is the same, a group a panel: +0x18 is
  0x14d4b718, under the player's panel 0x14d4b614. Under each group, in the layout file's words:
  `ally_gauge` or `enemy_gauge` (the frame), `mega_mark`, the bar (`black`), the status icon
  (`sick_icon_01_mahi`, swapped as the condition changes), `name_blank`, and the enemy's
  `ball_get`. The gauge holds most of them: +0x1C `name_blank`, +0x20 the status icon, +0x24
  `mega_mark`, +0x2C the level TextBox, **+0x30 the bar**, and the player's +0x5C its HP TextBox.
- **The battle record holds the HP** as well as the gauge: +0x0E u16 maximum, +0x10 u16 current
  (Blaze 36 and 29, Beldum 26 and 26), +0x18 u8 the level, and +0x19 u8 the Pokémon's place in
  the battle's parties, 0 and 1 the player's, 12 and 13 the Twins'.
- **So the reader starts from the gauges** (2026-10-01): each gauge whose level TextBox is on screen
  is one Pokémon, whose HP and status icon are what hangs from its group and whose share is its
  bar's width; the player's are the `BtlvGaugeSelf`s; and every layout a gauge's level is in is the
  HP box's, never screen text. Nothing in it counts the Pokémon or knows how a panel is laid out.
- **A horde reads the same** (or/movie16_hoard_battle, Orion, 2026-10-01): five Zigzagoon and
  Blaze said as the battle opened, each enemy's fall to 0% as it fainted. The enemy side is said
  in battle position order, which in a horde runs round a half circle from the bottom right:
  bottom right, top right, top middle, top left, bottom left (Orion found it fainting the
  bottom left one and hearing the fifth go to 0%; the keys file says so above the key).
  Not yet seen: a triple, a rotation, or a partner's Pokémon beside the player's.
