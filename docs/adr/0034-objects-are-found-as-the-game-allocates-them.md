# 34. Objects are found as the game allocates them

- **Status:** Accepted
- **Date:** 2026-09-26
- **Deciders:** Orion

## Context

The runtime finds what it reads by sweeping the heap for vtables (ADR-0003): at first every
couple of seconds, from spike 99 on when something asked for one. A sweep reads ten megabytes to
find the handful of objects built since the last, and finds each of them late: an object built
between two sweeps waits for the next. Sweeping was a seventh of the runtime's time on main.

Every object a sweep finds was allocated through the game's own heaps, and each keeps a list of
its blocks in use, with a new block put on the list's end, or on its front when taken from the top
of the heap ([findings, spike 110](../findings/memory-and-discovery.md)).

## Decision

- **Watch the heaps.** The game's heap objects are found by class in the first sweep and checked
  by the signature of the SDK head inside them; each links to its siblings and children, so one
  leads to all. Every tenth of a second pa3ds reads each head, and the game's count of blocks in
  use beside it, and walks the blocks new since the last look, at either end of the list
  (`pa3ds.core.heaps`). A block freed and taken again where it was looks exactly as it did, and the
  game does it all the time, so whenever a heap changes, the few blocks at either end of its list
  are looked at too, and handed over where their memory no longer reads as it did.
- **Sweep what is new.** Each new block's memory is swept as it appears and again half a second
  and two seconds on, since a pool is taken whole and built into after: all of it up to 128 KiB,
  the first 8 KiB of a larger one (`games.gen6.gflheap`). Such a sweep is partial, and readers add
  what it finds to what they know rather than replacing it.
- **Watch a layout pool as it is made, and wait for a layout to be built.** Some layouts are
  built into a pool's slots, not allocated, so a pool found in new memory is watched before any
  layout is built into it; a work found before its layout is built is looked at again at each
  screen poll for a minute; and each known work's layout pointer is read at each screen poll,
  since a pool's slot is built into again and again.
- **Sweep the whole heap now and then:** at the start, 1.5 s after another app starts, when a
  reader is waiting for something only a sweep finds (a gauge, a map block, the talk object behind
  a page the field shows), and every 30 s as a net. The net logs what it finds that the watch did
  not deliver.

**Amended 2026-09-27** ([spike 123](../findings/memory-and-discovery.md)): Sun's heaps are
watched too, keeping to its small blocks (ADR-0035), and the watch was mended for both games. A new
heap's siblings are adopted as new, where the second heap made between two looks had none of its
blocks handed over; a heap whose count of blocks in use grew past what the walks found has its
whole list looked over; the ends of a changed list are walked no further than the sixteen looked
at, where the whole list was walked and cut; and a layout just taken in has its tree walked again
on its next five screen polls, and waits for a root pane before it is taken in at all.

**Amended 2026-09-28** ([spikes 130, 133 and 134](../findings/memory-and-discovery.md)): a new
block's sweep reads the pages its span is in and not the memory between it and the next, as this
record meant all along (`merge`'s default gap of a megabyte had read seven times what the new blocks
held); the field no longer asks for a whole sweep while another app runs; and ORAS's field coming
back after a battle or a menu gets no whole sweep 1.5 s on, since it builds its map blocks again
where they were and those sweeps found nothing a reader asks for by class. Every other app start
still gets one. Finding things cost 145.6 s over the eleven ORAS movies, where it cost 276 s the
morning before.

## Consequences

- Finding things cost 142.5 s over movies 1, 2, 4 and 5 (2,839 s of play), against 466 s on main
  and 252 s after spike 99: 5% of the runtime's time, where it was 16%. Pauses in polling over
  100 ms fell from 886 to 264.
- A new object is found within a tenth of a second of its allocation, where a sweep found it up
  to two seconds later on main, and up to six after spike 99.
- **Anchors, not addresses** (ADR-0003) holds: every heap is found by class and validated by its
  signature, and nothing is reached through an address written down.
- A reader must take a partial find: add what it holds, forget nothing for its absence, and check
  what it keeps. The dialogue reader checks a window's class every poll, because a freed window's
  pointers lead anywhere.
- What is built long after its memory was taken, into a pool or a block allocated before, is
  found by the sweep after an app starts, by the net, or when a reader asks: the talk windows a
  door builds, the field's zone data. The net's log line says what it caught. The first page of a
  talk whose talk window no sweep has found yet goes without its speaker's name.
- **Replays are touchier.** Reading hard while the game loads a file shifts how long the emulator
  charges for it, and a replay can go its own way with nothing out of sync; a regression run says
  whether the game wrote the save its movie ends on (`spikes/regress.py`).

**Rejected:** sweeping a new block once, as it appears (a pool is built into after it is taken);
reading every new block whole (315 MB over movie5, against 89 MB, for no object more); watching the
linear heap's heaps (the battle's layouts there are reached through their pools, spike 64);
sweeping the whole heap two seconds after every door (it came while the zone was loading, and
movie1's replay went its own way every time); resting between a sweep's chunks (it did not stop
that); looking at the heaps five times a second (9% less spent, and menus read out whole as they
opened, before their cursors were found).
