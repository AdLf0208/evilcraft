#!/usr/bin/env python3
"""Assemble the slice-1 `content/` folder for EvilCraft from the pack's own jar.

The jar is the ATM10 file (`ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`),
kept as the licence and provenance evidence. Nothing is invented: the two textures and
the two names come out of the jar, and the block/menu/tab declarations and the loot
table are written here in Horizon's 26.3 shape (a block names a vanilla template, a
loot table uses the singular typed condition).

What is *not* carried, and why, is in `../FINDINGS.md`:

- the BER chest model (`assets/evilcraft/textures/model/blood_chest.png`, drawn by
  `RenderBlockEntityBloodChest`) — content models cannot do a block entity renderer, so
  the block is a cube over the mod's own `colossal_blood_chest` block texture;
- the recipes (their ingredients — `blood_infusion_core`, `dark_spike`, `dark_gem` — are
  other machines and ores this slice does not carry);
- the tank-data loot function (`evilcraft:copy_tank_data` has no Horizon equivalent yet).

Rerun it to rebuild `content/` (it deletes its own generated folders first):

    python3 tools/assemble_content.py
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]
JAR = ROOT.parent / "_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar"
JAR_SHA1 = "720215e3810d3733846d1083ee3c95a19f42153d"
CONTENT = ROOT / "content"
MOD = "evilcraft"

MANIFEST = '''# Port-owned content mod for the EvilCraft 1.2.100 slice: the blood chest block, the
# blood extractor item, the chest's menu type and the carried assets. The two textures
# are upstream's own MIT assets, read from the exact ATM10 jar (see ../FINDINGS.md).
id = "evilcraft"
name = "EvilCraft"
version = "0.1.0"
api = "0.2"
side = "both"
capabilities = ["registries"]
'''

BLOCK = {
    "template": "minecraft:furnace",
    "properties": {
        "hardness": 2.5,
        "explosion_resistance": 2.5,
        "requires_tool": True,
        "map_color": 13,
        "sounds": "minecraft:oak_planks",
    },
    "item": True,
}

MENU = {"slots": 11, "data_slots": 2}

TAB = {
    "title": "itemGroup.evilcraft",
    "icon": "evilcraft:blood_chest",
    "items": ["evilcraft:blood_chest", "evilcraft:blood_extractor"],
}

LOOT = {
    "type": "minecraft:block",
    "pools": [
        {
            "rolls": 1.0,
            "entries": [{"type": "minecraft:item", "name": "evilcraft:blood_chest"}],
            "condition": {"type": "minecraft:survives_explosion"},
        }
    ],
}

BLOCK_MODEL = {
    "parent": "minecraft:block/cube_all",
    "textures": {"all": "evilcraft:block/blood_chest"},
}

ITEM_MODEL = {"parent": "evilcraft:block/blood_chest"}

EXTRACTOR_MODEL = {
    "parent": "minecraft:item/generated",
    "textures": {"layer0": "evilcraft:item/blood_extractor"},
}

# A plain item: no behaviour of its own (the mod's hooks own what it does).
EXTRACTOR = {"template": "minecraft:flint"}

ITEM_DEFINITION = {
    "model": {"type": "minecraft:model", "model": "evilcraft:item/blood_chest"}
}
EXTRACTOR_DEFINITION = {
    "model": {"type": "minecraft:model", "model": "evilcraft:item/blood_extractor"}
}

# `sounds` names a *block* whose SoundType is copied (wood = oak planks).
# The furnace template gives `facing` (four horizontals) and `lit`; every variant draws
# the same cube, as the block has no state-dependent look in this slice.
BLOCKSTATE = {
    "variants": {
        f"facing={facing},lit={lit}": {"model": f"{MOD}:block/blood_chest"}
        for facing in ("north", "south", "west", "east")
        for lit in ("false", "true")
    }
}

# Copied byte for byte out of the jar.
CARRIED = {
    "assets/evilcraft/textures/item/blood_extractor.png": "assets/evilcraft/textures/item/blood_extractor.png",
    "assets/evilcraft/textures/block/colossal_blood_chest.png": "assets/evilcraft/textures/block/blood_chest.png",
}

# Carried names, read from the jar's `en_us.json` (absent keys are an error, not a
# silent English fallback).
CARRIED_LANG = [
    "block.evilcraft.blood_chest",
    "item.evilcraft.blood_extractor",
]


def write(path: Path, data) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, bytes):
        path.write_bytes(data)
    else:
        path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def main() -> None:
    if not JAR.is_file():
        raise SystemExit(f"missing jar: {JAR}")
    raw = JAR.read_bytes()
    digest = hashlib.sha1(raw).hexdigest()
    if digest != JAR_SHA1:
        raise SystemExit(f"jar SHA-1 {digest} is not the pack record {JAR_SHA1}")
    archive = zipfile.ZipFile(JAR)
    for folder in ("assets", "data"):
        shutil.rmtree(CONTENT / folder, ignore_errors=True)

    for source, target in CARRIED.items():
        write(CONTENT / target, archive.read(source))

    lang = json.loads(archive.read("assets/evilcraft/lang/en_us.json"))
    names = {}
    for key in CARRIED_LANG:
        if key not in lang:
            raise SystemExit(f"the jar has no {key}")
        names[key] = lang[key]
    names["itemGroup.evilcraft"] = "EvilCraft"

    write(CONTENT / f"assets/{MOD}/lang/en_us.json", names)
    write(CONTENT / f"assets/{MOD}/blockstates/blood_chest.json", BLOCKSTATE)
    write(CONTENT / f"assets/{MOD}/models/block/blood_chest.json", BLOCK_MODEL)
    write(CONTENT / f"assets/{MOD}/models/item/blood_chest.json", ITEM_MODEL)
    write(CONTENT / f"assets/{MOD}/models/item/blood_extractor.json", EXTRACTOR_MODEL)
    write(CONTENT / f"assets/{MOD}/items/blood_chest.json", ITEM_DEFINITION)
    write(CONTENT / f"assets/{MOD}/items/blood_extractor.json", EXTRACTOR_DEFINITION)
    write(CONTENT / f"data/{MOD}/horizon/block/blood_chest.json", BLOCK)
    write(CONTENT / f"data/{MOD}/horizon/menu/blood_chest_menu.json", MENU)
    write(CONTENT / f"data/{MOD}/horizon/item/blood_extractor.json", EXTRACTOR)
    write(CONTENT / f"data/{MOD}/horizon/creative_tab/blood.json", TAB)
    write(CONTENT / f"data/{MOD}/loot_table/blocks/blood_chest.json", LOOT)
    (CONTENT / "horizon-mod.toml").write_text(MANIFEST)

    files = sorted(p for p in CONTENT.rglob("*") if p.is_file() and p.name != "README.md")
    print(f"{len(files)} files under {CONTENT}")
    for path in files:
        print(" ", path.relative_to(CONTENT))


if __name__ == "__main__":
    main()
