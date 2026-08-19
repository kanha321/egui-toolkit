# Workspace Rules — egui-widgetkit

## 1. Mandatory Pre-Task Instruction (CRITICAL)
Before responding to ANY user request or executing ANY task in this workspace, you MUST ALWAYS read and consult the core specification documents in `docs/`:
1. `docs/ARCHITECTURE_PRD.md` — Project requirements, library crate responsibilities, and single-repo workspace design.
2. `docs/FOLDER_STRUCTURE.md` — Exact workspace layout (`src/egui-widgetkit/` vs `src/test-app/`) and module conventions.
3. `docs/CODING_RULES.md` — Enforceable coding standards, state ownership rules, and crate dependencies.
4. `docs/BUILD_PLAN.md` — Incremental phased build plan, subpart milestones, and common pitfalls to avoid.

---

## 2. Workspace & Crate Architecture
- The library is **`egui-widgetkit`** and lives strictly in `src/egui-widgetkit/`:
  - `egui-layout`: Responsive nested splits (`Split::horizontal()`, `Split::vertical()`).
  - `spring-core`: Pure math analytical spring physics (**zero `egui` dependency**).
  - `egui-spring`: Spring-animated selection highlights (`SpringRect`, `egui::Shape` output).
  - `egui-vim-nav`: Vim-style (HJKL) keyboard navigation.
  - `egui-themes`: Token-based theming, palette presets, and live preview widget.
- The test application lives at `src/test-app/`.
- **Isolation Rule**: No crate under `src/egui-widgetkit/` may ever import, reference, or assume the existence of any type from `src/test-app/` or any specific application.

---

## 3. Strict Module Convention (No `mod.rs`)
- **NEVER create `mod.rs` files.**
- Every module with children must be a `foo.rs` file sitting next to a `foo/` directory.
- The `foo.rs` file must contain ONLY `pub mod child;` declarations and `pub use child::PrimaryType;` re-exports — never actual logic.

---

## 4. State Ownership & Rendering Standards
- **No static/global mutable state**: All interactive and animated state must be plain structs owned by the consuming application and passed by `&mut` reference.
- **`egui::Shape` Only**: Never use raw `wgpu`/`glow` or custom `PaintCallback` render pipelines.
- **Continuous Motion Repaint**: Any animated widget must call `ctx.request_repaint()` every frame until settled.
- **Public API Pattern**: Follow the builder pattern: `Type::new()` / `Type::named()` → chained setters → `.show(ui: &mut Ui) -> Response` (or `.update(&mut self, dt)`).
- **Corner Rounding Invariant**: The 4 rounded corners of widgets, cards, and highlights must always stay fully rendered and intact across all window sizes (never sliced flat by parent clipping bounds).
