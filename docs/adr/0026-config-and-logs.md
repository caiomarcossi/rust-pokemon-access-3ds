# 26. Config, logs, and a diagnostics key

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Partly built.** A rotating log exists at `<pa3ds>/user/logs`, ten runs deep (in a folder of
  its own since 2026-09-24), and the keys have a file of their own in `user` (`keys.toml`,
  ADR-0022, 2026-09-22). `config.toml` since 2026-09-30 (`pa3ds.core.config`). The diagnostics
  key does not exist yet.

## Decision

TOML at `<pa3ds>/user/config.toml` — TOML for everything a person reads or writes (ADR-0031), beside pa3ds itself rather than in `%APPDATA%` (ADR-0032) — a rotating log beside it, and a **diagnostics key** that
speaks RPC latency, poll rate, anchor status and the detected ROM version.

**Keys are not in it.** `keys.toml` came first and stayed separate (ADR-0022): a player edits it
by hand, half of it is about the emulator rather than about pa3ds, and most of it is the comment
listing Azahar's own bindings — all of which would make a general settings file worse. When
`config.toml` arrives it holds the settings that are flags today, and the two files stay two.

**Built 2026-09-30**, written with the defaults and a comment each on the first run, and never
overwritten, as the keys file is. A setting of the wrong kind or an unknown name is logged and
left at its default. Its tables, which Orion laid out the same day after hearing the first, flat
file:

- `[pa3ds]`: `keys`, `poll_rate`.
- `[pa3ds.ui]`, menus, lists and screens: `pauses` (a colon after a label, a comma elsewhere),
  `interrupt`, `unfocused`.
- `[pa3ds.navigation]`, the map's list and the way to a thing: `positions` ("door, 1 of 3"),
  `position_first` (false says "door, 1 north, 1 of 3", Orion's own order) and `turn_by_turn`
  (false says the way once, and P again says it from where the player stands).

**Settings are not flags** (Orion, 2026-09-30): with this many, a flag beside each is too much.
Only what a developer passes for one run stays a flag — `--silent`, `--seconds`, `--port`, the
log's, `--no-keys` — and **`--rom`**, which is not in the file: a ROM belongs to one game, and a
file naming Omega Ruby's would say nothing while X was running, or worse, look as if it applied.
pa3ds finds the ROM among the ones Azahar has opened anyway.

**A setting works the same in every game, or it is not a setting** (Orion, 2026-09-30). Each one
changes pa3ds at one place, whatever the game and the screen: how pieces of text are joined, how
speech is queued, how the navigator words what it lists. None asks a game, a menu or a layout
anything the readers do not already know. So `[pa3ds.ui] count` ("YES, 1 of 2" as a list
opened, the old `--positions`) and `positions` (every item's place) were taken out the day they
were built. They reached only list windows and choice boxes, which know their length; a cursor
reads the item it is on and nothing about the menu around it, so saying "3 of 6" or "row 2
column 1" there wanted each menu's length and shape — X's Pokédex and menus laid out otherwise
than ORAS's and Sun's, the X menu filling as the story gives its items — in every game. Nobody
playing wanted them yet. A list's place is still logged; either comes back if a player asks, from
a reader that knows the whole menu, not from a table of menus.

**`[azahar]` is Azahar's own settings**, patched into `qt-config.ini` as the keys file's buttons
are (Orion, 2026-09-30): what a new player's Azahar needs and does not ship with. The RPC server,
which Azahar ships off and without which pa3ds hears nothing; `log_filter` keeping the RPC server
out of the log, which it fills at a megabyte a second; and what made the game smooth on this
machine — Vulkan (`graphics_api=2`: on OpenGL Azahar's window answers the screen reader in 6.6 ms
rather than 125 us, spike 53), asynchronous shaders, the fast
shader multiply and no vsync. **Nothing that changes emulated timing is here** (ADR-0028):
`cpu_clock_percentage`, `is_new_3ds`, `lle_applets` and the LLE services would desync every
movie on a machine that had them otherwise. A line the player takes out leaves Azahar's value
alone, and any other setting of Azahar's may be added by its name in `qt-config.ini`.

Azahar saves its settings over the file when it closes, so nothing is written while it is open
(ADR-0022): the player is told to close it and start pa3ds again. A start that changes nothing
writes nothing and says nothing, which on Orion's machine, tuned by hand before any of this, is
every start.

## Consequences

The diagnostics key matters more here than in a sighted tool: **there is no console to glance at**,
and the plan is eventually to hide it altogether. Anything a developer would read off a terminal
has to be speakable.
