"""Refuse a Bash heredoc with a backslash in it: on Windows the Bash tool eats the backslashes,
even in a quoted heredoc, and the script that runs is not the one written."""

import json
import sys

command = json.load(sys.stdin).get("tool_input", {}).get("command", "")
if "<<" in command and "\\" in command:
    print(
        "A heredoc with a backslash loses it on this machine. Write the script to the scratchpad "
        "with the Write tool and run the file instead.",
        file=sys.stderr,
    )
    sys.exit(2)
