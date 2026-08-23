# egui-widgetkit

A modular suite of app-agnostic UI crates for [egui](https://github.com/emilk/egui), providing responsive layouts, spring physics, animated highlights, Vim-style navigation, screen navigation stacks, and a token-based theming engine.

## Crates

| Crate | Description |
|---|---|
| [`spring-core`](src/egui-widgetkit/spring-core) | Framework-agnostic analytical spring physics solver (pure math, **zero `egui` dependency**) |
| [`egui-layout`](src/egui-widgetkit/egui-layout) | Declarative, responsive nested split layouts with card styling |
| [`egui-spring`](src/egui-widgetkit/egui-spring) | Spring-animated selection highlights with Bézier borders |
| [`egui-vim-nav`](src/egui-widgetkit/egui-vim-nav) | Vim-style (HJKL) keyboard & mouse navigation across focus graphs |
| [`egui-themes`](src/egui-widgetkit/egui-themes) | Token-based theming engine with 12 curated palettes and smooth morphing |
| [`egui-nav-stack`](src/egui-widgetkit/egui-nav-stack) | App-owned back-stack screen navigation with spring-animated transitions |

## Quick Start

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
egui-themes = { path = "path/to/egui-themes" }
egui-layout = { path = "path/to/egui-layout" }
egui-spring = { path = "path/to/egui-spring" }
```

### Minimal Themed App

```rust
use egui_themes::{ThemePreset, ThemeState};

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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt.min(0.1));
        self.theme.update(dt, ctx);
        self.theme.apply_to_ctx(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, themed world!");
        });
    }
}
```

All egui widgets automatically inherit the active theme's colors — no manual color passing required.

## Design Principles

### Single Source of Truth
**`ThemePalette`** defines every color used across the entire UI. All library crates read their default card backgrounds, text colors, and border strokes from `egui::Visuals`, which `ThemeState::apply_to_ctx()` synchronizes with the active palette. No hardcoded RGB values.

### State Ownership
Every stateful type (`Spring`, `SpringRect`, `NavStack`, `ThemeState`, `Navigator`) is a plain struct owned by the consuming application and passed by `&mut` reference. There is no global, singleton, or hidden static state.

### `egui::Shape` Only
All visual output uses pure `egui::Shape` primitives. No raw `wgpu`/`glow` or custom `PaintCallback` render pipelines.

### Continuous Motion Repaint
Any animated widget calls `ctx.request_repaint()` every frame until the animation settles, then stops to conserve CPU.

## Workspace Structure

```
.
├── Cargo.toml                              # Virtual workspace root
├── README.md
├── docs/
│   ├── ARCHITECTURE_PRD.md                 # Requirements & crate responsibilities
│   ├── FOLDER_STRUCTURE.md                 # Workspace layout conventions
│   ├── CODING_RULES.md                     # Enforceable coding standards
│   ├── BUILD_PLAN.md                       # Incremental build plan
│   └── progress.md                         # Build progress tracking
└── src/
    ├── egui-widgetkit/                     # The reusable library crates
    │   ├── spring-core/                    # Pure math spring solver
    │   ├── egui-spring/                    # Spring-animated widgets
    │   ├── egui-layout/                    # Responsive split layouts
    │   ├── egui-vim-nav/                   # Vim keyboard navigation
    │   ├── egui-themes/                    # Token-based theming
    │   └── egui-nav-stack/                 # Screen navigation stack
    └── test-app/                           # Interactive demo application
```

## Running the Demo App

```bash
cargo run -p test-app
```

The test app includes interactive demos for every crate: responsive layouts, spring physics visualization, Vim navigation with keyboard rendering, screen navigation with undo/redo, and live theme switching across 12 palettes.

## Running Tests

```bash
cargo test --workspace --all-features
```

## Running Examples

Each crate ships with standalone examples:

```bash
cargo run -p egui-layout --example nested_layout
cargo run -p egui-spring --example selection_highlight
cargo run -p egui-vim-nav --example grid_navigation
cargo run -p egui-nav-stack --example stack_navigation
cargo run -p egui-themes --example theme_switcher
```

## License

See individual crate `Cargo.toml` files for license information.
