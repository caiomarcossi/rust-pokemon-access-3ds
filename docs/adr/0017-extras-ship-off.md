# 17. Anything the game does not show is an Extra, and Extras ship off

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Not built.** No Extra is read at all yet, so there is nothing to toggle. The rule already
  decides arguments, which is why it is written down now.

## Decision

Hidden items, exact enemy HP, IVs, EVs, natures and abilities before they are revealed, shiny
detection and encounter slots are **Extras**. Each is its own toggle and each **ships off**, so the
tool is honest by default. The list is open to additions.

## Consequences

This is what decides arguments that would otherwise be about taste: the enemy's HP is a percentage
(ADR-0015) because the game draws a bar, and an exact number belongs behind a toggle if it is ever
wanted at all.
