# 4. The poll loop runs at 60 Hz, reads in phases, and never stalls

- **Status:** Accepted
- **Date:** 2026-09-11, revised 2026-09-16 and 2026-09-26
- **Deciders:** Orion

## Context

A screen reader that misses a line of dialogue is worse than one that is slow. The first runtime
polled at 30 Hz and managed 20.6 polls a second, because each heap sweep held polling up for about
300 ms ([findings, spike 59](../findings/memory-and-discovery.md)).

## Decision

- **60 Hz**, which the measured RPC comfortably affords, configurable down with `poll_rate` in the settings file.
- **Nothing may hold polling up for long.** A sweep reads a chunk at a time and polls dialogue
  between chunks.
- Each poll reads in a **phase** of `memory.PageCache`: pages are fetched as whole 1024-byte RPC
  reads, pipelined in one burst with replies matched by header id, and a phase starts by fetching
  the pages it read last time ([findings, spike 62](../findings/memory-and-discovery.md)).
- **Events are produced by diffing** this tick's regions against the last.
- The rate **rises automatically** when the emulator's frame counter outruns the wall clock, so
  fast-forward does not silently drop text.

## Consequences

Per-read latency is flat in size but per-packet overhead is not, so batching is what makes 60 Hz
affordable. Sweeping was the only expensive thing left, and since 2026-09-26 what is new is
found where the game allocated it, and only that memory is swept (ADR-0034); a whole sweep is the
net under it. Until then the heap was swept on a schedule: around what had been found every couple
of seconds and whole every few, with the whole heap a second after every page ([findings, spike
69](../findings/memory-and-discovery.md)), then, after spike 99, when an app started, a question was
asked or the screen changed.
