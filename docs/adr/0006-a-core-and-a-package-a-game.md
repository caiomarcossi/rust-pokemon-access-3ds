# 6. A game-agnostic core, and a package a game

- **Status:** Accepted
- **Date:** 2026-09-11, restructured 2026-09-21
- **Deciders:** Orion

## Context

pa3ds is written for ORAS, but the same reading worked when the user tried Pokémon X, and a second
game should not mean a second project. Two accessibility layers solved this before: Pokemon Access
and the Black and White layer both have a core, a directory a game, and a directory a language
inside it — and both have one enormous file at the middle (`pokemon.lua`, `bw.lua`, some 23,000
lines apiece).

## Decision

A **game-agnostic core** — transport, poll and diff, speech, hotkeys, camera, pathfinding — plus
**one package a game** holding everything Pokémon-specific.

In the code the core is `pa3ds.core` — `rpc`, `memory`, `snapshot`, `binary`, `rtti`,
`discovery` (find objects by class), `heaps`, `speech`, `keys`, `names` and the rest — with `app`
(the schedule) beside it, since it picks a game, and `gdb` in `pa3ds.debug`, since nothing at
runtime uses it (ADR-0002). The games live under `games/`, and the gen 6 profile is
`games.gen6.profile`, which says what to find, builds the readers and polls them.

**Each of the game's structures has one module** — `games.common.lyt`, `common.msgwin`,
`common.strbuf`, `gen6.talk`, `gen6.mmodel` — and a reader takes offsets from them rather than
defining its own. A new reader is a
module of its own, the classes it needs added to the profile's, and a line in its `track` and poll.

A game package also holds `data/<language>/`, for the words the game itself does not say
(ADR-0023).

## Consequences

- The seam exists from the start; the second game does not, and will not until another forces the
  design.
- **The second game, 2026-09-24: X and Y, as a package of numbers.** X's field said "no map"
  because its zone files have a section fewer than ORAS's, and measured whole it differs from ORAS
  only in numbers: section counts, where the ROM keeps each file, and one move model offset (spike
  100). So `games.xy` holds those numbers in a `Game` and runs ORAS's profile and readers, and the
  readers take either game's section counts rather than one each; a package of its own readers
  is for a game that reads differently, which X does not (the user, 2026-09-24: an X package only
  where sharing would be hard to maintain).
- What is deliberately **not** copied from the layers that came before: the single huge module.
  pa3ds's largest is 621 lines, and a reader is a module.
- The 2026-09-21 move of `oras` under `games/` cost one import rewrite across src, tests and
  spikes, and made room for the data folder the naming work needed.
- **Packages by what shares them, 2026-09-28** (Orion; the plan is in git history,
  `docs/plans/code-reorganization.md` at `d9f7a11`). With Sun on main, `games/oras` was three things
  at once: what every game is read the same way in (Sun used 14 of its modules), the gen 6 engine's
  readers, and what one game has alone. It is now `games.common`, each module given its game's
  **shapes** where the engines differ; `games.gen6`, the profile and readers of X, Y and ORAS; and a
  package a game for what it alone has — `oras` and `xy` their numbers and their `Game`, ORAS's
  DexNav among them, and `sm` a profile and readers of its own. Imports go one way: the core, then
  `common`, then `gen6` and `sm`, then `oras` and `xy`, and `catalog` and `app` above them all. It
  is the shape Pokemon Access has, a `game/common/` beside a folder a game. X's Lumiose
  (`gen6.rail`, `gen6.camera`) stays in gen 6 for now: the field reader walks the rails, nothing can
  test taking them out until X's movies can be pinned, and ORAS has none to share them with: every
  sign and warp in its 536 zone files is placed on the grid, where X places 382 on rails in 11
  zones, Lumiose's seven and one each in Coumarine City, Anistar City, Route 7 and Parfum Palace
  (the ROMs' zone files read on 2026-09-28).
- **Each engine puts its own game together** (Orion, 2026-09-28). The runtime asks the game's
  engine for a `Session` (`games.common.session`: the profile, the classes to sweep for, and how
  the heaps are watched and read), built by `gen6.session.start` or `sm.session.start`, which
  `catalog.start` picks; `app` names no game's module. There is one `start` an engine and not
  one a game: what one game needs of its own is said in its `Game`, as ORAS's DexNav is.
