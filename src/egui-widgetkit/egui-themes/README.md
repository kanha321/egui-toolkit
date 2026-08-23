# egui-themes

Token-based theming engine with 12 curated palette presets, smooth morph transitions, and live preview widgets for [egui](https://github.com/emilk/egui).

## What It Does

Provides a **single source of truth** for all UI colors in your application. Every color — card backgrounds, text, borders, accents, status indicators — flows from a `ThemePalette`. When you switch themes, all widgets update automatically with smooth animated color morphing.

Features:
- **19 semantic color tokens**: Backgrounds, surfaces, overlays, text, accents, and status colors
- **12 curated presets**: Catppuccin (Mocha, Macchiato, Frappé, Latte), Tokyo Night, Dracula, Monokai Pro, One Dark Pro, Gruvbox, Night Owl, Material Ocean, Rosé Pine
- **Smooth theme morphing**: Exponential lerp with per-channel convergence guarantees
- **Native egui sync**: `apply_to_ctx()` maps palette tokens to `egui::Visuals` so all native widgets inherit theme colors automatically
- **Live preview widget**: `ThemePreview` renders a scaled mock app with animated elements
- **Gallery widget**: `ThemeGallery` renders a responsive preset card grid

## Quick Start

```rust
use egui_themes::{ThemePreset, ThemeState, ThemeGallery};

// In your app state:
struct MyApp {
    theme: ThemeState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            theme: ThemeState::new(ThemePreset::CatppuccinMocha),
        }
    }
}

// In your update loop:
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Advance morph animation
        let dt = ctx.input(|i| i.stable_dt.min(0.1));
        self.theme.update(dt, ctx);

        // 2. Sync palette → egui::Visuals
        self.theme.apply_to_ctx(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            // 3. Show theme gallery — click a card to switch
            if let Some(preset) = ThemeGallery::new().show(ui, self.theme.active_preset) {
                self.theme.set_preset(preset);
            }
        });
    }
}
```

## API

### `ThemePalette`

The single source of truth — a struct with 19 named color fields:

| Category | Tokens | Description |
|---|---|---|
| **Backgrounds** | `base`, `mantle`, `crust` | Primary, secondary, and deepest background layers |
| **Surfaces** | `surface0`, `surface1`, `surface2` | Card fills, borders, interactive element backgrounds |
| **Overlays** | `overlay0`, `overlay1`, `overlay2` | Muted elements, spring coils, subtle dividers |
| **Typography** | `text`, `subtext0`, `subtext1` | Primary text, muted text, secondary text |
| **Semantic** | `accent`, `success`, `warning`, `danger`, `info`, `info_alt`, `sys_controls` | Interactive accents and status indicators |
| **Swatches** | `swatches: Vec<Color32>` | Designer color strip for galleries and pickers |

```rust
// Access a color
let bg = palette.base;
let title = palette.accent;
let border = palette.surface1;

// Access via token enum
use egui_themes::ThemeToken;
let color = palette.get(ThemeToken::Accent);
```

### `ThemePreset`

Enum of all 12 built-in themes:

```rust
use egui_themes::ThemePreset;

ThemePreset::CatppuccinMocha      // Dark, warm
ThemePreset::CatppuccinMacchiato  // Dark, mid-tone
ThemePreset::CatppuccinFrappe     // Dark, cool
ThemePreset::CatppuccinLatte      // Light
ThemePreset::TokyoNight           // Dark, blue-purple
ThemePreset::Dracula              // Dark, vivid
ThemePreset::MonokaiPro           // Dark, warm yellows
ThemePreset::OneDarkPro           // Dark, cool blues
ThemePreset::Gruvbox              // Dark, earthy
ThemePreset::NightOwl             // Dark, teal-blue
ThemePreset::MaterialOcean        // Dark, deep blue
ThemePreset::RosePine             // Dark, muted rose

// Get palette from preset
let palette = ThemePreset::CatppuccinMocha.palette();

// Iterate all presets
for preset in ThemePreset::all() {
    println!("{} (dark: {})", preset.name(), preset.is_dark());
}
```

### `ThemeState`

App-owned state manager for theme switching and morphing:

| Method | Description |
|---|---|
| `ThemeState::new(preset)` | Initialize with a preset |
| `ThemeState::from_palette(palette)` | Initialize with a custom palette |
| `.set_preset(preset)` | Smoothly morph to a new preset |
| `.set_preset_instant(preset)` | Switch instantly (no animation) |
| `.set_palette(palette)` | Smoothly morph to a custom palette |
| `.update(dt, ctx)` | Advance morph animation, request repaint |
| `.apply_to_ctx(ctx)` | Sync current palette → `egui::Visuals` |
| `.current` → `ThemePalette` | The current (possibly mid-morph) palette |
| `.target` → `ThemePalette` | The target palette being morphed towards |
| `.active_preset` → `Option<ThemePreset>` | Currently selected preset (if any) |
| `.animating` → `bool` | Whether a morph is in progress |

### `ThemePreview`

Live animated mock app preview:

```rust
use egui_themes::ThemePreview;

ThemePreview::new()
    .height(200.0)       // Optional fixed height
    .show(ui, &palette); // Renders a scaled mock app card
```

### `ThemeGallery`

Responsive preset card grid:

```rust
use egui_themes::ThemeGallery;

if let Some(preset) = ThemeGallery::new().show(ui, theme.active_preset) {
    theme.set_preset(preset);
}
```

## How Other Crates Use the Theme

All library crates in `egui-widgetkit` depend on `egui-themes` and read colors from the palette:

- **`egui-layout`**: Card backgrounds, borders, and title colors default to `ui.visuals()` fields that `apply_to_ctx()` populates from the palette
- **`egui-spring`**: `HighlightConfig::from_palette(&palette)` and `SpringRect::from_palette(rect, &palette)` read the accent color
- **`egui-vim-nav`**: Headless; consuming code passes palette-derived colors to rendering closures
- **`egui-nav-stack`**: Headless; consuming code passes palette-derived colors to screen rendering closures

## Running the Example

```bash
cargo run -p egui-themes --example theme_switcher
```
