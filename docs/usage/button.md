# Button

## What It Is

`Button` is a **spring-animated button widget** with press bounce, focus luminance glide, semantic variants (Primary, Secondary, Ghost, Danger, etc.), size presets, icon/shortcut/badge slots, and full keyboard navigation support.

**Module:** `egui-widgets` → `button`

---

## What It Does

- **Press bounce**: Springs compress on click, creating a tactile pop effect.
- **Focus animation**: Luminance glides up on focus, with a bounce on first focus entry.
- **Semantic variants**: Primary (accent fill), Secondary (surface), Ghost (transparent), Danger (red), Outline (bordered), Success, Warning — each with coordinated fill/stroke/text colors derived from the active theme palette.
- **Size presets**: Small, Medium, Large, or Custom padding/font.
- **Optional slots**: Leading icon, trailing shortcut hint, and notification badge.
- **State ownership**: Use the built-in ID-scoped memory, or pass your own `ButtonState` for explicit control.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::Button;

if Button::new("Click Me").show(ui).clicked() {
    println!("Button clicked!");
}
```

### Variants

```rust
Button::new("Save").primary().show(ui);
Button::new("Cancel").secondary().show(ui);
Button::new("Clear").ghost().show(ui);
Button::new("Delete").danger().show(ui);
Button::new("Export").outline().show(ui);
Button::new("Apply").success().show(ui);
Button::new("Caution").warning().show(ui);
```

### Sizes

```rust
Button::new("Small").small().show(ui);
Button::new("Medium").medium().show(ui);   // default
Button::new("Large").large().show(ui);
```

### With Icon, Shortcut, and Badge

```rust
Button::new("Search")
    .icon("🔍")
    .shortcut("Ctrl+K")
    .badge("3")
    .primary()
    .show(ui);
```

### Keyboard Navigation Integration

```rust
Button::new("Confirm")
    .focused(is_this_widget_focused)  // highlight border when focused
    .pressed(is_vim_key_held)         // visual press state
    .triggered(was_vim_clicked)       // trigger press bounce
    .show(ui);
```

### Custom Styling

```rust
Button::new("Custom")
    .fill(Color32::from_rgb(30, 40, 60))
    .highlight_fill(Color32::from_rgb(40, 55, 80))
    .active_fill(Color32::from_rgb(20, 30, 50))
    .stroke(Stroke::new(1.0, Color32::GRAY))
    .text_color(Color32::WHITE)
    .rounding(12.0)
    .padding(Vec2::new(20.0, 10.0))
    .min_size(Vec2::new(120.0, 0.0))
    .show(ui);
```

### With Theme Palette

```rust
Button::new("Themed")
    .palette(&palette)
    .primary()
    .show(ui);
```

### Explicit State Management

```rust
// Store in your app state
let mut btn_state = ButtonState::new();

Button::new("Stateful")
    .with_state(&mut btn_state)
    .show(ui);

// Query state
if btn_state.is_settled() { /* animation done */ }
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Button::new(text)` | Create button |
| `.primary()` / `.secondary()` / `.ghost()` / `.danger()` / `.outline()` / `.success()` / `.warning()` | Semantic variant |
| `.small()` / `.medium()` / `.large()` / `.size(ButtonSize)` | Size preset |
| `.icon(text)` | Leading icon |
| `.shortcut(text)` | Trailing shortcut hint |
| `.badge(text)` | Notification badge |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation state |
| `.fill(c)` / `.highlight_fill(c)` / `.active_fill(c)` | Custom fill colors |
| `.stroke(s)` / `.highlight_stroke(s)` / `.active_stroke(s)` | Custom strokes |
| `.text_color(c)` | Text color |
| `.rounding(r)` / `.padding(v)` / `.min_size(v)` | Geometry |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit state |
| `.id_source(id)` | Custom ID for state storage |
| `.show(ui) -> ButtonResponse` | Render |

### `ButtonResponse`

| Method | Description |
| :--- | :--- |
| `.clicked()` | Was clicked this frame |
| `.is_pressed()` | Currently held down |
| `.is_held()` | Alias for is_pressed |
| `.into_inner()` | Unwrap to `egui::Response` |
