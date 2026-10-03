# 14. Name entry reads the key under the cursor

- **Status:** Accepted
- **Date:** 2026-09-14
- **Deciders:** Orion
- **Built** (`games.common.cursor`, the key cursor): the key under the cursor, the wordless confirm
  key by its texture, and the letters left after a deletion.

## Decision

The character under the cursor as it moves, "Cap O" style for capitals, the layout announced when
L or R switches it, and the box contents read when focus lands on it.

## Consequences

The keyboard's cursor is a picture the game moves into the chosen key's touch area, so the item is
the key's own words — and for the key with no words, the one that confirms a name, the words of
its texture, "return" ([findings, spike 68](../findings/screens-and-cursors.md)). Deleting a
letter says the letters left, which the user asked for after typing a name by ear.
