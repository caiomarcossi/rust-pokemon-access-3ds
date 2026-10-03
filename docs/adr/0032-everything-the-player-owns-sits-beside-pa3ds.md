# 32. Everything the player owns sits beside pa3ds, and nothing is installed

- **Status:** Accepted
- **Date:** 2026-09-22
- **Deciders:** Orion
- **Built.** `keyfile.home_path` is the one place that decides, and the keys file, the names and
  the log all come from it.

## Context

The keys file, the player's names and the log were under `%APPDATA%\pa3ds`, which separates a
player's things from the shipped ones neatly and is what a Windows application is expected to do.
Against that: Pokemon Access and the Black/White layer both ship a portable emulator with
everything beside it, and a folder you can copy to another machine whole — names, keys and all —
is worth more here than tidiness. A blind player moving to a new machine should not have to know
what `%APPDATA%` is.

## Decision

**Beside pa3ds itself: `<pa3ds>/user/`.** Never `%APPDATA%`, and **never Program Files.**

The user, 2026-09-22, making this binding: *"no installers, or at least not a conventional kind.
I'm fine with a setup wizard of sorts to put pa3ds and Azahar wherever they should be (portable,
next to each other, probably) but we'll just not write to program files."*

So there is no per-user application directory and no elevation, ever. A setup wizard, if one is
written, places pa3ds and its emulator side by side somewhere the player chose and can write to.
Whatever else it does, **it does not install**.

`<pa3ds>` is the folder holding the executable once there is one, and the checkout otherwise, so a
run from source behaves the way a shipped one will. `PA3DS_HOME` moves it: that is what a test
sets, and it is what keeps an agent's session off the files the person at this machine is really
playing with — before it existed, running `python -m pa3ds` from a checkout rotated their live
logs and rewrote their real keys.

## Consequences

- **A checkout's `user/` is the player's, not the repository's**, and is gitignored.
- **Nothing migrates out of `%APPDATA%\pa3ds`.** This is all unreleased work on a branch, so the
  only files in the old place were the ones written while building it (the user, 2026-09-22). The
  code that read them was taken out the same day.
- Replacing the pa3ds folder with a newer one would take the player's names with it unless they
  copy `user/` across. That is the cost of this choice, and a wizard that updates in place rather
  than replacing is the answer to it.
- Nothing here decides **how** pa3ds is shipped, which is deliberately open (ADR-0033).
- **What pa3ds works out from the player's ROM is cached here too**, in `user/cache/<kind>/<rom>.toml`
  (Orion, 2026-09-29), TOML as ADR-0031 has everything: the first is the index of the ROM's
  textures by their images (`games.gen6.textures`), built once per ROM in a process of its own
  and read in a fraction of a second after. A cache holds for every language of its ROM, so it
  is not under a language's folder as the names are, and deleting it only has it built again.
