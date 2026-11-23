//! EvilCraft for Horizon: the blood repair loop.
//!
//! Upstream (CyclopsMC/EvilCraft, read in `FINDINGS.md`) fills a carried Blood
//! Extractor with a mob's blood when the player kills it, and a Blood Chest spends
//! that blood repairing the ten damaged stacks in its slots. The rules and their
//! numbers are [`runtime`]; [`blood`] is the game side, and this file is the one
//! `impl Hooks` that attaches the two Parts and answers the two global moments.
//!
//! The blood chest is a block entity Part (its slots, tank, tick and screen), the
//! extractor an item Part (its tooltip); the kill, the two machines' drops and the
//! harness commands are global hooks. A blood stain is a block entity with a five-bucket
//! tank, and the extractor's crouching click on one draws that blood into the item. A kill
//! is heard on `living_drops` — every cause of death, so an arrow counts — and only when
//! its damage source's causing entity is a player. The mod keeps no state of its own: the
//! tanks live in the block entities' saved compounds, the extractor's blood in the stack's
//! components, and a broken machine's tank travels on its own item as
//! `minecraft:block_entity_data` ([`tank`]). A dark tank is one sixteen-bucket block entity
//! the extractor's crouching click pours blood into ([`dark_tank`]), and the colossal blood
//! chest is the same repair machine at the size of a hollow 3×3×3 shell ([`colossal`]).
pub mod blood;
pub mod colossal;
pub mod dark_tank;
pub mod infuser;
pub mod runtime;
pub mod stain;
pub mod tank;

use mod_api::{
    self, BlockDropsEvent, Flow, Hooks, LevelHost, LivingDeathEvent, RegisterCommandsEvent,
    Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
};

/// The mod has no settings and no state of its own.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Once, at startup: the chest's block entity type and Part, the chest's storage
    /// and menu, the extractor's Part, and the infuser's type, Part, storage, tank and
    /// recipe type. Both machines' items get the Part that puts a carried tank back.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let chest = r.block_key(runtime::CHEST);
        let kind = r
            .block_entity_type(blood::CHEST_ENTITY)
            .blocks(&[blood::CHEST_BLOCK])
            .build();
        r.block_entity_part(&kind, blood::Chest)
            .expect("evilcraft declares the blood chest's own block entity type");
        r.block_container(&chest, blood::container());
        r.block_fluids(&chest, blood::tanks());
        let extractor = r.item_key(runtime::EXTRACTOR);
        r.item_part(extractor, blood::Extractor);
        // The chest's own item: a broken chest carries its tank, a placed one takes it
        // back clamped (upstream's `copy_tank_data` + `onBlockPlacedBy`).
        tank::Machine::Chest.register(r);

        // The Blood Stain: its own block entity type and Part, and its five-bucket tank.
        // It has no tick — a stain is a splash that waits for an extractor — so the click
        // below is the whole of its behaviour in this slice.
        let stain = r.block_key(runtime::STAIN);
        let kind = r
            .block_entity_type(stain::STAIN_ENTITY)
            .blocks(&[stain::STAIN_BLOCK])
            .build();
        r.block_entity_part(&kind, stain::Stain)
            .expect("evilcraft declares the blood stain's own block entity type");
        r.block_fluids(&stain, stain::tanks());
        // A crouching right-click with the extractor draws a stain's blood into the item
        // (upstream's `ItemBloodExtractor.onItemUseFirst`): the second Part on the item,
        // beside its tooltip. The dark tank's Part is registered **before** it, because that
        // one answers `Some` only for its own block and this one answers every click (with
        // `Pass`), and the first Part to answer wins.
        r.item_part(extractor, dark_tank::TankFill);
        r.item_part(extractor, stain::ExtractorDraw);

        // The Dark Tank: one sixteen-bucket tank with no slots and no menu. Its block is a
        // content block; its own item carries the tank when it is broken, like the three
        // machines'.
        let tank_block = r.block_key(runtime::DARK_TANK);
        let kind = r
            .block_entity_type(dark_tank::DARK_TANK_ENTITY)
            .blocks(&[dark_tank::DARK_TANK_BLOCK])
            .build();
        r.block_entity_part(&kind, dark_tank::DarkTank)
            .expect("evilcraft declares the dark tank's own block entity type");
        r.block_fluids(&tank_block, dark_tank::tanks());
        tank::Machine::DarkTank.register(r);

        // The Blood Infuser: its own block entity type and Part, its seven slots, its
        // tank, and the recipe type its data files are written in. The recipe type must
        // be declared before its serializer, which is what makes the engine keep the
        // `evilcraft:blood_infuser` files and publish them to `recipes_of_type`.
        let infuser = r.block_key(runtime::INFUSER);
        let kind = r
            .block_entity_type(infuser::INFUSER_ENTITY)
            .blocks(&[infuser::INFUSER_BLOCK])
            .build();
        r.block_entity_part(&kind, infuser::Infuser)
            .expect("evilcraft declares the blood infuser's own block entity type");
        r.block_container(&infuser, infuser::container());
        r.block_fluids(&infuser, infuser::tanks());
        r.register_recipe_type(runtime::INFUSION_TYPE);
        // The serializer's **name is the recipe file's `type`**: the engine looks the file
        // up by it, so it is the same id upstream registers its `RecipeSerializer` under.
        // The recipe type it belongs to is the same string, as in vanilla (`minecraft:smelting`
        // is both), which is what `recipes_of_type` reads.
        r.register_raw_recipe_serializer(runtime::INFUSION_TYPE, runtime::INFUSION_TYPE);
        // The infuser's own item, the second machine whose drop carries its tank.
        tank::Machine::Infuser.register(r);

        // The Colossal Blood Chest: a block entity with 45 repair slots, a container slot,
        // four upgrade slots and a tank, whose 3×3×3 shell the tick scans every tick. Its
        // plank is a plain content block, and the controller's drop carries its tank like
        // the other two machines' (`copy_tank_data`).
        let colossal = r.block_key(runtime::COLOSSAL);
        let kind = r
            .block_entity_type(colossal::COLOSSAL_ENTITY)
            .blocks(&[colossal::COLOSSAL_BLOCK])
            .build();
        r.block_entity_part(&kind, colossal::Colossal)
            .expect("evilcraft declares the colossal blood chest's own block entity type");
        r.block_container(&colossal, colossal::container());
        r.block_fluids(&colossal, colossal::tanks());
        tank::Machine::Colossal.register(r);
    }

    /// A broken Blood Chest or Blood Infuser drops its own item carrying the blood in its
    /// tank (`copy_tank_data`; upstream's loot tables run it on both machines).
    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        tank::on_drops(level, event)
    }

    /// A mob or player died: a **fall** leaves a Blood Stain holding its blood
    /// (`BlockBloodStain.bloodStainedBlockEvent`), and the extractors the player credited
    /// with the kill take it (upstream's `ItemBloodExtractor.fillForAllBloodExtractors`, on
    /// the same `LivingDeathEvent`; a mob's or the environment's kill is not the player's).
    fn living_death(&self, host: &mut dyn ServerHost, event: &mut LivingDeathEvent<'_>) -> Flow {
        stain::on_death(host, event);
        blood::on_living_death(host, event);
        Flow::Continue
    }

    /// `/evilcraft info|blood|damage|held`, the harness commands (upstream has none).
    fn register_commands(&self, host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        blood::register_commands(host, event);
    }
    /// Run a parsed `/evilcraft ...` command.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        blood::run_command(host, event)
    }
}
