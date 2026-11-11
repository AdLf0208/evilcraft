// The chest's eleven-slot layout (`native_screen`, API 19).
fn observe_screen(block: &str, screen: &mut Option<NativeScreen>) {
    crate::game::evilcraft_state::on_screen(block, screen);
}
