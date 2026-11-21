#!/usr/bin/env python3
"""Parse one `/evilcraft infuser` line into shell assignments.

The line the command prints (one line, in chat):

    Blood Infuser [206, 101, 0]: tank 10000/10000 mB, tier 3, cont evilcraft:blood_extractor@0, in minecraft:bone@1, out minecraft:blaze_rod@1, progress 0, recipes 10/10

Usage (as the chest's `parse_info.py`):

    eval "$(python3 tools/parse_infuser.py "$LINE")"

It prints `ok=1` and `TANK`, `CAPACITY`, `TIER`, `CONT_ITEM`, `CONT_BLOOD`,
`IN_ITEM`, `IN_COUNT`, `OUT_ITEM`, `OUT_COUNT`, `PROGRESS`, `RECIPES`,
`CARRIED`, `TIME`; on a line it cannot read it prints `ok=0` and nothing else, so the
caller fails with the line it had.
"""

import re
import shlex
import sys

LINE = re.compile(
    r"^Blood Infuser \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"tank (?P<tank>-?\d+)/(?P<capacity>-?\d+) mB, "
    r"tier (?P<tier>-?\d+), "
    r"cont (?P<citem>[^@\s,]+)@(?P<cblood>-?\d+), "
    r"in (?P<iitem>[^@\s,]+)@(?P<icount>-?\d+), "
    r"out (?P<oitem>[^@\s,]+)@(?P<ocount>-?\d+), "
    r"progress (?P<progress>-?\d+), "
    r"recipes (?P<recipes>-?\d+)/(?P<carried>-?\d+)"
    r"(?: t=(?P<time>-?\d+))?"
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
        "TIER": match["tier"],
        "CONT_ITEM": match["citem"],
        "CONT_BLOOD": match["cblood"],
        "IN_ITEM": match["iitem"],
        "IN_COUNT": match["icount"],
        "OUT_ITEM": match["oitem"],
        "OUT_COUNT": match["ocount"],
        "PROGRESS": match["progress"],
        "RECIPES": match["recipes"],
        "CARRIED": match["carried"],
        "TIME": match["time"] or "0",
    }
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
