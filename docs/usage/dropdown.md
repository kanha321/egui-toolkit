# Dropdown

## What It Is

`Dropdown` is a **spring-animated in-place morphing dropdown menu** with trajectory morphing, 2-tier highlights (saved selection + sliding focus pill), custom row renderers, scrolloff keyboard navigation, and selection-anchored expansion. The closed state looks like a button that smoothly expands into a scrollable item list.

**Module:** `egui-widgets` → `dropdown`

---

## What It Does

- **In-place morphing**: The button smoothly expands into the dropdown panel (no separate popup/overlay window).
- **Spring-animated expansion**: Open/close is driven by spring physics with smooth height and chevron rotation transitions.
- **2-tier highlights**: A persistent "saved" highlight sits on the currently selected item, while a separate "focus" pill slides between items as you navigate with J/K keys.
- **Scrolloff navigation**: The visible items scroll to keep the highlighted item within comfort margins — just like Vim's scrolloff.
- **Selection-anchored expansion**: When opening, the scroll position resets to frame the selected item in the 2nd visible slot.
- **Custom item rendering**: Supply your own renderer for complex row layouts.
- **Rich item options**: Each `DropdownOption` supports label, icon, subtitle, description, badge, status dot, and disabled state.
- **Mouse + keyboard**: Full mouse interaction plus J/K navigation, Enter to select, Q/Escape to close.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::{Dropdown, DropdownOption};

#[derive(Clone, PartialEq)]
enum Theme { Mocha, Macchiato, Frappe, Latte }

let mut selected = Theme::Mocha;

let options = vec![
    DropdownOption::new(Theme::Mocha, "Catppuccin Mocha"),
    DropdownOption::new(Theme::Macchiato, "Catppuccin Macchiato"),
    DropdownOption::new(Theme::Frappe, "Catppuccin Frappé"),
    DropdownOption::new(Theme::Latte, "Catppuccin Latte"),
];

Dropdown::new(&mut selected, options)
    .show(ui);
```

### Rich Options

```rust
let options = vec![
    DropdownOption::new(Status::Online, "Online")
        .icon("🟢")
        .subtitle("Available for chat")
        .badge("3")
        .status_dot(Color32::GREEN),

    DropdownOption::new(Status::Away, "Away")
        .icon("🟡")
        .subtitle("Back in 5 min"),

    DropdownOption::new(Status::Offline, "Offline")
        .icon("⚫")
        .disabled(true),
];

Dropdown::new(&mut status, options)
    .placeholder("Select status")
    .show(ui);
```

### Button Variants and Sizes

```rust
Dropdown::new(&mut selected, options)
    .primary()       // or .secondary(), .ghost(), .outline()
    .small()         // or .large()
    .icon("⚙")
    .show(ui);
```

### Customizing the Dropdown

```rust
Dropdown::new(&mut selected, options)
    .width(250.0)                    // dropdown width
    .max_visible_items(8)            // max items before scrolling
    .item_height(36.0)               // row height
    .show_chevron(true)              // show ▼ indicator
    .chevron_icon("▾")               // custom chevron character
    .auto_scroll(true)               // auto-scroll on keyboard nav
    .rounding(8.0)
    .padding(Vec2::new(12.0, 8.0))
    .show(ui);
```

### Custom Item Renderer

```rust
Dropdown::new(&mut selected, options)
    .item_renderer(|ui, ctx| {
        ui.horizontal(|ui| {
            if let Some(icon) = &ctx.option.icon {
                ui.label(icon.clone());
            }
            ui.vertical(|ui| {
                let color = if ctx.is_highlighted {
                    Color32::WHITE
                } else {
                    Color32::GRAY
                };
                ui.colored_label(color, ctx.option.label.clone());
                if let Some(sub) = &ctx.option.subtitle {
                    ui.small(sub.text());
                }
            });
        });
    })
    .show(ui);
```

### Keyboard Navigation

```rust
Dropdown::new(&mut selected, options)
    .focused(is_focused)          // show focus border
    .pressed(is_vim_pressed)      // visual press state
    .triggered(was_vim_clicked)   // toggle open/close
    .show(ui);
```

### Close Behavior

```rust
Dropdown::new(&mut selected, options)
    .close_on_outside_click(false)   // default: false
    .close_on_focus_lost(false)      // default: false
    .show_internal_focus(false)      // default: false
    .show(ui);

// Dropdown closes via:
// - Clicking an item (selects it)
// - Pressing Q, Escape, or Ctrl+C
// - Calling state.close() programmatically
```

### State Management

```rust
// Store in app state
let mut dd_state = DropdownState::new();

let resp = Dropdown::new(&mut selected, options)
    .with_state(&mut dd_state)
    .show(ui);

// Programmatic control
dd_state.open();
dd_state.close();
dd_state.toggle();
dd_state.set_open(true);

// Query
if resp.is_open() { /* dropdown is expanded */ }
if resp.changed() { /* selection changed */ }
if resp.just_opened() { /* just expanded this frame */ }
if resp.is_navigating() { /* user is keyboard-navigating items */ }
```

### DropdownResponse Fields

```rust
let resp = Dropdown::new(&mut selected, options).show(ui);

resp.changed();          // selection changed
resp.is_open();          // currently expanded
resp.clicked();          // button was clicked
resp.is_pressed();       // button is held
resp.just_opened();      // expanded this frame
resp.is_navigating();    // keyboard nav active
resp.highlight_rect();   // current focus highlight rect
resp.expanded_rect();    // full expanded dropdown rect
resp.is_fully_visible(); // no clipping
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Dropdown::new(&mut T, options)` | Create dropdown |
| `.placeholder(text)` | Placeholder when no selection |
| `.icon(text)` | Button leading icon |
| `.primary()` / `.secondary()` / `.ghost()` / `.outline()` | Button variant |
| `.small()` / `.large()` / `.size(ButtonSize)` | Size preset |
| `.width(f32)` | Dropdown width |
| `.max_visible_items(usize)` | Max items before scroll |
| `.item_height(f32)` | Row height |
| `.show_chevron(bool)` | Show dropdown indicator |
| `.chevron_icon(str)` | Custom chevron character |
| `.auto_scroll(bool)` | Auto-scroll on keyboard nav |
| `.item_renderer(fn)` | Custom row renderer |
| `.close_on_outside_click(bool)` | Close on outside click |
| `.close_on_focus_lost(bool)` | Close on focus loss |
| `.show_internal_focus(bool)` | Show internal focus ring |
| `.rounding(r)` / `.padding(v)` | Geometry |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut DropdownState)` | Explicit state |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> DropdownResponse<T>` | Render |

### `DropdownOption<T>`

| Method | Description |
| :--- | :--- |
| `DropdownOption::new(value, label)` | Create option |
| `.icon(text)` | Leading icon |
| `.subtitle(text)` | Subtitle below label |
| `.description(text)` | Longer description |
| `.badge(text)` | Badge text |
| `.status_dot(color)` | Status dot indicator |
| `.badge_color(color)` / `.icon_color(color)` | Custom colors |
| `.disabled(bool)` | Disable this option |

### `DropdownState`

| Method | Description |
| :--- | :--- |
| `DropdownState::new()` | Create state |
| `.open()` / `.close()` / `.toggle()` / `.set_open(bool)` | Open/close control |
| `.trigger_click()` / `.trigger_focus_bounce()` | Animation triggers |
| `.is_settled() -> bool` | At rest check |
