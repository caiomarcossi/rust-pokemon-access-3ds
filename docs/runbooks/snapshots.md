# Snapshots and diffs

A **snapshot** is a capture of RAM regions at one moment, replayed through a reader in tests with
the spoken output asserted. Most findings come from diffing two of them.

## Capturing

**Capture with emulation paused.** A sweep takes seconds and the game runs at 60 Hz, so an unpaused
capture smears hundreds of frames together and buries any small signal: two captures of one
unchanging state came back 114,000 bytes apart. Paused (F4), repeat captures are byte-identical.
Always take two captures of a state and check they match before you trust a diff.

Pausing needs the user at the keyboard: the rig cannot press F4 ([rig.md](rig.md)) and
`--load-state` leaves the game running. A snapshot that becomes test evidence is taken paused. An
unpaused rig capture is for exploration, and the two-capture check tells you which bytes of it can
be trusted.

**Re-measure the heap during a run.** The heap grows after boot and again when a battle starts. A
span measured once at the start silently misses what is built later: three spikes and the runtime
each reported text "not there" that was sitting just past the span they had measured.
`snapshot.regions()` measures it.

## Two things that fool a diff

- A block around `0x16170000`-`0x1618d000` rewrites completely every frame.
- Layout structures are double-buffered, so a field can alternate with capture parity rather than
  with game state. Check it against a third state before believing it.

## Name a class before interpreting it

The executable ships RTTI: a vtable's word at -4 is its typeinfo, and the typeinfo's word at +4 is
the mangled class name (`src/pa3ds/core/rtti.py`; `class_name` in `spikes/11_rtti.py`). Freed memory
keeps its vtable, and a destructor leaves the *base* class's vtable behind, so a named object can
still be dead: check it is still linked or still in use.

## Where captures live

Raw snapshots contain the user's save data and stay out of the checkout, under
`%LOCALAPPDATA%\pa3ds\captures` (`snapshot.capture_dir`).
Only scrubbed snapshots, reduced to the byte ranges a reader actually touched, are committed
(to `fixtures/`).

[`spikes/README.md`](../../spikes/README.md) indexes how each finding was measured. Read a spike to
check a claim or repeat a measurement; the code that graduated lives in `src/pa3ds/`.
