# egui-widgetkit

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org)
[![egui](https://img.shields.io/badge/egui-0.27-blue.svg?style=flat-square)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg?style=flat-square)](Cargo.toml)
[![Tests](https://img.shields.io/badge/tests-186%20passing-brightgreen.svg?style=flat-square)](Cargo.toml)

A comprehensive, modular suite of high-contrast, physics-driven, keyboard-navigable UI crates for **[egui](https://github.com/emilk/egui)**.

`egui-widgetkit` combines **analytical ODE spring physics**, **token-based semantic theming**, **Vim-style modal keyboard navigation**, **responsive nested splits**, and a rich library of **animated interactive widgets** into a decoupled single-repository workspace.

---

## Workspace Crates

The library lives under `src/egui-widgetkit/` and is divided into 7 focused crates:

| Crate | Path | Description | Docs |
| :--- | :--- | :--- | :--- |
| **`spring-core`** | [`src/.../spring-core`](src/egui-widgetkit/spring-core) | Pure mathematical analytical spring physics solver (closed-form ODE, **zero `egui` dependency**). | [Guide](docs/usage/spring-core.md) |
| **`egui-themes`** | [`src/.../egui-themes`](src/egui-widgetkit/egui-themes) | Token-based theming engine with 12 curated palettes (Catppuccin, Dracula, TokyoNight, etc.) and live preview gallery. | [Guide](docs/usage/egui-themes.md) |
| **`egui-spring`** | [`src/.../egui-spring`](src/egui-widgetkit/egui-spring) | Animated selection highlights, continuous spring bounding rects (`SpringRect`), highlight groups, and cursor springs. | [Guide](docs/usage/egui-spring.md) |
| **`egui-vim-nav`** | [`src/.../egui-vim-nav`](src/egui-widgetkit/egui-vim-nav) | Spatial focus graphs, HJKL directional navigation, scrolloff margins, cursor autohide, and modal Vim buffer editing. | [Guide](docs/usage/egui-vim-nav.md) |
| **`egui-widgets`** | [`src/.../egui-widgets`](src/egui-widgetkit/egui-widgets) | Full suite of tactile, animated, theme-aware UI components (Button, Card, Dropdown, Slider, Switch, TextInput, Tabs, etc.). | [Docs Index](docs/usage/) |
| **`egui-layout`** | [`src/.../egui-layout`](src/egui-widgetkit/egui-layout) | Responsive, nested declarative split layouts (`Split::horizontal()`, `Split::vertical()`) with card framing. | [Guide](docs/usage/egui-layout.md) |
| **`egui-nav-stack`** | [`src/.../egui-nav-stack`](src/egui-widgetkit/egui-nav-stack) | App-owned back-stack screen navigation with spring-animated horizontal slide transitions. | [Guide](docs/usage/egui-nav-stack.md) |

---

## Component Inventory (`egui-widgets`)

All components feature **micro-interaction spring physics**, **token palette integration**, **Vim navigation bindings**, and **stateful/stateless duality**:

| Component | Description | Highlights | Usage Doc |
| :--- | :--- | :--- | :--- |
| **`Button`** | Tactile pop buttons | Press squash, focus bounce impulse, luminance glide, 7 semantic variants, badges, shortcut slots | [`docs/usage/button.md`](docs/usage/button.md) |
| **`Card`** | Container surface panels | Spring focus lift, ambient drop shadow, header slot, interactive mode, focus ring border | [`docs/usage/card.md`](docs/usage/card.md) |
| **`Dropdown`** | In-place morphing menu | 2-tier highlights (selection + sliding focus pill), scrolloff margins, selection-anchored framing | [`docs/usage/dropdown.md`](docs/usage/dropdown.md) |
| **`Switch`** | Fluid toggle switch | Elastic thumb travel, smooth track color crossfade, press squash, focus jiggle | [`docs/usage/switch.md`](docs/usage/switch.md) |
| **`Slider`** | Kinetic numeric slider | Dynamic scaling thumb, kinetic badge catchup, inline typing modal (`i`/`e`), direct click | [`docs/usage/slider.md`](docs/usage/slider.md) |
| **`TextInput`** | Modal text field | High-contrast block cursor with color inversion, spring focus glow, full Vim buffer editing | [`docs/usage/text-input.md`](docs/usage/text-input.md) |
| **`SegmentedTabs`** | Pill tab switcher | Continuous sliding indicator pill with spring width/position tracking, badges, icons | [`docs/usage/tabs.md`](docs/usage/tabs.md) |
| **`Checkbox`** | Animated check box | Physics-driven checkmark stroke draw, scale bounce overshoot, label alignment | [`docs/usage/checkbox.md`](docs/usage/checkbox.md) |
| **`RadioButton`** | Elastic radio dot | Concentric circle expansion, scale bounce, mutually exclusive groups | [`docs/usage/checkbox.md`](docs/usage/checkbox.md) |
| **`Badge`** | Status tags & indicators | Dynamic width spring, semantic variant colors, optional live pulse dot | [`docs/usage/badge.md`](docs/usage/badge.md) |
| **`ProgressBar`** | Kinetic progress meter | Elastic fill bar with physical spring catchup, striped animated variant | [`docs/usage/progress-bar.md`](docs/usage/progress-bar.md) |

---

## Quick Start

### 1. Add Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
egui = "0.27"
eframe = "0.27"

# Reference crates locally or via Git
egui-widgets   = { path = "path/to/egui-lib/src/egui-widgetkit/egui-widgets" }
egui-themes    = { path = "path/to/egui-lib/src/egui-widgetkit/egui-themes" }
egui-vim-nav   = { path = "path/to/egui-lib/src/egui-widgetkit/egui-vim-nav" }
egui-spring    = { path = "path/to/egui-lib/src/egui-widgetkit/egui-spring" }
egui-layout    = { path = "path/to/egui-lib/src/egui-widgetkit/egui-layout" }
egui-nav-stack = { path = "path/to/egui-lib/src/egui-widgetkit/egui-nav-stack" }
spring-core    = { path = "path/to/egui-lib/src/egui-widgetkit/spring-core" }
```

### 2. Complete Example App

```rust
use eframe::egui;
use egui_themes::{ThemePalette, ThemePreset};
use egui_widgets::{Button, Card, Dropdown, DropdownOption, SegmentedTabs, Slider, Switch, TabItem};

struct MyApp {
    palette: ThemePalette,
    active_tab: usize,
    turbo_mode: bool,
    bandwidth: f64,
    selected_region: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            palette: ThemePreset::CatppuccinMocha.palette(),
            active_tab: 0,
            turbo_mode: true,
            bandwidth: 42.0,
            selected_region: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.palette.base))
            .show(ctx, |ui| {
                ui.add_space(20.0);

                // Container Card with theme styling and spring focus lift
                Card::new()
                    .palette(&self.palette)
                    .title("Control Dashboard")
                    .subtitle("Spring-animated tactile widgets")
                    .show(ui, |ui| {
                        // Sliding Pill Tabs
                        SegmentedTabs::new(
                            &mut self.active_tab,
                            vec![
                                TabItem::new(0, "Overview").icon("📊"),
                                TabItem::new(1, "Network").icon("🌐"),
                                TabItem::new(2, "Settings").icon("⚙️"),
                            ],
                        )
                        .palette(&self.palette)
                        .show(ui);

                        ui.add_space(12.0);

                        // Elastic Switch
                        Switch::new(&mut self.turbo_mode)
                            .palette(&self.palette)
                            .label("Turbo Acceleration")
                            .show(ui);

                        ui.add_space(8.0);

                        // Kinetic Numeric Slider
                        Slider::new(&mut self.bandwidth, 0.0..=100.0)
                            .palette(&self.palette)
                            .label("Throughput")
                            .suffix(" GB/s")
                            .show(ui);

                        ui.add_space(8.0);

                        // In-Place Morphing Dropdown
                        let regions = vec![
                            DropdownOption::new(0, "US-East (N. Virginia)").icon("🇺🇸"),
                            DropdownOption::new(1, "EU-Central (Frankfurt)").icon("🇩🇪"),
                            DropdownOption::new(2, "AP-East (Tokyo)").icon("🇯🇵"),
                        ];
                        Dropdown::new("region_dropdown", &mut self.selected_region, &regions)
                            .palette(&self.palette)
                            .show(ui);

                        ui.add_space(16.0);

                        // Tactile Buttons
                        ui.horizontal(|ui| {
                            if Button::new("Deploy Changes")
                                .palette(&self.palette)
                                .primary()
                                .show(ui)
                                .clicked()
                            {
                                println!("Deployed!");
                            }

                            if Button::new("Reset")
                                .palette(&self.palette)
                                .ghost()
                                .show(ui)
                                .clicked()
                            {
                                self.bandwidth = 0.0;
                            }
                        });
                    });
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Widgetkit Demo",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
```

---

## Core Features & Subsystems

### 1. 🌸 Token-Based Theming (`egui-themes`)
- **12 Curated Presets**: Catppuccin (Mocha, Macchiato, Frappe, Latte), Dracula, TokyoNight (Dark/Storm/Light), Gruvbox (Dark/Light), Nord, and OneDark.
- **Design Tokens**: Single source of truth defining `base`, `mantle`, `crust`, `surface0..2`, `text`, `subtext0..1`, `accent`, `success`, `warning`, and `danger`.
- **Runtime Switching**: Smooth palette morphing with automatic `egui::Visuals` synchronization.

### 2. ⚡ Closed-Form Spring Physics (`spring-core` & `egui-spring`)
- **Pure Math Solver**: Closed-form ODE spring simulation for underdamped, critically damped, and overdamped regimes.
- **Continuous Motion Repaint**: Springs wake the UI with `ctx.request_repaint()` while in motion and idle at `0.00%` CPU when settled.
- **Bounding Highlights**: `SpringRect` provides smooth spatial interpolation across moving focus selections.

### 3. ⌨️ Vim Navigation & Modal Editing (`egui-vim-nav`)
- **Spatial Focus Graph**: Topology-agnostic node graph with HJKL directional edges and grid shortcuts.
- **Hierarchical 2-Pass Navigation**: Seamless movement within widget clusters and across layout sections.
- **Modal Text Buffer**: Normal, Insert, Visual, and Replace modes supporting motions (`w`, `b`, `0`, `$`, `f{char}`, `gg`, `G`), operators (`d`, `c`, `y`, `p`, `r`), text objects (`iw`, `i"`), registers, and undo/redo.
- **Scrolloff & Autohide**: Keeps focused items centered with comfort margins; autohides the mouse cursor on keyboard interaction.

### 4. 📐 Responsive Split Layouts (`egui-layout`)
- **Declarative Splits**: Compose responsive horizontal and vertical splits (`Split::horizontal()`, `Split::vertical()`).
- **Flexible Sizing**: Proportional weights, exact pixels, and auto-expanding panels with integrated card backgrounds.

### 5. 🗂️ Screen Navigation Stack (`egui-nav-stack`)
- **App-Owned Back-Stack**: Type-safe view management with forward/backward push/pop operations.
- **Animated Transitions**: Physical spring slide-in / slide-out transitions with configurable direction and easing.

---

## Architectural Principles & Invariants

1. **State Ownership**:
   All state is plain Rust structs owned by the consuming application (`&mut state`) or scoped to egui's temporary ID memory. Zero hidden singletons or global state.
2. **`egui::Shape` Rendering Only**:
   All visual output uses standard `egui::Shape` primitives. Zero raw `wgpu`/`glow` shader pipelines or opaque render callbacks.
3. **Unified Highlight Focus (No Raw Hover Residue)**:
   Widgets do not have disconnected mouse hover effects. Both pointer position and keyboard navigation feed into the same unified focus system (`.focused(bool)`), animating identical glides and bounce impulses.
4. **Corner Rounding Invariant**:
   All widget, card, and highlight corner roundings stay fully intact across arbitrary bounds without parent clipping distortion.

---

## Detailed Documentation (`docs/usage/`)

| Guide | Scope |
| :--- | :--- |
| 📖 [**Spring Core**](docs/usage/spring-core.md) | Analytical spring physics API, parameter tuning, ODE math |
| 📖 [**Layout Engine**](docs/usage/egui-layout.md) | Split layouts, section sizing, responsive layout containers |
| 📖 [**Spring Highlights**](docs/usage/egui-spring.md) | `SpringRect`, `HighlightGroup`, cursor physics, morphing boxes |
| 📖 [**Themes & Palettes**](docs/usage/egui-themes.md) | `ThemePalette`, 12 presets, live theme gallery, color lerp |
| 📖 [**Vim Navigation**](docs/usage/egui-vim-nav.md) | FocusGraph, Navigator, VimBuffer modal editing, scrolloff |
| 📖 [**Navigation Stack**](docs/usage/egui-nav-stack.md) | Screen back-stack, push/pop routing, animated transitions |
| 📖 [**Button**](docs/usage/button.md) | Button variants, sizes, icons, shortcut hints, badges |
| 📖 [**Card**](docs/usage/card.md) | Panel containers, focus lift, shadows, headers |
| 📖 [**Dropdown**](docs/usage/dropdown.md) | Morphing dropdown, 2-tier highlights, scrolloff, options |
| 📖 [**Switch**](docs/usage/switch.md) | Elastic toggle switch, track crossfade, label placement |
| 📖 [**Slider**](docs/usage/slider.md) | Kinetic slider, inline Vim typing modal, spring knob |
| 📖 [**TextInput**](docs/usage/text-input.md) | Inverted block cursor, focus glow ring, modal buffer |
| 📖 [**Segmented Tabs**](docs/usage/tabs.md) | Sliding pill switcher, badge counts, icons |
| 📖 [**Checkbox & Radio**](docs/usage/checkbox.md) | Animated checkmarks, radio dots, scale bounce overshoot |
| 📖 [**Badge**](docs/usage/badge.md) | Status badges, dynamic width spring, semantic colors |
| 📖 [**ProgressBar**](docs/usage/progress-bar.md) | Physical catchup bar, striped animation, percentage text |

---

## Interactive Demo Application

Run the bundled showcase app at `src/test-app/` to explore all widgets, physics visualizers, and navigation modes interactively:

```bash
# Launch interactive showcase
cargo run -p test-app

# Or in release mode for maximum fluidity
cargo run -p test-app --release
```

## Running the Test Suite

```bash
# Run all 186 unit, integration, and doc tests across all crates
cargo test --workspace
```

---

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
