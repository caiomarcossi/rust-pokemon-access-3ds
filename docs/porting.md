# Porting a Python module to Rust

This repository is a port of the Python pa3ds (`port.toml` names the upstream commit). A ported
module must do what the Python does, step for step: the same reads of the emulator in the same
order, the same clock reads, the same log lines and the same speech. That is checked, not hoped
for: a **tape** records everything the emulator and the clock answered the Python runtime during
a movie's replay, and the Rust runtime replayed on it must ask the same questions and log and say
the same lines (`tools/tape/README.md`). A port that is "the same idea" fails the tape.

So port **faithfully, not idiomatically**: keep the Python's structure, names, order of operations
and its bugs. A bug noticed while porting is ported as it is and reported, never fixed here: the
fix belongs upstream first.

The reference ports show every rule below in use: `src/core/rpc.rs`, `src/core/memory.rs`,
`src/core/speech.rs`, `src/core/binary.rs`, and their tests in `tests/`.

## Where things go

- `src/pa3ds/<path>.py` is `src/<path>.rs`; a package's `__init__.py` is its `mod.rs`;
  `tests/<name>.py` is `tests/<name>.rs`. `python tools/sync.py layout` made every file and every
  `mod` declaration already. Do not add, remove or rename modules.
- Fill in only the files you were given. If a module you depend on needs a small change (an item
  made `pub`, a missing accessor), make the smallest change and list it in your report.
- `src/py.rs` holds what Python does differently from Rust (below). Do not edit it; ask for what
  it lacks in your report, and write a private helper meanwhile.
- A data file (`games/*/data/**`) is read with `include_str!` from where `tools/sync.py mirror`
  copied it, so the binary carries it.

## Names and documentation

- Same names: functions, methods, constants, classes (as structs, traits or enums) and fields keep
  their Python names. `_private` loses its underscore and is private in Rust, or `pub(crate)` if a
  test reaches it.
- Docstrings become `///` or `//!` doc comments, and comments `//` comments, word for word: they
  are the reasons the code is the way it is. Wrap at 100 columns. Add no comment of your own except
  where the Rust differs from the Python in a way the next porter would trip on, and then say why.
- A Python name that is a Rust keyword takes `r#`: `r#type`, `r#match`.

## Types

| Python | Rust |
| --- | --- |
| `int` | `i64`, always, addresses included. A byte is still `i64` once it is a number. |
| `float` | `f64` |
| `str` | `String`, `&str` to borrow |
| `bytes` | `Vec<u8>`, `&[u8]` to borrow |
| `list`, `Sequence` | `Vec<T>`, `&[T]` |
| `tuple` | a tuple, or a struct where the Python named its parts |
| `dict` | `IndexMap` (Python iterates a dict in insertion order) |
| `set`, `frozenset` | `py::PySet` if iterated and of ints, `HashSet` if only tested for membership |
| `X \| None` | `Option<X>` |
| `@dataclass` | a struct with `pub` fields, `#[derive(Debug, Clone, PartialEq)]` |
| `Protocol` | a trait |
| `Enum`, `IntEnum`, `StrEnum` | an enum, with `value()` where the value is used |
| `Callable[..., T]` | `Rc<dyn Fn(...) -> T>`; `py::Log` for `log` |
| `Final` constant | `pub const`, or a `fn` returning it where a const cannot hold it |

Keyword arguments with defaults: a constructor taking every argument, plus a short one for the
common call, or a `Default` impl and `..Default::default()`. Follow what the tests need.

## Objects that are shared

Python hands one object to many holders: the RPC client, a memory, the speech, a cache. In Rust
a holder keeps an `Rc<dyn Trait>` (or `Rc<Struct>`), and the methods take `&self`, with the
mutable state inside in `Cell` or `RefCell`. Never keep a `RefCell` borrowed across a call that
might come back into the same object.

- The emulator is `&dyn Rpc` to a function and `Rc<dyn Rpc>` to a holder (`core::rpc`). The UDP
  client is `UdpRpc`; tests use the fake emulator (`tests/fake_emulator.rs`).
- Memory is `&dyn Memory` / `Rc<dyn Memory>` (`core::memory`). Tests use `FakeMemory`, written
  through `&self` so a test can change it while a reader holds it.
- `with cache.phase("name"): ...` is `memory::phase(&cache, "name", || { ...; Ok(()) })?`.
- A Python object one owner mutates stays a plain struct with `&mut self` methods.
- A thread gets `Arc` and `Mutex`, only where the Python has a thread.

## Exceptions

`py::Error` is a raised exception: its class (`py::Kind`), its message, and the line that raised
it. `py::Result<T>` is `Result<T, py::Error>`.

- A function that can raise returns `Result`; one that cannot does not.
- `raise RomError("x")` is `return Err(Error::new(Kind::RomError, "x"))`. Keep the message word
  for word: it is logged.
- `except (A, B):` is a `match` on `error.is(Kind::A) || error.is(Kind::B)` (`is` is
  `isinstance`, bases included); anything else is returned on with `?` or `return Err(e)`.
- What Python raises without a `raise`:
  - `struct.unpack_from` on too few bytes is `struct.error`: `core::binary`'s functions return
    `Result` and raise it, so use them, with `?`.
  - Where the Python catches `IndexError` or `KeyError`, produce that error at the same spot
    (`Error::index`, `Error::key`) with `.get()`. Where it does not catch it, returning the error
    is still better than a panic, but a panic is caught by the runtime as Python's was.
  - Never turn a raise into a default value: a reader that failed must fail.
- The runtime keys a fault by where it was raised (`Error::fault`): construct errors where the
  Python raised them, not in a shared helper (`#[track_caller]` on a helper that raises for its
  caller).

## Arithmetic

Rust's operators differ from Python's on negatives and halves. Use `py`:

| Python | Rust |
| --- | --- |
| `a // b` on ints | `py::floordiv(a, b)` |
| `a % b` on ints | `py::modulo(a, b)` |
| `a // b`, `a % b` on floats | `py::floordiv_f`, `py::modulo_f` |
| `round(x)` | `py::round(x)?`: halves to even |
| `round(x, n)` | `py::round_to(x, n)` |
| `int(x)` of a float | `py::int(x)?` |
| `math.floor(x)`, `math.ceil(x)` | `py::floor(x)?`, `py::ceil(x)?` |
| `a / b` on ints | `a as f64 / b as f64` |

`round`, `int`, `floor` and `ceil` return `Result` because Python raises on a NaN (`ValueError`)
and an infinity (`OverflowError`), and a float read out of freed memory is either; `as i64` would
quietly make it a number. Never cast a float to an int with `as` where Python called one of
these.
| `~x` | `!x` |

`a // b` and `a % b` with both sides known non-negative may use `/` and `%`, but `py::` is never
wrong. A division Python could make by zero raises `ZeroDivisionError`; make it do so in Rust only
where it can happen.

## Text

- `len(text)` counts characters: `text.chars().count()`. Indexing or slicing a `str` is by
  character: collect a `Vec<char>` first.
- `text.split()` is `py::split`; `strip`, `lstrip`, `rstrip` are `py::strip` and the rest
  (Python's whitespace is wider than Rust's). `split(sep)` with a separator is Rust's `split`.
- f-strings: an `int` formats as itself. A `float` is **never** `{x}`: Python writes `1.0` where
  Rust writes `1`, so use `py::float_repr(x)`. `{x:.1f}` is `{x:.1}`; `{x:#010x}` and `{x:x}` are
  the same in Rust (on `i64`, mind a negative number: Python writes `-0x10`, Rust two's complement;
  where it can be negative, format the sign yourself).
- `{x!r}` and `repr(text)` are `py::repr(text)`. `str(list)` writes each element's repr:
  `['a', 'b']`, `[(0, 0), (40, 0)]`. Write what Python writes.
- `bytes.decode(...)` is `py::decode_utf16le`, `py::decode_ascii`, `py::decode_latin1`.
- `re` is the `regex` crate. It has no look-around or back-references: if a pattern needs them,
  say so in the report.

## Slices and order

- `data[a:b]` clamps where Rust panics: `py::slice(data, a, b)` and `py::slice_from`. Index
  directly only where the Python would raise.
- `sorted` and `.sort()` are stable; so are `sort_by` and `sort_by_key`. Never `sort_unstable`.
- `max(..., key=f)` returns the **first** of equal bests, and Rust's `max_by_key` the **last**:
  use `.rev().max_by_key(f)` or a loop where ties can happen. `min` and `min_by_key` both return
  the first. A float key needs `max_by`/`min_by` with `total_cmp`.
- Python iterates a `set` in hash order, not insertion order, and that order does reach what
  pa3ds reads: an insertion-ordered set made `layout.track` read its works in another order. A
  set of **ints** that is iterated is `py::PySet`, which lays the set out as CPython does
  (`PySet::of` for `set(iterable)` and `{a, b}`, `update_dict` for `set.update(a_dict)` and
  `{*a_dict}`). If you need an operation it lacks (`-`, `&`, `discard`), write it from CPython's
  `Objects/setobject.c` beside the others, check it against CPython, and report it. A set of
  strings Python iterates in an order that changes every run, so pa3ds cannot depend on it:
  `IndexSet`, and list the spot in your report.

## Time, files, threads

- `time.monotonic()` is `py::time::monotonic()` and `time.sleep(s)` is `py::time::sleep(s)`, so
  a tape can answer them. Never `std::time` or `std::thread::sleep` directly; the RPC client is
  the one exception, being below the tape. A `clock` parameter stays a parameter,
  `Rc<dyn Fn() -> f64>`, defaulting to `py::time::monotonic`.
- `pathlib.Path` is `PathBuf`; an `OSError` from the file system comes from `?` on `io::Error`.
- TOML is read with the `toml` crate (insertion order kept). `pa3ds.core.toml` writes it.
- Windows calls are `windows-sys`. A `ctypes` structure is a `#[repr(C)]` struct.

## Tests

- Port every test in the test files you were given, each as a `#[test] fn` of the **same name**:
  `python tools/sync.py tests` checks the names.
- A parametrized test is one Rust test looping over its cases, naming the case in its assertion
  message. A fixture is a helper function.
- `pytest.raises(X, match="m")` is `let error = ....unwrap_err(); assert!(error.is(Kind::X) &&
  error.message.contains("m"))` (or a regex where `match` used one).
- Assert against the same literal values the Python asserts. A test that would still pass with
  every function returning a default proves nothing.
- A test that cannot be ported at all (it tests Python itself) goes in `port.toml`'s
  `[untestable]` with the reason, and in your report. This should be rare.

## Before you report

1. `cargo build` and `cargo test --lib <your test file names>` pass.
2. `cargo clippy --all-targets` shows no warning in your files.
3. `cargo fmt` has been run.
4. `python tools/sync.py tests` shows none of your test names missing.
5. Commit on your branch: `feat(port): <modules>`, ending with the attribution line you were
   given.

Report: each module ported (Python lines to Rust lines), tests ported and passing, every place the
Rust behaves differently from the Python (the target is none), every set whose order you guessed,
every edit outside your files, any crate added, and any upstream bug you ported as it is.
