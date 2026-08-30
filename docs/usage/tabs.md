# SegmentedTabs

## What It Is

`SegmentedTabs` is a **spring-animated segmented tab bar** with a continuously sliding selection pill. When the user switches tabs, the highlight pill glides smoothly to the new position with spring physics, including velocity-injected bounce on click.

**Module:** `egui-widgets` → `tabs`

---

## What It Does

- **Sliding pill**: The selection indicator glides between tabs with spring physics — including overshoot bounce on rapid clicks.
- **Press squash**: The pill compresses vertically on press and bounces back on release.
- **Tab items with slots**: Each tab can have a text label, leading icon, and badge.
- **Theme-aware**: Container, pill, and text colors auto-derive from `ThemePalette`.
- **Keyboard navigation**: Supports focused/pressed/triggered states for Vim navigation.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::{SegmentedTabs, TabItem};

#[derive(Clone, Copy, PartialEq)]
enum ViewMode { Grid, List, Table }

let mut mode = ViewMode::Grid;

SegmentedTabs::new(&mut mode)
    .tab(ViewMode::Grid, "Grid View")
    .tab(ViewMode::List, "List View")
    .tab(ViewMode::Table, "Table View")
    .show(ui);
```

### With Icons and Badges

```rust
SegmentedTabs::new(&mut mode)
    .item(TabItem::new(ViewMode::Grid, "Grid").icon("⊞"))
    .item(TabItem::new(ViewMode::List, "List").icon("≡"))
    .item(TabItem::new(ViewMode::Table, "Table").icon("▤").badge("3"))
    .show(ui);
```

### Custom Styling

```rust
SegmentedTabs::new(&mut mode)
    .tab(ViewMode::Grid, "Grid")
    .tab(ViewMode::List, "List")
    .height(40.0)
    .rounding(8.0)
    .container_fill(Color32::from_rgb(30, 33, 45))
    .container_stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 70)))
    .pill_fill(Color32::from_rgb(45, 48, 65))
    .pill_stroke(Stroke::new(1.0, Color32::from_rgb(80, 85, 100)))
    .active_text_color(Color32::WHITE)
    .inactive_text_color(Color32::from_rgb(140, 145, 160))
    .show(ui);
```

### Keyboard Navigation

```rust
SegmentedTabs::new(&mut mode)
    .tab(ViewMode::Grid, "Grid")
    .tab(ViewMode::List, "List")
    .focused(is_focused)
    .pressed(is_vim_pressed)
    .triggered(was_clicked)
    .show(ui);
```

### With Theme Palette

```rust
SegmentedTabs::new(&mut mode)
    .tab(ViewMode::Grid, "Grid")
    .tab(ViewMode::List, "List")
    .palette(&palette)
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `SegmentedTabs::new(&mut T)` | Create tabs bound to selection |
| `.tab(value, label)` | Add a tab with value + label |
| `.item(TabItem)` | Add a fully configured tab item |
| `.height(f32)` | Container height (default: 36) |
| `.rounding(r)` | Corner rounding |
| `.container_fill(c)` / `.container_stroke(s)` | Outer container styling |
| `.pill_fill(c)` / `.pill_stroke(s)` | Selection pill styling |
| `.active_text_color(c)` / `.inactive_text_color(c)` | Tab text colors |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation state |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut TabsState)` | Explicit state |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> TabsResponse` | Render |

### `TabItem<T>`

| Method | Description |
| :--- | :--- |
| `TabItem::new(value, label)` | Create tab item |
| `.icon(text)` | Leading icon |
| `.badge(text)` | Notification badge |

### `TabsResponse`

| Method | Description |
| :--- | :--- |
| `.clicked()` | Tab was clicked |
| `.is_pressed()` | Currently held |
| `.changed()` | Selection changed this frame |
