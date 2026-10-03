# 2. The GDB stub is a reverse-engineering instrument, never a runtime dependency

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** Orion

## Context

The stub answers questions nothing else can — a watchpoint names the code that wrote a byte
([findings, spike 4](../findings/tools-and-rig.md)) — and it is tempting to keep using it at
runtime.

## Decision

The stub is for **finding things out**, never for running. Nothing a player runs may depend on it.

## Consequences

It stops the world on every hit and holds a single client, so a crash in pa3ds while the game is
halted **freezes the player's game**. That is acceptable for an agent at work and never acceptable
for a player. It also means the stub's rules are the opposite of capture's — emulation must be
*running* for the stub and *paused* for a snapshot — so the two can never be held at once.
