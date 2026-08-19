# Part 3 — `egui-spring` (Spring-Animated Selection Highlight)

## Overview
`egui-spring` translates the mathematical physics of `spring-core` into visual `egui::Shape` primitives. It powers the "elastic smear" selection highlight — a dynamic, organic cursor where the 4 corners stretch and lag during travel and snap cleanly into place with Bézier-rounded corners.

---

## What It Does

1. **4-Corner Analytical Bundle (`CornerSprings`)**:
   - Manages 4 independent 2D spring corners (Top-Left, Top-Right, Bottom-Right, Bottom-Left), totaling 8 independent 1D analytical spring ODE solvers.
   - **Direction-Aware Smear**: Corners leading in the direction of motion gain increased frequency and lower damping to stretch forward, while trailing corners lag.
   - **Distance-Scaled Logarithmic Damping**: Prevents excessive oscillation when jumping across large multi-monitor distances.

2. **Cubic Bézier Boundary Reconstruction (`build_bezier_boundary`)**:
   - Connects the deformed 4 corners using cubic Bézier arcs with circle approximation constant $\kappa = \frac{4}{3}(\sqrt{2} - 1) \approx 0.55228475$.
   - **Corner Rounding Invariant**: Automatically computes $r_{\text{eff}} = \min(r, \frac{1}{2}\min(L_{\text{prev}}, L_{\text{next}}))$ so corners stay convex and intact without self-intersecting.

3. **`SpringRect` Builder Widget**:
   - Builder API: `SpringRect::new(target)` $\to$ `.with_fill(color)` $\to$ `.with_stroke(stroke)` $\to$ `.with_rounding(r)` $\to$ `.with_params(params)`.
   - **Continuous Motion Repaint (`CODING_RULES §4`)**: Calls `ctx.request_repaint()` every frame while moving, halting immediately upon settling to consume 0% idle CPU.
   - **Pure `egui::Shape` Output**: Emits `Shape::convex_polygon` and `Shape::closed_line` (zero custom GL or wgpu `PaintCallback`s).

---

## How It Is Used in the Project (`test-app`)

```rust
use egui_spring::SpringRect;
use spring_core::SpringParams;

// 1. App state owns the highlight
let mut highlight = SpringRect::new(egui::Rect::ZERO)
    .with_fill(egui::Color32::from_rgba_unmultiplied(0, 255, 136, 40))
    .with_stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 136)))
    .with_rounding(8.0)
    .with_params(SpringParams::openrgb());

// 2. Set new target rect when an item is selected
if let Some(selected_rect) = active_selection_rect {
    highlight.set_target(selected_rect);
}

// 3. Update & paint
let dt = ui.input(|i| i.stable_dt).min(0.05);
highlight.update(dt);

if !highlight.is_settled() {
    ui.ctx().request_repaint();
}

highlight.paint(ui.painter());
```

---

## Settings Page Customization Options

When designing a **Settings / Highlight & Animation Page**, the following parameters can be exposed to the user:

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `highlight.fill_color` | ColorPicker (RGBA) | `Color32` | Interior tint fill of the selection highlight |
| `highlight.stroke_color` | ColorPicker (RGB) | `Color32` | Border outline glow color |
| `highlight.stroke_width` | Slider | `0.5 ..= 4.0` px | Border stroke thickness |
| `highlight.corner_rounding` | Slider | `0.0 ..= 24.0` px | Bézier arc corner radius |
| `highlight.smear_intensity` | Slider | `0.0 ..= 2.0` | Multiplier for leading corner stretch elasticity |
| `highlight.preset` | ComboBox | `Gentle`, `Snappy`, `Bouncy`, `OpenRGB`, `Custom` | Spring stiffness and damping preset |
| `highlight.glow_effect` | Checkbox / Toggle | `true` / `false` | Enables dual-layer outer glow outline |
