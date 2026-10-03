# 36. A picture the game fills in as it runs is named by the image it draws

- **Status:** Accepted
- **Date:** 2026-09-29, built the same day and 2026-09-30 (spike 147)
- **Deciders:** Orion
- **Built** from the plan "Naming a picture by the image it draws" (2026-09-28), deleted as it
  landed; git history has it.

## Context

Some screens swap a picture's texture as they run, so the layout file names a placeholder or
nothing: the Pokédex's OTHER LANGUAGES tabs, the type icons beside a Pokémon and its moves, a
move's category, a Pokémon's status condition (every layout that draws one names paralysis's).
pa3ds said nothing there, or said the placeholder's words.

## Decision

**A picture is named by its image, looked up in an index of the player's own ROM.** A picture's
material says what it draws now: the image's physical address, the size it is stored at and the
GPU's format (`lyt.MATERIAL_*`), which is the whole image, in FCRAM or in video memory. That image
is the ROM file's, byte for byte, so a hash of it finds the file (`games.gen6.textures`).

- **The index is built from the player's ROM, never shipped** (ADR-0008): a copy of the game
  whose pictures differ indexes its own. Built once per ROM, in a process of its own so reading
  the game goes on meanwhile (13.6 s for Omega Ruby), and kept as TOML in
  `user/cache/textures/<rom>.toml` (ADR-0031, ADR-0032). Only named textures, in the archives a
  game lists (`RomLayout.textures`), and every language's copy of each (`garc.garc_variants`).
- **What is said comes from the game's own text**, by a rule from the texture's name: a tab's
  language as the language choice names it ("English", Orion's choice over the tabs' letters),
  a type by the game's number for it ("Fire"), a category as the summary does ("PHYSICAL"), a
  condition as the game's list of them does ("PARALYSIS"; badly poisoned, which has no word of
  its own, as "POISONED").
- **Where it is said:** a window cursor with no words under it says the pictures inside it (the
  tabs); screen text says every named picture in a layout with words on screen, where it is
  drawn ("Male TYPE Fire"), and the battle reader adds a condition to its side ("Torchic ♀ L5
  20/20 PARALYSIS"). Two icons side by side are one, "Bug/Water" (Orion); a move's type is read
  straight after its name, though the battle's move list mirrors its columns and draws it after
  the PP on one side (Orion: the mirroring cannot be heard, only the lopsidedness). A picture
  wholly off its screen is not said.

## Consequences

- The first run on a ROM names no picture for about 14 s. Deleting the cache only has it built
  again.
- Summary move rows gain their types, "Ember Fire PP 25/25", and the move details their
  category: parity, since the game shows both.
- `garc.garc_files` keeps an entry's first copy, the default: anything read from a GARC for its
  words may want the player's language's copy instead.
- **No image larger than 4 KB is read** (2026-09-30). The image is in the GPU's memory, and Azahar
  flushes its GPU's copy of what the RPC reads from the RPC's own thread: reading a 128 KB image
  the GPU was drawing into as a battle began crashed Azahar 5 times in 12 (spike 149, findings in
  `tools-and-rig.md`). Every texture a rule names is 2 or 4 KB, and an image drawn into is a
  screen's size, so a rule naming a larger texture would reopen this, and needs the race closed
  in Azahar first.

## Rejected

- **Indexing the whole ROM** at first run: 554 s, most of it LZ11 (since made 2.6 times faster).
  Nine archives hold only textures without names, which give nothing to say.
- **A texture's own words** for these: the tabs' `eng`, `fra`, and the types' romanised Japanese
  (`honoo`) where the game has words of its own.
- **A data file of pa3ds's own** for language names: the game names them, in every language.
- **Saying the locked language tabs** ("English, 6 locked"), drawn faint and never reached by
  the cursor: left unsaid for now (Orion, 2026-09-29).

## Open

- **Heard only in tests:** two types side by side and a status condition, which no movie reaches.
- The textures of models and backgrounds, most of what stays unmatched, are not `.bclim`s.
- Texture names that are not words (`waza_plate_00`), and pictures with no rule yet.
- X and Y, and Sun, need a census of their own for their `RomLayout.textures`.
