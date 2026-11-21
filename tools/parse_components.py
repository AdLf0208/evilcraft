#!/usr/bin/env python3
"""Read the blood tank a dropped machine's item carries.

`ctl entities`/`ctl inventory` print a stack as
`{"item": ..., "count": ..., "components": <SNBT>}` (`game/agent.rs::stack_value`),
where the SNBT is `snbt::to_snbt` of the stack's whole component patch. A broken
Blood Chest or Blood Infuser carries its tank there, as the vanilla component
`minecraft:block_entity_data` tagged with the block entity type's id:

    {minecraft:block_entity_data: {fluids: [{Amount: 6500, Fluid: "evilcraft:blood"}],
                                   id: "evilcraft:blood_chest"}}

Usage (as the port's other parsers):

    eval "$(python3 tools/parse_components.py "$COMPONENTS" evilcraft:blood_chest)"

It prints `ok=1`, `carried=<mB>`, `fluid=<id>`, `carrier=<block entity type id>` and
`items=<how many Items/items keys the payload holds>`; `ok=0` (and nothing else) when
the stack carries no block entity data of `carrier` with a blood tank, so the caller
fails with the string it had.

`items` is the double-carry check: the engine already spills a mod container's stacks
as their own item entities when the block is removed, so a payload that also carried
them would put every stack in the world twice.
"""

import re
import shlex
import sys

KEY = "minecraft:block_entity_data"
AMOUNT = re.compile(r"\bAmount:\s*(-?\d+)")
FLUID = re.compile(r'\bFluid:\s*"([^"]*)"')


def value_after(text, key):
    """The balanced `{...}` block that follows `key:` anywhere in `text`."""
    at = text.find(key)
    if at < 0:
        return None
    start = text.find("{", at + len(key))
    if start < 0:
        return None
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
    return None


def main(argv):
    if len(argv) != 3:
        print("ok=0")
        return 1
    carrier = argv[2]
    payload = value_after(argv[1], KEY)
    if payload is None:
        print("ok=0")
        return 1
    # TypedEntityData only loads a payload whose id names the block entity it is
    # placed as, so the tag is part of the claim.
    if not re.search(r'\bid:\s*"%s"' % re.escape(carrier), payload):
        print("ok=0")
        return 1
    amounts = AMOUNT.findall(payload)
    fluids = FLUID.findall(payload)
    if len(amounts) != 1 or len(fluids) != 1:
        print("ok=0")
        return 1
    fields = {
        "ok": "1",
        "carried": amounts[0],
        "fluid": fluids[0],
        "carrier": carrier,
        "items": str(len(re.findall(r"\bItems:|\bitems:", payload))),
    }
    for key, value in fields.items():
        print(f"{key}={shlex.quote(value)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
