# Slice 15 (0.2.15) plan — the two undead-tree Blood Infuser recipes

## Why this slice

Slice 14 (`0.2.14`) carried the eleven Undead Wood blocks and the jar's **seven crafting
recipes**, and left this exact row in `notes.md`:

> | jar's `base/undead_sapling`, `base/reinforced_undead_planks`, `broom_part_rod_undead` recipes |
>   not ported | add-on/infuser families outside this slice's crafting set |

The two `base/` files are **Blood Infuser** recipes — the machine this port ships since 0.2.2 —
and their inputs and outputs are all carried now (`minecraft:dead_bush`,
`evilcraft:undead_planks`, `evilcraft:undead_sapling` from slice 14,
`evilcraft:reinforced_undead_planks` from slice 7). They close the loop 0.2.14 opened.
`broom_part_rod_undead` is an add-on-family crafting recipe (no broom family is carried) and
stays a gap.

**Class/tier (ledger authority):** content / **deep**. This slice is `logic`/`data` inside deep:
two data files, their `CARRIED` declarations and the ids they name. No block, block entity,
entity, renderer, hook or worldgen. No parent mod outside the registry.

## The artefact (read from the jar, not the dispatch note)

`ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`:

| file | input | blood | output | count | duration | tier |
|---|---|---:|---|---:|---:|---:|
| `data/evilcraft/recipe/blood_infuser/base/undead_sapling.json` | `minecraft:dead_bush` | 800 mB | `evilcraft:undead_sapling` | 1 | 200 t | **omitted → 0** |
| `data/evilcraft/recipe/blood_infuser/base/reinforced_undead_planks.json` | `evilcraft:undead_planks` | 1,000 mB | `evilcraft:reinforced_undead_planks` | 1 | 250 t | **2** |

Both are carried byte for byte at `data/evilcraft/recipe/blood_infuser/base/<name>.json`,
exactly as `base/blook.json` was (same directory, no flattening). The jar's object shapes
(`{"item": …}`, `{"id": …, "amount": …}`) are what `decode_infusion` already reads.

## Approaches considered

1. **Carry the two jar files as data only** (chosen). The infuser already reads all recipes of
   its type through `Host::recipes_of_type`; no machine code changes. The tests pin the two
   rules and the 10 → 12 count; the game proves the readout and both infusions. This is the
   smallest change that makes the carried ids reachable.
2. Hard-code the two recipes in `runtime.rs` as extra `Infusion` values. Rejected: recipes are
   data upstream (a pack may override them), and a hard-coded entry would not show up in
   `recipes_of_type` or the `recipes 12/12` readout.
3. Carry `broom_part_rod_undead` too. Rejected: its broom family is not carried, so it would be
   dead data — exactly the defect the port's `every_carried_recipe_names_items…` test refuses.

## Files and steps

1. **RED first** (no data files, no `CARRIED` entries yet): in `src/runtime.rs`, add
   `the_infuser_carries_twelve_recipes`, `a_dead_bush_becomes_an_undead_sapling_at_tier_zero`
   and `undead_planks_become_reinforced_planks_at_tier_two`; widen the tier assertion in
   `every_carried_recipe_fits_its_tank_and_names_a_tier` from `1..=3` to `0..=3` (the sapling
   file omits `tier`, which upstream reads as 0); add the three item ids to `CARRIED_ITEMS`.
   Run `cargo test` and keep the verbatim failure in `sdk-test-red-slice15.log`.
2. **GREEN**: copy the two jar recipe files; add their `(id, include_str!)` entries to
   `src/infuser.rs::CARRIED`; rerun tests/clippy/fmt.
3. Bump `horizon-mod.toml` + `Cargo.toml` to 0.2.15, update the description, credit the two
   files in `LICENSE-NOTICE.md`, re-run `tools/stamp_license.py` — all **before** the final
   build, so `verify-sdk-package.py` reports "package matches the tree".
4. Ladder through `build-slot.sh` (no `BUILD_LOCAL=1`, no `--target-dir`, forwarded to
   AdLf-PC): red/green tests, fmt/clippy, project check, two `rm -f Cargo.lock` builds that
   `cmp` identical, package check, two `fits` runs (the pin and `0b14f8a`), SDK gate.
5. In-game: preserve the 0.2.14 export, `launcher-game.sh build evilcraft`, confirm
   `Compiling evilcraft (1 of 1)`, then one e2e default leg with a 0.2.14 `10/10` leg, phase
   **AO** (`12/12`, dead_bush → sapling at tier 0 for 800 mB/200 t; reinforced planks inert
   at tier 0 and one block at tier 3 for 1,000 mB/250 t), a control leg (`EXIT:1`) and
   `SELFTEST=undeadinfuse` (`EXIT:1`). Keep every earlier leg green under `--strict`.

## Done

Version 0.2.15, two `cmp`-identical builds, gate 1/1, all e2e logs ending in `EXIT:0`
(default) / `EXIT:1` (control, selftest), coverage table and transcript in `notes.md` /
`VALIDATION.md`, and a `## Next slice` proposal naming the remaining gap.
