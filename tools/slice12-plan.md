# Slice 12 — the comparator signal (plan)

**Goal:** carry the analogue (comparator) output of EvilCraft's tank blocks at pin
`0a875da…`/mixins `e59da909…`, prove it in game, package 0.2.12.

**Gap:** slice 9 coverage row `particles, action bar, comparator signal | not ported | deviations 7`.
Tier `deep`; this is the `logic`/redstone half of a machine already carried — no new block,
block entity, entity or renderer.

**Artefact rule (read from the upstream clone, tag `1.21.1-1.2.100`, commit `aa81220`):**
`BlockPurifier.getAnalogOutputSignal`, `BlockDarkTank.getAnalogOutputSignal` and
`BlockWithEntityGuiTank.getAnalogOutputSignal` (the chest, infuser and colossal chest) all do

```java
float output = (float) tile.getTank().getFluidAmount() / (float) tile.getTank().getCapacity();
return (int) Math.ceil(MinecraftHelpers.COMPARATOR_MULTIPLIER * output);
```

with `COMPARATOR_MULTIPLIER = 15` (CyclopsCore `MinecraftHelpersCommon.getComparatorMultiplier`,
branch `master-26-lts`). So: `ceil(15 × amount / capacity)`, clamped to 0..=15.
3000/3000 → 15; 2000/3000 → 10; 6000/10000 → 9; empty → 0.
**Brief discrepancy:** the dispatch note says 2,000 mB on the Purifier reads **9**; the artefact
reads **10** there. The artefact wins (AGENTS.md § *Upstream source analysis*); the report states it
and the 9 case is exercised where the rule really gives 9 (a 10,000 mB Blood Chest at 6,000 mB).

**Route (powah 0.2.14's, proved at this pin):**
1. Pure rule `runtime::comparator_strength(stored, capacity)` + red unit tests.
2. `src/redstone.rs`: `(LevelKey, [i32;3]) → strength` cache (`Outputs`) and a
   `LevelRef → BTreeSet<pos>` queue; `BlockPart::{has_analog_output, analog_output}` answer
   from the cache (the hook's `LevelHost` is read-only and cannot reach the block entity).
3. Machine BE ticks queue their position; `Hooks::level_tick_post` drains it, reads the tank
   from the block entity (back in the world), records the strength, and on a change calls
   `LevelRef::update_neighbors(host, pos, state, NeighbourUpdate::OutputSignal)`.
   `/evilcraft blood` and the two crouching pours publish immediately. `world_lifecycle`
   clears both caches.
4. Registrar: attach a `BlockPart` per tank block (chest, infuser, colossal, purifier, dark tank).
5. In-game: a comparator on a machine's north face, a redstone wire in front of it read at
   `power` (a comparator has no block entity at this pin, so `data get block … OutputSignal`
   cannot read it), `powered=true`; a comparator facing stone reads 0 (vanilla differential);
   `SELFTEST=comparator` inverts the equality → `EXIT:1`. Existing 19 logs stay green.

**Ladder:** red test → glue → green test/fmt/clippy → two identical builds → check/two fits →
SDK gate → launcher build (`Compiling evilcraft (1 of 1)`) → e2e default/control/selftest →
`--strict` → notes/VALIDATION. Docs edited before the final build; `CARGO_INCREMENTAL=0`;
build-slot.sh only; forward compile steps to AdLf-PC, game legs local.
