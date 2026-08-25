//! Spring-animated checkbox and radio button controls.
//!
//! Provides [`Checkbox`] with spring-animated checkmark scale pop, and [`RadioButton`]
//! with animated inner dot expansion and palette token synchronization.
//!
//! # State Ownership
//!
//! Animation springs are managed in ID storage or caller-supplied state (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Color32, Id, Rect, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Persistent animation state for spring-driven checkboxes.
#[derive(Clone, Debug)]
pub struct CheckboxState {
    /// Spring driving checkmark scale pop ($0.0 \to 1.0$).
    pub check_spring: Spring,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            check_spring: Spring::new(0.0, SpringParams::new(26.0, 0.42)),
        }
    }
}

impl CheckboxState {
    /// Creates a state initialized to the given checked status.
    pub fn new(checked: bool) -> Self {
        Self {
            check_spring: Spring::new(
                if checked { 1.0 } else { 0.0 },
                SpringParams::new(26.0, 0.42),
            ),
        }
    }

    /// Updates internal checkmark spring and requests repaint if moving.
    pub fn update(&mut self, dt: f32, checked: bool, clicked: bool, ctx: &egui::Context) {
        if clicked {
            if checked {
                // Checking: energetic spring pop with overshoot
                self.check_spring.params = SpringParams::new(26.0, 0.42);
                self.check_spring.current = 0.0;
                self.check_spring.velocity = 22.0;
                self.check_spring.set_target(1.0);
            } else {
                // Unchecking: clean snappy retraction without reverse bounce/flash
                self.check_spring.params = SpringParams::new(32.0, 0.90);
                self.check_spring.set_target(0.0);
            }
        } else {
            if checked {
                self.check_spring.params = SpringParams::new(26.0, 0.42);
                self.check_spring.set_target(1.0);
            } else {
                self.check_spring.params = SpringParams::new(32.0, 0.90);
                self.check_spring.set_target(0.0);
            }
        }

        self.check_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if checkmark has settled.
    pub fn is_settled(&self) -> bool {
        self.check_spring.is_settled()
    }
}

/// A theme-aware, spring-animated checkbox widget.
///
/// # Example
/// ```no_run
/// use egui_widgets::Checkbox;
///
/// # egui::__run_test_ui(|ui| {
/// let mut remember_me = false;
/// Checkbox::new(&mut remember_me)
///     .label("Remember this device")
///     .show(ui);
/// # });
/// ```
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: Option<WidgetText>,
    box_size: f32,
    rounding: Option<Rounding>,
    fill_checked: Option<Color32>,
    fill_unchecked: Option<Color32>,
    stroke_checked: Option<Stroke>,
    stroke_unchecked: Option<Stroke>,
    checkmark_color: Option<Color32>,
    label_color: Option<Color32>,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut CheckboxState>,
}

impl<'a> Checkbox<'a> {
    /// Creates a new checkbox bound to a boolean reference.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            label: None,
            box_size: 18.0,
            rounding: None,
            fill_checked: None,
            fill_unchecked: None,
            stroke_checked: None,
            stroke_unchecked: None,
            checkmark_color: None,
            label_color: None,
            spring_params: SpringParams::new(26.0, 0.42),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
        }
    }

    /// Sets an optional label text.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the side length of the checkbox square (default `18.0pt`).
    pub fn box_size(mut self, size: f32) -> Self {
        self.box_size = size;
        self
    }

    /// Explicitly overrides checkbox corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Explicitly overrides the checked background fill color.
    pub fn fill_checked(mut self, color: Color32) -> Self {
        self.fill_checked = Some(color);
        self
    }

    /// Explicitly overrides the unchecked background fill color.
    pub fn fill_unchecked(mut self, color: Color32) -> Self {
        self.fill_unchecked = Some(color);
        self
    }

    /// Explicitly overrides the checked border stroke.
    pub fn stroke_checked(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke_checked = Some(stroke.into());
        self
    }

    /// Explicitly overrides the unchecked border stroke.
    pub fn stroke_unchecked(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke_unchecked = Some(stroke.into());
        self
    }

    /// Explicitly overrides the checkmark icon color.
    pub fn checkmark_color(mut self, color: Color32) -> Self {
        self.checkmark_color = Some(color);
        self
    }

    /// Sets custom spring parameters for the checkmark scale bounce.
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

    /// Binds an external [`CheckboxState`] struct.
    pub fn with_state(mut self, state: &'a mut CheckboxState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the checkbox and updates `checked` upon click.
    pub fn show(self, ui: &mut Ui) -> Response {
        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let mut total_size = vec2(self.box_size, self.box_size);
        if let Some(ref lg) = label_galley {
            total_size.x += lg.size().x + 8.0;
            total_size.y = total_size.y.max(lg.size().y);
        }

        let (rect, mut response) = ui.allocate_exact_size(total_size, Sense::click());

        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }

        // Motion physics
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let is_checked = *self.checked;

        let is_clicked = response.clicked();
        let check_scale = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, is_checked, is_clicked, ui.ctx());
                state.check_spring.value()
            } else {
                let id = self.id_source.unwrap_or_else(|| {
                    if let Some(ref l) = self.label {
                        ui.make_persistent_id(l.text())
                    } else {
                        response.id
                    }
                });
                let mut state: CheckboxState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| CheckboxState::new(is_checked))
                });

                state.update(dt, is_checked, is_clicked, ui.ctx());
                let val = state.check_spring.value();
                ui.data_mut(|d| d.insert_temp(id, state));
                val
            }
        } else {
            if is_checked { 1.0 } else { 0.0 }
        };

        // Resolve colors
        let (fill_on, fill_off, stroke_on, stroke_off, check_col, label_col) =
            if let Some(p) = self.palette {
                (
                    self.fill_checked.unwrap_or(p.accent),
                    self.fill_unchecked.unwrap_or(p.surface0),
                    self.stroke_checked.unwrap_or(Stroke::new(1.5, p.accent)),
                    self.stroke_unchecked.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.checkmark_color.unwrap_or(if p.dark { p.crust } else { Color32::WHITE }),
                    self.label_color.unwrap_or(p.text),
                )
            } else {
                let v = ui.visuals();
                (
                    self.fill_checked.unwrap_or(v.selection.bg_fill),
                    self.fill_unchecked.unwrap_or(v.widgets.inactive.bg_fill),
                    self.stroke_checked.unwrap_or(Stroke::new(1.5, v.selection.stroke.color)),
                    self.stroke_unchecked.unwrap_or(v.widgets.inactive.bg_stroke),
                    self.checkmark_color.unwrap_or(v.widgets.active.fg_stroke.color),
                    self.label_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                )
            };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let box_rect = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - self.box_size * 0.5),
                vec2(self.box_size, self.box_size),
            );
            let rounding = self.rounding.unwrap_or(Rounding::same(4.0));

            // Continuous interpolation of background & stroke (no binary threshold jumps)
            let t = check_scale.clamp(0.0, 1.0);
            let current_fill = lerp_color(fill_off, fill_on, t);

            let stroke_color = lerp_color(stroke_off.color, stroke_on.color, t);
            let stroke_width = stroke_off.width + (stroke_on.width - stroke_off.width) * t;
            let current_stroke = Stroke::new(stroke_width, stroke_color);

            painter.add(Shape::rect_filled(box_rect, rounding, current_fill));
            if current_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(box_rect, rounding, current_stroke));
            }

            // Draw checkmark lines with scale bounce and alpha fade
            if check_scale > 0.02 {
                let center = box_rect.center();
                let scale = check_scale.clamp(0.0, 1.3);

                let p1 = center + vec2(-4.5, 0.0) * scale;
                let p2 = center + vec2(-1.0, 3.5) * scale;
                let p3 = center + vec2(4.5, -3.5) * scale;

                let alpha = (t * 255.0).clamp(0.0, 255.0) as u8;
                let check_col_alpha = Color32::from_rgba_premultiplied(
                    ((check_col.r() as u16 * alpha as u16) / 255) as u8,
                    ((check_col.g() as u16 * alpha as u16) / 255) as u8,
                    ((check_col.b() as u16 * alpha as u16) / 255) as u8,
                    alpha,
                );

                let stroke = Stroke::new(2.0 * scale.min(1.0), check_col_alpha);
                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
            }

            // Label
            if let Some(ref lg) = label_galley {
                let label_pos = pos2(
                    box_rect.right() + 8.0,
                    rect.center().y - lg.size().y * 0.5,
                );
                painter.galley(label_pos, lg.clone(), label_col);
            }
        }

        response
    }
}

/// A theme-aware, spring-animated radio button widget.
pub struct RadioButton<'a, T: PartialEq + Clone> {
    value: T,
    selected: &'a mut T,
    label: Option<WidgetText>,
    radius: f32,
    fill_active: Option<Color32>,
    stroke_active: Option<Stroke>,
    label_color: Option<Color32>,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
}

impl<'a, T: PartialEq + Clone> RadioButton<'a, T> {
    /// Creates a new radio button for the given option value.
    pub fn new(value: T, selected: &'a mut T) -> Self {
        Self {
            value,
            selected,
            label: None,
            radius: 8.5,
            fill_active: None,
            stroke_active: None,
            label_color: None,
            palette: None,
            id_source: None,
        }
    }

    /// Sets an optional ID source for persistent state tracking.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Sets an optional label text.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the outer radius of the radio circle (default `8.5pt`).
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Renders the radio button into the UI.
    pub fn show(self, ui: &mut Ui) -> Response {
        let is_selected = self.value == *self.selected;
        let label_galley = self.label.as_ref().map(|l| {
            l.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let mut total_size = vec2(self.radius * 2.0, self.radius * 2.0);
        if let Some(ref lg) = label_galley {
            total_size.x += lg.size().x + 8.0;
            total_size.y = total_size.y.max(lg.size().y);
        }

        let (rect, mut response) = ui.allocate_exact_size(total_size, Sense::click());

        if response.clicked() && !is_selected {
            *self.selected = self.value.clone();
            response.mark_changed();
        }

        // Colors
        let (active_fill, active_stroke, text_color) = if let Some(p) = self.palette {
            (
                self.fill_active.unwrap_or(p.accent),
                self.stroke_active.unwrap_or(Stroke::new(1.5, p.accent)),
                self.label_color.unwrap_or(p.text),
            )
        } else {
            let v = ui.visuals();
            (
                self.fill_active.unwrap_or(v.selection.bg_fill),
                self.stroke_active.unwrap_or(Stroke::new(1.5, v.selection.stroke.color)),
                self.label_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
            )
        };

        // Motion physics for radio dot pop
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let id = self.id_source.unwrap_or_else(|| {
            if let Some(ref l) = self.label {
                ui.make_persistent_id(l.text())
            } else {
                response.id
            }
        });
        let dot_scale = {
            let mut spring: Spring = ui.data_mut(|d| {
                d.get_temp(id).unwrap_or_else(|| {
                    Spring::new(if is_selected { 1.0 } else { 0.0 }, SpringParams::new(28.0, 0.44))
                })
            });
            if is_selected {
                spring.params = SpringParams::new(28.0, 0.44);
                if response.clicked() {
                    spring.current = 0.0;
                    spring.velocity = 22.0;
                }
                spring.set_target(1.0);
            } else {
                spring.params = SpringParams::new(32.0, 0.90);
                spring.set_target(0.0);
            }
            spring.update(dt);
            if !spring.is_settled() {
                ui.ctx().request_repaint();
            }
            let val = spring.value();
            ui.data_mut(|d| d.insert_temp(id, spring));
            val
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = pos2(rect.left() + self.radius, rect.center().y);

            // Outer ring with smooth continuous stroke interpolation
            let t = dot_scale.clamp(0.0, 1.0);
            let base_stroke = Stroke::new(1.0, if let Some(p) = self.palette { p.surface2 } else { Color32::from_gray(120) });
            let stroke_color = lerp_color(base_stroke.color, active_stroke.color, t);
            let stroke_width = base_stroke.width + (active_stroke.width - base_stroke.width) * t;
            let ring_stroke = Stroke::new(stroke_width, stroke_color);
            painter.add(Shape::circle_stroke(center, self.radius, ring_stroke));

            // Inner dot when active with spring pop
            if dot_scale > 0.02 {
                let dot_rad = (self.radius * 0.55 * dot_scale.clamp(0.0, 1.3)).max(0.0);
                painter.add(Shape::circle_filled(center, dot_rad, active_fill));
            }

            // Label
            if let Some(ref lg) = label_galley {
                let label_pos = pos2(
                    rect.left() + self.radius * 2.0 + 8.0,
                    rect.center().y - lg.size().y * 0.5,
                );
                painter.galley(label_pos, lg.clone(), text_color);
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
