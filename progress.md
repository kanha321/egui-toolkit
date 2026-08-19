# Progress Tracker — egui-widgetkit

This document tracks implementation progress across all parts and subparts defined in [`docs/BUILD_PLAN.md`](file:///d:/randoms/egui-lib/docs/BUILD_PLAN.md) (v6).

---

## Overall Status

| Part | Description | Status | Tests / Verification |
|---|---|---|---|
| **Part 0** | Workspace Scaffolding (All 6 Library Crates + App) | ✅ Completed | Root workspace configured; `cargo check --workspace` passes cleanly |
| **Part 1** | `egui-layout` (Constraint Solver & Spacing Engine) | ✅ Completed | Constraint solver + `compute_min_length` implemented; 8 unit tests + 2 doctests passed |
| **Part 2** | `spring-core` (Pure Math ODE Solver) | ✅ Completed | Analytical closed-form solver; 8 unit tests + 1 doctest passed; zero egui dep verified |
| **Part 3** | `egui-spring` (SpringRect & Bézier Smear) | ✅ Completed | 4-corner independent springs, Bézier smear geometry, Shape emission; 3 unit tests passed |
| **Part 4** | `egui-vim-nav` (Intra-Screen Focus Graph) | ⬜ Pending | `grid_navigation` example |
| **Part 5** | `egui-themes` (Token-Based Theming & Live Preview) | ⬜ Pending | `theme_switcher` example |
| **Part 6** | `egui-nav-stack` (Navigation 3 Screen Back-Stack) | ⬜ Pending | `stack_push_pop_replace` tests + `stack_navigation` example |
| **Part 7** | `test-app` (Multi-Crate Integration & Showcase App) | ⬜ Pending | Isolated scenes + `combined_demo` |

---

## Detailed Task Checklist

### Part 0 — Workspace Scaffolding
- [x] Root `Cargo.toml` with `[workspace.dependencies]` (`egui = "0.27"`, `edition = "2021"`)
- [x] Directory skeletons for all 6 library crates (`src/egui-widgetkit/`) + `src/test-app/` matching `docs/FOLDER_STRUCTURE.md`
- [x] `.gitignore` & root `README.md`
- [x] Initialized Git repository (branch `feat/spring/b1`)
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
- [x] `src/egui-widgetkit/spring-core/Cargo.toml` (zero `egui` dependency)
- [x] `src/egui-widgetkit/spring-core/README.md`
- [x] `src/egui-widgetkit/spring-core/src/lib.rs`
- [x] **2.1** `params.rs` — `SpringParams` value type + presets (`gentle`, `snappy`, `bouncy`)
- [x] **2.2** `solver.rs` — Closed-form step functions (underdamped, critically damped with $\epsilon$-band, overdamped)
- [x] **2.3** `spring.rs` — `Spring` state struct with `.update(dt)` and `.is_settled()` checking position & velocity
- [x] **2.4** Tests:
  - [x] `tests/solver_matches_closed_form.rs` (5 tests passed: underdamped, critically damped, overdamped, framerate invariance, epsilon band continuity)
  - [x] `tests/settles_within_tolerance.rs` (3 tests passed: settle & snap clean, moving-through-target protection, all presets settle)
- [x] Verify: `cargo tree -p spring-core` confirms zero `egui` dependency

---

### Part 3 — `egui-spring`
- [x] `src/egui-widgetkit/egui-spring/Cargo.toml` (depends on `spring-core`, `egui`)
- [x] `src/egui-widgetkit/egui-spring/README.md`
- [x] `src/egui-widgetkit/egui-spring/src/lib.rs`
- [x] **3.1** `corner_springs.rs` — 4 independent corner `Spring` instances with direction-aware stretch & logarithmic damping
- [x] **3.2** `bezier.rs` — Bézier-rounded border geometry ($\kappa = \frac{4}{3}(\sqrt{2} - 1)$) with intact corner bounds
- [x] **3.3** `spring_rect.rs` — `SpringRect` widget (`.show(&mut self, ui) -> Response`, `request_repaint()` while unsettled)
- [x] **3.4** Tests & Example:
  - [x] `tests/spring_rect_tests.rs` (3 unit tests passed)
  - [x] `examples/selection_highlight.rs` (compiled & verified)

---

### Part 4 — `egui-vim-nav` (Focus Navigation Within One Screen)
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

### Part 6 — `egui-nav-stack` (Navigation 3 Screen Back-Stack)
- [ ] `src/egui-widgetkit/egui-nav-stack/Cargo.toml` (`optional = true` for `egui-spring`)
- [ ] `src/egui-widgetkit/egui-nav-stack/README.md`
- [ ] `src/egui-widgetkit/egui-nav-stack/src/lib.rs`
- [ ] **6.1** `stack.rs` — `NavStack<K>` (`push`, `pop`, `replace_top`, `pop_to(predicate)`, `top`)
- [ ] **6.2** Entry resolution (inline closure in `display.rs`)
- [ ] **6.3** `display.rs` — `NavDisplay<K>` widget returning `Option<NavAction<K>>` requests
- [ ] **6.4** `transition.rs` — `#[cfg(feature = "animated-transitions")]` spring push/pop transition
- [ ] **6.5** Tests & Example:
  - [ ] `tests/stack_push_pop_replace.rs` (plain data assertions)
  - [ ] `examples/stack_navigation.rs`

---

### Part 7 — `test-app` (Multi-Crate Integration & Showcase App)
- [ ] `src/test-app/Cargo.toml` (path-depends on all 6 `egui-widgetkit/*` crates)
- [ ] **7.1** `src/test-app/src/main.rs` (minimal bootstrap script with windows subsystem and dynamic min_inner_size)
- [ ] **7.2** `src/test-app/src/app.rs` & `app/` (`state.rs`, `update.rs`)
- [ ] **7.3** Isolated demo scenes in `scenes/`:
  - [x] `scenes/layout_demo.rs` (exercises `egui-layout` with dynamic min size & rounded corners)
  - [x] `scenes/spring_demo.rs` (exercises `spring-core` + `egui-spring`)
  - [ ] `scenes/vim_nav_demo.rs` (exercises `egui-vim-nav`)
  - [ ] `scenes/nav_stack_demo.rs` (exercises `egui-nav-stack`)
  - [ ] `scenes/theme_demo.rs` (exercises `egui-themes`)
- [ ] **7.4** `scenes/combined_demo.rs` (Full integrated multi-crate showcase: layout + spring highlight + nav-stack + themes + vim nav)

---

## Log of Completed Milestones
- **Part 0 Completed**: Root virtual workspace `Cargo.toml` configured with `[workspace.dependencies]` pinning `egui = "0.27"`, git initialized with `.gitignore`, full directory skeletons built under `src/egui-widgetkit/` (all 6 crates) and `src/test-app/`, verified with `cargo check --workspace` (0 warnings).
- **Part 1 Completed & Hardened**: Upgraded `egui-layout` to a constraint-aware layout solver supporting `Size::Fraction`, `Size::Exact`, `Size::Remainder`, `.section_min()`, and `.section_fixed()`. Added `compute_min_length()` API and configured `test-app` dynamic window min inner size with corner rounding protection. 10/10 tests passing.
- **Part 2 Completed & Verified**: Built `spring-core` pure math analytical closed-form ODE solver covering underdamped, critically damped ($\epsilon$-band snapped), and overdamped regimes with `SpringParams` presets and framerate-independent step invariance. 8 unit tests + 1 doctest passed; confirmed zero `egui` dependency via `cargo tree -p spring-core`.
- **Part 3 Completed & Verified**: Built `egui-spring` with `CornerSprings` (4 independent 2D spring corners with travel-alignment stretch and logarithmic damping), `build_bezier_boundary` ($\kappa = 0.55228$), and `SpringRect` emitting pure `egui::Shape` primitives with continuous motion repaint. 3 unit tests and standalone example verified.
