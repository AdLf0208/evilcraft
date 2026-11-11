// A carried Blood Extractor takes the blood of the player's last kill
// (`native_inventory_tick`, API 39).
fn observe_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    crate::game::evilcraft_state::on_inventory_tick(game, slot, selected);
}
