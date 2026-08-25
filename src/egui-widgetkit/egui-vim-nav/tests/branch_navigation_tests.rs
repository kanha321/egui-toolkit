//! Unit tests for `egui-vim-nav` 1-to-many branch navigation, strategies, and memory lifecycle.

use egui_vim_nav::{BranchStrategy, Direction, FocusGraph, Navigator};

#[test]
fn test_lateral_movement_updates_remember_last() {
    let mut graph = FocusGraph::new();
    graph.connect_branch("Slider", Direction::Down, &["Btn25", "Btn50", "Btn75", "Btn100"]);

    let mut nav = Navigator::new().with_initial_focus("Slider");

    // 1. Move Down into the branch -> lands on first child (Btn25)
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves to first child");
    assert_eq!(ev.current, "Btn25");

    // 2. Move Right laterally across siblings to Btn75
    let ev = nav.move_focus(&graph, Direction::Right).expect("moves right to 50");
    assert_eq!(ev.current, "Btn50");
    let ev = nav.move_focus(&graph, Direction::Right).expect("moves right to 75");
    assert_eq!(ev.current, "Btn75");

    // 3. Move Up back to the parent Slider
    let ev = nav.move_focus(&graph, Direction::Up).expect("moves up to parent");
    assert_eq!(ev.current, "Slider");

    // 4. Move Down again -> should return directly to Btn75!
    let ev = nav.move_focus(&graph, Direction::Down).expect("returns to last focused child");
    assert_eq!(ev.current, "Btn75");
}

#[test]
fn test_branch_strategy_first() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_with_strategy(
        "Slider",
        Direction::Down,
        &["Btn25", "Btn50", "Btn75", "Btn100"],
        BranchStrategy::First,
    );

    let mut nav = Navigator::new().with_initial_focus("Slider");

    // Move Down -> Btn25
    nav.move_focus(&graph, Direction::Down);
    // Move Right to Btn75
    nav.move_focus(&graph, Direction::Right);
    nav.move_focus(&graph, Direction::Right);
    assert_eq!(nav.focused(), Some(&"Btn75"));

    // Move Up to Slider
    nav.move_focus(&graph, Direction::Up);
    assert_eq!(nav.focused(), Some(&"Slider"));

    // Move Down -> should reset to First (Btn25)
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down");
    assert_eq!(ev.current, "Btn25");
}

#[test]
fn test_branch_strategy_last() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_with_strategy(
        "Slider",
        Direction::Down,
        &["Btn25", "Btn50", "Btn75", "Btn100"],
        BranchStrategy::Last,
    );

    let mut nav = Navigator::new().with_initial_focus("Slider");

    // Move Down -> should jump directly to Last (Btn100)
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down");
    assert_eq!(ev.current, "Btn100");

    // Move Left to Btn50
    nav.move_focus(&graph, Direction::Left);
    nav.move_focus(&graph, Direction::Left);
    assert_eq!(nav.focused(), Some(&"Btn50"));

    // Move Up to Slider
    nav.move_focus(&graph, Direction::Up);

    // Move Down -> should jump to Last (Btn100)
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down");
    assert_eq!(ev.current, "Btn100");
}

#[test]
fn test_branch_strategy_edge_aware() {
    let mut graph = FocusGraph::new();
    // Top parent connects Down to vertical strip with EdgeAware
    graph.connect_branch_full(
        "TopCard",
        Direction::Down,
        &["Item1", "Item2", "Item3", "Item4"],
        Direction::Down,
        BranchStrategy::EdgeAware,
    );
    // Bottom parent connects Up to the same strip with EdgeAware
    graph.connect_branch_full(
        "BottomCard",
        Direction::Up,
        &["Item1", "Item2", "Item3", "Item4"],
        Direction::Down,
        BranchStrategy::EdgeAware,
    );

    // 1. Enter from TopCard moving Down -> lands on top-most Item1
    let mut nav = Navigator::new().with_initial_focus("TopCard");
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down to first");
    assert_eq!(ev.current, "Item1");

    // 2. Enter from BottomCard moving Up -> lands on bottom-most Item4!
    let mut nav2 = Navigator::new().with_initial_focus("BottomCard");
    let ev2 = nav2.move_focus(&graph, Direction::Up).expect("moves up to last");
    assert_eq!(ev2.current, "Item4");
}

#[test]
fn test_branch_strategy_anchor() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_with_strategy(
        "Slider",
        Direction::Down,
        &["Btn25", "Btn50", "Btn75", "Btn100"],
        BranchStrategy::Anchor("Btn50"),
    );

    let mut nav = Navigator::new().with_initial_focus("Slider");

    // Move Down -> should target anchor Btn50
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down to anchor");
    assert_eq!(ev.current, "Btn50");

    // Move Right to Btn100
    nav.move_focus(&graph, Direction::Right);
    nav.move_focus(&graph, Direction::Right);
    assert_eq!(nav.focused(), Some(&"Btn100"));

    // Move Up to Slider
    nav.move_focus(&graph, Direction::Up);

    // Move Down -> should land on Anchor Btn50
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down to anchor");
    assert_eq!(ev.current, "Btn50");
}

#[test]
#[should_panic(expected = "Anchor node \"Invalid\" is not in the provided children list")]
fn test_anchor_startup_validation_panics() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_with_strategy(
        "Parent",
        Direction::Down,
        &["A", "B", "C"],
        BranchStrategy::Anchor("Invalid"),
    );
}

#[test]
fn test_disconnect_branch_purges_memory() {
    let mut graph = FocusGraph::new();
    graph.connect_branch("Parent", Direction::Down, &["A", "B", "C"]);

    let mut nav = Navigator::new().with_initial_focus("Parent");
    nav.move_focus(&graph, Direction::Down);
    nav.move_focus(&graph, Direction::Right); // focus is B
    assert_eq!(nav.get_last_branch_focus(&"Parent", Direction::Down), Some(&"B"));

    // Disconnect branch
    let removed = graph.disconnect_branch(&"Parent", Direction::Down);
    assert!(removed.is_some());

    // Clear navigator memory for that branch
    nav.clear_branch_memory_for_dir(&"Parent", Direction::Down);
    assert_eq!(nav.get_last_branch_focus(&"Parent", Direction::Down), None);
}

#[test]
fn test_update_branch_children_anchor_self_heals() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_with_strategy(
        "Parent",
        Direction::Down,
        &["A", "B", "C"],
        BranchStrategy::Anchor("B"),
    );

    // Mutate children removing anchor B
    graph.update_branch_children(&"Parent", Direction::Down, &["X", "Y", "Z"]);

    let mut nav = Navigator::new().with_initial_focus("Parent");
    // Should safely self-heal to first child "X" without panic
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down");
    assert_eq!(ev.current, "X");
}

#[test]
fn test_update_branch_children_remember_last_self_heals() {
    let mut graph = FocusGraph::new();
    graph.connect_branch("Parent", Direction::Down, &["A", "B", "C"]);

    let mut nav = Navigator::new().with_initial_focus("Parent");
    nav.move_focus(&graph, Direction::Down);
    nav.move_focus(&graph, Direction::Right); // focused on B
    nav.move_focus(&graph, Direction::Up); // back to Parent

    // Update children to set that does not contain B
    graph.update_branch_children(&"Parent", Direction::Down, &["New1", "New2"]);

    // Navigating down should safely prune stale B and land on New1
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down");
    assert_eq!(ev.current, "New1");
}

#[test]
fn test_scoped_memory_clear() {
    let mut graph = FocusGraph::new();
    graph.connect_branch("Parent1", Direction::Down, &["A1", "B1"]);
    graph.connect_branch("Parent2", Direction::Down, &["A2", "B2"]);

    let mut nav = Navigator::new();
    nav.set_focus_with_graph(Some("B1"), &graph);
    nav.set_focus_with_graph(Some("B2"), &graph);

    assert_eq!(nav.get_last_branch_focus(&"Parent1", Direction::Down), Some(&"B1"));
    assert_eq!(nav.get_last_branch_focus(&"Parent2", Direction::Down), Some(&"B2"));

    // Scoped clear for Parent1
    nav.clear_branch_memory_for(&"Parent1");
    assert_eq!(nav.get_last_branch_focus(&"Parent1", Direction::Down), None);
    assert_eq!(nav.get_last_branch_focus(&"Parent2", Direction::Down), Some(&"B2"));
}

#[test]
fn test_all_decoupled_api_combinations() {
    let mut graph = FocusGraph::new();

    // 1. Inferred axis + default strategy
    graph.connect_branch("P1", Direction::Down, &["C1", "C2"]);
    // 2. Inferred axis + explicit strategy
    graph.connect_branch_with_strategy("P2", Direction::Down, &["C3", "C4"], BranchStrategy::Last);
    // 3. Explicit axis + default strategy
    graph.connect_branch_with_axis("P3", Direction::Right, &["C5", "C6"], Direction::Down);
    // 4. Explicit axis + explicit strategy
    graph.connect_branch_full("P4", Direction::Right, &["C7", "C8"], Direction::Down, BranchStrategy::First);

    let mut nav = Navigator::new();

    nav.set_focus(Some("P1"));
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("C1"));

    nav.set_focus(Some("P2"));
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("C4"));

    nav.set_focus(Some("P3"));
    assert_eq!(nav.move_focus(&graph, Direction::Right).map(|e| e.current), Some("C5"));

    nav.set_focus(Some("P4"));
    assert_eq!(nav.move_focus(&graph, Direction::Right).map(|e| e.current), Some("C7"));
}

#[test]
fn test_horizontal_branch_navigation_remember_last() {
    let mut graph = FocusGraph::new();
    // LeftNode connects Right to a vertical child column [Item1, Item2, Item3, Item4], which connects Right to RightNode
    graph.connect_branch_between(
        "LeftPanel",
        Direction::Right,
        &["Item1", "Item2", "Item3", "Item4"],
        "RightPanel",
    );

    let mut nav = Navigator::new().with_initial_focus("LeftPanel");

    // 1. Move Right into vertical column -> lands on Item1
    let ev = nav.move_focus(&graph, Direction::Right).expect("moves right to first item");
    assert_eq!(ev.current, "Item1");

    // 2. Move Down vertically across children to Item3
    nav.move_focus(&graph, Direction::Down); // Item2
    let ev = nav.move_focus(&graph, Direction::Down).expect("moves down to Item3");
    assert_eq!(ev.current, "Item3");

    // 3. Move Left back to LeftPanel
    let ev = nav.move_focus(&graph, Direction::Left).expect("moves left to LeftPanel");
    assert_eq!(ev.current, "LeftPanel");

    // 4. Move Right again from LeftPanel -> should remember Item3!
    let ev = nav.move_focus(&graph, Direction::Right).expect("returns to Item3");
    assert_eq!(ev.current, "Item3");

    // 5. Move Right to RightPanel
    let ev = nav.move_focus(&graph, Direction::Right).expect("moves right to RightPanel");
    assert_eq!(ev.current, "RightPanel");

    // 6. Move Left from RightPanel -> should also remember Item3!
    let ev = nav.move_focus(&graph, Direction::Left).expect("returns to Item3 from right");
    assert_eq!(ev.current, "Item3");
}

#[test]
fn test_connect_branch_between_two_way_memory() {
    let mut graph = FocusGraph::new();
    graph.connect_branch_between("TokenInput", Direction::Down, &["Dev", "Staging", "Prod"], "CheckNotify");

    let mut nav = Navigator::new();

    // Start at Top -> Down to Dev
    nav.set_focus(Some("TokenInput"));
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("Dev"));

    // Move laterally to Staging
    assert_eq!(nav.move_focus(&graph, Direction::Right).map(|e| e.current), Some("Staging"));

    // Move Up to TokenInput -> Down must return to Staging
    assert_eq!(nav.move_focus(&graph, Direction::Up).map(|e| e.current), Some("TokenInput"));
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("Staging"));

    // Move Down to CheckNotify -> Up must return to Staging
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("CheckNotify"));
    assert_eq!(nav.move_focus(&graph, Direction::Up).map(|e| e.current), Some("Staging"));

    // Move laterally to Prod
    assert_eq!(nav.move_focus(&graph, Direction::Right).map(|e| e.current), Some("Prod"));

    // Move Down to CheckNotify -> Up must return to Prod
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("CheckNotify"));
    assert_eq!(nav.move_focus(&graph, Direction::Up).map(|e| e.current), Some("Prod"));

    // Move Up to TokenInput -> Down must return to Prod
    assert_eq!(nav.move_focus(&graph, Direction::Up).map(|e| e.current), Some("TokenInput"));
    assert_eq!(nav.move_focus(&graph, Direction::Down).map(|e| e.current), Some("Prod"));
}

