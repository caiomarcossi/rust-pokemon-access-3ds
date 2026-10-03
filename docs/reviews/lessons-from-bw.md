# What pa3ds can take from Real Amethyst's Black and White scripts

Read on 2026-09-16 in the `Lua` folder of the Black and White fork's install: `main.lua` (204 lines),
six `core/*.lua` files (~430 lines together) and `games/bw.lua` (23,064 lines). They run inside a
BizHawk fork on melonDS, so nothing here is code to copy — the project is a different licence, a
different console and a different language. What transfers is the reasoning, and the mistakes
already paid for.

**Clean-room still applies.** Behaviour and formats may be read; code may not be copied. Nothing
below is a line of theirs.

## The things worth taking

- **A modifier turns the pad into an accessibility layer.** Hold the left trigger and the game
  receives no input at all, while the pad drives whichever list is live — the overworld nav list in
  the field, the HP list in battle (`core/pad.lua`). The pad presses are folded into the same key
  names the keyboard readers already listen for, so every reader gains controller support without
  being touched. pa3ds plans pokemon-access mnemonics on the keyboard and a rebind of Azahar to a
  Game Boy-like layout; a held-modifier layer is a better answer to the same problem, because it
  needs no keys taken away from the game. Azahar has no Lua, so pa3ds would block the button by
  the one means it has, and it does not have one yet — pa3ds never presses buttons. Worth its own
  decision.

- **Fail closed on every read, and never raise.** Their memory helpers answer 0 for an address out
  of range and tolerate a nil address, with a comment saying why: in BizHawk a Lua error kills the
  whole script, and a blind player then loses every bit of speech with no way to tell that from a
  quiet moment in the game. pa3ds is a separate process and survives, but the *silence* is the same
  failure. ADR-0019 already says a reader that throws is logged and snapshotted with one spoken
  "reader error"; this is a second vote for that rule, from a year of live use.

- **A width on every range check.** Their guard originally tested only the first byte of a read, so
  a pointer landing in the last three bytes of RAM read off the end of the domain. `Memory.read`
  in pa3ds goes over the RPC and the emulator answers, so nothing crashes — but a read that
  straddles the end of a region comes back short, and only some callers check.

- **The sprite outranks the script when naming somebody.** Their NPC names come from an offline
  script interpreter, and a cutscene's script slot is named after whoever speaks *first* in it,
  which is often not the actor standing there. Ten sprites belong to exactly one character each, so
  a name from the script that contradicts the sprite is provably somebody else's line and is
  dropped. **A generic word beats a wrong name.** This is the rule to hold on to for pa3ds's own
  speaker naming, whatever the source turns out to be.

- **Check the romhacking tools before reverse-engineering.** Their whole NPC naming rests on a
  script opcode argument-length table, and they got most of it from an existing community tool
  rather than deriving it. Their own note calls it the lesson of the project.

## The things to avoid

- **23,000 lines in one function.** `games/bw.lua` is one chunk, wrapped in `do` blocks to get past
  Lua's 200-locals-per-scope limit, with comments explaining that adding one more `local` fails to
  compile. Real facts are then kept as fields on unrelated tables to dodge the ceiling. pa3ds is
  3,900 lines of Python across eleven modules with tests; there is nothing to take here except the
  warning.

- **The stand-down list is the cost of having no ownership rule.** Their main loop decides which of
  several readers is the authority for the current screen, and the field reader has to stand down
  for the party screen, the shop, the Pokédex, the naming keyboard, the hatching screen, the
  evolution screen and an Xtransceiver call — each one a separate bug found in live play, each with
  its own comment. pa3ds hit the same class of bug and answered it with a rule instead: a layout a
  cursor has been over belongs to that cursor, and a reader judges liveness from the object rather
  than from a list of screens. That rule is worth defending; it is what keeps this list from
  growing here.

- **Hardcoded name tables.** Their character names are generated offline into Lua tables — 214
  names over 71 zones, plus sprite and role tables. pa3ds forbids hardcoded game text, and
  everything spoken comes from the ROM or RAM. Their approach is the fast one and it is not open
  to us.

- **Writing to game memory.** They ship an instant-text option that writes to RAM. pa3ds announces
  and never acts; that is settled.

## Where they differ from us and both may be right

- **Interrupt or queue.** Their rule: queue what is on screen together, interrupt whatever has
  *replaced* what came before — so a new screen interrupts, a second element of the same screen
  queues, a cursor move interrupts, and a second page of a message interrupts, because pressing A
  wiped the box. pa3ds decided on 2026-09-14 that nothing interrupts by default, because NVDA
  already cuts speech when the player arrows or presses A in the game window, and pa3ds
  interrupting itself cut the cursor item off with the description the same keypress brought up.
  Both were decided from live play by a blind user. The difference is worth remembering if
  `--interrupt` is ever revisited: theirs is not "interrupt everything", it is "interrupt only what
  has been replaced".

- **Synthetic touch versus the mouse.** BizHawk lets them press the touch screen from Lua, one
  frame at a time, as a queued gesture; they found the hard way that the coordinates have to go
  through an input *override* rather than a sticky axis hold. pa3ds's one approved exception to
  "announce, never act" is moving the OS mouse to a named touch target, which is a weaker
  mechanism — it cannot hold, drag or release reliably. If pa3ds ever moved to BizHawk, this is the
  one capability that would arrive for free.

## What they do not solve, and we still want

**Nothing in their scripts says who is speaking a line.** Their NPC names are for the navigation
list — what you can walk to and talk to — not for dialogue attribution. A back-and-forth cutscene
reads there exactly as it reads in pa3ds today: lines with no names on them. So spike 51 is not
re-treading ground somebody else has covered.
