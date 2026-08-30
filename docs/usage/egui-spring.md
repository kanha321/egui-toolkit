# egui-spring

## What It Is

`egui-spring` is an **animated rectangle and highlight system** for `egui`. It wraps `spring-core`'s analytical spring physics into 2D widgets that glide, morph, and bounce between target positions — producing the fluid selection highlights, cursor animations, and focus rings you see in polished desktop apps like Neovide and OpenRGB.

**Crate path:** `src/egui-widgetkit/egui-spring/`

---

## What It Does

- **`SpringRect`**: An animated selection highlight that glides between widget rects with elastic directional smear — leading corners shoot forward while trailing corners lag behind.
- **`HighlightGroup<K>`**: A multi-layer stack of `SpringRect`s indexed by key, enabling simultaneous section-level, item-level, and cursor-level highlights.
- **`SpringCursor`**: A fluid text cursor with macro-to-micro morphing — peels off from a parent highlight into a thin caret, and fades back out when focus leaves.
- **`CornerSprings`**: Four independent 2D spring-driven corners that create the elastic smear effect during movement.
- **Bézier boundary rendering**: Smooth rounded-rect outlines built from cubic Bézier arcs, with rounding that morphs during flight.
- **Pure `egui::Shape` output**: No custom shaders or render callbacks — just polygons and strokes.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
egui-spring = { path = "../egui-spring" }
```

---

### `SpringRect` — Single Animated Highlight

The primary building block. Animates between target rects with spring physics.

**Create and drive a highlight:**

```rust
use egui_spring::{SpringRect, SpringParams, MotionPhysics};

// Store in your app state
let mut highlight = SpringRect::new(initial_rect)
    .with_motion(MotionPhysics::Snappy)
    .with_fill(Color32::from_rgba_unmultiplied(137, 180, 250, 45))
    .with_stroke(Stroke::new(2.0, Color32::from_rgb(137, 180, 250)))
    .with_rounding(8.0)
    .with_padding(3.0);

// Each frame:
let dt = ui.input(|i| i.stable_dt).min(0.05);
highlight.set_target(new_widget_rect);
highlight.update(dt);
highlight.paint(ui.painter());

if !highlight.is_settled() {
    ui.ctx().request_repaint();
}
```

**Preset constructors:**

```rust
let h = SpringRect::snappy(rect);   // crisp, fast
let h = SpringRect::gentle(rect);   // smooth, cushioned
let h = SpringRect::bouncy(rect);   // elastic bounce
let h = SpringRect::openrgb(rect);  // Neovide/OpenRGB fluid feel
let h = SpringRect::off(rect);      // instant snap, no animation
```

**From a theme palette:**

```rust
let h = SpringRect::from_palette(rect, &palette);
```

**Using `HighlightConfig`:**

```rust
let config = HighlightConfig::new()
    .with_motion(MotionPhysics::Snappy)
    .with_fill(Color32::from_rgba_unmultiplied(100, 150, 255, 20))
    .with_stroke(Stroke::new(2.0, Color32::from_rgb(100, 150, 255)))
    .with_rounding(8.0)
    .with_padding(4.0);

let h = SpringRect::from_config(rect, &config);
```

**Key methods:**

| Method | Description |
| :--- | :--- |
| `set_target(rect)` | Glide to a new rect (smooth transition) |
| `set_target_with_rounding(rect, r)` | Glide with rounding morph |
| `reset(rect)` | Teleport instantly (no animation) |
| `update(dt)` | Advance physics by dt seconds |
| `paint(painter)` | Draw the highlight |
| `show(ui) -> Response` | Update + paint in one call |
| `is_settled() -> bool` | True when at rest |
| `current_bounding_rect() -> Rect` | Current animated bounding rect |

---

### `HighlightGroup<K>` — Multi-Layer Highlights

Manage multiple named highlight layers simultaneously. Use when you need a section-level highlight gliding between panels AND an item-level highlight gliding between buttons.

```rust
use egui_spring::{HighlightGroup, HighlightConfig, MotionPhysics};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Layer { Section, Item, Cursor }

// Store in app state
let mut highlights = HighlightGroup::new()
    .with_config(Layer::Section, HighlightConfig::new().with_motion(MotionPhysics::Gentle))
    .with_config(Layer::Item, HighlightConfig::new().with_motion(MotionPhysics::Snappy));

// Each frame — retarget layers
highlights.set_target(&Layer::Section, panel_rect);
highlights.set_target(&Layer::Item, focused_widget_rect);

// Update all layers and paint
highlights.update(dt);
highlights.paint_all(ui.painter());

if !highlights.is_settled() {
    ui.ctx().request_repaint();
}
```

**Key methods:**

| Method | Description |
| :--- | :--- |
| `add(key, config)` | Add a layer from config |
| `insert(key, rect)` | Add a pre-built SpringRect |
| `get(key)` / `get_mut(key)` | Access a layer |
| `remove(key)` | Remove a layer |
| `set_target(key, rect)` | Retarget a specific layer |
| `set_target_with_rounding(key, rect, r)` | Retarget with rounding |
| `reset_layer(key, rect, rounding)` | Teleport a layer instantly |
| `update(dt)` | Advance all layers |
| `paint_layer(key, painter)` | Paint one layer |
| `paint_all(painter)` | Paint all layers (sorted by key) |
| `is_settled() -> bool` | True when ALL layers are at rest |

---

### `SpringCursor` — Fluid Text Cursor

A cursor that morphs from a large highlight rect down to a thin caret when entering text editing mode.

```rust
use egui_spring::{SpringCursor, SpringParams};

// Store in app state
let mut cursor = SpringCursor::new(SpringParams::new(24.0, 0.65));

// When focus enters a text widget — morph from outer highlight to caret
cursor.spawn_from(
    outer_highlight_rect, 6.0,   // origin rect + rounding
    caret_rect, 1.5,             // target caret rect + rounding
);

// Each frame while editing:
cursor.update(caret_rect, is_focused, dt, ctx);
cursor.paint(ui.painter(), fill_color, stroke);

// When focus leaves:
cursor.fade_out();
```

---

### `highlight_sync` — Batch Utilities

Convenience functions for updating and painting groups in one call:

```rust
use egui_spring::{update_and_paint, sync_highlight_stroke_color, set_highlight_fill};

// Update + paint all layers, returns true if still animating
let animating = update_and_paint(&mut highlights, dt, ui.painter());
if animating {
    ui.ctx().request_repaint();
}

// Smoothly crossfade stroke color toward a target
sync_highlight_stroke_color(&mut highlights, &Layer::Item, target_color, dt, 8.0);

// Set fill color immediately
set_highlight_fill(&mut highlights, &Layer::Item, new_fill);
```

---

### How the Directional Smear Effect Works

Unlike simple lerp-based rect animations, `egui-spring` breaks each rectangle into 4 independent corner points. During movement:

1. **Leading corners** (in the travel direction) receive higher stiffness (+60%) and less damping — they shoot forward.
2. **Trailing corners** lag behind with more damping — creating an elastic stretch effect.
3. **Logarithmic distance scaling** prevents chaotic oscillation on large jumps.
4. **Symmetric resize guard**: When morphing between very different sizes (e.g. highlight → caret), directional bias is disabled for clean expansion.

The result is the signature Neovide / OpenRGB elastic smear where the highlight visually stretches along its direction of travel before snapping into shape.

---

### API Reference

#### `HighlightConfig`

| Method | Description |
| :--- | :--- |
| `new()` | Default config |
| `from_palette(palette)` | Config from theme palette |
| `.with_motion(MotionPhysics)` | Set physics preset |
| `.with_fill(Color32)` | Interior fill |
| `.with_stroke(Stroke)` | Border stroke |
| `.with_rounding(f32)` | Corner rounding |
| `.with_padding(f32)` | Rect expansion padding |

#### `SpringRect`

| Method | Description |
| :--- | :--- |
| `new(rect)` | Create with default physics |
| `from_config(rect, config)` | Create from HighlightConfig |
| `snappy(rect)` / `gentle(rect)` / `bouncy(rect)` / `openrgb(rect)` / `off(rect)` | Preset constructors |
| `.with_motion(m)` / `.with_fill(c)` / `.with_stroke(s)` / `.with_rounding(r)` / `.with_padding(p)` | Builder methods |
| `set_target(rect)` | Smooth retarget |
| `reset(rect)` | Instant teleport |
| `update(dt)` | Advance physics |
| `paint(painter)` | Render to painter |
| `show(ui) -> Response` | Update + paint |
| `is_settled() -> bool` | At rest check |
