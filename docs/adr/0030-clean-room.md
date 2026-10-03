# 30. Clean-room: their behaviour and formats, never their code

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion

## Context

pokemon-access and PKHeX are GPL, and Azahar is GPL-2. pa3ds is MIT. All three are worth reading:
they solved problems pa3ds has.

## Decision

**Their behaviour and their file formats are fair game; their code is not.** Read what they do and
what a format means, then write it here from scratch.

## Consequences

- The Gen 6 text key, the GARC layout and the save's record format were all implemented from what
  the formats are, not from anyone's source.
- It holds in the other direction too: no Azahar code is copied into pa3ds, even though the fork
  is ours to change (ADR-0005).
- When a layout is copied deliberately — Pokemon Access's shipped-table-plus-renames split
  (ADR-0023) — it is the *idea* that is copied, and the write-up says so.
