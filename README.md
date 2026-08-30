# egui-widgetkit

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org)
[![egui](https://img.shields.io/badge/egui-0.27-blue.svg?style=flat-square)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg?style=flat-square)](Cargo.toml)
[![Tests](https://img.shields.io/badge/tests-186%20passing-brightgreen.svg?style=flat-square)](Cargo.toml)

**`egui-widgetkit` is a modular, high-contrast, tactile UI engine and component suite for [egui](https://github.com/emilk/egui).**

It bridges the gap between raw immediate-mode graphics and the polished, organic feel of modern high-end desktop applications (like Linear, Raycast, and Blender)—combining **analytical ODE spring physics**, **Vim-style modal keyboard navigation**, **in-place morphing geometry**, and **token-driven semantic palettes** into a decoupled, zero-global-state architecture.

---

## 🎯 The Purpose & Vision

Traditional immediate-mode UIs often feel static, linear, or visually disconnected:
- Colors are hardcoded with ad-hoc hex values.
- Animations rely on linear `lerp` timers rather than physical mass and velocity.
- Mouse hover states exist in isolation from keyboard focus.
- Popups and dropdowns spawn as disconnected floating windows that clip or stutter.

**`egui-widgetkit` re-imagines egui with five core pillars:**

1. **Physical Tactility (Game-Feel Physics)**  
   Every interaction—a button click, a tab switch, a dropdown expansion, or a slider adjustment—is driven by closed-form analytical spring ODEs. UI elements carry simulated mass, momentum, and elasticity. When an animation finishes, physics settle cleanly to **0.00% CPU utilization**.

2. **Keyboard-First & Vim Modal Agility**  
   Your hands never need to leave the home row. Directional focus graphs let you navigate complex layouts using `H`/`J`/`K`/`L` or arrow keys. Text inputs feature a full modal editing engine with Normal, Insert, and Visual modes, text objects (`iw`, `i"`), operators (`d`, `c`, `y`), and register memory.

3. **Unified Highlight-Focus Paradigm**  
   There is no separate "mouse hover" vs "keyboard focus" logic. Both mouse pointer movement and keyboard navigation feed into the same spatial navigator. When an element is focused, it animates with identical luminance glides, elevation lifts, and bounce pulses regardless of input source.

4. **In-Place Morphing Over Jarring Overlays**  
   Menus, dropdowns, and edit fields morph smoothly within the layout rather than spawning detached OS windows or jarring instant overlays.

5. **Token-Based Design System**  
   Zero hardcoded colors. All components automatically derive their background fills, active states, border strokes, and typographic hierarchy from unified semantic tokens (`base`, `surface0..2`, `accent`, `text`, `danger`, etc.) with instant live palette switching.

---

## 🌟 The Feature Showroom

```
                               ┌────────────────────────┐
                               │     egui-widgetkit     │
                               └───────────┬────────────┘
         ┌───────────────────┬─────────────┼───────────────┬───────────────────┐
         ▼                   ▼             ▼               ▼                   ▼
  ⚡ Spring Physics   ⌨️ Vim Nav     🎨 Themes       🧩 Widgets       📐 Layout & Stacks
  (ODE math engine)   (HJKL & Modal) (12 Palettes)   (11 Components)  (Splits & Screens)
```

---

### 1. ⚡ Analytical Spring Physics (`spring-core` & `egui-spring`)
*True physical mass-spring-damper dynamics without frame-rate dependency.*

- **Closed-Form ODE Solver**: Computes position and velocity instantaneously via exact analytical equations ($x(t)$ and $v(t)$) for underdamped, critically damped, and overdamped regimes.
- **Micro-Interaction Tuning**:
  - **Press Squash & Pop**: Compresses on mouse-down/Enter, snaps back on release with organic overshoot.
  - **Focus Jiggle**: Injects a momentary velocity impulse upon selection arrival.
  - **Continuous Spring Highlights (`SpringRect`)**: A dynamic bounding box that stretches, squashes, and glides smoothly as focus moves between elements.
- **Smart Idle Repaint**: Automatically calls `ctx.request_repaint()` every frame while springs are in motion, and seamlessly sleeps when all springs settle.

---

### 2. ⌨️ Vim Navigation & Modal Text Engine (`egui-vim-nav`)
*Full spatial keyboard navigation and embedded modal editing.*

- **Spatial Focus Graph**: Define your layout as a graph of nodes with directional edges (`connect_horizontal`, `connect_vertical`, `connect_grid`).
- **2-Tier Hierarchical Traversal**: First navigates within the current card/section, then seamlessly crosses section boundaries when hitting margins.
- **Scrolloff Viewport Keeping**: Automatic spring-driven viewport scrolling that keeps the focused widget framed with comfortable padding margins (just like Vim's `scrolloff`).
- **Modal Text Buffer (`VimBufferState`)**:
  - **Modes**: Normal, Insert, Visual (character-wise), and Operator-Pending.
  - **Motions**: `h`/`j`/`k`/`l`, `w`/`b`/`e`, `0`/`$`, `gg`/`G`, `f{char}`/`t{char}`.
  - **Operators**: Delete (`d`), Change (`c`), Yank (`y`), Paste (`p`), Replace (`r`), Undo (`u`).
  - **Text Objects**: Inner word (`iw`), inner quotes (`i"`), inner parens (`i(`).
- **Cursor Autohide**: Hides the mouse cursor during keyboard navigation to prevent visual clutter, automatically restoring it when the mouse moves.

---

### 3. 🧩 Tactile Component Suite (`egui-widgets`)
*11 customizable, spring-animated, theme-aware components.*

- **Button**: 7 semantic variants (Primary, Secondary, Ghost, Danger, Outline, Success, Warning) with icon, badge, and shortcut slots.
- **Card**: Elevated container panels with optional spring focus lift, ambient drop shadows, and structured header layouts.
- **Dropdown**: In-place height morphing with trajectory springs, 2-tier highlights (saved item marker + sliding focus pill), and selection-anchored expansion framing.
- **Switch**: Elastic thumb translation, smooth track color crossfading, and press squash.
- **Slider**: Spring-scaling knob thumb, kinetic badge catchup, direct drag, and modal inline typing (`i` to type value, Enter to commit).
- **TextInput**: Inverted high-contrast block cursor, spring focus glow ring, and integrated Vim buffer state.
- **SegmentedTabs**: Sliding indicator pill that physically tracks tab position and width changes.
- **Checkbox & RadioButton**: Physics-driven stroke draw animations and concentric dot expansions with scale overshoot.
- **Badge & ProgressBar**: Elastic width badges with live pulse dots, and progress meters with physical spring catchup.

---

### 4. 🎨 Token-Based Design System (`egui-themes`)
*Cohesive, designer-grade color harmony across the entire UI.*

- **12 Curated Palettes Ready Out-of-the-Box**:
  - 🐱 **Catppuccin**: Mocha, Macchiato, Frappé, Latte
  - 🧛 **Dracula**
  - 🏙️ **Tokyo Night**: Dark, Storm, Light
  - 🪵 **Gruvbox**: Dark, Light
  - ❄️ **Nord**
  - 🌌 **One Dark**
- **Semantic Token Hierarchy**: Colors are organized logically (`base`, `mantle`, `crust`, `surface0..2`, `text`, `subtext0..1`, `accent`, `danger`, `warning`, `success`).
- **Runtime Theme Morphing**: Live switching interpolates colors smoothly across frames and automatically synchronizes with `egui::Visuals`.

---

### 5. 📐 Split Layouts & Screen Transitions (`egui-layout` & `egui-nav-stack`)
*Responsive structure and fluid view management.*

- **Nested Declarative Splits**: Build complex multi-pane layouts with `Split::horizontal()` and `Split::vertical()` using exact, proportional, or auto-expanding sizes.
- **App-Owned Navigation Stack (`NavStack`)**: Type-safe screen management with push, pop, replace, and back operations.
- **Physics Slide Transitions**: Smooth horizontal screen slide-in / slide-out animations between views.

---

## 📚 Complete Documentation Directory

Every crate and component has a dedicated, comprehensive usage guide with full API references and copy-pasteable examples:

### 📦 Core Library Crates

| Guide | Crate Path | What It Covers |
| :--- | :--- | :--- |
| [**`spring-core`**](docs/usage/spring-core.md) | `src/egui-widgetkit/spring-core` | ODE physics math, `Spring`, `SpringParams`, parameter tuning |
| [**`egui-themes`**](docs/usage/egui-themes.md) | `src/egui-widgetkit/egui-themes` | `ThemePalette`, 12 presets, `ThemeState`, color interpolation |
| [**`egui-spring`**](docs/usage/egui-spring.md) | `src/egui-widgetkit/egui-spring` | `SpringRect`, `HighlightGroup`, `CornerSprings`, highlight sync |
| [**`egui-vim-nav`**](docs/usage/egui-vim-nav.md) | `src/egui-widgetkit/egui-vim-nav` | Focus graph, Navigator, Vim modal text buffer, scrolloff |
| [**`egui-layout`**](docs/usage/egui-layout.md) | `src/egui-widgetkit/egui-layout` | `Split::horizontal()`, `Split::vertical()`, section sizing |
| [**`egui-nav-stack`**](docs/usage/egui-nav-stack.md) | `src/egui-widgetkit/egui-nav-stack` | `NavStack`, animated screen transitions, routing |

### 🧩 UI Components (`egui-widgets`)

| Guide | Component | Key Highlights |
| :--- | :--- | :--- |
| [**Button**](docs/usage/button.md) | `Button`, `ButtonState` | Tactile bounce, 7 variants, size presets, icon/badge slots |
| [**Card**](docs/usage/card.md) | `Card`, `CardState` | Focus elevation lift, drop shadow, header slots, focus rings |
| [**Dropdown**](docs/usage/dropdown.md) | `Dropdown`, `DropdownState` | In-place morphing, 2-tier highlights, scrolloff, rich options |
| [**Switch**](docs/usage/switch.md) | `Switch`, `SwitchState` | Elastic thumb spring, smooth color crossfade, focus bounce |
| [**Slider**](docs/usage/slider.md) | `Slider`, `SliderState` | Spring knob, kinetic badge, inline typing modal (`i`/`e`) |
| [**TextInput**](docs/usage/text-input.md) | `TextInput`, `InputState` | High-contrast block cursor, spring focus glow, Vim buffer |
| [**Segmented Tabs**](docs/usage/tabs.md) | `SegmentedTabs`, `TabsState` | Sliding pill indicator, spring width/position tracking |
| [**Checkbox & Radio**](docs/usage/checkbox.md) | `Checkbox`, `RadioButton` | Stroke drawing animation, concentric radio dots, overshoot |
| [**Badge**](docs/usage/badge.md) | `Badge`, `BadgeVariant` | Dynamic width spring, status dots, semantic colors |
| [**Progress Bar**](docs/usage/progress-bar.md) | `ProgressBar`, `ProgressState` | Elastic fill catchup, striped animated variant |

---

## ⚡ Quick Integration

Add the crates to your `Cargo.toml` using local paths or Git:

```toml
[dependencies]
egui = "0.27"
eframe = "0.27"

egui-widgets = { path = "path/to/egui-lib/src/egui-widgetkit/egui-widgets" }
egui-themes  = { path = "path/to/egui-lib/src/egui-widgetkit/egui-themes" }
egui-vim-nav = { path = "path/to/egui-lib/src/egui-widgetkit/egui-vim-nav" }
```

### Minimal Themed Dashboard:

```rust
use eframe::egui;
use egui_themes::{ThemePalette, ThemePreset};
use egui_widgets::{Button, Card, Slider, Switch};

struct App {
    palette: ThemePalette,
    turbo: bool,
    power: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            palette: ThemePreset::TokyoNightStorm.palette(),
            turbo: true,
            power: 75.0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.palette.base))
            .show(ctx, |ui| {
                Card::new()
                    .palette(&self.palette)
                    .title("System Controls")
                    .subtitle("Spring physics enabled")
                    .show(ui, |ui| {
                        Switch::new(&mut self.turbo)
                            .palette(&self.palette)
                            .label("Hyperdrive")
                            .show(ui);

                        Slider::new(&mut self.power, 0.0..=100.0)
                            .palette(&self.palette)
                            .label("Output")
                            .suffix(" %")
                            .show(ui);

                        if Button::new("Engage Engine")
                            .palette(&self.palette)
                            .primary()
                            .show(ui)
                            .clicked()
                        {
                            println!("Engaged at {}%", self.power);
                        }
                    });
            });
    }
}
```

---

## 🏛️ Architectural Invariants

`egui-widgetkit` enforces strict architectural guarantees across all crates:

1. **Explicit State Ownership**: All state is owned by your application struct (`&mut State`) or safely isolated in egui's ID memory. No global singletons, hidden static variables, or mysterious background threads.
2. **Pure `egui::Shape` Output**: 100% standard egui mesh output. Zero custom `wgpu`/`glow` pipelines or opaque `PaintCallback` calls—making it completely portable to WebAssembly (Wasm) and all egui backends.
3. **Continuous Motion Repaint Loop**: Animated widgets request continuous redraws (`ctx.request_repaint()`) only while physical velocities are nonzero. The moment springs settle, CPU usage drops to idle.
4. **Intact Corner Rounding**: Rounded corners on cards, pills, buttons, and highlights are guaranteed never to clip flat against parent boundaries.
5. **No Hover Residue**: Unified focus architecture ensures mouse pointers and keyboard events animate the exact same tactile highlights.

---

## 🚀 Running the Interactive Showcase

The workspace includes a rich, multi-scene interactive test application at `src/test-app`:

```bash
# Run the interactive showcase application
cargo run -p test-app

# Or in release mode for maximum physics smoothness
cargo run -p test-app --release
```

### Running the Test Suite:

```bash
# Run all 186 unit, integration, and doc tests across all 7 crates
cargo test --workspace
```

---

## 📄 License

Dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.
