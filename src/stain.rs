//! The game side of EvilCraft's Blood Stain (`BlockBloodStain`,
//! `BlockEntityBloodStain`).
//!
//! Upstream's stain is a splash a mob's fall leaves on the floor, holding
//! `bloodMBPerHP x max health` mB in a five-bucket tank; a **crouching** right-click with a
//! Blood Extractor moves that blood into the extractor itself
//! (`ItemBloodExtractor.onItemUseFirst`, through `Capabilities.Fluid.BLOCK`), and a stain
//! a committed change drains to 0 removes its block (`BlockEntityBloodStain.setAmount`).
//! The rules are [`crate::runtime`]; this file reads and writes the game.
//!
//! The tank is the pin's saved `fluids` list, exactly as the Blood Chest's and Blood
//! Infuser's are ([`crate::blood::read_tank_with`]/[`crate::blood::write_tank_with`]):
//! `Hooks::register` declares the capability, so a neighbouring pipe can drain it, and the
//! click writes the compound as well.

use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockPart, CommandInvocation,
    CommandSource, Compound, Hand, Host, HostSide, InteractionResult, ItemPart, LevelSide,
    ModFluidTanks, PlayerRef, ServerHost, UseOnEvent,
};

use crate::runtime::{STAIN, STAIN_TANK, stain_draw};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const STAIN_ENTITY: &str = "blood_stain";
/// The block the type belongs to, as the declaration builder takes it.
pub const STAIN_BLOCK: &str = "blood_stain";
/// Vanilla's inventory slot of the off hand: the slot number
/// `PlayerRef::set_inventory_slot` takes for a click made with it.
const OFF_HAND_SLOT: u32 = 40;

// ---------------------------------------------------------------------------
// The block entity
// ---------------------------------------------------------------------------

/// The Blood Stain's block entity: it has no tick, no slots and no screen — only the tank
/// `Hooks::register` declares.
pub struct Stain;

#[mod_api::horizon_part]
impl BlockPart for Stain {}

#[mod_api::horizon_part]
impl BlockEntityPart for Stain {
    /// The entity starts with an empty saved compound: the creation declaration owns its
    /// type, so no data is needed to make one.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        None
    }
}

/// One five-bucket tank every face reaches, and that only gives: upstream's
/// `BlockEntityBloodStain.FluidHandler` answers `insert` with 0 and takes a drain. Blood
/// gets in through the fall a mob leaves it from (a later slice) or the harness command,
/// never through a pipe.
///
/// Built by assignment, not with a literal: `ModFluidTanks` is `#[non_exhaustive]`.
pub fn tanks() -> ModFluidTanks {
    let all = vec![0];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![STAIN_TANK];
    tanks.faces = [
        all.clone(),
        all.clone(),
        all.clone(),
        all.clone(),
        all.clone(),
        all,
    ];
    tanks.inputs = Vec::new();
    tanks.outputs = vec![0];
    tanks
}

// ---------------------------------------------------------------------------
// The extractor's crouching click
// ---------------------------------------------------------------------------

/// The Blood Extractor's click on a Blood Stain: upstream's `onItemUseFirst`. A standing
/// click does nothing, and no other block answers it.
pub struct ExtractorDraw;

#[mod_api::horizon_part]
impl ItemPart for ExtractorDraw {
    /// A player right-clicks a block holding this extractor. Only a crouching click on a
    /// stain takes anything; every other answer is `Pass`, so the item's own use still
    /// runs (`ItemBloodExtractor.use` toggles auto-supply there).
    fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        // The tank and the stack are the server's; a client prediction would move either,
        // and the server's answer syncs the extractor back.
        let HostSide::Server(host) = host else {
            return Some(InteractionResult::Pass);
        };
        let Some(player) = host.players().acting_player() else {
            return Some(InteractionResult::Pass);
        };
        Some(if draw(host, player, event) {
            InteractionResult::Success
        } else {
            InteractionResult::Pass
        })
    }
}

/// The crouching click itself: the stain at the clicked position times the held extractor,
/// one transfer, and the stain's block removed when it is emptied.
///
/// Upstream does both halves through fluid capabilities in one transaction. That is not
/// available across a block entity and a player's slot at this pin, so the tank is written
/// first and put back if the stack write is refused: a refused write must not leave the
/// player holding blood the stain still has.
fn draw(host: &mut dyn ServerHost, player: PlayerRef, event: &UseOnEvent<'_>) -> bool {
    // Upstream's `if (!context.getPlayer().isCrouching()) return PASS`.
    if !player.is_secondary_use_active(&*host).unwrap_or(false) {
        return false;
    }
    let Some(level) = player.level(&*host) else {
        return false;
    };
    let ours = level
        .block(&*host, event.pos)
        .is_some_and(|key| key.id(&*host) == STAIN);
    if !ours {
        return false;
    }
    let Some(entity) = host.block_entity(level, event.pos) else {
        return false;
    };
    let Some(data) = entity.data(&*host) else {
        return false;
    };
    let stain = crate::blood::read_tank_with(&data, STAIN_TANK);
    let outcome = stain_draw(stain, crate::blood::extractor_blood(event.stack));
    if outcome.moved == 0 {
        return false;
    }
    if !write_stain(host, &entity, outcome.stain) {
        return false;
    }
    let filled = crate::blood::with_extractor_blood(event.stack, outcome.extractor);
    let written = hand_slot(&*host, player, event.hand)
        .is_some_and(|slot| player.set_inventory_slot(host, slot, Some(filled)).is_ok());
    if !written {
        // The extractor took nothing, so the blood stays where it was.
        let _ = write_stain(host, &entity, stain);
        return false;
    }
    if outcome.drains_the_stain() {
        let _ = level.remove_block(host, event.pos, false);
    }
    true
}

/// Write the stain's tank, clamped to its own capacity. False when the game refused the
/// write, which leaves the caller's blood where it was.
fn write_stain(host: &mut dyn ServerHost, entity: &BlockEntityRef, amount: i32) -> bool {
    let mut patch = Compound::new();
    crate::blood::write_tank_with(&mut patch, amount, STAIN_TANK);
    matches!(entity.merge_data(host, patch), Ok(true))
}

/// The inventory slot the click's hand holds: the selected hotbar slot for the main hand,
/// vanilla's off-hand slot for the other.
fn hand_slot(host: &dyn Host, player: PlayerRef, hand: Hand) -> Option<u32> {
    match hand {
        Hand::Main => player
            .selected_slot(host)
            .and_then(|slot| u32::try_from(slot).ok()),
        Hand::Off => Some(OFF_HAND_SLOT),
    }
}

// ---------------------------------------------------------------------------
// The harness command
// ---------------------------------------------------------------------------

/// `/evilcraft stain <pos>`: the tank, so a scenario can read a stain without a GUI
/// (upstream has no commands at all in this area).
pub fn readout(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !crate::blood::may_read(source.permission()) {
        crate::blood::say(
            host,
            format!(
                "evilcraft stain needs an operator (permission level {})",
                crate::blood::OPERATOR_LEVEL
            ),
        );
        return;
    }
    let Some(pos) = crate::blood::block_pos(invocation) else {
        crate::blood::say(host, "usage: /evilcraft stain <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) = crate::blood::our_entity(host, source, pos, STAIN, "Blood Stain") else {
        return;
    };
    let Some(data) = entity.data(&*host) else {
        crate::blood::say(host, format!("{pos:?} is not a Blood Stain."));
        return;
    };
    let tank = crate::blood::read_tank_with(&data, STAIN_TANK);
    let time = entity
        .level(&*host)
        .and_then(|level| level.game_time(&*host))
        .unwrap_or(0);
    crate::blood::say(
        host,
        format!("Blood Stain {pos:?}: tank {tank}/{STAIN_TANK} mB, t={time}"),
    );
}
