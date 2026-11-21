# evilcraft — EvilCraft: the blood loop (slice 1: repair loop, slice 2: Blood Infuser, slice 3: the tank in the dropped machine)

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
| projectile kills giving blood | not ported | `mob_killed` at this pin hears melee, sweep and spear (gap 4) |
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
