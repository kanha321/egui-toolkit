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
        Self {
            angular_frequency: 20.0,
            damping_ratio: 0.5,
        }
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

    /// Smooth, silky glide with subtle damping (`angular_frequency: 18.0`, `damping_ratio: 0.90`).
    pub const fn gentle() -> Self {
        Self {
            angular_frequency: 18.0,
            damping_ratio: 0.90,
        }
    }

    /// Crisp, responsive response (`angular_frequency: 32.0`, `damping_ratio: 0.85`).
    pub const fn snappy() -> Self {
        Self {
            angular_frequency: 32.0,
            damping_ratio: 0.85,
        }
    }

    /// Fast, lively elastic spring with energetic bounce (`angular_frequency: 24.0`, `damping_ratio: 0.40`).
    pub const fn bouncy() -> Self {
        Self {
            angular_frequency: 24.0,
            damping_ratio: 0.40,
        }
    }

    /// Signature OpenRGB / Neovide cursor feel (`angular_frequency: 22.0`, `damping_ratio: 0.65`).
    pub const fn openrgb() -> Self {
        Self {
            angular_frequency: 22.0,
            damping_ratio: 0.65,
        }
    }

    /// Fast, impactful spring dynamics with energetic elastic bounce (`angular_frequency: 26.0`, `damping_ratio: 0.44`).
    pub const fn responsive() -> Self {
        Self {
            angular_frequency: 26.0,
            damping_ratio: 0.44,
        }
    }

    /// Alias for [`Self::responsive`].
    pub const fn dropdown() -> Self {
        Self::responsive()
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

/// Motion physics mode for spring animations, including named presets and an instant `Off` mode.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MotionPhysics {
    /// Motion physics is disabled — highlights snap instantly to target with zero animation.
    Off,
    /// Default balanced spring dynamics ($\omega_0 = 20.0, \zeta = 0.50$).
    #[default]
    Default,
    /// Smooth, cushioned movement with heavy damping ($\omega_0 = 18.0, \zeta = 0.90$).
    Gentle,
    /// Responsive and impactful dynamics from dropdown morphing animations ($\omega_0 = 28.0, \zeta = 0.75$).
    Responsive,
    /// Crisp, rapid target snap ($\omega_0 = 32.0, \zeta = 0.85$).
    Snappy,
    /// Energetic elastic bounce with overshoot ($\omega_0 = 20.0, \zeta = 0.35$).
    Bouncy,
    /// OpenRGB / Neovide fluid travel ($\omega_0 = 22.0, \zeta = 0.65$).
    OpenRGB,
    /// Custom frequency ($\omega_0$) and damping ratio ($\zeta$).
    Custom(SpringParams),
}

impl MotionPhysics {
    /// Returns `true` if motion physics is disabled (`MotionPhysics::Off`).
    pub const fn is_off(&self) -> bool {
        matches!(self, Self::Off)
    }

    /// Returns `true` if motion physics is active (`!self.is_off()`).
    pub const fn is_enabled(&self) -> bool {
        !self.is_off()
    }

    /// Converts the motion physics mode to `Option<SpringParams>`, returning `None` when `Off`.
    pub fn to_params(&self) -> Option<SpringParams> {
        match self {
            Self::Off => None,
            Self::Default => Some(SpringParams::default()),
            Self::Gentle => Some(SpringParams::gentle()),
            Self::Responsive => Some(SpringParams::responsive()),
            Self::Snappy => Some(SpringParams::snappy()),
            Self::Bouncy => Some(SpringParams::bouncy()),
            Self::OpenRGB => Some(SpringParams::openrgb()),
            Self::Custom(params) => Some(*params),
        }
    }

    /// Convenience helper returning the display label for UI selectors.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Off => "Off (Instant)",
            Self::Default => "Default (20 / 0.50)",
            Self::Gentle => "Gentle (18 / 0.90)",
            Self::Responsive => "Fast & Bouncy (26 / 0.44)",
            Self::Snappy => "Snappy (32 / 0.85)",
            Self::Bouncy => "Bouncy (24 / 0.40)",
            Self::OpenRGB => "OpenRGB (22 / 0.65)",
            Self::Custom(_) => "Custom",
        }
    }
}
