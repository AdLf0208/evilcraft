// The pure rules of EvilCraft's blood repair loop (slice 1).
//
// Upstream runs a Blood Chest's ten repair slots through
// `RepairItemTickAction`/`BloodChestRepairActionRegistry` and its Blood Extractor
// through `ItemBloodExtractor.fillForAllBloodExtractors`. Those rules are stated
// here as plain functions so they can be tested without the game:
//
// - `BloodChestRepairActionRegistry` (DamageableItemRepairAction, AnvilRepairAction)
// - `BlockBloodChestConfig.mBPerDamage = 5`, `.ticksPerDamage = 2`,
//   `.randomBadEnchantsChance = 10000`, `.itemBlacklist = ["minecraft:stick"]`
// - `BlockEntityBloodChest.LIQUID_PER_SLOT = FluidHelpers.BUCKET_VOLUME * 10`
// - `ItemBloodExtractorConfig.containerSize = 5000`, `.minimumMobMultiplier = 5`,
//   `.maximumMobMultiplier = 40`
//
// See `FINDINGS.md` for the file each number was read from.

/// One bucket, as the pin counts fluids (`FluidHelpers.BUCKET_VOLUME`).
pub const BUCKET: i32 = 1_000;
/// Upstream `BlockEntityBloodChest.LIQUID_PER_SLOT`: ten buckets of blood.
pub const TANK_CAPACITY: i32 = 10 * BUCKET;
/// Upstream `BlockBloodChestConfig.mBPerDamage`.
pub const MB_PER_DAMAGE: i32 = 5;
/// Upstream `BlockBloodChestConfig.ticksPerDamage`.
pub const TICKS_PER_DAMAGE: u32 = 2;
/// Upstream `AnvilRepairAction.repair` returns the multiplier `25F`, which
/// `RepairItemTickAction` scales by `mBPerDamage`: an anvil step costs 125 mB.
pub const ANVIL_MULTIPLIER: i32 = 25;
/// `ceil(mBPerDamage × ANVIL_MULTIPLIER)`, the anvil step's price.
pub const ANVIL_MB: i32 = MB_PER_DAMAGE * ANVIL_MULTIPLIER;
/// Upstream `ItemBloodExtractorConfig.containerSize`.
pub const EXTRACTOR_CAPACITY: i32 = 5_000;
/// The Blood Infuser, this slice's second machine (upstream `BlockBloodInfuser`).
pub const INFUSER: &str = "evilcraft:blood_infuser";
/// The infuser's own recipe type: `data/evilcraft/recipe/blood_infuser/*.json`, read at
/// run time through `Host::recipes_of_type` (`RecipeBloodInfuser`, `RecipeSerializerBloodInfuser`).
pub const INFUSION_TYPE: &str = "evilcraft:blood_infuser";
/// Upstream `BlockEntityBloodInfuser.SLOT_CONTAINER`.
pub const INFUSER_CONTAINER_SLOT: usize = 0;
/// Upstream `BlockEntityBloodInfuser.SLOT_INFUSE`.
pub const INFUSER_INPUT_SLOT: usize = 1;
/// Upstream `BlockEntityBloodInfuser.SLOT_INFUSE_RESULT`.
pub const INFUSER_OUTPUT_SLOT: usize = 2;
/// Upstream `BlockEntityBloodInfuser.LIQUID_PER_SLOT`: ten buckets, as the chest's tank.
pub const INFUSER_TANK: i32 = 10 * BUCKET;
/// Upstream `BlockEntityWorking.INVENTORY_SIZE_UPGRADES`.
pub const UPGRADE_SLOTS: usize = 4;
/// The infuser's slots: its three machine slots, then the four upgrade slots.
pub const INFUSER_SLOTS: usize = 3 + UPGRADE_SLOTS;
/// Upstream `ItemPromise`'s tier upgrades (`Upgrades.UPGRADE_TIER1..3`), the machine's tier.
pub const TIER_PROMISES: &[(&str, u32)] = &[
    ("evilcraft:promise_tier_1", 1),
    ("evilcraft:promise_tier_2", 2),
    ("evilcraft:promise_tier_3", 3),
];

/// One `evilcraft:blood_infuser` recipe, as its own file states it
/// (`RecipeBloodInfuser`): an item, the blood it costs, what it becomes, and how long.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Infusion {
    /// The recipe's file id, `<namespace>:<path>`, for the readout.
    pub id: String,
    /// The item the infusion slot must hold.
    pub input: String,
    /// What the output slot receives.
    pub output: String,
    /// How many of it (`output_item.count`, one when the file leaves it out).
    pub output_count: u32,
    /// `input_fluid.amount`, spent once when the infusion completes.
    pub blood: i32,
    /// `duration`, the ticks the infusion takes.
    pub ticks: u32,
    /// `tier`, the machine tier the recipe needs (0/absent for none).
    pub tier: u32,
}

/// The recipe for `input` on a machine of `tier`, or nothing. Upstream's
/// `RecipeBloodInfuser.matches` compares the recipe's tier with the machine's, and
/// the machine's own `Metadata.getTier` reads it from a promise in an upgrade slot.
///
/// Two files may infuse the same item (`minecraft:rotten_flesh` makes leather and
/// spider eyes); the one with the highest tier wins, as upstream's `getRecipe` takes
/// the highest-tier match.
#[must_use]
pub fn infusion_for<'a>(recipes: &'a [Infusion], input: &str, tier: u32) -> Option<&'a Infusion> {
    recipes
        .iter()
        .filter(|recipe| recipe.input == input && recipe.tier <= tier)
        .max_by_key(|recipe| recipe.tier)
}

/// One tick of an infusion: `(progress after the tick, whether it completed)`.
///
/// Upstream's `TickComponent.tick` adds one to the action's tick, runs it, and resets
/// the tick to 0 once it reaches the recipe's duration; `InfuseItemTickAction.canTick`
/// refuses to tick while the tank cannot pay the recipe's blood, which resets the
/// progress. `None` is that refusal.
///
/// `duration` is checked to be at least one: `RecipeSerializerBloodInfuser` refuses a
/// zero duration, so a file that got here has one.
#[must_use]
pub fn infuse_tick(progress: u32, recipe: &Infusion, tank: i32) -> Option<(u32, bool)> {
    if tank < recipe.blood {
        return None;
    }
    let next = progress.saturating_add(1);
    if next >= recipe.ticks.max(1) {
        Some((0, true))
    } else {
        Some((next, false))
    }
}

/// The machine's tier: the highest promise in its upgrade slots, 0 without one
/// (upstream `BlockEntityWorking.Metadata.getTier` + `ItemPromise.isTierUpgrade`).
///
/// Upstream also gates which upgrade slot a tier fits in; this port carries only the
/// three tier promises, so the slot index does not change the answer and is not modelled.
#[must_use]
pub fn machine_tier<'a>(upgrades: impl IntoIterator<Item = &'a str>) -> u32 {
    upgrades
        .into_iter()
        .filter_map(|item| {
            TIER_PROMISES
                .iter()
                .find(|(id, _)| *id == item)
                .map(|(_, tier)| *tier)
        })
        .max()
        .unwrap_or(0)
}
/// Upstream `ItemBloodExtractorConfig.minimumMobMultiplier`.
pub const MIN_MOB_MULTIPLIER: f64 = 5.0;
/// Upstream `ItemBloodExtractorConfig.maximumMobMultiplier`.
pub const MAX_MOB_MULTIPLIER: f64 = 40.0;
/// Upstream `BlockBloodChestConfig.randomBadEnchantsChance` (1 in this many).
/// Kept as the documented upstream value; this slice does not apply curses
/// (see the gap list in `notes.md`).
#[allow(
    dead_code,
    reason = "upstream's curse chance, not applied in this slice"
)]
pub const BAD_ENCHANT_CHANCE: u32 = 10_000;

/// Upstream `BlockEntityBloodChest.SLOTS_CHEST`: `2 rows × 5 columns`.
pub const CHEST_SLOTS: usize = 10;
/// Upstream `BlockEntityBloodChest.SLOT_CONTAINER`.
pub const CONTAINER_SLOT: usize = CHEST_SLOTS;
/// Upstream `BlockEntityBloodChest.SLOTS`.
pub const SLOT_COUNT: usize = CHEST_SLOTS + 1;

/// The block this port carries.
pub const CHEST: &str = "evilcraft:blood_chest";
/// The item this port carries.
pub const EXTRACTOR: &str = "evilcraft:blood_extractor";
/// The Blood Stain: the splash a mob's fall leaves, and an extractor's source.
pub const STAIN: &str = "evilcraft:blood_stain";
/// `ItemBlookConfig`: a Book infused with Blood (`ItemBlookConfig` registers a plain item).
pub const BLOOK: &str = "evilcraft:blook";
/// Every item id this package registers besides vanilla's: the extractor, the blook and the
/// three tier promises (`data/evilcraft/horizon/item/`). A carried recipe may name these or
/// `minecraft:`; nothing else is an item the game or the package has.
pub const CARRIED_ITEMS: &[&str] = &[
    EXTRACTOR,
    BLOOK,
    "evilcraft:promise_tier_1",
    "evilcraft:promise_tier_2",
    "evilcraft:promise_tier_3",
];
/// Upstream `BlockEntityBloodStain.CAPACITY`: five buckets of blood.
pub const STAIN_TANK: i32 = 5 * BUCKET;
/// The tank's fluid id. Horizon has no blood block or bucket, so this is a tank id.
pub const BLOOD: &str = "evilcraft:blood";

/// Upstream `BlockBloodChestConfig.itemBlacklist`.
pub const BLACKLIST: &[&str] = &["minecraft:stick"];

/// The three anvil states, as `AnvilRepairAction` names them.
pub const ANVIL: &str = "minecraft:anvil";
pub const CHIPPED_ANVIL: &str = "minecraft:chipped_anvil";
pub const DAMAGED_ANVIL: &str = "minecraft:damaged_anvil";

/// The number of values `nextInt` is asked for at a kill:
/// `max(1, floor(hp × 40) − floor(hp × 5))`.
#[must_use]
pub fn kill_span(max_health: f32) -> u32 {
    let minimum = (f64::from(max_health) * MIN_MOB_MULTIPLIER) as i32;
    let maximum = (f64::from(max_health) * MAX_MOB_MULTIPLIER) as i32;
    (maximum - minimum).max(1) as u32
}

/// How much blood a player kill puts into the extractors:
/// `floor(hp × 5) + nextInt(max(1, floor(hp × 40) − floor(hp × 5)))`.
///
/// `roll` stands in for `RandomSource.nextInt`: the caller gives a number from
/// the level's random and this reduces it, so the arithmetic stays pure.
#[must_use]
pub fn blood_from_kill(max_health: f32, roll: u32) -> i32 {
    let minimum = (f64::from(max_health) * MIN_MOB_MULTIPLIER) as i32;
    minimum + (roll % kill_span(max_health)) as i32
}

/// Whether this tick repairs a damage point (upstream `ticksPerDamage`).
#[must_use]
pub fn repair_due(ticks: u32) -> bool {
    ticks.is_multiple_of(TICKS_PER_DAMAGE)
}

/// Whether a stack may sit in a repair slot: damageable and damaged, or an
/// anvil state, and never the blacklist (`isItemValidForSlot` + `canRepair`).
#[must_use]
pub fn can_hold(item: &str, damage: i32) -> bool {
    if BLACKLIST.contains(&item) {
        return false;
    }
    damage > 0 || anvil_target(item).is_some()
}

/// One damage point for `MB_PER_DAMAGE` mB: `(damage, tank)` after the repair,
/// or nothing when the item is whole or the tank cannot pay.
#[must_use]
pub fn repair(damage: i32, tank: i32) -> Option<(i32, i32)> {
    if damage <= 0 || tank < MB_PER_DAMAGE {
        return None;
    }
    Some((damage - 1, tank - MB_PER_DAMAGE))
}

/// The item a broken anvil becomes (`AnvilRepairAction`).
fn anvil_target(item: &str) -> Option<&'static str> {
    match item {
        CHIPPED_ANVIL => Some(ANVIL),
        DAMAGED_ANVIL => Some(CHIPPED_ANVIL),
        _ => None,
    }
}

/// One anvil step for `ANVIL_MB` mB: `(item it becomes, tank)` after the step,
/// or nothing when it is not a broken anvil or the tank cannot pay.
#[must_use]
pub fn anvil_step(item: &str, tank: i32) -> Option<(&'static str, i32)> {
    anvil_stack_step(item, 1, tank)
}

/// One step for a **stack** of `count` broken anvils: `(item each becomes, tank)` after
/// the step, or nothing when it is not a broken anvil or the tank cannot pay for all of
/// them.
///
/// Upstream's `AnvilRepairAction.repair` answers a fresh `ItemStack` of **one** item, so a
/// stack of anvils lost every item but the first (and every component on it) — a data-loss
/// defect this port does not reproduce (AGENTS.md, "Upstream defects"). The price stays
/// upstream's: `ANVIL_MB` for each anvil converted. A tank that cannot pay for the whole
/// stack leaves it untouched, so no step can ever drop an anvil.
#[must_use]
pub fn anvil_stack_step(item: &str, count: u32, tank: i32) -> Option<(&'static str, i32)> {
    if count == 0 {
        return None;
    }
    let price = ANVIL_MB.checked_mul(i32::try_from(count).ok()?)?;
    if tank < price {
        return None;
    }
    anvil_target(item).map(|next| (next, tank - price))
}

/// What a broken machine's own item carries (`LootFunctionCopyTankData`): the tank's
/// amount, or nothing when the tank is empty.
///
/// Upstream inserts the fluid into the item only when it is not empty
/// (`if (!resource.isEmpty())`), so a broken empty machine drops a plain item that needs
/// no restoring on placement. A negative amount is not a tank either.
/// The tank a broken machine's item carries, clamped to the machine's own capacity, or
/// nothing at all: an empty tank travels on no item, and a negative amount is not a tank.
#[must_use]
pub fn carried_tank_with(tank: i32, capacity: i32) -> Option<i32> {
    (tank > 0).then(|| tank.min(capacity))
}

/// The tank a broken ten-bucket machine's item carries.
#[must_use]
pub fn carried_tank(tank: i32) -> Option<i32> {
    carried_tank_with(tank, TANK_CAPACITY)
}

/// The tank a placed item's payload restores, clamped to the machine's capacity
/// (upstream sets the item's tank capacity first, so the insert cannot pass it; a
/// `/give`d item naming 20,000 mB places a full machine).
#[must_use]
pub fn restored_tank_with(carried: i32, capacity: i32) -> i32 {
    carried.clamp(0, capacity)
}

/// The tank a placed ten-bucket machine's payload restores.
#[must_use]
pub fn restored_tank(carried: i32) -> i32 {
    restored_tank_with(carried, TANK_CAPACITY)
}

/// Who a mob's death is credited to, as the extractor's rule reads it.
///
/// Upstream (`ItemBloodExtractor.bloodObtainEvent`) fills only when the killing blow's
/// damage source names a `ServerPlayer` as its causing entity: a player's melee blow and a
/// player's arrow do, a skeleton's arrow does not (its causing entity is the skeleton), and
/// neither do a fall, lava, the void or `/kill`, whose source names no entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Credit {
    /// A player is credited with the kill.
    Player,
    /// Nobody is: a mob's blow or arrow, the environment, `/kill`.
    Nobody,
}

impl Credit {
    /// Whether this death fills the killer's extractors: only a player's kill does.
    #[must_use]
    pub fn fills(self) -> bool {
        matches!(self, Self::Player)
    }
}

/// Empty a blood container into a tank of `capacity`, never past it.
/// Returns `(tank, what the container keeps)`.
#[must_use]
pub fn drain_container_into(tank: i32, container: i32, capacity: i32) -> (i32, i32) {
    let room = (capacity - tank).max(0);
    let moved = container.clamp(0, room);
    (tank + moved, container - moved)
}

/// Empty a blood container into the chest's tank, never past its capacity.
/// Returns `(tank, what the container keeps)`.
#[must_use]
pub fn drain_container(tank: i32, container: i32) -> (i32, i32) {
    drain_container_into(tank, container, TANK_CAPACITY)
}

/// Fill one Blood Extractor from the kill's pool, never past its capacity.
/// Returns `(what the extractor holds, what the pool keeps)`.
#[must_use]
pub fn fill_extractor(held: i32, pool: i32) -> (i32, i32) {
    let room = (EXTRACTOR_CAPACITY - held).max(0);
    let moved = pool.clamp(0, room);
    (held + moved, pool - moved)
}

/// Whether a stack of `count` Blood Extractors may be filled or drained as it stands.
///
/// The amount is stored **once per stack** (`minecraft:custom_data.evilcraft:blood`), so a
/// stack of two is one amount, not two: a fill on it is counted once for two items (loss),
/// and every later split of the stack copies the amount onto each item (duplication). The
/// item is therefore unstackable (`max_stack_size = 1`), and only a single extractor is a
/// container; a larger stack is a save from before that fix and is resolved first
/// ([`legacy_extractors`]).
#[must_use]
pub fn one_extractor(count: u32) -> bool {
    count == 1
}

/// How a stack of `count` extractors holding `blood` is resolved: one amount per item, the
/// **first** taking the whole stored amount (clamped to the item's own 5,000 mB) and the
/// rest empty. `None` for a single extractor, which needs no resolution.
///
/// The sum is the amount the stack stored — the rule conserves blood exactly, and the item
/// count travels unchanged: nothing is duplicated and nothing is lost.
#[must_use]
pub fn legacy_extractors(count: u32, blood: i32) -> Option<Vec<i32>> {
    if count <= 1 {
        return None;
    }
    let mut amounts = vec![0; usize::try_from(count).ok()?];
    amounts[0] = blood.clamp(0, EXTRACTOR_CAPACITY);
    Some(amounts)
}

/// One crouching click with a Blood Extractor on a Blood Stain, as upstream's
/// `ItemBloodExtractor.onItemUseFirst` performs it: the click moves the stain's blood
/// into the extractor itself, so nothing reads or writes a block on the way.
///
/// The stain keeps what the extractor has no room for, and the extractor takes no more
/// than the stain holds (`move(source, .., Integer.MAX_VALUE, ..)` bounded by the item's
/// tank): the click is one transfer, never a spill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StainDraw {
    /// What the stain keeps.
    pub stain: i32,
    /// What the extractor holds afterwards.
    pub extractor: i32,
    /// How much moved from the stain into the extractor.
    pub moved: i32,
}

impl StainDraw {
    /// Whether the click emptied the stain: upstream's block entity removes its block as
    /// soon as a committed change leaves the tank at 0 (`BlockEntityBloodStain.setAmount`).
    #[must_use]
    pub fn drains_the_stain(self) -> bool {
        self.moved > 0 && self.stain == 0
    }
}

/// The blood a crouching click moves from a stain into an extractor: as much as the
/// extractor has room for, never more than the stain holds. A stain never holds more
/// than its own 5-bucket tank, whatever a `/give`d payload or a config says.
#[must_use]
pub fn stain_draw(stain: i32, extractor: i32) -> StainDraw {
    let stain = stain.clamp(0, STAIN_TANK);
    let extractor = extractor.clamp(0, EXTRACTOR_CAPACITY);
    let room = (EXTRACTOR_CAPACITY - extractor).max(0);
    let moved = stain.min(room);
    StainDraw {
        stain: stain - moved,
        extractor: extractor + moved,
        moved,
    }
}

// ---------------------------------------------------------------------------
// Slice 6: a fall death leaves a Blood Stain (`BlockBloodStain.bloodStainedBlockEvent`)
// ---------------------------------------------------------------------------

/// Upstream `BlockBloodStainConfig.bloodMBPerHP`: 20 mB of blood for each point of max health.
pub const BLOOD_MB_PER_HP: i32 = 20;

/// Where a fall death's blood goes and what the cell holds afterwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallStain {
    /// The cell the blood belongs to: the stain below the mob, or the mob's own cell.
    pub cell: [i32; 3],
    /// Whether that cell needs a fresh stain first (it was empty).
    pub place: bool,
    /// The tank afterwards, never above [`STAIN_TANK`].
    pub tank: i32,
}

/// What the mob's own cell holds — the cell its feet are in, which is where a stain that
/// killed it before stands (upstream's `get(pos, BlockEntityBloodStain.class)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// Empty, and standing on a floor a stain can rest on (upstream's `isEmptyBlock` +
    /// `canSurvive`): a fresh stain goes here.
    Free,
    /// Already a stain, holding this much blood: this death tops it up.
    Stain(i32),
    /// Anything else: no stain can be placed and there is none to top up.
    Taken,
}

/// The blood a mob's fall leaves: `bloodMBPerHP x max health`, clamped to the stain's tank.
///
/// Upstream truncates the float product to an `int` and the block entity's `addAmount`
/// clamps the sum to `CAPACITY`; a mob with no health leaves nothing.
#[must_use]
pub fn stain_blood(max_health: f32) -> i32 {
    let amount = f64::from(max_health) * f64::from(BLOOD_MB_PER_HP);
    (amount as i32).clamp(0, STAIN_TANK)
}

/// Which cell a fall death stains, and what it holds afterwards.
///
/// Upstream's `bloodStainedBlockEvent` reads the block **below** the dying mob first: when it
/// is already a stain the blood goes to that cell. Otherwise it moves to the mob's own cell:
/// a stain already standing there (the mob died on top of it) is topped up, an empty cell
/// gets a fresh stain, and anything else is left alone. `below` is the tank of the stain
/// below, when there is one.
#[must_use]
pub fn fall_stain(
    victim: [i32; 3],
    max_health: f32,
    below: Option<i32>,
    own: Cell,
) -> Option<FallStain> {
    let blood = stain_blood(max_health);
    if blood == 0 {
        return None;
    }
    let topped_up = |existing: i32| (existing.clamp(0, STAIN_TANK) + blood).clamp(0, STAIN_TANK);
    if let Some(existing) = below {
        return Some(FallStain {
            cell: [victim[0], victim[1] - 1, victim[2]],
            place: false,
            tank: topped_up(existing),
        });
    }
    let tank = match own {
        Cell::Stain(existing) => topped_up(existing),
        Cell::Free => topped_up(0),
        Cell::Taken => return None,
    };
    Some(FallStain {
        cell: victim,
        place: own == Cell::Free,
        tank,
    })
}

// ---------------------------------------------------------------------------
// Slice 7: the Colossal Blood Chest (`BlockColossalBloodChest`)
// ---------------------------------------------------------------------------

/// `BlockColossalBloodChest`: the controller of the 3×3×3 structure.
pub const COLOSSAL: &str = "evilcraft:colossal_blood_chest";
/// `BlockReinforcedUndeadPlank`: the block the rest of the shell is built from.
pub const REINFORCED_PLANKS: &str = "evilcraft:reinforced_undead_planks";
/// `ContainerColossalBloodChest.CHEST_INVENTORY_ROWS × …_COLUMNS` = 5 × 9.
pub const COLOSSAL_SLOTS: usize = 45;
/// `BlockEntityColossalBloodChest.SLOT_CONTAINER`: the fluid-container slot.
pub const COLOSSAL_CONTAINER_SLOT: usize = COLOSSAL_SLOTS;
/// `BlockEntityColossalBloodChest.SLOTS`: the repair slots and the container slot.
pub const COLOSSAL_SLOT_COUNT: usize = COLOSSAL_SLOTS + 1;
/// The machine's whole inventory: its own slots and `BlockEntityWorking.INVENTORY_SIZE_UPGRADES`.
pub const COLOSSAL_INVENTORY: usize = COLOSSAL_SLOT_COUNT + UPGRADE_SLOTS;
/// `BlockEntityColossalBloodChest.exactSizeValidator`'s `ExactSizeValidator(new Vec3i(2, 2, 2))`:
/// the inner size of a hollow cube, so the cube a controller sits in is 3×3×3.
pub const COLOSSAL_SIZE: i32 = 3;
/// The shell's cells: the cube's 27 cells less the one air cell in its middle.
pub const COLOSSAL_SHELL: usize = 26;

/// What one cell of the 3×3×3 cube holds, as `CubeDetector` reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    /// The controller itself (`evilcraft:colossal_blood_chest`).
    Chest,
    /// A reinforced undead plank.
    Planks,
    /// Air: allowed in the cube's middle, nowhere else.
    Air,
    /// Anything else.
    Other,
}

/// The cell a block id is. Upstream's detector allows exactly two blocks in the cube's
/// shell (`getCubeDetector`) and calls everything else invalid; its hollow rule lets only
/// air into the middle.
#[must_use]
pub fn structure_cell(id: &str) -> Shell {
    match id {
        COLOSSAL => Shell::Chest,
        REINFORCED_PLANKS => Shell::Planks,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => Shell::Air,
        _ => Shell::Other,
    }
}

/// The minimum corner and the cell count of the material `controller` sits in, found by
/// walking the structure's own blocks out from it — upstream's `HollowCubeDetector`,
/// which flood-fills the allowed blocks rather than guessing a corner at each axis.
///
/// The walk stops as soon as the region is bigger than a 3×3×3 cube could be, so a wall
/// of planks costs 28 cells and no more: `ExactSizeValidator`'s rule is that the region
/// must be exactly one cube, never a piece of a larger mass.
#[must_use]
fn colossal_region(
    controller: [i32; 3],
    cell: &impl Fn([i32; 3]) -> Shell,
) -> Option<([i32; 3], usize)> {
    let material = |pos: [i32; 3]| matches!(cell(pos), Shell::Chest | Shell::Planks);
    if !material(controller) {
        return None;
    }
    let mut seen: Vec<[i32; 3]> = Vec::with_capacity(COLOSSAL_SHELL + 1);
    let mut stack = vec![controller];
    let mut min = controller;
    let mut max = controller;
    while let Some(pos) = stack.pop() {
        if seen.contains(&pos) || !material(pos) {
            continue;
        }
        seen.push(pos);
        if seen.len() > COLOSSAL_SHELL {
            return None;
        }
        for axis in 0..3 {
            min[axis] = min[axis].min(pos[axis]);
            max[axis] = max[axis].max(pos[axis]);
        }
        for axis in 0..3 {
            for step in [-1, 1] {
                let mut next = pos;
                next[axis] += step;
                stack.push(next);
            }
        }
    }
    Some((min, seen.len()))
}

/// The structure a controller at `controller` forms: its minimum corner when the blocks
/// reachable from it are exactly the 26 cells of a 3×3×3 cube's shell, the cube's middle is
/// air (`HollowCubeDetector` + `ExactSizeValidator(2, 2, 2)`), and the controller is the
/// shell's **only** `colossal_blood_chest` (`ExactBlockCountValidator(1)`).
/// `None` for every other arrangement.
#[must_use]
pub fn colossal_formed(controller: [i32; 3], cell: impl Fn([i32; 3]) -> Shell) -> Option<[i32; 3]> {
    let (anchor, cells) = colossal_region(controller, &cell)?;
    if cells != COLOSSAL_SHELL {
        return None;
    }
    let mut chests = 0;
    for x in 0..COLOSSAL_SIZE {
        for y in 0..COLOSSAL_SIZE {
            for z in 0..COLOSSAL_SIZE {
                let pos = [anchor[0] + x, anchor[1] + y, anchor[2] + z];
                match ((x, y, z) == (1, 1, 1), cell(pos)) {
                    (true, Shell::Air) => {}
                    (false, Shell::Chest) => chests += 1,
                    (false, Shell::Planks) => {}
                    _ => return None,
                }
            }
        }
    }
    (chests == 1).then_some(anchor)
}

// ---------------------------------------------------------------------------
// Slice 8: the Dark Tank (`BlockDarkTank`, `BlockEntityDarkTank`)
// ---------------------------------------------------------------------------

/// `BlockEntityDarkTank.BASE_CAPACITY`: sixteen buckets of blood.
pub const DARK_TANK_CAPACITY: i32 = 16 * BUCKET;
/// `BlockDarkTank`: the block, its block entity type and its item's id.
pub const DARK_TANK: &str = "evilcraft:dark_tank";

/// What one crouching click with a filled Blood Extractor leaves behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TankFill {
    /// The tank afterwards, never above [`DARK_TANK_CAPACITY`].
    pub tank: i32,
    /// What the extractor keeps.
    pub extractor: i32,
    /// How much moved from the extractor into the tank.
    pub moved: i32,
}

/// The blood one crouching click moves from a filled Blood Extractor into a Dark Tank, as
/// upstream's `BlockDarkTank.useItemOn` fills a tank from a held fluid container
/// (`FluidUtil.interactWithFluidHandler`): as much as the tank has room for, never more
/// than the extractor holds, so a full tank leaves the extractor untouched and a partly
/// filled one keeps what did not fit.
#[must_use]
pub fn tank_fill(tank: i32, extractor: i32) -> TankFill {
    let tank = tank.clamp(0, DARK_TANK_CAPACITY);
    let extractor = extractor.clamp(0, EXTRACTOR_CAPACITY);
    let (filled, keeps) = drain_container_into(tank, extractor, DARK_TANK_CAPACITY);
    TankFill {
        tank: filled,
        extractor: keeps,
        moved: extractor - keeps,
    }
}

// ---------------------------------------------------------------------------
// Slice 9: the Purifier (`BlockPurifier`, `ToolBadEnchantPurifyAction`)
// ---------------------------------------------------------------------------

/// `BlockPurifier`: the block, its block entity type and its item's id.
pub const PURIFIER: &str = "evilcraft:purifier";
/// `BlockEntityPurifier.SLOT_PURIFY`: the stack the machine works on.
pub const PURIFIER_PURIFY_SLOT: usize = 0;
/// `BlockEntityPurifier.SLOT_ADDITIONAL`: upstream's book/bottle/head slot, which only the
/// three actions this package does not carry ever fill.
pub const PURIFIER_ADDITIONAL_SLOT: usize = 1;
/// `BlockEntityPurifier.SLOTS`.
pub const PURIFIER_SLOTS: usize = 2;
/// `BlockEntityPurifier.MAX_BUCKETS`: three buckets of blood, the smallest machine tank.
pub const PURIFIER_TANK: i32 = 3 * BUCKET;
/// `ToolBadEnchantPurifyAction.PURIFY_DURATION`: the ticks one curse level takes.
pub const PURIFY_DURATION: u32 = 60;
/// `EnchantmentTags.CURSE` (`#minecraft:curse`): vanilla's two curses, named here because the
/// pin's tag reads cover the block, item and fluid registries only ([`crate::purifier`]).
pub const CURSES: &[&str] = &["minecraft:binding_curse", "minecraft:vanishing_curse"];

/// Whether an enchantment is one this action removes (`EnchantmentTags.CURSE`).
#[must_use]
pub fn is_curse(enchantment: &str) -> bool {
    CURSES.contains(&enchantment)
}

/// One tick of the purifier's curse action: `(tick after this one, whether it completed)`.
///
/// Upstream's `BlockEntityPurifier.Ticker` adds one to the component's tick and then runs
/// the action, and `ToolBadEnchantPurifyAction.canWork` refuses a machine with no cursed
/// stack or with less than one bucket of blood (`getBucketsFloored() > 0`). `None` is that
/// refusal, which resets the clock. `PURIFY_DURATION` is checked to be at least one.
#[must_use]
pub fn purify_tick(tick: u32, curses: usize, tank: i32) -> Option<(u32, bool)> {
    if curses == 0 || tank < BUCKET {
        return None;
    }
    let next = tick.saturating_add(1);
    if next >= PURIFY_DURATION.max(1) {
        Some((0, true))
    } else {
        Some((next, false))
    }
}

/// What a completed purify writes, and how many curses it stripped: every curse loses one
/// level (and is gone at level 0), every other enchantment is untouched.
///
/// `ToolBadEnchantPurifyAction.work` runs `EnchantmentHelpers.runIterationOnItem` over the
/// stack and, on the duration tick, sets each curse's level to `level - 1`. One bucket of
/// blood pays for each curse it touched (`setBuckets(getBucketsFloored() - 1, rest)`), so
/// the returned count is also the buckets the caller must spend.
#[must_use]
pub fn purified(levels: &[(String, u32)]) -> (Vec<(String, u32)>, i32) {
    let mut curses = 0;
    let kept = levels
        .iter()
        .filter_map(|(id, level)| {
            if !is_curse(id) {
                return Some((id.clone(), *level));
            }
            curses += 1;
            (*level > 1).then(|| (id.clone(), level - 1))
        })
        .collect();
    (kept, curses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zombies_twenty_hp_is_one_hundred_to_seven_hundred_ninety_nine_mb() {
        assert_eq!(kill_span(20.0), 700);
        assert_eq!(blood_from_kill(20.0, 0), 100);
        assert_eq!(blood_from_kill(20.0, 699), 799);
    }

    #[test]
    fn a_cows_ten_hp_is_fifty_to_three_hundred_ninety_nine_mb() {
        assert_eq!(blood_from_kill(10.0, 0), 50);
        assert_eq!(blood_from_kill(10.0, 349), 399);
    }

    #[test]
    fn every_roll_of_a_kill_stays_inside_the_multipliers() {
        for roll in 0..2_000 {
            let blood = blood_from_kill(20.0, roll);
            assert!(
                (100..=799).contains(&blood),
                "roll {roll} gave {blood} mB, outside 100..=799"
            );
        }
    }

    #[test]
    fn one_damage_point_costs_five_mb_and_leaves_the_tank_short() {
        assert_eq!(repair(10, 5), Some((9, 0)));
        assert_eq!(repair(1, 1_000), Some((0, 995)));
    }

    #[test]
    fn a_tank_under_five_mb_repairs_nothing() {
        assert_eq!(repair(10, 4), None);
        assert_eq!(repair(10, 0), None);
    }

    #[test]
    fn an_undamaged_item_is_never_repaired() {
        assert_eq!(repair(0, 1_000), None);
        assert_eq!(repair(-1, 1_000), None);
    }

    #[test]
    fn a_repair_is_due_every_second_tick() {
        assert!(repair_due(0));
        assert!(!repair_due(1));
        assert!(repair_due(2));
        assert!(!repair_due(3));
        assert!(repair_due(4));
    }

    #[test]
    fn a_chipped_anvil_becomes_an_anvil_for_one_hundred_twenty_five_mb() {
        assert_eq!(anvil_step(CHIPPED_ANVIL, ANVIL_MB), Some((ANVIL, 0)));
        assert_eq!(anvil_step(CHIPPED_ANVIL, ANVIL_MB - 1), None);
    }

    #[test]
    fn a_damaged_anvil_becomes_a_chipped_one() {
        assert_eq!(anvil_step(DAMAGED_ANVIL, 130), Some((CHIPPED_ANVIL, 5)));
    }

    #[test]
    fn a_sound_anvil_or_an_ordinary_item_has_no_step() {
        assert_eq!(anvil_step(ANVIL, 1_000), None);
        assert_eq!(anvil_step("minecraft:stone", 1_000), None);
    }

    #[test]
    fn a_stick_is_never_taken_for_repair() {
        assert!(!can_hold("minecraft:stick", 3));
    }

    #[test]
    fn a_damaged_tool_and_a_broken_anvil_are_taken() {
        assert!(can_hold("minecraft:diamond_pickaxe", 3));
        assert!(can_hold(CHIPPED_ANVIL, 0));
        assert!(can_hold(DAMAGED_ANVIL, 0));
        assert!(!can_hold("minecraft:diamond_pickaxe", 0));
        assert!(!can_hold("minecraft:stone", 0));
    }

    #[test]
    fn the_tank_holds_ten_buckets_of_blood() {
        assert_eq!(TANK_CAPACITY, 10 * BUCKET);
        assert_eq!(drain_container(9_000, 5_000), (TANK_CAPACITY, 4_000));
    }

    #[test]
    fn a_full_tank_keeps_what_the_container_holds() {
        assert_eq!(drain_container(TANK_CAPACITY, 500), (TANK_CAPACITY, 500));
        assert_eq!(drain_container(TANK_CAPACITY, 0), (TANK_CAPACITY, 0));
    }

    #[test]
    fn an_extractor_takes_the_whole_kill_when_it_fits() {
        assert_eq!(fill_extractor(0, 700), (700, 0));
        assert_eq!(fill_extractor(100, 0), (100, 0));
    }

    #[test]
    fn an_extractor_keeps_the_blood_that_does_not_fit() {
        assert_eq!(fill_extractor(4_800, 700), (EXTRACTOR_CAPACITY, 500));
        assert_eq!(
            fill_extractor(EXTRACTOR_CAPACITY, 700),
            (EXTRACTOR_CAPACITY, 700)
        );
    }

    #[test]
    fn the_slot_layout_matches_upstream() {
        assert_eq!((CHEST_SLOTS, CONTAINER_SLOT, SLOT_COUNT), (10, 10, 11));
    }

    // -----------------------------------------------------------------------
    // Slice 3: the tank on the broken machine's own item
    // -----------------------------------------------------------------------

    #[test]
    fn a_broken_machine_carries_its_tank_and_an_empty_one_carries_nothing() {
        assert_eq!(carried_tank(6_500), Some(6_500));
        assert_eq!(carried_tank(TANK_CAPACITY), Some(TANK_CAPACITY));
        assert_eq!(carried_tank(0), None, "an empty tank travels on no item");
        assert_eq!(carried_tank(-40), None, "a negative amount is not a tank");
    }

    #[test]
    fn a_carried_tank_comes_back_whole() {
        for amount in [1, 3_500, TANK_CAPACITY] {
            let carried = carried_tank(amount).expect("a filled tank travels");
            assert_eq!(
                restored_tank(carried),
                amount,
                "{amount} mB did not survive"
            );
        }
    }

    #[test]
    fn a_doctored_tank_is_clamped_to_the_capacity_when_it_is_placed() {
        assert_eq!(restored_tank(20_000), TANK_CAPACITY);
        assert_eq!(restored_tank(TANK_CAPACITY + 1), TANK_CAPACITY);
        assert_eq!(restored_tank(-1), 0);
        assert_eq!(restored_tank(0), 0);
        assert_eq!(restored_tank(6_500), 6_500);
    }

    // -----------------------------------------------------------------------
    // Slice 4: a projectile kill fills the extractor (`living_drops`)
    // -----------------------------------------------------------------------

    #[test]
    fn a_players_word_or_arrow_kill_fills_the_extractor() {
        // The death is credited to the player who swung or fired.
        assert!(Credit::Player.fills());
    }

    #[test]
    fn a_mobs_arrow_kills_but_fills_nothing() {
        // The same shape with the credit on a skeleton: a death, not the player's.
        assert!(!Credit::Nobody.fills());
    }

    #[test]
    fn the_environment_and_kill_command_fill_nothing() {
        // A fall, lava, the void, `/kill`: the source names no causing entity.
        assert!(!Credit::Nobody.fills());
    }

    // -----------------------------------------------------------------------
    // Slice 5: the Blood Stain, and the extractor's crouching click
    // -----------------------------------------------------------------------

    #[test]
    fn a_crouching_click_moves_only_the_room_the_extractor_has() {
        let draw = stain_draw(3_500, 4_000);
        assert_eq!(draw.moved, 1_000, "the extractor had room for 1,000 mB");
        assert_eq!(draw.stain, 2_500, "the stain keeps the rest");
        assert_eq!(draw.extractor, EXTRACTOR_CAPACITY);
        assert!(!draw.drains_the_stain());
    }

    #[test]
    fn a_stain_with_less_than_the_room_empties_into_the_extractor() {
        let draw = stain_draw(1_000, 0);
        assert_eq!((draw.moved, draw.stain, draw.extractor), (1_000, 0, 1_000));
        assert!(draw.drains_the_stain(), "an emptied stain is removed");
    }

    #[test]
    fn a_full_extractor_takes_nothing_and_leaves_the_stain_alone() {
        let draw = stain_draw(3_500, EXTRACTOR_CAPACITY);
        assert_eq!(draw.moved, 0);
        assert_eq!(draw.stain, 3_500);
        assert_eq!(draw.extractor, EXTRACTOR_CAPACITY);
        assert!(!draw.drains_the_stain());
    }

    #[test]
    fn a_stain_never_holds_more_than_five_buckets() {
        assert_eq!(STAIN_TANK, 5 * BUCKET);
        assert_eq!(STAIN, "evilcraft:blood_stain");
        let draw = stain_draw(9_000, 0);
        assert_eq!(
            (draw.moved, draw.stain, draw.extractor),
            (STAIN_TANK, 0, STAIN_TANK),
            "a doctored stain gives at most its own tank"
        );
    }

    // -----------------------------------------------------------------------
    // Slice 6: a fall death leaves a Blood Stain
    // -----------------------------------------------------------------------

    #[test]
    fn a_twenty_hp_zombies_fall_leaves_four_hundred_mb_in_its_own_cell() {
        // Upstream's `bloodMBPerHP = 20` times the mob's max health, in the cell it died in:
        // a stain rests *on* the floor, so the blood is the mob's own block position.
        assert_eq!(stain_blood(20.0), 400);
        assert_eq!(stain_blood(10.0), 200);
        assert_eq!(
            fall_stain([10, 101, 20], 20.0, None, Cell::Free),
            Some(FallStain {
                cell: [10, 101, 20],
                place: true,
                tank: 400,
            })
        );
    }

    #[test]
    fn a_second_fall_tops_the_stain_it_lands_on_up_instead_of_placing_another() {
        // The mob's own cell already holds the first death's stain (`Cell::Stain`): the
        // second death adds to it in place, and no block is placed.
        assert_eq!(
            fall_stain([10, 101, 20], 20.0, None, Cell::Stain(400)),
            Some(FallStain {
                cell: [10, 101, 20],
                place: false,
                tank: 800,
            })
        );
    }

    #[test]
    fn a_fall_one_cell_above_a_stain_tops_that_stain_up() {
        // Upstream reads the block *below* the mob first; a stain there takes the blood.
        assert_eq!(
            fall_stain([10, 101, 20], 20.0, Some(400), Cell::Free),
            Some(FallStain {
                cell: [10, 100, 20],
                place: false,
                tank: 800,
            })
        );
        // The mob's own cell does not matter when the stain below takes it.
        assert_eq!(
            fall_stain([10, 101, 20], 20.0, Some(400), Cell::Taken).map(|landing| landing.tank),
            Some(800)
        );
    }

    #[test]
    fn a_fall_never_fills_a_stain_past_its_five_buckets() {
        // Eight buckets of blood from one death, and a stain that already holds 4,800 mB.
        assert_eq!(
            stain_blood(400.0),
            STAIN_TANK,
            "an 8,000 mB fall fits 5,000"
        );
        assert_eq!(
            fall_stain([10, 101, 20], 20.0, None, Cell::Stain(4_800)).map(|landing| landing.tank),
            Some(STAIN_TANK)
        );
    }

    #[test]
    fn a_fall_with_no_cell_to_stain_leaves_the_level_alone() {
        // No stain below, and the mob's own cell holds neither air nor a stain: nothing is
        // written (a mob that dies inside a block leaves no stain).
        assert_eq!(fall_stain([10, 101, 20], 20.0, None, Cell::Taken), None);
        // A death with no health to give is not blood either.
        assert_eq!(stain_blood(0.0), 0);
        assert_eq!(stain_blood(-5.0), 0);
        assert_eq!(fall_stain([10, 101, 20], 0.0, None, Cell::Free), None);
    }

    // -----------------------------------------------------------------------
    // Slice 7: the Colossal Blood Chest's structure
    // -----------------------------------------------------------------------

    /// A 3×3×3 cube at `anchor`: 25 reinforced planks, air in the middle, `controller` a chest.
    fn cube(
        anchor: [i32; 3],
        controller: [i32; 3],
    ) -> std::collections::BTreeMap<[i32; 3], &'static str> {
        let mut cells = std::collections::BTreeMap::new();
        for x in 0..COLOSSAL_SIZE {
            for y in 0..COLOSSAL_SIZE {
                for z in 0..COLOSSAL_SIZE {
                    cells.insert(
                        [anchor[0] + x, anchor[1] + y, anchor[2] + z],
                        REINFORCED_PLANKS,
                    );
                }
            }
        }
        cells.insert(
            [anchor[0] + 1, anchor[1] + 1, anchor[2] + 1],
            "minecraft:air",
        );
        cells.insert(controller, COLOSSAL);
        cells
    }

    /// The cube's cells as a reader; anything the map does not hold is stone, as the world
    /// outside a structure is.
    fn reader<'a>(
        cells: &'a std::collections::BTreeMap<[i32; 3], &'static str>,
    ) -> impl Fn([i32; 3]) -> Shell + 'a {
        move |pos| structure_cell(cells.get(&pos).copied().unwrap_or("minecraft:stone"))
    }

    /// One tick of the machine's repair loop over the repair slots, when it is formed:
    /// `(damage, tank)` after `ticks` ticks.
    fn repair_ticks(formed: bool, mut damage: i32, mut tank: i32, ticks: u32) -> (i32, i32) {
        for tick in 0..ticks {
            if formed
                && repair_due(tick)
                && let Some((left, tank_left)) = repair(damage, tank)
            {
                damage = left;
                tank = tank_left;
            }
        }
        (damage, tank)
    }

    #[test]
    fn the_structure_is_a_three_cube_with_one_air_cell() {
        assert_eq!(COLOSSAL_SIZE, 3);
        assert_eq!(COLOSSAL_SHELL, 26);
        assert_eq!(
            COLOSSAL_SHELL as i32,
            COLOSSAL_SIZE * COLOSSAL_SIZE * COLOSSAL_SIZE - 1
        );
        assert_eq!(COLOSSAL, "evilcraft:colossal_blood_chest");
        assert_eq!(REINFORCED_PLANKS, "evilcraft:reinforced_undead_planks");
    }

    #[test]
    fn every_shell_cell_of_the_cube_can_be_the_controller() {
        let anchor = [10, 101, -20];
        for x in 0..COLOSSAL_SIZE {
            for y in 0..COLOSSAL_SIZE {
                for z in 0..COLOSSAL_SIZE {
                    if (x, y, z) == (1, 1, 1) {
                        continue;
                    }
                    let controller = [anchor[0] + x, anchor[1] + y, anchor[2] + z];
                    let cells = cube(anchor, controller);
                    assert_eq!(
                        colossal_formed(controller, reader(&cells)),
                        Some(anchor),
                        "a chest at {controller:?} is a shell cell of the cube"
                    );
                }
            }
        }
    }

    #[test]
    fn a_missing_shell_block_breaks_the_structure() {
        let anchor = [10, 101, -20];
        let controller = [10, 101, -20];
        let mut cells = cube(anchor, controller);
        // One plank of the shell removed: the counterfactual the in-game leg performs.
        for corner in [[10, 103, -20], [11, 101, -18]] {
            let removed = cells.remove(&corner).expect("the shell held the plank");
            assert_eq!(
                colossal_formed(controller, reader(&cells)),
                None,
                "the shell is open at {corner:?}"
            );
            cells.insert(corner, removed);
            assert_eq!(
                colossal_formed(controller, reader(&cells)),
                Some(anchor),
                "putting the plank back at {corner:?} forms the cube again"
            );
        }
    }

    #[test]
    fn a_filled_middle_breaks_the_structure() {
        let anchor = [10, 101, -20];
        let controller = [10, 101, -20];
        let mut cells = cube(anchor, controller);
        cells.insert([11, 102, -19], REINFORCED_PLANKS);
        assert_eq!(
            colossal_formed(controller, reader(&cells)),
            None,
            "the hollow cube's middle is not hollow"
        );
    }

    #[test]
    fn a_second_colossal_chest_breaks_the_structure() {
        // `ExactBlockCountValidator(1)`: exactly one colossal chest in the cube.
        let anchor = [10, 101, -20];
        let controller = [10, 101, -20];
        let mut cells = cube(anchor, controller);
        cells.insert([12, 103, -18], COLOSSAL);
        assert_eq!(colossal_formed(controller, reader(&cells)), None);
    }

    #[test]
    fn a_chest_in_the_middle_is_no_structure() {
        // The middle cell must be air: a controller there is not a shell cell at all.
        let anchor = [10, 101, -20];
        let middle = [11, 102, -19];
        let cells = cube(anchor, middle);
        assert_eq!(colossal_formed(middle, reader(&cells)), None);
    }

    #[test]
    fn a_lone_chest_and_a_wall_of_planks_do_not_form() {
        // A fresh controller in the world: its shell is air, so nothing forms.
        let mut lone = std::collections::BTreeMap::new();
        lone.insert([10, 101, -20], COLOSSAL);
        assert_eq!(colossal_formed([10, 101, -20], reader(&lone)), None);
        // A solid 5×3×3 mass of planks with a chest in it: the candidate cube's middle is
        // planks, not air, so the detector refuses it.
        let mut wall = std::collections::BTreeMap::new();
        for x in 0..5 {
            for y in 0..3 {
                for z in 0..3 {
                    wall.insert([10 + x, 101 + y, -20 + z], REINFORCED_PLANKS);
                }
            }
        }
        wall.insert([12, 101, -20], COLOSSAL);
        assert_eq!(colossal_formed([12, 101, -20], reader(&wall)), None);
    }

    #[test]
    fn a_cube_that_is_not_exactly_three_wide_does_not_form() {
        // A valid cube with one more plank layer beside it: the corner walk stops at the
        // outer shell, whose middle is planks, so the exact-size rule refuses it.
        let anchor = [10, 101, -20];
        let controller = [10, 101, -20];
        let mut cells = cube(anchor, controller);
        for y in 0..3 {
            for z in 0..3 {
                cells.insert([9, 101 + y, -20 + z], REINFORCED_PLANKS);
            }
        }
        assert_eq!(colossal_formed(controller, reader(&cells)), None);
    }

    #[test]
    fn the_machine_has_forty_five_repair_slots_one_container_and_four_upgrades() {
        assert_eq!(COLOSSAL_SLOTS, 45);
        assert_eq!(COLOSSAL_CONTAINER_SLOT, 45);
        assert_eq!(COLOSSAL_SLOT_COUNT, 46);
        assert_eq!(COLOSSAL_INVENTORY, 50);
        assert_eq!(TANK_CAPACITY, 10 * BUCKET);
    }

    #[test]
    fn a_formed_colossal_repairs_one_point_every_two_ticks_for_five_mb() {
        // The brief's own assertion, as the machine's loop: 250 damage, 1,000 mB, 40 ticks.
        assert_eq!(
            repair_ticks(true, 250, 1_000, 40),
            (230, 900),
            "20 points in 40 ticks at 5 mB a point"
        );
        // And it keeps going until the tank is empty: 1,000 mB is 200 points, 400 ticks.
        assert_eq!(repair_ticks(true, 250, 1_000, 400), (50, 0));
    }

    #[test]
    fn an_unformed_colossal_repairs_nothing() {
        // The same tank, the same pickaxe, the same 40 ticks: a shell that is not whole
        // leaves the item and the blood alone (`BulkRepairItemTickAction.canTick` asks
        // `canWork()` first).
        assert_eq!(repair_ticks(false, 250, 1_000, 40), (250, 1_000));
        // And a formed one with less than one point's blood stops exactly where a point
        // no longer fits, which is the plain chest's own `repair` rule.
        assert_eq!(repair_ticks(true, 250, 4, 40), (250, 4));
    }

    // -----------------------------------------------------------------------
    // Slice 2: the Blood Infuser
    // -----------------------------------------------------------------------

    /// The recipes this port carries, decoded from the files it ships.
    fn carried() -> Vec<Infusion> {
        crate::infuser::CARRIED
            .iter()
            .map(|(name, json)| {
                crate::infuser::decode_infusion(name, json).expect("a carried recipe decodes")
            })
            .collect()
    }

    #[test]
    fn a_bone_becomes_a_blaze_rod_for_two_and_a_half_thousand_mb() {
        let recipes = carried();
        let bone = infusion_for(&recipes, "minecraft:bone", 3).expect("a bone is infused");
        assert_eq!(bone.output, "minecraft:blaze_rod");
        assert_eq!(bone.output_count, 1);
        assert_eq!(bone.blood, 2_500);
        assert_eq!(bone.ticks, 500);
    }

    #[test]
    fn a_rotten_flesh_makes_eight_spider_eyes() {
        let recipes = carried();
        let eyes = infusion_for(&recipes, "minecraft:rotten_flesh", 3).expect("rotten flesh");
        // Two jar files infuse rotten flesh at the same 375 mB: this one, tier 2, makes
        // eight spider eyes, and `minecraft_leather.json`, tier 1, would make leather.
        // Upstream's `getRecipe` takes the highest tier, so it can never pick the
        // leather one (an upstream defect, recorded in `notes.md`); the port does not
        // carry dead data, and this asserts the winner is the eye recipe.
        assert_eq!(
            (eyes.output.as_str(), eyes.output_count, eyes.blood),
            ("minecraft:spider_eye", 8, 375)
        );
        assert!(
            recipes
                .iter()
                .all(|recipe| recipe.output != "minecraft:leather"),
            "the jar's shadowed leather recipe is carried"
        );
    }

    #[test]
    fn an_unknown_item_has_no_infusion() {
        let recipes = carried();
        assert!(infusion_for(&recipes, "minecraft:diamond", 3).is_none());
    }

    #[test]
    fn a_tier_three_recipe_needs_a_tier_three_machine() {
        let recipes = carried();
        // Upstream's convenience recipes declare the tier of the machine they need.
        assert!(infusion_for(&recipes, "minecraft:bone", 2).is_none());
        assert!(infusion_for(&recipes, "minecraft:bone", 3).is_some());
        assert!(infusion_for(&recipes, "minecraft:dirt", 0).is_none());
        assert!(infusion_for(&recipes, "minecraft:dirt", 1).is_some());
    }

    #[test]
    fn the_infusion_completes_on_its_duration_tick() {
        let bone = infusion_for(&carried(), "minecraft:bone", 3)
            .cloned()
            .expect("a bone is infused");
        assert_eq!(infuse_tick(0, &bone, 10_000), Some((1, false)));
        assert_eq!(infuse_tick(498, &bone, 10_000), Some((499, false)));
        assert_eq!(infuse_tick(499, &bone, 10_000), Some((0, true)));
    }

    #[test]
    fn a_tank_that_cannot_pay_its_blood_stalls_the_infusion() {
        let bone = infusion_for(&carried(), "minecraft:bone", 3)
            .cloned()
            .expect("a bone is infused");
        assert_eq!(infuse_tick(10, &bone, 2_499), None);
        assert_eq!(infuse_tick(10, &bone, 2_500), Some((11, false)));
    }

    #[test]
    fn every_carried_recipe_fits_its_tank_and_names_a_tier() {
        let recipes = carried();
        assert_eq!(recipes.len(), crate::infuser::CARRIED.len());
        for recipe in &recipes {
            assert!(
                recipe.blood > 0 && recipe.blood <= INFUSER_TANK,
                "{} costs {} mB of a {INFUSER_TANK} mB tank",
                recipe.id,
                recipe.blood
            );
            assert!(recipe.ticks > 0, "{} takes no time", recipe.id);
            assert!(
                (1..=3).contains(&recipe.tier),
                "{} names tier {}",
                recipe.id,
                recipe.tier
            );
            assert_ne!(
                recipe.input, recipe.output,
                "{} infuses into itself",
                recipe.id
            );
        }
    }

    #[test]
    fn a_tier_promise_is_the_machines_tier() {
        assert_eq!(machine_tier([]), 0);
        assert_eq!(machine_tier(["minecraft:stone"]), 0);
        assert_eq!(machine_tier(["evilcraft:promise_tier_1"]), 1);
        assert_eq!(machine_tier(["evilcraft:promise_tier_3"]), 3);
        assert_eq!(
            machine_tier(["evilcraft:promise_tier_2", "evilcraft:promise_tier_3"]),
            3
        );
        assert_eq!((INFUSER_SLOTS, UPGRADE_SLOTS), (7, 4));
    }

    // -----------------------------------------------------------------------
    // Slice 8: the Dark Tank
    // -----------------------------------------------------------------------

    #[test]
    fn a_dark_tank_holds_sixteen_buckets() {
        // `BlockEntityDarkTank.BASE_CAPACITY = 16000`, twice the machines' ten buckets.
        assert_eq!(DARK_TANK_CAPACITY, 16_000);
        assert_eq!(DARK_TANK, "evilcraft:dark_tank");
        assert_eq!(DARK_TANK_CAPACITY, 2 * TANK_CAPACITY - 4_000);
    }

    #[test]
    fn a_filled_extractor_empties_into_a_dark_tank() {
        let fill = tank_fill(0, EXTRACTOR_CAPACITY);
        assert_eq!(
            (fill.moved, fill.tank, fill.extractor),
            (EXTRACTOR_CAPACITY, EXTRACTOR_CAPACITY, 0),
            "a 5,000 mB extractor fits a fresh 16,000 mB tank whole"
        );
    }

    #[test]
    fn a_dark_tank_takes_only_its_room_and_the_extractor_keeps_the_rest() {
        let fill = tank_fill(14_000, 5_000);
        assert_eq!(fill.moved, 2_000, "the tank had room for 2,000 mB");
        assert_eq!(fill.tank, DARK_TANK_CAPACITY);
        assert_eq!(
            fill.extractor, 3_000,
            "the extractor keeps what did not fit"
        );
    }

    #[test]
    fn a_full_dark_tank_or_an_empty_extractor_moves_nothing() {
        let full = tank_fill(DARK_TANK_CAPACITY, EXTRACTOR_CAPACITY);
        assert_eq!(
            (full.moved, full.tank, full.extractor),
            (0, DARK_TANK_CAPACITY, EXTRACTOR_CAPACITY)
        );
        let empty = tank_fill(1_000, 0);
        assert_eq!((empty.moved, empty.tank, empty.extractor), (0, 1_000, 0));
    }

    #[test]
    fn a_doctored_dark_tank_is_read_at_its_own_capacity() {
        // A saved amount above its capacity (a `/give`d payload) is the capacity, and a
        // click can add nothing to a tank that is already full.
        let fill = tank_fill(20_000, 5_000);
        assert_eq!((fill.moved, fill.tank), (0, DARK_TANK_CAPACITY));
    }

    #[test]
    fn a_dark_tank_payload_carries_sixteen_buckets_and_comes_back_clamped() {
        assert_eq!(carried_tank_with(12_000, DARK_TANK_CAPACITY), Some(12_000));
        assert_eq!(
            carried_tank_with(20_000, DARK_TANK_CAPACITY),
            Some(DARK_TANK_CAPACITY)
        );
        assert_eq!(carried_tank_with(0, DARK_TANK_CAPACITY), None);
        assert_eq!(
            restored_tank_with(20_000, DARK_TANK_CAPACITY),
            DARK_TANK_CAPACITY
        );
        assert_eq!(restored_tank_with(12_000, DARK_TANK_CAPACITY), 12_000);
        // The ten-bucket machines keep their own clamp.
        assert_eq!(carried_tank(20_000), Some(TANK_CAPACITY));
    }

    // -----------------------------------------------------------------------
    // Slice 9: the Purifier's curse action, and the Blook
    // -----------------------------------------------------------------------

    /// One stack's enchantments, as the component states them: `(id, level)` in file order.
    fn levels(entries: &[(&str, u32)]) -> Vec<(String, u32)> {
        entries
            .iter()
            .map(|(id, level)| ((*id).to_owned(), *level))
            .collect()
    }

    #[test]
    fn a_book_becomes_a_blook_for_five_hundred_mb() {
        // `data/evilcraft/recipe/blood_infuser/base/blook.json`, the jar's own file: an
        // `evilcraft:blood_infuser` recipe of tier 1 that costs 500 mB over 500 ticks.
        let recipes = carried();
        let blook = infusion_for(&recipes, "minecraft:book", 1).expect("a book is infused");
        assert_eq!(blook.output, BLOOK);
        assert_eq!(blook.output_count, 1);
        assert_eq!((blook.blood, blook.ticks, blook.tier), (500, 500, 1));
        // And a tier-1 machine is the least that runs it, as the file's tier says.
        assert_eq!(infusion_for(&recipes, "minecraft:book", 0), None);
    }

    #[test]
    fn every_carried_recipe_names_items_this_package_or_the_game_has() {
        // The infuser's files are data: a recipe naming an id nobody registers is dead data
        // (the jar's own 40 include exactly that class of defect). Every input and product
        // is vanilla's or one of this package's own items.
        let recipes = carried();
        assert!(!recipes.is_empty());
        for recipe in &recipes {
            for id in [&recipe.input, &recipe.output] {
                assert!(
                    id.starts_with("minecraft:") || CARRIED_ITEMS.contains(&id.as_str()),
                    "{} names {id}, which neither the game nor this package registers",
                    recipe.id
                );
            }
        }
        // The blook is one of the ids the test above relies on.
        assert!(CARRIED_ITEMS.contains(&BLOOK));
    }

    #[test]
    fn a_curse_takes_sixty_ticks_and_one_bucket_of_blood() {
        // `ToolBadEnchantPurifyAction.PURIFY_DURATION` = 60; the ticker adds one before the
        // action runs, so the sixtieth tick is the one that finishes it.
        assert_eq!(purify_tick(0, 1, PURIFIER_TANK), Some((1, false)));
        assert_eq!(purify_tick(58, 1, PURIFIER_TANK), Some((59, false)));
        assert_eq!(purify_tick(59, 1, PURIFIER_TANK), Some((0, true)));
        assert_eq!(PURIFY_DURATION, 60);
    }

    #[test]
    fn a_clean_stack_or_a_tank_under_one_bucket_never_works() {
        // `canWork`: `!getPurifyItem().isEmpty() && getBucketsFloored() > 0` and a curse.
        assert_eq!(
            purify_tick(10, 0, PURIFIER_TANK),
            None,
            "no curse to remove"
        );
        assert_eq!(
            purify_tick(10, 1, BUCKET - 1),
            None,
            "a tank under one bucket cannot pay"
        );
        assert_eq!(purify_tick(10, 1, BUCKET), Some((11, false)));
    }

    #[test]
    fn a_finished_purify_strips_one_level_of_every_curse_and_keeps_the_rest() {
        // A level-1 curse is gone; every other enchantment is untouched, and one bucket a
        // curse is spent.
        let (kept, curses) = purified(&levels(&[
            ("minecraft:vanishing_curse", 1),
            ("minecraft:sharpness", 3),
        ]));
        assert_eq!(kept, levels(&[("minecraft:sharpness", 3)]));
        assert_eq!(curses, 1);
        // A level-2 curse loses one level and stays, as `setEnchantmentLevel(level - 1)` says.
        let (kept, curses) = purified(&levels(&[("minecraft:binding_curse", 2)]));
        assert_eq!(kept, levels(&[("minecraft:binding_curse", 1)]));
        assert_eq!(curses, 1);
        // Two curses cost two buckets in the one tick, as upstream's iteration does.
        let (kept, curses) = purified(&levels(&[
            ("minecraft:binding_curse", 1),
            ("minecraft:vanishing_curse", 1),
            ("minecraft:mending", 1),
        ]));
        assert_eq!(kept, levels(&[("minecraft:mending", 1)]));
        assert_eq!(curses, 2);
        // And a stack with no curse is written back unchanged.
        let plain = levels(&[("minecraft:sharpness", 2)]);
        assert_eq!(purified(&plain), (plain.clone(), 0));
    }

    #[test]
    fn only_vanillas_two_curses_are_curses() {
        assert!(is_curse("minecraft:binding_curse"));
        assert!(is_curse("minecraft:vanishing_curse"));
        assert!(!is_curse("minecraft:sharpness"));
        assert!(!is_curse("minecraft:fortune"));
        assert_eq!(CURSES.len(), 2);
    }

    #[test]
    fn the_purifier_has_two_slots_and_three_buckets() {
        // `BlockEntityPurifier.SLOTS` and `.MAX_BUCKETS`.
        assert_eq!((PURIFIER_SLOTS, PURIFIER_ADDITIONAL_SLOT), (2, 1));
        assert_eq!(PURIFIER_PURIFY_SLOT, 0);
        assert_eq!(PURIFIER_TANK, 3 * BUCKET);
        assert_eq!(PURIFIER, "evilcraft:purifier");
    }

    #[test]
    fn a_stack_of_broken_anvils_is_repaired_whole_for_one_hundred_twenty_five_mb_each() {
        // Review finding 1: `AnvilRepairAction` answers a fresh stack of **one** anvil, so the
        // port must not reproduce it — a stack of N becomes N of the next state, at 125 mB each.
        assert_eq!(
            anvil_stack_step(CHIPPED_ANVIL, 2, 2 * ANVIL_MB),
            Some((ANVIL, 0)),
            "two chipped anvils become two anvils for 250 mB"
        );
        assert_eq!(
            anvil_stack_step(DAMAGED_ANVIL, 3, 3 * ANVIL_MB + 5),
            Some((CHIPPED_ANVIL, 5)),
            "three damaged anvils become three chipped ones for 375 mB"
        );
        // A tank that cannot pay for **all** of them leaves the stack alone: no partial
        // replacement and no lost anvil.
        assert_eq!(
            anvil_stack_step(CHIPPED_ANVIL, 2, 2 * ANVIL_MB - 1),
            None,
            "249 mB cannot convert two anvils"
        );
        // One item is the upstream price, unchanged, and a whole or ordinary item has no step.
        assert_eq!(
            anvil_stack_step(CHIPPED_ANVIL, 1, ANVIL_MB),
            Some((ANVIL, 0))
        );
        assert_eq!(anvil_stack_step(ANVIL, 2, 1_000), None);
        assert_eq!(anvil_stack_step("minecraft:stone", 2, 1_000), None);
        // A zero count is no stack at all.
        assert_eq!(anvil_stack_step(CHIPPED_ANVIL, 0, 1_000), None);
    }

    #[test]
    fn a_legacy_stack_of_extractors_resolves_to_the_first_holding_all_the_blood() {
        // Review finding 2: the amount is stored once per stack, so resolving a stack of N
        // must give the whole amount to the first extractor and nothing to the rest.
        let amounts = legacy_extractors(2, 4_000).expect("a stack of two resolves");
        assert_eq!(
            amounts,
            vec![4_000, 0],
            "the first takes the stack's 4,000 mB"
        );
        assert_eq!(
            amounts.iter().sum::<i32>(),
            4_000,
            "the resolved stack holds exactly the blood it stored"
        );
        assert_eq!(
            legacy_extractors(3, 0),
            Some(vec![0, 0, 0]),
            "an empty stack of three resolves to three empty extractors"
        );
        // A stored amount above the item's own tank (a `/give`d payload) clamps on the item
        // that keeps it, exactly as `extractor_blood` reads it.
        assert_eq!(
            legacy_extractors(2, EXTRACTOR_CAPACITY + 500),
            Some(vec![EXTRACTOR_CAPACITY, 0])
        );
        // One extractor is already a container and resolves to nothing.
        assert_eq!(legacy_extractors(1, 4_000), None);
        assert_eq!(legacy_extractors(0, 4_000), None);
        // And no drain may treat a stack of more than one as a container.
        assert!(one_extractor(1));
        assert!(!one_extractor(2));
        assert!(!one_extractor(0));
    }
}
