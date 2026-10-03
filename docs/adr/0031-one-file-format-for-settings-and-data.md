# 31. One file format for everything a person reads or writes: TOML

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Orion
- **Built** for the keys file, the shipped tables and the player's names. There is nothing else
  yet that a person opens.

## Context

Three kinds of file grew up separately. The shipped name tables were TOML because they are edited
by hand (ADR-0006). `keys.toml` was TOML because most of it is comments explaining the emulator's
own bindings (ADR-0022). The player's own names were JSON, for no better reason than that Python
writes JSON out of the box and TOML only in.

The user, 2026-09-22: **one format for configs and data, whichever it is.** Logs are excluded —
they are for development and are read by whoever is debugging, not by a player.

## Decision

**TOML**, for every file a person reads or writes. The player's names move from
`names/<rom>.json` to `names/<language>/<rom>.toml`.

The deciding argument is **comments**. JSON has none, and every one of these files needs to
explain itself to somebody who did not write it: which game a names file is for, what a stick's
modifier does, which buttons exist. `sango-1` is what the console calls the ROM and says nothing
to somebody who has just been sent a names file, so the file opens by saying it is Omega Ruby, in
English. A format that cannot carry that sentence is the wrong format here.

What TOML costs is a writer: `tomllib` reads and the standard library does not write. `pa3ds.core.toml`
is that writer — a few lines, because these files are tables of strings, and hand-writing it is
also what lets a file carry the comments that chose the format. It is not a general TOML
serialiser and does not try to be. **Escaping is the one thing it must not get wrong**, since a
player may call something `the "old" man \\ upstairs`, and a name written unescaped would take the
rest of the file down with the line it broke.

**Per language, as the shipped tables always were.** A name is a word in a language: English names
for a ROM played in French belong to neither.

**The game's name comes from the emulator.** Azahar titles its window `Azahar <build> | <game>`,
where the game came from `loader->ReadTitle()` — the ROM's own SMDH (`citra_qt.cpp`,
`UpdateWindowTitle`). `pa3ds.core.window` reads it, so **pa3ds holds no table of game names at all**
and the game goes on naming itself, which is the rule the project is built on. A shipped table was
tried first and taken out again (the user, 2026-09-22): it was the one thing here that could be
called hardcoded game text, and it turned out not to be needed. Pokemon Access does keep such a
table, in code (`lua/gb.lua`, `codemap`), which is what made it worth asking whether we had to.

A title that cannot be found — the game is closed, or Azahar has changed its window — is "", and
the ROM names itself: `sango-1`. Nothing is ever spoken from this and nothing depends on it.

## Consequences

- **Nothing migrates.** Renaming and the keys file are both new and unreleased, so nobody has a
  file in an older shape; the code that read one was written and taken out again the same day (the
  user, 2026-09-22). A future move that does reach somebody is a decision of its own.
- `pa3ds.core.toml` holds the table and string readers that `names` and `keyfile` had each grown their
  own copy of. That duplication was the beginning of the five-modules problem AGENTS.md warns
  about.
- A settings file, when it arrives (ADR-0026), is TOML too, and so is anything an Extra toggle
  ends up in (ADR-0017).
