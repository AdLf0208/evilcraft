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
}
