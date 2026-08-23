//! Optional spring-animated push/pop transition support with directional slide, shrink, fade, and pop overshoot.
//!
//! Available only when the `animated-transitions` feature is enabled.

#[cfg(feature = "animated-transitions")]
use egui_spring::{MotionPhysics, Spring, SpringParams};

/// Direction from which an incoming destination slides into the viewport.
#[cfg(feature = "animated-transitions")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SlideDirection {
    /// Incoming screen slides in from the right edge (and exits back to the right on pop).
    #[default]
    FromRight,
    /// Incoming screen slides in from the left edge (and exits back to the left on pop).
    FromLeft,
    /// Incoming screen slides in from the top edge (and exits back to the top on pop).
    FromTop,
    /// Incoming screen slides in from the bottom edge (and exits back to the bottom on pop).
    FromBottom,
}

/// Transition mode: Push (new screen on top) vs Pop (returning to screen below).
#[cfg(feature = "animated-transitions")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionKind {
    /// Pushing a new screen onto the stack.
    Push,
    /// Popping the active screen to reveal the screen below.
    Pop,
}

/// App-owned state tracking spring-animated transitions between destinations.
///
/// # State Ownership
///
/// `NavTransition<K>` is owned and stored by the consuming application alongside `NavStack<K>`
/// (e.g. on `TestAppState`) and passed by `&mut` reference to [`NavDisplay::transition`](crate::NavDisplay::transition).
///
/// # Physics & Dynamics
///
/// - **Push**: The background screen shrinks ($1.0 \to 0.90$) and fades ($1.0 \to 0.0$) while the incoming screen
///   slides into view from the configured [`SlideDirection`].
/// - **Pop**: The top screen slides out towards the same side it entered, while the screen below grows from behind,
///   using spring motion physics to smoothly overshoot/overexpand ($\approx 110\%$) before settling cleanly at $100\%$.
///
/// # Direct Triggering
///
/// Transitions can be explicitly triggered at the exact moment a navigation action occurs via
/// [`trigger_push`](Self::trigger_push), [`trigger_pop`](Self::trigger_pop), or animated stack helpers.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "animated-transitions")]
/// # {
/// use egui_nav_stack::{NavTransition, SlideDirection, MotionPhysics};
///
/// let mut transition: NavTransition<&'static str> = NavTransition::new()
///     .with_direction(SlideDirection::FromRight)
///     .with_physics(MotionPhysics::Default);
/// # }
/// ```
#[cfg(feature = "animated-transitions")]
#[derive(Clone, Debug)]
pub struct NavTransition<K> {
    pub(crate) previous_screen: Option<K>,
    pub(crate) current_screen: Option<K>,
    pub(crate) kind: TransitionKind,
    pub(crate) slide_direction: SlideDirection,
    pub(crate) physics: MotionPhysics,
    pub(crate) shrink_factor: f32,
    pub(crate) overshoot_factor: f32,
    pub(crate) spring: Spring,
    pub(crate) settled: bool,
}

#[cfg(feature = "animated-transitions")]
impl<K> Default for NavTransition<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "animated-transitions")]
impl<K> NavTransition<K> {
    /// Creates a new `NavTransition` state with default spring physics ($\omega_0 = 26.0, \zeta = 0.58$)
    /// and slide from right.
    pub fn new() -> Self {
        let default_params = SpringParams::new(26.0, 0.58);
        Self {
            previous_screen: None,
            current_screen: None,
            kind: TransitionKind::Push,
            slide_direction: SlideDirection::FromRight,
            physics: MotionPhysics::Custom(default_params),
            shrink_factor: 0.10, // Shrinks to 90% scale
            overshoot_factor: 0.40, // Enables overexpansion up to ~110% on pop
            spring: Spring::new(1.0, default_params),
            settled: true,
        }
    }

    /// Configures motion physics (presets like `Gentle`, `Snappy`, `Bouncy`, `OpenRGB`, `Custom`, or `Off`).
    pub fn with_physics(mut self, physics: MotionPhysics) -> Self {
        self.set_physics(physics);
        self
    }

    /// Sets motion physics dynamically at runtime.
    pub fn set_physics(&mut self, physics: MotionPhysics) {
        self.physics = physics;
        if let Some(params) = physics.to_params() {
            self.spring.params = params;
        } else {
            // MotionPhysics::Off
            self.snap();
        }
    }

    /// Configures the directional entry axis for sliding screens.
    pub fn with_direction(mut self, direction: SlideDirection) -> Self {
        self.slide_direction = direction;
        self
    }

    /// Sets the slide direction dynamically.
    pub fn set_direction(&mut self, direction: SlideDirection) {
        self.slide_direction = direction;
    }

    /// Configures the background shrink factor (default: `0.10`, meaning shrink to `90%` scale).
    pub fn with_shrink(mut self, shrink_factor: f32) -> Self {
        self.shrink_factor = shrink_factor.clamp(0.0, 0.5);
        self
    }

    /// Sets the background shrink factor dynamically.
    pub fn set_shrink(&mut self, shrink_factor: f32) {
        self.shrink_factor = shrink_factor.clamp(0.0, 0.5);
    }

    /// Configures the pop overexpansion factor (default: `0.40`, scaling spring overshoot up to `~110%`).
    pub fn with_overshoot(mut self, overshoot_factor: f32) -> Self {
        self.overshoot_factor = overshoot_factor.clamp(0.0, 1.0);
        self
    }

    /// Sets the pop overexpansion factor dynamically.
    pub fn set_overshoot(&mut self, overshoot_factor: f32) {
        self.overshoot_factor = overshoot_factor.clamp(0.0, 1.0);
    }

    /// Explicitly triggers a Push transition from `outgoing` to `incoming`.
    pub fn trigger_push(&mut self, outgoing: K, incoming: K) {
        if let Some(params) = self.physics.to_params() {
            self.previous_screen = Some(outgoing);
            self.current_screen = Some(incoming);
            self.kind = TransitionKind::Push;
            self.spring = Spring::with_target(0.0, 1.0, params);
            self.settled = false;
        } else {
            self.previous_screen = None;
            self.current_screen = Some(incoming);
            self.settled = true;
        }
    }

    /// Explicitly triggers a Pop transition from `outgoing` back to `incoming`.
    pub fn trigger_pop(&mut self, outgoing: K, incoming: K) {
        if let Some(params) = self.physics.to_params() {
            self.previous_screen = Some(outgoing);
            self.current_screen = Some(incoming);
            self.kind = TransitionKind::Pop;
            self.spring = Spring::with_target(0.0, 1.0, params);
            self.settled = false;
        } else {
            self.previous_screen = None;
            self.current_screen = Some(incoming);
            self.settled = true;
        }
    }

    /// Returns the active motion physics mode.
    pub fn physics(&self) -> MotionPhysics {
        self.physics
    }

    /// Returns the active slide direction.
    pub fn slide_direction(&self) -> SlideDirection {
        self.slide_direction
    }

    /// Returns `true` if no transition animation is currently running.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Forces the transition to complete immediately, snapping to the active screen.
    pub fn snap(&mut self) {
        self.spring.reset(1.0);
        self.spring.target = 1.0;
        self.previous_screen = None;
        self.settled = true;
    }
}
