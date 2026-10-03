# 3. Anchors, not addresses

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** Orion

## Context

Every existing tool hardcodes addresses per game version, and every one of them breaks on a patch.
ORAS also ships an unpatched v1.0 whose dump differs.

## Decision

**Find structures by scanning and validating at runtime.** At startup, sweep the game process's
readable virtual address space — not physical FCRAM, which the RPC cannot see — for recognisable
structures, confirm each with a **validator**, derive everything else as offsets from it, and cache
the result against the ROM fingerprint.

A bare address is never the way to reach data. Hardcoded struct layouts and anchor definitions are
fine; a hardcoded address is not.

## Consequences

- A patch, a different ROM revision, or a rebuilt emulator costs nothing.
- Anchors are revalidated on an interval, and on failure pa3ds rescans and says "reconnecting"
  once.
- The cheapest anchor there is turned out to be the class name: ORAS ships with RTTI, so an object
  names its own class through its vtable ([findings, spike
  13](../findings/memory-and-discovery.md)). A name says what an object was built as, never that
  it is still alive — which is what validators are for.

**Rejected: hardcoded addresses per version.**
