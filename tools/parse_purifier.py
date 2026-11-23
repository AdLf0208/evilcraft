#!/usr/bin/env python3
"""Parse one `/evilcraft purifier` line into shell assignments.

The line the command prints (one line, in chat):

    Purifier [206, 101, 0]: tank 3000/3000 mB, purge minecraft:diamond_pickaxe@1,
    add empty@0, ench 1, curses 1, tick 0, t=8412

Usage (as the dark tank's `parse_darktank.py`):

    eval "$(python3 tools/parse_purifier.py "$LINE")"

It prints `ok=1` and `X`, `Y`, `Z`, `TANK`, `CAPACITY`, `PURGE_ITEM`, `PURGE_COUNT`,
`ADD_ITEM`, `ADD_COUNT`, `ENCH`, `CURSES`, `TICK` and `TIME`; on a line it cannot read it
prints `ok=0` and nothing else, so the caller fails with the line it had.
"""

import re
import shlex
import sys

LINE = re.compile(
    r"^Purifier \[(?P<x>-?\d+), (?P<y>-?\d+), (?P<z>-?\d+)\]: "
    r"tank (?P<tank>-?\d+)/(?P<capacity>-?\d+) mB, "
    r"purge (?P<purge_item>\S+)@(?P<purge_count>-?\d+), "
    r"add (?P<add_item>\S+)@(?P<add_count>-?\d+), "
    r"ench (?P<ench>-?\d+), curses (?P<curses>-?\d+), tick (?P<tick>-?\d+)"
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
        "PURGE_ITEM": match["purge_item"],
        "PURGE_COUNT": match["purge_count"],
        "ADD_ITEM": match["add_item"],
        "ADD_COUNT": match["add_count"],
        "ENCH": match["ench"],
        "CURSES": match["curses"],
        "TICK": match["tick"],
        "TIME": match["time"] or "0",
    }
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
