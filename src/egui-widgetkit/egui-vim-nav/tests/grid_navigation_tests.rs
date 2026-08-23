//! Unit tests for `egui-vim-nav` focus graph and navigator.
//!
//! These are pure data-structure tests — no `egui::Context`, no rendering.
//! Per CODING_RULES §7, widget crates use `examples/` for visual verification;
//! these tests cover the graph traversal and navigator logic only.

use egui_vim_nav::{Direction, FocusGraph, FocusWrap, Navigator};

// ─── FocusGraph: connect_horizontal (bidirectional) ────────────────────────

#[test]
fn connect_horizontal_is_bidirectional() {
    let mut graph = FocusGraph::new();
    graph.connect_horizontal("a", "b");

    assert_eq!(graph.get_neighbor(&"a", Direction::Right), Some(&"b"));
    assert_eq!(graph.get_neighbor(&"b", Direction::Left), Some(&"a"));
}

#[test]
fn connect_vertical_is_bidirectional() {
    let mut graph = FocusGraph::new();
    graph.connect_vertical("top", "bottom");

    assert_eq!(graph.get_neighbor(&"top", Direction::Down), Some(&"bottom"));
    assert_eq!(graph.get_neighbor(&"bottom", Direction::Up), Some(&"top"));
}

#[test]
fn connect_directed_is_one_way() {
    let mut graph = FocusGraph::new();
    graph.connect_directed("a", "b", Direction::Right);

    assert_eq!(graph.get_neighbor(&"a", Direction::Right), Some(&"b"));
    // Reverse direction should NOT be set
    assert_eq!(graph.get_neighbor(&"b", Direction::Left), None);
}

// ─── FocusGraph: connect_grid ──────────────────────────────────────────────

#[test]
fn connect_grid_2x3() {
    let mut graph = FocusGraph::new();
    graph.connect_grid(&[
        &["a", "b", "c"],
        &["d", "e", "f"],
    ]);

    // Horizontal connections
    assert_eq!(graph.get_neighbor(&"a", Direction::Right), Some(&"b"));
    assert_eq!(graph.get_neighbor(&"b", Direction::Right), Some(&"c"));
    assert_eq!(graph.get_neighbor(&"b", Direction::Left), Some(&"a"));
    assert_eq!(graph.get_neighbor(&"c", Direction::Left), Some(&"b"));

    // Vertical connections
    assert_eq!(graph.get_neighbor(&"a", Direction::Down), Some(&"d"));
    assert_eq!(graph.get_neighbor(&"d", Direction::Up), Some(&"a"));
    assert_eq!(graph.get_neighbor(&"b", Direction::Down), Some(&"e"));
    assert_eq!(graph.get_neighbor(&"e", Direction::Up), Some(&"b"));

    // Edge nodes have no neighbor in the boundary direction
    assert_eq!(graph.get_neighbor(&"a", Direction::Left), None);
    assert_eq!(graph.get_neighbor(&"a", Direction::Up), None);
    assert_eq!(graph.get_neighbor(&"f", Direction::Right), None);
    assert_eq!(graph.get_neighbor(&"f", Direction::Down), None);

    // All 6 nodes exist
    assert_eq!(graph.len(), 6);
}

#[test]
fn connect_grid_1x1() {
    let mut graph = FocusGraph::new();
    graph.connect_grid(&[&["only"]]);

    assert!(graph.contains(&"only"));
    assert_eq!(graph.get_neighbor(&"only", Direction::Left), None);
    assert_eq!(graph.get_neighbor(&"only", Direction::Right), None);
    assert_eq!(graph.get_neighbor(&"only", Direction::Up), None);
    assert_eq!(graph.get_neighbor(&"only", Direction::Down), None);
}

#[test]
fn connect_grid_empty() {
    let mut graph: FocusGraph<&str> = FocusGraph::new();
    graph.connect_grid(&[]);
    assert!(graph.is_empty());
}

// ─── FocusGraph: query methods ─────────────────────────────────────────────

#[test]
fn get_neighbor_missing_node_returns_none() {
    let graph: FocusGraph<&str> = FocusGraph::new();
    assert_eq!(graph.get_neighbor(&"nonexistent", Direction::Left), None);
}

#[test]
fn contains_and_clear() {
    let mut graph = FocusGraph::new();
    graph.connect_horizontal("a", "b");
    assert!(graph.contains(&"a"));
    assert!(graph.contains(&"b"));
    assert!(!graph.contains(&"c"));

    graph.clear();
    assert!(graph.is_empty());
    assert!(!graph.contains(&"a"));
}

// ─── Direction ─────────────────────────────────────────────────────────────

#[test]
fn direction_opposite() {
    assert_eq!(Direction::Left.opposite(), Direction::Right);
    assert_eq!(Direction::Right.opposite(), Direction::Left);
    assert_eq!(Direction::Up.opposite(), Direction::Down);
    assert_eq!(Direction::Down.opposite(), Direction::Up);
}

// ─── Navigator: basic traversal ────────────────────────────────────────────

#[test]
fn navigator_traverse_grid() {
    let mut graph = FocusGraph::new();
    graph.connect_grid(&[
        &["a", "b"],
        &["c", "d"],
    ]);

    let mut nav = Navigator::new().with_initial_focus("a");
    assert_eq!(nav.focused(), Some(&"a"));

    // Move right: a → b
    let event = nav.move_focus(&graph, Direction::Right);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.previous, Some("a"));
    assert_eq!(event.current, "b");
    assert_eq!(nav.focused(), Some(&"b"));

    // Move down: b → d
    let event = nav.move_focus(&graph, Direction::Down).unwrap();
    assert_eq!(event.current, "d");

    // Move left: d → c
    let event = nav.move_focus(&graph, Direction::Left).unwrap();
    assert_eq!(event.current, "c");

    // Move up: c → a
    let event = nav.move_focus(&graph, Direction::Up).unwrap();
    assert_eq!(event.current, "a");
}

#[test]
fn navigator_clamp_at_edge() {
    let mut graph = FocusGraph::new();
    graph.connect_horizontal("a", "b");

    let mut nav = Navigator::new()
        .with_initial_focus("a")
        .with_wrap(FocusWrap::Clamp);

    // Try to move left from "a" — should clamp (return None)
    let event = nav.move_focus(&graph, Direction::Left);
    assert!(event.is_none());
    assert_eq!(nav.focused(), Some(&"a")); // Focus unchanged

    // Try to move up from "a" — no vertical neighbors
    let event = nav.move_focus(&graph, Direction::Up);
    assert!(event.is_none());
    assert_eq!(nav.focused(), Some(&"a"));
}

// ─── Navigator: graceful fallback ──────────────────────────────────────────

#[test]
fn navigator_no_focus_initializes_to_first_node() {
    let mut graph = FocusGraph::new();
    graph.connect_horizontal("x", "y");

    let mut nav: Navigator<&str> = Navigator::new(); // No initial focus
    assert_eq!(nav.focused(), None);

    // Any move should pick up the first available node
    let event = nav.move_focus(&graph, Direction::Right);
    assert!(event.is_some());
    // Focus should now be on some node (HashMap order is arbitrary)
    assert!(nav.focused().is_some());
}

#[test]
fn navigator_missing_node_recovers_gracefully() {
    let mut graph = FocusGraph::new();
    graph.connect_horizontal("a", "b");

    // Navigator focused on a node that's NOT in the graph
    let mut nav = Navigator::new().with_initial_focus("removed");

    // Should not panic — should fall back to first available node
    let event = nav.move_focus(&graph, Direction::Right);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.previous, Some("removed"));
    // current should be some node from the graph
    assert!(graph.contains(&event.current));
}

#[test]
fn navigator_empty_graph_returns_none() {
    let graph: FocusGraph<&str> = FocusGraph::new();
    let mut nav: Navigator<&str> = Navigator::new();

    // No nodes to focus on
    let event = nav.move_focus(&graph, Direction::Right);
    assert!(event.is_none());
    assert_eq!(nav.focused(), None);
}

#[test]
fn navigator_set_focus_manually() {
    let mut nav: Navigator<&str> = Navigator::new();
    assert_eq!(nav.focused(), None);

    nav.set_focus(Some("target"));
    assert_eq!(nav.focused(), Some(&"target"));

    nav.set_focus(None);
    assert_eq!(nav.focused(), None);
}
