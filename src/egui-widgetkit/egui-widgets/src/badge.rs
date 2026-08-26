//! Semantic status badges, chips, and tag indicators.
//!
//! Provides [`Badge`] with status color variants, high-contrast [`BadgeStyle`] modes
//! (technical [`BadgeStyle::Outline`] and punchy [`BadgeStyle::Solid`]), optional glowing dots,
//! and theme palette synchronization.

use egui::{
    pos2, vec2, Color32, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui, Vec2,
    WidgetText,
};
use egui_themes::ThemePalette;

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
            padding: vec2(8.0, 3.5),
            palette: None,
        }
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
    pub fn show(self, ui: &mut Ui) -> Response {
        let text_galley = self.text.into_galley(
            ui,
            Some(false),
            f32::INFINITY,
            TextStyle::Small,
        );

        let mut content_w = text_galley.size().x;
        if self.dot {
            content_w += 12.0;
        }

        let desired_size = vec2(
            content_w + self.padding.x * 2.0,
            text_galley.size().y + self.padding.y * 2.0,
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

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

            // Dot
            let mut cursor_x = rect.left() + self.padding.x;
            if self.dot {
                let dot_center = pos2(cursor_x + 4.0, rect.center().y);
                if self.style == BadgeStyle::Outline {
                    // Glowing dot: translucent outer halo + crisp core
                    painter.add(Shape::circle_filled(
                        dot_center,
                        4.5,
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
                cursor_x += 12.0;
            }

            // Text
            let text_pos = pos2(cursor_x, rect.center().y - text_galley.size().y * 0.5);
            painter.galley(text_pos, text_galley, text_color);
        }

        response
    }
}
