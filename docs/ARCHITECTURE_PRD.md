# egui-widgetkit — Architecture PRD

**Status:** Draft v6 (supersedes v5 — adds a sixth crate,
`egui-nav-stack`, a back-stack-based screen/destination navigation
system modeled on the philosophy of Android's Navigation 3: the stack is
a plain list the consuming app owns, not state hidden inside a
controller. This is additive — nothing about the five existing crates or
the single-repo model changes.)

**Scope:** "egui-widgetkit" is the name of the library itself — the
collection of crates at `src/egui-widgetkit/` (layout, spring physics,
vim-style focus navigation, back-stack screen navigation, theming). It
is not the name of the whole repo:
the repo also contains, at `src/test-app/`, the actual app being built
today, and the repo can be named whatever that app is called. The
library stays fully app-agnostic in code (§1, §3) regardless of what the
outer repo is called or how many apps eventually depend on it.

---

## 1. Problem statement

Building a polished egui app (responsive nested layouts, spring-driven
selection highlights, vim-style keyboard navigation, consistent theming)
means writing the same non-trivial UI plumbing from scratch every time.
egui-widgetkit extracts that plumbing into small, focused, reusable
crates, kept in `src/egui-widgetkit/`, so the app in `src/test-app/` —
and, later, any other egui app that takes a path dependency on
`src/egui-widgetkit/` — gets that same polish without re-deriving it.

**This document describes the library only.** It contains zero
domain-specific types, zero business logic, and zero assumptions about
what kind of app consumes it — **including the app in `src/test-app/`
itself.** That app's own architecture (Clean Architecture, MVU, or
anything else) is entirely out of scope here and belongs in its own
docs, not this library's.

---

## 2. Goals

- Ship six focused, independently-usable crates (see §5) under
  `src/egui-widgetkit/` that `src/test-app/` — or any future egui app —
  can path-depend on individually. An app that only wants layout
  shouldn't have to pull in the spring or theming crates.
- Keep every crate **fully app-agnostic**: no crate in egui-widgetkit may
  import a type from, or encode an assumption about, any specific
  consuming application — `src/test-app/` included.
- Make animation feel smooth using **egui's existing GPU pipeline** —
  `request_repaint()` plus ordinary `egui::Shape` output — not a custom
  renderer.
- Keep the public API of each crate small, consistent, and predictable
  across crates (same builder shape, same naming conventions).

## 3. Non-goals

- **No custom GPU renderer, shader pipeline, or `PaintCallback`-based
  custom draw path.** Every widget in egui-widgetkit draws using
  ordinary `egui::Shape`s and lets whichever backend the *consuming app*
  chose (`wgpu` or `glow`, an app-level `eframe` feature choice)
  rasterize them.
- **No app-specific domain modeling.** Nothing resembling a concept from
  `src/test-app/` (or any other app) belongs in egui-widgetkit, ever —
  that stays in the app.
- **No crates.io publish in this phase** (decision below, §4).
- **No ECS, no networking, no persistence.** This is a UI-only library.

---

## 4. Packaging & distribution — decision

**Decision: single repo, single Cargo workspace — egui-widgetkit and the
app co-located.**

egui-widgetkit lives at `src/egui-widgetkit/` in this repo, alongside
the app that uses it at `src/test-app/`. Both are members of one
workspace, declared once at the repo root:

```toml
# Cargo.toml (repo root)
[workspace]
members = [
  "src/egui-widgetkit/egui-layout",
  "src/egui-widgetkit/spring-core",
  "src/egui-widgetkit/egui-spring",
  "src/egui-widgetkit/egui-vim-nav",
  "src/egui-widgetkit/egui-nav-stack",
  "src/egui-widgetkit/egui-themes",
  "src/test-app",
]
```

The app depends on whichever egui-widgetkit crates it needs via an
in-repo path dependency:

```toml
# src/test-app/Cargo.toml
[dependencies]
egui-layout   = { path = "../egui-widgetkit/egui-layout" }
spring-core   = { path = "../egui-widgetkit/spring-core" }
egui-spring   = { path = "../egui-widgetkit/egui-spring" }
egui-vim-nav  = { path = "../egui-widgetkit/egui-vim-nav" }
egui-nav-stack = { path = "../egui-widgetkit/egui-nav-stack" }
egui-themes   = { path = "../egui-widgetkit/egui-themes" }
eframe        = "0.27"
```

Each crate under `src/egui-widgetkit/` still keeps its own `Cargo.toml`
and version number, so it's independently versioned in spirit — that
part of the earlier reasoning is unchanged. What changed this round is
only the folder name: `library` → `egui-widgetkit`.

**Trigger to revisit:** if, later, a second and genuinely separate app —
in a different repo — wants egui-widgetkit, that's the point to extract
`src/egui-widgetkit/` into its own top-level repo/workspace, optionally
publishing to crates.io (worth checking the name is still free on
crates.io at that point — see FOLDER_STRUCTURE.md §9). Because every
crate already has zero knowledge of `src/test-app/` (§1, and
CODING_RULES §1), that extraction is a folder copy plus a new workspace
root — no internal restructuring required. Until a second app actually
exists, single-repo is the answer.

---

## 5. Crate inventory

| Crate | Status | Depends on | What it does |
|---|---|---|---|
| `egui-layout` | ✅ Implemented | `egui` only | Declarative, responsive nested split layouts (`Split::horizontal()/vertical()`) |
| `spring-core` | Stub | *(nothing — no `egui` dep)* | Framework-agnostic analytical spring physics solver (pure math) |
| `egui-spring` | Stub | `spring-core`, `egui` | `SpringRect` and other spring-driven widgets (elastic selection highlights) |
| `egui-vim-nav` | Stub | `egui` only | Vim-style (HJKL) keyboard navigation across nested UI elements *within one screen* |
| `egui-nav-stack` | Stub | `egui` only (optionally `egui-spring` — see §5.6) | App-owned back stack of screens/destinations + a display widget that renders the top of it, modeled on Android Navigation 3 |
| `egui-themes` | Stub | `egui` only | Theme tokens, switching, live preview panel |

Dependency direction within egui-widgetkit: `egui-spring` depends on
`spring-core`; `egui-nav-stack` *optionally* depends on `egui-spring`
(feature-gated, §5.6) for animated push/pop transitions; nothing else
cross-depends. Each crate can still be pulled in alone — the optional
dependency is off by default.

**`egui-vim-nav` vs. `egui-nav-stack` — different things, similar name:**
`egui-vim-nav` moves *focus* between regions already visible on one
screen (sidebar → controls → grid). `egui-nav-stack` moves *between
whole screens* (list → detail → settings), tracked as a stack the app
pushes and pops. An app can use either alone, both together, or neither.

### 5.1 `egui-layout` — implemented today

```rust
use egui_layout::Split;

Split::horizontal()
    .section(0.33, |ui| { ui.label("Sidebar"); })
    .section(0.67, |ui| {
        Split::vertical()
            .section(0.20, |ui| { ui.label("Header (20%)"); })
            .section(0.80, |ui| { ui.label("Content (80%)"); })
            .show(ui);
    })
    .show(ui);
```

```rust
impl<'a> Split<'a> {
    pub fn horizontal() -> Self;
    pub fn vertical() -> Self;
    pub fn spacing(mut self, px: f32) -> Self;          // default 4.0
    pub fn section(mut self, fraction: f32, add_contents: impl FnOnce(&mut Ui) + 'a) -> Self;
    pub fn show(self, ui: &mut Ui) -> Response;          // consumes available space
}
```

Fractions don't need to sum to `1.0` — normalized automatically. Every
fraction is recomputed from `ui.available_size()` each frame, so layouts
are responsive with no manual breakpoints.

**Naming note (open item):** `Split::horizontal()` names the split *line*
(left/right), `Split::vertical()` names the split line (top/bottom) — this
is the inverse of how some toolkits name by arrangement axis. Either
rename to `Split::row()`/`Split::column()` for unambiguity, or keep the
current names with a pinned doc comment. Decide before the API is used
outside `src/test-app/`.

### 5.2 `spring-core` — planned

Analytical (closed-form, not Euler-integrated) 1D spring solver,
framerate-independent, covering underdamped / critically-damped /
overdamped cases. Zero `egui` dependency — pure math, independently
testable and reusable outside egui entirely.

### 5.3 `egui-spring` — planned

`SpringRect`: four independent corner springs (via `spring-core`) driving
an "elastic smear" selection effect with Bézier-rounded borders. Emits
`egui::Shape`s only; calls `ctx.request_repaint()` every frame it hasn't
settled (see §7).

### 5.4 `egui-vim-nav` — planned

HJKL/arrow navigation across an app-defined graph of focusable
regions (sidebar → controls → sliders → grid, or whatever shape a given
consuming app has). The graph itself is built by the consuming app via a
registration API — this crate has no opinion on what the regions *are*.

### 5.5 `egui-themes` — planned

Token-based theme definitions (named color/spacing/radius slots), a
switcher, and a live-preview panel widget. Ships with a set of curated
built-in palettes but lets a consuming app define its own tokens too.

### 5.6 `egui-nav-stack` — planned

A back-stack-based screen navigation system, modeled on the philosophy
of Android's Navigation 3 library — **the back stack is a plain list the
consuming app owns**, not hidden state inside a controller object. This
crate ports that philosophy, not Navigation 3's literal Compose-shaped
API surface — egui is immediate-mode, so the natural shape here is
closer to `Split`'s builder pattern than to a declarative routing DSL.

Two public pieces:

- **`NavStack<K>`** — a thin wrapper around `Vec<K>`, generic over an
  app-defined key type `K` (typically an enum of screens + their
  params). `push(&mut self, key: K)`, `pop(&mut self) -> Option<K>`,
  `pop_to(&mut self, predicate)`, `replace_top(&mut self, key: K)`,
  `top(&self) -> Option<&K>`. This crate has zero knowledge of what `K`
  actually is — same app-agnostic stance as every other crate here.
- **`NavDisplay<K>`** — `NavDisplay::new(&nav_stack).show(ui, |key, ui|
  { /* app matches on key and draws that screen */ })`. Renders only
  the top-of-stack entry by default; returns any navigation request the
  app's closure produced (a push/pop) so the app applies it to its own
  `&mut NavStack` after `.show()` returns, rather than the closure
  mutating the stack it's currently being read from.

**Optional, feature-gated:** an `animated-transitions` feature adds
spring-driven push/pop transitions by depending on `egui-spring` (and
transitively `spring-core`). Off by default, so an app that just wants
the plain stack + display isn't forced to pull in the spring crates.

**Explicitly out of scope for this crate** (consistent with PRD §3's
non-goals): no routing/URL parsing, no deep-linking, no
serialization/persistence of the stack — if `K` is serializable, saving
and restoring `Vec<K>` across app launches is the consuming app's job,
same as Navigation 3 leaves state-saving to `rememberSaveable` on the
app side. Multi-pane display (Navigation 3's "Scenes," e.g. list-detail
side-by-side) is a plausible future addition but not in the first pass —
default to single-pane, top-of-stack-only rendering.

---

## 6. Design principles

- **Small, focused crates over one monolith.** Each crate does one thing;
  an app depends only on what it needs.
- **No global or static mutable state, anywhere.** Every widget's state
  (a `Spring`, a `Navigator`, a `NavStack`, a `ThemeSwitcher`) is an explicit value the
  consuming app owns and passes in — today that's `src/test-app/`, but
  the rule doesn't special-case it just because it's co-located. This is
  what lets multiple instances coexist and keeps egui-widgetkit trivially
  testable.
- **Consistent builder API shape across crates.** Where a widget has
  configuration, prefer the `Foo::new() -> Self` + chained setters +
  `.show(self, ui: &mut Ui) -> Response` shape already established by
  `Split` — a user of one crate should recognize the pattern in another.
- **`-core` crates stay `egui`-free.** Any crate whose name ends in
  `-core` (currently just `spring-core`) must have zero `egui` dependency,
  so its logic is testable and reusable independent of egui entirely.
- **Backend-agnostic, always.** No crate in egui-widgetkit may depend on
  `wgpu`, `glow`, or any backend-specific crate. Every widget draws via
  `egui::Shape`; the backend is exclusively the consuming app's choice.

---

## 7. Rendering & animation policy

Confirmed decision (carried over from earlier discussion, restated here
since it's egui-widgetkit's own policy, not an app's): **no custom GPU
code in this library.** Smoothness comes from two things only:

1. Pure CPU math computing the current animated value each frame
   (`spring-core`'s closed-form solver — cheap enough that "GPU cost"
   isn't a meaningful concept for it).
2. `ctx.request_repaint()` (or `request_repaint_after`) called by the
   widget every frame the animation hasn't settled, so egui keeps
   redrawing between input events. This is the entire mechanism — nothing
   else is needed, and no `PaintCallback`/`CallbackTrait` custom render
   path belongs in egui-widgetkit.

---

## 8. egui version compatibility

- **Pinned `egui = "0.27"`** across every `egui`-consuming crate in this
  workspace — egui-widgetkit and `src/test-app/` alike — this is the
  version that compiled cleanly against the Rust 1.75 toolchain
  available when `egui-layout` was built, and `child_ui`'s signature has
  changed across `egui` versions (0.27 takes 2 args, not 3).
- The pin is a **published compatibility contract** for any future app
  depending on `src/egui-widgetkit/`, not an internal build detail —
  bumping it is a breaking-change-adjacent decision and should be called
  out in each crate's changelog/README, with `cargo check --workspace`
  re-run before merging a bump.

---

## 9. Testing & examples strategy

- **Math crates (`spring-core`):** plain `#[test]` functions asserting
  exact numeric output (closed-form solution vs. expected curve, settling
  tolerance) — no `egui`, no mocking.
- **Widget crates (`egui-layout`, `egui-spring`, `egui-vim-nav`,
  `egui-nav-stack`, `egui-themes`):** egui rendering isn't practically unit-testable, so
  each widget crate ships an `examples/` directory with a small `eframe`
  demo app. This is both the manual QA surface and living documentation.
  (`NavStack<K>`'s push/pop/replace logic is plain data manipulation
  though, so give it ordinary `#[test]`s too, same as a `-core` crate,
  in addition to `egui-nav-stack`'s `examples/` demo for `NavDisplay`.)
- Since `src/test-app/` is, today, the *only* real consumer, its
  `combined_demo` scene (FOLDER_STRUCTURE.md §4/§7) already does the job
  of "prove the crates compose correctly together" — there's no separate
  proof app needed outside this repo right now.

---

## 10. Known issues / open items

| Issue | Where | Action |
|---|---|---|
| `Split::horizontal()`/`vertical()` naming ambiguity | `egui-layout` | Decide: rename to `row()`/`column()`, or document the convention explicitly (§5.1) |
| `egui = "0.27"` pin tied to a specific Rust 1.75 environment | `egui-layout`, and every future `egui`-consuming crate | Track whether this constraint is permanent; if temporary, roadmap removing it |
| No crate currently documents its "no global state" contract in doc comments | all | Add a top-of-`lib.rs` doc comment per crate stating this explicitly, so a future external consumer doesn't assume a singleton pattern |

---

## 11. Success criteria

- `cargo check --workspace` passes with all 6 egui-widgetkit crates plus
  `src/test-app` present.
- `cargo tree -p spring-core` shows no `egui` dependency.
- `src/test-app` (or any future consuming app) can add a single crate
  (e.g. just `egui-layout`) as a path dependency without pulling in the
  other five.
- `egui-nav-stack` with the default feature set (no `animated-transitions`)
  pulls in `egui` only — `cargo tree -p egui-nav-stack` shows no
  `egui-spring`/`spring-core` unless that feature is explicitly enabled.
- Spring-driven widgets hold a consistent frame rate using only
  `request_repaint()` — no custom render path anywhere in egui-widgetkit.
- Each widget crate has at least one runnable `examples/` demo.

---

## 12. Roadmap

See `BUILD_PLAN.md` for this roadmap broken into concrete parts and
subparts, with what to keep in mind and what errors to avoid for each
one. The steps below are the summary; that doc is the detail.

1. `spring-core`: closed-form solver + `SpringParams` presets + unit tests.
2. `egui-spring`: `SpringRect`, corner-spring bundle, Bézier border
   geometry, `request_repaint` wiring, one `examples/` demo.
3. `egui-vim-nav`: focus graph, navigator, key handler, registration API,
   one `examples/` demo.
4. `egui-nav-stack`: `NavStack<K>`, `NavDisplay`, one `examples/` demo;
   `animated-transitions` feature (depends on `egui-spring`) as a
   later, optional pass.
5. `egui-themes`: token type, curated palettes, switcher, live preview
   panel, one `examples/` demo.
6. Resolve the `Split` naming question (§10) before any app outside this
   repo takes a dependency on `egui-layout`.
