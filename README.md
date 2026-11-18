# EvilCraft — a Horizon port

**EvilCraft** is a Minecraft mod. An evil mod for Minecraft. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [CyclopsMC/EvilCraft](https://github.com/CyclopsMC/EvilCraft) — licence **MIT**.

| | |
|---|---|
| Upstream mod | EvilCraft ([source](https://github.com/CyclopsMC/EvilCraft)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `evilcraft.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What this slice implements

- **Blood Extractor** (`evilcraft:blood_extractor`): a container for 5,000 mB of blood
  that fills when the player's melee kills a mob — `floor(hp × 5)` to
  `floor(hp × 40)` mB, so a 20 HP zombie gives 100–799 mB.
- **Blood Chest** (`evilcraft:blood_chest`): 10 repair slots and a factor slot over a
  10,000 mB tank of blood. A damaged item loses one damage point every two ticks for
  5 mB; a chipped anvil becomes an anvil for 125 mB; a Blood Extractor in the factor
  slot empties into the tank. Its own 11-slot menu and a tank readout.

Both halves ship in the single `.hmod`: one package carries the block, the item, the menu
type and the code, so installing one file gives the content and the behaviour.

## Not yet ported

- **Recipes are not carried.** `blood_chest` needs `evilcraft:blood_infusion_core`
- **The curse roll is not applied.** `randomBadEnchants = true`,
- **The BER chest model is not carried.** `RenderBlockEntityBloodChest` draws a
- **Projectile kills give no blood.** Upstream hears any player kill whose damage
- **`evilcraft:copy_tank_data`** (the loot function that carries the tank into the
- **The tank holds blood only by convention.** `block_entity_accepts_fluid` is not
- **No blood stain, colossal chest, purifier, blood infuser, dark ore, undead wood,
- **`/evilcraft blood|damage|held` are harness commands**, not upstream features

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `evilcraft.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

EvilCraft was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/CyclopsMC/EvilCraft

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run evilcraft` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/evilcraft/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
