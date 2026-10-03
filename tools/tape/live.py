"""Replay a movie with the Rust `pa3ds.exe` reading the running emulator, as `regress.py run` does
with the Python runtime, and write what it logged beside a Python run of the same movie.

    <python-repo>/.venv/Scripts/python.exe tools/tape/live.py <replay>... [--python]

A live run is the real thing: the emulator, the clock and the executable a player runs. Two live
runs of one commit differ by tens of lines (`docs/runbooks/rig.md`), so a live run is judged
against a Python run of the same movie and the noise between two Python runs, never line for line;
the tapes are the line-for-line check (`tools/tape/README.md`).

Each run writes `%LOCALAPPDATA%/pa3ds/captures/regress/<label>/<replay>.log`, `rust-live` for the
executable and `python-live` with `--python`, in `regress.py run`'s format, so
`regress.py compare python-live rust-live` diffs them.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PYTHON_REPO = Path(os.environ.get("PA3DS_PYTHON", ROOT.parent / "pokemon-access-3ds"))
sys.path.insert(0, str(PYTHON_REPO / "spikes"))
sys.path.insert(0, str(PYTHON_REPO / "src"))

import regress  # noqa: E402
from rig import MOVIES, ROM, Rig, saves  # noqa: E402

EXECUTABLE = ROOT / "target" / "release" / "pa3ds.exe"


def run_rust(replay: regress.Replay, folder: Path) -> str:
    """One replay with the executable reading it; returns the run's summary line."""
    path = folder / f"{replay.name}.log"
    with saves(replay.saves), Rig(movie=MOVIES / replay.movie, rom=replay.rom or ROM) as rig:
        if replay.slot is None:
            rig.boot(state=None, wait=1, frame_limit=100, deterministic=replay.deterministic)
        else:
            rig.boot(state=replay.slot, frame_limit=100, deterministic=replay.deterministic)
        # The replays' own user folder, never the player's: as `regress.settings_at` gives the
        # Python runtime. No keys are bound, as a replay binds none.
        env = dict(os.environ, PA3DS_HOME=str(regress.REPLAY_HOME))
        began = time.monotonic()
        done = subprocess.run(
            [str(EXECUTABLE), "--silent", "--no-keys", "--log-path", str(path),
             "--seconds", str(replay.seconds)],
            env=env, capture_output=True, text=True, encoding="utf-8", errors="replace",
            check=False)
        took = time.monotonic() - began
    with path.open("a", encoding="utf-8") as out:
        out.write(f"{took:8.2f} done\n")
    drift = regress.EMULATOR_LOG.read_text(encoding="utf-8", errors="replace").count(
        "out of sync")
    followed = None if not replay.says else regress.says_all(path, replay.says)
    tail = done.stderr.strip().splitlines()[-1:] if done.returncode else []
    return (f"{replay.name}: exit {done.returncode}, {len(regress.lines(path))} lines, out of "
            f"sync {drift}, followed {followed}" + (f", {tail[0]}" if tail else ""))


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("replays", nargs="+")
    parser.add_argument("--python", action="store_true",
                        help="run the Python runtime instead, as the control")
    args = parser.parse_args()
    known = {replay.name: replay for replay in regress.replays()}
    label = "python-live" if args.python else "rust-live"
    folder = regress.CAPTURES / label
    folder.mkdir(parents=True, exist_ok=True)
    for name in args.replays:
        replay = known[name]
        if args.python:
            drift, followed = regress.run_one(replay, folder)
            print(f"{name}: {len(regress.lines(folder / f'{name}.log'))} lines, out of sync "
                  f"{drift}, followed {followed}")
        else:
            print(run_rust(replay, folder))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
