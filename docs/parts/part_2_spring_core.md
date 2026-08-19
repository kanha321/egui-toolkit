# Part 2 — `spring-core` (Analytical Spring Physics Solver)

## Overview
`spring-core` is a pure mathematical, analytical (closed-form) 1D damped harmonic oscillator solver with **zero `egui` or GUI dependencies**. It solves the spring ODE continuously across time, guaranteeing **100% numerical stability and framerate independence** (never diverges or explodes like Euler approximations).

---

## What It Does

1. **Analytical Closed-Form Evaluation**:
   - Solves $\ddot{x} + 2\zeta\omega_0\dot{x} + \omega_0^2(x - x_{\text{target}}) = 0$ directly in continuous time:
     - **Underdamped ($\zeta < 1.0$)**: Harmonic oscillation with exponential envelope decay.
     - **Critically Damped ($\zeta \approx 1.0$)**: Fastest arrival at target with zero oscillation; evaluated within an $\epsilon$-band to prevent division-by-zero.
     - **Overdamped ($\zeta > 1.0$)**: Non-oscillatory exponential creep through thick damping.

2. **Framerate Independence**:
   - Evaluating one step of $100\text{ ms}$ yields the exact same position and velocity as evaluating $100$ steps of $1\text{ ms}$.

3. **Natural Parameterization (`SpringParams`)**:
   - Parameterized by natural angular frequency $\omega_0$ (radians/sec) and dimensionless damping ratio $\zeta$.
   - **Presets**:
     - `SpringParams::gentle()` ($\omega_0 = 14.0, \zeta = 0.90$): Smooth, soft glide.
     - `SpringParams::snappy()` ($\omega_0 = 28.0, \zeta = 0.85$): Fast, crisp arrival with near-zero overshoot.
     - `SpringParams::bouncy()` ($\omega_0 = 18.0, \zeta = 0.50$): Elastic spring with visible bounce.

4. **Two-Fold Settling Guarantee**:
   - `spring.is_settled()` validates **both** distance to target ($|x - x_{\text{target}}| < \text{settle\_dist}$) **and** velocity ($|v| < \text{settle\_vel}$).
   - Prevents premature animation freezes when an oscillating spring crosses target at high speed.

---

## How It Is Used in the Project (`test-app`)

```rust
use spring_core::{Spring, SpringParams};

// 1. App state owns the spring
let mut spring = Spring::new(0.0, SpringParams::snappy());

// 2. Set new target upon user interaction
spring.set_target(150.0);

// 3. Update each frame in egui update loop
let dt = ui.input(|i| i.stable_dt).min(0.05);
spring.update(dt);

// 4. Request continuous repaint while in motion (CODING_RULES §4)
if !spring.is_settled() {
    ui.ctx().request_repaint();
}

// 5. Use animated value to draw UI elements
let current_pos = spring.value();
```

---

## Settings Page Customization Options

When designing a **Settings / Animation & Physics Page**, the following parameters can be exposed to the user:

| Setting Key | UI Control | Range / Options | Description |
|---|---|---|---|
| `physics.preset` | ComboBox / Radio | `Gentle`, `Snappy`, `Bouncy`, `Custom` | Master animation feel preset |
| `physics.custom_frequency` | Slider | `2.0 ..= 50.0` rad/s ($\omega_0$) | Speed / stiffness of motion (higher = faster snap) |
| `physics.custom_damping` | Slider | `0.05 ..= 2.50` ($\zeta$) | Damping factor ($<1.0$ bouncy, $=1.0$ critical, $>1.0$ overdamped) |
| `physics.settle_distance` | DragValue / Scientific | `0.0001 ..= 0.01` px | Position tolerance threshold before halting animation |
| `physics.settle_velocity` | DragValue / Scientific | `0.001 ..= 0.1` px/s | Velocity tolerance threshold before halting animation |
| `physics.time_scale` | Slider | `0.1 ..= 2.0` (speed multiplier) | Global slow-motion or fast-forward multiplier applied to `dt` |
| `physics.enable_momentum` | Checkbox / Toggle | `true` / `false` | Enables physical kinetic impulse kicks on rapid clicks or drag release |
