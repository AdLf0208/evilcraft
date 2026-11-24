# Slice 14 plan — the undead wood set (0.2.14)

**Goal.** Close slice-1 gap 7's *undead wood* row (verbatim: "No blood stain, colossal
chest, purifier, dark ore, **undead wood**, spirits/vengeance, werewolves,
promises/bowls, brooms, or the environmental accumulator — the rest of the jar.")
Carry the jar's own wood set and its own seven crafting recipes; prove them in game;
deliver 0.2.14 as an Update of our own registry entry.

**Approach chosen.** `data`/`content` shape inside the port's **deep** tier: jar
registrations, models/textures, loot tables, tags and recipes only. No new block
entity, machine, renderer, entity, dimension or native hook; the seven recipes are
carried as 26.3-dialect data and read by the game itself. Self-contained, no parent
mod. This is the smallest remaining family that is both in the jar and provable with
a fail-capable `ctl` assertion at this pin.

**Alternatives dropped.** The other gap-7 families: dark ore needs inert worldgen;
spirits/vengeance, werewolves and brooms need entities/AI (outside this slice's
shape); bowls/promises need the infuser's whole progression chain; the
environmental accumulator needs a per-weather tick; the purifier's other three
actions are already `blocked(...)`. Reinforced undead planks is already carried for
the Colossal shell and is **not** duplicated here.

**Enumerated from the jar** (`unzip -l …/evilcraft-1.21.1-neoforge-1.2.100.jar | grep
-i undead`), not from the brief. Every id the brief names is present. 11 new blocks:
`undead_log`, `undead_log_stripped`, `undead_wood`, `undead_wood_stripped`,
`undead_planks`, `undead_planks_stairs`, `undead_slab`, `undead_fence`,
`undead_fence_gate`, `undead_leaves`, `undead_sapling`. The jar also has
`reinforced_undead_planks_stairs` (not in the brief; left for the shell's later
slice) and `tree_undead` worldgen (inert at this pin).

**Seven carried crafting recipes**, each `data/evilcraft/recipe/crafting/<id>.json`
converted from the jar's 1.21.1 ingredient-object shape to 26.3's bare strings:

| file | shape | output |
|---|---|---|
| `undead_planks` | shapeless, `#evilcraft:undead_logs` | 4 undead_planks |
| `undead_wood` | shapeless, 4 undead_log | 3 undead_wood |
| `undead_wood_stripped` | shapeless, 4 undead_log_stripped | 3 undead_wood_stripped |
| `undead_planks_stairs` | shaped, 6 planks | 4 stairs |
| `undead_slab` | shaped, 3 planks | 6 slabs |
| `undead_fence` | shaped, 2 planks + 2 sticks | 3 fences |
| `undead_fence_gate` | shaped, 2 planks + 2 sticks | 1 gate |

The jar's two `undead_logs` tags (block and item) are carried so the shapeless
planks recipe resolves. Its non-crafting recipes (`base/undead_sapling`,
`base/reinforced_undead_planks`, `broom_part_rod_undead`) are not carried: they are
infuser-type/other-family recipes, not the "jar's own crafting recipes" this slice
covers.

**Loot tables.** The simple jar tables are carried as single-pool self-drops. The
leaves table's 1.21.1 `conditions` arrays, the slab's nested `conditions` in
`set_count` and the fence's `survives_explosion` are **not** in the 26.3 dialect
(`check_261_dialect.py` measures them fatal); each is replaced by the minimal
self-drop that is, and the deviation is recorded in `notes.md` with the proof.

**TDD.** Add pure rules and their tests to `src/runtime.rs` with stubs first:
`wood_recipe`, `carried_wood_ids`, `undead_logs_tag`. Watch them fail. Fill them
from the jar's ids/counts. The in-game leg then checks give → place (name + `axis`
from the face), all seven crafts, the unknown-id refusal and a loot drop, with
`SELFTEST=wood` inverting the block-name equality.

**Ladder.** `build-slot.sh` leases only, compile steps forwarded to AdLf-PC, no
`BUILD_LOCAL=1`, no `--target-dir`, `CARGO_INCREMENTAL=0`. Bump both versions to
0.2.14, build twice (`rm -f Cargo.lock` between), `cmp`-identical, `check`, two
`fits` (0a875da made-for + 0b14f8a fits), gate 1/1. Edit docs **before** the final
build so "package matches the tree" is green. Launcher build, then default /
control / `SELFTEST=wood` legs and `check-e2e-logs.py evilcraft-e2e --strict`.
