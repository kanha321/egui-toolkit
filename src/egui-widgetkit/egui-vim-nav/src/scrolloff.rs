//! Reusable directional scrolloff and spring-damped viewport scrolling for egui `ScrollArea`.

use egui::{scroll_area::ScrollAreaOutput, Rect, ScrollArea};
use spring_core::{Spring, SpringParams};

use crate::Direction;

/// Direction of navigation used for viewport scrolloff tiebreaking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NavDirection {
    Up,
    Down,
    #[default]
    Unknown,
}

impl From<Option<Direction>> for NavDirection {
    fn from(dir: Option<Direction>) -> Self {
        match dir {
            Some(Direction::Up) => NavDirection::Up,
            Some(Direction::Down) => NavDirection::Down,
            _ => NavDirection::Unknown,
        }
    }
}

/// Pure geometric delta calculator for directional scrolloff margins.
///
/// Computes the required vertical viewport scroll adjustment to satisfy directional
/// scrolloff margins without unnecessary screen movement.
pub fn compute_scrolloff_delta(
    target: Rect,
    viewport: Rect,
    margin_top: f32,
    margin_bottom: f32,
    direction: NavDirection,
) -> f32 {
    let overshoot_down = (target.bottom() - (viewport.bottom() - margin_bottom)).max(0.0);
    let overshoot_up = ((viewport.top() + margin_top) - target.top()).max(0.0);

    match (overshoot_down > 0.0, overshoot_up > 0.0) {
        (false, false) => 0.0, // Comfort zone: viewport remains completely still

        (true, false) => overshoot_down, // Case A: normal widget, clipped at bottom

        (false, true) => -overshoot_up, // Case B: normal widget, clipped at top

        (true, true) => {
            // Case D: Widget taller than comfort zone (cannot satisfy both margins simultaneously)
            match direction {
                NavDirection::Down => overshoot_down, // Priority: bottom edge
                NavDirection::Up | NavDirection::Unknown => -overshoot_up, // Priority: top header
            }
        }
    }
}

/// Reusable spring-animated directional scrolloff manager for egui `ScrollArea`s.
///
/// Manages directional margins (e.g. 36px), critically damped harmonic spring viewport
/// smoothing, transition-frozen direction resolution, frame-local manual scroll synchronization,
/// and bounded max-scroll clamping.
#[derive(Clone, Debug)]
pub struct Scrolloff<T = egui::Id> {
    /// Top padding margin in pixels (default: 144.0).
    pub margin_top: f32,
    /// Bottom padding margin in pixels (default: 144.0).
    pub margin_bottom: f32,
    /// Analytical spring physics driver.
    pub spring: Spring,
    /// Target vertical scroll offset.
    pub target_offset: f32,
    /// Offset value injected into the `ScrollArea` during the current frame.
    pub offset_applied: f32,
    /// Last returned scroll offset from `ScrollAreaOutput`.
    pub last_scroll_offset: f32,
    /// Identity of the last focused widget/node.
    pub last_focused: Option<T>,
    /// Frozen navigation direction for oversized widget tiebreaking.
    pub active_direction: NavDirection,
    /// Whether scrolloff should be evaluated this frame.
    pub trigger_scrolloff: bool,
    /// Whether mouse/pointer-driven focus changes should trigger autoscroll (default: false).
    pub autoscroll_on_pointer: bool,
}

impl<T: PartialEq + Clone> Default for Scrolloff<T> {
    fn default() -> Self {
        Self {
            margin_top: 144.0,
            margin_bottom: 144.0,
            spring: Spring::new(0.0, SpringParams::new(24.0, 1.0)),
            target_offset: 0.0,
            offset_applied: 0.0,
            last_scroll_offset: 0.0,
            last_focused: None,
            active_direction: NavDirection::Unknown,
            trigger_scrolloff: false,
            autoscroll_on_pointer: false,
        }
    }
}

impl<T: PartialEq + Clone> Scrolloff<T> {
    /// Creates a new `Scrolloff` instance with default 144px margins and critically damped spring physics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom top and bottom scrolloff margins in pixels.
    pub fn with_margins(mut self, top: f32, bottom: f32) -> Self {
        self.margin_top = top;
        self.margin_bottom = bottom;
        self
    }

    /// Sets custom analytical spring physics parameters for viewport scrolling.
    pub fn with_spring_params(mut self, params: SpringParams) -> Self {
        self.spring = Spring::new(self.target_offset, params);
        self
    }

    /// Configures whether mouse/pointer-driven focus changes trigger autoscroll (default: `false`).
    pub fn with_autoscroll_on_pointer(mut self, enabled: bool) -> Self {
        self.autoscroll_on_pointer = enabled;
        self
    }

    /// Evaluates focus transitions and keypress events, freezing navigation direction
    /// strictly when a real transition or directional keypress occurs.
    ///
    /// Autoscroll is triggered when directional navigation occurs (`dir_input.is_some()`)
    /// or when pointer autoscroll is explicitly enabled.
    pub fn record_nav_event(&mut self, current_focused: Option<T>, dir_input: Option<Direction>) {
        let focus_transitioned = current_focused != self.last_focused;
        let is_key_nav = dir_input.is_some();
        if focus_transitioned || is_key_nav {
            self.active_direction = NavDirection::from(dir_input);
            self.last_focused = current_focused;
        }
        self.trigger_scrolloff = is_key_nav || (focus_transitioned && self.autoscroll_on_pointer);
    }

    /// Advances the internal spring simulation and configures `ScrollArea` with the animated offset.
    pub fn inject_into(&mut self, scroll_area: ScrollArea, dt: f32) -> (ScrollArea, f32) {
        self.spring.update(dt);
        let val = self.spring.value();
        self.offset_applied = val;
        (scroll_area.vertical_scroll_offset(val), val)
    }

    /// Synchronizes manual user scrolling (mouse wheel or scrollbar drag) by diffing
    /// the returned offset against the value injected this frame.
    pub fn sync_manual_scroll<R>(&mut self, scroll_output: &ScrollAreaOutput<R>) {
        let returned_offset = scroll_output.state.offset.y;
        const EPS: f32 = 0.5;
        if (returned_offset - self.offset_applied).abs() > EPS {
            self.target_offset = returned_offset;
            self.spring.reset(returned_offset);
        }
        self.last_scroll_offset = returned_offset;
    }

    /// Recomputes target scrolloff delta and clamps against content height if triggered.
    pub fn adjust_for_target(&mut self, target: Rect, viewport: Rect, content_height: f32) {
        if self.trigger_scrolloff {
            let delta = compute_scrolloff_delta(
                target,
                viewport,
                self.margin_top,
                self.margin_bottom,
                self.active_direction,
            );
            if delta != 0.0 {
                let max_scroll = (content_height - viewport.height()).max(0.0);
                self.target_offset = (self.offset_applied + delta).clamp(0.0, max_scroll);
                self.spring.set_target(self.target_offset);
            }
            self.trigger_scrolloff = false;
        }
    }

    /// Returns `true` if the viewport scroll spring has settled at its target position.
    pub fn is_settled(&self) -> bool {
        self.spring.is_settled()
    }

    /// Requests continuous frame repaints from `egui::Context` while the scroll spring is moving.
    pub fn request_repaint_if_needed(&self, ctx: &egui::Context) {
        if !self.is_settled() {
            ctx.request_repaint();
        }
    }
}
