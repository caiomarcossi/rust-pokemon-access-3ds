# The PokéNav Plus, and touching the bottom screen

**Status:** decided, and steps 1 to 4 built (2026-10-01); steps 5 to 7 to come. Written 2026-10-01, after
Orion asked for the PokéNav Plus: "I want a legitimately obtained shiny Ralts and to play with
and super train my Pokémon." Roadmap milestone 6.

## The problem

The PokéNav Plus is ORAS's bottom screen whenever the player is on the field. It has four apps:
the AreaNav (the map), the DexNav (the wild Pokémon here, and searching for one), the PlayNav
(trading, the PSS, Pokémon-Amie and Super Training) and the BuzzNav (news and TV). The player
switches between them by touching their icons, and nearly everything inside them is touched too.

Three things make it harder than anything pa3ds has read so far:

- **pa3ds cannot read what nobody has recorded.** Every reader so far was built from a movie, and
  Orion cannot record a movie of a screen he cannot use: movies 3 and 4 have the DexNav coming
  up on its own as a hidden Pokémon appears, and that is all.
- **The player cannot touch.** A mouse pointer has to land on a spot of a window whose size,
  layout and full-screen state are the player's, and the spot moves when the game rearranges a
  screen: the X menu has no Pokédex until the player has one, and the PokéNav has no BuzzNav for
  Orion yet.
- **The apps are not all ORAS's.** The PlayNav's Amie and Super Training are on X and Y's bottom
  screen too, and probably read the same way; the DexNav and the AreaNav are ORAS's alone; Sun and
  Moon's bottom screen is the Rotom Dex, which nobody has looked at.

## What is known

- **Every button on these screens is a touch area pa3ds can already find and place.** The game
  draws a touch area as an `nw::lyt::Bounding` pane, and pa3ds reads those for the PC Box's slots
  and buttons (spikes 129 and 143) and the habitat's map (spike 152). At 17.5 s into
  or/movie7_menus, on the field in Petalburg City, the bottom screen holds the AreaNav's layout
  (four touch areas and its words, "Petalburg City"), the PlayNav's backgrounds (`pss_bg.bclim`,
  three 40 by 40 buttons drawn with `menu_bg_button.bclim`, not shown then), a layout of 70 by 30
  buttons, and the Hoenn map's 72 places, each a touch area the ROM names (spike 152). Several of
  these layouts were not matched to their files yet, so what each button is called is not known.
- **Where a touch area is, is where the game will be touched.** A pane's place is in the bottom
  screen's own units, 320 by 240 from its middle, which are the touch screen's pixels: no window,
  scale or full-screen state comes into it.
- **The ROM holds every app's layouts**, the BuzzNav's included, so what each screen *can* show
  is read without the player having it (spike 96 walks every layout in the ROM).
- **Stock Azahar takes touches from another program.** Its HID service reads the touch screen from
  the mouse first and, while the mouse is not pressing, from a second "controller" touch device
  (`core/hle/service/hid/hid.cpp`, `controller_touch_device`), which can be Citra's cemuhook UDP
  client (`input_common/udp/`): a program on the same machine, a *DSU server*, says where a
  touchpad is pressed, and Azahar scales that to the touch screen by a calibration in the same
  setting (`min_x`, `max_x`, `min_y`, `max_y`). The mouse keeps working beside it. The protocol
  is a public one (DS4Windows and others speak it); nothing of Azahar's is needed to speak it.
- **A movie records the touch screen** (`core/movie.cpp`, `ControllerStateType::Touch`), so once
  the player can touch, the player can record a movie of the PokéNav and the rig can replay it.
- **Amethyst's touch module for Black and White does not move the pointer.** It writes the touch
  straight into the emulator's input, in the DS screen's own coordinates, a press as frames held
  and a release after them, with a watchdog that never leaves a finger down
  (`pokemon black and white\Lua\core\touch.lua`; its behaviour read, its code not used). The same
  shape fits a DSU server.
- **During a movie's replay the movie's touches replace any other**, so testing a touch needs a
  boot that plays no movie, which the rig does not have yet (`Rig` always passes `-p`).

## Decided (Orion, 2026-10-01)

1. **A tap, not the mouse**: pa3ds is the cemuhook server Azahar's second touch device reads,
   and taps the target it has just named on the player's key. ADR-0009 is amended for it. Moving
   the mouse needed the bottom screen's place under every layout and window size; our fork
   touching would have meant shipping the fork (ADR-0005); Azahar's touch-from-button binds a
   key to one fixed spot.
2. **Keys of their own, rebindable** (`[pa3ds.navigation.touch]`): the mode key steps on
   to touch mode after direction mode (shift and T for a day); J and L walk, I and O
   a row at a time, K again, Enter taps (ADR-0022).
3. **Taps first**; drags and holds once a reader has measured what each wants.

## Steps

Each step is useful on its own, and none needs a movie the player cannot make.

1. **Done: the census** (findings, spike 171). On the field: the app buttons, named by the ROM's
   text file 123 ("DexNav"), the AreaNav's window onto the map, and the X menu's six icons along
   the bottom as a shortcut bar, named by the X menu's own hidden words. The map's places are
   left to the AreaNav's reader.
2. **Done: touch targets** (`games.common.touch`): touch mode walks and says them.
3. **Half done: a boot without a movie** (`Rig(movie=None)`): it boots a save, but the game stops
   at PRESS START, which a tap does not pass, and a state saved in a replay loads only with its
   movie playing (findings, spike 172). Left: a movie cut short so it ends on the field, which
   hands the game to live input, or Orion's own Azahar, as the tap was proved on.
4. **Done: the tap** (`pa3ds.core.touch`): Orion's Azahar opened the DexNav, tapped at (285, 53)
   (findings, spike 172). A player's Azahar needs `use_touchpad` and `controller_touch_device`,
   which `config.toml`'s `[azahar]` patches in while Azahar is closed.
5. **Movies, by Orion**, once he can touch: the DexNav (searching, a chain, a hidden Pokémon's
   details), the AreaNav, the PlayNav's Amie and Super Training; each with a `[[replay]]`.
6. **A reader an app**, from those movies: **the AreaNav first** if it is the simpler (Orion), its
   map's places named already (spike 152); the DexNav (the shiny Ralts); Amie and Super
   Training, which X and Y share; the BuzzNav when Orion has it.
7. **X and Y, and Sun and Moon.** X and Y draw Amie and Super Training on the bottom screen with
   no PokéNav round them: the same tap, and step 1's census on X's movies. Sun's Rotom Dex reads
   through Sun's pane shape (spike 112), and nothing of it is known.

## Not in this plan

- Calling the game's own functions instead of touching (roadmap, deferred: it needs code
  injection).
- Writing the touch into the game's memory: ADR-0009 forbids it, and the emulator overwrites it
  every frame anyway.
- Touching anything the player did not ask for at that moment: pa3ds taps only on the player's
  key, the target it has just named.
