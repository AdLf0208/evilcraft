//! The Dark Tank (`BlockDarkTank`, `BlockEntityDarkTank`).
//!
//! Upstream's tank is one sixteen-bucket tank (`BlockEntityDarkTank.BASE_CAPACITY =
//! 16000`) with no slots and no menu: `BlockDarkTank.useItemOn` hands the click to
//! `FluidUtil.interactWithFluidHandler`, which fills the tank from the held fluid container.
//! Horizon has no blood fluid a capability item may hold ([`crate::blood`] keeps the
//! extractor's blood in the stack's `minecraft:custom_data`), so this port's container is the
//! Blood Extractor, and its **crouching** right-click pours it into the tank — the same
//! gesture the stain's draw uses in the other direction. A tank below a pipe or a container
//! item is upstream's auto-supply, which this slice does not carry (see `notes.md`).
//!
//! The tank is the pin's saved `fluids` list, exactly as the chests', the infuser's and the
//! stain's are ([`crate::blood::read_tank_with`]/[`crate::blood::write_tank_with`]):
//! `Hooks::register` declares the capability, the click writes the compound, and
//! `/data get block … fluids` reads back what was written.

use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockPart, CommandInvocation,
    CommandSource, Compound, HostSide, InteractionResult, ItemPart, LevelSide, ModFluidTanks,
    PlayerRef, ServerHost, UseOnEvent,
};

use crate::runtime::{DARK_TANK, DARK_TANK_CAPACITY, tank_fill};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const DARK_TANK_ENTITY: &str = "dark_tank";
/// The block the type belongs to, as the declaration builder takes it.
pub const DARK_TANK_BLOCK: &str = "dark_tank";

/// The Dark Tank's block entity: no tick, no slots and no screen — only the tank
/// `Hooks::register` declares.
pub struct DarkTank;

#[mod_api::horizon_part]
impl BlockPart for DarkTank {}

#[mod_api::horizon_part]
impl BlockEntityPart for DarkTank {
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

/// One sixteen-bucket tank every face reaches, and that only gives: the blood gets in through
/// the extractor's crouching click, never through a pipe (the machines' own gap 6: no
/// `block_entity_accepts_fluid` at this pin, so a foreign fluid would be pushed in and the
/// tank would then read 0 until blood is put back).
///
/// Built by assignment, not with a literal: `ModFluidTanks` is `#[non_exhaustive]`.
pub fn tanks() -> ModFluidTanks {
    let all = vec![0];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![DARK_TANK_CAPACITY];
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

/// The Blood Extractor's click on a Dark Tank: upstream's `BlockDarkTank.useItemOn` filling
/// the tank from a held fluid container. A standing click, an empty extractor and a full tank
/// all leave the click to the item itself, which is why this Part answers `None` beside a
/// `Success`: the stain's Part must still hear a click that is not this tank's.
pub struct TankFill;

#[mod_api::horizon_part]
impl ItemPart for TankFill {
    /// A player right-clicks a block holding this extractor. Only a crouching click on a dark
    /// tank pours anything; every other answer is `None`, so the item's own use (and the
    /// stain's Part) still runs.
    fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        // The tank and the stack are the server's; a client prediction would move either,
        // and the server's answer syncs the extractor back.
        let HostSide::Server(host) = host else {
            return None;
        };
        let player = host.players().acting_player()?;
        fill(host, player, event).then_some(InteractionResult::Success)
    }
}

/// The crouching click itself: the tank at the clicked position times the held extractor, one
/// transfer. Returns whether any blood moved.
///
/// Upstream does both halves through fluid capabilities in one transaction. That is not
/// available across a block entity and a player's slot at this pin, so the tank is written
/// first and put back if the stack write is refused: a refused write must not leave the
/// extractor holding blood the tank was already given.
fn fill(host: &mut dyn ServerHost, player: PlayerRef, event: &UseOnEvent<'_>) -> bool {
    // Upstream's tank interaction is a plain right-click; this port's container gesture is
    // the extractor's crouching click (the stain's rule, in the other direction), so a
    // standing click pours nothing.
    if !player.is_secondary_use_active(&*host).unwrap_or(false) {
        return false;
    }
    let Some(level) = player.level(&*host) else {
        return false;
    };
    let ours = level
        .block(&*host, event.pos)
        .is_some_and(|key| key.id(&*host) == DARK_TANK);
    if !ours {
        return false;
    }
    let Some(entity) = host.block_entity(level, event.pos) else {
        return false;
    };
    let Some(data) = entity.data(&*host) else {
        return false;
    };
    let tank = crate::blood::read_tank_with(&data, DARK_TANK_CAPACITY);
    let outcome = tank_fill(tank, crate::blood::extractor_blood(event.stack));
    if outcome.moved == 0 {
        return false;
    }
    if !write_tank(host, &entity, outcome.tank) {
        return false;
    }
    let emptied = crate::blood::with_extractor_blood(event.stack, outcome.extractor);
    let written = crate::blood::hand_slot(&*host, player, event.hand)
        .is_some_and(|slot| player.set_inventory_slot(host, slot, Some(emptied)).is_ok());
    if !written {
        // The extractor kept its blood, so the tank gives back what it was just given.
        let _ = write_tank(host, &entity, tank);
        return false;
    }
    true
}

/// Write the tank, clamped to its own capacity. False when the game refused the write, which
/// leaves the caller's blood where it was.
fn write_tank(host: &mut dyn ServerHost, entity: &BlockEntityRef, amount: i32) -> bool {
    let mut patch = Compound::new();
    crate::blood::write_tank_with(&mut patch, amount, DARK_TANK_CAPACITY);
    matches!(entity.merge_data(host, patch), Ok(true))
}

// ---------------------------------------------------------------------------
// The harness command
// ---------------------------------------------------------------------------

/// `/evilcraft darktank <pos>`: the tank, so a scenario can read the tank without a GUI and
/// without a pipe (upstream has no commands at all in this area).
pub fn readout(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !crate::blood::may_read(source.permission()) {
        crate::blood::say(
            host,
            format!(
                "evilcraft darktank needs an operator (permission level {})",
                crate::blood::OPERATOR_LEVEL
            ),
        );
        return;
    }
    let Some(pos) = crate::blood::block_pos(invocation) else {
        crate::blood::say(host, "usage: /evilcraft darktank <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) = crate::blood::our_entity(host, source, pos, DARK_TANK, "Dark Tank") else {
        return;
    };
    let Some(data) = entity.data(&*host) else {
        crate::blood::say(host, format!("{pos:?} is not a Dark Tank."));
        return;
    };
    let tank = crate::blood::read_tank_with(&data, DARK_TANK_CAPACITY);
    let time = entity
        .level(&*host)
        .and_then(|level| level.game_time(&*host))
        .unwrap_or(0);
    crate::blood::say(
        host,
        format!("Dark Tank {pos:?}: tank {tank}/{DARK_TANK_CAPACITY} mB, t={time}"),
    );
}
