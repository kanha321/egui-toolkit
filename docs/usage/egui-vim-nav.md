# egui-vim-nav

## What It Is

`egui-vim-nav` is a **Vim-style keyboard navigation and modal text editing system** for `egui`. It provides a generic focus graph for HJKL directional navigation between widgets, a 2-tier hierarchical section/widget navigation engine, scrolloff-aware viewport scrolling, Vim modal text editing with motions/operators/text objects, and a cursor autohide system.

**Crate path:** `src/egui-widgetkit/egui-vim-nav/`

---

## What It Does

- **Focus Graph**: A topology-agnostic graph where widgets are nodes and directional edges (up/down/left/right) define navigation paths. Supports grids, trees, and branching navigation.
- **Navigator**: Tracks the currently focused node and traverses the graph. Supports branch memory (remembering the last focused child in a sub-menu).
- **VimKeyHandler**: Maps keyboard inputs (HJKL, arrows, Ctrl+sections, Tab) to navigation directions and action events (click, back, forward).
- **Hierarchical Navigation**: 2-pass navigation — first try within the current section's widget graph, then fall back to cross-section navigation.
- **Modal Text Editing**: Full Vim buffer with Normal/Insert/Visual/Replace/OperatorPending modes, 25+ motions, 15+ operators, text objects, undo/redo, and registers.
- **Scrolloff**: Spring-animated viewport scrolling that keeps the focused widget within comfort margins.
- **Cursor Autohide**: Hides the mouse cursor when navigating via keyboard, restores it on mouse movement.
- **FocusRegion**: Immediate-mode helper that registers a widget as a focus target and reports hover/click/focus state.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
egui-vim-nav = { path = "../egui-vim-nav" }
```

---

### Focus Graph — Define Navigation Topology

Create a graph of navigable widgets:

```rust
use egui_vim_nav::{FocusGraph, Direction};

let mut graph = FocusGraph::new();

// Connect individual pairs
graph.connect_horizontal("sidebar", "main");   // sidebar ↔ main
graph.connect_vertical("header", "content");   // header ↔ content

// Or build a grid at once
graph.connect_grid(&[
    &["btn_a", "btn_b", "btn_c"],
    &["btn_d", "btn_e", "btn_f"],
]);
// Automatically connects all horizontal and vertical neighbors

// One-way connections
graph.connect_directed("search", "results", Direction::Down);
```

---

### Navigator — Track and Move Focus

```rust
use egui_vim_nav::Navigator;

let mut nav = Navigator::new()
    .with_initial_focus("btn_a");

// Read current focus
if let Some(focused) = nav.focused() {
    println!("Focused: {:?}", focused);
}

// Move focus programmatically
if let Some(event) = nav.move_focus(&graph, Direction::Right) {
    println!("Moved from {:?} to {:?}", event.previous, event.current);
}

// Set focus directly
nav.set_focus(Some("btn_e"));
```

---

### VimKeyHandler — Keyboard Input

Maps keyboard events to navigation directions and actions:

```rust
use egui_vim_nav::{VimKeyHandler, VimAction};

let key_handler = VimKeyHandler::new()
    .with_hjkl(true)           // HJKL for directions
    .with_arrows(true)         // Arrow keys
    .with_ctrl_sections(true)  // Ctrl+HJKL for section navigation
    .with_actions(true)        // F=click, D=secondary, Q=back
    .with_tab(true);           // Tab for forward navigation

// In your update loop:

// Navigation — moves focus in the graph
if let Some(event) = key_handler.handle_input(ctx, &mut nav, &graph) {
    // Focus changed
}

// Actions — click, back, forward
if let Some(action) = key_handler.handle_action(ctx) {
    match action {
        VimAction::PrimaryClick => { /* F, Enter, Space */ }
        VimAction::SecondaryClick => { /* D */ }
        VimAction::Back => { /* Q, Escape */ }
        VimAction::Forward => { /* mouse extra2 */ }
        VimAction::Enter => { /* legacy alias */ }
    }
}

// Detailed action state (press, hold, double-click, long-press)
let state = key_handler.primary_action_state(ctx);
if state.double_clicked { /* ... */ }
if state.long_pressed { /* ... */ }
```

---

### Hierarchical Navigation — Sections + Widgets

For apps with multiple sections (sidebar, header, content), use 2-pass navigation:

```rust
use egui_vim_nav::hierarchical_move;

// You need two graphs and two navigators:
// 1. Section-level: sidebar ↔ main ↔ header
// 2. Widget-level: individual buttons/inputs within each section

let result = hierarchical_move(
    direction,
    &mut widget_nav,     // Navigator<WidgetId>
    &widget_graph,       // FocusGraph<WidgetId>
    &mut section_nav,    // Navigator<SectionId>
    &section_graph,      // FocusGraph<SectionId>
    |widget_id| widget_id.to_section(),            // map widget → section
    |section_id, dir| section_id.entry_widget(dir), // section entry point
);

if let Some(result) = result {
    if result.crossed_section {
        // Navigated to a different section
    }
}
```

---

### Branch Navigation — Sub-Menus and Expandable Groups

```rust
use egui_vim_nav::BranchStrategy;

// Connect a parent node to a group of children
graph.connect_branch("menu", Direction::Down, &["item_a", "item_b", "item_c"]);

// With custom strategy for how to enter the branch:
graph.connect_branch_with_strategy(
    "menu", Direction::Down,
    &["item_a", "item_b", "item_c"],
    BranchStrategy::RememberLast,  // returns to last focused child
);

// Other strategies:
// BranchStrategy::First          — always enter on first child
// BranchStrategy::Last           — always enter on last child
// BranchStrategy::EdgeAware      — first or last depending on direction
// BranchStrategy::Anchor(child)  — always enter on specific child
```

---

### Scrolloff — Viewport Scrolling

Keeps the focused widget within comfort margins, auto-scrolling when focus approaches the viewport edge:

```rust
use egui_vim_nav::Scrolloff;

// Store in app state
let mut scrolloff = Scrolloff::new()
    .with_margins(60.0, 60.0)              // top/bottom comfort zones
    .with_autoscroll_on_pointer(false);     // don't scroll on mouse hover

// Record navigation events (call when focus changes)
scrolloff.record_nav_event(Some(focused_id), Some(Direction::Down));

// Inject into ScrollArea
let (scroll_area, offset) = scrolloff.inject_into(
    egui::ScrollArea::vertical(),
    dt,
);

let output = scroll_area.show(ui, |ui| {
    // ... render widgets ...
    // When focus changes, tell scrolloff about the target rect:
    scrolloff.adjust_for_target(target_rect, viewport_rect, content_height);
});

// Sync with manual mouse scrolling
scrolloff.sync_manual_scroll(&output);

// Request repaint while scrolling
scrolloff.request_repaint_if_needed(ctx);
```

---

### Modal Text Editing — Vim Buffer

Full Vim-style text editing with modes, motions, operators, and text objects:

```rust
use egui_vim_nav::{VimBufferState, VimMode};

// Store in app state
let mut vim_buf = VimBufferState::new("Hello, world!");

// Process keyboard input
vim_buf.handle_input(ctx);

// Query state
let text = vim_buf.text();
let cursor = vim_buf.cursor();
let mode = vim_buf.mode();
let selection = vim_buf.selection_range();

// Programmatic operations
vim_buf.insert_char('x');
vim_buf.backspace();
vim_buf.undo();
vim_buf.redo();
```

**Supported Vim features:**
- **Modes**: Normal, Insert, Visual (character/line/block), Replace, OperatorPending
- **Motions**: `h/j/k/l`, `w/e/b/W/E/B`, `0/^/$`, `f/t/F/T/;/,`, `gg/G`, `%`
- **Operators**: `d` (delete), `c` (change), `y` (yank), `p/P` (paste), `r` (replace char), `s` (substitute), `~` (toggle case), `gu/gU` (lower/upper), `>/<` (indent/outdent)
- **Text Objects**: `iw/aw` (word), `i"/a"` (quotes), `i(/a(` (parens), `i[/a[` (brackets), `i{/a{` (braces)
- **Undo/Redo**: Full history stack
- **Registers**: Unnamed, yank, and named registers
- **Visual mode operations**: Delete, change, yank, paste, join, indent, case toggle

---

### Modal Text Transition — Switching Between Navigation and Editing

```rust
use egui_vim_nav::{check_modal_transition, FocusLevel, ModalTransition};

let transition = check_modal_transition(
    current_focus_level,        // FocusLevel::Navigation or TextEditing
    is_on_text_widget,          // is the focused widget a text input?
    externally_entered_edit,    // external trigger to enter editing
    is_clean_normal_mode,       // vim buffer in normal mode, no pending keys
    externally_exited_edit,     // external trigger to exit editing
    section_nav_triggered,      // Ctrl+H/L triggered section jump
    ctx,
);

match transition {
    ModalTransition::EnteredText => { /* switch to editing mode */ }
    ModalTransition::ExitedText  => { /* switch to navigation mode */ }
    ModalTransition::None        => { /* no change */ }
}
```

---

### Cursor Autohide

Hides the OS cursor when navigating via keyboard:

```rust
use egui_vim_nav::CursorAutohide;

let mut cursor_hide = CursorAutohide::new();

// Each frame:
cursor_hide.update(ctx);

// Query
if egui_vim_nav::is_cursor_hidden(ctx) {
    // keyboard navigation mode — cursor is hidden
}
```

---

### FocusRegion — Widget Registration Helper

Wraps a widget and reports whether it's focused, hovered, or clicked:

```rust
use egui_vim_nav::{FocusRegion, Navigator};

let resp = FocusRegion::show(ui, &mut nav, &widget_id, |ui, is_focused| {
    // Render your widget, using is_focused for styling
    ui.label(if is_focused { "★ Focused!" } else { "Normal" })
});

if resp.is_focused { /* highlight this widget */ }
if resp.is_clicked { /* user clicked it */ }
if let Some(rect) = resp.highlight_rect() {
    // Use this rect as the target for SpringRect/HighlightGroup
}
```

---

### API Reference

#### `FocusGraph<T>`

| Method | Description |
| :--- | :--- |
| `new()` | Empty graph |
| `connect_horizontal(left, right)` | Bidirectional H link |
| `connect_vertical(top, bottom)` | Bidirectional V link |
| `connect_directed(from, to, dir)` | One-way link |
| `connect_grid(&[&[T]])` | Build a full grid |
| `connect_branch(parent, dir, children)` | Branch group |
| `get_neighbor(node, dir) -> Option<&T>` | Query neighbor |
| `contains(node) -> bool` | Node exists? |

#### `Navigator<T>`

| Method | Description |
| :--- | :--- |
| `new()` | Create navigator |
| `.with_initial_focus(node)` | Set starting focus |
| `focused() -> Option<&T>` | Current focus |
| `set_focus(Option<T>)` | Set focus directly |
| `move_focus(graph, dir) -> Option<FocusEvent>` | Traverse graph |

#### `VimKeyHandler`

| Method | Description |
| :--- | :--- |
| `new()` | Create handler |
| `.with_hjkl(bool)` | Enable/disable HJKL |
| `.with_arrows(bool)` | Enable/disable arrow keys |
| `handle_input(ctx, nav, graph) -> Option<FocusEvent>` | Process navigation |
| `handle_action(ctx) -> Option<VimAction>` | Process actions |

#### `Scrolloff<T>`

| Method | Description |
| :--- | :--- |
| `new()` | Create scrolloff |
| `.with_margins(top, bottom)` | Set comfort zones |
| `record_nav_event(focused, dir)` | Record focus change |
| `inject_into(scroll_area, dt)` | Apply to ScrollArea |
| `adjust_for_target(rect, viewport, height)` | Scroll to target |
| `is_settled() -> bool` | At rest check |
