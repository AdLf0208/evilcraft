# Licence provenance

- Mod: `evilcraft` — original: <https://github.com/CyclopsMC/EvilCraft> (EvilCraft).
- Original author(s): rubensworks (aka kroeser).
- Original jar this port was written against: `evilcraft-1.21.1-neoforge-1.2.100.jar`
  (SHA-1 `720215e3810d3733846d1083ee3c95a19f42153d`), which declares **MIT** in its
  `META-INF/neoforge.mods.toml` (built from `gradle.properties`: `license=MIT`).
- Copyright of the original: Copyright (c) rubensworks (aka kroeser).

## Which grant applies (resolved 2026-11-18, review #1168)

The exact release this port was written against declares **two** grants, and they are not the
same:

| Artefact | Where it is declared | Grant |
|---|---|---|
| code | `gradle.properties` (`license=MIT`), copied into the jar's `neoforge.mods.toml` | **MIT** |
| **code and images** | the release's own `README.md` (tag `1.21.1-1.2.100`): *"All code and images are licenced under [Creative Commons 4](http://creativecommons.org/licenses/by/4.0/)"* | **CC BY 4.0** |

The README is the narrower, more specific grant and it is the one that covers the **six carried
textures** — `assets/evilcraft/textures/item/blood_extractor.png`,
`assets/evilcraft/textures/block/blood_chest.png` (the upstream file of
`colossal_blood_chest.png`), `assets/evilcraft/textures/block/blood_stain_0.png` …`_2.png` and
`assets/evilcraft/textures/item/blood_stain.png`. This port therefore distributes under
**CC BY 4.0**, which requires attribution, and keeps the MIT grant on record as the build
metadata's alternative.

### Attribution (CC BY 4.0 §3(a))

- **Creator:** rubensworks (aka kroeser), for EvilCraft.
- **Source:** <https://github.com/CyclopsMC/EvilCraft>, tag `1.21.1-1.2.100`.
- **Changes:** this is an independent, clean-room **Rust** rewrite for Horizon (Minecraft Java
  26.3). No Java code is translated. The carried textures are the upstream images, unchanged
  except for the chest's block texture being renamed to `blood_chest.png`; the Blood Stain's
  blockstate, its three block models and the `minecraft:replaceable` tag it adds are also the
  jar's own files, byte for byte. Every other model, blockstate, item definition, loot table,
  recipe, menu and language file is written by this port.
- **Licence of this port:** CC BY 4.0 (© 2026 the Horizon port authors), with the original
  copyright above retained. The MIT grant remains available for the code under the upstream
  build metadata (`LICENSE-MIT`).

## Where the texts travel

`LICENSE` (CC BY 4.0 legal code) and `LICENSE-MIT` (the MIT text) are the repository's licence
files; `LICENSE-NOTICE.md` is this notice. All three travel **inside the package** as
unreferenced data the game ignores:

- `data/evilcraft/license/license.txt` (CC BY 4.0),
- `data/evilcraft/license/license-mit.txt` (MIT),
- `data/evilcraft/license/notice.txt` (this file),

written by `tools/stamp_license.py` before packaging. `cargo horizon-mod build` packs whatever is
there, so rerun the tool after any licence change.
