#!/usr/bin/env python3
"""Copy this repository's licence and provenance notice into the carried content.

The `.hmod` is what players download, and the licence above requires its notice to
travel with copies of the work. A mod's carried files are `assets/` and `data/`, plus
the root files the packer takes, so the notice rides as data the game ignores:
`data/evilcraft/license/license.txt` and `data/evilcraft/license/notice.txt`, written from
the repository's `LICENSE` and `LICENSE-NOTICE.md`. Rerun after any licence change;
`cargo horizon-mod build` packs whatever is there.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "data" / "evilcraft" / "license"

OUT.mkdir(parents=True, exist_ok=True)
for source, name in (("LICENSE", "license.txt"), ("LICENSE-NOTICE.md", "notice.txt")):
    text = (ROOT / source).read_text()
    (OUT / name).write_text(text)
    print(f"wrote {OUT / name}")
