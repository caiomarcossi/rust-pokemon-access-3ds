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

## Consequences

This is a constraint on optimisation as much as on features: a change that makes the emulator
quieter is not a speed-up, it is a regression.
