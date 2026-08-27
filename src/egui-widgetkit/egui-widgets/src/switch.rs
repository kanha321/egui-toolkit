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
    pos2, vec2, Color32, Id, Rect, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
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

/// Persistent animation state for spring-driven toggle switches.
#[derive(Clone, Debug)]
pub struct SwitchState {
    /// Spring driving thumb horizontal translation ($0.0 \to 1.0$).
    pub thumb_spring: Spring,
    /// Spring driving track hover expansion / brightness ($0.0 \to 1.0$).
    pub hover_spring: Spring,
    /// Critically damped spring driving smooth track background crossfade ($0.0 \to 1.0$).
    pub color_spring: Spring,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            thumb_spring: Spring::new(0.0, SpringParams::new(26.0, 0.46)),
            hover_spring: Spring::new(0.0, SpringParams::new(24.0, 0.50)),
            color_spring: Spring::new(0.0, SpringParams::new(14.0, 1.0)),
        }
    }
}

impl SwitchState {
    /// Creates a state initialized to the given boolean state.
    pub fn new(is_on: bool) -> Self {
        let initial_val = if is_on { 1.0 } else { 0.0 };
        Self {
            thumb_spring: Spring::new(initial_val, SpringParams::new(26.0, 0.46)),
            hover_spring: Spring::new(0.0, SpringParams::new(24.0, 0.50)),
            color_spring: Spring::new(initial_val, SpringParams::new(14.0, 1.0)),
        }
    }

    /// Updates internal springs and requests repaint if in motion.
    pub fn update(&mut self, dt: f32, is_on: bool, is_hovered: bool, clicked: bool, ctx: &egui::Context) {
        self.hover_spring.set_target(if is_hovered { 1.0 } else { 0.0 });
        self.color_spring.set_target(if is_on { 1.0 } else { 0.0 });
        if clicked {
            self.thumb_spring.velocity = if is_on { 18.0 } else { -18.0 };
            self.thumb_spring.set_target(if is_on { 1.0 } else { 0.0 });
        } else {
            self.thumb_spring.set_target(if is_on { 1.0 } else { 0.0 });
        }

        self.thumb_spring.update(dt);
        self.hover_spring.update(dt);
        self.color_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have reached target equilibrium.
    pub fn is_settled(&self) -> bool {
        self.thumb_spring.is_settled() && self.hover_spring.is_settled() && self.color_spring.is_settled()
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

    /// Explicitly triggers a toggle action this frame (e.g. from Enter/Space/F key).
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
    pub fn show(self, ui: &mut Ui) -> Response {
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

        let is_clicked = response.clicked() || self.triggered;
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
        let is_hovered = self.focused;
        let is_on = *self.selected;

        let (thumb_progress, hover_factor, color_progress) = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, is_on, is_hovered, is_clicked, ui.ctx());
                (
                    state.thumb_spring.value(),
                    state.hover_spring.value(),
                    state.color_spring.value(),
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

                state.update(dt, is_on, is_hovered, is_clicked, ui.ctx());
                let values = (
                    state.thumb_spring.value(),
                    state.hover_spring.value(),
                    state.color_spring.value(),
                );
                ui.data_mut(|d| d.insert_temp(id, state));
                values
            }
        } else {
            (
                if is_on { 1.0 } else { 0.0 },
                if is_hovered { 1.0 } else { 0.0 },
                if is_on { 1.0 } else { 0.0 },
            )
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Track positioning
            let track_rect = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - track_h * 0.5),
                track_size,
            );
            let track_rounding = Rounding::same(track_h * 0.5);

            // Interpolate track fill smoothly via dedicated color crossfade
            let base_track_fill = lerp_color(track_off, track_on, color_progress.clamp(0.0, 1.0));
            let final_track_fill = if hover_factor > 0.01 {
                base_track_fill.linear_multiply(1.0 + hover_factor.clamp(0.0, 1.0) * 0.15)
            } else {
                base_track_fill
            };

            // Paint track
            painter.add(Shape::rect_filled(track_rect, track_rounding, final_track_fill));
            if track_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(track_rect, track_rounding, track_stroke));
            }

            // Thumb calculation with overshoot and elastic dynamic squash & stretch
            let min_x = track_rect.left() + track_h * 0.5;
            let max_x = track_rect.right() - track_h * 0.5;
            let travel_distance = max_x - min_x;
            let thumb_x = min_x + thumb_progress * travel_distance;
            let thumb_center = pos2(thumb_x, track_rect.center().y);

            let thumb_color = lerp_color(thumb_off, thumb_on, color_progress.clamp(0.0, 1.0));

            // Dynamic elastic squash along movement axis
            let target_val = if is_on { 1.0 } else { 0.0 };
            let stretch_factor = (thumb_progress - target_val).abs().min(0.5) * 0.40;
            let thumb_w = thumb_r * (1.0 + stretch_factor);
            let thumb_h_rad = thumb_r * (1.0 - stretch_factor * 0.4);
            let thumb_rect = Rect::from_center_size(thumb_center, vec2(thumb_w * 2.0, thumb_h_rad * 2.0));

            // Paint thumb knob
            painter.add(Shape::rect_filled(thumb_rect, Rounding::same(thumb_h_rad), thumb_color));
            if let Some(stroke) = self.thumb_stroke {
                painter.add(Shape::rect_stroke(thumb_rect, Rounding::same(thumb_h_rad), stroke));
            }

            // Paint optional label
            if let Some(ref lg) = label_galley {
                let label_pos = pos2(
                    track_rect.right() + 8.0,
                    rect.center().y - lg.size().y * 0.5,
                );
                painter.galley(label_pos, lg.clone(), label_color);
            }
        }

        response
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
