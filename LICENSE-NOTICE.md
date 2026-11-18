# Licence provenance

- Mod: `evilcraft` — original: <https://github.com/CyclopsMC/EvilCraft> (EvilCraft).
- Original author(s): rubensworks (aka kroeser).
- Original jar this port was written against: `evilcraft-1.21.1-neoforge-1.2.100.jar`, which declares **MIT (its `META-INF/neoforge.mods.toml` declares `license="MIT"`)**.
- Copyright of the original: Copyright (c) rubensworks (aka kroeser).
- This port is **MIT**, © 2026 the Horizon port authors.

The upstream repository's GitHub licence metadata returns 404 because it carries no root licence file; the MIT declaration above is the jar's own metadata, which is the artefact this port was written against.

The notice travels **inside the distributed archives**, not only in this source
repository: the native bundle carries `mixins/evilcraft/LICENSE` and
`mixins/evilcraft/LICENSE-NOTICE.md`, and the carried content mod carries
`data/evilcraft/license/license.txt` and `data/evilcraft/license/notice.txt`
(`tools/stamp_license.py` writes them from these two files before packaging).
