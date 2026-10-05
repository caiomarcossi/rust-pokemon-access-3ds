# 25. The game's own audio is part of the interface

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion

## Decision

Move sound effects are how a player tells one move from another, so ORAS's Battle Effects stay on.
**Nothing — performance tuning included — may propose muting or skipping the game's own sound.**

Terrain follows from it: ORAS already plays footsteps for some terrain and not others, so pa3ds
**fills the gaps** with its own footstep sounds, one per step, rather than replacing what the game
does. No panning, no HRTF, no audio engine.

**pa3ds's own tones, 2026-10-01** (Orion): a short tone beside the game's sound where nothing else
says where a cursor is, the Hoenn map's between places and touch mode's over a screen the player
cannot see (`pa3ds.core.tones`). Never instead of a word, and `[pa3ds.ui] tones` turns them off.

## Consequences

This is a constraint on optimisation as much as on features: a change that makes the emulator
quieter is not a speed-up, it is a regression.
