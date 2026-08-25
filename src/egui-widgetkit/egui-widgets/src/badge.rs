//! Semantic status badges, chips, and tag indicators.
//!
//! Provides [`Badge`] and [`StatusChip`] with status color variants, optional pulsing dots,
//! dismiss buttons, and theme palette synchronization.

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

/// A theme-aware status badge or tag pill.
///
/// # Example
/// ```no_run
/// use egui_widgets::{Badge, BadgeVariant};
///
/// # egui::__run_test_ui(|ui| {
/// Badge::new("Online")
///     .variant(BadgeVariant::Success)
///     .dot(true)
///     .show(ui);
/// # });
/// ```
pub struct Badge<'a> {
    text: WidgetText,
    variant: BadgeVariant,
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
            dot: false,
            fill: None,
            stroke: None,
            text_color: None,
            rounding: None,
            padding: vec2(6.0, 2.5),
            palette: None,
        }
    }

    /// Sets the semantic badge variant.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
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

    /// Sets inner margin padding (default `vec2(6.0, 2.5)`).
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
            content_w += 10.0;
        }

        let desired_size = vec2(
            content_w + self.padding.x * 2.0,
            text_galley.size().y + self.padding.y * 2.0,
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Resolve colors
        let (bg_fill, bg_stroke, text_color, dot_color) = if let Some(p) = self.palette {
            match self.variant {
                BadgeVariant::Accent => (
                    self.fill.unwrap_or(p.accent.linear_multiply(0.20)),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.accent.linear_multiply(0.40))),
                    self.text_color.unwrap_or(p.accent),
                    p.accent,
                ),
                BadgeVariant::Success => (
                    self.fill.unwrap_or(p.success.linear_multiply(0.20)),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.success.linear_multiply(0.40))),
                    self.text_color.unwrap_or(p.success),
                    p.success,
                ),
                BadgeVariant::Warning => (
                    self.fill.unwrap_or(p.warning.linear_multiply(0.20)),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.warning.linear_multiply(0.40))),
                    self.text_color.unwrap_or(p.warning),
                    p.warning,
                ),
                BadgeVariant::Danger => (
                    self.fill.unwrap_or(p.danger.linear_multiply(0.20)),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.danger.linear_multiply(0.40))),
                    self.text_color.unwrap_or(p.danger),
                    p.danger,
                ),
                BadgeVariant::Info => (
                    self.fill.unwrap_or(p.info.linear_multiply(0.20)),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.info.linear_multiply(0.40))),
                    self.text_color.unwrap_or(p.info),
                    p.info,
                ),
                BadgeVariant::Neutral => (
                    self.fill.unwrap_or(p.surface0),
                    self.stroke.unwrap_or(Stroke::new(1.0, p.surface1)),
                    self.text_color.unwrap_or(p.text),
                    p.subtext0,
                ),
                BadgeVariant::Custom { fill, text } => (
                    self.fill.unwrap_or(fill),
                    self.stroke.unwrap_or(Stroke::NONE),
                    self.text_color.unwrap_or(text),
                    text,
                ),
            }
        } else {
            let v = ui.visuals();
            (
                self.fill.unwrap_or(v.selection.bg_fill.linear_multiply(0.3)),
                self.stroke.unwrap_or(Stroke::NONE),
                self.text_color.unwrap_or(v.widgets.active.fg_stroke.color),
                v.selection.stroke.color,
            )
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
                let dot_center = pos2(cursor_x + 3.0, rect.center().y);
                painter.add(Shape::circle_filled(dot_center, 3.0, dot_color));
                cursor_x += 10.0;
            }

            // Text
            let text_pos = pos2(cursor_x, rect.center().y - text_galley.size().y * 0.5);
            painter.galley(text_pos, text_galley, text_color);
        }

        response
    }
}
