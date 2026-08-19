# Part 1 — `egui-layout` (Constraint-Aware Layout Engine)

## Overview
`egui-layout` is a responsive, nested split layout engine for `egui`. It automatically subdivides available screen space among child sections without requiring hardcoded pixel breakpoints, manual resize handlers, or fixed aspect ratios.

---

## What It Does

1. **Responsive Proportional Splits (`Split::horizontal()` / `Split::vertical()`)**:
   - Divides primary axis space according to fractional proportions (`Size::Fraction`), fixed pixel lengths (`Size::Exact`), or remaining space fill (`Size::Remainder`).
   - Normalizes fractions automatically — fractions do not need to sum to `1.0`.

2. **Constraint Relaxation Engine (`Size::min_size()` / `Size::max_size()`)**:
   - Implements an iterative constraint solver. If shrinking available space causes a fractional pane to drop below its minimum required size, the solver locks that pane to its minimum and dynamically relaxes the remaining sections proportionally.

3. **Sub-Pixel Remainder Absorption**:
   - Prevents floating-point gaps and rounding seams at window edges by distributing leftover fractional pixels to the final section.

4. **Dynamic Minimum Size Calculation (`Split::compute_min_length`)**:
   - Analyzes all registered section sizing policies and inter-section spacing to calculate the exact minimum window dimension required along that axis:
     $$\text{Min Primary Length} = \sum \text{Min Section Lengths} + (N - 1) \times \text{Spacing}$$

5. **Corner Rounding Invariant**:
   - Enforces that widgets and cards render their 4 rounded corners fully intact without being sliced flat by parent clipping rects.

---

## How It Is Used in the Project (`test-app`)

```rust
use egui_layout::{Split, Size};

// 1. Calculate dynamic window minimum bounds
let min_width = Split::compute_min_length(8.0, &[
    Size::Fraction(0.3).min_size(150.0),
    Size::Remainder.min_size(300.0),
]);

// 2. Render responsive layout
Split::horizontal()
    .spacing(8.0)
    .section_min(0.3, 150.0, |ui| {
        // Left sidebar
    })
    .section_remainder(300.0, |ui| {
        // Main content area
        Split::vertical()
            .spacing(8.0)
            .section(0.7, |ui| { /* Top editor */ })
            .section(0.3, |ui| { /* Bottom console */ })
            .show(ui);
    })
    .show(ui);
```

---

## Settings Page Customization Options

When designing a **Settings / Layout Configuration Page**, the following parameters can be exposed to the user:

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `layout.section_spacing` | Slider | `0.0 ..= 24.0` px | Controls the gutter spacing between adjacent split panes |
| `layout.sidebar_fraction` | Slider / DragValue | `0.15 ..= 0.50` | Default width fraction allocated to sidebars / navigation panes |
| `layout.min_section_width` | DragValue / Number | `100.0 ..= 400.0` px | Minimum allowable width before constraint relaxation kicks in |
| `layout.min_section_height` | DragValue / Number | `50.0 ..= 300.0` px | Minimum allowable height for stacked vertical panes |
| `layout.corner_rounding` | Slider | `0.0 ..= 16.0` px | Corner radius applied to section cards and container surfaces |
| `layout.auto_min_window_size` | Checkbox / Toggle | `true` / `false` | Automatically locks `egui::ViewportCommand::MinInnerSize` to computed layout min length |
