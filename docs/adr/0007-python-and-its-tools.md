# 7. Python 3.13, uv, ruff, pyright, and prismatoid for speech

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion

## Decision

Python 3.13 with uv as the package manager, pytest, ruff and pyright in strict mode. Speech goes
through `prismatoid`, which reaches NVDA at priority 103 ([findings, spike
1](../findings/tools-and-rig.md)).

## Consequences

Rust or a direct C++ binding remains available if the Python path fails; nothing so far suggests it
will. The measured loop runs at 58 polls a second with the readers in it, which is the number that
matters.

What this costs at shipping time — a player has neither Python nor uv — was raised on 2026-09-22
and deliberately left open: ADR-0033 says why, and what was ruled out on the way.
