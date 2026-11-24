# Slice 11 — the Vengeance Pickaxe's purifier immunity (0.2.11)

## The slice, from this port's own gaps

Slice 9's coverage table (`notes.md`, row `ItemVengeancePickaxe immunity`):

> | `ItemVengeancePickaxe` immunity | not ported | the item is not carried, so no stack can match it |

### What the artefact says the rule actually is (the brief's guess was wrong)

The dispatch note suggested "a named attacker wearing/holding it is immune to the pickaxe's
revenge damage" and a possible damage-source query. The artefact disagrees, and wins:

- `ports/_shared/upstream/evilcraft/src/main/java/org/cyclops/evilcraft/blockentity/tickaction/purifier/ToolBadEnchantPurifyAction.java:40,73`
  guards both `canWork` and `work` with
  `enchantment.is(EnchantmentTags.CURSE) && !(purifyItem.getItem() instanceof ItemVengeancePickaxe)`.
  The immunity is **the Purifier's curse action refusing to strip curses from the Vengeance
  Pickaxe** — no damage hook, no HurtSource query, no entity needed.
- `ItemVengeancePickaxe.java` only overrides destroy speed and the enchanted stack it is
  crafted as (`minecraft:fortune` 5 + `evilcraft:vengeance` 3); `evilcraft:vengeance` is in
  `data/minecraft/tags/enchantment/curse.json`. So the pickaxe is born with a curse that the
  Purifier deliberately will not remove.
- The jar has `assets/evilcraft/models/item/vengeance_pickaxe.json`,
  `assets/evilcraft/textures/item/vengeance_pickaxe.png` and two lang keys. Its crafting
  recipe needs `evilcraft:hardened_blood_shard` and `evilcraft:dark_stick`, which this port
  does not carry, so the recipe stays a gap (gap 1) and the item is `/give`-able / creative.

## Approaches considered (brainstorming)

1. **Damage immunity** (the note's first guess) — would need a damage-source query naming a
   Vengeance Spirit attacker and an item worn by the victim; *the artefact has no such rule*,
   so this would invent a feature. Rejected.
2. **Comparator signal for a carried block** (the note's second guess) — expressible at this
   pin (`has_analog_output`/`analog_output`, powah proved it), but it is a redstone detail on
   an existing machine, needs a cache/notification path, and carries no jar item.
3. **Purifier curse immunity + the carried item** (chosen) — one item and one pure rule on the
   machine slice 9 already ships; the in-game assertion reuses the existing purifier harness
   and phase AG is its ready-made vanilla differential. Cheapest family that is really in the
   jar, really in the gap list, and provable fail-capably at this pin.

**Tier:** `deep` still governs the port; this slice is a `logic` family inside it — one item and
one rule on the existing machine, no block entity, entity, renderer or dimension added.

## What this slice ships

| File | Change |
|---|---|
| `src/runtime.rs` | `VENGEANCE_PICKAXE` const, `purifiable_curses(item, levels)`, `purified_for(item, levels)`; add the id to `CARRIED_ITEMS`; **2 new tests** |
| `src/purifier.rs` | `block_entity_tick` passes the purge stack's item id so `canWork` and `work` both refuse that item (upstream guards both) |
| `data/evilcraft/horizon/item/vengeance_pickaxe.json`, `assets/evilcraft/{items,models/item,textures/item}/…`, `assets/evilcraft/lang/en_us.json`, creative tab | the jar's own model/texture/lang bytes; `template: minecraft:diamond_pickaxe` (upstream `PickaxeItem(Tiers.DIAMOND)`); creative row |
| `tools/e2e.sh` | phase AJ: a cursed Vengeance Pickaxe in a full purifier keeps its curse and its tank after 140 ticks, the same cursed vanilla pickaxe loses both, unknown id refused, `SELFTEST=vengeance` flips the assertion |
| `horizon-mod.toml`, `Cargo.toml`, `README.md`, `notes.md`, `VALIDATION.md`, `LICENSE-NOTICE.md` | version 0.2.11, before the final build |

## Proof

1. Red: the two runtime tests written first with `purifiable_curses`/`purified_for` stubs that
   ignore the item id (old behaviour); both fail with the Vengeance Pickaxe case.
2. Green: the two rules, `cargo test` clean, `cargo fmt --check` / `cargo clippy -D warnings`.
3. In game: `/evilcraft purifier` reads `purge evilcraft:vengeance_pickaxe@1, ench 1, curses 1,
   tick 0` and `tank 3000/3000` after 140 ticks (the curse and the blood untouched); the same
   rig with a cursed vanilla diamond pickaxe strips the curse and spends 1,000 mB (phase AG's
   rule, restated in AJ); a comparator-style stone/vanilla differential is not needed here —
   the vanilla item is the differential; `SELFTEST=vengeance` ends `EXIT:1`.

## Deviations

- The item's 1.25× destroy speed and its `evilcraft:vengeance` enchantment (and the
  spirit-summoning entity effect) are Java-side; a content item with the diamond-pickaxe
  template cannot set either. The in-game proof uses a vanilla curse so the purifier's own
  curse path is exercised; the immunity is about the *item id*, which is carried faithfully.
- The crafting recipe is not carried (its ingredients are not in this port — gap 1).
