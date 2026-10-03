# 27. Movies, not sessions

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** Orion

## Context

The user is blind and cannot check what pa3ds says against the screen; the agent cannot see the
screen either. Asking him to reach a game state by hand, repeatedly, does not scale and is not
reproducible.

## Decision

The user **records the target stretch of gameplay once**; the agent replays it deterministically
and captures snapshots at any frame, repeatedly, without him present. Asking him to reach a state
by hand is the fallback, not the method.

## Consequences

- Every claim about behaviour can be checked twice, which is what the findings log rests on.
- A replay desyncs now and then even when everything is right, and a desynced replay is *quiet*,
  not wrong — so it looks exactly like a reader that has just been broken. `docs/runbooks/rig.md`
  carries the checks.
- The rig needed save states during playback, which upstream Azahar refuses; that is what the fork
  is for (ADR-0005).
