# 1. Attach over Azahar's RPC, and over its GDB stub for what a scan cannot answer

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion

## Context

pa3ds has to read a running game's memory. The tools that came before it read the emulator's
process with `ReadProcessMemory` and hardcoded where things are, which breaks whenever the
emulator is rebuilt.

## Decision

Attach to a running Azahar over its **RPC** (UDP, port 45987) for polling, and over its **GDB
stub** when a question is better answered by the emulator than by a scan — "who wrote these
bytes" is a watchpoint, not a search. **No ROM patching**, ever.

Measured before it was settled: 1024 bytes a packet, about 73 µs sequential and about 24 µs a read
pipelined at a batch of eight ([findings, spike 2](../findings/tools-and-rig.md)).

## Consequences

- pa3ds survives an emulator update, because nothing depends on the emulator's own layout.
- Every read costs a packet, so the page cache and its phases (ADR-0004) carry the design.

**Rejected: `ReadProcessMemory`.** It breaks on emulator updates, which is the failure every
existing tool has.

**Not adopted, still open:** BizHawk with its Encore core, weighed in
[emulator-comparison.md](../reviews/emulator-comparison.md).
