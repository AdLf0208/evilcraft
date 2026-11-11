// The chest ticks on the server (`native_block_entity_ticks`, API 15).
fn observe_block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == "evilcraft:blood_chest" {
        *ticks = true;
    }
}
