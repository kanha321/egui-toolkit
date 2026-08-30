# Badge

## What It Is

`Badge` is a **semantic status indicator widget** — small colored chips/tags for showing status, counts, or labels. Supports `Outline` (subtle border) and `Solid` (filled) styles with 7 semantic variants and an optional status dot.

**Module:** `egui-widgets` → `badge`

---

## What It Does

- **Semantic variants**: Accent, Success, Warning, Danger, Info, Neutral, or Custom colors.
- **Two visual styles**: `Outline` (border + transparent fill) and `Solid` (filled background).
- **Optional status dot**: Small colored circle indicator before the text.
- **Spring animation**: Width/height smoothly animate when badge text changes.
- **Theme-aware**: Auto-derives colors from `ThemePalette`.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::Badge;

Badge::new("Online").show(ui);
```

### Variants

```rust
Badge::new("Primary").accent().show(ui);
Badge::new("Active").success().show(ui);
Badge::new("Pending").warning().show(ui);
Badge::new("Error").danger().show(ui);
Badge::new("Info").info().show(ui);
Badge::new("Draft").neutral().show(ui);
```

### Styles

```rust
Badge::new("Outline Style").outline().show(ui);  // default
Badge::new("Solid Style").solid().show(ui);
```

### With Status Dot

```rust
Badge::new("Online")
    .success()
    .dot(true)
    .show(ui);
```

### Custom Colors

```rust
use egui_widgets::BadgeVariant;

Badge::new("Custom")
    .variant(BadgeVariant::Custom {
        fill: Color32::from_rgb(30, 60, 90),
        text: Color32::WHITE,
    })
    .show(ui);
```

### With Theme Palette

```rust
Badge::new("Themed")
    .palette(&palette)
    .success()
    .dot(true)
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `Badge::new(text)` | Create badge |
| `.accent()` / `.success()` / `.warning()` / `.danger()` / `.info()` / `.neutral()` | Semantic variant |
| `.variant(BadgeVariant)` | Set variant explicitly |
| `.outline()` / `.solid()` | Visual style |
| `.dot(bool)` | Show/hide status dot |
| `.fill(c)` / `.stroke(s)` / `.text_color(c)` | Custom colors |
| `.rounding(r)` / `.padding(v)` | Geometry |
| `.motion(bool)` | Enable/disable animation |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut s)` | Explicit `BadgeState` |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> Response` | Render |
