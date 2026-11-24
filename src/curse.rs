//! The harness's forced curse roll for the Blood Chest's repair tick.
//!
//! Upstream draws `nextInt(randomBadEnchantsChance)` from the level's random on
//! every damage point it repairs, so a 1-in-10,000 curse is not reproducible in a
//! scenario. `/evilcraft curse <pos> <roll> <index>` arms a **sticky** draw for one
//! chest position: while armed, that chest's non-bulk repairs use these two numbers
//! instead of the level's random. `roll = -1` disarms it. A world with no armed draw
//! behaves exactly as upstream (the real level stream), so this is test scaffolding,
//! not a change to the port's rules.
//!
//! The state is keyed by position because several machines from earlier phases can
//! be ticking in the same arena; a global one-shot would be consumed by the first of
//! them to repair anything.

use std::sync::{Mutex, OnceLock};

use crate::runtime::{CURSES, curse_roll_domain};

/// One armed draw: the `nextInt(chance)` result and the `nextInt(curses.size())`
/// index into the `#minecraft:curse` tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForcedRoll {
    pub(crate) roll: u32,
    pub(crate) index: u32,
}

#[derive(Default)]
struct Armed {
    by_position: std::collections::HashMap<[i32; 3], ForcedRoll>,
}

static ARMED: OnceLock<Mutex<Armed>> = OnceLock::new();

fn armed() -> &'static Mutex<Armed> {
    ARMED.get_or_init(|| Mutex::new(Armed::default()))
}

/// Arm (or, with `clear = true`, disarm) the draw for `pos`. A roll outside
/// `nextInt`'s domain and an index outside the curse tag are refused.
pub(crate) fn set_forced(pos: [i32; 3], roll: u32, index: u32, clear: bool) -> bool {
    if !clear && (!curse_roll_domain(roll) || index as usize >= CURSES.len()) {
        return false;
    }
    let mut armed = armed().lock().expect("the evilcraft curse lock is held for one write");
    if clear {
        armed.by_position.remove(&pos);
    } else {
        armed.by_position.insert(pos, ForcedRoll { roll, index });
    }
    true
}

/// The draw armed for `pos`, if any. Repair ticks read it without consuming it.
#[must_use]
pub(crate) fn forced_for(pos: [i32; 3]) -> Option<ForcedRoll> {
    armed()
        .lock()
        .expect("the evilcraft curse lock is held for one read")
        .by_position
        .get(&pos)
        .copied()
}

/// A world restart clears the harness's draws with the rest of its state.
pub(crate) fn clear_all() {
    armed()
        .lock()
        .expect("the evilcraft curse lock is held for one clear")
        .by_position
        .clear();
}
