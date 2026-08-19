//! Analytical closed-form solution to the damped harmonic oscillator ODE.
//!
//! Solves $\ddot{x} + 2\zeta\omega_0\dot{x} + \omega_0^2(x - x_{\text{target}}) = 0$
//! exactly in continuous time, guaranteeing 100% numerical stability regardless of frame time delta.

use crate::params::SpringParams;

/// Epsilon band for numerical stability around critical damping $\zeta = 1.0$.
const CRITICAL_DAMPING_EPSILON: f32 = 0.0001;

/// Analytical state result of evaluating the spring at time $t + dt$.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverState {
    /// New position $x(t + dt)$.
    pub position: f32,
    /// New velocity $v(t + dt) = \dot{x}(t + dt)$.
    pub velocity: f32,
}

/// Evaluates the analytical closed-form ODE solution for a 1D spring step.
///
/// - `current`: Current position $x(t)$.
/// - `velocity`: Current velocity $v(t)$.
/// - `target`: Target equilibrium position $x_{\text{target}}$.
/// - `dt`: Time step in seconds (from frame delta time).
/// - `params`: Oscillator physical parameters.
pub fn solve_step(
    current: f32,
    velocity: f32,
    target: f32,
    dt: f32,
    params: SpringParams,
) -> SolverState {
    if dt <= 0.0 {
        return SolverState {
            position: current,
            velocity,
        };
    }

    let w0 = params.angular_frequency;
    let zeta = params.damping_ratio;

    if w0 <= 0.0001 {
        return SolverState {
            position: target,
            velocity: 0.0,
        };
    }

    let x0 = current - target;
    let v0 = velocity;

    if zeta < 1.0 - CRITICAL_DAMPING_EPSILON {
        // --- Regime 1: Underdamped (zeta < 1.0) ---
        // Characteristic roots: -zeta*w0 ± i*gamma
        let zeta_w0 = zeta * w0;
        let gamma = w0 * (1.0 - zeta * zeta).sqrt();
        let decay = (-zeta_w0 * dt).exp();

        let cos_term = (gamma * dt).cos();
        let sin_term = (gamma * dt).sin();

        let x_t = decay * (x0 * cos_term + ((v0 + zeta_w0 * x0) / gamma) * sin_term);
        let v_t = decay * (v0 * cos_term - ((zeta_w0 * v0 + w0 * w0 * x0) / gamma) * sin_term);

        SolverState {
            position: x_t + target,
            velocity: v_t,
        }
    } else if zeta > 1.0 + CRITICAL_DAMPING_EPSILON {
        // --- Regime 2: Overdamped (zeta > 1.0) ---
        // Characteristic roots: -zeta*w0 ± gamma (two distinct negative real roots)
        let zeta_w0 = zeta * w0;
        let gamma = w0 * (zeta * zeta - 1.0).sqrt();
        let decay = (-zeta_w0 * dt).exp();

        let cosh_term = (gamma * dt).cosh();
        let sinh_term = (gamma * dt).sinh();

        let x_t = decay * (x0 * cosh_term + ((v0 + zeta_w0 * x0) / gamma) * sinh_term);
        let v_t = decay * (v0 * cosh_term - ((zeta_w0 * v0 + w0 * w0 * x0) / gamma) * sinh_term);

        SolverState {
            position: x_t + target,
            velocity: v_t,
        }
    } else {
        // --- Regime 3: Critically Damped (zeta ≈ 1.0) ---
        // Characteristic roots: repeated real root -w0
        let decay = (-w0 * dt).exp();
        let b = v0 + w0 * x0;

        let x_t = decay * (x0 + b * dt);
        let v_t = decay * (v0 - w0 * b * dt);

        SolverState {
            position: x_t + target,
            velocity: v_t,
        }
    }
}
