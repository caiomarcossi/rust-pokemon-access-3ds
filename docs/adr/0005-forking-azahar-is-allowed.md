# 5. Forking Azahar is allowed, under two constraints

- **Status:** Accepted
- **Date:** 2026-09-12, reversing an earlier "no emulator fork"
- **Deciders:** Orion
- **Built:** the fork exists and the rig drives it (`spikes/rig.py`). Nothing is shipped to
  players, and the runtime does not need it.

## Context

Reading Azahar's source paid for itself immediately: the watchpoint support in
[findings, spike 4](../findings/tools-and-rig.md) is invisible from the outside. And save states
during a movie replay — the thing that makes the rig usable — needed one scoping change the
upstream build will not make ([findings, spikes 6 and 7](../findings/tools-and-rig.md)).

## Decision

Forking is allowed. The source is a clone of https://github.com/KenzonYeoh/azahar (2126.0,
`d204921`), built where `azahar` in `spikes/rig.toml` says, and the fork is what the rig drives
([rig.md](../runbooks/rig.md#setting-up-on-a-new-machine)).

**Shipping** one to players is a different question and still answered "only if needed": the
standing plan is to fork only if the runtime needs push-based hooks that do not stop the world,
and to prefer a finding that keeps the runtime pure polling.

Two constraints if that ever happens:

1. The fork is a derivative of GPL-2 code, so it lives in **its own repository under GPL-2** while
   pa3ds stays MIT and talks to it over a protocol.
2. The clean-room rule holds in the other direction too: **no Azahar code is copied into pa3ds**
   (ADR-0030).

## Consequences

Users of a shipped fork would need our build, which is the cost that keeps this at "only if
needed". The rig's fork is a development tool and costs players nothing.
