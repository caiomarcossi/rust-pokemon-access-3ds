# 9. Announce, never act

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion

## Decision

pa3ds **never presses buttons and never writes game memory**. The player walks themselves; pa3ds
tells them what is there.

The single approved exception, late in the roadmap, is **moving the OS mouse** to a named touch
target, so touch-only features become reachable at all.

## Consequences

- Every feature is a reading problem, which is why the findings log is the shape it is.
- Answering a key (ADR-0022) is not acting: the key drives pa3ds's own speech, never the game.
- It also keeps pa3ds honest about what the player has actually done, which matters for a game
  with a save file.
