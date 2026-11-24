//! The analogue (comparator) output of EvilCraft's tank blocks — slice 12.
//!
//! Upstream's `BlockPurifier`, `BlockDarkTank` and `BlockWithEntityGuiTank` declare an analogue
//! output and answer `ceil(15 x amount / capacity)` from the block entity. The SDK's comparator
//! hooks get only a read-only [`mod_api::LevelHost`], which cannot reach a block entity, so this
//! module publishes each tank's strength from the machine's own tick into a
//! `(dimension, position)` cache. The block Parts answer from that cache, and a changed strength
//! tells the comparators beside the block to read it again with
//! [`mod_api::LevelRef::update_neighbors`] / [`mod_api::NeighbourUpdate::OutputSignal`] — vanilla's
//! `setChanged` → `updateNeighbourForOutputSignal`, which a tank changing without a block state
//! change does not get on its own. This is the route powah 0.2.14 proved at this pin.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Mutex, OnceLock};

use mod_api::{LevelDimension, LevelKey, LevelRef, NeighbourUpdate, ServerHost};

use crate::blood::read_tank_with;
use crate::infuser::INFUSER_ID;
use crate::runtime::{
    CHEST, COLOSSAL, DARK_TANK, DARK_TANK_CAPACITY, PURIFIER, PURIFIER_TANK, TANK_CAPACITY,
    comparator_strength,
};

/// The reading each tank block last had, per level and position. The key is the level's dimension
/// so two dimensions' machines at one position never share a number.
#[derive(Default)]
struct Outputs {
    strengths: HashMap<(LevelKey, [i32; 3]), i32>,
}

impl Outputs {
    /// Record `strength` and answer whether the stored value changed (a first value counts).
    fn record(&mut self, dimension: LevelKey, pos: [i32; 3], strength: i32) -> bool {
        self.strengths.insert((dimension, pos), strength) != Some(strength)
    }

    fn get(&self, dimension: LevelKey, pos: [i32; 3]) -> Option<i32> {
        self.strengths.get(&(dimension, pos)).copied()
    }

    fn clear(&mut self) {
        self.strengths.clear();
    }
}

static OUTPUTS: OnceLock<Mutex<Outputs>> = OnceLock::new();

fn outputs() -> &'static Mutex<Outputs> {
    OUTPUTS.get_or_init(|| Mutex::new(Outputs::default()))
}

/// The strength a comparator beside the machine at `pos` gets: the last value the machine's tick
/// published, or 0 where no machine published one (a comparator beside anything else keeps
/// vanilla's own answer).
pub(crate) fn cached_reading(dimension: LevelDimension, pos: [i32; 3]) -> i32 {
    outputs()
        .lock()
        .expect("the evilcraft output lock is held for one read")
        .get(dimension.key(), pos)
        .unwrap_or(0)
}

/// The positions whose tank ticked this tick, per level. A block entity is out of the world
/// during its own tick, so the strength is read at `level_tick_post`, where every machine is back
/// in the world and `host.block_entity` reaches its saved tank.
#[derive(Default)]
struct Queue {
    pending: HashMap<LevelRef, BTreeSet<[i32; 3]>>,
}

impl Queue {
    fn push(&mut self, level: LevelRef, pos: [i32; 3]) {
        self.pending.entry(level).or_default().insert(pos);
    }

    fn take(&mut self, level: LevelRef) -> BTreeSet<[i32; 3]> {
        self.pending.remove(&level).unwrap_or_default()
    }

    fn clear(&mut self) {
        self.pending.clear();
    }
}

static QUEUE: OnceLock<Mutex<Queue>> = OnceLock::new();

fn queue() -> &'static Mutex<Queue> {
    QUEUE.get_or_init(|| Mutex::new(Queue::default()))
}

/// The level the server ticks now, the one a block-entity tick runs in.
fn level_now(host: &dyn ServerHost) -> Option<LevelRef> {
    host.level(host.dimension())
}

/// A machine's tick leaves its position here; the level tick's drain reads the tank.
pub(crate) fn queue_tick(host: &dyn ServerHost, pos: [i32; 3]) {
    if let Some(level) = level_now(host) {
        queue()
            .lock()
            .expect("the evilcraft queue lock is held for one insert")
            .push(level, pos);
    }
}

/// Every position queued for `level` this tick, in position order (empty afterwards).
pub(crate) fn take_queued(level: LevelRef) -> BTreeSet<[i32; 3]> {
    queue()
        .lock()
        .expect("the evilcraft queue lock is held for one drain")
        .take(level)
}

/// No reading or queued position outlives its world.
pub(crate) fn clear_worlds() {
    outputs()
        .lock()
        .expect("the evilcraft output lock is held for one clear")
        .clear();
    queue()
        .lock()
        .expect("the evilcraft queue lock is held for one clear")
        .clear();
}

/// The tank capacity a comparator-reading block holds, `None` where the block has no analogue
/// output upstream (the blood stain, a vanilla block).
fn capacity_of(id: &str) -> Option<i32> {
    match id {
        CHEST | INFUSER_ID | COLOSSAL => Some(TANK_CAPACITY),
        PURIFIER => Some(PURIFIER_TANK),
        DARK_TANK => Some(DARK_TANK_CAPACITY),
        _ => None,
    }
}

/// The reading the block at `pos` holds now: the fraction of its tank, `None` if it is not one of
/// upstream's comparator blocks or its block entity is gone.
fn strength_at(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) -> Option<i32> {
    let id = level.block(&*host, pos)?.id(&*host);
    let capacity = capacity_of(&id)?;
    let entity = host.block_entity(level, pos)?;
    let saved = entity.data(&*host)?;
    Some(comparator_strength(
        read_tank_with(&saved, capacity),
        capacity,
    ))
}

/// `Level.updateNeighbourForOutputSignal`: the comparators beside `pos` read the answer again.
fn notify_comparators(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) {
    let Some(state) = level.block_state(&*host, pos) else {
        return;
    };
    // A level that refuses the update leaves the reading in place for the next change.
    let _ = level.update_neighbors(&mut *host, pos, state, NeighbourUpdate::OutputSignal);
}

fn publish(host: &mut dyn ServerHost, level: LevelRef, changed: Vec<[i32; 3]>) {
    for pos in changed {
        notify_comparators(host, level, pos);
    }
}

fn dimension_key(host: &dyn ServerHost, level: LevelRef) -> Option<LevelKey> {
    let id = level.dimension().id(host)?;
    LevelKey::parse(&id).ok()
}

/// Record and notify every machine that changed its tank this tick.
pub(crate) fn on_level_tick_post(
    host: &mut dyn ServerHost,
    level: LevelRef,
    queued: BTreeSet<[i32; 3]>,
) {
    if !level.in_place(host) {
        return;
    }
    let Some(dimension) = dimension_key(host, level) else {
        return;
    };
    let mut changed = Vec::new();
    for pos in queued {
        let Some(strength) = strength_at(host, level, pos) else {
            continue;
        };
        if outputs()
            .lock()
            .expect("the evilcraft output lock is held for one record")
            .record(dimension, pos, strength)
        {
            changed.push(pos);
        }
    }
    publish(host, level, changed);
}

/// Publish a tank the harness wrote directly (`/evilcraft blood`) or a crouching pour changed
/// outside a block-entity tick (the dark tank and the purifier have no tank-writing tick of their
/// own for those paths).
pub(crate) fn publish_tank(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    tank: i32,
    capacity: i32,
) {
    let Some(dimension) = dimension_key(host, level) else {
        return;
    };
    if outputs()
        .lock()
        .expect("the evilcraft output lock is held for one record")
        .record(dimension, pos, comparator_strength(tank, capacity))
    {
        publish(host, level, vec![pos]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_first_reading_counts_as_a_change_and_an_equal_one_does_not() {
        let mut out = Outputs::default();
        assert!(out.record(LevelKey::OVERWORLD, [1, 2, 3], 7));
        assert!(!out.record(LevelKey::OVERWORLD, [1, 2, 3], 7));
        assert!(out.record(LevelKey::OVERWORLD, [1, 2, 3], 8));
        assert_eq!(out.get(LevelKey::OVERWORLD, [1, 2, 3]), Some(8));
        assert_eq!(out.get(LevelKey::OVERWORLD, [1, 2, 4]), None);
        assert_eq!(out.get(LevelKey::NETHER, [1, 2, 3]), None);
        out.clear();
        assert_eq!(out.get(LevelKey::OVERWORLD, [1, 2, 3]), None);
    }

    #[test]
    fn only_the_tank_blocks_answer_a_comparator() {
        assert_eq!(capacity_of(CHEST), Some(TANK_CAPACITY));
        assert_eq!(capacity_of(INFUSER_ID), Some(TANK_CAPACITY));
        assert_eq!(capacity_of(COLOSSAL), Some(TANK_CAPACITY));
        assert_eq!(capacity_of(PURIFIER), Some(PURIFIER_TANK));
        assert_eq!(capacity_of(DARK_TANK), Some(DARK_TANK_CAPACITY));
        assert_eq!(capacity_of("evilcraft:blood_stain"), None);
        assert_eq!(capacity_of("minecraft:stone"), None);
    }
}
