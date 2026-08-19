//! 1D Spring state and simulation container.

use crate::params::SpringParams;
use crate::solver::solve_step;

/// Position threshold for settling in logical units (default: `0.005`).
pub const DEFAULT_SETTLE_DISTANCE: f32 = 0.005;

/// Velocity threshold for settling in logical units per second (default: `0.05`).
pub const DEFAULT_SETTLE_VELOCITY: f32 = 0.05;

/// A 1D analytical spring state type.
///
/// # State Ownership
///
/// `Spring` is a plain value struct owned by the consuming application and updated
/// frame-by-frame (`CODING_RULES §2`). There is zero global or static state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Current animated position.
    pub current: f32,
    /// Current physical velocity in units per second.
    pub velocity: f32,
    /// Target equilibrium position.
    pub target: f32,
    /// Physical parameters governing movement dynamics.
    pub params: SpringParams,
    /// Position threshold below which motion is considered settled.
    pub settle_distance: f32,
    /// Velocity threshold below which motion is considered settled.
    pub settle_velocity: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self::new(0.0, SpringParams::default())
    }
}

impl Spring {
    /// Creates a new spring at an initial position with target equal to initial.
    pub fn new(initial: f32, params: SpringParams) -> Self {
        Self {
            current: initial,
            velocity: 0.0,
            target: initial,
            params,
            settle_distance: DEFAULT_SETTLE_DISTANCE,
            settle_velocity: DEFAULT_SETTLE_VELOCITY,
        }
    }

    /// Creates a spring at `initial` position with a distinct `target` equilibrium.
    pub fn with_target(initial: f32, target: f32, params: SpringParams) -> Self {
        Self {
            current: initial,
            velocity: 0.0,
            target,
            params,
            settle_distance: DEFAULT_SETTLE_DISTANCE,
            settle_velocity: DEFAULT_SETTLE_VELOCITY,
        }
    }

    /// Sets the target equilibrium position without interrupting current velocity.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Teleports the spring position and zeroes velocity immediately.
    pub fn reset(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Configures custom settling tolerance thresholds.
    pub fn with_settle_tolerances(mut self, distance: f32, velocity: f32) -> Self {
        self.settle_distance = distance.max(1e-6);
        self.settle_velocity = velocity.max(1e-6);
        self
    }

    /// Advances the spring simulation by delta time `dt` in seconds.
    ///
    /// If the spring is settled, position is clamped cleanly to `target` and velocity is zeroed.
    pub fn update(&mut self, dt: f32) {
        if self.is_settled() {
            self.current = self.target;
            self.velocity = 0.0;
            return;
        }

        let next = solve_step(self.current, self.velocity, self.target, dt, self.params);
        self.current = next.position;
        self.velocity = next.velocity;

        if self.is_settled() {
            self.current = self.target;
            self.velocity = 0.0;
        }
    }

    /// Checks if the spring has physically settled at its target.
    ///
    /// Evaluates **both** position distance $|x - x_{\text{target}}| < \text{settle\_distance}$
    /// **and** velocity $|v| < \text{settle\_velocity}$ (`BUILD_PLAN §2.3`).
    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() <= self.settle_distance
            && self.velocity.abs() <= self.settle_velocity
    }

    /// Returns current position value.
    pub fn value(&self) -> f32 {
        self.current
    }

    /// Returns current velocity value.
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Returns current target value.
    pub fn target(&self) -> f32 {
        self.target
    }
}
