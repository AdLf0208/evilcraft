#!/usr/bin/env python3
"""Parse one `/evilcraft stain` line into shell assignments.

The line the command prints (one line, in chat):

    Blood Stain [206, 101, -10]: tank 2500/5000 mB, t=8412

Usage (as the infuser's `parse_infuser.py`):

    eval "$(python3 tools/parse_stain.py "$LINE")"

It prints `ok=1` and `TANK`, `CAPACITY`, `TIME`; on a line it cannot read it prints
`ok=0` and nothing else, so the caller fails with the line it had.
"""

import re
import shlex
import sys

LINE = re.compile(
    r"^Blood Stain \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"tank (?P<tank>-?\d+)/(?P<capacity>-?\d+) mB"
    r"(?:, t=(?P<time>-?\d+))?"
)


def main(argv):
    if len(argv) != 2:
        print("ok=0")
        return 1
    match = LINE.match(argv[1].strip())
    if not match:
        print("ok=0")
        return 1
    fields = {
        "ok": "1",
        "X": match["x"],
        "Y": match["y"],
        "Z": match["z"],
        "TANK": match["tank"],
        "CAPACITY": match["capacity"],
        "TIME": match["time"] or "0",
    }
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
