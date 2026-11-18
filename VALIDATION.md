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
