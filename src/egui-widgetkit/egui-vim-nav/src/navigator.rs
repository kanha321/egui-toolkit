use egui::Id;
use super::focus_graph::{Direction, FocusGraph};

#[derive(Clone, Debug, Default)]
pub struct Navigator {
    pub current: Option<Id>,
}

impl Navigator {
    pub fn move_dir(&mut self, _graph: &FocusGraph, _dir: Direction) {
    }
}
