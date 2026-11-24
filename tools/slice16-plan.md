# Slice 16 (0.2.16) plan — the remaining vanilla-keyed Blood Infuser recipe

## Why this slice / candidate audit

`notes.md` § **Next slice** (2026-11-24, after the slice-16 promises gate) points at slice 2's
row *"31 of the jar's 40 infuser recipes | not ported | their input or output is an evilcraft item
this package does not carry"* and asks for the next `base/`+`convenience/` file whose **input and
output are both vanilla or already-carried ids**. The 40 jar files were dumped and every
`input_item`/`output_item` parsed (audit below).

- 12 are carried (9 `convenience/`, `base/blook`, the two undead `base/`).
- 7 `core/` bowl/acceptor files are excluded by the slice-16 gate (`blocked(...)`: no placeable
  acceptor surface, inert dark-ore root, tank over-capacity).
- Of the 21 remaining `base/`+`convenience/` files, **20 fail** the gate:
  - 17 name an un-carried evilcraft item on the input or the output (e.g. `blood_orb_empty`,
    `dark_gem`, `potentia_sphere`, `ender_tear`, `dull_dust`, `flesh_humanoid`);
  - `convenience/evilcraft_blood_waxed_coal.json` uses the tag `minecraft:coals` **and** an
    un-carried output;
  - `convenience/minecraft_leather.json` is both ids vanilla but is the **upstream defect**
    recorded in slice 2: the same `minecraft:rotten_flesh` input at tier 1 loses to the carried
    tier-2 spider-eye file, so it is unreachable dead data (`a_rotten_flesh_makes_eight_spider_eyes`);
  - `convenience/minecraft_wither_skeleton_skull.json` is both ids vanilla but asks **25,000 mB**
    of the 10,000 mB tank (the other slice-2 defect) — `every_carried_recipe_fits_its_tank...` refuses it.
- **The one qualifying file** is `convenience/minecraft_fermented_spider_eye.json`:
  `minecraft:spider_eye` → `minecraft:fermented_spider_eye@1`, **1,500 mB**, **500 ticks**, **tier 2**.
  Both ids are vanilla; the cost fits the tank; the tier is in `0..=3`; no other carried recipe
  uses spider_eye as an input, so there is no highest-tier conflict. This is the cheapest honest
  file: one data file, one `CARRIED` entry, one new unit rule, one in-game phase. No machine,
  block, item, entity, renderer, hook or worldgen.

**Class/tier (ledger authority):** content / **deep**; this slice is `data`/`logic` inside deep.

## Approaches considered

1. **Carry the jar file verbatim as data** (chosen). `Host::recipes_of_type` already hands every
   file of the type to the infuser, so no machine code changes — only the file, its `CARRIED`
   declaration, the count and one rule test. A data pack can still override it as upstream allows.
2. Hard-code an extra `Infusion` in `runtime.rs`. Rejected: it would not appear in
   `recipes_of_type` or in the parsed `recipes 13/13` readout, and recipes are data upstream.
3. Carry the leather or wither-skull file as well. Rejected: both are the recorded upstream
   defects (shadowed tier, over-capacity) and would be dead data or unreachable.

## Files and steps

1. **RED first** (before the data file / `CARRIED` entry): rename the count test to
   `the_infuser_carries_thirteen_recipes` and assert 13; add
   `a_spider_eye_becomes_a_fermented_spider_eye_at_tier_two` (tier 0/1 none, tier 2 one
   fermented eye for 1,500 mB/500 t). Run `cargo test` through the slot runner and keep the
   verbatim failure in `sdk-test-red-slice16.log`.
2. **GREEN:** copy the jar file byte-for-byte to
   `data/evilcraft/recipe/blood_infuser/minecraft_fermented_spider_eye.json` (the other
   convenience files are flattened the same way); add its `(id, include_str!)` entry to
   `src/infuser.rs::CARRIED`; rerun tests / fmt / clippy.
3. Bump `horizon-mod.toml` + `Cargo.toml` to 0.2.16, extend the description, credit the 13th
   recipe in `LICENSE-NOTICE.md`, re-run `tools/stamp_license.py` — all **before** the final build
   so `verify-sdk-package.py` reports "package matches the tree".
4. Ladder through `build-slot.sh` (no `BUILD_LOCAL=1`, no `--target-dir`, Rust forwarded to
   AdLf-PC): red/green tests, fmt/clippy, project check, two `rm -f Cargo.lock` builds that
   `cmp` identical, package check, two `fits` (pin + `0b14f8a`), SDK gate, 26.3 dialect gate.
5. In-game: preserve the 0.2.15 export as `evilcraft-0.2.15`, `launcher-game.sh build evilcraft`,
   confirm `Compiling evilcraft (1 of 1)` and an export byte-identical to the port. One e2e
   default leg with the 0.2.14 (10/10) and 0.2.15 (12/12) readouts as differentials, phase **AP**
   (13/13; a spider eye stays untouched for 560 ticks at tier 0 and becomes one fermented spider
   eye for 1,500 mB in 500 ticks at tier 2), a package-less control and
   `SELFTEST=fermented` (`EXIT:1`). Keep every earlier leg green under `--strict`. Regenerate
   `sdk-test.log` after the final build (the remote evidence pull overwrites it).

## Done

Version 0.2.16, two `cmp`-identical builds, gate 1/1, all e2e logs ending in `EXIT:0`
(default) / `EXIT:1` (control, selftest), coverage table and transcript in `notes.md` /
`VALIDATION.md`, and a `## Next slice` proposal naming the remaining gap.
