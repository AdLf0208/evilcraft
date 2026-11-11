// Declare `/evilcraft info|blood|damage|held` (API 8).
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::evilcraft_state::on_register_commands(commands);
}
