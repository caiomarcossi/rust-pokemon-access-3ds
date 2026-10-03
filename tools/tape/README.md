# Tapes: the port checked against the Python runtime

A **tape** is everything the emulator and the clock answered the Python runtime during one replay
of a movie, in the order it asked: each `Rpc` call with its reply, and each `time.monotonic` and
`time.sleep` made from pa3ds's own modules. Replaying a tape needs no emulator and no ROM rig,
and a faithful port asks exactly the questions the Python asked, so it gets exactly the same
answers and logs and says exactly the same lines. Where it asks something else, the replay stops
at that record and says what each side asked.

Two live runs of the same commit differ by tens of lines (`docs/runbooks/rig.md`), which is why a
live comparison cannot prove a port line for line and a tape can.

Tapes hold raw game RAM, which holds the player's save data (ADR-0029). They live under
`%LOCALAPPDATA%\pa3ds\tapes`, never in the repository.

## Recording

From this repository, with the Python checkout's interpreter (it has pa3ds and the rig):

```
..\pokemon-access-3ds\.venv\Scripts\python.exe tools\tape\py.py record <replay> %LOCALAPPDATA%\pa3ds\tapes\<replay>.tape.gz
```

`<replay>` is a `[[replay]]` in the Python checkout's `spikes/rig.toml`, played as
`spikes/regress.py run` plays it. Beside the tape go `<tape>.log` (what the runtime logged,
without times) and `<tape>.said` (what it said, `!` before a line that interrupts).

A tape is checked by replaying it through the Python runtime itself, which must reproduce both
files exactly:

```
..\pokemon-access-3ds\.venv\Scripts\python.exe tools\tape\py.py replay <tape> <out.log>
```

## Replaying through the Rust runtime

```
cargo run --release --bin tape -- <tape>
```

writes `<tape>.rust.log` and `<tape>.rust.said` and compares them with the recording's.

## What is compared, and what is not

Every log line and every spoken line, in order. Two kinds of line are compared by their first
line only: a reader's failure, whose traceback names Python files, and the snapshot path written
for it.

`read_each` batches are compared as sets: the Python sends some batches in the order of a `set`,
which no other language reproduces, and a batch's order changes only the order its replies come
back in.

## Format

gzip of: the magic `PA3DSTAPE1\n`, then records, each a one-byte kind and its fields,
little-endian. A string is a `u32` byte length and UTF-8; a blob a `u32` length and bytes. An
outcome is a `u8`: 0 for a reply, which follows, or the class of the exception raised, followed by
its message: 1 `RpcError`, 2 `RpcTimeoutError`, 3 `RpcRefusedError`, 4 `RpcLostError`,
5 `ValueError`.

| kind | record | fields |
| --- | --- | --- |
| 1 | `time.monotonic()` | `f64` the time |
| 2 | `time.sleep(s)` | `f64` s |
| 3 | `read(address, size)` | `u32` address, `u32` size, outcome, blob |
| 4 | `read_each(reads)` | `u32` n, n × (`u32` address, `u32` size), outcome, `u32` m, m × (`u32` address, blob) |
| 5 | `unsure` | `u8` |
| 6 | `reply_seconds` | `f64` |
| 7 | `confirm()` | outcome, `u8` |
| 8 | `attach()` | outcome, `u32` pid, `u64` title id, string name |

`read_many` is `read_each` underneath and is recorded as one. Calls the client makes of itself,
as `read` asking `unsure` and `confirm` before it reads, are inside the outer call and not
recorded. `src/tape.rs` reads the same format.
