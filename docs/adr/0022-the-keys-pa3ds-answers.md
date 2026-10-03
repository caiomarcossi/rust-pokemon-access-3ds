# 22. pa3ds answers its own keys, taken with a hook rather than by rebinding the emulator

- **Status:** Accepted
- **Date:** 2026-09-10, built 2026-09-21 and 2026-09-22
- **Deciders:** Orion
- **Built** for the map's list, for the keys file that says what every key does, and for P,
  pathfind, on 2026-09-24. The mnemonics of the original plan still waiting on the features
  behind them are text, position and health.

## Context

The plan was pokemon-access's mnemonics — p pathfind, m map, t text, y position, h health, j/k/l
interactable cycling, shift for variants — which needs Azahar rebound to a Game Boy-like layout,
written by a setup wizard into `qt-config.ini`.

## Decision

**A low-level keyboard hook** (`WH_KEYBOARD_LL`) on a thread of its own takes the keys pa3ds is
bound to **while the emulator's window is in front**, and keeps them from reaching the game. Every
key passes through untouched anywhere else, Azahar's own dialogs included: a window the game is
not shown in, by its title, is not the game's (Orion, 2026-10-01, who could not type I or O into
Azahar's settings).

Built 2026-09-21 (`keys`, `games.common.navigate`): the map's list under **I** and **O**
(categories: exits, signs, people, items and, since 2026-09-22, pokémon, said with their count,
"exits, 3", and skipped when empty), **J** and **L** (items, nearest first, with their place: "door, 1 of 3, 1 north") and
**K** (the item again). The keys and the wording are the user's.

Each press reads the map afresh, so the bearing is from where the player stands now. `--no-keys`
binds nothing, and the rig's replays bind nothing either.

**Direction mode, the same day.** The user asked for a second way of finding things: instead of
walking the list, look one way at a time. **M** toggles between the two modes, said as "direction
mode" and "list mode". In direction mode **O**, **semicolon**, **L** and **K** are laid out as a
compass — north, east, south, west — and each press says the next thing on that side of the
player, nearest first and wrapping: standing south of a door, O says "door, 1 of 2, 1 north".
Pressing another direction starts again from the nearest that way; the same direction steps on
from the thing last said. The **brackets** step the category in both modes (I and O still do in
list mode). What "that side" means is ADR-0024's.

The hook delivers key names and the navigator looks each up by mode, since O is the category in
one mode and north in the other; a key bound in either mode is kept from the emulator in both.
Punctuation keys are named by their US-layout character from their OEM virtual key, which is the
key's place, not its character. Whether a place is said in direction mode as well ("1 of 2") is
the user's to judge by ear.

**P, the way there, 2026-09-24.** The user's key, in both modes: it finds the way on foot to
whatever the list or a direction is on, and guides the player along it (ADR-0024). On this
machine P is Azahar's GPIO 14 button, which ORAS never reads, so taking it costs nothing and is
only logged. A keys file written before P existed has no line for it, and pa3ds says so at
startup rather than writing one in (`keyfile.unreachable`).

**One file for every key, 2026-09-22** (`keyfile`). pa3ds's keys and the emulator's buttons are
two halves of one question, because **a key pa3ds takes is a button the player has lost**. Both
live in `<pa3ds>\user\keys.toml` (ADR-0032), written with the defaults on the first run and never
overwritten afterwards.

**`[pa3ds.navigation]`** is key to action. A key that means the same thing in both navigation
modes is written once there; a key that does not goes in `[pa3ds.navigation.list]` or
`[pa3ds.navigation.direction]`, which win over it. The names are the user's (2026-09-22): two
tables called `list` and `direction` said nothing about being two ways of doing one thing, and
the toggle between them belongs to the pair rather than to each half.

**`[azahar]`** is button to key, patched into `qt-config.ini` at every start, with
`[azahar.circle_pad]` and `[azahar.c_stick]` for the two sticks.

`[azahar]` ships **empty**, so nothing is rebound until the player asks — this machine's config
is hand-tuned, and a file that silently rewrote it would be the opposite of useful. Azahar's own
bindings are written into it as a comment instead, which is the only way to read them without
Azahar's settings dialog. Whatever the section says, pa3ds reads the emulator's real bindings and
**says which of its keys are sitting on a button**. It does not refuse to bind them: a player who
wants K on the C-stick may have it, and being told is what they were missing. The controls ORAS
never uses — the C-stick, the debug and GPIO 14 buttons — are logged but not spoken, since saying
them at every start would be noise.

**This is what the mode toggle cost.** M was chosen for it on 2026-09-21, and M is this machine's
Start button: for a day the player could not open the game's own menu while pa3ds was running,
and nothing said so. The toggle is the **apostrophe** from 2026-09-22 (the user), beside the
semicolon that is already east.

**Modifiers**, so one key can both read a thing and rename it. The hook reads shift, ctrl and alt
with `GetAsyncKeyState` — not `GetKeyState`, which reads the calling thread's own idea of the
keyboard, and the hook's thread has no window to shape one — and names a press
`ctrl+alt+shift+k`, always in that order so one combination has one name. Shift **falls back** to
the bare key, so a player still holding shift from the last thing they typed finds K working;
ctrl and alt never fall back, because ctrl and a letter belongs to whatever is in front.
**No key is pa3ds's while the screen reader's modifier or a Windows key is held** (2026-09-29,
insert and R, NVDA's OCR, heard as R). NVDA's modifier, insert or caps lock, is held by pa3ds's own
hook's word: NVDA swallows it in its hook, so Windows never marks it held, but pa3ds's hook is
installed after NVDA's and called first (`keys.track`).

**A stick is not one key.** It holds four directions, a modifier and a scale in one value,
and the modifier is what ORAS is walked with rather than run: `Analog::GetStatus` multiplies that
stick's own x and y by `modifier_scale` while its own modifier key is down
(`input_common/analog_from_button.cpp`). Both of this machine's sticks have D as their modifier,
which is not a conflict — each scales only itself. Writing a stick means the player can put
walking on w/a/s/d and the modifier somewhere else, which is what asked for this (the user,
2026-09-22).

The value is a **parameter package** (`common/param_package.cpp`): `key:value` pairs separated by
commas, with `$0`, `$1` and `$2` for a colon, a comma and a dollar inside a part, held in a sorted
map so it comes back out alphabetically. Azahar also keeps a per-direction annotation *keyed by
that direction's own binding*, so pa3ds changes the pairs it was asked for, moves the annotation
with the direction it belongs to, and writes everything else back untouched. A stick rebuilt from
scratch would quietly drop whatever a gamepad had left in it.

Two more facts about `qt-config.ini` that the patching turns on, both now in `pa3ds.core.keyfile`:

- **`code:` is a Qt key code, not a Windows virtual-key code.** Azahar stores `event->key()`
  (`citra_qt/configuration/configure_input.cpp`). Letters and digits are the same number in both,
  which is why nothing had noticed; the semicolon is 59 to Qt and 186 to Windows, and an arrow key
  is not in the virtual-key range at all.
- **A value is ignored while its `\default` flag is `true`**, and Azahar re-saves the file at
  startup — the trap `frame_limit` sprang on the rig. Every binding written sets the flag false
  beside it.

Azahar rewrites `qt-config.ini` when it exits (`GMainWindow::closeEvent` calls `config->Save()`),
so a patch made while it runs is lost. Until 2026-09-30 pa3ds patched anyway and said "restart the
game for them to take effect", which was telling the player to close Azahar and so throw the patch
away. Now **nothing is written while Azahar is open**: pa3ds says "close Azahar and start pa3ds
again to apply your keys", and patches at the next start with Azahar closed, saying "keys applied
to Azahar" only when the file really changed (`keyfile.save`).

## Consequences

- **A key's answer interrupts**, and it is the only thing that does by default (ADR-0010): the
  player asked for it now, and the key never reached the game's window for the screen reader to
  cut speech on. The runtime dropped this on the floor until 2026-09-24, and ADR-0010 says how.
- **The player's own file is the wizard.** There is no setup dialog and no live rebinding: Azahar
  reads its config at startup, and rebinding it while it runs would mean driving its UI, which
  pa3ds does not do. A player who wants Z as the A button says so in `keys.toml`, closes Azahar
  and starts pa3ds before it.
- **Only the keyboard.** `engine:keyboard` bindings are read and written; a gamepad is left alone
  and its buttons cost the player no key. Nobody has asked for more.
- **A key pa3ds gains later is not written into a file that exists** (Orion, 2026-09-29, when
  R and shift and R came). Writing it in could take a key the player had bound to something
  else, and telling a key unbound on purpose from one never offered needs the file to remember
  what it was offered. The log says the line to add and its table, and the spoken sentence says
  the log has it. A utility beside pa3ds that walks a player through its settings may take this
  on, with the move of the flags into a `config.toml` ([roadmap](../roadmap.md)).
- Settings otherwise live in flags, not in a file (ADR-0026). Keys are their own file because a
  player edits them by hand and because half of the file is about the emulator, not about pa3ds.
- Answering a key is not acting on the game (ADR-0009).
