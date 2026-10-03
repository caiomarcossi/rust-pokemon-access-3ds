# 21. Silent while the emulator is unfocused

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Built** 2026-09-29: speech asks `window.in_front`, the test the keyboard hook takes a key
  by (ADR-0022), before each line.

## Decision

pa3ds says nothing while Azahar's window is not in front, so desktop browsing is never
interrupted. Configurable.

Built as (2026-09-29, while Orion slept, for him to hear and overrule):

- **What the game shows is dropped, not kept**, while another window is in front: coming back
  to the game says nothing of what was missed. Saying the last page again on coming back was not
  decided, and would say it twice to a player who only glanced away.
- **pa3ds's own lines and a key's answer are said anywhere** (`speech.HEARD_ANYWHERE`): "pa3ds
  ready" and "Waiting for the game" come while the player may still be in the window they started
  pa3ds from, "Reader error" is about pa3ds itself, and a key is only answered while the game is
  in front anyway.
- **`[pa3ds.ui] unfocused`** (a flag, `--speak-unfocused`, until 2026-09-30) says everything, as before. A replay builds its speech without the
  check: the rig's emulator is never in front, and what a replay logs is what readers report, not
  what reaches the screen reader.

**Azahar's own dialogs are another window** (2026-10-01): "in front" is one of the windows the
game is shown in, its title carrying the game's name, so the settings pa3ds took I and O out of
are typed into as anywhere else, and the game is not said over them.

## Consequences

The same test — is the emulator in front? — is what lets the keyboard hook swallow a key without
ever taking one from another program (ADR-0022).
