# evilcraft — EvilCraft: the blood loop (slice 1: the repair loop, slice 2: the Blood Infuser, slice 3: the tank in the dropped machine, slice 4: projectile kills, slice 5: the Blood Stain, slice 6: the fall-death stain, slice 7: the Colossal Blood Chest)

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
