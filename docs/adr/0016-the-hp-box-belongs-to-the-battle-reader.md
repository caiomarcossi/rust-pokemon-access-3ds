# 16. The HP box belongs to the battle reader alone

- **Status:** Accepted
- **Date:** 2026-09-16
- **Deciders:** Orion

## Context

Screen text said the HP box's pieces as it slid in — "5 18/20 3" — on top of the battle reader's
"Torchic ♀ L5 18/20".

## Decision

A layout **is** the HP box once the battle reader has found the player's panel in it, and while a
battle is up its pieces are left out of screen text.

The box's pane names are remembered too, because they name its layout file: a box built at another
address for the next battle is known from its first piece. That matters because the pieces arrive
one at a time — battles.ctm's second battle showed "6" eight seconds before "22/ 22".

Before the box is known, a level alone on the top screen in a panel holding a bar is the enemy's
and left out too: the enemy's panel can arrive a poll ahead of the player's.

## Consequences

Out of a battle nothing is left out, so a layout built later where a box used to be is read as
usual.
