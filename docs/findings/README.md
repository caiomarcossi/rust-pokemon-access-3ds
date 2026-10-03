# Findings

What has actually been measured about the game, one spike an entry: the question it asked, what
came back, and what was built from it. A claim in an ADR, in the code or in a commit should be
traceable to one of these; a claim that is not is a guess.

An entry is never rewritten when a later spike contradicts it — the later one says so, and both
stay. That is what makes the log worth keeping: it records what was believed as well as what is.

`spikes/README.md` maps the scripts that did the measuring to these entries. The scripts are a
record too, not maintained code.

## By area

| Area | What it covers |
|---|---|
| [The instruments](tools-and-rig.md) | Speech out, the emulator in, and the rig that replays a recording so a finding can be checked twice. |
| [Memory, and finding things in it](memory-and-discovery.md) | Where memory is, how an object is found by its class, what a sweep costs, and how a reader is handed what it needs before the moment passes. |
| [Text, and who is shown it](text-and-dialogue.md) | Message windows, pages, control codes, prompts and choice boxes: everything between the game's words and a spoken line. |
| [Screens and cursors](screens-and-cursors.md) | Layouts, panes, screen text and the kinds of cursor: menus, the battle's buttons, the keyboard's keys, a frame that slides onto a choice, the Pokédex's plate, cursors named by their textures, and Wally's pointer. |
| [Battle](battle.md) | The HP box, the battlers' names, the gauges behind each panel, and the Pokémon records the game keeps them in, in ORAS and in Sun. |
| [The field, and the ROM](field-and-rom.md) | Who is speaking, what is standing on a map, what a zone says about it, and what can be read out of the player's own ROM. |

## Every spike

| # | Question | Date |
|---|---|---|
| [1](tools-and-rig.md) | speech | 2026-09-10 |
| [2](tools-and-rig.md) | RPC | 2026-09-10 |
| [3](text-and-dialogue.md) | the layout pane tree | 2026-09-11 |
| [3b](text-and-dialogue.md) | live dialogue text | 2026-09-11 |
| [4](tools-and-rig.md) | the GDB stub as an instrument | 2026-09-12 |
| [5](tools-and-rig.md) | movie replay as the test rig | 2026-09-12 |
| [6](tools-and-rig.md) | save states during a replay | 2026-09-12 |
| [7](tools-and-rig.md) | the fork, built and working | 2026-09-12 |
| [8](memory-and-discovery.md) | the text pane, found by shape | 2026-09-12 |
| [9](tools-and-rig.md) | the rig, and how to see the screen | 2026-09-12 |
| [10](memory-and-discovery.md) | what the pane is, and what it is not | 2026-09-12 |
| [11](text-and-dialogue.md) | the control codes, decoded | 2026-09-12 |
| [12](memory-and-discovery.md) | the line records, and a validator for stale panes | 2026-09-12 |
| [13](memory-and-discovery.md) | the classes have names | 2026-09-13 |
| [14](text-and-dialogue.md) | list windows, and the line as typed | 2026-09-13 |
| [15](text-and-dialogue.md) | the replay's Yes/No prompts, caught by breakpoint | 2026-09-13 |
| [16](memory-and-discovery.md) | a class census across the first battle | 2026-09-13 |
| [17](text-and-dialogue.md) | the opening, which the reader did not read | 2026-09-13 |
| [18](screens-and-cursors.md) | the cursor moves | 2026-09-13 |
| [19](text-and-dialogue.md) | all of movie2, read by the runtime | 2026-09-13 |
| [20](text-and-dialogue.md) | when a list window is on screen | 2026-09-13 |
| [21](text-and-dialogue.md) | the same words again | 2026-09-13 |
| [22](text-and-dialogue.md) | the Yes/No reader, run live | 2026-09-13 |
| [23](text-and-dialogue.md) | the prompts live play found silent | 2026-09-13 |
| [24](tools-and-rig.md) | the field's Yes/No, from a saved state | 2026-09-13 |
| [25](tools-and-rig.md) | why replays drift, and what a save state survives | 2026-09-13 |
| [26](text-and-dialogue.md) | the field's choice box | 2026-09-13 |
| [27](screens-and-cursors.md) | the options screen's descriptions | 2026-09-13 |
| [28](screens-and-cursors.md) | the text no message window holds | 2026-09-14 |
| [29](screens-and-cursors.md) | which of them are on screen, inside one layout |  |
| [30](screens-and-cursors.md) | what says a layout is drawn |  |
| [31](screens-and-cursors.md) | a prototype layout reader, checked against pictures |  |
| [32](screens-and-cursors.md) | screen text in the runtime | 2026-09-14 |
| [33](screens-and-cursors.md) | where each piece of screen text is | 2026-09-14 |
| [34](text-and-dialogue.md) | does a field message box leave the screen between conversations? | 2026-09-14 |
| [35](screens-and-cursors.md) | the summary screen reads pages that are not on screen | 2026-09-14 |
| [36](battle.md) | the battle's layouts are in the linear heap | 2026-09-14 |
| [37](screens-and-cursors.md) | button managers are touch buttons | 2026-09-14 |
| [38](screens-and-cursors.md) | the chosen X menu button pulses | 2026-09-14 |
| [39](screens-and-cursors.md) | the X menu cursor is a window in a layout of its own | 2026-09-14 |
| [40](screens-and-cursors.md) | what a menu cursor layout looks like, and which screen a layout is on | 2026-09-14 |
| [41](screens-and-cursors.md) | the projection, decoded right | 2026-09-14 |
| [42](screens-and-cursors.md) | menus heard under the cursor, in the runtime | 2026-09-14 |
| [43](screens-and-cursors.md) | the user's first live test, and the fixes | 2026-09-14 |
| [44](screens-and-cursors.md) | the summary's pages, watched for a page turn | 2026-09-14 |
| [45](battle.md) | the battle's move list, and how to reach it | 2026-09-14 |
| [46](memory-and-discovery.md) | the linear heap, swept a slice at a time | 2026-09-14 |
| [47](screens-and-cursors.md) | the user's second live test | 2026-09-14 |
| [48](screens-and-cursors.md) | menus.ctm, the user's third live test replayed | 2026-09-14 |
| [49](screens-and-cursors.md) | AreaNav, late | 2026-09-14 |
| [50](battle.md) | battles.ctm, the battle menus | 2026-09-14 |
| [51](field-and-rom.md) | who is speaking a field message | 2026-09-16 |
| [52](field-and-rom.md) | what names the speaking actor | 2026-09-16 |
| [53](battle.md) | the battle's HP box, and what a panel is | 2026-09-16 |
| [54](battle.md) | what the battle's HP box holds | 2026-09-16 |
| [55](battle.md) | the battlers' names and genders are in RAM after all | 2026-09-16 |
| [56](battle.md) | the name array holds for another battle, and where the real HP is | 2026-09-16 |
| [57](battle.md) | the anchor for the battlers | 2026-09-16 |
| [58](text-and-dialogue.md) | why a signpost read twice was still silent | 2026-09-16 |
| [59](memory-and-discovery.md) | where the runtime's time goes | 2026-09-16 |
| [60](memory-and-discovery.md) | finding a new message window without a sweep | 2026-09-16 |
| [61](memory-and-discovery.md) | what holds a layout work | 2026-09-16 |
| [62](memory-and-discovery.md) | what the runtime reads, and what each sweep finds that nothing else does | 2026-09-16 |
| [63](memory-and-discovery.md) | what leads to the battle's layout pool and to layout files | 2026-09-17 |
| [64](memory-and-discovery.md) | a layout pool read from a message window's handle | 2026-09-17 |
| [65](battle.md) | the save's own Pokémon records are in RAM | 2026-09-17 |
| [66](battle.md) | which Pokémon each HP panel shows | 2026-09-17 |
| [67](battle.md) | what a gauge's numbers do | 2026-09-17 |
| [68](screens-and-cursors.md) | where words and cursors are | 2026-09-17 |
| [69](memory-and-discovery.md) | what the heap sweep reads, and where the objects it finds are | 2026-09-18 |
| [70](memory-and-discovery.md) | the first battle's window, and new memory the heap measured the same with | 2026-09-19 |
| [71](screens-and-cursors.md) | the opening's choice of boy or girl | 2026-09-19 |
| [72](screens-and-cursors.md) | the main menu, and what comes before it | 2026-09-21 |
| [73](field-and-rom.md) | who is speaking, with three characters in one conversation | 2026-09-21 |
| [74](field-and-rom.md) | the game's own text, out of the user's ROM | 2026-09-21 |
| [75](field-and-rom.md) | the overworld's own array of models | 2026-09-21 |
| [76](field-and-rom.md) | the zone's own list of what is on a map | 2026-09-21 |
| [77](field-and-rom.md) | the trainer table, and whether it reaches an overworld sprite | 2026-09-21 |
| [78](field-and-rom.md) | the map's interactables and the player's cell | 2026-09-21 |
| [80](field-and-rom.md) | the DexNav's hidden Pokémon, and the record the game keeps for it | 2026-09-22 |
| [82](field-and-rom.md) | the map's chunks, and which cells can be walked | 2026-09-22 |
| [83](field-and-rom.md) | the Pokémon Center's PC, and the cells the player uses by facing them | 2026-09-23 |
| [84](tools-and-rig.md) | a Bag a millisecond late, and a replay gone long before it | 2026-09-27 |
| [—](field-and-rom.md) | the field map's width, read on Route 104 | 2026-09-23 |
| [85](field-and-rom.md) | what the ROM already names: the rival, the zones, and what is in a town | 2026-09-23 |
| [86](field-and-rom.md) | naming a door from the ROM alone | 2026-09-23 |
| [87](field-and-rom.md) | the player's own model, and a zone's name sooner than its banner | 2026-09-23 |
| [88](field-and-rom.md) | where the game places its work objects | 2026-09-23 |
| [89](field-and-rom.md) | the sprite a work object was filled with | 2026-09-23 |
| [90](field-and-rom.md) | the people a sprite is, through the trainers who stand as it | 2026-09-23 |
| [91](field-and-rom.md) | the people the ROM never lists, in every language it carries | 2026-09-23 |
| [92](field-and-rom.md) | the way there, over the game's own collision | 2026-09-24 |
| [93](field-and-rom.md) | the PC's own word, the old tile keys, and where an exit leads | 2026-09-24 |
| [94](field-and-rom.md) | a chunk read while it is still loading | 2026-09-24 |
| [95](screens-and-cursors.md) | the Pokédex list, the Poké Mart, and Wally's tutorial | 2026-09-24 |
| [96](screens-and-cursors.md) | cursors by their names through the whole ROM; the main menu, Wally's pointer and the Pokédex entry | 2026-09-24 |
| [97](tools-and-rig.md) | replays in the background, and what a faithful one writes | 2026-09-25 |
| [98](screens-and-cursors.md) | the Poké Mart's Yes and No, and a window that holds panes | 2026-09-25 |
| [99](memory-and-discovery.md) | chains from the executable's globals, and the app that is running | 2026-09-25 |
| [100](field-and-rom.md) | Pokémon X's field, and what it keeps where ORAS does not | 2026-09-24 |
| [101](field-and-rom.md) | X's Pokémon Center door, named by its Town Map | 2026-09-25 |
| [102](field-and-rom.md) | Lumiose City's boulevards are rails, not cells | 2026-09-25 |
| [103](field-and-rom.md) | Lumiose's rails, their walls, and which way the stick walks them | 2026-09-25 |
| [104](field-and-rom.md) | on Lumiose's rails the D-pad turns the player | 2026-09-25 |
| [105](field-and-rom.md) | on Lumiose's rails up walks where the camera looks, and a tap leaves it anywhere | 2026-09-25 |
| [106](field-and-rom.md) | Lumiose's streets load with their chunks, and the ROM has them all | 2026-09-25 |
| [107](field-and-rom.md) | north of the blackout: how a line is measured, its tracks and joints, and the names on the doors | 2026-09-25 |
| [108](field-and-rom.md) | X's people, named by sprite | 2026-09-25 |
| [109](field-and-rom.md) | ORAS's plates, and the town's own word for a plate with a slogan | 2026-09-25 |
| [110](memory-and-discovery.md) | the game's heaps, and what they have just allocated | 2026-09-26 |
| [111](memory-and-discovery.md) | Pokémon Sun: its text, its dialogue, and the memory it may be read in | 2026-09-26 |
| [112](screens-and-cursors.md) | Pokémon Sun's menus: ORAS's layouts, another pane shape, and a cursor pane | 2026-09-26 |
| [113](field-and-rom.md) | Sun's field | 2026-09-26 |
| [114](field-and-rom.md) | Sun's walls, out of the ROM | 2026-09-26 |
| [115](field-and-rom.md) | What the things on Sun's field are | 2026-09-26 |
| [116](memory-and-discovery.md) | Where a Sun sweep's time goes | 2026-09-26 |
| [117](field-and-rom.md) | People and items on Sun's field that nobody sees | 2026-09-26 |
| [118](field-and-rom.md) | Sun's props, and a way that ended inside a flower pot | 2026-09-26 |
| [119](field-and-rom.md) | Which way up walks in Sun | 2026-09-26 |
| [120](field-and-rom.md) | Heights on Sun's ground | 2026-09-26 |
| [121](tools-and-rig.md) | a replay pinned to one faithful run's load timings | 2026-09-27 |
| [122](tools-and-rig.md) | pinning a movie a Bag at a time | 2026-09-27 |
| [123](memory-and-discovery.md) | Sun's heaps, watched as ORAS's are | 2026-09-27 |
| [124](battle.md) | Sun's first trainer battle: the HP box, its gauges, the command menu | 2026-09-27 |
| [125](screens-and-cursors.md) | Sun's picture-only buttons, and what their panes are called | 2026-09-27 |
| [126](field-and-rom.md) | which way a ledge drops | 2026-09-27 |
| [127](screens-and-cursors.md) | the PC Box's menu for a chosen Pokémon, and a cursor spelt `cursol` | 2026-09-27 |
| [128](screens-and-cursors.md) | what a picture under a cursor draws, where no words are | 2026-09-27 |
| [129](screens-and-cursors.md) | the PC Box's slots: its grid, its party and a Pokémon's marks | 2026-09-27 |
| [130](memory-and-discovery.md) | what the net finds that the heaps' watch missed, and where | 2026-09-28 |
| [131](field-and-rom.md) | what a battle does to the thing being walked to | 2026-09-28 |
| [132](field-and-rom.md) | Dewford Gym's conveyor belts, and what a trigger's fields say | 2026-09-28 |
| [133](memory-and-discovery.md) | what the sweeps of new blocks read, and what they find | 2026-09-28 |
| [134](memory-and-discovery.md) | why each whole sweep happens, and what it costs | 2026-09-28 |
| [135](field-and-rom.md) | what the "kakuremino" classes are, and what holds the DexNav's record | 2026-09-28 |
| [136](field-and-rom.md) | water, and the way across it with Surf | 2026-09-28 |
| [137](field-and-rom.md) | the bush Cut clears and the rock Rock Smash breaks, and the way through them | 2026-09-28 |
| [138](battle.md) | why a battle's first HP line came late, and the gauges found by class | 2026-09-29 |
| [139](screens-and-cursors.md) | the Bag's party list as a TM is taught, and the move-forget screen | 2026-09-28 |
| [140](memory-and-discovery.md) | where the game's code modules load, and why: after whatever is loaded beneath them | 2026-09-29 |
| [141](field-and-rom.md) | the berry spots, which no zone's data has | 2026-09-29 |
| [142](field-and-rom.md) | the way that went quiet halfway up Route 110 | 2026-09-29 |
| [143](screens-and-cursors.md) | the PC Box's buttons drawn as pictures alone: three modes, two arrows, and the touch bar's return, exit and search | 2026-09-29 |
| [144](battle.md) | the HP box while a hit lands and while a Pokémon faints: the player's panel away, the enemy's 0% on screen 2.5 s | 2026-09-29 |
| [145](field-and-rom.md) | the Dowsing Machine's lights: a colour for how near, lit only facing the item, and the record behind them | 2026-09-29 |
| [146](field-and-rom.md) | Mauville Gym's barriers, switches and two rooms | 2026-09-29 |
| [147](screens-and-cursors.md) | a picture's image read whole from its material, and named by the ROM texture it is: the Pokédex's language tabs and its type | 2026-09-29 |
| [148](screens-and-cursors.md) | the summary's labels told from their values by place, a colon after each with pauses on, and HP's slash | 2026-10-01 |
| [149](tools-and-rig.md) | the battle crashes: a 128 KB image read while the GPU drew into it, azahar-emu/azahar#2619 | 2026-09-30 |
| [150](battle.md) | a double battle's gauges, a group each, and a horde's five | 2026-10-01 |
| [151](field-and-rom.md) | talking across a counter: the one kind of cell the game talks across, and the way to the Nurse and a Poké Mart's Clerk | 2026-10-01 |
| [152](screens-and-cursors.md) | the Pokédex's habitat: its places drawn on the AreaNav's map, named through the AreaNav's list | 2026-10-01 |
| [153](screens-and-cursors.md) | the habitat's grass, water and rod icons, when its map is open, and the Pokédex's delay | 2026-10-01 |
| [154](field-and-rom.md) | Secret Bases: every spot in the executable, the player's own in the save, the way out, the PC and flag, and the secret team in the walls; decorating, the floor cursor's cell and what stands in each cell, which P had walked through | 2026-10-01 |
| [154](screens-and-cursors.md) | a Secret Base's menus: their cursor a pane a row too small or two rows too tall, and words rewritten under it | 2026-10-01 |
| [155](field-and-rom.md) | Route 113's ash piles: a list the route's own module writes, and a byte a pile in the save's gimmick work | 2026-10-02 |
| [156](field-and-rom.md) | Mt. Chimney's catwalk: a chunk with a deck has a cell table and a height mesh a floor, nine or eleven sections, and P walks the floor at the player's height | 2026-10-02 |
| [157](field-and-rom.md) | Lavaridge Gym's trapdoors and geysers: 24 one-cell triggers, a trapdoor dropping the player into the hot spring and a geyser shooting a raft back up | 2026-10-03 |
| [158](field-and-rom.md) | the Mach Bike: a bit of the player's move model while riding; the four sandy slopes' kinds, a way each, by the ground's mesh; and Jagged Pass's Hiker behind ledges | 2026-10-03 |
| [159](field-and-rom.md) | Route 119's bridge: P's ground walked the path under it and cut it, the only way north; the zones east Hoenn's places are split into, whose edges said no place | 2026-10-03 |
| [160](field-and-rom.md) | Fortree City: platforms beside the ground in one cell table, ladders as pairs of triggers, drops of two cells' height, and the invisible Kecleon in the Gym's one way in | 2026-10-03 |
| [161](field-and-rom.md) | Fortree Gym's rotating doors: the Gym's module builds seven, keeps each one's pattern and turn, and its tables say every pattern's arms; P searches through them | 2026-10-03 |
