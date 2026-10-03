# Text, and who is shown it

Message windows, pages, control codes, prompts and choice boxes: everything between the game's
words and a spoken line.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 3 — the layout pane tree

2026-09-11. Structure yes, text no:

- **Captures must be taken with emulation paused.** A 1.8 s capture of a running game is a smear
  across ~110 frames, not a snapshot: two captures of the same idle state differed by 114,000
  bytes, and diffs scaled with elapsed time rather than with game state, burying every small
  signal. With F4 pause held during the read, five consecutive state pairs came back
  **byte-identical**. Every snapshot pa3ds tests against is captured paused.
- Two traps in diffing this game. A ~115 KiB block at `0x16170000-0x1618d000` rewrites completely
  every frame (audio, most likely) and swamps any unmasked diff. And layout structures are
  double-buffered, so a field can alternate purely with capture parity — the giveaway is
  `m0 == m2` across cursor positions 0, 1, 2. A real cursor field has `m0 != m2`.
- ORAS draws its UI with Nintendo's standard CTR layout engine, and the resources are resident:
  20 `CLYT` files with 227 `pan1`, 700 `pic1`, 42 `txt1`, 43 `wnd1`, 83 `grp1` sections, plus
  named resources like `msg_cursor.bclyt` and `menu_icon5.bclim`. The documented field layout
  parses cleanly: section magic and size, then the pane header with its 24-byte name at +12, and
  for `txt1` the text fields at +0x4C with the string at `section + textOffset` as UTF-16LE.
- **So the pane tree is walkable — but it carries no meaning.** Pane names are minified
  two-character identifiers (`1h`, `28`, `4x`, `c`); only `RootPane` is descriptive. And the text
  in a resident `txt1` is the layout designer's placeholder — `１２３４５６７８９０`, `999`, and
  leftover Japanese `ずかん` and `サーチする` — not what is on screen. NW4C layouts are templates;
  runtime text is set into a separate pane object.
- The menu cursor is readable as a coordinate, not an index: four bytes at `0x165a90b2`, `+0x20`,
  `+0x48`, `+0x68` read 0 with the menu closed and 112, 118, 125, 118 as the cursor moved down,
  down, up. Menu position is derivable from the highlight's Y, with no index field found.
- Layout resources are pooled, not allocated per screen: section counts are identical with the
  menu open and closed, and the structures that change on open do not revert on close.
- **No English game text is resident while only the main menu is open.** Identical counts of
  English-looking UTF-16LE runs open and closed (2,412 vs 2,413), all of it mesh and vertex data
  read as text by coincidence. This is a fact about that screen, not about the game.

## Spike 3b — live dialogue text

2026-09-11, with "Would you like to save the game?" on screen:

- **Live dialogue text is in RAM as NUL-terminated UTF-16LE.** Found in two copies: `0x08608810`
  in the heap and `0x1585c2e0` in the linear heap, both holding the full 32-character string
  including its question mark. So the no-hardcoded-text rule is satisfiable and milestone 1 is
  not blocked.
- Both copies carry a character count in the u32 `0x1C` bytes before the string (`0x20` = 32,
  exact). The containers differ though: the pointer to the string sits `0x3C` before it in the
  heap copy and `0x64` before it in the linear heap copy, so the enclosing structure is not one
  shared shape.
- **Refuted: `0x5544` is not a string-object tag.** It appears `0x30` bytes before both strings,
  but there are 16,058 of those dwords in 38 MiB and a length-plus-terminator check on them
  admits 45% as "strings" that are plainly heap pointers and padding. Do not retry this.
- The validator lesson: scanning for well-formed UTF-16LE *prose* — multiple words, mixed case,
  letters and ordinary punctuation only — located the string immediately and with no false
  positives worth mentioning. Scanning for a constant dword located nothing. Anchor validators
  should test the shape of the payload, not the presence of a magic number.
- A full sweep of `0x00000000`-`0x30000000` at 1 KiB granularity, keeping only non-zero pages,
  takes about 20 s and yields 38.1 MiB across 38,966 pages. That sweep is the readable map, at
  1 KiB resolution, and supersedes the coarse probe from spike 2.
- The code region from `0x00100000` holds engine strings, not game text: the prose there is
  Quazal/NetZ networking library error messages. Game text lives in the heap and linear heap.

## Spike 11 — the control codes, decoded

2026-09-12:

    u16[0] = 0x0010     the marker
    u16[1] = count      halfwords following the count
    u16[2] = code       the control code
    u16[3..] = count-1 arguments
    next = pointer + (count + 2) halfwords

- Read from `0x0014dfb4` (the code), `0x0014dfcc` (argument *n*, bounds-checked against
  `count - 1`) and `0x0014e008` (skip to the next text). Parsing the text spike 7 recorded with it
  reads **172 sequences with no failures**, and only four distinct codes in the whole opening:
  `0xbe01` 151 times with no arguments, `0xbe00` 5 times, and `0xff00` with an argument of 2 and
  of 0, always paired.
- `0xbe01` and `0xbe00` are the page breaks — they fall exactly where the box waits for A and then
  clears or scrolls. **That is how to speak a dialogue page at a time** rather than dumping the
  whole message.
- `0xff00 2` ... `0xff00 0` brackets "Professor Birch lives right next door", which the game draws
  in a different colour. It is emphasis, not an insert.
- **The player's name is already substituted.** The recorded text reads "You should go and see
  your new room upstairs, wee!" with the name inline and no code around it, so name inserts are
  resolved before the text ever reaches the pane. The unknown about expanding them was mostly a
  false alarm; what is left is knowing which codes carry meaning worth speaking.

## Spike 14 — list windows, and the line as typed

2026-09-13. `spikes/12_menu_classes.py` reads the code and one old capture;
`spikes/12_menu_window.py` watched 152 moments of the replay after slot 0.

- **`app::tool::MenuCursor` is the cursor graphic, not the cursor.** It is 0x18 bytes with two
  virtual slots, and its one real method clamps an (x, y) pair and writes it into a layout pane's
  translation — the coordinate spike 3 found. The only code that allocates one is a
  `app::tool::MenuWindowDrawer` method, which stores it at `this+0x1a0`; `AppMenuDrawer` and
  `KawaigariMenuDrawer` are drawers too, and `MenuWindowController` builds them. So the cursor is
  shared by one family of list windows, and its index lives elsewhere.
- **`app::tool::MenuWindow` holds the list.** Its constructor `0x003478b4` sets +0x14 and +0x18,
  whose product is the item count, allocates 0x1c-byte item records at +0x20, and zeroes +0x2c.
  The setter beside it stores +0x2c only when the new value is below the item count, and the same
  four instructions are inlined five more times in `MenuWindowController`'s code, which is what
  moving a cursor would look like. Each record's +0x14 is a `gfl::str::StrBuf` label.
- The 2026-09-11 capture of the in-game save prompt holds exactly one: 2 by 1, labels `YES` and
  `NO` read from RAM, +0x2c = 0 with YES the default, and a `MenuWindowController` pointing at
  both it and an `AppMenuDrawer`. That is milestone 1's Yes/No prompt with its labels in hand.
  **That +0x2c follows the cursor is still a reading of the code**, not a watched fact; the one
  prompt in the replay was missed (below).
- The `0x5544` word in front of every heap object is the heap allocator's block header, followed by
  the block size. That is why spike 3b's "tag" matched 16,058 times.
- The in-game X menu is probably not this family. The census names
  `field::EventSubScreen::EventFieldMenu` and `savedata::FieldMenu`, and nothing ties those to a
  MenuWindow yet.
- **A TextBox holds its line as far as it has been typed.** `nw::lyt::TextBox` +0xd4 points at the
  line's UTF-16LE text, +0xf8 is the buffer's capacity in halfwords and +0xfa the number typed so
  far. The pair reads as one word like `0x00240086`, and the first reader built on this split it
  the wrong way round and matched nothing for a whole replay. Read the length, never a
  terminator: a colour code's argument of 0 ends the string early. +0xf4 points back into the
  owning window, at `window+0x9c`, and is zero once the window has closed. **All 245 typed lines
  in the replay were the start of a line of their window's message**, which is what lets the page
  on screen be read rather than inferred.
- **The two page breaks differ.** `0xbe01` clears the box. `0xbe00` scrolls it up one line, and
  the previous page's last line stays on screen above the new one. Either way the new line goes
  into a recycled line object, whose byte +0x66 in the line record is its row.
- Liveness held throughout: at every moment the windows with dead line objects were the stale
  ones, and the window taking new text was live.
- ~~Battle text is not in a message window.~~ **Wrong; see spike 19.** This spike measured the
  heap once at the start, and the battle builds its window where that measurement did not reach.
- **The dialogue reader built on this works on the replay.** `python -m pa3ds --silent` announced
  all 106 pages this spike recorded, in the same order, plus six its slower polling had missed.
  Three bugs came out of running it live, none of which the unit tests could see because the fake
  memory shared each misreading: the line count is one byte, not two; a TextBox's capacity comes
  before its length; and a heap sweep that runs during a scene load misses a window for a moment,
  so progress has to outlive the sweep or pages are skipped when the window comes back.
- **A sweep of a fast replay can miss a whole screen.** The movie answers a Yes/No prompt ("why not
  give a nickname to..."), and no MenuWindow appeared: at frame_limit=1000 the prompt is up for
  about a second, and the sweep ran every 1.8 s. Absent from a sweep is not absent from the game.

## Spike 15 — the replay's Yes/No prompts, caught by breakpoint

2026-09-13 (`spikes/13_menu_cursor.py`):

- **A constructor breakpoint catches what a sweep cannot.** Stopping in `MenuWindow`'s
  constructor hands over `this` in r0 at any replay speed. The replay after slot 0 builds two list
  windows, both 2 by 1 with `YES` and `NO` read from RAM: one at `0x084fb398` while Birch's
  "In my Bag!" is up, choosing a starter, and one at `0x08643ab8` about 100 s later, the nickname
  question. The second is at the same address as the save prompt in the user's own capture from
  2026-09-11 — another session and another prompt.
- **Neither prompt moved its cursor, so +0x2c is still unproven.** A write watchpoint on +0x2c saw
  the constructor's zero, one more zero from setup code at `0x003516d4`, and then the object being
  freed, 1.7 s after construction for the nickname prompt. Answering YES needs no movement, and the
  movie answers the nickname question NO without moving the cursor, most likely with B. This movie
  cannot show +0x2c move; that needs a recording that moves a cursor on a Yes/No prompt.
- Watching the address from the moment of attaching recorded 31 writes before the constructor ran,
  from whatever used that heap memory earlier. A watchpoint on a heap address means nothing until
  the object's own constructor has run; the vtable read at each hit says whose write it was.
- **Under the JIT a watchpoint's pc can be past the write.** The constructor's zero was reported
  at `0x0012344c`, inside the allocator it calls next, with lr pointing back into the constructor.
  The first run took that for a foreign write and dropped the watchpoint. Judge a hit by lr and by
  the object's vtable, never by pc alone.
- While the stub holds the game halted, `PrintWindow` returns solid black; and from 157 s on,
  `rig.screenshot()` found no visible emulator window at all, even after resuming. Pictures from
  a stub session are not evidence. Label hits by reading memory through the stub.

## Spike 17 — the opening, which the reader did not read

2026-09-13 (`spikes/15_intro_text.py`). The user ran pa3ds live over a new game and heard nothing
while Birch introduces himself in the retro 8-bit style. movie2 was replayed from frame 0 at real
speed, with a picture, every message window and a heap census every 4 s.

- **The opening's text is in ordinary message windows.** `print::MsgWin`, TextBox line objects,
  typed text at +0xd4 exactly as elsewhere: "And this...is what we call a POKéMON." at
  `0x08331a2c`, then `0x0832588c`, `0x082b43cc`, `0x082826b0` and `0x082d0aec`. The whole
  monologue is resident as StrBufs from the moment the opening starts, including "Select a player".
- **The windows are built where the heap was not yet measured.** `regions()` 20 s after boot put the
  end of the first heap block at `0x08300000`; the opening's windows are at `0x0832xxxx` and
  `0x0833xxxx`, and only a re-measure at 40 s showed the block reaching `0x08338000`. The runtime
  measured the heap once, at startup, so pa3ds started before the game had grown its heap could
  never sweep the opening's windows. The spike made the same mistake on its first run and saw
  nothing for two minutes. `app.run` now re-measures every 10 s.
- **Windows hand their line objects on.** At 68 s the closed window `0x0832588c` still had records
  pointing at the two TextBoxes that `0x082b43cc` was typing its own message into. Its class check
  passed and its text did not match, so nothing wrong was spoken, but only by luck. A TextBox's
  +0xf4 points at `window+0x9c` of the window that owns it, which is exact: the reader now requires
  it.
- A `MsgWin`'s destructor leaves the `gfl::str::StrWin` vtable behind, the same pattern as spike 13's
  TextBox, which is why census counts flick between the two classes at one address.
- **A list window exists long before its prompt.** `app::tool::MenuWindow`, its drawer and its
  system appear in the census 28 s in, while Birch is still a sprite. Being constructed is not
  being shown, so a Yes/No reader needs a separate test for that.

## Spike 19 — all of movie2, read by the runtime

2026-09-13. The same loop as `python -m pa3ds --silent`, started at boot, over movie2 from frame 0
at frame_limit=1000.

- **The replay is faithful to the end.** The opening, the name confirmation, Mom and the house,
  May, the Poochyena fight, Treecko this time, the nickname question, and the Route 103 question
  answered NO ("Oh, don't be that way! You should go meet my kid.") and then YES ("Great!") —
  the user's account of the recording, in order.
- **Battle text is in an ordinary message window,** `0x0855b2a4`: 17 pages, from "The wild
  Poochyena used Tackle!" to "The wild Poochyena fainted!". Spikes 14 and 16 said otherwise
  because each measured the heap once, before the fight; the reader re-measures every 10 s and read
  it like any other. Three spikes and the first runtime each made that one mistake.
- 135 pages from ten windows. At 10x the opening's first four lines are over before the first
  sweep; at real speed they are read (spike 17).
- The prompts, for labelling list windows: after "So you're wee?" in `0x082d0aec`, after the
  nickname question in `0x08286478`, and after "What do you think?" in that same window.

## Spike 20 — when a list window is on screen

2026-09-13 (`spikes/16_list_shown.py` at real speed, read back with `16_list_shown_analyse.py`).
movie2 from frame 0, the whole list-window family read every quarter second through the name
confirmation, with no GDB stub attached.

- **The drawer says it.** `AppMenuDrawer` +0x20 + i x 0x18 is item i's state: 0 for every item of
  the opening's early window through its hundred hidden seconds, 2 on the item under the cursor
  while "So you're wee?" waits for an answer, and 6 on the item chosen. The prompt had its own
  drawer and controller, built for it; the early ones stayed at 0.
- **The cursor moved 0, 1, 0, 1, 0, 1, 0 and YES was chosen,** straight to "Ah... I see!". The
  item states flipped with it. `MenuWindowSystem` +0x11 read 0 hidden, 1 shown, 2 chosen and 3
  closed, but nothing found yet points at the system object; it is simply allocated after its
  window. `MenuWindow` +0x30 and +0x32 turn 1 when an item is chosen.
- **Spike 18's replay with the stub attached did not follow the recording to the end.** It saw
  the same 0, 1, 0, 1, 0, 1 but never the final 0, answered NO, went back to name entry and came out
  with "So you're !!!!!!!!!!!0?". Without the stub the same movie answers YES every time. Halting
  the game from the stub can knock a replay off its recording; check a stub session's story against
  a replay without one before believing where it ended up.
- So a list is **shown while the drawer's state for the cursor's item is 2**, and it is reached
  from its controller, which a sweep finds: controller +0x08 is the drawer, +0x0c the window, and
  the drawer points back at both. `src/pa3ds/games/gen6/lists.py` reads it that way.

## Spike 21 — the same words again

2026-09-13. Live play by the user: talking to the same person or object twice was heard only once.
The reader now logs why it starts a message from its first page, and movie2 was replayed as far as
the house at 10x.

- **In the field a message window stays open between conversations.** Every repeat in the run was
  caught by the typing starting over — "again, typed from the start" — and none by the window
  closing or being rebuilt. The buffer gets identical words, the owner links never clear, and only
  the typed text going back to the start of the first line shows the game is saying it again. The
  old reader, which waited for a change of message or a closed window, could never repeat anything
  there.
- The three repeats are real ones: Mom's "Well, wee? Don't you want to go see your new room?" twice,
  20 s of game time apart, as the player tried the front door twice; and "It's a Snorlax Doll!"
  twice, twice over. Out of about forty messages nothing else restarted, so the test is not firing
  on ordinary typing.
- The test rules out the two things that also leave less typed than a poll ago: a scroll, which
  keeps a whole line on screen, and a cleared page that begins like the first one, which also
  matches a later page (`_restarted` in `src/pa3ds/games/common/dialogue.py`, with tests for both).

## Spike 22 — the Yes/No reader, run live

2026-09-13. `python -m pa3ds --silent`'s loop with the list reader, over movie2 from boot at real
speed and over all of it at 10x.

- **At real speed it reads the name confirmation:** "YES, 1 of 2" as the prompt opened, then "NO",
  "YES", "NO", "YES", "NO" as the cursor moved.
- **It first announced the prompt before the question.** "So you're wee?" was in a window no sweep
  had found yet, and was said 1.6 s after "YES, 1 of 2". Opening a list now forces a sweep and a
  dialogue poll before the list is announced.
- **At 10x only the starter choice was announced.** Each prompt gets a new controller the moment it
  appears, and at 10x a prompt answered in under two seconds is gone before the next sweep. The
  reader now keeps reading every controller address it has ever found, since the game rebuilds
  prompts at addresses it has used, and sweeps again a second after any dialogue page.
- **The replay did not answer the same way every time.** The name confirmation was answered YES,
  as recorded, in six runs — both real-speed runs without the list reader, all four 10x reader runs,
  and the real-speed list spike — and NO, back to name entry, in two: spike 18's run with the GDB
  stub, and the first real-speed run with the list reader, which saw five cursor moves where the
  recording has six. The cause is not known; the 10x run with the list reader stayed faithful, so
  it is not simply load from reading. A replay is not proof of the recording's story: check the
  story in every run before believing what it shows.
- **A rerun to check the question-first fix never left the title screen.** Azahar's log shows the
  title module loading at 19 s and no game activity after it, so pa3ds, correctly, said nothing for
  150 s. The fix is covered by unit tests only until a replay gets through.
- **Every RPC read of unmapped memory is an error line in Azahar's log:** about 11,000 in 150 s in
  each of the last two runs, from heap measurements probing past the heap's top and from sweeps
  crossing small gaps inside it. The log rotates within minutes. Whether that load is what knocks
  replays off their recordings is not known, but it is load pa3ds need not cause: the heap's own
  allocator objects (`gfl::heap::CTRExpHeap`, named by the census) know their extents exactly, and
  reading those would replace probing.

## Spike 23 — the prompts live play found silent

2026-09-13 (`spikes/17_field_prompts.py`). The user reported the nickname question and Birch's
Route 103 question never saying their options, and the starter choice saying its options but not
"Choose this Pokémon?". movie2 at 10x, list controllers and message windows read every 0.1 s, a
heap sweep with every new StrBuf every 2 s.

- **The starter choice is an ordinary list window,** reached through its controller at
  `0x0826a2a4`: drawer state 2 on YES, then 6 as it was chosen, 0.3 s apart at 10x. The reader's
  shown test holds for it.
- **Its question is not in the heap.** The starter screen's own words are StrBufs — "Choose a
  Pokémon.", "The Grass-type Pokémon / Treecko" as the cursor moves between balls — but "Choose
  this Pokémon?" appeared in no StrBuf and no message window. It is drawn some other way, most
  likely a layout TextBox given its string directly.
- **No list controller existed for the nickname question.** The question itself was read from its
  message window, and no `MenuWindowController` turned up in any sweep before the answer. At 10x
  that could be timing, but the user waited at real speed and heard nothing, so the likelier
  reading is that prompts which appear only after A on the question are another class.
  `app::tool::TalkCursorWindow`, a talk window with a cursor, is the first candidate.
- The replay answered the nickname question YES this time ("Yup! That's the spirit!"), where
  earlier runs of the same movie answered NO — spike 22's warning again.
- These prompts are twenty-odd minutes into movie2 and last a fraction of a second at 10x. Studying
  them needs the replay stopped just before them: save states made at a chosen moment, which the
  rig cannot yet ask for.

## Spike 26 — the field's choice box

2026-09-13 (`spikes/20_field_yes_no.py`, movie3 slots 2 and 3, real speed).

- **A read watchpoint on the label entries stopped in the box's draw loop** (`0x00357d20`) with the
  box in r4 and the row in r5, and in its update (`0x00358158`), which picks the answer as the entry
  at +0x84 plus +0x80 (`0x003583a8`). Its layout: +0x7c the entry table, +0x80 the cursor row, +0x84
  the first row shown, at most four rows, +0x88 the count, +0x90 a state.
- **Spike 24's 0x30-byte struct is the entry table's allocation.** The allocator puts 0x20 bytes of
  its own (8, 0x20, zeros) before every block, the box's included; the entries start after them.
- **The box has no vtable, and no vtable holds its functions.** Among the heap objects it points at
  is `gfl::ui::CTR_DeviceManager` at +0x0c, a class a sweep can name. That is the anchor: every word
  pointing at the manager is a candidate, and the layout above is the validator.
- **Watched every 50 ms, +0x80 is the cursor:** 0, 1, 0, 1 as the recording arrowed, and the answer
  followed it — NO to the first Route 103 question, YES to the re-ask. The on-screen arrow trails it
  by a few frames. The state went 3 to 4 on each answer, and the box was freed half a second later.
- **Polled from the load, the box appears only after A.** The question sits fully typed, waiting,
  with no box; A allocates it in state 0 or 1; half a second later it is state 3 and on screen.
- **The nickname question, the Route 103 question and its re-ask were all this box,** at
  `0x08804680` all three times. `src/pa3ds/games/gen6/choices.py` reads it.
- **Run live from slot 2 at real speed, the runtime read all three prompts in story order:** each
  question, then "Yes" as the box opened 1.5 s later, every move after it, and Birch's reply to the
  answer.

## Spike 34 — does a field message box leave the screen between conversations?

2026-09-14 (`spikes/30_window_hidden.py`, movie3 slot 4, real speed). The dialogue reader now takes
typing that starts over as a repeat only if the box left the screen in between: the options screen
retypes the text speed description in a box that never leaves, as a demonstration, and the user
heard it again at each speed change on first launch.

- **Yes.** Birch's box's first line object read shown from "Whew..." and hidden 0.4 s after "You
  saved my hide. Thanks a lot!", shown again for "Oh?", hidden after "Come along to my lab". In the
  lab, the box hid between "...that Pokémon you used earlier?" and "wee received Torchic!".
- Replayed from slot 10 with the change, first-launch speed changes (FAST, NORMAL, SLOW) said
  nothing more, and returning to TEXT SPEED from another option still said its description.
- **The battle's message box was "unknown" throughout:** its line objects lead to a root no tracked
  layout owns, so the reader falls back to the old rule there. The battle's layout works are not in
  the heap span swept, which is likely also why its command menu gave no screen text (spike 32).
- Every message box's line objects are in the linear heap (`0x14c52b34`, `0x14e1d1b4`), which screen
  text never sweeps; menus' TextBoxes so far have all been in the heap.

## Spike 58 — why a signpost read twice was still silent

2026-09-16 (`spikes/59_sign_repeat.py`, signs.ctm from its first frame, real speed). The rule above
did not fix it. The user recorded talking to Mom three times, reading "wee's House" and "Professor
Birch's House" several times each, walking between them, and talking to an NPC three times. Their
log had Mom and the NPC "again, typed from the start" every time, and each sign said only when the
other had been read in between. The replay said exactly the same, so the movie reproduces it.

- **A sign's words are never typed.** Its first line object reads `own/11` for "wee's House" from
  the first poll that sees it, every time; Mom's reads `own/2`, then 26, 28, 30, 31. So "typing
  starts over" can never be seen for a sign, whatever the poll rate.
- **Its box does leave the screen, for long enough to see.** Watched at about 60 Hz, the first line
  object's placement went shown at 43.21 s, hidden 45.37, shown 46.76, hidden 48.78, shown 50.68:
  over a second off screen between reads, where "inside a poll" had been the guess. Only the
  visible flag moved; the layout stayed drawn.
- **So the box coming back is itself a new showing.** The dialogue reader now says the page on
  screen again when a box it has seen off screen since the last page comes back, in addition to
  typing that starts over. It joins at the page on screen, so a box that comes back mid-message
  does not re-read the pages before. Replayed, "wee's House" was heard four times and "Professor
  Birch's House" three, Mom and the NPC once per conversation.
- The options screen's box never leaves the screen, so this changes nothing there, and the battle's
  message box has no tracked layout (spike 34), so nothing there either.
- **The first read of the first sign was heard after its box had closed.** The sign builds a new
  message window, and the next sweep for windows, up to 2 s later, is what finds it. That is the
  delay the user hears on a first line, and it is in spike 59.
