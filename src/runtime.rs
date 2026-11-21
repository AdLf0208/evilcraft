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
    if tank < ANVIL_MB {
        return None;
    }
    anvil_target(item).map(|next| (next, tank - ANVIL_MB))
}

/// What a broken machine's own item carries (`LootFunctionCopyTankData`): the tank's
/// amount, or nothing when the tank is empty.
///
/// Upstream inserts the fluid into the item only when it is not empty
/// (`if (!resource.isEmpty())`), so a broken empty machine drops a plain item that needs
/// no restoring on placement. A negative amount is not a tank either.
#[must_use]
pub fn carried_tank(tank: i32) -> Option<i32> {
    (tank > 0).then(|| tank.min(TANK_CAPACITY))
}

/// The tank a placed item's payload restores, clamped to the machine's capacity
/// (upstream sets the item's tank capacity first, so the insert cannot pass it; a
/// `/give`d item naming 20,000 mB places a full 10,000 mB machine).
#[must_use]
pub fn restored_tank(carried: i32) -> i32 {
    carried.clamp(0, TANK_CAPACITY)
}

/// Empty a blood container into the chest's tank, never past its capacity.
/// Returns `(tank, what the container keeps)`.
#[must_use]
pub fn drain_container(tank: i32, container: i32) -> (i32, i32) {
    let room = (TANK_CAPACITY - tank).max(0);
    let moved = container.clamp(0, room);
    (tank + moved, container - moved)
}

/// Fill one Blood Extractor from the kill's pool, never past its capacity.
/// Returns `(what the extractor holds, what the pool keeps)`.
#[must_use]
pub fn fill_extractor(held: i32, pool: i32) -> (i32, i32) {
    let room = (EXTRACTOR_CAPACITY - held).max(0);
    let moved = pool.clamp(0, room);
    (held + moved, pool - moved)
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
}
