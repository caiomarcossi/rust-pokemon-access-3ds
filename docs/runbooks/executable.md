# The executable

Every code address in the findings (`0x003a6f10` and the rest) came from a dump of the executable at
`%LOCALAPPDATA%\pa3ds\captures\code.bin`: 8 MiB from `0x00100000`, taken over the RPC and
disassembled with capstone (`spikes/disassemble.py`). The 3DS maps the executable at a fixed base
with no ASLR, so those addresses hold across runs and the dump can be re-read without booting
anything. The code region measures `0x00100000`-`0x00728000` (spike 9); retake the dump if the ROM
ever changes.

The dump is the user's ROM. It stays out of the repository.
