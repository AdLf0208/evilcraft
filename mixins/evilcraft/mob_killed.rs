// The player's blow killed the mob: turn its max health into blood for the
// extractors the player carries (`native_mob_killed`, API 40).
fn observe_mob_killed(game: &mut crate::game::Game<'_>, player: u128, entity: u32) {
    crate::game::evilcraft_state::on_mob_killed(game, player, entity);
}
