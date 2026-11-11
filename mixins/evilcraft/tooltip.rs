// The extractor's tooltip: its blood (`native_tooltip`, API 12).
fn observe_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    crate::game::evilcraft_state::on_tooltip(game, stack, lines);
}
