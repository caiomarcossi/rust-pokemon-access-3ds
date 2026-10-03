# The GDB stub

`src/pa3ds/debug/gdb.py` sets watchpoints, which answer "who wrote these bytes" instead of "what changed
since last time". It is a reverse-engineering instrument; nothing a player runs depends on it.

Enable it with `use_gdbstub=true` in `qt-config.ini`, then restart Azahar.

Its rules are the opposite of capture's ([snapshots.md](snapshots.md)):

- **The stub needs emulation running.** It is served from the emulator's CPU loop, so a paused
  emulator answers nothing and will not even accept the connection. Hold a capture or a stub
  session, never both.
- **Resume before detaching.** Azahar's detach handler forgets the process before it continues it,
  so a bare detach leaves the game frozen with no error.
- **Screenshots are black while the game is halted.** Label a hit by reading memory through the
  stub, and take any picture after resuming.
