//! Spring-animated numeric slider controls.
//!
//! Provides [`Slider`] with spring-scaling knob animations on hover/drag, active track highlights,
//! centered inline geometry, prominent monospace value chips, and inline Vim-enabled direct numeric editing.
//!
//! # State Ownership
//!
//! Knob scale, position animations, and direct text editing are tracked in ID storage or an app-owned
//! [`SliderState`] (`CODING_RULES §2`).

use egui::{
    emath::Numeric, pos2, vec2, Align, Color32, FontId, Id, Layout, Rect, Response, Rounding, Sense, Shape,
    Stroke, TextStyle, Ui, WidgetText,
};
use egui_themes::ThemePalette;
use egui_vim_nav::{VimBufferState, VimMode};
use spring_core::{Spring, SpringParams};
use std::ops::RangeInclusive;

use crate::input::{InputState, TextInput};

/// Layout orientation mode for [`Slider`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SliderLayout {
    /// Single horizontal row: label (optional) + track + prominent value badge, all centered on `center.y`.
    #[default]
    Inline,
    /// Two-row stacked: Row 1 has label and value badge; Row 2 has full-width centered track.
    Stacked,
}

/// Persistent animation and interaction state for interactive sliders.
#[derive(Clone, Debug)]
pub struct SliderState {
    /// Spring driving knob radius scale multiplier ($0.0 \to 1.0$).
    pub knob_scale_spring: Spring,
    /// Spring driving visual knob position along the track ($0.0 \to 1.0$).
    pub position_spring: Spring,
    /// Whether the initial position has been set.
    pub initialized: bool,
    /// Whether direct numeric text editing is actively open.
    pub editing: bool,
    /// Live text buffer being edited in the inline text field.
    pub edit_buffer: String,
    /// Focus spring & animated cursor state for the inline text box.
    pub input_state: InputState,
    /// Dedicated modal Vim buffer state for editing.
    pub vim_buffer: VimBufferState,
}

/// Fixed internal knob hover/scale spring — fast pop, not user-facing.
const KNOB_SCALE_PARAMS: SpringParams = SpringParams { angular_frequency: 28.0, damping_ratio: 0.50 };

impl Default for SliderState {
    fn default() -> Self {
        Self {
            knob_scale_spring: Spring::new(0.0, KNOB_SCALE_PARAMS),
            position_spring: Spring::new(0.0, SpringParams::new(28.1, 0.64)),
            initialized: false,
            editing: false,
            edit_buffer: String::new(),
            input_state: InputState::default(),
            vim_buffer: VimBufferState::new(""),
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

    /// Returns `true` if all motion springs and text-edit animations have settled.
    pub fn is_settled(&self) -> bool {
        self.knob_scale_spring.is_settled()
            && self.position_spring.is_settled()
            && self.input_state.is_settled()
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
    layout: SliderLayout,
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
    focused: bool,
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
            layout: SliderLayout::Inline,
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
            focused: false,
        }
    }

    /// Sets whether this slider currently has keyboard/Vim focus.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the layout orientation ([`SliderLayout::Inline`] or [`SliderLayout::Stacked`]).
    pub fn layout(mut self, layout: SliderLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the layout to [`SliderLayout::Inline`] (single horizontal row with centered track).
    pub fn inline(self) -> Self {
        self.layout(SliderLayout::Inline)
    }

    /// Sets the layout to [`SliderLayout::Stacked`] (label and value on top, track centered below).
    pub fn stacked(self) -> Self {
        self.layout(SliderLayout::Stacked)
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
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        // Numeric string formatting
        let formatted_val = if let Some(s) = self.step {
            if s >= 1.0 && s.fract() == 0.0 {
                format!("{:.0}", self.value.to_f64())
            } else {
                format!("{:.1}", self.value.to_f64())
            }
        } else {
            format!("{:.1}", self.value.to_f64())
        };
        let display_value = format!(
            "{}{}{}",
            self.prefix.unwrap_or(""),
            formatted_val,
            self.suffix.unwrap_or("")
        );

        let value_font = FontId::monospace(12.0);
        let value_galley = ui.painter().layout_no_wrap(
            display_value.clone(),
            value_font.clone(),
            Color32::WHITE,
        );

        let badge_w = (value_galley.size().x + 18.0).max(56.0);
        let badge_h = 22.0;

        let row_height = (self.knob_radius * 2.0 + 12.0).max(28.0);
        let total_height = match self.layout {
            SliderLayout::Inline => row_height,
            SliderLayout::Stacked => row_height + 22.0,
        };

        let desired_size = vec2(ui.available_width().max(120.0), total_height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Range math
        let min_val = self.range.start().to_f64();
        let max_val = self.range.end().to_f64();
        let val_span = (max_val - min_val).max(1e-6);

        let id = self.id_source.unwrap_or_else(|| {
            if let Some(ref l) = self.label {
                ui.make_persistent_id(l.text())
            } else {
                response.id
            }
        });

        // Geometry computation: perfectly centered on center.y
        let (track_left, track_right, track_y, badge_rect) = match self.layout {
            SliderLayout::Inline => {
                let track_y = rect.center().y;
                let track_left = if let Some(ref lg) = label_galley {
                    rect.left() + lg.size().x + 10.0 + self.knob_radius
                } else {
                    rect.left() + self.knob_radius + 4.0
                };
                if self.show_value {
                    let b_rect = Rect::from_center_size(
                        pos2(rect.right() - badge_w * 0.5 - 2.0, track_y),
                        vec2(badge_w, badge_h),
                    );
                    let track_right = b_rect.left() - self.knob_radius - 8.0;
                    (track_left, track_right, track_y, b_rect)
                } else {
                    let track_right = rect.right() - self.knob_radius - 4.0;
                    (track_left, track_right, track_y, Rect::ZERO)
                }
            }
            SliderLayout::Stacked => {
                let top_y = rect.top() + 10.0;
                let track_y = rect.bottom() - self.knob_radius - 6.0;
                let track_left = rect.left() + self.knob_radius + 4.0;
                let track_right = rect.right() - self.knob_radius - 4.0;
                let b_rect = if self.show_value {
                    Rect::from_center_size(
                        pos2(rect.right() - badge_w * 0.5 - 2.0, top_y),
                        vec2(badge_w, badge_h),
                    )
                } else {
                    Rect::ZERO
                };
                (track_left, track_right, track_y, b_rect)
            }
        };

        let track_width = (track_right - track_left).max(10.0);

        // Fetch persistent animation and text editing state
        let mut state: SliderState = if let Some(ref ext) = self.external_state {
            (*ext).clone()
        } else {
            ui.data_mut(|d| {
                d.get_temp(id).unwrap_or_else(|| {
                    let mut s = SliderState::default();
                    let current_val_normalized =
                        ((self.value.to_f64() - min_val) / val_span).clamp(0.0, 1.0) as f32;
                    s.position_spring = Spring::new(current_val_normalized, self.spring_params);
                    s.knob_scale_spring = Spring::new(0.0, KNOB_SCALE_PARAMS);
                    s.initialized = true;
                    s
                })
            })
        };

        // Track interactive click and drag
        let track_hit_rect = Rect::from_min_max(
            pos2(track_left - self.knob_radius, track_y - self.knob_radius - 4.0),
            pos2(track_right + self.knob_radius, track_y + self.knob_radius + 4.0),
        );
        let track_resp = ui.interact(track_hit_rect, id.with("track"), Sense::click_and_drag());

        if !state.editing && (track_resp.clicked() || track_resp.dragged()) {
            if let Some(pos) = track_resp.interact_pointer_pos() {
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
        response = response.union(track_resp);

        // Direct numeric text editing with unified TextInput
        if self.show_value && badge_rect.is_positive() {
            // If focused in Level 1 Vim navigation, 'i' / 'a' enters direct numeric editing
            if self.focused && !state.editing {
                let enter_edit = ui.input(|i| {
                    !i.modifiers.ctrl && !i.modifiers.alt && (
                        i.key_pressed(egui::Key::I) || i.key_pressed(egui::Key::A)
                    )
                });
                if enter_edit {
                    state.editing = true;
                    state.edit_buffer = format!("{:.1}", self.value.to_f64());
                    let mut vbuf = VimBufferState::new(state.edit_buffer.clone());
                    vbuf.cursor = state.edit_buffer.len();
                    vbuf.mode = VimMode::Insert;
                    state.vim_buffer = vbuf;
                }
            }

            // Sync buffer from self.value while NOT actively editing
            if !state.editing {
                state.edit_buffer = format!("{:.1}", self.value.to_f64());
            }

            // Auto-close editing if Level 1 focus is lost (e.g. user moved away via HJKL)
            if state.editing && !self.focused {
                if let Ok(num) = state.edit_buffer.trim().parse::<f64>() {
                    let clamped = num.clamp(min_val, max_val);
                    *self.value = T::from_f64(clamped);
                    response.mark_changed();
                }
                state.editing = false;
                ui.ctx().memory_mut(|m| m.stop_text_input());
            }

            let mut child_ui = ui.child_ui(badge_rect, Layout::left_to_right(Align::Center));
            child_ui.set_clip_rect(ui.clip_rect());
            let stroke_color = if let Some(p) = self.palette {
                p.accent
            } else {
                Color32::from_rgb(180, 190, 254)
            };
            let fill_color = if let Some(p) = self.palette {
                p.crust
            } else {
                Color32::from_gray(25)
            };

            let slider_highlight = rect.expand(3.0);
            let mut input_builder = TextInput::new(&mut state.edit_buffer)
                .mode_indicator(false)
                .clear_button(false)
                .glow_ring(false)
                .padding(vec2(6.0, 2.0))
                .min_width(badge_w)
                .desired_width(badge_w)
                .height(badge_h)
                .rounding(Rounding::same(4.0))
                .fill(fill_color)
                .stroke(Stroke::new(1.0, stroke_color))
                .focus_stroke(Stroke::new(1.0, stroke_color))
                .with_state(&mut state.input_state)
                .vim_buffer(&mut state.vim_buffer)
                .focused(state.editing)
                .spawn_origin(slider_highlight, 6.0)
                .id_source(id.with("val_input"));

            if let Some(p) = self.palette {
                input_builder = input_builder.palette(p);
            }
            let input_resp = input_builder.show(&mut child_ui);

            // Handle click on badge to enter editing
            if !state.editing && input_resp.clicked() {
                state.editing = true;
                state.edit_buffer = format!("{:.1}", self.value.to_f64());
                let mut vbuf = VimBufferState::new(state.edit_buffer.clone());
                vbuf.cursor = state.edit_buffer.len();
                vbuf.mode = VimMode::Insert;
                state.vim_buffer = vbuf;
            }

            // Handle edit completion: Enter, click outside, or Esc in Normal mode
            if state.editing {
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let esc_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let any_click = ui.input(|i| i.pointer.any_click());
                let pointer_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(pos2(-1000.0, -1000.0)));
                let clicked_outside = any_click && !badge_rect.contains(pointer_pos);

                if enter_pressed || clicked_outside || (esc_pressed && state.vim_buffer.mode == VimMode::Normal && state.vim_buffer.parser.pending_keys_label().is_empty()) {
                    let cleaned = state.edit_buffer.trim().replace('\n', "").replace('\r', "");
                    if let Ok(num) = cleaned.parse::<f64>() {
                        let clamped = num.clamp(min_val, max_val);
                        *self.value = T::from_f64(clamped);
                        response.mark_changed();
                    }
                    state.editing = false;
                    ui.ctx().memory_mut(|m| m.stop_text_input());
                }
            }
        }

        // Motion physics
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let pos_params = self.spring_params;
        let click_momentum = self.click_momentum;
        let knob_scale_mult = self.knob_scale_mult;

        let current_val_normalized =
            ((self.value.to_f64() - min_val) / val_span).clamp(0.0, 1.0) as f32;

        let (knob_scale_factor, visual_progress) = if self.motion {
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
            (
                if response.hovered() || response.dragged() { 1.0 } else { 0.0 },
                current_val_normalized,
            )
        };

        // Write state back
        if let Some(ref mut ext) = self.external_state {
            **ext = state;
        } else {
            ui.data_mut(|d| d.insert_temp(id, state));
        }

        let knob_x = track_left + visual_progress.clamp(0.0, 1.0) * track_width;
        let knob_center = pos2(knob_x, track_y);

        // Resolve colors
        let (track_act, track_inact, knob_fill, knob_stroke, label_col) =
            if let Some(p) = self.palette {
                (
                    self.track_active.unwrap_or(p.accent),
                    self.track_inactive.unwrap_or(p.surface0),
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

            // Render label if present
            if let Some(ref lg) = label_galley {
                let label_pos = match self.layout {
                    SliderLayout::Inline => pos2(rect.left(), rect.center().y - lg.size().y * 0.5),
                    SliderLayout::Stacked => pos2(rect.left(), rect.top() + 2.0),
                };
                painter.galley(label_pos, lg.clone(), label_col);
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
