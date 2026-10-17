# Changelog

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
