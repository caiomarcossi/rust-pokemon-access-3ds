# Runbooks

How to do the things that cannot be re-derived from the code, because they live in an emulator's
UI, in a dump on another drive, or in a procedure with traps in it.

| Runbook | For |
|---|---|
| [rig.md](rig.md) | Booting Azahar to a known moment, replaying a movie, screenshots, and the ways a replay lies |
| [snapshots.md](snapshots.md) | Capturing RAM, what fools a diff, naming heap objects by RTTI, where captures live |
| [gdb-stub.md](gdb-stub.md) | Watchpoints and "who wrote these bytes" — rules that are the opposite of capture's |
| [executable.md](executable.md) | The code dump, disassembly, and what a fixed address is good for |
| [speech.md](speech.md) | Writing text that is heard rather than read: what repeats, what interrupts |

These were `docs/agents/` until 2026-09-21, when `agents/` went back to being configuration for
the engineering skills.
