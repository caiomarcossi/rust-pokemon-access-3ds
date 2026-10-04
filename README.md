# pa3ds, in Rust

A screen reader accessibility layer for Nintendo 3DS Pokemon games running under the
[Azahar](https://azahar-emu.org/) emulator: Omega Ruby and Alpha Sapphire, X and Y, and Sun and
Moon.

It reads the game's actual memory over Azahar's RPC interface and speaks through
[Prism](https://github.com/ethindp/prism) to NVDA (or SAPI5). No OCR, no screenshots, and no
hand-written game text: every word spoken comes out of the ROM or the game's RAM.

This is a port of [the Python pa3ds](https://github.com/KenzonYeoh/pokemon-access-3ds) to Rust, so
that a player runs one `pa3ds.exe` with nothing to install. The Python repository is where
features are built first; this one follows it module for module (`port.toml` names the commit it
matches) and is checked against it line for line on recorded replays (`tools/tape/`).

**Status: pre-alpha.** No release yet. The decisions are in [docs/adr/](docs/adr/README.md), the
evidence behind them in [docs/findings/](docs/findings/README.md), and the vocabulary in
[CONTEXT.md](CONTEXT.md).

## Requirements

- Windows
- Azahar with its RPC server reachable on UDP 45987
- Your own legally obtained, decrypted Pokemon ROM
- NVDA, or any Prism-supported screen reader, or SAPI5

ROMs are not distributed here and never will be.

## Building

```
cargo build --release
```

`target\release\pa3ds.exe` with `prism.dll` beside it is the whole program. To pack it with its
licences into one zip a player unpacks and runs:

```
powershell -ExecutionPolicy Bypass -File tools\package.ps1
```

which runs the tests, builds the release, checks it starts, and writes
`target\package\pa3ds-<version>-windows.zip`.
