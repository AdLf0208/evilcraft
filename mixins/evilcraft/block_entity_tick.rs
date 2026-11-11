// One server tick of the chest: empty the container slot, then repair
// (`native_block_entity_tick`, API 15).
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::evilcraft_state::on_tick(game, pos, entity);
}
