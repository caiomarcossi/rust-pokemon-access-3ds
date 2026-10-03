# 23. Names for what the game never names, with the player's winning

- **Status:** Accepted
- **Date:** 2026-09-21, renaming built 2026-09-22, the rival's name from the ROM 2026-09-23
- **Deciders:** Orion (reopening his own constraint)

## Context

ORAS knows a speaker as an **overworld sprite** and a sign as a **slot on a map**, and has no word
for either. Four spikes went looking: not on the model, not in the zone's own record of it, not
through the script id, and not through the trainer table
([findings, spikes 52, 73, 75, 76 and 77](../findings/field-and-rom.md)). The game never shows a
speaker's name, so it never stores one.

The standing constraint said every name spoken comes from the ROM or RAM. Keeping it meant every
conversation stayed unattributed.

## Decision

pa3ds carries names of its own, under three rules the user set when he reopened the constraint:

1. **A name pa3ds invents never replaces one the game says.**
2. **It lives in a data file, not in code** — a table a game and a language, beside the game's
   code.
3. **The player's own names always win**, in a file a ROM fingerprint under `<pa3ds>\user\names` (ADR-0032),
   and are shared by sending that file.

Two key spaces, because they are different things: a **sprite** is one character everywhere in the
game, so naming Norman once names him in every scene; a **place** is a slot on one map, so a sign
named in Petalburg says nothing about one in Rustboro.

**A speaker is said when it changes**, not before every line: "Norman: Not bad... It was a good
battle.", then his next line plain.

**Renaming while playing, 2026-09-22** (`dialog`, `common.navigate`). The shipped table is a
starting point; the player's own file is where the game actually gets named, because **the player
is the one standing in front of the thing**. **Shift and K** renames whatever the navigation keys
are on, as Pokemon Access does it; **shift and J** renames whoever last spoke, which is pa3ds's
own, because pa3ds knows who is speaking and because the moment a player learns who somebody is
is the moment they have just heard them.

**Which name space a thing belongs to is the thing's business, not the player's.** A **move
model** — a person, an item ball, a prop standing on the map — is named as a *character*, the
same number the speaker is read as, so naming the person in the Gym names Norman in the street
and in dialogue too. Everything else is a *place*, a slot on one map, so a sign named in
Petalburg says nothing about a sign in Rustboro. That answers the question the plan left open,
and it means one key does the right thing without asking.

The cost is a sprite everybody nameless shares: naming "the man in the Mart" names every
townsperson drawn as him. That is already how the shipped table works — `0x119` is "Townsperson"
— so it is the behaviour being kept, not a new one.

**A native Win32 dialog**, a `#32770` built from a template in memory with `ctypes`, on a thread
of its own so the poll loop does not stop while it is up; the answer comes back through a queue
the runtime drains at the next poll. Not Tk, and this was measured rather than assumed (spike 81,
NVDA on this machine): the native box is read as "Sign, 1 north. Called: Signpost 1 — edit —
selected Signpost 1", while the Tk one announced only its window class **and was refused the
foreground**, so it opened behind the game. The native one took the foreground from behind Azahar
every time, which was the risk — pa3ds is not the foreground process when the key is pressed,
since the hook sees the key but never receives it.

**The new name is said back**, because a box closing is not evidence to a blind player that
anything was saved, and it is read back out of the store rather than assumed: an empty name
clears the player's and gives the thing back to the game's own word or the shipped table, so
what is said is what is now true. Nothing chosen opens no box at all — that is the common case for a mistyped key, and a
window left in front of the game would be worse than silence.

**Where they live, from 2026-09-22.** `names/<language>/<rom>.toml` under the player's own
folder, laid out the same way as the shipped table beside the game's code: TOML like everything
else a person reads (ADR-0031), and per language, because a name is a word in a language and
English names for a ROM played in French belong to neither. The file opens with a comment saying
which game, ROM and language it is for, since sharing names is a matter of sending the file to
somebody who then has to know what they have been sent. The game's name is read from the
emulator's own window title (`pa3ds.core.window`), which Azahar took from the ROM, so pa3ds keeps no
table of game names.

## Consequences

- Replayed over wally.ctm, every line of the Petalburg scene landed on the right character, and
  the trainer nobody has named — a sprite shared by many people — read exactly as before.
- **A character is not always one person.** `0x7a` is May to a boy and Brendan to a girl, so what
  ships is "Rival"; a player who wants "May" says so.

  **Why, and what it would take, 2026-09-23** ([spike 85](../findings/field-and-rom.md), after the
  user asked whether this would work in other languages). `0x7a` is not a character at all: the
  ROM calls it `WKOBJCODE00`, a **work object** — a slot a script fills with whoever is meant to be
  standing there. The name itself is no trouble: the ROM's trainer name list carries both
  protagonists in all seven languages (`Brendan`/`May`, `ユウキ`/`ハルカ`, `Brice`/`Flora`,
  `Brix`/`Maike`…) at the same line numbers in each, so a **line number is a fact about ORAS and
  the word is the ROM's** — nothing of pa3ds's would be spoken, and a French ROM would say Flora
  without anyone typing "Flora". What was missing was which of the two the player is, and the
  user's call was that "Rival" stays unless that can be read reliably: a name we cannot get right
  in every language is worse than a word that is right in none.

  **Built the same day.** The user suggested replaying the two openings, one played as each
  ([spike 87](../findings/field-and-rom.md)): the player's own model is `0xac HERO_RS` as the boy
  and `0xab HEROINE_RS` as the girl in every zone of both, the other family's house included. So
  **the game's own word** now sits between the player's names and the shipped table
  (`Names.use_game`, `Gen6Profile._game_word`): the player's name for the rival wins, then May or
  Brendan out of the ROM, and "Rival" is only heard without a readable ROM. This is rule 1 at
  work — the shipped word was standing in for one the game has.

  The one reservation: `0x7a` is a work object, and a script could fill it with somebody else later
  in the story. The shipped "Rival" had the same exposure, so nothing is newly wrong; a movie that
  shows it will say so. **Superseded the same day** by reading who filled it (below): the rival is
  now named by their own sprite, and the player's gender is not read at all.
- **A work object's name is a slot's, not a person's** ([spike 88](../findings/field-and-rom.md),
  asked by the user, 2026-09-23). A fixed sprite is one look, so naming a youngster names all 35
  of them in 31 zones, which is this ADR's choice for a shared sprite. But sixteen sprite ids are
  work objects, thirteen of them used, and a move model keeps the work object's id rather than
  the sprite a script filled it with — `0x7a` alone stands in 27 zones, Meteor Falls and
  Mt. Chimney among them. A name given to a work object, by the player or by pa3ds, goes to
  whoever fills it next. Reading the sprite it was filled with would fix that, and wants a spike.

  **Fixed the same day** ([spike 89](../findings/field-and-rom.md), the user asking for renames to
  be targeted). A move model's `+0x898` is the sprite it is drawn with: the rival's model reads
  `0x7a` at `+0x10` and `SUPPORT_F_RS` (May) or `SUPPORT_M_RS` (Brendan) at `+0x898`. So a work
  object **is the character it was filled with** — for the speaker, the navigation list and a
  rename alike — and the slot itself is nobody. Every other model is its `+0x10` as before. The
  rival's name comes from the ROM by their own sprite, and "Rival" ships for those sprites, heard
  only without a ROM. A name the player gave `0x7a` before this is left in their file and no
  longer matches anyone; there is no migration, as with everything unreleased.
- **Most people take the game's own word too, from 2026-09-23** ([spike
  90](../findings/field-and-rom.md), the user asking for the ROM first and the shipped table as the
  fallback). A trainer's map record says which trainer they are, and the trainer's class says what
  kind: a sprite that two trainers or more stand as, all of one class, is that class in the
  player's language ("Youngster", "Team Magma Grunt"), and a pair class is left out. The story's
  people are not trainers on the map, so a table of ours maps each one's sprite to the trainer they
  are — read off the ROM's sprite ids and where each stands — and the ROM gives the name. The
  shipped table is the English fallback for all of it, plus the people the ROM never lists, and
  says "May" and "Brendan" now that the rival's sprites are known to be theirs.
  **A fallback in every language** the game carries, the same day ([spike 91](../findings/field-and-rom.md)):
  the ROM's own words, and the people it never lists read off sentences that name them in every
  language at the same line. The Nurse, the Townsperson and Mrs. Birch are pa3ds's own English
  and stay English-only.
- **Doors take the game's own word too, from 2026-09-23** (ADR-0008): a door is a place, a slot
  on one map, and where the ROM pairs it with a building the place is called what the game calls
  that building, behind the player's own name.
- **A slot on a map says which table it is in, from 2026-09-23.** Signs and warps are numbered in
  tables of their own, and the key was the bare number, so sign 1 and door 1 on one map were one
  place: renaming one renamed the other, and the ROM's door names were said for signs. A warp's
  slot now carries a bit of its own (`field.WARP_SOURCE`), as a use tile's does. A door renamed
  before then is keyed as the sign of its number and wants naming again.
- **Every kind of place carries a tag, the same day** (issue 1). Two marker bits were the same
  fault waiting: an edge's slot was `zone << 16 | gap`, and a zone from `0x100` up put a bit on
  `TILE_SOURCE`, so an edge into the Safari Zone and a use tile at (2272, 0) were one place —
  out of ORAS's reach, not X/Y's. A slot now carries a four-bit tag above 32 bits
  (`field.SOURCE_KIND`) saying which kind it is (furniture, warp, use tile, edge); every number
  under it is built from u16s the game holds, so nothing read from the game can reach the tag,
  and a slot's kind is read off it rather than off one bit. A door, a use tile or an edge
  renamed before then wants naming again.
- **An exit's place is the game's, and a name does not replace it** (2026-09-24, ADR-0024). An
  exit into another place is said with that place after its name, "the gap in the trees to
  Route 101": the name is the player's, the place is something the game says, and rule 1 keeps
  it. A name that already says the place is left as it is.
- **Use tiles renamed before the kind tag moved above 32 bits were keyed `1 << 24 | cell`** and
  answer nothing now; the user's first thirteen, in Alpha Sapphire, were re-keyed in their file by
  hand, since no code migrates (spike 93). They are cells, the same every run, not addresses.
- **The houses swap by gender** — the player's is zone `0xdf` for a boy and `0xe1` for a girl — so
  a name kept against a (map, slot) belongs to one playthrough's gender. Worth knowing before
  place names are shared between players.
- Pokemon Access is laid out the same way, a shipped table a game and a language with renames in a
  file of their own, which is the part worth copying.
