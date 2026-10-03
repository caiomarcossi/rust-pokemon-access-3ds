# 20. Speech goes to the player's own screen reader, with a dedicated voice beside it

- **Status:** Accepted
- **Date:** 2026-09-10
- **Deciders:** Orion
- **Partly built.** Speech reaches the player's screen reader through prismatoid. The fallback
  order, the startup warning when NVDA is not chosen, the dedicated second backend and braille are
  not wired.

## Decision

The player's **NVDA voice by default**, with no rate or voice control applied, because the NVDA
backend exposes none. The optional dedicated voice is a *second* backend held alongside NVDA —
SAPI or OneCore, both of which do expose voice, rate, volume and pitch.

Fallback order is NVDA, OneCore, SAPI, with a spoken warning at startup when NVDA is not the one
chosen, and screen-reader output switchable off entirely.

Prism abstracts JAWS and others; pa3ds neither special-cases nor tests them. Braille gets the raw
string where speech gets the formatted one, and is a bonus rather than a goal.

## Consequences

A player hears pa3ds in the voice they already know, at the rate they already set. The formatting
rules (ADR-0013) apply to speech only, which is why braille takes the raw string.
