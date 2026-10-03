# Architecture Decision Records

Each record answers **one** question: what was decided, why, and what it costs. An ADR is
**binding** — if your work contradicts one, say so rather than quietly doing otherwise:

> _Contradicts ADR-0010 (nothing interrupts by default) — but worth reopening because…_

The **Rejected** and **Open** parts are the load-bearing ones. They exist so an idea is not
re-litigated every month, and so the things that are known to be unfinished stay visible.

These were retro-fitted on 2026-09-21 from `docs/adr/README.md`, which had grown to 2,822 lines with
decisions, findings and plans mixed together. Each keeps the date the decision was actually made;
the evidence behind it is in [findings/](../findings/README.md), and the originals are in git
history.

## Reaching the game

| # | Decision |
|---|---|
| [0001](0001-attach-over-the-rpc-and-the-gdb-stub.md) | Attach over Azahar's RPC, and over its GDB stub for what a scan cannot answer |
| [0002](0002-the-gdb-stub-is-an-instrument.md) | The GDB stub is a reverse-engineering instrument, never a runtime dependency |
| [0003](0003-anchors-not-addresses.md) | Anchors, not addresses |
| [0004](0004-the-poll-loop-reads-in-phases.md) | The poll loop runs at 60 Hz, reads in phases, and never stalls |
| [0034](0034-objects-are-found-as-the-game-allocates-them.md) | Objects are found as the game allocates them |
| [0005](0005-forking-azahar-is-allowed.md) | Forking Azahar is allowed, under two constraints |
| [0035](0035-sun-is-read-only-in-its-small-allocations.md) | Sun is read only in its small allocations, and to the byte |

## The shape of the code

| # | Decision |
|---|---|
| [0006](0006-a-core-and-a-package-a-game.md) | A game-agnostic core, and a package a game |
| [0007](0007-python-and-its-tools.md) | Python 3.13, uv, ruff, pyright, and prismatoid for speech |
| [0008](0008-static-data-from-the-users-own-rom.md) | Static data comes out of the user's own ROM, through an extractor of ours |

## What pa3ds says

| # | Decision |
|---|---|
| [0009](0009-announce-never-act.md) | Announce, never act |
| [0010](0010-nothing-interrupts-by-default.md) | Nothing interrupts by default, and everything the game shows is heard every time |
| [0011](0011-dialogue-is-spoken-a-page-at-a-time.md) | Dialogue is spoken a page at a time, as the page starts to type |
| [0012](0012-a-menu-says-the-item-under-its-cursor.md) | A menu says the item under its cursor, and never lists itself |
| [0013](0013-screen-text.md) | Screen text is heard when it comes on screen and when it changes |
| [0014](0014-name-entry.md) | Name entry reads the key under the cursor |
| [0015](0015-battle-hp-is-hybrid.md) | Battle HP is said as it changes, and on a key |
| [0016](0016-the-hp-box-belongs-to-the-battle-reader.md) | The HP box belongs to the battle reader alone |
| [0036](0036-a-picture-is-named-by-the-image-it-draws.md) | A picture the game fills in as it runs is named by the image it draws |
| [0017](0017-extras-ship-off.md) | Anything the game does not show is an Extra, and Extras ship off |
| [0018](0018-an-unknown-screen-is-scraped-and-snapshotted.md) | An unknown screen is scraped, marked as such, and snapshotted |
| [0019](0019-a-reader-that-throws-is-logged-not-spoken.md) | A reader that throws is logged and snapshotted, not spoken |
| [0020](0020-speech-goes-to-the-players-own-screen-reader.md) | Speech goes to the player's own screen reader, with a dedicated voice beside it |
| [0021](0021-silent-while-the-emulator-is-unfocused.md) | Silent while the emulator is unfocused |
| [0022](0022-the-keys-pa3ds-answers.md) | pa3ds answers its own keys, taken with a hook rather than by rebinding the emulator |
| [0023](0023-names-for-what-the-game-never-names.md) | Names for what the game never names, with the player's winning |
| [0024](0024-interactables-are-cycled-nearest-first.md) | Interactables are cycled nearest-first, by category |
| [0025](0025-the-games-own-audio-is-part-of-the-interface.md) | The game's own audio is part of the interface |
| [0026](0026-config-and-logs.md) | Config, logs, and a diagnostics key |
| [0031](0031-one-file-format-for-settings-and-data.md) | One file format for everything a person reads or writes: TOML |
| [0032](0032-everything-the-player-owns-sits-beside-pa3ds.md) | Everything the player owns sits beside pa3ds, and nothing is installed |
| [0033](0033-how-pa3ds-ships-is-not-decided-yet.md) | How pa3ds ships is not decided yet, and nothing is added for it |

## How the work is done

| # | Decision |
|---|---|
| [0027](0027-movies-not-sessions.md) | Movies, not sessions |
| [0028](0028-emulator-settings-are-frozen.md) | The emulator settings that affect timing are frozen |
| [0029](0029-raw-snapshots-stay-on-c.md) | Raw snapshots stay out of the checkout, and only scrubbed ones are committed |
| [0030](0030-clean-room.md) | Clean-room: their behaviour and formats, never their code |

## The four that are hard constraints

`AGENTS.md` names four of these as settled, to be reopened only by asking first: **announce, never
act** (0009), **no hardcoded game text** (0023, as the user amended it), **anchors, not addresses**
(0003), **clean-room** (0030), and **parity by default** (0017).
