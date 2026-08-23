# egui-vim-nav

Vim-style (HJKL) keyboard navigation for egui applications.

## What it does

`egui-vim-nav` provides generic, topology-agnostic intra-screen focus
navigation. The consuming application defines the focus graph shape
(grid, tree, custom layout) and this crate provides the traversal engine,
key handler with text-input guards, and immediate-mode rendering helpers.

## No global state

Every type in this crate (`FocusGraph`, `Navigator`, `VimKeyHandler`) is a
plain value the consuming application owns and persists across frames.
There is no hidden global or static mutable instance.

## Quick start

```rust
use egui_vim_nav::{FocusGraph, Navigator, VimKeyHandler, FocusRegion};

// Build the focus graph (app defines the topology)
let mut graph = FocusGraph::new();
graph.connect_grid(&[
    &["a", "b", "c"],
    &["d", "e", "f"],
]);

// Create a navigator (app owns this state)
let mut nav = Navigator::new().with_initial_focus("a");

// In your egui update loop:
let handler = VimKeyHandler::new();
handler.handle_input(ctx, &mut nav, &graph);
```

## Key features

- **Generic**: Works with any ID type (`&str`, enums, `egui::Id`)
- **Topology-agnostic**: Grid, tree, ring, or any custom shape
- **Input guard**: HJKL keys are suppressed when a text field has focus
- **Click-to-focus**: `FocusRegion` handles mouse clicks automatically
- **Zero dependencies** on other `egui-widgetkit` crates

## Running the example

```
cargo run -p egui-vim-nav --example grid_navigation
```
