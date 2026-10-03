#!/usr/bin/env bash
# Record each named replay as a tape, make its references by replaying it through the Python
# runtime, then replay it through the Rust runtime. Stops at the first recording that fails, so a
# rig left mid-swap is not reused (tools/tape/README.md).
set -u
here="$(cd "$(dirname "$0")/../.." && pwd)"
py="${PA3DS_PYTHON:-$here/../pokemon-access-3ds}/.venv/Scripts/python.exe"
tapes="${LOCALAPPDATA}/pa3ds/tapes"
mkdir -p "$tapes"
for name in "$@"; do
    if ! "$py" "$here/tools/tape/py.py" record "$name" "$tapes/$name.tape.gz" > "$tapes/$name.record.out" 2>&1; then
        echo "FAILED to record $name:"; tail -20 "$tapes/$name.record.out"; exit 1
    fi
    tail -1 "$tapes/$name.record.out"
    "$py" "$here/tools/tape/py.py" replay "$tapes/$name.tape.gz" "$tapes/$name.py.log" | tail -1
    diff -q --strip-trailing-cr "$tapes/$name.tape.gz.log" "$tapes/$name.py.log" > /dev/null         || echo "$name: the Python replay logged otherwise than its recording"
    diff -q --strip-trailing-cr "$tapes/$name.tape.gz.said" "$tapes/$name.py.said" > /dev/null         || echo "$name: the Python replay said otherwise than its recording"
done
"$here/target/release/tape.exe" $(for name in "$@"; do echo "$tapes/$name.tape.gz"; done)
