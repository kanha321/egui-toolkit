# egui-widgetkit

A modular suite of app-agnostic UI crates for [egui](https://github.com/emilk/egui), providing responsive nested split layouts, pure analytical spring physics, elastic selection highlights, Vim-style keyboard navigation, and token-based theming.

## Crate Inventory

| Crate | Directory | Dependencies | Description |
|---|---|---|---|
| **`egui-layout`** | `src/egui-widgetkit/egui-layout` | `egui` | Declarative, responsive nested split layouts (`Split::horizontal()`, `Split::vertical()`). |
| **`spring-core`** | `src/egui-widgetkit/spring-core` | *(none)* | Framework-agnostic analytical spring physics solver (pure math, **zero `egui` dependency**). |
| **`egui-spring`** | `src/egui-widgetkit/egui-spring` | `spring-core`, `egui` | `SpringRect` and spring-driven widgets with Bézier borders and `request_repaint()`. |
| **`egui-vim-nav`** | `src/egui-widgetkit/egui-vim-nav` | `egui` | Vim-style (HJKL) keyboard navigation across an app-defined focus graph. |
| **`egui-themes`** | `src/egui-widgetkit/egui-themes` | `egui` | Token-based theme system, curated palettes, and live preview panel widget. |
| **`test-app`** | `src/test-app` | all above + `eframe` | Interactive showcase application and integration test bed. |

## Workspace Structure

```
.
├── Cargo.toml                              # Virtual workspace root
├── README.md
├── docs/
│   ├── ARCHITECTURE_PRD.md
│   ├── FOLDER_STRUCTURE.md
│   ├── CODING_RULES.md
│   ├── BUILD_PLAN.md
│   └── progress.md
└── src/
    ├── egui-widgetkit/                     # The reusable library crates
    │   ├── egui-layout/
    │   ├── spring-core/
    │   ├── egui-spring/
    │   ├── egui-vim-nav/
    │   └── egui-themes/
    └── test-app/                           # Integration test bed & demo application
```

## Running the Test Application

```bash
cargo run -p test-app
```
