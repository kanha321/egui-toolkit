# Part 5 — `egui-themes` (Single-Source-of-Truth Theme Engine)

## Overview
`egui-themes` provides a token-based, single-source-of-truth theming system for `egui`, inspired by `openrgb-effects-fixed`.

Rather than scattering hardcoded RGB colors across widgets, **every color in the application flows strictly from the active [`ThemePalette`]**. The engine provides 12 curated theme presets, smooth animated theme morphing, native `egui::Visuals` synchronization, an interactive mock component preview widget ([`ThemePreview`]), and a responsive swatch gallery grid ([`ThemeGallery`]).

---

## What It Does

### 1. Single Source of Truth (`ThemePalette`)
- **State Ownership (`CODING_RULES §2`)**: The consuming application owns `ThemeState` / `ThemePalette` as plain struct data (e.g., inside `TestAppState`) and passes it by reference.
- **Zero Cross-Crate Coupling (`CODING_RULES §1`)**: `egui-themes` depends solely on `egui`.
- **Semantic Tokens (`ThemeToken`)**:
  - **Background Layers**: `Base`, `Mantle`, `Crust`
  - **Surfaces**: `Surface0` (cards), `Surface1` (hovered/borders), `Surface2` (active/pressed)
  - **Overlays**: `Overlay0` (muted outlines), `Overlay1` (separators), `Overlay2` (highlights)
  - **Typography**: `Text` (primary), `Subtext0` (secondary), `Subtext1` (captions)
  - **Semantic Status & Accents**: `Accent`, `Success`, `Warning`, `Danger`, `Info`, `InfoAlt`, `SysControls`
  - **Designer Swatches**: `swatches: Vec<Color32>`

---

### 2. 12 Curated Theme Presets (`ThemePreset`)
1. **Catppuccin Mocha** (Default Dark — soothing pastel dark)
2. **Catppuccin Macchiato** (Medium dark pastel)
3. **Catppuccin Frappé** (Muted dark pastel)
4. **Catppuccin Latte** (Light mode pastel)
5. **Tokyo Night** (Neon cyberpunk dark)
6. **One Dark Pro** (Classic editor dark)
7. **Dracula** (Gothic dark vibrant)
8. **Monokai Pro** (High contrast warm dark)
9. **Night Owl** (Deep navy blue dark)
10. **Material Ocean** (Deep oceanic dark)
11. **Rosé Pine** (Warm aesthetic soho vibes)
12. **Gruvbox** (Retro groove earthy dark)

---

### 3. Smooth Exponential Lerp Theme Morphing
- When switching presets or customizing tokens, `ThemeState::update` smoothly interpolates each color channel towards the target palette using exponential smoothing:
  \[
  \text{factor} = 1.0 - e^{-\text{speed} \cdot \Delta t}
  \]
- Finite-step snapping logic guarantees integer convergence in finite frames without asymptotic stalling.
- Calls `ctx.request_repaint()` every frame until all colors have settled.

---

### 4. Native Egui Visuals Synchronization
- `state.apply_to_ctx(ctx)` maps the active palette's semantic colors directly to `ctx.style_mut().visuals`.
- All native egui widgets (buttons, text edits, checkboxes, menus, windows, panels) automatically inherit the palette colors.

---

### 5. Interactive Components

- **`ThemePreview`**:
  - Live animated mock application card demonstrating window control traffic dots, mock sidebar navigation, system options card with active switch, controls card with pulsing indicator, slider, and real-time audio spectrum equalizer bars.
- **`ThemeGallery`**:
  - Responsive multi-column grid of preset cards with selection indicators and bottom designer color palette strips with rounded corner geometry.

---

## Code Example

```rust
use egui_themes::{ThemeGallery, ThemePreset, ThemePreview, ThemeState, ThemeToken};

// In app state:
let mut theme = ThemeState::new(ThemePreset::CatppuccinMocha);

// In update loop:
theme.update(ui.input(|i| i.stable_dt), ui.ctx());
theme.apply_to_ctx(ui.ctx());

// Render live component preview:
ThemePreview::new().height(160.0).show(ui, &theme.current);

// Render preset selection gallery:
if let Some(new_preset) = ThemeGallery::new().show(ui, theme.active_preset) {
    theme.set_preset(new_preset);
}
```

---

## Settings Page Customization Options

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `theme.preset` | Gallery / ComboBox | 12 Presets | Curated designer theme preset selection |
| `theme.morph_speed` | Slider | `1.0 ..= 25.0` | Exponential lerp morphing transition speed |
| `theme.token.<name>` | ColorEdit / Picker | `Color32` | Live override for specific semantic color tokens |
