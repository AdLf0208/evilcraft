# EvilCraft for Horizon — the blood economy

Ported from **EvilCraft 1.2.100** (`evilcraft`, CC BY 4.0 for code and images per its
release README, MIT in its build metadata, CyclopsMC/EvilCraft) for
Minecraft Java 26.3 on Horizon at pin `0a875da` + mixins `e59da909`.

## What the package carries

- **Blood Extractor** (`evilcraft:blood_extractor`): a 5,000 mB container that fills with
  `floor(hp × 5)` to `floor(hp × 40)` mB of blood when its carrier kills a mob (melee,
  projectile or any other cause credited to the player). A crouching click drains a Blood
  Stain or pours into a Dark Tank.
- **Blood Chest** (`evilcraft:blood_chest`), **Blood Infuser** (`evilcraft:blood_infuser`),
  **Colossal Blood Chest** (`evilcraft:colossal_blood_chest`) and **Purifier**
  (`evilcraft:purifier`): machines over tanks of blood — repairing items (one damage point
  every two ticks for 5 mB, an anvil step for 125 mB), infusing items through the jar's own
  recipe files, and stripping one curse per 60 ticks and 1,000 mB. Each damage point the
  Blood Chest repairs has an upstream 1-in-10,000 chance to add a random `#minecraft:curse`
  enchantment (binding or vanishing curse, level 1); the Colossal Chest's bulk repair never
  curses, as upstream's `!isBulk` gate requires.
- **Blood Stain** (`evilcraft:blood_stain`): left by a mob's fall death with 20 mB per point
  of max health; a crouching extractor click drains it.
- **Dark Tank** (`evilcraft:dark_tank`): a 16,000 mB field sink.
- **Blook**, three **Promise of Tenacity** tiers and the **Vengeance Pickaxe**: carried items.
  Upstream's `ToolBadEnchantPurifyAction` refuses to strip curses from the pickaxe, so its
  own curse survives the Purifier; this package keeps that item-id rule.
- Every machine's own item carries its blood tank when broken, clamped to that machine's
  capacity when placed again (`evilcraft:copy_tank_data`).
- The five tank blocks declare a **comparator output**: upstream's `ceil(15 × amount / capacity)`
  of their blood tank, published from each machine's tick and announced to nearby comparators
  whenever it changes. A full Purifier reads 15; 2,000 of 3,000 mB reads 10.

All of it ships in one flat SDK `.hmod`: content, native code and manifest together.

## What it does not

The other three Purifier actions, the jar's crafting recipes (their ingredients are not
carried), worldgen, entities and renderers that a content block cannot express. See
`notes.md` for the full coverage table and the recorded deviations.

## Building

The port is the flat SDK package (`api = "0.2"`, `[native]` pinned to the pin above):

```sh
cargo horizon-mod build                    # target/horizon-mod/evilcraft.hmod
cargo horizon-mod check evilcraft.hmod
cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a
```

Through the campaign's build registry (compile steps forward to the remote box):

```sh
ports/tools/build-slot.sh run --job G12-evilcraft --port ports/evilcraft --step build -- \
    bash -c 'rm -f Cargo.lock && cargo horizon-mod build'
```

The deliverable is `evilcraft.hmod` in this folder; `target/` is disposable. Its size and
SHA-256 come from `cargo horizon-mod check evilcraft.hmod` and move whenever a packed file
changes. `tools/horizon.py`, `tools/package.py`, `tools/package.sh` and
`tools/assemble_content.py` belong to the retired `.hnmod` format and are kept as history.

## Licence

The upstream mod grants CC BY 4.0 for all code and images (its release README) and MIT in its
build metadata; both texts are kept here, and the carried textures, models and language
strings come from the jar (`FINDINGS.md` records the SHA-1). Local only until the maintainer
approves publishing.
