// Game-side glue for EvilCraft's blood repair loop (injected into `crate::game` as
// `evilcraft_state`; it needs the game, so it is not part of the standalone library).
//
// Upstream's loop, carried here: `ItemBloodExtractor` fills every extractor a player
// carries when the player kills a mob (`floor(hp x 5) .. floor(hp x 40)` mB), and a
// `BlockEntityBloodChest` repairs the ten stacks in its chest slots from a 10-bucket
// tank, one damage point every two ticks for 5 mB, anvils in 125 mB steps. The pure
// arithmetic is in `runtime.rs`; this file reads and writes the game.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer, ModFluidTanks};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_core::random::RandomSource;
use horizon_world::nbt::{Compound, JavaString, Tag};

use crate::game::Game;
use crate::game::evilcraft::{
    BLOOD, CHEST, CHEST_SLOTS, CONTAINER_SLOT, EXTRACTOR, EXTRACTOR_CAPACITY, SLOT_COUNT,
    TANK_CAPACITY, anvil_step, blood_from_kill, can_hold, drain_container, fill_extractor,
    kill_span, repair, repair_due,
};
use crate::game::native_hooks::NativeScreen;
use crate::game::native_server::{NativeCommand, NativeServerPlayer};

/// The menu type the content mod declares (`data/evilcraft/horizon/menu/`).
pub const MENU: &str = "evilcraft:blood_chest_menu";
/// Where the extractor keeps its blood: an int under the stack's
/// `minecraft:custom_data`. The pin's item fluid helpers carry only water and lava
/// (API 147, `native_item_fluids.rs`), so a mod fluid lives in the stack's own data.
const CUSTOM_DATA: &str = "minecraft:custom_data";
const BLOOD_KEY: &str = "evilcraft:blood";

/// What the kills of this session left for the extractors.
#[derive(Default)]
struct Kills {
    pending: HashMap<u128, i32>,
    kills: u32,
}

fn kills() -> &'static Mutex<Kills> {
    static KILLS: OnceLock<Mutex<Kills>> = OnceLock::new();
    KILLS.get_or_init(|| Mutex::new(Kills::default()))
}

// ---------------------------------------------------------------------------
// The block entity: slots, tank, tick
// ---------------------------------------------------------------------------

/// The chest's eleven slots and its menu (`native_block_entity_container`, API 16).
pub fn on_container(block: &str, container: &mut Option<ModContainer>) {
    if block != CHEST {
        return;
    }
    *container = Some(ModContainer {
        slots: SLOT_COUNT,
        faces: std::array::from_fn(|_| Vec::new()),
        inputs: Vec::new(),
        outputs: Vec::new(),
        menu: Some(MENU.to_owned()),
    });
}

/// One 10-bucket tank, every face reaching it, so a neighbouring pipe can take the
/// blood out (`native_block_entity_fluids`, API 54).
pub fn on_fluids(block: &str, fluids: &mut Option<ModFluidTanks>) {
    if block != CHEST {
        return;
    }
    let all = vec![0usize];
    *fluids = Some(ModFluidTanks {
        tanks: vec![TANK_CAPACITY],
        faces: [
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all,
        ],
        inputs: Vec::new(),
        outputs: vec![0],
    });
}

/// One server tick of the chest (`native_block_entity_tick`, API 15).
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let _ = (game, pos);
    if entity.id != CHEST {
        return;
    }
    if entity.items.len() < SLOT_COUNT {
        entity.items.resize(SLOT_COUNT, None);
    }
    let mut tank = read_tank(&entity.nbt);
    // The container slot empties a blood container into the tank, as upstream's
    // `EmptyFluidContainerInTankTickAction` does.
    if let Some(container) = entity.items[CONTAINER_SLOT].take() {
        let (filled, keeps) = drain_container(tank, extractor_blood(&container));
        tank = filled;
        entity.items[CONTAINER_SLOT] = (container.count > 0).then(|| with_extractor_blood(&container, keeps));
    }
    let ticks = read_int(&entity.nbt, "ticks").unwrap_or(0).max(0) as u32;
    if repair_due(ticks) {
        for slot in 0..CHEST_SLOTS {
            let Some(mut stack) = entity.items[slot].take() else {
                continue;
            };
            if can_hold(&stack.item, damage_of(&stack)) {
                // Upstream's `RepairItemTickAction`: the action's multiplier scales
                // `mBPerDamage`, and the tank then pays `mBPerDamage x multiplier`.
                if damage_of(&stack) > 0 {
                    if let Some((damage, left)) = repair(damage_of(&stack), tank) {
                        stack.damage = u32::try_from(damage).unwrap_or(0);
                        stack.tidy();
                        tank = left;
                    }
                } else if let Some((next, left)) = anvil_step(&stack.item, tank) {
                    stack = ItemStack::new(next, 1);
                    tank = left;
                }
            }
            entity.items[slot] = Some(stack);
        }
    }
    write_tank(&mut entity.nbt, tank);
    write_int(&mut entity.nbt, "ticks", ticks.wrapping_add(1) as i32);
    entity.data = vec![tank, TANK_CAPACITY];
}

// ---------------------------------------------------------------------------
// The extractor: kill blood
// ---------------------------------------------------------------------------

/// The player's blow killed mob `entity` (`native_mob_killed`, API 40): the kill is
/// worth `floor(hp x 5) .. floor(hp x 40)` mB to the extractors that player carries.
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_mob_killed(game: &mut Game<'_>, player: u128, entity: u32) {
    // The mob is already at 0 health here, so `native_mob` (which filters the living)
    // gives nothing; its attributes stay in the game's own mob list.
    let health = game
        .mobs
        .iter()
        .find(|mob| mob.id == entity && !mob.removed)
        .map(|mob| mob.attrs.max_health);
    let Some(health) = health else {
        return;
    };
    let span = kill_span(health).max(1);
    let roll = u32::try_from(game.world.random.next_int_bound(span as i32).unwrap_or(0)).unwrap_or(0);
    let blood = blood_from_kill(health, roll);
    let mut cache = kills().lock().expect("evilcraft: the kill cache lock");
    cache.kills += 1;
    let pending = cache.pending.entry(player).or_insert(0);
    *pending = pending.saturating_add(blood);
}

/// A carried stack ticks (`native_inventory_tick`, API 39): a Blood Extractor takes
/// the blood the player's kills left, as `fillForAllBloodExtractors` does.
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_inventory_tick(game: &mut Game<'_>, slot: i32, selected: bool) {
    let _ = selected;
    // The acting player's UUID, as `native_mob_killed` hands it over: `native_acting_player`
    // is None for a single-player host (there is no LAN guest record), but the id exists.
    let player = game.native_acting_id();
    let pending = kills()
        .lock()
        .expect("evilcraft: the kill cache lock")
        .pending
        .get(&player)
        .copied()
        .unwrap_or(0);
    if pending <= 0 {
        return;
    }
    let Some(stack) = game.native_carried_stack(slot) else {
        return;
    };
    if &*stack.item != EXTRACTOR {
        return;
    }
    let held = extractor_blood(stack);
    let (now, left) = fill_extractor(held, pending);
    if now == held {
        return;
    }
    *stack = with_extractor_blood(stack, now);
    let mut cache = kills().lock().expect("evilcraft: the kill cache lock");
    cache.pending.insert(player, left);
}

// ---------------------------------------------------------------------------
// The screen
// ---------------------------------------------------------------------------

/// The chest's eleven-slot layout (`native_screen`, API 19): ten repair slots in two
/// rows of five, the container slot beside them, the player's inventory under.
pub fn on_screen(block: &str, screen: &mut Option<NativeScreen>) {
    if block != CHEST {
        return;
    }
    let mut slots = Vec::with_capacity(SLOT_COUNT);
    for row in 0..2 {
        for column in 0..5 {
            slots.push(Some([8.0 + 18.0 * column as f32, 18.0 + 18.0 * row as f32]));
        }
    }
    slots.push(Some([116.0, 27.0]));
    *screen = Some(NativeScreen {
        width: 176.0,
        height: 166.0,
        slots,
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
    });
}

/// The chest's tank readout (`native_screen_draw`, API 19): the menu's data slots are
/// `[tank, capacity]`.
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    if block != CHEST {
        return;
    }
    let tank = data.first().copied().unwrap_or(0).max(0);
    let text = format!("{tank} / {TANK_CAPACITY} mB");
    game.native_text(
        batch,
        &text,
        [origin[0] + 8.0, origin[1] + 58.0],
        0x404040,
        false,
    );
}

/// The extractor's tooltip (`native_tooltip`, API 12): the blood it holds.
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_tooltip(_game: &Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if &*stack.item != EXTRACTOR {
        return;
    }
    let blood = extractor_blood(stack);
    lines.push((
        Text::literal(format!("Blood: {blood}/{EXTRACTOR_CAPACITY} mB")),
        0x00A0A0,
    ));
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// The commands (`native_register_commands`, API 8). Upstream has none: they exist so
/// the harness can read and set a chest without a GUI (precedent:
/// `/charginggadgets info|energy|item`, `/powah info|set`).
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("evilcraft info <pos:block_pos>"));
    commands.push(NativeCommand::op(
        "evilcraft blood <pos:block_pos> <amount:integer>",
        2,
    ));
    commands.push(NativeCommand::op(
        "evilcraft damage <pos:block_pos> <slot:integer> <amount:integer>",
        2,
    ));
    commands.push(NativeCommand::new("evilcraft held"));
    commands.push(NativeCommand::op("evilcraft held <amount:integer>", 2));
}

/// `/evilcraft info|blood|damage|held` (`native_perform_command`, API 8).
#[expect(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_perform_command(
    game: &mut Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() != Some(&"evilcraft") {
        return;
    }
    *ran = true;
    match words.get(1).copied() {
        Some("info") => {
            let Some(pos) = parse_pos(&words[2..]) else {
                game.native_feedback(Text::literal("usage: /evilcraft info <x> <y> <z>"));
                return;
            };
            game.native_feedback(Text::literal(readout(game, pos)));
        }
        Some("blood") => {
            let Some((pos, amount)) = parse_pos_amount(&words[2..]) else {
                game.native_feedback(Text::literal(
                    "usage: /evilcraft blood <x> <y> <z> <amount>",
                ));
                return;
            };
            let amount = amount.clamp(0, TANK_CAPACITY);
            let changed = game
                .native_with_mod_block_entity(pos, |entity| {
                    if entity.id != CHEST {
                        return false;
                    }
                    write_tank(&mut entity.nbt, amount);
                    entity.data = vec![amount, TANK_CAPACITY];
                    true
                })
                .unwrap_or(false);
            game.native_feedback(Text::literal(if changed {
                format!("Blood Chest at {pos:?}: {amount}/{TANK_CAPACITY} mB.")
            } else {
                format!("{pos:?} is not a blood chest.")
            }));
        }
        Some("damage") => {
            let Some(pos) = parse_pos(&words[2..]) else {
                game.native_feedback(Text::literal(
                    "usage: /evilcraft damage <x> <y> <z> <slot> <amount>",
                ));
                return;
            };
            let (Some(slot), Some(amount)) = (
                words.get(5).and_then(|word| word.parse::<usize>().ok()),
                words.get(6).and_then(|word| word.parse::<u32>().ok()),
            ) else {
                game.native_feedback(Text::literal(
                    "usage: /evilcraft damage <x> <y> <z> <slot> <amount>",
                ));
                return;
            };
            let changed = game
                .native_with_mod_block_entity(pos, |entity| {
                    if entity.id != CHEST {
                        return None;
                    }
                    let stack = entity.items.get_mut(slot)?.as_mut()?;
                    stack.damage = amount;
                    stack.tidy();
                    Some(format!("{} at damage {amount}", stack.item))
                })
                .flatten();
            game.native_feedback(Text::literal(match changed {
                Some(item) => format!("Blood Chest at {pos:?}: {item}."),
                None => format!("{pos:?} has no such stack."),
            }));
        }
        Some("held") => match words.get(2).and_then(|word| word.parse::<i32>().ok()) {
            Some(amount) => {
                let amount = amount.clamp(0, EXTRACTOR_CAPACITY);
                match set_held_blood(game, amount) {
                    Some(blood) => game.native_feedback(Text::literal(format!(
                        "Blood Extractor held: {blood}/{EXTRACTOR_CAPACITY} mB."
                    ))),
                    None => game.native_feedback(Text::literal("Hold a Blood Extractor first.")),
                }
            }
            None => {
                let blood = game.held().map_or(-1, |stack| {
                    if &*stack.item == EXTRACTOR {
                        extractor_blood(stack)
                    } else {
                        -1
                    }
                });
                let (pending, kill_count) = {
                    let cache = kills().lock().expect("evilcraft: the kill cache lock");
                    (
                        cache.pending.get(&game.native_acting_id()).copied().unwrap_or(0),
                        cache.kills,
                    )
                };
                game.native_feedback(Text::literal(if blood < 0 {
                    "Hold a Blood Extractor first.".to_owned()
                } else {
                    format!(
                        "Blood Extractor held: {blood}/{EXTRACTOR_CAPACITY} mB (pending {pending} mB, {kill_count} kills)."
                    )
                }));
            }
        },
        _ => game.native_feedback(Text::literal(
            "usage: /evilcraft info|blood|damage|held ...",
        )),
    }
}

/// The chest's state, as `/evilcraft info` prints it: the tank, the container slot
/// and every chest slot with its damage.
fn readout(game: &Game<'_>, pos: [i32; 3]) -> String {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return format!("{pos:?} is not a mod block entity.");
    };
    if entity.id != CHEST {
        return format!("{pos:?} is a {} not a blood chest.", entity.id);
    }
    let tank = read_tank(&entity.nbt);
    let container = match entity.items.get(CONTAINER_SLOT).and_then(Option::as_ref) {
        Some(stack) => format!("{}@{}", stack.item, extractor_blood(stack)),
        None => "empty@0".to_owned(),
    };
    let mut slots = String::new();
    for (index, stack) in entity.items.iter().take(CHEST_SLOTS).enumerate() {
        let Some(stack) = stack else {
            continue;
        };
        slots.push_str(&format!("{index}:{}@{} ", stack.item, stack.damage));
    }
    format!(
        "Blood Chest {pos:?}: tank {tank}/{TANK_CAPACITY} mB, container {container}, slots [{}], t={}",
        slots.trim_end(),
        game.world.game_time
    )
}

/// Set the held extractor's blood (the harness's way to make a filled one, as
/// `/charginggadgets held` does for FE).
fn set_held_blood(game: &mut Game<'_>, amount: i32) -> Option<i32> {
    let slot = game.held_slot();
    let stack = game.stack_in_mut(slot).take()?;
    if &*stack.item != EXTRACTOR {
        *game.stack_in_mut(slot) = Some(stack);
        return None;
    }
    *game.stack_in_mut(slot) = Some(with_extractor_blood(&stack, amount));
    game.hand_stack_replaced(slot);
    Some(amount)
}

fn parse_pos(words: &[&str]) -> Option<[i32; 3]> {
    Some([
        words.first()?.parse().ok()?,
        words.get(1)?.parse().ok()?,
        words.get(2)?.parse().ok()?,
    ])
}

fn parse_pos_amount(words: &[&str]) -> Option<([i32; 3], i32)> {
    let pos = parse_pos(words)?;
    let amount = words.get(3)?.parse().ok()?;
    Some((pos, amount))
}

// ---------------------------------------------------------------------------
// The extractor's stack data
// ---------------------------------------------------------------------------

/// The stack's damage as the runtime reads it.
fn damage_of(stack: &ItemStack) -> i32 {
    i32::try_from(stack.damage).unwrap_or(i32::MAX)
}

/// The blood one Blood Extractor holds, from its `minecraft:custom_data`.
fn extractor_blood(stack: &ItemStack) -> i32 {
    if &*stack.item != EXTRACTOR {
        return 0;
    }
    let Some(Tag::Compound(data)) = stack.data().and_then(|data| data.other(CUSTOM_DATA)) else {
        return 0;
    };
    match data.get(&JavaString::from(BLOOD_KEY)) {
        Some(Tag::Int(blood)) => (*blood).clamp(0, EXTRACTOR_CAPACITY),
        _ => 0,
    }
}

/// The same stack holding `amount` mB of blood; an empty extractor carries no data, so
/// it stacks with a plain one.
fn with_extractor_blood(stack: &ItemStack, amount: i32) -> ItemStack {
    let mut one = stack.clone();
    let mut data = match one.data().and_then(|data| data.other(CUSTOM_DATA)) {
        Some(Tag::Compound(data)) => data.clone(),
        _ => Compound::new(),
    };
    if amount > 0 {
        data.insert(
            JavaString::from(BLOOD_KEY),
            Tag::Int(amount.clamp(0, EXTRACTOR_CAPACITY)),
        );
    } else {
        data.remove(&JavaString::from(BLOOD_KEY));
    }
    let value = (!data.is_empty()).then_some(Tag::Compound(data));
    one.data_mut().set_other(CUSTOM_DATA, value);
    // What a server sent for the stack no longer describes it; kept, an emptied
    // extractor would not stack with a plain one.
    one.data_mut().wire = Default::default();
    one.tidy();
    one
}

// ---------------------------------------------------------------------------
// Saved-compound reads
// ---------------------------------------------------------------------------

fn key(name: &str) -> JavaString {
    JavaString::from(name)
}

fn read_int(nbt: &Compound, name: &str) -> Option<i32> {
    match nbt.get(&key(name)) {
        Some(Tag::Int(value)) => Some(*value),
        _ => None,
    }
}

fn write_int(nbt: &mut Compound, name: &str, value: i32) {
    nbt.insert(key(name), Tag::Int(value));
}

/// The blood in the entity's first tank, in the pin's saved `fluids` shape. A tank
/// holding another fluid, or nothing, reads as empty.
fn read_tank(nbt: &Compound) -> i32 {
    let Some(Tag::List(list)) = nbt.get(&key("fluids")) else {
        return 0;
    };
    let Some(Tag::Compound(entry)) = list.first() else {
        return 0;
    };
    let ours = matches!(
        entry.get(&key("Fluid")),
        Some(Tag::String(fluid)) if fluid.to_string_lossy() == BLOOD
    );
    if !ours {
        return 0;
    }
    match entry.get(&key("Amount")) {
        Some(Tag::Int(amount)) => (*amount).clamp(0, TANK_CAPACITY),
        _ => 0,
    }
}

/// Write the first tank back in the pin's `fluids` shape.
fn write_tank(nbt: &mut Compound, amount: i32) {
    let amount = amount.clamp(0, TANK_CAPACITY);
    let mut entry = Compound::new();
    entry.insert(
        key("Fluid"),
        Tag::String(JavaString::from(if amount > 0 {
            BLOOD
        } else {
            "minecraft:empty"
        })),
    );
    entry.insert(key("Amount"), Tag::Int(amount));
    nbt.insert(key("fluids"), Tag::List(vec![Tag::Compound(entry)]));
}
