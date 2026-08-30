//! Spring-animated checkbox and radio button controls.
//!
//! Provides [`Checkbox`] with spring-animated checkmark scale pop, and [`RadioButton`]
//! with animated inner dot expansion and palette token synchronization.
//!
//! # State Ownership
//!
//! Animation springs are managed in ID storage or caller-supplied state (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Color32, Id, Rect, Rounding, Sense, Shape, Stroke, TextStyle, Ui,
    WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Response returned by [`Checkbox::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`clicked`, `is_pressed`, `is_held`, `changed`).
#[derive(Clone, Debug)]
pub struct CheckboxResponse {
    /// The underlying [`egui::Response`].
    pub response: egui::Response,
    /// Whether the checkbox is currently pressed / held down (mouse or keyboard).
    pub is_pressed: bool,
    /// Standard click action: fired strictly on release.
    pub clicked: bool,
}

impl CheckboxResponse {
    /// Returns `true` if the checkbox was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` while the checkbox is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` while the checkbox is currently held down (alias for `is_pressed`).
    #[inline]
    pub fn is_held(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if the checkbox checked state changed this frame.
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

impl std::ops::Deref for CheckboxResponse {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl std::ops::DerefMut for CheckboxResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl From<CheckboxResponse> for egui::Response {
    #[inline]
    fn from(r: CheckboxResponse) -> Self {
        r.response
    }
}

/// Persistent animation state for spring-driven checkboxes.
#[derive(Clone, Debug)]
pub struct CheckboxState {
    /// Spring driving checkmark scale pop ($0.0 \to 1.0$).
    pub check_spring: Spring,
    /// Spring driving box squash and depression when pressed ($0.0 \to 1.0$).
    pub press_spring: Spring,
    /// Spring driving instantaneous organic expand-and-shrink focus jiggle ($0.0 \to 0.0$).
    pub focus_spring: Spring,
    /// Tracks previous frame's focus state to detect focus entrance.
    pub was_focused: bool,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            check_spring: Spring::new(0.0, SpringParams::new(26.0, 0.42)),
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
            focus_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            was_focused: false,
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
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
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

    /// Updates internal springs and requests repaint if moving.
    pub fn update(
        &mut self,
        dt: f32,
        checked: bool,
        is_focused: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if is_focused && !self.was_focused {
            self.trigger_focus_bounce();
        }
        self.was_focused = is_focused;

        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.press_spring.velocity = (self.press_spring.velocity + 14.0).min(20.0);
            self.press_spring.set_target(0.0);
        } else {
            self.press_spring.set_target(0.0);
        }

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
        self.press_spring.update(dt);
        self.focus_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled.
    pub fn is_settled(&self) -> bool {
        self.check_spring.is_settled() && self.press_spring.is_settled() && self.focus_spring.is_settled()
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
    focused: bool,
    triggered: bool,
    pressed: bool,
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
            focused: false,
            triggered: false,
            pressed: false,
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

    /// Explicitly marks the checkbox as focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly marks the checkbox as pressed / held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers a toggle action this frame (e.g. from Enter/Space/F key release).
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the checkbox and updates `checked` upon click.
    pub fn show(self, ui: &mut Ui) -> CheckboxResponse {
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
            *self.checked = !*self.checked;
            response.mark_changed();
        }

        // Motion physics
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let is_checked = *self.checked;

        let (check_scale, press_factor, focus_bounce) = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, is_checked, is_focused, is_pressed, is_clicked, ui.ctx());
                (state.check_spring.value(), state.press_spring.value(), state.focus_spring.value())
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

                state.update(dt, is_checked, is_focused, is_pressed, is_clicked, ui.ctx());
                let val = (state.check_spring.value(), state.press_spring.value(), state.focus_spring.value());
                ui.data_mut(|d| d.insert_temp(id, state));
                val
            }
        } else {
            (
                if is_checked { 1.0 } else { 0.0 },
                if is_pressed { 1.0 } else { 0.0 },
                0.0,
            )
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

            // Uniform size scaling: bouncy focus pulse & press compression (exact 1:1 square)
            let scale = (1.0 + (focus_bounce * 0.22) - (press_factor * 0.10)).max(0.60);
            let y_sink = press_factor * 1.5;
            let unscaled_box = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - self.box_size * 0.5 + y_sink),
                vec2(self.box_size, self.box_size),
            );
            let box_rect = Rect::from_center_size(
                unscaled_box.center(),
                vec2(self.box_size * scale, self.box_size * scale),
            );
            let rounding = self.rounding.unwrap_or(Rounding::same(4.0 * scale));

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
                let check_factor = scale * check_scale.clamp(0.0, 1.3);

                let p1 = center + vec2(-4.5, 0.0) * check_factor;
                let p2 = center + vec2(-1.0, 3.5) * check_factor;
                let p3 = center + vec2(4.5, -3.5) * check_factor;

                let alpha = (t * 255.0).clamp(0.0, 255.0) as u8;
                let check_col_alpha = Color32::from_rgba_premultiplied(
                    ((check_col.r() as u16 * alpha as u16) / 255) as u8,
                    ((check_col.g() as u16 * alpha as u16) / 255) as u8,
                    ((check_col.b() as u16 * alpha as u16) / 255) as u8,
                    alpha,
                );

                let stroke = Stroke::new(2.0 * check_factor.min(1.0), check_col_alpha);
                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
            }

            // Label
            if let Some(ref lg) = label_galley {
                let label_pos = pos2(
                    rect.left() + self.box_size + 8.0,
                    rect.center().y - lg.size().y * 0.5,
                );
                painter.galley(label_pos, lg.clone(), label_col);
            }
        }

        CheckboxResponse {
            response,
            is_pressed,
            clicked: is_clicked,
        }
    }
}

/// Response returned by [`RadioButton::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`clicked`, `is_pressed`, `is_held`, `changed`).
#[derive(Clone, Debug)]
pub struct RadioResponse {
    /// The underlying [`egui::Response`].
    pub response: egui::Response,
    /// Whether the radio button is currently pressed / held down (mouse or keyboard).
    pub is_pressed: bool,
    /// Standard click action: fired strictly on release.
    pub clicked: bool,
}

impl RadioResponse {
    /// Returns `true` if the radio button was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` while the radio button is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` while the radio button is currently held down (alias for `is_pressed`).
    #[inline]
    pub fn is_held(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if the radio selection changed this frame.
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

impl std::ops::Deref for RadioResponse {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl std::ops::DerefMut for RadioResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl From<RadioResponse> for egui::Response {
    #[inline]
    fn from(r: RadioResponse) -> Self {
        r.response
    }
}

/// Persistent animation state for spring-driven radio buttons.
#[derive(Clone, Debug)]
pub struct RadioState {
    /// Spring driving inner dot scale pop ($0.0 \to 1.0$).
    pub dot_spring: Spring,
    /// Spring driving outer ring squash and depression when pressed ($0.0 \to 1.0$).
    pub press_spring: Spring,
    /// Spring driving instantaneous organic expand-and-shrink focus jiggle ($0.0 \to 0.0$).
    pub focus_spring: Spring,
    /// Tracks previous frame's focus state to detect focus entrance.
    pub was_focused: bool,
    /// Tracks previous frame's selection state to prevent re-animating already-active buttons.
    pub was_selected: bool,
}

impl Default for RadioState {
    fn default() -> Self {
        Self {
            dot_spring: Spring::new(0.0, SpringParams::new(26.0, 0.44)),
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
            focus_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            was_focused: false,
            was_selected: false,
        }
    }
}

impl RadioState {
    /// Creates a state initialized to the given selected status.
    pub fn new(selected: bool) -> Self {
        Self {
            dot_spring: Spring::new(
                if selected { 1.0 } else { 0.0 },
                SpringParams::new(26.0, 0.44),
            ),
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
            focus_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            was_focused: false,
            was_selected: selected,
        }
    }

    /// Triggers an immediate smooth focus jiggle impulse.
    pub fn trigger_focus_bounce(&mut self) {
        self.focus_spring.current = 0.0;
        self.focus_spring.velocity = 20.0;
        self.focus_spring.set_target(0.0);
    }

    /// Updates internal springs and requests repaint if moving.
    pub fn update(
        &mut self,
        dt: f32,
        is_selected: bool,
        is_focused: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if is_focused && !self.was_focused {
            self.trigger_focus_bounce();
        }
        self.was_focused = is_focused;

        // Press spring: squashes down on press, bounces on release
        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.press_spring.velocity = (self.press_spring.velocity + 14.0).min(20.0);
            self.press_spring.set_target(0.0);
        } else {
            self.press_spring.set_target(0.0);
        }

        // Dot selection spring:
        // ONLY fire the explosive entrance pop if this radio button was NOT already selected!
        if is_selected {
            self.dot_spring.params = SpringParams::new(26.0, 0.44);
            if clicked && !self.was_selected {
                // Newly selected: energetic spring pop with overshoot
                self.dot_spring.current = 0.0;
                self.dot_spring.velocity = 20.0;
            }
            self.dot_spring.set_target(1.0);
        } else {
            // Deselected: clean snappy collapse without bounce
            self.dot_spring.params = SpringParams::new(32.0, 0.85);
            self.dot_spring.set_target(0.0);
        }

        self.was_selected = is_selected;

        self.dot_spring.update(dt);
        self.press_spring.update(dt);
        self.focus_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled.
    pub fn is_settled(&self) -> bool {
        self.dot_spring.is_settled() && self.press_spring.is_settled() && self.focus_spring.is_settled()
    }
}

enum RadioTarget<'a, T> {
    Exact(&'a mut T),
    Optional(&'a mut Option<T>, bool),
}

/// A theme-aware, spring-animated radio button widget.
pub struct RadioButton<'a, T: PartialEq + Clone> {
    value: T,
    target: RadioTarget<'a, T>,
    label: Option<WidgetText>,
    radius: f32,
    fill_active: Option<Color32>,
    stroke_active: Option<Stroke>,
    label_color: Option<Color32>,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut RadioState>,
    focused: bool,
    triggered: bool,
    pressed: bool,
}

impl<'a, T: PartialEq + Clone> RadioButton<'a, T> {
    /// Creates a new radio button for the given option value with standard non-nullable semantics.
    pub fn new(value: T, selected: &'a mut T) -> Self {
        Self {
            value,
            target: RadioTarget::Exact(selected),
            label: None,
            radius: 8.5,
            fill_active: None,
            stroke_active: None,
            label_color: None,
            palette: None,
            id_source: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
        }
    }

    /// Creates a new nullable / optional radio button where clicking an already-selected
    /// option deselects it to `None`.
    pub fn nullable(value: T, selected: &'a mut Option<T>) -> Self {
        Self {
            value,
            target: RadioTarget::Optional(selected, true),
            label: None,
            radius: 8.5,
            fill_active: None,
            stroke_active: None,
            label_color: None,
            palette: None,
            id_source: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
        }
    }

    /// Alias for [`RadioButton::nullable`].
    pub fn optional(value: T, selected: &'a mut Option<T>) -> Self {
        Self::nullable(value, selected)
    }

    /// Configures whether clicking an already-selected optional radio button deselects it to `None`
    /// (default `true` when created via `nullable`/`optional`).
    pub fn allow_deselect(mut self, allow: bool) -> Self {
        if let RadioTarget::Optional(_, ref mut allow_deselect) = self.target {
            *allow_deselect = allow;
        }
        self
    }

    /// Sets an optional ID source for persistent state tracking.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Binds an external [`RadioState`] struct.
    pub fn with_state(mut self, state: &'a mut RadioState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Explicitly marks the radio button as focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly marks the radio button as pressed / held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers a selection action this frame (e.g. from Enter/Space/F key release).
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
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
    pub fn show(self, ui: &mut Ui) -> RadioResponse {
        let mut is_selected = match &self.target {
            RadioTarget::Exact(sel) => self.value == **sel,
            RadioTarget::Optional(sel, _) => sel.as_ref() == Some(&self.value),
        };
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
            match self.target {
                RadioTarget::Exact(sel) => {
                    if !is_selected {
                        *sel = self.value.clone();
                        is_selected = true;
                        response.mark_changed();
                    }
                }
                RadioTarget::Optional(sel, allow_deselect) => {
                    if is_selected {
                        if allow_deselect {
                            *sel = None;
                            is_selected = false;
                            response.mark_changed();
                        }
                    } else {
                        *sel = Some(self.value.clone());
                        is_selected = true;
                        response.mark_changed();
                    }
                }
            }
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

        // Motion physics for radio dot pop and press compression
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        let (dot_scale, press_factor, focus_bounce) = {
            if let Some(state) = self.external_state {
                state.update(dt, is_selected, is_focused, is_pressed, is_clicked, ui.ctx());
                (state.dot_spring.value(), state.press_spring.value(), state.focus_spring.value())
            } else {
                let id = self.id_source.unwrap_or_else(|| {
                    if let Some(ref l) = self.label {
                        ui.make_persistent_id(l.text())
                    } else {
                        response.id
                    }
                });
                let mut state: RadioState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| RadioState::new(is_selected))
                });

                state.update(dt, is_selected, is_focused, is_pressed, is_clicked, ui.ctx());
                let vals = (state.dot_spring.value(), state.press_spring.value(), state.focus_spring.value());
                ui.data_mut(|d| d.insert_temp(id, state));
                vals
            }
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let y_sink = press_factor * 1.5;
            let center = pos2(rect.left() + self.radius, rect.center().y + y_sink);
            // Uniform size scaling: bouncy focus pulse & press compression (exact circle)
            let scale = (1.0 + (focus_bounce * 0.22) - (press_factor * 0.10)).max(0.60);
            let rad = self.radius * scale;

            // Outer ring with smooth continuous stroke interpolation
            let t = dot_scale.clamp(0.0, 1.0);
            let base_stroke = Stroke::new(1.0, if let Some(p) = self.palette { p.surface2 } else { Color32::from_gray(120) });
            let stroke_color = lerp_color(base_stroke.color, active_stroke.color, t);
            let stroke_width = base_stroke.width + (active_stroke.width - base_stroke.width) * t;
            let ring_stroke = Stroke::new(stroke_width, stroke_color);
            painter.add(Shape::circle_stroke(center, rad, ring_stroke));

            // Inner dot when active with spring pop
            if dot_scale > 0.02 {
                let dot_rad = (rad * 0.55 * dot_scale.clamp(0.0, 1.3)).max(0.0);
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

        RadioResponse {
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
