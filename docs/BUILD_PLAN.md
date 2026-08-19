# Build Plan — egui-widgetkit

Companion to `ARCHITECTURE_PRD.md` (the *why*), `FOLDER_STRUCTURE.md`
(the *where*), and `CODING_RULES.md` (the rules). This is the *how, in
what order* — egui-widgetkit broken into parts and subparts small enough
to build and verify one at a time, each with what to keep in mind while
building it and the specific errors that tend to show up there.

Nothing here changes any decision already made in the other three docs.
Where a subpart's guidance is really just "apply CODING_RULES §X," this
doc says so instead of repeating the rule.

---

## How to use this doc

Each Part is one crate (or, for Part 6, `test-app`). Each Subpart is one
file or one tight cluster of files, small enough to build, test, and
move on. Work through a Part's subparts in order — they're ordered by
dependency within the crate. Parts themselves don't all have to happen
in order; see §"Suggested build order" at the end for which ones can run
in parallel.

For every subpart: build it, satisfy its own "keep in mind" points,
check it against its "errors to avoid," *then* move on. Don't build two
subparts and debug them together — immediate-mode UI bugs and spring-math
bugs look similar from the outside (both show up as "the animation looks
wrong") and are much easier to isolate one layer at a time.

---

## Part 0 — Workspace scaffolding

**What to build:** repo root `Cargo.toml` ([workspace] table),
`README.md`, the `src/egui-widgetkit/` and `src/test-app/` directory
skeletons from `FOLDER_STRUCTURE.md` §4, git init, `.gitignore`.

**Keep in mind:**
- Consider a `[workspace.dependencies]` table at the root pinning
  `egui = "0.27"` once, with every crate inheriting it via
  `egui = { workspace = true }`. That's the mechanical way to actually
  guarantee CODING_RULES §6 ("all `egui`-consuming crates pin the same
  version") instead of relying on remembering to update five files by
  hand every time the pin changes.
- Decide the Rust edition (2021, per `FOLDER_STRUCTURE.md` §5) once and
  use it in every crate's `Cargo.toml` from the start.

**Errors to avoid:**
- A typo in a workspace `members` path (e.g. `src/egui-widgetkit/egui-layout`
  written as `src/egui-widgetkit/egui_layout`) doesn't error loudly — it
  just silently excludes that crate from `cargo check --workspace`,
  which then reports false-positive success.
- Don't put a `[dependencies]` table on the *root* `Cargo.toml` itself
  unless you actually want the root to be a package, not just a virtual
  workspace — mixing the two confuses `cargo build`'s default target.
- Commit `Cargo.lock` — this workspace has a binary (`test-app`), and an
  uncommitted lockfile means "works on my machine" version drift.

---

## Part 1 — `egui-layout` (harden what's already built)

This crate is implemented (PRD §5.1), so this Part is a verification
pass, not new construction — do this before building anything that
might come to depend on its shape.

**Keep in mind:**
- Resolve the `Split::horizontal()`/`vertical()` naming question (PRD
  §5.1, §10) now, while only `test-app`'s `layout_demo.rs` depends on
  it. A rename after `egui-vim-nav` or anything else starts assuming a
  particular axis convention is a breaking change with a wider blast
  radius.

**Errors to avoid (check for these specifically):**
- **Fraction normalization edge cases:** all-zero fractions, a single
  section, or a negative fraction. Per CODING_RULES §3 these should
  degrade sensibly (e.g. treat as 0, or clamp) — not panic and not
  silently divide by zero.
- **Spacing subtracted at the wrong point:** `spacing()` must come out
  of the total available space *before* dividing by fractions, not
  after — otherwise `n` sections overflow the container by
  `n * spacing` in total.
- **Accumulated float rounding:** normalizing fractions by dividing by
  their sum can leave a few leftover pixels unassigned at the container
  edge on some fraction combinations — worth an explicit test with an
  odd fraction set (e.g. `0.33 / 0.33 / 0.34`) rather than only round
  numbers.
- **Corner rounding preservation:** ensure the allocated section rectangle
  and any outer card shapes preserve all 4 rounded corners without being
  sliced by parent clipping boundaries when windows shrink.

---

## Part 2 — `spring-core`

Pure math, zero `egui` dependency (CODING_RULES §5). Finish this Part
completely — including tests — before starting Part 3; `egui-spring` is
only a thin wrapper and any bug here will otherwise get misdiagnosed as
a widget/rendering bug later.

### 2.1 `params.rs` — `SpringParams` + presets

**What to build:** a value type carrying the physical parameters
(stiffness/damping or damping-ratio/frequency — pick one parameterization
and stick to it) plus a few named presets (e.g. "gentle," "snappy,"
"bouncy").

**Keep in mind:**
- Derive presets from *natural* parameters (damping ratio + angular
  frequency), not raw magic numbers, so a preset behaves predictably
  regardless of what mass/target-distance combination it's applied to.

**Errors to avoid:**
- Accepting a negative damping ratio without rejecting or clamping it —
  physically meaningless, and left unchecked it produces a solver that
  diverges (grows without bound) instead of settling. Return
  `Option`/`Result` or clamp, per CODING_RULES §3 — don't let it reach
  the solver as garbage input.

### 2.2 `solver.rs` — closed-form step functions

**What to build:** the actual closed-form solutions to the damped
harmonic oscillator for all three regimes — underdamped, critically
damped, overdamped — since PRD §5.2/§7 explicitly rules out
Euler-integration here.

**Keep in mind:**
- These are three genuinely different formulas (the oscillator's
  characteristic equation has complex roots, a repeated real root, or
  two distinct real roots respectively) — there's no way to write one
  formula that quietly covers all three regimes.
- Use whatever time unit egui reports frame delta in (seconds, via
  `ctx.input(|i| i.stable_dt)`) consistently — decide this once and
  document it on the function signature.

**Errors to avoid:**
- **The damping-ratio-≈-1 boundary is numerically unstable.** Both the
  underdamped and overdamped closed forms involve a term that approaches
  zero right at critical damping, so evaluating either formula near that
  boundary (rather than switching to the critical-damping formula) can
  blow up or produce NaN. Snap to the critical-damping branch within a
  small epsilon band around ratio = 1 — don't rely on exact float
  equality to pick a branch.
- A sign error in a decay exponent is the classic bug here, and it's
  silent: instead of an error, the spring visibly diverges (grows) on
  screen instead of settling, which is easy to misattribute to
  `egui-spring`'s rendering code instead of the math.

### 2.3 `spring.rs` — the `Spring` struct

**What to build:** the public state type (`value`, `velocity`, `target`
+ `SpringParams`) with `.update(&mut self, dt: f32)` and
`.is_settled(&self) -> bool`.

**Keep in mind:**
- `is_settled()` must check **both** position-near-target **and**
  velocity-near-zero. A spring can pass through its target at speed and
  still be visibly moving.

**Errors to avoid:**
- Checking only position for settling is the single most consequential
  bug in this file — it's invisible in `spring-core`'s own tests
  (position-only assertions can pass fine) but surfaces two layers up:
  `egui-spring` stops calling `request_repaint()` the moment
  `is_settled()` lies, and the widget visibly freezes mid-motion.
- Leaving the settle-tolerance values undocumented — `egui-spring`'s
  visual judgment of "done" and this crate's numeric one need to agree,
  and that only works if the tolerance is a documented, deliberate
  constant, not an arbitrary number picked once and forgotten.

### 2.4 Tests

**What to build:** per CODING_RULES §7 — exact closed-form value
assertions at specific `t`, plus settling-within-tolerance assertions.

**Errors to avoid:**
- Only testing the underdamped case. It's the "nice," intuitively
  obvious oscillating one, which is exactly why the critically-damped
  and overdamped branches — touched less often, more likely to hide a
  sign or branch-selection bug — need their own explicit test cases too.

---

## Part 3 — `egui-spring`

Depends on Part 2 being solid. This is the *only* place that turns
`spring-core` output into `egui::Shape`s (CODING_RULES §5).

### 3.1 `corner_springs.rs` — four independent springs

**What to build:** a bundle of four `Spring` instances (from
`spring-core`), one per rect corner, per PRD §5.3.

**Keep in mind:**
- The four springs must be genuinely independent, each with its own
  velocity/target — that independence is what produces the "elastic
  smear" effect. A single shared spring driving all four corners
  uniformly just reproduces a slower rectangle, not the intended effect.

**Errors to avoid:**
- Forgetting to re-target all four springs when the selection moves to a
  new rect. This bug is unusually hard to notice: a spring whose target
  wasn't updated is technically "settled" (it's sitting still at the old
  target), so it won't trigger `request_repaint()` and won't throw an
  error — it just looks like the widget silently stopped responding to
  selection changes.

### 3.2 `bezier.rs` — Bézier-rounded border geometry

**What to build:** border geometry through the 4 (now independently
moving, so not perfectly rectangular) corner points.

**Keep in mind:**
- Because corners move independently, control points need to be derived
  per-corner from that corner's own trajectory/velocity, not from an
  idealized rectangle shape.

**Errors to avoid:**
- Naive per-corner interpolation can self-intersect or go non-convex at
  extreme spring velocity/deformation. Clamp the maximum smear offset so
  the shape stays sane even when a selection jumps a long distance in
  one frame.

### 3.3 `spring_rect.rs` — the public `SpringRect` widget

**What to build:** the builder-shaped widget wiring `corner_springs` +
`bezier` into `egui::Shape` output: `SpringRect::new(target_rect)` →
setters (e.g. `.params(...)`) → `.show(&mut self, ui: &mut Ui) ->
Response`. Takes `&mut self` because the widget owns and must mutate its
own spring state across frames (its state lives on `test-app`'s side per
CODING_RULES §2 — this method just mutates what's handed in).

**Errors to avoid:**
- **This is the bug CODING_RULES §4 explicitly calls out as the first
  thing to check:** forgetting `ctx.request_repaint()` while unsettled.
  Without it, egui only redraws on input events, so the animation
  visibly *steps* between mouse moves/clicks instead of *gliding*.
- The opposite bug is just as real: calling `request_repaint()`
  unconditionally, even once fully settled, burns CPU/GPU in an infinite
  redraw loop. Gate it strictly on `!spring.is_settled()`.

### 3.4 Example (`selection_highlight.rs`)

**Keep in mind:** run this demo specifically to sanity-check the
settle-tolerance tuning from Part 2.4 — if the demo keeps requesting
repaints forever even after the highlight visibly stops moving on
screen, that's a settle-tolerance bug in `spring-core`, not an egui or
rendering bug, and it's much faster to catch here than to debug inside
`test-app` later.

---

## Part 4 — `egui-vim-nav`

Independent of Parts 2–3; can be built in parallel with them.

### 4.1 `focus_graph.rs` — the navigable-element graph

**What to build:** a generic node-id + directional-neighbor structure
(e.g. `HashMap<NodeId, DirectionalNeighbors>`) — per PRD §5.4, the
*shape* of the graph (grid, tree, whatever) is entirely defined by the
consuming app, not this crate.

**Errors to avoid:**
- Silently assuming a topology that matches `egui-layout`'s `Split`
  (e.g. treating panes as 1:1 with nodes). That would violate "each
  crate can be pulled in alone" (PRD §2) and CODING_RULES §1's
  dependency table, which does not list `egui-layout` as something
  `egui-vim-nav` may depend on.

### 4.2 `navigator.rs` — cursor + `move(direction)`

**Keep in mind:** decide up front what happens at a graph edge (no
neighbor in that direction) and when the focused node no longer exists
(e.g. the app tore down that region while it was focused).

**Errors to avoid:**
- Panicking on an unregistered/removed node, or `unwrap()`-ing a missing
  neighbor lookup. Per CODING_RULES §3 this must degrade gracefully —
  clamp, no-op, or clear focus — never crash the whole consuming app
  over a UI-navigation edge case.

### 4.3 `key_handler.rs` — HJKL/arrow mapping

**Keep in mind:** this crate cannot unconditionally claim every H/J/K/L
keypress — the consuming app will have its own widgets (text fields,
search boxes) that need those same letters for normal typing.

**Errors to avoid:**
- Capturing HJKL globally regardless of what else has focus. In
  practice this means checking something like
  `ctx.wants_keyboard_input()` (or an equivalent "is a text-input widget
  currently focused" check) before intercepting — otherwise wiring this
  crate in silently breaks typing anywhere else in the app.

### 4.4 `register.rs` — registration API

**Keep in mind:** immediate-mode UIs typically re-declare their widget
tree every frame — decide explicitly whether regions register once
per frame (typical for immediate mode) or once outside the frame loop,
and document that choice as the state-ownership contract (CODING_RULES
§3). Key regions by a stable id (an `egui::Id` or caller-supplied id),
not a freshly generated handle each frame, or focus will silently reset
constantly.

**Errors to avoid:** picking a registration pattern that's inconsistent
with immediate mode (e.g. an API that assumes "register once, keep
forever" in a codebase that re-declares UI every frame) — this is a
design decision, not a bug you'll catch with a test, so get it right
before `test-app`'s scenes start depending on the shape of this API.

### 4.5 Example (`grid_navigation.rs`)

---

## Part 5 — `egui-themes`

Independent of Parts 2–4; can be built in parallel with them.

### 5.1 `token.rs` — `ThemeToken`

**What to build:** named color/spacing/radius slots.

**Keep in mind:** per PRD §5.5, a consuming app must be able to define
its own tokens too — so this needs to be an open/extensible design
(a trait, or a string-keyed map with typed accessors), not a closed enum
that only this crate's own code can add variants to.

**Errors to avoid:** baking specific palette *values* into the token
*definition* type itself. Keep "what slots exist" (tokens) and "what
fills them" (palettes, §5.2) as two separate concerns — conflating them
is what makes §5.2/§5.3 awkward to build afterward.

### 5.2 `palette.rs` — curated built-in palettes

**Keep in mind:** a palette is just data — a token→color mapping.

**Errors to avoid:** defining built-in palettes as
`static PALETTE: Lazy<Palette> = ...` (via `once_cell`/`lazy_static`).
Even read-only, this sits awkwardly next to CODING_RULES §2's "no global
state anywhere" and CODING_RULES §9's anti-pattern list, which calls out
exactly this pattern. Prefer plain constructor functions (`fn dark() ->
Palette`) that the caller stores wherever it likes.

### 5.3 `switcher.rs` — current palette + apply-to-ctx

**What to build:** applies the active palette into `egui::Context`
(likely via `ctx.style_mut()`).

**Errors to avoid:** applying the theme once at startup only. If the
switcher's `.set(new_palette)` doesn't propagate into `Style` on the
very next frame, switching themes at runtime silently requires an app
restart to take effect — easy to miss if you only ever test the
startup path.

### 5.4 `preview.rs` — live preview panel

**Keep in mind:** the preview panel should read from the *same*
`ThemeSwitcher` the consumer owns, not a separate copy.

**Errors to avoid:** hardcoding sample widgets in the preview that don't
actually route through the token system being demonstrated — the
preview then shows something that doesn't match what changing a token
will actually do elsewhere in the app.

### 5.5 Example (`theme_switcher.rs`)

---

## Part 6 — `test-app`

Build each subpart's matching scene (6.3) right after its crate lands
in Parts 1–5 — don't batch all the scenes at the end. Catching an API
awkwardness the moment a crate is "done" is the entire reason
`test-app` exists (`FOLDER_STRUCTURE.md` §7).

### 6.1 `main.rs`

Per `FOLDER_STRUCTURE.md` §6 — kept to a bootstrap script, no logic.

### 6.2 `app/state.rs` + `app/update.rs`

**Keep in mind:** `ActiveScene` and any shared/per-scene state live
here — this is the one place in `test-app` allowed to hold that state,
per the state-ownership rule (CODING_RULES §2) applied to the app side.

**Errors to avoid:** letting per-scene state leak into the `scenes/*`
files as module-level statics "just for convenience" — that's the same
anti-pattern CODING_RULES §9 flags inside the library, and it's just as
much of a problem in the app.

### 6.3 Individual demo scenes — one per crate

**Keep in mind:** each scene should exercise *only* its one crate.

**Errors to avoid:** "just adding a bit of theming" to `layout_demo.rs`
to make it look nicer, or similar cross-contamination between scenes.
Once a single-crate scene quietly depends on a second crate, it stops
being useful as an isolated sanity check — you lose the fast,
one-crate-at-a-time debugging story that's the entire point of having
per-scene isolation (`FOLDER_STRUCTURE.md` §7).

### 6.4 `combined_demo.rs`

**What to build:** several crates active together — the one scene that
actually proves composition (PRD §9, §11).

**Keep in mind:** this is where real integration bugs surface — state
ownership conflicts between crates that never show up in isolation.
Specifically worth checking here: does vim-nav focus interact correctly
with a spring-driven selection highlight moving to the newly-focused
region, and does an active theme correctly restyle a layout that's also
under vim-nav control.

**Errors to avoid:** deferring this scene "for later." Per PRD §9/§11 it
*is* the integration proof — there's no other place in this repo where
that proof happens. If anything gets cut under time pressure, per
`FOLDER_STRUCTURE.md` §7 this is the one scene to keep.

---

## Suggested build order

Dependency-driven, not strictly linear:

1. **Part 0** — once, first.
2. **Part 1** (`egui-layout` hardening) — quick, and resolves the naming
   question before anything else can start assuming an axis convention.
3. **Part 2** (`spring-core`), fully including tests, before Part 3 —
   CODING_RULES §5 wants `-core` crates independently correct first, and
   in practice it's much faster to find solver bugs in plain `#[test]`
   output than inside a rendered widget.
4. **Part 3** (`egui-spring`) — depends on Part 2.
5. **Part 4** (`egui-vim-nav`) and **Part 5** (`egui-themes`) — both
   independent of 2/3 and of each other; do them in either order, or in
   parallel if more than one person is working on this.
6. **Part 6.3** — add the matching `test-app` scene immediately after
   each of Parts 1/3/4/5 lands, not batched at the end.
7. **Part 6.4** (`combined_demo`) — last, once layout + spring + themes
   exist at minimum; vim-nav can be folded into it slightly later
   without blocking the rest.
