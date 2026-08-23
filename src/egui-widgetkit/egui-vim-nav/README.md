# egui-vim-nav

Vim-style (HJKL) keyboard and mouse navigation for [egui](https://github.com/emilk/egui) applications.

## What It Does

Provides generic, topology-agnostic focus navigation. Your app defines the focus graph shape (grid, tree, ring, or any custom topology) and this crate provides the traversal engine, key handler with text-input guards, and immediate-mode rendering helpers.

Features:
- **Generic**: Works with any ID type (`&str`, enums, integers, `egui::Id`)
- **Topology-agnostic**: Grid, tree, ring, or any custom shape
- **Input guard**: HJKL keys are automatically suppressed when a text field has focus
- **Click-to-focus**: `FocusRegion` handles mouse clicks automatically
- **Completely headless**: Zero visual output — your rendering closures control all styling
- **Zero dependencies** on other `egui-widgetkit` crates

## Quick Start

```rust
use egui_vim_nav::{FocusGraph, Navigator, VimKeyHandler, FocusRegion};

// 1. Build the focus graph (app defines the topology)
let mut graph = FocusGraph::new();
graph.connect_grid(&[
    &["a", "b", "c"],
    &["d", "e", "f"],
]);

// 2. Create a navigator (app owns this state)
let mut nav = Navigator::new().with_initial_focus("a");

// 3. In your egui update loop:
let handler = VimKeyHandler::new();
handler.handle_input(ctx, &mut nav, &graph);

// 4. Use FocusRegion to render focusable items
FocusRegion::new("a", &nav)
    .on_click(true)  // Click-to-focus enabled
    .show(ui, |ui, is_focused| {
        if is_focused {
            ui.label("I'm focused!");
        } else {
            ui.label("Click me or press H/J/K/L");
        }
    });
```

## API

### `FocusGraph<T>`

Defines the spatial topology:

| Method | Description |
|---|---|
| `FocusGraph::new()` | Create an empty graph |
| `.connect_horizontal(a, b)` | Connect a ←→ b (bidirectional left/right) |
| `.connect_vertical(a, b)` | Connect a ↕ b (bidirectional up/down) |
| `.connect_directed(from, to, dir)` | One-way connection in a specific direction |
| `.connect_grid(rows)` | Auto-wire a 2D grid from rows of IDs |
| `.get_neighbor(id, dir)` → `Option<&T>` | Query the neighbor in a direction |
| `.contains(id)` → `bool` | Check if a node exists |

### `Navigator<T>`

Tracks the currently focused node:

| Method | Description |
|---|---|
| `Navigator::new()` | Create with no initial focus |
| `.with_initial_focus(id)` | Set the starting focused node |
| `.focus()` → `Option<&T>` | Get the currently focused node |
| `.set_focus(id)` | Programmatically move focus |
| `.try_move(dir, graph)` → `bool` | Move in a direction if neighbor exists |

### `VimKeyHandler`

Translates keyboard input into navigation:

```rust
let handler = VimKeyHandler::new()
    .with_tab(true)      // Enable Tab/Shift+Tab cycling (default: false)
    .with_arrows(true);  // Enable arrow key navigation (default: true)

// In your update loop:
handler.handle_input(ctx, &mut nav, &graph);
```

**Key bindings:**
| Key | Direction | Modifier |
|---|---|---|
| `H` / `←` | Left | Direct or with Ctrl for inter-section |
| `J` / `↓` | Down | Direct or with Ctrl for inter-section |
| `K` / `↑` | Up | Direct or with Ctrl for inter-section |
| `L` / `→` | Right | Direct or with Ctrl for inter-section |
| `Tab` | Next | Optional (`.with_tab(true)`) |
| `Shift+Tab` | Previous | Optional |

> **Text input safety**: When any `TextEdit` has focus, HJKL keys are automatically suppressed to prevent navigation from interfering with typing.

### `FocusRegion`

Immediate-mode rendering helper:

```rust
FocusRegion::new("my-item", &nav)
    .on_click(true)              // Click anywhere in region to focus
    .on_secondary_click(true)    // Right-click also focuses
    .show(ui, |ui, is_focused| {
        // Render your item — is_focused tells you if it's the active one
        let bg = if is_focused { palette.surface1 } else { palette.base };
        // ...
    });
```

## Multi-Tier Navigation

For complex UIs with sections containing items, use two layers of graphs:

```rust
// Outer: sections (Ctrl + HJKL)
let mut section_graph = FocusGraph::new();
section_graph.connect_horizontal(Section::Left, Section::Right);

let mut section_nav = Navigator::new().with_initial_focus(Section::Left);

// Inner: items within each section (HJKL)
let mut item_graph = FocusGraph::new();
item_graph.connect_grid(&[&["a", "b"], &["c", "d"]]);

let mut item_nav = Navigator::new().with_initial_focus("a");

// Use Ctrl modifier to distinguish section vs item navigation
```

## Running the Example

```bash
cargo run -p egui-vim-nav --example grid_navigation
```
