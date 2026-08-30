//! Semantic status badges, chips, and tag indicators.
//!
//! Provides [`Badge`] with status color variants, high-contrast [`BadgeStyle`] modes
//! (technical [`BadgeStyle::Outline`] and punchy [`BadgeStyle::Solid`]), optional glowing dots,
//! and theme palette synchronization.

use egui::{
    pos2, vec2, Color32, Id, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui, Vec2,
    WidgetText,
};
use egui_themes::ThemePalette;
use spring_core::{Spring, SpringParams};

/// Persistent animation state for spring-animated badges.
#[derive(Clone, Debug)]
pub struct BadgeState {
    /// Spring driving smooth width size transitions ($0.0 \to \text{target}$).
    pub width_spring: Spring,
    /// Spring driving smooth height size transitions ($0.0 \to \text{target}$).
    pub height_spring: Spring,
    /// Whether size springs have been initialized.
    pub size_initialized: bool,
}

impl Default for BadgeState {
    fn default() -> Self {
        Self {
            width_spring: Spring::new(0.0, SpringParams::new(26.0, 0.58)),
            height_spring: Spring::new(0.0, SpringParams::new(26.0, 0.58)),
            size_initialized: false,
        }
    }
}

impl BadgeState {
    /// Updates size springs towards target dimensions and requests repaint if moving.
    pub fn update(&mut self, dt: f32, target_size: Vec2, ctx: &egui::Context) {
        if !self.size_initialized {
            self.width_spring.reset(target_size.x);
            self.height_spring.reset(target_size.y);
            self.size_initialized = true;
            return;
        }

        self.width_spring.set_target(target_size.x);
        self.height_spring.set_target(target_size.y);
        self.width_spring.update(dt);
        self.height_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all size springs have settled.
    pub fn is_settled(&self) -> bool {
        self.width_spring.is_settled() && self.height_spring.is_settled()
    }
}

/// Semantic color variants for [`Badge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    /// Uses palette primary accent tone.
    #[default]
    Accent,
    /// Positive / Success feedback (e.g. green).
    Success,
    /// Caution / Warning feedback (e.g. yellow).
    Warning,
    /// Destructive / Danger feedback (e.g. red).
    Danger,
    /// Informational notice (e.g. blue).
    Info,
    /// Subtle neutral surface badge.
    Neutral,
    /// Custom fill and text colors.
    Custom { fill: Color32, text: Color32 },
}

/// Visual presentation style for [`Badge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeStyle {
    /// Option A (Default): High-contrast technical outline pill.
    /// Deep dark container fill (`p.crust`), sharp 1px semantic border,
    /// crisp high-contrast text (`p.text`), and a glowing status dot.
    #[default]
    Outline,
    /// Option B: 100% opaque solid semantic fill paired with high-contrast `p.on_*` text.
    Solid,
}

/// A theme-aware status badge or tag pill.
///
/// # Example
/// ```no_run
/// use egui_widgets::{Badge, BadgeStyle, BadgeVariant};
///
/// # egui::__run_test_ui(|ui| {
/// Badge::new("Online")
///     .variant(BadgeVariant::Success)
///     .outline()
///     .dot(true)
///     .show(ui);
/// # });
/// ```
pub struct Badge<'a> {
    text: WidgetText,
    variant: BadgeVariant,
    style: BadgeStyle,
    dot: bool,
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    text_color: Option<Color32>,
    rounding: Option<Rounding>,
    padding: Vec2,
    palette: Option<&'a ThemePalette>,
    motion: bool,
    id_source: Option<Id>,
    external_state: Option<&'a mut BadgeState>,
}

impl<'a> Badge<'a> {
    /// Creates a new badge with the given label text.
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Accent,
            style: BadgeStyle::Outline,
            dot: false,
            fill: None,
            stroke: None,
            text_color: None,
            rounding: None,
            padding: vec2(10.0, 4.5),
            palette: None,
            motion: true,
            id_source: None,
            external_state: None,
        }
    }

    /// Explicitly attaches an app-owned [`BadgeState`] struct.
    pub fn with_state(mut self, state: &'a mut BadgeState) -> Self {
        self.external_state = Some(state);
        self
    }

    /// Provides an explicit unique ID source for state storage.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Enables or disables spring motion animations (default `true`).
    pub fn motion(mut self, motion: bool) -> Self {
        self.motion = motion;
        self
    }

    /// Sets the semantic badge variant.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the visual presentation style ([`BadgeStyle::Outline`] or [`BadgeStyle::Solid`]).
    pub fn style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the badge style to [`BadgeStyle::Outline`] (high-contrast dark container + semantic border + bright text).
    pub fn outline(self) -> Self {
        self.style(BadgeStyle::Outline)
    }

    /// Sets the badge style to [`BadgeStyle::Solid`] (100% opaque semantic fill + `on_*` contrast text).
    pub fn solid(self) -> Self {
        self.style(BadgeStyle::Solid)
    }

    /// Shortcut for [`BadgeVariant::Accent`].
    pub fn accent(self) -> Self {
        self.variant(BadgeVariant::Accent)
    }

    /// Shortcut for [`BadgeVariant::Success`].
    pub fn success(self) -> Self {
        self.variant(BadgeVariant::Success)
    }

    /// Shortcut for [`BadgeVariant::Warning`].
    pub fn warning(self) -> Self {
        self.variant(BadgeVariant::Warning)
    }

    /// Shortcut for [`BadgeVariant::Danger`].
    pub fn danger(self) -> Self {
        self.variant(BadgeVariant::Danger)
    }

    /// Shortcut for [`BadgeVariant::Info`].
    pub fn info(self) -> Self {
        self.variant(BadgeVariant::Info)
    }

    /// Shortcut for [`BadgeVariant::Neutral`].
    pub fn neutral(self) -> Self {
        self.variant(BadgeVariant::Neutral)
    }

    /// Enables an active leading status dot.
    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    /// Explicitly overrides background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Explicitly overrides border stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
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

    /// Sets inner margin padding (default `vec2(8.0, 3.5)`).
    pub fn padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Explicitly attaches a [`ThemePalette`] override.
    pub fn palette(mut self, palette: &'a ThemePalette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Renders the badge into the UI.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let text_galley = self.text.into_galley(
            ui,
            Some(false),
            f32::INFINITY,
            TextStyle::Small,
        );

        let dot_spacing = 15.0;
        let mut content_w = text_galley.size().x;
        if self.dot {
            content_w += dot_spacing;
        }

        let desired_size = vec2(
            content_w + self.padding.x * 2.0,
            (text_galley.size().y + self.padding.y * 2.0).max(22.0),
        );

        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let id = self.id_source.unwrap_or_else(|| ui.next_auto_id());

        let mut temp_state = if self.external_state.is_none() {
            Some(ui.data_mut(|d| d.get_temp::<BadgeState>(id).unwrap_or_default()))
        } else {
            None
        };

        let state: &mut BadgeState = if let Some(ref mut ext) = self.external_state {
            ext
        } else {
            temp_state.as_mut().unwrap()
        };

        if self.motion {
            state.update(dt, desired_size, ui.ctx());
        }

        let allocated_size = if self.motion {
            vec2(state.width_spring.value().max(4.0), state.height_spring.value().max(4.0))
        } else {
            desired_size
        };

        let (rect, response) = ui.allocate_exact_size(allocated_size, Sense::hover());

        if let Some(st) = temp_state {
            ui.data_mut(|d| d.insert_temp(id, st));
        }

        // Resolve colors based on ThemePalette and BadgeStyle
        let (bg_fill, bg_stroke, text_color, dot_color) = if let Some(p) = self.palette {
            match self.style {
                BadgeStyle::Outline => match self.variant {
                    BadgeVariant::Accent => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.accent)),
                        self.text_color.unwrap_or(p.text),
                        p.accent,
                    ),
                    BadgeVariant::Success => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.success)),
                        self.text_color.unwrap_or(p.text),
                        p.success,
                    ),
                    BadgeVariant::Warning => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.warning)),
                        self.text_color.unwrap_or(p.text),
                        p.warning,
                    ),
                    BadgeVariant::Danger => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.danger)),
                        self.text_color.unwrap_or(p.text),
                        p.danger,
                    ),
                    BadgeVariant::Info => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.info)),
                        self.text_color.unwrap_or(p.text),
                        p.info,
                    ),
                    BadgeVariant::Neutral => (
                        self.fill.unwrap_or(p.crust),
                        self.stroke.unwrap_or(Stroke::new(1.0, p.overlay1)),
                        self.text_color.unwrap_or(p.subtext1),
                        p.subtext0,
                    ),
                    BadgeVariant::Custom { fill, text } => (
                        self.fill.unwrap_or(fill),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(text),
                        text,
                    ),
                },
                BadgeStyle::Solid => match self.variant {
                    BadgeVariant::Accent => (
                        self.fill.unwrap_or(p.accent),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_accent),
                        p.on_accent,
                    ),
                    BadgeVariant::Success => (
                        self.fill.unwrap_or(p.success),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_success),
                        p.on_success,
                    ),
                    BadgeVariant::Warning => (
                        self.fill.unwrap_or(p.warning),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_warning),
                        p.on_warning,
                    ),
                    BadgeVariant::Danger => (
                        self.fill.unwrap_or(p.danger),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_danger),
                        p.on_danger,
                    ),
                    BadgeVariant::Info => (
                        self.fill.unwrap_or(p.info),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_info),
                        p.on_info,
                    ),
                    BadgeVariant::Neutral => (
                        self.fill.unwrap_or(p.surface1),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(p.on_surface),
                        p.on_surface,
                    ),
                    BadgeVariant::Custom { fill, text } => (
                        self.fill.unwrap_or(fill),
                        self.stroke.unwrap_or(Stroke::NONE),
                        self.text_color.unwrap_or(text),
                        text,
                    ),
                },
            }
        } else {
            let v = ui.visuals();
            match self.style {
                BadgeStyle::Outline => (
                    self.fill.unwrap_or(v.panel_fill),
                    self.stroke.unwrap_or(Stroke::new(1.0, v.selection.stroke.color)),
                    self.text_color.unwrap_or(v.widgets.active.fg_stroke.color),
                    v.selection.stroke.color,
                ),
                BadgeStyle::Solid => (
                    self.fill.unwrap_or(v.selection.bg_fill),
                    self.stroke.unwrap_or(Stroke::NONE),
                    self.text_color.unwrap_or(v.widgets.active.fg_stroke.color),
                    v.widgets.active.fg_stroke.color,
                ),
            }
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let rounding = self.rounding.unwrap_or(Rounding::same(rect.height() * 0.5));

            // Background & border
            painter.add(Shape::rect_filled(rect, rounding, bg_fill));
            if bg_stroke.width > 0.0 {
                painter.add(Shape::rect_stroke(rect, rounding, bg_stroke));
            }

            // Center content inside rect
            let mut cursor_x = rect.center().x - content_w * 0.5;
            if self.dot {
                let dot_center = pos2(cursor_x + 4.5, rect.center().y);
                if self.style == BadgeStyle::Outline {
                    // Glowing dot: translucent outer halo + crisp core
                    painter.add(Shape::circle_filled(
                        dot_center,
                        4.0,
                        Color32::from_rgba_unmultiplied(
                            dot_color.r(),
                            dot_color.g(),
                            dot_color.b(),
                            60,
                        ),
                    ));
                    painter.add(Shape::circle_filled(dot_center, 2.5, dot_color));
                } else {
                    // Solid badge dot
                    painter.add(Shape::circle_filled(dot_center, 2.8, dot_color));
                }
                cursor_x += dot_spacing;
            }

            // Text
            let text_pos = pos2(cursor_x, rect.center().y - text_galley.size().y * 0.5);
            painter.galley(text_pos, text_galley, text_color);
        }

        response
    }
}
