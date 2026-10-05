# The instruments

Speech out, the emulator in, and the rig that replays a recording so a finding can be checked
twice.

Each entry is one spike: the question, what was measured, and what was built from it. They are in
the order they were done, because a later one often corrects an earlier one — and says so.

[← all findings](README.md)

## Spike 1 — speech, settled

2026-09-10, heard out loud by the user:

- `prismatoid` reaches NVDA. Prism registers 12 backends, ranks NVDA highest (priority 103), and
  `Context.acquire_best()` returns it. `speak`, `speak(interrupt=True)`, `braille`, `output` and
  `stop` all work; four test phrases were audible in the user's own NVDA voice.
- The package name is `prismatoid`; the module it installs is `prism`.
- **The NVDA backend cannot report whether it is still speaking.** `supports_is_speaking`,
  `supports_pause` and `supports_resume` are all false. pa3ds can therefore never wait for speech
  to finish. NVDA does its own queueing when `interrupt=False`, so the speech queue is NVDA's, not
  ours; all pa3ds decides is the interrupt flag per speech category. `stop()` works, so cutting
  speech on A is safe.
- **The NVDA backend exposes no rate, volume, pitch or voice control** — every one raises
  `PrismNotImplementedError`. This is the right default (the player's own voice, untouched) but it
  means the optional dedicated voice cannot go through NVDA.
- SAPI (27 voices, including the Natural ones) and OneCore (9 voices) both support voice, rate,
  volume and pitch in full. NVDA and SAPI can be acquired and held **at the same time**, so the
  dedicated voice is a second backend running alongside NVDA, not instead of it.
- Prism's own priority order is NVDA (103) > OneCore (98) > SAPI (97).
- `Context.exists(id)` means the backend is *registered*, not that the screen reader is running —
  JAWS and ZoomText also report true here. `features.is_supported_at_runtime` is the real check.

## Spike 2 — RPC, settled

2026-09-10, measured against ORAS running from a save in Littleroot:

- Wire format, from `src/core/rpc/packet.h`: a 16-byte little-endian header of
  `version, id, packet_type, data_size` followed by up to 1024 bytes. `CURRENT_VERSION` is 1
  (2 since azahar-emu/azahar#2631, below).
  `ReadMemory` (1) takes `<u32 addr, u32 size>`; `ProcessList` (3) takes `<u32 start, u32 max>` and
  returns `<u32 count>` plus 20-byte `ProcessInfo` records of `<u32 pid, u64 title_id, char[8]>`;
  `SetGetProcess` (4) takes `<u32 op, u32 pid>` and returns the selected pid when `op == 0`.
- **The per-packet read cap is exactly 1024 bytes.** Confirmed empirically: 1024 succeeds, 1025 and
  above come back with a zero-length reply.
- **The memory map, measured rather than assumed** (2026-09-12, ORAS at the save state). Fifteen
  regions, 56 MiB in all: code at `0x00100000`-`0x00728000` and a scrap at `0x01308000`; **heap in
  four blocks — `0x08000000`-`0x08338000`, `0x085e4000`-`0x08700000`, `0x08804000`-`0x08808000`
  and `0x08968000`-`0x08da0000`**; a page of IO at `0x10000000`; linear heap in five blocks from
  `0x14000000` to `0x16b48000` plus `0x1bc00000`-`0x1bf00000`; VRAM at `0x1f000000`. Spikes 5, 6
  and 7 swept `0x08000000`-`0x08800000`, which is the *first* heap block only and misses the
  largest of the four — the one holding `0x08c3bd54`, the manager every text pane points at.
  `snapshot.regions()` measures this at startup instead; nothing should type a span in again.
- **A running emulator answers something like 35,000 page reads and then stops.** Not a bad
  address: VRAM was blamed and cleared, reading all 5,740 of its pages on its own without a
  problem, and the game kept rendering throughout. Whatever comes last in a long sweep is what
  fails, and neither retrying nor backing off nor pacing in chunks helps — the server drains its
  queue alongside a game running at sixty frames a second and falls behind for good. Earlier
  60 MiB captures worked because they were taken **paused**, which is what AGENTS.md already asks
  for. Sweep what a question needs, not everything.
- **Latency is flat in read size.** 4 bytes and 1024 bytes both round-trip in ~73 us sequentially.
  Packet count is the only cost, so every read should ask for the full 1024 bytes.
- Sequential reads: ~73 us each, ~13,600/sec. Pipelined — fire a batch, then match replies by the
  header `id` — ~24 us each at a batch of 8, ~41,900/sec, and it plateaus there. Zero packet loss
  at every batch size up to 256.
- **60 Hz is comfortably affordable.** A 2 ms slice of each 16.67 ms frame buys about 84 reads,
  which is 84 KiB of watch regions per tick.
- `ProcessList` gives the game away for free: one process, `pid=11`, `name='sango-1'`,
  `title_id=0x000400000011c400`. So the game and region need no fingerprinting; only the version
  does, and 8 MiB of code is readable at `0x00100000` to hash a slice of.
- Reads work with no process selected (`SetGetProcess` reports `0xffffffff`) and return identical
  bytes after `set_process(11)`. pa3ds selects explicitly anyway rather than lean on that.
  **With no process running, the same read crashes Azahar**: with nothing selected the server
  reads through the kernel's current process, a null pointer until the game's exists. The server
  is part of the emulated system, so a new one, with nothing selected, answers whenever the system
  starts again: another emulator on the port (2026-09-25), and **the game restarting itself**.
  Leaving Live Competition or Mystery Gift on the title menu jumps to the game's own title, which
  Azahar cannot find on the SD card for a ROM loaded from a file, and it resets the game instead
  (`Failed to find title ... resetting current title instead`). The old server is gone from the
  shutdown's start until the new system's init, about 0.3 s, and `read_many` resent its stalled
  reads into the new one: Azahar crashed 0.7 s after leaving Live Competition, its last log line
  `No target process selected` (movie13, both in Orion's session and in a replay, 2026-09-29).
  `Rpc` now asks whether the game is still selected before a read that follows a stall or 50 ms
  of quiet (`QUIET`), and the same replay went through both restarts, attaching again each time.
  **A module read while it loads crashes Azahar too** (movie5, 2026-09-30): pa3ds read a new
  module whole as its header first appeared, while Azahar was still mapping and fixing it and
  freeing the part past what it keeps (`DllBattle` at `0x006f3000`, fixed to `0x007e0000`, read
  to `0x007f3000`). Two of four replays crashed as Wally's battle loaded its module, and three
  of three on main did not, the timing being the difference. `Discovery.learn` now reads a module
  only when the look before listed it the same (name, place, size and magic), a poll later; three
  replays after followed their recording, the battle's classes learned as before.
  **That was not the cause, or not the main one** (spike 149, 2026-09-30). Movie4 crashed at its
  first battle on main, with that change in, 5 times in 12 replays from frame zero, and never in
  20 with no picture named by its image (ADR-0036, which landed the same day as the module
  reads). Logging each image read, every crash came reading a 128 KB image in video memory,
  `0x1F5A0000`, as the battle began, and the read after it, which every surviving replay made
  0.02 s later, never came. **Reading the GPU's memory makes Azahar flush the GPU's copy of it,
  from the RPC's own thread**: `ReadBlockImpl` calls `RasterizerFlushVirtualRegion` for a
  rasterizer-cached page (`memory.cpp:213`), and the rasterizer cache's `FlushRegion` downloads
  every surface the GPU has drawn into there (`rasterizer_cache.h:1305`), with no lock, while
  the emulation thread runs the game's GPU commands on the same cache. An icon the GPU draws
  from is never dirty, so a read of one downloads nothing, and every texture pa3ds names is 2 or
  4 KB: an image larger than 4 KB is no longer read (`textures.MAX_IMAGE`), and movie4 and
  movie5 each went through their battles 8 times of 8. What is left is `FlushRegion` looking up
  its dirty regions on the wrong thread, a race far narrower, which the linear heap's sweeps
  have run every 30 s for weeks. It is upstream's, unchanged on its master of 2026-09-30, and
  reported by nobody until Orion filed it as azahar-emu/azahar#2619 (2026-09-30).
  **Confirmed without pa3ds** (spike 149's `poc`): a client of its own reading all of VRAM over
  the RPC, beside movie5 from frame zero, crashed the fork 10 times in 10, each within 13 passes,
  about 5 s after the game started, during the logos. The same fork with RPC reads made through
  `ReadBlockImpl<true>`, which skips the flush (a throwaway fork branch, `test/rpc-unsafe-read`,
  deleted since), survived 10 in 10, about 375 passes each in 60 s. On Orion's own Azahar
  2126.1.1 the proof of concept crashed it in 3 s, before its first pass. The issue's text and
  attachment are in `user/azahar-issue/`.
  **Fixed upstream by azahar-emu/azahar#2631** (merged 2026-10-04): built with
  `ENABLE_SCRIPTING_SYNC`, on by default, the server hands every request that touches the emulated
  system to the emulation thread, which serves them between CPU slices and while the frame limiter
  waits, so no read runs beside the GPU's commands. The fork, rebased onto it (2026-10-05), survived
  10 trials of 10: the proof of concept, made to speak version 2, read all of VRAM beside movie4
  from frame zero for 60 s (the old fork crashed in the logos every movie opens with), 184 to 197
  passes a trial, about half fix A's, since every read now waits its turn on the emulation thread: a
  read costs about 94 us in turn and 29 us pipelined with movie4 running, where spike 2 measured 73
  and 24. Players' Azahar keeps the race until a release carries the fix, so the 4 KB image cap
  stays. The same change made the protocol **version 2**, and a server of either version answers a
  packet of the other with an empty reply; `Rpc.processes` asks with 2 and then with 1. Version 2
  also raises the read cap to 32 KiB, which pa3ds does not use, since version 1 refuses it, and
  **answers nothing while emulation is paused**: the emulation thread waits for the resume without
  serving requests, and pa3ds logs that the emulator stopped answering until it does. The GDB stub,
  which upstream reworked in the same change, still serves pa3ds's watchpoints: the party menu's
  draw order, read through read watchpoints on movie22, came out read for read as on 2026-10-03.
- **Physical FCRAM is not visible through the RPC.** `0x20000000` and `0x28000000` read as zeros.
  Reads go through the selected process's virtual address space. Probing 1 KiB at every MiB
  boundary found 41 MiB non-zero across 15 ranges: code at `0x00100000-0x00900000`, heap from
  `0x08000000`, `0x10000000-0x10100000`, the linear heap from `0x14000000` to about `0x16b00000`,
  `0x1bc00000-0x1bf00000`, and VRAM at `0x1f000000-0x1f600000`. That map is a sketch from a coarse
  probe, not the real VM layout — holes in it may just be sampling gaps.
- A full sweep of those 41 MiB at 1 KiB per packet is 41,984 packets: about 3.1 s sequential, or
  1.0 s pipelined. A startup anchor scan of everything readable therefore costs about a second.

## Spike 4 — the GDB stub as an instrument

2026-09-12, measured against ORAS at the save-select menu. The emulator-fork decision was reversed
by the user on this date, and reading Azahar's source is what produced all of this:

- **Write watchpoints work under the JIT.** Azahar added `PageType::MemoryWatchpoint`:
  `MemorySystem::RegisterWatchpoint` nulls the watched page's entry in the pointer array dynarmic
  reads, so every access to that page falls back through `Memory::Write<T>`, where the stub checks
  its breakpoint list. Only the watched pages pay for it. Citra could not do this, which is why
  "there are no execution hooks" in spike 2 was true then and is wrong now.
- Measured: attach round-trips in 12 ms, and the first watchpoint hit landed 0.28 s after
  resuming. A hit yields pc, lr, sp, r0-r15 and cpsr, plus memory reads over the same connection.
- **The stub is served from `System::RunLoop`.** A paused emulator answers nothing and does not
  even accept the connection. This is the exact opposite of the snapshot rule, and the two can
  never be held at once.
- **Detaching with `D` leaves the game frozen.** Azahar's handler calls `ToggleServer(false)`
  first, which resets the stub's `current_process`, and the `Continue()` immediately after it then
  returns having done nothing. Observed: after a bare `D` the game stopped advancing entirely, and
  a fresh connection sending `c` restarted it. Always resume, then detach.
- `?` must be the first packet: until it has chosen a process the stub refuses breakpoints with
  `E01`. It also halts the game.
- There is no single step — `s` answers `E5F`. Execute breakpoints are therefore effectively
  one-shot, and they work by writing a BKPT into the game's code, which pa3ds itself may not do.
  Watchpoints change no game memory at all, only the emulator's page table.
- Under the JIT the reported pc is approximate (`ARM_Dynarmic::HasSingleInstructionBreakAccuracy`
  is false). It lands in the right function. `use_cpu_jit=false` gives the exact instruction at
  interpreter speed.
- **Menu labels are resident after all**, as UTF-16LE in the heap around
  `0x08209000`-`0x08220000`, one copy each: `CONTINUE`, `MYSTERY GIFT`, `LIVE COMPETITION`,
  `Badges`, the player's name and `Littleroot Town` were all readable while the save-select menu
  was open. They are **freed when the screen closes** — a scan taken afterwards finds none of
  them. Spike 3's "no English game text is resident" was about the in-game main menu, a different
  screen, and still stands for it.
- The instruction that writes text is always a copy routine — memcpy at `0x0031fe78`, memset at
  `0x0017e80c` — so the pc alone says nothing. Walking the stack for values whose preceding
  instruction is a `bl` recovers the chain, and the frames that repeat across hits are the real
  callers: `0x00374a0c`, `0x00374224`, `0x0046c06c` and `0x0073f634`.

## Spike 5 — movie replay as the test rig

2026-09-12. The user recorded the Littleroot stretch up to the Poochyena fight
(`%APPDATA%\Azahar\movies\movie1.ctm`, 361,904 frames, ~100 minutes, recorded on build `fbd3fb0`):

- **The whole movie replays in about 7 minutes** with `frame_limit=1000`. Frame limit is a host
  throttle and does not touch emulated timing, so it is safe to raise for a replay and is not one
  of the frozen settings. Put it back to 100 afterwards.
- Launching: `azahar.exe -p "<movie>.ctm" "<rom>.3ds"`. **The ROM path must be the last argument**
  — Azahar only accepts the game as the final argument and otherwise exits with no window — and
  the short `-p` is required, because 2126.0's getopt pre-pass rejects `--movie-play` with
  "invalid option -- 'm'" despite listing it in `--help`.
- **A movie pins nothing about the state it started from.** The CTM header carries the program id,
  the emulator revision, the clock init time and the base tick count, and nothing about the save,
  the NAND or the SD card. This movie was recorded immediately after deleting `nand` and `sdmc`,
  so it starts from *no save at all* — title screen, NEW GAME, Birch's monologue, name entry.
- **Replaying it against an existing save desynced silently, and that is the dangerous part.** The
  first input landed in a different context (CONTINUE instead of NEW GAME) and every input after
  it was applied to the wrong game, yet the result still looked like a plausible session: wild
  battles, move text, a level-up. Nothing announced a desync. Coherent output is not evidence that
  a replay is faithful. The only proof is that the *expected* text appears in the *expected order*.
- **Replays run against an isolated, throwaway emulator directory.** On Windows Azahar prefers a
  `user` folder beside the executable over `%APPDATA%` (`SetUserPath` in `common/file_util.cpp`);
  `AppDataRoamingDirectory` goes through `SHGetKnownFolderPath`, so the `APPDATA` environment
  variable cannot redirect it and copying the install is the way. A copy of the install, kept
  apart from the player's own, is that: the same binaries, the player's `qt-config.ini` (so every timing-affecting setting
  matches the recording), and a `user` directory of its own. Clear `user/nand` and `user/sdmc`
  before each run to reproduce the recording's starting state. A replay then physically cannot
  touch the player's save, which is the real reason to do it this way.
- **Verified faithful this way.** The replay reproduced the recorded session in order: Mom's
  "They finished moving everything in downstairs", the TV report from in front of Petalburg Gym,
  Mrs. Birch's "We have a daughter about the same age as you", the Littleroot signpost, and Route
  101's "It's not safe to go out there if you don't have any Pokemon".
- **Playback completion pauses emulation** and opens a modal dialog. The stub goes deaf and the
  RPC reports zero churn, which looks exactly like a hang and is not one.
- **The RPC server logs every request and reply at Info**, about 1 MB/s while sweeping: 100 MB in
  102 seconds, then the log rotates. `log_filter=*:Info RPC_Server:Error` is now set in
  qt-config.ini and should stay set.
- Sweeping the 8 MiB at `0x08000000` costs ~0.3 s, so text can be logged at 2 Hz. A 60-second
  window caught Mom's dialogue, the Littleroot signpost, a Birch line, then a wild Wurmple battle
  with move text, a faint and a level-up.
- **Text buffers are reused within a screen.** Battle messages arrived one after another at
  `0x0846e4b4` ("Torchic used Scratch", "The wild Wurmple used Tackle", "The wild Wurmple
  fainted"), overworld lines at `0x08288538`. A reused buffer is what makes a watchpoint pay.
- **The stub stops listening after repeated connect/detach cycles.** A loop that opened a new
  connection per scan got "connection refused" after a few rounds, with the game still running and
  the RPC still answering. Hold one connection for a whole session instead of reconnecting.
- 79 watchpoint hits across the run separate the writers cleanly. Writes that leave **text**
  behind come from one unrolled copy routine at `0x0014e260`, `0x0014e288` and `0x0014e328`;
  writes that leave the buffer **empty** come from `0x003a6fb8`. The nearest stack frames above a
  text write repeat as `0x00419b0c` then `0x0034836c` in 72 of 79 hits. **Replicated on the
  faithful replay**, which produced the same writer addresses and the same two callers, so the
  finding does not depend on which run produced the text. Following that pair to a named function
  is where this picks up.

## Spike 6 — save states during a replay

2026-09-12. **They do not work as shipped, and the reason is a one-line bug:**

- Save states exist, are keyed by `(program_id, movie_id)` with 11 slots, and land in
  `<user>/states/<program id>.movie<movie id>.<slot>.cst` — about 15 MB each. Slot 0 is the
  quicksave slot. `Quick Save`, `Quick Load` and `Advance Frame` are real hotkey actions with no
  default binding, so the replay copy binds them to Ctrl+F1, Ctrl+F2 and Ctrl+F3. Saving works:
  Ctrl+F1 during a replay wrote a valid `.cst`.
- **Loading one during read-only movie playback is refused every single time.** `Movie::serialize`
  guards read-only loads with `recorded_input_.size() >= recorded_input.size()` and throws
  "Future event savestate not allowed in R/O mode". That test was written for *recording*, where
  the buffer grows as you play, so a state from the future is bigger. During *playback*
  `recorded_input` holds the entire movie from the first frame, so both sides are always equal and
  the guard always fires. Nothing about the save point matters.
- **Turning read-only off is not a workaround.** That path sets `play_mode = Recording`, bumps the
  rerecord count and calls `SaveMovie()`, which overwrites the movie file. Never do this to a
  movie that matters; work on a copy if it is ever needed.
- The other workaround is to close the movie (Tools > Movie > Close) before saving, which produces
  a movie-independent state that loads on a plain boot. It costs the rest of the replay's input, so
  it buys one frozen checkpoint per run, and `action_Close_Movie` has no hotkey — driving that menu
  by keystroke was tried and did not land.
- So the choice was a 10-minute replay per iteration, or fixing the guard.

## Spike 7 — the fork, built and working

2026-09-12:

- **The fix is one scoping change** and it works. The size test now only runs when
  `play_mode == PlayMode::Recording`; the timeline check after it is untouched and still proves
  the state came from this movie's input. Branch `pa3ds/playback-savestates`, commit `92f63d6`, in
  the fork's clone (https://github.com/KenzonYeoh/azahar).
- Verified end to end: saved a state at ORAS's name-entry keyboard, killed the emulator, booted the
  movie again from frame 0, loaded — and the heap addresses snapped back to exactly the ones the
  save had (`0x082a0d12`, `0x08324f74`, ...) rather than the slightly different ones playback had
  reached. The log shows `Begin load of slot 0` with no `Error loading` after it, where before the
  next line was always the R/O refusal. **Playback then continued from the load point**, through
  the TV report and Birch's "go stop by while I clean up around here".
- Cost per iteration is now a ~30 s boot plus a ~3 s load, against a ~10 minute replay.
- Building it: CMake 3.31.6 and Ninja 1.12.1 ship inside VS Build Tools but are not on PATH;
  `vcvars64.bat` puts them there, and it needs `%ProgramFiles(x86)%\Microsoft Visual Studio\Installer`
  on PATH first or it cannot find `vswhere`. Configure took 73 s, the build about 25 minutes, and
  the tree is 6.2 GB. Qt 6.10.3 is downloaded by CMake itself — but its bundled `aqt_config.ini`
  caps concurrency at 2 and its mirror list sent both archives to a Japanese mirror that stalled
  for 15 minutes with no error; running `aqt` by hand with `--base https://ftp.fau.de/qtproject/`
  pulled all 834 MB in under a minute.
- **Do not drive the rig with focus-based synthetic hotkeys.** `SetForegroundWindow` is refused
  when another process owns the foreground, and neither `AttachThreadInput` nor the ALT-unlock
  trick nor `SwitchToThisWindow` got round it. What works with no focus at all is `PostMessage` of
  `WM_KEYDOWN`/`WM_KEYUP` to every window of the process — but only for a shortcut bound to a
  **bare** key, because Qt reads modifier state from `GetKeyState` and a posted message cannot set
  it. The fork's portable config therefore binds Quick Save to `F1` and Quick Load to `F12`.
- The fork build carries its own `user` directory at `build/bin/Release/user`, so it is isolated
  the same way the stock replay copy is, and supersedes it as the rig.

## Spike 9 — the rig, and how to see the screen

2026-09-12:

- **Synthetic key presses do not drive Azahar, and never did.** Posting `WM_KEYDOWN`/`WM_KEYUP` to
  every window of the process appeared to work once and does not: with another process holding the
  foreground, Qt turns those messages into nothing. F1, F4 and F12 were each verified dead — F4
  failed to pause, proved by the GDB stub still answering, which it cannot do while paused. Taking
  the foreground is both refused by Windows and wrong, since it would interrupt the user's screen
  reader. **`--load-state <slot>` is in the fork instead** (commit `5d27287`), firing
  `Signal::Load` 35 s after boot, once the CPU loop is running and the movie is in playback.
- **Screenshots come from `PrintWindow` with `PW_RENDERFULLCONTENT`**, which captures the OpenGL
  window while it sits in the background, at the instant asked for rather than whenever a hotkey
  lands. Call `SetProcessDpiAwarenessContext(-4)` first: a DPI-unaware process gets a virtualised
  client rect while the render is physical, and the picture comes back silently cropped.
- This is what makes any claim about the display checkable. Neither the user nor an agent can see
  the screen, so "this text is on screen" was previously an assertion; now it is a picture.
- `spikes/rig.py` is the whole loop: boot, load, grab, stop. About a minute to a known moment.

## Spike 24 — the field's Yes/No, from a saved state

2026-09-13. The fork now saves or loads a state when a script writes a slot number into `pa3ds-
save-state` or `pa3ds-load-state` in its user folder (fork commit `5bb16ad`; `Rig.save_state`). A
10x replay saved slots 2, 4 and 5 on the page before the nickname question, the starter choice and
the gender choice (`spikes/18_prompt_study.py`, `spikes/19_text_owner.py`, run from them at real
speed).

- **Slot 2 landed a little late** — the replay had already answered the nickname question and was in
  the nickname keyboard — but half a minute on it reaches Birch's Route 103 question, the other
  silent prompt.
- **That prompt is not a list window.** No `MenuWindowController`, no `TalkWindow`, no
  `TalkCursorWindow` existed while it was up. Its labels are two new StrBufs, "Yes" and "No" in
  mixed case, where every list window so far said YES and NO.
- **The labels hang off a struct with no vtable,** 0x30 bytes at `0x08804724` in the small heap block
  at `0x08804000`: +0x00 8, +0x04 0x20, then two entries of (u32, StrBuf pointer) at +0x20 and
  +0x28 — (1, "Yes") and (0, "No"). It is built while the question is still typing and lives until
  just after the answer, so its existence does not mean the options are on screen. Nothing in it
  changed while it was up; this replay answered YES without moving the cursor.
- Not yet known: which object owns that label list, where its cursor is, and what says the options
  are showing. Being class-less, it will need an anchor other than a class name — what points at it,
  or the code that builds it.
- (Corrected by spike 26: the box is built when A is pressed on the question, not while it types.
  This study swept every half second and caught the box of the prompt before.)

## Spike 25 — why replays drift, and what a save state survives

2026-09-13. movie3 replayed at 10x five times, compared line by line
(`spikes/22_capture_movie3.py`, `spikes/22_compare_replays.py`).

- **Not dropped input.** The movie hands the game its buttons at each emulated pad poll, and its
  header fixes the clock and the base tick count, so running at 10x cannot lose a press.
- **Asynchronous file reads.** A RomFS read runs on a host thread; the game thread sleeps for the
  modelled delay *minus the host time the read took* and wakes whenever the host thread reports
  back (`File::Read`, `HLERequestContext::RunAsync` in the fork's source). Every room load takes a
  slightly different number of frames, more so at 10x, and the presses after it land early or late.
  One replay sat on one of Birch's pages for its last four minutes of input.
- **`deterministic_async_operations` makes the reads synchronous.** Upstream sets it only from the
  Debug dialog, for one session; fork commit `0bbd6d4` saves it, and `Rig.boot(deterministic=...)`
  writes it. Two replays with it on told the same story to the end, differing only in pages the
  reader missed at 10x. Both left the recording at the same place, Birch's front door, because
  movie3 was recorded with it off. With it off, one of three replays followed movie3 to the save.
- **A save state loads only into the build that saved it.** On another git revision the fork shows
  "Savestate version mismatch", which also pauses emulation, so the RPC sees a frozen heap.
  Rebuilding stranded every state. Taking the check out of the fork was refused by the session's
  permission classifier, so the states were saved again instead. (Fork commit `92fbdb6` took both
  checks out on 2026-09-14, with the user's approval: movie3's slot 0, saved by `5bb16ad`, loaded
  into the new build with a warning naming that build, and the replay ran on.)
- **Reading over RPC across a state load broke both.** A sweep running as the fork loaded slot 3 left
  "Begin load of slot 3" with no "Load completed", and the RPC server silent for good. The load also
  drops the server's selected process ("No target process selected"). The rig now reads nothing for
  45 s after launch. Whether a player loading a state under pa3ds does the same is untested.

## Spike 97 — replays in the background, and what a faithful one writes

2026-09-25, on the user's five new sequential movies (`spikes/97_replays_and_focus.py`,
`spikes/regress.py check`, fork commit `e172ecd`).

- **The fork took the foreground three times as it booted.** A WinEvent hook on the foreground
  caught the two game screens' own windows 1.3 s in, then the main window at 2.8 s. Qt's window
  log (`QT_LOGGING_RULES=qt.qpa.window=true`) says why the first two happen: the render widget's
  `windowHandle()->create()` makes each a top-level window titled "Azahar", which Qt moves inside
  the render window 20 ms later, and Windows activates it on the way. A `STARTUPINFO` asking for no
  activation reached only the main window's first show.
- **`-B` stops all three.** It shows every window with `WA_ShowWithoutActivating`, makes the render
  widgets' native windows with `winId()`, inside their parent from the start, and skips the two
  explicit activations. Three boots with Notepad, the terminal and VS Code in front: the fork never
  took the foreground. The game renders as before and `PrintWindow` still captures it.
- **Until the main window was saved maximized** (2026-09-25, evening; the user heard Azahar take
  the focus during a regression run, then go to desktop 5). `foreground --desktop` boots with `-D`
  as the rig does and logs each of the fork's windows as it is shown, with whether it is on the
  user's desktop: on fork `1d687d6` the main window was shown on the user's desktop and took the
  foreground 10 ms later in all five boots, three with `-D` and two without, and the foreground
  never came back to the user's window once it had gone to desktop 5. The fork's
  `qt-config.ini` had its main window saved maximized. Without that line it never took the
  foreground (two boots); Windows activates a window shown maximized whatever it is asked, and
  taking the maximized state off again after `restoreGeometry` did not help (three boots, all
  taken). **Fixed** in fork `fb22ac7`: under `-B` the main window's saved geometry is not
  restored, and `-D` moves it before `show()`, not after. Three boots on the maximized config:
  shown on desktop 5, never on the user's, and never the foreground. x_city replayed on it said
  the same as before, out of sync 0.
- **A faithful replay writes the recording's save, byte for byte, but for one word.** Two replays
  of movie4 and its recording agreed on all 483,328 bytes of the save but the four at `0x75E00`,
  which also differed between the two replays. movie1, movie2 and movie4 each wrote the save the
  user handed over, at 10x, with no out-of-sync line.
- **movie3 and movie5 drifted where the accelerometer came on.** A `.ctm` is one 7-byte entry an
  input read, the first byte its kind (0 buttons, 234 a second from boot; 1 touch, one with every
  button entry; 2 accelerometer), and the accelerometer is read only while the game has it on:
  movie5 from 106 to 126 s and 298 to 322 s into the movie, exactly the Bag opened to sell at the
  Mart and at the Center (against the user's log). The replays broke with `Expected to read type
  0, but found 2` right after `DllBag` loaded — at 106 s at real speed twice (deterministic reads
  on and off alike), at 298 s at 10x twice. movie3 broke at the second of its three (681 s) in
  both 10x runs, with the same 122,984 out-of-sync lines each time. movie2 came through both of
  its two. The old shop.ctm, which always lost its Sell, is the same thing.
- **The sensor was the symptom.** Azahar schedules the first accelerometer reading a fixed interval
  after the game turns it on, so where its entries fall among the buttons' depends on the exact
  tick of that call. A fork that read each kind of input from its own place in the movie took
  every out-of-sync line away from movie3 and movie5, kept movie1, movie2 and movie4 faithful, and
  still neither wrote its end save: the game's own timing had moved, and the buttons landed a
  frame off whatever order they were read in.
- **The timing is the host's.** Azahar charges a file open, and a ROM read bigger than one 8 KB
  cache line, the modelled delay *less the host time it took*, deterministic setting or not
  (`File::Read` in `core/hle/service/fs/file.cpp`, `OpenFile` and `OpenFileDirectly` in
  `fs_user.cpp`). Charging the modelled delay alone when deterministic operations are on broke all
  five movies, movie1, movie2 and movie4 included: the recordings carry this PC's host times, and
  a replay stays faithful only while its own stay close to them. The Bag reads a big module
  (`DllBag`) as it opens. Neither experiment reached the rig's branch, and their branch,
  `pa3ds/timing-experiments`, was deleted on 2026-09-27 once pins (spike 121) had replaced
  them: what each did and what it measured is all here.
- **What pa3ds's polls can do at speed.** With the tick at 1 ms over movie4 at real speed,
  dialogue polled 491 times a second (60 is the default, 56 to 58 achieved), screens and cursors
  stayed on their own 0.2 s timer, and 70 heap sweeps averaged 244 ms. At 3x with every timer a
  third as long (`regress.py run --speed 3`), all 94 dialogue pages were heard as at real speed,
  and 13 cursor lines against 19: the pointer flicking across Wally's battle menu was read as
  whole screens. The diff against a real-speed run was 153 lines, against 31 between two
  real-speed runs.
- **Not the build.** The user records on Azahar 2126.1.1 (`3e574df`); the fork is 2126.0
  (`d204921`), and the emulator warns of it at every playback. Upstream's differences between the
  two are rendering and HTTP only, nothing in input, timing or movies.

## Spike 121 — a replay pinned to one faithful run's load timings

2026-09-27, on movie4 (`spikes/121_pinned_timing.py`, fork branch `pa3ds/pinned-timing`).

- **Three numbers are all the host gives the game.** The fork reads the host's clock where the game
  can feel it in three places only: an uncached ROM read (`File::Read`), `OpenFile` and
  `OpenFileDirectly`, each charging the modelled delay less the host's time (spike 97). Whether a
  read is cached turns on its size alone (`DirectRomFSReader::CacheReady`), and nothing else in
  the core reads the host's clock for the game. With deterministic async operations on, those
  charges are the only thing a replay does not take from its movie.
- **So the fork writes them down, and charges from them.** `PA3DS_TIMING_OUT` names a file each
  of those charges is written to as `<tick> <kind> <nanoseconds>`, by the emulated tick of the
  call; `PA3DS_TIMING_IN` names such a file to charge from instead. Both unset, the emulator
  charges as before. movie4 made 1,034 such charges and never two on one tick.
- **Pinned to a faithful run, a replay follows every time.** The first run with the runtime
  reading that wrote movie4's end save became its pins. Replayed from them, with the runtime
  reading: three of three at real speed and one at 10x wrote the end save, and each charged the
  same 1,034 nanosecond values on the same ticks as the pins. Unpinned, alongside: one of two at
  real speed and one of two at 10x; both that went their own way made 890 charges, so they left
  by the same load.
- **The pins are a chain.** Two unpinned runs charge the first load a few microseconds apart, and
  from the second call on no two runs share a tick. A frame's wait usually swallows the
  difference, which is why most replays follow anyway; it is also why a tick missing from the pins
  is the first thing a pinned replay that left its run would show, and the fork logs each one
  (`no pinned charge`).
- **A run at 10x that says it went its own way may only have been cut short.** `regress.py run`
  reads for the movie's length divided by the speed, and with the runtime reading the fork does
  not reach 10x: a pinned run given that long made 887 of the 1,034 charges, all of them the pins',
  and stopped before the save. Given 1,000 game seconds it wrote it. Each timing file says how far
  a run got; a short one is not a drift.
- **Wired into the rig the same night.** `Rig.boot` hands the fork a movie's pins when they sit
  beside it, and `regress.py` keeps the charges of the first replay that writes the end save.
  `check` pinned movie1, movie2 and movie4 at 10x on its first pass and followed pinned on the
  second. Then with the runtime reading at real speed, pinned: movie1, which had followed three
  times in eight unpinned, wrote its end save, and so did movie4, neither with a load unpinned.
- **A replay runs on past its pins.** The pinned run was stopped within two seconds of its save;
  a later one stopped a moment after, and its last loads found no pin: 3 on movie1 and 1 on
  movie2, both of which followed. The fork now logs a load without a pin only before the pins'
  last tick (fork `a13bd46`).
- **A movie that ends on no save is pinned by its recording's log** (sun_10.ctm, 2026-09-27).
  Replayed with deterministic reads and nothing out of sync, Kukui's Rockruff came out 25 or 24
  HP in nine runs, five 24s and four 25s, where the user's recording had 23; with asynchronous
  reads 24, and out of sync where the battle's Bag turned the accelerometer on. The tenth run
  rolled 23 and said every line `says` names from the user's log; kept as the pins, the three
  replays since rolled 23 and followed, no load unpinned.

## Spike 122 — pinning a movie a Bag at a time

2026-09-27, on movie3 and movie5 (`spikes/122_bag_search.py`, fork commit `22d4a7f`).

- **A movie says when the game turned the accelerometer on, to a millisecond.** The emulator reads
  the buttons every 4.27 ms from the first read on, and the accelerometer 9.6 ms after the game
  turns it on and every 9.6 ms after that; a movie holds each reading among the buttons' in the
  order they came. Four readings take nine button reads less 75 ticks, so across a stretch of
  readings the ticks the game could have turned it on at in the recording narrow to a window
  1.07 ms wide, in every stretch of movies 2, 3 and 5. The fork marks the first button read
  (`P`) and each turning-on (`A`) and off (`a`) among the charges it writes.
- **The windows hold.** Pinned movie2 turned the accelerometer on inside its window at both of
  its Bags.
- **A replay that misses misses by whole frames.** movie5's observed run turned it on 16.25 ms
  past its window at the first Bag and 16.6 ms at the second; one frame back, 16.71 ms, is inside
  both. The game turns it on at the same point in a frame each time: 6.955 ms in at both of
  movie2's Bags, 11.07 and 11.09 ms at movie5's.
- **Pinned a stretch at a time, movie3 followed.** `heal` replays from the pins found so far,
  live after them, and keeps a run whose next turning-on lands in its window with nothing out of
  sync. movie3 landed each of its three Bags at the first run and then wrote its end save, the
  first replay of it that ever has; `regress.py check` has followed it pinned twice since. movie5
  landed its first Bag at the first or second run and its second at the first or fourth.
- **movie5's last minute and a half did not.** From pins through its second Bag, 25 runs to the
  end never wrote the end save. Of the 17 compared more closely, 8 wrote all of it but a word at
  `0x4200` that looks random and differs in every save, and the footer's checksum of its block
  (`0x75E72`); 9 differed in more. Nothing after the last Bag marks where a run leaves the
  recording, so there is nothing to pin that stretch by.
- **The first searches read a stale file.** Each trial stopped at the first `A` in the timing
  file, and read the file before the fork had started and emptied it, so it read the run before's:
  36 trials of a search through single loads' charges, of re-rolls, and of re-rolls with random
  charges all "found" the same frame in a second each. `Rig.boot` now deletes the file before it
  starts the fork. Neither the single-load search nor the random charges has been tried since.

## Spike 84 — a Bag a millisecond late, and a replay gone long before it

2026-09-27, on the user's eight Omega Ruby movies of that day (`spikes/84_nudge_a_charge.py`,
fork commit `eced308`).

- **Five of the six that end on a save pinned.** `regress.py check` pinned movie2, movie3,
  movie5 and movie8 on its first or second pass; `122_bag_search.py heal` landed each of movie4's
  four Bags at the first run and pinned it. All five followed pinned in the next `check`.
  movie6 and movie7 end without saving, so nothing can say whether a replay of them followed.
- **movie1 never followed**, whatever reads it: at 10x twice, and at real speed with the runtime
  reading, each replay went 174 frames out of sync at its only Bag, a few seconds before its end.
- **A miss is not always whole frames.** `heal` called movie1's Bag "1 frames off" five runs of
  five; the one measured was 0.45 ms after a window 1.07 ms wide. Charged 8.36 ms less, the
  biggest read before it (63.9 ms) turned the accelerometer on 8.75 ms earlier, 7.2 ms before
  the window, and 1 ms less landed it inside with nothing out of sync. Spike 122's "misses by whole frames"
  held for movie5's Bags; it is not a rule.
- **Landing its Bag did not make movie1 follow.** Three runs to the end from those pins never
  wrote its save, and the last differed from it in 334 bytes: the starter's record, and the
  nickname, "!blaze" for "Blaze". What the runtime said over a real-speed replay agrees with the
  user's own log of the recording line for line until the starter's battle, where his Torchic had
  20 HP and the replay's 19, with nothing out of sync. So the replay leaves the recording before
  the starter, seven minutes before the only Bag, where nothing marks the place.
- **The afternoon's recordings pinned, but for X's.** Orion recorded movie1, movie6 and movie7
  again, each ending on a save, and X's first two movies. movie7 pinned at `check`'s first run, and
  movie6 by `heal` at its third. movie1, recorded on an empty SD card, followed at the second of two
  runs on a card holding Omega Ruby's extdata alone and at the third of three on an empty one, the
  first replays of either movie1 of that day to write its end save; in the others its starter came
  out different, as the morning's had. So a movie1 follows by chance, a run in two or three, and the
  extdata made no difference seen. It is pinned on the empty card, as it was recorded. All three
  followed pinned in the next `check`. movie7 starts from the save movie6 ends on, not movie5's.
- **A save folder with its metadata and no save file is a corrupted save.** With
  `data\00000001.metadata` in place and `main` gone, as on a card built by guesswork for movie1,
  Omega Ruby says "The saved game data is corrupted. Refer to the electronic manual for details."
  and waits for an A that movie1 never presses. Orion had seen the same on X, deleting `main` alone
  to start a new game.
- **A replay can go the same way and miss the save by frames.** Of six unpinned 10x runs of movie6
  and X's two, two runs each, five differed from the recording only in the milliseconds of the time
  the game saved (a u16 at `0x120e`, after the year at `0x1204` and the month, day, hour, minute and
  second), 1 to 9 frames off, in the word at `0x4200`, in one byte off by one in two of them
  (`0x2508` in Omega Ruby's, `0x20e8` in X's), and in those blocks' checksums, X's movie2 among them
  with 10,016 and 660 out of sync in its Pokédex. The sixth, movie6 with 6,556 out of sync, differed
  in 51 bytes. X's save keeps the same footer as Omega Ruby's, 0x200 from its end (`0x65400`), where
  `check` had looked for the word that varies at Omega Ruby's offset.
- **X's two movies are not pinned.** `heal` landed movie1's two Bags at the first run each, and
  eight runs to the end from there missed its save, the last by 3 bytes besides the time and the
  random word. movie2's one stretch, the Pokédex, turns on the accelerometer and the gyroscope
  together from 16 s to 97 s: it landed at the third run, and every run after it went 180 out of
  sync and missed. The fork marks the accelerometer turning on, not the gyroscope.
