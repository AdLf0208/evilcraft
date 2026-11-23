# EvilCraft — jar inventory and upstream findings

- **Mod:** EvilCraft (`evilcraft`), content / **deep**, core, P0.
- **Jar read:** `ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`
  (2,072 entries, 5,411,079 B), `META-INF/neoforge.mods.toml` → `modId="evilcraft"`,
  `version="1.2.100"`, **`license="MIT"`**, author rubensworks (kroeser), requires
  `cyclopscore [1.28.0,)`. A second mod id in the same jar, `evilcraftcompat` 1.1.0,
  carries only integration glue (`data/evilcraftcompat/info`, lang/textures).
- **Upstream source read (analysis only):** <https://github.com/CyclopsMC/EvilCraft>,
  tag `1.21.1-1.2.99`, commit `de23efdb83ea1b84addad6e3b1dc51a37147dc1a`
  (2026-10-16). The jar is 1.2.100 (one build after the last 1.21.1 tag); the files
  read below are unchanged between that tag and the jar's data files as far as the
  carried assets show.
- **Licence rank 0** (MIT, permissive). Local-only until the maintainer approves
  publishing (ARCHITECTURE §8).

## What the jar contains (families, from `unzip -l` and the models/data)

A large mod (2,072 entries; ~180 items, ~80 blockstates). The families the jar
expresses, not the name:

| Family | Examples (jar ids) |
|---|---|
| Blood machinery | `blood_chest`, `colossal_blood_chest`, `blood_infuser`, `purifier`, `spirit_furnace`, `spirit_reanimator`, `environmental_accumulator`, `sanguinary_environmental_accumulator`, `dark_tank`, `entangled_chalice`, `display_stand`, `box_of_eternal_closure` |
| Blood items | `blood_extractor`, `bucket_blood`, `blood_orb_empty/filled`, `blood_waxed_coal(+block)`, `condensed_blood`, `blood_potash`, `blood_pearl_of_teleportation`, `creative_blood_drop`, `bound_blood_drop`, `blood_infusion_core` |
| Dark ore / gems | `dark_ore`, `dark_ore_deepslate`, `dark_gem`, `dark_gem_crushed`, `dark_power_gem(+block)`, `dark_block`, `dark_brick`, `dark_blood_brick`, `dull_dust`, `dark_spike`, `dark_stick` |
| Undead wood set | `undead_log(_stripped)`, `undead_wood(_stripped)`, `undead_planks(+stairs/slab/fence/fence_gate)`, `undead_leaves`, `undead_sapling`, `reinforced_undead_planks(+stairs)` |
| Vengeance / spirits (deep) | `vengeance_spirit`, `vengeance_focus_0..3`, `piercing_vengeance_focus_0..3`, `vengeance_essence(+materialized)`, `vengeance_ring`, `vein_sword`, `necromancer_staff`, `spectral_glasses`, `spirit_portal`, `controlled_zombie` |
| Werewolf / mobs | `werewolf`, `werewolf_fur`, `werewolf_bone`, `netherfish`, `poisonous_libelle`, `blook`, `poison_sac` |
| Promises / bowls (progression) | `bowl_of_promises_*`, `promise_acceptor_{iron,gold,diamond}`, `promise_{speed,efficiency}_0`, `exalted_crafter(_wooden/_empowered)` |
| Misc | `lightning_bomb`, `lightning_grenade`, `redstone_grenade`, `kineticator(+repelling)`, `broom`/`broom_part_*`, `mace_of_destruction`, `mace_of_distortion`, `sceptre_of_thunder`, `effortless_ring`, `invigorating_pendant(+straight)`, `primed_pendant`, `obscured_glass`, `invisible_redstone`, `spiked_plate`, `blood_stain`, `hardened_blood(+_shard)`, `bloody_cobblestone`, `flesh_rejuvenated/humanoid/werewolf`, `garmonbozia`, `golden_string`, `weather_container`, `biome_extract`, `potentia_sphere`, `ender_tear`, `corrupted_tear`, `excrement_pile`, `darkened_apple`, `poison(_bottle)`, `large_door_item` |

Recipes fall into `crafting/` (≈110), `blood_infuser/base` + `blood_infuser/core` +
`blood_infuser/convenience` (a custom recipe type — infusion in blood), `smelting/`,
`environmental_accumulator/` (a custom type), `stonecutting/` and `special/`.

**The brief says "smallest complete blood/ritual loop".** The artifact confirms a
blood economy (extractor → blood → blood chest / blood infuser / purifier), not a
ritual system (`sanguinary_pedestal` is a *blood* pedestal, and the `environmental
accumulator` is weather/environment-driven, not a ritual). The slice below is read
from the jar, not guessed from the name.

## The blood loop, exactly as upstream does it

### Source 1 — `evilcraft:blood_extractor` (item)

`ItemBloodExtractor` + `ItemBloodExtractorConfig` (1.21.1-1.2.99):

- container capacity **5,000 mB** (`containerSize = 5000`; an item fluid container).
- On `LivingDeathEvent` where the killer's `DamageSource.getEntity()` is a
  `ServerPlayer`, `fillForAllBloodExtractors(player, min, max)` fills **every Blood
  Extractor anywhere in the player's inventory**, one at a time, until the amount is
  spent:
  - `minimumMB = floor(maxHealth × 5)` (`minimumMobMultiplier = 5`),
  - `maximumMB = floor(maxHealth × 40)` (`maximumMobMultiplier = 40`),
  - `toFill = minimumMB + rand.nextInt(max(1, maximumMB − minimumMB))`.
  - A Vein Sword held raises both multipliers by `extractionBoost`.
- So a 20 HP zombie gives 100–799 mB; a 10 HP cow 50–399 mB.
- Shift-right-click on a `blood_stain` block drains it into the extractor.
- `autoFillBuckets = false` by default.

### Sink — `evilcraft:blood_chest` (block + block entity)

`BlockEntityBloodChest` + `BlockBloodChestConfig` + `RepairItemTickAction` +
`BloodChestRepairActionRegistry`:

- **Slots:** `CHEST_INVENTORY_ROWS = 2` × `CHEST_INVENTORY_COLUMNS = 5` = **10** repair
  slots plus **1** fluid-container slot (`SLOT_CONTAINER = 10`), stack size 64.
- **Tank:** `LIQUID_PER_SLOT = FluidHelpers.BUCKET_VOLUME * 10` = **10,000 mB** of
  blood; a tank of the mod's own `evilcraft:blood` fluid, and an
  `ImplicitFluidConversionTank` (other mods' blood-like fluids convert at a ratio).
- **Repair:** `mBPerDamage = 5`, `ticksPerDamage = 2`. Each damaged item loses **1
  damage point per 2 ticks** while the tank holds ≥ 5 mB, and the tank loses
  `ceil(5 × multiplier)` mB (multiplier 1 for damageable items).
- **Curses:** `addRandomBadEnchants = true`, `randomBadEnchantsChance = 10000`: each
  repaired damage point has a 1/10,000 chance to add a random **curse** enchantment.
- **Anvils:** `AnvilRepairAction` — `chipped_anvil → anvil`, `damaged_anvil →
  chipped_anvil`, **25 mB** a step.
- **Blacklist:** `itemBlacklist = ["minecraft:stick"]` (regex, item id).
- The container slot empties a blood container into the tank
  (`EmptyFluidContainerInTankTickAction`), so a player tops the chest up with a
  filled extractor.
- Block: `requiresCorrectToolForDrops`, `strength(2.5)`, `SoundType.WOOD`, a
  `facing` property, and a BER chest model
  (`assets/evilcraft/textures/model/blood_chest.png`).
- Loot: itself, `copy_name` + `evilcraft:copy_tank_data` (the tank travels with the
  dropped block).

### Secondary source — `evilcraft:blood_stain` (block)

`BlockBloodStain` + `BlockBloodStainConfig`: a mob that dies from **fall damage**
leaves a blood stain block (on the block below, or one above it when that is already
a stain) holding `bloodMBPerHP = 20` × the mob's max health mB, capacity **5,000**;
draining it to 0 removes the block. Not carried in this slice (it needs a per-death
hook that tells us the mob's max health, which this pin only gives for player melee
kills — see the gap list).

## Slice chosen for this port

**The blood repair loop, end to end:** `blood_extractor` fills from a player kill →
its blood is emptied into a `blood_chest` tank by the chest's container slot → the
chest repairs the damaged items in its ten slots at 5 mB a damage point, 1 point
every 2 ticks, with the 25 mB anvil steps. Both ends of the loop are in the jar, both
are observable in game, and the numbers above are the jar's/upstream's own.

Carried content: `blood_chest` (block, item, blockstate, model, loot), `blood_extractor`
(item, model, textures), the chest's menu type, lang. Not carried: recipes (their
ingredients — `blood_infusion_core`, `dark_spike`, `dark_gem` — are not in this
slice), the BER chest model, the blood stains, the colossal chest, every other
machine.

> **Later slices (0.2.2–0.2.7)** carried the Blood Infuser, the tank on a broken machine's
> drop, projectile kills, the Blood Stain, the fall-death stain and the **Colossal Blood
> Chest**; `notes.md` is the current record of what is carried and what is still a gap.

## Deviations recorded while reading

- Blood is a **mod fluid id** (`evilcraft:blood`) in the chest tank; Horizon has no
  blood block or bucket ("the world itself has only vanilla's fluids"), so the tank
  is filled by the extractor, not by a bucket.
- The extractor's blood is kept in the stack's `minecraft:custom_data`
  (`evilcraft:blood`, an int): the pin's item fluid helpers only carry
  `minecraft:water`/`minecraft:lava` (API 147 rejects any other fluid id), so the
  item is a container of ours rather than a `Capabilities.FluidHandler.ITEM`.
- Upstream gives blood on any player kill whose damage source names the player
  (arrows included). The pin's `mob_killed` hook hears **melee, sweep and spear**
  kills only, so projectile kills give no blood here.
- The mob's max health is read before the hit (`living_incoming_damage_v2`) and
  remembered for the kill: at `mob_killed` the mob is already dead and the typed mob
  read returns nothing.
