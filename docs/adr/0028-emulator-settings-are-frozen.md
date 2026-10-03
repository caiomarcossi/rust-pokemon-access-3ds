# 28. The emulator settings that affect timing are frozen

- **Status:** Accepted
- **Date:** 2026-09-13
- **Deciders:** Orion

## Decision

Movie replay stays deterministic only if the settings that affect emulated timing are identical to
the recording: `cpu_clock_percentage`, `is_new_3ds`, `audio_emulation`, `lle_applets` and the LLE
service toggles. They are pinned before the first movie is recorded, checked at startup, and
recorded alongside the movie.

**File reads are one more.** A movie replays faithfully only if "Force deterministic async
operations" was ticked when it was recorded and is on when it is replayed
([findings, spike 25](../findings/tools-and-rig.md)). movie1-3 predate that and replay with it
off, approximately.

## Consequences

**Performance tuning that touches any of them invalidates every movie taken before it.** That is a
real cost and the reason the list is short and explicit.
