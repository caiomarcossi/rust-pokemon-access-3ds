"""A tape: every answer the emulator and the clock gave the Python runtime during one replay.

The Rust port is checked against the Python runtime by feeding it the same answers. Record a
movie's replay through the Python runtime once, then replay the tape through either runtime with
no emulator: a faithful port asks the same questions in the same order, so it gets the same
answers and logs the same lines. Where it asks something else, the replay stops and says where.

What is recorded is the runtime's side of `pa3ds.core.rpc.Rpc` (`read`, `read_each`, `confirm`,
`attach`, `unsure`, `reply_seconds`) and every `time.monotonic` and `time.sleep` called from
pa3ds's own modules. `read_many` is `read_each` underneath and is recorded as one.

The format is in `tools/tape/README.md`; `src/tape.rs` reads the same bytes.
"""

from __future__ import annotations

import gzip
import struct
import sys
import time
from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Any, BinaryIO

MAGIC = b"PA3DSTAPE1\n"

CLOCK, SLEEP, READ, READ_EACH, UNSURE, REPLY_SECONDS, CONFIRM, ATTACH = range(1, 9)
NAMES = {CLOCK: "clock", SLEEP: "sleep", READ: "read", READ_EACH: "read_each",
         UNSURE: "unsure", REPLY_SECONDS: "reply_seconds", CONFIRM: "confirm", ATTACH: "attach"}
OK, RPC_ERROR, TIMEOUT, REFUSED, LOST, VALUE_ERROR = range(6)


class TapeEnd(BaseException):  # noqa: N818 -- not an error: the replay has run out of tape
    """The tape has no more answers. A BaseException, so no reader's `except Exception` takes
    it for a reader failing."""


class Diverged(BaseException):  # noqa: N818
    """The runtime asked something the recorded run did not ask at this point."""


BELOW_THE_TAPE = "pa3ds.core.rpc"
"""The client's own clock and back-off sleeps are inside the calls the tape answers whole."""


def _pa3ds_caller() -> bool:
    frame = sys._getframe(2)  # noqa: SLF001 -- the caller of the patched function
    module = str(frame.f_globals.get("__name__", ""))
    return module.startswith("pa3ds") and module != BELOW_THE_TAPE


class Writer:
    def __init__(self, path: Path) -> None:
        self._out: BinaryIO = gzip.open(path, "wb", compresslevel=6)  # noqa: SIM115
        self._out.write(MAGIC)
        self.count = 0

    def close(self) -> None:
        self._out.close()

    def u8(self, v: int) -> None:
        self._out.write(struct.pack("<B", v))

    def u32(self, v: int) -> None:
        self._out.write(struct.pack("<I", v & 0xFFFFFFFF))

    def u64(self, v: int) -> None:
        self._out.write(struct.pack("<Q", v))

    def f64(self, v: float) -> None:
        self._out.write(struct.pack("<d", v))

    def blob(self, v: bytes) -> None:
        self.u32(len(v))
        self._out.write(v)

    def text(self, v: str) -> None:
        self.blob(v.encode("utf-8"))

    def tag(self, kind: int) -> None:
        self.count += 1
        self.u8(kind)

    def error(self, error: BaseException | None) -> None:
        from pa3ds.core.rpc import RpcError, RpcLostError, RpcRefusedError, RpcTimeoutError

        if error is None:
            self.u8(OK)
            return
        kind = (LOST if isinstance(error, RpcLostError)
                else REFUSED if isinstance(error, RpcRefusedError)
                else TIMEOUT if isinstance(error, RpcTimeoutError)
                else RPC_ERROR if isinstance(error, RpcError)
                else VALUE_ERROR if isinstance(error, ValueError)
                else None)
        if kind is None:
            raise TypeError(f"no tape encoding for {error!r}")
        self.u8(kind)
        self.text(str(error))


class Reader:
    def __init__(self, path: Path) -> None:
        self._data = gzip.decompress(path.read_bytes())
        if not self._data.startswith(MAGIC):
            raise ValueError(f"{path} is not a tape")
        self._at = len(MAGIC)
        self.count = 0

    def _take(self, n: int) -> bytes:
        if self._at + n > len(self._data):
            raise TapeEnd
        chunk = self._data[self._at:self._at + n]
        self._at += n
        return chunk

    def u8(self) -> int:
        return self._take(1)[0]

    def u32(self) -> int:
        return struct.unpack("<I", self._take(4))[0]

    def u64(self) -> int:
        return struct.unpack("<Q", self._take(8))[0]

    def f64(self) -> float:
        return struct.unpack("<d", self._take(8))[0]

    def blob(self) -> bytes:
        return self._take(self.u32())

    def text(self) -> str:
        return self.blob().decode("utf-8")

    def expect(self, kind: int, what: str) -> None:
        if self._at >= len(self._data):
            raise TapeEnd
        got = self.u8()
        self.count += 1
        if got != kind:
            raise Diverged(f"record {self.count}: the runtime asked {what}, the tape has "
                           f"{NAMES.get(got, got)}")

    def error(self) -> BaseException | None:
        from pa3ds.core.rpc import RpcError, RpcLostError, RpcRefusedError, RpcTimeoutError

        kind = self.u8()
        if kind == OK:
            return None
        message = self.text()
        return {RPC_ERROR: RpcError, TIMEOUT: RpcTimeoutError, REFUSED: RpcRefusedError,
                LOST: RpcLostError, VALUE_ERROR: ValueError}[kind](message)


def patch_clock(on_monotonic: Callable[[Callable[[], float]], float],
                on_sleep: Callable[[Callable[[float], None], float], None]) -> None:
    """Route pa3ds's own `time.monotonic` and `time.sleep` through the tape. Must run before
    pa3ds is imported: `navigate`, `apps` and `field` take `time.monotonic` as a default argument
    when their modules load."""
    real_monotonic, real_sleep = time.monotonic, time.sleep

    def monotonic() -> float:
        if _pa3ds_caller():
            return on_monotonic(real_monotonic)
        return real_monotonic()

    def sleep(seconds: float) -> None:
        if _pa3ds_caller():
            on_sleep(real_sleep, seconds)
        else:
            real_sleep(seconds)

    time.monotonic = monotonic
    time.sleep = sleep


class Depth:
    """Records only the outermost call: `read` asks `unsure` and `confirm` on its own."""

    def __init__(self) -> None:
        self.level = 0

    def __enter__(self) -> bool:
        self.level += 1
        return self.level == 1

    def __exit__(self, *_: object) -> None:
        self.level -= 1


def recording_rpc(tape: Writer) -> Any:
    from pa3ds.core.rpc import Process, Rpc

    depth = Depth()

    class RecordingRpc(Rpc):
        @property
        def reply_seconds(self) -> float:  # type: ignore[override]
            value = self._reply_seconds
            if depth.level == 0:
                tape.tag(REPLY_SECONDS)
                tape.f64(value)
            return value

        @reply_seconds.setter
        def reply_seconds(self, value: float) -> None:
            self._reply_seconds = value

        @property
        def unsure(self) -> bool:  # type: ignore[override]
            value = Rpc.unsure.fget(self)  # type: ignore[attr-defined]
            if depth.level == 0:
                tape.tag(UNSURE)
                tape.u8(int(value))
            return value

        def read(self, address: int, size: int) -> bytes:
            with depth as outer:
                try:
                    data = super().read(address, size)
                except BaseException as error:
                    if outer:
                        tape.tag(READ)
                        tape.u32(address)
                        tape.u32(size)
                        tape.error(error)
                    raise
                if outer:
                    tape.tag(READ)
                    tape.u32(address)
                    tape.u32(size)
                    tape.error(None)
                    tape.blob(data)
                return data

        def read_each(self, reads: Iterable[tuple[int, int]]) -> dict[int, bytes]:
            reads = list(reads)
            with depth as outer:
                def head() -> None:
                    tape.tag(READ_EACH)
                    tape.u32(len(reads))
                    for address, size in reads:
                        tape.u32(address)
                        tape.u32(size)

                try:
                    out = super().read_each(reads)
                except BaseException as error:
                    if outer:
                        head()
                        tape.error(error)
                    raise
                if outer:
                    head()
                    tape.error(None)
                    tape.u32(len(out))
                    for address, data in out.items():
                        tape.u32(address)
                        tape.blob(data)
                return out

        def confirm(self) -> bool:
            with depth as outer:
                try:
                    value = super().confirm()
                except BaseException as error:
                    if outer:
                        tape.tag(CONFIRM)
                        tape.error(error)
                    raise
                if outer:
                    tape.tag(CONFIRM)
                    tape.error(None)
                    tape.u8(int(value))
                return value

        def attach(self) -> Process:
            with depth as outer:
                try:
                    process = super().attach()
                except BaseException as error:
                    if outer:
                        tape.tag(ATTACH)
                        tape.error(error)
                    raise
                if outer:
                    tape.tag(ATTACH)
                    tape.error(None)
                    tape.u32(process.pid)
                    tape.u64(process.title_id)
                    tape.text(process.name)
                return process

    return RecordingRpc


def replaying_rpc(tape: Reader) -> Any:
    """An `Rpc` that answers from the tape and checks each question against it."""
    from pa3ds.core.rpc import Process

    class ReplayingRpc:
        def __init__(self, *_: object, **__: object) -> None:
            pass

        def __enter__(self) -> ReplayingRpc:
            return self

        def __exit__(self, *_: object) -> None:
            pass

        def close(self) -> None:
            pass

        @property
        def reply_seconds(self) -> float:
            tape.expect(REPLY_SECONDS, "reply_seconds")
            return tape.f64()

        @property
        def unsure(self) -> bool:
            tape.expect(UNSURE, "unsure")
            return bool(tape.u8())

        def read(self, address: int, size: int) -> bytes:
            tape.expect(READ, f"read {address:#x} {size}")
            got = (tape.u32(), tape.u32())
            if got != (address, size):
                raise Diverged(f"record {tape.count}: read {address:#x} {size}, the tape has "
                               f"read {got[0]:#x} {got[1]}")
            if (error := tape.error()) is not None:
                raise error
            return tape.blob()

        def read_many(self, addresses: Iterable[int], size: int = 1024) -> dict[int, bytes]:
            return self.read_each((address, size) for address in addresses)

        def read_each(self, reads: Iterable[tuple[int, int]]) -> dict[int, bytes]:
            reads = list(reads)
            tape.expect(READ_EACH, f"read_each of {len(reads)}")
            recorded = [(tape.u32(), tape.u32()) for _ in range(tape.u32())]
            if recorded != reads:
                first = next((i for i, (a, b) in enumerate(zip(recorded, reads, strict=False))
                              if a != b), min(len(recorded), len(reads)))
                raise Diverged(f"record {tape.count}: read_each of {len(reads)}, the tape has "
                               f"{len(recorded)}, first differing at {first}")
            if (error := tape.error()) is not None:
                raise error
            return {tape.u32(): tape.blob() for _ in range(tape.u32())}

        def confirm(self) -> bool:
            tape.expect(CONFIRM, "confirm")
            if (error := tape.error()) is not None:
                raise error
            return bool(tape.u8())

        def attach(self) -> Process:
            tape.expect(ATTACH, "attach")
            if (error := tape.error()) is not None:
                raise error
            return Process(pid=tape.u32(), title_id=tape.u64(), name=tape.text())

    return ReplayingRpc
