# For upstream

Bugs and oddities found in the Python pa3ds while porting it, each ported here as it is, since
the port behaves as upstream does (`AGENTS.md`). Fix one upstream first; the fix then arrives here
by `tools/sync.py` like any other change. Found at upstream `349fd43`; the rest of what was
found there was fixed by `02e463e`.

## Smaller

- `games/sm/battle.py`: `SunBattleReader.track(adding=True)` does not drop gauges already held, so
  its list can hold one twice.
