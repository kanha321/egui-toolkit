# Checkbox & RadioButton

## What It Is

`Checkbox` and `RadioButton` are **spring-animated selection controls**. Checkbox has a scale-pop checkmark animation, and RadioButton has a dot-pop fill animation. Both support keyboard navigation, theme-aware colors, and explicit state management.

**Module:** `egui-widgets` → `checkbox`

---

## What It Does

### Checkbox
- **Animated checkmark**: Spring-driven scale pop when toggling on/off.
- **Press bounce**: Visual squash on click.
- **Focus bounce**: Spring ring animation when keyboard-focused.
- **Custom colors**: Per-state fill, stroke, and checkmark colors.

### RadioButton
- **Animated dot**: Spring-driven dot pop when selected/deselected.
- **Nullable selection**: Supports `Option<T>` for optional/deselectable radio groups.
- **Group management**: Bind multiple radios to the same `&mut T` or `&mut Option<T>`.
- **Focus bounce**: Same as checkbox.

---

## How To Use It

### Checkbox — Basic

```rust
use egui_widgets::Checkbox;

let mut checked = false;

Checkbox::new(&mut checked)
    .label("Enable notifications")
    .show(ui);
```

### Checkbox — Custom Styling

```rust
Checkbox::new(&mut checked)
    .label("Custom")
    .box_size(18.0)
    .rounding(4.0)
    .fill_checked(Color32::from_rgb(137, 180, 250))
    .fill_unchecked(Color32::from_rgb(40, 42, 54))
    .stroke_checked(Stroke::new(1.5, Color32::from_rgb(137, 180, 250)))
    .checkmark_color(Color32::WHITE)
    .show(ui);
```

### Checkbox — Keyboard Navigation

```rust
Checkbox::new(&mut checked)
    .label("Toggleable")
    .focused(is_focused)       // show focus ring
    .pressed(is_vim_pressed)   // visual press
    .triggered(was_clicked)    // toggle on vim click
    .show(ui);
```

### Checkbox — With Theme Palette

```rust
Checkbox::new(&mut checked)
    .label("Themed")
    .palette(&palette)
    .show(ui);
```

---

### RadioButton — Basic

```rust
use egui_widgets::RadioButton;

#[derive(Clone, PartialEq)]
enum Choice { A, B, C }

let mut selected = Choice::A;

RadioButton::new(Choice::A, &mut selected)
    .label("Option A")
    .show(ui);

RadioButton::new(Choice::B, &mut selected)
    .label("Option B")
    .show(ui);

RadioButton::new(Choice::C, &mut selected)
    .label("Option C")
    .show(ui);
```

### RadioButton — Nullable (Optional Selection)

```rust
let mut selected: Option<Choice> = None;

// Clicking the selected radio deselects it (sets to None)
RadioButton::nullable(Choice::A, &mut selected)
    .label("Option A")
    .show(ui);

RadioButton::nullable(Choice::B, &mut selected)
    .label("Option B")
    .show(ui);
```

### RadioButton — Optional (With Explicit Deselect Control)

```rust
let mut selected: Option<Choice> = Some(Choice::A);

RadioButton::optional(Choice::A, &mut selected)
    .allow_deselect(true)   // clicking selected radio toggles to None
    .label("Option A")
    .show(ui);
```

### RadioButton — Keyboard Navigation

```rust
RadioButton::new(Choice::A, &mut selected)
    .label("Option A")
    .focused(is_focused)
    .pressed(is_vim_pressed)
    .triggered(was_clicked)
    .show(ui);
```

---

### Builder Reference

#### Checkbox

| Method | Description |
| :--- | :--- |
| `Checkbox::new(&mut bool)` | Create checkbox |
| `.label(text)` | Label text |
| `.box_size(f32)` | Checkbox square size |
| `.rounding(r)` | Corner rounding |
| `.fill_checked(c)` / `.fill_unchecked(c)` | Background per state |
| `.stroke_checked(s)` / `.stroke_unchecked(s)` | Border per state |
| `.checkmark_color(c)` | Checkmark color |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `CheckboxState` |
| `.show(ui) -> CheckboxResponse` | Render |

#### RadioButton

| Method | Description |
| :--- | :--- |
| `RadioButton::new(value, &mut T)` | Create radio (required selection) |
| `RadioButton::nullable(value, &mut Option<T>)` | Deselectable radio |
| `RadioButton::optional(value, &mut Option<T>)` | Optional with control |
| `.allow_deselect(bool)` | Allow clicking selected to deselect |
| `.label(text)` | Label text |
| `.radius(f32)` | Circle radius |
| `.focused(bool)` / `.pressed(bool)` / `.triggered(bool)` | Navigation |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `RadioState` |
| `.show(ui) -> RadioResponse` | Render |
