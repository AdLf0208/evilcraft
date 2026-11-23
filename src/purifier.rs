//! The Purifier (`BlockPurifier`, `BlockEntityPurifier`), carrying **one** of its four
//! `IPurifierAction`s: `ToolBadEnchantPurifyAction`, which strips a curse from any item.
//!
//! Upstream's machine is a GUI-less block: `BlockPurifier.useItemOn` puts the held item into
//! its purify slot and an empty hand takes it back, `FluidUtil` fills its tank from a blood
//! container, and its ticker asks `PurifierActionRegistry.canWork` for the first action that
//! applies and runs it for that action's own duration. The registry holds four actions —
//! curse removal, disenchantment into a Blook, potion collection from a living entity, and
//! the mob-head downgrade — and this package carries the first alone (see `notes.md`'s
//! coverage rows for the three that stay blocked). The rules are [`crate::runtime`]; this
//! file reads and writes the game.
//!
//! Its tank is the pin's saved `fluids` list, as the chests', the infuser's, the stain's and
//! the dark tank's are: `Hooks::register` declares it, the tick writes it, and
//! `/data get block … fluids` reads back what was written. The curses live in the stack's
//! `minecraft:enchantments` component, which at 26.3 is a bare map of enchantment to level
//! (`{"minecraft:vanishing_curse": 1}`); this file reads that shape and the pre-1.21.5
//! `{levels: {…}}` wrapper, and writes 26.3's.

use mod_api::{
    self, BlockClick, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTicksEvent, BlockPart, CommandInvocation, CommandSource, Component, Compound, Hand,
    Host, HostSide, HostTransaction, InteractionResult, ItemPart, ItemStack, LevelSide,
    ModContainer, ModFluidTanks, PlayerRef, ServerHost, Tag, Text, UseItemOnEvent, UseOnEvent,
    UseWithoutItemEvent,
};

use crate::runtime::{
    BUCKET, PURIFIER, PURIFIER_ADDITIONAL_SLOT, PURIFIER_PURIFY_SLOT, PURIFIER_SLOTS,
    PURIFIER_TANK, is_curse, purified, purify_tick,
};

/// The block entity type: a bare name, so the mod's namespace qualifies it.
pub const PURIFIER_ENTITY: &str = "purifier";
/// The block the type belongs to, as the declaration builder takes it.
pub const PURIFIER_BLOCK: &str = "purifier";
/// The block's full id, as `Level::block` names it.
pub const PURIFIER_ID: &str = "evilcraft:purifier";

/// The block entity's saved action clock, in ticks (`BlockEntityPurifier.tick`).
const TICK: &str = "tick";
/// The vanilla component an item's enchantments live in (`DataComponents.ENCHANTMENTS`).
const ENCHANTMENTS: &str = "minecraft:enchantments";
/// The key a **pre-1.21.5** save wraps the enchantment map in (`{levels: {…}}`). 26.3's codec
/// writes the bare map and still *reads* the wrapper (`horizon-items`' `read_levels`), so a
/// stack this port reads may arrive either way.
const LEVELS: &str = "levels";

// ---------------------------------------------------------------------------
// The declarations
// ---------------------------------------------------------------------------

/// The purifier's two slots (`BlockEntityPurifier.SLOTS`): the stack it works on and
/// upstream's additional slot. It declares **no menu**: upstream's block has no GUI (the
/// click is its whole interface), so no face, input or output reaches a slot either — the
/// click is the route in and out, as it is for a player.
#[must_use]
pub fn container() -> ModContainer {
    ModContainer {
        slots: PURIFIER_SLOTS,
        menu: None,
        ..ModContainer::default()
    }
}

/// One three-bucket tank every face reaches and that only gives: the blood gets in through
/// the extractor's crouching click, never through a pipe (the machines' own gap: no
/// `block_entity_accepts_fluid` at this pin, so a foreign fluid would be pushed in and the
/// machine would then read 0 mB until blood is put back).
///
/// Built by assignment, not with a literal: `ModFluidTanks` is `#[non_exhaustive]`.
#[must_use]
pub fn tanks() -> ModFluidTanks {
    let all = vec![0];
    let mut tanks = ModFluidTanks::default();
    tanks.tanks = vec![PURIFIER_TANK];
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
// The block: upstream's `useItemOn` and `useWithoutItem`
// ---------------------------------------------------------------------------

/// One tick of the purifier's curse action, and the clicks: the game side.
///
/// **Diagnostics.** `PROBE` is the in-game probe's switch (`tools/probe-purifier.sh`); the
/// shipped package ships `false` and the compiler removes the lines.
const PROBE: bool = false;

/// One diagnostic line, into chat, while the probe builds run.
fn probe(host: &mut dyn ServerHost, line: impl Into<String>) {
    if PROBE {
        let _ = host
            .commands()
            .feedback(Text::literal(format!("PROBE {}", line.into())));
    }
}

/// The Purifier's block (`evilcraft:purifier`): its two clicks, and through the block entity
/// Part below, its tick.
pub struct Purifier;

#[mod_api::horizon_part]
impl BlockPart for Purifier {
    /// `BlockPurifier.useItemOn`: a filled Blood Extractor pours its blood into the tank when
    /// it is clicked crouching (this port's container gesture, as on the dark tank and the
    /// stain), and any other click puts one item into the purify slot when that slot is
    /// empty. The empty hand takes the item back, and asks for `use_without_item` so the
    /// engine's own empty-hand route answers too.
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = *event.click;
        let held = click
            .stack
            .map(|stack| stack.item().id(&*host).to_owned())
            .unwrap_or_else(|| "none".to_owned());
        probe(
            host,
            format!(
                "use_item_on pos={:?} held={held} hand={:?}",
                click.pos, click.hand
            ),
        );
        // The main hand only. This hook is server-side, so the clicking client cannot predict
        // it: its main-hand attempt passes, and it sends a second click for the off hand,
        // which would take the item straight back out of the slot the first one filled
        // (measured 2026-11-23). Upstream's `useItemOn` answers for the hand that clicked;
        // this port's is the main hand, the one `use_without_item` is asked for.
        if click.hand != Hand::Main {
            return None;
        }
        if let Some(stack) = click.stack {
            // A click this machine does not take answers `Pass`, never "try the empty hand":
            // upstream's `useItemOn` passes to the item's own use, and an empty-hand retry
            // after a refused insert would take the item out of the slot it just found full
            // (measured 2026-11-23).
            let poured = host
                .player(click.player)
                .is_some_and(|player| pour(host, player, click.pos, click.hand, stack));
            let taken = poured || insert(host, click);
            return Some(if taken {
                InteractionResult::Success
            } else {
                InteractionResult::Pass
            });
        }
        // An empty hand: the item comes back out. `Pass` when it does not, for the same
        // reason — the slot is either empty or this port's to report, not the item's.
        Some(if take(host, click) {
            InteractionResult::Success
        } else {
            InteractionResult::Pass
        })
    }

    /// `BlockPurifier.useWithoutItem`: an empty hand takes the purify slot's item back, and
    /// then the additional slot's, as upstream's `itemStack.isEmpty()` branches do.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = *event.click;
        probe(
            host,
            format!("use_without_item pos={:?} hand={:?}", click.pos, click.hand),
        );
        if click.hand != Hand::Main {
            return None;
        }
        take(host, click).then_some(InteractionResult::Success)
    }
}

// ---------------------------------------------------------------------------
// The block entity: the tank, the clock and the action
// ---------------------------------------------------------------------------

#[mod_api::horizon_part]
impl BlockEntityPart for Purifier {
    /// The entity starts with an empty saved compound: the creation declaration owns its
    /// type, so no data is needed to make one.
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

    /// One server tick: upstream's `BlockEntityPurifier.Ticker` — ask the action table
    /// whether anything applies, count the tick, and run the action on its duration tick.
    /// With one action carried, the table is [`crate::runtime::purify_tick`].
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let entity = &mut *event.entity;
        if entity.items.len() < PURIFIER_SLOTS {
            return;
        }
        let mut tank = crate::blood::read_tank_with(&entity.nbt, PURIFIER_TANK);
        let tick = crate::blood::read_int(&entity.nbt, TICK)
            .unwrap_or(0)
            .max(0) as u32;
        let levels = enchantment_levels(entity.items[PURIFIER_PURIFY_SLOT].as_ref());
        let curses = levels.iter().filter(|(id, _)| is_curse(id)).count();
        let mut next = 0;
        match purify_tick(tick, curses, tank) {
            // `canWork` refused: the clock resets and nothing is spent.
            None => {}
            Some((after, false)) => next = after,
            Some((_, true)) => {
                let (kept, spent) = purified(&levels);
                if let Some(stack) = entity.items[PURIFIER_PURIFY_SLOT].as_mut() {
                    write_enchantments(stack, &kept);
                }
                tank = (tank - spent * BUCKET).max(0);
            }
        }
        let _ = host;
        crate::blood::write_int(&mut entity.nbt, TICK, next as i32);
        crate::blood::write_tank_with(&mut entity.nbt, tank, PURIFIER_TANK);
    }
}

// ---------------------------------------------------------------------------
// The stack's enchantments
// ---------------------------------------------------------------------------

/// `minecraft:enchantments`, as a component the stack's patch is read from and written to.
struct Enchantments;

impl Component for Enchantments {
    type Value = Compound;

    fn id(&self) -> &str {
        ENCHANTMENTS
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

/// One stack's enchantments, in the component's own order: `(id, level)`.
///
/// The port reads the stack's own component patch, which is where a curse on a tool lives.
/// 26.3's `minecraft:enchantments` component is the **bare map** —
/// `/give … [minecraft:enchantments={"minecraft:vanishing_curse":1}]` — and that is what the
/// game writes back; a save from before 1.21.5 wraps it in `{levels: {…}}`, which the game's
/// own `read_levels` still reads, so this does too. A level the tag does not state as an
/// integer is not an enchantment this action can read.
#[must_use]
fn enchantment_levels(stack: Option<&ItemStack>) -> Vec<(String, u32)> {
    let Some(stack) = stack else {
        return Vec::new();
    };
    let Some(data) = stack.get(&Enchantments) else {
        return Vec::new();
    };
    let map = match crate::blood::field(&data, LEVELS) {
        Some(Tag::Compound(inner)) => inner,
        _ => &data,
    };
    map.iter()
        .filter_map(|(id, tag)| match tag {
            Tag::Int(level) if *level > 0 => Some((id.to_string_lossy().to_owned(), *level as u32)),
            _ => None,
        })
        .collect()
}

/// Write a stack's enchantments back in 26.3's bare-map shape, an emptied map removing the
/// component entirely so the item is the plain one it was before the curse
/// (`setEnchantmentLevel(enchantment, 0)` removes an enchantment, and an item with none
/// carries no component).
fn write_enchantments(stack: &mut ItemStack, levels: &[(String, u32)]) {
    if levels.is_empty() {
        stack.remove(&Enchantments);
        return;
    }
    let mut data = Compound::new();
    for (id, level) in levels {
        data.insert(id.as_str().into(), Tag::Int(*level as i32));
    }
    let _ = stack.set(&Enchantments, data);
}

/// How many enchantments a stack carries, for the readout.
#[must_use]
fn enchantment_count(stack: Option<&ItemStack>) -> usize {
    enchantment_levels(stack).len()
}

// ---------------------------------------------------------------------------
// The clicks
// ---------------------------------------------------------------------------

/// The purifier at the click's position, with the block it stands as checked: a click this
/// port's block did not receive (another block, another mod's) is not this machine's.
fn purifier_at(host: &mut dyn ServerHost, click: BlockClick<'_>) -> Option<BlockEntityRef> {
    let player = host.player(click.player)?;
    let level = player.level(&*host)?;
    let block = level
        .block(&*host, click.pos)
        .map(|key| key.id(&*host).to_owned())
        .unwrap_or_else(|| "none".to_owned());
    let ours = block == PURIFIER;
    let entity = host.block_entity(level, click.pos);
    probe(
        host,
        format!(
            "at pos={:?} block={block} entity={}",
            click.pos,
            if entity.is_some() { "yes" } else { "no" }
        ),
    );
    if !ours {
        return None;
    }
    entity
}

/// The Blood Extractor's crouching click on a Purifier: upstream's `BlockPurifier.useItemOn`
/// handing the click to `FluidUtil.interactWithFluidHandler`, which fills the tank from the
/// held blood container. This port's container is the extractor and its gesture is the same
/// crouching click the stain's draw and the dark tank's fill use — and, as with both of those,
/// it lives on the **item**, because that is the Part a crouching right-click reaches at this
/// pin (measured 2026-11-23: a crouching click ran no block hook at all).
///
/// A standing click, another block and an empty extractor all answer **`None`**, not `Pass`:
/// the first Part to answer ends the chain, so a `Pass` here would stop the stain's Part from
/// ever drawing (measured 2026-11-23 — phase S drew nothing until this returned `None`, the
/// same contract `dark_tank::TankFill` keeps).
pub struct Pour;

#[mod_api::horizon_part]
impl ItemPart for Pour {
    fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            return None;
        };
        let player = host.players().acting_player()?;
        probe(
            host,
            format!(
                "use_on pos={:?} face={:?} hit={:?} hand={:?}",
                event.pos, event.face, event.hit, event.hand
            ),
        );
        pour(host, player, event.pos, event.hand, event.stack).then_some(InteractionResult::Success)
    }
}

/// The crouching click itself: the tank at `pos` times the held extractor, one transfer.
/// Returns whether any blood moved.
///
/// Upstream does both halves through fluid capabilities in one transaction. That is not
/// available across a block entity and a player's slot at this pin, so the tank is written
/// first and put back if the stack write is refused: a refused write must not leave the
/// extractor holding blood the tank was already given.
fn pour(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    pos: [i32; 3],
    hand: Hand,
    stack: &ItemStack,
) -> bool {
    // Upstream's tank interaction is a plain right-click; this port's container gesture is
    // the extractor's crouching click (the dark tank's rule), so a standing click pours
    // nothing and puts the extractor in the slot instead.
    let crouching = player.is_secondary_use_active(&*host).unwrap_or(false);
    // One amount belongs to one extractor; a stack is split by the inventory first.
    let single = crate::runtime::one_extractor(stack.count());
    let blood = crate::blood::extractor_blood(stack);
    probe(
        host,
        format!(
            "pour: pos={pos:?} crouching={crouching} blood={blood} count={} hand={hand:?}",
            stack.count()
        ),
    );
    if !crouching || !single || blood <= 0 {
        return false;
    }
    let Some(level) = player.level(&*host) else {
        return false;
    };
    let ours = level
        .block(&*host, pos)
        .is_some_and(|key| key.id(&*host) == PURIFIER);
    if !ours {
        return false;
    }
    let Some(entity) = host.block_entity(level, pos) else {
        return false;
    };
    let Some(data) = entity.data(&*host) else {
        return false;
    };
    let tank = crate::blood::read_tank_with(&data, PURIFIER_TANK);
    let (filled, keeps) = crate::runtime::drain_container_into(tank, blood, PURIFIER_TANK);
    probe(
        host,
        format!("pour: tank={tank} filled={filled} keeps={keeps}"),
    );
    if filled == tank {
        return false;
    }
    let mut patch = Compound::new();
    crate::blood::write_tank_with(&mut patch, filled, PURIFIER_TANK);
    if !matches!(entity.merge_data(host, patch), Ok(true)) {
        probe(host, "pour: the tank write was refused");
        return false;
    }
    let emptied = crate::blood::with_extractor_blood(stack, keeps);
    let written = crate::blood::hand_slot(&*host, player, hand)
        .is_some_and(|slot| player.set_inventory_slot(host, slot, Some(emptied)).is_ok());
    probe(host, format!("pour: hand written={written}"));
    if !written {
        // The extractor kept its blood, so the tank gives back what it was just given.
        let mut patch = Compound::new();
        crate::blood::write_tank_with(&mut patch, tank, PURIFIER_TANK);
        let _ = entity.merge_data(host, patch);
        return false;
    }
    true
}

/// A click with an item and an empty purify slot: upstream's `isItemValidForMainSlot` +
/// `getPurifyItem().isEmpty()`, one item in.
///
/// Upstream's registry answers yes for any item (its `DisenchantPurifyAction.isAllowed` is
/// everything but a regex over the item id, and no `tetra` is in play), so the
/// port accepts any single item too — the machine only *acts* on a cursed one. The
/// additional slot is only ever filled by the three actions this package does not carry, so
/// it is not a drop target for a click.
///
/// The hand is taken first and put back if the slot refuses it: the other order would leave
/// the item in both places when one of the two writes failed.
fn insert(host: &mut dyn ServerHost, click: BlockClick<'_>) -> bool {
    let Some(entity) = purifier_at(host, click) else {
        return false;
    };
    match entity.item_slot(&*host, PURIFIER_PURIFY_SLOT as u32) {
        Ok(None) => {}
        _ => return false,
    }
    let Some(held) = click.stack else {
        probe(host, "insert: no held stack");
        return false;
    };
    // Taking one extractor out of a legacy stack would copy the stack's one amount onto the
    // item taken (`shrink(1)` keeps the components). The inventory resolves such a stack
    // first (the item is unstackable); until then the machine refuses it.
    if held.item().id(&*host) == crate::runtime::EXTRACTOR
        && !crate::runtime::one_extractor(held.count())
    {
        probe(host, "insert: a stacked extractor is not a container");
        return false;
    }
    let Some(player) = host.player(click.player) else {
        probe(host, "insert: no player handle");
        return false;
    };
    let Some(hand) = crate::blood::hand_slot(&*host, player, click.hand) else {
        probe(host, "insert: no hand slot");
        return false;
    };
    // One item of the stack, components and damage included, as upstream's
    // `copy.setCount(1)` — never rebuilt from the id, which would drop the curse.
    let mut one = held.clone();
    if one.set_count(1).is_err() {
        return false;
    }
    let left = held.clone().shrink(1);
    let hand_taken = player.set_inventory_slot(host, hand, left).is_ok();
    probe(host, format!("insert: hand={hand} taken={hand_taken}"));
    if !hand_taken {
        return false;
    }
    let inserted = write_slot(host, &entity, PURIFIER_PURIFY_SLOT, Some(one));
    probe(host, format!("insert: slot write={inserted}"));
    if !inserted {
        // The machine refused it, so the hand gets its item back.
        let _ = player.set_inventory_slot(host, hand, Some(held.clone()));
    }
    inserted
}

/// An empty-hand click: the purify slot's item goes back to the player's hand, and then the
/// additional slot's, as `BlockPurifier.useItemOn`'s two `isEmpty()` branches do. Returns
/// whether anything was taken.
fn take(host: &mut dyn ServerHost, click: BlockClick<'_>) -> bool {
    let Some(player) = host.player(click.player) else {
        return false;
    };
    let Some(entity) = purifier_at(host, click) else {
        return false;
    };
    for slot in [PURIFIER_PURIFY_SLOT, PURIFIER_ADDITIONAL_SLOT] {
        let Ok(Some(stack)) = entity.item_slot(&*host, slot as u32) else {
            continue;
        };
        if !write_slot(host, &entity, slot, None) {
            probe(host, format!("take: slot {slot} could not be cleared"));
            return false;
        }
        let Some(hand) = crate::blood::hand_slot(&*host, player, click.hand) else {
            let _ = write_slot(host, &entity, slot, Some(stack));
            return false;
        };
        if player
            .set_inventory_slot(host, hand, Some(stack.clone()))
            .is_err()
        {
            // The hand refused it, so the machine keeps it.
            let _ = write_slot(host, &entity, slot, Some(stack));
            return false;
        }
        return true;
    }
    false
}

/// One slot of the purifier, written under its own transaction.
fn write_slot(
    host: &mut dyn ServerHost,
    entity: &BlockEntityRef,
    slot: usize,
    stack: Option<ItemStack>,
) -> bool {
    host.transaction(|host, tx| -> Result<bool, ()> {
        Ok(entity
            .set_item_slot(host, tx, slot as u32, stack.clone())
            .unwrap_or(false))
    })
    .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// The harness command
// ---------------------------------------------------------------------------

/// `/evilcraft purifier <pos>`: the tank, the two slots, the curse clock and how many
/// enchantments the purge slot's stack carries, so a scenario can read the machine without a
/// GUI (upstream has no commands at all in this area).
pub fn readout(host: &mut dyn ServerHost, source: &CommandSource, invocation: &CommandInvocation) {
    if !crate::blood::may_read(source.permission()) {
        crate::blood::say(
            host,
            format!(
                "evilcraft purifier needs an operator (permission level {})",
                crate::blood::OPERATOR_LEVEL
            ),
        );
        return;
    }
    let Some(pos) = crate::blood::block_pos(invocation) else {
        crate::blood::say(host, "usage: /evilcraft purifier <x> <y> <z>".to_owned());
        return;
    };
    let Some(entity) = crate::blood::our_entity(host, source, pos, PURIFIER_ID, "Purifier") else {
        return;
    };
    let Some(data) = entity.data(&*host) else {
        crate::blood::say(host, format!("{pos:?} is not a Purifier."));
        return;
    };
    let tank = crate::blood::read_tank_with(&data, PURIFIER_TANK);
    let tick = crate::blood::read_int(&data, TICK).unwrap_or(0).max(0);
    let purge = entity.item_slot(&*host, PURIFIER_PURIFY_SLOT as u32);
    let (line, ench, curses) = match purge {
        Ok(Some(stack)) => (
            format!("{}@{}", stack.item().id(&*host), stack.count()),
            enchantment_count(Some(&stack)),
            enchantment_levels(Some(&stack))
                .iter()
                .filter(|(id, _)| is_curse(id))
                .count(),
        ),
        _ => ("empty@0".to_owned(), 0, 0),
    };
    let additional = match entity.item_slot(&*host, PURIFIER_ADDITIONAL_SLOT as u32) {
        Ok(Some(stack)) => format!("{}@{}", stack.item().id(&*host), stack.count()),
        _ => "empty@0".to_owned(),
    };
    let time = entity
        .level(&*host)
        .and_then(|level| level.game_time(&*host))
        .unwrap_or(0);
    crate::blood::say(
        host,
        format!(
            "Purifier {pos:?}: tank {tank}/{PURIFIER_TANK} mB, purge {line}, add {additional}, ench {ench}, curses {curses}, tick {tick}, t={time}"
        ),
    );
}
