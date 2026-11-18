# EvilCraft slice 1 (blood repair loop) — implementation plan

> **Historical (2026-11-18):** this plan was written for the retired `.hnmod` (AE2 embedding,
> `support.rs`, stamped hooks). The port is now the flat SDK package; the plan's behaviour,
> numbers and review focus still hold, its architecture paragraph does not. See `notes.md`
> § Migration to the SDK format.

**Goal:** a verified EvilCraft port that fills a Blood Extractor from a player kill and
repairs damaged items in a Blood Chest with the blood, with upstream's own numbers, in game.

**Architecture:** one `.hnmod` carrying the content (AE2 embedding pattern) plus a pure,
unit-tested `runtime.rs`; game work in `support.rs` behind stamped hooks only.

**Spec:** `FINDINGS.md` (this directory) — every number quoted there comes from the jar or
from `CyclopsMC/EvilCraft` tag `1.21.1-1.2.99`.

## Global constraints

- Pin `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
- Stamped hooks only (`ports/_shared/pins/horizon-1d63e0a/native-hooks.json`).
- Builds/tests/e2e only through `ports/tools/build-slot.sh`, `CARGO_INCREMENTAL=0`, no `--target-dir`.
- Ids `evilcraft:*`; the slice owns `blood_chest` and `blood_extractor` only.
- `notes.md` carries `Skills: superpowers/…; BeautifulRust/…`; the report opens with `SKILLS:`.

## Review Focus

- A repair with a tank under 5 mB must change nothing (the tank is the gate, not the tick).
- Blood never appears from nowhere: a kill with no extractor carried gives no blood, and a
  tank that is full loses nothing when an extractor is emptied into it.
- An item that is not damaged, is a stick, or is not an anvil state must never be touched.
- An empty tank must not repair, even with a damaged item in the slot, over many ticks.
- The 10,000 mB tank must clamp at capacity from both directions (repair and fill).

---

### Task 1: pure runtime

**Files:** create `mixins/evilcraft/runtime.rs`; test in the same file (`#[cfg(test)] mod tests`).

**Interfaces (produced):**
`TANK_CAPACITY: i32`, `MB_PER_DAMAGE: i32`, `TICKS_PER_DAMAGE: u32`,
`ANVIL_MB: i32`, `CHEST_SLOTS: usize`, `CONTAINER_SLOT: usize`, `SLOT_COUNT: usize`,
`EXTRACTOR_CAPACITY: i32`, `MIN_MOB_MULTIPLIER: f64`, `MAX_MOB_MULTIPLIER: f64`,
`BLACKLIST: &[&str]`, `blood_from_kill(max_health: f32, roll: u32) -> i32`,
`fill_extractors(pool: i32, held: &mut [i32]) -> ()` (or `fill_stack(current, amount)`),
`repair_due(ticks: u32) -> bool`, `repair_cost() -> i32`,
`repair(damage: i32, tank: i32) -> Option<(i32, i32)>`,
`anvil_step(item: &str) -> Option<&'static str>`,
`can_hold(item: &str, damage: i32) -> bool`,
`drain_container(tank: i32, container: i32) -> (i32, i32)`.

- [ ] **Step 1 (RED):** write the tests named in `runtime.rs` (kill range, 5 mB cost, 2-tick cadence,
  anvil 25 mB both steps, stick refused, extractor capacity 5,000 and a pool that spills, tank clamp
  10,000, container drain without overflow). Run `cargo test` through a slot and watch them fail for
  the missing functions.
- [ ] **Step 2 (GREEN):** implement each to the quoted upstream rule; no invented numbers.
- [ ] **Step 3:** `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green.

### Task 2: glue and hooks

**Files:** `mixins/evilcraft/support.rs`, one helper per injection, `horizon-mixin.toml`.

- [ ] Block entity state: tank in `nbt["fluids"]` (`evilcraft:blood`), slots in `items[0..=10]`,
  menu data `[tank, capacity]`.
- [ ] Tick: empty the container slot into the tank; every 2 ticks repair one damage point in each
  repairable slot at 5 mB, anvils at 25 mB; stop at 0 damage.
- [ ] `inventory_tick`: a carried Blood Extractor takes the pending kill blood into its
  `custom_data` (capacity 5,000, one stack at a time).
- [ ] `living_incoming_damage_v2` + `mob_killed`: remember the victim's max health, then convert the
  kill into `floor(hp×5) .. floor(hp×40)` mB of pending blood for that player.
- [ ] Screen: 11-slot layout and a tank readout; `/evilcraft info|blood|damage|held` for the harness.

### Task 3: content

- [ ] `content/` assembled from the jar by `tools/assemble_content.py` (block declaration template
  `minecraft:barrel`, blockstate facing/open, cube model with the mod's own texture, item model,
  loot table in the singular 26.3 shape, lang, menu type, creative tab).
- [ ] `tools/package.sh`: `cargo horizon-mod build` → `content/evilcraft.hmod`, sync into
  `mixins/evilcraft/data/evilcraft-content.hmod`, `tools/package.py` → `evilcraft.hnmod`.

### Task 4: ladder + evidence

- [ ] `horizon.py stage → check → test → build` through `build-slot.sh` (exit 0, 0 warnings).
- [ ] `tools/e2e.sh`: default leg (kill a mob with the extractor → blood → repair a damaged pickaxe
  and a chipped anvil), control leg (empty tank repairs nothing), `EMBEDDED=1` leg with no
  `--dev-mod` plus an unusable-`.horizon-cache` control, `SELFTEST=1` must exit non-zero.
- [ ] `check-e2e-logs.py evilcraft --strict`, `check-skills.py --notes ports/evilcraft`,
  `upstream_ready.py evilcraft` all exit 0; logs under `ports/_shared/evidence/evilcraft/`.
