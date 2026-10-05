# 9. Announce, never act

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Amended:** 2026-10-01, the mouse's exception made a tap (plans/pokenav-and-touch.md)

## Decision

pa3ds **never presses buttons and never writes game memory**. The player walks themselves; pa3ds
tells them what is there.

The single approved exception is **a tap on a named touch target**, so touch-only features become
reachable at all: the PokéNav Plus, Amie, Super Training.

**Amended 2026-10-01 (Orion).** The exception was "moving the OS mouse" to the target. pa3ds taps
instead, as a cemuhook server Azahar reads a second touch device from (`pa3ds.core.touch`), in the
touch screen's own pixels: a pointer would have to land on a spot of a window whose size, layout
and full screen are the player's, and would take a pointer they may be using. It is the same act
the mouse was: pa3ds points at the target it has just named, and the player says when. So:

- **pa3ds taps only on the player's key**, in touch mode (ADR-0022), and only the target it has
  just said: with nothing chosen, or the chosen one gone from the screen, the key names one and
  touches nothing (`games.common.touch`).
- **A tap is a press and a release**, nothing held between keys; drags and holds, which Amie and
  Super Training want, wait until a reader has measured what each needs.
- Nothing else pa3ds does sends the game any input, and it still never writes the game's memory.

## Consequences

- Every feature is a reading problem, which is why the findings log is the shape it is.
- Answering a key (ADR-0022) is not acting: the key drives pa3ds's own speech, never the game,
  but for touch mode's tap.
- It also keeps pa3ds honest about what the player has actually done, which matters for a game
  with a save file.
