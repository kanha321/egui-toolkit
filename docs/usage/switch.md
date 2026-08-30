# Switch

## What It Is

`Switch` is a **fluid spring toggle switch** with squash & stretch thumb physics, track color crossfading, and keyboard interaction. Think iOS/Material toggle but with analytical spring physics.

**Module:** `egui-widgets` → `switch`

---

## What It Does

- **Spring thumb travel**: The thumb slides between on/off positions with spring overshoot.
- **Squash & stretch**: The thumb squashes on press and bounces back on release.
- **Track crossfade**: Track color smoothly transitions between on/off colors.
- **Focus ring**: Spring-animated focus bounce for keyboard navigation.
- **Size presets**: Compact, Standard, Large, or Custom dimensions.
- **Theme-aware**: Auto-derives on/off colors from palette accent and surface tokens.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::Switch;

let mut enabled = true;

Switch::new(&mut enabled)
    .label("Dark Mode")
    .show(ui);
```

### Sizes

```rust
Switch::new(&mut enabled).compact().show(ui);
Switch::new(&mut enabled).standard().show(ui);  // default
Switch::new(&mut enabled).large().show(ui);
```

### Custom Colors

```rust
Switch::new(&mut enabled)
    .track_on(Color32::from_rgb(137, 180, 250))
    .track_off(Color32::from_rgb(60, 63, 80))
    .thumb_on(Color32::WHITE)
    .thumb_off(Color32::from_rgb(180, 180, 190))
    .track_stroke(Stroke::new(1.0, Color32::from_rgb(80, 85, 100)))
    .thumb_stroke(Stroke::NONE)
    .label_color(Color32::WHITE)
    .show(ui);
```

### Keyboard Navigation

```rust
Switch::new(&mut enabled)
    .label("Notifications")
    .focused(is_focused)
    .pressed(is_vim_pressed)
    .triggered(was_clicked)
    .show(ui);
```

### With Theme Palette

```rust
Switch::new(&mut enabled)
    .palette(&palette)
    .label("Auto-save")
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Switch::new(&mut bool)` | Create switch |
| `.label(text)` | Label text |
| `.compact()` / `.standard()` / `.large()` / `.size(SwitchSize)` | Size preset |
| `.track_on(c)` / `.track_off(c)` | Track colors |
| `.thumb_on(c)` / `.thumb_off(c)` | Thumb colors |
| `.track_stroke(s)` / `.thumb_stroke(s)` | Border strokes |
| `.label_color(c)` | Label text color |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `SwitchState` |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> SwitchResponse` | Render |

### `SwitchResponse`

| Method | Description |
| :--- | :--- |
| `.clicked()` | Toggled this frame |
| `.is_pressed()` | Currently held down |
| `.changed()` | Value changed |
