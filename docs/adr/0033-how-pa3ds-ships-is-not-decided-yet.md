# 33. How pa3ds ships is not decided yet, and nothing is added for it

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Orion
- **Nothing built, on purpose.** There is no packaging tool in this repository and none is to be
  added until this is reopened.

## Context

pa3ds needs Python and uv to run, which no player will have. Every other accessibility layer of
this kind uses the emulator's own Lua, which Azahar does not offer. The question was raised on
2026-09-22 with four answers on the table: freeze it, compile it with Nuitka, port it to Rust, or
move to BizHawk and Lua.

Two things were established while looking, and both are worth keeping even though nothing was
decided:

**BizHawk does emulate the 3DS.** Its **Encore** core, itself a Citra fork like Azahar, arrived in
2.11. An earlier claim here that BizHawk had no 3DS core was wrong. What makes it unattractive is
not that it cannot run the game:

- Its Lua runs **on the emulator thread**, so a heap sweep becomes a frame-rate problem rather
  than a background one. pa3ds sweeps in 512-page chunks *between* polls precisely to avoid that
  (spike 59).
- Its Lua is NLua, with **no FFI**, so reaching a screen reader means a native DLL or a helper
  process — which is the second process Lua was supposed to save.
- The RPC, the GDB stub and the fork's save-state-during-replay change are the whole debugging
  loop, and none of them come along.

**Nuitka is not the ahead-of-time escape hatch it looks like.** It compiles to C, but against the
CPython API, so it still ships a runtime. Faster startup and harder to pick apart; not a
runtime-free binary.

## Decision

**Nothing yet.** No PyInstaller, no Nuitka, no port, no move.

The user, 2026-09-22: *"We're still in development and not ready to ship anyway, and I'd rather
not put in pyinstaller then removing it later to replace it with a Rust port."* Adding a packaging
tool now buys nothing that can be used and commits the repository to a shape nobody has chosen.

What is **not** ruled out: a Rust port is a real option and a good one *later*, as a port of
something finished rather than a rewrite of something half-built. Anchors, struct layouts and
validators are still being found, and the Python toolchain — 79 spikes, the movie rig, the tests —
is why that goes quickly. Performance is not forcing anything either: the loop does 2.14 ms of
work against a 16.67 ms budget (`azahar-config-tuned`).

## Consequences

- A player cannot run pa3ds yet without a developer's tools. That is accepted for now.
- Two facts to check whenever this is reopened, neither of which needs deciding today: what a
  frozen build actually weighs, and whether `prism` survives freezing — it ships `prism.dll` and a
  cffi `.pyd`, and native files have to be named explicitly to a freezer. A spike, not a rewrite.
- Where the player's files live **is** decided and does not depend on this (ADR-0032): beside
  pa3ds, whatever pa3ds turns out to be.
- ADR-0007's "Rust or a direct C++ binding remains available if the Python path fails" still
  stands, and this says when that would be judged: at a shipping version, not before.
