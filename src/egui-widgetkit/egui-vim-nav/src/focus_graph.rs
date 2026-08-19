use std::collections::HashMap;
use egui::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug, Default)]
pub struct DirectionalNeighbors {
    pub left: Option<Id>,
    pub right: Option<Id>,
    pub up: Option<Id>,
    pub down: Option<Id>,
}

#[derive(Clone, Debug, Default)]
pub struct FocusGraph {
    pub nodes: HashMap<Id, DirectionalNeighbors>,
}
