# 8. Static data comes out of the user's own ROM, through an extractor of ours

- **Status:** Accepted
- **Date:** 2026-09-11, built 2026-09-21, in the runtime 2026-09-23
- **Deciders:** Orion
- **Built:** `pa3ds.core.rom` reads the 3DS side and finds the ROM; `games.gen6.archives` reads the
  game's own text. The runtime uses three things from it: **place names**, the **rival's name**
  and **door names**. Species categories and trainer names in battle are not wired up.
  **Since** (2026-09-28): the kinds of trainer people are drawn as, the PC's word, the plates
  beside doors, Lumiose's streets and their rails (`games.gen6.archives`), and Sun's words, zones
  and walls (`games.sm`), all through the same extractor.

## Context

Some of what a player needs to hear is not in RAM at the moment it is wanted — species categories,
place names, trainer names. Other tools shell out to pk3DS, which is a GUI, possibly inaccessible,
and a dependency users would have to install.

## Decision

A **headless Python extractor** reads the user's own ROM: NCSD, NCCH, RomFS, GARC and the Gen 6
text key stream, about 200 lines and no other tool
([findings, spike 74](../findings/field-and-rom.md)).

It is written language-agnostic — the ROM carries Japanese twice, English, French, Italian, German,
Spanish and Korean — and `Settings.language` picks the archive.

**Finding the ROM, 2026-09-23: from Azahar, by title id.** The RPC says which title is running;
Azahar's own `qt-config.ini` lists the ROMs it has opened (`Paths\recentFiles`, newest first) and
the folder it opens them from (`Paths\romsPath`); and every NCCH carries its program id. The first
decrypted image whose id matches the running game's is the one being played. It is found and
validated the way an anchor is (ADR-0003), not written down, and it asks nothing of the player.
`--rom` names one outright and is still checked against the running title.

**Without a readable ROM, pa3ds carries on** with what it said before: AreaNav's words say a
place, a moment later, and the shipped table says "Rival". That is logged, not spoken — it would
be heard at every start by a player who can do nothing about it mid-game.

## Consequences

- The ROM on this machine is a decrypted NCSD, so no keys are needed and none will ever ship. An
  encrypted dump is refused with a message saying so. **The Alpha Sapphire image on this machine
  was encrypted** (2026-09-23), so an Alpha Sapphire game got no names from its ROM; Orion and
  Khai both have a decrypted one now (2026-09-28).
- **Zone names come from here**, not from a table of ours: file 90 of `a/0/7/3` is 356 place names,
  and the zone table's row +0x1c is the index into it — the **low ten bits** of the u16 there,
  measured in [spike 85](../findings/field-and-rom.md) against every zone pa3ds already knows by
  number (this ADR said +0x1d, which was a guess and is wrong). The bits above are flags, one of
  which marks the zone that carries the place's name. **Built 2026-09-23:** the place is said
  when it changes in RAM, 0.6 to 0.8 s before AreaNav's words (spike 87), which are then left
  unsaid; a town and its buildings share a place, so walking out of a house says nothing, as the
  game has it.
- **So is the rival's name**: trainer names 1 to 3 are the boy and 4 to 6 the girl in all seven
  languages (spike 85), and the player's own move model says which they are (spike 87). Built
  2026-09-23; ADR-0023 has the rest.
- **Replayed before and after, 2026-09-23** (opening.ctm and wally.ctm, with a second run of the old
  code as the control; no out-of-sync lines). wally said the same nine place names at the same
  moments, six of them now from RAM 0.1 to 2.3 s sooner and the other three AreaNav coming back on
  screen after a battle or a menu, which the field rightly leaves alone. opening said all sixteen
  of the rival's lines as May instead of Rival. Nothing else changed beyond what the control run
  changes on its own. The moving truck's AreaNav shows "???", and pa3ds said it before and says it
  now: it is what the game shows.
- **Every overworld sprite's id** is file 94, the 544 ids a move model's +0x10 indexes. The list is
  the same in every archive — the game's own ids, not words for a player — so it names things for
  *pa3ds*, not for speech.
- **The AreaNav's facilities come from here too**: `a/0/6/6` file 0 is a row a town, naming what is
  in it ("Pokémon Center", "Petalburg City Gym") in the player's language, and every zone's data
  file is in `a/0/1/3`, so every door and the zone behind it can be listed without playing to it.
  Pairing the two names 38 of 143 doors by derivation alone ([spike
  86](../findings/field-and-rom.md)). **Built 2026-09-23**, on the user's word: a named door is
  said by its building in the navigation list, in place of "door", as a rename is; the player's
  own name wins, and clearing it gives the door back to the game's. The rest stay "door". The
  derivation runs at every start and takes about 0.1 s. Replayed over wally.ctm and opening.ctm,
  the warps read live from RAM got the names the ROM's pairing gives — Petalburg's doors to
  `0x103`, `0x104` and `0x1af` as the Pokémon Center, the Poké Mart and Petalburg City Gym, and
  Littleroot's to `0xe3` as the Pokémon Lab — and no house door got one; nothing but the log's
  listing lines changed beyond the control's noise.
- **X and Y, 2026-09-24**: the same extractor reads them under their own numbers (a `RomLayout`,
  `games.xy`): place names, trainer classes for the people and the PC's word, in every language X
  carries. Its trainer list does not open with the protagonists, so its text is checked by its
  place names fitting the zone table ([spike 100](../findings/field-and-rom.md)). X has no
  AreaNav, but its Town Map lists what each town has the same way, and names its doors by the
  same derivation, the Pokémon Center in thirteen towns (2026-09-25,
  [spike 101](../findings/field-and-rom.md)).
- Map and collision extraction is deferred; map state comes from RAM first
  ([findings, spike 78](../findings/field-and-rom.md)).

**Rejected: shelling out to pk3DS.**
