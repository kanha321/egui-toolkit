# egui-themes

## What It Is

`egui-themes` is a **token-based theming system** for `egui` with 12 curated palette presets, smooth animated color morphing between themes, and interactive UI components for theme selection and preview.

**Crate path:** `src/egui-widgetkit/egui-themes/`

---

## What It Does

- **25 semantic design tokens**: Colors are organized into background layers, surfaces, overlays, typography, semantic roles, and on-color contrast pairs — not raw hex values.
- **12 curated presets**: Catppuccin (Mocha, Macchiato, Frappé, Latte), Tokyo Night, One Dark Pro, Dracula, Monokai Pro, Night Owl, Material Ocean, Rosé Pine, Gruvbox.
- **Animated theme morphing**: Switch themes with a smooth exponential crossfade across all 25+ color channels — no abrupt flash.
- **Native egui visuals sync**: Maps palette tokens directly to `egui::Visuals` so all stock egui widgets automatically match.
- **Interactive gallery and preview widgets**: Built-in `ThemeGallery` (swatch card grid) and `ThemePreview` (live mock-app preview).
- **WCAG contrast utility**: Auto-selects readable text color for any background.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
egui-themes = { path = "../egui-themes" }
```

---

### `ThemePalette` — The Color Data

A plain struct holding all 25 color tokens plus a swatch strip:

```rust
pub struct ThemePalette {
    pub dark: bool,

    // Background layers
    pub base: Color32,       // main canvas
    pub mantle: Color32,     // sidebar / cards
    pub crust: Color32,      // deepest headers / borders

    // Surfaces (progressive elevation)
    pub surface0: Color32,   // default cards
    pub surface1: Color32,   // hovered controls
    pub surface2: Color32,   // pressed / active

    // Overlays
    pub overlay0: Color32,   // muted decorations
    pub overlay1: Color32,   // subtle separators
    pub overlay2: Color32,   // prominent borders

    // Typography
    pub text: Color32,       // primary text (high contrast)
    pub subtext0: Color32,   // secondary text
    pub subtext1: Color32,   // captions

    // Semantic roles
    pub accent: Color32,     // primary accent
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub info_alt: Color32,
    pub sys_controls: Color32,

    // On-color contrast tokens
    pub on_accent: Color32,
    pub on_surface: Color32,
    pub on_success: Color32,
    pub on_warning: Color32,
    pub on_danger: Color32,
    pub on_info: Color32,

    pub swatches: Vec<Color32>,
}
```

**Get a palette from a preset:**

```rust
use egui_themes::{ThemePreset, ThemePalette};

let palette = ThemePreset::CatppuccinMocha.palette();
// or directly:
let palette = ThemePalette::tokyo_night();
```

**Access colors by token:**

```rust
use egui_themes::ThemeToken;

let bg = palette.get(ThemeToken::Base);
let accent = palette.get(ThemeToken::Accent);
palette.set(ThemeToken::Accent, Color32::from_rgb(255, 100, 100));
```

**WCAG contrast helper:**

```rust
// Returns palette.crust or palette.text — whichever is readable on `bg`
let text_color = palette.contrast_on(some_background_color);
```

---

### `ThemePreset` — The 12 Built-In Themes

```rust
pub enum ThemePreset {
    CatppuccinMocha,      // Dark — mauve accent
    CatppuccinMacchiato,  // Dark — mauve accent
    CatppuccinFrappe,     // Dark — mauve accent
    CatppuccinLatte,      // Light — mauve accent
    TokyoNight,           // Dark — purple accent
    OneDarkPro,           // Dark — purple accent
    Dracula,              // Dark — purple accent
    MonokaiPro,           // Dark — purple accent
    NightOwl,             // Dark — purple accent
    MaterialOcean,        // Dark — teal accent
    RosePine,             // Dark — iris accent
    Gruvbox,              // Dark — purple accent
}
```

**Useful methods:**

```rust
let name = ThemePreset::Dracula.name();          // "Dracula"
let is_dark = ThemePreset::Dracula.is_dark();    // true
let palette = ThemePreset::Dracula.palette();     // ThemePalette

// Iterate all presets (for building a selector UI)
for preset in ThemePreset::all() {
    println!("{}", preset.name());
}
```

---

### `ThemeState` — Animated Theme Switching

Manages the current palette, target palette, and smooth morphing animation.

```rust
use egui_themes::{ThemeState, ThemePreset};

// Store in your app state
let mut theme = ThemeState::new(ThemePreset::CatppuccinMocha);

// Switch themes (smooth animated morph)
theme.set_preset(ThemePreset::TokyoNight);

// Or switch instantly (no animation)
theme.set_preset_instant(ThemePreset::Dracula);

// Or morph toward a custom palette
theme.set_palette(custom_palette);

// Each frame in your update loop:
let dt = ctx.input(|i| i.stable_dt);
theme.update(dt, ctx);        // advances the color morph animation
theme.apply_to_ctx(ctx);      // maps current palette → egui::Visuals

// Access the current (animated) palette for widget styling
let palette = &theme.current;
```

**Key fields:**

```rust
theme.current          // ThemePalette — the live, animated palette
theme.target           // ThemePalette — the destination being morphed toward
theme.active_preset    // Option<ThemePreset> — which preset is active (None for custom)
theme.animating        // bool — true while morph is in progress
theme.morph_speed      // f32 — exponential decay speed (default: 9.0)
```

---

### `ThemeGallery` — Interactive Theme Selector

A responsive grid of clickable theme cards with active indicators and color swatch strips.

```rust
use egui_themes::ThemeGallery;

if let Some(preset) = ThemeGallery::new()
    .card_min_width(170.0)
    .card_height(54.0)
    .show(ui, theme.active_preset)
{
    theme.set_preset(preset);
}
```

---

### `ThemePreview` — Live Mock-App Preview

Renders a scaled mock application card showing real-time theme colors.

```rust
use egui_themes::ThemePreview;

ThemePreview::new()
    .height(200.0)
    .show(ui, &theme.current);
```

---

### `ThemeToken` — Semantic Token Enum

For programmatic access and iteration:

```rust
use egui_themes::ThemeToken;

// All 25 tokens
for token in ThemeToken::all() {
    let color = palette.get(*token);
    println!("{}: {:?} ({})", token.name(), color, token.category());
}
```

**Categories:** `"Background"`, `"Surfaces"`, `"Overlays"`, `"Typography"`, `"Semantic"`, `"Contrast"`.

---

### Color Utilities

```rust
use egui_themes::lerp_color;

// Linear interpolation between two colors (t clamped to 0.0..=1.0)
let blended = lerp_color(color_a, color_b, 0.5);
```

---

### API Reference

#### `ThemePreset`

| Method | Description |
| :--- | :--- |
| `name() -> &str` | Human-readable name |
| `is_dark() -> bool` | True for dark themes |
| `palette() -> ThemePalette` | Construct the palette |
| `all() -> &[ThemePreset]` | All 12 presets |

#### `ThemePalette`

| Method | Description |
| :--- | :--- |
| `get(token) -> Color32` | Get color by token |
| `set(token, color)` | Set color by token |
| `contrast_on(bg) -> Color32` | WCAG-readable text color for bg |
| `interpolate(&mut self, target, dt, speed) -> bool` | Smooth morph toward target |

#### `ThemeState`

| Method | Description |
| :--- | :--- |
| `new(preset) -> Self` | Initialize with preset |
| `from_palette(palette) -> Self` | Initialize with custom palette |
| `set_preset(preset)` | Smooth morph to preset |
| `set_preset_instant(preset)` | Instant switch |
| `set_palette(palette)` | Smooth morph to custom palette |
| `update(dt, ctx)` | Advance animation |
| `apply_to_ctx(ctx)` | Sync to egui::Visuals |
