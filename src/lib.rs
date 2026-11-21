//! EvilCraft for Horizon: the blood repair loop.
//!
//! Upstream (CyclopsMC/EvilCraft, read in `FINDINGS.md`) fills a carried Blood
//! Extractor with a mob's blood when the player kills it, and a Blood Chest spends
//! that blood repairing the ten damaged stacks in its slots. The rules and their
//! numbers are [`runtime`]; [`blood`] is the game side, and this file is the one
//! `impl Hooks` that attaches the two Parts and answers the two global moments.
//!
//! The blood chest is a block entity Part (its slots, tank, tick and screen), the
//! extractor an item Part (its tooltip); the kill and the harness commands are
//! global hooks. The mod keeps no state of its own: the tank lives in the block
//! entity's saved compound and the extractor's blood in the stack's components.
pub mod blood;
pub mod infuser;
pub mod runtime;

use mod_api::{
    self, Flow, Hooks, MobKilledEvent, RegisterCommandsEvent, Registrar, RegistrationContext,
    RunModCommandEvent, ServerHost,
};

/// The mod has no settings and no state of its own.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Once, at startup: the chest's block entity type and Part, the chest's storage
    /// and menu, the extractor's Part, and the infuser's type, Part, storage, tank and
    /// recipe type.
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
    }

    /// `player`'s blow killed a mob: the extractors the player carries take its blood
    /// (upstream's `ItemBloodExtractor.fillForAllBloodExtractors`).
    fn mob_killed(&self, host: &mut dyn ServerHost, event: &mut MobKilledEvent<'_>) {
        blood::on_kill(host, event);
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
