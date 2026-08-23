# egui-layout

Declarative, responsive, constraint-aware nested split layouts for [egui](https://github.com/emilk/egui).

## What It Does

Provides `Split` — a builder that divides available UI space into proportional sections, supporting arbitrary nesting, minimum size constraints, fixed-height slots, and optional card styling with titles.

Features:
- **Horizontal & Vertical splits**: `Split::horizontal()`, `Split::vertical()`
- **Sizing policies**: `Section::fraction(0.33)`, `Section::fixed(48.0)`, `Section::remainder()`
- **Minimum size constraints**: `.min_size_2d(width, height)`, `.min_cross(size)`
- **Card styling**: `.card()` with `.title()`, `.subtitle()`, `.bg()`, `.stroke()`
- **Global style configuration**: `SplitStyle` configures all visual defaults in one place
- **Theme-aware defaults**: Card backgrounds and text colors auto-inherit from `egui::Visuals`
- **Outer frame support**: Optional background, border, and padding around the entire split
- **Automatic fraction normalization**: Fractions don't need to sum to 1.0
- **Zero sub-pixel gaps**: Float rounding remainder absorption

## Quick Start

```rust
use egui_layout::{Section, Split, SplitStyle};

// Simple 2-column layout
Split::horizontal()
    .spacing(6.0)
    .section(0.30, |ui| {
        ui.label("Sidebar (30%)");
    })
    .section(0.70, |ui| {
        ui.label("Content (70%)");
    })
    .show(ui);
```

## SplitStyle — Global Visual Configuration

`SplitStyle` lets you configure all visual defaults in one place. Set it once on a `Split`, and all child sections inherit.

```rust
use egui_layout::{Split, Section, SplitStyle};
use egui::{Color32, Rounding, Stroke};

let style = SplitStyle::default()
    .with_spacing(8.0)              // Gap between sections
    .with_card_rounding(12.0)       // Corner radius for all cards
    .with_card_padding(12.0)        // Inner padding for all cards
    .with_card_stroke(Stroke::new(2.0, Color32::from_rgb(80, 80, 100))) // Border thickness + color
    .with_title_size(14.0)          // Title font size
    .with_subtitle_size(11.0)       // Subtitle font size
    .with_frame_bg(Color32::from_rgb(20, 20, 30))   // Outer container background
    .with_frame_rounding(16.0)      // Outer container corner radius
    .with_frame_padding(8.0);       // Padding between frame and sections

Split::horizontal()
    .style(style)
    .add_section(Section::fraction(0.3).card().title("Sidebar").content(|ui| {
        ui.label("All cards share the same rounding, padding, and stroke.");
    }))
    .add_section(Section::remainder().card().title("Content").content(|ui| {
        ui.label("No need to repeat style on every section.");
    }))
    .show(ui);
```

### Resolution Order

For any visual property, the value is resolved in this priority:

```
Section override  >  SplitStyle value  >  ui.visuals() fallback
```

For example, if a `Section` has `.rounding(16.0)`, that wins. Otherwise the `SplitStyle.card_rounding` is used. If neither is set, the card reads from `ui.visuals()` (which `ThemeState::apply_to_ctx()` populates from the active palette).

### SplitStyle Properties

| Category | Property | Default | Description |
|---|---|---|---|
| **Gaps** | `spacing` | `4.0` | Space between sections in pixels |
| **Card** | `card_bg` | `None` → `visuals.faint_bg_color` | Card background fill |
| | `card_stroke` | `None` → `visuals.window_stroke` | Card border (thickness + color) |
| | `card_rounding` | `Rounding::same(8.0)` | Card corner radius |
| | `card_padding` | `8.0` | Card inner margin |
| **Text** | `title_color` | `None` → `visuals.strong_text_color()` | Title text color |
| | `subtitle_color` | `None` → `visuals.text_color()` | Subtitle text color |
| | `title_size` | `13.0` | Title font size in points |
| | `subtitle_size` | `10.5` | Subtitle font size in points |
| **Frame** | `frame_bg` | `None` (transparent) | Outer container background |
| | `frame_stroke` | `None` (no border) | Outer container border |
| | `frame_rounding` | `Rounding::ZERO` | Outer container corner radius |
| | `frame_padding` | `0.0` | Space between frame edge and sections |

## Section Sizing Policies

```rust
use egui_layout::Section;

Section::fraction(0.33)    // Takes 33% of available space
Section::fixed(48.0)       // Takes exactly 48 logical pixels
Section::remainder()       // Fills whatever space is left
```

## Section Builder

```rust
Section::fraction(0.25)
    .min_size_2d(150.0, 100.0)   // Minimum width and height
    .min_cross(200.0)            // Minimum size in the cross-axis
    .card()                      // Enable card container styling
    .title("📁 Explorer")        // Card title text
    .subtitle("Left panel")     // Card subtitle text
    .title_color(palette.info)   // Override title color
    .subtitle_color(palette.overlay2) // Override subtitle color
    .bg(palette.mantle)          // Override card background
    .stroke(Stroke::new(1.0, palette.surface1))  // Override border
    .rounding(Rounding::same(12.0))  // Override corner rounding
    .padding(10.0)               // Inner padding in pixels
    .content(|ui| {
        ui.label("Card content here");
    })
```

## Nested Layouts

Splits can be arbitrarily nested. Shared styles propagate cleanly:

```rust
let style = SplitStyle::default()
    .with_spacing(6.0)
    .with_card_rounding(10.0)
    .with_card_padding(10.0);

Split::horizontal()
    .style(style.clone())
    .section(0.25, |ui| {
        ui.label("Sidebar");
    })
    .section_remainder(|ui| {
        Split::vertical()
            .style(style.clone())
            .section_fixed(48.0, |ui| {
                ui.label("Header");
            })
            .section_remainder(|ui| {
                ui.label("Main Content");
            })
            .show(ui);
    })
    .show(ui);
```

## Minimum Size Calculation

Pre-calculate the minimum space a layout needs:

```rust
let layout = Split::horizontal()
    .spacing(6.0)
    .add_section(Section::fraction(0.3).min_size_2d(150.0, 100.0))
    .add_section(Section::remainder().min_size_2d(200.0, 100.0));

let min = layout.min_size();
// Enforce on the window:
layout.enforce_min_size(ctx);
```

## Theme Integration

`egui-layout` has **zero dependency** on `egui-themes`. Theme integration works through `egui::Visuals`:

1. `ThemeState::apply_to_ctx(ctx)` maps palette tokens to `egui::Visuals`
2. Card defaults read from `ui.visuals()` (e.g. `faint_bg_color`, `window_stroke`, `strong_text_color()`)
3. When you switch themes, all cards automatically update

No manual color passing required for default styling.

## Running the Example

```bash
cargo run -p egui-layout --example nested_layout
```
