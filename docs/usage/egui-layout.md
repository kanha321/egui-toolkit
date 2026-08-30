# egui-layout

## What It Is

`egui-layout` is a **responsive nested split layout engine** for `egui`. It lets you declaratively divide available screen space into horizontal or vertical sections with proportional fractions, fixed pixel sizes, and min/max constraints — then nest splits inside each other to build complex IDE-like panel arrangements.

**Crate path:** `src/egui-widgetkit/egui-layout/`

---

## What It Does

- **Horizontal and vertical splits**: Place sections side-by-side (columns) or stacked (rows).
- **Three sizing policies**: `Fraction` (proportional with optional min/max), `Exact` (fixed pixel), and `Remainder` (fills leftover space).
- **Multi-pass constraint solver**: Handles min/max clamping with iterative redistribution — no 1px gap defects.
- **Built-in card framing**: Any section can auto-render as a styled card with title, subtitle, background, stroke, rounding, and padding.
- **Deep nesting**: Splits can contain splits, creating complex multi-pane layouts.
- **Window minimum size enforcement**: Automatically calculates and enforces the minimum window size from layout constraints.
- **Visual style cascade**: Section overrides → SplitStyle values → `ui.visuals()` fallbacks.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
egui-layout = { path = "../egui-layout" }
```

---

### Core Types

#### `Size` — Sizing Policy

Controls how a section claims space along the primary axis.

```rust
pub enum Size {
    Fraction { fraction: f32, min: Option<f32>, max: Option<f32> },
    Exact(f32),
    Remainder { min: Option<f32> },
}
```

**Constructors:**

```rust
Size::fraction(0.3)                   // 30% of flexible space
Size::fraction(0.3).min_size(150.0)   // 30%, but at least 150px
Size::fraction(0.3).max_size(400.0)   // 30%, but at most 400px
Size::exact(250.0)                    // always exactly 250px
Size::remainder()                     // takes all remaining space
Size::remainder().min_size(200.0)     // remainder, at least 200px
```

---

#### `Split` — Layout Builder

The main entry point. Build a layout by chaining sections.

**Basic 2-column layout:**

```rust
use egui_layout::Split;

Split::horizontal()
    .spacing(8.0)
    .section(0.30, |ui| {
        ui.label("Sidebar (30%)");
    })
    .section(0.70, |ui| {
        ui.label("Main content (70%)");
    })
    .show(ui);
```

**Vertical layout with a fixed header:**

```rust
Split::vertical()
    .section_fixed(44.0, |ui| {
        ui.heading("Dashboard Header");
    })
    .section_remainder(|ui| {
        ui.label("Scrollable content area");
    })
    .show(ui);
```

**Section shorthand methods:**

```rust
Split::horizontal()
    // Proportional section (fraction weight)
    .section(0.5, |ui| { /* ... */ })

    // Proportional with minimum constraint
    .section_min(0.3, 180.0, |ui| { /* ... */ })

    // Proportional with min AND max constraints
    .section_constrained(0.3, 150.0, 400.0, |ui| { /* ... */ })

    // Fixed pixel size
    .section_fixed(250.0, |ui| { /* ... */ })

    // Fills all remaining space
    .section_remainder(|ui| { /* ... */ })

    // Custom Size enum
    .section_custom(Size::fraction(0.2).min_size(100.0), |ui| { /* ... */ })

    .show(ui);
```

---

#### `Section` — Full Builder

For maximum control, use the `Section` builder directly:

```rust
use egui_layout::{Section, Split};

Split::horizontal()
    .section_with(|s| {
        s.fraction(0.25)
            .min_size(180.0)
            .card()
            .title("Navigation")
            .subtitle("Project files")
            .bg(Color32::from_rgb(30, 32, 48))
            .rounding(8.0)
            .padding(10.0)
            .on_rect(|rect| { /* track allocated bounds */ })
            .content(|ui| {
                ui.label("Explorer tree...");
            })
    })
    .section_remainder(|ui| {
        ui.label("Main editor area");
    })
    .show(ui);
```

**Section builder methods:**

| Method | Description |
| :--- | :--- |
| `.fraction(f)` / `.fixed(px)` / `.remainder()` | Sizing shorthand constructors |
| `.size(Size::...)` | Set sizing policy explicitly |
| `.min_size(px)` | Minimum along primary axis |
| `.max_size(px)` | Maximum along primary axis |
| `.min_cross(px)` | Minimum along cross axis |
| `.max_cross(px)` | Maximum along cross axis |
| `.card()` | Enable card framing (background + border) |
| `.title("...")` / `.subtitle("...")` | Card header text |
| `.title_color(c)` / `.subtitle_color(c)` | Header text colors |
| `.bg(color)` | Card background fill |
| `.stroke(stroke)` | Card border stroke |
| `.rounding(r)` | Card corner rounding |
| `.padding(px)` | Card inner padding |
| `.content(\|ui\| { ... })` | Section content closure |
| `.on_rect(\|rect\| { ... })` | Callback receiving allocated `Rect` |

---

#### `SplitStyle` — Global Styling

Configure defaults for all sections in a split:

```rust
use egui_layout::SplitStyle;

let style = SplitStyle::default()
    .with_spacing(6.0)
    .with_card_bg(Color32::from_rgb(25, 27, 35))
    .with_card_stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 75)))
    .with_card_rounding(8.0)
    .with_card_padding(10.0)
    .with_title_color(Color32::WHITE)
    .with_subtitle_color(Color32::GRAY)
    .with_title_size(14.0)
    .with_subtitle_size(11.0)
    .with_frame_bg(Color32::from_rgb(18, 19, 26))
    .with_frame_rounding(12.0)
    .with_frame_padding(6.0);

Split::horizontal()
    .style(style)
    .section(0.3, |ui| { /* ... */ })
    .section(0.7, |ui| { /* ... */ })
    .show(ui);
```

---

### Nested Layouts

Splits compose naturally:

```rust
Split::horizontal()
    .spacing(6.0)
    .section_with(|s| {
        s.fraction(0.25).min_size(180.0).card().title("Sidebar")
            .content(|ui| { ui.label("Nav items"); })
    })
    .section_remainder(|ui| {
        // Nest a vertical split inside the main content area
        Split::vertical()
            .spacing(6.0)
            .section_fixed(44.0, |ui| {
                ui.heading("Header Bar");
            })
            .section_with(|s| {
                s.remainder().card().title("Dashboard")
                    .content(|ui| { ui.label("Widgets here"); })
            })
            .show(ui);
    })
    .show(ui);
```

---

### Batch Section Creation

```rust
// N equal-width sections
Split::horizontal()
    .sections_equal(4, |index, ui| {
        ui.label(format!("Panel {}", index));
    })
    .show(ui);

// Weighted proportional sections
Split::horizontal()
    .sections_proportional(&[1.0, 2.0, 1.0], |index, frac, ui| {
        ui.label(format!("Section {} ({}%)", index, frac * 100.0));
    })
    .show(ui);
```

---

### Window Minimum Size Enforcement

```rust
let layout = Split::horizontal()
    .section_min(0.3, 150.0, |ui| { /* sidebar */ })
    .section_min(0.7, 300.0, |ui| { /* main */ });

// Set the window's minimum size from layout constraints
layout.enforce_min_size(ctx);
layout.show(ui);
```

---

### API Reference

#### `Size`

| Method | Signature | Description |
| :--- | :--- | :--- |
| `fraction` | `fn fraction(f: f32) -> Self` | Proportional weight |
| `exact` | `fn exact(px: f32) -> Self` | Fixed pixel size |
| `remainder` | `fn remainder() -> Self` | Fills remaining space |
| `min_size` | `fn min_size(self, px: f32) -> Self` | Set minimum bound |
| `max_size` | `fn max_size(self, px: f32) -> Self` | Set maximum bound |

#### `Split`

| Method | Description |
| :--- | :--- |
| `horizontal()` / `vertical()` | Create a horizontal or vertical split |
| `.style(SplitStyle)` | Set global styling |
| `.spacing(px)` | Inter-section gap |
| `.section(frac, \|ui\| ...)` | Add proportional section |
| `.section_min(frac, min, \|ui\| ...)` | Proportional with minimum |
| `.section_constrained(frac, min, max, \|ui\| ...)` | Proportional with min+max |
| `.section_fixed(px, \|ui\| ...)` | Fixed-size section |
| `.section_remainder(\|ui\| ...)` | Remainder section |
| `.section_card(frac, \|s\| ...)` | Card-framed section |
| `.section_with(\|s\| ...)` | Full Section builder |
| `.add_section(Section)` | Add pre-built Section |
| `.sections_equal(n, \|i, ui\| ...)` | N equal sections |
| `.sections_proportional(&[f], \|i, f, ui\| ...)` | Weighted sections |
| `.enforce_min_size(ctx)` | Set window min size |
| `.show(ui) -> Response` | Render the layout |
