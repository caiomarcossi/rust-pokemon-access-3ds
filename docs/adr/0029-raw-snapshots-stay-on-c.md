# 29. Raw snapshots stay out of the checkout, and only scrubbed ones are committed

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** Orion

## Decision

**Raw** snapshots are complete, contain the user's save data, and live in
`%LOCALAPPDATA%\pa3ds\captures` (`snapshot.capture_dir`), outside the checkout, with a size cap
and oldest-first pruning. The checkout's own disk can be short of room: the one this was decided
on had under 30 GB free.

**Scrubbing** reduces a snapshot to the byte ranges a reader actually touched, zeroing the rest.
Only scrubbed snapshots are committed to `fixtures/`, opt-in.

Scrubbing is **a command, never automatic**.

## Consequences

Scrubbing before a reader is known to be correct destroys the evidence that it is not — which is
exactly when the raw bytes are wanted. The cost is disk, and the cap handles it.
