#!/usr/bin/env python3
"""Parse one `/evilcraft info` line into shell assignments.

The line the command prints (one line, in chat):

    Blood Chest [202, 101, 0]: tank 1000/10000 mB, container evilcraft:blood_extractor@250, slots [0:minecraft:diamond_pickaxe@234 1:minecraft:chipped_anvil@0], t=1234

Usage (as every other port's `parse_info.py`):

    eval "$(python3 tools/parse_info.py "$LINE")"

It prints `ok=1` and `TANK`, `CAPACITY`, `CONTAINER_ITEM`, `CONTAINER_BLOOD`,
`SLOT<n>_ITEM`, `SLOT<n>_DAMAGE`; on a line it cannot read it prints `ok=0`
and nothing else, so the caller fails with the line it had.
"""

import re
import shlex
import sys

LINE = re.compile(
    r"^Blood Chest \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"tank (?P<tank>-?\d+)/(?P<capacity>-?\d+) mB, "
    r'container (?P<item>[^@\s,]+)@(?P<blood>-?\d+), ' 
    r"slots \[(?P<slots>[^\]]*)\](?:, t=(?P<time>-?\d+))?"
)
SLOT = re.compile(r"(?P<index>\d+):(?P<item>[^@\s]+)@(?P<damage>-?\d+)")


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
        "CONTAINER_ITEM": match["item"],
        "CONTAINER_BLOOD": match["blood"],
        "TIME": match["time"] or "0",
    }
    for slot in SLOT.finditer(match["slots"]):
        index = slot["index"]
        fields[f"SLOT{index}_ITEM"] = slot["item"]
        fields[f"SLOT{index}_DAMAGE"] = slot["damage"]
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
