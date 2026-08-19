# egui-widgetkit Mandatory Rules

## Pre-Task Routine
For every request and before taking any action:
1. Always read `docs/ARCHITECTURE_PRD.md`, `docs/FOLDER_STRUCTURE.md`, `docs/CODING_RULES.md`, and `docs/BUILD_PLAN.md`.
2. Apply their constraints to all code, architecture, and discussions.

## Key Constraints
- Reusable crates belong in `src/egui-widgetkit/`.
- The test app belongs in `src/test-app/`.
- No `mod.rs` files anywhere (`foo.rs` alongside `foo/` directory).
- `spring-core` has zero egui dependency.
- State ownership is explicit and passed by `&mut` reference (no static/global state).
- All drawing is via `egui::Shape`s with `request_repaint()` during animation.
- Corner Rounding Invariant: The 4 rounded corners of widgets and cards must always stay fully rendered and intact.
- Total isolation: no crate in `src/egui-widgetkit/` may reference `test-app`.
- Follow the incremental step-by-step build order in `docs/BUILD_PLAN.md`.
