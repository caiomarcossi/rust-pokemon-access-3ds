# Game states: movies and the rig

A game state is reached by replaying a **movie** (an Azahar input recording), not by the user
playing. `spikes/rig.py` boots the fork with the movie and `--load-state 0`: about a minute to a
known moment, with the same heap addresses every run. When no movie covers the state you need, say
so and ask for a recording. A by-hand step from the user is the fallback, not the method.

## Setting up on a new machine

The rig drives a fork of Azahar, not the release: https://github.com/KenzonYeoh/azahar, branch
`pa3ds/playback-savestates`. Its commits on top of 2126.0 let a save state load during
read-only playback (upstream refuses every time; findings, spike 6), add `--load-state` and a
request file so a script can save and load without window focus, persist the deterministic
file-read setting, load a state saved by another build, add `-B` so it starts without taking
the foreground, and pin what file loads charge the game (below). It keeps a portable `user` folder
beside its exe, so it never touches the user's own Azahar, saves or movies.

```
git clone --branch pa3ds/playback-savestates --recursive https://github.com/KenzonYeoh/azahar.git %USERPROFILE%\Games\azahar-pa3ds
```

Build from an x64 Native Tools prompt for Visual Studio 2022 (cmake and ninja on PATH; CMake
downloads Qt itself):

```
cmake -S %USERPROFILE%\Games\azahar-pa3ds -B %USERPROFILE%\Games\azahar-pa3ds\build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build %USERPROFILE%\Games\azahar-pa3ds\build
```

`spikes/rig.py` expects `build\bin\Release\azahar.exe` under that clone. Anything elsewhere, and
the ROMs always, goes in `spikes/rig.toml` (gitignored; copy `spikes/rig.example.toml`): `rom` for
Omega Ruby, and `x_rom` and `sun_rom` for the spikes that read Pokémon X or Sun, which stop and say
which key is missing (`rig.rom_for`). Movies are read from the user's own
`%APPDATA%\Azahar\movies`, where Azahar records them, or from `movies`.

**Nothing specific to one machine is committed.** Paths to ROMs, the fork, movies and saves live in
`rig.toml`, and the runtime finds the player's ROM from their own Azahar's recent files. A doc or
spike that names a path gives it as an environment variable (`%APPDATA%`, `%LOCALAPPDATA%`,
`%USERPROFILE%`), a config key, or a placeholder, never one person's folder.

## Booting

- `Rig.boot` waits for the state to finish loading. Read nothing over RPC before it returns.
- Save states survive rebuilding the fork (since fork commit `92fbdb6`).
- **A state boot resumes about 19 s after the moment the state was saved.** The fork loads it about
  41 s after launch and `Rig.boot` waits 60, with the game running in between. To study a moment,
  save the state 25 s or more before it.

## Seeing the screen

`rig.screenshot()` captures the window with `PrintWindow` while it sits in the background, at the
instant you ask. It is the only ground truth about what the game is showing, so check any claim
about the display against a picture rather than asserting it. It comes back solid black while the
GDB stub has the game halted ([gdb-stub.md](gdb-stub.md)).

Drive the emulator through its command line and RPC only. Posted key messages never become Qt
shortcuts while another process owns the foreground (F1, F4 and F12 each did nothing at all), and
taking the foreground would interrupt the user's screen reader.

## Replays in the background

**The rig never takes the foreground**, so whoever is at the machine can go on reading or typing
while a replay runs. `Rig.boot` starts the fork with `-B`, which shows every window without
activating it (fork commit `e172ecd`, 2026-09-25). Without it the fork took the foreground
three times as it booted: its main window as it was shown, and before that the two game screens'
own windows, which Qt creates as top-level windows and only then moves inside the main one. A
WinEvent hook on the foreground, through a whole boot with Notepad in front, saw none of the
three with `-B`. The window still appears on the taskbar and on screen; it just never gets the
keyboard.

- **Replays are silent** because the fork's own config (`<fork>\user\config\qt-config.ini`) has
  `volume=0`. It is the fork's, not the user's Azahar, so nothing changes it back.
- **A fork built before that commit ignores `-B`** and takes the foreground as it always did:
  pull the branch and rebuild ([setting up](#setting-up-on-a-new-machine)).
- **A fork built before `fb22ac7` takes it anyway once its window has been saved maximized**,
  which happens the first time anyone maximizes a replay to watch it: Windows activates a window
  shown maximized whatever it is asked. From `fb22ac7`, `-B` leaves the saved size alone and
  opens at the default one. `spikes/97_replays_and_focus.py foreground <movie> --desktop` says
  whether a build keeps to the background.
- **By hand**, the same: `azahar.exe -B -p <movie>.ctm <rom>.3ds`. Use the short form:
  `--background` is read by a getopt pre-pass as the compression tool's `-c` (`spikes/rig.py`
  explains).

**Better still, on a virtual desktop of its own.** Set `desktop` in `rig.toml` to a Windows virtual
desktop's name as Task View shows it, or its number counting from 1, and the fork opens there
(`-D <id>`, fork commit `1d687d6`; from `fb22ac7` it is moved there before it is first shown,
so it never flashes up on the user's desktop): out of Alt+Tab and the taskbar of the desktop the user is on,
and out of reach of a stray minimise, which leaves the rig nothing to take a picture of. Make one
with Win+Ctrl+D and come back with Win+Ctrl+Left; the rig finds its id in the registry at each boot
(`rig.virtual_desktop`), and says so and opens on the current desktop when there is no such one.

- Windows lets a program move only its own windows to another desktop: the rig moving the fork's
  was refused (`0x80070005`), which is why the fork moves itself.
- Pictures still come through from the other desktop, and movie4 replayed there wrote its end save
  byte for byte (2026-09-25).
- `--desktop` has the same trap as `--background`: the pre-pass reads it as `-o`. Use `-D`.

## Replays

**A movie that has followed once follows every time: it is pinned.** The host reaches the game
only through what the emulator charges it for a file load, the modelled delay less the host's
own time (below). The fork writes each charge it makes to `<fork>\user\log\pa3ds-timing.txt`,
by the emulated tick of the load, and charges from a movie's **pins** instead when they sit
beside it (`<movie>.pins`). `regress.py check` and `run` keep the charges of the first replay
that writes the movie's end save as its pins, and every replay after it is charged the same: it
follows as that one did, whatever pa3ds reads and at any speed (findings, spike 121). Each line
they print says `pinned` or `not pinned`.

- `pinned, but N loads found no pin` means the replay left the run it was pinned to. Pins go
  stale when the fork's modelled delays change (an upstream merge) or the movie's save does:
  delete the `.pins` file, and the next replay that follows pins it afresh.
- Only a replay from the movie's first frame pins. A save state holds the ticks of whichever run
  saved it, so save states for a pinned movie in a pinned replay.
- A movie that never follows can be pinned a Bag at a time:
  `uv run python spikes/122_bag_search.py heal <movie>` keeps each stretch of a replay whose
  Bag opens where the recording's did (findings, spike 122). It pinned movie3. After a movie's
  last Bag nothing marks where a replay leaves it, and movie5's last minute and a half never
  wrote its end save; nor before a Bag far from the one before it, and the first movie1 of
  2026-09-27 left its recording at the starter, seven minutes before its only Bag (spike 84). It
  pinned movie4 and movie6 of that day.

**A replay follows its movie only in the file-read mode it was recorded in.** Asynchronous reads
make room loads take a different number of frames every run, so the presses after them land early
or late. Ask for new recordings with "Force deterministic async operations" ticked
(Emulation > Configure > Debug; Azahar forgets it on restart), and boot older movies with
`deterministic=False`.

**Ask for each recording to be closed before the emulator is.** Azahar writes a recording as it
closes, and closing it while pa3ds reads can crash it first: its shutdown destroys the GPU before
it stops the RPC server, and a read of memory the GPU caches in between calls into the GPU that is
gone (`GPU::Renderer`, offset `0x6d0e30` of stock 2126.1.1: five crashes from 2026-09-14 to
2026-09-27, two of them losing X's first movie). Tools > Movie > Close writes the movie at once;
quitting pa3ds first keeps the crash away.

**A replay desyncs now and then even when everything is right.** battles.ctm ran three times in an
hour with identical code, config, saves and movie: two were faithful, one diverged and said almost
nothing. A desynced replay is *quiet*, not wrong, so it looks exactly like a reader you have just
broken. Before believing a replay that says less than the last one did, `grep -c "out of sync"` in
`<fork>/user/log/azahar_log.txt` (the emulator notices and says so) and re-run once.

**A count of zero is not proof either.** Two replays of opening.ctm, both with no out-of-sync lines
and the same presses throughout, chose a Torchic with 19 HP in one and 20 in the other; the
recording had 20. Check the moments a study depends on against the recorder's own log of the
recording.

**A movie that ends on a save proves its own replay.** A movie is recorded to end on a save, handed
over with it, even where the save holds nothing new: a replay that drifted writes a different one.
A `[[replay]]`'s `ends_on` names that file, and

```
uv run python spikes/regress.py check
```

plays each movie at 10x with nothing reading it and waits for the game to write it. A faithful
replay writes the same bytes, since the movie fixes the console's clock, all but the footer's
first word, 0x200 from the save's end (`0x75E00` in Omega Ruby's, `0x65400` in X's), which differs
on every run; a replay that drifted never writes it. A faithful movie answers in a tenth of its
length plus the boot: movie1, 22 minutes long, in two and a half (2026-09-25). Run it on every new
movie before listening to any of it, and after any change to the fork.

**Nor does a replay that saved a frame late.** One that pressed what its movie did but reached
the save a frame or more away from the recording writes the same save but for the milliseconds of
the time it was saved (`0x120e`), the random-looking word at `0x4200`, and their blocks'
checksums (spike 84): the game went the same way, and `check` still says it never wrote the
save, since only an exact one can be pinned. `122_bag_search.py` calls such a save "all but the
random word and the time".

**A movie that ends on no save is proved by its recording's log.** None of the Sun movies ends
on a save, and nothing out of sync is not enough: sun_10.ctm, Kukui's catching tutorial, replayed
with nothing out of sync and his Rockruff came out 25 HP where the recording's had 23, and with
asynchronous reads 24 (2026-09-27). The game's rolls follow the host's load times, as the rest
does. A `[[replay]]`'s `says` names lines of the user's own log of the recording
(`user\logs\pa3ds.log` and the nine before it, the session the movie's time matches), a roll
among them; `regress.py run` counts a replay that says each of them, in that order, as followed,
and pins the movie from the first one that does. `check` has nothing to wait for on such a movie,
since nothing reads the replay.

**A replay follows its recording only while this PC reads files about as fast as it did then.**
Azahar charges the game for opening a file, and for any read of the ROM bigger than 8 KB, the
modelled time *less the time the host took*, and it does so with "Force deterministic async
operations" ticked too (`core/hle/service/fs/file.cpp`, `fs_user.cpp`; findings, spike 97). The
user's recordings carry the host times of the session they were made in. A replay's differ a
little, which a frame's wait usually swallows, and once in a while one crosses a frame and every
press after it lands a frame early or late. Charging the modelled time alone broke all five of the
user's movies of 2026-09-25, the three faithful ones included, so the replays depend on the host
time and cannot be made to ignore it.

- **Where it shows: the Bag.** Opening it reads a big module (`DllBag`) and turns the
  accelerometer on, and the accelerometer's readings, which the movie holds among the buttons',
  are the first thing to fall out of step: `Expected to read type 0, but found 2`. movie2 came
  through both of its Bag openings; movie3 lost the second of three in both runs, and movie5 the
  first or the second from one run to the next; shop.ctm always lost its Sell.
- **The sensor readings are a symptom, not the cause.** A fork that read each kind of input from
  its own place in the movie took every out-of-sync line away from movie3 and movie5, and neither
  wrote its end save (findings, spike 97; the experiment's branch is deleted).
- `spikes/97_replays_and_focus.py sensors <movie>` lists a movie's accelerometer stretches, which
  are its Bag openings: the places to distrust a replay after, and to expect `check` to fail.
- A movie recorded on a build that charged the modelled time alone would replay exactly on it; the
  user's own Azahar does not, and would have to be the fork for that.

**Omega Ruby's title id is `0011c400`.** A save folder for the rig holds two of them —
`00055d00` is Pokémon X — and a save written into the wrong one is read by nobody: wally.ctm
replayed twice against battles2's old save, reaching a Pokémon Center that was nowhere in the
movie, with no out-of-sync line to show for it (2026-09-21). The main menu's line in a runtime log
names the place and the play time the save holds, which is the quickest check that the right save
is in place. A movie of X is played on X: its `[[replay]]` table names the ROM with `rom`, and the
rig's X save is the recorder's own, byte for byte (movie_x_map.ctm, 2026-09-24).

**Keep each save point a movie was recorded on.** A movie replays only from the save it started
on, so each needs its own, however far along the recorder has saved since. The saves live beside
the fork's `user` as `user-saves-<name>` folders, each a `nand` and an `sdmc`, and each
`[[replay]]` in `rig.toml` names its folder with `saves`. They are built from the recorder's own
Azahar and the save file handed over with each movie, the game's `main`:

- **From NEW GAME:** `user-saves-new-game` holds the recorder's `nand` and no `sdmc`. The rig
  makes an empty SD card for a folder without one, which is what a new game is recorded on: Orion
  moves his whole `sdmc` aside first (2026-09-27). **Deleting the save's `main` alone is not a new
  game:** a save folder left with its `00000001.metadata` makes the game say "The saved game data
  is corrupted", on the recorder's Azahar and in a replay alike, and wait for an A the movie never
  presses.
- **After a movie:** `user-saves-<game>-after-movieN` holds the recorder's `nand` and `sdmc`, with
  the save movieN ended on in place of their own at
  `sdmc\Nintendo 3DS\<32 zeros>\<32 zeros>\title\00040000\<title id>\data\00000001\main`. The
  rest of the SD card is as the game laid it out (its extdata and the save's `.metadata` are made
  on the game's first boot and never change), so one SD card serves every movie of a game, less
  the other games' extdata and titles. Omega Ruby's movies were recorded without X's; X's were
  recorded with Omega Ruby's, and are replayed without them.
- **Take the NAND from the recorder's Azahar** (`%APPDATA%\Azahar\nand`), not the fork's: the
  system settings in them differ (`sysdata\00010017\00000000\config`).

Orion keeps his movies, logs and saves a game a folder (2026-09-27): `<checkout>\user\movies\or`,
`user\logs\or`, and `user\saves\or (0011c400)\after_<movie>`, with `x` and `x (00055d00)` for X.

**The code span in the first line says where the game was, not what pa3ds can name.** A run's
first line gives the span the runtime read, and it differs from run to run — signs.ctm read
`0x00100000-0x00630000` in a run that said nothing for 140 s and `0x00100000-0x00698000` in the
same replay a minute later, which said its 20 lines (2026-09-21). The span follows the DLLs the
game has mapped, so two runs with different spans were in different places when pa3ds attached,
which is worth knowing about a silent run. It is **not** why the run was silent: every class pa3ds
names by RTTI has its vtable between `0x005dbc30` and `0x005df748`, inside even the shortest span
seen, and opening.ctm reads `0x00630000` on every run and speaks throughout. A silent replay is a
replay that did not get where it should have.

**A movie does not carry pa3ds's own keys, so no replay ever answers one.** A `.ctm` is the
emulated console's buttons frame by frame, and a key pa3ds is bound to never becomes one: the hook
takes it and returns 1 while Azahar's window is in front, so the emulator never sees the press
(ADR-0022, `pa3ds.core.keys`). Replays bind nothing anyway — `spikes/regress.py` and
`spikes/verify_runtime.py` both call `run` with a plain `Settings()`, whose `keys` is None, and no
hook is started at all.

What follows from that:

- **The game state in a replay is exactly the live one.** The keys pa3ds took never reached the
  game, so nothing about the playthrough depends on them.
- **The replay's log is missing every navigation line the live log has** — each "door, 1 of 3, 1
  north", each category change, each rename. A live log and a replay log of the same stretch are
  not comparable line for line if the player used the navigation keys, and `regress.py compare`
  will show them as removed lines. Compare a replay against a replay.
- **So a movie cannot be the test for the navigation keys.** What a key says is tested by driving
  the navigator over a map of its own (`tests/test_navigate.py`); what a replay is good for is the
  map the navigator reads, not the pressing.

## Real speed to listen, 10x to reach

`frame_limit` throttles the host, not the game, so at 10x (`frame_limit=1000`) a movie plays the
same game faster: movie1, movie2 and movie4 wrote their end saves byte for byte at 10x. The rig
plays at 10x only where nothing listens — `regress.py check`, or playing up to a moment to save a
state there — and at real speed wherever pa3ds reads (`regress.py run`, `verify_movie.py`):

- **pa3ds reads on the wall clock**, 60 polls a second, and a heap sweep takes 150 to 250 ms. At
  10x a poll is ten frames from the next and a sweep is two seconds of game, so whatever the game
  shows briefly is gone before it is read: of movie2's prompts, only the starter choice was heard
  at 10x (findings, spike 22). ADR-0004 says the poll rate rises when the game runs fast; that was
  never built.
- **The polls could go faster; the sweeps cannot.** With the tick at 1 ms, dialogue polled 491
  times a second over movie4, but screens and cursors keep their own timer (`screen_tick`, 0.2 s)
  and each sweep still took 244 ms (spike 97).
- **A regression run compares what was said**, and what pa3ds says at 10x is not what the player
  hears, so a diff of two 10x runs tests the wrong thing.

`regress.py run --speed 3` plays at 3x with every timer of the runtime a third as long. Over
movie4 it heard all 94 dialogue pages the real-speed runs heard, and 13 cursor lines against 19:
the pointer flicking across Wally's battle menu was read as whole screens instead. Its diff
against a real-speed run was 153 lines where two real-speed runs differed by 31. With the heaps'
watch (ADR-0034) it is no better: at 5x movie4 followed its movie and heard all 94 pages and 69
speakers' names, but 8 cursor lines of the real-speed run's 13, and read the move list and the
Bag's pockets out whole; at 10x the replay went its own way (2026-09-26). The heaps are looked at
by the wall clock too, and a sweep of new blocks takes as long at any speed. So a faster run
is for work on dialogue, compared only with runs at the same speed; anything that touches cursors
or menus is listened to at real speed.

Save states are what make listening cheap instead: play to just before a moment once, save a state
there, and boot the state at real speed as often as needed ([Booting](#booting)).

## Before and after a change

`spikes/regress.py` replays movies with the runtime reading them silently, and compares two runs'
logs line by line with their times left out. Which movies it replays is each machine's own: list
yours as `[[replay]]` tables in `spikes/rig.toml` (`rig.example.toml` shows the fields). Pick the
ones that reach what the change touches.

```
uv run python spikes/regress.py run before
uv run python spikes/regress.py run after
uv run python spikes/regress.py compare before after
```

It runs the code checked out where it is started, and each replay imports that code as it starts,
so **committing or stashing before a run is not optional** — an edit landing mid-run changes what
the later replays test. Check the "before" commit out in place:

```
git checkout --detach <the commit to compare against>
uv run python spikes/regress.py run before
git checkout <your branch>
uv run python spikes/regress.py run after
uv run python spikes/regress.py compare before after
```

`spikes/rig.toml` is gitignored, so it survives the checkout and needs no copying, and the logs go
under `%LOCALAPPDATA%` rather than the checkout, so both runs land in the same place whatever is
checked out.

A worktree was used for this until 2026-09-22, so that the main checkout could be edited while a
run went on. It bought nothing: **nothing may use the CPU while a replay runs** — pytest and
pyright during one desynced menus.ctm and battles2.ctm, both clean when re-run on an idle machine
— so there is nothing to do in the other checkout anyway, and one emulator at a time is the rule
regardless. A detached checkout is the same thing with one fewer copy of the repository to keep in
step (Orion, 2026-09-22).

Some lines race by design (a cursor item "opened" in one run and "moved" in the next, a panel
caught while it is revealed, a command menu item said twice when its cursor is lost for two polls),
so read a diff rather than count it, and re-run a replay whose out-of-sync count is not zero.

**A clean diff is usually not an empty one**, and on 2026-09-22 this was measured rather than
assumed. Running the *same commit* twice and comparing gave 26 changed lines on opening, 62 on
battles2 and 5 on wally, all with an out-of-sync count of zero. The emulator follows the movie
exactly — that is what out-of-sync zero means — but pa3ds polls on the wall clock, so which frame
a poll lands on moves from run to run. What moves with it: the player's cell while they walk, and
every bearing measured from it; an HP bar caught mid-animation; the name entry's cursor on a
different letter; a summary panel half revealed.

So **compare a change's diff against a same-code control**, and read two things rather than one:

- **How many.** A change whose diff is smaller than the control's is inside the noise. The keys
  and renames branch gave 32, 44 and 9 against main, where the control gave 26, 62 and 5 — on
  battles2 the same code disagreed with itself *more* than the change did.
- **What kind.** Classify the changed lines by their first word (`dialogue`, `field`, `cursor`,
  `battle`, `screen`, `list`). A kind that appears when the code changed and never appears in the
  control is the thing to go and look at. On that branch there were none, in any replay.

A third thing worth counting if a change goes near them: the speaker tags, `[Norman]` and the
like. They held at 74 on wally and 27 on battles2 across all three runs, which no amount of poll
drift moves, because a speaker is resolved per page rather than per poll. A
movie recorded without deterministic reads drifts more: its save states can land on different
screens from run to run, so check that what was said matches the screens it reached rather than
matching the other run.

**Every run is a measurement as well.** Each replay's line in `summary.txt` carries what finding
things cost it: its sweeps (how many, their seconds and pages, and how many swept the whole heap
and how many only the blocks the heaps had just allocated), the looks at the heaps and their
seconds, the heap probes' seconds, dialogue polls a second, and the gaps between them (the 99th
percentile, and how many were over 100 ms, which is how long a line can wait unread), with the
slowest looks and sweeps of new blocks over 100 ms. A change to when or where the runtime sweeps is compared on
these against main over the same movies, as spike 99's was
([findings](../findings/memory-and-discovery.md)). A replay that drifted compares only up to
where it drifted, and only with one that drifted at the same point: movie5 drifts with either
17669 or 9833 frames out of sync, and two runs with the same count went the same way.

**A replay can go its own way with nothing out of sync.** The emulator charges a file the game
reads the time the host took, so what pa3ds reads while the game loads can change when a load ends,
and the random numbers after it: Wally's lent Zigzagoon came out with 22 or 23 HP instead of 24,
and his catch 70 s later (movie4, 2026-09-26). Each replay's line says whether it followed its
recording, by the save the movie ends on (`ends_on`) or the lines the recording said (`says`); one
that "WENT ITS OWN WAY" says nothing about the code, and is run again. Since the runtime reads the game's heaps ten times a second (ADR-0034) it happens more
often: movie1 and movie4 did on some runs. A pinned movie does not (see Replays, above). At 10x a
run can also stop before the save: `run` reads for the movie's length over the speed, and the fork
does not reach 10x while pa3ds reads.

## One RPC client at a time, and one emulator

A second client sweeping while the first was starting up put the server behind for good within
seconds: every later request from either timed out, and only restarting the emulator recovers it.
To look at a running rig, wait for its script to finish.

A second *emulator* is as bad: both serve the RPC on the same port, and a replay run beside a
leftover one filled its log with "the emulator stopped answering; retrying" and proved nothing
(2026-09-18). A run that ends badly can leave one behind, so check `tasklist | grep -i azahar` is
empty before starting, and again before believing a quiet log.
