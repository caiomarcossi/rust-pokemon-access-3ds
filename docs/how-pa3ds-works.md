# How pa3ds works

This is a tour of pa3ds from the start, for a coder who is not a reverse engineer. It tells what
each part does and why it is necessary. It does not tell every detail: the detail is in the code,
the [ADRs](adr/README.md) and the [findings](findings/README.md).

Words in **bold** are words of [CONTEXT.md](../CONTEXT.md), the vocabulary that the code uses.
Technical words (DLL, vtable, heap) have a short meaning in the [glossary](#glossary) at the end.

Written 2026-09-29. If the code and this page do not agree, the code is correct.

## 1. The problem

A blind player plays a 3DS Pokémon game in Azahar, an emulator. The game shows words and
pictures on two screens. A screen reader cannot read them, because they are pixels, not text.

pa3ds reads the game's memory while the game runs. It finds the words that the game shows, and it
sends them to the player's screen reader as text.

pa3ds has two rules that make this difficult:

- **It only reads.** It never pushes a button and never writes to the game's memory (ADR-0009).
- **It never writes down an address.** The game puts each thing at a different address in each
  run. So pa3ds must find each thing again in each run (ADR-0003).

Most of pa3ds is the answer to one question: "Where in memory is the thing that the game shows
now?"

## 2. The parts, from the game to the voice

```
Azahar runs the game
   |  the RPC: UDP messages on port 45987, "read N bytes at address A"
   v
pa3ds.core.rpc       sends the reads, many at once
pa3ds.core.memory    keeps pages of memory it read, so one poll reads each page once
pa3ds.core.discovery finds objects in memory by their class ("sweeps")
pa3ds.core.heaps     watches the game's heaps for new memory, so sweeps stay small
   |
   v
readers (pa3ds.games.*)   turn objects into words: dialogue, menus, battle, the field
   |
   v
pa3ds.core.speech    sends the words to the screen reader (Prism, then NVDA)

The keyboard hook (pa3ds.core.keys) takes pa3ds's own keys, for the navigation of the map.
The player's ROM file (pa3ds.core.rom) gives the game's own words that memory holds only as numbers.
```

`pa3ds.app` is the loop that joins these parts. It runs about 60 times a second.

## 3. What the game's memory contains

The game is one program in the emulated 3DS. It sees one range of addresses, from 0 to 4 GB. pa3ds
reads the same addresses over the RPC. Four areas are important:

| Area | Where | What is there | Does it move from run to run? |
|---|---|---|---|
| The **executable** | from `0x00100000` | The game's main code, and its fixed data | No |
| The **DLLs** | after the executable | Code that the game loads for one **app** only: the field, a battle, the Bag | Sometimes (see [section 5](#5-classes-in-the-executable-and-classes-in-a-dll)) |
| The **heap** | `0x08000000` to `0x09000000` | Almost every object that pa3ds reads | Yes, always |
| The linear heap | `0x14000000` and up, in the runs seen | Memory that the graphics chip reads too; the battle's layouts are here | Yes |

The objects that pa3ds wants are in the heaps. The game makes them and frees them all the time. A
small change in timing changes the order, and so the addresses. That is why pa3ds cannot keep an
address from one run to the next.

## 4. How pa3ds recognises an object: vtables and class names

The game is written in C++. This gives pa3ds a way to recognise objects.

### A vtable is the same number at the start of every object of a class

A C++ class with virtual methods has one **vtable**: a table of pointers to its methods. The
vtable is in the code, not in the heap. Each object of the class starts with a pointer to that
vtable. So every `print::MsgWin` (a **message window**) starts with the same 4 bytes: the address
of the `print::MsgWin` vtable.

If pa3ds knows that address, it can find every message window. It reads memory and keeps each
aligned 4-byte word that is equal to the address. This search is a **sweep**.

### RTTI gives each vtable a name

The game also contains RTTI (run-time type information). The word before a vtable points to a
**typeinfo**. The typeinfo points to the class's name, as text. The name is "mangled":
`N5print6MsgWinE` is `print::MsgWin`. `N` starts a name in a namespace, each number is the length
of the next part, and `E` ends the name.

So pa3ds can go in two directions:

- **From a name to a vtable** (`Discovery`, `rtti.find_vtables`). pa3ds finds the text
  `N5print6MsgWinE` in the code, then the typeinfo that points to it, then the vtable that
  points to the typeinfo. pa3ds does this once, when it starts, for each class that a reader
  wants.
- **From an object to a name** (`rtti.ClassNames`). pa3ds reads the first word of an object and
  follows the pointers to the name. A reader uses this to check an object before it reads it.

This is the cheapest **anchor** that pa3ds has. No search for a guessed shape is necessary: the
game names its own objects.

### A name is not proof that the object is alive

When the game frees an object, its memory stays as it was until something else uses it. The
vtable pointer stays too. So a sweep also finds dead objects. Each reader checks more than the
name (a **validator**): that the object is still linked from something alive, that its pointers go
to sensible places, or that its data decodes correctly.

## 5. Classes in the executable and classes in a DLL

Both kinds have a vtable and a name. The difference is where the vtable is, and so how long pa3ds
can trust what it knows.

### A class in the executable

The executable is loaded once, when the game starts, and never moves. So:

- pa3ds knows the vtable's address from the start of the session.
- The address is correct for the whole session.
- A name that pa3ds looked up once stays correct. So it can keep the answer (a cache).

Examples: message windows, `nw::lyt::TextBox`, layouts, cursor windows, `gfl::str::StrBuf` (the
battle's **name plates**), the process manager, list windows, `PokemonParam` (a **Pokémon record**).

### A class in a DLL

A DLL is a module that the game loads for one app. The game loads the battle's module when a battle
starts. When the battle ends, the field's module can go into the same memory. So:

- The vtable does not exist until the app starts. When an app starts, pa3ds lists the modules
  that are loaded, and reads a module's code the first time it sees that module
  (`Discovery.learn`; see "Where a module goes" below).
- When the app ends, that memory becomes another module's. The same address can then hold
  another class's vtable, or no vtable.
- So a name that pa3ds looked up for such an address can become incorrect. It is not safe to keep
  it for the whole session.

Examples: `field::MapBlock` (the **map blocks**), the battle's HP **gauges**, each app's process
object, and the talk object's class while a talk runs.

### How each game looks up names

Both profiles use the same `ClassNames` class, and both keep every answer for the session.

- **Gen 6** (X, Y, ORAS) looks up names only inside the code that was there when pa3ds started.
  The check exists to stop a heap word that looks like a pointer from giving a false name. The
  result is that gen 6 cannot name a class of a DLL loaded later.
- **Sun and Moon** look up names in the whole range where code can be (`CODE_SPAN`). So Sun can
  name a class of a DLL loaded later. But Sun also keeps that answer for the session. If another
  module goes into that memory later, the answer can be incorrect. Nobody has seen this happen,
  and nobody has measured it.

### How the readers check an object of a DLL class

A sweep finds these objects by their learned vtable. After that, the readers check them in one of
three ways:

1. **They compare the first word with the vtable that the object had when it was found.** If the
   word is different, the object is gone. The field's map blocks use this. The battle's gauges use
   it since 2026-09-29.
2. **They look up the name again each time, with a new `ClassNames`.** The app poll does this
   (`gen6.apps`), because a kept answer can be another app's by now.
3. **They do not check.** The talk objects are kept for the session once found.

### Where a module goes

The game chooses where each module goes, and the emulator maps it exactly there. Neither the
3DS nor Azahar moves an address at random (there is no ASLR). The game puts a module directly
after the modules that are already loaded, from the bottom of the memory it keeps for them. So:

- A module that the game always loads alone, as the field's and a battle's are, always goes to
  the bottom.
- A module loaded on top of another goes after it. The Box's module goes after the PC menu's.
- The same module can go to different places. The name entry's keyboard was seen at three.

Each module starts with a **header** that gives its name and size, and a link to the next
module loaded. At each app start `Discovery.learn` follows those links, in a few reads. It reads
a module's code the first time it sees that module, and keeps where each wanted vtable is inside
it. When the module comes back at another address, its vtables move by the same amount, so no
read is necessary (spike 140).

## 6. Finding objects: sweeps and the heaps' watch

### A whole sweep

A **whole** sweep reads all of the heap: about 11,700 pages of 1 KB, in about half a second. It
finds every object of the wanted classes. It is slow, so pa3ds does it only:

- when pa3ds starts,
- 1.5 s after another app starts (not when ORAS's field comes back),
- when a reader asks, because it waits for an object that only a sweep can find,
- every 30 s, as a net for what the other ways did not find.

A whole sweep reads in chunks. Between chunks pa3ds polls the dialogue, so a page of text is not
late (ADR-0004).

### The heaps' watch

The game gets its memory from its heaps (`gfl::heap::CTRExpHeap`). Each heap keeps a list of its
**blocks** in use, and puts a new block at one end of the list. So every 0.1 s pa3ds reads the
head of each heap and walks only the blocks that are new since the last look (`pa3ds.core.heaps`).

pa3ds then sweeps only the new blocks: a **partial** sweep. It sweeps each block again 0.5 s and
2 s later, because the game often takes a block first and builds objects in it later (ADR-0034).

A partial sweep tells what is new. It does not tell what is gone. So a reader adds what a partial
sweep finds, and forgets nothing because of it. A whole sweep tells everything, so a reader
replaces what it knows with the result.

### Objects found through other objects

Some objects have no class of their own, or are not in a new block. pa3ds reaches them through an
object that it has found:

- A layout's panes, from the layout (a tree of pointers).
- A **choice box**, through the input device manager that it holds.
- The DexNav's **hidden record**, through the player's work (`field::PlayerWork`).
- The battle's layouts, through a pool that the message window points to.

## 7. Reading memory cheaply: the RPC, pages and phases

Each RPC read is one UDP message, with at most 1,024 bytes of answer. One read takes about 70
microseconds. When pa3ds sends many reads before it waits for the answers ("pipelined"), each
read costs about 24 microseconds. So pa3ds reads in large groups.

`memory.PageCache` reads memory in pages of 1 KB. A reader asks for bytes; the cache gives them
from its pages, and reads a page only once in a poll. Each part of a poll (dialogue, screen,
battle) is a **phase**. At the start of a phase, the cache reads again, in one group, all the pages
that the phase read the last time. Then most reads of the poll need no message at all.

The loop does not do everything on each tick:

| What | How often |
|---|---|
| Dialogue | each tick, 60 times a second |
| The heaps' watch | each 0.1 s |
| Screen text, menu cursors, the battle's HP | each 0.2 s |
| A whole sweep | see [section 6](#6-finding-objects-sweeps-and-the-heaps-watch) |

When the emulator loads a file, each read takes 2 to 20 ms. Then pa3ds delays the work that can
wait, for 1.5 s at most (`app.Busy`). Dialogue is polled all the same.

## 8. Readers: from objects to words

A **reader** owns one kind of screen or state. Each poll, it reads its objects and reports what
changed. The runtime says the change.

### Dialogue

A **message window** (`print::MsgWin`) keeps the whole message in a `gfl::str::StrBuf`. It shows
one **page** at a time. For each line of the page it has one **line object**, a
`nw::lyt::TextBox`, and the game types the line into it, one character at a time.

pa3ds says the whole page when the first character appears (ADR-0011). It finds the page in the
message, because the typed text is the start of one of its pages. Many windows keep old text, so
pa3ds reads only a **live** window: one whose line objects are all still TextBoxes and are not in
use by another window.

### Screens and menus

Every screen is a NintendoWare **layout**: a tree of **panes**. Some kinds of pane:

- `nw::lyt::TextBox`: words. This is **screen text**, said when it comes on screen and when it
  changes (ADR-0013).
- `nw::lyt::Picture`: an image. Some buttons are only a picture. pa3ds then says the words in the
  name of the image's file (`fight_button.bclim` is "fight").
- `nw::lyt::Window`: a box. The red corners of a menu's cursor are a window.
- `nw::lyt::Bounding`: an area that the player can touch.

A pane is on screen only if it and all the panes above it are visible. A menu says only the item
under its cursor (ADR-0012). The cursor is found by its shape: for example, a window alone in its
layout, placed exactly over one item.

### The battle

The battle's HP box has one **panel** for each side. The words in the panel give the level and the
player's HP. Two more objects give the rest:

- The **gauge** (`btl::BtlvGaugeSelf`, `btl::BtlvGaugeBase`) tells which Pokémon the panel shows,
  and whether its HP is still counting.
- The **name plate**, a `StrBuf` such as "Blaze♂", gives the name with its gender sign.

pa3ds says a side when the gauge has stopped counting and the words are the same for two polls
(ADR-0015). Section 12 tells what went wrong with this, and why.

### The field

On the field, pa3ds reads:

- The **move models**: the people and the player, in a pool of 32.
- The **zone data**: the zone's file from the ROM, which the game keeps in the heap. Its
  **entity table** lists the people, signs, doors and triggers, with their cells.
- The **chunks**: the ground, 40 by 40 cells each. The **cell table** says, for each cell,
  whether the player can walk there (bit 0), whether it is water (bit 1), and what kind of cell it
  is (the top byte).

From these, the navigation keys list what is near, and **P** finds a way to it
(`core.pathfind`).

### The game's own words, from the ROM

Memory often has only a number: a zone number, a place name's index. The words are in the game's
files. pa3ds reads the player's own ROM file: the 3DS container (NCSD), the game's partition in it
(NCCH), its files (RomFS), the game's archives (GARC), and the text files in them (ADR-0008).
pa3ds finds the ROM through the list of files that Azahar opened recently. It never ships any
game data.

### Names of our own

The game never names some things: a person in the street, a sign. pa3ds can give them a **name of
our own**, from a file for each game and language. The player's own names come first (ADR-0023).

## 9. Speaking and keys

**Speech.** pa3ds sends each line to the player's screen reader through Prism (the `prismatoid`
package). Each line has a **speech category**. Most lines go into the screen reader's queue. The
answer to one of pa3ds's own keys interrupts, because the key did not reach the game
(ADR-0010).

**Keys.** A low-level keyboard hook takes the keys that pa3ds uses, only while Azahar's window is
in front, and keeps them from the game (ADR-0022). The **keys file** (`user\keys.toml`) says which
keys they are. Every key that pa3ds takes is a button that the player cannot use in the game.

## 10. Two engines, two profiles

pa3ds keeps what is special to a game in a **profile** (`pa3ds.games`):

- **Gen 6** reads X, Y, Omega Ruby and Alpha Sapphire. They use one engine, so one set of readers
  does all four (`games.gen6`).
- **Sun and Moon** use another engine (gfl2), with other class names and other layouts
  (`games.sm`). Sun also keeps graphics buffers among its objects, and a read of one of them
  stops the emulator. So pa3ds reads Sun only in its small blocks of 4 KB or less, which it gets
  from the **heap table** (ADR-0035).

What both engines share is in `games.common`: the layouts, dialogue, cursors, the map's list.

## 11. How we know that it works

The game is closed source, and nobody can look at the screen for Orion. So every claim needs
evidence that a test or a log can show.

- **A spike** is a short script that measures one thing in the running game (`spikes/`). What it
  measured is a **finding** (`docs/findings/`). The numbers in the code comments ("spike 66") are
  the findings.
- **A movie** is a recording of the buttons that a player pushed (a `.ctm` file). Azahar replays
  it exactly, so anyone can reach the same game state without playing.
- **The rig** (`spikes/rig.py`) starts a changed copy of Azahar that replays a movie in the
  background. `spikes/regress.py` replays movies with pa3ds reading them, and compares what pa3ds
  said before and after a change. **Pins** stop a replay from going another way when the PC is
  faster or slower than the recorder's.
- **A snapshot** is memory captured at one moment. Tests replay it through a reader and check
  what the reader says. Most unit tests use a fake memory with objects written into it.
- **The GDB stub** answers "which code wrote these bytes?" with a watchpoint. It is a tool for
  research, never a part of pa3ds itself (ADR-0002).

## 12. A worked example: the late HP line (2026-09-29)

This example shows most of the parts working together.

**The report.** Orion heard the battle's HP after "fight", and sometimes later.

**The evidence.** The logs of old replays showed the time from "What will Blaze do?" to the first
HP line. Before 2026-09-26 it was 0.1 to 0.3 s. From 2026-09-26, when the heaps' watch arrived,
one battle in three was 1.45 to 1.9 s late.

**The cause.** Spike 138 logged each step. The battle found a gauge as "the object that points to
this panel's level TextBox". It could ask for that search only when the panel was on screen, about
0.6 s before "What will Blaze do?". The search was a whole sweep of about 0.5 s. Then the panels
went off screen for 1.45 s while the command menu came up. If the sweep ended too late, the HP
line waited for the panels to come back.

**Why it was built like that.** On 2026-09-17 the gauge's class was in the battle's DLL, and
pa3ds could not learn a DLL's classes. So a search by class was not possible. From 2026-09-23,
`Discovery.learn` could learn them, but nobody changed the battle.

**The fix.** pa3ds now finds the gauges by their class, in the partial sweeps, as the battle builds
them. That is about 8 s before the panel comes on screen.

**The mistake on the way.** The first version checked each gauge's name with gen 6's
`ClassNames`. That cannot name a class of a DLL loaded later (section 5), so it refused every gauge,
and no HP was said at all. The tests did not see this, because their fake classes were in the
executable. Now the tests put the gauge classes after the executable, as the game does. The
gauges are checked by the vtable that they had when a sweep found them, as the map blocks are.

## Glossary

**Address.** A number that tells where a byte is in memory. On the 3DS an address is 32 bits.

**Aligned word.** Four bytes that start at an address that is a multiple of 4. Pointers and vtable
pointers are always aligned, so a sweep looks only at aligned words.

**Anchor.** An object that pa3ds can find without an address, and that leads to other objects.

**App.** One of the game's screens as the game runs it: the field, a battle, the Bag. Each is a
process object. Its class name says which app runs.

**ADR (Architecture Decision Record).** A short document that records one decision, why it was
made, and what was rejected. In `docs/adr/`. Binding on the code.

**Block.** One piece of memory that a heap gave to the game. It has a small header that says its
size and links it to the next block.

**Cache.** A copy of data that pa3ds keeps, so that it does not read or compute it again.

**Class name.** The name of a C++ class, as the game's RTTI stores it. Mangled: `N5print6MsgWinE`
is `print::MsgWin`.

**CRO.** The 3DS's file format for a DLL. The game's files include `DllBag.cro` and
`DllSangoZukan.cro` (the Pokédex).

**DLL, module.** Code that a program loads while it runs, not at the start. The game loads one
for each app and replaces it with the next app's module later, often in the same memory.

**Emulator.** A program that runs another machine's software. Azahar runs 3DS games on a PC.

**Executable.** The game's main program (`code.bin`). It is loaded at the start, at `0x00100000`,
and stays there for the whole session.

**FCRAM.** The 3DS's main memory. The RPC cannot see it directly, only as the game's addresses.

**Fingerprint.** A value that identifies exactly which game and version runs.

**GARC.** Game Freak's archive format: one file in the ROM that holds many small files.

**GDB stub.** A part of Azahar that lets a debugger stop the game and set watchpoints.

**Heap.** An area of memory from which a program takes memory when it needs it, and to which it
gives the memory back. The game has many heaps, each a `gfl::heap::CTRExpHeap`.

**Linear heap.** Memory that the graphics chip can read as well as the game.

**Mangled name.** A class name in the form that the compiler stores it, with lengths and markers.

**NCSD, NCCH, RomFS.** The layers of a 3DS ROM file: the whole cartridge image, one partition of
it, and the file system inside the partition.

**Object.** One instance of a C++ class in memory. If the class has virtual methods, the object
starts with a pointer to the class's vtable.

**Page.** 1,024 bytes of memory: one RPC read, and one unit of pa3ds's page cache.

**Pipelined.** Many requests sent before any answer comes back, so the waits overlap.

**Pointer.** A word whose value is the address of something else.

**Poll.** One look at the memory that a reader watches. The loop polls about 60 times a second.

**Process manager.** `gamesystem::GameProcManager`, the object that points to the app that runs
now.

**Profile.** Everything special to one engine: its class names, its structures, its readers.

**ROM.** The game's cartridge image as a file. pa3ds reads the player's own, decrypted copy.

**RPC.** Azahar's interface on UDP port 45987, which lets another program read the game's memory.

**RTTI (run-time type information).** Data that the compiler puts beside each class so that the
program can know an object's class while it runs. It gives pa3ds the class names.

**Snapshot.** Memory captured at one moment, which a test replays through a reader.

**Spike.** A short script that measures one thing in the running game. Its result is a finding.

**Sweep.** A search of memory for objects of the wanted classes, by their vtable pointers.
**Whole**: all of the heap. **Partial**: only the blocks that are new.

**TextBox, Picture, Window, Pane.** Parts of a NintendoWare layout. A pane is one node of the
tree; a TextBox holds words; a Picture shows an image; a Window draws a box.

**Tick.** One turn of pa3ds's main loop.

**Title id.** The number that identifies a game. Omega Ruby's ends in `0011c400`.

**Typeinfo.** The RTTI record for one class. The word before a vtable points to it, and it points
to the class's name.

**UDP.** A way to send short messages over a network, with no connection. The RPC uses it.

**Validator.** A check that proves that a found object is the real, living thing and not an
accident or a dead copy.

**Vtable (virtual table).** A table of pointers to a class's virtual methods. There is one for
each class, in the code, and every object of the class starts with a pointer to it. That is why
pa3ds can find objects by their class.

**Watchpoint.** An instruction to a debugger: stop the game when it writes to this address.
