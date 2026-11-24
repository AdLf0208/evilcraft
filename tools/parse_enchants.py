#!/usr/bin/env python3
"""Parse the `minecraft:enchantments` component of a saved stack.

`data get block <pos> Items[n]` answers

    <x>, <y>, <z> has the following block data: {components: {"minecraft:enchantments":
    {"minecraft:vanishing_curse": 1}}, count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}

The 26.3 component is a bare map from enchantment id to level. This prints
`ok=1 ench=<id>@<level>[,<id>@<level>...]`, or `ok=0 ench=none` when the map is absent
(an unenchanted stack carries no component at all). The caller asserts the parsed
value — never the absence of an error.

Usage: eval "$(python3 tools/parse_enchants.py "$SAVED_ITEM")"
"""

import re
import sys

MAP = re.compile(r'minecraft:enchantments"\s*:\s*\{([^{}]*)\}')
ENTRY = re.compile(r'"([^"]+)"\s*:\s*(\d+)')


def main() -> None:
    line = sys.argv[1] if len(sys.argv) > 1 else ""
    found = MAP.search(line)
    if found is None:
        print("ok=0 ench=none")
        return
    entries = ENTRY.findall(found.group(1))
    print("ok=1 ench=" + (",".join(f"{key}@{level}" for key, level in entries) or "empty"))


if __name__ == "__main__":
    main()
