# Slice 10 / package 0.2.10 — three review findings (two data-loss bugs)

Pin `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins `e59da909…`. 0.2.9 published; this
package is the review fix on top of it.

## Findings and the rule each adopts

| # | Finding | Rule adopted | Why |
|---|---|---|---|
| 1 | `src/colossal.rs:100-117` replaces the whole input stack with **one** repaired anvil and drops its components (count and every component lost). The same code exists in `src/blood.rs:121-126` for the plain Blood Chest. | **Preserve the count and components; charge per item.** A stack of N broken anvils becomes N of the next anvil state for N × 125 mB, every component (a custom name) carried over. A tank that cannot pay for all N leaves the stack **untouched** (no partial replacement, no loss). | "Charge per item" is upstream's own price (`AnvilRepairAction` returns 25 × 5 mB); carrying N items forward is the only option that neither invents a balance number nor destroys items. The skip-when-poor half is what makes it lossless at any tank. |
| 2 | `src/stain.rs:229-246`: a drain debits the stain **once** and writes the resulting blood onto the **whole held stack**, so every later split of that stack carries the same amount again (duplication) or the stack is counted once for N items (loss). The same write shape is in `dark_tank.rs`, `purifier.rs`, the machines' container slots and `fill_for`. | **Extractors are unstackable (`max_stack_size = 1`).** A stack from an older save is resolved deterministically: **the first extractor keeps the whole stored amount (clamped to 5,000 mB) and the others are empty; the item count is preserved and the extras are `/give`n back to the player.** No drain path ever writes onto a stack of more than one; the machine container slot leaves such a stack alone for the player to take out. | The amount is stored once per stack, so "each extractor carries it" is not representable; giving the total to the first item conserves it exactly (sum before == sum after), and the extras are handed back as items, so nothing is lost either. |
| 3 | Attribution: the nine carried upstream infusion recipe files are not credited in `LICENSE-NOTICE.md` (0.2.8 credited the colossal-chest and reinforced-planks textures only). | Name all carried upstream recipe files, **ten** as of 0.2.9 (the nine `convenience/` files of the finding + `base/blook.json`, carried by 0.2.9), each verified `cmp`-identical to the jar, and state the only change (the `convenience/` directory flattened). | CC BY 4.0 §3(a) needs the changes stated per carried file. |

## Red first

`src/runtime.rs` gains the two rules as stubs returning "the old behaviour" (`anvil_stack_step`
→ the single-item step is all there is; `legacy_extractors` → None, `one_extractor` → true for
every count), and the tests:

- `a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each`
- `a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood`

Red log `sdk-test-red-slice10.log`, green `sdk-test.log`.

## Glue

- `blood::anvil_stack(host, next, from, count)` — the new item's stack of `count`, its component
  patch copied from the old stack (`Components::as_nbt` + `set_components`); used by both machines.
- `blood::split_extractor_stack(stack)` — `(kept, extras)` from `runtime::legacy_extractors`.
- `ItemPart::inventory_tick` on the extractor: a carried stack of >1 is resolved (kept written to
  its slot, extras `/give`n); `fill_for` resolves the same way before filling.
- `runtime::one_extractor(count)`: the guard every drain path (stain draw, dark tank pour,
  purifier pour, the three machines' container slot) asks before it reads or writes blood.

## Proof

- unit: the two rules; `one_extractor` around 0/1/2.
- in game (default leg): **D2** a count-2 chipped-anvil stack in a Blood Chest becomes two anvils
  for 250 mB with its custom name kept, and the same stack with 125 mB is left untouched;
  **D3** a count-2 enchanted damaged pickaxe keeps both pickaxes and the enchantment while its
  damage falls; **E2** a count-2 extractor stack (`/item replace … 2`, the shape a pre-fix save
  holds) is split into two single extractors before a drain, the first carrying the whole 4,000 mB
  and the second 0, then a crouching click on a stain fills only the first (stain 3,500 → 2,500,
  held 5,000, the other still 0); **Y2** the Colossal Blood Chest does the same anvil step as D2
  (two anvils for 250 mB, name kept; 125 mB leaves it alone).
- selftests `SELFTEST=anvil` (D2's count) and `SELFTEST=extractor` (E2's split), each `EXIT:1`;
  the whole-package control still fails at A.
