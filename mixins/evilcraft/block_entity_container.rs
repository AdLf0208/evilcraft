// Ten repair slots and the factor container slot, with the chest's menu
// (`native_block_entity_container`, API 16/17).
fn observe_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::evilcraft_state::on_container(block, container);
}
