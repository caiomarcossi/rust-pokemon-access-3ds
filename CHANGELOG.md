# Changelog

## [0.5.0] - 2026-10-20

### Added

- Touch mode reads Pokémon-Amie's decorate screen: its arrows, return and exit, each Poké Puff by
  its kind and each Interior Item by name.
- More scenes are described under `descriptions`: May's room before you meet her, the Beautifly
  on Route 102, and the mural in Granite Cave, each version's own, then Steven leaving.

### Fixed

- In X and Y, the Pokémon-Amie page names your Pokémon and its menu offers PLAY, DECORATE and
  SWITCH; the Super Training page names your party, so a tap changes Pokémon, and names its
  training bags, its regimens and its Super Training button.
- While a panel or a Yes and No is up in Super Training, touch mode leaves out the buttons behind
  it that a tap no longer reaches, and the PlayNav's arrows and the X menu's icons while a page
  has the input.
- The PC's menus say only the row the cursor moves to, not the other rows as new.
- X's language select says the language: "Play Pokémon X in English".
- The Bag says "Nothing" at an empty pocket again.
- A TM named for its move's type, as TM29 Psychic is, says its type among its details.
- The Bag no longer stops with a reader error as it opens.

## [0.4.0] - 2026-10-19

### Added

- pa3ds's own words, the names it gives and the scenes it describes are in your game's language:
  German, English, Spanish, French, Italian, Japanese, Korean and Chinese. Omega Ruby, Alpha
  Sapphire, X and Y take it from your save; in Sun and Moon, set `language` under `[pa3ds.ui]`.
  The translations began as a machine's. The Languages page of the manual says how to correct a
  word or translate pa3ds into another language.
- Footsteps where Omega Ruby and Alpha Sapphire play none, as on streets, floors and in caves: the
  game's own step sound, as loud as the game and Azahar would play it.
- `[pa3ds.audio]` sets how loud pa3ds's own sounds are beside the game's: `tones` for the map's
  cursor and touch mode, `footsteps` for the footsteps. 1 is as loud as the game, 0 none. If you
  had turned the tones off, they stay off.
- The battle's move info screen is read: each move's category, power, accuracy and description,
  with the four moves said as "move 1" to "move 4". So is the move to forget after a level up.

### Changed

- The map's signs category is called interactables, in every language.
- The tones follow Azahar's volume slider and its slider in Windows' mixer, as the footsteps do.
- The manual is a short README, with pages for keys, settings and languages in the `manual`
  folder.
- The `data` folder holds only pa3ds's words, the files you translate. The rest is built into
  pa3ds.

### Fixed

- The Bag says only what is new as you move: not the pocket and the item's name again before its
  count and description, not the party's ABLE!, UNABLE! and levels at every TM, and not the whole
  Bag as it comes back from teaching a move. The move to forget no longer reads the Pokémon's
  number, name and stats with every move.
- pa3ds no longer stops while you soar on Latios or Latias.
- On Sharpedo, on Kyogre and under the water, pa3ds no longer says you are on the bike, and says
  where a current that catches you will leave you.
- Battles are said on the Chinese Omega Ruby translated by fans.
- X and Y's Poké Mart says your money, each item's description and how many you are buying.
- The moving truck at the start of Omega Ruby and Alpha Sapphire lists its way out to Littleroot
  Town.
- A comment in `config.toml` or `keys.toml` holding an unusual line break no longer breaks the
  file when pa3ds adds a setting to it.
- The manual names `U` as touch mode's tap key, not Enter.

## [0.3.0] - 2026-10-17

### Added

- While Azahar is open, pa3ds hides its own window, so it is not in Alt+Tab between you and the
  game. When you close Azahar, pa3ds closes too.
- To restart pa3ds, start it again: the new one closes the one running, says "pa3ds restarted",
  and carries on the same log.
- With a portable Azahar, pa3ds asks for the folder `azahar.exe` is in the first time it finds no
  Azahar settings, records it as `azahar_folder` in `user\config.toml`, and changes that Azahar's
  settings in the same start.
- `keys.toml` and `config.toml` gain the keys and settings each new version of pa3ds adds, and
  never lose a line you have. A new key whose default you already use for something else is added
  with a `#` in front of it, and pa3ds says so as it starts.
- If `config.toml` or `keys.toml` has a mistake pa3ds cannot read past, pa3ds says so as it
  starts, with the line the mistake is on, and uses its defaults until you fix it.

## [0.2.1] - 2026-10-17

### Fixed

- `P` finds the way across Santalune Gym's spider web in X and Y, along its diagonal threads, and
  says a step along one as "1 north-east".
- In Santalune Gym, the pole down to the web and the one back up are exits, as are the stairs past
  Viola's tent, and the lobby's statues and photographs are said as statues and photographs.
- The gate behind Viola in Santalune Gym is no longer listed as five nameless people.

## [0.2.0] - 2026-10-17

### Added

- X and Y name what is on their maps the way Omega Ruby and Alpha Sapphire do: their people
  ("Dexio", "Hairdresser"), the Pokémon walking about ("Fletchling"), things like the bedroom's
  Kalos Pokédex awards, Berry trees, sparkles and cones, and what you face and read, a TV, a trash
  can, an elevator, a photo spot, a bed, a Gogoat Shuttle stop.
- X and Y's obstacles: bushes for Cut, cracked boulders and cracked walls for Rock Smash, and big
  boulders for Strength.
- More scenes described with the `descriptions` setting. In Omega Ruby: Wally on Victory Road, the
  Elite Four's bridges as you cross them and their rooms, Steven and the Hall of Fame, the
  credits, May's night on Route 103, and the Delta Episode's opening. In Alpha Sapphire: the Hall
  of Fame, the credits, and the Delta Episode's opening.
- The credits name the Pokémon your own save remembers from each Gym and great battle, by the
  nickname you gave it, or else by its species in the form it had then ("Primal Reversion Groudon").

### Changed

- In your house in X and Y, only the room you are in is listed, with its stairs.
- Sun and Moon's passport photos are described in every language, in English for now.
- The release zip keeps every licence in one `licenses` folder, one file each.

### Fixed

- Doors and door mats facing east, west or south are said as a door or a way out instead of a
  warp.
- Lumiose City's openings onto the next street are said as paths.
- Pokémon walking about in X and Y, like Mom's Rhyhorn, are named instead of said as a person.

## [0.1.0] - 2026-10-16

The first release. pa3ds is a screen reader accessibility layer for 3DS Pokémon games running in the Azahar emulator: it reads the game's own memory while you play and sends what it finds to your screen reader, every word in your game's language. It reads Omega Ruby and Alpha Sapphire most completely, X and Y with some screens still silent, and Sun and Moon's dialogue, menus and screens, name entry, battles and map.
