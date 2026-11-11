// The chest declares one 10-bucket tank of `evilcraft:blood`
// (`native_block_entity_fluids`, API 54).
fn observe_block_entity_fluids(
    block: &str,
    fluids: &mut Option<horizon_client::containers::ModFluidTanks>,
) {
    crate::game::evilcraft_state::on_fluids(block, fluids);
}
