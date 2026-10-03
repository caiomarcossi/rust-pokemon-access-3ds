# 35. Sun is read only in its small allocations, and to the byte

- **Status:** Accepted
- **Date:** 2026-09-26, amended the same day for the menus (spike 112), the field (spike 113) and
  the walk's cost (spike 116), and on 2026-09-27 for the heaps' watch (spike 123)
- **Deciders:** Orion

## Context

pa3ds finds what it reads by sweeping the heap for vtables (ADR-0003, `discovery`), a page at a
time. In ORAS the heap is ordinary memory at `0x08000000`, and nothing else is there. Sun keeps
nearly everything in linear memory from `0x30000000`, and the GPU's own buffers sit in that memory
too: in the gaps between gfl2's heaps, and inside some of them. Azahar writes a buffer the GPU has
cleared back to memory when anything else reads it, and that asserts (`DownloadFillSurface`,
`rasterizer_cache.h:1172`) and the emulator dies. The code is shared by both renderers. The
user's own Azahar runs Vulkan. Over sun_1.ctm (spike 111):

| Read | Renderer | Emulator |
|---|---|---|
| Probing the address space past VRAM | Vulkan | died at 8 s, `0x1F5FC000` |
| Probing the linear memory | Vulkan | died at 40 s, `0x342D8000` |
| Sweeping the linear memory whole, every 3 s | OpenGL | died at 189 s, `0x33430000` |
| Sweeping every heap the heap table lists | OpenGL | died at 268 s, `0x333F4800` |
| Sweeping heaps 5 and 0x0b, where every dialogue object was, every 2 s | OpenGL | 120 sweeps, whole movie |
| The same | Vulkan | died at 277 s, `0x31C40000`, in a 529 KB block of a heap inside heap 5 |

(The OpenGL rows were run as Vulkan: an OpenGL run had left the fork's config marking the renderer
as default, and Azahar's default on Windows is OpenGL. The fork's log says which renderer started.)

## Decision

In Sun, pa3ds **never reads memory it has not been told is a small allocation**:

- The heaps are walked, not swept: from the game's **heap table**, the tree of heaps under heap 5
  and heap 0x0b (`sm.heaps.DIALOGUE_HEAPS`), by their `nn::fnd` heads, and each expanded heap's
  used blocks by their 16-byte headers.
- Only a block of 4 KB or less is swept (`sm.heaps.SMALL`). The dialogue's objects are 0x120 bytes
  or less; a buffer the GPU renders into is not that small.
- **Every read is to the byte.** The walk reads each small block exactly, the last read of each
  cut short at the block's end, and Sun's readers read through `RpcMemory` rather than the page
  cache, so a read of an object never takes in the block beside it.

**Amended for the field and the menus, 2026-09-26** ([spike 113](../findings/field-and-rom.md),
[spike 112](../findings/screens-and-cursors.md)). The move models, their manager and the area the
field has loaded are small allocations of **heap 3**, a heap of 27 MB that holds heap 0x0b; the
field's other objects are under heap 5 already. So:

- **Heap 3 is walked too** (`sm.field.HEAPS`), under the same rules: its heads, its used blocks'
  headers, and its blocks of 4 KB or less, to the byte. Spike 112's capture had read every small
  block of every heap every 2 s for 100 s of sun_3.ctm, and nothing went down; neither did 33
  walks of heaps 3, 5 and 0x0b over sun_1.ctm, nor two runtime replays of sun_3.
- **A pointer is followed only into a small block the last walk found**, and only as far as that
  block's end (`sm.field.SunFieldReader`): the field, the area loaded and the models are reached
  from objects a sweep found, and none of them has been read anywhere else. Not "a block a sweep
  read": when this was decided, a sweep between whole sweeps read only the blocks near what was
  found before.
- **The menus' readers keep to the same rule**:
  ORAS's layout, screen text and cursor readers follow pointers from a layout's work down its
  tree of panes to their words, through `sm.heaps.InBlocks`, which reads only inside a small block
  the last walk found and reads anything else as zeros, a pane with no class that the readers
  pass over. A pane is read only as far as the smallest pane goes, a TextBox as far as its text
  position (`sm.lyt`).
- **A big block is not read even where a pointer leads to it.** The actors point at their
  records in the area's event data, a block of 8.4 KB; the ROM holds the same file
  (`sm.zones`), and the actors hold their records' places themselves.

**Amended for the walk's cost, 2026-09-26** ([spike 116](../findings/memory-and-discovery.md)).
On Route 1 a sweep was 1.1 s of every 2 s, and nothing but dialogue was polled meanwhile: the
walk read 13,600 headers in 4,221 rounds, then the sweep 14,000 bodies. A read costs about 28 us
and a round 140 us more. So:

- **A body is read with the walk**, in the round after its header, together with the next
  block's header where that follows it, as most do; a sweep is the walk (`sm.heaps.Heaps.walk`,
  `Discovery`'s `walk`). Every read is still inside a small block or a header.
- **Heap 0x21's own blocks are not walked** (`sm.heaps.UNREAD`); the heaps inside it are. They
  were 4,219 of the 13,000 blocks, the rendering engine's animations, textures and command
  buffers, and over 58 of spike 112's captures no class pa3ds reads was in one.
- **The field, the one object pa3ds reads there, is read through its own block's header.** Its
  pointer is a live `Fieldmap::State`'s, its block's header is 0x30 before it, and it is read
  only where that header, read to the byte, says 'UD' and a small block that holds it
  (`sm.field.SunFieldReader`): the block a walk of 0x21 would have found.
- **Keys, screens and the field are polled between a walk's chunks**, as dialogue is (`app`),
  reading what the last walk found until this one is done.

**Amended for the heaps' watch, 2026-09-27** ([spike 123](../findings/memory-and-discovery.md)).
Sun is under ADR-0034 as ORAS is: gfl2's heads are laid out as ORAS's and a heap object's +0x08
counts its blocks in use, so the heaps under the walk's roots are looked at ten times a second and
only what they have just allocated is swept, and the walk above is the whole sweep, made at the
start, 1.5 s after another app starts (`GameSys::GameProcManager` +0x10, `sm.apps`), when a reader
asks, and every 30 s. What this decision asks of every read, the watch keeps to:

- **A block larger than 4 KB is known by its header alone**, never read, and told from the block
  that was there by its size (`pa3ds.core.heaps`, `largest`). Heap 0x21's own blocks go unwalked, told
  by its id as it is adopted, since it is made after the watch starts; the roots' siblings, the
  GPU's and the sound's heaps, are not watched at all (`under`).
- **A new small block is swept to the byte** as it appears and again 0.5 s and 2 s on, if it is
  still one that may be read (`sm.heaps.to_sweep`, `Discovery.sweep_spans`).
- **What may be read is every small block the last walk found and every one allocated since, less
  any a newer block lies over** (`sm.heaps.SmallBlocks`): a small block freed and its memory taken
  again as part of a large one is out of bounds the look the watch sees it, and the readers take
  the change in without waiting for a sweep. Such a block is forgotten by the watch, so one taken
  again where it was, as it was, is handed over as new.
- **The heads are read in one pipelined burst**, each to the byte (`memory.ExactCache`): read one
  after another, 170 heads took 12.5 ms a look.

## Consequences

- Finding things over sun_4.ctm (200 s) took 3.8 s of sweeps and 7.1 s of looks, polling included,
  where the walk every 2 s took 27.2 s; over sun_1.ctm (390 s), 5.3 s and 4.1 s against 34.5 s.
  Every page, speaker's name, menu and cursor the walk heard was heard, and pauses in polling over
  100 ms were 8 and 13 against 8 and 7 (spike 123).
- A walk of Route 1 is 10,600 reads in 1,849 rounds, about 550 ms, with keys, menus and the field
  polled through it (spike 116). Before that amendment:
  a sweep read about 1.5 MB, where heaps 5 and 0x0b whole were 43 MB. Walking the blocks' headers
  is a read a block, sent a block of every heap at a time. With heap 3 it is 1.6 MB, 8,251 blocks
  under 169 heads, where heaps 5 and 0x0b were 995 KiB in 5,740 at the same moment (spike 113).
- **An area load allocates thousands of blocks at once**, and the watch reads their headers one
  at a time: a look can take 2 s, polling as it goes every 256 headers.
- **Heap 6 is walked for the app running**: the manager points at a record there whose +0x0c is
  the process, two small blocks in all (spike 116, `processes`). A process is named the poll its
  blocks can be read, a look after it starts: `movie::MovieProc`, `App::TitleMenu::Proc`,
  `Field::FieldProc` and `app::config::ConfigProc` over sun_3.ctm, 0.2 to 1.3 s after each.
- Every poll's reads are fetched ahead in one burst as the next poll of its kind starts, kept to
  the small blocks that may be read now (`memory.ExactCache`, `within`): dialogue polls 8.5 s to
  6.1 s over sun_4.ctm's 120 s, screen polls 8.8 s to 4.8 s (spike 60 against Sun, 2026-09-27).
- **A block a walk no longer finds is forgotten by the watch** (`SmallBlocks.walked`), so one
  taken again as it was is handed over as new. Remembered, the field's layouts, freed under the
  save prompt and taken again as they were, were never handed over, and the X menu's cursor went
  unread at one opening in four on sun_3.ctm until the next walk.
- **A window built in another heap, or in a block over 4 KB, is not found.** A screen whose
  dialogue goes unsaid in Sun may be using a heap of its own: measure where its window is, and add
  the heap.
- **Nothing in heap 0x21's own blocks is found by a sweep, and a pointer into one reads zeros**,
  the field excepted. A screen that goes unsaid where the field is up may be keeping something
  there: `spikes/116_sun_walk_cost.py owners` says which heaps held what.

## Rejected

- **Sweeping everything, or every heap, or the dialogue's heaps whole.** Each took the emulator
  down; see the table.
- **Asking the player to run OpenGL.** OpenGL died too, and it would make pa3ds the reason a player
  changes their emulator's renderer.
- **Leaving out the pages that crashed.** They moved from run to run.

## Open

- **Chains from a global to the windows**, as spike 99 began for ORAS, would reach them without a
  sweep at all.
- The Bag, the party and every screen no movie has reached have not been looked for in Sun, so
  which heaps they use is unknown. The battle's are under heap 5 (spike 124): its gauges, records
  and Pokémon are reached with no change to the walk.
