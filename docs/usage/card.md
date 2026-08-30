# Card

## What It Is

`Card` is a **theme-aware container widget** with optional spring focus lift, soft ambient drop shadow, interactive press bounce, and built-in title/subtitle header. Use it to group related content into visually distinct panels.

**Module:** `egui-widgets` → `card`

---

## What It Does

- **Container framing**: Background fill, border stroke, rounding, and inner padding — all theme-derived or custom.
- **Focus lift**: Optional spring-driven elevation effect on focus.
- **Press bounce**: Spring-driven squash animation on click.
- **Title + subtitle header**: Built-in card header with configurable text and colors.
- **Interactive mode**: Can act as a clickable card (like a list item or nav tile).
- **Focus ring**: Keyboard-focus highlight border for Vim navigation.

---

## How To Use It

### Basic Container

```rust
use egui_widgets::Card;

Card::new()
    .show(ui, |ui| {
        ui.label("Card content goes here");
    });
```

### With Title and Subtitle

```rust
Card::new()
    .title("System Metrics")
    .subtitle("Live CPU & Memory utilization")
    .show(ui, |ui| {
        ui.label("CPU: 14% | RAM: 3.2 GB");
    });
```

### Interactive Card

```rust
let (response, _) = Card::new()
    .title("Settings")
    .interactive(true)
    .show(ui, |ui| {
        ui.label("Click to open settings");
    });

if response.clicked() {
    // Navigate to settings
}
```

### With Keyboard Navigation

```rust
Card::new()
    .title("Panel")
    .focused(is_focused)
    .pressed(is_vim_pressed)
    .triggered(was_vim_clicked)
    .interactive(true)
    .show(ui, |ui| { /* ... */ });
```

### Custom Styling

```rust
Card::new()
    .fill(Color32::from_rgb(25, 28, 38))
    .highlight_fill(Color32::from_rgb(30, 34, 48))
    .stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 70)))
    .highlight_stroke(Stroke::new(1.5, accent_color))
    .rounding(12.0)
    .padding(Vec2::new(16.0, 12.0))
    .min_size(Vec2::new(200.0, 100.0))
    .focus_lift(4.0)
    .show(ui, |ui| { /* ... */ });
```

### With Theme Palette

```rust
Card::new()
    .palette(&palette)
    .title("Themed Card")
    .show(ui, |ui| { /* ... */ });
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Card::new()` | Create card |
| `.title(text)` / `.subtitle(text)` | Header text |
| `.title_color(c)` / `.subtitle_color(c)` | Header colors |
| `.fill(c)` / `.highlight_fill(c)` | Background colors |
| `.stroke(s)` / `.highlight_stroke(s)` | Border strokes |
| `.rounding(r)` / `.padding(v)` / `.min_size(v)` | Geometry |
| `.interactive(bool)` | Enable click/hover |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation state |
| `.focus_lift(f32)` | Elevation on focus (px) |
| `.spring_params(p)` | Physics parameters |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `CardState` |
| `.id_source(id)` | Custom ID |
| `.show(ui, \|ui\| ...) -> (Response, R)` | Render with content |
