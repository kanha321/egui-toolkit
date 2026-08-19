# Progress Tracker — egui-widgetkit

This document tracks implementation progress across all parts and subparts defined in [`docs/BUILD_PLAN.md`](file:///d:/randoms/egui-lib/docs/BUILD_PLAN.md).

---

## Overall Status

| Part | Description | Status | Tests / Verification |
|---|---|---|---|
| **Part 0** | Workspace Scaffolding | ✅ Completed | Root workspace configured; `cargo check --workspace` passes cleanly |
| **Part 1** | `egui-layout` | ✅ Completed | Constraint solver + `compute_min_length` implemented; 8 unit tests + 2 doctests passed |
| **Part 2** | `spring-core` (Pure Math) | ⬜ Pending | Closed-form ODE unit tests |
| **Part 3** | `egui-spring` (SpringRect) | ⬜ Pending | `selection_highlight` example |
| **Part 4** | `egui-vim-nav` (Keyboard Graph) | ⬜ Pending | `grid_navigation` example |
| **Part 5** | `egui-themes` (Theming & Preview) | ⬜ Pending | `theme_switcher` example |
| **Part 6** | `test-app` (Integration & Scenes) | ⬜ Pending | Isolated scenes + `combined_demo` |

---

## Detailed Task Checklist

### Part 0 — Workspace Scaffolding
- [x] Root `Cargo.toml` with `[workspace.dependencies]` (`egui = "0.27"`, `edition = "2021"`)
- [x] Directory skeleton matching `docs/FOLDER_STRUCTURE.md` (`src/egui-widgetkit/` + `src/test-app/`)
- [x] `.gitignore` & root `README.md`
- [x] Initialized Git repository
- [x] Verified: `cargo check --workspace` passes with 0 warnings
- [x] Verified: `cargo tree -p spring-core` confirms zero `egui` dependency

---

### Part 1 — `egui-layout`
- [x] `src/egui-widgetkit/egui-layout/Cargo.toml`
- [x] `src/egui-widgetkit/egui-layout/README.md`
- [x] `src/egui-widgetkit/egui-layout/src/lib.rs` (re-exports `Split` and `Size`)
- [x] `src/egui-widgetkit/egui-layout/src/size.rs` (`Size::Fraction`, `Size::Exact`, `Size::Remainder`, `.min_size()`, `.max_size()`)
- [x] `src/egui-widgetkit/egui-layout/src/split.rs`
  - [x] `Split::horizontal()` / `Split::vertical()`
  - [x] Builder methods: `.section()`, `.section_min()`, `.section_fixed()`, `.section_remainder()`, `.section_constrained()`
  - [x] Constraint solver with relaxation loop
  - [x] Minimum length calculation API: `min_primary_length()`, `compute_min_length()`
  - [x] Spacing deduction before fraction distribution
  - [x] Remainder absorption for sub-pixel boundary alignment
- [x] Tests:
  - [x] `tests/split_normalizes_fractions.rs` (8 unit tests passed)
  - [x] `tests/split_nested_sections.rs` (2 unit tests passed)
  - [x] Doctests: 2 passed
- [x] Example:
  - [x] `examples/nested_layout.rs` (compiled & verified)

---

### Part 2 — `spring-core` (Pure Math — Zero `egui` Dependency)
- [ ] `src/egui-widgetkit/spring-core/Cargo.toml` (zero `egui` dependency)
- [ ] `src/egui-widgetkit/spring-core/README.md`
- [ ] `src/egui-widgetkit/spring-core/src/lib.rs`
- [ ] **2.1** `params.rs` — `SpringParams` value type + presets (`gentle`, `snappy`, `bouncy`)
- [ ] **2.2** `solver.rs` — Closed-form step functions (underdamped, critically damped with $\epsilon$-band, overdamped)
- [ ] **2.3** `spring.rs` — `Spring` state struct with `.update(dt)` and `.is_settled()` checking position & velocity
- [ ] **2.4** Tests:
  - [ ] `tests/solver_matches_closed_form.rs`
  - [ ] `tests/settles_within_tolerance.rs`
- [ ] Verify: `cargo tree -p spring-core` confirms zero `egui` dependency

---

### Part 3 — `egui-spring`
- [ ] `src/egui-widgetkit/egui-spring/Cargo.toml` (depends on `spring-core`, `egui`)
- [ ] `src/egui-widgetkit/egui-spring/README.md`
- [ ] `src/egui-widgetkit/egui-spring/src/lib.rs`
- [ ] **3.1** `corner_springs.rs` — 4 independent corner `Spring` instances
- [ ] **3.2** `bezier.rs` — Bézier-rounded border geometry ($\kappa = \frac{4}{3}(\sqrt{2} - 1)$) with max smear clamp
- [ ] **3.3** `spring_rect.rs` — `SpringRect` widget (`.show(&mut self, ui) -> Response`, `request_repaint()` while unsettled)
- [ ] **3.4** Example:
  - [ ] `examples/selection_highlight.rs`

---

### Part 4 — `egui-vim-nav`
- [ ] `src/egui-widgetkit/egui-vim-nav/Cargo.toml`
- [ ] `src/egui-widgetkit/egui-vim-nav/README.md`
- [ ] `src/egui-widgetkit/egui-vim-nav/src/lib.rs`
- [ ] **4.1** `focus_graph.rs` — Generic node-id & directional neighbor graph structure
- [ ] **4.2** `navigator.rs` — Cursor & `move(direction)` with graceful edge fallback
- [ ] **4.3** `key_handler.rs` — HJKL & arrow mapping with `wants_keyboard_input()` guard
- [ ] **4.4** `register.rs` — Immediate-mode registration API with stable `egui::Id`s
- [ ] **4.5** Example:
  - [ ] `examples/grid_navigation.rs`

---

### Part 5 — `egui-themes`
- [ ] `src/egui-widgetkit/egui-themes/Cargo.toml`
- [ ] `src/egui-widgetkit/egui-themes/README.md`
- [ ] `src/egui-widgetkit/egui-themes/src/lib.rs`
- [ ] **5.1** `token.rs` — Extensible `ThemeToken` slot definitions
- [ ] **5.2** `palette.rs` — Curated palettes without global/static state
- [ ] **5.3** `switcher.rs` — Active palette state & `.set(palette)` applying to `ctx.style_mut()`
- [ ] **5.4** `preview.rs` — Live preview panel widget
- [ ] **5.5** Example:
  - [ ] `examples/theme_switcher.rs`

---

### Part 6 — `test-app` (Integration & Showcase App)
- [ ] `src/test-app/Cargo.toml` (path-depends on `egui-widgetkit/*` crates)
- [ ] **6.1** `src/test-app/src/main.rs` (minimal bootstrap script with dynamic min_inner_size)
- [ ] **6.2** `src/test-app/src/app.rs` & `app/` (`state.rs`, `update.rs`)
- [ ] **6.3** Isolated demo scenes in `scenes/`:
  - [ ] `scenes/layout_demo.rs` (exercises `egui-layout`)
  - [ ] `scenes/spring_demo.rs` (exercises `spring-core` + `egui-spring`)
  - [ ] `scenes/vim_nav_demo.rs` (exercises `egui-vim-nav`)
  - [ ] `scenes/theme_demo.rs` (exercises `egui-themes`)
- [ ] **6.4** `scenes/combined_demo.rs` (Full integrated multi-crate showcase: layout + spring highlight + themes + vim nav)

---

## Log of Completed Milestones
- **Part 0 Completed**: Root virtual workspace `Cargo.toml` configured with `[workspace.dependencies]` pinning `egui = "0.27"`, git initialized with `.gitignore`, full directory skeletons built under `src/egui-widgetkit/` and `src/test-app/`, verified with `cargo check --workspace` (0 warnings).
- **Part 1 Completed & Hardened**: Upgraded `egui-layout` to a constraint-aware layout solver supporting `Size::Fraction`, `Size::Exact`, `Size::Remainder`, `.section_min()`, and `.section_fixed()`. Added `compute_min_length()` API and configured `test-app` window min inner size. 10/10 tests passing.
