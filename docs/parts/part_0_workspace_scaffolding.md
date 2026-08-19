# Part 0 — Workspace Architecture & Scaffolding

## Overview
Part 0 establishes the multi-crate virtual workspace architecture for `egui-widgetkit`. It enforces strict boundaries between reusable library crates and consuming applications, guaranteeing clean state ownership and zero version drift.

---

## Workspace Structure & Components

```
egui-lib/
├── Cargo.toml                      # Root virtual workspace pinning dependencies
├── docs/                           # Architectural PRDs, coding rules, & specifications
├── src/
│   ├── egui-widgetkit/             # Library crates (strictly isolated)
│   │   ├── egui-layout/            # Responsive nested splits & constraint engine
│   │   ├── spring-core/            # Analytical closed-form spring ODE solver (zero egui dep)
│   │   ├── egui-spring/            # Spring-driven selection highlights & Bézier smear
│   │   ├── egui-vim-nav/           # Intra-screen focus graph & HJKL navigation
│   │   ├── egui-nav-stack/         # Screen navigation back-stack (Navigation 3)
│   │   └── egui-themes/            # Token-based theming & palette switcher
│   └── test-app/                   # Showcase & verification application
```

---

## Key Design Guarantees

1. **Dependency Pinning (`[workspace.dependencies]`)**:
   - The workspace root pins `egui = "0.27"` and `eframe = "0.27"`.
   - All crates inherit versions via `egui = { workspace = true }`, eliminating version skew across the ecosystem.

2. **Strict Crate Isolation Rule (`CODING_RULES §1`)**:
   - No library crate under `src/egui-widgetkit/` may ever import, reference, or assume the existence of any type from `src/test-app/` or any specific application.
   - Each library crate is independently publishable to crates.io.

3. **Strict Module Convention (No `mod.rs`) (`CODING_RULES §8`)**:
   - Every module with children uses the Rust 2018+ naming pattern: `foo.rs` accompanied by a `foo/` directory.
   - Parent `.rs` files contain only `pub mod child;` declarations and `pub use child::PrimaryType;` re-exports — zero inline business logic.

---

## Settings Page Integration Potential

When creating a global **Settings / Diagnostics Page** in the application, Part 0 provides:
* **Crate Version & Build Metadata**: Display active workspace crate versions and compile-time features.
* **Feature Toggles**: Dynamic toggling of optional features (e.g. enabling/disabling `animated-transitions` for screen switches).
