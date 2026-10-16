"""Record a movie's replay through the Python runtime as a tape, or replay a tape through it.

Run with the Python checkout's own interpreter, which has pa3ds and the rig:

    <python-repo>/.venv/Scripts/python.exe tools/tape/py.py record cave tapes/cave.tape.gz
    <python-repo>/.venv/Scripts/python.exe tools/tape/py.py replay tapes/cave.tape.gz out.log

`record` plays the `[[replay]]` of that name from the Python checkout's `spikes/rig.toml`, as
`spikes/regress.py run` does, and writes the tape and, beside it, the lines the runtime logged
(`<tape>.log`, without times) and what it said (`<tape>.said`, `!` before a line that
interrupts). `replay` runs the Python runtime on the tape alone and writes what
it logged, and what it said beside it; a sound tape's are identical to the recording's.

A tape named `<replay>_described` plays `<replay>` with `[pa3ds.ui] descriptions` on, recorded
and replayed alike, so the scenes are checked as well.

The Python checkout is `PA3DS_PYTHON`, or `../pokemon-access-3ds` beside this repository.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PYTHON_REPO = Path(os.environ.get("PA3DS_PYTHON", HERE.parents[2] / "pokemon-access-3ds"))
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(PYTHON_REPO / "spikes"))
sys.path.insert(0, str(PYTHON_REPO / "src"))

import tape  # noqa: E402 -- the clock must be patched before pa3ds loads


class Heard:
    """A backend that keeps what would be said: `!` before a line that interrupts."""

    def __init__(self, said: list[str]) -> None:
        self._said = said

    def speak(self, text: str, interrupt: bool = False) -> None:
        self._said.append(("!" if interrupt else "") + text)

    def braille(self, text: str) -> None:
        pass


def strip_time(line: str) -> str:
    return line[9:] if len(line) > 9 and line[8] == " " else line


DESCRIBED = "_described"
"""What a tape's name ends in when it was recorded with `descriptions` on."""


def described(tape: Path) -> bool:
    return tape.name.removesuffix(".tape.gz").endswith(DESCRIBED)


def with_descriptions(regress, tape: Path) -> None:  # noqa: ANN001
    """Have `regress.settings_at` turn descriptions on for a described tape."""
    if not described(tape):
        return
    import dataclasses

    from pa3ds.core.config import Ui

    plain = regress.settings_at

    def settings_at(speed):  # noqa: ANN001, ANN202
        settings = plain(speed)
        return dataclasses.replace(settings,
                                   ui=dataclasses.replace(settings.ui, descriptions=True))

    regress.settings_at = settings_at


def record(name: str, out: Path) -> int:
    writer = tape.Writer(out)

    def monotonic(real):  # noqa: ANN001, ANN202
        value = real()
        writer.tag(tape.CLOCK)
        writer.f64(value)
        return value

    def sleep(real, seconds):  # noqa: ANN001, ANN202
        writer.tag(tape.SLEEP)
        writer.f64(seconds)
        real(seconds)

    tape.patch_clock(monotonic, sleep)
    import regress

    regress.Rpc = tape.recording_rpc(writer)
    with_descriptions(regress, out)
    name = name.removesuffix(DESCRIBED)
    said: list[str] = []
    regress.Silent = lambda: Heard(said)
    replay = next((r for r in regress.replays() if r.name == name), None)
    if replay is None:
        print(f"no [[replay]] named {name} in rig.toml", file=sys.stderr)
        return 2
    folder = out.parent / (out.name + ".run")
    folder.mkdir(parents=True, exist_ok=True)
    try:
        drift, followed = regress.run_one(replay, folder)
    finally:
        writer.close()
    logged = (folder / f"{replay.name}.log").read_text(encoding="utf-8").splitlines()
    lines = [strip_time(line) for line in logged if strip_time(line) != "done"]
    Path(f"{out}.log").write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")
    Path(f"{out}.said").write_text("".join(f"{line}\n" for line in said), encoding="utf-8")
    print(f"{out}: {writer.count} records, {len(lines)} lines, out of sync {drift}, "
          f"followed {followed}")
    return 0


def replay(path: Path, out: Path) -> int:
    reader = tape.Reader(path)

    def monotonic(_real):  # noqa: ANN001, ANN202
        reader.expect(tape.CLOCK, "clock")
        return reader.f64()

    def sleep(_real, seconds):  # noqa: ANN001, ANN202
        reader.expect(tape.SLEEP, f"sleep {seconds}")
        recorded = reader.f64()
        if recorded != seconds:
            raise tape.Diverged(f"record {reader.count}: sleep {seconds}, the tape has {recorded}")

    tape.patch_clock(monotonic, sleep)
    import regress

    from pa3ds.app import run
    from pa3ds.core.speech import Speech

    with_descriptions(regress, path)
    lines: list[str] = []
    said: list[str] = []
    ending = "end of tape"
    try:
        run(tape.replaying_rpc(reader)(), Speech(Heard(said)), regress.settings_at(1),
            until=lambda: False, log=lines.append)
    except tape.TapeEnd:
        pass
    except tape.Diverged as diverged:
        ending = f"diverged: {diverged}"
    out.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")
    out.with_suffix(".said").write_text("".join(f"{line}\n" for line in said), encoding="utf-8")
    print(f"{out}: {reader.count} records, {len(lines)} lines, {len(said)} said, {ending}")
    return 0 if ending == "end of tape" else 1


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    rec = sub.add_parser("record")
    rec.add_argument("replay")
    rec.add_argument("tape", type=Path)
    rep = sub.add_parser("replay")
    rep.add_argument("tape", type=Path)
    rep.add_argument("log", type=Path)
    args = parser.parse_args()
    if args.command == "record":
        return record(args.replay, args.tape)
    return replay(args.tape, args.log)


if __name__ == "__main__":
    raise SystemExit(main())
