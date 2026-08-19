# Coding Rules — egui-widgetkit

Companion to `ARCHITECTURE_PRD.md`. This is the enforceable checklist for
writing code inside `src/egui-widgetkit/`. Everything here applies to a
**standalone, app-agnostic UI library** — there is no "domain layer," no
"daemon," no app-specific business logic anywhere in `src/egui-widgetkit/`.
If a rule here ever implies knowledge of a specific consuming app, that
rule is wrong and should be removed.

---

## 1. The one rule everything else follows

**No crate under `src/egui-widgetkit/` may import, reference, or assume
the existence of any type from a specific consuming application — and
that includes `src/test-app/` itself,** even though it lives in this
same repo and it'd be easy to reach across. Every crate must compile and
make sense dropped into a completely unrelated egui app. If you're
tempted to add a special case for "the way the current app does X,"
stop — either generalize it into a parameter/trait the consuming app
supplies, or it doesn't belong in egui-widgetkit at all.

| Crate | May depend on | Must NOT depend on |
|---|---|---|
| `egui-layout` | `egui` | any other crate in this workspace, any app type (`test-app` included) |
| `spring-core` | *(nothing but small general-purpose crates)* | `egui`, any other crate in this workspace, any app type |
| `egui-spring` | `spring-core`, `egui` | any app type |
| `egui-vim-nav` | `egui` | any other crate in this workspace, any app type |
| `egui-nav-stack` | `egui`; optionally `egui-spring` (behind `animated-transitions` feature, off by default) | `egui-layout`, `egui-vim-nav`, `egui-themes`, any app type |
| `egui-themes` | `egui` | any other crate in this workspace, any app type |

`egui-nav-stack` is the one exception to "each crate depends on `egui`
only" — its optional feature depends on `egui-spring` the same way
`egui-spring` depends on `spring-core`: opt-in, not a hard requirement
for the base functionality.

---

## 2. State ownership rule

**No global or static mutable state, anywhere in `src/egui-widgetkit/`.**
Every type that holds animated/interactive state (`Spring`, `Navigator`,
`NavStack`, `ThemeSwitcher`, focus graphs) is a plain value the *consuming app*
owns — typically as a field on `test-app`'s own state struct — and
passes in by `&mut` reference each frame.

Why this is a hard rule, not a style preference: a `static` or
thread-local spring state means only one `SpringRect` can exist per
process, which defeats the entire point of a reusable widget. If a widget
needs "state," its constructor returns a value the caller stores; it never
reaches for `lazy_static`, `OnceCell` holding mutable data, or similar.

---

## 3. Public API conventions

- **Builder shape**, consistent across crates: `Type::new()` (or a
  named constructor like `Split::horizontal()`) → chained setter methods
  consuming and returning `Self` → a terminal method that actually does
  the work (`.show(self, ui: &mut Ui) -> Response` for a widget,
  `.update(&mut self, dt: f32)` for a non-drawing state type like
  `Spring`). Don't invent a new configuration shape per crate.
- **Fractions/ratios don't need to sum to 1.0** where that pattern applies
  (established by `Split`) — normalize internally rather than requiring
  the caller to do math.
- **Every public type gets a doc comment stating its state-ownership
  contract** — e.g. "the caller owns and persists this across frames;
  there is no hidden global instance."
- **No `unwrap()`/`panic!()` on caller-supplied input** in public API
  surfaces. Return `Option`/`Result` for anything that can fail based on
  what the caller passed in (e.g. malformed fractions, an unregistered
  focus-graph node).
- **A widget that needs to mutate data it's also reading from returns a
  request, it doesn't mutate directly.** E.g. `NavDisplay::show()` reads
  `&NavStack` to render the current screen; if the app's own closure
  decides to navigate, `.show()` returns that as a value (a push/pop
  request) for the app to apply to its own `&mut NavStack` afterward —
  it doesn't hand the closure a live `&mut` into the same stack being
  read. Avoids fighting the borrow checker and keeps state mutation in
  exactly one place: the caller.

---

## 4. Animation & rendering rules

- Widgets emit **`egui::Shape`s only** (`ui.painter().add(...)`,
  `Shape::rect_filled`, `PathShape`, etc.). Never touch `wgpu` or `glow`
  directly — the backend is the consuming app's choice, made in its own
  `eframe::NativeOptions`, and egui-widgetkit must work under either.
- **Any widget with continuous motion must call `ctx.request_repaint()`
  (or `request_repaint_after`) every frame it hasn't settled.** This is
  the entire smoothness mechanism in immediate-mode egui — there is no
  other. A `SpringRect`-style widget that doesn't call this will visibly
  step instead of glide, and that's the bug to look for first if an
  animation feels choppy.
- **No `PaintCallback`/`CallbackTrait` custom render path** in
  egui-widgetkit (PRD §7, non-goal). If a future widget genuinely can't
  be expressed as `egui::Shape`s, that's a new proposal, reviewed on its
  own — not something to reach for by default.

---

## 5. `-core` crate rules

Any crate named `<capability>-core` (currently `spring-core`) must:

- Have **zero dependency on `egui`.**
- Contain pure logic/math only — structs, functions, no widget code.
- Be independently unit-testable with plain `#[test]`, asserting exact
  numeric output where the underlying math has a known closed form (e.g.
  compare the analytical solver against a numerically-integrated reference
  to confirm they converge, and assert settling behavior within a stated
  tolerance).

The paired `egui-<capability>` crate (`egui-spring`) is the *only* place
that wraps the `-core` crate's types in `egui::Shape` output.

---

## 6. Version pinning rules

- Any `egui` version pin in a crate's `Cargo.toml` must have a comment
  stating **why** (e.g. "0.27 — child_ui signature differs across
  versions; re-run cargo check across the workspace after bumping").
- All `egui`-consuming crates in this workspace — egui-widgetkit and
  `test-app` alike — must pin to the **same** `egui` version; a mismatch
  between, say, `egui-layout` and `egui-spring` will not compile in an
  app that uses both.
- Bumping the pin is a workspace-wide change: update every crate's
  `Cargo.toml` together, then `cargo check --workspace`, in one commit.

---

## 7. Testing rules by crate type

| Crate type | Test style | What's disallowed |
|---|---|---|
| `-core` (pure math) | Plain `#[test]`, exact expected values | `egui` in the test, mocking frameworks |
| `egui-*` (widgets) | `examples/` demo app for manual/visual verification | Assuming CI can verify rendered pixel output |

Every `-core` crate must have tests. Every `egui-*` crate must have at
least one runnable example under `examples/` — that's the equivalent
requirement, since widget correctness here means "looks and feels right,"
not "assertion passes."

---

## 8. Naming conventions

- `-core` suffix: pure logic, zero `egui` dependency.
- `egui-` prefix: the paired crate that turns a `-core` crate's output
  (or standalone logic, for crates with no `-core` half) into `egui`
  widgets.
- Public widget entry points follow `Type::new()`/named constructor +
  `.show(ui) -> Response`, matching `Split` (§3).
- No crate, type, or function name may reference `test-app` or any other
  specific consuming app, even in a comment example — use a generic name
  ("Sidebar", "Content", "Item") in doc examples instead.

---

## 9. Anti-patterns — reject these in review

- Any import of an app-specific type, anywhere in `src/egui-widgetkit/`
  (`test-app` types included).
- A `static`/`lazy_static`/global `OnceCell` holding mutable widget state.
- `egui` as a dependency of any `-core` crate.
- `wgpu` or `glow` imported directly in any crate.
- A widget that animates but never calls `request_repaint()`.
- A new configuration API that doesn't follow the established builder +
  `.show()` shape "just this once."
- An undocumented `egui` version pin, or a pin that differs between two
  crates in this workspace (or between egui-widgetkit and `test-app`).
- A widget crate merged with no corresponding `examples/` demo.

---

## 10. Definition of done (per PR)

- [ ] `cargo check --workspace` passes.
- [ ] `cargo tree -p spring-core` (or any future `-core` crate) shows no
      `egui` dependency.
- [ ] No new type/function under `src/egui-widgetkit/` references
      anything app-specific, `test-app` included — grep the diff for
      anything that looks like a `test-app`-specific name before
      merging.
- [ ] Any new animated widget calls `request_repaint()` while unsettled.
- [ ] Any new widget crate has at least one `examples/` demo.
- [ ] Any changed `egui` version pin is applied workspace-wide
      (egui-widgetkit and `test-app` together) and documented with a
      reason.
