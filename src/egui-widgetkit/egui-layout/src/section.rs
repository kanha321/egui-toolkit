//! Declarative builder for individual layout sections.

use egui::{Color32, Rect, Rounding, Stroke, Ui};
use crate::size::Size;

/// A builder representing a single configurable layout section.
///
/// `Section` allows declarative configuration of primary and cross-axis sizing constraints,
/// optional visual card framing (background, stroke, corner rounding, padding, headers),
/// and UI content closures.
///
/// # State Ownership
///
/// `Section` is an ephemeral builder constructed and consumed per frame;
/// it holds no global or static mutable state (`CODING_RULES §2`).
///
/// # Examples
///
/// ```rust
/// use egui::{Color32, Stroke, Rounding};
/// use egui_layout::Section;
///
/// let section = Section::fraction(0.33)
///     .min_size(150.0)
///     .card()
///     .title("Explorer")
///     .bg(Color32::from_rgb(30, 32, 48))
///     .rounding(8.0)
///     .padding(10.0)
///     .content(|ui| {
///         ui.label("Section content");
///     });
/// ```
pub struct Section<'a> {
    /// Sizing policy along the primary axis (Fraction, Exact, Remainder).
    pub size: Size,
    /// Optional minimum constraint along the cross axis in logical points.
    pub min_cross: Option<f32>,
    /// Optional maximum constraint along the cross axis in logical points.
    pub max_cross: Option<f32>,

    /// Whether this section should automatically render a styled card background and border.
    pub is_card: bool,
    /// Optional card title text rendered in the card header.
    pub title: Option<String>,
    /// Optional card subtitle text rendered below the title.
    pub subtitle: Option<String>,
    /// Optional accent color for title text.
    pub title_color: Option<Color32>,
    /// Optional accent color for subtitle text.
    pub subtitle_color: Option<Color32>,
    /// Background fill color for the card container.
    pub bg: Option<Color32>,
    /// Border stroke for the card container.
    pub stroke: Option<Stroke>,
    /// Corner rounding for the card container.
    pub rounding: Option<Rounding>,
    /// Inner margin padding in logical points (default: 8.0 when card is enabled).
    pub padding: Option<f32>,

    /// Closure providing the inner UI contents.
    pub add_contents: Box<dyn FnOnce(&mut Ui) + 'a>,
    /// Optional callback receiving the exact allocated `Rect` for this section.
    pub on_rect: Option<Box<dyn FnOnce(Rect) + 'a>>,
}

impl<'a> Default for Section<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Section<'a> {
    /// Creates a new default section taking 1.0 fraction of flexible space.
    pub fn new() -> Self {
        Self {
            size: Size::fraction(1.0),
            min_cross: None,
            max_cross: None,
            is_card: false,
            title: None,
            subtitle: None,
            title_color: None,
            subtitle_color: None,
            bg: None,
            stroke: None,
            rounding: None,
            padding: None,
            add_contents: Box::new(|_| {}),
            on_rect: None,
        }
    }

    /// Creates a section with a proportional fraction of flexible space.
    pub fn fraction(fraction: f32) -> Self {
        Self::new().size(Size::fraction(fraction))
    }

    /// Creates a section with an exact fixed size in logical points.
    pub fn fixed(px: f32) -> Self {
        Self::new().size(Size::exact(px))
    }

    /// Creates a section that claims all remaining flexible space.
    pub fn remainder() -> Self {
        Self::new().size(Size::remainder())
    }

    /// Sets the primary-axis sizing policy.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Adds a minimum size constraint along the primary axis in logical points.
    pub fn min_size(mut self, min_px: f32) -> Self {
        self.size = self.size.min_size(min_px);
        self
    }

    /// Adds a maximum size constraint along the primary axis in logical points.
    pub fn max_size(mut self, max_px: f32) -> Self {
        self.size = self.size.max_size(max_px);
        self
    }

    /// Adds a minimum constraint along the cross axis in logical points.
    pub fn min_cross(mut self, min_px: f32) -> Self {
        self.min_cross = Some(min_px.max(0.0));
        self
    }

    /// Adds a maximum constraint along the cross axis in logical points.
    pub fn max_cross(mut self, max_px: f32) -> Self {
        self.max_cross = Some(max_px.max(0.0));
        self
    }

    /// Sets 2D minimum dimensions (primary axis min, cross axis min).
    pub fn min_size_2d(mut self, primary_min: f32, cross_min: f32) -> Self {
        self.size = self.size.min_size(primary_min);
        self.min_cross = Some(cross_min.max(0.0));
        self
    }

    /// Enables automatic visual card container rendering (background, border, padding).
    pub fn card(mut self) -> Self {
        self.is_card = true;
        self
    }

    /// Sets the card header title text and enables card mode.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.is_card = true;
        self.title = Some(title.into());
        self
    }

    /// Sets the card header subtitle text and enables card mode.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.is_card = true;
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the title text color.
    pub fn title_color(mut self, color: Color32) -> Self {
        self.title_color = Some(color);
        self
    }

    /// Sets the subtitle text color.
    pub fn subtitle_color(mut self, color: Color32) -> Self {
        self.subtitle_color = Some(color);
        self
    }

    /// Sets the card background fill color and enables card mode.
    pub fn bg(mut self, bg: Color32) -> Self {
        self.is_card = true;
        self.bg = Some(bg);
        self
    }

    /// Sets the card border stroke and enables card mode.
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.is_card = true;
        self.stroke = Some(stroke);
        self
    }

    /// Sets the card corner rounding in logical points and enables card mode.
    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.is_card = true;
        self.rounding = Some(rounding.into());
        self
    }

    /// Sets the inner padding margin in logical points (default: 8.0).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding.max(0.0));
        self
    }

    /// Sets the UI contents closure for this section.
    pub fn content(mut self, add_contents: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.add_contents = Box::new(add_contents);
        self
    }

    /// Registers a callback to receive the allocated `Rect` for this section.
    pub fn on_rect(mut self, on_rect: impl FnOnce(Rect) + 'a) -> Self {
        self.on_rect = Some(Box::new(on_rect));
        self
    }

    /// Returns the minimum primary axis size required by this section.
    pub fn primary_min(&self) -> f32 {
        match self.size {
            Size::Exact(px) => px.max(0.0),
            Size::Fraction { min, .. } => min.unwrap_or(0.0),
            Size::Remainder { min } => min.unwrap_or(0.0),
        }
    }

    /// Returns the minimum cross axis size required by this section.
    pub fn cross_min(&self) -> f32 {
        self.min_cross.unwrap_or(0.0)
    }
}
