# Folder Structure & File Organization — egui-widgetkit

Companion to `ARCHITECTURE_PRD.md` (the *why*) and `CODING_RULES.md` (the
rules). This is the *where* — for any piece of code, which file it goes
in.

**Correction alongside `ARCHITECTURE_PRD.md` v5:** the library folder is
now named `src/egui-widgetkit/` instead of the generic `src/library/`.
Everything else about the layout below is unchanged from before — this
is a rename, not a restructure.

---

## 1. Philosophy

- **Prefer many small files over few large ones**: A file earns its existence by
  having one reason to change, not by hitting a minimum line count.
- **Flexible, High-Quality Architecture**: We will use the folder structure as an
  architectural guideline rather than a rigid limit — introducing helper modules,
  cleaner abstractions, and dedicated math/utility files wherever it makes the
  codebase more modular and readable.

---

## 2. Module convention

No `mod.rs` files anywhere. Every module with children is a `foo.rs` file
sitting next to a `foo/` directory, where `foo.rs` contains only `pub mod
child;` declarations and `pub use child::PrimaryType;` re-exports — never
actual logic.

---

## 3. `src/egui-widgetkit/` vs. `src/test-app/`

The repo root's `src/` holds exactly two things, kept deliberately
separate:

- **`src/egui-widgetkit/`** — the actual reusable crates, i.e. the
  library itself. This is the only thing that could ever be
  path-depended on by a *different* app, if one shows up later (§9).
  Nothing in here knows it's being consumed by `test-app`; nothing in
  here is allowed to reference `test-app`.
- **`src/test-app/`** — your actual, current app, a real small `eframe`
  application living inside this repo. It's simple today, and that
  simplicity is exactly why it can also serve double duty as the
  integration test bed: it's the one place proving the
  `src/egui-widgetkit/` crates actually work together, not just in
  isolation, and it catches API awkwardness before anything outside this
  repo ever depends on them. If it grows into something bigger later,
  none of this changes.

`test-app` depends on `egui-widgetkit/*`; `egui-widgetkit/*` never
depends on `test-app` — that direction never reverses, no matter how
much `test-app` grows or how tempting it is to reach across since
they're in the same repo. This is the one rule that keeps the library
actually reusable instead of secretly coupled to this one app.

---

## 4. Full workspace layout

```
your-repo/                                      # single repo — egui-widgetkit AND your current app
├── Cargo.toml                                  # [workspace] members = [...]
├── README.md
│
└── src/
    ├── egui-widgetkit/                         # path-dependency surface — the library
    │   │
    │   ├── egui-layout/                        # ✅ implemented
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src/
    │   │   │   ├── lib.rs                      # pub mod split; pub use split::Split;
    │   │   │   └── split.rs
    │   │   ├── tests/
    │   │   │   ├── split_normalizes_fractions.rs
    │   │   │   └── split_nested_sections.rs
    │   │   └── examples/
    │   │       └── nested_layout.rs             # quick single-crate sanity check (see §7)
    │   │
    │   ├── spring-core/                         # stub — NO egui dependency, pure math
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src/
    │   │   │   ├── lib.rs
    │   │   │   ├── spring.rs                    # Spring struct: value/velocity/target + API
    │   │   │   ├── solver.rs                    # closed-form step functions
    │   │   │   └── params.rs                    # SpringParams value type + presets
    │   │   └── tests/
    │   │       ├── solver_matches_closed_form.rs
    │   │       └── settles_within_tolerance.rs
    │   │
    │   ├── egui-spring/                         # stub
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src/
    │   │   │   ├── lib.rs
    │   │   │   ├── spring_rect.rs                # SpringRect widget: Shape emission, request_repaint
    │   │   │   ├── corner_springs.rs             # 4-corner spring bundle
    │   │   │   └── bezier.rs                     # Bézier-rounded border geometry
    │   │   └── examples/
    │   │       └── selection_highlight.rs
    │   │
    │   ├── egui-vim-nav/                        # stub
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src/
    │   │   │   ├── lib.rs
    │   │   │   ├── focus_graph.rs                # navigable-element graph
    │   │   │   ├── navigator.rs                  # cursor + move(direction)
    │   │   │   ├── key_handler.rs                # HJKL/arrow key mapping
    │   │   │   └── register.rs                   # API for a consumer to register focusable regions
    │   │   └── examples/
    │   │       └── grid_navigation.rs
    │   │
    │   └── egui-themes/                          # stub
    │       ├── Cargo.toml
    │       ├── README.md
    │       ├── src/
    │       │   ├── lib.rs
    │       │   ├── token.rs                      # ThemeToken: named color/spacing/radius slots
    │       │   ├── palette.rs                    # curated built-in palettes
    │       │   ├── switcher.rs                   # current palette + apply-to-ctx
    │       │   └── preview.rs                    # live preview panel widget
    │       └── examples/
    │           └── theme_switcher.rs
    │
    └── test-app/                                 # your current app — built ON egui-widgetkit, and its integration test bed
        ├── Cargo.toml                            # path-depends on every crate it needs under src/egui-widgetkit/*
        └── src/
            ├── main.rs                           # ~10 lines — see §6
            ├── app.rs                            # re-exports only
            ├── app/
            │   ├── state.rs                      # which demo scene is active + any shared state
            │   └── update.rs                     # impl eframe::App — routes to the active scene
            ├── scenes.rs                         # re-exports only
            └── scenes/
                ├── layout_demo.rs                 # exercises egui-layout alone
                ├── spring_demo.rs                  # exercises spring-core + egui-spring alone
                ├── vim_nav_demo.rs                  # exercises egui-vim-nav alone
                ├── theme_demo.rs                     # exercises egui-themes alone
                └── combined_demo.rs                   # several crates together — the "looks like a real app" screen
```

("your-repo" is a placeholder — name the repo after your actual app if
you like; nothing about egui-widgetkit's crate names or paths depends on
what the outer repo is called.)

---

## 5. `test-app`'s `Cargo.toml`

```toml
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
egui-layout  = { path = "../egui-widgetkit/egui-layout" }
spring-core  = { path = "../egui-widgetkit/spring-core" }
egui-spring  = { path = "../egui-widgetkit/egui-spring" }
egui-vim-nav = { path = "../egui-widgetkit/egui-vim-nav" }
egui-themes  = { path = "../egui-widgetkit/egui-themes" }
eframe       = "0.27"
```

Workspace root:

```toml
# Cargo.toml (repo root)
[workspace]
members = [
  "src/egui-widgetkit/egui-layout",
  "src/egui-widgetkit/spring-core",
  "src/egui-widgetkit/egui-spring",
  "src/egui-widgetkit/egui-vim-nav",
  "src/egui-widgetkit/egui-themes",
  "src/test-app",
]
```

---

## 6. `test-app/src/main.rs` — the target

Same "bootstrap script, not a program" rule as any app's `main.rs` —
this crate is, after all, your app.

```rust
mod app;
mod scenes;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "test-app",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app::state::TestAppState::default())),
    )
}
```

`app/state.rs` owns which scene is showing (`enum ActiveScene { Layout,
Spring, VimNav, Theme, Combined }` + whatever per-scene state each demo
needs); `app/update.rs` is the `impl eframe::App` that reads that state
and calls into the matching function in `scenes/`. Each file in
`scenes/` is a plain `fn show(ui: &mut egui::Ui, ...)`-shaped function —
no framework beyond that.

---

## 7. Why both `examples/` (per crate) and `test-app` exist

They serve different purposes and both are worth keeping, even though
`test-app` is your real app and not just a throwaway harness:

- **`examples/` per crate** — fastest possible sanity check for one crate
  in isolation (`cargo run -p egui-spring --example selection_highlight`),
  useful even if you're only touching that one crate and don't want to
  boot the whole app.
- **`test-app`** — the only place that proves the egui-widgetkit crates
  compose correctly together (shared theme applied across a layout with
  spring-driven selection and vim navigation all active at once), which
  no single crate's own `examples/` can demonstrate on its own — and,
  since it's your real app, that proof matters for real, not just as QA.

If this ends up feeling redundant in practice, the per-crate `examples/`
are the ones to drop first — `test-app`'s `combined_demo.rs` scene is the
one that actually matters for catching integration issues.

---

## 8. Recipes: adding new code

| Task | Files touched, in order |
|---|---|
| **New widget inside an existing egui-widgetkit crate** | New file under that crate's `src/`, one type per file → re-export in that crate's `lib.rs` → update or add an `examples/` demo |
| **New crate (a 6th capability)** | New directory under `src/egui-widgetkit/` → own `Cargo.toml`, `README.md`, `src/lib.rs` → add to the workspace `Cargo.toml`'s `members` → add it as a path dependency in `test-app/Cargo.toml` → at least one scene in `test-app/src/scenes/` exercising it |
| **New test-app scene** | New file under `test-app/src/scenes/` → re-export in `scenes.rs` → add the matching `ActiveScene` variant in `app/state.rs` and route to it in `app/update.rs` |
| **New preset/variant inside `spring-core`** | `spring-core/src/params.rs` → new test in `spring-core/tests/` |

---

## 9. Future: if a second, separate app wants egui-widgetkit

Not needed today — right now `src/test-app/` is the only consumer, and
it lives in this same repo. But if a genuinely separate app, in a
different repo, ever wants `src/egui-widgetkit/`, that app would depend
on it like this:

```toml
# some-other-app/Cargo.toml — a different repo entirely
[dependencies]
egui-layout = { path = "../your-repo/src/egui-widgetkit/egui-layout" }
egui-spring = { path = "../your-repo/src/egui-widgetkit/egui-spring" }
```

Only crates under `src/egui-widgetkit/` are ever meant to be depended on
this way. `src/test-app/` is never a dependency target for anyone, in
this repo or any other — it only exists to be your app and to exercise
`src/egui-widgetkit/` from inside this repo. See ARCHITECTURE_PRD.md §4
for when this extraction is actually worth doing — and worth
double-checking the `egui-widgetkit` name is still free if you ever
publish it to crates.io, since name availability can change over time.

---

## 10. When it's OK to combine things in one file

Rare, and only when splitting would separate two things that cannot be
understood or changed independently — e.g. a struct and its only inherent
`impl` block. A genuinely separate type or concern still gets its own
file, regardless of size, including inside `test-app`.
