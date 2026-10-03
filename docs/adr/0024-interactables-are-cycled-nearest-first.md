# 24. Interactables are cycled nearest-first, by category

- **Status:** Accepted
- **Date:** 2026-09-10, built 2026-09-21
- **Deciders:** Orion
- **Built** for the list and its keys, and pathfinding over the loaded chunks since 2026-09-24.
  The light camera is not.

## Decision

Interactables cycle **nearest first** with category filters, announced name first, then direction
and distance — "Pokémon Center, 3 east, 3 north". A **light camera** lets the player explore the
map without moving. Real pathfinding waits for collision data (built on the chunks' blocked
bit, 2026-09-24: below).

## Consequences

Built on the zone's own entity table and the move-model pool
([findings, spikes 75 and 78](../findings/field-and-rom.md)). The chosen item is kept by *what it
is*, not by its place in the list: walking re-sorts the list, and keeping a place meant one key
tracked one person and then, a few steps on, another.

Zone numbers stay in the log until the ROM's place names are wired up (ADR-0008).

**"All" is a category** (the user, 2026-09-21): every kind together, nearest first, and the one
both modes start on. List mode's item keys therefore walk everything before a category is
chosen, where they started on exits.

**Direction mode looks at a half-box** (the user, 2026-09-21; the keys are ADR-0022). A
direction's things are those on that side of the player — north is everything with a smaller z,
so a sign to the north-east is heard under north and under east both, and a thing on the
player's own row is east or west but neither north nor south — within **10 cells each way**, a
box rather than a radius so the diagonal is not cut short. The user asked that the directions
not cycle the whole map, and how much of it a screen shows: from a rig screenshot at the Route
101 crossing the player is about 23 of the top screen's 400 pixels wide and a cell about 30, so
a screen shows about 13 cells across and 9 deep, more ahead than behind since the camera looks
down from behind, and the count shifts with the zoom the game picks a map. So "one screen" is a
rule of thumb, and 10 is a little over a screen sideways and two ahead (`navigate.RANGE`).
Beyond it a direction says "nothing north" just as an empty one does, or "no signs north" under
a category; the far things are still there under list mode, which keeps the whole map.

**A hidden Pokémon is a category of its own** (the user, 2026-09-22): the DexNav's find, the
shaking tail in the tall grass, is listed as "hidden Pokémon" under **pokémon**, after items,
with its bearing like anything else. It is read from the game's one record for it, not from the
model pool ([findings, spike 80](../findings/field-and-rom.md)), and is read every poll rather
than only after a sweep, since it comes and goes while the player stands still. A battle and a
zone change both leave the Pokémon in the record; one byte of it says whether the tail is up, and
the matrix's zone for its cell is checked besides. The species is
in the log and not said: the game shows a tail, and the DexNav names the Pokémon only once the
species is registered, so a name aloud would be an Extra (ADR-0017) and needs the ROM's text
besides (ADR-0008).

**An edge is the gap, not the border** (the user, 2026-09-22). Littleroot's north side is 40
cells wide and the way onto Route 101 is three of them, and an edge listed as the whole border
sent the player at the trees. The chunks the game keeps loaded around the player say which
cells can be walked ([findings, spike 82](../findings/field-and-rom.md)), so an edge is now each
run of border cells open on both sides, one exit a gap, at its nearest cell like anything else.
Only the four chunks around the player are loaded, so a border is often known on one side
only, which is used; a border with no chunk on either side is listed whole, as before, rather
than left out. This is the first use of collision and not pathfinding: a way across is a cell
the player can stand on either side of, and nothing here says the player can reach it.

**A thing used by facing it is listed even with no record behind it** (the user, 2026-09-23,
finding the PC missing). A Pokémon Center's PC, and the trash cans and shelves like it, are
cells of the map whose kind the game runs a script for on A, not entries in the zone's table
([findings, spike 83](../findings/field-and-rom.md)). They are read from the loaded chunks, like
an edge's gap, and listed under **signs** with the furniture, since both are read by facing
them; a run of cells of one kind is one thing. The PC is called "PC", a word of pa3ds's own like
"sign" and "door", since the game never labels it. The rest are called "object", as a prop is,
until each has been seen: which kind is the trash can is in the scripts, which pa3ds does not
read, and a wrong word would send the player to the wrong thing.

**P finds the way there, and guides the player along it** (the user, 2026-09-24). In either
mode, P finds the way on foot to whatever the keys are on and says it as its legs, "5 east, 2
north". Then, as the player walks, the next leg is said the moment they reach each turn: at the
end of the 5 east, "2 north". A thing solid all over — a door, a sign, a person, a PC — is walked
up to, to an open cell beside it (the user's words: "an open space near it which is 1 tile
away"). Arriving says the thing as the keys would, "door, 1 north", and ends the guiding. The
search is A* in the core (`pa3ds.core.pathfind`, ADR-0006), over the cells the game's own collision
leaves open (`field.Ground`, [findings, spike 92](../findings/field-and-rom.md)). What was
decided, and what each costs:

- **A turn costs as much as a step.** A player who cannot see the map counts both, so a way two
  steps longer that saves three turns is the one taken. Arriving facing the thing only settles a
  tie, so the way to a door ends walking into it where that costs nothing.
- **Only the loaded chunks are read, and the rest is taken to be open.** A way to something
  beyond them is a guess: it can run through a building no chunk has shown yet. The path left to
  walk is checked at every step against the chunks loaded by then, and found again when it hits
  a wall. The ROM holds every chunk (`a/0/3/9`, spike 83), which is how to stop guessing.
- **A ledge is jumped down, and never climbed** (the user, 2026-09-27, issue 3; until then a
  ledge was walked round like a wall). Its kind says which way it drops
  ([findings, spike 126](../findings/field-and-rom.md)): a path crosses it that way only, from
  the cell before it to the cell after, at two steps' cost, and the ledge's cell is counted in
  the leg like any other, so "43 south" is 43 cells even though the jumps make it 40 presses.
  Tall grass is walked through, wild encounters and all.
- **A sign is reached at whichever open side is nearest.** Whether the game lets one be read
  from behind is not known: Littleroot's sign at (100, 183) was reached from the north.
- **A player who leaves the way is given a new one from where they stand**, said when it turns
  them. One that starts the way they are already walking is taken without a word, because a
  player who took their own way round, as short as ours, heard the whole way again at every step
  in the rig. A player standing on a warp that is not the goal is left to it, since the game
  puts them in a door's cell before the warp.
- **A warp ends the guiding without a word**, known by the player's model and the zone it was
  built in, which every warp makes anew and an edge does not (spike 78). A field rebuilt around
  a player standing still — after a battle, say — changes the model without moving them and does
  not end it.
- **Guiding is polled every tick while it lasts**, not with the rest of the field five times a
  second: a turn said a fifth of a second late is a step already taken past it at a run. It
  reads nothing when nothing is being guided. Its speech has a category of its own, `guidance`,
  which queues like everything but a key's answer (ADR-0010).
- **Nothing stops the guiding but arriving, a warp, the thing going, no way on from where the
  player has got to, or P again for something else.** A player who wanders off on purpose is
  given the way back as they go.

**The PC has the game's own word after all** (the user, 2026-09-24, asking whether the ROM names
a use tile): nothing labels it where it stands, but the PC's own lines — booting it, and the
three PCs it offers — all name it, and the word they share is "PC" in six languages and
"パソコン" in Japanese ([findings, spike 93](../findings/field-and-rom.md)). That word is used
where the ROM can be read, and "PC" is pa3ds's only without it. The other use tiles have no word
in the game's text, only what each says, and stay objects.

**An exit into another place says it** (the user, 2026-09-24: nothing said the gap north of
Littleroot leads onto Route 101). "edge to Route 101, 2 of 3, 5 north": the place of the zone
it leads to, in the game's own words, when that is not the place the player is in. A place and
not a zone, as for arriving (spike 87), so the doors of a town's houses, their way out and the
stairs between their floors say nothing more. A name the player gives an exit comes before the
place rather than instead of it (ADR-0023).

**P goes as near as it can, and an edge inside one place says it** (the user, 2026-10-03, at
the foot of Route 119; [findings, spike 159](../findings/field-and-rom.md)). P there said "no
path to edge" and nothing more. Where nothing reaches the thing now, the way goes to the cell
nearest it and is said with ", then can't continue", and arriving says "can't continue" before
the thing and its bearing; "no path" is left for nowhere nearer than where the player stands.
The way on Route 119 was there all along, over a bridge the ground had cut by walking the path
below it, and P now tries each floor's height before giving up. The edge was said bare because
both of the route's zones are Route 119: an edge onto another zone of the player's own matrix
now says the place as every other exit does, though it is the player's own. A house's door, the
way out of it and the stairs between its floors still say nothing more.

The same day, in Fortree City ([findings, spike 160](../findings/field-and-rom.md)), the way as
near as the player could get as the crow flies ended behind a wall from the Gym, where the one
way in had the invisible Kecleon standing in it. Where somebody stands in the way, P goes up to
them and says who: "then can't continue, person in the way".
