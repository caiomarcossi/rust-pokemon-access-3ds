# For upstream

Bugs and oddities found in the Python pa3ds while porting it, each ported here as it is, since
the port behaves as upstream does (`AGENTS.md`). Fix one upstream first; the fix then arrives here
by `tools/sync.py` like any other change. Found at upstream `349fd43`.

## Bugs

- **`core/memory.py`, `PageCache.phase`.** The phase is pushed onto `_phases` before its first
  fetch, and the fetch is outside the `try`: when it raises (the emulator stalling), the phase is
  never popped, and from then on the cache keeps pages outside any phase until the next phase
  clears them.
- **`core/config.py`, `prepare`.** Its docstring says nothing raises at a player, but four things
  do: `float(scale)` on an Azahar `modifier_scale` that is not a number (`ValueError`);
  `qt_code("ß")`, where `ord()` is handed a two-character upper case (`TypeError`);
  `keyboard_code` on `code:²`, which passes `isdigit` and fails `int()` (`ValueError`); and a
  `qt-config.ini` or `config.toml` that is not UTF-8 (`UnicodeDecodeError`). A `config.toml` key
  such as `__doc__ = "x"` passes `hasattr` and then makes `dataclasses.replace` raise
  `TypeError`.
- **`core/toml.py`, `read`.** Lets `UnicodeDecodeError` out, against its docstring.
- **`games/gen6/field.py`, `_bike_ways`.** Looks up the corner of a slope patch's bounding box
  among the patch's cells; for an L-shaped patch the corner is not one of them, and it raises
  `KeyError`.
- **`games/common/slots.py`.** Reads panes at ORAS's offsets (`lyt.X`, `lyt.Y`, `lyt.WIDTH`,
  `lyt.HEIGHT`, `lyt.NAME`, `lyt.ALPHA`) rather than the layout's shape's, so slots are read at the
  wrong offsets in Sun.
- **`games/common/announce.py`, `ScreenAnnouncer._detail_panels`.** Never pruned: it grows for as
  long as a session runs.
- **`games/gen6/textures.py`.** The `except (IndexError, ValueError)` around `darc_files` can never
  fire, since `darc_entries` raises `struct.error`; and `said` reads `conditions[-1]` for
  `sick_icon_00_`, raising `IndexError` on an empty list.

## Smaller

- `core/names.py`: `Names.__init__` reads the names file twice, once a section; the module
  docstring repeats a paragraph.
- `games/sm/battle.py`: `SunBattleReader.track(adding=True)` does not drop gauges already held, so
  its list can hold one twice.
- `tests/test_heaps.py`, `test_an_unread_heaps_own_blocks_are_not_walked...`: `Guarded(own)` is
  given `(address, 0x40)`, read as the span from `address` to `0x40`, which is empty, so the guard
  it sets never fires.
- Stray docstrings, attached to no name: after `MATERIAL_FORMAT` in `games/common/lyt.py`, after
  `DIRECTORY` in `games/common/layout_file.py` (it describes `MAX_ARCHIVE`), after `_settling` in
  `games/common/layout.py` (it describes `_walked`), and a second one after `_slack` in
  `games/common/navigate.py`.
