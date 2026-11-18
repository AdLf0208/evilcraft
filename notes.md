# evilcraft — EvilCraft, slice 1: the blood repair loop

> **Format (2026-11-18):** this port is now the flat **SDK package** (`api = "0.2"`,
> `[native]` at `0b14f8a`) built by `cargo horizon-mod build`. The `.hnmod`, `mixins/evilcraft/`,
> `content/` and `horizon-mixin.toml` described in the sections below are **retired** and were
> deleted; read them as history. The migration, the hook-by-hook mapping and the new ladder are
> in [§ Migration to the SDK format](#migration-to-the-sdk-format-2026-11-18) at the foot.

- **Mod:** EvilCraft (`evilcraft`), **content / deep**, core, priority **P0**.
- **Licence:** **MIT**, read from the jar's `META-INF/neoforge.mods.toml`
  (`license="MIT"`, `modId="evilcraft"`, `version="1.2.100"`). **Rank 0** (permissive).
  Local-only until the maintainer approves publishing (ARCHITECTURE §8).
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
   (a Blood Infuser product) and `blood_extractor` needs `dark_spike` + `dark_gem`;
   none of those are in this slice, so both items are creative-only for now.
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
7. **No blood stain, colossal chest, purifier, blood infuser, dark ore, undead wood,
   spirits/vengeance, werewolves, promises/bowls, brooms, or the environmental
   accumulator** — the rest of the jar. `FINDINGS.md` lists the families.
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
  **Repair:** `LICENSE-NOTICE.md` now records that the 1.21.1 jar's `META-INF/neoforge.mods.toml` declares `license="MIT"` and that the repository has no root licence file (hence GitHub's 404).
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
