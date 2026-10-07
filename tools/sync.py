"""Keep the Rust port in step with the Python pa3ds it was ported from.

The Python checkout is upstream: features land there first, and this repository follows. Every
Python file has one place here (`rust_path`), so a change upstream maps to a file to change here.
`port.toml` records the upstream commit the port matches.

    python tools/sync.py status            what changed upstream since the commit port.toml names
    python tools/sync.py layout            create the Rust file for any Python file without one
    python tools/sync.py mirror            copy the files carried over as they are (data, licence)
    python tools/sync.py tests             every Python test without a Rust test of its name
    python tools/sync.py mark [<commit>]   record that the port now matches <commit> (default HEAD)

The Python checkout is `PA3DS_PYTHON`, or `../pokemon-access-3ds` beside this repository. Needs
only the standard library and git.
"""

from __future__ import annotations

import ast
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON = Path(os.environ.get("PA3DS_PYTHON", ROOT.parent / "pokemon-access-3ds"))
MANIFEST = ROOT / "port.toml"

MIRRORED = ("LICENSE", "tests/fake_qt_config.ini")
"""Carried over byte for byte."""
ADAPTED = ("README.md", "AGENTS.md", "CLAUDE.md")
"""Written again for the Rust repository: they name commands and tools."""
NOT_PORTED = ("spikes/", "docs/", "GLOSSARY.md", "pyproject.toml", "uv.lock", ".gitignore",
              ".claude/", ".github/")
"""Stay upstream. A spike records how a finding was reached, run against the Python package;
the rig drives the emulator and ships with nothing. The decisions, findings and vocabulary are
the same whichever language reads the game, so they are read in the Python checkout rather than
copied here."""


def rust_path(python: str) -> str | None:
    """Where a Python file's port lives, relative to this repository, or None for a file that is
    not ported (`NOT_PORTED`)."""
    if python.startswith(NOT_PORTED):
        return None
    if python.startswith(MIRRORED) or python in ADAPTED:
        return python
    if python.startswith("src/pa3ds/"):
        rel = python.removeprefix("src/pa3ds/")
        if "/data/" in rel:
            return f"src/{rel}"
        if rel == "__init__.py":
            return "src/lib.rs"
        if rel == "__main__.py":
            return "src/__main__.rs"
        if rel.endswith("/__init__.py"):
            return f"src/{rel.removesuffix('/__init__.py')}/mod.rs"
        if rel.endswith(".py"):
            return f"src/{rel.removesuffix('.py')}.rs"
    if python.startswith("tests/") and python.endswith(".py"):
        return python.removesuffix(".py") + ".rs"
    return None


def git(*args: str) -> str:
    return subprocess.run(["git", "-C", str(PYTHON), *args], check=True, capture_output=True,
                          text=True, encoding="utf-8").stdout


def manifest() -> dict[str, object]:
    with MANIFEST.open("rb") as file:
        return tomllib.load(file)


def tracked() -> list[str]:
    return [path for path in git("ls-files").splitlines() if path]


def status() -> int:
    upstream = str(manifest()["upstream"])
    head = git("rev-parse", "HEAD").strip()
    print(f"port.toml matches {upstream[:9]}; upstream HEAD is {head[:9]}")
    if git("status", "--porcelain", "--", "src", "tests"):
        print("upstream has uncommitted changes; they are not counted")
    if head == upstream:
        print("nothing to sync")
        return 0
    print("\ncommits:")
    print(git("log", "--oneline", "--no-decorate", f"{upstream}..{head}"))
    changes = [line.split("\t") for line in
               git("diff", "--name-status", "-M", f"{upstream}..{head}").splitlines()]
    port, mirror, adapt, skipped = [], [], [], []
    for change in changes:
        kind, paths = change[0], change[1:]
        target = rust_path(paths[-1])
        line = f"  {kind[0]} {' -> '.join(paths)}"
        if target is None:
            skipped.append(line)
        elif paths[-1].startswith(MIRRORED) or "/data/" in paths[-1]:
            mirror.append(line)
        elif paths[-1] in ADAPTED:
            adapt.append(line + f"   (rewrite by hand: {target})")
        else:
            port.append(line + f"   -> {target}")
    for title, lines in (("port by hand", port), ("copied by `sync.py mirror`", mirror),
                         ("adapted by hand", adapt), ("not ported", skipped)):
        if lines:
            print(f"{title}:")
            print("\n".join(lines))
    print(f"\neach file's change: git -C {PYTHON} diff {upstream[:9]}..HEAD -- <path>")
    return 0


def layout() -> int:
    """Create an empty Rust file for every ported Python module and test, and declare every
    module in its parent's `mod.rs` (or `lib.rs`), so each file compiles as soon as it exists.
    Never overwrites a file."""
    created = 0
    for python in tracked():
        target = rust_path(python)
        if target is None or not target.endswith(".rs"):
            continue
        path = ROOT / target
        if not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("", encoding="utf-8")
            created += 1
    for folder in [ROOT / "src", *sorted(p for p in (ROOT / "src").rglob("*") if p.is_dir())]:
        if folder.name == "bin" or "data" in folder.relative_to(ROOT).parts:
            continue
        parent = folder / ("lib.rs" if folder == ROOT / "src" else "mod.rs")
        children = sorted(
            [p.stem for p in folder.glob("*.rs") if p.name not in ("mod.rs", "lib.rs", "main.rs")]
            + [p.name for p in folder.iterdir() if p.is_dir() and (p / "mod.rs").exists()])
        text = parent.read_text(encoding="utf-8") if parent.exists() else ""
        missing = [c for c in children if not re.search(rf"^\s*(pub )?mod {c};", text, re.M)]
        if missing:
            text = text.rstrip("\n") + ("\n\n" if text.strip() else "")
            text += "".join(f"pub mod {c};\n" for c in missing)
            parent.write_text(text, encoding="utf-8", newline="\n")
            created += len(missing)
    tests = ROOT / "tests"
    test_mods = sorted(p.stem for p in tests.glob("*.rs") if p.name != "mod.rs")
    mod_rs = tests / "mod.rs"
    text = mod_rs.read_text(encoding="utf-8") if mod_rs.exists() else ""
    missing = [m for m in test_mods if not re.search(rf"^\s*(pub )?mod {m};", text, re.M)]
    if missing:
        text = text.rstrip("\n") + ("\n" if text.strip() else "")
        text += "".join(f"mod {m};\n" for m in missing)
        mod_rs.write_text(text, encoding="utf-8", newline="\n")
    print(f"layout: {created} files or declarations added")
    return 0


def mirror() -> int:
    """Copy every mirrored file and data file from the Python checkout's committed HEAD."""
    copied = 0
    for python in tracked():
        target = rust_path(python)
        if target is None or not (python.startswith(MIRRORED) or "/data/" in python):
            continue
        data = subprocess.run(["git", "-C", str(PYTHON), "show", f"HEAD:{python}"],
                              check=True, capture_output=True).stdout
        path = ROOT / target
        if not path.exists() or path.read_bytes() != data:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            copied += 1
    print(f"mirror: {copied} files copied")
    return 0


def python_tests() -> dict[str, list[str]]:
    """Test file -> its test functions, parametrized ones counted once."""
    out: dict[str, list[str]] = {}
    for path in sorted((PYTHON / "tests").glob("test_*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        names = [node.name for node in ast.walk(tree)
                 if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")]
        out[path.stem] = names
    return out


def tests() -> int:
    """Every Python test function has a Rust `#[test] fn` of the same name in the same file.
    A test the port cannot have says why in `port.toml`'s `[untestable]`."""
    untestable = manifest().get("untestable", {})
    assert isinstance(untestable, dict)
    missing_total = ported_total = 0
    for stem, names in python_tests().items():
        path = ROOT / "tests" / f"{stem}.rs"
        text = path.read_text(encoding="utf-8") if path.exists() else ""
        have = set(re.findall(r"#\[test\][^\n]*\n(?:\s*#\[[^\n]*\n)*\s*fn (test_\w+)", text))
        missing = [n for n in names if n not in have and f"{stem}::{n}" not in untestable]
        ported_total += len(names) - len(missing)
        missing_total += len(missing)
        if missing:
            print(f"{stem}: {len(missing)} of {len(names)} missing: {', '.join(missing)}")
    print(f"tests: {ported_total} ported, {missing_total} missing, "
          f"{len(untestable)} untestable by port.toml")
    return 1 if missing_total else 0


def mark(commit: str) -> int:
    sha = git("rev-parse", commit).strip()
    text = MANIFEST.read_text(encoding="utf-8")
    text = re.sub(r'^upstream = "[0-9a-f]+"', f'upstream = "{sha}"', text, count=1, flags=re.M)
    MANIFEST.write_text(text, encoding="utf-8", newline="\n")
    print(f"port.toml now matches {sha[:9]}")
    return 0


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    command = sys.argv[1] if len(sys.argv) > 1 else "status"
    if command == "status":
        return status()
    if command == "layout":
        return layout()
    if command == "mirror":
        return mirror()
    if command == "tests":
        return tests()
    if command == "mark":
        return mark(sys.argv[2] if len(sys.argv) > 2 else "HEAD")
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
