//! Theme-aware surface containers and interactive cards.
//!
//! Provides [`Card`] and [`Surface`] with optional spring focus lift, title/subtitle slots,
//! customizable padding, and palette token styling.
//!
//! # State Ownership
//!
//! Interactive focus lift is stored in ID temporary storage or an app-owned [`CardState`] (`CODING_RULES §2`).

use egui::{
    vec2, Color32, Id, Response, Rounding, Stroke, TextStyle, Ui,
    Vec2, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Persistent animation state for container cards with focus and press elevation springs.
#[derive(Clone, Debug)]
pub struct CardState {
    /// Spring driving focus elevation lift ($0.0 \to 1.0$).
    pub focus_spring: Spring,
    /// Spring driving press compression and 3D depression ($0.0 \to 1.0$).
    pub press_spring: Spring,
}

impl Default for CardState {
    fn default() -> Self {
        Self {
            focus_spring: Spring::new(0.0, SpringParams::new(22.0, 0.48)),
            press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
        }
    }
}

impl CardState {
    /// Updates focus lift and press springs and requests repaint if moving.
    pub fn update(
        &mut self,
        dt: f32,
        is_focused: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        self.focus_spring.set_target(if is_focused { 1.0 } else { 0.0 });
        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.press_spring.velocity = (self.press_spring.velocity + 12.0).min(18.0);
            self.press_spring.set_target(0.0);
        } else {
            self.press_spring.set_target(0.0);
        }

        self.focus_spring.update(dt);
        self.press_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled.
    pub fn is_settled(&self) -> bool {
        self.focus_spring.is_settled() && self.press_spring.is_settled()
    }
}

/// A theme-aware container card with optional spring focus elevation.
///
/// # Example
/// ```no_run
/// use egui_widgets::Card;
///
/// # egui::__run_test_ui(|ui| {
/// Card::new()
///     .title("System Metrics")
///     .subtitle("Live CPU & Memory utilization")
///     .interactive(true)
///     .show(ui, |ui| {
///         ui.label("CPU: 14% | RAM: 3.2 GB");
///     });
/// # });
/// ```
pub struct Card<'a> {
    title: Option<WidgetText>,
    subtitle: Option<WidgetText>,
    title_color: Option<Color32>,
    subtitle_color: Option<Color32>,
    fill: Option<Color32>,
    highlight_fill: Option<Color32>,
    stroke: Option<Stroke>,
    highlight_stroke: Option<Stroke>,
    rounding: Option<Rounding>,
    padding: Vec2,
    min_size: Vec2,
    interactive: bool,
    focused: bool,
    triggered: bool,
    pressed: bool,
    focus_lift: f32,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut CardState>,
}

impl<'a> Default for Card<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Card<'a> {
    /// Creates a new container card.
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            title_color: None,
            subtitle_color: None,
            fill: None,
            highlight_fill: None,
            stroke: None,
            highlight_stroke: None,
            rounding: None,
            padding: vec2(14.0, 14.0),
            min_size: Vec2::ZERO,
            focus_lift: 4.0,
            interactive: false,
            focused: false,
            triggered: false,
            pressed: false,
            spring_params: SpringParams::new(22.0, 0.48),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
        }
    }

    /// Explicitly marks the card as pressed / held down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers a click action this frame.
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Sets an optional card title header.
    pub fn title(mut self, title: impl Into<WidgetText>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets an optional card subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Explicitly sets title text color.
    pub fn title_color(mut self, color: Color32) -> Self {
        self.title_color = Some(color);
        self
    }

    /// Explicitly sets subtitle text color.
    pub fn subtitle_color(mut self, color: Color32) -> Self {
        self.subtitle_color = Some(color);
        self
    }

    /// Explicitly overrides card background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicitly overrides card highlighted fill.
    pub fn highlight_fill(mut self, fill: Color32) -> Self {
        self.highlight_fill = Some(fill);
        self
    }

    /// Explicitly overrides card border stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides card highlighted border stroke.
    pub fn highlight_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.highlight_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides card corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Sets inner margin padding around card contents.
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets minimum allowable card dimensions.
    pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
        self.min_size = min_size.into();
        self
    }

    /// Enables or disables interactive focus lift effects.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets keyboard focus / active state for this card container.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the vertical pixel distance for focus lift (default `3.0pt`).
    pub fn focus_lift(mut self, lift: f32) -> Self {
        self.focus_lift = lift;
        self
    }

    /// Configures physical spring dynamics parameters for focus elevation.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Binds an external [`CardState`] struct.
    pub fn with_state(mut self, state: &'a mut CardState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the card container and executes `add_contents` inside.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        // Measure headers
        let title_galley = self.title.as_ref().map(|t| {
            t.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Heading,
            )
        });

        let subtitle_galley = self.subtitle.as_ref().map(|s| {
            s.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Body,
            )
        });

        let (bg_fill, bg_stroke, _highlight_stroke, title_col, subtitle_col) =
            if let Some(p) = self.palette {
                (
                    self.fill.unwrap_or(p.mantle),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.surface0)),
                    self.highlight_stroke.unwrap_or(Stroke::new(1.5, p.accent)),
                    self.title_color.unwrap_or(p.text),
                    self.subtitle_color.unwrap_or(p.subtext0),
                )
            } else {
                let v = ui.visuals();
                (
                    self.fill.unwrap_or(v.widgets.noninteractive.bg_fill),
                    self.stroke.unwrap_or(v.widgets.noninteractive.bg_stroke),
                    self.highlight_stroke.unwrap_or(Stroke::new(1.5, v.selection.stroke.color)),
                    self.title_color.unwrap_or(v.widgets.noninteractive.fg_stroke.color),
                    self.subtitle_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                )
            };

        let rounding = self.rounding.unwrap_or(Rounding::same(10.0));
        let padding = self.padding;

        let id = self.id_source.unwrap_or_else(|| {
            if let Some(ref t) = self.title {
                ui.make_persistent_id(t.text())
            } else {
                ui.next_auto_id()
            }
        });

        // Read previous frame's focus animation value for this frame's rendering
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let prev_focus_val: f32 = if self.motion {
            ui.data_mut(|d| {
                d.get_temp::<f32>(id.with("focus_val")).unwrap_or(0.0)
            })
        } else {
            0.0
        };

        let current_stroke = bg_stroke;
        let t = prev_focus_val.clamp(0.0, 1.0);
        let current_fill = if self.motion && t > 0.001 {
            bg_fill.linear_multiply(1.0 + 0.08 * t)
        } else {
            bg_fill
        };

        // Spring-animated soft ambient drop shadow (clean bidirectional fade in & out)
        let shadow = if self.motion && t > 0.001 {
            egui::epaint::Shadow {
                offset: vec2(0.0, 2.0 + 4.0 * t),
                blur: 8.0 + 14.0 * t,
                spread: 1.5 + 3.0 * t,
                color: Color32::from_black_alpha((55.0 * t) as u8),
            }
        } else {
            egui::epaint::Shadow::NONE
        };

        let prepared_frame = egui::Frame::none()
            .fill(current_fill)
            .stroke(current_stroke)
            .rounding(rounding)
            .shadow(shadow)
            .inner_margin(padding);

        let mut inner_ret: Option<R> = None;
        let frame_response = prepared_frame.show(ui, |ui| {
            ui.set_min_size(self.min_size);

            // Render headers
            if let Some(ref tg) = title_galley {
                ui.painter().galley(ui.cursor().min, tg.clone(), title_col);
                ui.add_space(tg.size().y + 2.0);
            }
            if let Some(ref sg) = subtitle_galley {
                ui.painter().galley(ui.cursor().min, sg.clone(), subtitle_col);
                ui.add_space(sg.size().y + 8.0);
            }
            if title_galley.is_some() || subtitle_galley.is_some() {
                ui.add_space(4.0);
            }

            inner_ret = Some(add_contents(ui));
        });

        let response = frame_response.response;

        // Active state is driven strictly by focus (highlight selection)
        let is_active = self.focused;
        let (is_key_down, is_key_released) = if self.focused {
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

        let is_pressed = self.pressed || response.is_pointer_button_down_on() || is_key_down;
        let is_clicked = response.clicked() || is_key_released || self.triggered;

        // Update animation state using active focus state (animates both IN and OUT)
        if self.motion {
            if let Some(state) = self.external_state {
                state.update(dt, is_active, is_pressed, is_clicked, ui.ctx());
                let val = state.focus_spring.value();
                ui.data_mut(|d| d.insert_temp(id.with("focus_val"), val));
            } else {
                let mut state: CardState = ui.data_mut(|d| {
                    d.get_temp(id).unwrap_or_else(|| CardState {
                        focus_spring: Spring::new(0.0, self.spring_params),
                        press_spring: Spring::new(0.0, SpringParams::new(20.0, 0.45)),
                    })
                });
                state.focus_spring.params = self.spring_params;
                state.update(dt, is_active, is_pressed, is_clicked, ui.ctx());
                let val = state.focus_spring.value();
                ui.data_mut(|d| {
                    d.insert_temp(id, state);
                    d.insert_temp(id.with("focus_val"), val);
                });
            }
        }

        let ret = inner_ret.expect("Card closure should execute");
        (response, ret)
    }
}
