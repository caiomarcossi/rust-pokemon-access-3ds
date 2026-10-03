# Memory, and finding things in it

Where memory is, how an object is found by its class, what a sweep costs, and how a reader is
handed what it needs before the moment passes.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 8 — the text pane, found by shape

2026-09-12. **The dialogue anchor is settled.**

Following `0x00419b0c` and `0x0034836c` from spike 4 led to the setter at `0x003a6f10`, and reading
it gave the struct layout outright:

    pane    +0x08 -> buffer     +0x38 u32 length     +0x91 s8 enabled
    buffer  +0x04 -> UTF-16LE   +0x08 u16 capacity   +0x0A u16 length

- `0x00419b0c` returns from a `bl` to `0x003a6f10`; the memset writer `0x003a6fb8` is inside it.
  `0x0034836c` returns from a `blx r2` through a vtable at `[object+0x1c]` slot `+0x10`, which then
  sets a state byte to 4 — the message system dispatching to whatever renders the text.
- The setter explains everything the black-box work had observed. `movlo r5, #0x200` is why buffers
  are never smaller than 512 bytes; `ldrh r1, [r0, #8]; cmp r1, r5; bhs` is the reuse test, which is
  why one address holds message after message; and `bl #0x14e240` is the copy whose inner PCs
  (`0x0014e260`, `0x0014e288`, `0x0014e328`) are exactly what the watchpoints kept stopping on.
- **The validator is `pane[0x38] == buffer[0x0A]`.** The length is written in two places by two
  different instructions, so agreement is not a coincidence anything else achieves. Scanning 31,976
  pages of heap and linear heap found **3 matches, all three real dialogue, no false positives**:
  Birch's "I went into the tall grass to survey wild Pokemon", a Trainer School line, and
  "Torchic gained\n24 Exp. Points!".
- This replaces prose scanning entirely. Reading the current text is now
  `pane -> +8 -> +4`, with the length at `+0x0A`, and no guessing about what looks like a sentence.
- Message source strings share the buffer shape and are resident in bulk: the whole of Birch's
  opening monologue was readable at once, correctly decoded down to the curly quotes and the
  accented e in "Pokémon".

Two things it does not answer, and **both of its guesses about them turned out wrong**: `+0x91` is
not an enabled flag, and the `\x10` codes are not name inserts. Spikes 10 and 11 have the readings.

## Spike 10 — what the pane is, and what it is not

2026-09-12:

- **The pane carries no "on screen" flag.** Moments were labelled by reading the pictures — the
  words displayed matched against the pane holding them — and then every byte of each pane's first
  0x100 was compared between the pane being shown and the panes that were not. **Nothing
  separates them.** `+0x90` and `+0x91` are 1 for every pane, `+0x1e` and `+0x20` are both 2,
  `+0x18`, `+0x1a`, `+0x1c` and `+0x2a` are constant. The pane holds the text; it does not know
  whether anyone is looking at it.
- **`+0x91` does not mean "enabled".** Spike 8 guessed that from the name it deserved, not from
  the code. `0x003a65d0` appends fixed 0x68-byte line records into the array at `+0x04`, counts
  them in `+0x20` against the capacity in `+0x1e`, and sets `+0x91` only when the last slot fills.
  It means full. A two-line message box is always full, which is why all three panes said 1.
- The rest of the layout, from the same reading: `+0x04` is an array of `+0x1e` line records of
  0x68 bytes each (2 for an ordinary message box), `+0x14` a font and resource provider, `+0x98` a
  manager shared by every pane at once, `+0xbc` a pointer to itself, and the vtables are
  `0x005de1dc` for the base class and `0x005df748` for the 0xf0-byte derived message window that
  `0x003482a8` allocates. Both classes appear on screen, so the vtable is not the test either.
- **The heap does not stop at `0x08800000`.** Probing it a block at a time finds data in
  `0x08000000`-`0x08400000`, again in `0x08800000`-`0x08c00000`, a little above that, and nothing
  at all past `0x09000000`. Spikes 6 and 7 swept only to `0x08800000`, so every scan so far has
  been of a heap with its top cut off — and `0x08c3bd54`, the manager every pane points at, sits
  in the part that was missing.
- **A pane holds the whole message, not the page on screen.** The picture shows one page of a
  message whose full text — several pages of it — is already in the buffer, so reading a pane
  gives more than is displayed and speaking it whole would run ahead of the player.
- Save state loads are **repeatable to the address**: the same panes come back at `0x082acd90` and
  `0x0832588c` run after run, and dialogue reuses `0x08286988` and `0x082882d8`. That is the first
  evidence on whether an anchor can be cached across runs, and it points at yes.
- **Nothing points at it either.** `0x003482a8` stores the window at `controller+0x1c` and
  faults if that field is already set, so it has to be cleared when the box closes — which made
  "something points at me" the obvious test. Sweeping the whole heap for pointers into every pane,
  across six labelled moments: **12 agree and 11 disagree**, and the disagreement is not noise.
  The pane consistently pointed at is `0x082acd90`, which is stale in every one of them, while the
  pane actually on screen usually has nothing pointing at it at all.
- **Nor the line records.** `+0x04` holds `+0x1e` records of 0x68 bytes, one per line, and no byte
  of the first record separates shown from hidden. The `u16` at `+0x64` that the constructor zeroes
  is 0 for every pane at every moment, so it is not a typewriter counter. The byte at `+0x66` is
  the line's index, and it is *swapped* between the two records of a box that has scrolled — which
  is a real signal about scrolling, and says nothing about visibility.
- **What does work is recency.** In four runs, at every moment checked against a picture, the pane
  on screen is the one whose text had just changed. That is enough for the thing pa3ds actually
  has to do — announce dialogue as it appears — but it is a behaviour, not a validator, and it
  cannot answer "what is on screen now" for a screen that was already open.
- Where the answer probably is, untried: each line record's first word points into the **linear
  heap** (`0x14c52b34` and the like), which is where the laid-out glyphs go and what the renderer
  reads. A GDB read watchpoint on two panes' glyph buffers would say which one is consumed per
  frame. The shared manager at `0x08c3bd54`, which every pane holds at `+0x98`, is the other lead.

## Spike 12 — the line records, and a validator for stale panes

2026-09-12:

- A pane's `+0x04` array holds one 0x68-byte record per line of the **page currently laid out**,
  not per line of the message: a one-line page has one record and the next two-line page has two.
  Each record's first word points into the linear heap, at an object that starts with a vtable.
- **Following that pointer says whether the pane is still assembled.** `0x082882d8` points at live
  objects while its message is on screen, and at reused memory the moment the next message takes
  over — while its text still reads back perfectly. `0x0832588c` was dead at every moment. This is
  the validator spike 6's finder never had: it distinguishes a pane that still holds text from a
  pane that is still *being used*, and it prunes most of the stale ones for free.
- The line objects come in two classes. `0x005dbe58` appeared only on the pane being shown and
  `0x005dbc30` only on a pane that was not, which looked like the answer to the on-screen question
  — but the two vtables share 20 of 48 slots, differ in 28 including the destructor, and
  `0x005dbe58`'s tail repeats `0x005dbc30`'s head, which is what a second base class looks like.
  It is a class distinction. Whether a `0x005dbc30` line is ever displayed is untested, and
  claiming visibility without that test would repeat spike 8's mistake about `+0x91`.
- Together, liveness and the vtable pick the pane in the picture at every labelled moment. The
  first is the part to rely on.

Spike 13 read the class names and corrected two things here: `0x005dbc30` is not a plainer line
class but the base-class vtable a destroyed TextBox is left with, and the "second base class" is
the next class's vtable sitting after this one.

## Spike 13 — the classes have names

2026-09-13. **ORAS ships with RTTI**, which settled spike 12's open question from the code alone,
without booting anything:

- A vtable's word at -4 is its typeinfo. A typeinfo's word at +4 is the mangled class name, and
  its word at +8 is the base class's typeinfo when the typeinfo is a `__si_class_type_info`. So
  every polymorphic object in the heap can be named. `spikes/11_rtti.py` does it offline from
  `code.bin`.
- The line objects are NintendoWare layout panes: `0x005dbe58` is `nw::lyt::TextBox`, and
  `0x005dbc30` is `nw::lyt::Pane`, its base. The objects spike 8 called panes are Game Freak's:
  `0x005de1dc` is `gfl::str::StrWin`, and `0x005df748` is `print::MsgWin`, derived from it.
- **A line object with the Pane vtable is a destroyed TextBox.** The TextBox constructor at
  `0x003273d4` calls the Pane constructor first and writes the TextBox vtable after. Both TextBox
  destructors (`0x00327590`, `0x00327640`) write the TextBox vtable first and call the Pane
  destructor `0x00322e50` after, and its first instruction writes the Pane vtable back. Freed
  memory is not cleared, so the object reads as a Pane, with its sibling links at +4 and +8
  zeroed, until something reuses it. The window at `0x082acd90` still holds Birch's intro line
  ("Come see me in my Pokémon Lab!"), and its TextBoxes were destroyed.
- So a `0x005dbc30` line is never a line on screen, and spike 12's liveness test was too loose:
  it accepted any vtable in code, which a destroyed object still has. **A line is live when its
  object's class is `nw::lyt::TextBox`.** With that test exactly one message window is live at
  each of spike 10's five moments, and it is the one whose text had just changed. Liveness is the
  validator; the class is part of the test, not a second one.
- Spike 12's "second base class" was a misreading. The TextBox vtable ends at slot 34, and what
  follows is the next class's vtable, `nw::lyt::Bounding`.
- **The pictures were weaker evidence than spikes 10 and 12 treated them as.** Spike 10 takes its
  screenshot the instant a text changes, and four of its five pictures show no text box at all:
  the text reaches RAM before the box is drawn. Only `001.png` shows a box, and the live window at
  that moment holds its words. `000.png` was labelled as showing the truck line; it shows the
  truck and no text.
- **Classes are an anchor, and a far cheaper one than struct shapes.** A census of one old heap
  capture (the in-game save prompt, 2026-09-11) names 385 classes, among them
  `app::tool::MenuWindow`, `app::tool::MenuCursor`, `app::tool::MenuWindowController`,
  `app::tool::ButtonManager::Button` (33 of them), `field::EventSubScreen::EventFieldMenu`,
  `field::mmodel::MoveModel` (32, very likely the overworld objects), `field::PlayerWork`,
  `pml::pokepara::PokemonParam` (6, a party's worth) and `savedata::BAG`. These are leads, not
  readers. The census counts dead objects exactly as it counts live ones, which is this spike's
  own lesson.
- A class name is a middleware identifier, not game text, so matching on one does not break the
  no-hardcoded-text rule. It is an anchor definition, like a struct layout.

## Spike 16 — a class census across the first battle

2026-09-13 (`spikes/14_battle_census.py`). A census of the heap every 5 s from 110 s to 210 s after
slot 0, with every `gfl::str::StrBuf`'s text; `008.png` at 150 s shows Torchic facing Poochyena.

- ~~Nothing in the heap belongs to the battle.~~ **Wrong; see spike 19.** The census measured the
  heap once, before the fight, and the battle builds its message window at `0x0855b2a4`, where that
  measurement did not reach. What it did see is still true: for the four censuses covering the
  fight no TextBox or battle class was in the swept span, and `field::EventBattleDll`,
  `field::EventProcessCall` and `xy_system::sequence::CommandSetMessageBuf` appeared.
- The two explanations offered here — the linear heap, and a separately loaded battle module whose
  vtables `class_name` would reject — were never needed. Whether the battle's own classes live in
  such a module is still unknown; the text does not.
- The starter selection screen keeps its text in plain StrBufs: "Choose a Pokémon.", "The
  Fire-type Pokémon / Torchic", then "You chose Torchic!". Its Yes/No labels sit beside them.
- Message source text is resident twice over while dialogue runs, once with the player's name
  substituted and once with the `0x0100` name code still in place — spike 11's "already
  substituted" is true of the copy a window shows, not of every copy in memory.

## Spike 46 — the linear heap, swept a slice at a time

2026-09-14 (`src/pa3ds/rolling.py`, run by `spikes/verify_runtime.py` over movie3 slot 4 for 90 s,
silent). The runtime now reads 2,048 pages of the linear heap every 0.25 s, keeping what each page
held when last read, and gives what it finds to the layout, screen text and cursor readers beside
the heap sweep's.

- **The battle's move list was read** for the first time: "Scratch PP 35 35 / ...", along with
  Torchic's level and HP and the Poochyena's level, and each HP change as its row ("16/ 19"). The
  starter choice read under its cursor, "The Fire-type Pokémon Torchic" and each ball after it, and
  the field's location signs were new: "Route 101", "Littleroot Town".
- **Two faults came with it.** What came on screen together was said in build order, which put the
  battle's two "35"s side by side and joined them, "PP 3535 /"; it is now read a layout at a time,
  each in place. And Birch's lines were said twice, as screen text ("Whew...") just before the
  dialogue reader found his box: the rolling sweep found the box's line objects first. Screen text
  now leaves out any TextBox laid out by a message window's own tag processor.
- The battle's command menu (FIGHT, BAG, RUN, POKÉMON) and its move list have no cursor the cursor
  reader recognises, so the moves are said as one line when they come up.
- **Rerun with both fixes:** Birch's lines came only from the dialogue reader. The move list was not
  said at all: movie3 picks Ember within a second of the list coming up, and a layout that has just
  come on screen waits 0.8 s for a cursor, so the list was gone before its wait was over. A player
  choosing a move keeps it up longer; the order within it is covered by `test_announce.py` only.

## Spike 59 — where the runtime's time goes

2026-09-16 (`spikes/60_poll_timing.py`, signs.ctm from its first frame, real speed, 150 s). The
user hears dialogue a second late now and then, and "Littleroot Town" a few seconds into its music,
and asked whether polling at 60 Hz rather than 30 would help. The design has said 60 Hz since the
start; `Settings.tick` said 30. This times every piece of the loop.

- **Polling never reached 30 Hz.** 3,022 dialogue polls in the run, 20.6 a second. A dialogue poll
  itself took 0.65 ms and a screen text poll 5.3 ms, so the rate was never the cost.
- **The sweeps were.** The heap sweep read about 11,000 pages in one go every 2 s, 286 ms each, and
  polling waited for it; with the region probes (71 ms, and 270 ms every 10 s for the linear heap),
  the tree walks of new layouts (58 ms) and the searches (21 ms) it was up to 660 ms. Gaps between
  dialogue polls: median 34 ms, 99th percentile 463 ms, 85 over 250 ms. The linear heap's rolling
  sweep held polling 53 ms every 0.25 s on top.
- **Changed:** sweeps and region probes read 512 pages at a time and poll dialogue, and screen text
  when it is due, between chunks (screen text no longer, since spike 60); the rolling sweep reads
  512 pages every 1/16 s instead of 2,048 every 0.25 s; and the tick is 1/60 s, `--poll-rate 30`
  for a machine that cannot keep up.
  Replayed: 50.1 polls a second, median gap 17 ms, 99th percentile 98 ms, longest 310 ms, 9 gaps
  over 250 ms and none over 500. The same lines were said in the same order, each a little earlier
  than at 30 Hz, and the movie stayed in sync, so reading faster did not slow the emulator down.
- **What is left is finding new things, not reading them.** A message window or layout built since
  the last sweep is not read until the next one finds it, up to 2 s later: the first "wee's House"
  was said after its box had closed (spike 58). The remaining long gaps are the work after a sweep
  (tree walks up to 199 ms, searches up to 65 ms). Neither is a poll rate question. Sweeping more
  often costs reads in proportion; reaching new windows from something already known, without a
  sweep, would remove the wait and is unexplored.
- **Faster exposed one race.** battles.ctm's level up said its stats 1.4 s sooner, and in three
  utterances a screen poll apart — "HP + 2 ATTACK DEFENSE", "+ 1 + 1 SP. ATK + 2 SP. DEF + 1 SPEED",
  "+ 1" — where every run before said them as one. The panel reveals its rows over about half a
  second, and pa3ds now gets there while it does. The race was there before: an earlier Vulkan
  replay caught it too and lost "SP. ATK" outright. Every word is said, in order. Saying a panel
  whole would mean waiting until its words stop arriving, a screen poll more on every screen, which
  is the user's trade to make. menus.ctm replayed the same as its last faithful run.

## Spike 60 — finding a new message window without a sweep

2026-09-16 (`spikes/61_discovery.py`, signs.ctm from its first frame, 75 s, twice). The user put
the perceived lag first. What is left of it is finding things built since the last sweep, so this
logged when a sweep first handed the dialogue reader each window, against two faster routes, and
against a run sweeping back to back.

- **A window names itself in its lines as it starts typing.** Every TextBox a tracked layout holds
  is known (47 on the field), and a message window's line objects carry the tag processor the
  window embeds at +0x9c. Read every 50 ms, that named Mom's window at 21.41 s, 0.84 s before the
  sweep gave it, and the first sign's at 39.75 s, 1.38 s before — the difference between hearing
  "wee's House" while its box is up and after it has closed. Sweeping back to back found them only
  at 21.60 and 39.98, later than the tag in either run, so no sweep schedule beats it.
- **The TextBoxes were known before the windows.** Mom's window typed into `0x14e22818` and the
  town's into `0x14c52b34`, each in a layout already tracked, so the screen text reader, which reads
  every such TextBox, meets a window's lines at its next poll. Whether a later window types into the
  same TextBoxes as an earlier one was not shown here: the town's sign and NPC shared one window.
- **The talk objects of spike 51 come too late to help.** The six `app::tool::TalkWindow` at
  `0x08287a6c` onwards were first found by a sweep at 111 s, long after Mom and the first sign had
  spoken, so they are no route to a first window.
- **Built:** the screen text reader hands on the TextBoxes it sees a window typing into (within a
  screen poll, 0.2 s), the dialogue reader reads those and its own windows' line objects every
  poll, and a window named in one is read at once, from its first page, and kept even if the next
  sweep misses it. Replayed over 140 s: Mom said at 21.32 s where it had been 22.30, the first
  "wee's House" at 39.66 s where it had been 41.18, each within 10 ms of the tag naming its window,
  and everything after unchanged. Both readers now ignore a tag processor below 0x9c: the spike's
  own first run met one, from a torn read, and taking it for a window's read an address below zero
  and crashed. The runtime had the same subtraction.
- **Two faults the faster timing turned up**, replaying battles.ctm and menus.ctm with it:
  - The first battle said "3", the wild Wurmple's level, as it opened. The enemy's HP panel came
    on screen a poll before the player's, and the box is recognised by the player's panel. During a
    battle a level alone on the top screen, in a panel holding a bar, is now left to the battle
    reader too; the party screen and the summary show a level beside a bar on the bottom screen.
  - The options screen's confirmation cursor was read as "FAST ON Yes" before its question. Screen
    text had been read between a sweep's chunks since spike 59, with what the previous sweep found,
    and the list whose labels keep that cursor quiet was in the sweep not yet finished. Only
    dialogue is read between chunks now.
- **Not a fault of this change:** both battles.ctm replays with it missed the command menu's
  first item, "pokémon", which the five before had said. From the state before the first battle,
  twice with it and twice without (`spikes/63_first_cursor_ab.py`), the menu's items were named only
  as the sweeps found their layout files, 3 to 6.7 s after the load: the first item named came at
  4.63 and 4.67 s with the change ("run", "bag") and 4.73 and 4.98 s without ("bag" both times).
  It is the open "first battle's first cursor item late", another thing waiting on a sweep.
- One menus.ctm replay read the options screen's cursor as "POKÉMON TEXT SPEED BAGFAST" as the
  screen opened from the X menu: the X menu's words, kept a poll after they leave, were under the
  new cursor. Once in six replays, and not from anything changed here.
- **Layouts are not helped by this.** "Littleroot Town" came 0.8 s sooner with sweeps back to back
  (19.80 s against 20.63 s): the field's layouts were first tracked at 18.65 s against 20.36 s.

## Spike 61 — what holds a layout work

2026-09-16 (`spikes/62_layout_owners.py`, signs.ctm at 30 s, the heap, the linear heap and the
executable swept once and every word pointing at a work listed).

- **Works live in pools.** 150 works were found, 19 alive, in runs 0x104 apart: 10 live at
  `0x0823a694` sharing first pointer `0x0823d858`; the field's 8 at `0x085fc290` sharing
  `0x08602514`, AreaNav's `0x085fc8a8` among them; one at `0x08c42dac`. The unused slots read
  alive 0 and zeros. Each pool has a table of pointers to its slots just before it, 8 bytes a slot
  (`0x0823a4e8`, `0x085fbf64`, `0x08c42d70`), and a work's first pointer is 0xf0 into a DrawInfo
  that pool shares.
- **Nothing named leads to the pools.** The words before each table name no class, and
  `app::util::G2dLayoutContainer`'s vtable only turned up inside the executable, in literal pools,
  not as a live object.
- **What it could buy.** Once a pool is known, reading its slots' alive bytes each poll would find a
  layout built into it at once. A screen that brings its own pool, as the field does after a load,
  still waits for a sweep to find the pool. Which case "Littleroot Town" is, and how often new
  layouts land in a pool already known, is not yet measured.
- **Battle, the same way** (battles.ctm at 75 s, the command menu up): every battle layout — the
  command menu, its cursor, the HP box, the move list and six more — is in one pool of ten at
  `0x14a9f140` in the linear heap, sharing a DrawInfo at `0x14aa09b4`. The field's pools were still
  there at the same addresses with every slot alive 0. So pools outlive the screens using them,
  and a battle brings one new pool.

## Spike 62 — what the runtime reads, and what each sweep finds that nothing else does

2026-09-16 (`spikes/64_runtime_audit.py`, battles.ctm 215 s and menus.ctm 300 s from their first
frames). The user asked for a cleanup and optimisation pass before more readers are added.

- **Reads per reader.** Every `RpcMemory.read` is a packet answered before the next is sent:

      per call          battles.ctm           menus.ctm           calls
                        reads   pages         reads   pages
      dialogue.poll     12.7    7.6           6.9     3.9         every tick
      screen.poll       45.5    50.9          90.0    90.5        every 0.2 s
      battle.poll       79.7    30.4          50.6    13.4        every 0.2 s
      layouts.revive    32.3    8.8           47.5    14.8        every 0.2 s
      layouts.track     490.9   104.0         579.8   142.7       every sweep
      cursors.track     32.2    20.7          113.3   71.3        every sweep

  "pages" is the distinct 1 KiB pages those reads touched, which is what one pipelined read per
  page would cost; pipelined, a read costs about a third of one sent alone (spike 2).
- **The battle reader reads every StrBuf in the heap on every screen poll**, battle or not: 50
  reads a poll through menus.ctm, which has no battle. Only the ones ending in a gender sign matter.
- **Sweeping for TextBoxes finds nothing the tree walks miss.** Every TextBox whose words were
  reported, 48 in battles.ctm and 148 in menus.ctm, was also found by walking its layout's tree.
  One that is not under a tracked layout cannot be reported at all, since its placement is unknown.
- **The linear heap's rolling sweep matters only in battle.** In menus.ctm all 69 layouts tracked
  had their works found by the heap sweep. In battles.ctm 17 of 53 came from the rolling sweep, the
  battle's pool, and so did the 3 layout files used, of 48 found. Spike 59 timed the rolling sweep at
  a sixth of the runtime's time, and it reads 8,192 pages a second throughout.
- Finding vtables in swept pages: one regular expression per page, or unpacking words into a set,
  was 25% faster than a `find` per vtable per page on 11,000 synthetic pages. Too little to matter
  beside the reads.
- **After the cleanup pass of 2026-09-17** (the structures gathered into `lyt`, `msgwin` and
  `strbuf`; the page cache; the battle reader reading only names each poll; the runtime split into
  discovery, the profile and the schedule), timed again over signs.ctm for 150 s at 60 Hz: the
  readers took 6.4 s including the page cache's fetches, against 20.0 s; dialogue 0.66 s against
  8.54 s; 57.0 polls a second against 50.1; 99th percentile gap 52 ms against 98, longest 165 ms
  against 310, 13 gaps over 100 ms against 69. What is left is sweeping: the rolling sweep 24.8 s,
  the heap sweep 17.9 s, region probes 6.4 s. The replay set said the same things before and after,
  apart from races already seen in earlier runs.

## Spike 63 — what leads to the battle's layout pool and to layout files

2026-09-17 (`spikes/65_pool_anchors.py`, battles.ctm at 75 s, every heap swept once). Towards
removing the rolling sweep, which spike 62 found matters only for these two.

- **A pool's table entry is a handle, and the heap holds them.** Each entry is 8 bytes: the pool at
  +0, the slot's work at +4. The battle's heap objects hold entry addresses: `btl::BtlvUiGauge`,
  `btl::BtlvUiMsgWin`, `btl::BtlvUiTokWin` and `btl::BtlvUiTimer` at +0x14, `btl::app::BattleCursor`
  at +0x0c, `app::tool::MsgCursor` at +0x04, and the battle's own `print::MsgWin` at +0x14. So the
  message window the dialogue reader already reads leads to its layout's pool without sweeping the
  linear heap.
- **Layout files are in archives with an owner.** Each `nw::lyt::ArcResourceAccessor` holds a
  `darc` archive at +0x04 (and +0x20), whose files include the layouts' CLYT and CLAN. The
  accessors are in the linear heap themselves; whether a work or its Layout leads to one is spike 64.

## Spike 64 — a layout pool read from a message window's handle

2026-09-17 (`spikes/66_pool_handles.py`, battles.ctm at 75 s and signs.ctm at 45 s). The same
layout on both, the battle's pool in the linear heap and the field's in the heap:

    message window  +0x14 -> its layout's table entry
    table entry     +0x00 -> the pool    +0x04 -> the work        (8 bytes)
    pool            +0x00 -> the table   +0x04 u16 capacity   +0x06 u16 in use
                    +0x0c -> a work of its own, whose +0x104 every slot's +0x00 points at
                    +0x1c -> the end of the table; the slots start 0x30 after it, 0x104 each
    work            +0xbc -> a resource   resource +0x10 -> the `darc` archive, +0x14 -> its accessor

- The battle's pool holds 24, 10 in use; the field's 48, 10 in use. The table starts 0x48 into the
  pool on both.
- **A work leads to its own layout files**: its resource's archive holds its CLYT. Layout files have
  been found by scanning every swept page for "CLYT"; reading each tracked work's archive once
  finds the file for every layout tracked, in either heap, with no sweep.
- **A message window leads to its screen's pool**, and the battle's is the pool the rolling sweep was
  there for. Reading a known pool's table each screen poll finds a layout built into it at once.
- **Built 2026-09-17** (`Layouts.watch_pools` and `poll_pools`, `LayoutFiles.add_archive`; the
  rolling sweep, the linear heap's region probes and the page scan for "CLYT" deleted). Over
  battles.ctm (`spikes/64_runtime_audit.py`): 54 layouts tracked, 36 from the heap sweep and 18
  from pools; of the 7 layouts whose words were said, 5 came from pools. Reading a pool costs 5
  pages a screen poll. 38 layout files were read from 21 archives, about 5 MiB over the whole
  movie, and 3 were used, as before. Every TextBox said was found by a tree walk; the 7 pieces the
  audit could not place are words read from a picture's texture, which it does not label. Over
  signs.ctm for 150 s at 60 Hz (`spikes/60_poll_timing.py`): 57.9 polls a second against 57.0;
  99th percentile gap 33 ms against 52, longest 158 ms against 165, 2 gaps over 100 ms against 13;
  region probes 2.4 s against 6.4, and nothing left of the rolling sweep's 24.8 s. What remains is
  the heap sweep, 18.0 s including the dialogue polls between its chunks. Battle menus are found
  as the prompt appears: in the battles.ctm replay the first command menu was heard on "fight"
  before the cursor moved, where it used to open on wherever the cursor was by the time the
  rolling sweep came round. The replay
  set said the same things otherwise, apart from races seen before; movie3's state replays drift
  from run to run and are compared by eye.

## Spike 69 — what the heap sweep reads, and where the objects it finds are

2026-09-18 (`spikes/71_sweep_cost.py`, signs.ctm for 60 s and battles.ctm for 120 s). The sweep is
what is left of the runtime's cost (spike 64), and the user asked for it before the races that
remain.

- **It reads the heap whole, every two seconds.** Over signs.ctm: 30 sweeps, 131,302 pages, 7.3 s
  of 60 s. Over battles.ctm: 60 sweeps, 296,332 pages, 17.6 s of 120 s. The heap measures 8,656
  and 11,504 pages.
- **Everything it finds sits in a tenth of it.** Bucketed by 64 KiB: signs.ctm's 349 objects were
  in 15 buckets of 135, battles.ctm's 397 in 18 of 179 — 960 and 1,152 pages against 8,656 and
  11,504. Message windows were in one bucket, the device manager in one, layout works in seven,
  name plates in nine to twelve.
- **Built 2026-09-18.** A sweep reads the spans around the objects found, 64 KiB either side,
  merged and clipped to the heap; the whole heap is swept every 8 s, and at once when a reader is
  waiting for something only a sweep finds — a question's list controller (`after_page`) or a
  battle's gauges (`sweep_wanted`).
- **New memory wants a whole sweep, and that is the one thing the replays caught.** The heap grows
  when a battle starts and the battle's message window is built in the new memory, where no sweep
  had been: "A wild Wurmple appeared!" was gone before its window was found (battles.ctm). The
  heap is measured every 2 s anyway, so a measurement that comes back different now forces a whole
  sweep, and both battles read their opening line again.
- Timed over signs.ctm for 150 s at 60 Hz (`spikes/60_poll_timing.py`): sweeping 8.8 s against
  18.0, polls 58.1 a second against 57.9, 99th percentile gap 31 ms against 33, one gap over 100 ms
  against two. The replay set said the same things, apart from races seen before.
- **Left open: the first battle of a session can still lose "A wild X appeared!"** (the user,
  2026-09-19, once in a playthrough; later battles in the same session were fine). The heap is
  measured every 2 s, so a battle that grows it waits up to that long for the whole sweep that
  finds its message window, and nothing else can find it first: the tag-processor route needs a
  tracked layout and the pool route needs the window. Sweeping the new regions the moment a
  measurement changes is the obvious next move. **Found and fixed in spike 70:** the measurement
  did not change at all.

## Spike 70 — the first battle's window, and new memory the heap measured the same with

2026-09-19 (`spikes/73_battle_growth.py`, battles.ctm from its first frame, the heap probed as
`regions` does, every 30 ms or so, and each new block swept for message windows).

- **The window is built 2.7 s before its first line, and the line is up for 1.5 s.** The battle
  wrote new heap blocks from 51.1 s to 57.2 s. Its window, `0x0855429c`, was built at 53.10 s, in
  the poll its 16 KiB block first read non-zero, holding its designer's placeholder "文字列１行目";
  "A wild Wurmple appeared!" started typing at 55.79 s and "Go! Torchic!" replaced it at 57.24 s.
  So a sweep has about 4 s to find the window.
- **The runtime's measurement never saw it arrive.** `measure_heap` compared the heap's spans, and
  `regions` merges spans less than a megabyte apart. The battle's first blocks joined two spans
  into one between 51 and 53 s; every block after them, the window's included, landed inside that
  span, and the heap measured the same. Replayed through a 2 s schedule at ten phases, four saw
  their only change before the window existed (at 52.4 to 53.0 s) and left it to the whole sweep,
  which comes every 8 s. Later battles build the window where the first one did, and the dialogue
  reader already reads that address, so only the first battle loses its line.
- **Built:** the heap is compared block by block (`snapshot.probe`). A block new since the last
  measurement is swept at once with 64 KiB either side, and stays among the spans read between
  whole sweeps for four measurements, because an object can be built after its memory is first
  written. A new block no longer forces a whole sweep: at a third of the measurements over
  battles.ctm from 30 s to 75 s some block was new, which would have doubled the sweeping.

- **The replay never loses the line, before or after** (`spikes/74_first_battle_ab.py`, 20
  replays of battles.ctm from its first frame, the runtime started 0, 0.4, 0.8, 1.2 and 1.6 s
  after the boot). "A wild Wurmple appeared!" was said in every one: at HEAD, with the change, and
  with each of those reading 1 KiB around every object found instead of 64 KiB, which takes away
  the luck of the window being built beside something already known. The window was tracked 2 to
  3 s before the line throughout, and the change moved that earlier by about a second. So the miss
  the user hit once on 2026-09-19 is not in this replay, and the change is reasoned rather than
  proven: it takes the first battle's window out of the hands of a whole sweep that comes every
  8 s and gives it to the next sweep, whatever the measurement's phase.

## Spike 99 — chains from the executable's globals, and the app that is running

2026-09-25 (`spikes/99_static_roots.py`). The runtime finds what it reads by sweeping the heap
for vtables every couple of seconds, and a screen's objects wait for the next sweep. The
executable is mapped at a fixed base, so its globals are at fixed addresses: sixteen captures of
the executable and both heaps (movie5 twice, at the field, the clerk's greeting, the Buy list, the
Yes/No, the Bag, the X menu and the Pokédex; movie4's field and Wally's battle), walked back from
every object of a class the runtime sweeps for to a word in the executable's data.

- **Roots are the writable data and .bss only**, 0x005ec000 to 0x006ae620 in Omega Ruby, from the
  code set info of the ROM's extended header (text 0x00100000, rodata 0x0057a000, data
  0x005ec000, .bss after it). Words in the code read as pointers now and then and made false
  roots until then; code modules' own data comes and goes with the module and is left out.
- **One global leads to the game's managers.** `[0x0062f7c4]` is an object with no vtable whose
  +0x14 is `gamesystem::GameProcManager`, at the same address in all sixteen captures of both
  runs. On the field, the same global leads to the map blocks, the move models, the choice box's
  device manager and the party's records, three to five hops down.
- **The manager's +0x10 follows the running app.** It points at a word inside the app's process
  object, and every app so far was built at one address, with its class at the start:
  `field::FieldmapProc` (+0x64) on the field, `app::bag::Proc` (+0x58), the Pokédex's
  `app::sango_zukan::ZukanProc` (+0x150), and Wally's battle `app::capture::CaptureDemo` (+0x90).
  Each process's +0x1c points back at the manager, and it holds its app's parts: the Bag's
  `app::bag::MainSystem` and the battle's `app::capture::Main` at +0x24, the Pokédex's
  background, messages and data from +0x9c.
- **The field's layouts follow one recipe:** `[0x0062f7c4]` +0x48 +0x19c, then records 0xf4 apart
  from +0x164, each leading to a slot of the layout pool, whose +0x4 is the layout's work. It reaches
  the ten layouts of the field's first pool.
- **Live layouts reached from any global within six hops:** field 18 of 19, the Mart 17 of 21, the
  Bag 4 of 6, the X menu 1 of 6, the Pokédex 1 of 12, the battle 4 of 10 to 13 — the same in both
  runs, whose heaps held within a few words the same number of pointers. The apps' own layouts
  are the gap.
- **A pool's layouts link to each other**: each work's DrawInfo +0xac and +0xb0 point at its
  neighbours', so from one layout every other in its pool follows.
- **A message window is held only by its own fields and by the TextBoxes it types into** (+0xf4,
  at the window's tag processor, +0x9c): reached through its layout, as the runtime already does
  (spike 60).
- **What it could buy.** One read a poll of the manager's +0x10 says a new app has started, and
  the class at the start of its process says which: the moment to sweep for its layouts, cursors
  and windows, where now a new screen waits up to two seconds for the schedule. From the process,
  its parts; a recipe from each app to its layouts is the next step.
- **Caveats.** The walk takes the nearest address anything points at, at or below a field, for
  the object the field is in; some chains it found run through a file-read manager and a thread
  and mean nothing. Apps' classes live in the code modules loaded for them, so their vtables are
  outside the executable's rodata, and a census of the executable's classes does not list them.
- **Built 2026-09-26, and measured against main** (`spikes/regress.py run`, movies 1, 2, 4 and 5
  at real speed, each replay's sweeping as `summary.txt` records it). Sweeping went from 15% of
  the runtime's time to 7%:

  | Movie | Length | Main: sweeps, seconds | Now: sweeps, seconds | Polling gaps over 100 ms |
  |---|---|---|---|---|
  | movie1_opening | 1,376 s | 729, 187.1 s | 392, 70.4 s | 372 → 120 |
  | movie2_route103_and_back | 696 s | 390, 120.6 s | 253, 61.5 s | 265 → 138 |
  | movie4_meeting_wally | 316 s | 182, 47.6 s | 156, 27.7 s | 99 → 64 |
  | movie5_menus_and_shops | 451 s | 237, 67.0 s | 200, 48.2 s | 150 → 94 |
  | All four | 2,839 s | 1,538, 422.3 s, 7.25 M pages | 1,001, 207.8 s, 4.05 M pages | 886 → 416 |

  Movies 1 and 4 were measured before the app's first sweep moved a tenth of a second on, which
  cost movies 2 and 5 under 2%. Movie5 drifts the same way on both (out of sync 17669), so it is
  compared up to there.
  What now asks for a sweep, and what the schedule under it is:
  - **An app starting** (`Gen6Profile.poll_app`): the whole heap a tenth of a second on, once the
    screen has been read, and again at half a second and a second and a half.
  - **The screen starting to change** (`Gen6Profile.screen_changed`): a watched pool's count of
    slots in use moving, or every word of a layout gone from the screen, sweeps the whole heap
    half a second later. What the next screen is read by can be where no pool leads: without it
    the battle's Bag was silent for four seconds and the save's question went unread.
  - **A page that asks something** sweeps the whole heap a second later, and around what has been
    found three times more while its prompt may be coming; **any other page sweeps nothing.** A
    whole sweep a second after every page was most of main's sweeping.
  - **Every layout's pool is watched** from the work's first word, so a layout built into one is
    taken in at the next screen poll, and a battle whose gauge is missing asks again every fifth
    poll.
  - **The schedule**: around what has been found every 6 s, the whole heap every 24 s, where it
    was 2 s and 8 s.
- **The spans had grown to three quarters of the heap.** Spike 69 measured them at a tenth. The
  hidden Pokémon's record (2026-09-22) brought the game's heaps, `gfl::heap::CTRExpHeap`, into the
  sweep: 33 objects, each at the head of its memory, whose spans alone covered 4,711 of the 8,656
  pages in movie4's field capture, and a sweep of the spans cost 0.23 s against a whole sweep's
  0.27 s. With no span drawn round them (`UNSPANNED`) the spans cover 3,396 pages there, 3,261 of
  9,792 in the battle's Bag and 3,897 of 11,616 in the Pokédex, and a sweep of them costs 0.12 s
  against 0.30 s.
- **What was said.** Every dialogue page main said, and every speaker's name, was said in all four
  movies. What changed, beyond the races two runs of main disagree on:
  - The save's question is read at the end of every movie, as on main, and its YES in three of
    four; movie2's is answered within 1.3 s, and main misses it too. Before the screen-change sweep
    this branch had lost movie1's question altogether.
  - The battle's Bag is followed by its cursor pocket by pocket, and the move list is no longer
    read out whole before the cursor lands on it: a layout built again where it was is the one
    that was there, and a menu stays a menu (`Layouts.buried`).
  - A list opening on words its cursor has just said stays quiet, as a choice box already did:
    with the list found later than its cursor, the save's YES had been said by both.
  - The Pokédex reads the entry it opens on, Torchic's, before the player scrolls to Mudkip's, and
    the Bag's KEY ITEMS pocket is said as KEY ITEMS and Nothing, where main said the POKÉ BALLS
    pocket's Poké Ball over it.
  - Lost: the pocket step of Wally's catching lesson, POKÉ BALLS, which the game picks for him,
    and a cursor passing through "bag" for a fifth of a second.
- **New screens are heard sooner.** From an app starting to its first words: the title menu 0.2 to
  0.4 s against 0.8 to 1.8 s on main, the Bag 0.6 s against 1.3 to 1.7 s, the party 0.7 s against 0.9
  to 1.8 s, a Pokémon's summary 1.4 s against 2.5 s. The X menu back after the Bag closes is as
  quick as on main, 0.6 to 1.2 s; the place's banner after a battle is 0.2 to 0.4 s later, at
  about 2.4 s, because the field's rebuilding asks for sweep after sweep and the screen is not
  read during one.
- **Left open: the heaps themselves.** The 33 `gfl::heap::CTRExpHeap` are the game's allocators.
  If one read of each can say that something was allocated, and where, a sweep could read only
  what is new, and the schedule could go. Spike 110 found it could, and its watch of the heaps
  replaced the schedule, the spans and the sweeps after a question or a change of screen before
  any of this was merged; what outlived it is the sweep after an app starts, every layout's pool
  watched, the gauge asked for again, and the fixes to what was said.

## Spike 110 — the game's heaps, and what they have just allocated

2026-09-26 (`spikes/110_heap_allocators.py`: `tree` and `diff` over spike 99's captures, `watch`
live over movies 4 and 5). What the runtime spends is sweeping, and every object a sweep finds is
one the game allocated. The game allocates through `gfl::heap::CTRExpHeap`, which the hidden
Pokémon's reader already sweeps for, and each wraps the SDK's expanded heap.

- **The heap object holds the SDK's head at +0x38**, and the head keeps two lists of blocks:

      CTRExpHeap  +0x00 vtable    +0x04 the heap's id    +0x08 blocks in use
      head +0x00 'EXPH'           +0x04 -> previous sibling's head   +0x08 -> next's
           +0x0c -> first child's head   +0x10 -> last child's   +0x14 u16 children
           +0x18 start   +0x1c end
           +0x24 -> first free block     +0x28 -> last free block
           +0x2c -> first used block     +0x30 -> last used block
      block +0x00 u16 'UD' in use, 'FR' free   +0x04 size   +0x08 -> previous   +0x0c -> next
            +0x10 the game's own header: the heap's id, then its length, 0x20
            +0x30 the object

  The count at +0x08 matched a walk of the used list in all 99 heaps of two captures whose pages
  were all there.
  Every message window, name buffer, layout work, heap and Pokémon record in the captures starts
  at its block's +0x30; the talk windows and the camera sit deeper in blocks of their own.
- **One heap leads to all of them.** Siblings and children are linked through their heads, and
  the root heaps to each other: 72 heaps over movie4, under two roots in the application heap
  (0x08001224–0x08803000 and 0x08803000–0x08df0000) and trees in the linear heap. A layout work is
  a block of its own in an app's heap.
- **A new block goes on the end of its heap's used list.** Between two moments of one run, every
  new block came after every block that stayed: 244 of 244 in the battle, 37 of 38 as its Bag
  opened, 405 of 405 from the Mart's greeting to its list, 5 of 5 to the Yes/No. A block resized
  where it is keeps its place: a child heap shrank from 3.9 MB to 2 MB between the field and the
  battle. Watched live over movie4, the walk forward from the old last block found the new ones
  1,088 times; 233 times the old last had been freed and the walk went back from the new last
  instead.
- **Every object the sweeps found was in a block the watch had seen appear.** Heads read and new
  blocks walked while the whole heap was swept every 2 s, as main did:

  | Class | movie4, heads at 60 Hz | movie5, heads at 5 Hz |
  |---|---|---|
  | layout works | 308 of 308 | 367 of 367 |
  | name buffers | 143 of 143 | 384 of 386 |
  | list controllers | 1 of 1 | 4 of 4 |
  | message and name windows | 5 of 5 | 4 of 4 |
  | move models | 32 of 32 | 32 of 32 |
  | talk windows | 12 of 12 | 20 of 20 |
  | heaps | 44 of 46 | 44 of 52 |
  | Pokémon records | 2 of 2 | 2 of 3 |

  The block appeared a median second or two before the sweep found what was in it (1.5 s over
  movie4, 1 s over movie5), and never less than 0.17 s before. What was missed was mostly heaps:
  the watch takes a heap it has just adopted as it finds it, so what it held before the next look
  never counts as new; that is the likely reason, not checked.
- **An object is built by the time its block is seen.** 23,016 of 66,623 new blocks over movie4
  named their class at +0x30 the poll they appeared — panes, materials, TextBoxes, windows, 3D
  resources; looked at again for two seconds, eleven more did. The rest hold no class: buffers.
- **Watching costs little.** Over movie5 at 5 Hz, reading each block once: 7.7 s of 450 for 2,169
  looks at 73 heads and 92,057 new blocks, walked in 23,104 pages and named by vtable, against
  92.9 s for the 224 whole sweeps. At 60 Hz, looking again at every block that named nothing, the
  same watch took 44 s of 316 over movie4.
- **What it could buy.** Most of the sweeping. A new object would be found the look after it is
  built, by reading a few headers, where a sweep reads the heap for it, and the question and
  screen-change sweeps of spike 99 would have nothing left to wait for. The heaps' own extents
  could stand in for probing where the heap is, which costs as much as sweeping its spans (spike
  99).
- **Left open.** A reader wants some objects that are not at +0x30 (the talk windows' pool, the
  camera, map blocks inside larger blocks), so a new block of a class it wants may still need its
  memory read. Readers take a sweep's whole `Found` and several replace what they know with it, so
  handing them only what is new needs each one's `track` to add rather than replace. The linear
  heap's trees read broken where a capture held only some of their pages; live, they walked.
- **What the watch missed, and why** (the user, 2026-09-26: find what it misses). Each miss the
  strict watch or the runtime showed had one cause:
  - *Blocks taken from the top of a heap go on the front of its list*, not its end: the Pokémon
    record 0x10 bytes from its heap's end went unseen until the front was watched too (72 times
    over movie5).
  - *A heap torn down and made again where it was between two looks* leaves its parent's count
    and last child as they were: the field's heaps, rebuilt as the Bag closed. A heap's children
    are listed again for five looks after its blocks change.
  - *A heap object is built after its block appears* (up to 0.6 s), so it is adopted from its
    parent's list of children, not from the block.
  - *A block freed and taken again where it was looks exactly as it did*, and the game does it
    all the time: the save's three layout works went into the very blocks the X menu's five had
    just given up, and the walk stopped at the first as a block it knew. The game's own count of
    blocks in use (`CTRExpHeap` +0x08) is read with the head, and whenever a heap changes, the
    sixteen blocks at each end of its list are looked at and handed over where their first 0x100
    bytes no longer read as they did (0x40 was too few: a work built for another layout differs
    at +0xbc, its resource). Handed over whatever they held, the ends swept a third of the heap
    again and doubled what finding things cost. Looking at the ends only where a walk could have
    missed a block (one put at the front, a last block freed, a count that moved with both ends
    where they were) was tried, and taken back while the level up's panel was missing; that turned
    out to be a pool's slot built into again (below), so whether the narrower look misses anything
    was never measured.
  - *A pool is taken whole and built into after*: the move models' pool, the talk windows', the
    camera's. New blocks are swept again half a second and two seconds on.
  - *Layouts are built into pool slots, not allocated*, and a known slot is built into again: the
    level up's panel went into a slot of the battle's pool, in the linear heap, that the layout
    reader already knew. Each known work's layout pointer is read at every screen poll, a work
    found before its layout is built waits for it for a minute, and a pool found in new memory
    is watched before any layout goes into it.
  - *A door builds the next zone's talk windows into memory taken as the field loaded*: Mom's
    first pages in the house had no name. The speaker reader asks for a whole sweep when the field
    shows a page no talk object it keeps is showing; swept for two seconds after every door
    instead, movie1's replay went its own way each time, the sweep landing while the zone loaded,
    and resting 20 ms between that sweep's chunks did not change it.
- **Built 2026-09-26** (ADR-0034): the runtime looks at every heap in the application heap ten
  times a second, sweeps the blocks new since the last look, and sweeps the whole heap at the
  start, 1.5 s after an app starts, when a reader asks, and every 30 s as a net that logs what it
  found the watch had not. Measured with `spikes/regress.py run` over the four movies, finding
  things cost:

  | Movie | Main | After spike 99 | Now |
  |---|---|---|---|
  | movie1_opening | 208.5 s | 91.9 s | 46.1 s |
  | movie2_route103_and_back | 131.3 s | 72.2 s | 43.7 s |
  | movie4_meeting_wally | 52.5 s | 32.6 s | 20.5 s |
  | movie5_menus_and_shops | 73.9 s | 55.0 s | 32.2 s |
  | All four, 2,839 s of play | 466 s | 252 s | 142.5 s |

  Sweeps, looks at the heaps and measurements of where the heap is, together: 5% of the time,
  where main spent 16%. Of the 142.5 s, whole sweeps were 51.7 s, sweeps of new blocks 50.7 s,
  the looks 31.7 s (the ends of every changed list, read each tenth of a second, are most of that),
  measuring 5.0 s, and a list's own sweeps and code loaded since startup 3.4 s. Pauses in polling
  over 100 ms went from 886 on main to 264.
- **Five looks a second were tried, and left.** With `heap_tick` at 0.2 s (the user, 2026-09-26),
  finding things cost 129.7 s over the four movies against 142.5 s, and the looks 24.4 s against
  31.7 s: half the looks, but each walks and compares twice what changed, so they cost more each.
  Every dialogue page and speaker's name was said in the three movies that followed, and the net
  caught what it catches at ten (movie2: 19 catches against 20, the same classes). But a menu is
  found later as it opens, and its words were read out whole before its cursor was known: the
  battle's move list both times it opened in movie4, where the run at ten looks read it neither
  time and main the first time only, and the X menu once in movie1, never seen in twelve runs at
  ten. About 13 s of 2,839 was not worth menus read out whole.
- **What is said.** Movies 2 and 4, which followed their recordings, say everything main says
  but for races two runs of main disagree on, and some things main did not: the save's YES, the
  move list no longer read out whole the first time it comes up, the Pokédex entry the player
  opens on. Every dialogue page and speaker's name main said is said (movie1 177 and 81, as on
  main, on the run that followed). Left: the party screen the battle's Bag shows after USE, up
  for two seconds, is not read; main read it. The net still finds name buffers, map blocks,
  Pokémon records and now and then a list controller the watch did not deliver, which is what its
  log line is for.
- **Replays are touchier.** With the heads read ten times a second, movie1 and movie4 went their
  own way on some runs with nothing out of sync, where main's never did: the reads shift how long
  the emulator charges for the files the game loads, and the random numbers come out otherwise (a
  lent Zigzagoon with 22 or 23 HP instead of 24). It changes nothing for a player; a run that did
  it is flagged ("WENT ITS OWN WAY") and says nothing about the code, and is run again.

## Spike 111 — Pokémon Sun: its text, its dialogue, and the memory it may be read in

2026-09-26 (`spikes/111_sun_first_look.py`). The user: pa3ds "instantly crashes" on Sun. It did
not crash. Sun (title `00164800`) was not in the catalog, so it was read as ORAS, the executable
has neither of ORAS's message window classes, and `_session` logged "no message window classes"
and returned without a word spoken. sun_1.ctm, the user's recording of a new game from the
language choice to the player's bedroom (385 s), replayed three times with no out-of-sync line.

- **The ROM's text is ORAS's format.** Every one of Sun's archives is GARC version 0x0600 (ORAS's
  and X's are all 0x0400), and `garc.garc_files` reads a 0x0600 archive's data offset from
  +0x18, its largest file's size; it is at +0x10 in both versions. Read from there,
  `message_lines` reads every file unchanged: system text `a/0/3/0`-`9`, story text
  `a/0/4/0`-`9`, ten languages. "Hang on, now. Gimme just a sec..." is system text file 34 line 1.
  Not fixed yet: nothing read in Sun needs the ROM.
- **The executable has RTTI under gfl2's names**, 2,337 classes by the field, and most of the game
  is in 115 CROs loaded into the code span as they are needed. (A census that took the NUL between
  two names as part of the first found only half of them.)
- **Its memory is elsewhere.** `HEAP_SPAN` holds a main heap of 2.8 MB and `LINEAR_SPAN` nothing;
  everything read below is in linear memory from `0x30000000` to about `0x342DC000`, outside
  `FULL_SPAN` too.
- **A message window is ORAS's under other names.** `print::MessageWindow`, dropping to its base
  `gfl2::str::StrWin` once closed, holds its line records at +0x04 (0x68 apart), its message at
  +0x08, its line count at +0x20 and its tag processor at +0x9c, as `print::MsgWin` does; a closed
  window's TextBoxes read as `nw::lyt::Pane`, as in ORAS. `gfl2::str::StrBuf` is ORAS's shape.
  **A TextBox holds every field 0x30 before ORAS's**: text +0xa4, tag processor +0xc8, capacity
  +0xcc, length +0xce. Mom's scrolled page: "It's so warm and bright here!" typed into one box,
  "The first day spent under Alola's sun!" left in the other, and the message's page breaks are
  ORAS's 0xBE00 and 0xBE01.
- **Sun names its speakers.** `App::Tool::TalkWindow` +0x60 is an `app::util::G2DUtil` whose
  +0x34 is the message window, at all four moments a talk window was up, and +0xa8 is a StrBuf
  holding the name on the plate over the box: "Mom", and empty through Kukui's video call. A talk
  window lasts from one message to the next.
- **Sun names its speakers' windows through a util that outlives them.** The field's
  `app::util::G2DUtil` was there after Mom's window had closed, its +0x34 back to zero; each of
  the four talk windows' +0x60 was the util whose +0x34 was its message window.
- **Reading the GPU's memory takes the emulator down.** Azahar writes a buffer the GPU cleared
  back to memory when anything else reads it, and asserts (`DownloadFillSurface`,
  `rasterizer_cache.h:1172`) in code both renderers share. On Vulkan, probing the address space
  died past VRAM at 8 s (`0x1F5FC000`) and at the top of the linear memory at 40 s (`0x342D8000`).
  An OpenGL capture run then left the fork's config marking the renderer as default, and Azahar's
  default on Windows is OpenGL, so the next five runs, meant for Vulkan, ran OpenGL; the fork's
  log says which started (`Renderer_GraphicsAPI`). On OpenGL: sweeping the linear memory whole
  every 3 s died at 189 s (`0x33430000`).
- **The heap table.** The start of the application heap is a table of gfl2's heaps, eight bytes an
  id, each a `gfl2::heap::CTRExpHeap` whose +0x04 is its id and +0x38 the head of the `nn::fnd`
  heap it embeds: 'EXPH', +0x08 the next sibling's head, +0x0c the first child's, +0x18 and +0x1c
  the range, +0x2c the first used block. A used block is a 16-byte header ('UD', +0x04 its size,
  +0x0c the next) and what it holds. On OpenGL, sweeping every heap the table lists died at 268 s
  (`0x333F4800`), in a heap the move to the house built and took down again.
- **Every message window, talk window and TextBox was under heap 5 or heap 0x0b** at 38, 78, 118,
  125, 300 and 392 s: heap 5, 38 MB, from the start, and 0x0b, 4 MB, which the field builds.
  Sweeping just those two every 2 s ran the whole movie on OpenGL (120 sweeps), and **died on
  Vulkan at 277 s** (`0x31C40000`), inside a 529 KB block of heap -24, under heap 5: one of 16
  allocations in a heap of 11 MB.
- **The dialogue's objects are small allocations.** Walked from heaps 5 and 0x0b through the heads
  and the used blocks, the 300 s capture holds 6,294 blocks; the 6,120 of 4 KB or less come to
  1.4 MB and hold every message window (0x114 bytes a block), talk window (0xe4) and TextBox
  (0x120), and none of the places the emulator died. ADR-0035.
- **Built:** `games.sm`, Sun and Moon's profile: dialogue through ORAS's reader given Sun's classes
  and TextBox fields, the speaker by its plate, windows taken in from their util the poll it points
  at them, and sweeps of only the small blocks under heaps 5 and 0x0b (`sm.heaps`), read to the
  byte (`Discovery`'s `where`, `Rpc.read_each`), with Sun's readers reading to the byte too.
- **The language choice** (sun_2.ctm, the user scrolling the list both ways and moving between
  the confirmation's two choices; `watch`, which polls one word and takes a picture each time it
  changes). `App::LangSelect::LangSelectUpperDraw`, in the LangSelect module at `0x006b2700`, past
  where the executable ends: +0xb4 is the screen, 0 for the list and 1 for "Selected language";
  +0x12c the language under the cursor, 0 English to 8 繁體中文 in the list's order, a step each
  press and matching the language by the arrow in every picture; +0x12d the confirmation's choice,
  0 "Begin game" and 1 "Select language". The list's buttons are pictures, and no word of them is
  in memory; the game's own words for them are the English names in system text file 51, lines 8
  to 16 in the list's order, which "Selected language" shows the choice by. Lines 4 and 5 are the
  titles, 17 and 18 the confirmation's choices, 1 and 2 the bottom screen's instructions.
- **An app's classes arrive with it.** The code span measured as pa3ds attached ended at
  `0x00688000`, so `ClassNames` over it could not name the choice's objects, and learning classes
  only at a whole sweep said the list 7 s after it came up. Sun's names read the whole code span,
  and learn at every sweep: the list was said the poll it came up, before the first press.

## Spike 116 — Where a Sun sweep's time goes

2026-09-26 (`spikes/116_sun_walk_cost.py`). The user asked for the experience to be smoother
after sun_4.ctm, their walk from the house to Iki Town. On Route 1 every sweep walked 13,000
small blocks, 2.9 MB, every 2 s.

- **A read costs about 28 us and a round of reads about 140 us more**: a lone read took 150 to
  300 us. The walk read each block's header, a round a block along each heap's list: 13,600
  reads in 4,221 rounds, 620 ms. The sweep then read the bodies: 14,000 reads, 450 ms. Nothing
  but dialogue was polled through either, so a key, a menu's cursor or the field waited up to
  1.1 s of every 2.
- **One heap's list was the rounds.** Heap 0x21 is the field's, and its own 4,219 blocks were the
  rendering engine's: animation, texture and material nodes, textures, command buffers. Over 58
  of spike 112's captures (menus, the language choice, the keyboard, the bedroom), none of
  pa3ds's classes was in one; the panes, TextBoxes and windows were in 0x23, the apps' heaps and
  heaps with no id of their own. The field itself, which has no class, is there: it is read
  through its block's own header (ADR-0035).
- **Reading each body with the header after it** made the sweep the walk: 15,600 reads, 850 ms.
  Leaving 0x21's own blocks out: 10,600 reads in 1,849 rounds, 500 to 650 ms, and the same 96
  objects found. Walking each list from both ends as well (its last block is the head's +0x30,
  a block's previous +0x08) halved the rounds and took as long, since a body read going
  backwards shares no read with a header.
- **Keys, screens and the field are polled between the walk's chunks**, as dialogue is: the
  longest gap between two polls through a walk was 20 to 140 ms.
- **Replayed**, sun_4.ctm said what it had said, with the speaker's name on three more of
  Kukui's pages, where the plate had been heard after them.

## Spike 123 — Sun's heaps, watched as ORAS's are

2026-09-27 (`spikes/123_sun_heap_watch.py`: `heads` on sun_4.ctm's Route 1 state, `watch` over
sun_4.ctm and sun_3.ctm from their first frames). The user, after main's heaps' watch (spike 110,
ADR-0034): make Sun follow it. Sun walked every small block under its dialogue's and field's heaps
every 2 s, 27 s of a 200 s replay.

- **gfl2's heads are ORAS's.** All 169 heads under the walk's roots are `gfl2::heap::CTRExpHeap`,
  the last child at +0x10 and their count at +0x14 in every one, the last used block at +0x30 in
  all 95 with any; the object's +0x08 counts its blocks in use in 169 of 169, as ORAS's does. The
  heap objects are at 0x3000010c to 0x34296650. `GameSys::GameProcManager`'s +0x10 moves when
  another app starts (1.9 s, 8.4 s and 20.4 s over sun_4, the last the field) and reads zero in
  between.
- **Watched beside the walk**, a look ten times a second at the heads, each new small block read
  to the byte and again 0.5 s and 2 s on: of about 800 objects the walks found over sun_4, the
  watch found all but 3 first, most about a second sooner. The looks took 7.4 s of 200 against the
  walks' 26.4 s; each head read alone, they had taken 15.3 s. Four things stood in the way first:
  - *A heap made between two looks beside another was taken as known*, through its sibling's link:
    the actors' heap, as the field loaded, had none of its blocks handed over. Main's watch did
    this too.
  - *Heap 0x21 is made after the watch starts*, so it had been watched whole, 4,219 blocks; it is
    told by its id as it is adopted.
  - *A count grown past what the walks found*: Route 1's second zone put 86 actors into blocks
    freed a minute before, and the walk back from the new last block stopped at the first it knew.
    The whole list is looked over then.
  - *An object built into its block after the class that names it is loaded* was swept before the
    class was known: the field's classes come with its module, learned when the app starts.
- **Run as the runtime** (`regress.py`, against the walk every 2 s): sun_4 said its 23 pages and
  one more, speaker names 18 and then 20; sun_1 its language choice, keyboard and 31 pages; sun_3
  its menus, cursors and save prompts. Finding things over sun_4 took 3.8 s of sweeps and 7.1 s of
  looks, polls inside them included, against 27.2 s; over sun_1, 5.3 s and 4.1 s against 34.5 s.
  Pauses in polling over 100 ms: 8, 13 and 7 against 8, 7 and 3. The replays found what the walk
  had been covering for:
  - *Sun's screen poll never tracked a cursor* taken in between sweeps, as ORAS's does, and the X
    menu's and the save prompt's Yes and No were read out whole.
  - *A layout object is made before its root pane*: taken in then, the save prompt's cursor had a
    root of zero, and was never taken in again. A work now waits for its root.
  - *A layout taken in as its work is allocated has no panes yet*: walked empty, it was never
    walked again. It is walked again on its next five screen polls.
  - *A block taken over by a larger one and then built again as it was* went unseen, its panes out
    of bounds, until the watch forgot every block taken over.
  - *An area load allocates thousands of blocks*, their headers read one at a time: a look took
    0.75 s with nothing polled, and pauses over 100 ms went to 39. A look polls every 256 headers.
- **Left open.** A screen poll was 21.6 s of 120 over sun_4, 12.5 s of it `Layouts.revive` looking
  again at the 128 works whose layouts are gone, a read at a time. ORAS's `pc` said the same with
  the mended watch, its looks 0.3 to 0.5 s where main's were 0.7.

## Spike 60 against Sun — where a Sun poll's time goes, and what fetching ahead buys

2026-09-27 (`spikes/60_poll_timing.py` over sun_4.ctm for 120 s, now timing Sun's polls, looks and
walks too; `spikes/regress.py` over sun_1.ctm and sun_3.ctm). Issue 4's lag items: Sun's readers
sent each read alone where ORAS's come from a page cache, screen polls were 21.6 s of 120 under
the walk, and a look after an area load read thousands of headers one at a time.

- **What a poll costs under the watch, before any change**, over 120 s: dialogue polls 8.5 s
  (6,652 of them, 1.3 ms each), screen polls 7.0 s (561, 12.5 ms each, 3.3 s of it
  `Layouts.revive` over the works whose layouts are gone), looks 3.9 s (1,090, the worst 1.6 s),
  field polls 1.6 s (the worst 0.8 s, the listing after a sweep), walks 1.3 s (7). So the screen
  poll is a third of what it was under the walk, and `revive` a quarter.
- **A poll's reads fetched ahead** (`memory.ExactCache.phase`, the reads a poll made last time
  fetched in one burst as the next of its kind starts, kept to the small blocks that may be read
  now): dialogue polls 8.5 s to 6.1 s and their reads 6.1 s to 2.4 s, the field's the same or
  less, over five runs of the movie. The gain holds on every run; the rest moves from run to run
  by as much (`revive` 1.9 to 4.3 s over five runs of one configuration).
- **The X menu's lost cursor was the watch's, not the fetching ahead.** The screen poll was first
  left live because, fetched ahead, the X menu's cursor went unfound at one of its openings over
  sun_3.ctm. Twenty-eight replays later: the fits were the same on every run, and the opening
  after the save prompt was missed one run in four whether the screen poll was fetched ahead or
  not (the live control run `base` missed it too, and so did one from before any of this). The
  cursor's layout object was in a block the save prompt had taken, freed, and the field then took
  again as it was. A walk that no longer found the block dropped it from `SmallBlocks` without
  telling the watch, so the watch still knew it and, its first 0x100 bytes the same as before,
  never handed it over; the save prompt's block left over it in `SmallBlocks` ended before the
  layout object did, `InBlocks` read the object as zeros, and the cursor waited up to 6 s for the
  next walk. `SmallBlocks.walked` now returns the blocks it no longer found and the watch forgets
  them: six replays, every opening. The screen poll is fetched ahead since: 8.8 s to 4.8 s over
  sun_4's 120 s, `revive` 4.3 s to 1.6 s, twice.
- **Still open: the first opening's cursor is one look late** at 2 of 26 replays: the X menu's
  layout is handed over a look before its cursor's, and "SAVE OPTIONS" is said as screen text
  before the cursor is known to be over SAVE.
- **The looks' header walk is serial.** Each header's address is in the header before it, so a
  walk of thousands cannot be pipelined; the marks of the blocks found can be, and are
  (`heaps.Heaps`, `read_each`), thousands in one burst where each was a round trip. Measured, it
  bought nothing visible: pauses over 100 ms 11 to 13-17 over sun_4's 120 s across four runs, 17
  to 13-17 over sun_3, 31 to 32-37 over sun_1; the worst look 1.6 to 2.4-2.8 s. The variance
  between runs of one movie is as large as any change. A burst's reads are not kept for the next
  phase: fetched ahead, thousands of them cost a look more than they answered.
- **Most pauses are not the looks'.** Spike 60 now says, for each pause over 100 ms, which pieces
  ran in it. Over sun_4's 120 s, 8 of 13 were `Discovery.learn`, 120 to 290 ms before each
  whole walk, reading the code for classes loaded since; 2 the sweep of the blocks an area load
  had just allocated, and the tracking after; 1 the field's listing after a sweep, 430 ms, run
  from inside a look's call back. Over sun_1's 390 s, 21 of 33 were `Discovery.learn`, up to
  1.7 s as an area loaded. The header walk held up none. What a look did hold up: its burst of
  marks, 146 and 130 ms over sun_1, now read `BETWEEN_READS` at a time with a call back between;
  and, as the area at 266 s loaded, 1.3 s of walking in which the emulator answered a read in
  5 ms, so 256 headers never came, now called back every `BETWEEN_SECONDS` (50 ms) as well. Over
  sun_1 after both: no pause over 1 s where there was one of 2.2 s, and 27-29 over 100 ms, 18 of
  them `Discovery.learn`. Capping how far a look walks, finished at the next, was the plan, and
  would have bought nothing: the walk was never what held polling up.
- **Spike 64 was not run against Sun**: it counts a reader's reads through the page cache and
  labels what a heap sweep, a layout pool or a tree walk found, and Sun has none of the three
  as ORAS has them. Spike 60's per-piece totals above are its "reads per reader" answer.

## Spike 60 over sun_10.ctm — the code learned from, and the field listed, in bursts

2026-09-27 (`spikes/60_poll_timing.py` over sun_10.ctm for 152 s, Kukui's catching tutorial on
Route 1; `spikes/regress.py`, pinned). Two pieces held polling up most over the movie.

- **`Discovery.learn` searched the whole executable again for every class**, three passes over
  about 6 MB a class, before each whole walk and at each app's start: 13 times, 2.85 s, 100 to
  290 ms each with no poll between. The executable's vtables were found as it was read, and a
  module's class keeps its name, typeinfo and vtable in the module. It now searches only the code
  past the executable, and not at all when that code reads as it did last time: 0.68 s, the worst
  96 ms.
- **The field's listing on arriving on Route 1 took 1.35 s**, called back from inside a look while
  the area loaded, when the emulator answers each read in about 5 ms: a read of each actor and
  model, one after another. Each actor's and model's head is now fetched in one burst
  (`SunFieldReader._fetch`), and `ExactCache.read_each` keeps a burst's replies for the rest of its
  phase; a key press is a phase of its own for it. The listing: 1.42 s to 0.19 s over the movie,
  the worst 1,346 to 165 ms.
- **Pauses over 100 ms 26 to 8, the worst 1.36 s to 0.72 s.** What is left: the screen poll's
  reads fetched ahead as Route 1 loads, 353 and 643 ms in two pauses, and the tracking after a
  sweep, about 100 ms. Over `regress.py`'s pinned runs, 20 pauses over 100 ms before and 5 and 7
  after, with the same dialogue, speakers, battle lines and cursors.
- **While an area loads, every read is slow, whoever sends it.** Timing each burst over sun_10's
  first 20 s: 0.01 ms a read normally, 2 to 20 ms as Route 1 loads, pipelined or not, with
  asynchronous reads as with deterministic ones. The burst that held the loop up was a different
  one each run: the field's heads (30 reads, 140 ms), the screen poll's reads fetched ahead (42,
  434 ms), a look's marks (100, 1.1 s), a sweep of new blocks (512, 0.9 to 1.2 s). So the loop
  now puts off the look, and the sweep of new blocks after it, and the screen and field polls,
  while the emulator's last reply took over 1 ms a read, for 1.5 s at most (`app.Busy`,
  `Rpc.reply_seconds`); dialogue is polled all the same. Over spike 60: the worst pause 725 to
  235 ms, none over 250 ms. Pauses over 100 ms 8 to 17: a look already walking as the load
  began still walks, calling back every 50 ms, and each dialogue poll between costs about 60 ms
  of slow reads itself. Two pinned replays: 6 and 12 over 100 ms, the same things said.
- **The listing now reads the field 0.1 s after the field's app starts**, as the tutorial's battle
  hands back to it, and the camera still looks as it did in the battle: the first listing's
  directions are turned half round, `(-571, -312)`, and 0.4 s later right, on every run. The old
  listing was not ready so soon, and its first after the battle matched the last before it. A key
  pressed in those 0.4 s would say directions the wrong way round. Fixed the same day: the field builds its
  camera's translator again after a battle and through a door, and it reads 0.0, the world's
  south, until its +0x14 turns from 0 to 1 as it is given an angle (pi at first, then the
  scene's -2.36 on Route 1, and the porch's -1.93 after the front door in sun_5). The compass
  keeps its quarter while +0x14 is 0 (`camera.SET`): no half-turned listing in two pinned
  replays. Through the front door the translator reads pi for 0.2 s, set, before the porch's
  west, so the first porch listing is unturned as it always was: +0x14 cannot tell that one.

## Spike 130 — what the net finds that the heaps' watch missed, and where

2026-09-28 (`spikes/130_missed_finds.py`, movie4 replayed with the runtime reading it, pinned, and
it followed its recording). Every whole sweep logs what it found that the heaps' watch had not
handed over (ADR-0034). Over the eleven ORAS movies that was mostly name buffers (214), move
models (96), CoreParams (64), heaps (63) and map blocks (50). For each object at each whole
sweep, the spike says which block it lies in now, how big that block is, how far into it the
object is, and when the watch handed that block over.

- **The map blocks are the same two objects all movie.** They sit `0x867ec` and `0xa1f2c` into
  one block of 1.3 MB in heap 0xf1. The watch handed that block over 3.7 s into the replay,
  before the field's DLL had been read for their class (14.5 s). So both reasons
  guessed for them hold at once: the class was learned late, and the objects lie far past the
  8 KB a large new block's sweep reads. After that the net "found" them after every battle. A
  battle tears them down where they are, the whole sweeps during it see none, and the field
  builds them again in the same place. The field never lost them: a sweep that finds no block
  leaves the field's list as it was, and the field reads them again once they say the load is
  done.
- **So every one of the field's asks for a sweep was wasted.** During a battle the field reads
  its blocks as holding no chunk, and asked for four whole sweeps each time. All 72 asks in the
  recordings' logs of ORAS and X came while another app ran: a battle, the starter's choice,
  the name entry, Wally's catch. In movie4 that was 32 whole sweeps. Built: the field asks only
  while it is the app running. A want raised in another app is spent, so it is not swept for
  the moment the field comes back (`gen6.field` `sweep_wanted`).
- **The CoreParams are in blocks nothing reads.** Thirteen lie `0x2374`, `0x2544`, `0x2728` and
  `0x1fee8` into one block of heap 4 of `0x20020` bytes: the game's 128 KB and its own header
  of 0x20. That is just over the 128 KB a new block is swept whole up to (`gflheap.WHOLE`), so
  only its first 8 KB is swept. The watch handed it over 24 times, as its first bytes changed.
  Sixty more turned up at once as a battle ended (Blaze's level up), 0x1e4 apart, in a block of
  5 MB in heap 0 that no new block had brought. Two were in no watched heap. Only the DexNav
  reads a CoreParam, and it finds its record by a whole sweep and keeps it all session
  (`oras.hidden`), so none of these misses costs a sweep. Sweeping `0x20020` whole was not
  done: it would quieten the net's log and give no reader anything.
- **The asks for the zone table and the process manager (`997f0e4`) cost next to nothing.**
  With the change, over the eleven ORAS movies "no zone
  table" asked for 3 whole sweeps and "no process manager" for none; over X's two, 2 and none
  (neither X movie is pinned, and both went their own way).
- **Measured** with `regress.py run` over the eleven ORAS movies, against two runs of main's code
  (`reorg2-after`, `reorg2-control`):

  | | main | main again | now |
  |---|---|---|---|
  | whole sweeps | 314 in 103.9 s | 311 in 102.9 s | 258 in 81.8 s |
  | everything finding things costs | 276.3 s | 276.0 s | 253.9 s |
  | pauses in polling over 100 ms | 466 | 476 | 375 |

  movie4 alone: 69 whole sweeps in 25.9 s became 38 in 13.5 s, and its pauses over 100 ms 105
  became 63. Every movie followed its recording with the same dialogue pages, and the diffs
  against main are the size and kinds of the control's, less the 32 "no chunk loaded" lines.
- **Left open.** Sweeps of new blocks are now half of what finding things costs (126 s of 254),
  the slowest 0.2 s for 3,000 to 4,000 blocks at once. And the net's log still names the map
  blocks and CoreParams after every battle, which is noise.

## Spike 133 — what the sweeps of new blocks read, and what they find

2026-09-28 (`spikes/133_block_sweeps.py`: `record` replays movies with the runtime reading them and
writes every sweep of new blocks; `weigh` reads them back). After spike 130, sweeps of new blocks
were half of what finding things cost over the eleven ORAS movies, 126 s of 254.

- **They read seven times what the blocks hold.** `Discovery.sweep_spans` joined the new blocks'
  spans with `merge`'s default gap, a megabyte, so it read all the memory between two new blocks
  up to a megabyte apart. Over movie4 the spans held 138 MB and the sweeps read 951 MB, 33 s of
  the replay; over movie7, 99 MB and 475 MB, 17 s. The commit that made the sweep
  (`ba607c1`) says it reads "the memory of new blocks alone", as ADR-0034 does: the gap was
  `merge`'s default, not a choice.
- **What the memory between found.** 527 objects nothing else had found yet over movie4 (448
  CoreParams, 51 name buffers, 21 heaps, 6 map blocks, a message window) and 196 over movie7 (138
  name buffers, 49 layout works, 9 heaps).
- **Built**: the spans are joined only where they share a page (`gap=0`, as Sun's reads already
  were). Over the eleven movies, all followed:

  | | before | now |
  |---|---|---|
  | sweeps of new blocks | 126.1 s | 25.0 s |
  | everything finding things costs | 253.9 s | 154.0 s |
  | pages read by every sweep | 4.4 GB | 2.4 GB |
  | pauses in polling over 100 ms | 375 | 283 |
  | 99th percentile gap between polls | 52 ms | 39 ms |

  The same dialogue pages, battle lines and speaker's names in every movie, and diffs the size
  and kinds of a same-code control. The net now catches what the memory between used to find:
  name buffers 123 to 361 over the eleven movies, and in movie1 the intro's 114 layout works,
  which the layout reader was building from their pool already, 1.5 s after the intro started,
  before anything showed. Its YES was said at 101.61 s against 101.66 s.
- **The sweeps again are two thirds of what is left, and find little.** Over the eleven movies
  the sweep 2 s on found about ten name buffers a movie, and in movie11 talk windows, a heap, map
  blocks and the camera, the pool-built objects it is there for; the sweep 0.5 s on found a
  CoreParam or a heap now and then. Dropping the one at 0.5 s would save about 8 s over the
  eleven movies and was not done: ADR-0034 chose both, and a find it makes would wait 1.5 s more.
- **Left open.** Whole sweeps are now most of the cost, 82 s of 154, 260 of them at 0.3 s each:
  at the start, 1.5 s after each app starts, when a reader asks, and every 30 s.

## Spike 134 — why each whole sweep happens, and what it costs

2026-09-28 (`spikes/134_whole_sweeps.py`: `record` replays movies with the runtime reading them and
writes each whole sweep with its reason and the seconds of each part; `weigh` reads them back).
After spike 133, whole sweeps were most of what finding things cost over the eleven ORAS movies:
260 of them, 82 s of 154.

- **Where a whole sweep's time goes**, over movies 4 and 7, 65 sweeps and 34 s: reading the heap
  over RPC 22.0 s, the readers taking in what it found 5.6 s, scanning the pages for vtables
  4.6 s, measuring where the heap is first 2.1 s. The readers' part is not in `regress.py`'s
  figure for whole sweeps.
- **What it reads.** About 11,700 pages a sweep, 53% of them all zeros: the heap as measured only
  ever grows, and each block reaches one block further either side (spike 72). Only a third of
  the pages lie in blocks the watched heaps have in use; but live objects lie outside them too
  (the process manager), so a sweep cannot keep to those blocks without losing them. Walked from
  the swept pages, the heaps' lists are also a little wrong, since the pages were read at slightly
  different moments while the game changed them.
- **Why each happened**, over the two movies:

  | Reason | Sweeps | Caught something the watch had not, by class |
  |---|---|---|
  | an app starting | 39 | the field coming back (16): map blocks and CoreParams only |
  | the 30 s timer | 15 | name buffers, map blocks, a CoreParam |
  | the battle asking for its gauges | 7 | 60 CoreParams, once |
  | the start | 2 | nothing |
  | the speaker's talk object | 2 | name buffers, list controllers |

  "Caught" counts objects found by class, not what is found by what points at it (a gauge, a
  choice box), which is what the battle's and the app starts' sweeps are for.
- **The readers' 5.6 s was mostly the DexNav**: 3.9 s of movie4's 6.6 s went on looking for the
  record at every whole sweep, every page searched for every CoreParam, though the record lives
  all session. It now checks the record it has, two reads, and looks only when that no longer
  reads as one (`hidden.find`'s `known`): 6.6 s to 3.7 s. The zone data's holder is looked for the
  same way, 1.0 s of movie4, and was left: whether it stays where it is when the field is built
  again is not known, and freed memory can still look like it.
- **Searching pages that follow on from each other as one run**, where `find_objects` searched a
  page at a time, 4 million searches over movie4, was tried and taken back: 111 ms to 83 ms on
  5,500 pages of made-up data, and 2.79 s to 2.98 s on movie4's own. The search itself is the
  cost, a target at a time, not the number of calls.
- **Built: no whole sweep when the field comes back.** ORAS's field coming back after a battle or
  a menu builds its map blocks again where they were (spike 130) and keeps the rest; the sweep
  1.5 s on found nothing else a reader asks for by class. The profile says whether an app start
  wants the sweep (`Gen6Profile.app_sweep_wanted`, `RunningApp.back`); Sun's always does. Over
  the eleven movies, all followed:

  | | before | now |
  |---|---|---|
  | whole sweeps | 260 in 82.0 s | 234 in 72.9 s |
  | everything finding things costs | 154.3 s | 145.6 s |
  | pauses in polling over 100 ms | 280 | 248 |

  The same dialogue pages, battle lines, place names and speaker's names in every movie, and the
  changed lines of the kinds a same-code control shows. Fewer sweeps went than the field came back,
  5 of movie4's 9, since a sweep no longer made at an app start leaves the 30 s timer to come
  sooner.
- **Left open**, each an entry in the [roadmap](../roadmap.md): the net every 60 s, a whole sweep
  reading less, the zone data's holder, and the net's log naming what nobody reads.

## Spike 140 — where the game's code modules load, and why

2026-09-29 (`spikes/140_modules.py`: `record` replays movies and, at the start and at every app
start, lists every module whose header a page of the code span holds; `weigh` says every address
each was seen at). Orion asked whether the modules land where they do because nothing on a 3DS
randomises addresses, and whether that could be relied on.

- **The game chooses, the emulator never does.** Azahar's `RO::LoadCRO` takes, with each request,
  the buffer the game read the module into and the address to map it at, checks that the address
  is page-aligned and inside the process's image, and maps it there. No ASLR, on the console or in
  the emulator, so nothing moves a module but the game's own choice.
- **A module goes after whatever is loaded beneath it**, from the bottom of the modules' memory,
  `0x006f3000` in Omega Ruby. The field's, a battle's, the Bag's, the PC menu's, each loaded alone,
  were at the bottom every time; the Box's at `0x006fa000`, on the PC menu's; the summary's at
  `0x0070b000`, on the party list's; the field's contact encounters' at `0x00838000`, on the
  field's. Over the eleven ORAS movies, 25 modules, 22 of them at one address each, and the name
  entry's keyboard (`DllStrInput`) at three, `0x006f3000`, `0x00724000` and `0x00742000`, and
  `DllBackGround` at two. The same things loaded in the same order land in the same place; a
  module loaded on top of something else lands elsewhere.
- **Each module's header, 0x80 into it**, is `CRO0`, or `FIXD` once fixed where it is; then its
  name's offset, the next module loaded (0 for the last), the previous (the first's is the last),
  its size, and at +0xc0 and +0xc4 where its name is and how long, as Azahar's `CROHelper` lays
  the fields out. The first module's `previous` is the last, so the list's head is the one whose
  previous has no next.
- **What `learn` did cost, and missed.** It read the code past the executable's image at every
  app start (spike 138), 25,000 to 55,000 pages more over movies 2 and 4. And a module loaded below
  where that image ended was never read: started on the field, the code measured at attach runs on
  through the field's module, and a battle's, loaded where the field's had been, never had its
  gauges found. Every replay attaches before the title's modules load, so none showed it.
- **Built: a module is learned once, and found again by its header** (`core.modules`,
  `Discovery.learn`). The bottom is found once, as the lowest page holding a header, and each look
  checks it first and follows the list: four reads with one module loaded. A module is read the
  first time it is seen and each class's vtable is kept as an offset into it; seen again
  anywhere, its vtables are worked out from where it landed. A vtable's first slot may point into
  the executable, where a module's class keeps a method it inherited. Over movies 2, 4 and 7,
  sweeps and reading code cost 10.2, 15.4 and 11.3 s, against 10.7, 17.0 and 11.5 s reading the
  code past the image at every app start and 10.2, 15.5 and 11.4 s stopping once every class was
  known; what was said differed only in what races.
