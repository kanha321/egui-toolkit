# egui-spring

Spring-animated selection highlights with 4-corner Bézier border deformation for [egui](https://github.com/emilk/egui).

## What It Does

Provides `SpringRect` — a spring-animated rectangle that smoothly morphs its position, size, and per-corner rounding to follow a moving target. Used for selection highlights, focus indicators, and animated UI elements.

Features:
- **4-corner independent springs**: Each corner animates independently via analytical ODE solving
- **Bézier border deformation**: Smooth curved borders using cubic Bézier approximation
- **Multi-layer highlight groups**: Stack multiple independent highlights (e.g. section + item layers)
- **Theme-aware**: `from_palette()` constructors read accent colors directly from `ThemePalette`
- **Pure `egui::Shape` output**: No custom render pipelines

## Quick Start

```rust
use egui::{Rect, Rounding, Stroke, Color32};
use egui_spring::{SpringRect, HighlightConfig, MotionPhysics};

// Create a spring highlight (in your app state)
let mut highlight = SpringRect::new(Rect::ZERO);

// Or create from theme palette:
// let mut highlight = SpringRect::from_palette(Rect::ZERO, &palette);

// In your update loop:
let dt = ui.input(|i| i.stable_dt).min(0.05);

// Set target to follow the focused widget
highlight.set_target(focused_rect);
highlight.update(dt);

// Paint the animated highlight
highlight.paint(ui.painter());

// Request repaint while animating
if !highlight.is_settled() {
    ui.ctx().request_repaint();
}
```

## API

### `HighlightConfig`

Reusable configuration bundle:

```rust
use egui_spring::HighlightConfig;

// Default (green accent)
let config = HighlightConfig::new();

// From theme palette (reads palette.accent)
let config = HighlightConfig::from_palette(&palette);

// Custom configuration via builder
let config = HighlightConfig::new()
    .with_motion(MotionPhysics::Snappy)
    .with_fill(Color32::from_rgba_unmultiplied(100, 150, 255, 20))
    .with_stroke(Stroke::new(2.0, Color32::from_rgb(100, 150, 255)))
    .with_rounding(8.0)
    .with_padding(4.0);
```

### `SpringRect`

The animated highlight widget:

| Method | Description |
|---|---|
| `SpringRect::new(rect)` | Create with default green accent |
| `SpringRect::from_palette(rect, palette)` | Create with accent from `ThemePalette` |
| `SpringRect::from_config(rect, config)` | Create from a `HighlightConfig` |
| `.set_target(rect)` | Set the target rectangle to animate towards |
| `.set_target_with_corner_rounding(rect, rounding)` | Set target with per-corner rounding morph |
| `.update(dt)` | Advance the spring animation by `dt` seconds |
| `.paint(painter)` | Emit `egui::Shape` primitives to the painter |
| `.is_settled()` → `bool` | True when animation has converged |
| `.set_motion(physics)` | Change motion physics preset |
| `.set_fill(color)` | Change fill color |
| `.set_stroke(stroke)` | Change border stroke |

### `HighlightGroup<K>`

Manages multiple independent highlight layers:

```rust
use egui_spring::{HighlightGroup, HighlightConfig};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Layer { Section, Item }

let mut highlights = HighlightGroup::new();
highlights.add(Layer::Section, HighlightConfig::new().with_motion(MotionPhysics::Gentle));
highlights.add(Layer::Item, HighlightConfig::new().with_motion(MotionPhysics::Snappy));

// Set targets independently
highlights.set_target(Layer::Section, section_rect);
highlights.set_target(Layer::Item, item_rect);

// Update all layers
highlights.update(dt);

// Paint all layers
highlights.paint_all(ui.painter());
```

## Running the Example

```bash
cargo run -p egui-spring --example selection_highlight
```
