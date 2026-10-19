#!/usr/bin/env bash
# Record each named replay as a tape through the Python checkout's spikes/tape_replay.py, verify it
# against its recording, make its references by replaying it through the Python runtime under the
# label `port`, then replay it through the Rust runtime against them (tools/tape/README.md).
# Stops if a recording fails, so a rig left mid-swap is not reused.
set -u
here="$(cd "$(dirname "$0")/../.." && pwd)"
python_repo="${PA3DS_PYTHON:-$here/../pokemon-access-3ds}"
tapes="${LOCALAPPDATA}/pa3ds/tapes"
references="${LOCALAPPDATA}/pa3ds/captures/regress/port"
if ! (cd "$python_repo" && uv run python spikes/tape_replay.py record "$@"); then
    echo "FAILED to record"; exit 1
fi
(cd "$python_repo" && uv run python spikes/tape_replay.py verify "$@")
(cd "$python_repo" && uv run python spikes/tape_replay.py replay port "$@")
"$here/target/release/tape.exe" --against "$references" $(for name in "$@"; do echo "$tapes/$name.tape.gz"; done)
