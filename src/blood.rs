//! The game side of EvilCraft's blood repair loop: the Blood Chest's block entity
//! Part, the Blood Extractor's item Part, and the harness commands.
//!
//! Upstream runs a Blood Chest's ten repair slots through
//! `RepairItemTickAction`/`BloodChestRepairActionRegistry` and its Blood Extractor
//! through `ItemBloodExtractor.fillForAllBloodExtractors`; [`crate::runtime`] states
//! those rules, and this file reads and writes the game around them.
//!
//! The chest's tank is the pin's saved `fluids` list: one compound of `Fluid` and
//! `Amount` per tank (`crates/horizon-client/src/game/blocks/machines/native_fluids.rs`
//! reads and writes exactly that), so the tank survives restarts and other blocks
//! reach it through [`FLUIDS`]. The extractor's blood is an int under the stack's
//! `minecraft:custom_data`, because the pin's item fluid helpers carry vanilla's
//! fluids only (`ITEM_FLUIDS` has water, lava and a mod fluid a provider registers).

use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTicksEvent, BlockPart, ClientHost, Command, CommandArgumentValue, CommandInvocation,
    CommandSource, Component, Compound, EntityRef, Flow, Hand, Host, HostTransaction, Identifier,
    ItemPart, ItemPlace, ItemStack, LevelRef, LevelSide, LivingDeathEvent, ModContainer,
    ModFluidTanks, PlayerRef, RandomSource, RegisterCommandsEvent, RunModCommandEvent, Screen,
    ScreenDrawEvent, ScreenEvent, ServerHost, Tag, Text, TooltipEvent,
};

use crate::runtime::{
    BLOOD, CHEST_SLOTS, CONTAINER_SLOT, Credit, EXTRACTOR, EXTRACTOR_CAPACITY, SLOT_COUNT,
    TANK_CAPACITY, anvil_step, blood_from_kill, can_hold, drain_container, fill_extractor,
    kill_span, repair, repair_due,
};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const CHEST_ENTITY: &str = "blood_chest";
/// The block the type belongs to, as the declaration builder takes it.
pub const CHEST_BLOCK: &str = "blood_chest";
/// The Blood Chest's full block id, as `Level::block` names it.
const CHEST_ID: &str = "evilcraft:blood_chest";

/// The menu type the content declares (`data/evilcraft/horizon/menu/blood_chest_menu.json`).
const MENU: &str = "evilcraft:blood_chest_menu";
/// Where the extractor keeps its blood: an int under the stack's
/// `minecraft:custom_data`. The pin's item fluid helpers carry vanilla's fluids only,
/// so a mod fluid lives in the stack's own data.
const CUSTOM_DATA: &str = "minecraft:custom_data";
const BLOOD_KEY: &str = "evilcraft:blood";
/// The block entity's saved tick counter, which paces the repair (`ticksPerDamage`).
const TICKS: &str = "ticks";
/// The saved fluid list of a mod block's tanks.
const FLUIDS: &str = "fluids";

// ---------------------------------------------------------------------------
// The Parts
// ---------------------------------------------------------------------------

/// The blood chest's block entity (`evilcraft:blood_chest`): its tick, its screen and
/// its slot count come from the declarations in `Hooks::register`.
pub struct Chest;

/// The blood extractor item: its tooltip.
pub struct Extractor;

#[mod_api::horizon_part]
impl BlockPart for Chest {}

#[mod_api::horizon_part]
impl BlockEntityPart for Chest {
    /// The entity starts with an empty saved compound: the creation declaration owns
    /// its type, so no data is needed to make one.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        None
    }

    /// The chest ticks on the server.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: empty the container slot into the tank, then repair the stacks
    /// the tick's pace says are due.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let entity = &mut *event.entity;
        if entity.items.len() < SLOT_COUNT {
            return;
        }
        let mut tank = read_tank(&entity.nbt);
        // The container slot empties a blood container into the tank, as upstream's
        // `EmptyFluidContainerInTankTickAction` does.
        if let Some(container) = entity.items[CONTAINER_SLOT].take() {
            // The container stays in its slot once emptied, as upstream's
            // `EmptyFluidContainerInTankTickAction` leaves it; the old code dropped the
            // extractor item when its last blood went into the tank. Any other item is
            // left exactly as it was: only an extractor carries blood.
            let extractor = container.item().id(host) == EXTRACTOR;
            let (filled, keeps) = drain_container(tank, extractor_blood(&container));
            tank = filled;
            entity.items[CONTAINER_SLOT] = Some(if extractor {
                with_extractor_blood(&container, keeps)
            } else {
                container
            });
        }
        let ticks = read_int(&entity.nbt, TICKS).unwrap_or(0).max(0) as u32;
        if repair_due(ticks) {
            for slot in 0..CHEST_SLOTS {
                let Some(mut stack) = entity.items[slot].take() else {
                    continue;
                };
                let damage = i32::try_from(stack.damage()).unwrap_or(i32::MAX);
                let item = stack.item().id(host);
                if can_hold(&item, damage) {
                    // Upstream's `RepairItemTickAction`: the action's multiplier scales
                    // `mBPerDamage`, and the tank then pays `mBPerDamage x multiplier`.
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
        write_tank(&mut entity.nbt, tank);
        write_int(&mut entity.nbt, TICKS, ticks.wrapping_add(1) as i32);
        // The menu's two data slots: the tank and its capacity. Set only where the
        // value moved, because the vector is what the window syncs.
        if entity.data.len() != 2 || entity.data[0] != tank {
            entity.data = vec![tank, TANK_CAPACITY];
        }
    }

    /// The chest's eleven-slot layout: ten repair slots in two rows of five, the
    /// container slot beside them, the player's inventory under.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        let mut slots = Vec::with_capacity(SLOT_COUNT);
        for row in 0..2 {
            for column in 0..5 {
                slots.push(Some([8.0 + 18.0 * column as f32, 18.0 + 18.0 * row as f32]));
            }
        }
        slots.push(Some([116.0, 27.0]));
        Some(Screen {
            width: 176.0,
            height: 166.0,
            slots,
            inventory_y: 84.0,
            inventory_x: 8.0,
            own_labels: false,
            own_background: false,
            ..Screen::default()
        })
    }

    /// The chest's tank readout: the menu's data slots are `[tank, capacity]`.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let tank = event.data.first().copied().unwrap_or(0).max(0);
        let text = format!("{tank} / {TANK_CAPACITY} mB");
        let at = [event.origin[0] + 8.0, event.origin[1] + 58.0];
        event.draw.text(&text, at, 0xFF40_4040, false);
    }
}

#[mod_api::horizon_part]
impl ItemPart for Extractor {
    /// The extractor's own tooltip: the blood it holds.
    fn tooltip(&self, _host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let blood = extractor_blood(event.stack);
        event.lines.push((
            Text::literal(format!("Blood: {blood}/{EXTRACTOR_CAPACITY} mB")),
            0xFF00_A0A0,
        ));
    }
}

/// The chest's eleven slots and its menu (`native_block_entity_container`'s
/// replacement: `Registrar::block_container`). No face reaches the slots, so a pipe or
/// a hopper cannot move a stack in or out, as upstream's `WorldlyContainer` says.
pub fn container() -> ModContainer {
    ModContainer {
        slots: SLOT_COUNT,
        menu: Some(MENU.to_owned()),
        // No face reaches a slot, so no pipe or hopper moves a stack in or out, as
        // upstream's `WorldlyContainer` says; the rest is empty storage.
        ..ModContainer::default()
    }
}

/// One 10-bucket tank, every face reaching it, so a neighbouring pipe can take the
/// blood out (`native_block_entity_fluids`'s replacement: `Registrar::block_fluids`).
///
/// Built by assignment, not with a literal: `ModFluidTanks` is `#[non_exhaustive]`,
/// so another crate may fill a `Default` but not name its struct expression.
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

// ---------------------------------------------------------------------------
// The extractor: kill blood
// ---------------------------------------------------------------------------

/// A mob or player is about to die (`living_death`, NeoForge's `LivingDeathEvent`): the
/// extractors the player credited with the kill carries take the victim's blood.
///
/// This is upstream's own event: `ItemBloodExtractor.bloodObtainEvent` asks whether the
/// killing blow's damage source names a `ServerPlayer`, so a player's arrow counts and a
/// skeleton's arrow does not. The pin's `mob_killed` heard melee, sweep and spear only
/// (gap 4), and its kill-credit step (`loot_drops`) runs too late to write a player's
/// inventory: a fill made there is gone by the next read (measured — the hook's own re-read
/// saw the new stack, `/evilcraft held` read 0 afterwards). `living_death` is the moment
/// the blow is still live and a write lands.
pub fn on_living_death(host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) {
    let (credit, player) = credit(host, event.source.causing);
    if !credit.fills() {
        return;
    }
    let Some(player) = player else {
        return;
    };
    let Some(health) = max_health(host, event.entity) else {
        return;
    };
    fill_for(host, player, health);
}

/// The credit a death carries and the player it names, if any, from the killing blow's
/// causing entity: upstream's answer (`getSource().getEntity()`), resolved through the
/// server's own entities. A skeleton's arrow names the skeleton, `/kill` names nobody.
fn credit(host: &mut dyn ServerHost, causing: i32) -> (Credit, Option<PlayerRef>) {
    let Some(entity) = u32::try_from(causing)
        .ok()
        .filter(|id| *id != 0)
        .and_then(|id| host.entity(id))
    else {
        return (Credit::Nobody, None);
    };
    match player_at(host, entity) {
        Some(player) => (Credit::Player, Some(player)),
        None => (Credit::Nobody, None),
    }
}

/// The server player behind `entity`, if it is one.
fn player_at(host: &mut dyn ServerHost, entity: EntityRef) -> Option<PlayerRef> {
    host.players()
        .server_players()
        .into_iter()
        .find(|player| player.entity() == entity)
}

/// Fill the extractors `player` carries from the blood of a mob of `health` maximum health.
fn fill_for(host: &mut dyn ServerHost, player: PlayerRef, health: f32) {
    let Some(level) = player.level(host) else {
        return;
    };
    // Upstream's `rand.nextInt(max(1, maximumMB - minimumMB))` on the level's own
    // random, so the draw moves vanilla's later ones as a Java mod's does.
    let span = i32::try_from(kill_span(health)).unwrap_or(1).max(1);
    let roll = level
        .with_random(host, |random| random.next_int_bound(span).unwrap_or(0))
        .unwrap_or(0);
    let blood = blood_from_kill(health, u32::try_from(roll.max(0)).unwrap_or(0));
    if blood <= 0 {
        return;
    }
    let Some(slots) = player.inventory_slots(host) else {
        return;
    };
    let mut pool = blood;
    for (slot, stack) in slots {
        if pool <= 0 {
            break;
        }
        if stack.item().id(host) != EXTRACTOR {
            continue;
        }
        let held = extractor_blood(&stack);
        let (now, left) = fill_extractor(held, pool);
        if now == held {
            continue;
        }
        // A refused write (a slot the player no longer has) keeps the blood for the
        // next extractor rather than losing it.
        if player
            .set_inventory_slot(host, slot, Some(with_extractor_blood(&stack, now)))
            .is_ok()
        {
            pool = left;
        }
    }
}

/// The killed mob's maximum health, the number upstream multiplies (the stain's blood too).
pub(crate) fn max_health(host: &dyn Host, entity: u32) -> Option<f32> {
    let entity = host.entity(entity)?;
    let living = host.living(entity)?;
    living.max_health(host)
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// The commands (`register_commands`). Upstream has none: they exist so the harness
/// can read and set a chest without a GUI (precedent: `/charginggadgets info|energy`).
pub fn register_commands(_host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
    // The readout names the chest's container slot, its ten slots' damage and its tank
    // anywhere in the level, so it is registered at the operator level `/data get block`
    // needs (2, game master) and its body asks again before reading anything.
    event.commands.push(Command::op(
        "evilcraft info <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
    event.commands.push(Command::op(
        "evilcraft blood <pos:block_pos> <amount:integer>",
        2,
    ));
    event.commands.push(Command::op(
        "evilcraft damage <pos:block_pos> <slot:integer> <amount:integer>",
        2,
    ));
    // The infuser's readout and its slot loader: the machine's own menu is the player's
    // route, these let a scenario read and feed it without a GUI.
    event.commands.push(Command::op(
        "evilcraft infuser <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
    // The Blood Stain's readout: its five-bucket tank, for a scenario that cannot click
    // its way to a number.
    event.commands.push(Command::op(
        "evilcraft stain <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
    // The Colossal Blood Chest's readout: whether its 3×3×3 shell is whole, its tank, its
    // container slot and its repair slots.
    event.commands.push(Command::op(
        "evilcraft colossal <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
    // The Dark Tank's readout: its sixteen-bucket tank.
    event.commands.push(Command::op(
        "evilcraft darktank <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
    // `<id:id>` is a free `namespace:path` identifier: a `word` argument stops at the
    // colon (the campaign's `item`-argument trap, so no `item` kind here).
    event.commands.push(Command::op(
        "evilcraft load <pos:block_pos> <slot:integer> <id:id>",
        2,
    ));
    event.commands.push(Command::new("evilcraft held"));
    event
        .commands
        .push(Command::op("evilcraft held <amount:integer>", 2));
}

/// `/evilcraft info|blood|damage|infuser|stain|colossal|darktank|load|held`
/// (`run_mod_command`).
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let source = event.source;
    let invocation = event.command;
    if invocation.literals.first().map(String::as_str) != Some("evilcraft") {
        return Flow::Continue;
    }
    match invocation.literals.get(1).map_or("", String::as_str) {
        "info" => info(host, source, invocation),
        "blood" => set_blood(host, source, invocation),
        "damage" => set_damage(host, source, invocation),
        "infuser" => crate::infuser::readout(host, source, invocation),
        "stain" => crate::stain::readout(host, source, invocation),
        "colossal" => crate::colossal::readout(host, source, invocation),
        "darktank" => crate::dark_tank::readout(host, source, invocation),
        "load" => crate::infuser::load(host, source, invocation),
        "held" => held(host, source, invocation),
        _ => say(
            host,
            "usage: /evilcraft info|blood|damage|infuser|stain|colossal|darktank|load|held ..."
                .to_owned(),
        ),
    }
    Flow::Cancel
}

/// `/evilcraft info <pos>`: the tank, the container slot and every chest slot's damage.
fn info(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !may_read(source.permission()) {
        say(
            host,
            format!("evilcraft info needs an operator (permission level {OPERATOR_LEVEL})"),
        );
        return;
    }
    let Some(pos) = block_pos(invocation) else {
        say(host, "usage: /evilcraft info <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) = mod_entity(host, source, pos) else {
        return;
    };
    let Some(data) = entity.data(host) else {
        say(host, format!("{pos:?} is not a Block Chest."));
        return;
    };
    let tank = read_tank(&data);
    let container = match entity.item_slot(host, CONTAINER_SLOT as u32) {
        Ok(Some(stack)) => format!("{}@{}", stack.item().id(host), extractor_blood(&stack)),
        Ok(None) => "empty@0".to_owned(),
        Err(_) => "unknown@0".to_owned(),
    };
    let mut slots = String::new();
    for slot in 0..CHEST_SLOTS {
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
    say(
        host,
        format!(
            "Blood Chest {pos:?}: tank {tank}/{TANK_CAPACITY} mB, container {container}, slots [{}], t={time}",
            slots.trim_end()
        ),
    );
}

/// `/evilcraft blood <pos> <amount>`: set a machine's tank, for a scenario that cannot
/// wait for a kill or a fill. The tank's own capacity is the clamp: 10,000 mB for the two
/// machines, 5,000 for the Blood Stain.
fn set_blood(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    let Some(pos) = block_pos(invocation) else {
        say(
            host,
            "usage: /evilcraft blood <x> <y> <z> <amount>".to_owned(),
        );
        return;
    };
    let Some(amount) = integer(invocation, "amount") else {
        say(
            host,
            "usage: /evilcraft blood <x> <y> <z> <amount>".to_owned(),
        );
        return;
    };
    let Some((entity, capacity)) = tanked(host, source, pos) else {
        return;
    };
    let amount = amount.clamp(0, capacity);
    let mut patch = Compound::new();
    write_tank_with(&mut patch, amount, capacity);
    // A merge, not a replacement: the saved tick counter has to survive the command.
    let changed = matches!(entity.merge_data(host, patch), Ok(true));
    if changed {
        let _ = entity.set_data_slots(host, vec![amount, capacity]);
        say(
            host,
            format!("Blood tank at {pos:?}: {amount}/{capacity} mB."),
        );
    } else {
        say(host, format!("{pos:?} has no blood tank."));
    }
}

/// `/evilcraft damage <pos> <slot> <amount>`: set a stack's damage, so a scenario does
/// not need to wear a tool down.
fn set_damage(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    let usage = "usage: /evilcraft damage <x> <y> <z> <slot> <amount>";
    let (Some(pos), Some(slot), Some(amount)) = (
        block_pos(invocation),
        integer(invocation, "slot"),
        integer(invocation, "amount"),
    ) else {
        say(host, usage.to_owned());
        return;
    };
    let Some((entity, noun, slots)) = machine_at(host, source, pos) else {
        return;
    };
    let slot = u32::try_from(slot.max(0)).unwrap_or(0);
    if slot as usize >= slots {
        say(host, format!("{pos:?} has no slot {slot}."));
        return;
    }
    let Ok(Some(mut stack)) = entity.item_slot(host, slot) else {
        say(host, format!("{pos:?} has no such stack."));
        return;
    };
    write_damage(&mut stack, u32::try_from(amount.max(0)).unwrap_or(0));
    let item = stack.item().id(host);
    let written = host
        .transaction(|host, tx| -> Result<bool, ()> {
            Ok(entity
                .set_item_slot(host, tx, slot, Some(stack))
                .unwrap_or(false))
        })
        .unwrap_or(false);
    say(
        host,
        match written {
            true => format!("{noun} at {pos:?}: {item} at damage {}.", amount.max(0)),
            false => format!("{pos:?} kept its stack."),
        },
    );
}

/// The machine at `pos` whose stack slots this port's harness commands may write — the
/// Blood Chest, the Blood Infuser or the Colossal Blood Chest — with the name a reply uses
/// and how many slots it has, or a line saying the position is not one of them.
pub(crate) fn machine_at(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
) -> Option<(BlockEntityRef, &'static str, usize)> {
    let Some(level) = level_of(host, source) else {
        say(host, "evilcraft: no level is open here.".to_owned());
        return None;
    };
    let machine = level
        .block(host, pos)
        .and_then(|key| match key.id(host).as_str() {
            CHEST_ID => Some(("Blood Chest", SLOT_COUNT)),
            crate::infuser::INFUSER_ID => Some(("Blood Infuser", crate::runtime::INFUSER_SLOTS)),
            crate::runtime::COLOSSAL => {
                Some(("Colossal Blood Chest", crate::runtime::COLOSSAL_INVENTORY))
            }
            _ => None,
        });
    match (machine, host.block_entity(level, pos)) {
        (Some((noun, slots)), Some(entity)) if entity.data(host).is_some() => {
            Some((entity, noun, slots))
        }
        _ => {
            say(host, format!("{pos:?} is not a machine of this mod."));
            None
        }
    }
}

/// `/evilcraft held [<amount>]`: read — or, for an operator, set — the held extractor's
/// blood, so a scenario can make a full one without killing 7 cows.
fn held(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    let CommandSource::Player { player, .. } = source else {
        say(host, "a player must run /evilcraft held.".to_owned());
        return;
    };
    let Some(slot) = player.selected_slot(host) else {
        say(host, "a player must run /evilcraft held.".to_owned());
        return;
    };
    let slot = u32::try_from(slot).unwrap_or(0);
    match integer(invocation, "amount") {
        Some(amount) => {
            let amount = amount.clamp(0, EXTRACTOR_CAPACITY);
            let place = ItemPlace::PlayerSlot {
                player: player.uuid(),
                slot: i32::try_from(slot).unwrap_or(0),
            };
            let Some(stack) = place.stack(host) else {
                say(host, "Hold a Blood Extractor first.".to_owned());
                return;
            };
            if stack.item().id(host) != EXTRACTOR {
                say(host, "Hold a Blood Extractor first.".to_owned());
                return;
            }
            let filled = with_extractor_blood(&stack, amount);
            match player.set_inventory_slot(host, slot, Some(filled)) {
                Ok(()) => {
                    let time = game_time(host, *player);
                    say(
                        host,
                        format!(
                            "Blood Extractor held: {amount}/{EXTRACTOR_CAPACITY} mB, t={time}."
                        ),
                    );
                }
                Err(_) => say(host, "the held stack could not be written.".to_owned()),
            }
        }
        None => {
            let place = ItemPlace::PlayerSlot {
                player: player.uuid(),
                slot: i32::try_from(slot).unwrap_or(0),
            };
            let Some(stack) = place.stack(host) else {
                say(host, "Hold a Blood Extractor first.".to_owned());
                return;
            };
            if stack.item().id(host) != EXTRACTOR {
                say(host, "Hold a Blood Extractor first.".to_owned());
                return;
            }
            let blood = extractor_blood(&stack);
            let time = game_time(host, *player);
            say(
                host,
                format!("Blood Extractor held: {blood}/{EXTRACTOR_CAPACITY} mB, t={time}."),
            );
        }
    }
}

/// The player's level's game time, so a `/evilcraft held` reply can never equal the
/// previous one: the harness reads the newest chat line and a fixed wait once read a stale
/// readout (the infuser's `t=` exists for the same reason).
fn game_time(host: &mut dyn ServerHost, player: PlayerRef) -> i64 {
    player
        .level(host)
        .and_then(|level| level.game_time(host))
        .unwrap_or(0)
}

/// The mod block entity at `pos` in the command source's level, or a line saying it is
/// not one.
fn mod_entity(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
) -> Option<BlockEntityRef> {
    our_entity(host, source, pos, CHEST_ID, "Block Chest")
}

/// The mod block entity `block` stands at `pos`, or a line saying the position is not
/// one of `noun`'s.
///
/// The diagnostic reads a machine's slots and tank, so it answers only for this mod's own
/// blocks: any other block entity's saved data (another mod's machine, a vanilla chest
/// with a name) is not this port's to report.
pub(crate) fn our_entity(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    block: &str,
    noun: &str,
) -> Option<BlockEntityRef> {
    let Some(level) = level_of(host, source) else {
        say(host, "evilcraft: no level is open here.".to_owned());
        return None;
    };
    let ours = level
        .block(host, pos)
        .is_some_and(|key| key.id(host) == block);
    match host.block_entity(level, pos) {
        Some(entity) if ours && entity.data(host).is_some() => Some(entity),
        _ => {
            say(host, format!("{pos:?} is not a {noun}."));
            None
        }
    }
}

/// The mod block at `pos` that holds a blood tank — a Blood Chest, a Blood Infuser, a
/// Colossal Blood Chest, a Blood Stain or a Dark Tank — with the capacity of its own tank,
/// or a line saying it holds none.
///
/// They share the saved `fluids` shape and the `/evilcraft blood` setter, but not their
/// size: a stain is upstream's five-bucket splash, the three machines ten, the dark tank
/// sixteen.
fn tanked(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
) -> Option<(BlockEntityRef, i32)> {
    let Some(level) = level_of(host, source) else {
        say(host, "evilcraft: no level is open here.".to_owned());
        return None;
    };
    let capacity = level
        .block(host, pos)
        .and_then(|key| match key.id(host).as_str() {
            CHEST_ID | crate::infuser::INFUSER_ID | crate::runtime::COLOSSAL => Some(TANK_CAPACITY),
            crate::runtime::STAIN => Some(crate::runtime::STAIN_TANK),
            crate::runtime::DARK_TANK => Some(crate::runtime::DARK_TANK_CAPACITY),
            _ => None,
        });
    match (capacity, host.block_entity(level, pos)) {
        (Some(capacity), Some(entity)) if entity.data(host).is_some() => Some((entity, capacity)),
        _ => {
            say(host, format!("{pos:?} has no blood tank."));
            None
        }
    }
}

/// The level a command acts in: the runner's own, else the overworld (the console's
/// execution level in vanilla). The commands name a position, never a dimension, as
/// the harness ran them before the merge.
pub(crate) fn level_of(host: &mut dyn ServerHost, source: &CommandSource) -> Option<LevelRef> {
    match source {
        CommandSource::Player { player, .. } => player.level(host),
        _ => host.level("minecraft:overworld"),
    }
}

/// The level `/evilcraft info` is registered at: 2, the game master level `/data get block`
/// needs.
pub(crate) const OPERATOR_LEVEL: u8 = 2;

/// Whether a command source may read a chest's contents: an operator, or the console (which
/// always carries level 4).
pub(crate) fn may_read(permission: mod_api::PermissionLevel) -> bool {
    permission.level() >= OPERATOR_LEVEL
}

/// Vanilla's inventory slot of the off hand: the slot number
/// `PlayerRef::set_inventory_slot` takes for a click made with it.
pub(crate) const OFF_HAND_SLOT: u32 = 40;

/// The inventory slot the click's hand holds: the selected hotbar slot for the main hand,
/// vanilla's off-hand slot for the other. The stain's draw and the dark tank's fill both
/// write the clicked stack back through it.
pub(crate) fn hand_slot(host: &dyn Host, player: PlayerRef, hand: Hand) -> Option<u32> {
    match hand {
        Hand::Main => player
            .selected_slot(host)
            .and_then(|slot| u32::try_from(slot).ok()),
        Hand::Off => Some(OFF_HAND_SLOT),
    }
}

/// The `pos` argument of a command, resolved by the dispatcher against the source.
pub(crate) fn block_pos(invocation: &CommandInvocation) -> Option<[i32; 3]> {
    match invocation.argument("pos")? {
        CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

/// The `amount`-style integer argument, absent when the usage left it out.
pub(crate) fn integer(invocation: &CommandInvocation, name: &str) -> Option<i32> {
    match invocation.argument(name)? {
        CommandArgumentValue::Integer(value) => Some(*value),
        _ => None,
    }
}

/// A command's answer, routed to the source with vanilla's silence rules.
pub(crate) fn say(host: &mut dyn ServerHost, line: String) {
    let _ = host.commands().feedback(Text::literal(line));
}

// ---------------------------------------------------------------------------
// The extractor's stack data
// ---------------------------------------------------------------------------

/// `minecraft:custom_data`, as a component the stack's patch is read from.
struct CustomData;

impl Component for CustomData {
    type Value = Compound;

    fn id(&self) -> &str {
        CUSTOM_DATA
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

/// The blood one Blood Extractor holds, from its `minecraft:custom_data`.
pub(crate) fn extractor_blood(stack: &ItemStack) -> i32 {
    let Some(data) = stack.get(&CustomData) else {
        return 0;
    };
    match Tag::Compound(data).get(BLOOD_KEY) {
        Some(Tag::Int(blood)) => (*blood).clamp(0, EXTRACTOR_CAPACITY),
        _ => 0,
    }
}

/// The same stack holding `amount` mB of blood; an empty extractor carries no data, so
/// it stacks with a plain one.
pub(crate) fn with_extractor_blood(stack: &ItemStack, amount: i32) -> ItemStack {
    let mut one = stack.clone();
    if amount > 0 {
        let mut data = one.get(&CustomData).unwrap_or_default();
        data.insert(
            BLOOD_KEY.into(),
            Tag::Int(amount.clamp(0, EXTRACTOR_CAPACITY)),
        );
        let _ = one.set(&CustomData, data);
    } else {
        one.remove(&CustomData);
    }
    one
}

/// A stack's damage: a whole one carries no damage component at all, so it stacks
/// with a fresh one, as the retired port's `tidy()` gave.
fn write_damage(stack: &mut ItemStack, damage: u32) {
    if damage == 0 {
        stack.clear_damage();
    } else {
        stack.set_damage(damage);
    }
}

/// The item `id` as a fresh stack of one, for an anvil step that changes the item.
fn item_stack(host: &dyn Host, id: &str) -> Option<ItemStack> {
    let identifier = Identifier::parse(id).ok()?;
    host.item_key(&identifier).map(ItemStack::of)
}

// ---------------------------------------------------------------------------
// Saved-compound reads
// ---------------------------------------------------------------------------

/// The tag saved under `name`, by the plain name the game's own readers use.
pub(crate) fn field<'a>(nbt: &'a Compound, name: &str) -> Option<&'a Tag> {
    nbt.iter()
        .find(|(key, _)| key.to_string_lossy() == name)
        .map(|(_, tag)| tag)
}

pub(crate) fn read_int(nbt: &Compound, name: &str) -> Option<i32> {
    match field(nbt, name) {
        Some(Tag::Int(value)) => Some(*value),
        _ => None,
    }
}

pub(crate) fn write_int(nbt: &mut Compound, name: &str, value: i32) {
    nbt.insert(name.into(), Tag::Int(value));
}

/// The blood in the entity's first tank, in the pin's saved `fluids` shape, exactly as the
/// compound states it: the amount is **not** clamped, so a payload an item was given
/// (`/give … Amount:20000`) can be told apart from a tank the machine filled itself.
/// A tank holding another fluid, or nothing, reads as empty.
pub(crate) fn read_tank_raw(nbt: &Compound) -> Option<i32> {
    let Some(Tag::List(list)) = field(nbt, FLUIDS) else {
        return None;
    };
    let Some(Tag::Compound(entry)) = list.first() else {
        return None;
    };
    let ours = matches!(
        field(entry, "Fluid"),
        Some(Tag::String(fluid)) if fluid.to_string_lossy() == BLOOD
    );
    if !ours {
        return None;
    }
    match field(entry, "Amount") {
        Some(Tag::Int(amount)) => Some(*amount),
        _ => None,
    }
}

/// The blood in the entity's first tank, clamped to the capacity the machine holds.
pub(crate) fn read_tank(nbt: &Compound) -> i32 {
    read_tank_with(nbt, TANK_CAPACITY)
}

/// The blood in the entity's first tank, clamped to `capacity`: the two machines hold ten
/// buckets, the Blood Stain five.
pub(crate) fn read_tank_with(nbt: &Compound, capacity: i32) -> i32 {
    read_tank_raw(nbt).unwrap_or(0).clamp(0, capacity)
}

/// Write the first tank back in the pin's `fluids` shape, an empty tank as
/// `minecraft:empty` so the tank's index stays in place.
pub(crate) fn write_tank(nbt: &mut Compound, amount: i32) {
    write_tank_with(nbt, amount, TANK_CAPACITY);
}

/// Write the first tank back in the pin's `fluids` shape, clamped to `capacity`, an empty
/// tank as `minecraft:empty` so the tank's index stays in place.
pub(crate) fn write_tank_with(nbt: &mut Compound, amount: i32, capacity: i32) {
    let amount = amount.clamp(0, capacity);
    let mut entry = Compound::new();
    entry.insert(
        "Fluid".into(),
        Tag::String(if amount > 0 { BLOOD } else { "minecraft:empty" }.into()),
    );
    entry.insert("Amount".into(), Tag::Int(amount));
    nbt.insert(FLUIDS.into(), Tag::List(vec![Tag::Compound(entry)]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_readout_needs_an_operator_and_names_only_this_chest() {
        use mod_api::PermissionLevel;
        assert!(!may_read(PermissionLevel::All));
        assert!(!may_read(PermissionLevel::Moderators));
        assert!(may_read(PermissionLevel::Gamemasters));
        assert!(may_read(PermissionLevel::Owners));
        assert_eq!(OPERATOR_LEVEL, 2);
        // The block the diagnostic answers for is the full id its own block registers.
        assert_eq!(CHEST_ID, format!("evilcraft:{CHEST_BLOCK}"));
    }
}
