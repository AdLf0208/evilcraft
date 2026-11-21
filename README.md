# EvilCraft for Horizon — slice 1: the blood repair loop

Ported from **EvilCraft 1.2.100** (`evilcraft`, CC BY 4.0 for code and images per its
release README, MIT in its build metadata, CyclopsMC/EvilCraft) for
Minecraft Java 26.3 on Horizon.

## What this slice implements

- **Blood Extractor** (`evilcraft:blood_extractor`): a container for 5,000 mB of blood
  that fills when the player's melee kills a mob — `floor(hp × 5)` to
  `floor(hp × 40)` mB, so a 20 HP zombie gives 100–799 mB.
- **Blood Chest** (`evilcraft:blood_chest`): 10 repair slots and a factor slot over a
  10,000 mB tank of blood. A damaged item loses one damage point every two ticks for
  5 mB; a chipped anvil becomes an anvil for 125 mB; a Blood Extractor in the factor
  slot empties into the tank. Its own 11-slot menu and a tank readout.

Both halves ship in the single `.hmod`: one package carries the block, the item, the menu
type and the code, so installing one file gives the content and the behaviour.

## What it does not

No recipes (their ingredients are other machines and ores), no curse roll, no BER
chest model, no projectile-kill blood, and none of the rest of the jar (blood stains,
the colossal chest, the infuser, the purifier, dark ore, undead wood, spirits,
werewolves, promises). See `notes.md` for the full gap list and the deviations
(blood is a tank id, the extractor's blood lives in the stack's `custom_data`).

## Building

The port is the flat SDK package (`api = "0.2"`, `[native]` at nightly `0b14f8a`):

```sh
cargo horizon-mod build     # run in this folder: target/horizon-mod/evilcraft.hmod
cargo horizon-mod check evilcraft.hmod
cargo horizon-mod fits evilcraft.hmod
```

Through the campaign's build registry:

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-ec --port ports/evilcraft --step check -- \
    bash -lc 'cd ports/evilcraft && cargo horizon-mod check evilcraft.hmod'
```

The deliverable is `evilcraft.hmod` in this folder. `cargo horizon-mod build` writes it under
`target/`, which is disposable — copy it here after a build, and take the size and SHA-256 from
`cargo horizon-mod check evilcraft.hmod` (the recorded pair is in `notes.md` and `VALIDATION.md`;
it moves whenever a packed file, this README included, changes). 
`tools/horizon.py`, `tools/package.py`, `tools/package.sh` and `tools/assemble_content.py`
belong to the retired `.hnmod` format and are kept only as history.

## Licence

The upstream mod grants CC BY 4.0 for all code and images (its release README) and MIT in its
build metadata; both texts are kept here, and the two carried textures and
the two item/block names come from the jar (`FINDINGS.md` records the SHA-1). Local
only until the maintainer approves publishing.
