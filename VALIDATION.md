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
