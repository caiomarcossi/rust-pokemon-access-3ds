# Building pa3ds

This is a port of [the Python pa3ds](https://github.com/pa3ds/pokemon-access-3ds) to Rust, so
that a player runs one `pa3ds.exe` with nothing to install. The Python repository is where
features are built first; this one follows it module for module (`port.toml` names the commit it
matches) and is checked against it line for line on recorded replays (`tools/tape/`). The
decisions, the evidence behind them and the vocabulary are the Python repository's, in its
`docs/` and `GLOSSARY.md`.

```
cargo build --release
```

Building needs Rust's MSVC toolchain (rustup's default on Windows), Visual Studio 2022 or newer
with its C and C++ compiler and ATL (the Build Tools are enough), and CMake 4.2 or newer on the
path, since Prism is compiled into pa3ds; the first build takes about a minute longer for it.
Visual Studio's own CMake is 3.31, which is too old.
`target\release\pa3ds.exe` is the whole program. To pack it with its data, its manual and its
licences into one zip a player unpacks and runs:

```
powershell -ExecutionPolicy Bypass -File tools\package.ps1
```

(or `tools\package.bat`, which runs the same), which runs the tests, builds the release, checks it
starts, and writes `target\package\pa3ds-<version>-windows.zip`. Only a pa3ds built this way
checks for updates; `cargo build` makes one that does not.

A release is cut with `/release` in Claude Code (`.claude/skills/release/SKILL.md`): it writes what
changed for players into `CHANGELOG.md`, sets the version, and tags it. Pushing a `v<version>` tag
builds the zip on GitHub and publishes it as that version's release
(`.github/workflows/release.yml`).
