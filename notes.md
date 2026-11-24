# evilcraft — EvilCraft: the blood loop (slice 1: the repair loop, slice 2: the Blood Infuser, slice 3: the tank in the dropped machine, slice 4: projectile kills, slice 5: the Blood Stain, slice 6: the fall-death stain, slice 7: the Colossal Blood Chest, slice 8: the Dark Tank, slice 9: the Purifier's curse action and the Blook, slice 10: review fixes, slice 11: the Vengeance Pickaxe's immunity, slice 12: the tanks' comparator signal, slice 13: the Blood Chest's curse roll, slice 14: the Undead Wood set, slice 15: the two undead-tree infusions; three of the purifier's four actions stay blocked)

> **Format (2026-11-18):** this port is now the flat **SDK package** (`api = "0.2"`,
> `[native]` at `0b14f8a`) built by `cargo horizon-mod build`. The `.hnmod`, `mixins/evilcraft/`,
> `content/` and `horizon-mixin.toml` described in the sections below are **retired** and were
> deleted; read them as history. The migration, the hook-by-hook mapping and the new ladder are
> in [§ Migration to the SDK format](#migration-to-the-sdk-format-2026-11-18) at the foot.

- **Mod:** EvilCraft (`evilcraft`), **content / deep**, core, priority **P0**.
- **Licence:** **CC BY 4.0** for all code and images — the exact release's own README
  (`1.21.1-1.2.100`) says so — with **MIT** as the grant its build metadata declares
  (`gradle.properties: license=MIT`, copied into the jar's `neoforge.mods.toml`). Resolved
  2026-11-18 (review #1168): the README is the narrower grant and it covers the two carried
  textures, so the port distributes under CC BY 4.0 with attribution, and keeps the MIT text
  as `LICENSE-MIT`. **Rank 0** either way. Local-only until the maintainer approves
  publishing (ARCHITECTURE §8).
- **Ported from:** EvilCraft 1.2.100, Minecraft Java 1.21.1 NeoForge, author
  rubensworks (kroeser). Jar kept as evidence:
  `ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`
  (SHA-1 `720215e3810d3733846d1083ee3c95a19f42153d`).
- **Approach (from `horizon/docs/modpacks.md`):** `deep` tier → block entities,
  fluids, and possibly an entity; take the smallest loop that is complete in itself.
  The jar's blood economy (extractor → blood → chest/infuser/purifier) is the loop;
  this slice is the two ends of it. Content rides inside the `.hnmod` (the AE2
  embedding pattern, ARCHITECTURE §3), so the port ships **one file**.
- **Pin:** Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, horizon-mixins
  `5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6). `compatibility.json` carries
  the pin and the `game.rs` hash (`64813cf4…f80f`).

## The slice

| Half | What | Source |
|---|---|---|
| Item | `evilcraft:blood_extractor` — a container for 5,000 mB of blood that fills when the player kills a mob | jar item + `ItemBloodExtractor`, `ItemBloodExtractorConfig` |
| Block | `evilcraft:blood_chest` — 10 repair slots + 1 factor slot, a 10,000 mB tank of blood, repairs 1 damage a point every 2 ticks for 5 mB, anvils in 125 mB steps, a 1/10,000 curse roll | jar block + `BlockEntityBloodChest`, `BlockBloodChestConfig`, `RepairItemTickAction`, `BloodChestRepairActionRegistry`, `AnvilRepairAction` |
| Menu | `evilcraft:blood_chest_menu` — 11 slots, 2 data slots (tank, capacity) | `ContainerBloodChest` (2×5 + 1) |

Every number is upstream's; `FINDINGS.md` names the file each came from. The one
that reads wrong at first glance: `AnvilRepairAction` returns **25F**, which
`RepairItemTickAction` *multiplies* by `mBPerDamage` (5), so an anvil step costs
**125 mB** — pinned by `a_chipped_anvil_becomes_an_anvil_for_one_hundred_twenty_five_mb`.

### The loop, in game

1. Kill a mob with the player's melee while a Blood Extractor is carried: the kill is
   worth `floor(hp × 5) + nextInt(max(1, floor(hp × 40) − floor(hp × 5)))` mB
   (a 20 HP zombie: 100–799 mB), which the carried extractor takes on its next
   inventory tick.
2. Put the extractor in the chest's container slot: the tick empties it into the tank.
3. Damaged tools in the ten chest slots lose a damage point every second tick,
   5 mB each; a chipped anvil becomes an anvil for 125 mB.

## Hooks used (all stamped in `native-hooks.json` at the pin)

*(Historical: these are the retired `.hnmod` injections. The SDK parts and hooks that replaced
each one are in [§ Migration to the SDK format](#migration-to-the-sdk-format-2026-11-18).)*

| Injection | Hook | API | What it does |
|---|---|---|---|
| `embedded-content` | `content_root` (raw target, AE2 pattern) | 41 | writes `data/evilcraft-content.hmod` to `.horizon-cache/` and appends it to the dev list |
| `block-entity` | `block_entity` v1 | 15 | gives `evilcraft:blood_chest` a block entity |
| `block-entity-ticks` | `block_entity_ticks` v1 | 15 | ticks it on the server |
| `block-entity-container` | `block_entity_container` v1 | 16/17 | 11 slots + the menu |
| `block-entity-fluids` | `block_entity_fluids` v1 | 54 | one 10,000 mB tank (`evilcraft:blood`), every face an output |
| `block-entity-tick` | `block_entity_tick` v1 | 15 | empties the container slot, repairs |
| `inventory-tick` | `inventory_tick` v1 | 39 | a carried extractor takes the kill's blood |
| `mob-killed` | `mob_killed` v1 | 40 | the kill's blood for that player |
| `screen` / `screen-draw` | `screen`, `screen_draw` v1 | 19 | the 11-slot layout and the tank readout |
| `tooltip` | `tooltip` v1 | 12 | "Blood: x/5,000 mB" on the extractor |
| `register-commands` / `perform-command` | `register_commands`, `perform_command` v1 | 8 | `/evilcraft info|blood|damage|held` (harness commands, upstream has none) |

## Upstream source read (analysis only, not copied)

<https://github.com/CyclopsMC/EvilCraft>, tag `1.21.1-1.2.99`, commit
`de23efdb83ea1b84addad6e3b1dc51a37147dc1a`. Files read:

- `src/main/java/org/cyclops/evilcraft/item/ItemBloodExtractorConfig.java`,
  `item/ItemBloodExtractor.java`
- `blockentity/BlockEntityBloodChest.java`, `blockentity/BlockEntityBloodChestConfig.java`
- `block/BlockBloodChestConfig.java`, `block/BlockBloodStain.java`,
  `block/BlockBloodStainConfig.java`, `blockentity/BlockEntityBloodStain.java`
- `blockentity/tickaction/bloodchest/{RepairItemTickAction,BulkRepairItemTickAction,BloodChestRepairActionRegistry,DamageableItemRepairAction,AnvilRepairAction}.java`
- `inventory/container/ContainerBloodChest.java`
- `core/blockentity/BlockEntityTankInventory.java`
- `client/render/blockentity/RenderBlockEntityBloodChest.java`
- `GeneralConfig.java` (the blood converters, unused here)

Bugs/quirks kept: the anvil step's 25× multiplier (see above); the blacklist is a
**regex** over the item id (`itemBlacklist = ["minecraft:stick"]`), which this slice
applies as an exact id; the extractor fills **every** extractor the player carries,
one at a time, until the amount is spent.

## Gaps (what this slice does not do)

1. **Recipes are not carried.** `blood_chest` needs `evilcraft:blood_infusion_core`
   (a crafted item, not an infusion) and `blood_extractor` needs `dark_spike` +
   `dark_gem`; none of those are in this slice, so both items are creative-only for
   now. Slice 2 adds the Blood Infuser that both hang off — see § **Slice 2**.
2. **The curse roll is not applied.** `randomBadEnchants = true`,
   `randomBadEnchantsChance = 10000` is recorded in `runtime.rs`
   (`BAD_ENCHANT_CHANCE`) but adding an enchantment component is not implemented.
3. **The BER chest model is not carried.** `RenderBlockEntityBloodChest` draws a
   chest model with `textures/model/blood_chest.png`; a content model cannot do a
   block-entity renderer, so the block is a cube over the mod's own
   `colossal_blood_chest` texture.
4. **Projectile kills give no blood.** Upstream hears any player kill whose damage
   source names the player (arrows included); the pin's `mob_killed` hook hears
   melee, sweep and spear only.
5. **`evilcraft:copy_tank_data`** (the loot function that carries the tank into the
   dropped block) has no Horizon equivalent; the block drops bare.
6. **The tank holds blood only by convention.** `block_entity_accepts_fluid` is not
   stamped at this pin, so another mod's pipe can push water into the tank; the
   chest then reads 0 mB and repairs nothing until blood is put back (no data loss:
   the foreign fluid is left untouched).
7. **No blood stain, colossal chest, purifier, dark ore, undead wood,
   spirits/vengeance, werewolves, promises/bowls, brooms, or the environmental
   accumulator** — the rest of the jar. `FINDINGS.md` lists the families. *(The Blood
   Infuser was in this list and is carried as of slice 2 — see § Slice 2 below; the
   promise items are carried as its upgrades only.)*
8. **`/evilcraft blood|damage|held` are harness commands**, not upstream features
   (upstream has no commands at all in this area); they exist so the scenario can
   read and set a chest without a GUI.

## Deviations

- **Item fluid storage.** The extractor keeps its blood in the stack's
  `minecraft:custom_data` (`evilcraft:blood`, an int) rather than as a
  `Capabilities.FluidHandler.ITEM`: the pin's item fluid helpers carry only water and
  lava (API 147). The stack still saves, syncs and stacks by contents.
- **Blood is a tank id** (`evilcraft:blood`), not a world fluid: Horizon has no blood
  block or bucket ("the world itself has only vanilla's fluids").
- **The max health is read from the game's mob list** (`game.mobs`) in `mob_killed`,
  because at that point the mob is at 0 health and every typed helper
  (`native_mob`) filters the living. Reading the field is compile-checked and loud if
  the pin moves.
- **The repair tick is one global counter**, so every eligible slot repairs on the
  same tick; upstream ticks ten independent components. The rate per slot is the same
  (one point per two ticks), and the e2e measures it.
- The block template is `minecraft:furnace` (a horizontal `facing`, a full cube) and
  the sound is `minecraft:oak_planks` (upstream `SoundType.WOOD`).

## Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Verification

See `VALIDATION.md` for the ladder, the artifacts and the three e2e legs; the logs
are under `ports/_shared/evidence/evilcraft/`.

## Reviewer repairs (2026-11-11) — PR review items and what changed


Each item below is quoted from `ports/_shared/horizon-mods/reviews/pr-*-evilcraft.md` (the review is the specification), with what this repair did about it.

- **Reviewer:** licence notice not in the bundle
  **Repair:** `tools/package.py` ships `LICENSE` + `LICENSE-NOTICE.md` in the `.hnmod`; the carried `.hmod` carries `data/evilcraft/license/`.
- **Reviewer:** relative upstream link
  **Repair:** Already absolute (`https://github.com/CyclopsMC/EvilCraft`).
- **Reviewer:** upstream permission unverified
  **Repair:** `LICENSE-NOTICE.md` now records that the 1.21.1 jar's `META-INF/neoforge.mods.toml` declares `license="MIT"`, that the exact release's README declares **CC BY 4.0 for all code and images**, and that the repository has no root licence file (hence GitHub's 404). Resolved again in the 2026-11-18 review-fix pass: the package distributes under CC BY 4.0, see [`## Review fixes (2026-11-18)`](#review-fixes-2026-11-18).
- **Reviewer:** registry artifact delivery
  **Repair:** **Staging (orchestrator, not ours)**: the artifact is not in registry release/staging as `<sha256>.<ext>`; the orchestrator answers it with `horizon-mods#686` (no write access to the registry's releases). Every repackaged artifact below has a fresh sha256 for that re-push, and the author release must be re-uploaded from the port repository.

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Migration to the SDK format (2026-11-18)

Horizon's mod-API merge (horizon-rust/horizon#16941, launcher ≥ 0.2.70) retired `.hnmod`, and the
registry closed **#665** for it. This port is now one flat `.hmod`: content, native code and the
manifest in one archive, built by `cargo horizon-mod build` against **nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, launcher 0.2.78) and **mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. Version **0.2.0** (the old PR carried 0.1.0; the form
refuses a duplicate while an older version's PR is open).

### What the package is now

```text
evilcraft.hmod                    130,338 B  sha256 a25beba6…7908f4
  horizon-mod.toml                the only manifest: id/name/version/api/side/capabilities + [native]
  Cargo.toml, Cargo.lock          depends on horizon-mod-api from .horizon/sdk/ (fetched, never packed)
  src/lib.rs                      the one #[mod_api::horizon_mod] impl Hooks
  src/runtime.rs                  9,629 B, byte-identical to the retired mixins/evilcraft/runtime.rs
                                  (sha256 5325b286…8c223a, unchanged before and after the ladder)
  src/blood.rs                    the two Parts, the kill, the commands, the tank/stack readers
  assets/evilcraft/  data/evilcraft/   the carried content, plus data/evilcraft/license/
  icon.png                        512x512, 99,194 B
  LICENSE, LICENSE-NOTICE.md, README.md, uses.json
```

### Files created, moved and deleted

| Action | Path |
|---|---|
| created | `horizon-mod.toml`, `src/lib.rs`, `src/blood.rs` |
| moved, unchanged | `mixins/evilcraft/runtime.rs` → `src/runtime.rs` (byte-identical, sha256 above) |
| moved | `content/assets/evilcraft/` → `assets/evilcraft/`, `content/data/evilcraft/` → `data/evilcraft/` |
| deleted | `evilcraft.hnmod`, `mixins/` (15 helper `.rs`, `horizon-mixin.toml`, `support.rs`), `content/` (sources + the built `content/evilcraft.hmod`), `compatibility.json`, `rust-toolchain.toml`, the standalone `Cargo.toml`/`Cargo.lock` |
| kept | `LICENSE`, `LICENSE-NOTICE.md`, `icon.png` (copied from the publish tree), `FINDINGS.md`, `PLAN.md`, `README.md`, `VALIDATION.md`, `tools/` |

### Manifest mapping

| retired | now |
|---|---|
| `horizon-mixin.toml` `[mod] id/version`, `[compatibility] horizon_revision` | `horizon-mod.toml`: `id`, `version`, `[native] horizon` (+ `game`, `mixins`) |
| `horizon-native.json` (one file per project) | gone; one `.hmod` **is** the mod, content included |
| `content/horizon-mod.toml` (a second manifest) | merged into the root manifest: `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]` |
| `[[module]] runtime` / `state` (injected as `crate::game::evilcraft*`) | plain Rust modules `src/runtime.rs` / `src/blood.rs` in the same crate |
| `[[injection]]` (13 of them) + one helper `.rs` per hook | `#[mod_api::horizon_part]` impls, `Hooks` methods and two `Registrar` declarations |
| the AE2 embedding (`content_root` raw target, `native_data!`, `.horizon-cache` write) | **gone**: `assets/` and `data/` sit in the same package as `src/`, so nothing is embedded, written or registered twice |
| `Cargo.toml` `[lib] path = "mixins/evilcraft/runtime.rs"` | crate root `src/lib.rs` + `horizon-mod-api` from `.horizon/sdk/` |

### Each old hook, and what replaced it

| retired helper (hook) | SDK replacement | notes |
|---|---|---|
| `embedded_content.rs` (`content_root`, raw target) | nothing | content and code are one package |
| `block_entity.rs` (`block_entity` v1) | `Registrar::block_entity_type("blood_chest").blocks(&["blood_chest"]).build()` + `BlockEntityPart::block_entity` → `None` | the declaration owns the type; `None` is the empty saved compound |
| `block_entity_ticks.rs` | `BlockEntityPart::block_entity_ticks` → `event.ticks = true` | |
| `block_entity_container.rs` (`native_block_entity_container`) | `Registrar::block_container(&block, ModContainer { slots: 11, menu: Some("evilcraft:blood_chest_menu"), .. })` | no face reaches a slot, as before |
| `block_entity_fluids.rs` (`native_block_entity_fluids`) | `Registrar::block_fluids(&block, ModFluidTanks)` | **the risk area**: the tank contents are still the saved `fluids` list — `Fluid` (string) + `Amount` (int) per tank, `minecraft:empty` for an empty one — read and written by the engine in `crates/horizon-client/src/game/blocks/machines/native_fluids.rs`; the declaration must match it (`0xFF…`-style key names unchanged, same shape). `FLUIDS` (`Capabilities.Fluid.BLOCK`) now reads those tanks, so a pipe can still drain them |
| `block_entity_tick.rs` (`block_entity_tick` v1) | `BlockEntityPart::block_entity_tick` with `event.entity` (`ModBlockEntityView` → `items`, `nbt`, `data`) | the entity is out of the world during the call; the reply stays `Changed` because the tick writes `ticks` |
| `inventory_tick.rs` (`native_inventory_tick`) | **folded into `mob_killed`** | see the gap below: `ItemPart::inventory_tick`'s event carries no player, so nothing can name the `ItemPlace::PlayerSlot` to write. Upstream fills at the kill anyway (`fillForAllBloodExtractors`), and `mob_killed` hands the player's uuid over, so the port fills there and the kill cache is gone |
| `mob_killed.rs` (`native_mob_killed`) | `Hooks::mob_killed`; `host.entity(id)` → `host.living(..)` → `living.max_health(host)`; the roll from `level.with_random(host, \|random\| random.next_int_bound(span))` | no more `game.mobs[..].attrs.max_health`; no more reading the game's mob list |
| `screen.rs` (`native_screen`) | `BlockEntityPart::screen` → `mod_api::Screen` | same 11-slot layout, same inventory origin |
| `screen_draw.rs` (`native_screen_draw`) | `BlockEntityPart::screen_draw` → `DrawContext::text(&text, at, argb, shadow)` | colour is now **ARGB** (`0xFF404040`), was RGB |
| `tooltip.rs` (`native_tooltip`) | `ItemPart::tooltip`, attached to `evilcraft:blood_extractor` | only that item hears it |
| `register_commands.rs` | `Hooks::register_commands` → `Vec<mod_api::Command>` (`Command::new` / `Command::op`) | |
| `perform_command.rs` (`native_perform_command`, raw line) | `Hooks::run_mod_command`, a **parsed** `CommandInvocation` (`literals`, typed `arguments`), `Flow::Cancel` when handled | feedback via `host.commands().feedback(Text::literal(..))`; `pos` arrives resolved as `BlockPos` |

### Host calls the glue now makes (what replaced the old game access)

| retired (`crate::game::Game`) | now |
|---|---|
| `game.native_with_mod_block_entity(pos, \|e\| ..)` | `host.block_entity(level, pos)` → `BlockEntityRef`; `entity.data(host)`, `entity.item_slot(host, slot)`, `entity.merge_data(host, patch)`, `entity.set_data_slots(host, v)`, `entity.set_item_slot(host, tx, slot, stack)` inside `host.transaction(..)` |
| `game.world.block_entities.get(&key)` | `host.block_entity(level, pos)` |
| `game.held()` / `held_slot()` / `stack_in_mut()` / `hand_stack_replaced()` | `player.selected_slot(host)`, `ItemPlace::PlayerSlot { player: player.uuid(), slot }`, `place.stack(host)`, `player.set_inventory_slot(host, slot, stack)` |
| `game.native_acting_id()` | the hook's own player: `event.player` (`mob_killed`), `CommandSource::Player { player, .. }` (`run_mod_command`) |
| `game.world.random.next_int_bound(n)` | `level.with_random(host, \|r\| r.next_int_bound(n))` (needs `RandomSource` in scope) |
| `game.world.game_time` | `level.game_time(host)` |
| `game.mobs[..].attrs.max_health` | `host.living(host.entity(id)).max_health(host)` |
| `stack.data()/data_mut()` + `tidy()` | `ItemStack::{get,set,remove}` with this port's own `Component` descriptor for `minecraft:custom_data`; `set_damage`/`clear_damage` for damage |
| `game.native_text(batch, ..)` | `ScreenDrawEvent.draw.text(..)` |
| `game.native_feedback(Text)` | `host.commands().feedback(Text)` |

The blood in the extractor is still an int under the stack's `minecraft:custom_data`
(`evilcraft:blood`), and still removed entirely when it reaches 0 so an emptied extractor stacks
with a plain one; the tank is still `nbt["fluids"]`.

### Deviations that changed in the migration

1. **The kill fills the extractors; the inventory tick no longer does.** Upstream does exactly this
   (`ItemBloodExtractor.fillForAllBloodExtractors(player, ..)` on `LivingDeathEvent`); the retired
   port had to park the blood in a side cache because the old hook could not write an inventory
   slot. The cache, the per-player pending map and the mod's only global state are gone, so a
   restart cannot lose a kill's blood either.
2. **`/evilcraft held` no longer prints "pending N mB, K kills"** — there is no cache to report.
   The read form prints the held blood; the operator form still sets it.
3. **A repaired stack loses its damage component entirely** (`clear_damage`) rather than writing 0,
   so it stacks with a fresh one, which is what the retired `tidy()` gave.

### New and remaining gaps

1. **API gap (new, worth reporting upstream): `ItemPart::inventory_tick` names no player.** Its
   event is `{ stack, slot, selected }`, and the write route it documents — `ItemPlace::PlayerSlot
   { player, slot }` — needs the player's uuid, which only `host.acting_player()` might supply and
   that hook is not listed among the ones that swap a player's state in. Issue shape: *"the player
   a `inventory_tick` stack belongs to, in the event or as `ItemPlace`"*. The port does not depend
   on it: it fills at the kill.
2. `ModFluidTanks` is `#[non_exhaustive]`: it cannot be built with a struct literal from a mod; fill
   a `Default` (recorded as a trap below).
3. The slice's content gaps are unchanged: recipes are not carried (both items are creative-only),
   the curse roll is not applied, the BER chest model is a cube, projectile kills still give no
   blood (the pin's `mob_killed` hears melee, sweep and spear), `evilcraft:copy_tank_data` has no
   equivalent, and blood is a tank id, not a world fluid.

### Ladder (all run in one `build-slot.sh` lease, log: `ports/_shared/evidence/evilcraft/sdk-migration-ladder.log`)

```sh
cd ports/evilcraft
cargo fmt --check                                        # clean
cargo clippy --all-targets -- -D warnings                # clean
cargo test                                               # 17 passed; 0 failed
cargo horizon-mod build                                  # Built ./target/horizon-mod/evilcraft.hmod
cp target/horizon-mod/evilcraft.hmod evilcraft.hmod      # the deliverable lives in the port, not target/
cargo horizon-mod check evilcraft.hmod                    # exit 0, 130,338 B, sha256 a25beba6…
cargo horizon-mod fits evilcraft.hmod                     # exit 0: "0b14f8a  made for"
cargo horizon-mod check                                   # project mode: cargo check (host + wasm32) + uses
```

Through the shared registry (the campaign's only supported entry point):

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-ec --port ports/evilcraft --step check -- \
    bash -lc 'cd ports/evilcraft && cargo horizon-mod check evilcraft.hmod'
```

`cargo horizon-mod check` prints *"native source: unsandboxed; not executed by the registry
check"*: the check validates the package, the manifest, the pins and the API-use list, and compiles
nothing. The compile is the separate `cargo check` (project mode) and `cargo test` above.

**No in-game run in this window.** `tools/e2e.sh` drives the retired ladder (`horizon.py stage →
build → ctl`) and `--dev-mod` loads `content/`, which no longer exists; the SDK package needs a game
built at `0b14f8a`, and the shared binaries on disk are the `1d63e0a` pair. The three e2e logs under
`ports/_shared/evidence/evilcraft/` are the **retired bundle's** evidence and stay as history — they
are not evidence about this package. The first in-game leg for this format is
`cargo horizon-mod run --game <a 0b14f8a game>`, which the wave must schedule once such a binary
exists.

### Traps this migration cost

1. **Run the tool with the port as the working directory, and force local.** `build-slot.sh run`
   forwards compile steps to the far box, which syncs the port into *its* root; `bash -lc 'cd
   ports/evilcraft && …'` then dies with "No such file or directory" there. `BUILD_LOCAL=1` (or a
   command without `cd`) is what worked.
2. **`cargo horizon-mod check <file>` and `fits <file>` resolve the path against the cwd**, so the
   same `cd` is needed; without it: `evilcraft.hmod: No such file or directory`.
3. **Host reads live on handles, not on the host.** `host.item_key_id(..)`, `host.player_level(..)`,
   `host.player_inventory_slots(..)`, `host.living_max_health(..)`, `host.level_game_time(..)` and
   `host.game_stack(..)` do not exist on `&dyn Host`/`&mut dyn ServerHost`; they are `key.id(host)`,
   `player.level(host)`, `player.inventory_slots(host)`, `living.max_health(host)`,
   `level.game_time(host)` and `place.stack(host)`. `rustc`'s *"there is a method `item_key` with a
   similar name"* is the tell.
4. **`ModFluidTanks` is `#[non_exhaustive]`** (unlike `ModContainer`): a struct literal is refused
   even with `..Default::default()`; assign fields on a `Default` value.
5. **`next_int_bound` needs `RandomSource` in scope**, or `rustc` offers `next_int`.
6. **A `BlockEntityPart` needs its `BlockPart` companion impl too**, each annotated with its own
   `#[mod_api::horizon_part]` (an empty `impl BlockPart for Chest {}` here). Registering the part on
   the block entity type is enough for the block-scoped hooks (`screen`, `screen_draw`): the
   installer expands the type's blocks onto the Part automatically.
7. **The artifact lands in `target/`**, which is disposable: copy it to the port root before
   recording (`evilcraft.hmod`), as the campaign's delivery rule says.
8. **The packer decides what travels.** It took `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
   `icon.png`, `assets/`, `data/`, `src/` and `uses.json`; the licence also rides as
   `data/evilcraft/license/` because `tools/stamp_license.py` now writes there.
9. `cargo fmt` in the port runs over `src/runtime.rs` too; it is already rustfmt-clean, so the
   byte-identical requirement survived (checked before and after: `5325b286…8c223a`).

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog for the resubmission (verbatim)

> Supersedes #665, closed because Horizon retired `.hnmod` (#17992). Changes: flat SDK package with
> `api = "0.2"` and `[native]` pinned to `0b14f8a`; hooks rewritten as `impl mod_api::Hooks`
> methods against typed hosts/events (the chest is a `BlockEntityPart`, the extractor an
> `ItemPart`, the kill and the commands global hooks; the tank and its slots are now
> `Registrar::block_fluids`/`block_container` declarations, and the AE2 content embedding is gone
> because one `.hmod` carries the content and the code); `runtime.rs` and its tests unchanged
> (17/17, byte-identical).

## Review fixes (2026-11-18) — registry source review #1168

Skills: superpowers/using-superpowers,systematic-debugging,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices

File: `var/review-all/1168-evilcraft.md`. Package **0.2.1**, `evilcraft.hmod` 142,922 B, sha256
`dc620c8b8de282fd8ee2edca590774059a5ffa53866477942035469ffd1cfb0c`.

1. **Licence: MIT-only omitted the upstream README's CC BY 4.0 grant for code and images.**
   Resolved from the artefacts: the exact tag `1.21.1-1.2.100`'s `README.md` says *"All code and
   images are licenced under Creative Commons 4 (CC BY 4.0)"*, its `gradle.properties` says
   `license=MIT`. The README is the narrower grant and it covers the two carried textures, so the
   port now distributes under **CC BY 4.0** (`LICENSE` = the full CC BY 4.0 legal code) and keeps
   the MIT text as `LICENSE-MIT`; `LICENSE-NOTICE.md` records both grants, the attribution
   (rubensworks), the source tag and the changes. `tools/stamp_license.py` writes the three files
   into `data/evilcraft/license/` so they travel inside the package. **Entry change is the
   orchestrator's**: `license = "CC-BY-4.0"`.
2. **`src/blood.rs:278` `/evilcraft info` read any block entity with saved data at any loaded
   position (slots, damage, tank), and `mod_entity` accepted another mod's container.**
   Fixed on both axes: `info` is declared `Command::op(…, 2)` (the level `/data get block` needs)
   and its body re-checks `may_read(permission)`; `mod_entity` now requires the block at `pos` to
   be `evilcraft:blood_chest` and answers *"is not a Block Chest"* otherwise. Pinned by
   `the_readout_needs_an_operator_and_names_only_this_chest` (unit) and by the in-game phase
   **A2** (a vanilla chest with a custom name is refused) and **G** (a de-opped player is refused,
   the console re-ops). `cargo test`: **18 passed**.

**Found by the in-game leg (fixed here, not in the reviewer's list).** The Blood Chest's tick
`take()` the container slot and only wrote it back when blood remained, so a **fully drained
extractor was destroyed** instead of staying in its slot (upstream's
`EmptyFluidContainerInTankTickAction` leaves the container). The slot now keeps the item — an
extractor rewritten empty, any other item untouched (its custom data is never touched) — and the
e2e's phase F asserts `container evilcraft:blood_extractor@0` next to `tank 760`. The e2e also
gained phase **A2** (a vanilla chest with saved data answers *is not a Block Chest*) and **G**
(the de-opped player is refused). In-game: `PASS` + `E2E EXIT:0` (`evidence/evilcraft-e2e/`), the
same command on the package-less Nightly `EXIT:1`.

## Slice 2 — the Blood Infuser (2026-11-21)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. The port moves from
`0b14f8a` to the campaign pin; `.horizon/sdk` was deleted and refetched (the tool prints `SDK 0a875da`).
Package **0.2.2** — the form/PR discipline is the orchestrator's; this file records what the bytes are.

### The gap this slice closes, and why this family

From the slice-1 gap list (unchanged § *Gaps*, item 7):

> No blood stain, colossal chest, **purifier, blood infuser**, dark ore, undead wood, spirits/vengeance,
> werewolves, promises/bowls, brooms, or the environmental accumulator — the rest of the jar.

The **Blood Infuser** is the jar's central blood machine: every other machine's inputs (the infusion
core, the purifier, the promise chain) flow through it, and it is the only family in the remaining list
that is *both* self-contained and provable in a game at this pin. It also closes the other half of gap
1 for later slices: once `blood_infusion_core` exists, `blood_chest` and `blood_extractor` become
craftable with the jar's own recipes (still not carried — see the gaps below).

**Tier:** `deep` still governs the port; this slice is a `machines` family inside it — a block entity
with slots and a tank, a content menu type, a custom recipe type read as data, and a tick. No entity,
renderer or dimension, so it does not exceed the tier.

### What is in it

| Half | What | Source |
|---|---|---|
| Block | `evilcraft:blood_infuser` — `facing`/`lit` from the `minecraft:furnace` template, `strength 2.5`, `SoundType.STONE`, `requiresCorrectToolForDrops` | jar block + `BlockBloodInfuser`, `BlockBloodInfuserConfig` |
| Block entity | 7 slots (`SLOT_CONTAINER=0`, `SLOT_INFUSE=1`, `SLOT_INFUSE_RESULT=2`, 4 upgrade slots), a 10,000 mB blood tank, the infusion tick | `BlockEntityBloodInfuser` (`SLOTS`, `SLOT_*`, `LIQUID_PER_SLOT`) |
| Menu | `evilcraft:blood_infuser_menu` — 7 slots, 3 data slots (tank, progress, duration) | `ContainerBloodInfuser` (3 machine slots + 4 upgrade slots) |
| Recipe type | `evilcraft:blood_infuser`, read through `Host::recipes_of_type` — 9 of the jar's 40 files | `RecipeBloodInfuser`, `RecipeSerializerBloodInfuser` |
| Tier upgrades | `evilcraft:promise_tier_1..3` (items, creative-only here) → the machine's tier | `ItemPromise`, `BlockEntityWorking.Metadata.getTier` |
| Commands | `/evilcraft infuser <pos>`, `/evilcraft load <pos> <slot> <id>`, and `/evilcraft blood` now sets either machine's tank | harness only (upstream has no commands) |

The tick is upstream's: `TickComponent` adds one to the action's tick, runs it, and resets at the
recipe's `duration`; `InfuseItemTickAction.canTick` refuses while the tank cannot pay the recipe's
`input_fluid.amount`, so a stalled machine loses its progress; the product joins the result slot (same
item, room for `output_item.count`), one input leaves the infusion slot and the blood is drained once
at completion. `RecipeBloodInfuser.matches` gates on `tier <= machine tier`, and the machine's tier is
the promise in an upgrade slot — so with no promise the machine does nothing, which phase I proves.

Carried recipes (the jar's `blood_infuser/convenience/` files whose ingredient and product are both
items the game or this package has): `bone→blaze_rod` 2,500 mB/500 t, `wheat_seeds→nether_wart`
5,000/1,000, `sand→soul_sand` 2,500/1,000, `dirt→grass_block` 350/300, `cobblestone→netherrack`
500/500, `rotten_flesh→spider_eye` ×8 375/500, `brown_mushroom→red_mushroom` 2,000/150,
`skeleton_skull→zombie_head` 1,000/500, `zombie_head→creeper_head` 5,000/1,000.

### TDD: the red proof, before the implementation

`sdk-test-red.log` (a run with the tests written and `infusion_for`/`infuse_tick`/`machine_tier`/
`decode_infusion` stubbed):

```
test result: FAILED. 20 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_bone_becomes_a_blaze_rod_for_two_and_a_half_thousand_mb stdout ----
thread 'runtime::tests::a_bone_becomes_a_blaze_rod_for_two_and_a_half_thousand_mb' (1320410) panicked at src/runtime.rs:357:64:
a bone is infused
---- runtime::tests::a_tier_three_recipe_needs_a_tier_three_machine stdout ----
thread 'runtime::tests::a_tier_three_recipe_needs_a_tier_three_machine' (1320423) panicked at src/runtime.rs:388:9:
assertion failed: infusion_for(&recipes, "minecraft:bone", 3).is_some()
---- runtime::tests::a_tier_promise_is_the_machines_tier stdout ----
thread 'runtime::tests::a_tier_promise_is_the_machines_tier' (1320422) panicked at src/runtime.rs:438:9:
assertion `left == right` failed
  left: 0
 right: 1
```

Green: `sdk-test.log` — **26 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (all in `build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 413 API uses, content accepted | `sdk-project-check.log` |
| tests (red) | `cargo test` | **FAILED 6/26** | `sdk-test-red.log` |
| tests (green) | `cargo test` | **26 passed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build` twice | **164,572 B**, sha256 `3c1dbf04…`, **`cmp` identical**, 61 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, **164,572 B**, sha256 `3c1dbf04…`, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** | report in the job |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`launcher-game.sh build evilcraft` → `native-0a875da-19366e9d0373`, with the log line
`Compiling evilcraft (1 of 1)`; the export's `horizon-native-mods/evilcraft/{src,data}` and
`horizon-mod.toml` are byte-identical to the port (`diff -r` empty, `cmp` empty) — the game ran the
sources this package is built from.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` | **PASS + `EXIT:0`** (phases A, A2, B–F, H–N) | `e2e-launcher.log` |
| control | `CONTROL=1 GAME=$CONTROL_GAME` | **FAIL + `EXIT:1`**: *without the package: no chest or infuser item, both blocks stayed `minecraft:air`* | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **FAIL + `EXIT:1`** at C (the pickaxe *was* repaired) | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=infuser` | **FAIL + `EXIT:1`** at K (the bone *was* infused) | `e2e-launcher-selftest-infuser.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=4 ok=4 missing-exit=0
bad-exit=0`, exit 0.

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== H: the Blood Infuser's own recipe files load, and an unknown id is refused ==
  Blood Infuser [206, 101, 0]: tank 0/10000 mB, tier 0, cont empty@0, in empty@0, out empty@0, progress 0, recipes 9/9
  block evilcraft:blood_infuser, 9/9 recipe files in force
  evilcraft:no_such_item is not an item this game has.
== I: a tier-3 recipe waits for a tier-3 machine (the tier gate) ==
  Blood Infuser [206, 101, 0]: tank 10000/10000 mB, tier 0, cont empty@0, in minecraft:bone@1, out empty@0, progress 0, recipes 9/9 t=…
  560 ticks at tier 0: the bone is untouched, no product, 10000 mB left
== J: the menu's seven slots, and the promise that tiers the machine ==
  Blood Infuser [206, 101, 0]: tank 10000/10000 mB, tier 3, cont empty@0, in minecraft:bone@1, out empty@0, progress 17, recipes 9/9 t=…
  the menu lists 7 slots; a Promise of Tenacity III in slot 303 makes the machine tier 3
== K: the bone is infused into a blaze rod for 2,500 mB in 500 ticks ==
  Blood Infuser [206, 101, 0]: tank 7500/10000 mB, tier 3, cont empty@0, in empty@0, out minecraft:blaze_rod@1, progress 0, recipes 9/9 t=…
  500 ticks: tank 10000 -> 7500 mB, one blaze rod for one bone
  took 1 x minecraft:blaze_rod out of the result slot into inventory slot 1
== L: every other carried recipe is crafted the way its file declares ==
  minecraft:rotten_flesh -> minecraft:spider_eye x8 for 375 mB in 500 ticks
  took 8 x minecraft:spider_eye out of the result slot into inventory slot 6
  … (all nine recipes assert product, count, consumed input and blood spent)
== M: the container slot empties a filled extractor into the tank ==
  extractor 1,000 mB -> tank 1000 mB, extractor 0 mB
== N: both machines break into themselves (loot tables and the mineable tag) ==
  evilcraft:blood_infuser: mined with a pickaxe, 1 on the ground, 0 picked up
  evilcraft:blood_chest: mined with a pickaxe, 1 on the ground, 0 picked up
```

Phase I is the infuser's own differential: the same machine, the same bone, the same full tank and 560
ticks produce nothing at tier 0 and one blaze rod for 2,500 mB once a tier-3 promise is in slot 303.
Phase N is a survival break (a creative player's break drops nothing at this pin — the first version of
the phase failed on exactly that), and it now proves **both** machines' loot tables and
`minecraft:mineable/pickaxe` membership.

### Coverage table (this package, family → proof)

| Family (jar) | Status | Proof |
|---|---|---|
| `blood_chest` block, item, menu, tank, repair tick | tested | A–D, F (slice 1, still green) |
| `blood_extractor` item, kill fill, tooltip | tested | E, F |
| `blood_infuser` block, menu (7 slots), tank, infusion tick | tested | H–M |
| `evilcraft:blood_infuser` recipe type (9 files) | tested | L (nine recipes, product + count + input consumed + blood spent) |
| tier gate (`RecipeBloodInfuser.matches` + `ItemPromise`) | tested | I (nothing at tier 0) vs J/K (infuses at tier 3) |
| `promise_tier_1..3` items as upgrades | tested (as upgrades) | J, K, L |
| loot tables + `minecraft:mineable/pickaxe` for both blocks | tested | N |
| harness commands `info`, `blood`, `damage`, `infuser`, `load`, `held` | tested | A2, B, D, F, H, I, M |
| `blood_stain`, `colossal_blood_chest`, `purifier`, `spirit_*`, `dark_tank`, `display_stand`, `environmental_accumulator`, `box_of_eternal_closure`, `entangled_chalice` | not ported | the rest of the jar's machinery (gap 7) |
| dark ore / gems, undead wood, vengeance/spirits, werewolves, brooms, misc items | not ported | gap 7; dark ore would need worldgen, inert at this pin |
| promise/bowl family (`bowl_of_promises_*`, `promise_acceptor_*`, speed/efficiency promises) | not ported | the promise items are carried as machine upgrades only; their crafting chain is a later slice |
| jar recipes for `blood_chest` / `blood_extractor` / `blood_infusion_core` | not ported | their ingredients (`hardened_blood_shard`, `dark_gem`, `dark_spike`, `dark_power_gem`) are not carried yet — gap 1 |
| 31 of the jar's 40 infuser recipes | not ported | their input or output is an evilcraft item this package does not carry; two are unreachable upstream (defects below) |
| `copy_tank_data` (the tank in the dropped block) | tested | O/P (6,500 mB broken, carried, re-placed; an empty chest carries nothing; a 20,000 mB payload clamps to 10,000) — slice 3 |
| projectile kills giving blood | **tested** | Q (slice 4: `living_death` credits the player's causing entity; a skeleton's arrow does not) |
| curse roll on repair | not ported | gap 2 (unchanged) |
| `experience` on infusion | not ported | upstream stores `xp` and drops it when the result is taken; a mod content menu has no slot-take hook to hang that on (deviation 1) |
| worldgen (dark ore) | blocked(configured-feature-not-registered) | mod worldgen is inert at `0a875da` (HANDOVER §9d 19) — not spent on |

### Deviations this slice adds

1. **No experience.** Upstream `BlockEntityBloodInfuser.xp` accumulates the recipe's `xp` and the
   result slot's `onTake` drops it at the player. The pin's content menu has no per-slot take callback
   (a `MenuPart` click hook may exist later), so the port neither stores nor awards it; no gameplay
   output is lost besides the points.
2. **No speed/efficiency promises.** The four upgrade slots exist and only the three tier promises are
   carried and read (the machine's tier). `promise_speed_*`/`promise_efficiency_*` are not carried, so
   their multipliers are not modelled.
3. **The tank is not resized by the tier.** Upstream multiplies the tank by the tier (`getTankTierMultiplier`,
   `1 << (tier * 2)`; the lang says ×2/×4/×8). The port keeps 10,000 mB for every tier; every carried
   recipe costs ≤ 5,000 mB, so nothing carried becomes unreachable.
4. **`lit` is never set.** The furnace template's `lit` state exists (its blockstate maps both values to
   the same model), but the tick does not flip it; upstream's `ON` property changes the model and the
   light level. Same cosmetic gap as the chest's.
5. **The tier is the highest promise anywhere in the upgrade slots.** Upstream refuses a second tier
   promise and gates which slot each tier may sit in (`getRequiredTierForSlot`); the port reads the
   highest of the four slots and does not police the index. One promise in any slot gives the same answer.
6. **The result slot is not `SlotRemoveOnly`.** A content menu cannot refuse placement per slot, so a
   player can put an item into the result slot; the machine will not overwrite it (the tick needs the
   slot empty or holding the product) and a pipe cannot reach it (`outputs = [container, result]`,
   `inputs = [container, infuse]`).
7. **Nine of forty infuser recipes.** The rest name items this package does not carry; each carried file
   is the jar's own bytes (only the directory flattened from `convenience/` to
   `recipe/blood_infuser/`), so a data pack can override or add them as upstream's `RecipeManager` allows.

### Upstream defects found (and what the port does instead)

1. **`rotten_flesh → leather` can never be made.** `convenience/minecraft_leather.json` (tier 1,
   375 mB) has the same input and the same cost as `convenience/minecraft_spider_eye.json` (tier 2,
   375 mB), and upstream's `getRecipe` takes the **highest tier** match
   (`BlockEntityBloodInfuser.recipeCache`: `max(comparingInt(getInputTier))`). The leather file is dead
   data upstream. The port does not carry it (`runtime::tests::a_rotten_flesh_makes_eight_spider_eyes`
   asserts the winner and the absence), and `runtime::infusion_for` documents the rule. Fixing it would
   mean changing a balance number (its tier or cost), which is not this campaign's call.
2. **`creeper_head → wither_skeleton_skull` cannot pay.** The file asks for **25,000 mB** of a
   **10,000 mB** tank (`BlockEntityBloodInfuser.LIQUID_PER_SLOT`), so `canTick` never passes and the
   recipe is unreachable without a tier upgrade (which multiplies the tank — deviation 3 does not).
   Not carried; `runtime::tests::every_carried_recipe_fits_its_tank_and_names_a_tier` pins that every
   carried recipe fits the tank. Worth an upstream report with defect 1.
3. **Not upstream — this port's own defect, found and fixed here:** both machines are
   `requires_tool: true` and the jar lists both in `minecraft:mineable/pickaxe`, but slice 1 never
   carried that tag, so **a survival break dropped nothing** (measured: a diamond pickaxe in hand, the
   block became air, `drops: []`). The tag now carries both blocks (`data/minecraft/tags/block/mineable/pickaxe.json`,
   the jar's own list) and phase N proves both drops. The chest's drop was untested before this slice.

### API notes this slice cost (worth an upstream issue/doc fix)

1. **A custom recipe type's serializer name is the recipe file's `type`.** `register_recipe_type("evilcraft:blood_infuser")`
   plus `register_raw_recipe_serializer(name, "evilcraft:blood_infuser")` with a *bare* `name` registers
   cleanly and then **crashes the game at data load**: `recipe …/minecraft_blaze_rod.json, unregistered
   recipe serializer evilcraft:blood_infuser`. Passing the type as the name
   (`register_raw_recipe_serializer("evilcraft:blood_infuser", "evilcraft:blood_infuser")`) is what works.
   The doc says "the name" without saying it is the file's `type`; the crash gives no hint.
2. **`Host::recipes_of_type` hands back each recipe's `json` already parsed** (`Arc<serde_json::Value>`),
   so a machine reads its own fields without a second parse — worth documenting for porters
   (`Recipe::json`), and the reason this port needs no JSON dependency of its own.
3. **`/evilcraft load` needs the `<id:id>` argument kind.** A `word` argument stops at `:`
   (`argument.entity...` style), and `string` is quoted-only, so an item id in a command needs `id`
   (delivered as `CommandArgumentValue::String`) — the campaign's `item`-argument trap applies to
   `fits`, this is the other side of it.

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Data-shape gate (`check_261_dialect.py`), and the one offender

```
$ python3 ports/tools/check_261_dialect.py ports/evilcraft/evilcraft.hmod
  FAIL evilcraft.hmod:data/evilcraft/loot_table/blocks/blood_chest.json: pools[0] spells "rolls": 1.0; 26.3 writes an integral roll as 1
data shape: 1 offender(s) in 11 recipe/loot file(s)
  ok   … the infuser's loot table, and all nine recipe files: bare item/tag strings
```

The one offender is slice 1's chest table, and `"rolls": 1.0` is one of the three shapes the checker's
own header calls **unverified** ("26.3 may well accept them"). It does: phase N breaks a fresh chest in
survival and gets its item (1 dropped). The three *fatal* shapes are clean — no `{"item": …}` ingredient
anywhere (the infuser's own `input_item` object is this mod's recipe type, which the checker reads as the
mod's own and accepts), no `"conditions": [ … ]` loot array, and both `requires_tool` blocks are in
`minecraft:mineable/pickaxe`. The value is left as the jar-verified bytes so the delivered package stays
byte-identical to the game build that produced the e2e evidence.

## Slice 3 — the tank in the dropped machine (2026-11-21)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.3**
(0.2.2 was the last build this package had before the slice; the registry entry and the PR are the
orchestrator's).

### The gap this slice closes, and why this family

Slice 1's gap list, item 5, unchanged in the slice-2 section:

> **`evilcraft:copy_tank_data`** (the loot function that carries the tank into the dropped block) has no
> Horizon equivalent; the block drops bare.

It is the jar's own behaviour for **both** machines: `data/evilcraft/loot_table/blocks/blood_chest.json`
and `…/blood_infuser.json` both run `minecraft:copy_name` then `evilcraft:copy_tank_data`
(`LootFunctionCopyTankData`). Upstream source read for this slice:
<https://raw.githubusercontent.com/CyclopsMC/EvilCraft/master/src/main/java/org/cyclops/evilcraft/loot/functions/LootFunctionCopyTankData.java>
(the `master` branch's file, fetched 2026-11-21 — a raw fetch, so no commit hash is pinned; the two loot
tables were read from the jar itself, `ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`).
Its `run` copies the block entity's tank — and only the tank — into the item's fluid handler, setting the
item tank's capacity first; nothing else from the block entity travels. Blood is the mod's currency, so a
machine that loses its whole tank to a pickaxe is the gap a player notices first.

**Tier:** `deep` still governs the port; this slice is a `machines`/`logic` family inside it — a drop
listener and an item Part on the two machines, no new block, machine, entity or renderer. It does not
exceed the tier.

### What is in it

| Half | What | Source |
|---|---|---|
| Drop listener | `Hooks::block_drops` on both machines' blocks: the block entity's saved data is read from the event (`level.block_entity_data`) and the tank written onto the machine's own drop as `minecraft:block_entity_data` | `LootFunctionCopyTankData`, the two loot tables |
| Placement | `ItemPart::on_block_placed_by` on both machines' items: the placed machine takes the tank its item carried, clamped to 10,000 mB | upstream sets the item tank's capacity before inserting (`IFluidHandlerCapacity.setTankCapacity`) |
| Rules | `runtime::carried_tank` (an empty tank travels on nothing; a negative amount is not a tank) and `runtime::restored_tank` (the clamp) | `if (!resource.isEmpty())`, the capacity |

Nothing else changed: the payload is the **tank alone**. The engine already spills a mod container's
stacks as their own item entities when the block is removed
(`crates/horizon-client/src/game/spill.rs`, `PreRemove::Contents`), so a payload that also carried
`Items` would put every stack in the world twice — once on the ground, once inside the block when it is
placed back. Phase O puts a cobblestone in slot 0 and asserts both halves of that: the stack spills
(`items_left_on_ground: 2`), and the payload has no `Items` key and the replaced chest's slots are empty.

### The item's shape

```text
evilcraft:blood_chest  components: {"minecraft:block_entity_data":
                                     {id: "evilcraft:blood_chest",
                                      fluids: [{Amount: 6500, Fluid: "evilcraft:blood"}]}}
```

The `id` is the block entity **type**'s registry id (the same string as the block), because
`TypedEntityData.loadInto` only loads a payload that names the block entity it is placed as. This is the
route the campaign already proved for a machine's buffers (`industrialforegoing` 0.2.2,
`rftoolsutility` 0.2.1); the difference here is the container half above.

### TDD: the red proof, before the implementation

`sdk-test-red-slice3.log` (the three tests written, `carried_tank`/`restored_tank` stubbed to
`None`/`0`):

```
test result: FAILED. 26 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_broken_machine_carries_its_tank_and_an_empty_one_carries_nothing stdout ----
thread '…' panicked at src/runtime.rs:391:9:
assertion `left == right` failed
  left: None
 right: Some(6500)

---- runtime::tests::a_carried_tank_comes_back_whole stdout ----
thread '…' panicked at src/runtime.rs:400:48:
a filled tank travels

---- runtime::tests::a_doctored_tank_is_clamped_to_the_capacity_when_it_is_placed stdout ----
thread '…' panicked at src/runtime.rs:407:9:
assertion `left == right` failed
  left: 0
 right: 10000
```

Green: `sdk-test.log` — **33 passed, 0 failed** (30 before the slice; the three new ones plus four in
`src/tank.rs`: the payload holds the id and the tank and nothing else, an empty machine carries nothing,
a doctored payload is clamped, and each machine names its own block). `sdk-clippy.log`:
`cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

The four `tank.rs` tests are the ones that would catch a regression of the two traps this slice had to
respect: a payload that grows an `Items` key (`a_broken_machine_carries_only_its_tank` asserts exactly two
keys) and a payload accepted past the capacity.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 3/29** (`EXIT:101`) | `sdk-test-red-slice3.log` |
| tests (green) | `cargo test` | **33 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 455 API uses, 14 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **170,472 B**, sha256 `fde8475d…3590`, `cmp` identical, 62 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | job report |
| artifact record | `sdk-artifact-0.2.3.txt` | bytes/sha/entries/manifest/runtime sha + check and fits lines | `sdk-artifact-0.2.3.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-732cfde441bf`, with the log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical to
the port (`diff -r`/`cmp` empty, `version = "0.2.3"`), so the game ran the sources this package is built
from.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` | **`PASS` + `EXIT:0`** (A, A2, B–F, H–P) | `e2e-launcher.log` |
| control (no package at all) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the control game has neither item and both blocks stay `minecraft:air`, then the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C (*the pickaxe was not repaired … damage 216*) | `e2e-launcher-selftest.log` |
| selftest (slice 2) | `SELFTEST=infuser` | **`FAIL` + `EXIT:1`** at K (*the bone was not infused …*) | `e2e-launcher-selftest-infuser.log` |
| selftest (this slice) | `SELFTEST=tank` | **`FAIL` + `EXIT:1`** at O (*the dropped chest carries none mB, expected 6500*) | `e2e-launcher-selftest-tank.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=5 ok=5 missing-exit=0 bad-exit=0`
(exit 0).

Transcript of the two new phases (verbatim from `e2e-launcher.log`):

```
== N: both machines break into themselves (loot tables and the mineable tag) ==
  evilcraft:blood_infuser: mined with a pickaxe, 1 dropped, 0 picked up
  evilcraft:blood_chest: mined with a pickaxe, 1 dropped, 0 picked up
== O: a broken machine's item carries its blood tank ==
  Blood Chest [202, 101, -4]: tank 6500/10000 mB, container empty@0, slots [0:minecraft:cobblestone@0], t=8080
  break: {"block":"minecraft:air","drops":{"evilcraft:blood_chest":1,"minecraft:cobblestone":1},"picked_up":{},"items_left_on_ground":2}
  the dropped evilcraft:blood_chest: Player59308 has the following entity data: {components: {"minecraft:block_entity_data": {id: "evilcraft:blood_chest", fluids: [{Amount: 6500, Fluid: "evilcraft:blood"}]}}, count: 1, id: "evilcraft:blood_chest"}
  the picked-up chest carries 6500 mB of evilcraft:blood in evilcraft:blood_chest, 0 stack(s)
  the cobblestone the chest held spilled on its own (2 item(s) left on the ground)
  Blood Chest [202, 101, -8]: tank 6500/10000 mB, container empty@0, slots [], t=8304
  placed again: tank 6500/10000 mB, slots empty
== P: an empty chest carries nothing, and a doctored payload is clamped ==
  Blood Chest [206, 101, -4]: tank 0/10000 mB, container empty@0, slots [], t=8352
  break: {"block":"minecraft:air","drops":{"evilcraft:blood_chest":1},"picked_up":{},"items_left_on_ground":1}
  the dropped evilcraft:blood_chest: Player59308 has the following entity data: {count: 1, id: "evilcraft:blood_chest"}
  an empty chest's item carries no block entity data: '{count: 1, id: "evilcraft:blood_chest"}'
  Blood Chest [206, 101, -8]: tank 0/10000 mB, container empty@0, slots [], t=8538
  placed again: tank 0/10000 mB
  the doctored chest: Player59308 has the following entity data: {components: {"minecraft:block_entity_data": {id: "evilcraft:blood_chest", fluids: [{Amount: 20000, Fluid: "evilcraft:blood"}]}}, count: 1, id: "evilcraft:blood_chest"}
  the doctored item asks for 20000 mB of evilcraft:blood
  Blood Chest [206, 101, -12]: tank 10000/10000 mB, container empty@0, slots [], t=8596
  206, 101, -12 has the following block data: 10000
  20,000 mB asked by the item, 10000 mB in the machine (saved 10000)
```

**Fail-capable, three ways.** (1) `SELFTEST=tank` inverts O's own assertion and exits 1 (log above).
(2) Phase P is the differential on the same block, the same survival break and the same block entity
path: a chest with **nothing** in its tank drops an item with no payload and places back at 0, so the tank
in O can only come from the listener reading the block entity. (3) The whole-package control
(`CONTROL=1 GAME=$CONTROL_GAME`) leaves both ids unknown. A *fourth* one — the same e2e against a package
built **without** the listener — was not produced: 0.2.2 (which had no `block_drops`) had already been
removed by the launcher's version cache, and rebuilding it costs a second game link; phase P is the same
item state a listener-less break leaves behind, and `SELFTEST=tank` is the assertion inverted.

### Harness facts learned here (record these before the next drop/listener slice pays for them again)

1. **`ctl entities`' `components` is the client's mirror and does not carry a mod component.** An item
   entity whose stack holds `minecraft:block_entity_data` reads `.item.components: null` there while the
   server holds it (measured: `/evilcraft probe` showed `applied=Ok(())` on a drop whose mirror was
   `null`). Read a held stack with `ctl command "data get entity @s SelectedItem"` and parse
   `… has the following entity data: {…}` — that is the server's own view and it shows the payload.
2. **`ctl break`'s reply is the server's drop count** (`{block, drops, picked_up, items_left_on_ground}`)
   and is a sturdier assertion than counting item entities on the client; `items_left_on_ground` is also
   the container-spill check. `break_into` and `break_survival` now use it.
3. **`data get block <pos> <path>` prints `<x>, <y>, <z> has the following block data: <value>`
   (`commands.data.block.query`), not the vanilla `… after scale factor of … is …` form** — that form is
   only used when a `scale` argument is given.
4. **A stale chat line is a real false-pass risk**: `SelectedItem` replies are read newest-first, so the
   helper waits for a line that differs from the previous one (`selected_item_line`), the same fix the
   infuser readout needed in slice 2.
5. **The launcher has one global build lock** (`.build.lock`): a `native-build` refuses with *"A native
   build or cache cleanup is running"* while another agent builds, so retry against `flock -n` rather
   than in a tight loop.

### Deviations this slice adds

1. **The payload is applied twice on a normal placement, harmlessly.** The engine applies the item's
   `minecraft:block_entity_data` to the new block entity before `on_block_placed_by` runs (the same
   behaviour `industrialforegoing` and `rftoolsutility` rely on), and `MachineItem` then writes the tank
   back only when the clamped value differs from what the machine holds — so a legal payload is written
   once (by the engine) and a doctored one is corrected by the Part. A machine with no payload is left
   exactly as the game placed it.
2. **A dispenser has no placer**, so the Part cannot reach the level to clamp; a dispenser-placed machine
   carries the payload unclamped until its own first tick, which writes the tank back through
   `write_tank` (clamped). No carried case is reachable through a dispenser (`block_entity_data` is only
   set by `/give`, creative pick-block or a drop).
3. **An explosion cannot carry the tank**: the event's block entity is gone for an explosion, exactly as
   upstream's `LootContextParams.BLOCK_ENTITY` is absent there, so a blown-up machine drops bare. This is
   upstream behaviour, not a gap.

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `copy_tank_data` on `blood_chest` and `blood_infuser` | **tested** | O (6,500 mB in, 6,500 on the item, 6,500 back after placing), P (empty → nothing; 20,000 → 10,000 in the readout *and* in the saved data) |
| the container's stacks on a broken machine | **tested (the opposite)** | O: the cobblestone in slot 0 spills as its own item entity, the payload has no `Items`, the replaced chest's slots are empty |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 4 — projectile kills give blood (2026-11-21)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.4**.
The registry entry, the PR and the ledger stay the orchestrator's.

### The gap this slice closes, and why this family

Slice 1's gap 4, unchanged since:

> **Projectile kills give no blood.** Upstream hears any player kill whose damage source names the
> player (arrows included); the pin's `mob_killed` hook hears melee, sweep and spear only.

Upstream's rule is `ItemBloodExtractor.bloodObtainEvent(LivingDeathEvent)`: if
`event.getSource().getEntity() instanceof ServerPlayer`, every extractor that player carries takes
`floor(hp × 5) .. floor(hp × 40)` mB. **`living_death` is that same event at this pin** — NeoForge's
`LivingDeathEvent`, with the source's causing/direct ids and the weapon — so the fill moved there.
One hook hears every cause of death, so a melee kill cannot be double-filled (phase E is the
regression guard).

**Tier:** `deep` still governs the port; this slice is a `logic` family inside it — one listener on an
existing death moment, no block, machine, entity or renderer.

### Why not `loot_drops` (the shape the brief named), measured

`loot_drops` **does** carry the credit (`damage_source.causing` resolves to the player for a bow
kill — a temporary marker proved the rule reached its last step), but **a player-inventory write made
there does not survive the call**: the hook's own re-read saw the written stack, and `/evilcraft held`
read 0 afterwards. The write is inside the loot roll; `living_death` runs before the loot, where the
same write lands. (It is also the event upstream listens to.) Evidence in the job's probe runs.

Two further harness facts this cost, both recorded below: the pin captures a death's killing blow
only for a mod listening on `living_death`/`loot_drops`, and a player **switched** from survival to
creative no longer keeps a death-hook inventory write.

### What is in it

| Half | What | Source |
|---|---|---|
| Listener | `Hooks::living_death` → `runtime::killer`/`Credit`: the source's **causing** entity resolved through `host.entity`/`host.players`, a player's blow or arrow credited, a mob's arrow/fall/`/kill` not | `ItemBloodExtractor.bloodObtainEvent`, `LivingDeathEvent` |
| Rule | `runtime::Credit::{Player, Nobody}` + `Credit::fills` (the only credit that fills) | same |
| Harness | `/evilcraft held` replies end in `t=<game time>`, so a readout can never equal the previous one | the infuser's readout had the same fault |

`runtime::blood_from_kill`/`fill_extractor`/`kill_span` are unchanged; the amount is still
`floor(hp × 5) + nextInt(max(1, floor(hp × 40) − floor(hp × 5)))` on the level's random.

### TDD: the red proof, before the implementation

`sdk-test-red-slice4.log` (the three credit tests written, `Credit::fills` stubbed to `false`):

```
test result: FAILED. 35 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_players_word_or_arrow_kill_fills_the_extractor stdout ----
thread 'runtime::tests::a_players_word_or_arrow_kill_fills_the_extractor' (545841) panicked at src/runtime.rs:449:9:
assertion failed: Credit::Player.fills()
```

Green: `sdk-test.log` — **36 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 1/36** (`EXIT:101`) | `sdk-test-red-slice4.log` |
| tests (green) | `cargo test` | **36 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 467 API uses, 14 hooks, `EXIT:0` | `sdk-build-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **172,019 B**, sha256 `a6632c3c…90b8`, `CMP-IDENTICAL`, 62 entries, no `Cargo.lock` | `sdk-build-check.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-build-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | job report |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-cc7e9092a372`/`…155194bd99f7`, with
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical
to the port (`diff -r`/`cmp` empty, `version = "0.2.4"`), so the game ran the sources this package is
built from.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1 OLD_GAME=<0.2.3 export>`) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–R) | `e2e-launcher.log` |
| control (no package at all) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C (*the pickaxe was not repaired … damage 218*) | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=blood` | **`FAIL` + `EXIT:1`** at Q (*a bow kill filled 0 mB, expected at least 100*) | `e2e-launcher-selftest-blood.log` |
| selftest (slice 2, 0.2.3) | `SELFTEST=infuser` | **`FAIL` + `EXIT:1`** at K — kept as history | `e2e-launcher-selftest-infuser.log` |
| selftest (slice 3, 0.2.3) | `SELFTEST=tank` | **`FAIL` + `EXIT:1`** at O — kept as history | `e2e-launcher-selftest-tank.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=6 ok=6 missing-exit=0
bad-exit=0` (exit 0).

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== Q: a bow kill fills the carried extractor (every cause of death, the player's) ==
  the carried bow killed the zombie at 206.5,101,10.5
  Blood Extractor held: 541/5000 mB, t=8779.
  a 20 HP zombie killed by an arrow filled the carried extractor with 541 mB
== R: a skeleton's arrow kills the same mob and fills nothing (the kill's differential) ==
  Arrow has the following entity data: {NoGravity: 1b, Motion: [0.0d, 0.0d, 0.0d], damage: 100.0d, Owner: [I; -1676472134, 1616924017, -1795264654, 1366240336], …}
  Blood Extractor held: 0/5000 mB, t=8912.
  the skeleton's arrow killed the zombie and the extractor stayed at 0 mB
```

and the previous package's leg, first in the same run (`== the previous package (0.2.3): the same bow
kill fills nothing ==`): `Blood Extractor held: 0/5000 mB.` after the same bow kill.

**Fail-capable, three ways.** (1) `SELFTEST=blood` inverts Q's assertion and exits 1. (2) Phase R is
the rule's differential on the same extractor and the same mob: a lethal arrow whose `Owner` is read
back as the skeleton's UUID kills the zombie and leaves the extractor at 0, so only a *player's*
causing entity fills. (3) The whole-package control (`CONTROL=1`) fails at phase A's first check, and
the **previous package** (0.2.3, the launcher's export kept as `evilcraft-0.2.3`) runs the same bow
kill in the same run and reads 0 — the gap this slice closes.

### Harness facts learned here

1. **`loot_drops` cannot write a player's inventory at this pin.** A fill made there is visible to a
   re-read inside the hook and gone by the next tick. `living_death` (before the loot) lands.
2. **The pin only captures a mob's killing blow for a mod listening on `living_death`/`loot_drops`.**
   A port that registers only a later hook sees deaths with no source (the arena's `/kill` deaths
   still reach them, with a generic source and no player).
3. **A player switched from survival to creative stops keeping a death hook's inventory write.**
   Measured: the same bow kill filled 266 mB (creative from launch), 598 mB (survival), and 0 after a
   `gamemode survival` → `gamemode creative` switch. The kill phases therefore run in survival, where
   arrows are consumed and 32 are given (`arm_bow`), with Resistance 5 so the AI mob cannot kill the
   player mid-phase.
4. **A mob needs AI for projectile damage.** `NoAI:1b` freezes it so completely that an arrow does
   nothing (20 HP stays 20); melee still lands. Phase Q's target is an AI zombie; phase E's (melee)
   stays frozen.
5. **A skeleton's own arrow can be summoned**: `summon minecraft:arrow … {NoGravity:1b,Owner:<skeleton
   UUID int-array>,damage:100.0}`; `data get entity` reads `Owner` back for the assertion, then
   `data merge entity … {Motion:[-2.0,0,0]}` sends it. A frozen (`NoAI`) zombie takes no projectile
   damage, so the target zombie is AI.
6. **`/evilcraft held` now ends in `t=<game time>`**: the harness reads the newest chat line, and a
   readout that can repeat its exact text made a fixed wait read the previous one (the infuser's
   readout had the same fault in slice 2).

### Deviations this slice adds

1. **The fill is on `living_death`, not `loot_drops`** (see above). The credit rule is unchanged:
   upstream's causing entity. A tame wolf's kill credits the wolf (not its owner), as upstream does;
   a player killed by a player fills the killer's extractor, also as upstream's event would.
2. **The random is the level's, not the player's** (`player.getRandom()` upstream) — unchanged from
   slice 1, recorded there.
3. **No Vein Sword boost.** Upstream multiplies the span by `ItemVeinSwordConfig.extractionBoost` when
   the killing player holds an `ItemVeinSword`; the port does not carry that item, so no boost.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe and the 25,000 mB recipe are the
earlier slices'.)

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| a **projectile** kill by the player (arrow) fills the carried extractor | **tested** | Q: a 20 HP zombie at range, 32 arrows, 541 mB (100..799 asserted) |
| a **mob's** projectile kill fills nothing | **tested** | R: a skeleton's arrow (its `Owner` read back) kills the zombie, extractor 0 |
| the previous package (0.2.3) on the same bow kill | **tested (the gap)** | the run's first leg: 0 mB |
| melee kills still fill (the regression guard) | **tested** | E: 100..799 (unchanged phase) |
| `/kill` and the environment fill nothing | **tested** | R's `/kill` cleanup + phase C's empty-tank control; unit `the_environment_and_kill_command_fill_nothing` |
| projectile kills giving blood | **tested** | Q (was `not ported`) |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 5 — the Blood Stain (`BlockBloodStain`, `BlockEntityBloodStain`) (2026-11-22)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.5**.
The registry entry, the PR and the ledger stay the orchestrator's.

### The gap this slice closes, and why this family

Slice 1's gap 7, unchanged since:

> No **blood stain**, colossal chest, purifier, blood infuser, dark ore, undead wood,
> spirits/vengeance, werewolves, promises/bowls, brooms, or the environmental accumulator —
> the rest of the jar.

`FINDINGS.md` (read from `BlockBloodStain`, `BlockEntityBloodStain`, `ItemBloodExtractor.onItemUseFirst`)
states the family precisely: a mob's **fall death** leaves a stain holding `bloodMBPerHP = 20` × its max
health mB in a **5,000 mB** tank, and a **crouching right-click with a Blood Extractor** drains it into the
extractor; a stain a committed change drains to 0 removes its block. It is the source end of the blood
economy the port already ships (extractor → infuser/chest), it is self-contained, and it is provable in a
game with one click — so it is the highest-value family left that fits the tier.

**Tier:** `deep` still governs the port; this slice is a `logic`/`machines` family inside it — one block
with a block entity and a small tank, and one item-use hook. No new machine, entity, renderer or
dimension.

### What is in it

| Half | What | Source |
|---|---|---|
| Block | `evilcraft:blood_stain` — the `minecraft:white_carpet` template (a 1-pixel shape on the floor, sturdy support below), `hardness 0.5`, `explosion_resistance 0.5`, `SoundType.STONE`, its item | jar block + `BlockBloodStain`, `BlockBloodStainConfig` |
| Tank | one **5,000 mB** tank in the block entity's saved `fluids` list, every face reaching it, **no input** (a pipe can drain, never fill), exactly upstream's `BlockEntityBloodStain.FluidHandler` (`insert` answers 0) | `BlockEntityBloodStain.CAPACITY`, its `FluidHandler` |
| Click | `ItemPart::use_on` on `evilcraft:blood_extractor`: a **crouching** right-click on a stain moves `min(stain, extractor room)` mB into the item; standing, another block, a full extractor and an empty stain all answer `Pass` | `ItemBloodExtractor.onItemUseFirst` |
| Removal | a click that empties the stain removes its block (`LevelRef::remove_block`) | `BlockEntityBloodStain.setAmount(0, true)` |
| Data | the jar's `blockstates/blood_stain.json` (12 weighted, rotated variants), `models/block/blood_stain_0..2.json`, `textures/block/blood_stain_0..2.png`, `textures/item/blood_stain.png`, its item model (the jar's parent is `cyclopscore:item/flat`, which this package does not carry), the lang keys, the creative-tab row, and the jar's `data/minecraft/tags/block/replaceable.json` | the jar's own files |
| Command | `/evilcraft stain <pos>` (the tank readout) and `/evilcraft blood <pos> <amount>` now reaching the stain, clamped to its own 5,000 mB | harness only (upstream has none) |

The rules are [`crate::runtime::stain_draw`] (`StainDraw { stain, extractor, moved }` and
`drains_the_stain`), `STAIN`/`STAIN_TANK`; the game side is [`crate::stain`]; the capacity-aware tank
readers are `blood::read_tank_with`/`write_tank_with` (the chest and infuser keep their 10,000 mB ones).

### TDD: the red proof, before the implementation

`sdk-test-red-slice5.log` (the four tests written, `stain_draw` stubbed to a zero move):

```
test result: FAILED. 36 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_crouching_click_moves_only_the_room_the_extractor_has stdout ----
thread '…' panicked at src/runtime.rs:512:9:
assertion `left == right` failed: the extractor had room for 1,000 mB
  left: 0
 right: 1000

---- runtime::tests::a_stain_with_less_than_the_room_empties_into_the_extractor stdout ----
thread '…' panicked at src/runtime.rs:521:9:
assertion `left == right` failed
  left: (0, 0, 0)
 right: (1000, 0, 1000)

---- runtime::tests::a_full_extractor_takes_nothing_and_leaves_the_stain_alone stdout ----
thread '…' panicked at src/runtime.rs:529:9:
assertion `left == right` failed
  left: 0
 right: 3500

---- runtime::tests::a_stain_never_holds_more_than_five_buckets stdout ----
thread '…' panicked at src/runtime.rs:539:9:
assertion `left == right` failed: a doctored stain gives at most its own tank
  left: (0, 0, 0)
 right: (5000, 0, 5000)
```

Green: `sdk-test.log` — **40 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 4/40** (`EXIT:101`) | `sdk-test-red-slice5.log` |
| tests (green) | `cargo test` | **40 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 494 API uses, 16 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **182,285 B**, sha256 `7a1ead32…0e69bc`, `CMP-IDENTICAL`, 75 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | unchanged: slice 1's one `"rolls": 1.0`; the new block file and tag are not recipes/loot | job report |
| artifact record | `sdk-artifact-0.2.5.txt` | bytes/sha/entries/manifest/runtime sha, check and fits lines, the packed `data/` list | `sdk-artifact-0.2.5.txt` |

The two `fits` runs name the **full** commit: with `--build 0a875da` the tool first asks GitHub for the
release list, and that was rate-limited (`403`) and connection-reset twice on 2026-11-22, while the full
sha is answered from `.horizon/cache/<sha>/` and prints the same `0a875da  made for`.

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-f476825c075c`, with the log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical to
the port (`diff -r`/`cmp` empty, `version = "0.2.5"`), so the game ran the sources this package is built
from.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–U) | `e2e-launcher.log` |
| control (no package at all) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C (*the pickaxe was not repaired … damage 218*) | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=stain` | **`FAIL` + `EXIT:1`** at S (*the crouching click moved 0 mB into the extractor, expected exactly the 1000 mB room it had*) | `e2e-launcher-selftest-stain.log` |
| selftest (slice 2, 0.2.3) | `SELFTEST=infuser` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-infuser.log` |
| selftest (slice 3) | `SELFTEST=tank` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-tank.log` |
| selftest (slice 4) | `SELFTEST=blood` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-blood.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=7 ok=7 missing-exit=0 bad-exit=0`
(exit 0).

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== S: a crouching click with the extractor draws a stain's blood into it ==
  placed evilcraft:blood_stain at 204,101,-10
  Blood Stain [204, 101, -10]: tank 0/5000 mB, t=8972
  Blood Stain [204, 101, -10]: tank 3500/5000 mB, t=8990
  the stain's saved tank: 204, 101, -10 has the following block data: [{Amount: 3500, Fluid: "evilcraft:blood"}]
  Blood Extractor held: 4000/5000 mB, t=9030.
  crouch + right-click on 204,101,-10 (aim on evilcraft:blood_stain)
  Blood Extractor held: 5000/5000 mB, t=9070.
  Blood Stain [204, 101, -10]: tank 2500/5000 mB, t=9077
  extractor 4000 -> 5000 mB, stain 3500 -> 2500 mB (moved exactly 1000)
== T: a standing click moves nothing; a crouching click on stone moves nothing ==
  right-click on 204,101,-10 (aim on evilcraft:blood_stain)
  Blood Extractor held: 4000/5000 mB, t=9123.
  Blood Stain [204, 101, -10]: tank 3500/5000 mB, t=9129
  standing: extractor 4000 mB, stain 3500 mB (nothing moved)
  crouch + right-click on 204,101,-10 (aim on evilcraft:blood_stain)
  Blood Extractor held: 5000/5000 mB, t=9161.
  Blood Stain [204, 101, -10]: tank 2500/5000 mB, t=9167
  crouching: extractor 4000 -> 5000 mB, stain 3500 -> 2500 mB (the same click, sneaking)
  crouch + right-click on 204,101,-8 (aim on minecraft:stone)
  Blood Extractor held: 4000/5000 mB, t=9226.
  the same crouching click on minecraft:stone: extractor still 4000 mB
== U: a drained stain removes its block, and vanilla may replace it ==
  Blood Stain [204, 101, -10]: tank 400/5000 mB, t=9256
  crouch + right-click on 204,101,-10 (aim on evilcraft:blood_stain)
  Blood Extractor held: 5000/5000 mB, t=9289.
  a stain drained to 0 is gone (minecraft:air), and the extractor took its 400 mB (5000 mB)
  placed evilcraft:blood_stain at 204,101,-12
  right-click on 204,101,-12 (aim on evilcraft:blood_stain)
  a stone placed onto a stain landed above it (evilcraft:blood_stain stays, minecraft:stone above): minecraft:replaceable is inert at this pin
```

**Fail-capable, four ways.** (1) `SELFTEST=stain` inverts S's own assertion and exits 1 (log above).
(2) Phase T is the crouch rule's differential on the same stain and the same extractor: standing moves
nothing, and the *same* click crouching moves 1,000. (3) Phase T's second half is the block rule's
differential: the same crouching click on `minecraft:stone` moves nothing. (4) The whole-package control
leaves every id unknown, so S cannot even place the stain.

### Harness facts learned here (record these before the next thin-block slice pays for them again)

1. **`ctl look-at-block` cannot aim at a one-pixel block, and neither can `ctl interact`.** Both aim at
   the *cell's* centre (`pos + 0.5`), which for a stain is above its shape (`y .. y+0.0625`), so the ray
   sails over it: `no face of block [204, 101, -10] is visible from here`. A stain is aimed at with
   `ctl look-at <x+0.5> <y+0.02> <z+0.5>` (a point inside the shape) and clicked with `ctl mouse right
   click`; `ctl looking-at` then proves the crosshair is on `evilcraft:blood_stain`.
2. **A cell's centre is `+0.5` in *every* axis — not string concatenation.** `"$z.5"` for `z=-10` is
   `-10.5`, which is the centre of cell **-11**: the first aim was half a block off in z, read
   `minecraft:stone`, and cost a run. The helper computes the centre with `awk`.
3. **`ctl place` reports the block two ticks after its click**, so a click the client dropped (or the
   server applied a tick late) reads as *the block did not change*; the stain's placement helper re-reads
   after 20 ticks and only then falls back to a plain right-click on the floor, printing what it saw.
4. **`minecraft:replaceable` is not read at this pin.** It is a per-state block fact in the game's
   `facts` export (a mod block inherits its *template's* flag), and `BlockProperties` in
   `horizon-mods/src/content.rs` has no `replaceable` key, so a content block cannot set it. Measured: a
   stone placed onto a stain lands **above** it (phase U). The jar's `data/minecraft/tags/block/replaceable.json`
   is carried (it is upstream's own data) and is **inert** here; the gap is
   `blocked(replaceable-not-expressible)` for that one behaviour.
5. **A busy server can answer `/evilcraft held` five seconds late** (the chat is read newest-first, so a
   short budget reads the previous readout). The helper's budget is now 12 s of ticks; measured on
   `SELFTEST=stain`'s first attempt, where phase R timed out on a reply that arrived a moment later.
6. **An enderman in the cell blocks a placement, and the End spawns them.** `ctl place`'s
   *the block did not change (blocked by an entity, or the item cannot be placed)* was literal: a dump
   showed five endermen around the player, and the fallback click failed the same way. The stain's
   placement helper kills non-player entities within 8 blocks of the player first, as `arena` does at the
   start of the run; a mob's own phase must therefore not expect a mob it summoned earlier.

### Deviations this slice adds

1. **The stain is the `minecraft:white_carpet` template.** Its shape is vanilla's carpet (a 1-pixel box
   across the cell), which is upstream's `Block.box(0, 0, 0, 16, 1, 16)` exactly, but its **collision**
   is the carpet's 1-pixel box where upstream is `noCollision()`. The model, the blockstate and the four
   textures are the jar's own, so it draws as the jar's flat splash.
2. **No fall-death spawn yet.** Upstream's listener places (or tops up) a stain when a mob dies from
   **fall damage**, `bloodMBPerHP = 20` × its max health, capped at the tank. This slice is the block, its
   tank and the click; the spawn is the next slice — **closed by slice 6, below** — and the surface it
   needs is there: `living_death`'s
   `event.source` is a `HurtSource` with `damage_type` (`"minecraft:fall"`), `position` and the victim id,
   so the rule, the placement and its differential are expressible without a new hook. The e2e sets the
   tank with the harness command in the meantime.
3. **No particles.** Upstream spawns blood splashes on the click, on an attack and when an entity walks
   through; a content block has no client-side particle path in this port (same class as the chest's BER).
4. **No hopper support.** Upstream also lets a stain rest on a hopper (`blockstate.getBlock() == HOPPER`);
   the carpet template requires a face-sturdy block below, so a stain on a hopper cannot survive.
5. **The bloom of rotations is the jar's blockstate**, 12 weighted variants with `y` 0/90/180/270; the pin
   supports weighted variants, so each stain picks one at random as upstream does.
6. **The tank is not resized and holds blood only by convention**, exactly as the two machines'
   (gap 6 of slice 1): `block_entity_accepts_fluid` is not stamped, so another mod's pipe can push a
   foreign fluid in; the stain then reads 0 mB and the click moves nothing until blood is written back.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe, the 25,000 mB recipe and the missing
pickaxe tag are the earlier slices'.)

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `blood_stain` block, its item, blockstate, three models, four textures, lang, creative row | **tested** | S: placed from its own item, read back at `204,101,-10`, `0/5000 mB` |
| the stain's 5,000 mB tank in the saved `fluids` list | **tested** | S: `/evilcraft blood` sets 3,500 and `/data get block … fluids` reads `[{Amount: 3500, Fluid: "evilcraft:blood"}]` |
| the crouching click moves exactly the extractor's room | **tested** | S: 3,500 → 2,500 and 4,000 → 5,000, exactly 1,000 |
| a **standing** click moves nothing (the rule) | **tested** | T: extractor 4,000, stain 3,500 unchanged |
| a crouching click on a **vanilla** block moves nothing | **tested** | T: `minecraft:stone`, extractor 4,000 unchanged |
| a drained stain removes its block | **tested** | U: 400 mB out, cell `minecraft:air` |
| the jar's `minecraft:replaceable` entry | blocked(replaceable-not-expressible) | U measures it inert: a stone placed onto a stain lands above it |
| fall-death spawning (`bloodMBPerHP = 20`) | **tested** | slice 6: V (a 20 HP zombie's fall → `400/5000`), X (a second fall → `800/5000`, no second block), W (a sword kill leaves the cell air) |
| click/attack/walk-through particles | not ported | deviation 3 |
| `blood_stain` on a hopper | not ported | deviation 4 (template support rule) |
| `/evilcraft stain`, and `/evilcraft blood` reaching the stain | **tested** | S, T, U (every readout above) |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 6 — a fall death leaves a Blood Stain (`BlockBloodStain.bloodStainedBlockEvent`) (2026-11-22)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.6**.
The registry entry, the PR and the ledger stay the orchestrator's.

### The gap this slice closes, and why this family

Slice 5's deviation 2, and its coverage row, named it exactly:

> **No fall-death spawn yet.** Upstream's listener places (or tops up) a stain when a mob dies from
> **fall damage**, `bloodMBPerHP = 20` × its max health, capped at the tank.

It is the source end of the stain slice 5 already ships (the block, its five-bucket tank and the
extractor's crouching click), it is self-contained, and the assertion is one number a scenario can
read: `/evilcraft stain <pos>`.

**Tier:** `deep` still governs the port; this slice is a `logic` family inside it — one listener on the
death moment the port already hears (`living_death`, slice 4), a level write and a tank merge. No new
block, machine, entity or renderer.

### Upstream source read (analysis only, not copied)

<https://github.com/CyclopsMC/EvilCraft>, `master` checkout at `ports/_shared/upstream/evilcraft`
(commit `aa81220`, read 2026-11-22):

- `block/BlockBloodStain.java` — `bloodStainedBlockEvent(LivingDeathEvent)`, the whole rule;
- `block/BlockBloodStainConfig.java` — `bloodMBPerHP = 20`, `spawnBlacklist = ["tconstruct:.*"]`,
  `spawnOnBlockEntities = false`, `noCollission()`, `strength(0.5F)`;
- `blockentity/BlockEntityBloodStain.java` — `CAPACITY = 5000`, `addAmount` clamps to it and removes
  the block at 0.

The rule as the file states it: when the death's source **is** the level's fall damage source (and the
mob is not a vengeance spirit), take `floor(entity x/y/z)`, read the block at `y-1`; if that is a stain
add to *it*, otherwise move to the mob's own cell and place a stain there when it is empty and can
survive; then add `20 × maxHealth` mB to the block entity at that cell. Upstream defers the write one
tick (`event.getEntity().getServer().execute(..)`) because Forge's fall handler caches the block it
landed on.

### What is in it

| Half | What | Source |
|---|---|---|
| Listener | `stain::on_death` from `Hooks::living_death`: only `event.source.damage_type == "minecraft:fall"`; the victim's own `EntityRef::block_position`, its max health through the same `blood::max_health` the extractor fill uses | `BlockBloodStain.bloodStainedBlockEvent` |
| Rule | `runtime::stain_blood` (`20 × hp`, capped at the tank), `runtime::fall_stain(victim, hp, below, own)` with `Cell::{Free, Stain(tank), Taken}` and `FallStain { cell, place, tank }` | the same file's order: below first, then the mob's own cell |
| Placement | `LevelRef::set_block` with the stain's own default state (`host.block_state_id(STAIN)`, `NEIGHBORS \| CLIENTS`), then the tank merged into the new block entity's saved `fluids` | `setBlockAndUpdate(pos, defaultBlockState())` + `addAmount` |
| Support | `room_for_stain`: the cell is `minecraft:air`/`cave_air`/`void_air` **and** `block_face_sturdy(below, Up, Center)` — the carpet template's own `canSurvive` | `isEmptyBlock` + `canSurvive` |

Nothing else changed: the extractor fill, the tank's serialisation, the click and the commands are the
earlier slices'. `/evilcraft stain` (slice 5) is the readout this slice's assertions use.

### TDD: the red proof, before the implementation

`sdk-test-red-slice6.log` (the five tests written, `stain_blood`/`fall_stain` stubbed to `0`/`None`):

```
test result: FAILED. 41 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_fall_never_fills_a_stain_past_its_five_buckets stdout ----
thread 'runtime::tests::a_fall_never_fills_a_stain_past_its_five_buckets' (196483) panicked at src/runtime.rs:690:9:
assertion `left == right` failed: an 8,000 mB fall fits 5,000
  left: 0
 right: 5000

---- runtime::tests::a_fall_one_cell_above_a_stain_tops_that_stain_up stdout ----
thread 'runtime::tests::a_fall_one_cell_above_a_stain_tops_that_stain_up' (196484) panicked at src/runtime.rs:672:9:
assertion `left == right` failed
  left: None
 right: Some(FallStain { cell: [10, 100, 20], place: false, tank: 800 })

---- runtime::tests::a_second_fall_tops_the_stain_it_lands_on_up_instead_of_placing_another stdout ----
thread 'runtime::tests::a_second_fall_tops_the_stain_it_lands_on_up_instead_of_placing_another' (196493) panicked at src/runtime.rs:659:9:
assertion `left == right` failed
  left: None
 right: Some(FallStain { cell: [10, 101, 20], place: false, tank: 800 })

---- runtime::tests::a_twenty_hp_zombies_fall_leaves_four_hundred_mb_in_its_own_cell stdout ----
thread 'runtime::tests::a_twenty_hp_zombies_fall_leaves_four_hundred_mb_in_its_own_cell' (196502) panicked at src/runtime.rs:643:9:
assertion `left == right` failed
  left: 0
 right: 400
```

The fifth test (`a_fall_with_no_cell_to_stain_leaves_the_level_alone`: `Cell::Taken` → `None`, zero and
negative health → no blood) passes against the stub by construction — it is the boundary a *wrong*
implementation would break, not one the stub breaks.

**A red run before it changed the rule.** The first draft of `fall_stain` only knew "the cell below is a
stain" and "the cell below is not", so a second death on the same spot would have found the mob's own
cell *occupied* and written nothing — the counterfactual's 800 mB would have failed in game. Re-reading
`BlockBloodStain.java` showed the second death tops up the block entity **in the mob's own cell** (the
mob stands on the stain), and the rule gained the `Cell::Stain` case; the red log above is the run
against that corrected rule, not the first draft.

Green: `sdk-test.log` — **45 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 4/45** (`EXIT:101`) | `sdk-test-red-slice6.log` |
| tests (green) | `cargo test` | **45 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 510 API uses, 16 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **185,614 B**, sha256 `b99d911e…1872c`, `CMP-IDENTICAL`, 75 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | unchanged: slice 1's one `"rolls": 1.0`; no new data file this slice | job report |
| artifact record | `sdk-artifact-0.2.6.txt` | bytes/sha/entries/manifest/runtime+stain sha, check and fits lines, the packed `data/` list | `sdk-artifact-0.2.6.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-0d2d381f7c4f`, with the log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical
to the port (`diff -r`/`cmp` empty, `version = "0.2.6"`), so the game ran the sources this package is
built from.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–W) | `e2e-launcher.log` |
| control (no package at all) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C (*the pickaxe was not repaired … damage 217*) | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=fall` | **`FAIL` + `EXIT:1`** at V (*the fall left a stain holding 0 mB, expected 20 x 20 = 400*) | `e2e-launcher-selftest-fall.log` |
| selftest (slices 2–5, history) | `SELFTEST=infuser|tank|blood|stain` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=8 ok=8 missing-exit=0
bad-exit=0` (exit 0).

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== V: a mob's fall death leaves a Blood Stain holding its blood ==
  Blood Stain [204, 101, -6]: tank 400/5000 mB, t=9625
  204, 101, -6 has the following block data: 400
  the 20 HP zombie died of its fall and left evilcraft:blood_stain at 204,101,-6 holding 400 mB
== X: a second fall death on the same cell tops the stain up to 800 mB ==
  Blood Stain [204, 101, -6]: tank 800/5000 mB, t=9682
  204, 101, -6 has the following block data: 800
  a second fall death on the same cell: 400 -> 800 mB in one stain, no second block
== W: the same mob killed with a sword leaves the cell empty (the death's differential) ==
  the same cell after a sword kill is minecraft:air (no stain)
```

**Fail-capable, four ways.** (1) `SELFTEST=fall` inverts V's own assertion and exits 1. (2) X is the
counterfactual: the same cell, the same mob, a second fall — 400 → 800 in the *same* block, and the cell
above stays air, so a "second stain instead of a top-up" would fail. (3) W is the death's differential:
the same cell and the same zombie killed with a diamond sword leave `minecraft:air`, so the blood in V
and X can only come from the fall's damage type. (4) The whole-package control leaves every id unknown,
so V cannot even place a stain.

### The probe that found the two harness traps (kept as `probe-fall.sh`, not a verdict)

The first full run failed at V with *a fall death left 'minecraft:air'*. A diagnostic build (chat lines
from each step of `on_death`) and a throwaway probe showed the mod was never reached because the zombie
was not dying of a fall at all: summoned into a **one-cell** stone shaft, it died with
`type="minecraft:in_wall"` before it fell. With the shaft three cells wide the same run printed
`death type="minecraft:fall"`, `cell=[204, 101, -6]`, `health=20`, `own=Free`, `set_block … -> Ok(true)`,
`merge Ok(true) tank=400`, and a second drop `own=Stain(400) -> tank=800`. The diagnostics were removed
before the package was rebuilt; the probe stays as `ports/_shared/evidence/evilcraft-e2e/probe-fall.sh`
(its own `EXIT:0` is a probe, not this slice's evidence).

### Harness facts learned here (record these before the next fall/placement slice pays for them again)

1. **A mob summoned into a one-cell shaft dies of `minecraft:in_wall`, not of the fall.** Three cells of
   air inside the shaft is enough; the stone shell can be five cells wide.
2. **`HurtSource.position` is `None` for a fall death** (measured: `type="minecraft:fall" causing=0
   direct=0 pos=None`). The victim's own `EntityRef::block_position` is the cell upstream reads, and it
   is exact at `living_death` (the mob is still in the level).
3. **`"$z.5"` for `z=-6` is `-6.5`, the centre of cell -7** — the same half-cell trap slice 5 paid for.
   The summon point is computed with `awk` (`x + 0.5`), and the run asserts the cell the blood landed in.
4. **A 1 HP zombie still has 20 max health**, so `{Health:1.0f}` makes a six-block drop lethal while the
   stain's 400 mB proves the *max* health is the field upstream multiplies (a current-health reading
   would give 20).
5. **The launcher's `native-build` is content-addressed and says *Up to date (no build needed)* when the
   sources match an existing build** — and `launcher-game.sh`'s `built_tag` fallback then picks the
   **newest** tag with a byte-identical *manifest*, which can be a different (stale) build of the same
   version. Always `diff -r` the export against the port after a build (this run caught a diagnostic
   build being exported that way) and delete the stale tag rather than trusting the printed tag.
6. **`/evilcraft stain` reads the block entity's clamp, `/data get block … fluids` reads the saved
   compound** — both were asserted (400 and 800 in each), so a readout that disagreed with the save
   would fail.

### Deviations this slice adds

1. **The stain is placed at the death, not one tick later.** Upstream defers the write because Forge's
   fall handler caches the landed block; at this pin `living_death` is the engine's own event and the
   write lands where the mob died (measured in V/X). The rule and the numbers are unchanged.
2. **`canSurvive` is the carpet template's rule**: the cell must be air and the block below a sturdy
   face. Upstream additionally refuses a block below that has a block entity (`spawnOnBlockEntities =
   false`) and accepts a hopper (`blockstate.getBlock() == HOPPER`); slice 5's deviation 4 already
   records that a stain on a hopper cannot survive here, and the block-entity case is accepted by the
   pin's rule where upstream refuses it.
3. **The `tconstruct:.*` spawn blacklist is not carried**: Tinkers' Construct is not in this port's
   carried set, so no block the port ships can match it.
4. **The vengeance-spirit exclusion is not modelled**: `EntityVengeanceSpirit` is not carried.
5. **Max health is read at the death** (the mob is still alive at `living_death`), through the same
   helper the extractor fill uses — no stored "last health", no extra hook.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe, the 25,000 mB recipe and the missing
pickaxe tag are the earlier slices'.)

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| a mob's **fall death** leaves a stain, `bloodMBPerHP = 20` × max health | **tested** | V: a 20 HP zombie's fall → `evilcraft:blood_stain` at `204,101,-6`, `400/5000` mB in the readout **and** in the saved `fluids` |
| a second fall on the same cell **tops the stain up** | **tested** | X: `400 → 800` in the same block, `minecraft:air` above it (no second stain) |
| a death that is **not** a fall leaves no stain | **tested** | W: the same zombie killed with a diamond sword leaves the cell `minecraft:air` |
| the tank's 5,000 mB cap | **tested (unit + in game)** | `a_fall_never_fills_a_stain_past_its_five_buckets`; every in-game read is against `5000` |
| a fall with no cell to stain | **tested (unit)** | `a_fall_with_no_cell_to_stain_leaves_the_level_alone` |
| the `tconstruct:.*` spawn blacklist, `spawnOnBlockEntities`, the vengeance-spirit exclusion | not ported | deviations 3–4 (none of the three is in this port's carried set) |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 7 — the Colossal Blood Chest (`BlockColossalBloodChest`, `BlockEntityColossalBloodChest`) (2026-11-23)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.7**.
The registry entry, the PR and the ledger stay the orchestrator's.

### The gap this slice closes, and why this family

Slice 1's gap 7, unchanged since (quoted as slice 2's coverage row):

> No blood stain, **colossal chest**, purifier, … — the rest of the jar's machinery (gap 7).

It is the third machine of the blood economy the port already ships, its numbers are the Blood
Chest's own, and it is a **multiblock** — the family the parity rule names explicitly ("every
multiblock is built from the mod's own blocks, asserted to form, and one operation run"). The
assertion is a `formed` bit plus the repair rate, both readable through `/evilcraft colossal`.

**Tier:** `deep` still governs the port; this slice is a `machines`/`deep` family inside it — a
block entity with 45 repair slots, a 50-slot menu and a screen, a tank, and a structure rule read
from the world. No entity, renderer or dimension.

### Upstream source read (analysis only, not copied)

<https://github.com/CyclopsMC/EvilCraft>, clone at `ports/_shared/upstream/evilcraft` (tag
`1.21.1-1.2.100`, commit `aa81220`, read 2026-11-23):
`block/BlockColossalBloodChest.java`, `block/BlockColossalBloodChestConfig.java`,
`block/BlockReinforcedUndeadPlank.java`, `blockentity/BlockEntityColossalBloodChest.java`,
`blockentity/BlockEntityColossalBloodChestConfig.java`,
`inventory/container/ContainerColossalBloodChest.java` (+`Config`),
`blockentity/tickaction/bloodchest/BulkRepairItemTickAction.java`,
`blockentity/tickaction/EmptyFluidContainerInTankTickAction.java`,
`blockentity/tickaction/EmptyInTankTickAction.java`,
`core/blockentity/BlockEntityWorking.java`; plus the jar's own
`assets/evilcraft/{blockstates,models}/…colossal…`, `data/evilcraft/loot_table/blocks/…`,
`data/minecraft/tags/block/mineable/axe.json` and the lang keys.

### What the artefact says the family is (read, not guessed)

`BlockColossalBloodChest`, `BlockEntityColossalBloodChest`, `BlockReinforcedUndeadPlank`
(upstream clone at tag `1.21.1-1.2.100`, commit `aa8122019`), plus CyclopsCore's
`HollowCubeDetector`/`ExactSizeValidator`/`ExactBlockCountValidator` and the jar's own
`assets/`/`data/` files:

- **Structure:** `HollowCubeDetector` + `AllowedBlock[]` = reinforced undead planks (any count)
  + **exactly one** `colossal_blood_chest` (`ExactBlockCountValidator(1)`), with
  `ExactSizeValidator(new Vec3i(2, 2, 2))` → a **3×3×3 outer cube whose 27th cell is air**: 25
  planks plus one controller on the shell, the middle hollow. The jar's own lang says it too:
  *"make a **hollow** 3x3x3 structure of Reinforced Undead Planks … exactly one Colossal Blood
  Chest block somewhere in the structure wall, … not … in the middle of the structure"*.
  **The brief's "27 controllers in a cube" is not what the mod builds; the artefact wins and
  this is the shape that ships.**
- **Machine:** 45 repair slots (`ContainerColossalBloodChest.CHEST_INVENTORY_ROWS 5` ×
  `COLUMNS 9`), one container slot (`SLOT_CONTAINER`), four upgrade slots
  (`BlockEntityWorking.INVENTORY_SIZE_UPGRADES`), a 10,000 mB tank (`LIQUID_PER_SLOT`), the
  Blood Chest's `baseMBPerDamage = 5` and `ticksPerDamage = 2`, and an anvil step of
  `25 × 5 = 125 mB`. `BulkRepairItemTickAction.canTick` asks `tile.canWork()` first, and
  `canWork()` is `getSize() == ExactSizeValidator(2,2,2)` — an open shell repairs nothing. The
  container slot's `EmptyFluidContainerInTankTickAction.canTick` does **not** ask `canWork`, so
  a filled extractor still empties into the tank while the cube is open.
- **Block:** `requiresCorrectToolForDrops`, `strength(5.0)`, `SoundType.WOOD`, `noOcclusion`;
  the plank is `strength(5.0)`, `SoundType.WOOD`, `noOcclusion`, and both are in the jar's
  `minecraft:mineable/axe` tag. The chest's loot table runs `minecraft:copy_name` +
  `evilcraft:copy_tank_data`.

### What is in it

| Half | What | Source |
|---|---|---|
| Structure rule | `runtime::colossal_formed`: flood-fill of the mod's two blocks out from the controller (`HollowCubeDetector`), the region must be exactly the 26 cells of a 3×3×3 shell, the middle cell air (`ExactSizeValidator(2,2,2)`), and exactly one chest — this one (`ExactBlockCountValidator(1)`) | `getCubeDetector`, the lang |
| Block | `evilcraft:colossal_blood_chest` (template `minecraft:smithing_table` — a full cube with no state properties whose *class* is a container, see harness fact 1) and `evilcraft:reinforced_undead_planks` (template `minecraft:stone`) | `BlockColossalBloodChestConfig`, `BlockReinforcedUndeadPlankConfig` |
| Block entity | 50 slots (45 repair, the container slot at 45, four upgrade slots), the tick, the tank | `BlockEntityColossalBloodChest` (`SLOTS_CHEST`, `SLOT_CONTAINER`, `SLOTS`, `LIQUID_PER_SLOT`) |
| Menu | `evilcraft:colossal_blood_chest_menu` — 50 slots, 2 data slots (tank, capacity) | `ContainerColossalBloodChest` |
| Repair | the chest's own numbers: one damage point every two ticks for 5 mB, `minecraft:stick` never, a broken anvil 125 mB a step | `BulkRepairItemTickAction`, `BloodChestRepairActionRegistry` |
| Tank drop | the third `tank::Machine`: a broken controller's own item carries `minecraft:block_entity_data` with the tank, and placing it back clamps it to 10,000 mB | the chest's `copy_tank_data` route (slice 3) |
| Command | `/evilcraft colossal <pos>` (`formed`, tank, repair-slot count, slots, container, slots' damage); `/evilcraft blood` and `/evilcraft damage|load` now reach it | harness only (upstream has no commands) |

### TDD: the red proof, before the implementation

`sdk-test-red-slice7.log` (the eleven tests written; `colossal_formed` stubbed to `None`, the
rule not implemented). The two that assert a structure *forms* fail; the ones that assert a
wrong arrangement is refused pass against a stub by construction — they are the boundary a
*wrong* implementation breaks, and the in-game counterfactual (Z) is their leg:

```
test result: FAILED. 54 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_missing_shell_block_breaks_the_structure stdout ----
thread '…' panicked at src/runtime.rs:906:13:
assertion `left == right` failed: putting the plank back at [10, 103, -20] forms the cube again
  left: None
 right: Some([10, 101, -20])

---- runtime::tests::every_shell_cell_of_the_cube_can_be_the_controller stdout ----
thread '…' panicked at src/runtime.rs:882:21:
assertion `left == right` failed: a chest at [10, 101, -20] is a shell cell of the cube
  left: None
 right: Some([10, 101, -20])

test result: FAILED. 54 passed; 2 failed; …
EXIT:101
```

**A red run that changed the rule.** The first draft of `colossal_anchor` walked one cell out
of the controller on each axis (`CubeDetector.navigateToBorder`'s shape) and derived the cube's
corner from what it found. `every_shell_cell_of_the_cube_can_be_the_controller` proved that
wrong for a **hollow** cube: a shell cell whose inner neighbour is the middle cell sees *air*
behind it and is mistaken for the cube's edge, so a controller at a corner next to the hollow
(e.g. `[11, 102, -18]`) read `None`. The rule became the flood fill `HollowCubeDetector`
actually is; the red log above is the run against the stub of *that* rule, and the same test
went green with it.

Green: `sdk-test.log` — **56 passed, 0 failed** (45 before the slice + 11 new: the structure
constants, every shell cell as the controller, a missing plank, a filled middle, a second
chest, a chest in the middle, a lone chest/wall, a cube that is not exactly three wide, the
slot layout, the formed repair rate, the unformed gate). `sdk-clippy.log` — `cargo fmt --check`
clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 2/56** (`EXIT:101`) | `sdk-test-red-slice7.log` |
| tests (green) | `cargo test` | **56 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 518 API uses, 16 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **199,156 B**, sha256 `71f78499…b6d9f1`, `CMP-IDENTICAL`, 92 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (56 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 13 recipe/loot files; unchanged slice 1 `"rolls": 1.0`; **both new loot tables `ok`** | job report |
| artifact record | `sdk-artifact-0.2.7.txt` | bytes/sha/entries/manifest/runtime+colossal sha, check and fits lines, the packed data list | `sdk-artifact-0.2.7.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-e9c9da9cf9df`, with the log
line `Compiling evilcraft (1 of 1)`; the export's `src/`, `data/` and `horizon-mod.toml` are
byte-identical to the port (`diff -r`/`cmp` empty, `version = "0.2.7"`), so the game ran the
sources this package is built from. (Two earlier builds were deleted, not exported: the first
had a `minecraft:stone` controller whose menu never opened, see harness fact 1.)

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–W, **Y, Z, AA, AB**) | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=colossal` | **`FAIL` + `EXIT:1`** at Y (*the hollow 3x3x3 shell … is not formed (formed=true)*) | `e2e-launcher-selftest-colossal.log` |
| selftest (slices 2–6, history) | `SELFTEST=infuser|tank|blood|stain|fall` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=9 ok=9 missing-exit=0
bad-exit=0` (exit 0): the eight earlier legs stay green and the ninth is this slice's.

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== Y: a hollow 3x3x3 shell forms one Colossal Blood Chest, which repairs one point a two ticks ==
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 0/10000 mB, repair 45, slots 46, container empty@0, [], t=9854
  a hollow 3x3x3 of evilcraft:reinforced_undead_planks around one controller: formed=true, 45 repair slots, 46 slots
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 955/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@241], t=9910
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 820/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@214], t=9964
  54 ticks: damage 241 -> 214 (27 points), tank 955 -> 820 (-135 mB)
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 970/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@243], t=9986
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 0/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@49], t=10455
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 0/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@49], t=10503
  the tank emptied: damage 243 -> 49 for 970 mB (194 points at 5 mB), and 40 more ticks repair nothing
== Z: open the shell and the machine stops (the counterfactual) ==
  Colossal Blood Chest [198, 101, -10]: formed=false, tank 0/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@49], t=10522
  Colossal Blood Chest [198, 101, -10]: formed=false, tank 1000/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@250], t=10542
  Colossal Blood Chest [198, 101, -10]: formed=false, tank 1000/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@250], t=10590
  one shell block out: formed=false, damage 250 and tank 1000 unchanged over 48 ticks
  Colossal Blood Chest [198, 101, -10]: formed=false, tank 1000/10000 mB, repair 45, slots 46, container evilcraft:blood_extractor@0, [0:minecraft:diamond_pickaxe@250], t=10682
  the menu lists 50 slots, and an extractor of 1,000 mB empties into the open machine's tank (1000 mB, container 0 mB)
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 930/10000 mB, repair 45, slots 46, container evilcraft:blood_extractor@0, [0:minecraft:diamond_pickaxe@242], t=10722
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 795/10000 mB, repair 45, slots 46, container evilcraft:blood_extractor@0, [0:minecraft:diamond_pickaxe@215], t=10775
  the plank back: formed=true, damage 242 -> 215 in 40 ticks again
== AA: the same pickaxe in a plain Blood Chest repairs at the same rate (the differential) ==
  Blood Chest [198, 101, -12]: tank 965/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@243], t=10838
  Blood Chest [198, 101, -12]: tank 825/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@215], t=10894
  plain chest: 56 ticks, damage 243 -> 215 (28 points), tank -140 mB — one point a 2 ticks for 5 mB, as the colossal chest's 27
== AB: the controller breaks into itself carrying its tank, and a plank drops itself ==
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 4000/10000 mB, repair 45, slots 46, container evilcraft:blood_extractor@0, [], t=11020
  break: {"block":"minecraft:air","drops":{"evilcraft:reinforced_undead_planks":1},"picked_up":{},"items_left_on_ground":1}
  a plank of the shell drops itself with an axe
  break: {"block":"minecraft:air","drops":{"evilcraft:blood_extractor":1,"evilcraft:colossal_blood_chest":1},"picked_up":{},"items_left_on_ground":2}
  the dropped evilcraft:colossal_blood_chest: Player66689 has the following entity data: {components: {"minecraft:block_entity_data": {id: "evilcraft:colossal_blood_chest", fluids: [{Amount: 4000, Fluid: "evilcraft:blood"}]}}, count: 1, id: "evilcraft:colossal_blood_chest"}
  the controller broke into itself carrying 4000 mB of evilcraft:blood
```

**Fail-capable, four ways.** (1) `SELFTEST=colossal` inverts Y's `formed` assertion and exits 1
(log above). (2) Z is the counterfactual: the *same* machine, the same damage-250 pickaxe and the
same 1,000 mB repair **nothing** over 48 ticks once one shell block is out, and the repair resumes
when the plank is back — a machine that ignored `canWork` would repair in both windows. (3) AA is
the differential: the same pickaxe, the same 1,000 mB and the same window in a plain
`evilcraft:blood_chest` cure the same number of points for the same 5 mB each, so the rate Y
measures is the Blood Chest's rule and not a colossal-specific one. (4) The whole-package control
leaves every id unknown, so Y cannot even place the controller.

### Harness facts learned here

1. **A mod container's menu opens only if the block's *template class* is one of the container
   classes.** `horizon-world/src/level/interact.rs` emits `PlayerEffect::OpenMenu` for
   `Class::AbstractFurnace | Anvil | Barrel | Beacon | BrewingStand | CartographyTable | Chest |
   … | SmithingTable | Stonecutter | TrappedChest | Lectern`, and nothing for `Class::Block`. A
   `minecraft:stone` controller therefore never opens its 50-slot menu: the click is accepted and
   `screen` stays `playing`, with **no error anywhere** (measured: the container was there — a
   stack in slot 0 spilled on break — and `/evilcraft colossal` read it fine). The other two
   machines hid this because they use `minecraft:furnace`. The colossal now uses
   `minecraft:smithing_table`: a full cube with **no state properties** whose class is a container,
   which keeps the blockstate a single `""` variant. The probe that found it is kept as
   `tools/probe-colossal.sh` (a diagnostic, not this slice's evidence).
2. **The `id:id` command argument namespaces a bare word.** `/evilcraft load … 0 empty` arrives as
   `minecraft:empty`, so the documented "`empty` clears the slot" never ran (it answered *"is not
   an item this game has"*). The command accepts both spellings now; AB's tank assertion depends
   on it (without the clear, the machine spent 110 mB repairing the pickaxe before the break and
   the dropped item carried 3,890 instead of 4,000).
3. **`ctl break` needs line of sight.** A far corner of the cube is behind the cube's own wall:
   `ctl break 200 103 -8` from outside reported *did not break*. Break a shell block on the near
   face (`198, 101, -8` here), and the controller on the same face.
4. **The machine repairs between a harness write and the readout.** `damage … 250` then a
   `colossal` read (which polls up to 160 ticks for a fresh line) shows `@241`, tank `955`. So the
   setup assertions are bounds (`≤ 250`, `≥ 900 mB`) and the *rate* is measured between two
   readouts, using the readout's own `t=` for the window (54 ticks → 27 points; 55 → 28).
5. **The tank-empty leg is one invariant, not two constants.** With 970 mB in the tank at the
   read, exactly 194 more points are payable: assert `damage_end == damage_start - tank/5` and
   `tank == 0`, never a literal damage value.
6. **The launcher is content-addressed and every source change is a new tag** (`…-2393d4c982cf`,
   `…-7c89aa82cc5d`, `…-e9c9da9cf9df` for 0.2.7's three builds), each with `Compiling evilcraft
   (1 of 1)` and the export `diff -r`-identical to the port. An old tag stays on disk and must not
   be mistaken for this build.

### Deviations this slice adds

1. **The efficiency discount is not modelled.** `BulkRepairItemTickAction.getRequiredFluid` scales
   the cost down by an `efficiency` that rises one per repaired point (capped 200) and decays by
   `baseConcurrentItems` each repair tick, so a large batch costs less than 5 mB a point upstream.
   The port charges the base 5 mB a point, as the brief asks ("reuse the existing
   `RepairItemTickAction` numbers, invent no new rule"). The measured rate (27 points for 135 mB,
   194 for 970) is therefore the *base* price; upstream's would drift below it on a long run.
2. **The four upgrade slots are storage only.** Upstream's `getUpgrades()` lists efficiency, speed
   and tier 1–3; none of their effects is modelled (no cheaper blood, no faster ticks, no tank
   multiplier). This matches the infuser's deviation 2/3; the slots exist so a carried promise can
   sit in them as upstream's GUI allows.
3. **The menu's slot order is the block entity's, not upstream's.** Upstream's container adds the
   container slot first and the 45 chest slots after it; the pin maps menu slot *n* to inventory
   slot *n*, so the port's screen lists the 45 repair slots (0–44), then the container (45), then
   the four upgrades (46–49). The container slot is menu slot 345 in `ctl` terms, not 300. The
   slot *counts* are upstream's.
4. **The structure is derived from the world on every tick, not saved.** Upstream saves a `size`
   and a detector re-runs it on place/destroy; the port's tick reads the 26 shell cells each time
   (a block entity's tick carries only its position, so the level is found among
   `Levels::list_levels` by the controller's own block). The rule and its answer are the same; a
   chunk unload of a shell cell unforms the machine until the chunk is back, where upstream's
   saved size would keep it formed.
5. **No `active` blockstate and no BER.** Upstream's `active` property selects between the chest
   model and an invisible one, and the animated chest is drawn by a block entity renderer. The
   port has no renderer (same class as the Blood Chest's cube), so the block is always the jar's
   `colossal_blood_chest` texture on a cube and no state property is declared.
6. **The block has no inventory faces.** Upstream's `ItemHandlerSlotMasked` exposes the container
   slot to every face and the repair slots to all but the bottom; the port declares no faces, as
   the Blood Chest and Infuser do, so a pipe or hopper moves nothing in or out. The menu and the
   tick are the routes.
7. **`copy_tank_data` reaches the third machine through the same `block_drops` route as the other
   two** (`minecraft:block_entity_data`), not through a loot function; the chest's `copy_name` is
   carried in the loot file and the tank is the listener's.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe, the 25,000 mB recipe and the
missing pickaxe tag are the earlier slices'.) One upstream oddity is recorded, not "fixed": the
jar's `blockstates/colossal_blood_chest.json` maps `active=false` to the *visible* model and
`active=true` to the invisible one, which is the inverse of what reading the property suggests;
the port draws the visible model in every state (deviation 5).

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `colossal_blood_chest` block, item, blockstate, model, texture, lang, creative row | **tested** | Y: placed from the game's own `/fill` + `/setblock`, read back at `198,101,-10` |
| the 3×3×3 hollow shell (`HollowCubeDetector` + `ExactSizeValidator(2,2,2)` + `ExactBlockCountValidator(1)`) | **tested** | Y: `formed=true` with 25 planks and one controller, middle air; unit tests for every shell cell, a missing plank, a filled middle, a second chest, a lone chest and an oversized cube |
| one shell block out → not formed | **tested** | Z: `formed=false` and no repair over 48 ticks; the unit `a_missing_shell_block_breaks_the_structure` |
| 45 repair slots, the container slot, the four upgrade slots | **tested** | Y/Z: `repair 45`, `slots 46`, the menu lists **50** slots (phase Z opens it) |
| the repair tick (`baseMBPerDamage 5`, `ticksPerDamage 2`, the anvil step) | **tested** | Y: 27 points / 135 mB in 54 ticks; AA: the plain chest 28 / 140 in 56; the anvil path is the shared `runtime::anvil_step` (slice 1's phase D) |
| the tank's 10,000 mB and the "until it empties" rule | **tested** | Y: 970 mB → 194 points → tank 0, 40 more ticks repair nothing |
| the container slot emptying a filled extractor **while unformed** | **tested** | Z: 1,000 mB extractor → tank 1,000, container 0 |
| `copy_tank_data` on the colossal chest (the tank in the dropped controller) | **tested** | AB: 4,000 mB broken, carried, read back from the dropped item's `minecraft:block_entity_data` |
| loot tables + `minecraft:mineable/axe` for both blocks | **tested** | AB: a plank drops itself, the controller drops itself, both with a diamond axe in survival |
| `/evilcraft colossal`, and `blood`/`damage`/`load` reaching the machine | **tested** | every readout above |
| the efficiency discount (`MAX_EFFICIENCY`, `baseConcurrentItems`) | not ported | deviation 1 |
| efficiency/speed/tier upgrades in the four upgrade slots | not ported | deviation 2 (the slots exist as storage) |
| the `active` blockstate and the animated BER chest | not ported | deviation 5 (no renderer at this tier's shape) |
| the jar's `colossal_blood_chest` / `reinforced_undead_planks` recipes | not ported | their ingredients (`blood_infusion_core`, `undead_planks`) are not carried — gap 1 |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 8 — the Purifier is blocked; the Dark Tank (`BlockDarkTank`, `BlockEntityDarkTank`) (2026-11-23)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.8**.
The registry entry, the PR and the ledger stay the orchestrator's.

### The purifier: `blocked(no-purifier-action-table)` — measured, not guessed

The brief's first pick was the Purifier (`evilcraft:purifier`, the consumer of the Blood Chest's
curse roll), with a stop condition: *if the jar's purifier binds to items this package does not
carry (its actions are Java classes with no `data/evilcraft/recipe/purifier/`), stop and take the
`dark_tank`.* The artefact meets it:

- `unzip -l evilcraft-1.21.1-neoforge-1.2.100.jar | grep data/evilcraft | grep -i purif` → only
  `data/evilcraft/loot_table/blocks/purifier.json` and `data/evilcraft/recipe/crafting/purifier.json`;
  there is **no** `data/evilcraft/recipe/purifier/`. The behaviour is four Java classes registered in
  code (`blockentity/tickaction/purifier/PurifierActionRegistry.java`, upstream tag `1.21.1-1.2.100`,
  commit `aa81220`):
  - `ToolBadEnchantPurifyAction` — removes curses from any item (self-contained);
  - `DisenchantPurifyAction` — binds **`evilcraft:blook`** (`ALLOWED_BOOK`), an item this package
    does **not** carry (`data/evilcraft/horizon/item/` holds the extractor and the three promises; the
    blook's own recipe is a blood infusion of a book);
  - `CollectPotionPurifyAction` — binds `minecraft:glass_bottle` **and a `LivingEntity` in the
    block's AABB** (`getEntitiesOfClass(LivingEntity.class, new AABB(pos, pos+(1,2,1)))`);
  - `MobHeadDowngradePurifyAction` — vanilla heads, meaningful only with the Blood Infuser's head
    progression (which only this action consumes).
- A purifier that ships one of its four actions is an inert machine for the other three; the
  campaign's rule is that a stopped report naming the gap is the complete result (`mifa`'s precedent).
  The purifier is **not ported**: `blocked(no-purifier-action-table)`, needing `evilcraft:blook`
  (item + its infusion recipe), the entity query the potion action needs, and the head progression.

### The gap this slice closes, and why this family

Slice 1's gap 7, quoted as slice 7's coverage row:

> `blood_stain`, `colossal_blood_chest`, `purifier`, `spirit_*`, **`dark_tank`**, `display_stand`,
> `environmental_accumulator`, … | not ported | the rest of the jar's machinery (gap 7).

It is the brief's own fallback, it is a **tank** family (the port already carries four tanks and the
`copy_tank_data` drop route), and it gives the extractor a second sink in the field — the gesture the
Blood Stain already uses, in the other direction. **Tier:** the port is `deep`; this slice is a
`machines`-shaped family inside it (a block entity with one tank, an item Part, a command). No
entity, renderer or dimension.

### What the artefact says the family is (read, not guessed)

`BlockDarkTank`, `BlockDarkTankConfig`, `BlockEntityDarkTank` (upstream clone at tag `1.21.1-1.2.100`,
commit `aa81220`), plus the jar's own `assets/evilcraft/{blockstates,models}/dark_tank.json`,
`data/evilcraft/loot_table/blocks/dark_tank.json`, `data/evilcraft/recipe/crafting/dark_tank.json`
and the lang keys:

- **Tank:** `BlockEntityDarkTank.BASE_CAPACITY = 16000` mB, one tank, **no slots and no menu**
  (`BlockEntityTankInventory` with `0, 0` slots).
- **Click:** `BlockDarkTank.useItemOn` → `FluidUtil.interactWithFluidHandler(player, hand, world, pos, UP)`:
  a held fluid container empties into the tank; a click **without** a container toggles `enabled` and
  prints `amount / capacity mB` in the action bar. `enabled` gates the server ticker, which pushes the
  tank down into a handler below, or fills container items/players standing under it.
- **Block:** `strength(0.5)`, `SoundType.GLASS`, a 14/16 shape (`Block.box(2, 0.02, 2, 14, 15.98, 14)`);
  the model is a custom `evilcraft:dark_tank` loader that renders the fluid — not expressible as
  content, so the port draws a cube over the jar's own two textures (deviation 2).
- **Item:** `ItemBlockFluidContainer` — a block item carrying its tank and capacity; the loot table
  runs `copy_name` + `copy_tank_data`.
- **Recipe:** `dark_gem` + `c:ingots/iron` + `c:glass_blocks`; `dark_gem` is not carried (gap 1), so
  the recipe is not either. The three `special/` recipes (combine, clear, large) are crafting-special
  recipes — gap 1 again.

### What is in it

| Half | What | Source |
|---|---|---|
| Block | `evilcraft:dark_tank`, template `minecraft:glass` (full cube, no state properties, no tool needed), `hardness 0.5`, `sounds minecraft:glass` | `BlockDarkTankConfig` |
| Block entity | one 16,000 mB tank, saved as the pin's `fluids` list; no tick, no slots, no menu | `BlockEntityDarkTank` |
| Click | a **crouching** right-click with a filled Blood Extractor pours as much as fits and keeps the rest in the item | `BlockDarkTank.useItemOn` → `FluidUtil`, through the port's own container |
| Tank drop | a broken tank's own item carries `minecraft:block_entity_data` with the tank, clamped to 16,000 on placement | the chest's `copy_tank_data` route (slice 3) |
| Command | `/evilcraft darktank <pos>` (`tank`, capacity) and `/evilcraft blood` reaching the 16,000 tank | harness only (upstream has no commands) |

### TDD: the red proof, before the implementation

`sdk-test-red-slice8.log` (seven tests written, the rules stubbed: `tank_fill` returning zeros,
`drain_container_into` returning `(0, 0)`, `carried_tank_with`/`restored_tank_with` returning
`None`/`0`, `Machine::DarkTank.capacity()` returning the machines' 10,000):

```
test result: FAILED. 57 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_filled_extractor_empties_into_a_dark_tank stdout ----
thread '…' panicked at src/runtime.rs:1214:9:
assertion `left == right` failed: a 5,000 mB extractor fits a fresh 16,000 mB tank whole
  left: (0, 0, 0)
 right: (5000, 5000, 0)

---- runtime::tests::a_dark_tank_takes_only_its_room_and_the_extractor_keeps_the_rest stdout ----
thread '…' panicked at src/runtime.rs:1224:9:
assertion `left == right` failed: the tank had room for 2,000 mB
  left: 0
 right: 2000

---- tank::tests::a_dark_tank_carries_its_sixteen_buckets stdout ----
thread '…' panicked at src/tank.rs:314:9:
assertion `left == right` failed
  left: 10000
 right: 12000

failures:
    runtime::tests::a_dark_tank_payload_carries_sixteen_buckets_and_comes_back_clamped
    runtime::tests::a_dark_tank_takes_only_its_room_and_the_extractor_keeps_the_rest
    runtime::tests::a_doctored_dark_tank_is_read_at_its_own_capacity
    runtime::tests::a_filled_extractor_empties_into_a_dark_tank
    runtime::tests::a_full_dark_tank_or_an_empty_extractor_moves_nothing
    tank::tests::a_dark_tank_carries_its_sixteen_buckets

test result: FAILED. 57 passed; 6 failed; …
EXIT:101
```

The seventh new test (`a_dark_tank_holds_sixteen_buckets`, the two constants) passed against the
stubs by construction; it is the boundary a wrong capacity breaks (the in-game `0/16000` readout is
its leg). Green: `sdk-test.log` — **63 passed, 0 failed** (56 before + 7). `sdk-clippy.log` —
`cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 6/63** (`EXIT:101`) | `sdk-test-red-slice8.log` |
| tests (green) | `cargo test` | **63 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 519 API uses, 16 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **206,244 B**, sha256 `4d8f7f67…1de410`, `CMP-IDENTICAL`, 100 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (63 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 14 recipe/loot files; unchanged slice 1 `"rolls": 1.0`; **the new loot table `ok`** | job report |
| artifact record | `sdk-artifact-0.2.8.txt` | bytes/sha/entries/manifest/runtime+dark_tank sha, check and fits lines, the packed list | `sdk-artifact-0.2.8.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-2a04e567b6f4`, with the log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/`, `assets/` and `horizon-mod.toml` are
byte-identical to the port (`diff -r`/`cmp` empty, `version = "0.2.8"`), so the game ran the sources
this package is built from (`e2e-export-check-0.2.8.log`).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `launcher-game.sh run evilcraft` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–W, Y, Z, AA, AB, **AC, AD, AE**) | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=darktank` | **`FAIL` + `EXIT:1`** at AC (*the crouching click poured 0 mB into the tank, expected the 2,000 mB room it had*) | `e2e-launcher-selftest-darktank.log` |
| selftest (slices 2–7, history) | `SELFTEST=infuser|tank|blood|stain|fall|colossal` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=10 ok=10 missing-exit=0
bad-exit=0` (exit 0): the nine earlier legs stay green and the tenth is this slice's.

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== AC: a Dark Tank holds sixteen buckets, and a filled extractor's crouching click fills it ==
  Dark Tank [204, 101, 6]: tank 0/16000 mB, t=11077
  a fresh evilcraft:dark_tank at 204,101,6 reads 0/16000 mB
  Dark Tank [204, 101, 6]: tank 14000/16000 mB, t=11094
  Blood Extractor held: 5000/5000 mB, t=11122.
  crouch + right-click on 204,101,6 (aim on evilcraft:dark_tank)
  Dark Tank [204, 101, 6]: tank 16000/16000 mB, t=11162
  Blood Extractor held: 3000/5000 mB, t=11169.
  extractor 5000 -> 3000 mB, dark tank 14000 -> 16000 mB (the 2,000 mB room, the rest kept)
  the dark tank's saved tank: 204, 101, 6 has the following block data: [{Amount: 16000, Fluid: "evilcraft:blood"}]
== AD: a standing click pours nothing, and neither does a crouching click on stone ==
  Dark Tank [204, 101, 6]: tank 0/16000 mB, t=11200
  right-click on 204,101,6 (aim on evilcraft:dark_tank)
  Dark Tank [204, 101, 6]: tank 0/16000 mB, t=11224
  Blood Extractor held: 5000/5000 mB, t=11232.
  standing: extractor 5000 mB, dark tank 0 mB (nothing moved)
  crouch + right-click on 204,101,8 (aim on minecraft:stone)
  Blood Extractor held: 5000/5000 mB, t=11279.
  the same crouching click on minecraft:stone: extractor still 5000 mB
  crouch + right-click on 204,101,6 (aim on evilcraft:dark_tank)
  Dark Tank [204, 101, 6]: tank 5000/16000 mB, t=11319
  Blood Extractor held: 0/5000 mB, t=11327.
  crouching on the tank: extractor 5000 -> 0 mB, dark tank 0 -> 5000 mB
== AE: the broken tank drops itself carrying its blood, and places back full ==
  Dark Tank [204, 101, 6]: tank 12000/16000 mB, t=11356
  break: {"block":"minecraft:air","drops":{"evilcraft:dark_tank":1},"picked_up":{},"items_left_on_ground":1}
  the dropped evilcraft:dark_tank: Player81900 has the following entity data: {components: {"minecraft:block_entity_data": {id: "evilcraft:dark_tank", fluids: [{Amount: 12000, Fluid: "evilcraft:blood"}]}}, count: 1, id: "evilcraft:dark_tank"}
  Dark Tank [204, 101, 6]: tank 12000/16000 mB, t=11544
  the dark tank broke into itself carrying 12000 mB of evilcraft:blood, and places back at 12000/16000 mB
```

**Fail-capable, four ways.** (1) `SELFTEST=darktank` inverts AC's pour assertion and exits 1 (log
above). (2) AD is the counterfactual: the *same* tank, the *same* full extractor and the *same*
click **standing** pour nothing, and the same crouching click on `minecraft:stone` pours nothing,
while the same crouching click on the tank pours the whole 5,000 — a Part that ignored either the
crouch or the block would pour in one of those windows. (3) AC's clamp is its own differential: with
14,000 mB in the tank only 2,000 moves and the extractor keeps 3,000, so a click that drained the
whole container would fail. (4) The whole-package control leaves every id unknown, so AC cannot even
place the tank.

### Harness facts learned here

1. **The launcher's export carries every mod of its instance, not just this port's** — the game's
   `horizon-native-mods/` in `~/.cache/horizon-e2e/0a875da/evilcraft/` lists other ports' libs. The
   export check therefore compares this port's own directory (`horizon-native-mods/evilcraft/`)
   against the port, not the export root.
2. **A subagent's outer `echo EXIT:$?` wrapper swallows the leg's status.** `e2e-guard.sh` prints its
   own `EXIT:<rc>` from `wait "$child"`, so a `bash -c '…; echo EXIT:$?'` around the leg made a
   FAILing control look like `EXIT:0` (measured 2026-11-23: the first control log ended
   `EXIT:1 / EXIT:1 / EXIT:0`). Run the leg directly (`env CONTROL=1 … bash tools/e2e.sh`) and let
   its own trap and the guard report the status; `check-e2e-logs.py` reads the last `EXIT:` line.
3. **A `minecraft:glass` template block is a plain full cube** and drops through its own loot table
   with any tool (`requires_tool` unset): a diamond pickaxe breaks it in survival and `ctl break`
   reports the drop.
4. **`aim_at`/`crouch_click` work on a full-cube mod block at `h=0.5`** with the block's own id as
   the expected crosshair state (`evilcraft:dark_tank`); the stain's `h=0.03` is only for its
   one-pixel shape.

### Deviations this slice adds

1. **The click's gesture and its container are the port's own.** Upstream takes *any* right-click
   with any fluid-container item through the item fluid capability; this pin's capability items carry
   water and lava only, so the port's container is the extractor (custom data, as everywhere else) and
   its gesture is the crouching click the stain already uses. A standing click pours nothing (phase AD
   asserts it).
2. **The block is a cube over the jar's two textures.** Upstream's model is a custom
   `evilcraft:dark_tank` loader that draws the fluid inside a 14/16 block with a 14/16 shape; a content
   block cannot (same class as the Blood Chest's cube deviation), so the port's block is the full cube
   of the template with `dark_tank_side`/`dark_tank_up`.
3. **The auto-supply half is not carried.** `enabled`, the right-click readout in the action bar, the
   downward auto-fill into a handler or container items/players below, the in-inventory auto-fill
   (`autoFillBuckets`), the item's tank tooltip and the light emission are all part of
   `BlockEntityDarkTank.TickerServer` / `ItemBlockFluidContainer`; none is in this slice. The tank's
   own pipe faces are declared as outputs only (the machines' gap 6: no
   `block_entity_accepts_fluid` at this pin, so the tank would hold a foreign fluid and read 0).
4. **The block entity has no tick.** Upstream's ticker only drives deviation 3's auto-supply, so with
   it gone there is nothing to tick; the click writes the tank and the engine saves it.
5. **The recipe is not carried** (gap 1): `dark_gem` is not in the port, and the three `special/`
   recipes are crafting-special handlers this pin does not express as content.
6. **The creative tab lists the tank empty.** Upstream fills the tab with a tank per fluid, including a
   blood tank at every capacity step (`capacity << 2` up to `maxTankCreativeSize`); the port's tab
   holds the one plain block, and the blood is put in with the extractor.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe, the 25,000 mB recipe and the
missing pickaxe tag are the earlier slices'.) One upstream oddity is recorded, not "fixed": the
`dark_tank` model's `down` face uses `#up`, so both ends of the upstream tank show the top texture —
the port's cube keeps that (`cube_bottom_top` with `bottom` = `dark_tank_up`).

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `dark_tank` block, item, blockstate, model, texture, lang, creative row | **tested** | AC: placed from its own item at `204,101,6`, read back `0/16000 mB`; AE: broken and placed back |
| the 16,000 mB tank (`BlockEntityDarkTank.BASE_CAPACITY`) | **tested (unit + in game)** | the unit `a_dark_tank_holds_sixteen_buckets`; AC's `0/16000`, `14000/16000`, `16000/16000` reads; `/data get block … fluids` |
| a filled extractor pours in, keeping what does not fit | **tested** | AC: 14,000 + 5,000 → tank 16,000, extractor 3,000; AD: fresh tank 0 + 5,000 → 5,000, extractor 0 |
| a standing click pours nothing; a crouching click on stone pours nothing | **tested** | AD, both directions |
| the saved tank survives in the block entity's own compound | **tested** | AC: `204, 101, 6 has the following block data: [{Amount: 16000, Fluid: "evilcraft:blood"}]` |
| `copy_tank_data` on the tank (the tank in the dropped item) | **tested** | AE: 12,000 mB broken, carried, placed back at 12,000/16,000 |
| `/evilcraft darktank`, and `/evilcraft blood` reaching the 16,000 tank | **tested** | every readout and the harness set in AC/AE |
| the auto-supply toggle, the downward/held auto-fill, the item tooltip, the light emission | not ported | deviations 1, 3 (the container item capability the pin lacks) |
| the fluid-rendering model and the 14/16 shape/outline | not ported | deviation 2 (no block-entity renderer at this tier's shape) |
| the jar's `dark_tank` recipe and its three `special/` recipes | not ported | gap 1 (`dark_gem` not carried; crafting-special not expressible) |
| the purifier (`BlockPurifier` + `BlockEntityPurifier`) | **carried by slice 9** | the curse action, the blook and the two clicks; the other three actions are `blocked(…)` rows in slice 9's coverage table |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Slice 9 — the Blook, and the Purifier's curse action (`ToolBadEnchantPurifyAction`) (2026-11-23)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.9**.
The registry entry, the PR (#1680, retained: the publisher appends to it) and the ledger stay the
orchestrator's.

### The two steps, and why this family

Slice 8's row, verbatim:

> | the purifier (`BlockPurifier` + `BlockEntityPurifier`) | **blocked(no-purifier-action-table)** | the four
> `IPurifierAction` classes; needs `evilcraft:blook`, the potion entity query and the head progression |

The brief splits that row: **first** carry `evilcraft:blook` (the item, its model and texture, and
its own infusion recipe), because it is what the disenchant action binds to and the one piece the
package lacked; **then** carry the purifier with the single action of the four that is
self-contained, `ToolBadEnchantPurifyAction` — the curse remover. The other three stay named rows
below: `DisenchantPurifyAction` needs the blook **and** a full tank plus a random enchantment
generator over the item's component, `CollectPotionPurifyAction` needs a living-entity query in the
block's AABB, and `MobHeadDowngradePurifyAction` needs the Blood Infuser's head progression.

**Tier:** `deep` still governs the port; this slice is a `machines` family inside it — a block
entity with two slots and a three-bucket tank, a content menu (none: upstream has no GUI), a tick,
an action clock, and two click paths. No entity, renderer or dimension.

### Upstream source read (analysis only, not copied)

`ports/_shared/upstream/evilcraft`, tag `1.21.1-1.2.100`, commit `aa81220`:

- `block/BlockPurifier.java`, `block/BlockPurifierConfig.java`
- `blockentity/BlockEntityPurifier.java`, `BlockEntityPurifierConfig.java`
- `blockentity/tickaction/purifier/{PurifierActionRegistry,ToolBadEnchantPurifyAction,DisenchantPurifyAction,CollectPotionPurifyAction,MobHeadDowngradePurifyAction}.java`
- `item/ItemBlookConfig.java`
- plus the jar's own `assets/evilcraft/{blockstates,models,textures}/…purifier…`, `…blook…`,
  `data/evilcraft/recipe/blood_infuser/base/blook.json`, `data/evilcraft/loot_table/blocks/purifier.json`,
  `data/minecraft/tags/block/mineable/pickaxe.json` and the lang keys.

The rule as the files state it: the registry is a **list of four actions** (`canWork` returns the
first index that applies, `work(i)` runs it); `ToolBadEnchantPurifyAction` takes any item in
`SLOT_PURIFY` carrying a `#minecraft:curse` enchantment, needs `getBucketsFloored() > 0`, counts to
`PURIFY_DURATION = 60`, and then sets each curse's level to `level - 1` and calls
`setBuckets(getBucketsFloored() - 1, getBucketsRest())` — **one bucket (1,000 mB) per curse**. The
block entity is `SLOTS = 2` (`SLOT_PURIFY = 0`, `SLOT_ADDITIONAL = 1`) with
`MAX_BUCKETS = 3`, and the block has **no menu**: `BlockPurifier.useItemOn` fills the slot from the
held item, `useWithoutItem` hands it back, and `FluidUtil` fills the tank from a blood container.

### What is in it

| Half | What | Source |
|---|---|---|
| Blook | `evilcraft:blook` (item, model, item model, the jar's own texture, lang, creative row) and its infusion `minecraft:book` → blook for 500 mB in 500 ticks at tier 1 | `ItemBlookConfig`, the jar's `base/blook.json` |
| Block | `evilcraft:purifier` — template `minecraft:stone` (`requires_tool`, `hardness`/resistance 2.5, `SoundType.STONE`), the jar's `minecraft:mineable/pickaxe` entry, its loot table, the three textures on a cube | `BlockPurifierConfig`, the jar's files |
| Block entity | two slots and a **3,000 mB** tank, one saved `fluids` list | `BlockEntityPurifier.SLOTS`/`MAX_BUCKETS` |
| Click in | a standing right-click puts **one** item (components and damage included) into `SLOT_PURIFY` when it is empty, taking it from the hand | `BlockPurifier.useItemOn` |
| Click out | an empty-hand right-click hands the purge slot's stack back, then the additional slot's | the same, `isEmpty()` branches |
| Blood in | a **crouching** click with a filled Blood Extractor pours into the tank, keeping what does not fit | `FluidUtil.interactWithFluidHandler`, this port's gesture (the dark tank's) |
| Action | `runtime::purify_tick` + `runtime::purified`: 60 ticks, every curse loses one level, one bucket each, `canWork` refusing with no curse or under one bucket | `ToolBadEnchantPurifyAction` |
| Tank drop | a broken purifier's own item carries its tank (`copy_tank_data`'s route, slice 3) | the jar's loot table + `tank::Machine::Purifier` |
| Command | `/evilcraft purifier <pos>` (tank, both slots, `ench`, `curses`, `tick`), and `blood`/`load` reaching the machine | harness only (upstream has none) |

`runtime.rs` holds the rules (`PURIFIER`, `PURIFIER_TANK`, `PURIFY_DURATION`, `CURSES`,
`is_curse`, `purify_tick`, `purified`), `src/purifier.rs` the game side, and the tank's capacity is
in `tank::Machine::Purifier.capacity()`.

### TDD: the red proof, before the implementation

The four rules were stubbed (`is_curse` → `false`, `purify_tick` → `None`, `purified` → unchanged,
no curses) and their tests written; `sdk-test-red-slice9.log`:

```
test result: FAILED. 66 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_clean_stack_or_a_tank_under_one_bucket_never_works stdout ----
thread '…' panicked at src/runtime.rs:1407:9:
assertion `left == right` failed
  left: None
 right: Some((11, false))

---- runtime::tests::a_curse_takes_sixty_ticks_and_one_bucket_of_blood stdout ----
thread '…' panicked at src/runtime.rs:1392:9:
assertion `left == right` failed
  left: None
 right: Some((1, false))

---- runtime::tests::a_finished_purify_strips_one_level_of_every_curse_and_keeps_the_rest stdout ----
thread '…' panicked at src/runtime.rs:1418:9:
assertion `left == right` failed
  left: [("minecraft:vanishing_curse", 1), ("minecraft:sharpness", 3)]
 right: [("minecraft:sharpness", 3)]

---- runtime::tests::only_vanillas_two_curses_are_curses stdout ----
thread '…' panicked at src/runtime.rs:1439:9:
assertion failed: is_curse("minecraft:vanishing_curse")
```

The two data tests (`a_book_becomes_a_blook_for_five_hundred_mb` and
`every_carried_recipe_names_items_this_package_or_the_game_has`) pass against the delivered data —
they are the *keys resolve* gate the brief asks for: every carried infusion's input and product is
`minecraft:` or one of `runtime::CARRIED_ITEMS` (the extractor, the blook, the three promises), so
no recipe can name an id nobody registers.

Green: `sdk-test.log` — **71 passed, 0 failed** (63 before + 8), `sdk-clippy.log` — `cargo fmt
--check` clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 4/70** (66 passed, 4 failed) (`EXIT:101`) | `sdk-test-red-slice9.log` |
| tests (green) | `cargo test` | **71 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 558 API uses, **18 hooks**, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **221,406 B**, sha256 `e1266a7f…1568f`, `CMP-IDENTICAL`, 113 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91…` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8ac…` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (71 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 16 recipe/loot files: slice 1's one unverified `"rolls": 1.0`; **the new purifier table and the blook recipe `ok`** | job report |
| artifact record | `sdk-artifact-0.2.9.txt` | bytes/sha/entries/manifest, each packed `src/` file's sha, check and fits lines, the packed tree | `sdk-artifact-0.2.9.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-a1c6103432dc`, log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/`, `assets/` and `horizon-mod.toml` are
byte-identical to the port (`diff -r`/`cmp` empty, `version = "0.2.9"`, `sdk-export-check-0.2.9.log`).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** (A, A2, B–F, H–W, Y, Z, AA, AB, AC, AD, AE, **AF–AI**) | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=purifier` | **`FAIL` + `EXIT:1`** at AG (*the pickaxe still carries 1 curse(s) after 140 ticks*) | `e2e-launcher-selftest-purifier.log` |
| selftest (slices 2–8, history) | `SELFTEST=infuser|tank|blood|stain|fall|colossal|darktank` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=11 ok=11 missing-exit=0
bad-exit=0` (exit 0): the ten earlier legs stay green and the eleventh is this slice's.

Transcript of the new phases (verbatim from `e2e-launcher.log`):

```
== AF: a cursed tool goes into the Purifier's purge slot by a click ==
  Purifier [206, 101, -10]: tank 0/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=12270
  a fresh evilcraft:purifier at 206,101,-10 reads 0/3000 mB, purge empty
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 0/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 1, curses 1, tick 0, t=12340
  206, 101, -10 has the following block data: {components: {"minecraft:enchantments": {"minecraft:vanishing_curse": 1}}, count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}
  the saved pickaxe carries the curse
== AG: sixty ticks of the action strip the curse for exactly one bucket ==
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 1, curses 1, tick 18, t=12382
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 0, curses 0, tick 0, t=12536
  206, 101, -10 has the following block data: {count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}
  the curse is gone from the saved pickaxe and the tank fell 3000 -> 2000 mB (one bucket)
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=12588
  the empty hand took the purified pickaxe back out of the purge slot
== AH: the same cursed pickaxe in a Blood Chest keeps its curse (the differential) ==
  Blood Chest [206, 101, -12]: tank 1000/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0], t=12704
  Blood Chest [206, 101, -12]: tank 1000/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0], t=12864
  206, 101, -12 has the following block data: {components: {"minecraft:enchantments": {"minecraft:vanishing_curse": 1}}, count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}
  a Blood Chest for 140 ticks: the pickaxe still carries the curse
== AI: a standing extractor click loads the slot, the empty hand takes it back, the crouch pours ==
  Blood Extractor held: 5000/5000 mB, t=12972.
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge evilcraft:blood_extractor@1, add empty@0, ench 0, curses 0, tick 0, t=13010
  standing: the extractor went into the purge slot and the tank stayed at 2000 mB
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=13048
  Blood Extractor held: 5000/5000 mB, t=13060.
  Blood Extractor held: 5000/5000 mB, t=13108.
  crouch + right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=13158
  Blood Extractor held: 4000/5000 mB, t=13170.
  206, 101, -10 has the following block data: 3000
  extractor 5000 -> 4000 mB, purifier tank 2000 -> 3000 mB (the 1,000 mB room, the rest kept)
```

and, in the same run's phase L: `minecraft:book -> evilcraft:blook x1 for 500 mB in 500 ticks` /
`took 1 x evilcraft:blook out of the result slot into inventory slot 10` — the blook is crafted
through the Blood Infuser with the jar's own recipe, which is also what proves its keys resolve.

**Fail-capable, four ways.** (1) `SELFTEST=purifier` inverts AG's curse-gone assertion and exits 1
(log above). (2) AH is the differential: the *same* cursed pickaxe, the same 140 ticks and a
working Blood Chest leave the curse on the item — read from the chest's own saved `Items[0]` — and
spend nothing, so AG's result can only be the purifier's action. (3) AF is AG's positive control:
the same pickaxe's saved stack carries `minecraft:enchantments` *before* the action, so "no
enchantments" afterwards is not a stack that never had one. (4) The whole-package control leaves
every id unknown, so AF cannot even place the purifier.

### What the game measured, and the four API/harness facts it cost

1. **26.3's `minecraft:enchantments` component is the bare map.** The 1.21.1 shape
   `{levels: {"minecraft:vanishing_curse": 1}}` is **refused by this build's `/give`**:
   *"Malformed 'minecraft:enchantments' component: 'Not a number; Failed to get element
   minecraft:levels from registry minecraft:enchantment'"*. The accepted and written shape is
   `{"minecraft:vanishing_curse": 1}` (`horizon-items`' `levels_tag` writes it and its
   `read_levels` still *reads* the old wrapper). The brief's assertion syntax uses the old shape;
   the artefact wins, and this port reads both and writes 26.3's.
2. **SNBT's unquoted keys may not contain a colon** (`Reader::is_allowed_in_unquoted`), so an
   enchantment id inside a component is written `{"minecraft:vanishing_curse":1}`, never
   `{minecraft:vanishing_curse:1}`.
3. **A right-click reaches `use_item_on` twice, and the second one is the off hand.** This hook is
   server-only, so the clicking client cannot predict it: its main-hand attempt passes and it sends
   a second click for the off hand, which took the item straight back out of the slot the first
   click had filled (measured: `use_item_on held=minecraft:diamond_pickaxe hand=Main` then
   `use_item_on held=none hand=Off` in the same tick). The block Part therefore answers for
   `Hand::Main` only.
4. **`None` from `use_item_on` means `TRY_WITH_EMPTY_HAND`, not `Pass`.** A refused insert that
   answered `None` made the engine run `use_without_item`, which handed the *occupant* of the slot
   into the player's hand, overwriting what they held. A held-item click this machine declines must
   answer `Some(Pass)`.
5. **A crouching right-click reaches an item Part, not a block Part.** The purifier's pour worked
   only once it moved to an `ItemPart` on the extractor, as the stain's draw and the dark tank's
   fill already were: a crouching click ran **no** block hook at all (measured: the pour's probe
   printed nothing while the item Part's printed `crouching=true blood=5000`).
6. **An item Part that declines must answer `None`, not `Pass`.** The first Part to answer wins,
   so the purifier's `Pass` stopped the stain's Part from drawing (phase S went red until it
   returned `None`).

Three harness facts learned here too:

1. **`clear @s` leaves the clicking client's held stack stale**, and a right-click with a stale
   stack sends no use packet at all: `empty_hand` clears, then selects another hotbar slot, and
   fails if the hand still reads occupied.
2. **A click that follows the machine's own write to the player's hand carries the client's older
   stack** (the same crouching click poured 0 mB straight after the take-back), so a pour is
   measured with an extractor the command gave, as phase AC does.
3. **`open_menu` stands the player at `$z.5`, which for a negative `z` is the cell *south* of the
   block** — a chest on the arena's edge (z = -14) put the player in the void, and the click missed.
   The differential chest sits two cells from the purifier for that reason.
4. Two pre-existing flakes were made deterministic in the shared helpers: `place_chest` now clears
   non-player entities within 8 blocks (an End enderman in the cell refuses the placement) and
   retries the click three times before falling back to a plain right-click (a dropped click reads
   as "the block did not change"); the arena floor runs six cells west of X0, because
   `break_survival` stands at `x - 3` and the Colossal Blood Chest's breaks were made from outside
   it; and phase R counts zombies within 30 blocks, since its target is summoned 19.9 blocks from
   where the player stands.

### Deviations this slice adds

1. **No menu, and no per-slot filter.** Upstream's purifier has no GUI — the click is its whole
   interface — and the pin's content container has no per-slot placement rule, so the purge slot
   takes one item of any kind and the machine only *acts* on a cursed one. Upstream's registry
   answers `isItemValidForMainSlot` for any item too (through `DisenchantPurifyAction.isAllowed`),
   so no player is blocked; what is missing is the disenchant *effect* (row below).
2. **The additional slot is never filled by a click.** Only the three actions this package does not
   carry accept an item there (a Blook, a glass bottle, a head), so the slot exists (upstream's
   `SLOTS = 2`) and the readout prints it, but nothing puts anything in it except `/evilcraft load`.
3. **The gestures are the port's.** Blood goes in by a crouching extractor click (the dark tank's
   and the stain's container gesture, since this pin's capability items carry water and lava only),
   and the item goes in and out by a standing click. Upstream hands the tank to `FluidUtil` on any
   right-click with a fluid container.
4. **The block is a cube over the jar's textures**, not upstream's cauldron-shaped model and
   outline: a content block cannot declare per-state shapes without a state property, and a
   cauldron *template* would bring the cauldron's own water interactions with it.
5. **The curses are the two vanilla ids, named as constants.** `EnchantmentTags.CURSE` is a
   `#minecraft:curse` enchantment tag, and the pin's tag reads cover the block, item and fluid
   registries only (`Tags`), so a data pack that tags another enchantment as a curse is not heard.
   `horizon-mod-vocab`'s `ItemEnchantments` is the `get_enchantment_level` event's mutable view, not
   a component reader, so the component is read as the compound it is.
6. **The item's component is read from its own patch.** An enchantment that lived in an *item
   default* rather than the stack's patch would not be seen — no vanilla tool ships one, and the
   purifier's whole input is a `/give`n or found stack.
7. **No particles, no action-bar readout, no comparator signal.** `showEffect`/`showEnchantedEffect`,
   the `enabled` toggle and `getAnalogOutputSignal` are client or redstone surfaces this port does
   not carry for any machine.

### Upstream defects fixed

None new. (The shadowed leather recipe, the 25,000 mB recipe, the missing pickaxe tag and
`copy_tank_data`'s applier are the earlier slices'.) One thing recorded rather than "fixed": the
purifier's registry comment in `BlockPurifierConfig` says the enchantment blacklist disqualifies
curses from purification, and it is empty by default — the port carries no blacklist, which is the
same behaviour with the shipped defaults.

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `evilcraft:blook` item, model, item model, texture, lang, creative row | **tested** | L: crafted in the Blood Infuser and taken out of the result slot |
| the blook's own infusion (`base/blook.json`: book → blook, 500 mB, 500 ticks, tier 1) | **tested** | L (`minecraft:book -> evilcraft:blook x1 for 500 mB in 500 ticks`) + the unit `a_book_becomes_a_blook_for_five_hundred_mb` |
| every carried recipe's keys resolve to an item the game or this package registers | **tested** | unit `every_carried_recipe_names_items_this_package_or_the_game_has`; in game, `recipes 10/10` in force and the blook crafted |
| `purifier` block, item, blockstate, model, textures, lang, creative row, loot table, `minecraft:mineable/pickaxe` | **tested** | AF places it from its own item (`ctl place`) and reads it back at `0/3000 mB` |
| the two slots and the **3,000 mB** tank | **tested** | AF/AG/AI readouts (`0/3000`, `3000/3000`, `2000/3000`); `/data get block … fluids` reads 3000 |
| a standing click puts one item in the empty purge slot, components included | **tested** | AF: the cursed pickaxe, its saved `Items[0]` carrying `minecraft:vanishing_curse`; AI: a filled extractor |
| an empty-hand click hands the item back | **tested** | AG: `purge empty@0` and the pickaxe back in the hand; AI: the extractor with its 5,000 mB intact |
| `ToolBadEnchantPurifyAction` — 60 ticks, one level per curse, one bucket each | **tested** | AG: curse gone, tank 3,000 → 2,000; the unit `a_curse_takes_sixty_ticks_and_one_bucket_of_blood` |
| a level-2 curse loses one level and stays | **tested (unit)** | `a_finished_purify_strips_one_level_of_every_curse_and_keeps_the_rest` |
| `canWork`: no curse, or under one bucket, does nothing | **tested (unit)** | `a_clean_stack_or_a_tank_under_one_bucket_never_works` |
| the tank filled by a crouching extractor click, keeping the rest | **tested** | AI: 2,000 → 3,000, extractor 5,000 → 4,000, `/data get block … fluids` 3000 |
| `copy_tank_data` on the purifier (the tank in the dropped block) | **tested (unit)** | `tank::tests::a_purifier_carries_its_three_buckets`; the in-game route is the two machines' and the dark tank's (O/P/AE) |
| `DisenchantPurifyAction` (a Blook in the additional slot, full tank, one enchantment moved) | **blocked(no-disenchant-generator)** | needs the blook (now carried) **and** the random-enchantment generator over `DataComponents.ENCHANTMENTS` plus a full-tank gate; not expressible as this port's single action |
| `CollectPotionPurifyAction` (a glass bottle and a living entity in the block's AABB) | **blocked(no-entity-aabb-query)** | the pin's `living` helpers carry no "entities in a box" query in a block tick |
| `MobHeadDowngradePurifyAction` (the head progression, full tank) | **blocked(no-head-progression)** | meaningful only with the Blood Infuser's head recipes this package does not carry |
| `ItemVengeancePickaxe` immunity | not ported | the item is not carried, so no stack can match it |
| the `enchantmentIdBlacklist` config | not ported | empty upstream by default; no config surface in a content mod |
| particles, action bar, comparator signal | not ported | deviations 7 |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 10 — the three review findings, and two data-loss bugs fixed (0.2.10)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.10**
on top of the published 0.2.9 (the publisher appends to the retained PR #1680; the registry entry,
the PR and the ledger are the orchestrator's).

This slice is a **review fix**, not a new family: three findings, the first two data-loss bugs in
shipped versions (0.2.5–0.2.8). No tier question arises — the port stays **content / deep**, and
the fixes are inside families already carried (the repair action, the extractor's stack shape).

### Finding 1 — the anvil step replaced the whole stack (`src/colossal.rs:100-117`, 0.2.7–0.2.8)

`AnvilRepairAction.repair` answers `new ItemStack(anvil)`: **one** item, whatever the stack held,
and `RepairItemTickAction` writes that answer back into the slot. The Java mod therefore turns a
stack of N broken anvils into one anvil and drops the other N−1 items *and every component of the
stack* (a custom name included) — and 0.2.7/0.2.8 reproduced it in `colossal.rs`, with the same
code in `blood.rs` for the plain Blood Chest.

**Rule adopted: preserve the count and components, and charge per item.** A stack of N broken
anvils becomes N of the next anvil state for **N × 125 mB**, its component patch copied over, and
a tank that cannot pay for all N leaves the stack **untouched** — no partial replacement, no lost
anvil. `runtime::anvil_stack_step(item, count, tank)` is the rule (`anvil_step` now delegates with
count 1, so the upstream price is unchanged); `blood::anvil_stack` builds the new stack with
`set_count` + `set_components(from.components().as_nbt())`. Both machines call it.

Anvils stack to 64 in vanilla, so this is a stack a player really has; a stack of two *pickaxes*
cannot exist at any legal stack limit (tools are max 1), which is why the in-game proof uses the
legal case (a custom-named chipped-anvil stack) plus an enchanted pickaxe for the component half.

### Finding 2 — the extractor was stackable (`src/stain.rs:229-246`, 0.2.5–0.2.8)

The amount is stored **once per stack** (`minecraft:custom_data.evilcraft:blood`), so a drain on a
stack of N debited the source once and wrote the result onto all N items; the next split of that
stack copied the amount onto each item (duplication), and a fill was counted once for N (loss). The
Java item declared no stack limit either, so upstream shares the defect.

**Rules adopted:**

1. **The item is unstackable**: `data/evilcraft/horizon/item/blood_extractor.json` declares
   `"max_stack_size": 1`.
2. **A legacy stack is resolved deterministically** (`runtime::legacy_extractors(count, blood)`):
   the **first** extractor keeps the whole stored amount (clamped to the item's 5,000 mB) and the
   others are empty. The sum equals what the stack stored and the item count is unchanged, so
   nothing is duplicated and nothing is lost. The stack's other components (a custom name) travel
   on each resolved item, as a vanilla split copies them.
3. **No drain path writes onto a stack of more than one** (`runtime::one_extractor(count)`): the
   stain draw, the dark tank pour, the purifier pour, the three machines' container slot and the
   purifier's insert all refuse one, and `fill_for` resolves before it fills.
4. **Where the stack is resolved**: the extractor's `ItemPart::inventory_tick` (server, per carried
   stack) writes the kept extractor back into its slot and `/give`s the empty extras back to the
   player (`PlayerRef::give_stack`, overflow dropped at their feet). A machine's container slot
   leaves a stacked extractor alone — the player takes it out and their inventory resolves it.

Cost: the tick Part runs only for stacks of this item, and returns after one count check for the
normal single extractor.

### Finding 3 — attribution of the carried infusion recipes

0.2.8's `LICENSE-NOTICE.md` credited the carried textures only; the **nine** carried
`convenience/` infusion recipes (the finding's count at 0.2.8) and the `base/blook.json` 0.2.9
added were uncredited. The notice now names **all ten** files, each verified `cmp`-identical to
`evilcraft-1.21.1-neoforge-1.2.100.jar`, and states the only change (the `convenience/` directory
flattened into `recipe/blood_infuser/`). The same pass added the Blook's and the Purifier's
textures, the `purifier` blockstate and the `dark_tank` item model to the "carried unchanged" list
(same class of omission).

### TDD: the red proof, before the implementation

`src/runtime.rs` gained both rules as stubs holding the old behaviour (`anvil_stack_step` charged
one item whatever the stack held; `legacy_extractors` → `None`; `one_extractor` → true for every
count) and the two tests. `sdk-test-red-slice10.log` (test step exit 101):

```
test result: FAILED. 71 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- runtime::tests::a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood stdout ----
thread 'runtime::tests::a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood' (1088259) panicked at src/runtime.rs:1550:51:
a stack of two resolves

---- runtime::tests::a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each stdout ----
thread 'runtime::tests::a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each' (1088269) panicked at src/runtime.rs:1521:9:
assertion `left == right` failed: two chipped anvils become two anvils for 250 mB
  left: Some(("minecraft:anvil", 125))
 right: Some(("minecraft:anvil", 0))

failures:
    runtime::tests::a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood
    runtime::tests::a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each
```

Green: `sdk-test.log` — **73 passed, 0 failed** (71 + 2); `sdk-clippy.log` — `cargo fmt --check`
clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 2/73** (71 passed, 2 failed), exit 101 | `sdk-test-red-slice10.log` |
| tests (green) | `cargo test` | **73 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 573 API uses, **19 hooks**, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **225,856 B**, sha256 `dbbdf5d8…4013ef`, **`CMP-IDENTICAL`**, 113 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (73 tests, 0 clippy warnings, runtime byte-identical, package matches the tree, check + fits, Skills line) | job report |
| artifact record | `sdk-artifact-0.2.10.txt` | bytes/sha/entries/manifest, every packed `src/` file's sha, check and fits lines, the packed tree | `sdk-artifact-0.2.10.txt` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → tag `native-0a875da-f804bd58891f`, log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/`, `assets/` and `horizon-mod.toml` are
byte-identical to the port (`diff -r`/`cmp` empty, `version = "0.2.10"`,
`sdk-export-check-0.2.10.log`).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1` for 0.2.3 and `LEGACY=1` for 0.2.9) | **`PASS` + `EXIT:0`** (the 0.2.3 and 0.2.9 legs, A, A2, B–F, **D2**, **D3**, **E2**, H–W, Y, **Y2**, Z, AA–AI) | `e2e-launcher-0.2.10.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control-0.2.10.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C (*the pickaxe was not repaired in 59 ticks*) | `e2e-launcher-selftest-0.2.10.log` |
| selftest (finding 1) | `SELFTEST=anvil` | **`FAIL` + `EXIT:1`** at D2 (*the repaired stack … count only 1*) | `e2e-launcher-selftest-anvil-0.2.10.log` |
| selftest (finding 2) | `SELFTEST=extractor` | **`FAIL` + `EXIT:1`** at E2 (*the legacy stack reads as 1 extractor slot(s)*) | `e2e-launcher-selftest-extractor-0.2.10.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=16 ok=16 missing-exit=0
bad-exit=0` (exit 0): the ten earlier legs stay green, and the four new logs are this slice's.

The **legacy save** is written by the 0.2.9 export (`legacy_leg`, a separate launch on the same
scratch world): 0.2.9's item declared no `max_stack_size`, so `/item replace … 2` + `/evilcraft
held 4000` leave one stack of two extractors carrying the amount once — exactly what an old save
holds. The main 0.2.10 launch continues that world, so the resolution under test is the real one.

Transcript of the new phases (verbatim from `e2e-launcher-0.2.10.log`):

```
== the previous package (0.2.9): a save with one stack of two Blood Extractors ==
  Blood Extractor held: 4000/5000 mB, t=96.
  0.2.9 save: one stack of 2 extractors carrying 4,000 mB once
== D2: a stack of broken anvils is repaired whole, one price each (review finding 1) ==
  Blood Chest [202, 101, 0]: tank 0/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0 2:minecraft:chipped_anvil@0], t=591
  Blood Chest [202, 101, 0]: tank 125/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0 2:minecraft:chipped_anvil@0], t=625
  202, 101, 0 has the following block data: {components: {"minecraft:custom_name": '"Two anvils"'}, count: 2, Slot: 2b, id: "minecraft:chipped_anvil"}
  125 mB: the stack of two chipped anvils is untouched
  Blood Chest [202, 101, 0]: tank 0/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0 2:minecraft:anvil@0], t=665
  202, 101, 0 has the following block data: {components: {"minecraft:custom_name": '"Two anvils"'}, count: 2, Slot: 2b, id: "minecraft:anvil"}
  two anvils for 250 mB: count 2
  the custom name travelled with both anvils
== D3: a damaged enchanted pickaxe is repaired, and keeps its enchantment ==
  Blood Chest [202, 101, 0]: tank 970/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0 2:minecraft:anvil@0 3:minecraft:diamond_pickaxe@244], t=730
  Blood Chest [202, 101, 0]: tank 835/10000 mB, container empty@0, slots [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0 2:minecraft:anvil@0 3:minecraft:diamond_pickaxe@217], t=783
  202, 101, 0 has the following block data: {components: {"minecraft:enchantments": {"minecraft:sharpness": 3}, "minecraft:damage": 211}, count: 1, Slot: 3b, id: "minecraft:diamond_pickaxe"}
  one pickaxe, repaired from damage 244 to 217
  the enchantment survived the repair
== E2: a legacy stack of two extractors is split before any drain (review finding 2) ==
  the 0.2.9 stack of two resolved into 2 single extractors (2 items)
  Blood Extractor held: 4000/5000 mB, t=815.
  Blood Extractor held: 0/5000 mB, t=825.
  placed evilcraft:blood_stain at 204,101,4
  crouch + right-click on 204,101,4 (aim on evilcraft:blood_stain)
  Blood Stain [204, 101, 4]: tank 2500/5000 mB, t=902
  Blood Extractor held: 5000/5000 mB, t=908.
  Blood Extractor held: 0/5000 mB, t=917.
  the crouching click drew 1,000 mB into one extractor (4,000 -> 5,000); the other still holds 0 mB
...
== Y2: the Colossal Blood Chest repairs a whole anvil stack, one price each ==
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 125/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@0 1:minecraft:chipped_anvil@0], t=11911
  198, 101, -10 has the following block data: {components: {"minecraft:custom_name": '"Colossal pair"'}, count: 2, Slot: 1b, id: "minecraft:chipped_anvil"}
  125 mB: the colossal's stack of two is untouched
  Colossal Blood Chest [198, 101, -10]: formed=true, tank 0/10000 mB, repair 45, slots 46, container empty@0, [0:minecraft:diamond_pickaxe@0 1:minecraft:anvil@0], t=11947
  198, 101, -10 has the following block data: {components: {"minecraft:custom_name": '"Colossal pair"'}, count: 2, Slot: 1b, id: "minecraft:anvil"}
  the colossal converted both anvils (count 2)
  the custom name travelled with both anvils
```

**Fail-capable, four ways.** (1) `SELFTEST=anvil` inverts D2's count and exits 1 — the log above;
the same inversion of any single-item step would pass (a tank of 125 leaves the stack alone, which
the same phase asserts, so a one-item implementation cannot satisfy both). (2) `SELFTEST=extractor`
inverts E2's split and exits 1. (3) E2's drain is the differential for "not onto a second": the same
click fills one extractor to 5,000 and the other still reads 0. (4) The whole-package control has no
extractor, anvil or stain, so none of the phases can even run.

### Upstream defects fixed

1. **`AnvilRepairAction` destroys a stack of anvils** (one item returned for N, components lost) —
   proved from the class (`return Pair.of(25F, new ItemStack(itemStack.getItem() == Items.CHIPPED_ANVIL
   ? Items.ANVIL : Items.CHIPPED_ANVIL));`) and from `RepairItemTickAction.onTick`, which writes
   the answer back into the slot. The port keeps the count and the components and charges 125 mB
   each; a user notices that a stack of anvils is no longer swallowed. Worth an upstream report.
2. **The Blood Extractor stacks and shares one amount** (`ItemBloodExtractor`'s `Item.Properties()`
   sets no `stacksTo`; NeoForge's per-stack fluid handler then fills the whole stack once). The port
   makes the item unstackable and resolves a legacy stack losslessly; a user notices two extractors
   instead of one stack, and never a second extractor carrying the first one's blood.

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| the Blood Chest's anvil step on a **stack** | tested | D2: 2 chipped anvils → 2 anvils for 250 mB, custom name kept; 125 mB leaves them; unit `a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each` |
| the Colossal Blood Chest's anvil step on a stack | tested | Y2: the same pair on the machine the finding names (the two machines are separate code paths) |
| a damaged stack's repair keeps count and components | tested | D3: a count-1 enchanted pickaxe keeps `minecraft:enchantments` (sharpness 3) through 27 damage points; D2 keeps the anvil stack's count and `custom_name` |
| the extractor's stack shape | tested | the item declares `max_stack_size: 1`; E2's resolution and the 0.2.9 save that produced the stack |
| a legacy extractor stack (a pre-fix save) | tested | E2: 2 singles, 4,000 + 0, then a drain into one (5,000) leaves the other at 0; unit `a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood` |
| the carried infusion recipes' attribution | tested (file bytes) | `LICENSE-NOTICE.md` names the ten files; each `cmp`-identical to the 1.2.100 jar |
| the extractor in a machine container slot | **refused, not split** | deviation 1 below: the player takes it out and the inventory resolves it (unit `one_extractor`) |

### Deviations this slice adds

1. **A machine's container slot does not resolve a legacy stack.** It has no player to hand the
   extra items to, and it must not read the one amount as N items; it leaves the stack alone (no
   blood moves) until the player takes it out. Upstream had no legacy state to handle at all.
2. **The resolution's extras are `/give`n**, so a full inventory drops them at the player's feet
   (vanilla's `/give` overflow) rather than somewhere the port invents.

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog for the record (0.2.10, verbatim)

> 0.2.10 — review fixes on top of 0.2.9. The Blood Chest and the Colossal Blood Chest no longer
> replace a stack of broken anvils with one anvil: a stack of N becomes N anvils for 125 mB each,
> with its components kept, and a tank that cannot pay for all N leaves the stack untouched. The
> Blood Extractor is unstackable (`max_stack_size: 1`); a stack from an older save is resolved
> before any drain — the first extractor keeps the whole stored amount, the others are empty and
> handed back — so no blood is duplicated and no item is lost, and no drain path ever writes onto
> a stack of more than one. `LICENSE-NOTICE.md` now credits all ten carried upstream Blood Infuser
> recipe files (nine `convenience/` files, their directory flattened, and `base/blook.json`), each
> byte-identical to the jar, and the Blook's and Purifier's textures and two more upstream
> blockstates/models. 73 tests, two identical builds, the in-game leg green (two new phases on the
> anvil stack and the legacy extractor stack, plus two selftests that fail as they must).

## Slice 11 — the Vengeance Pickaxe's purifier immunity (0.2.11)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.11**
on top of the published 0.2.10 (an Update of our own entry; the registry entry, the PR and the
ledger are the orchestrator's). The full design, including the alternative slices considered, is
in `tools/slice11-plan.md`.

### The gap this slice closes — and a correction to the brief

From slice 9's coverage table:

> | `ItemVengeancePickaxe` immunity | not ported | the item is not carried, so no stack can match it |

The dispatch note guessed a damage-source immunity ("a named attacker wearing/holding it is
immune to the pickaxe's revenge damage") and suggested naming a missing damage-source symbol.
**The artefact disagrees, and wins.** Read from the upstream clone
(`ports/_shared/upstream/evilcraft`, tag `1.21.1-1.2.100`, `ToolBadEnchantPurifyAction.java:40,73`),
`canWork` and `work` both guard every curse with
`enchantment.is(EnchantmentTags.CURSE) && !(purifyItem.getItem() instanceof ItemVengeancePickaxe)`:
the immunity is **the Purifier's curse action refusing to strip curses from the Vengeance Pickaxe**.
`ItemVengeancePickaxe.java` only sets a 1.25× destroy speed and the enchanted stack it is crafted
as (`minecraft:fortune` 5 + `evilcraft:vengeance` 3); `evilcraft:vengeance` is in the jar's
`data/minecraft/tags/enchantment/curse.json`. So no damage hook, HurtSource query or entity is
needed — none was invented, and no `blocked(<symbol>)` applies.

**Tier:** `deep` still governs the port; this slice is a `logic` family inside it — one carried
item and one rule on the machine slice 9 already ships. No block entity, entity, renderer or
dimension added.

### What is in it

| Half | What | Source |
|---|---|---|
| Item | `evilcraft:vengeance_pickaxe`, template `minecraft:diamond_pickaxe`, its 101 B item model and 254 B texture carried byte for byte, two lang keys, creative row | jar `assets/evilcraft/models/item/vengeance_pickaxe.json`, `textures/item/vengeance_pickaxe.png`, `en_us.json`; `ItemVengeancePickaxe` |
| Rule | `runtime::purifiable_curses(item, levels)` and `purified_for(item, levels)`: a Vengeance Pickaxe has **zero** purifiable curses, so `canWork` resets the clock and spends no blood, and `work` would keep every enchantment | `ToolBadEnchantPurifyAction.canWork`/`work` |
| Glue | the purge slot stack's id is read through `stack.item().id(host)` in the block-entity tick and passed to both rules; the harness readout still shows the real enchantment/curse count, so the immunity is visible | `src/purifier.rs` |

Nothing else changed: the curse action's 60-tick duration, 1,000 mB bucket and the two vanilla
curses are slice 9's. The crafting recipe is **not** carried (gap 1): it needs
`evilcraft:hardened_blood_shard` and `evilcraft:dark_stick`, which this port does not have.

### TDD: the red proof, before the implementation

`sdk-test-red-slice11.log` (the two tests written, `purifiable_curses`/`purified_for` stubs
ignoring the item id — the old behaviour):

```
test result: FAILED. 74 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_vengeance_pickaxes_curse_is_immune_to_the_purifier stdout ----
thread 'runtime::tests::a_vengeance_pickaxes_curse_is_immune_to_the_purifier' (165696) panicked at src/runtime.rs:1524:9:
assertion `left == right` failed
  left: 1
 right: 0
```

The differential test (`the_immunity_is_only_for_the_vengeance_pickaxe`) passed against the stub
by construction — it is the boundary a *wrong* implementation breaks, not one the stub breaks.
Green: `sdk-test.log` — **75 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases, job `G12-evilcraft`, no `BUILD_LOCAL=1`, no `--target-dir`)

Compile steps were forwarded: `build-slot: forwarding test for ports/evilcraft to the remote
runner`, slot 1/5 / slot 2/5, remote box **AdLf-PC** (`/home/deepseek/atm10`, rust 1.95).
The e2e legs stay local: `--step e2e` is not a forwarded step, and the launcher's game export
needs the prepared 26.3 resources on this box.

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` | **FAILED 1/75** (`EXIT:101`) | `sdk-test-red-slice11.log` |
| tests (green) | `cargo test` | **75 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| launcher game | `ports/tools/launcher-game.sh build evilcraft` | `native-0a875da-93e85970f01e`, `Compiling evilcraft (1 of 1)`; export `src/`, `data/`, `assets/`, `horizon-mod.toml` byte-identical to the port | `launcher-build-0.2.11.log` |
| default leg | `tools/e2e.sh` (`OLD=1`, `LEGACY=1`) | **`PASS` + `EXIT:0`** — every earlier phase plus AJ | `e2e-launcher-0.2.11.log` |
| control | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — no chest item | `e2e-launcher-control-0.2.11.log` |
| selftest | `SELFTEST=vengeance` | **`FAIL` + `EXIT:1`** at AJ — *the Vengeance Pickaxe carries 0 curse(s), expected its curse kept* | `e2e-launcher-selftest-vengeance-0.2.11.log` |
| strict logs | `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` | `logs=19 ok=19 missing-exit=0 bad-exit=0`, exit 0 | job report |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **228,363 B**, sha256 `e80f46bf…25d`, **`CMP-IDENTICAL`**, 117 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, exit 0 | `sdk-check-fits.log` |
| fits, the pin | `… fits … --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, exit 0 | `sdk-check-fits.log` |
| fits, another build | `… fits … --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, exit 0 | `sdk-check-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (75 tests, 0 warnings, runtime byte-identical, package matches the tree, R9, no lock, check + fits, Skills line) | `sdk-artifact-0.2.11.txt` |

### The in-game transcript (verbatim, `e2e-launcher-0.2.11.log`)

```
== AJ: a Vengeance Pickaxe's curse is immune to the Purifier's action ==
  evilcraft:no_such_item is not an item this game has.
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14387
  the carried evilcraft:vengeance_pickaxe reads back in hand
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge evilcraft:vengeance_pickaxe@1, add empty@0, ench 1, curses 1, tick 0, t=14445
  the saved Vengeance Pickaxe carries the curse
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge evilcraft:vengeance_pickaxe@1, add empty@0, ench 1, curses 1, tick 0, t=14613
  the Vengeance Pickaxe kept its curse and the tank stayed 3000/3000 mB after 140 ticks
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 1, curses 1, tick 20, t=14765
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 0, curses 0, tick 0, t=14919
  the same curse on a vanilla pickaxe was stripped for one bucket (the item is the difference)
```

`SELFTEST=vengeance` ends `EXIT:1` at AJ (*after 140 ticks the Vengeance Pickaxe carries 0
curse(s), expected its curse kept*); the whole-package control ends `EXIT:1` when the chest item
does not register. The vanilla half of AJ is the differential: the same machine, tank and
140 ticks strip the curse from an ordinary pickaxe and spend 1,000 mB.

**A harness trap fixed while writing this phase:** the first AJ draft called `saved_item`
after the 140-tick wait, but that helper waits for a *changing* `/data get` reply — an immune
stack's saved bytes deliberately never change, so it timed out (`no data get reply for Items[0]`)
even though the product behaviour was already correct (`tank 3000`, `curses 1`, `ench 1`, `tick 0`).
The phase now reads the server-side `/evilcraft purifier` readout after the wait, which is the
same saved data and does change its `t=` suffix. The first `saved_item` call still proves the
curse is on the stack before the wait.

### Changelog for the record (0.2.11, verbatim)

> 0.2.11 — the Vengeance Pickaxe is carried (the jar's own model, texture and lang) and its
> immunity is honoured: upstream's Purifier curse action refuses that item, so a cursed pickaxe in
> the Purifier keeps its curse, its full 3,000 mB tank and a reset action clock over 140 ticks,
> while the same curse on a vanilla pickaxe is stripped for one bucket. The dispatch note's guess
> of a damage-source immunity was corrected from the artefact: the real rule is an item-id guard
> in `ToolBadEnchantPurifyAction`, so no damage hook or HurtSource query is needed. 75 tests, two
> identical builds, the in-game leg green with a new fail-capable phase AJ plus a control and a
> `SELFTEST=vengeance` that end `EXIT:1`.

### Coverage table — the row this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `ItemVengeancePickaxe` immunity | **tested** | AJ: a full purifier and 140 ticks leave the pickaxe with 1 enchantment, 1 curse, the tank at 3,000/3,000 and the clock at 0; the same cursed vanilla pickaxe loses its curse and spends one bucket; unit `a_vengeance_pickaxes_curse_is_immune_to_the_purifier` |
| `evilcraft:vengeance_pickaxe` item | **tested** | AJ: given from its own id, read back in hand, inserted by a click and read from the saved `Items[0]` |
| the jar's `vengeance_pickaxe` recipe | not ported | gap 1: its ingredients (`hardened_blood_shard`, `dark_stick`) are not carried |
| the `evilcraft:vengeance` enchantment and its spirit-summoning effect | not ported | Java entity effect, not expressible as content; the immunity is an item-id rule, so the in-game proof uses a vanilla curse |
| particles, action bar | not ported | deviation 7 of slice 9 (unchanged) |

### Deviations this slice adds

1. **The item's 1.25× destroy speed and its `evilcraft:vengeance` enchantment are not carried.**
   A content item with the `minecraft:diamond_pickaxe` template cannot set either; the jar's
   model and texture are carried faithfully, and the in-game proof puts a vanilla curse on it so
   the Purifier's own curse path runs. The immunity itself is the jar's item-id check.
2. **A re-set of an equal tank value answers `has no blood tank` at this pin** (a `merge_data`
   no-op), so the scenario relies on the tank already being full from phase AI rather than
   re-setting it; the readout asserts it is 3,000 before and after the wait.

### Upstream defects fixed

None new. (`copy_tank_data`'s applier, the shadowed leather recipe, the 25,000 mB recipe, the
missing pickaxe tag and the two slice-10 data-loss fixes are the earlier slices'.)

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 12 — the tank blocks' analogue (comparator) output (0.2.12)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher 0.2.92) +
mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3. Package **0.2.12**
on top of 0.2.11 (an Update of our own entry; the registry entry, the PR and the ledger are the
orchestrator's). The design and the alternative considered are in `tools/slice12-plan.md`.

### The gap this slice closes, and why this family

Slice 9's coverage table, verbatim:

> | particles, action bar, comparator signal | not ported | deviations 7 |

The **comparator signal** half of that row is the one expressible at this pin: `BlockPart::has_analog_output`
and `BlockPart::analog_output` exist in SDK `0a875da`, and powah 0.2.14/0.2.15 proved the route in
game at this exact pin. The two other halves are client surfaces (a particle spawn and an action-bar
readout) that a content mod with no renderer cannot express; they remain `not ported`. No new block,
block entity, entity or renderer is needed, so the slice stays inside tier **deep** — it is the
`logic`/redstone half of machines already carried.

### The fraction rule, read from the artefact (and a correction to the brief)

Upstream clone `ports/_shared/upstream/evilcraft`, tag `1.21.1-1.2.100`, commit `aa81220`:

- `block/BlockPurifier.java:120-124`, `block/BlockDarkTank.java:87-91` and
  `core/block/BlockWithEntityGuiTank.java:48-52` (the Blood Chest's, Infuser's and Colossal's base)
  all read the block entity's tank and answer
  `(int)Math.ceil(MinecraftHelpers.COMPARATOR_MULTIPLIER * amount / capacity)`.
- CyclopsCore `MinecraftHelpersCommon.getComparatorMultiplier()` returns **15** (read from
  `CyclopsMC/CyclopsCore`, branch `master-26-lts`, `loader-common/.../helper/MinecraftHelpersCommon.java`),
  confirming the multiplier.

So the rule is **`ceil(15 × stored / capacity)`**, clamped into 0..=15 and 0 for an empty or
invalid tank: a full Purifier (3,000/3,000) reads **15**, 2,000/3,000 reads **10**, 1,800/3,000
reads **9**, 6,000/10,000 reads 9, an empty tank reads 0.

**The dispatch note says 2,000 mB reads 9. The artefact disagrees and wins** (AGENTS.md: when a
brief and the artefact disagree, the artefact wins and the agent reports it): 15 × 2,000/3,000 is
exactly 10, so `ceil` is 10, not 9. The port carries the artefact's rule; the 9 is still exercised
in game at 1,800/3,000 mB, where the same rule really gives 9.

### The cache / update route

The comparator hooks receive only a read-only `LevelHost`, which cannot reach a block entity, so
the reading cannot be computed live there (the same gap powah 0.2.14 recorded):

1. `src/runtime.rs::comparator_strength(stored, capacity)` is the pure arithmetic; it has three
   new unit tests and no game types.
2. `src/redstone.rs` keeps a `(LevelKey, [i32;3]) → i32` **cache** of the last strength per world
   and position, and a `LevelRef → BTreeSet<pos>` **queue** of machine ticks. A static
   `OnceLock<Mutex<…>>` holds each, cleared by `world_lifecycle` so no reading outlives a world.
3. Each ticking machine's `block_entity_tick` (chest, infuser, colossal, purifier) queues its
   position. `Hooks::level_tick_post` drains the queue where every machine is back in the world,
   reads its saved tank, records the strength, and on a change calls
   `LevelRef::update_neighbors(host, pos, state, NeighbourUpdate::OutputSignal)` — vanilla's
   `setChanged` → `updateNeighbourForOutputSignal`, which a tank changing without a block-state
   change does not get on its own.
4. The harness `/evilcraft blood` setter and the dark tank's / purifier's crouching pours publish
   immediately, since they change a tank outside a block-entity tick.
5. `has_analog_output`/`analog_output` are added to the existing `BlockPart`s of the five tank
   blocks (chest, infuser, colossal, purifier, dark tank); the answer comes from the cache. The
   blood stain is deliberately not a comparator block upstream.

### TDD: the red proof, before the implementation

`sdk-test-red-slice12.log` (the three rule tests written with `comparator_strength` stubbed to `0`):

```
test result: FAILED. 75 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_doctored_tank_reading_is_clamped_into_zero_to_fifteen stdout ----
thread 'runtime::tests::a_doctored_tank_reading_is_clamped_into_zero_to_fifteen' (284655) panicked at src/runtime.rs:1690:9:
assertion `left == right` failed
  left: 0
 right: 15

---- runtime::tests::a_full_tank_reads_fifteen_and_an_empty_one_reads_zero stdout ----
thread 'runtime::tests::a_full_tank_reads_fifteen_and_an_empty_one_reads_zero' (284666) panicked at src/runtime.rs:1672:9:
assertion `left == right` failed
  left: 0
 right: 15

---- runtime::tests::the_comparator_rounds_up_the_tank_fraction stdout ----
thread 'runtime::tests::the_comparator_rounds_up_the_tank_fraction' (284699) panicked at src/runtime.rs:1681:9:
assertion `left == right` failed
  left: 0
 right: 10
```

Green: `sdk-final-check-0.2.12.log` — **80 passed, 0 failed** (75 + 3 rule tests + 2 cache tests),
`cargo fmt --check` clean and `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (`build-slot.sh` leases; compile steps forwarded to **AdLf-PC**, game legs local)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` (rule stubbed to 0) | **FAILED 3/78** (`EXIT:101`) | `sdk-test-red-slice12.log` |
| tests (green) | `cargo test` | **80 passed, 0 failed** | `sdk-final-check-0.2.12.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-final-check-0.2.12.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, **629 API uses, 23 hooks**, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **234,814 B**, sha256 `d203fad4…b4e78`, **`CMP-IDENTICAL`**, 118 entries, no `Cargo.lock` | `sdk-build-check-fits-0.2.12.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `EXIT:0` | `sdk-build-check-fits-0.2.12.log` |
| fits, the pin | `… fits … --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `EXIT:0` | `sdk-build-check-fits-0.2.12.log` |
| fits, another build | `… fits … --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `EXIT:0` | `sdk-build-check-fits-0.2.12.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (runtime byte-identical, package matches the tree, R9, no lock, check + fits, Skills line) | `sdk-gate-0.2.12.log` |

### The in-game leg (`evidence/evilcraft-e2e/`, launcher-built game at the pin)

`ports/tools/launcher-game.sh build evilcraft` → tag **`native-0a875da-ddf8ad0a3724`**, log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/`, `assets/` and `horizon-mod.toml` are
byte-identical to the port (`diff -r`/`cmp` empty, `version = "0.2.12"`).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (`OLD=1` for 0.2.3, `LEGACY=1` for 0.2.9) | **`PASS` + `EXIT:0`** — every earlier phase plus **AK** | `e2e-launcher-0.2.12.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control-0.2.12.log` |
| selftest (this slice) | `SELFTEST=comparator` | **`FAIL` + `EXIT:1`** at AK — *SELFTEST flipped: the comparator read 10 at 2,000 mB, as it should* | `e2e-launcher-selftest-comparator-0.2.12.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=22 ok=22 missing-exit=0
bad-exit=0` (exit 0): the 19 earlier legs stay green and the three 0.2.12 logs are this slice's.

Transcript of the new phase (verbatim from `e2e-launcher-0.2.12.log`):

```
== AK: a comparator reads the Purifier's tank fraction (the analogue output) ==
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14776
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14804
  purifier 3000/3000 mB: comparator powered=true, wire reads 15
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14882
  purifier 2000/3000 mB: wire reads 10 (ceil(15 x 2000 / 3000) = 10)
  purifier 1800/3000 mB: wire reads 9 (ceil(15 x 1800 / 3000) = 9)
== AK: a comparator facing vanilla stone reads 0 (the vanilla differential) ==
  stone: comparator powered=false, wire reads 0
```

**Fail-capable, three ways.** (1) `SELFTEST=comparator` inverts AK's 2,000 mB equality and exits 1.
(2) The stone half is the vanilla differential on the same rig: a comparator facing a block with
no analogue output stays unpowered and its wire reads 0. (3) The whole-package control has no
purifier, so AK cannot even place one. The 15 → 10 → 9 sequence is also the counterfactual for a
stale cache: without the `OutputSignal` notification the wire would keep reading 15 after the
harness changed the tank (the first draft's helper-position bug showed the opposite failure —
empty readings — and was fixed in the harness, not the mod).

### A harness trap this slice cost

The first draft's `cmp_signal`/`cmp_powered` took the **comparator's** position (powah's
convention), while AK passes the **machine's**; the result was empty `powered`/`power` values and
`EXIT:1` at AK. The helpers now explicitly take the machine position (wire at z−2, comparator at
z−1) and the comment at their definition records the difference. The mod's first run otherwise
reached AK with all 36 earlier phases green.

### Coverage table — the row this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `hasAnalogOutputSignal` on the five tank blocks | tested | AK: a comparator beside the Purifier is `powered=true`; beside `minecraft:stone` it is not |
| `getAnalogOutputSignal` = `ceil(15 × amount/capacity)` | tested | AK: 15 at 3,000/3,000, 10 at 2,000/3,000, 9 at 1,800/3,000; unit `the_comparator_rounds_up_the_tank_fraction` |
| the changed-reading notification (`updateNeighbourForOutputSignal`) | tested | AK: 15 → 10 → 9 re-read on the wire with no nudge between the readings |
| the vanilla differential | tested | AK: the same comparator rig facing `minecraft:stone` reads 0 |
| particles and the action-bar readout (the other halves of gap 7) | not ported | client surfaces a content mod with no renderer cannot express |

### Deviations this slice adds

1. **The reading is a published cache, not a live read.** Upstream reads the block entity inside
   `getAnalogOutputSignal`; the SDK hook cannot, so a strength is published from the tick (and the
   harness setter/pours) and a changed value notifies the neighbours. A machine whose tank changes
   through a path this port does not own (a foreign pipe, an explosion) keeps its last published
   strength until one of the published paths runs — blood tanks are not world fluids here, so no
   such path is reachable from the carried content.
2. **The blood stain declares no comparator output**, matching upstream (only `BlockPurifier`,
   `BlockDarkTank` and `BlockWithEntityGuiTank`'s subclasses override the two methods).

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 13 (0.2.13) — the Blood Chest's curse roll on repair

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
game 26.3. The registry entry, the PR and the ledger stay the orchestrator's.

### The gap this slice closes, and why this family

Slice 1's gap list, item 2, verbatim:

> **The curse roll is not applied.** `randomBadEnchants = true`,
> `randomBadEnchantsChance = 10000` is recorded in `runtime.rs` (`BAD_ENCHANT_CHANCE`) but adding
> an enchantment component is not implemented.

**Tier:** `deep` still governs the port; this slice is one pure `logic` rule inside it — a
1-in-10,000 boundary and an enchantment-component write on a stack the chest already repairs.
No block, machine, entity, renderer or dimension is added.

### The roll condition, read from the artefact (not the constant)

`ports/_shared/upstream/evilcraft/src/main/java/org/cyclops/evilcraft/blockentity/tickaction/bloodchest/DamageableItemRepairAction.java`
lines 34–55, after `setDamageValue(getDamageValue() - 1)`. The gate is **four** conditions:

```java
if (!isBulk
    && BlockBloodChestConfig.addRandomBadEnchants          // = true
    && BlockBloodChestConfig.randomBadEnchantsChance >= 0  // = 10000
    && random.nextInt(BlockBloodChestConfig.randomBadEnchantsChance) == 0) { … }
```

1. `!isBulk` — the ordinary Blood Chest's `RepairItemTickAction` passes `false`; the Colossal
   Blood Chest's `BulkRepairItemTickAction` passes `true` and **never** curses. The port applies the
   curse only in `blood.rs`'s chest path, never in `colossal.rs`.
2. `addRandomBadEnchants == true` and 3. `chance >= 0` are the config constants.
4. `random.nextInt(chance) == 0`: `nextInt` is bounded to `[0, chance)`, so the curse is added on
   **roll 0 only** (1/10,000), not on any roll below the chance. `runtime::curse_roll_domain`
   pins `roll < BAD_ENCHANT_CHANCE`, `bad_enchant_roll` pins `roll == 0` inside it.

The enchantment is **not** a hardcoded `minecraft:vanishing_curse`. Upstream lists every element of
the `EnchantmentTags.CURSE` (`#minecraft:curse`) registry tag and picks one with
`random.nextInt(curses.size())`; at this pin that is vanilla's two curses, already in
`runtime::CURSES` from slice 9 — `minecraft:binding_curse` and `minecraft:vanishing_curse`. The
level is `minLevel + nextInt(maxLevel - minLevel + 1) - 1`; both curses declare `min == max == 1`,
so it is always 1 (one index draw and one level draw upstream, as `curse_level` documents). The
in-game phase forces index 1, the artefact's `minecraft:vanishing_curse`, so the saved stack can be
asserted exactly.

### TDD: the red proof, before any glue

`sdk-test-red-slice13.log` (the four tests written, the rules stubbed to `false`/`None`/`0`):

```
---- runtime::tests::a_roll_of_zero_is_the_only_one_in_ten_thousand_that_curses stdout ----
thread '…' panicked at src/runtime.rs:1727:9:
upstream's nextInt(randomBadEnchantsChance) == 0
---- runtime::tests::the_roll_is_bounded_by_bad_enchant_chance stdout ----
thread '…' panicked at src/runtime.rs:1740:9:
assertion failed: curse_roll_domain(0)
---- runtime::tests::the_rolled_curse_is_a_vanilla_curse_tag_entry_at_level_one stdout ----
thread '…' panicked at src/runtime.rs:1751:9:
assertion `left == right` failed
  left: None
 right: Some("minecraft:binding_curse")
---- runtime::tests::a_roll_of_zero_adds_the_curse_to_a_clean_stack stdout ----
thread '…' panicked at src/runtime.rs:1762:9:
assertion `left == right` failed
  left: [("minecraft:sharpness", 3)]
 right: [("minecraft:sharpness", 3), ("minecraft:vanishing_curse", 1)]

test result: FAILED. 80 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Green: `sdk-test.log` — **84 passed, 0 failed**; `sdk-clippy.log` — `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` 0 warnings.

### What is in it

| Half | What | Source |
|---|---|---|
| Rules | `runtime::{curse_roll_domain, bad_enchant_roll, rolled_curse, curse_level, add_curse}` | `DamageableItemRepairAction.repair` |
| Chest tick | on a **non-bulk** damage repair only, after `write_damage`: draw the roll (and on a win the curse index) from the level's `RandomSource`, then write the enchantment onto the stack | same file, `RepairItemTickAction`'s `isBulk = false` |
| Writer | `purifier`'s `Enchantments` component, now `pub(crate)` — the same 26.3 bare-map path slice 9 proved in game | `DataComponents.ENCHANTMENTS` |
| Harness | `/evilcraft curse <pos> <roll> <index>` arms a sticky draw for one chest position, `-1` clears; unarmed ticks use the real level random | harness only (upstream has no commands) |
| Parser | `tools/parse_enchants.py` reads the `minecraft:enchantments` map out of `data get block … Items[0]` | harness only |

A failed random draw falls back to the chance itself (never to roll 0), so a machine with no
random stream cannot spontaneously curse a tool.

### Ladder (all through `build-slot.sh`, no `BUILD_LOCAL=1`; compile steps forwarded to AdLf-PC)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` (rules stubbed) | **FAILED 4/84** | `sdk-test-red-slice13.log` |
| tests (green) | `cargo test` | **84 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |

The remaining ladder rows, the artifact, the in-game transcript and the gates are recorded in
`VALIDATION.md` § Slice 13.

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `addRandomBadEnchants` 1/10,000 gate (`roll < chance`, win on 0) | tested | unit `a_roll_of_zero_is_the_only_one_in_ten_thousand_that_curses`, `the_roll_is_bounded_by_bad_enchant_chance`; AL roll 0 vs 9,999 |
| `#minecraft:curse` tag entry and level 1 | tested | unit `the_rolled_curse_is_a_vanilla_curse_tag_entry_at_level_one`; AL reads `minecraft:vanishing_curse@1` |
| the enchantment-component write on a repaired stack | tested | unit `a_roll_of_zero_adds_the_curse_to_a_clean_stack`; AL's `data get block … Items[0]` component map |
| `!isBulk` (the Colossal's bulk repair never curses) | tested | code path: the write lives only in `blood.rs`; `colossal.rs` is untouched, and the Y2/AA legs still read an uncursed stack |

### Deviations this slice adds

1. **The harness's armed draw is sticky per position**, not a one-shot. Upstream draws every
   repair; arming a draw for one chest position lets a scenario reach the rare branch. With no
   draw armed the code uses the level's random exactly as upstream does, so the shipped behaviour
   is unchanged outside a scenario.
2. **The level roll is only a boundary in the pure rule.** `curse_level` ignores its draw because
   both vanilla curses are level 1; the port does not spend a third random on a value upstream's
   `nextInt(1)` always returns as 0. No observable behaviour differs.

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 14 — the Undead Wood set (0.2.14)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da909…`, game 26.3, launcher 0.2.92.
The registry entry, the PR and the ledger stay the orchestrator's. The full design and the
alternatives considered are in `tools/slice14-plan.md`.

### The gap this slice closes, and why this family

Slice 1's gap 7, verbatim:

> No blood stain, colossal chest, purifier, dark ore, **undead wood**, spirits/vengeance,
> werewolves, promises/bowls, brooms, or the environmental accumulator — the rest of the jar.

**Tier:** `deep` still governs the port; this slice is the `data`/`content` half of it — eleven
block registrations, the jar's models, textures, tags, self-drop loot tables and its own seven
crafting recipes, in 26.3's data dialect. No block entity, machine, renderer, entity or
dimension; `reinforced_undead_planks` is already carried for the Colossal shell and is **not**
duplicated. Self-contained, no parent mod, no new hook.

### The jar's id inventory (enumerated from the jar, not the brief)

`unzip -l evilcraft-1.21.1-neoforge-1.2.100.jar | grep -i undead`: every id the brief names
is present. The eleven carried blocks: `undead_log`, `undead_log_stripped`, `undead_wood`,
`undead_wood_stripped`, `undead_planks`, `undead_planks_stairs`, `undead_slab`, `undead_fence`,
`undead_fence_gate`, `undead_leaves`, `undead_sapling`. The jar also contains
`reinforced_undead_planks_stairs` (not in the brief — left for the shell's later slice) and
`data/evilcraft/worldgen/configured_feature/tree_undead.json` (worldgen is inert at this pin).
The jar's `data/evilcraft/tags/{block,item}/undead_logs.json` (four logs each) and its
`minecraft:mineable/axe` list name these blocks; both are carried.

### What is in it

The jar's own **eleven blockstates**, its **block and item models** (the Sapling's item model
alone has its absent `cyclopscore:item/flat` parent changed to `minecraft:item/generated`),
its **seven block textures**, the two `undead_logs` tags and the `undead_*` entries of its
`mineable/axe` tag — all byte for byte. The jar's **seven crafting recipes**, with only the
1.21.1 ingredient objects flattened to 26.3's bare strings:

| file | shape and ingredient | output |
|---|---|---|
| `undead_planks` | shapeless, `#evilcraft:undead_logs` | 4 |
| `undead_wood` / `undead_wood_stripped` | shapeless, 4 of the matching log | 3 each |
| `undead_planks_stairs` | shaped, 6 planks | 4 |
| `undead_slab` | shaped, 3 planks | 6 |
| `undead_fence` | shaped, 2 planks + 2 sticks | 3 |
| `undead_fence_gate` | shaped, 2 planks + 2 sticks | 1 |

The jar's other undead recipes (`base/undead_sapling`, `base/reinforced_undead_planks`,
`broom_part_rod_undead`) are an infusion and an add-on-family recipe, not this slice's
crafting set, and are not carried.

### TDD: the red proof, before the implementation

`sdk-test-red-slice14.log` (four rules stubbed to empty/`None`):

```
test result: FAILED. 84 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
---- runtime::tests::every_carried_wood_recipe_yields_its_jar_count stdout ----
  left: None  right: Some(("evilcraft:undead_planks", 4))
---- runtime::tests::the_planks_recipe_yields_four_from_one_log stdout ----
  left: None  right: Some(("evilcraft:undead_planks", 4))
---- runtime::tests::the_undead_logs_tag_names_exactly_the_four_logs stdout ----
  left: []  right: ["evilcraft:undead_log", "evilcraft:undead_wood", "evilcraft:undead_log_stripped", "evilcraft:undead_wood_stripped"]
---- runtime::tests::the_wood_set_adds_eleven_ids_and_no_reinforced_planks_duplicate stdout ----
  left: 0  right: 11
```

Green: `sdk-test.log` — **88 passed, 0 failed** (84 + 4); `sdk-clippy.log` —
`cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (build-slot leases; compile steps forwarded to AdLf-PC, game legs local)

All ladders rows, the artifact and the in-game legs are recorded in `VALIDATION.md` § Slice 14.

### The in-game leg (fail-capable)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-1bac1c969e7b`, log line
`Compiling evilcraft (1 of 1)`; the export's `src/`, `data/`, `assets/` and `horizon-mod.toml`
are byte-identical to the port (`diff -r`/`cmp` empty). New phases **AM** and **AN**:

```
== AM: the undead log is given, placed with its axis from the clicked face, and an unknown id is refused ==
  evilcraft:undead_log given 1: the parsed inventory count is 1
  placed at 200,101,0: name 'evilcraft:undead_log', axis 'y'
  the same item against an east face: name 'evilcraft:undead_log', axis 'x'
  {"error":"unknown item evilcraft:no_such_wood"}
== AN: every carried undead-wood recipe is crafted the way its jar file declares ==
  -- undead_planks --
  recipe undead_planks.json: crafting_shapeless -> {"count": 4, "item": "evilcraft:undead_planks", "slot": 200}
    the result reached the inventory (4 evilcraft:undead_planks), the grid emptied, and evilcraft:undead_log was consumed
  vanilla control: {… "drops": {"minecraft:crafting_table": 1} …}
  evilcraft:undead_log: {… "drops": {"evilcraft:undead_log": 1}, "block_before": "evilcraft:undead_log[axis=y]" …}
  -- undead_wood .. undead_fence_gate --
  each recipe: the declared id/count, the grid emptied, the ingredient consumed
```

**Fail-capable three ways.** (1) `SELFTEST=wood` inverts AM's block-name equality and exits 1
(*SELFTEST flipped: the block name is evilcraft:undead_log, as it should*). (2) AM's two
placements are the face differential: the same item on the floor's down face reads `axis=y`,
against a west neighbour's face it reads `axis=x` (a log's RotatedPillarBlock axis follows the
clicked face). (3) The whole-package control has no undead log at all, so AM cannot give it.
The seven recipes are each proved through a real crafting table by the campaign's
`check_261_dialect.py prove` harness, which asserts the output id and count, that the grid
emptied and that the declared ingredient was consumed; the planks proof also breaks a log with
a vanilla crafting-table control beside it.

### Harness facts learned here

1. **`ctl place` chooses the face itself.** It aims at the first solid neighbour in the
   agent's `down, up, north, south, west, east` order (`horizon-client/game/agent.rs`), not at
   the crosshair. The arena floor under every cell wins with the down face (`axis=y`); to prove
   the horizontal axis, clear the floor under the target and leave only a side neighbour.
2. **A shapeless recipe's tag ingredient resolves in the proof spec's `items` map**, not
   `tag_items` (the latter is only consulted for a shaped recipe's `key`). The spec uses
   `{"items": {"0": "evilcraft:undead_log"}}` for `undead_planks`.
3. **The default leg flaked four times under load** — Q's bow kill twice, AE's dark-tank break
   once and the first AM side placement once. Each flake log is kept beside the green one
   (`e2e-launcher-0.2.14-attempt{1..5}.log`); no assertion was weakened. The sixth run was green.
4. **The launcher's global native-build lock** (`.build.lock` under `horizon-launcher/`) is held
   by other agents' builds; the build waited for both launcher locks with `flock -n` before
   invoking `launcher-game.sh build` (the campaign's "retry against `flock -n`" rule).

### Deviations this slice adds

1. **Self-drop loot tables, not the jar's conditional ones.** The jar's leaves table uses 1.21.1
   `conditions` arrays (shears/silk-touch alternatives, fortune chances), the slab uses a
   `conditions` array inside `set_count`, and the fence uses `survives_explosion` — all fatal in
   26.3's dialect. Each is replaced by the minimal single-pool self-drop the checker accepts;
   the leaves no longer drop dead bushes/sticks and a double slab drops one. The jar's byte
   inventory, the converted recipe bytes and this deviation are recorded in `LICENSE-NOTICE.md`.
2. **No undead tree worldgen.** `tree_undead` is a configured feature upstream; worldgen is
   inert at this pin. The sapling's `random_ticks` is off (the jar's leaves likewise do not
   decay) so neither block performs a tree/growth tick it cannot complete.
3. **Templates approximate the Java class.** Logs/wood use `minecraft:oak_log` (the jar's
   `RotatedPillarBlock` with `axis`), planks use `minecraft:stone`, stairs/slab/fence/gate use the
   vanilla stone-stairs/slab, oak-fence and oak-fence-gate templates, leaves `minecraft:oak_leaves`
   and the sapling `minecraft:poppy`; each with the jar's hardness, resistance, sounds and
   map colour. Fire spread, flammability, compostability and the axe-strip listener have no
   content surface here.
4. **The sapling's item model parent changed** from the jar's absent `cyclopscore:item/flat` to
   `minecraft:item/generated`; the texture is the jar's own.

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| 11 undead wood blocks, items, blockstates, models, textures, lang, creative tab | tested | AM: `ctl give` + `ctl place` read back `evilcraft:undead_log` with `axis=y` and `axis=x` |
| `minecraft:mineable/axe` for all 11 | tested | the planks proof's log breaks with an iron pickaxe, with a vanilla-table control |
| seven crafted recipes (`undead_planks`, `undead_wood`, `undead_wood_stripped`, stairs, slab, fence, gate) | tested | AN: each output id/count, grid emptied, ingredient consumed through `check_261_dialect.py prove` |
| `evilcraft:undead_logs` block and item tags | tested | AN: the shapeless planks recipe resolves the tag to the carried log |
| 11 self-drop loot tables | tested (log) | AN: `evilcraft:undead_log` self-drops beside the vanilla control; the other ten use the same data shape |
| `evilcraft:no_such_wood` refusal | tested | AM: `{"error":"unknown item evilcraft:no_such_wood"}` |
| axe-strip (log/wood → stripped), fire spread, compostability | not ported | no content surface; deviation 3 |
| `tree_undead` configured feature / sapling growth | not ported | worldgen inert at this pin; deviation 2 |
| jar's conditional leaves/slab/fence loot functions | not ported (converted) | deviation 1; 26.3 dialect gate |
| jar's `base/undead_sapling`, `base/reinforced_undead_planks` recipes | **carried by slice 15** | AO: the readout goes 10/10 → 12/12 and both infusions run in game |
| jar's `broom_part_rod_undead` recipe | not ported | an add-on-family recipe; the broom family is not carried |

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Slice 15 — the two undead-tree Blood Infuser recipes (0.2.15)

**Pin:** Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, launcher
0.2.92) + mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `api = "0.2"`, game 26.3.
Package **0.2.15** on top of 0.2.14 (an Update of our own entry; the registry entry, the PR and
the ledger stay the orchestrator's). The design and the alternative considered are in
`tools/slice15-plan.md`.

### The gap this slice closes, and why this family

Slice 14's coverage row, verbatim:

> | jar's `base/undead_sapling`, `base/reinforced_undead_planks`, `broom_part_rod_undead` recipes |
>   not ported | add-on/infuser families outside this slice's crafting set |

The two `base/` files are **Blood Infuser** recipes — the machine this port has carried since
0.2.2 — and every id they name is carried now: `minecraft:dead_bush` (vanilla),
`evilcraft:undead_planks` and `evilcraft:undead_sapling` (slice 14), and
`evilcraft:reinforced_undead_planks` (slice 7). They close the loop 0.2.14 opened.
`broom_part_rod_undead` stays a gap: the broom family is not carried, so carrying its recipe
would be the dead data this port's `every_carried_recipe_names_items_this_package_or_the_game_has`
test refuses.

**Tier:** `deep` still governs the port; this slice is `logic`/`data` inside it — two data files,
their `CARRIED` declarations and the item ids they name. No block, block entity, entity,
renderer, hook or worldgen, and no parent mod outside the registry.

### The numbers, read from the jar (not the brief)

`ports/_shared/licence-cache/evilcraft-1.21.1-neoforge-1.2.100.jar`, both files carried byte for
byte at `data/evilcraft/recipe/blood_infuser/base/<name>.json` (260 B and 288 B; `cmp` against the
jar empty, exactly as `base/blook.json` was):

| jar file | input | blood | output | count | duration | tier |
|---|---|---:|---|---:|---:|---:|
| `base/undead_sapling.json` | `minecraft:dead_bush` | **800 mB** | `evilcraft:undead_sapling` | 1 | **200 t** | **omitted → 0** |
| `base/reinforced_undead_planks.json` | `evilcraft:undead_planks` | **1,000 mB** | `evilcraft:reinforced_undead_planks` | 1 | **250 t** | **2** |

The dispatch prose named a "tier-3" recipe for the planks; the artefact says **tier 2**, which a
tier-3 Promise also satisfies, so the port carries 2 and the in-game assertion proves both that
tier 0 refuses it and that tier 3 runs it. The sapling file omits `tier`, which upstream's
`RecipeSerializerBloodInfuser` reads as 0 — so the existing
`every_carried_recipe_fits_its_tank_and_names_a_tier` range moved from `1..=3` to `0..=3`, and
`CARRIED_ITEMS` gained the three undead ids so a carried file may name them.

### TDD: the red proof, before the carry

`sdk-test-red-slice15.log` (the three new tests written, the two files and their `CARRIED`
entries not added yet):

```
test result: FAILED. 88 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

---- runtime::tests::a_dead_bush_becomes_an_undead_sapling_at_tier_zero stdout ----
thread 'runtime::tests::a_dead_bush_becomes_an_undead_sapling_at_tier_zero' (834747) panicked at src/runtime.rs:1620:14:
a dead bush is infused at tier 0

---- runtime::tests::the_infuser_carries_twelve_recipes stdout ----
thread 'runtime::tests::the_infuser_carries_twelve_recipes' (834801) panicked at src/runtime.rs:1610:9:
assertion `left == right` failed
  left: 10
 right: 12

---- runtime::tests::undead_planks_become_reinforced_planks_at_tier_two stdout ----
thread 'runtime::tests::undead_planks_become_reinforced_planks_at_tier_two' (834813) panicked at src/runtime.rs:1637:14:
undead planks are infused at tier 2
```

Green: `sdk-test.log` — **91 passed, 0 failed** (88 + 3), regenerated after the final build;
`sdk-clippy.log` — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` 0 warnings.

### The ladder (build-slot leases; compile steps forwarded to AdLf-PC, game legs local)

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` (files not carried) | **FAILED 3/91** (`EXIT:101`) | `sdk-test-red-slice15.log` |
| tests (green) | `cargo test` | **91 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 629 API uses, 23 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **269,193 B**, sha256 `2a76e9eb…65adbc6`, **`CMP-IDENTICAL`**, 211 entries, no `Cargo.lock` | `sdk-build-a/b/cmp-0.2.15.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, `CHECK_EXIT:0` | `sdk-check-fits-0.2.15.log` |
| fits, the pin | `… fits … --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for`, `FIT_PIN_EXIT:0` | `sdk-check-fits-0.2.15.log` |
| fits, another build | `… fits … --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, `FIT_OTHER_EXIT:0` | `sdk-check-fits-0.2.15.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** — 91 tests, 0 clippy warnings, runtime byte-identical, **package matches the tree**, R9 clean, check + fits, Skills gate PASS | `sdk-gate-0.2.15.log` |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 36 recipe/loot files; both new recipes `ok`; unchanged slice-1 `"rolls": 1.0` | `sdk-dialect-0.2.15.log` |
| artifact record | `sdk-artifact-0.2.15.txt` | bytes/sha/entries/manifest/runtime sha, check and fits lines | `sdk-artifact-0.2.15.txt` |
| launcher game | `ports/tools/launcher-game.sh build evilcraft` | `native-0a875da-5b295a50ac7b`, `Compiling evilcraft (1 of 1)`; export `src/`, `data/`, `assets/`, `horizon-mod.toml` byte-identical to the port | `launcher-build-0.2.15.log`, `sdk-export-check-0.2.15.log` |

`LICENSE-NOTICE.md` was edited **before** the final build (the twelve recipes are credited) and
`tools/stamp_license.py` re-run, so the gate's "package matches the tree" is green. No stale
`native-*` tag with a 0.2.15 manifest existed before the build; the launcher produced exactly one.

### The in-game leg (fail-capable)

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (`OLD=1`, `LEGACY=1`, `OLD14=1`) | **`PASS` + `EXIT:0`** — every earlier phase plus the 0.2.14 leg and **AO** | `e2e-launcher-0.2.15.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control-0.2.15.log` |
| selftest (this slice) | `SELFTEST=undeadinfuse` | **`FAIL` + `EXIT:1`** at AO — *SELFTEST flipped: the dead bush became evilcraft:undead_sapling, as it should* | `e2e-launcher-selftest-undeadinfuse-0.2.15.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=37 ok=37
missing-exit=0 bad-exit=0` (exit 0): all 34 earlier legs stay green and the three 0.2.15 logs
are this slice's. No flake occurred on this run.

Transcript of the new legs (verbatim from `e2e-launcher-0.2.15.log`):

```
== the previous package (0.2.14): the infuser reads 10/10 recipes ==
  Blood Infuser [206, 101, 0]: tank 0/10000 mB, tier 0, cont empty@0, in empty@0, out empty@0, progress 0, recipes 10/10 t=150
  0.2.14: 10/10 recipe files (no undead_sapling or reinforced_undead_planks)
== H: the Blood Infuser's own recipe files load, and an unknown id is refused ==
  Blood Infuser [206, 101, 0]: tank 0/10000 mB, tier 0, cont empty@0, in empty@0, out empty@0, progress 0, recipes 12/12 t=1284
  block evilcraft:blood_infuser, 12/12 recipe files in force
== AO: the two undead-tree infusions (the loop 0.2.14 opened) ==
  infuser reads 12/12 recipe files (the two undead-tree files included)
  Blood Infuser [206, 101, 0]: tank 9500/10000 mB, tier 0, cont empty@0, in empty@0, out empty@0, progress 0, recipes 12/12 t=9073
  Blood Infuser [206, 101, 0]: tank 9200/10000 mB, tier 0, cont empty@0, in empty@0, out evilcraft:undead_sapling@1, progress 0, recipes 12/12 t=9374
  tier 0: dead_bush -> undead_sapling@1 for 800 mB in 200 ticks, tank 9200
  took 1 x evilcraft:undead_sapling out of the result slot into inventory slot 12
  Blood Infuser [206, 101, 0]: tank 10000/10000 mB, tier 0, cont empty@0, in evilcraft:undead_planks@1, out empty@0, progress 0, recipes 12/12 t=9760
  330 ticks at tier 0: the planks are untouched, no product, 10000 mB left
  Blood Infuser [206, 101, 0]: tank 10000/10000 mB, tier 3, cont empty@0, in evilcraft:undead_planks@1, out empty@0, progress 23, recipes 12/12 t=9790
  Blood Infuser [206, 101, 0]: tank 9000/10000 mB, tier 3, cont empty@0, in empty@0, out evilcraft:reinforced_undead_planks@1, progress 0, recipes 12/12 t=10129
  tier 3: undead_planks -> reinforced_undead_planks@1 for 1,000 mB in 250 ticks, tank 9000
  took 1 x evilcraft:reinforced_undead_planks out of the result slot into inventory slot 13
```

**Fail-capable four ways.** (1) `SELFTEST=undeadinfuse` inverts AO's first output equality and
exits 1. (2) The tier-0 window is the differential on the same machine, the same full tank and
the same undead-planks input: 330 ticks with no promise consume nothing and spend nothing; the
same input with a tier-3 Promise yields one plank for 1,000 mB. (3) The 0.2.14 leg is the
readout's differential on the same rig: 10/10 before the carry, 12/12 after. (4) The whole-package
control has no infuser, so AO cannot even place one. The sapling's tier-0 run is its own
counterfactual against the tier-2 recipe (the same machine ran both).

### Coverage table — the rows this slice changes

| Family (jar) | Status | Proof |
|---|---|---|
| `base/undead_sapling` (`dead_bush` → sapling, 800 mB, 200 t, tier 0) | **tested** | AO: `out evilcraft:undead_sapling@1`, tank 10,000 → 9,200, input consumed, at tier 0; unit `a_dead_bush_becomes_an_undead_sapling_at_tier_zero` |
| `base/reinforced_undead_planks` (`undead_planks` → reinforced, 1,000 mB, 250 t, tier 2) | **tested** | AO: 330 t at tier 0 do nothing, then one block for 1,000 mB at tier 3; unit `undead_planks_become_reinforced_planks_at_tier_two` |
| `recipes 10/10 → 12/12` | **tested** | the 0.2.14 leg reads `10/10`, H and AO read `12/12`; unit `the_infuser_carries_twelve_recipes` |
| tier 0 is a legal carried tier | **tested** | `every_carried_recipe_fits_its_tank_and_names_a_tier` now pins `0..=3`; AO runs the tier-0 file in game |
| `broom_part_rod_undead` and the broom family | not ported | no broom family is carried; it would be dead data |

### Deviations this slice adds

1. **The two files are carried byte for byte**, including the jar's 1.21.1 object ingredient
   shapes; `decode_infusion` already reads them, and `check_261_dialect.py` accepts them as this
   mod's own recipe type. No ingredient flattening was needed (unlike the wood recipes).
2. **The tier range widened to 0..=3**, not a relaxation: the sapling file genuinely omits
   `tier`, so upstream reads 0, and a tier-0 machine must run it. The reinforced file declares 2,
   which a tier-3 Promise satisfies; the port does not police which promise slot holds it.

### Upstream defects fixed

None new. The dispatch note's "tier-3" for `reinforced_undead_planks` was corrected from the
artefact (tier 2); this is a correction to the brief, not a change to the mod.

### Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog for the record (0.2.15, verbatim)

> 0.2.15 — the two undead-tree Blood Infuser recipes from the 1.2.100 jar are carried byte for
> byte: a dead bush becomes one Undead Sapling for 800 mB in 200 ticks with no promise (the file
> omits `tier`), and Undead Planks become one Reinforced Undead Plank for 1,000 mB in 250 ticks
> at tier 2. The infuser now reads 12/12 recipe files where 0.2.14 read 10/10. 91 tests, two
> identical builds, the in-game leg green with a new fail-capable phase AO, a 0.2.14 readout
> differential, a package-less control and a `SELFTEST=undeadinfuse` that ends `EXIT:1`.

## Next slice

**Family:** the jar's `broom_part_rod_undead` and the rest of the promises/bowls progression —
specifically `evilcraft:bowl_of_promises_*` and the three `promise_acceptor_*` blocks, the next
named row in the coverage table that still fits `data`/`logic` inside `deep`. **Gap line:** slice
2's coverage row, "`promise/bowl family (`bowl_of_promises_*`, `promise_acceptor_*`, speed/efficiency
promises) | not ported | the promise items are carried as machine upgrades only; their crafting
chain is a later slice`", and slice 1's gap 7 ("...promises/bowls... the rest of the jar").
**Why it fits:** the three tier promises are already carried as infuser upgrades, so this is the
progression that makes them craftable rather than a new machine or entity; the infuser can run
the chain's own `base/` recipes as data. **In-game assertion:** after carrying the jar's bowl and
acceptor recipes, build one acceptor in the arena, insert the artefact's declared input through
`/evilcraft load`, and read the parsed `/evilcraft infuser` and acceptor readout: the bowl recipe
must be `recipes 14/14` (or its carried count) and yield one bowl for the jar's fluid amount and
ticks at the declared tier, while the same input at tier 0 produces nothing — with a
`SELFTEST=promises` inversion ending `EXIT:1`. Stop and report `blocked(<symbol>)` first if the
jar's acceptor behaviour is a Java class with no data table and no placeable surface at this pin.

