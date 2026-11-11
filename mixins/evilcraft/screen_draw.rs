// The chest's tank readout (`native_screen_draw`, API 19).
fn draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::evilcraft_state::on_screen_draw(game, block, data, batch, origin);
}
