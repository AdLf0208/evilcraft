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
    /// and menu, and the extractor's Part.
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
