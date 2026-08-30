# spring-core

## What It Is

`spring-core` is a **pure-math, zero-dependency, analytical 1D spring physics engine** for damped harmonic oscillators. It solves the second-order ODE:

$$\ddot{x} + 2\zeta\omega_0\dot{x} + \omega_0^2(x - x_{\text{target}}) = 0$$

in closed form — no iterative Euler/Verlet stepping, no accumulated error. It is completely framework-agnostic: it has **zero `egui` dependency** and can be used in any Rust project that needs smooth, physically-based animation curves.

**Crate path:** `src/egui-widgetkit/spring-core/`

---

## What It Does

- **Closed-form solving** across all three damping regimes:
  - **Underdamped** (ζ < 1): Oscillatory decay with overshoot — bouncy, elastic motion.
  - **Critically damped** (ζ ≈ 1): Fastest convergence without oscillation — snappy, responsive.
  - **Overdamped** (ζ > 1): Slow exponential convergence — gentle, cushioned.
- **Frame-rate independent**: The solver evaluates exact position and velocity at any `dt`, so results are identical whether your app runs at 30fps, 60fps, or 144fps.
- **Automatic settling**: Springs detect when they've reached equilibrium (position and velocity both below configurable thresholds) and clamp cleanly to the target — no residual jitter.
- **Built-in presets**: Named physics profiles (`Snappy`, `Gentle`, `Bouncy`, `OpenRGB`) for common UI motion styles, plus a `Custom` escape hatch.

---

## How To Use It

### Add the Dependency

```toml
[dependencies]
spring-core = { path = "../spring-core" }
```

---

### Core Types

#### `SpringParams`

Defines the physical character of a spring.

```rust
pub struct SpringParams {
    pub angular_frequency: f32,  // ω₀ — how fast the spring oscillates (rad/s)
    pub damping_ratio: f32,      // ζ  — how quickly oscillation dies out
}
```

**Constructors & Presets:**

| Method | ω₀ | ζ | Character |
| :--- | :---: | :---: | :--- |
| `SpringParams::default()` | 20.0 | 0.50 | Balanced standard dynamics |
| `SpringParams::gentle()` | 18.0 | 0.90 | Smooth, cushioned, heavy damping |
| `SpringParams::snappy()` | 32.0 | 0.85 | Crisp, fast, minimal overshoot |
| `SpringParams::bouncy()` | 20.0 | 0.50 | Elastic bounce with visible overshoot |
| `SpringParams::openrgb()` | 22.0 | 0.65 | Fluid travel (Neovide/OpenRGB feel) |

**Builder methods:**

```rust
let params = SpringParams::default()
    .with_frequency(25.0)      // override ω₀
    .with_damping_ratio(0.75); // override ζ
```

---

#### `Spring`

The main state container. You own this struct and advance it each frame.

```rust
pub struct Spring {
    pub current: f32,          // animated position
    pub velocity: f32,         // current velocity (units/sec)
    pub target: f32,           // target equilibrium
    pub params: SpringParams,  // physics parameters
    pub settle_distance: f32,  // position settle threshold (default: 0.005)
    pub settle_velocity: f32,  // velocity settle threshold (default: 0.05)
}
```

**Creating a Spring:**

```rust
use spring_core::{Spring, SpringParams};

// Start at 0.0 with default physics
let mut spring = Spring::new(0.0, SpringParams::default());

// Start at 0.0, target 100.0 immediately
let mut spring = Spring::with_target(0.0, 100.0, SpringParams::snappy());

// Custom settle tolerances for tighter convergence
let mut spring = Spring::new(0.0, SpringParams::gentle())
    .with_settle_tolerances(0.001, 0.01);
```

**Driving the animation loop:**

```rust
// Set a new target (does NOT reset velocity — preserves momentum)
spring.set_target(100.0);

// Each frame, advance by the frame's delta time
spring.update(dt);  // dt = ui.input(|i| i.stable_dt) in egui

// Read the current animated value
let pos = spring.value();        // current position
let vel = spring.velocity();     // current velocity
let tgt = spring.target();       // target position
let done = spring.is_settled();  // true when at rest

// Teleport instantly (resets velocity to zero)
spring.reset(50.0);
```

**Complete example — animating a widget position:**

```rust
use spring_core::{Spring, SpringParams};

struct MyApp {
    x_spring: Spring,
}

impl MyApp {
    fn new() -> Self {
        Self {
            x_spring: Spring::new(0.0, SpringParams::snappy()),
        }
    }

    fn update(&mut self, ui: &mut egui::Ui) {
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        // Move to 200px when hovered
        if ui.rect_contains_pointer(some_rect) {
            self.x_spring.set_target(200.0);
        } else {
            self.x_spring.set_target(0.0);
        }

        self.x_spring.update(dt);
        let x = self.x_spring.value();

        // Request repaint while animating
        if !self.x_spring.is_settled() {
            ui.ctx().request_repaint();
        }
    }
}
```

---

#### `MotionPhysics`

A high-level enum for UI dropdowns, settings panels, or preference toggles.

```rust
pub enum MotionPhysics {
    Off,              // instant snap, no animation
    Default,          // ω₀=20, ζ=0.50
    Gentle,           // ω₀=18, ζ=0.90
    Snappy,           // ω₀=32, ζ=0.85
    Bouncy,           // ω₀=20, ζ=0.50
    OpenRGB,          // ω₀=22, ζ=0.65
    Custom(SpringParams),
}
```

**Key methods:**

```rust
let mode = MotionPhysics::Snappy;

mode.is_off();       // false
mode.is_enabled();   // true
mode.label();        // "Snappy (32 / 0.85)" — for UI display

// Convert to SpringParams (None if Off)
if let Some(params) = mode.to_params() {
    let spring = Spring::new(0.0, params);
}
```

---

#### `solve_step` (Low-Level Solver)

If you need direct access to the solver without the `Spring` wrapper:

```rust
use spring_core::{solve_step, SpringParams, SolverState};

let state: SolverState = solve_step(
    current_position,   // x(t)
    current_velocity,   // v(t)
    target_position,    // equilibrium
    dt,                 // time step in seconds
    SpringParams::default(),
);

let new_x = state.position;  // x(t + dt)
let new_v = state.velocity;  // v(t + dt)
```

---

### API Reference

#### `SpringParams`

| Method | Signature | Description |
| :--- | :--- | :--- |
| `new` | `fn new(angular_frequency: f32, damping_ratio: f32) -> Self` | Validated constructor (clamps ω₀ ≥ 0.001, ζ ≥ 0.0) |
| `gentle` | `const fn gentle() -> Self` | Preset: ω₀=18, ζ=0.90 |
| `snappy` | `const fn snappy() -> Self` | Preset: ω₀=32, ζ=0.85 |
| `bouncy` | `const fn bouncy() -> Self` | Preset: ω₀=20, ζ=0.50 |
| `openrgb` | `const fn openrgb() -> Self` | Preset: ω₀=22, ζ=0.65 |
| `with_frequency` | `fn with_frequency(self, f: f32) -> Self` | Builder: set ω₀ |
| `with_damping_ratio` | `fn with_damping_ratio(self, ζ: f32) -> Self` | Builder: set ζ |

#### `Spring`

| Method | Signature | Description |
| :--- | :--- | :--- |
| `new` | `fn new(initial: f32, params: SpringParams) -> Self` | Create at position with target = initial |
| `with_target` | `fn with_target(initial: f32, target: f32, params: SpringParams) -> Self` | Create with distinct initial and target |
| `with_settle_tolerances` | `fn with_settle_tolerances(self, dist: f32, vel: f32) -> Self` | Custom convergence thresholds |
| `set_target` | `fn set_target(&mut self, target: f32)` | Set target without velocity reset |
| `reset` | `fn reset(&mut self, value: f32)` | Teleport to value, zero velocity |
| `update` | `fn update(&mut self, dt: f32)` | Advance simulation by dt seconds |
| `value` | `fn value(&self) -> f32` | Current animated position |
| `velocity` | `fn velocity(&self) -> f32` | Current velocity |
| `target` | `fn target(&self) -> f32` | Target equilibrium |
| `is_settled` | `fn is_settled(&self) -> bool` | True when at rest |

#### `MotionPhysics`

| Method | Signature | Description |
| :--- | :--- | :--- |
| `is_off` | `const fn is_off(&self) -> bool` | True if `Off` variant |
| `is_enabled` | `const fn is_enabled(&self) -> bool` | True if not `Off` |
| `to_params` | `fn to_params(&self) -> Option<SpringParams>` | Convert to params (None for Off) |
| `label` | `const fn label(&self) -> &'static str` | UI display string |
