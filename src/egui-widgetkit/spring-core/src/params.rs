//! Physical parameters and named presets for spring dynamics.

/// Physical parameters defining a damped harmonic oscillator.
///
/// Parameterized naturally by angular frequency $\omega_0$ (stiffness)
/// and dimensionless damping ratio $\zeta$ (zeta).
///
/// # Natural Regimes:
/// - $\zeta < 1.0$: **Underdamped** — Oscillates with overshoot before settling.
/// - $\zeta \approx 1.0$: **Critically Damped** — Fastest approach to equilibrium with minimal overshoot.
/// - $\zeta > 1.0$: **Overdamped** — Sluggish decay with zero oscillation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
    /// Natural angular frequency $\omega_0$ in radians per second.
    /// Higher values produce faster, stiffer response. Must be positive.
    pub angular_frequency: f32,
    /// Dimensionless damping ratio $\zeta$.
    /// Controls oscillation damping. Must be non-negative.
    pub damping_ratio: f32,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self::snappy()
    }
}

impl SpringParams {
    /// Creates a new `SpringParams` specification with validated non-negative parameters.
    pub fn new(angular_frequency: f32, damping_ratio: f32) -> Self {
        Self {
            angular_frequency: angular_frequency.max(0.001),
            damping_ratio: damping_ratio.max(0.0),
        }
    }

    /// Smooth, soft glide with subtle damping (`angular_frequency: 14.0`, `damping_ratio: 0.90`).
    pub const fn gentle() -> Self {
        Self {
            angular_frequency: 14.0,
            damping_ratio: 0.90,
        }
    }

    /// Crisp, rapid response with near-critical damping and negligible overshoot (`angular_frequency: 28.0`, `damping_ratio: 0.85`).
    pub const fn snappy() -> Self {
        Self {
            angular_frequency: 28.0,
            damping_ratio: 0.85,
        }
    }

    /// Lively, elastic spring with visible oscillatory bounce (`angular_frequency: 18.0`, `damping_ratio: 0.50`).
    pub const fn bouncy() -> Self {
        Self {
            angular_frequency: 18.0,
            damping_ratio: 0.50,
        }
    }

    /// Signature OpenRGB / Neovide cursor feel (`angular_frequency: 22.0`, `damping_ratio: 0.65`).
    pub const fn openrgb() -> Self {
        Self {
            angular_frequency: 22.0,
            damping_ratio: 0.65,
        }
    }

    /// Sets the angular frequency in radians per second.
    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.angular_frequency = frequency.max(0.001);
        self
    }

    /// Sets the dimensionless damping ratio.
    pub fn with_damping_ratio(mut self, damping_ratio: f32) -> Self {
        self.damping_ratio = damping_ratio.max(0.0);
        self
    }
}
