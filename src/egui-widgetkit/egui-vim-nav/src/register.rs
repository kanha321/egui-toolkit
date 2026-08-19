use egui::Id;
use super::focus_graph::{DirectionalNeighbors, FocusGraph};

pub fn register_region(graph: &mut FocusGraph, id: Id, neighbors: DirectionalNeighbors) {
    graph.nodes.insert(id, neighbors);
}
