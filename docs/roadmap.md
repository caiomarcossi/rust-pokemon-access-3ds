# Roadmap, and what is still unknown

## The goal

A blind player finishes Pokémon Omega Ruby, start to Champion, with no walkthrough and no sighted
help, using a screen reader. **That is v1.0.** The Delta Episode is v2. Battle Maison and the rest
of the postgame come after.

## Milestones

1. **The Littleroot slice.** Language selection, Birch's intro, gender select, name entry, the
   truck, the house, Littleroot, Route 101, the bag and starter selection, the Poochyena battle,
   the Route 103 rival battle, and the Pokédex handout. Dialogue, the basic menus (Pokémon, bag,
   Pokédex, options, save) and yes/no prompts, plus interactables by direction and distance.
   Nothing hardcoded.
2. Remaining menus: bag, PC boxes, shops, summary screens.
3. Full battle coverage — move selection, switching — plus the light camera.
4. Collision reading and real A* pathfinding.
5. Alpha Sapphire parity. A decrypted dump is available now: Orion and Khai both have one.
6. Touch targets and a spoken menu for DexNav, Super Training and other touch-only features
   ([plan](plans/pokenav-and-touch.md)).
7. A second game profile: done. X and Y are read by gen 6's profile with ORAS (spike 100), and
   Sun and Moon by a profile of their own (spike 111); ORAS is the most complete. Shauna's choice
   of nicknames for the player, not read when pa3ds first ran over X on 2026-09-14, is read now.

The repository is on GitHub.

## Known unknowns

- **Whether two message windows can be live at once.** Which window is on screen is settled enough
  to build on: **recency** says which text just arrived, and **liveness** says a window is still
  assembled when every line object is an `nw::lyt::TextBox` and none uses another window's tag
  processor. A screen with two text areas genuinely up together, such as battle text beside a
  prompt, is untested.
- **Which control codes carry meaning worth speaking.** The encoding is settled and the page
  breaks `0xbe00` and `0xbe01` are identified. `0xff00` turned out to be colour, not an insert,
  and the player's name arrives already substituted. What is left is the long tail of codes the
  opening never used.
- **Whether screen text lands at a reproducible address across runs.** A save state gives the same
  addresses every time, which covers the rig. Whether a fresh boot to the same place agrees is
  untested, and that is the case an anchor cache has to survive.
- **Screen text the model does not yet cover:** the bag's second "L" and "/", which no picture
  shows; whether a layout hidden without being freed clears its work's drawn byte; and whether the
  order panes were built in is reading order, which it is not for the options screen's title.
- **The title screen and the logos**, which are pictures with no words: "PRESS START" is a picture
  textured `push_start.bclim`. Not to be said as its texture (the user, 2026-09-29): see the
  descriptions under ideas.
- **A session where the game seemed stuck after "pa3ds ready"** (the user, 2026-09-29 at 14:15,
  Azahar's `azahar_log.old.txt` of that day): no music, and A did nothing, though Azahar drew 60
  frames a second throughout and loaded the opening's files by 9 s; nothing after. The user
  could not make it happen again, with async shader building on or off. pa3ds read address 0
  every 0.2 s through it, as in every replay: the dialogue reader following a null pointer, and
  its phase fetching page 0 ahead every poll after. No read below `memory.LOWEST` is sent now
  (2026-09-29), so that suspect is gone. There is no pa3ds log of the session: a recurrence with
  its log is what would tell.
- **Whether a player loading a save state while pa3ds reads hangs the load**, as a sweep across a
  load did in the rig.
- **Battle beyond its menus.** The command menu, the move list, the battle bag's pockets, the
  party screen, HP boxes and the level up are read, and switching, items used in battle and
  trainer battles are heard working (movie4 and May's battle in movie2, the user, 2026-09-29).
  Unknown: how to hear a screen's title ("Choose a Pokémon.") when its layout is a menu, and
  double battles, which wait on a recording.
- **Pictures whose texture name is not a word.** Every word pa3ds says for a picture with none
  is in `games.common.pictures` (2026-09-29): a texture's name less "button" and "icon", with the
  spellings a file name cannot carry or the designers got wrong, a Sun button's pane name, and
  the Box's marks as signs. English only, and may want a table per language (Orion). Left: names
  that are not words (`waza_plate_00`), and pictures named by their image
  ([ADR-0036](adr/0036-a-picture-is-named-by-the-image-it-draws.md)) that no rule reads yet.
- **The map collision format, beyond one bit.** A chunk's cell table says where the player
  cannot walk ([findings, spike 82](findings/field-and-rom.md)), which narrows an edge to its
  gap and is what pathfinding walks on (spike 92). What the other bits mean (which way a ledge
  drops, tall grass, water), which side a sign is read from, and how to read a chunk the game
  has not loaded are what keeps a way from being a guess past the chunks the game has loaded.
- **What makes a good validator for each anchor.**
- **Whether Sun's actors keep their keys.** A gen 6 person was known by its move model's slot
  alone, and walking into the next zone filled the slot with somebody else (spike 131). Sun's
  actors are known by their addresses too, and an actor's memory may be another actor's after a
  zone change: not measured.
- **X's movies do not follow their recordings.** Both went their own way on 2026-09-28 (15 and
  34,584 out of sync) and neither is pinned, so no change to X is checked by a replay; moving X's
  rails out of `gen6` waits on it. Part of why is known: what a movie is pinned by covers file
  loads and the accelerometer, and not the gyroscope (Orion, 2026-09-28).
- **Whether a Pokémon hiding elsewhere than the grass is kept in the DexNav's one record.** Only
  tails in the grass have been seen; the record is found through the player's work
  (`field::PlayerWork`), and nothing in the code's class names says how other kinds of hiding are
  kept. A movie with one up would tell.
- **Whether Sun's class names go stale.** Sun names classes over the whole code span and keeps
  every answer for the session, and its battle's module loads over the field's (spike 124): a
  name kept for an address the next module reuses would be wrong. Not seen, not measured.

## Heard missing in play

Screens the player reached that pa3ds does not read yet, left here until what each needs is known
(the user, 2026-09-27, from that day's movies):

- **The PC Box**: its menu for a chosen Pokémon (spike 127), and its slots and marks with whether
  each is set (spike 129), are read. Left: the Pokémon being carried, heard now only in the top
  screen's summary as it is picked up. Of the buttons drawn as pictures alone, the one the
  pointer reaches, the bottom bar's below the grid, is said "change box" (spike 143); the three
  modes above the box and the arrows beside its name were never reached with the D-pad
  (`or/movie12_box_buttons`, the user, 2026-09-29).
- **The Pokédex's More list** (`or/movie7_menus`, spike 128). APPEARANCE/CRY adds only pictures to
  the entry, the model and buttons drawn without words, and what can be read already is
  (`or/movie11_pokedex_and_box`): up and down switch the Pokémon, left and right its form, "Male"
  or "Female", both heard as the entry changes; Y plays the cry (the user). What X, L, R, Start
  and Select do shows only on the model, if anywhere: left for Khai to see. OTHER LANGUAGES draws its tabs (ENG FRE GER ITA JPN KOR SPA)
  as pictures whose textures the game fills in as it runs, from no archive pa3ds knows, and the
  entry's type (FIRE) is drawn from video memory: naming either needs a picture's image matched
  to the ROM's textures. The Habitat page says the Pokémon's places and whether the map's cursor is on one (spike 152);
  X's is not read.
- **X's opening**: the gender choice and the appearance screen are not read (`x/movie1_opening`).
- **X's Pokédex** (`x/movie2_pokedex`): its list's cursor says a row's number alone ("004"),
  where Omega Ruby's says "004 Torchic"; the name comes only with the details beside it.
- **Sun's nickname keyboard** (`sun/movie2_meeting_people`, 576 s, the user, 2026-09-29): naming
  Rowlet, a keyboard opened from the Pokédex's registration, said only the letters typed, never
  the key under the cursor, where naming the player in `sun/movie1_opening` said each key. The
  cursor's layout was not found; which heap it is in is not known.
- **Sun's first battle says its HP box as screen text** (`sun/movie2_meeting_people`, 735.9 s):
  "Popplio dart22/5２２", as the box came on 0.6 s before the sweep found its gauges, 1.5 s after
  the battle started. The reader knows a panel is its own only from a gauge; ORAS keeps a lone
  level beside a bar out of screen text for the same moment (ADR-0016).
- **X's player's house is one zone** (`x/movie1_opening`): the bedroom and the ground floor are
  both zone `0x2`, joined by stairs inside it, so from the bedroom the stairs down are not listed
  while everything downstairs is. Pathfinding is GitHub issue #3.
- **The title menu's Live Competition and Mystery Gift**, low priority, for much later (the
  user, 2026-09-29: no one can get a Wonder Card or join a competition on an emulator now), both
  in `or/movie13_mystery_gift`, which the rig replays. Live Competition's "Receive Digital Player
  ID?" says its Yes and No together as screen text, "Yes No", with no cursor on either. The card
  album's cursor says the card it left's date on an empty slot ("05 11/20/2014", then "05" 0.2 s
  later), and back on card 01 says "01 11/20/2014" without the card's text again: the top
  screen's words did not change, so screen text has nothing new to say.

## Spending less, not yet tried

What finding things costs, after spikes 130 and 133 (154 s over the eleven ORAS movies, whole
sweeps 82 s of it), and what could cut it further, each waiting on a decision or a measurement:

- **The net every 60 s instead of 30.** About half the sweeps the timer makes, 15 of 65 over
  movies 4 and 7, which caught only map blocks, CoreParams and name buffers. Readers would keep
  what has gone twice as long, checking each one's class every poll as they do. ADR-0034 chose
  30 s.
- **A whole sweep reading only what can hold a live object.** A sweep reads about 11,700 pages,
  53% of them all zeros; only a third lie in blocks the watched heaps have in use, but live objects
  lie outside them too (the process manager). Skipping free memory inside the watched heaps, and
  reading everything outside them, could halve the reads, and weakens the net's catching what the
  watch never saw. It needs a spike of its own first.
- **The zone data's holder, checked rather than looked for.** Looking scans every page for every
  zone file, 1 s of movie4's whole sweeps, as the DexNav's record did before 2026-09-28. Whether
  the holder stays where it is when the field is built again is not known, and freed memory can
  still look like it.
- **The net's log names what nobody reads**: the map blocks after every battle, and name buffers
  (CoreParams no longer, since the DexNav stopped sweeping for them). A miss that matters is easy
  to overlook among them.

## Decided but not built

Checked against the code on 2026-09-22, because an ADR that describes intent reads exactly like one
that describes behaviour; on 2026-09-28 two rows were gone, built since: a reader that raises is
caught and logged (ADR-0019), and the runtime reads the player's ROM (ADR-0008). Each of these
carries the detail in its own record:

| | |
|---|---|
| [0017](adr/0017-extras-ship-off.md) | No Extra is read at all yet, so there is nothing to toggle |
| [0018](adr/0018-an-unknown-screen-is-scraped-and-snapshotted.md) | An unclaimed screen is silent; nothing auto-captures |
| [0020](adr/0020-speech-goes-to-the-players-own-screen-reader.md) | No fallback chain, no startup warning, no braille |
| [0026](adr/0026-config-and-logs.md) | No diagnostics key. **A portable Azahar is never patched**: one with a `user` folder beside `azahar.exe` keeps its settings in `user\config\qt-config.ini`, and pa3ds looks only in `%APPDATA%\Azahar` (`keyfile.emulator_config`), so `[azahar]`'s settings and keys never reach it and nothing says so. To be built, not yet (Orion, 2026-10-06), after a friend's Azahar went unconfigured for a cause nobody knows |
| [0024](adr/0024-interactables-are-cycled-nearest-first.md) | No light camera |

[0033](adr/0033-how-pa3ds-ships-is-not-decided-yet.md) is not in that table: it decides that **how
pa3ds ships is not decided**, so there is nothing to build until it is reopened. A player still
needs a developer's tools to run pa3ds at all.

## Ideas without steps yet

- **Descriptions of scenes the game shows without words** (the user, 2026-09-29): the logos and
  the title with "PRESS START"; Birch's opening, between his lines; the Beautifly swarm, Kyogre
  and Groudon waking and what follows the crisis, the Delta Episode's scenes; X and Y's ultimate
  weapon firing; and most of all Sun and Moon, whose logs are silent for minutes through
  cutscenes. **Optional**, a setting a player turns on. A description is pa3ds's own words, in a
  game's data file for its language (ADR-0023), never a texture's name. How it is heard is open:
  the user's idea is recordings, ogg files under a game's `data/sounds/descriptions` (a sounds
  folder because footsteps may follow), against text the screen reader says at the player's own
  rate and voice. **The open question is pairing a scene with its description**, since a scene
  has no name to look up. Signals pa3ds already reads: the app running (`title::TitleProc`), the
  map and zone, and the message said just before or after a scene, by its text file and line in
  the ROM, the same in every language; Birch's own sprites are not named.

- **A utility beside pa3ds that walks a player through its settings** (Orion, 2026-09-29), for a
  player who would rather not edit TOML by hand: the flags moved into the `config.toml` ADR-0026
  decided, and the keys file's new keys offered to a file written before them, which pa3ds does
  not write in on its own (ADR-0022).

## Deferred

- Driving touch targets by calling game functions instead of moving the mouse. Interesting, needs
  code injection, not now.
- Multi-language support beyond English, though the extractor is built not to preclude it.
- Real hardware over NTR or Luma, rather than an emulator.
