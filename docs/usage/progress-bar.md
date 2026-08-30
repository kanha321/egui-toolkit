# ProgressBar

## What It Is

`ProgressBar` is a **spring-animated progress bar** with physical catchup animation. The fill bar glides to the target value with spring physics, creating a smooth, satisfying progress visualization.

**Module:** `egui-widgets` → `progress`

---

## What It Does

- **Spring-animated fill**: The progress bar smoothly catches up to the target value with spring physics — no abrupt jumps.
- **Semantic variants**: Accent, Success, Warning, Danger, Info, or Custom color.
- **Percentage display**: Optional percentage text overlay.
- **Label**: Optional descriptive label above the bar.
- **Custom geometry**: Configurable height, rounding, track fill, and track stroke.

---

## How To Use It

### Basic Usage

```rust
use egui_widgets::ProgressBar;

ProgressBar::new(0.72)
    .show(ui);
```

### With Label and Percentage

```rust
ProgressBar::new(0.72)
    .label("Download Progress")
    .show_percentage(true)
    .show(ui);
```

### Semantic Variants

```rust
use egui_widgets::ProgressVariant;

ProgressBar::new(0.85).variant(ProgressVariant::Success).show(ui);
ProgressBar::new(0.45).variant(ProgressVariant::Warning).show(ui);
ProgressBar::new(0.20).variant(ProgressVariant::Danger).show(ui);
ProgressBar::new(0.60).variant(ProgressVariant::Info).show(ui);
```

### Custom Styling

```rust
ProgressBar::new(0.5)
    .height(8.0)
    .rounding(4.0)
    .fill(Color32::from_rgb(137, 180, 250))
    .track_fill(Color32::from_rgb(40, 42, 54))
    .track_stroke(Stroke::new(1.0, Color32::from_rgb(60, 63, 80)))
    .show(ui);
```

### With Theme Palette

```rust
ProgressBar::new(progress)
    .palette(&palette)
    .label("Upload")
    .show_percentage(true)
    .show(ui);
```

### Animated Updates

```rust
// The bar smoothly animates to each new value
// Just update the progress value each frame:
ProgressBar::new(current_download_progress)
    .label("Downloading...")
    .show(ui);
```

---

### Builder Reference

| Method | Description |
| :--- | :--- |
| `ProgressBar::new(progress: f32)` | Create bar (0.0–1.0) |
| `.label(text)` | Label text |
| `.show_percentage(bool)` | Show percentage text |
| `.variant(ProgressVariant)` | Semantic color variant |
| `.height(f32)` | Bar height |
| `.rounding(r)` | Corner rounding |
| `.fill(c)` | Fill bar color |
| `.track_fill(c)` | Track background color |
| `.track_stroke(s)` | Track border stroke |
| `.spring_params(p)` / `.motion(bool)` | Physics control |
| `.palette(&p)` | Theme palette |
| `.with_state(&mut ProgressState)` | Explicit state |
| `.id_source(id)` | Custom ID |
| `.show(ui) -> Response` | Render |
