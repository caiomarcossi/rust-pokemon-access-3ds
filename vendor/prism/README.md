# Prism

`prism.dll` is Prism, the screen reader and speech library pa3ds speaks through
(https://github.com/ethindp/prism), unmodified, as prismatoid 0.18.2 ships it. It is loaded at
run time from beside `pa3ds.exe` (`src/core/speech.rs`), and `build.rs` copies it there.

Prism is under the Mozilla Public License 2.0 (`LICENSE`, `LICENSES/prism`); the libraries built
into it are under the licences in `LICENSES/`, as `NOTICE` lists. pa3ds links none of it: it is a
separate file, shipped as it came.

Copied prebuilt rather than built from source, because the MSVC in Visual Studio 2022 cannot
compile Prism's C23 sources. To update it, replace `prism.dll` and the licence files with a newer prismatoid
wheel's `prism/_native/prism.dll` and `licenses/`.
