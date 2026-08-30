//! Spring-animated toggle switch widget.
//!
//! Provides [`Switch`] with smooth, physical thumb translation, track color morphing,
//! optional label integration, and full theme & physics customization.
//!
//! # State Ownership
//!
//! Animations can be tracked using automatic ID-scoped memory or an explicit caller-owned
//! [`SwitchState`] struct (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Color32, Id, Rect, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Sizing presets for toggle switches.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SwitchSize {
    /// Compact switch (36×20pt) for dense toolbars and settings lists.
    Compact,
    /// Standard switch (46×26pt) for primary options.
    #[default]
    Standard,
    /// Large switch (56×32pt) for prominent triggers.
    Large,
    /// Custom track width and height.
    Custom { width: f32, height: f32 },
}

impl SwitchSize {
    /// Returns the (width, height, thumb_radius) tuple for this size preset.
    pub fn dimensions(&self) -> (f32, f32, f32) {
        match self {
            Self::Compact => (36.0, 20.0, 7.5),
            Self::Standard => (46.0, 26.0, 10.0),
            Self::Large => (56.0, 32.0, 12.5),
            Self::Custom { width, height } => (*width, *height, (*height * 0.5) - 3.0),
        }
    }
}

/// Response returned by [`Switch::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`clicked`, `is_pressed`, `is_held`, `changed`).
#[derive(Clone, Debug)]
pub struct SwitchResponse {
    /// The underlying [`egui::Response`].
    pub response: egui::Response,
    /// Whether the switch is currently pressed / held down (mouse or keyboard).
    pub is_pressed: bool,
    /// Standard click action: fired strictly on release.
    pub clicked: bool,
}

impl SwitchResponse {
    /// Returns `true` if the switch was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` while the switch is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` while the switch is currently held down (alias for `is_pressed`).
    #[inline]
    pub fn is_held(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if the switch value changed this frame.
    #[inline]
    pub fn changed(&self) -> bool {
        self.response.changed()
    }

    /// Unwraps and returns the inner [`egui::Response`].
    #[inline]
    pub fn into_inner(self) -> egui::Response {
        self.response
    }
}

impl std::ops::Deref for SwitchResponse {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl std::ops::DerefMut for SwitchResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl From<SwitchResponse> for egui::Response {
    #[inline]
    fn from(r: SwitchResponse) -> Self {
        r.response
    }
}

/// Persistent animation state for spring-driven toggle switches.
#[derive(Clone, Debug)]
pub struct SwitchState {
    /// Spring driving thumb horizontal translation ($0.0 \to 1.0$).
    pub thumb_spring: Spring,
    /// Spring driving track hover/focus highlight ($0.0 \to 1.0$).
    pub hover_spring: Spring,
    /// Critically damped spring driving smooth track background crossfade ($0.0 \to 1.0$).
    pub color_spring: Spring,
    /// Spring driving press compression and pop rebound ($0.0 \to 1.0$).
    pub press_spring: Spring,
    /// Spring driving instantaneous organic expand-and-shrink focus bounce ($0.0 \to 0.0$).
    pub focus_spring: Spring,
    /// Tracks previous frame's focus state to detect focus entrance.
    pub was_focused: bool,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            thumb_spring: Spring::new(0.0, SpringParams::new(26.0, 0.46)),
            hover_spring: Spring::new(0.0, SpringParams::new(18.0, 0.48)),
            color_spring: Spring::new(0.0, SpringParams::new(14.0, 1.0)),
            press_spring: Spring::new(0.0, SpringParams::new(14.0, 0.42)),
            focus_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            was_focused: false,
        }
    }
}

impl SwitchState {
    /// Creates a state initialized to the given boolean state.
    pub fn new(is_on: bool) -> Self {
        let initial_val = if is_on { 1.0 } else { 0.0 };
        Self {
            thumb_spring: Spring::new(initial_val, SpringParams::new(26.0, 0.46)),
            hover_spring: Spring::new(0.0, SpringParams::new(18.0, 0.48)),
            color_spring: Spring::new(initial_val, SpringParams::new(14.0, 1.0)),
            press_spring: Spring::new(0.0, SpringParams::new(14.0, 0.42)),
            focus_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            was_focused: false,
        }
    }

    /// Triggers an immediate smooth focus jiggle impulse.
    pub fn trigger_focus_bounce(&mut self) {
        self.focus_spring.current = 0.0;
        self.focus_spring.velocity = 20.0;
        self.focus_spring.set_target(0.0);
    }

    /// Triggers an elastic pop animation on toggle click release.
    pub fn trigger_click(&mut self, is_on: bool) {
        self.thumb_spring.velocity = if is_on { 18.0 } else { -18.0 };
        self.press_spring.velocity = (self.press_spring.velocity + 16.0).min(20.0);
        self.press_spring.set_target(0.0);
        self.thumb_spring.set_target(if is_on { 1.0 } else { 0.0 });
    }

    /// Updates internal springs and requests repaint if in motion.
    pub fn update(
        &mut self,
        dt: f32,
        is_on: bool,
        is_focused: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if is_focused && !self.was_focused {
            self.trigger_focus_bounce();
        }
        self.was_focused = is_focused;

        self.hover_spring.set_target(if is_focused { 1.0 } else { 0.0 });
        self.color_spring.set_target(if is_on { 1.0 } else { 0.0 });

        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.trigger_click(is_on);
        } else {
            self.press_spring.set_target(0.0);
            self.thumb_spring.set_target(if is_on { 1.0 } else { 0.0 });
        }

        self.thumb_spring.update(dt);
        self.hover_spring.update(dt);
        self.color_spring.update(dt);
        self.press_spring.update(dt);
        self.focus_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have reached target equilibrium.
    pub fn is_settled(&self) -> bool {
        self.thumb_spring.is_settled()
            && self.hover_spring.is_settled()
            && self.color_spring.is_settled()
            && self.press_spring.is_settled()
            && self.focus_spring.is_settled()
    }
}

/// A theme-aware, spring-animated toggle switch widget.
///
/// # Example
/// ```no_run
/// use egui_widgets::Switch;
///
/// # egui::__run_test_ui(|ui| {
/// let mut enabled = true;
/// Switch::new(&mut enabled)
///     .label("Enable High Refresh Rate")
///     .show(ui);
/// # });
/// ```
pub struct Switch<'a> {
    selected: &'a mut bool,
    label: Option<WidgetText>,
    size: SwitchSize,
    track_fill_on: Option<Color32>,
    track_fill_off: Option<Color32>,
    thumb_fill_on: Option<Color32>,
    thumb_fill_off: Option<Color32>,
    track_stroke: Option<Stroke>,
    thumb_stroke: Option<Stroke>,
    label_color: Option<Color32>,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut SwitchState>,
    focused: bool,
    triggered: bool,
    pressed: bool,
}

impl<'a> Switch<'a> {
    /// Creates a new toggle switch bound to the given boolean mutable reference.
    pub fn new(selected: &'a mut bool) -> Self {
        Self {
            selected,
            label: None,
            size: SwitchSize::Standard,
            track_fill_on: None,
            track_fill_off: None,
            thumb_fill_on: None,
            thumb_fill_off: None,
            track_stroke: None,
            thumb_stroke: None,
            label_color: None,
            spring_params: SpringParams::new(26.0, 0.46),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
        }
    }

    /// Sets an optional text label displayed alongside the switch.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the switch sizing preset.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Shortcut for [`SwitchSize::Compact`].
    pub fn compact(self) -> Self {
        self.size(SwitchSize::Compact)
    }

    /// Shortcut for [`SwitchSize::Standard`].
    pub fn standard(self) -> Self {
        self.size(SwitchSize::Standard)
    }

    /// Shortcut for [`SwitchSize::Large`].
    pub fn large(self) -> Self {
        self.size(SwitchSize::Large)
    }

    /// Explicitly overrides the active (ON) track background color.
    pub fn track_on(mut self, color: Color32) -> Self {
        self.track_fill_on = Some(color);
        self
    }

    /// Explicitly overrides the inactive (OFF) track background color.
    pub fn track_off(mut self, color: Color32) -> Self {
        self.track_fill_off = Some(color);
        self
    }

    /// Explicitly overrides the active (ON) thumb knob color.
    pub fn thumb_on(mut self, color: Color32) -> Self {
        self.thumb_fill_on = Some(color);
        self
    }

    /// Explicitly overrides the inactive (OFF) thumb knob color.
    pub fn thumb_off(mut self, color: Color32) -> Self {
        self.thumb_fill_off = Some(color);
        self
    }

    /// Explicitly overrides the track border stroke.
    pub fn track_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.track_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the thumb knob border stroke.
    pub fn thumb_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.thumb_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the label text color.
    pub fn label_color(mut self, color: Color32) -> Self {
        self.label_color = Some(color);
        self
    }

    /// Sets custom spring physics parameters for thumb sliding.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Enables or disables spring motion animation (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Explicitly binds an external [`SwitchState`].
    pub fn with_state(mut self, state: &'a mut SwitchState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Explicitly marks the switch as focused (aligns with keyboard/vim focus).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly marks the switch as pressed / held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers a toggle action this frame (e.g. from Enter/Space/F key release).
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Provides an explicit unique ID for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the toggle switch and updates `selected` upon interaction.
    pub fn show(self, ui: &mut Ui) -> SwitchResponse {
        let (track_w, track_h, thumb_r) = self.size.dimensions();
        let track_size = vec2(track_w, track_h);

        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let mut total_size = track_size;
        if let Some(ref lg) = label_galley {
            total_size.x += lg.size().x + 8.0;
            total_size.y = total_size.y.max(lg.size().y);
        }

        let (rect, mut response) = ui.allocate_exact_size(total_size, Sense::click());

        let is_focused = self.focused || response.has_focus();
        let (is_key_down, is_key_released) = if is_focused {
            ui.input(|i| {
                if i.modifiers.ctrl || i.modifiers.alt {
                    (false, false)
                } else {
                    let down = i.key_down(egui::Key::F) || i.key_down(egui::Key::Enter) || i.key_down(egui::Key::Space);
                    let released = i.key_released(egui::Key::F) || i.key_released(egui::Key::Enter) || i.key_released(egui::Key::Space);
                    (down, released)
                }
            })
        } else {
            (false, false)
        };

        let is_pressed = self.pressed
            || (is_focused && (is_key_down || ui.input(|i| i.pointer.primary_down())))
            || response.is_pointer_button_down_on();
        let is_clicked = self.triggered
            || (is_focused && is_key_released)
            || response.clicked();
        if is_clicked {
            *self.selected = !*self.selected;
            response.mark_changed();
        }

        // Resolve colors
        let (track_on, track_off, thumb_on, thumb_off, track_stroke, label_color) =
            if let Some(p) = self.palette {
                (
                    self.track_fill_on.unwrap_or(p.accent),
                    self.track_fill_off.unwrap_or(p.crust),
                    self.thumb_fill_on.unwrap_or(if p.dark { p.crust } else { Color32::WHITE }),
                    self.thumb_fill_off.unwrap_or(p.overlay1),
                    self.track_stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.label_color.unwrap_or(p.text),
                )
            } else {
                let v = ui.visuals();
                (
                    self.track_fill_on.unwrap_or(v.selection.bg_fill),
                    self.track_fill_off.unwrap_or(v.widgets.inactive.bg_fill),
                    self.thumb_fill_on.unwrap_or(v.widgets.active.fg_stroke.color),
                    self.thumb_fill_off.unwrap_or(v.widgets.inactive.fg_stroke.color),
                    self.track_stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                    self.label_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                )
            };

        // Motion physics: hover is driven strictly by focus
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let is_on = *self.selected;

        let (thumb_progress, hover_factor, color_progress, press_factor, focus_bounce) = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, is_on, is_focused, is_pressed, is_clicked, ui.ctx());
                (
                    state.thumb_spring.value(),
                    state.hover_spring.value(),
                    state.color_spring.value(),
                    state.press_spring.value(),
                    state.focus_spring.value(),
                )
            } else {
                let id = self.id_source.unwrap_or_else(|| {
                    if let Some(ref l) = self.label {
                        ui.make_persistent_id(l.text())
                    } else {
                        response.id
                    }
                });
                let mut state: SwitchState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| SwitchState::new(is_on))
                });

                state.update(dt, is_on, is_focused, is_pressed, is_clicked, ui.ctx());
                let values = (
                    state.thumb_spring.value(),
                    state.hover_spring.value(),
                    state.color_spring.value(),
                    state.press_spring.value(),
                    state.focus_spring.value(),
                );
                ui.data_mut(|d| d.insert_temp(id, state));
                values
            }
        } else {
            (
                if is_on { 1.0 } else { 0.0 },
                if is_focused { 1.0 } else { 0.0 },
                if is_on { 1.0 } else { 0.0 },
                if is_pressed { 1.0 } else { 0.0 },
                0.0,
            )
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Track positioning with smooth press depression & squash-and-stretch focus jiggle
            let track_y_sink = press_factor * 1.5;
            let scale_x = (1.0 + (focus_bounce * 0.12) - (press_factor * 0.08)).max(0.70);
            let scale_y = (1.0 - (focus_bounce * 0.07) - (press_factor * 0.12)).max(0.70);
            let scaled_track_size = vec2(track_size.x * scale_x, track_size.y * scale_y);
            let track_center = pos2(
                rect.left() + track_size.x * 0.5,
                rect.center().y + track_y_sink,
            );
            let track_rect = Rect::from_center_size(track_center, scaled_track_size);
            let track_rounding = Rounding::same(scaled_track_size.y * 0.5);

            // Interpolate track fill smoothly via dedicated color crossfade
            let base_track_fill = lerp_color(track_off, track_on, color_progress.clamp(0.0, 1.0));
            let final_track_fill = if hover_factor > 0.01 {
                base_track_fill.linear_multiply(1.0 + hover_factor.clamp(0.0, 1.0) * 0.10)
            } else {
                base_track_fill
            };

            // Paint track — clean base stroke, no added focus outline
            painter.add(Shape::rect_filled(track_rect, track_rounding, final_track_fill));
            if track_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(track_rect, track_rounding, track_stroke));
            }

            // Thumb calculation with overshoot and elastic dynamic squash & stretch
            let min_x = track_rect.left() + track_rect.height() * 0.5;
            let max_x = track_rect.right() - track_rect.height() * 0.5;
            let travel_distance = max_x - min_x;
            let thumb_x = min_x + thumb_progress * travel_distance;
            let thumb_center = pos2(thumb_x, track_rect.center().y);

            let thumb_color = lerp_color(thumb_off, thumb_on, color_progress.clamp(0.0, 1.0));

            // Dynamic elastic squash along movement axis & press compression
            let target_val = if is_on { 1.0 } else { 0.0 };
            let motion_stretch = (thumb_progress - target_val).abs().min(0.5) * 0.40;
            let press_stretch = press_factor * 0.25;
            let total_w_factor = 1.0 + motion_stretch + press_stretch;
            let total_h_factor = (1.0 - motion_stretch * 0.4 - press_factor * 0.15).max(0.65);

            let thumb_w = thumb_r * scale_x * total_w_factor;
            let thumb_h_rad = thumb_r * scale_y * total_h_factor;
            let thumb_rect = Rect::from_center_size(thumb_center, vec2(thumb_w * 2.0, thumb_h_rad * 2.0));

            // Paint thumb knob
            painter.add(Shape::rect_filled(thumb_rect, Rounding::same(thumb_h_rad), thumb_color));
            if let Some(stroke) = self.thumb_stroke {
                painter.add(Shape::rect_stroke(thumb_rect, Rounding::same(thumb_h_rad), stroke));
            }

            // Paint optional label with smooth contrast glide on focus
            if let Some(ref lg) = label_galley {
                let label_pos = pos2(
                    rect.left() + track_size.x + 8.0,
                    rect.center().y - lg.size().y * 0.5,
                );
                let final_label_color = if hover_factor > 0.01 {
                    if let Some(p) = self.palette {
                        lerp_color(label_color, p.text, hover_factor.clamp(0.0, 1.0))
                    } else {
                        label_color
                    }
                } else {
                    label_color
                };
                painter.galley(label_pos, lg.clone(), final_label_color);
            }
        }

        SwitchResponse {
            response,
            is_pressed,
            clicked: is_clicked,
        }
    }
}

/// Helper function to interpolate between two `Color32` values.
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}
