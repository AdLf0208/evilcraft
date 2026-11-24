# Slice 13 plan — the Blood Chest's curse roll on repair (0.2.13)

## The artefact's rule (read, not guessed)

`ports/_shared/upstream/evilcraft/src/main/java/org/cyclops/evilcraft/blockentity/tickaction/bloodchest/DamageableItemRepairAction.java`
lines 34–55. After `setDamageValue(getDamageValue() - 1)` (line 38):

```java
if (!isBulk
    && BlockBloodChestConfig.addRandomBadEnchants          // = true
    && BlockBloodChestConfig.randomBadEnchantsChance >= 0  // = 10000
    && random.nextInt(BlockBloodChestConfig.randomBadEnchantsChance) == 0) {
    List<Holder.Reference<Enchantment>> curses = lookup(Registries.ENCHANTMENT)
        .listElements().filter(e -> e.is(EnchantmentTags.CURSE)).toList();
    Holder<Enchantment> enchantment = curses.get(random.nextInt(curses.size()));
    itemStack.enchant(enchantment,
        enchantment.value().getMinLevel()
            + random.nextInt(getMaxLevel() - getMinLevel() + 1) - 1);
}
```

So the exact gate is **four** conditions, not just the constant:

1. `!isBulk` — the ordinary Blood Chest's `RepairItemTickAction` passes `false`. The
   Colossal Blood Chest's `BulkRepairItemTickAction` passes `true` and **never** curses.
2. `addRandomBadEnchants == true`.
3. `randomBadEnchantsChance >= 0` (guards `nextInt(0)`).
4. `random.nextInt(chance) == 0` — the draw is bounded to `[0, chance)`; the curse is added
   on **roll 0 only**, i.e. 1 in 10,000, not on `roll < chance`.

The enchantment is **not** a hardcoded `minecraft:vanishing_curse`: it is a random element of
the `#minecraft:curse` enchantment tag. At this pin that tag has vanilla's two curses —
`minecraft:binding_curse` and `minecraft:vanishing_curse` (already in `runtime::CURSES`, from
slice 9's purifier). The level is `min + nextInt(max - min + 1) - 1`; both curses declare
`min == max == 1`, so it is always **level 1**.

## Implementation

- `runtime.rs`: keep `BAD_ENCHANT_CHANCE` (drop the dead-code allowance), add
  `bad_enchant_roll(roll) -> bool` (`roll < chance && roll == 0`), `rolled_curse(index)`,
  `curse_level(roll)` and `add_curse(levels, curse, level)`. Tests red first.
- `blood.rs`: on the chest's **non-bulk** damage repair only, after `write_damage`, draw
  the roll from the level's random (upstream's `tile.getLevel().random`) unless the harness
  forced one; on roll 0 pick the `#curse` entry and write it into the stack's
  `minecraft:enchantments` component through `purifier`'s proven writer. The anvil and the
  Colossal (bulk) paths are untouched.
- `purifier.rs`: make `Enchantments`/`enchantment_levels`/`write_enchantments` `pub(crate)`
  so the chest uses the same 26.3 bare-map write path.
- A harness command `/evilcraft curse <pos> <roll> <index>` (op) forces the next chest
  repair's roll and curse index; nothing forced → the level's real random.

## In-game proof (`e2e.sh` phase AL)

A clean damaged diamond pickaxe in a funded Blood Chest, forced roll **0**, index 1 →
after the repair ticks `data get block <pos> Items[0]` shows
`minecraft:enchantments: {minecraft:vanishing_curse: 1}` and the damage/tank moved. The
**differential**: the same tool in the same chest at roll **9999** repairs just as much but
the saved stack has **no** `minecraft:enchantments`. `SELFTEST=curse` inverts the roll-0
assertion and exits 1. The 22 existing legs stay green.

## Order / traps

1. Red test first, through the slot runner (forwards to AdLf-PC; no `BUILD_LOCAL=1`).
2. Glue, then green tests + clippy + fmt, project check.
3. Bump both manifests to **0.2.13**. Regenerate `sdk-test.log`.
4. **Edit README.md (packed) before the final build**; `verify-sdk-package.py` must print
   `package matches the tree`. R9: SDK dep only, no lock, no extra Cargo tables.
5. Two `rm -f Cargo.lock && cargo horizon-mod build` → `cmp` identical; `check`; `fits`
   at `0a875da` and `0b14f8a`.
6. Launcher build → `Compiling evilcraft (1 of 1)`; remove stale `native-*` tags; use
   `launcher-game.sh env` so `E2E_TMP` is set (never `/tmp`).
7. e2e default + control + `SELFTEST=curse`; `--strict` green. Block on every lease.
