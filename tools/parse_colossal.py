#!/usr/bin/env python3
"""Parse one `/evilcraft colossal` line into shell assignments.

The line the command prints (one line, in chat):

    Colossal Blood Chest [198, 101, -10]: formed=true, tank 1000/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@250], t=8412

Usage (as the infuser's `parse_infuser.py`):

    eval "$(python3 tools/parse_colossal.py "$LINE")"

It prints `ok=1` and `FORMED`, `TANK`, `CAPACITY`, `REPAIR`, `SLOTS`,
`CONTAINER_ITEM`, `CONTAINER_BLOOD`, `SLOT<n>_ITEM`, `SLOT<n>_DAMAGE`; on a line it
cannot read it prints `ok=0` and nothing else, so the caller fails with the line it had.
"""

import re
import shlex
import sys

LINE = re.compile(
    r"^Colossal Blood Chest \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"formed=(?P<formed>true|false), "
    r"tank (?P<tank>-?\d+)/(?P<capacity>-?\d+) mB, "
    r"repair (?P<repair>-?\d+), slots (?P<slots>-?\d+), "
    r'container (?P<item>[^@\s,]+)@(?P<blood>-?\d+), '
    r"\[(?P<stacks>[^\]]*)\](?:, t=(?P<time>-?\d+))?"
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
        "FORMED": match["formed"],
        "TANK": match["tank"],
        "CAPACITY": match["capacity"],
        "REPAIR": match["repair"],
        "SLOTS": match["slots"],
        "CONTAINER_ITEM": match["item"],
        "CONTAINER_BLOOD": match["blood"],
        "TIME": match["time"] or "0",
    }
    for slot in SLOT.finditer(match["stacks"]):
        index = slot["index"]
        fields[f"SLOT{index}_ITEM"] = slot["item"]
        fields[f"SLOT{index}_DAMAGE"] = slot["damage"]
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
