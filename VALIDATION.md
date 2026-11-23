# evilcraft — validation evidence

## SDK format (2026-11-18) — current

The port is the flat SDK package. Full transcript:
`ports/_shared/evidence/evilcraft/sdk-migration-ladder.log` (one `build-slot.sh` lease).

| Step | Command | Result |
|---|---|---|
| runtime tests | `cargo test` | **17 passed, 0 failed** |
| logic unchanged | `sha256sum src/runtime.rs` before and after | `5325b286…8c223a`, byte-identical to the retired `mixins/evilcraft/runtime.rs` |
| fmt | `cargo fmt --check` | clean |
| clippy | `cargo clippy --all-targets -- -D warnings` | clean |
| package | `cargo horizon-mod build` → copied to the port root | `evilcraft.hmod` **130,338 B** `a25beba657f13fc32600324f69064689b7180841b25343c8c302c9df2f7908f4` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0 — `format native-1/0b14f8a…/e59da909…, for 26.3`; *native source not executed by the check* |
| fit | `cargo horizon-mod fits evilcraft.hmod` | exit 0 — `0b14f8a  made for` |
| project check | `cargo horizon-mod check` | exit 0 — `cargo check` for this box and wasm32; `uses 356 items of the API`; `ModContainer` and `Screen` built with `..` |

**Not run:** an in-game leg. The retired `tools/e2e.sh` stages `mixins/` with `horizon.py`, which
no longer exists in this port, and the SDK package needs a game built at `0b14f8a` (the shared
binaries on disk are the `1d63e0a` pair). The three `e2e-*.log` files below are the **retired
bundle's** evidence and are history, not evidence about this package. First in-game leg for the
new format: `cargo horizon-mod run --game <a 0b14f8a game>`.

## Retired `.hnmod` format (2026-11-10/11) — history

Everything below was run on 2026-11-10/11 in this checkout, through
`ports/tools/build-slot.sh` (`BUILD_LOCAL=1`), `CARGO_INCREMENTAL=0`, no `--target-dir`.


## Artifacts (deterministic: two packaging runs, identical bytes)

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `ports/evilcraft/evilcraft.hnmod` | 18,676 B | `d24d6d108667832d4653d1ec0d70d0374d38d22c279ee5de38f0f5eab3e4e12e` | 18 |
| `content/evilcraft.hmod` (also embedded at `mixins/evilcraft/data/evilcraft-content.hmod`) | 4,377 B | `8c56e041fc2a2f0b73866695af7aa81cffb67d0e8a952010a71bcb8ec429e696` | 15 |

`cargo horizon-mod check evilcraft.hnmod --kind native-mod` → accepted
(`format native-1/1d63e0a5…/5a64af93…, for 26.3`). Log:
`target/ladder/package-hmod.log`, `target/ladder/check-native.log`.

## Ladder

| Step | Command | Result |
|---|---|---|
| runtime tests (standalone) | `cargo test --locked --offline` | **17 passed, 0 failed** |
| fmt | `cargo fmt --check` | clean |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | clean (1 lint fixed, not silenced: `manual_is_multiple_of`) |
| staged check | `horizon.py check` | `cargo_exit: 0`, **0 warnings** |
| embedded tests | `horizon.py test` (`--lib game::evilcraft::tests`) | **17 passed, 0 failed**, 0 warnings |
| game build | `horizon.py build` (fast) | `cargo_exit: 0`, 0 warnings |

Logs: `target/horizon-check.log`, `target/horizon-test.log`, `target/horizon-build.log`.

## e2e (The End arena at x=200, scratch home, `--offscreen`)

| Leg | Command | Log | Verdict |
|---|---|---|---|
| default (content as `--dev-mod`) | `EMBEDDED=0 tools/e2e.sh` | `ports/_shared/evidence/evilcraft/e2e-default.log` | `PASS` + `EXIT:0` |
| embedded (no `--dev-mod`, content from the bundle) + unusable-`.horizon-cache` control | `EMBEDDED=1 tools/e2e.sh` | `.../e2e-embedded.log` | `PASS` + `EXIT:0` |
| selftest (C's assertion flipped) | `SELFTEST=1 tools/e2e.sh` | `.../e2e-selftest.log` | `FAIL` + `EXIT:1` |

What the passing run measured (from the log):

- A: a chest placed from its item reads `0/10000 mB`.
- B: with an empty tank, a pickaxe at damage 250 stays 250 over 40 ticks (the control).
- C: 1,000 mB → damage 244 → 217 while the tank went 970 → 835: **27 damage points for
  exactly 135 mB (5 mB a point)**, 27 points in 55 ticks — no faster than one point per
  two ticks.
- D: an empty tank leaves a chipped anvil alone; 125 mB turns `chipped_anvil` into
  `anvil` and the tank to 0 (**one 125 mB step**).
- E: killing a 20 HP zombie with a diamond sword fills the carried Blood Extractor
  (316 mB in the recorded default run, 385 mB in the embedded one — both inside
  100–799).
- F: the extractor in the container slot empties into the tank: **316 mB → tank 316 mB,
  extractor 0 mB** (385 mB → 385 mB in the embedded run).
- Embedded control: with `.horizon-cache` unusable, the chest item is unknown and
  `setblock evilcraft:blood_chest` leaves `minecraft:air` — the content really came
  from the embedded file, which is byte-identical to `content/evilcraft.hmod`.

## Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft --strict` — every log ends in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft` — the `Skills:` line.
- `python3 ports/tools/upstream_ready.py evilcraft` — registry-ready shape.

## Slice 2 — the Blood Infuser (2026-11-21), 0.2.2

Pin **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** + mixins `e59da909…`, game 26.3, launcher 0.2.92.
Every step ran through `ports/tools/build-slot.sh` (`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no
`--target-dir`). Logs are under `ports/_shared/evidence/evilcraft/` unless a row says otherwise.

| Step | Command | Result | Log |
|---|---|---|---|
| TDD red | `cargo test` (tests written, rules stubbed) | `FAILED. 20 passed; 6 failed` — `a bone is infused`, `assertion failed: infusion_for(&recipes, "minecraft:bone", 3).is_some()`, `left: 0 / right: 1` | `sdk-test-red.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 413 API uses, content accepted | `sdk-project-check.log` |
| fmt | `cargo fmt --check` | clean | `sdk-clippy.log` |
| clippy | `cargo clippy --all-targets -- -D warnings` | 0 warnings | `sdk-clippy.log` |
| unit tests | `cargo test` | **26 passed, 0 failed** | `sdk-test.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **164,572 B** sha256 `3c1dbf04f7de8a96898b6c52e305ac4a16359777405bc8d43e742831f14ab277`, `cmp` identical, 61 entries, no packed `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, 164,572 B, same sha, `EXIT:0` | `sdk-check.log` |
| fit, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fit, a different build | `cargo horizon-mod fits evilcraft.hmod --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** (runtime byte-identical, no Cargo.lock, 26 tests, 0 clippy warnings, check + fits, Skills line) | job report |
| artifact record | `sdk-artifact-0.2.2.txt` | bytes/sha/entries/manifest | `sdk-artifact-0.2.2.txt` |

### In-game (launcher-built game, `ports/_shared/evidence/evilcraft-e2e/`)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-19366e9d0373` (`Compiling evilcraft (1 of 1)`);
the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical to the port (`diff -r`/`cmp` empty).

| Leg | Command | Result | Log |
|---|---|---|---|
| default | `tools/e2e.sh` | **`PASS` + `EXIT:0`** — A, A2, B–F (slice 1) and H–N (the infuser) | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — *no chest or infuser item, both blocks stayed `minecraft:air`*, then the first id check fails | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at phase C | `e2e-launcher-selftest.log` |
| selftest (slice 2) | `SELFTEST=infuser` | **`FAIL` + `EXIT:1`** at phase K (`the bone was not infused into a blaze rod (out minecraft:blaze_rod, in empty, tank 7500)`) | `e2e-launcher-selftest-infuser.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=4 ok=4 missing-exit=0 bad-exit=0` (exit 0).

What the new phases measured (from `e2e-launcher.log`):

- H: a fresh infuser reads `tank 0/10000 mB, tier 0, recipes 9/9` — all nine carried recipe files are in
  force through `Host::recipes_of_type` (a package that did not register its type reads 0); an unknown
  item id is refused by `/evilcraft load`.
- I: with 10,000 mB and a bone in the infusion slot, **560 ticks at tier 0 leave the bone untouched, no
  product and the tank at 10,000** (the tier gate).
- J: the menu lists **7 slots** (three machine slots, four upgrade slots) and a
  `evilcraft:promise_tier_3` in slot 303 makes the machine **tier 3**.
- K: the same bone becomes **one blaze rod for 2,500 mB in 500 ticks** (tank 10,000 → 7,500), and the
  product is taken out of the result slot into the player's inventory.
- L: all nine carried recipes craft with their own file's product, count (`spider_eye` ×8), consumed
  input and blood cost.
- M: a filled extractor in the container slot empties into the tank (1,000 mB in, 0 left).
- N: a **survival** break with a diamond pickaxe drops both machines (`blood_infuser` 1, `blood_chest` 1)
  — the loot tables and the `minecraft:mineable/pickaxe` tag slice 1 was missing.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 4/4 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 3 — the tank in the broken machine (2026-11-21), 0.2.3

Pin **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** + mixins `e59da909…`, game 26.3, launcher 0.2.92.
Every step ran through `ports/tools/build-slot.sh` (`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no
`--target-dir`). Logs are under `ports/_shared/evidence/evilcraft/` unless a row says otherwise.

| Step | Command | Result | Log |
|---|---|---|---|
| TDD red | `cargo test` (tests written, rules stubbed) | `FAILED. 26 passed; 3 failed` — `left: None / right: Some(6500)`, `a filled tank travels`, `left: 0 / right: 10000` | `sdk-test-red-slice3.log` |
| unit tests | `cargo test` | **33 passed, 0 failed** | `sdk-test.log` |
| fmt | `cargo fmt --check` | clean | `sdk-clippy.log` |
| clippy | `cargo clippy --all-targets -- -D warnings` | 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 455 API uses, 14 hooks, `EXIT:0` | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **170,472 B** sha256 `fde8475d6769321c7d717b4c3254ca881c6b053c90810e86c338661341273590`, `cmp` identical, 62 entries, no packed `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` (sha printed first) | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, 170,472 B, same sha, `EXIT:0` | `sdk-check.log` |
| fit, the pin | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` (sha printed first) | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fit, a different build | `cargo horizon-mod fits evilcraft.hmod --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | unchanged: 1 unverified `"rolls": 1.0` in slice 1's chest table, 3 fatal shapes clean | job report |
| artifact record | `sdk-artifact-0.2.3.txt` | bytes/sha/entries/manifest/runtime sha, check and fits lines | `sdk-artifact-0.2.3.txt` |

### In-game (launcher-built game, `ports/_shared/evidence/evilcraft-e2e/`)

`ports/tools/launcher-game.sh build evilcraft` → `native-0a875da-732cfde441bf` (`Compiling evilcraft (1 of
1)`); the export's `src/`, `data/` and `horizon-mod.toml` are byte-identical to the port (`diff -r`/`cmp`
empty, `version = "0.2.3"`).

| Leg | Command | Result | Log |
|---|---|---|---|
| default | `tools/e2e.sh` | **`PASS` + `EXIT:0`** — A, A2, B–F, H–P | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — *no chest or infuser item, both blocks stayed `minecraft:air`*, then the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (slice 2) | `SELFTEST=infuser` | **`FAIL` + `EXIT:1`** at K | `e2e-launcher-selftest-infuser.log` |
| selftest (this slice) | `SELFTEST=tank` | **`FAIL` + `EXIT:1`** at O (*the dropped chest carries none mB, expected 6500*) | `e2e-launcher-selftest-tank.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=5 ok=5 missing-exit=0 bad-exit=0`
(exit 0).

What the new phases measured (from `e2e-launcher.log`):

- O: a chest with **6,500 mB** and one cobblestone in slot 0, broken in survival with a diamond pickaxe,
  drops **one chest** whose item carries `minecraft:block_entity_data` `{id: "evilcraft:blood_chest",
  fluids: [{Amount: 6500, Fluid: "evilcraft:blood"}]}` — **no `Items`** (the cobblestone spilled as its
  own item entity, `items_left_on_ground: 2`). Placed again: `tank 6500/10000 mB`, slots empty.
- P: the same chest broken **empty** drops an item with **no** payload and places back at `0/10000`; an
  item doctored to `Amount: 20000` places a chest that reads `10000/10000` and whose own saved
  `fluids[0].Amount` is `10000` (`/data get block`), not just the clamped readout.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 5/5 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 4 (0.2.4) — projectile kills give blood, `living_death`

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 4**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`.

| Step | Command | Result |
|---|---|---|
| tests (red) | `cargo test` with `Credit::fills` stubbed | **FAILED 1/36** (`sdk-test-red-slice4.log`) |
| tests (green) | `cargo test` | **36 passed, 0 failed** (`sdk-test.log`) |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings (`sdk-clippy.log`) |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 467 API uses, 14 hooks, exit 0 |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **172,019 B** `a6632c3c…90b8`, `CMP-IDENTICAL`, 62 entries, no `Cargo.lock` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-1/0a875da…/e59da909…`, same sha |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da` | exit 0 — `0a875da  made for` (`sdk-fits.log`) |
| fits (another build) | `… --build 0b14f8a` | exit 0 — `0b14f8a  fits` (`sdk-fits.log`) |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** |

| Leg | Log | Verdict |
|---|---|---|
| default (Q a bow kill fills, R a skeleton's arrow does not, plus the 0.2.3 leg) | `e2e-launcher.log` | `PASS` + `EXIT:0` |
| control (no package) | `e2e-launcher-control.log` | `FAIL` + `EXIT:1` |
| selftest (slice 1, C) | `e2e-launcher-selftest.log` | `FAIL` + `EXIT:1` |
| selftest (this slice, Q) | `e2e-launcher-selftest-blood.log` | `FAIL` + `EXIT:1` |

`check-e2e-logs.py evilcraft-e2e --strict` → `logs=6 ok=6` exit 0.

## Slice 5 (0.2.5) — the Blood Stain

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 5**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with `stain_draw` stubbed | **FAILED 4/40** — *the extractor had room for 1,000 mB / left 0 right 1000* | `sdk-test-red-slice5.log` |
| tests (green) | `cargo test` | **40 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 494 API uses, 16 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **182,285 B** `7a1ead321c53d2a4610c2b78c84327da10b5ec3bb612bf322beef9e81e0e69bc`, `CMP-IDENTICAL`, 75 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (40 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| artifact record | `sdk-artifact-0.2.5.txt` | bytes/sha/entries/manifest/runtime sha, check and fits lines, the packed `data/` list | `sdk-artifact-0.2.5.txt` |

Launcher build: `native-0a875da-f476825c075c`, log line `Compiling evilcraft (1 of 1)`; the export's
`src/`, `data/` and `horizon-mod.toml` are byte-identical to the port (`diff -r`/`cmp` empty, version
`0.2.5`).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` | **`PASS` + `EXIT:0`** — A, A2, B–F, H–U | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=stain` | **`FAIL` + `EXIT:1`** at S | `e2e-launcher-selftest-stain.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=7 ok=7 missing-exit=0 bad-exit=0`
(exit 0).

What the new phases measured (from `e2e-launcher.log`):

- S: a stain placed from its own item reads `0/5000 mB`; with 3,500 mB in it (and the saved compound
  read back as `[{Amount: 3500, Fluid: "evilcraft:blood"}]`) and a 4,000 mB extractor, **one crouching
  click** moved exactly the 1,000 mB the extractor had room for — stain 3,500 → 2,500, extractor
  4,000 → 5,000.
- T: the *same* click standing moved nothing (4,000 / 3,500), the same click crouching moved 1,000, and
  the same crouching click on `minecraft:stone` moved nothing.
- U: a stain at 400 mB drained to 0 and **its block was removed** (`minecraft:air`); the jar's
  `minecraft:replaceable` entry is measured **inert** at this pin (a stone placed onto a stain lands
  above it) — recorded as `blocked(replaceable-not-expressible)`.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 7/7 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 6 (0.2.6) — a fall death leaves a Blood Stain (`BlockBloodStain.bloodStainedBlockEvent`)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 6**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with `stain_blood`/`fall_stain` stubbed | **FAILED 4/45** — *left: 0 right: 400*, *left: None right: Some(FallStain { cell: [10, 100, 20], place: false, tank: 800 })* | `sdk-test-red-slice6.log` |
| tests (green) | `cargo test` | **45 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 510 API uses, 16 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **185,614 B** `b99d911e…1872c`, `CMP-IDENTICAL`, 75 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (45 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | unchanged: slice 1's one unverified `"rolls": 1.0`; no new data file | job report |
| artifact record | `sdk-artifact-0.2.6.txt` | bytes/sha/entries/manifest/runtime+stain sha, check and fits lines, the packed `data/` list | `sdk-artifact-0.2.6.txt` |

Launcher build: `native-0a875da-0d2d381f7c4f`, log line `Compiling evilcraft (1 of 1)`; the export's
`src/`, `data/` and `horizon-mod.toml` are byte-identical to the port (`diff -r`/`cmp` empty, version
`0.2.6`). A first diagnostic build was deleted rather than exported (`notes.md` § Slice 6, harness fact 5).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** — A, A2, B–F, H–W | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=fall` | **`FAIL` + `EXIT:1`** at V — *a 20 HP zombie's fall left a stain holding 0 mB, expected 20 x 20 = 400* | `e2e-launcher-selftest-fall.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=8 ok=8 missing-exit=0 bad-exit=0`
(exit 0).

What the new phases measured (from `e2e-launcher.log`):

- V: a 20 HP zombie dropped six cells down a three-cell-wide stone chimney with `{Health:1.0f}` dies of
  the fall, and the cell it died in (`204,101,-6`) becomes `evilcraft:blood_stain` holding
  **400 mB** (= 20 × 20) — in `/evilcraft stain` (`tank 400/5000 mB`) and in the block entity's own
  saved `fluids` (`data get block … fluids` → `400`).
- X: a second fall death on the same cell leaves **800 mB** in the *same* block and the cell above it
  `minecraft:air` — a top-up, not a second stain.
- W: the same cell and the same zombie killed with a diamond sword stay `minecraft:air` — the death's
  differential: only a fall death stains.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 8/8 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 7 (0.2.7) — the Colossal Blood Chest (`BlockColossalBloodChest`, upstream's hollow cube)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 7**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with `colossal_formed` stubbed to `None` | **FAILED 2/56** — *putting the plank back at [10, 103, -20] forms the cube again* (`left: None / right: Some([10, 101, -20])`), *a chest at [10, 101, -20] is a shell cell of the cube* | `sdk-test-red-slice7.log` |
| tests (green) | `cargo test` | **56 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 518 API uses, 16 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **199,156 B** `71f78499ac66335de499b50957ac522419abe0063cb8eabb8cdf260615b6d9f1`, `CMP-IDENTICAL`, 92 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (56 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 13 recipe/loot files: slice 1's one unverified `"rolls": 1.0`; both new loot tables `ok` | job report |
| artifact record | `sdk-artifact-0.2.7.txt` | bytes/sha/entries/manifest/runtime+colossal sha, check and fits lines, the packed data and asset list | `sdk-artifact-0.2.7.txt` |

Launcher build: `native-0a875da-e9c9da9cf9df`, log line `Compiling evilcraft (1 of 1)`; the
export's `src/`, `data/` and `horizon-mod.toml` are byte-identical to the port (`diff -r`/`cmp`
empty, version `0.2.7`). Two earlier builds of this slice were deleted rather than exported (the
first shipped a `minecraft:stone` controller whose menu never opened — `notes.md` § Slice 7,
harness fact 1).

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** — A, A2, B–F, H–W, Y, Z, AA, AB | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=colossal` | **`FAIL` + `EXIT:1`** at Y — *the hollow 3x3x3 shell at 198,101,-10 is not formed (formed=true)* | `e2e-launcher-selftest-colossal.log` |
| selftest (slices 2–6, history) | `SELFTEST=infuser|tank|blood|stain|fall` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=9 ok=9 missing-exit=0
bad-exit=0` (exit 0).

What the new phases measured (from `e2e-launcher.log`):

- Y: a hollow 3×3×3 of 25 `evilcraft:reinforced_undead_planks` around **one**
  `evilcraft:colossal_blood_chest` (middle cell air) reads `formed=true`, `repair 45`,
  `slots 46`, a 10,000 mB tank; with 955 mB and a damage-241 diamond pickaxe in slot 0 it cured
  **27 points in 54 ticks for 135 mB** (one point a two ticks, 5 mB each), and the tank emptied
  after exactly its own `tank / 5` points (`970 mB` → `194` points → `0` mB, then 40 ticks with
  no repair).
- Z: one shell block removed reads `formed=false` and the same pickaxe and tank stay `250` /
  `1,000 mB` over 48 ticks; the menu lists **50 slots** and a 1,000 mB extractor still empties
  into the tank of the *open* machine (`container 0`); the plank back reads `formed=true` and the
  repair resumes.
- AA: the same damage-243 pickaxe, the same 1,000 mB and a 56-tick window in a plain Blood Chest
  cure **28 points for 140 mB** — the same one-a-two-ticks and 5 mB a point.
- AB: with 4,000 mB and no damaged stack, a diamond axe in survival breaks a shell plank (it drops
  itself) and then the controller, which drops its own item carrying
  `minecraft:block_entity_data` `{id: "evilcraft:colossal_blood_chest", fluids: [{Amount: 4000,
  Fluid: "evilcraft:blood"}]}`.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 9/9 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.
