//! Spring-animated text inputs and search bars.
//!
//! Provides [`TextInput`] and [`SearchBar`] with animated focus glow rings, icon prefixes,
//! clear buttons, and theme palette synchronization.
//!
//! # State Ownership
//!
//! Focus animations are tracked in ID temporary storage or an app-owned [`InputState`] (`CODING_RULES §2`).

use egui::{
    pos2, vec2, Align2, Color32, FontId, Id, Rect, Response, Rounding, Sense, Shape, Stroke,
    TextEdit, TextStyle, Ui, Vec2, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Persistent animation state for text input focus effects.
#[derive(Clone, Debug)]
pub struct InputState {
    /// Spring driving focus glow ring expansion ($0.0 \to 1.0$).
    pub focus_spring: Spring,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focus_spring: Spring::new(0.0, SpringParams::new(26.0, 0.46)),
        }
    }
}

impl InputState {
    /// Updates focus spring and requests repaint if moving.
    pub fn update(&mut self, dt: f32, is_focused: bool, ctx: &egui::Context) {
        if is_focused && self.focus_spring.target < 0.5 {
            self.focus_spring.velocity = 18.0;
        }
        self.focus_spring.set_target(if is_focused { 1.0 } else { 0.0 });
        self.focus_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if focus spring has settled.
    pub fn is_settled(&self) -> bool {
        self.focus_spring.is_settled()
    }
}

/// A theme-aware text input widget with spring-animated focus rings.
///
/// # Example
/// ```no_run
/// use egui_widgets::TextInput;
///
/// # egui::__run_test_ui(|ui| {
/// let mut query = String::new();
/// TextInput::new(&mut query)
///     .placeholder("Search packages...")
///     .icon("🔍")
///     .clear_button(true)
///     .show(ui);
/// # });
/// ```
pub struct TextInput<'a> {
    text: &'a mut String,
    placeholder: Option<&'a str>,
    icon: Option<WidgetText>,
    clear_button: bool,
    password: bool,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    focus_stroke: Option<Stroke>,
    text_color: Option<Color32>,
    rounding: Option<Rounding>,
    padding: Vec2,
    min_width: f32,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut InputState>,
}

impl<'a> TextInput<'a> {
    /// Creates a new text input bound to the given string buffer.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            icon: None,
            clear_button: true,
            password: false,
            fill: None,
            stroke: None,
            focus_stroke: None,
            text_color: None,
            rounding: None,
            padding: vec2(10.0, 6.0),
            min_width: 140.0,
            spring_params: SpringParams::new(26.0, 0.46),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
        }
    }

    /// Sets placeholder hint text.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Sets an optional leading icon (e.g. search magnifying glass).
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Enables or disables the quick clear (`✖`) button (default `true`).
    pub fn clear_button(mut self, clear: bool) -> Self {
        self.clear_button = clear;
        self
    }

    /// Enables password masking mode.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Explicitly overrides background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicitly overrides unfocused border stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides focused border stroke.
    pub fn focus_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.focus_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides text color.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Explicitly overrides corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Sets inner margin padding (default `vec2(10.0, 6.0)`).
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets minimum input width (default `140.0pt`).
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Sets custom spring physics parameters for the focus ring.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Enables or disables spring motion animations (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Binds an external [`InputState`] struct.
    pub fn with_state(mut self, state: &'a mut InputState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the text input into the UI.
    pub fn show(self, ui: &mut Ui) -> Response {
        let height = 32.0;
        let available_w = ui.available_width().max(self.min_width);
        let desired_size = vec2(available_w, height);

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        // Resolve colors
        let (bg_fill, base_stroke, focus_stroke, text_color, placeholder_color) =
            if let Some(p) = self.palette {
                (
                    self.fill.unwrap_or(p.surface0),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.focus_stroke.unwrap_or(Stroke::new(1.5, p.accent)),
                    self.text_color.unwrap_or(p.text),
                    p.subtext1,
                )
            } else {
                let v = ui.visuals();
                (
                    self.fill.unwrap_or(v.extreme_bg_color),
                    self.stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                    self.focus_stroke.unwrap_or(Stroke::new(1.5, v.selection.stroke.color)),
                    self.text_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                    v.widgets.noninteractive.fg_stroke.color.linear_multiply(0.5),
                )
            };

        let rounding = self.rounding.unwrap_or(Rounding::same(6.0));

        // Sub-layout for internal TextEdit
        let mut left_offset = self.padding.x;
        let mut right_offset = self.padding.x;

        if self.icon.is_some() {
            left_offset += 18.0;
        }
        if self.clear_button && !self.text.is_empty() {
            right_offset += 20.0;
        }

        let edit_rect = Rect::from_min_max(
            pos2(rect.left() + left_offset, rect.top() + self.padding.y),
            pos2(rect.right() - right_offset, rect.bottom() - self.padding.y),
        );

        let mut edit = TextEdit::singleline(self.text)
            .password(self.password)
            .text_color(text_color)
            .frame(false);

        if let Some(ph) = self.placeholder {
            edit = edit.hint_text(WidgetText::from(ph).color(placeholder_color));
        }

        let edit_response = ui.put(edit_rect, edit);
        let has_focus = edit_response.has_focus();

        // Release text edit focus on Escape, Enter, or Ctrl navigation shortcuts
        if has_focus && ui.input(|i| {
            i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Enter) || (i.modifiers.ctrl && (
                i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::L)
                || i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowRight)
            ))
        }) {
            edit_response.surrender_focus();
        }

        if edit_response.changed() {
            response.mark_changed();
        }

        // Motion physics for focus glow ring
        let dt = ui.input(|i| i.stable_dt).min(0.05);

        let focus_factor = if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, has_focus, ui.ctx());
                state.focus_spring.value()
            } else {
                let id = self.id_source.unwrap_or_else(|| {
                    if let Some(p) = self.placeholder {
                        ui.make_persistent_id(p)
                    } else {
                        response.id
                    }
                });
                let mut state: InputState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_default()
                });

                state.update(dt, has_focus, ui.ctx());
                let val = state.focus_spring.value();
                ui.data_mut(|d| d.insert_temp(id, state));
                val
            }
        } else {
            if has_focus { 1.0 } else { 0.0 }
        };

        // Quick clear button interaction
        let mut clear_clicked = false;
        let mut clear_hovered = false;
        let clear_rect = if self.clear_button && !self.text.is_empty() {
            let r = Rect::from_center_size(
                pos2(rect.right() - 14.0, rect.center().y),
                vec2(16.0, 16.0),
            );
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if r.contains(pos) {
                    clear_hovered = true;
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        clear_clicked = true;
                    }
                }
            }
            Some(r)
        } else {
            None
        };

        if clear_clicked {
            self.text.clear();
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.add(Shape::rect_filled(rect, rounding, bg_fill));

            // Outer expanding focus glow ring
            if focus_factor > 0.01 {
                let expand = (focus_factor * 2.5).max(0.0);
                let glow_rect = rect.expand(expand);
                let glow_stroke = Stroke::new(
                    1.5,
                    focus_stroke.color.linear_multiply(0.40 * focus_factor.clamp(0.0, 1.0)),
                );
                painter.add(Shape::rect_stroke(glow_rect, Rounding::same(rounding.nw + expand), glow_stroke));
            }

            // Animated border stroke
            let current_stroke = if focus_factor > 0.01 {
                let color = lerp_color(base_stroke.color, focus_stroke.color, focus_factor.clamp(0.0, 1.0));
                let width = base_stroke.width + (focus_stroke.width - base_stroke.width) * focus_factor.clamp(0.0, 1.0);
                Stroke::new(width, color)
            } else {
                base_stroke
            };

            if current_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(rect, rounding, current_stroke));
            }

            // Leading icon
            if let Some(ref icon) = self.icon {
                let icon_galley = icon.clone().into_galley(
                    ui,
                    Some(false),
                    f32::INFINITY,
                    TextStyle::Button,
                );
                let icon_pos = pos2(
                    rect.left() + self.padding.x,
                    rect.center().y - icon_galley.size().y * 0.5,
                );
                let icon_color = if has_focus {
                    focus_stroke.color
                } else {
                    placeholder_color
                };
                painter.galley(icon_pos, icon_galley, icon_color);
            }

            // Quick clear button render
            if let Some(c_rect) = clear_rect {
                let clear_color = if clear_hovered {
                    text_color
                } else {
                    placeholder_color
                };
                painter.text(
                    c_rect.center(),
                    Align2::CENTER_CENTER,
                    "✖",
                    FontId::monospace(10.0),
                    clear_color,
                );
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
