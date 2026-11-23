//! The tank on a broken machine's own item (`LootFunctionCopyTankData`).
//!
//! Both machines' loot tables run `evilcraft:copy_tank_data` after `minecraft:copy_name`:
//! it copies the block entity's tank — and only the tank — into the dropped item's fluid
//! handler, so a broken Blood Chest or Blood Infuser keeps its blood. Horizon has neither
//! that loot function nor an item fluid that a mod tank can fill, so the port carries the
//! blood the way the engine already carries a vanilla block entity's data: the item's
//! `minecraft:block_entity_data` component, tagged with the block entity type's id.
//!
//! **Only the tank.** The engine spills a mod container's stacks as their own item
//! entities when the block is removed (`crates/horizon-client/src/game/spill.rs`,
//! `PreRemove::Contents`), so a payload that also carried `Items` would put every stack
//! in the world twice — once on the ground, once inside the block when it is placed back.
//!
//! [`crate::runtime::carried_tank`] says when a tank travels and how it is clamped;
//! [`MachineItem::on_block_placed_by`] puts it back, clamped to the machine's capacity.

use mod_api::{
    self, BlockDropsEvent, Component, Compound, Flow, ItemKey, ItemPart, ItemStack, LevelHost,
    OnBlockPlacedByEvent, Registrar, ServerHost, Tag,
};

use crate::runtime::{CHEST, COLOSSAL, INFUSER, TANK_CAPACITY, carried_tank, restored_tank};

/// The vanilla component that moves a block entity's saved data on its item
/// (`BlockEntity.applyComponentsFromItemStack`, `TypedEntityData.loadInto`).
const BLOCK_ENTITY_DATA: &str = "minecraft:block_entity_data";

// ---------------------------------------------------------------------------
// The two machines
// ---------------------------------------------------------------------------

/// One of the three machines whose own item carries its tank: `evilcraft:blood_chest`,
/// `evilcraft:blood_infuser` and `evilcraft:colossal_blood_chest`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Machine {
    /// The Blood Chest.
    Chest,
    /// The Blood Infuser.
    Infuser,
    /// The Colossal Blood Chest.
    Colossal,
}

impl Machine {
    /// The machines, in registration order.
    pub const ALL: [Self; 3] = [Self::Chest, Self::Infuser, Self::Colossal];

    /// The block. Its id is also the block entity type's registry id, which is the `id`
    /// `minecraft:block_entity_data` tags a payload with, and the id of the item it drops.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Chest => CHEST,
            Self::Infuser => INFUSER,
            Self::Colossal => COLOSSAL,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Chest => 0,
            Self::Infuser => 1,
            Self::Colossal => 2,
        }
    }

    /// The machine a block id names, if it is one of the three.
    #[must_use]
    fn of_block(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|machine| machine.id() == id)
    }

    /// Declare the item's Part and keep its key: `block_drops` runs with a read-only level
    /// host, which cannot name an item, and a drop is only given a tank when it is this
    /// machine's own.
    pub fn register(self, r: &mut dyn Registrar) {
        let item = r.item_key(self.id());
        let _ = ITEM[self.index()].set(item);
        r.item_part(item, MachineItem);
    }

    /// The item key, once [`Machine::register`] ran.
    fn item(self) -> Option<ItemKey> {
        ITEM[self.index()].get().copied()
    }
}

/// The machines' items, resolved once at registration.
static ITEM: [std::sync::OnceLock<ItemKey>; 3] = [
    std::sync::OnceLock::new(),
    std::sync::OnceLock::new(),
    std::sync::OnceLock::new(),
];

// ---------------------------------------------------------------------------
// The payload
// ---------------------------------------------------------------------------

/// `minecraft:block_entity_data`, as a component the item's patch is read from.
struct BlockEntityData;

impl Component for BlockEntityData {
    type Value = Compound;

    fn id(&self) -> &str {
        BLOCK_ENTITY_DATA
    }

    fn decode(&self, value: &Tag) -> Option<Compound> {
        match value {
            Tag::Compound(data) => Some(data.clone()),
            _ => None,
        }
    }

    fn encode(&self, value: Compound) -> Tag {
        Tag::Compound(value)
    }
}

/// The component patch a broken machine's item carries: the tank, tagged with the block
/// entity type's id (the engine loads a payload only when its `id` names the block entity
/// it is placed as). `None` when the tank is empty, which is upstream's
/// `if (!resource.isEmpty())`.
#[must_use]
pub fn carried_patch(tank: i32, type_id: &str) -> Option<Compound> {
    let amount = carried_tank(tank)?;
    let mut payload = Compound::new();
    payload.insert("id".into(), Tag::String(type_id.into()));
    crate::blood::write_tank(&mut payload, amount);
    let mut patch = Compound::new();
    patch.insert(BLOCK_ENTITY_DATA.into(), Tag::Compound(payload));
    Some(patch)
}

/// The blood a stack's payload asks a placed machine to hold, exactly as the payload
/// states it (a `/give`d item may name any amount); `None` when the stack carries no
/// blood payload at all.
#[must_use]
fn asked_tank(stack: &ItemStack) -> Option<i32> {
    let payload = stack.get(&BlockEntityData)?;
    crate::blood::read_tank_raw(&payload)
}

// ---------------------------------------------------------------------------
// The hooks
// ---------------------------------------------------------------------------

/// `block_drops`: a broken machine's own item carries its blood tank, so placing the item
/// puts the blood back (upstream's `copy_tank_data` on both machines' loot tables).
///
/// The drop's stacks are already built, and the block entity is out of the world: the
/// engine holds its saved data for the event (`level.block_entity_data`). An explosion's
/// blocks are gone before the event, so only a break that still holds its block entity (a
/// player's, a mob's) can carry the tank — upstream's loot function needs the same
/// `BLOCK_ENTITY` parameter and copies nothing without it.
pub fn on_drops(level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
    let Some(block) = level.state_block(event.state) else {
        return Flow::Continue;
    };
    let Some(machine) = Machine::of_block(block) else {
        return Flow::Continue;
    };
    let Some(entity) = event.block_entity else {
        return Flow::Continue;
    };
    let Some(saved) = level.block_entity_data(entity) else {
        return Flow::Continue;
    };
    let Some(patch) = carried_patch(crate::blood::read_tank(&saved), machine.id()) else {
        return Flow::Continue;
    };
    let Some(item) = machine.item() else {
        return Flow::Continue;
    };
    for drop in event.drops.iter_mut() {
        if drop.stack.item() == item {
            let _ = drop.stack.apply_components(&patch);
        }
    }
    Flow::Continue
}

/// The item half of both machines: a placed machine takes the tank its item carried, and
/// the amount is clamped to the capacity.
///
/// The game asks a Part only for the item it is attached to, and this one answers only for
/// this port's two blocks, so the same Part serves both.
pub struct MachineItem;

#[mod_api::horizon_part]
impl ItemPart for MachineItem {
    /// `BlockItemExtension.onBlockPlacedBy`, after the engine applied the item's
    /// `minecraft:block_entity_data` to the new block entity: write the tank back clamped,
    /// so an item doctored to 20,000 mB places a full 10,000 mB machine, and so a payload
    /// this port's own block entity type had to restore itself still arrives. A plain item
    /// carries no payload, and its machine is left exactly as the game placed it.
    fn on_block_placed_by(&self, host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
        let Some(asked) = asked_tank(event.stack) else {
            return;
        };
        // A dispenser has no placer, and the level is only reachable through one; that
        // machine's own first tick writes the tank back clamped instead.
        let Some(player) = event.placer else {
            return;
        };
        let Some(level) = player.level(&*host) else {
            return;
        };
        let ours = level
            .block(&*host, event.pos)
            .is_some_and(|block| Machine::of_block(&block.id(&*host)).is_some());
        if !ours {
            return;
        }
        let Some(entity) = host.block_entity(level, event.pos) else {
            return;
        };
        let Some(data) = entity.data(&*host) else {
            return;
        };
        let current = crate::blood::read_tank_raw(&data).unwrap_or(0);
        let wanted = restored_tank(asked).min(TANK_CAPACITY);
        if wanted == current {
            return;
        }
        let mut patch = Compound::new();
        crate::blood::write_tank(&mut patch, wanted);
        let _ = entity.merge_data(host, patch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The payload's tank, read back the way the machine reads its own.
    fn tank_of(patch: &Compound) -> i32 {
        let Some(Tag::Compound(payload)) = crate::blood::field(patch, BLOCK_ENTITY_DATA) else {
            return -1;
        };
        crate::blood::read_tank(payload)
    }

    #[test]
    fn a_broken_machine_carries_only_its_tank() {
        let patch = carried_patch(6_500, CHEST).expect("a filled tank travels");
        assert_eq!(tank_of(&patch), 6_500);
        let Some(Tag::Compound(payload)) = crate::blood::field(&patch, BLOCK_ENTITY_DATA) else {
            panic!("the patch holds no block entity data");
        };
        // The payload is the type id and the tank, nothing else: the engine spills a mod
        // container's stacks on removal, so an `Items` here would carry them twice.
        assert_eq!(
            payload.iter().count(),
            2,
            "the payload carries more than the id and the tank: {payload:?}"
        );
        assert!(
            crate::blood::field(payload, "Items").is_none(),
            "the payload carries the container's stacks"
        );
        assert!(
            matches!(
                crate::blood::field(payload, "id"),
                Some(Tag::String(id)) if id.to_string_lossy() == CHEST
            ),
            "the payload is not tagged with the machine's block entity type"
        );
    }

    #[test]
    fn an_empty_machine_carries_nothing() {
        assert!(carried_patch(0, CHEST).is_none());
        assert!(carried_patch(0, INFUSER).is_none());
    }

    #[test]
    fn a_doctored_payload_comes_back_clamped() {
        let patch = carried_patch(20_000, INFUSER).expect("a tank travels");
        // The payload itself is what a machine filled; the clamp is on placement.
        assert_eq!(tank_of(&patch), TANK_CAPACITY);
        assert_eq!(restored_tank(20_000), TANK_CAPACITY);
    }

    #[test]
    fn each_machine_names_its_own_block_and_type() {
        assert_eq!(Machine::of_block(CHEST), Some(Machine::Chest));
        assert_eq!(Machine::of_block(INFUSER), Some(Machine::Infuser));
        assert_eq!(Machine::of_block(COLOSSAL), Some(Machine::Colossal));
        assert_eq!(Machine::of_block("evilcraft:blood_extractor"), None);
        assert_eq!(Machine::of_block("minecraft:furnace"), None);
    }
}
