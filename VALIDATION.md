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

## Slice 8 (0.2.8) — the Purifier is blocked; the Dark Tank (`BlockDarkTank`, `BlockEntityDarkTank`)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 8**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

**The brief's first pick, the Purifier, is `blocked(no-purifier-action-table)`**: the jar has no
`data/evilcraft/recipe/purifier/` and the behaviour is four Java `IPurifierAction` classes; one of
them (`DisenchantPurifyAction`) binds `evilcraft:blook`, an item this package does not carry, and
`CollectPotionPurifyAction` binds a living entity in the block's AABB. The fallback (the brief's own)
is the Dark Tank, gap 7's next tank family.

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with the Dark Tank rules stubbed | **FAILED 6/63** — *a 5,000 mB extractor fits a fresh 16,000 mB tank whole* (`left: (0, 0, 0) / right: (5000, 5000, 0)`), *the tank had room for 2,000 mB* (`left: 0 / right: 2000`), *left: 10000 / right: 12000* | `sdk-test-red-slice8.log` |
| tests (green) | `cargo test` | **63 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 519 API uses, 16 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **206,244 B** `4d8f7f67…1de410`, `CMP-IDENTICAL`, 100 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (63 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 14 recipe/loot files: slice 1's one unverified `"rolls": 1.0`; the new loot table `ok` | job report |
| artifact record | `sdk-artifact-0.2.8.txt` | bytes/sha/entries/manifest/runtime+dark_tank sha, check and fits lines, the packed list | `sdk-artifact-0.2.8.txt` |

Launcher build: `native-0a875da-2a04e567b6f4`, log line `Compiling evilcraft (1 of 1)`; the
export's `src/`, `data/`, `assets/` and `horizon-mod.toml` are byte-identical to the port
(`diff -r`/`cmp` empty, version `0.2.8`) — `e2e-export-check-0.2.8.log`.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `launcher-game.sh run evilcraft` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** — A, A2, B–F, H–W, Y, Z, AA, AB, AC, AD, AE | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=darktank` | **`FAIL` + `EXIT:1`** at AC — *the crouching click poured 0 mB into the tank, expected the 2,000 mB room it had* | `e2e-launcher-selftest-darktank.log` |
| selftest (slices 2–7, history) | `SELFTEST=infuser|tank|blood|stain|fall|colossal` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=10 ok=10 missing-exit=0
bad-exit=0` (exit 0).

What the new phases measured (from `e2e-launcher.log`):

- AC: a fresh `evilcraft:dark_tank` reads `0/16000 mB`; with 14,000 mB and a full 5,000 mB extractor
  a crouching click pours exactly the **2,000 mB room** it had (tank 16,000, the extractor keeping
  3,000), and the block entity's own `fluids` reads
  `[{Amount: 16000, Fluid: "evilcraft:blood"}]`.
- AD: the same click **standing** pours nothing (tank 0, extractor 5,000), the same crouching click
  on `minecraft:stone` pours nothing, and the same crouching click on a fresh tank pours the whole
  5,000 (extractor 0).
- AE: in survival a diamond pickaxe breaks the tank, which drops itself carrying
  `minecraft:block_entity_data` `{id: "evilcraft:dark_tank", fluids: [{Amount: 12000, …}]}` — more
  than the ten-bucket machines' clamp — and placing it back reads `12000/16000 mB`.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 10/10 logs end consistently.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 9 (0.2.9) — the Blook, and the Purifier's curse action (`ToolBadEnchantPurifyAction`)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 9**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with `is_curse`/`purify_tick`/`purified` stubbed | **FAILED 4/70** (66 passed, 4 failed) — *left: None / right: Some((11, false))*, *left: None / right: Some((1, false))*, *left: [("minecraft:vanishing_curse", 1), …] / right: [("minecraft:sharpness", 3)]*, *assertion failed: is_curse("minecraft:vanishing_curse")* | `sdk-test-red-slice9.log` |
| tests (green) | `cargo test` | **71 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 558 API uses, 18 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **221,406 B** `e1266a7ffd974cbd488430c397e4d97248ac354bce596b63fd27c85c95c1568f`, `CMP-IDENTICAL`, 113 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91…` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8ac…` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (71 tests, 0 clippy warnings, runtime byte-identical, check + fits, Skills line) | job report |
| dialect gate | `python3 ports/tools/check_261_dialect.py evilcraft.hmod` | 16 recipe/loot files: slice 1's one unverified `"rolls": 1.0`; the new purifier table and the blook recipe `ok` | job report |
| artifact record | `sdk-artifact-0.2.9.txt` | bytes/sha/entries/manifest, every packed `src/` file's sha, check and fits lines, the packed tree | `sdk-artifact-0.2.9.txt` |
| export check | `diff -r`/`cmp` the launcher's export against the port | identical in `src/`, `data/`, `assets/` and the manifest, `version = "0.2.9"`, tag `native-0a875da-a1c6103432dc` | `sdk-export-check-0.2.9.log` |

Launcher build: `native-0a875da-a1c6103432dc`, log line `Compiling evilcraft (1 of 1)`.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (with `OLD=1`, the 0.2.3 export) | **`PASS` + `EXIT:0`** — A, A2, B–F, H–W, Y, Z, AA, AB, AC, AD, AE, AF–AI | `e2e-launcher.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C | `e2e-launcher-selftest.log` |
| selftest (this slice) | `SELFTEST=purifier` | **`FAIL` + `EXIT:1`** at AG — *the pickaxe still carries 1 curse(s) after 140 ticks, expected none* | `e2e-launcher-selftest-purifier.log` |
| selftest (slices 2–8, history) | `SELFTEST=infuser|tank|blood|stain|fall|colossal|darktank` | `FAIL` + `EXIT:1` — kept as history | `e2e-launcher-selftest-*.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=11 ok=11 missing-exit=0
bad-exit=0` (exit 0).

What the new phases measured (from `e2e-launcher.log`):

- AF: a fresh `evilcraft:purifier` reads `0/3000 mB` with an empty purge slot; a **standing** click
  with a `/give`n cursed diamond pickaxe puts exactly one in the slot, and the block entity's own
  saved `Items[0]` reads
  `{components: {"minecraft:enchantments": {"minecraft:vanishing_curse": 1}}, count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}`
  — the positive control of AG's assertion.
- AG: the tank at 3,000 and 140 ticks of the 60-tick action leave `ench 0 curses 0`, the tank at
  **2,000** (one bucket) and the saved `Items[0]` as `{count: 1, Slot: 0b, id: "minecraft:diamond_pickaxe"}`
  — **no `minecraft:enchantments` at all**; the item stays in the slot. The empty-hand click then
  hands it back.
- AH: the *same* cursed pickaxe in a working Blood Chest for the same 140 ticks still carries
  `minecraft:vanishing_curse` in its own saved stack and the chest spends nothing — the
  differential.
- AI: a standing click puts a filled extractor in the purge slot (tank unchanged, the gesture's
  differential), the empty hand takes it back with its 5,000 mB intact, and the crouching click
  pours exactly the 1,000 mB room the 2,000 mB tank had (the extractor keeping 4,000, the saved
  `fluids` 3,000).
- Phase L also crafts the Blook: `minecraft:book -> evilcraft:blook x1 for 500 mB in 500 ticks`,
  taken out of the result slot — the blook's own jar recipe, and the proof that its keys resolve.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 11/11 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 10 (0.2.10) — the three review findings

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`.
Full record: `notes.md` § **Slice 10**; logs under `ports/_shared/evidence/evilcraft/` and
`ports/_shared/evidence/evilcraft-e2e/`. Every compile step ran through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no `--target-dir`).

Two findings were **data-loss bugs in shipped versions**: the anvil step replaced a whole stack
with one anvil (0.2.7/0.2.8), and the extractor stacked while its blood is stored once per stack
(0.2.5–0.2.8). The rules adopted: **preserve the count and components and charge per item** (a
stack of N anvils costs N × 125 mB, and a tank that cannot pay for all N leaves the stack alone);
**the extractor is unstackable**, and a legacy stack is resolved by giving the whole amount to the
first extractor and handing the empty rest back. The third finding (attribution) is a docs fix:
`LICENSE-NOTICE.md` names all ten carried upstream infusion recipe files (nine `convenience/` +
`base/blook.json`), each `cmp`-identical to the jar.

| Step | Command | Result | Log |
|---|---|---|---|
| tests (red) | `cargo test` with `anvil_stack_step` at one item's price and `legacy_extractors`/`one_extractor` stubbed | **FAILED 2/73** (71 passed, 2 failed) — *a stack of two resolves*, *left: Some(("minecraft:anvil", 125)) / right: Some(("minecraft:anvil", 0))* | `sdk-test-red-slice10.log` |
| tests (green) | `cargo test` | **73 passed, 0 failed** | `sdk-test.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 573 API uses, 19 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **225,856 B** `dbbdf5d865689c45d98ab4151fefe00fad04f29f64ac7f669922a9c5364013ef`, `CMP-IDENTICAL`, 113 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | exit 0, `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check.log` |
| fits (the pin) | `cargo horizon-mod fits evilcraft.hmod --build 0a875da91…` | exit 0 — `0a875da  made for` | `sdk-fits.log` |
| fits (another build) | `… --build 0b14f8ac…` | exit 0 — `0b14f8a  fits` | `sdk-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1** (73 tests, 0 clippy warnings, runtime byte-identical, package matches the tree, check + fits, Skills line) | job report |
| artifact record | `sdk-artifact-0.2.10.txt` | bytes/sha/entries/manifest, every packed `src/` file's sha, check and fits lines, the packed tree | `sdk-artifact-0.2.10.txt` |
| export check | `diff -r`/`cmp` the launcher's export against the port | identical in `src/`, `data/`, `assets/` and the manifest, `version = "0.2.10"`, tag `native-0a875da-f804bd58891f` | `sdk-export-check-0.2.10.log` |

Launcher build: `native-0a875da-f804bd58891f`, log line `Compiling evilcraft (1 of 1)`.

| Leg | Command | Verdict | Log |
|---|---|---|---|
| default | `tools/e2e.sh` (`OLD=1` for the 0.2.3 export, `LEGACY=1` for the 0.2.9 one) | **`PASS` + `EXIT:0`** — the two previous-package legs, A, A2, B–F, D2, D3, E2, H–W, Y, Y2, Z, AA–AI | `e2e-launcher-0.2.10.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control-0.2.10.log` |
| selftest (slice 1) | `SELFTEST=1` | **`FAIL` + `EXIT:1`** at C — *the pickaxe was not repaired in 59 ticks* | `e2e-launcher-selftest-0.2.10.log` |
| selftest (finding 1) | `SELFTEST=anvil` | **`FAIL` + `EXIT:1`** at D2 — *the repaired stack … count only 1* | `e2e-launcher-selftest-anvil-0.2.10.log` |
| selftest (finding 2) | `SELFTEST=extractor` | **`FAIL` + `EXIT:1`** at E2 — *the legacy stack reads as 1 extractor slot(s)* | `e2e-launcher-selftest-extractor-0.2.10.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=16 ok=16 missing-exit=0
bad-exit=0` (exit 0). The ten earlier legs stay green in the same default run.

What the new phases measured (from `e2e-launcher-0.2.10.log`):

- **the 0.2.9 leg** writes the legacy save: one stack of two extractors, `held: 4000/5000 mB` —
  0.2.9's item had no `max_stack_size`, so that is a stack a pre-fix player really holds.
- **D2**: the same stack in a Blood Chest is untouched at 125 mB
  (`{components: {"minecraft:custom_name": '"Two anvils"'}, count: 2, Slot: 2b, id: "minecraft:chipped_anvil"}`)
  and becomes **two** anvils for 250 mB with the name kept
  (`… count: 2, Slot: 2b, id: "minecraft:anvil"`), tank 250 → 0.
- **D3**: a damage-244 enchanted pickaxe is repaired (244 → 217 over 40 ticks) and its saved stack
  still carries `minecraft:enchantments: {minecraft:sharpness: 3}` at `count: 1`.
- **E2**: the legacy stack resolves into **two single extractors** (2 items), the first reading
  4,000 mB and the second 0; a crouching click on a 3,500 mB stain draws exactly 1,000 mB into the
  first (5,000) and leaves the second at 0.
- **Y2**: the Colossal Blood Chest does the same anvil step as D2 (two anvils for 250 mB, name
  kept; 125 mB leaves the stack alone) — the machine the finding names has its own code path.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 16/16 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1.

## Slice 11 (0.2.11) — the Vengeance Pickaxe's purifier immunity

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da909…`, game 26.3, launcher 0.2.92.
Compile steps through `build-slot.sh run --job G12-evilcraft --port ports/evilcraft` were forwarded
to **AdLf-PC** (`build-slot: forwarding test for ports/evilcraft to the remote runner`; remote
root `/home/deepseek/atm10`, rust 1.95); the e2e legs stay local (`--step e2e` is not forwarded).
`CARGO_INCREMENTAL=0`, no `--target-dir`, no `BUILD_LOCAL=1`.

| Step | Command | Result | Log |
|---|---|---|---|
| TDD red | `cargo test` (rules stubbed) | **FAILED 1/75** — `a_vengeance_pickaxes_curse_is_immune_to_the_purifier`, `left: 1 / right: 0` | `sdk-test-red-slice11.log` |
| unit tests | `cargo test` | **75 passed, 0 failed** | `sdk-test.log` |
| fmt + clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-clippy.log` |
| launcher game | `ports/tools/launcher-game.sh build evilcraft` | `native-0a875da-93e85970f01e`, `Compiling evilcraft (1 of 1)`; export byte-identical to the port | `launcher-build-0.2.11.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **228,363 B**, sha256 `e80f46bf8feb2d6f40c2947abc34d7d95a72a78a19a9cfa10a3d57b36746f25d`, `CMP-IDENTICAL`, 117 entries, no `Cargo.lock` | `sdk-build.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha | `sdk-check-fits.log` |
| fits, the pin | `… fits … --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for` | `sdk-check-fits.log` |
| fits, another build | `… fits … --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits` | `sdk-check-fits.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | `sdk-artifact-0.2.11.txt` |
| default | `tools/e2e.sh` (`OLD=1` for 0.2.3, `LEGACY=1` for 0.2.9) | **`PASS` + `EXIT:0`** — the two previous-package legs, A, A2, B–F, D2, D3, E2, H–W, Y, Y2, Z, AA–AI, **AJ** | `e2e-launcher-0.2.11.log` |
| control (no package) | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — the chest item does not register | `e2e-launcher-control-0.2.11.log` |
| selftest (this slice) | `SELFTEST=vengeance` | **`FAIL` + `EXIT:1`** at AJ — *after 140 ticks the Vengeance Pickaxe carries 0 curse(s), expected its curse kept* | `e2e-launcher-selftest-vengeance-0.2.11.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=19 ok=19 missing-exit=0
bad-exit=0` (exit 0): every earlier leg stays green, and the three 0.2.11 logs are this slice's.

Transcript of the new phase (verbatim from `e2e-launcher-0.2.11.log`):

```
== AJ: a Vengeance Pickaxe's curse is immune to the Purifier's action ==
  evilcraft:no_such_item is not an item this game has.
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14387
  the carried evilcraft:vengeance_pickaxe reads back in hand
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge evilcraft:vengeance_pickaxe@1, add empty@0, ench 1, curses 1, tick 0, t=14445
  206, 101, -10 has the following block data: {components: {"minecraft:enchantments": {"minecraft:vanishing_curse": 1}}, count: 1, Slot: 0b, id: "evilcraft:vengeance_pickaxe"}
  the saved Vengeance Pickaxe carries the curse
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge evilcraft:vengeance_pickaxe@1, add empty@0, ench 1, curses 1, tick 0, t=14613
  the Vengeance Pickaxe kept its curse and the tank stayed 3000/3000 mB after 140 ticks
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge empty@0, add empty@0, ench 0, curses 0, tick 0, t=14679
  right-click on 206,101,-10 (aim on evilcraft:purifier)
  Purifier [206, 101, -10]: tank 3000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 1, curses 1, tick 20, t=14765
  Purifier [206, 101, -10]: tank 2000/3000 mB, purge minecraft:diamond_pickaxe@1, add empty@0, ench 0, curses 0, tick 0, t=14919
  the same curse on a vanilla pickaxe was stripped for one bucket (the item is the difference)
```

**Fail-capable, three ways.** (1) `SELFTEST=vengeance` inverts AJ's own assertion and exits 1.
(2) The second half of AJ is the differential on the *same* machine, tank and 140 ticks: a cursed
vanilla pickaxe loses its curse and the tank falls to 2,000, so a global "never purify" change
would fail it. (3) The whole-package control has no purifier, so AJ cannot even place one.
An unknown id is refused by `/evilcraft load` before the item is given.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 19/19 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1 (after the final build).

## Slice 12 (0.2.12) — the tank blocks' analogue (comparator) output

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da909…`, game 26.3, launcher 0.2.92.
Compile steps through `build-slot.sh run --job W1-evilcraft --port ports/evilcraft` were forwarded
to **AdLf-PC** (`build-slot: forwarding … to the remote runner`); the launcher build and the e2e
legs stayed local. `CARGO_INCREMENTAL=0`, no `--target-dir`, no `BUILD_LOCAL=1`, no raised slots.
The artefact rule is `ceil(15 × amount / capacity)` (CyclopsCore's `COMPARATOR_MULTIPLIER = 15`);
the brief's "2,000 mB reads 9" was corrected from the artefact — 2,000/3,000 reads 10, and 9 is
exercised at 1,800/3,000. Details in `notes.md` § **Slice 12** and `tools/slice12-plan.md`.

| Step | Command | Result | Log |
|---|---|---|---|
| TDD red | `cargo test` (rule stubbed to 0) | **FAILED 3/78** — *left: 0 / right: 15*, *left: 0 / right: 15*, *left: 0 / right: 10* | `sdk-test-red-slice12.log` |
| unit tests | `cargo test` | **80 passed, 0 failed** | `sdk-final-check-0.2.12.log` |
| fmt / clippy | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, 0 warnings | `sdk-final-check-0.2.12.log` |
| project check | `cargo horizon-mod check` | `Made for 0a875da`, 629 API uses, 23 hooks, exit 0 | `sdk-project-check.log` |
| package ×2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **234,814 B** `d203fad4d2fc96fb574724ae4688fc7d39448eb1517558023e1aab06f26b4e78`, `CMP-IDENTICAL`, 118 entries, no `Cargo.lock` | `sdk-build-check-fits-0.2.12.log` |
| package check | `cargo horizon-mod check evilcraft.hmod` | `native-mod, format native-1/0a875da…/e59da909…, for 26.3`, same sha, exit 0 | `sdk-build-check-fits-0.2.12.log` |
| fits (the pin) | `… fits … --build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a` | `0a875da  made for` | `sdk-build-check-fits-0.2.12.log` |
| fits (another build) | `… fits … --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits` | `sdk-build-check-fits-0.2.12.log` |
| SDK gate | `python3 ports/tools/verify-sdk-package.py evilcraft` | **1/1 package(s) verified** | `sdk-gate-0.2.12.log` |
| launcher game | `ports/tools/launcher-game.sh build evilcraft` | `native-0a875da-ddf8ad0a3724`, `Compiling evilcraft (1 of 1)`; export byte-identical to the port | `launcher-build-0.2.12.log` |

| e2e default | `tools/e2e.sh` (`OLD=1`, `LEGACY=1`) | **`PASS` + `EXIT:0`** — every earlier phase plus AK | `e2e-launcher-0.2.12.log` |
| e2e control | `CONTROL=1 GAME=$CONTROL_GAME` | **`FAIL` + `EXIT:1`** — no chest item | `e2e-launcher-control-0.2.12.log` |
| e2e selftest | `SELFTEST=comparator` | **`FAIL` + `EXIT:1`** at AK — *the comparator read 10 at 2,000 mB* | `e2e-launcher-selftest-comparator-0.2.12.log` |

`python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` → `logs=22 ok=22 missing-exit=0
bad-exit=0` (exit 0): all 19 earlier legs stay green and the three new legs are this slice's.

What the new phase measured (verbatim from `e2e-launcher-0.2.12.log`):

```
== AK: a comparator reads the Purifier's tank fraction (the analogue output) ==
  purifier 3000/3000 mB: comparator powered=true, wire reads 15
  purifier 2000/3000 mB: wire reads 10 (ceil(15 x 2000 / 3000) = 10)
  purifier 1800/3000 mB: wire reads 9 (ceil(15 x 1800 / 3000) = 9)
== AK: a comparator facing vanilla stone reads 0 (the vanilla differential) ==
  stone: comparator powered=false, wire reads 0
```

A comparator has no block entity at this pin, so its output is read as the `power` of the redstone
wire in front of it, never with `data get block … OutputSignal`. The 15 → 10 → 9 re-reads happen
with no nudge between them, which proves the port's `update_neighbors(.., OutputSignal)` path.

### Gates

- `python3 ports/tools/check-e2e-logs.py evilcraft-e2e --strict` — 22/22 logs end in `EXIT:<n>`.
- `python3 ports/tools/check-skills.py --notes ports/evilcraft --strict` — PASS.
- `python3 ports/tools/verify-sdk-package.py evilcraft` — 1/1 (after the final build).
