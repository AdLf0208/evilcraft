//! The game side of EvilCraft's Blood Infuser (`BlockEntityBloodInfuser`): the recipe
//! files it carries, their decoding, and the block entity Part that runs them.
//!
//! Upstream's machine has three slots (a container drainer, the infusion slot and the
//! result slot) and four upgrade slots, a ten-bucket blood tank, and its own recipe type
//! (`evilcraft:blood_infuser`), whose `matches` compares the recipe's `tier` with the
//! machine's. The rules are [`crate::runtime`]; this file reads and writes the game.
//!
//! The recipes are the mod's own data files, read back through
//! [`Host::recipes_of_type`](mod_api::Host::recipes_of_type) with the recipe type the
//! mod registers, so a data pack can add, override or remove an infusion as upstream's
//! `RecipeManager` allows.

use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTicksEvent, BlockPart, ClientHost, CommandArgumentValue, CommandInvocation,
    CommandSource, Compound, Host, HostTransaction, Identifier, ItemStack, LevelSide, ModContainer,
    ModFluidTanks, Screen, ScreenDrawEvent, ScreenEvent, ServerHost,
};

use crate::runtime::{
    BLOOD, INFUSER_INPUT_SLOT, INFUSER_OUTPUT_SLOT, INFUSER_SLOTS, INFUSER_TANK, Infusion,
    infuse_tick, infusion_for,
};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const INFUSER_ENTITY: &str = "blood_infuser";
/// The block the type belongs to, as the declaration builder takes it.
pub const INFUSER_BLOCK: &str = "blood_infuser";
/// The block's full id, as `Level::block` names it.
pub const INFUSER_ID: &str = "evilcraft:blood_infuser";
/// The menu type the content declares (`data/evilcraft/horizon/menu/blood_infuser_menu.json`).
const MENU: &str = "evilcraft:blood_infuser_menu";
/// The block entity's saved infusion progress, in ticks.
const PROGRESS: &str = "progress";

/// The recipe files this package carries, by the id their path gives them
/// (`data/evilcraft/recipe/blood_infuser/<name>.json` reads as
/// `evilcraft:blood_infuser/<name>`), with the file's own bytes.
///
/// They are the jar's `data/evilcraft/recipe/blood_infuser/convenience/` files whose
/// ingredient and product are both items this package or the game has; the rest of the
/// jar's 40 need items this slice does not carry (see the gap list in `notes.md`).
pub const CARRIED: &[(&str, &str)] = &[
    // `base/blook.json` is the jar's own path (the `base/` recipes are the machine's
    // additions; the nine below are `convenience/` float, flattened when they were carried).
    (
        "evilcraft:blood_infuser/base/blook",
        include_str!("../data/evilcraft/recipe/blood_infuser/base/blook.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_blaze_rod",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_blaze_rod.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_creeper_head",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_creeper_head.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_grass_block",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_grass_block.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_nether_wart",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_nether_wart.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_netherrack",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_netherrack.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_red_mushroom",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_red_mushroom.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_soul_sand",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_soul_sand.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_spider_eye",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_spider_eye.json"),
    ),
    (
        "evilcraft:blood_infuser/minecraft_zombie_head",
        include_str!("../data/evilcraft/recipe/blood_infuser/minecraft_zombie_head.json"),
    ),
];

// ---------------------------------------------------------------------------
// The recipe files
// ---------------------------------------------------------------------------

/// One recipe file, decoded into the rules it states
/// (`RecipeSerializerBloodInfuser`'s codec: `input_item`, `input_fluid.amount`,
/// `output_item` with its `count`, `duration` and the optional `tier`).
pub fn decode_infusion(id: &str, json: &str) -> Result<Infusion, String> {
    use mod_api::serde_json::Value;

    let value: Value = mod_api::serde_json::from_str(json)
        .map_err(|error| format!("{id} is not JSON: {error}"))?;
    decode_value(id, &value)
}

fn decode_value(id: &str, value: &mod_api::serde_json::Value) -> Result<Infusion, String> {
    use mod_api::serde_json::Value;

    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if kind != crate::runtime::INFUSION_TYPE {
        return Err(format!(
            "{id} is {kind}, not a {} recipe",
            crate::runtime::INFUSION_TYPE
        ));
    }
    // `input_item` is an ingredient; every carried file names one item.
    let input = value
        .get("input_item")
        .and_then(|item| item.get("item"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{id} names no input item"))?;
    // The fluid's id is the mod's own blood; a file naming another fluid is not this
    // machine's (Horizon has no blood world fluid, so the tank is a tank id).
    let fluid = value.get("input_fluid").unwrap_or(&Value::Null);
    let fluid_id = fluid.get("id").and_then(Value::as_str).unwrap_or_default();
    if fluid_id != BLOOD {
        return Err(format!("{id} infuses with {fluid_id}, not {BLOOD}"));
    }
    let blood = fluid
        .get("amount")
        .and_then(Value::as_i64)
        .and_then(|amount| i32::try_from(amount).ok())
        .ok_or_else(|| format!("{id} names no blood amount"))?;
    let output = value
        .get("output_item")
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{id} names no output item"))?;
    let output_count = value
        .get("output_item")
        .and_then(|item| item.get("count"))
        .and_then(Value::as_u64)
        .and_then(|count| u32::try_from(count).ok())
        .unwrap_or(1);
    let ticks = value
        .get("duration")
        .and_then(Value::as_u64)
        .and_then(|ticks| u32::try_from(ticks).ok())
        .ok_or_else(|| format!("{id} has no duration"))?;
    let tier = value
        .get("tier")
        .and_then(Value::as_u64)
        .and_then(|tier| u32::try_from(tier).ok())
        .unwrap_or(0);
    if blood <= 0 || ticks == 0 || output_count == 0 {
        return Err(format!("{id} states an empty infusion"));
    }
    Ok(Infusion {
        id: id.to_owned(),
        input: input.to_owned(),
        output: output.to_owned(),
        output_count,
        blood,
        ticks,
        tier,
    })
}

/// The infusions in force, decoded. A file the game kept but this machine cannot read
/// is skipped rather than failing the tick: a pack may add its own recipes of this type.
pub fn recipes(host: &dyn Host) -> Vec<Infusion> {
    host.recipes_of_type(crate::runtime::INFUSION_TYPE)
        .iter()
        .filter_map(|recipe| decode_value(&recipe.id, &recipe.json).ok())
        .collect()
}

// ---------------------------------------------------------------------------
// The declarations
// ---------------------------------------------------------------------------

/// The infuser's seven slots and its menu: upstream's three machine slots and four
/// upgrade slots. Faces are upstream's `ItemHandlerSlotMasked`: the top reaches the
/// infusion slot, the bottom the result, every side the container drainer. A pipe can
/// put into the container and infusion slots and take the result out, as upstream's
/// `canPlaceItemThroughFace`/`canTakeItemThroughFace` say; the upgrade slots are the
/// GUI's only.
pub fn container() -> ModContainer {
    let mut container = ModContainer {
        slots: INFUSER_SLOTS,
        menu: Some(MENU.to_owned()),
        ..ModContainer::default()
    };
    // `faces` is by `Direction` number: down, up, north, south, west, east.
    container.faces = [
        vec![INFUSER_OUTPUT_SLOT],
        vec![INFUSER_INPUT_SLOT],
        vec![crate::runtime::INFUSER_CONTAINER_SLOT],
        vec![crate::runtime::INFUSER_CONTAINER_SLOT],
        vec![crate::runtime::INFUSER_CONTAINER_SLOT],
        vec![crate::runtime::INFUSER_CONTAINER_SLOT],
    ];
    container.inputs = vec![crate::runtime::INFUSER_CONTAINER_SLOT, INFUSER_INPUT_SLOT];
    container.outputs = vec![crate::runtime::INFUSER_CONTAINER_SLOT, INFUSER_OUTPUT_SLOT];
    container
}

/// One ten-bucket tank every face reaches, so a pipe can fill the machine with blood
/// and drain it again (`BlockEntityTickingTankInventory`'s fluid capability).
///
/// Built by assignment: `ModFluidTanks` is `#[non_exhaustive]`.
pub fn tanks() -> ModFluidTanks {
    let all = vec![0];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![INFUSER_TANK];
    tanks.faces = [
        all.clone(),
        all.clone(),
        all.clone(),
        all.clone(),
        all.clone(),
        all,
    ];
    tanks.inputs = vec![0];
    tanks.outputs = vec![0];
    tanks
}

// ---------------------------------------------------------------------------
// The Part
// ---------------------------------------------------------------------------

/// The Blood Infuser's block entity (`evilcraft:blood_infuser`): its tick, its screen
/// and its slot count come from the declarations in `Hooks::register`.
pub struct Infuser;

#[mod_api::horizon_part]
impl BlockPart for Infuser {
    fn has_analog_output(
        &self,
        _level: &dyn mod_api::LevelHost,
        event: &mut mod_api::HasAnalogOutputEvent,
    ) {
        event.output = true;
    }

    fn analog_output(
        &self,
        _level: &dyn mod_api::LevelHost,
        event: &mut mod_api::AnalogOutputEvent,
    ) {
        event.output = crate::redstone::cached_reading(
            event.dimension,
            [event.pos.x, event.pos.y, event.pos.z],
        );
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for Infuser {
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        None
    }

    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: empty the container slot into the tank, then advance the
    /// infusion the input slot names, at the tier the upgrade slots set.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        crate::redstone::queue_tick(&*host, event.pos);
        let entity = &mut *event.entity;
        if entity.items.len() < INFUSER_SLOTS {
            return;
        }
        let mut tank = crate::blood::read_tank(&entity.nbt);
        // The container slot empties a blood container into the tank, upstream's
        // `EmptyFluidContainerInTankTickAction` (the same action the chest runs). A stack
        // from before the item was unstackable is left for the player to take out and for
        // their inventory to resolve.
        if let Some(container) = entity.items[crate::runtime::INFUSER_CONTAINER_SLOT].take() {
            let extractor = container.item().id(host) == crate::runtime::EXTRACTOR;
            if extractor && !crate::runtime::one_extractor(container.count()) {
                entity.items[crate::runtime::INFUSER_CONTAINER_SLOT] = Some(container);
            } else {
                let (filled, keeps) = crate::runtime::drain_container(
                    tank,
                    crate::blood::extractor_blood(&container),
                );
                tank = filled;
                entity.items[crate::runtime::INFUSER_CONTAINER_SLOT] = Some(if extractor {
                    crate::blood::with_extractor_blood(&container, keeps)
                } else {
                    container
                });
            }
        }
        // The machine's tier is the promise in its upgrade slots (`ItemPromise`).
        let mut tier = 0;
        for slot in 3..INFUSER_SLOTS {
            if let Some(stack) = &entity.items[slot] {
                tier = tier.max(crate::runtime::machine_tier([stack
                    .item()
                    .id(host)
                    .as_str()]));
            }
        }
        let progress = crate::blood::read_int(&entity.nbt, PROGRESS)
            .unwrap_or(0)
            .max(0) as u32;
        let input = entity.items[INFUSER_INPUT_SLOT]
            .as_ref()
            .map(|stack| stack.item().id(host).to_owned());
        let recipe = input.as_deref().and_then(|item| {
            let recipes = recipes(&*host);
            infusion_for(&recipes, item, tier).cloned()
        });
        let mut next = 0;
        let mut required = 0;
        if let Some(recipe) = &recipe {
            required = recipe.ticks;
            // Upstream's `canTick`: the result slot must take the product, then the
            // tank must pay the recipe's blood. A refusal resets the progress.
            if output_fits(host, entity.items[INFUSER_OUTPUT_SLOT].as_ref(), recipe)
                && let Some((advanced, done)) = infuse_tick(progress, recipe, tank)
            {
                if done {
                    if produce(host, &mut entity.items, recipe) {
                        tank = (tank - recipe.blood).max(0);
                    } else {
                        next = progress;
                    }
                } else {
                    next = advanced;
                }
            }
        }
        crate::blood::write_int(&mut entity.nbt, PROGRESS, next as i32);
        crate::blood::write_tank(&mut entity.nbt, tank);
        // The menu's three data slots: the tank, the progress and its duration.
        let required = i32::try_from(required).unwrap_or(i32::MAX);
        let data = vec![tank, next as i32, required];
        if entity.data != data {
            entity.data = data;
        }
    }

    /// The infuser's layout: the container, infusion and result slots in a row, the four
    /// upgrade slots above them, the player's inventory under.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        let mut slots = Vec::with_capacity(INFUSER_SLOTS);
        slots.push(Some([8.0, 36.0]));
        slots.push(Some([79.0, 36.0]));
        slots.push(Some([133.0, 36.0]));
        for column in 0..crate::runtime::UPGRADE_SLOTS {
            slots.push(Some([8.0 + 18.0 * column as f32, 18.0]));
        }
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

    /// The infuser's readouts: its tank and the infusion's progress.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let tank = event.data.first().copied().unwrap_or(0).max(0);
        let progress = event.data.get(1).copied().unwrap_or(0).max(0);
        let required = event.data.get(2).copied().unwrap_or(0).max(0);
        let salt = format!("{tank} / {INFUSER_TANK} mB");
        let work = format!("{progress} / {required} t");
        event.draw.text(
            &salt,
            [event.origin[0] + 8.0, event.origin[1] + 58.0],
            0xFF40_4040,
            false,
        );
        event.draw.text(
            &work,
            [event.origin[0] + 8.0, event.origin[1] + 68.0],
            0xFF40_4040,
            false,
        );
    }
}

/// Whether the result slot takes the recipe's product: empty, or the same item with
/// room, as upstream's `BloodInfuserTickAction.canTick` says.
fn output_fits(host: &dyn Host, slot: Option<&ItemStack>, recipe: &Infusion) -> bool {
    let Some(stack) = slot else {
        return true;
    };
    if stack.item().id(host) != recipe.output {
        return false;
    }
    let max = host.max_stack_size(stack.item()).unwrap_or(64);
    stack.count().saturating_add(recipe.output_count) <= max
}

/// Put the product in the result slot, take one input and spend the blood. Returns
/// whether the slot took it (it fits: the same test that let the tick run).
fn produce(host: &dyn Host, items: &mut [Option<ItemStack>], recipe: &Infusion) -> bool {
    let Some(key) = Identifier::parse(&recipe.output)
        .ok()
        .and_then(|id| host.item_key(&id))
    else {
        return false;
    };
    let output = match items[INFUSER_OUTPUT_SLOT].as_mut() {
        Some(stack) if stack.item() == key => {
            if stack.grow(recipe.output_count).is_err() {
                return false;
            }
            None
        }
        Some(_) => return false,
        None => Some(match ItemStack::new(key, recipe.output_count) {
            Some(stack) => stack,
            None => return false,
        }),
    };
    if let Some(stack) = output {
        items[INFUSER_OUTPUT_SLOT] = Some(stack);
    }
    // One item leaves the infusion slot.
    if let Some(stack) = items[INFUSER_INPUT_SLOT].take()
        && let Some(left) = stack.shrink(1)
    {
        items[INFUSER_INPUT_SLOT] = Some(left);
    }
    true
}

// ---------------------------------------------------------------------------
// The harness commands
// ---------------------------------------------------------------------------

/// `/evilcraft infuser <pos>`: the tank, the tier, the slots and the progress, so a
/// scenario can read a machine without a GUI (upstream has no such command).
pub fn readout(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !crate::blood::may_read(source.permission()) {
        crate::blood::say(
            host,
            format!(
                "evilcraft infuser needs an operator (permission level {})",
                crate::blood::OPERATOR_LEVEL
            ),
        );
        return;
    }
    let Some(pos) = crate::blood::block_pos(invocation) else {
        crate::blood::say(host, "usage: /evilcraft infuser <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) = entity_at(host, source, pos) else {
        return;
    };
    let Some(data) = entity.data(host) else {
        crate::blood::say(host, format!("{pos:?} is not a Blood Infuser."));
        return;
    };
    let tank = crate::blood::read_tank(&data);
    let progress = crate::blood::read_int(&data, PROGRESS).unwrap_or(0).max(0);
    let mut tier = 0;
    for slot in 3..INFUSER_SLOTS {
        if let Ok(Some(stack)) = entity.item_slot(host, slot as u32) {
            tier = tier.max(crate::runtime::machine_tier([stack
                .item()
                .id(host)
                .as_str()]));
        }
    }
    let container = container_line(host, &entity);
    let input = stack_line(host, &entity, INFUSER_INPUT_SLOT);
    let output = stack_line(host, &entity, INFUSER_OUTPUT_SLOT);
    let in_force = host.recipes_of_type(crate::runtime::INFUSION_TYPE).len();
    let time = entity
        .level(host)
        .and_then(|level| level.game_time(host))
        .unwrap_or(0);
    crate::blood::say(
        host,
        format!(
            "Blood Infuser {pos:?}: tank {tank}/{INFUSER_TANK} mB, tier {tier}, cont {container}, in {input}, out {output}, progress {progress}, recipes {in_force}/{} t={time}",
            CARRIED.len()
        ),
    );
}

/// `item@count`, or `empty@0`.
fn stack_line(host: &mut dyn ServerHost, entity: &BlockEntityRef, slot: usize) -> String {
    match entity.item_slot(host, slot as u32) {
        Ok(Some(stack)) => format!("{}@{}", stack.item().id(host), stack.count()),
        Ok(None) => "empty@0".to_owned(),
        Err(_) => "unknown@0".to_owned(),
    }
}

/// `item@blood`, or `empty@0`: the container slot's stack and the blood it holds.
fn container_line(host: &mut dyn ServerHost, entity: &BlockEntityRef) -> String {
    match entity.item_slot(host, crate::runtime::INFUSER_CONTAINER_SLOT as u32) {
        Ok(Some(stack)) => format!(
            "{}@{}",
            stack.item().id(host),
            crate::blood::extractor_blood(&stack)
        ),
        Ok(None) => "empty@0".to_owned(),
        Err(_) => "unknown@0".to_owned(),
    }
}

/// `/evilcraft load <pos> <slot> <id>`: put one item of `id` in a machine slot, so a
/// scenario does not have to click a slot for every recipe. An unknown id is refused.
pub fn load(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    let usage = "usage: /evilcraft load <x> <y> <z> <slot> <item id>";
    let (Some(pos), Some(slot), Some(item)) = (
        crate::blood::block_pos(invocation),
        crate::blood::integer(invocation, "slot"),
        string(invocation, "id"),
    ) else {
        crate::blood::say(host, usage.to_owned());
        return;
    };
    let Some((entity, noun, slots)) = crate::blood::machine_at(host, source, pos) else {
        return;
    };
    let slot = u32::try_from(slot.max(0)).unwrap_or(0);
    if slot as usize >= slots {
        crate::blood::say(host, format!("{pos:?} has no slot {slot}."));
        return;
    }
    // `empty` clears the slot, so a scenario can take a product out between recipes. The
    // dispatcher namespaces a bare word, so the argument arrives as `minecraft:empty`.
    let stack = if item == "empty" || item == "minecraft:empty" {
        None
    } else {
        let Some(key) = Identifier::parse(item)
            .ok()
            .and_then(|id| host.item_key(&id))
        else {
            crate::blood::say(host, format!("{item} is not an item this game has."));
            return;
        };
        let Some(stack) = ItemStack::new(key, 1) else {
            crate::blood::say(host, format!("{item} is not an item this game has."));
            return;
        };
        Some(stack)
    };
    let written = host
        .transaction(|host, tx| -> Result<bool, ()> {
            Ok(entity.set_item_slot(host, tx, slot, stack).unwrap_or(false))
        })
        .unwrap_or(false);
    crate::blood::say(
        host,
        if written {
            format!("{noun} at {pos:?}: {item} in slot {slot}.")
        } else {
            format!("{pos:?} kept its slot {slot}.")
        },
    );
}

/// A `string` command argument, for an item id: the `item` argument type would make a
/// build against another game read this crate's `Item` types (`fits`).
fn string<'a>(invocation: &'a CommandInvocation, name: &str) -> Option<&'a str> {
    match invocation.argument(name)? {
        CommandArgumentValue::String(value) => Some(value),
        _ => None,
    }
}

/// The mod's infuser at `pos` in the command source's level, or a line saying it is not
/// one.
fn entity_at(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
) -> Option<BlockEntityRef> {
    crate::blood::our_entity(host, source, pos, INFUSER_ID, "Blood Infuser")
}
