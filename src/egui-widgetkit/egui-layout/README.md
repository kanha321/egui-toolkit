# egui-layout

Declarative, responsive nested split layouts (`Split::horizontal()`, `Split::vertical()`) for [egui](https://github.com/emilk/egui).

## Features

- **Nested Proportional Splits**: Divide available screen space horizontally or vertically with arbitrary relative fractions (e.g. `0.33` and `0.67`).
- **Automatic Normalization**: Fractions do not need to sum to `1.0` — normalized internally.
- **Defensive & Resilient**: Handles negative fractions, zero sums, and single-child edge cases without panicking.
- **Accurate Spacing Deduction**: Inter-section spacing is subtracted before dividing available space, ensuring exact zero-overflow containment.
- **Float Rounding Remainder Absorption**: Eliminates sub-pixel gap artifacts at container edges.

## Usage

```rust
use egui_layout::Split;

Split::horizontal()
    .spacing(6.0)
    .section(0.33, |ui| {
        ui.label("Sidebar");
    })
    .section(0.67, |ui| {
        Split::vertical()
            .spacing(4.0)
            .section(0.20, |ui| {
                ui.label("Header (20%)");
            })
            .section(0.80, |ui| {
                ui.label("Main Content (80%)");
            })
            .show(ui);
    })
    .show(ui);
```

## State Ownership

`Split` is an ephemeral builder struct constructed per-frame. It does not use or require any global or static mutable state (conforming to `CODING_RULES §2`).
