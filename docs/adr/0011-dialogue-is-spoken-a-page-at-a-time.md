# 11. Dialogue is spoken a page at a time, as the page starts to type

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** Orion

## Context

The whole message is already in memory before the game types a character of it
([findings, spike 10](../findings/text-and-dialogue.md)), so pa3ds could say all of it at once —
and would then run ahead of the player, who is still pressing A through page one.

## Decision

A message is spoken **a page at a time** — the text between two waits for A — the moment the game
**starts to type that page**, ignoring the typewriter animation. The typed line is what says which
page is up ([findings, spike 14](../findings/text-and-dialogue.md)).

Pressing A cuts speech and advances. Nothing is ever half-read.

## Consequences

The player paces the game with A exactly as a sighted player paces it with their eyes, which is
the whole point: pa3ds follows the game rather than narrating it.
