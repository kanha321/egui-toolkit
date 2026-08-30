//! Spring-animated, theme-aware button controls.
//!
//! Provides [`Button`] with spring-driven press feedback, smooth focus
//! luminance morphing, multiple semantic variants, and full developer customization.
//!
//! # State Ownership
//!
//! Interactive animations can use either automatic ID-scoped memory or an explicit caller-owned
//! [`ButtonState`] struct passed via [`.with_state()`](Button::with_state) (`CODING_RULES §2`).

use egui::{
    vec2, Color32, Id, Rect, Rounding, Sense, Shape, Stroke, TextStyle,
    Ui, Vec2, WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Visual styling variants for buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Primary high-contrast button using palette accent.
    #[default]
    Primary,
    /// Subtle secondary button using palette surface layers.
    Secondary,
    /// Borderless ghost button with soft highlight background.
    Ghost,
    /// Negative / destructive action button using palette danger.
    Danger,
    /// Outlined button with transparent fill and colored stroke.
    Outline,
    /// Positive / confirmation action button using palette success.
    Success,
    /// Warning / caution action button using palette warning.
    Warning,
}

/// Standard button sizing presets.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ButtonSize {
    /// Compact height (24pt) with smaller padding and font.
    Small,
    /// Standard height (32pt) suitable for primary UI workflows.
    #[default]
    Medium,
    /// Large prominent height (40pt) for dialogs and call-to-actions.
    Large,
    /// Developer-defined custom padding and font size.
    Custom { padding: Vec2, font_size: f32 },
}

impl ButtonSize {
    /// Returns the (padding, font_size) tuple for this size preset.
    pub fn metrics(&self) -> (Vec2, f32) {
        match self {
            Self::Small => (vec2(8.0, 4.0), 12.0),
            Self::Medium => (vec2(14.0, 7.0), 14.0),
            Self::Large => (vec2(20.0, 10.0), 16.0),
            Self::Custom { padding, font_size } => (*padding, *font_size),
        }
    }
}

/// Response returned by [`Button::show`] providing standard egui interaction methods
/// alongside fine-grained lifecycle querying (`clicked`, `is_pressed`, `is_held`).
#[derive(Clone, Debug)]
pub struct ButtonResponse {
    /// The underlying [`egui::Response`].
    pub response: egui::Response,
    /// Whether the button is currently pressed down (mouse or keyboard).
    pub is_pressed: bool,
    /// Standard click action: fired strictly on release.
    pub clicked: bool,
}

impl ButtonResponse {
    /// Returns `true` if the button was clicked on release.
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` while the button is currently pressed down.
    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` while the button is currently held down (alias for `is_pressed`).
    #[inline]
    pub fn is_held(&self) -> bool {
        self.is_pressed
    }

    /// Unwraps and returns the inner [`egui::Response`].
    #[inline]
    pub fn into_inner(self) -> egui::Response {
        self.response
    }
}

impl std::ops::Deref for ButtonResponse {
    type Target = egui::Response;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl std::ops::DerefMut for ButtonResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl From<ButtonResponse> for egui::Response {
    #[inline]
    fn from(r: ButtonResponse) -> Self {
        r.response
    }
}

/// Persistent animation state for spring-driven button interactions.
///
/// Can be owned directly by the caller or stored in egui ID temporary storage.
#[derive(Clone, Debug)]
pub struct ButtonState {
    /// Spring driving press compression and bounce rebound ($0.0 \to 1.0 \to 0.0$).
    pub press_spring: Spring,
    /// Spring driving smooth focus glow/luminance transition ($0.0 \to 1.0$).
    pub focus_spring: Spring,
    /// Spring driving instantaneous organic expand-and-shrink focus bounce ($0.0 \to 0.0$).
    pub focus_bounce_spring: Spring,
    /// Spring driving smooth width size transitions when text/content changes.
    pub width_spring: Spring,
    /// Spring driving smooth height size transitions when content changes.
    pub height_spring: Spring,
    /// Tracks previous frame's focus state to detect entrance.
    pub was_focused: bool,
    /// Whether size springs have been initialized with the initial content size.
    pub size_initialized: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            press_spring: Spring::new(0.0, SpringParams::new(14.0, 0.42)),
            focus_spring: Spring::new(0.0, SpringParams::new(20.0, 0.55)),
            focus_bounce_spring: Spring::new(0.0, SpringParams::new(18.0, 0.30)),
            width_spring: Spring::new(0.0, SpringParams::new(26.0, 0.58)),
            height_spring: Spring::new(0.0, SpringParams::new(26.0, 0.58)),
            was_focused: false,
            size_initialized: false,
        }
    }
}

impl ButtonState {
    /// Creates a new default button state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggers an elastic squash-and-rebound pop animation on click release.
    pub fn trigger_click(&mut self) {
        self.press_spring.velocity = 18.0;
        self.press_spring.set_target(0.0);
    }

    /// Triggers an instantaneous size bounce impulse on focus arrival.
    pub fn trigger_focus_bounce(&mut self) {
        self.focus_bounce_spring.current = 0.0;
        self.focus_bounce_spring.velocity = 20.0;
        self.focus_bounce_spring.set_target(0.0);
    }

    /// Updates the button's internal springs and requests repaint if still moving.
    pub fn update(
        &mut self,
        dt: f32,
        is_focused: bool,
        is_pressed: bool,
        clicked: bool,
        ctx: &egui::Context,
    ) {
        if is_focused && !self.was_focused {
            self.trigger_focus_bounce();
        }
        self.was_focused = is_focused;

        self.focus_spring.set_target(if is_focused { 1.0 } else { 0.0 });

        if is_pressed {
            self.press_spring.set_target(1.0);
        } else if clicked {
            self.trigger_click();
        } else {
            self.press_spring.set_target(0.0);
        }

        self.focus_spring.update(dt);
        self.focus_bounce_spring.update(dt);
        self.press_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all motion springs have settled within tolerance.
    pub fn is_settled(&self) -> bool {
        self.focus_spring.is_settled()
            && self.focus_bounce_spring.is_settled()
            && self.press_spring.is_settled()
            && self.width_spring.is_settled()
            && self.height_spring.is_settled()
    }
}

/// A theme-aware, spring-animated interactive button widget.
///
/// # Example
/// ```no_run
/// use egui_widgets::Button;
///
/// # egui::__run_test_ui(|ui| {
/// if Button::new("Confirm Action").primary().icon("✓").show(ui).clicked() {
///     println!("Action confirmed!");
/// }
/// # });
/// ```
pub struct Button<'a> {
    text: WidgetText,
    icon: Option<WidgetText>,
    shortcut: Option<WidgetText>,
    badge: Option<WidgetText>,
    variant: ButtonVariant,
    size: ButtonSize,
    fill: Option<Color32>,
    highlight_fill: Option<Color32>,
    active_fill: Option<Color32>,
    stroke: Option<Stroke>,
    highlight_stroke: Option<Stroke>,
    active_stroke: Option<Stroke>,
    text_color: Option<Color32>,
    rounding: Option<Rounding>,
    padding: Option<Vec2>,
    min_size: Option<Vec2>,
    spring_params: SpringParams,
    motion: bool,
    palette: Option<&'a ThemePalette>,
    id_source: Option<Id>,
    external_state: Option<&'a mut ButtonState>,
    focused: bool,
    triggered: bool,
    pressed: bool,
}

impl<'a> Button<'a> {
    /// Creates a new button with the specified text label.
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            shortcut: None,
            badge: None,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            fill: None,
            highlight_fill: None,
            active_fill: None,
            stroke: None,
            highlight_stroke: None,
            active_stroke: None,
            text_color: None,
            rounding: None,
            padding: None,
            min_size: None,
            spring_params: SpringParams::new(14.0, 0.42),
            motion: true,
            palette: None,
            id_source: None,
            external_state: None,
            focused: false,
            triggered: false,
            pressed: false,
        }
    }

    /// Sets whether the button is explicitly focused (e.g. via keyboard navigation or selection).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Explicitly marks the button as physically pressed down.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Explicitly triggers an immediate click impulse on this frame (e.g. via keyboard action).
    pub fn triggered(mut self, triggered: bool) -> Self {
        self.triggered = triggered;
        self
    }

    /// Sets an optional leading icon (text, emoji, or icon font symbol).
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets an optional keyboard shortcut hint displayed on the right.
    pub fn shortcut(mut self, shortcut: impl Into<WidgetText>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Sets an optional small badge chip displayed on the button.
    pub fn badge(mut self, badge: impl Into<WidgetText>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Sets the visual styling variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shortcut for [`ButtonVariant::Primary`].
    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }

    /// Shortcut for [`ButtonVariant::Secondary`].
    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }

    /// Shortcut for [`ButtonVariant::Ghost`].
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }

    /// Shortcut for [`ButtonVariant::Danger`].
    pub fn danger(self) -> Self {
        self.variant(ButtonVariant::Danger)
    }

    /// Shortcut for [`ButtonVariant::Outline`].
    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }

    /// Shortcut for [`ButtonVariant::Success`].
    pub fn success(self) -> Self {
        self.variant(ButtonVariant::Success)
    }

    /// Shortcut for [`ButtonVariant::Warning`].
    pub fn warning(self) -> Self {
        self.variant(ButtonVariant::Warning)
    }

    /// Sets the sizing preset.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Shortcut for [`ButtonSize::Small`].
    pub fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }

    /// Shortcut for [`ButtonSize::Medium`].
    pub fn medium(self) -> Self {
        self.size(ButtonSize::Medium)
    }

    /// Shortcut for [`ButtonSize::Large`].
    pub fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }

    /// Explicitly overrides the background fill color.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicitly overrides the highlighted background fill color.
    pub fn highlight_fill(mut self, fill: Color32) -> Self {
        self.highlight_fill = Some(fill);
        self
    }

    /// Explicitly overrides the active/pressed background fill color.
    pub fn active_fill(mut self, fill: Color32) -> Self {
        self.active_fill = Some(fill);
        self
    }

    /// Explicitly overrides the border stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the highlighted border stroke.
    pub fn highlight_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.highlight_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the active border stroke.
    pub fn active_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.active_stroke = Some(stroke.into());
        self
    }

    /// Explicitly overrides the text label color.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Explicitly overrides the corner rounding.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Explicitly overrides the inner margin padding.
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    /// Sets the minimum allowable dimensions for this button.
    pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
        self.min_size = Some(min_size.into());
        self
    }

    /// Sets custom spring physics parameters for press/focus animations.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }

    /// Enables or disables spring motion animations (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Explicitly provides an active [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Binds an external [`ButtonState`] instance.
    pub fn with_state(mut self, state: &'a mut ButtonState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Sets an explicit ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Renders the button and returns a [`ButtonResponse`].
    pub fn show(mut self, ui: &mut Ui) -> ButtonResponse {
        let (default_padding, _font_size) = self.size.metrics();
        let padding = self.padding.unwrap_or(default_padding);

        let text_layout = self.text.clone().into_galley(
            ui,
            Some(false),
            f32::INFINITY,
            TextStyle::Button,
        );

        let icon_layout = self.icon.as_ref().map(|icon| {
            icon.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Button,
            )
        });

        let shortcut_layout = self.shortcut.as_ref().map(|sc| {
            sc.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Small,
            )
        });

        let badge_layout = self.badge.as_ref().map(|bdg| {
            bdg.clone().into_galley(
                ui,
                Some(false),
                f32::INFINITY,
                TextStyle::Small,
            )
        });

        // Compute content width & height
        let mut content_width = text_layout.size().x;
        let mut content_height = text_layout.size().y;

        if let Some(ref icon) = icon_layout {
            content_width += icon.size().x + 6.0;
            content_height = content_height.max(icon.size().y);
        }
        if let Some(ref sc) = shortcut_layout {
            content_width += sc.size().x + 12.0;
            content_height = content_height.max(sc.size().y);
        }
        if let Some(ref bdg) = badge_layout {
            content_width += bdg.size().x + 10.0;
            content_height = content_height.max(bdg.size().y);
        }

        let desired_size = vec2(
            content_width + padding.x * 2.0,
            content_height + padding.y * 2.0,
        );
        let min_size = self.min_size.unwrap_or(Vec2::ZERO);
        let target_size = desired_size.max(min_size);

        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let id = self.id_source.unwrap_or_else(|| ui.next_auto_id());

        let mut temp_state = if self.external_state.is_none() {
            Some(ui.data_mut(|d| d.get_temp::<ButtonState>(id).unwrap_or_default()))
        } else {
            None
        };

        let state: &mut ButtonState = if let Some(ref mut ext) = self.external_state {
            ext
        } else {
            temp_state.as_mut().unwrap()
        };

        if self.motion {
            if !state.size_initialized {
                state.width_spring.reset(target_size.x);
                state.height_spring.reset(target_size.y);
                state.size_initialized = true;
            } else {
                state.width_spring.set_target(target_size.x);
                state.height_spring.set_target(target_size.y);
                state.width_spring.update(dt);
                state.height_spring.update(dt);
            }
        }

        let allocated_size = if self.motion {
            vec2(state.width_spring.value().max(4.0), state.height_spring.value().max(4.0))
        } else {
            target_size
        };

        let (rect, response) = ui.allocate_exact_size(allocated_size, Sense::click());

        // Resolve colors from ThemePalette or ui.visuals()
        let (base_fill, highlight_fill, active_fill, base_stroke, text_color) =
            if let Some(p) = self.palette {
                match self.variant {
                    ButtonVariant::Primary => (
                        self.fill.unwrap_or(p.accent),
                        self.highlight_fill.unwrap_or(p.accent.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(p.accent.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_accent),
                    ),
                    ButtonVariant::Secondary => (
                        self.fill.unwrap_or(p.surface0),
                        self.highlight_fill.unwrap_or(p.surface1),
                        self.active_fill.unwrap_or(p.surface2),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                        self.text_color.unwrap_or(p.text),
                    ),
                    ButtonVariant::Ghost => (
                        self.fill.unwrap_or(Color32::TRANSPARENT),
                        self.highlight_fill.unwrap_or(p.surface0),
                        self.active_fill.unwrap_or(p.surface1),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.text),
                    ),
                    ButtonVariant::Danger => (
                        self.fill.unwrap_or(p.danger),
                        self.highlight_fill.unwrap_or(p.danger.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(p.danger.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_danger),
                    ),
                    ButtonVariant::Outline => (
                        self.fill.unwrap_or(Color32::TRANSPARENT),
                        self.highlight_fill.unwrap_or(p.accent.linear_multiply(0.15)),
                        self.active_fill.unwrap_or(p.accent.linear_multiply(0.25)),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.accent)),
                        self.text_color.unwrap_or(p.accent),
                    ),
                    ButtonVariant::Success => (
                        self.fill.unwrap_or(p.success),
                        self.highlight_fill.unwrap_or(p.success.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(p.success.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_success),
                    ),
                    ButtonVariant::Warning => (
                        self.fill.unwrap_or(p.warning),
                        self.highlight_fill.unwrap_or(p.warning.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(p.warning.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_warning),
                    ),
                }
            } else {
                let v = ui.visuals();
                match self.variant {
                    ButtonVariant::Primary => (
                        self.fill.unwrap_or(v.selection.bg_fill),
                        self.highlight_fill.unwrap_or(v.widgets.hovered.bg_fill),
                        self.active_fill.unwrap_or(v.widgets.active.bg_fill),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(v.selection.stroke.color),
                    ),
                    ButtonVariant::Secondary => (
                        self.fill.unwrap_or(v.widgets.inactive.bg_fill),
                        self.highlight_fill.unwrap_or(v.widgets.hovered.bg_fill),
                        self.active_fill.unwrap_or(v.widgets.active.bg_fill),
                        self.stroke.unwrap_or(v.widgets.inactive.bg_stroke),
                        self.text_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                    ),
                    ButtonVariant::Ghost => (
                        self.fill.unwrap_or(Color32::TRANSPARENT),
                        self.highlight_fill.unwrap_or(v.widgets.hovered.bg_fill.linear_multiply(0.5)),
                        self.active_fill.unwrap_or(v.widgets.active.bg_fill.linear_multiply(0.5)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(v.widgets.inactive.fg_stroke.color),
                    ),
                    ButtonVariant::Danger => (
                        self.fill.unwrap_or(v.error_fg_color),
                        self.highlight_fill.unwrap_or(v.error_fg_color.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(v.error_fg_color.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(Color32::WHITE),
                    ),
                    ButtonVariant::Outline => (
                        self.fill.unwrap_or(Color32::TRANSPARENT),
                        self.highlight_fill.unwrap_or(v.widgets.hovered.bg_fill.linear_multiply(0.3)),
                        self.active_fill.unwrap_or(v.widgets.active.bg_fill.linear_multiply(0.3)),
                        self.stroke.unwrap_or(Stroke::new(1.0, v.selection.stroke.color)),
                        self.text_color.unwrap_or(v.selection.stroke.color),
                    ),
                    ButtonVariant::Success => (
                        self.fill.unwrap_or(Color32::from_rgb(40, 167, 69)),
                        self.highlight_fill.unwrap_or(Color32::from_rgb(50, 190, 80)),
                        self.active_fill.unwrap_or(Color32::from_rgb(30, 140, 55)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(Color32::WHITE),
                    ),
                    ButtonVariant::Warning => (
                        self.fill.unwrap_or(v.warn_fg_color),
                        self.highlight_fill.unwrap_or(v.warn_fg_color.linear_multiply(1.15)),
                        self.active_fill.unwrap_or(v.warn_fg_color.linear_multiply(0.85)),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(Color32::BLACK),
                    ),
                }
            };

        let rounding = self.rounding.unwrap_or(Rounding::same(6.0));

        // Motion physics handling & Keyboard Event Detection
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

        let (focus_factor, focus_bounce, press_factor) = if self.motion {
            state.press_spring.params = self.spring_params;
            state.update(dt, is_focused, is_pressed, is_clicked, ui.ctx());
            (
                state.focus_spring.value(),
                state.focus_bounce_spring.value(),
                state.press_spring.value(),
            )
        } else {
            (
                if is_focused { 1.0 } else { 0.0 },
                0.0,
                if is_pressed { 1.0 } else { 0.0 },
            )
        };

        if let Some(st) = temp_state {
            ui.data_mut(|d| d.insert_temp(id, st));
        }

        // Scale geometry with bouncy focus pulse, press compression, 3D vertical sink, & release pop overshoot
        let scale = (1.0 + (focus_bounce * 0.12) - (press_factor * 0.14)).max(0.65);
        let y_offset = press_factor * 4.0;
        let center = rect.center() + vec2(0.0, y_offset);
        let animated_rect = Rect::from_center_size(
            center,
            vec2(rect.width() * scale, rect.height() * scale),
        );
        let animated_rounding = rounding * scale;

        // Interpolate fill and stroke colors
        let current_fill = if press_factor > 0.05 {
            lerp_color(highlight_fill, active_fill, press_factor.clamp(0.0, 1.0))
        } else {
            lerp_color(base_fill, highlight_fill, focus_factor.clamp(0.0, 1.0))
        };

        let current_stroke = if press_factor > 0.05 {
            self.active_stroke.unwrap_or_else(|| {
                Stroke::new(base_stroke.width + 0.5, base_stroke.color)
            })
        } else if focus_factor > 0.05 {
            self.highlight_stroke.unwrap_or(base_stroke)
        } else {
            base_stroke
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background surface
            painter.add(Shape::rect_filled(animated_rect, animated_rounding, current_fill));

            // Border stroke
            if current_stroke.width > 0.0 && current_stroke.color != Color32::TRANSPARENT {
                painter.add(Shape::rect_stroke(animated_rect, animated_rounding, current_stroke));
            }

            // Draw content (Icon + Text + Badge + Shortcut)
            let has_right_items = shortcut_layout.is_some() || badge_layout.is_some();
            let icon_spacing = 6.0;
            let icon_w = icon_layout.as_ref().map(|i| i.size().x + icon_spacing).unwrap_or(0.0);
            let main_content_width = icon_w + text_layout.size().x;

            let mut cursor_x = if has_right_items {
                animated_rect.left() + padding.x
            } else {
                animated_rect.center().x - main_content_width * 0.5
            };
            let content_center_y = animated_rect.center().y;

            if let Some(ref icon) = icon_layout {
                let icon_pos = egui::pos2(
                    cursor_x,
                    content_center_y - icon.size().y * 0.5,
                );
                painter.galley(icon_pos, icon.clone(), text_color);
                cursor_x += icon_w;
            }

            let text_pos = egui::pos2(
                cursor_x,
                content_center_y - text_layout.size().y * 0.5,
            );
            painter.galley(text_pos, text_layout, text_color);

            // Right-aligned elements
            let mut right_cursor_x = animated_rect.right() - padding.x;

            if let Some(ref sc) = shortcut_layout {
                right_cursor_x -= sc.size().x;
                let sc_pos = egui::pos2(
                    right_cursor_x,
                    content_center_y - sc.size().y * 0.5,
                );
                let sc_color = text_color.linear_multiply(0.6);
                painter.galley(sc_pos, sc.clone(), sc_color);
                right_cursor_x -= 8.0;
            }

            if let Some(ref bdg) = badge_layout {
                right_cursor_x -= bdg.size().x + 6.0;
                let badge_rect = Rect::from_min_size(
                    egui::pos2(right_cursor_x, content_center_y - bdg.size().y * 0.5 - 2.0),
                    vec2(bdg.size().x + 6.0, bdg.size().y + 4.0),
                );
                let badge_bg = text_color.linear_multiply(0.15);
                painter.rect_filled(badge_rect, Rounding::same(4.0), badge_bg);
                painter.galley(
                    badge_rect.min + vec2(3.0, 2.0),
                    bdg.clone(),
                    text_color,
                );
            }
        }

        ButtonResponse {
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
