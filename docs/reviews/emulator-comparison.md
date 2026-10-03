# Azahar or BizHawk

Asked 2026-09-13: should pa3ds run on a BizHawk fork, [RealAmethyst/BizHawk], instead of Azahar?
The user wants a comparison before any switch. This is what is known so far. The questions at the
end need hands-on tests, and nothing is decided.

## What the fork is

- **Upstream BizHawk plus one branch, `speech-lua-library`.** Two commits on 2026-06-03 add
  `SpeechLuaLibrary.cs` (150 lines) and `prism.dll`. Lua scripts get `speech.say(text,
  interrupt)`, `speech.output`, `speech.braille` and `speech.stop`, all through Prism — the same
  library pa3ds already speaks through. `interrupt` defaults to true.
- The fork's `master` has no commits of its own and is 79 behind upstream. There are no releases
  and no stars or forks. Using the speech library means building BizHawk from that branch.
- BizHawk is MIT. Prism is MPL-2.0. The 3DS core is a Citra port.

## BizHawk's 3DS core, Encore

- Added in BizHawk 2.10, 2025-01-07. The latest BizHawk is 2.11.1, 2026-05-01.
- Ported from `CasualPokePlayer/encore`, version "nightly-2104", credited to "Tropic Haze and
  Citra contributors". Azahar is 2126.0, so Encore sits on an older Citra. What that costs in speed
  or accuracy for ORAS is untested.
- **Memory is exposed as domains:** FCRAM, VRAM, DSP RAM, N3DS Extra RAM, and "System Bus", which
  is the game process's virtual address space — the same addresses the RPC reads today. Every
  anchor and offset in the findings would carry over, if the two emulators lay memory out alike.
- **No memory callbacks.** Encore does not implement BizHawk's `IDebuggable`, so Lua's
  `event.on_bus_write`, `on_bus_read` and `on_bus_exec` have nothing to attach to on 3DS. There
  is no equivalent of the GDB stub's watchpoints and breakpoints, which found the dialogue setter
  (spike 8) and caught the Yes/No prompts (spike 15).
- Settings include `use_cpu_jit`, New 3DS mode, CPU clock percentage, and a determinism flag that
  also turns off asynchronous shader compilation. It needs OpenGL 4.3 and runs one instance at a
  time. Its source notes the user folder must be empty when recording a movie, as with Azahar.

## Side by side

| | Azahar, with the pa3ds fork | BizHawk fork, Encore core |
|---|---|---|
| Reading memory | RPC over UDP from outside, 1 KiB a packet. One client at a time, or the server falls behind for good. | Lua inside the emulator, any size. Outside programs through `comm` sockets, HTTP or memory-mapped files. |
| When a read happens | Whenever the packet arrives, mid-frame. Captures must be taken paused (spike 3). | At a frame boundary, from `event.onframeend`. One read sees one frame. |
| Hooks | GDB stub: write watchpoints and execute breakpoints. Halts the game; for investigation only. | Frame events only, on 3DS. |
| Speech | pa3ds in Python, through `prismatoid`. | `speech.say` from Lua, or pa3ds unchanged behind a bridge. |
| Test rig | Movies. Save states during playback only in our fork; loaded from the command line; no frame advance from outside. | Movies, save states anywhere, frame advance and rerecording, all built in. |
| Speed on this machine | Measured: median 2.14 ms of work per 16.67 ms frame. | Untested. |
| Using the emulator with NVDA | The user plays with it now, but on OpenGL its GUI thread answers a screen reader's questions in 6.6 ms with a game loaded against 128 us without one — the lag the user hears. Switching it to Vulkan removes that entirely; see below. | Measured: 187 us median with a ROM, p90 3.5 ms. Responsive, but **less so than Azahar on Vulkan**, so this is no longer a reason to switch. WinForms. |
| Cost to change | None. | movie1 and movie2 are Azahar `.ctm` files and would be recorded again. A bridge or Lua readers. Keys set up again. |

## What a switch would mean for pa3ds

- **The readers would not need rewriting.** They read through the `Memory` protocol in
  `src/pa3ds/core/memory.py`. A BizHawk transport is a small Lua script that answers reads over a
  socket, plus a second `Memory` implementation. The RTTI, text and dialogue code stays as it is,
  and so do its tests.
- Writing the readers in Lua instead would give up Python, its tests and its type checking. Not
  recommended.
- Investigation would still want Azahar, because only its GDB stub can say who wrote a byte.
  Playing on BizHawk while investigating on Azahar works only if both put the same objects at the
  same virtual addresses. Their shared Citra ancestry makes that plausible; nobody has checked.
- The frame-boundary read is the strongest argument for BizHawk. Every smeared capture and every
  "captured while paused" rule in AGENTS.md comes from reads landing mid-frame.

## Where it stands, 2026-09-13

- **Driving BizHawk with NVDA is settled.** The fork's maintainer built an accessibility layer for
  Pokémon Black and White on it, and several of the user's friends have played with it. That
  layer is not public; it is distributed through the maintainer's
  [Accessibility Mod Manager](https://github.com/RealAmethyst/AccessibilityModManager), which reads
  its list from [a registry repository](https://github.com/RealAmethyst/accessibility-mod-manager-registry).
  (The user's report; not checked here.)
- **The user leans towards staying on Azahar**, for the watchpoints: they are how the dialogue
  setter, the TextBox layout and the Yes/No cursor were found, and Encore has nothing to replace
  them.
- **The frame-boundary argument is weaker than this page first made it.** For testing, movies and
  save states already give repeatable moments. At runtime, reads do land mid-frame on Azahar, but
  the reader's validators — class names, the tag processor check, a length within its capacity — already
  turn a torn read into "nothing to say" rather than wrong speech, and the reader works. What is
  left of the argument is cheaper consistent snapshots, which matter for investigation, not for
  playing.
- Lua was never required: a bridge keeps the Python readers. That is not a reason to switch
  either, only a reason switching would not be a rewrite.
- A distribution channel exists that blind players already use. That is worth remembering for
  shipping pa3ds, whichever emulator it runs on.

## What the user found, 2026-09-15 and 2026-09-16

The user played Omega Ruby in BizHawk for a few minutes and reported back. Checked here against
their install, the Accessibility Mod Manager's copy of the Black and White fork:

- **It is the fork, and it has both halves.** `dll/prism.dll` is present, `SpeechLuaLibrary` is
  compiled into `dll/BizHawk.Client.Common.dll`, and `dll/encore.dll` is the 3DS core. Every
  assembly reads `1.0.0+5d046ab67c0f12fa90c3401c06d2fe32c20564ba`, a build of the maintainer's own
  tree rather than a numbered BizHawk release. It arrived through the Accessibility Mod Manager,
  whose backup stamp is 2026-09-15.
- **Omega Ruby loads.** `config.ini`'s recent list holds
  their Omega Ruby ROM first, then the decrypted Pokémon X
  and Pokémon White. That answers most of question 1: the ROM runs.
- Its Encore settings are `UseCpuJit`, `IsNew3ds`, `AsyncShaderCompilation`, `ShadersAccurateMul`
  and `UseHwShader` all on, `CpuClockPercentage` 100, `TempUserFolder` on. Worth noting that
  Azahar runs here with `shaders_accurate_mul=false`, the *faster* setting, and BizHawk still felt
  quicker — so the impression is not a settings artefact in Azahar's favour.
- **The user's report:** BizHawk felt more responsive with a screen reader. The menu bar appeared
  the moment Alt was pressed, and alt-tabbing in and out did not stall NVDA. Azahar does both, but
  only once a ROM is loaded — with no ROM it behaves normally. Citra behaves like Azahar. Someone
  else told the user BizHawk becomes "clippy" over a long session; unmeasured.
- The user has not decided to switch.

## The screen reader lag, and what it actually is

The user's theory is that Azahar sends NVDA malformed or excessive accessibility information, and
that NVDA handles it poorly because it is weak at doing things asynchronously. Two things are worth
separating here.

**What the Qt frontend actually sends.** `GMainWindow::UpdateWindowTitle` (`citra_qt.cpp:4399`) is
called from three places, all of them load and unload paths — the title is not rewritten per frame,
so the obvious flood is not there. `UpdateStatusBar` (`citra_qt.cpp:3658`) runs on a one-second
timer that starts when a game starts (`:1635`) and stops when it stops (`:1762`), setting label
text and sometimes a style sheet. That does fire accessibility events, at about one a second, and
it does match "only when a ROM is loaded" — but a handful of events a second is not enough on its
own to explain the symptom.

**The explanation that turned out to be right.** A screen reader reads another process's
UI over COM, and those calls are answered by *that process's own GUI thread*. If the thread is slow
to pump messages, the reader blocks on every call until it answers or times out, and NVDA's core
has nothing to do but wait. That explains all of it: only with a ROM loaded, the same in
Citra, and the same in Event Viewer and Wireshark with a lot on screen — none of which sends
malformed information. It makes the emulator the thing to fix rather than the screen reader.

**Measured, 2026-09-16** (`spikes/53_gui_responsiveness.py`, the pa3ds fork,
`SendMessageTimeout(hwnd, WM_NULL)` — answered by the same thread and the same queue an
accessibility call goes through, with no screen reader involved). Every run is at
`frame_limit=100`, real play speed, and against a window confirmed to be painting the game by a
`PrintWindow` capture:

| | spaced median | p90 | worst | back to back |
|---|---|---|---|---|
| No ROM loaded | 128 us | 354 us | 1.7 ms | 17,600 answers/s |
| **OpenGL**, Omega Ruby running | **6.6 ms** | 10.9 ms | 68.9 ms | **195 answers/s** |
| **Vulkan**, Omega Ruby running | **125 us** | 309 us | 7.0 ms | **17,600 answers/s** |

**Two rhythms, because they answer different questions.** Spaced (one call every 10 ms) is what a
trickle costs. Back to back is what a screen reader does: it walks a control over many calls, on
one thread, with no gaps. Those do not cost the spaced figure each — they queue behind a whole
present pass and are then drained in a burst, which is why the burst median is 140 us while its p90
is 19 ms. What matters is the rate. Twenty questions take **1 ms** with no ROM, **103 ms** on
OpenGL and **1 ms** on Vulkan, and a focus change is a multiple of that. `WM_NULL` is
also the cheapest message there is; a real cross-process accessibility call costs more, so these
are floors.

**Real play is the worse case.** At `frame_limit=1000`, which the rig leaves behind after a replay,
frames arrive faster than the present loop asks for them and OpenGL measured 4.2 ms. At 100 a frame
comes every 16.7 ms and the loop waits most of it: 6.6 ms. Pin the frame limit before believing any
figure here.

**The cause is in the source, and it is OpenGL-only.** `OpenGLRenderWidget::paintEvent`
(`citra_qt/bootmanager.cpp:305`) calls `Present()` and then `update()`, which schedules the next
paint at once — so the GUI thread runs a continuous paint loop while a game is powered on. Each
pass does `TryPresent(100, ...)`, then `SwapBuffers()`, then **`glFinish()`** (`:302`), which blocks
the calling thread until the GPU has finished everything. Messages are only pumped between passes.
With no game, `Present()` returns immediately on `!system.IsPoweredOn()` (`:295`) — which is
precisely why the lag appears only once a ROM is loaded, and why Citra, sharing this code, behaves
the same.

So the user's instinct was right that Azahar is at fault, and wrong about the mechanism: it is not
sending NVDA too much or malformed information, it is **too slow to answer NVDA's questions**.
Nothing here needs fixing in NVDA.

### Vulkan is the fix, and it is a setting rather than a patch

`RendererVulkan::TryPresent` is an empty `{}` (`renderer_vulkan.h:84`), `VulkanRenderWidget` has
no `paintEvent` at all (`bootmanager.cpp:322`), and `PresentWindow` runs its **own presentation
thread** when `async_presentation` is on (`vk_present_window.cpp:111`, `:150`) — which is the
default and is on in the user's config. That setting exists only for Vulkan; the OpenGL path has no
equivalent. So on Vulkan the Qt GUI thread is out of the frame path entirely, which is what the
table measures.

The user is on `graphics_api=1`, OpenGL, which is only the desktop build default — Android defaults
to Vulkan, and their Quadro T1000 supports it. The change is Emulation > Configure > Graphics.

What was checked, and what was not:

- **It renders, in the same scene.** `PrintWindow` captures of both runs show Omega Ruby drawing in
  full colour — and both landed in the same place, the opening cutscene 50 s after launch — so an
  idle GUI thread is not a broken renderer, and the two figures are not two different workloads.
- **Not checked: a long session.** Only the first minute of boot was measured, and only the intro.
- **Checked: a movie recorded on OpenGL replays the same on Vulkan.** This matters more than it
  looks, because the user records the movies pa3ds is tested against and a desynced replay looks
  like a normal session (spike 5). `battles.ctm` was replayed from boot under `graphics_api=2` and
  compared with the OpenGL log event by event: **84 game events, every one the same and in the same
  order** — every dialogue line, every cursor move, the same layout addresses — running about 1.8 s
  earlier for boot. The three differences are all in pa3ds's own polling, not the game: the level
  up's "SP. ATK" fell into the next screen text poll instead of the batch before it, the HP box
  came as "22/ 22" then "6" instead of "6 22/ 22" (the known slide-in fault), and a trailing
  "Route 101" fell past the end of the run. pa3ds polls on wall-clock rather than on emulated
  frames, so two OpenGL runs vary the same way — spike 50 saw the first cursor item arrive 2 s late
  in two replays of three. Logs kept side by side as `captures/battles_verify_opengl.log` and
  `battles_verify_vulkan.log`.

**Nothing found against it so far.** It renders, it is as responsive as an idle window, and it
does not disturb the test rig. The remaining risk is the one nobody has exercised: a long session,
and effects later in the game than the opening.

**The user switched to Vulkan on 2026-09-16** and reported it working, and "definitely less laggy
than OpenGL", on a short try. So Azahar's side of this is settled in practice; what is left is
whether it holds over a long session.

### If Vulkan ever cannot be used

Each of these is small, none of them changes a pixel, and all are in the fork:

1. Drop `glFinish()`, or replace it with a fence the widget polls. It is a full CPU stall per frame
   and it changes only when the CPU may continue, never what was drawn.
2. Stop the self-scheduling paint loop: repaint from a frame-ready signal instead of calling
   `update()` at the end of every `paintEvent`. The same frames are presented, because `TryPresent`
   already blocks until one exists; the thread just waits in Qt's event loop, where messages get
   answered, rather than inside `paintEvent`, where they do not.
3. Cut `TryPresent`'s 100 ms ceiling. That number is a timeout passed to
   `OGLTextureMailbox::TryGetPresentFrame` (`gl_texture_mailbox.cpp:111`), and **on timeout it
   returns the previous frame**, which is presented again. So it drops nothing: it only bounds how
   long the GUI thread blocks before redrawing the same picture.
4. If the thread still cannot be freed, give the render widget no accessible object at all, so a
   screen reader never asks it anything.

### How BizHawk does it, and how it measures

From `BizHawk.Client.EmuHawk/MainForm.cs`, `ProgramRunLoop`: input, emulation
(`StepRunLoop_Core`), rendering, frame throttling and the message pump all run on the UI thread,
and `CheckMessages()` calls `Application.DoEvents()` each iteration. Structurally the same shape as
Azahar — one thread, pumped once per frame. The difference is how much of each frame the thread
spends blocked, and that `DoEvents()` drains the whole queue at once rather than one message.

**Measured 2026-09-16** at the user's request, same spike, same machine, against their own install
(`--emulator`, which touches no config of its own; the file came back byte-identical afterwards):

| | spaced median | p90 | worst | back to back |
|---|---|---|---|---|
| Azahar, no ROM | 128 us | 354 us | 1.7 ms | 17,600 answers/s |
| Azahar **OpenGL** + ROM | 6.6 ms | 10.9 ms | 68.9 ms | 195 answers/s |
| Azahar **Vulkan** + ROM | **125 us** | **309 us** | 7.0 ms | **17,600 answers/s** |
| BizHawk, no ROM | 396 us | 5.3 ms | 7.0 ms | 9,725 answers/s |
| BizHawk + ROM | 187 us | 3.5 ms | 7.4 ms | 11,386 answers/s |

**Azahar on Vulkan is the most responsive of the five**, on median, on p90 and on throughput. So
the responsiveness the user felt in BizHawk was real — it is roughly 35 times better than the
OpenGL Azahar they were comparing it against — but it is not a reason to switch, because the
setting change beats it.

BizHawk's p90 of 3.5 ms is its own once-per-frame block showing through: most pings land while
`DoEvents()` is draining and cost 50 us, some land mid-frame and cost milliseconds. Its idle figure
being *worse* than its running one is the same loop running either way.

Two caveats on BizHawk's row, both of which flatter it rather than the reverse:

- **It was on a static screen.** Encore runs with `TempUserFolder`, so there is no save and the ROM
  boots to language selection, a still menu — while Azahar, on the user's save, was in the animated
  opening. Less to draw, so if anything BizHawk had the easier workload.
- Only the first minute after launch, as with every row here.

## Still unknown, if switching is reconsidered## Still unknown, if switching is reconsidered

1. ~~Does ORAS run in BizHawk on this machine?~~ It loads and plays (the user, 2026-09-15).
   Whether it holds full speed over a long session is still open — the "clippy after a while"
   report is exactly the shape of thing a frame-time log would settle. *The user.*
2. Does a Lua script on the System Bus domain find a message window by its RTTI vtable and speak its
   text with the fork's `speech.say`? *An agent, once BizHawk is installed.*
3. Does the same moment put the same objects at the same addresses in both emulators? *An agent.*
4. How many reads per second does a Python client get through a `comm` socket bridge, against the
   RPC's ~42,000? *An agent.*

## Sources

- [RealAmethyst/BizHawk], and its `speech-lua-library` branch compared with upstream through the
  GitHub API on 2026-09-13.
- [BizHawk release history](https://tasvideos.org/Bizhawk/ReleaseHistory)
- [BizHawk Lua functions](https://tasvideos.org/Bizhawk/LuaFunctions)
- Upstream source: `src/BizHawk.Emulation.Cores/Consoles/Nintendo/3DS/` in
  [TASEmulators/BizHawk](https://github.com/TASEmulators/BizHawk) — `Encore.cs`,
  `Encore.IMemoryDomains.cs`, `Encore.ISettable.cs`, `LibEncore.cs`.

[RealAmethyst/BizHawk]: https://github.com/RealAmethyst/BizHawk
