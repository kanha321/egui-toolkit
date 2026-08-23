# spring-core

Framework-agnostic analytical spring physics solver. Pure math, **zero `egui` dependency**.

## What It Does

Solves the second-order ODE for a damped harmonic oscillator using closed-form analytical solutions — no iterative Euler/Verlet stepping. This gives exact, framerate-independent results with zero accumulated drift.

Supports three damping regimes:
- **Underdamped** (ζ < 1): Oscillates and overshoots before settling
- **Critically damped** (ζ = 1): Fastest approach without overshoot
- **Overdamped** (ζ > 1): Slow exponential approach, no oscillation

## Quick Start

```rust
use spring_core::{Spring, SpringParams};

// Create a spring at position 0.0 with snappy preset
let mut spring = Spring::new(0.0, SpringParams::snappy());

// Set a new target
spring.set_target(100.0);

// Advance by 16ms (one 60fps frame)
spring.update(1.0 / 60.0);

println!("Position: {:.2}, Velocity: {:.2}", spring.value(), spring.velocity());

// Keep updating until settled
while !spring.is_settled() {
    spring.update(1.0 / 60.0);
}

assert_eq!(spring.value(), 100.0);
```

## API

### `SpringParams`

Defines the spring's physical properties:

```rust
// Custom parameters
let params = SpringParams::new(20.0, 0.5); // ω₀ = 20 rad/s, ζ = 0.5

// Built-in presets
SpringParams::gentle();    // ω₀ = 18, ζ = 0.90 — slow, smooth, no overshoot
SpringParams::snappy();    // ω₀ = 32, ζ = 0.85 — fast, minimal overshoot
SpringParams::openrgb();   // ω₀ = 22, ζ = 0.65 — moderate overshoot
```

| Field | Description |
|---|---|
| `angular_frequency` (ω₀) | Natural frequency in rad/s. Higher = faster motion. |
| `damping_ratio` (ζ) | Controls oscillation. <1 = bounce, 1 = critical, >1 = sluggish. |

### `Spring`

The solver instance:

| Method | Description |
|---|---|
| `Spring::new(initial, params)` | Create at initial value with given params |
| `.set_target(target)` | Set the equilibrium position to approach |
| `.update(dt)` | Advance by `dt` seconds |
| `.value()` → `f32` | Current position |
| `.velocity()` → `f32` | Current velocity |
| `.is_settled()` → `bool` | True when position ≈ target and velocity ≈ 0 |

### `MotionPhysics`

High-level preset enum for selecting spring behavior:

```rust
use spring_core::MotionPhysics;

let physics = MotionPhysics::Default;  // Balanced default
let physics = MotionPhysics::Snappy;   // Fast, minimal overshoot
let physics = MotionPhysics::Gentle;   // Slow, cushioned
let physics = MotionPhysics::OpenRGB;  // Moderate overshoot
let physics = MotionPhysics::Off;      // Instant (no animation)
let physics = MotionPhysics::Custom(SpringParams::new(25.0, 0.6));
```

## Design Notes

- **Framerate independent**: Uses analytical closed-form solutions, not iterative integration. Passing `dt = 0.016` or `dt = 0.033` produces identical convergence.
- **Settling detection**: Uses position + velocity tolerance band. Once settled, the spring clamps to the exact target to prevent floating-point drift.
- **No allocations**: All operations are stack-only, no heap allocations.
