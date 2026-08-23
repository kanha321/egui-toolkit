# Part 1 — `egui-layout` (Declarative Layout & Sizing Engine)

## Overview
`egui-layout` is a responsive, constraint-aware, nested split layout engine for `egui`. It automatically subdivides available screen space among child sections without requiring hardcoded pixel breakpoints, manual resize handlers, or fixed aspect ratios.

It provides a declarative builder API (`Section<'a>` and `Split<'a>`) that simplifies creating, styling, and constraining any number or type of sections while automatically computing 2D layout boundaries (`Vec2`) to protect the window from clipping.

---

## What It Does

### 1. Declarative Section Builder (`Section<'a>`)
- Represents a single configurable layout section:
  - **Sizing Policies**: `Section::fraction(f32)`, `Section::fixed(px)`, `Section::remainder()`, or `.size(Size)`.
  - **2D Constraints**: `.min_size(px)` (primary axis), `.min_cross(px)` (cross axis), `.min_size_2d(primary_min, cross_min)`, `.max_size(px)`, `.max_cross(px)`.
  - **Visual Card Framing (Optional)**: `.card()` automatically renders background fill, border stroke, 4 intact rounded corners, and inner margin padding.
  - **Header Titles**: `.title(&str)`, `.subtitle(&str)`, `.title_color(Color32)`.
  - **Content Closures**: `.content(impl FnOnce(&mut Ui))` and `.on_rect(impl FnOnce(Rect))`.

### 2. Responsive Proportional Splits (`Split::horizontal()` / `Split::vertical()`)
- Slices available space horizontally (columns, left-to-right) or vertically (rows, top-to-bottom).
- Inter-section spacing via `.spacing(px)` (default: `4.0pt`).
- Normalizes fractions automatically — fractions do not need to sum to `1.0`.
- Direct section additions: `.add_section(section)`, `.section_with(|s| ...)`, `.section_card(fraction, |s| ...)`.

### 3. Batch Section Generators
- **`.sections_equal(n, |index, ui| ...)`**: Creates $N$ equal-width or equal-height sections in one line.
- **`.sections_proportional(&[f32], |index, fraction, ui| ...)`**: Slices into multiple weighted sections from a fraction slice.

### 4. Automated 2D Layout Bounding & Window Sizing
- **`.min_size() -> Vec2`**: Automatically computes the total 2D minimum dimensions required across all sections and spacing:
  - **Horizontal Split**:
    $$\text{min\_width} = \sum_{i=1}^N \text{section\_min}_i + (N - 1) \times \text{spacing}$$
    $$\text{min\_height} = \max_{i=1}^N (\text{section\_cross\_min}_i)$$
  - **Vertical Split**:
    $$\text{min\_height} = \sum_{i=1}^N \text{section\_min}_i + (N - 1) \times \text{spacing}$$
    $$\text{min\_width} = \max_{i=1}^N (\text{section\_cross\_min}_i)$$
- **`.min_width() -> f32`** and **`.min_height() -> f32`**.
- **`.enforce_min_size(ctx)`**: One-liner to clamp the OS window's minimum inner size via `egui::ViewportCommand::MinInnerSize`.
- **Nested Propagation**: Nested splits pass their computed `.min_size()` up to the parent section automatically.

### 5. Multi-Pass Constraint Relaxation Solver
- **Pass 1 (Pre-allocation)**: Exact fixed sizes (`Size::Exact`) are locked first.
- **Pass 2 (Iterative Relaxation)**: Flexible space is distributed proportionally according to fractional weights, clamped against min/max constraints, and locked until all sections reach equilibrium.
- **Pass 3 (Constrained Fallback & Remainder Absorption)**: Handles extreme window shrinking gracefully with proportional downscaling and absorbs sub-pixel float rounding errors to eliminate micro-gaps.

### 6. Corner Rounding Invariant
- Enforces that widgets and cards render their 4 rounded corners fully intact across all window sizes without being sliced flat by parent clipping rects.

---

## Code Examples

### Example 1: Declarative Card Layout
```rust
use egui::{Color32, Stroke, Rounding};
use egui_layout::{Section, Split};

Split::horizontal()
    .spacing(6.0)
    // Left Explorer Card (24% flex, min 150x120pt)
    .add_section(
        Section::fraction(0.24)
            .min_size_2d(150.0, 120.0)
            .card()
            .title("📁 Project Explorer")
            .subtitle("Workspace files")
            .bg(Color32::from_rgb(30, 32, 48))
            .stroke(Stroke::new(1.0, Color32::from_rgb(69, 71, 90)))
            .rounding(8.0)
            .padding(10.0)
            .content(|ui| {
                ui.label("src/");
                ui.label("  lib.rs");
            }),
    )
    // Right Workspace Area (Flexible Remainder)
    .add_section(
        Section::remainder().content(|ui| {
            Split::vertical()
                .spacing(6.0)
                // Top Ribbon (Fixed 48pt height)
                .add_section(
                    Section::fixed(48.0)
                        .card()
                        .title("⚡ Ribbon")
                        .content(|ui| { ui.button("Run"); }),
                )
                // Main Canvas (Flexible Remainder)
                .add_section(
                    Section::remainder()
                        .min_size_2d(200.0, 100.0)
                        .card()
                        .title("🎨 Canvas")
                        .content(|ui| { ui.label("Canvas view..."); }),
                )
                .show(ui);
        }),
    )
    .show(ui);
```

### Example 2: Batch Section Generation
```rust
use egui_layout::Split;

// Create 4 equal columns in one line
Split::horizontal()
    .spacing(4.0)
    .sections_equal(4, |index, ui| {
        ui.label(format!("Column {}", index + 1));
    })
    .show(ui);

// Create weighted proportional rows (1x, 2x, 1x)
Split::vertical()
    .spacing(6.0)
    .sections_proportional(&[1.0, 2.0, 1.0], |index, weight, ui| {
        ui.label(format!("Row {}: Weight {:.1}", index + 1, weight));
    })
    .show(ui);
```

### Example 3: Automatic Window Minimum Size Clamping
```rust
use egui_layout::{Section, Split};

let layout = Split::horizontal()
    .spacing(6.0)
    .add_section(Section::fraction(0.25).min_size_2d(150.0, 100.0))
    .add_section(Section::remainder().min_size_2d(300.0, 150.0));

// One-liner: OS window will never allow resizing smaller than (456.0, 150.0)
layout.enforce_min_size(ctx);
```

---

## Settings Page Customization Options

When designing a **Settings / Layout Configuration Page**, the following parameters can be exposed to the user:

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `layout.section_spacing` | Slider | `0.0 ..= 24.0` px | Gutter spacing between adjacent split panes |
| `layout.sidebar_fraction` | Slider / DragValue | `0.15 ..= 0.50` | Default width fraction allocated to sidebars / navigation panes |
| `layout.min_section_width` | DragValue / Number | `100.0 ..= 400.0` px | Minimum allowable section width before constraint relaxation kicks in |
| `layout.min_section_height` | DragValue / Number | `50.0 ..= 300.0` px | Minimum allowable section height for stacked vertical panes |
| `layout.card_rounding` | Slider | `0.0 ..= 16.0` px | Corner radius applied to section cards and container surfaces |
| `layout.card_padding` | Slider | `0.0 ..= 20.0` px | Inner margin padding inside section card containers |
| `layout.auto_min_window_size` | Checkbox / Toggle | `true` / `false` | Automatically locks `egui::ViewportCommand::MinInnerSize` to computed layout 2D min bounds |
