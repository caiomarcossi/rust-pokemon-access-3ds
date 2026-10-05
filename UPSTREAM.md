# For upstream

Bugs and oddities found in the Python pa3ds while porting it, each ported here as it is, since
the port behaves as upstream does (`AGENTS.md`). Fix one upstream first; the fix then arrives here
by `tools/sync.py` like any other change. Each entry names the upstream commit it was found at.

## Smaller

- `games/sm/battle.py`: `SunBattleReader.track(adding=True)` does not drop gauges already held, so
  its list can hold one twice. Found at `349fd43`.

## Larger

- `games/gen6/sky.py`: `flocks` walks the models' links `while within(HEAP_SPAN, model) and
  len(seen) < MODELS`, but a model already seen is not added again, so links that lead back to
  one (a model freed and its links rewritten, or one linking to itself) never grow `seen` and the
  walk never ends, hanging the runtime while the player soars. Found at `f0b9a94`.
