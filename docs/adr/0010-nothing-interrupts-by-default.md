# 10. Nothing interrupts by default, and everything the game shows is heard every time

- **Status:** Accepted
- **Date:** 2026-09-13 and 2026-09-14, both after live play
- **Deciders:** Orion

## Context

Two rules were written the obvious way first, and live play reversed both.

**"Identical text is never repeated"** silenced every second conversation with the same person —
but a sighted player sees that text again, so a blind player should hear it again.

**Menu moves interrupting** cost the player the item under the cursor: a screen reader already
cuts speech when a key is pressed in the game's window, so pa3ds cutting itself off on top of that
lost the item to the move description the same keypress brought up.

## Decision

- **Nothing interrupts by default** (the user, 2026-09-14). `interrupt` (`[pa3ds.ui]`, a flag until 2026-09-30) puts menu interruption
  back.
- **Everything the game shows is heard every time it shows it.** Readers report changes, so
  polling never repeats anything by itself, and a second conversation is heard again because it
  happened again.
- All of it is configurable, because it needs retuning during real play.

## Consequences

A key's own answer is the one thing that does interrupt (ADR-0022): the player asked for it now,
and the key never reached the game's window for the screen reader to cut speech on.

**It did not, until 2026-09-24.** The speech policy said so and its tests passed, but the
runtime built its speech with the interrupting categories given as menus under `--interrupt` and
as nothing otherwise, which replaced the key's default rather than adding to it. So from the day
the keys were built, cycling the list read every item out whole behind the last one (the user,
playing Alpha Sapphire, asking for exactly what this ADR already said: pa3ds's own keys
interrupt, Azahar's do not). The decision stands as written; `interrupt` now adds menus to the
key, and a test runs `main` itself rather than the policy alone.

**Amended 2026-10-05 (Khai and Orion): a moved cursor's details are one line, and never
interrupt.** Each panel of the details a move brought up was a line of its own, said as a menu's,
so under `interrupt` the count in the bag cut off the Poké Mart item it belonged to, and the
description the count. The details are now one line that queues: the item is heard at once and
whole, its details after it, and the next item still cuts them off. Holding the item until its
details settled and saying the two as one line (Khai, 2026-10-04, `feat/menu-item-with-details`
at b406a77) fixed the same, and was set aside for the wait it put before every item, the item
being what a player moving a cursor listens for. The default stays off.

**Amended 2026-10-05 (Orion): a warning interrupts too.** On a friend's machine Azahar never got
pa3ds's settings. With Azahar open as pa3ds starts, "close Azahar and start pa3ds again to apply
its settings" was a system line, queued behind the console pa3ds was started from being read
out, where a key or a switch to Azahar throws it away. A **warning**, something the player must
do before pa3ds can work, now interrupts and is said whatever window is in front
(`Category.WARNING`): that line, and "Azahar's settings were not found" for an Azahar never
started, which until then was only logged.
