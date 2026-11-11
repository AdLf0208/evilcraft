// The blood chest declares a block entity (`native_block_entity`, API 15).
fn observe_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    if block == "evilcraft:blood_chest" {
        *entity = Some(("evilcraft:blood_chest".to_string(), Default::default()));
    }
}
