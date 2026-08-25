//! Spring-animated numeric slider controls.
//!
//! Provides [`Slider`] with spring-scaling knob animations on hover/drag, active track highlights,
//! numeric badges, and full palette integration.
//!
//! # State Ownership
//!
//! Knob scale animations are tracked in ID storage or an app-owned [`SliderState`] (`CODING_RULES §2`).

use egui::{
    emath::Numeric, pos2, vec2, Color32, Id, Rect, Response, Rounding, Sense, Shape,
    Stroke, TextStyle, Ui, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};
use std::ops::RangeInclusive;

/// Persistent animation state for interactive sliders.
#[derive(Clone, Debug)]
pub struct SliderState {
    /// Spring driving knob radius scale multiplier ($0.0 \to 1.0$).
    pub knob_scale_spring: Spring,
    /// Spring driving visual knob position along the track ($0.0 \to 1.0$).
    pub position_spring: Spring,
    /// Whether the initial position has been set.
    pub initialized: bool,
}

/// Fixed internal knob hover/scale spring — fast pop, not user-facing.
const KNOB_SCALE_PARAMS: SpringParams = SpringParams { angular_frequency: 28.0, damping_ratio: 0.50 };

impl Default for SliderState {
    fn default() -> Self {
        Self {
            knob_scale_spring: Spring::new(0.0, KNOB_SCALE_PARAMS),
            position_spring: Spring::new(0.0, SpringParams::new(28.1, 0.64)),
            initialized: false,
        }
    }
}

impl SliderState {
    /// Updates the slider springs and requests repaint if moving.
    pub fn update(
        &mut self,
        dt: f32,
        target_normalized: f32,
        is_hovered: bool,
        is_dragged: bool,
        is_clicked: bool,
        pos_params: SpringParams,
        click_momentum: f32,
        ctx: &egui::Context,
    ) {
        if !self.initialized {
            self.position_spring = Spring::new(target_normalized, pos_params);
            self.knob_scale_spring = Spring::new(0.0, KNOB_SCALE_PARAMS);
            self.initialized = true;
            return;
        }

        self.position_spring.params = pos_params;

        if is_dragged {
            // When directly dragging, follow cursor with instant 1:1 precision
            self.position_spring.current = target_normalized;
            self.position_spring.target = target_normalized;
            self.position_spring.velocity = 0.0;
            self.knob_scale_spring.set_target(1.0);
        } else if is_clicked {
            // When clicked on track, glide towards clicked target with optional momentum kick
            if click_momentum > 0.0 {
                let dist = target_normalized - self.position_spring.current;
                self.position_spring.velocity += dist * click_momentum;
            }
            self.position_spring.set_target(target_normalized);
            self.knob_scale_spring.velocity = 12.0;
            self.knob_scale_spring.set_target(if is_hovered { 1.0 } else { 0.0 });
        } else {
            // Smoothly glide towards target if value changed
            self.position_spring.set_target(target_normalized);
            self.knob_scale_spring.set_target(if is_hovered { 1.0 } else { 0.0 });
        }

        self.position_spring.update(dt);
        self.knob_scale_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled.
    pub fn is_settled(&self) -> bool {
        self.knob_scale_spring.is_settled() && self.position_spring.is_settled()
    }
}

/// A theme-aware, spring-animated numeric slider widget.
///
/// # Example
/// ```no_run
/// use egui_widgets::Slider;
///
/// # egui::__run_test_ui(|ui| {
/// let mut volume = 75.0f32;
/// Slider::new(&mut volume, 0.0..=100.0)
///     .label("Master Volume")
///     .suffix("%")
///     .show(ui);
/// # });
/// ```
pub struct Slider<'a, T: Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    step: Option<f64>,
    label: Option<WidgetText>,
    show_value: bool,
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
    track_height: f32,
    knob_radius: f32,
    track_active: Option<Color32>,
    track_inactive: Option<Color32>,
    knob_fill: Option<Color32>,
    knob_stroke: Option<Stroke>,
    label_color: Option<Color32>,
    /// The single spring parameter controlling position glide animation.
    spring_params: SpringParams,
    click_momentum: f32,
    knob_scale_mult: f32,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut SliderState>,
}

impl<'a, T: Numeric> Slider<'a, T> {
    /// Creates a new slider bound to the given numeric value and range.
    pub fn new(value: &'a mut T, range: RangeInclusive<T>) -> Self {
        Self {
            value,
            range,
            step: None,
            label: None,
            show_value: true,
            prefix: None,
            suffix: None,
            track_height: 6.0,
            knob_radius: 8.0,
            track_active: None,
            track_inactive: None,
            knob_fill: None,
            knob_stroke: None,
            label_color: None,
            spring_params: SpringParams::new(28.1, 0.64),
            click_momentum: 0.0,
            knob_scale_mult: 0.25,
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
        }
    }

    /// Sets an optional step increment.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets an optional text label above the slider.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Controls whether the numeric value text is displayed.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Sets an optional text prefix (e.g. `$`).
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Sets an optional text suffix (e.g. `ms`, `%`, `px`).
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = Some(suffix);
        self
    }

    /// Sets the height of the slider track (default `6.0pt`).
    pub fn track_height(mut self, height: f32) -> Self {
        self.track_height = height;
        self
    }

    /// Sets the base radius of the thumb knob (default `8.0pt`).
    pub fn knob_radius(mut self, radius: f32) -> Self {
        self.knob_radius = radius;
        self
    }

    /// Explicitly overrides the active (filled) track color.
    pub fn track_active(mut self, color: Color32) -> Self {
        self.track_active = Some(color);
        self
    }

    /// Explicitly overrides the inactive track color.
    pub fn track_inactive(mut self, color: Color32) -> Self {
        self.track_inactive = Some(color);
        self
    }

    /// Explicitly overrides the thumb knob fill color.
    pub fn knob_fill(mut self, color: Color32) -> Self {
        self.knob_fill = Some(color);
        self
    }

    /// Explicitly overrides the thumb knob border stroke.
    pub fn knob_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.knob_stroke = Some(stroke.into());
        self
    }

    /// Sets spring physics parameters controlling the position glide animation.
    ///
    /// This is the **one unified spring** that drives how the slider knob travels
    /// to its target position on click. Higher `omega_0` = faster, lower `zeta` = bouncier.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Sets the momentum impulse multiplier when clicked on track (default `0.0`).
    pub fn click_momentum(mut self, momentum: f32) -> Self {
        self.click_momentum = momentum;
        self
    }

    /// Sets the knob scale expansion multiplier on hover/drag (default `0.25`).
    pub fn knob_scale_mult(mut self, mult: f32) -> Self {
        self.knob_scale_mult = mult;
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

    /// Binds an external [`SliderState`] struct.
    pub fn with_state(mut self, state: &'a mut SliderState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the slider and updates `value` upon interaction.
    pub fn show(self, ui: &mut Ui) -> Response {
        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let value_text = if self.show_value {
            let mut text = String::new();
            if let Some(p) = self.prefix {
                text.push_str(p);
            }
            text.push_str(&format!("{:.1}", self.value.to_f64()));
            if let Some(s) = self.suffix {
                text.push_str(s);
            }
            Some(
                WidgetText::from(text).into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Small,
                ),
            )
        } else {
            None
        };

        let mut total_height = self.track_height.max(self.knob_radius * 2.0);
        if label_galley.is_some() || value_text.is_some() {
            total_height += 20.0;
        }

        let desired_size = vec2(ui.available_width().max(120.0), total_height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // Range math
        let min_val = self.range.start().to_f64();
        let max_val = self.range.end().to_f64();
        let val_span = (max_val - min_val).max(1e-6);

        let mut top_y = rect.top();
        if label_galley.is_some() || value_text.is_some() {
            top_y += 18.0;
        }

        let track_y = top_y + self.knob_radius;
        let track_left = rect.left() + self.knob_radius;
        let track_right = rect.right() - self.knob_radius;
        let track_width = (track_right - track_left).max(1.0);

        // Interaction
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let normalized = ((pos.x - track_left) / track_width).clamp(0.0, 1.0) as f64;
                let mut new_val = min_val + normalized * val_span;

                if let Some(step) = self.step {
                    new_val = (new_val / step).round() * step;
                }
                new_val = new_val.clamp(min_val, max_val);

                *self.value = T::from_f64(new_val);
                response.mark_changed();
            }
        }

        // Motion physics
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let is_interacting = response.hovered() || response.dragged();

        let pos_params = self.spring_params;
        let click_momentum = self.click_momentum;
        let knob_scale_mult = self.knob_scale_mult;

        let current_val_normalized =
            ((self.value.to_f64() - min_val) / val_span).clamp(0.0, 1.0) as f32;

        let (knob_scale_factor, visual_progress) = if self.motion {
            if let Some(state) = self.external_state {
                state.update(
                    dt,
                    current_val_normalized,
                    response.hovered(),
                    response.dragged(),
                    response.clicked(),
                    pos_params,
                    click_momentum,
                    ui.ctx(),
                );
                (state.knob_scale_spring.value(), state.position_spring.value())
            } else {
                let id = self.id_source.unwrap_or_else(|| {
                    if let Some(ref l) = self.label {
                        ui.make_persistent_id(l.text())
                    } else {
                        response.id
                    }
                });
                let mut state: SliderState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| {
                        let mut s = SliderState::default();
                        s.position_spring = Spring::new(current_val_normalized, pos_params);
                        s.knob_scale_spring = Spring::new(0.0, KNOB_SCALE_PARAMS);
                        s.initialized = true;
                        s
                    })
                });

                state.update(
                    dt,
                    current_val_normalized,
                    response.hovered(),
                    response.dragged(),
                    response.clicked(),
                    pos_params,
                    click_momentum,
                    ui.ctx(),
                );
                let values = (state.knob_scale_spring.value(), state.position_spring.value());
                ui.data_mut(|d| d.insert_temp(id, state));
                values
            }
        } else {
            (if is_interacting { 1.0 } else { 0.0 }, current_val_normalized)
        };

        let knob_x = track_left + visual_progress.clamp(0.0, 1.0) * track_width;
        let knob_center = pos2(knob_x, track_y);

        // Resolve colors
        let (track_act, track_inact, knob_fill, knob_stroke, label_col) =
            if let Some(p) = self.palette {
                (
                    self.track_active.unwrap_or(p.accent),
                    self.track_inactive.unwrap_or(p.surface1),
                    self.knob_fill.unwrap_or(if p.dark { p.crust } else { Color32::WHITE }),
                    self.knob_stroke.unwrap_or(Stroke::new(2.0, p.accent)),
                    self.label_color.unwrap_or(p.text),
                )
            } else {
                let v = ui.visuals();
                (
                    self.track_active.unwrap_or(v.selection.bg_fill),
                    self.track_inactive.unwrap_or(v.widgets.inactive.bg_fill),
                    self.knob_fill.unwrap_or(v.widgets.active.fg_stroke.color),
                    self.knob_stroke.unwrap_or(Stroke::new(2.0, v.selection.stroke.color)),
                    self.label_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                )
            };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Render labels row if present
            if label_galley.is_some() || value_text.is_some() {
                if let Some(ref lg) = label_galley {
                    painter.galley(pos2(rect.left(), rect.top()), lg.clone(), label_col);
                }
                if let Some(ref vt) = value_text {
                    painter.galley(
                        pos2(rect.right() - vt.size().x, rect.top()),
                        vt.clone(),
                        label_col.linear_multiply(0.8),
                    );
                }
            }

            // Inactive track
            let track_rect = Rect::from_min_max(
                pos2(track_left, track_y - self.track_height * 0.5),
                pos2(track_right, track_y + self.track_height * 0.5),
            );
            let track_rounding = Rounding::same(self.track_height * 0.5);
            painter.add(Shape::rect_filled(track_rect, track_rounding, track_inact));

            // Active track
            if knob_x > track_left {
                let active_track_rect = Rect::from_min_max(
                    pos2(track_left, track_y - self.track_height * 0.5),
                    pos2(knob_x, track_y + self.track_height * 0.5),
                );
                painter.add(Shape::rect_filled(active_track_rect, track_rounding, track_act));
            }

            // Knob with energetic spring scale bounce
            let animated_radius = (self.knob_radius * (1.0 + knob_scale_factor * knob_scale_mult)).max(1.0);
            painter.add(Shape::circle_filled(knob_center, animated_radius, knob_fill));
            if knob_stroke.width > 0.0 {
                painter.add(Shape::circle_stroke(knob_center, animated_radius, knob_stroke));
            }
        }

        response
    }
}
