# 19. A reader that throws is logged and snapshotted, not spoken

- **Status:** Accepted
- **Date:** 2026-09-10, built 2026-09-23
- **Deciders:** Orion

## Decision

An exception in a reader is logged with a snapshot, **not** read out — except for a single "reader
error" the first time it happens on a screen, so silence is never ambiguous.

## How it is built (2026-09-23)

`app.Faults`, at the poll loop's own level, below the two RPC handlers that were already there: a
failed *read* is a read that missed (`RpcError`, logged once while it repeats), and everything else
a reader raises is a fault.

- **One bug is one fault**, keyed by what was raised and the file and line that raised it. A reader
  failing every poll says "Reader error" once and then nothing, at 60 polls a second.
- **A screen is a quiet spell.** The runtime has no screen of its own to key on — there is no id
  that says "the player has walked somewhere else" — so a fault that has stayed away for a minute
  and comes back is said again. That is the closest cheap reading of "the first time it happens on
  a screen", and it costs at most one "Reader error" a minute per bug.
- **The snapshot is what was read, not the whole heap.** A sweep takes seconds and wants emulation
  paused (`docs/runbooks/snapshots.md`), which is no use inside a live poll. `PageCache.recent()`
  already knows every page each phase read the last time it ran — the readers' own working set —
  so the capture is those pages and nothing else, one pipelined batch. It is already scrubbed in
  the sense that matters, but it can still hold save data, so it goes where raw captures go
  (`%LOCALAPPDATA%\pa3ds\captures`, ADR-0029), named `<stamp>_reader-error`.
- **Ten snapshots a run.** A run left going overnight on a broken screen would otherwise write one
  a minute per fault for as long as it ran. The first ten are the ones worth having.
- **The rest of that poll is lost**, not the loop: the next poll starts clean. Readers keep state
  across polls, so one that failed halfway may be half-updated; nothing cheap can fix that, and a
  poll later is the recovery.

## Consequences

A player mid-battle cannot act on a traceback, but they can act on knowing that this screen is
broken. The log and the snapshot are for the agent.

The failure this was written for was real and uncaught until now: anything but an `RpcError` left
the loop altogether, so one shape a reader did not expect took pa3ds down mid-game with no log line
and no evidence.

ADR-0018's other half — scraping an unclaimed screen and auto-capturing it — is still unbuilt. The
capture path is now shared, so it is the scraping that is left.
