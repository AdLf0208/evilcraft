//! The Colossal Blood Chest (`BlockColossalBloodChest`, `BlockEntityColossalBloodChest`).
//!
//! Upstream's machine is a 3×3×3 structure: a hollow cube of reinforced undead planks with
//! exactly one colossal blood chest in its wall (`HollowCubeDetector` +
//! `ExactSizeValidator(2, 2, 2)` + `ExactBlockCountValidator(1)`). The controller holds 45
//! repair slots in five rows of nine, one fluid-container slot, four upgrade slots and a
//! ten-bucket blood tank, and repairs one damage point per two ticks for five mB — the same
//! `RepairItemTickAction` numbers the Blood Chest uses — but only while the structure is
//! whole (`canWork()` asks `getSize()` for the exact size). The container slot empties into
//! the tank whether or not the cube is formed (`EmptyInTankTickAction.canTick` does not ask
//! `canWork`).
//!
//! The structure is derived from the world on every tick ([`crate::runtime::colossal_formed`])
//! instead of being remembered in saved data: the machine is only ever as formed as the
//! blocks around it, which is the rule the detector enforces at every change. Deviation: the
//! tick needs the level the controller stands in, and a block entity's tick carries only its
//! position, so the level is found among the server's open levels (`Levels::list_levels`)
//! by the controller's own block.

use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent,
    BlockPart, ClientHost, CommandInvocation, CommandSource, Compound, Host, LevelSide,
    ModContainer, ModFluidTanks, Screen, ScreenDrawEvent, ScreenEvent, ServerHost,
};

use crate::runtime::{
    COLOSSAL, COLOSSAL_CONTAINER_SLOT, COLOSSAL_INVENTORY, COLOSSAL_SLOT_COUNT, COLOSSAL_SLOTS,
    TANK_CAPACITY, anvil_step, can_hold, colossal_formed, drain_container, repair, repair_due,
    structure_cell,
};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const COLOSSAL_ENTITY: &str = "colossal_blood_chest";
/// The block the type belongs to, as the declaration builder takes it.
pub const COLOSSAL_BLOCK: &str = "colossal_blood_chest";
/// The menu type the content declares
/// (`data/evilcraft/horizon/menu/colossal_blood_chest_menu.json`).
const MENU: &str = "evilcraft:colossal_blood_chest_menu";
/// The block entity's saved tick counter, which paces the repair (`ticksPerDamage`).
const TICKS: &str = "ticks";

/// The Colossal Blood Chest's block entity: its structure scan, its repair tick, its fifty
/// slots and its screen all come from the declarations in `Hooks::register`.
pub struct Colossal;

#[mod_api::horizon_part]
impl BlockPart for Colossal {}

#[mod_api::horizon_part]
impl BlockEntityPart for Colossal {
    /// The entity starts with an empty saved compound: the creation declaration owns its
    /// type, so no data is needed to make one.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        None
    }

    /// The machine ticks on the server.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: empty the container slot into the tank, then (only while the 3×3×3
    /// shell is whole) repair the stacks the tick's pace says are due.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        // The structure is read before the entity is borrowed: the scan needs the level and
        // the entity needs the tick's mutable view.
        let formed = formed_at(host, event.pos);
        let entity = &mut *event.entity;
        if entity.items.len() < COLOSSAL_INVENTORY {
            return;
        }
        let mut tank = crate::blood::read_tank(&entity.nbt);
        // The container slot empties a blood container into the tank, as upstream's
        // `EmptyFluidContainerInTankTickAction` does; that action does not ask `canWork`,
        // so blood goes in even while the cube is open.
        if let Some(container) = entity.items[COLOSSAL_CONTAINER_SLOT].take() {
            let extractor = container.item().id(host) == crate::runtime::EXTRACTOR;
            let (filled, keeps) = drain_container(tank, crate::blood::extractor_blood(&container));
            tank = filled;
            entity.items[COLOSSAL_CONTAINER_SLOT] = Some(if extractor {
                crate::blood::with_extractor_blood(&container, keeps)
            } else {
                container
            });
        }
        let ticks = crate::blood::read_int(&entity.nbt, TICKS)
            .unwrap_or(0)
            .max(0) as u32;
        // `BulkRepairItemTickAction.canTick` asks `tile.canWork()` first: an open shell
        // repairs nothing, whatever the tank holds.
        if formed && repair_due(ticks) {
            for slot in 0..COLOSSAL_SLOTS {
                let Some(mut stack) = entity.items[slot].take() else {
                    continue;
                };
                let damage = i32::try_from(stack.damage()).unwrap_or(i32::MAX);
                let item = stack.item().id(host);
                if can_hold(&item, damage) {
                    // The chest's own numbers: `baseMBPerDamage` (5) a point, and an anvil
                    // step for the action's 25× multiplier (125 mB).
                    if damage > 0 {
                        if let Some((left, tank_left)) = repair(damage, tank) {
                            write_damage(&mut stack, u32::try_from(left).unwrap_or(0));
                            tank = tank_left;
                        }
                    } else if let Some((next, tank_left)) = anvil_step(&item, tank)
                        && let Some(whole) = item_stack(host, next)
                    {
                        stack = whole;
                        tank = tank_left;
                    }
                }
                entity.items[slot] = Some(stack);
            }
        }
        crate::blood::write_tank(&mut entity.nbt, tank);
        crate::blood::write_int(&mut entity.nbt, TICKS, ticks.wrapping_add(1) as i32);
        // The menu's two data slots: the tank and its capacity. Set only where the value
        // moved, because the vector is what the window syncs.
        if entity.data.len() != 2 || entity.data[0] != tank {
            entity.data = vec![tank, TANK_CAPACITY];
        }
    }

    /// The machine's fifty slots: the 45 repair slots in five rows of nine (their inventory
    /// order, which is the menu's), the container slot under them, the four upgrade slots
    /// beside it, and the player's inventory below.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        let mut slots = Vec::with_capacity(COLOSSAL_INVENTORY);
        for row in 0..5 {
            for column in 0..9 {
                slots.push(Some([
                    59.0 + 18.0 * column as f32,
                    13.0 + 18.0 * row as f32,
                ]));
            }
        }
        slots.push(Some([6.0, 46.0]));
        for column in 0..crate::runtime::UPGRADE_SLOTS {
            slots.push(Some([6.0 + 18.0 * column as f32, 68.0]));
        }
        Some(Screen {
            width: 256.0,
            height: 232.0,
            slots,
            inventory_y: 138.0,
            inventory_x: 28.0,
            own_labels: false,
            own_background: false,
            ..Screen::default()
        })
    }

    /// The machine's tank readout: the menu's data slots are `[tank, capacity]`.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let tank = event.data.first().copied().unwrap_or(0).max(0);
        let text = format!("{tank} / {TANK_CAPACITY} mB");
        let at = [event.origin[0] + 6.0, event.origin[1] + 30.0];
        event.draw.text(&text, at, 0xFF40_4040, false);
    }
}

/// The machine's fifty slots and its menu (`Registrar::block_container`). Upstream's
/// `ItemHandlerSlotMasked` exists on the block entity (the bottom face reaches only the
/// container slot), but the port's declarations reach no face at all for the Blood Chest and
/// the Infuser either, so a pipe moves nothing here: the menu and the tick are the routes.
pub fn container() -> ModContainer {
    ModContainer {
        slots: COLOSSAL_INVENTORY,
        menu: Some(MENU.to_owned()),
        ..ModContainer::default()
    }
}

/// One ten-bucket tank, every face reaching it, as the other two machines have.
///
/// Built by assignment, not with a literal: `ModFluidTanks` is `#[non_exhaustive]`.
pub fn tanks() -> ModFluidTanks {
    let all = vec![0];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![TANK_CAPACITY];
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

/// Whether the 3×3×3 shell around the controller at `pos` is whole, in the level the
/// controller stands in.
pub fn formed_at(host: &dyn ServerHost, pos: [i32; 3]) -> bool {
    for level in host.levels().list_levels() {
        let here = level
            .block(host, pos)
            .map(|key| key.id(host))
            .unwrap_or_default();
        if here != COLOSSAL {
            continue;
        }
        return colossal_formed(pos, |cell| {
            let id = level
                .block(host, cell)
                .map(|key| key.id(host))
                .unwrap_or_default();
            structure_cell(&id)
        })
        .is_some();
    }
    false
}

/// A stack's damage: a whole one carries no damage component at all, so it stacks with a
/// fresh one (the chest's own `write_damage`).
fn write_damage(stack: &mut mod_api::ItemStack, damage: u32) {
    if damage == 0 {
        stack.clear_damage();
    } else {
        stack.set_damage(damage);
    }
}

/// The item `id` as a fresh stack of one, for an anvil step that changes the item.
fn item_stack(host: &dyn Host, id: &str) -> Option<mod_api::ItemStack> {
    let identifier = mod_api::Identifier::parse(id).ok()?;
    host.item_key(&identifier).map(mod_api::ItemStack::of)
}

// ---------------------------------------------------------------------------
// The harness command
// ---------------------------------------------------------------------------

/// `/evilcraft colossal <pos>`: the structure state, the tank, the container slot and every
/// repair slot's damage, so a scenario can read the machine without a GUI (upstream has no
/// command).
pub fn readout(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !crate::blood::may_read(source.permission()) {
        crate::blood::say(
            host,
            format!(
                "evilcraft colossal needs an operator (permission level {})",
                crate::blood::OPERATOR_LEVEL
            ),
        );
        return;
    }
    let Some(pos) = crate::blood::block_pos(invocation) else {
        crate::blood::say(host, "usage: /evilcraft colossal <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) =
        crate::blood::our_entity(host, source, pos, COLOSSAL, "Colossal Blood Chest")
    else {
        return;
    };
    let Some(data) = entity.data(host) else {
        crate::blood::say(host, format!("{pos:?} is not a Colossal Blood Chest."));
        return;
    };
    let tank = crate::blood::read_tank(&data);
    let formed = formed_at(host, pos);
    let container = match entity.item_slot(host, COLOSSAL_CONTAINER_SLOT as u32) {
        Ok(Some(stack)) => format!(
            "{}@{}",
            stack.item().id(host),
            crate::blood::extractor_blood(&stack)
        ),
        Ok(None) => "empty@0".to_owned(),
        Err(_) => "unknown@0".to_owned(),
    };
    let mut slots = String::new();
    for slot in 0..COLOSSAL_SLOTS {
        let Ok(Some(stack)) = entity.item_slot(host, slot as u32) else {
            continue;
        };
        slots.push_str(&format!(
            "{slot}:{}@{} ",
            stack.item().id(host),
            stack.damage()
        ));
    }
    let time = entity
        .level(host)
        .and_then(|level| level.game_time(host))
        .unwrap_or(0);
    crate::blood::say(
        host,
        format!(
            "Colossal Blood Chest {pos:?}: formed={formed}, tank {tank}/{TANK_CAPACITY} mB, repair {COLOSSAL_SLOTS}, slots {COLOSSAL_SLOT_COUNT}, container {container}, [{}], t={time}",
            slots.trim_end()
        ),
    );
}
