use egui::Context;
use super::focus_graph::Direction;

pub fn handle_keys(ctx: &Context) -> Option<Direction> {
    if ctx.wants_keyboard_input() {
        return None;
    }
    None
}
