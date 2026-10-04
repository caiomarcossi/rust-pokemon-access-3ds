# Prism's licences

pa3ds speaks through Prism (https://github.com/ethindp/prism), the screen reader and speech
library, built into `pa3ds.exe` by the `prismer` crate's static build (`prism-sys` compiles Prism's
own source with CMake and the Visual Studio 2026 compiler). Nothing of Prism is kept here but its
licences.

Prism is under the Mozilla Public License 2.0 (`LICENSE`, `LICENSES/prism`), and the libraries
built into it under the licences in `LICENSES/`, as `NOTICE` lists. Since Prism's code is inside
the executable, these go with every copy of pa3ds: `tools/package.ps1` puts them in the release.
They are the licences prismatoid 0.18.2 ships with the same Prism; when `prismer` moves to a newer
Prism, take that version's.
