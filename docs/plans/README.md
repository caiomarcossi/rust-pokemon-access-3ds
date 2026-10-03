# Plans

Work that has been designed but not built. A plan is **not binding** — it is a proposal with
enough detail that building it is typing rather than deciding.

When a plan lands, **delete it** and fold what it decided into an ADR. A plan left lying around
after the code exists is the thing that goes stale first, and a stale plan is worse than none:
someone will follow it.

Waiting, one plan a document: none.

The plan for naming a picture by the image it draws landed on 2026-09-29 and 2026-09-30
([ADR-0036](../adr/0036-a-picture-is-named-by-the-image-it-draws.md); findings, spike 147); git
history has it.

The plan for the PC Box's buttons drawn as pictures alone landed on 2026-09-29 for the one button
the D-pad reaches ([ADR-0012](../adr/0012-a-menu-says-the-item-under-its-cursor.md); findings,
spike 143); git history has it.

Ideas with no steps yet are entries in the [roadmap](../roadmap.md), one an idea. The plan to find
the DexNav's record through the classes that hide a Pokémon landed the same day, through another
class than it guessed ([findings](../findings/README.md), spike 135).

The reorganization of the code by what shares it (2026-09-28) landed the same day: [ADR-0006](../adr/0006-a-core-and-a-package-a-game.md)'s
consequences have what it decided, and git history has the plan and its checklist.

Both plans written on 2026-09-21 landed on 2026-09-22:
the keys file is [ADR-0022](../adr/0022-the-keys-pa3ds-answers.md) and renaming while playing is
[ADR-0023](../adr/0023-names-for-what-the-game-never-names.md). What is decided but unbuilt is in
the [roadmap](../roadmap.md).

## What belongs here

- A feature whose shape is settled but whose code is not written.
- A migration with an order of operations worth agreeing before starting.
- Anything the user has approved in conversation that will take more than one sitting: write it
  down so the next session does not re-derive it.

## What does not

- A decision with no work attached → an [ADR](../adr/README.md).
- Something you are about to do in the next ten minutes → just do it.
- A list of ideas → the [roadmap](../roadmap.md), which already has the open questions.
