# pa3ds, in Rust

pa3ds speaks 3DS Pokemon games to blind players: it reads Azahar's emulated memory and sends text
to a screen reader. Two people work on it, Orion and Khai, each on a machine of their own, with
their own movies and saves. Orion is blind and cannot check your work by looking at the game, and
you cannot see the screen either: tests, snapshots, tapes and screenshots are the only shared
evidence.

**This repository is a port.** The Python pa3ds (https://github.com/KenzonYeoh/pokemon-access-3ds)
is upstream: features are found and built there first, with its spikes and movie rig, and arrive
here as a port of the commit `port.toml` names. Each Python file has one Rust file at the same
path (`docs/porting.md`), so an upstream change is a change to the files it maps to.

Before the first change in a session, read [CONTEXT.md](CONTEXT.md) in full — the vocabulary,
which code and conversation both use — and the **ADRs** for the area you are working in
([the index](docs/adr/README.md) is grouped by area). An ADR is binding: contradict one only by
saying so. `CONTEXT.md` and everything in `docs/` but `docs/porting.md` are upstream's, copied
byte for byte: change them upstream, never here.

## Hard constraints

Settled. Reopen one only by asking Orion first.

- **Announce, never act.** pa3ds never presses buttons and never writes to game memory. The one
  approved exception, much later, is moving the OS mouse to a named touch target.
- **No hardcoded game text.** Every name, label and message *the game itself shows* comes from
  the ROM or RAM. Hardcoded struct layouts and anchor definitions are fine; hardcoded strings are
  not. What the game **never** names — a speaker, a sign, a door — may carry a name of pa3ds's own
  under three rules (Orion, 2026-09-21, reopening this): it never replaces something the game
  says, it lives in a data file a game and a language rather than in code, and the player's own
  names always win. `pa3ds.core.names`.
- **Anchors, not addresses.** Find structures by scanning and validating at runtime, then cache
  the result against a ROM fingerprint. A bare address is never the way to reach data.
- **Clean-room.** pokemon-access and PKHeX are GPL and Azahar is GPL-2. Read their behaviour and
  their formats; never copy their code.
- **Parity by default.** Anything the game does not show on screen (IVs, exact HP, hidden items)
  lives behind a per-item toggle that ships off.

And one of this repository's own: **the port behaves as upstream does.** Same reads, same clock
reads, same log lines, same speech. A bug found while porting is ported as it is and fixed
upstream first.

## Commands

```
cargo build --release
cargo test --lib
cargo clippy --all-targets
cargo fmt
cargo run --release -- --silent --no-log --seconds 60   # the runtime, printing what it would say
powershell -ExecutionPolicy Bypass -File tools\package.ps1   # the release zip, in target\package
```

`--silent` is mandatory from an agent session: without it pa3ds speaks through the screen reader
of whoever is at the machine, over whatever they are doing. `--no-log` keeps a run from rotating
the live logs in `user\logs\` beside pa3ds.

`pa3ds.exe` needs nothing installed: `prism.dll`, which reaches the screen reader, is copied
beside it by `build.rs` from `vendor/prism/`.

## Keeping up with upstream

```
python tools/sync.py status     # what changed upstream since port.toml's commit, file by file
python tools/sync.py mirror     # copy the docs and data files across
python tools/sync.py layout     # create the Rust file for a new Python file
python tools/sync.py tests      # every Python test has a Rust test of its name
python tools/sync.py mark       # port.toml now matches upstream's HEAD
```

The Python checkout is `PA3DS_PYTHON`, or `../pokemon-access-3ds` beside this one. To sync:

1. `status`, then for each file listed to port by hand, read its upstream diff
   (`git -C ../pokemon-access-3ds diff <port.toml's commit>..HEAD -- <path>`) and make the same
   change to its Rust file, following `docs/porting.md`. Port the change to its tests too.
2. `mirror` for the docs; rewrite this file, `README.md` or `CLAUDE.md` by hand where upstream's
   changed in a way that applies here.
3. `cargo test --lib`, `cargo clippy --all-targets`, `sync.py tests`.
4. With the upstream commit checked out there, record a tape of a movie that reaches the change
   and replay it here, with the tapes already recorded (`tools/tape/record_all.sh <replay>`, then
   `target\release\tape.exe %LOCALAPPDATA%\pa3ds\tapes\*.tape.gz`; `tools/tape/README.md`).
   Every tape must say "identical to the Python runtime's". This is the proof; the tests alone
   are not. Tapes recorded on an older upstream commit are re-referenced first by replaying them
   through the new Python (`tools/tape/py.py replay`), since what the Python says has changed.
5. `mark`, and commit: `feat(port): <what upstream did> (upstream <short sha>)`.

## Commits

Conventional Commits: `type(scope): summary`, with `feat`, `fix`, `docs`, `refactor`, `test`,
`chore`, and a scope naming the area (`battle`, `rig`, `speech`, `port`).

## Runbooks

Upstream's runbooks are in `docs/runbooks/` and apply as written to the game and the emulator;
where they run `uv run` or a spike, run them in the Python checkout. Spikes and the movie rig stay
upstream and are not ported (the user, 2026-10-03): they record how a finding was reached, and
drive the emulator rather than ship.
