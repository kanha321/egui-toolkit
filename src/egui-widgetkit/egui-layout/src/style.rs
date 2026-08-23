//! Visual styling configuration for `Split` layouts and card sections.

use egui::{Color32, Rounding, Stroke};

/// Visual styling configuration for [`Split`](crate::Split) layouts and their card sections.
///
/// Set once on a `Split` via [`.style()`](crate::Split::style), and all child sections
/// inherit these defaults unless individually overridden via [`Section`](crate::Section) builder methods.
///
/// # Resolution Order
///
/// For any visual property (e.g. corner rounding), the value is resolved as:
///
/// 1. **Section override** — if explicitly set via `Section::rounding()`, use that.
/// 2. **SplitStyle value** — if set in the style struct, use that.
/// 3. **`ui.visuals()` fallback** — read from egui's native visuals (which `ThemeState::apply_to_ctx()` populates from the active palette).
///
/// # Examples
///
/// ```rust
/// use egui_layout::SplitStyle;
/// use egui::Rounding;
///
/// let style = SplitStyle::default()
///     .with_spacing(8.0)
///     .with_card_rounding(12.0)
///     .with_card_padding(12.0)
///     .with_title_size(14.0);
/// ```
#[derive(Clone, Debug)]
pub struct SplitStyle {
    // ── Section Gaps ──

    /// Gap between sections in logical pixels (default: `4.0`).
    pub spacing: f32,

    // ── Card Container ──

    /// Default card background fill color.
    /// `None` = read from `ui.visuals().faint_bg_color`.
    pub card_bg: Option<Color32>,

    /// Default card border stroke (thickness + color).
    /// `None` = read from `ui.visuals().window_stroke`.
    pub card_stroke: Option<Stroke>,

    /// Default card corner rounding (default: `Rounding::same(8.0)`).
    pub card_rounding: Rounding,

    /// Default inner padding in logical pixels (default: `8.0`).
    pub card_padding: f32,

    // ── Typography ──

    /// Default title text color.
    /// `None` = read from `ui.visuals().strong_text_color()`.
    pub title_color: Option<Color32>,

    /// Default subtitle text color.
    /// `None` = read from `ui.visuals().text_color()`.
    pub subtitle_color: Option<Color32>,

    /// Title font size in points (default: `13.0`).
    pub title_size: f32,

    /// Subtitle font size in points (default: `10.5`).
    pub subtitle_size: f32,

    // ── Outer Frame ──

    /// Optional background fill for the entire `Split` container.
    /// `None` = transparent (no outer frame drawn).
    pub frame_bg: Option<Color32>,

    /// Optional border stroke for the entire `Split` container.
    /// `None` = no outer border.
    pub frame_stroke: Option<Stroke>,

    /// Corner rounding for the outer `Split` frame (default: `Rounding::ZERO`).
    pub frame_rounding: Rounding,

    /// Padding between the outer frame edge and the first/last sections (default: `0.0`).
    pub frame_padding: f32,
}

impl Default for SplitStyle {
    fn default() -> Self {
        Self {
            spacing: 4.0,
            card_bg: None,
            card_stroke: None,
            card_rounding: Rounding::same(8.0),
            card_padding: 8.0,
            title_color: None,
            subtitle_color: None,
            title_size: 13.0,
            subtitle_size: 10.5,
            frame_bg: None,
            frame_stroke: None,
            frame_rounding: Rounding::ZERO,
            frame_padding: 0.0,
        }
    }
}

impl SplitStyle {
    /// Creates a new style with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Section Gaps ──

    /// Sets the gap between sections in logical pixels.
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    // ── Card Container ──

    /// Sets the default card background fill color.
    pub fn with_card_bg(mut self, bg: Color32) -> Self {
        self.card_bg = Some(bg);
        self
    }

    /// Sets the default card border stroke (thickness + color).
    pub fn with_card_stroke(mut self, stroke: Stroke) -> Self {
        self.card_stroke = Some(stroke);
        self
    }

    /// Sets the default card corner rounding uniformly.
    pub fn with_card_rounding(mut self, radius: f32) -> Self {
        self.card_rounding = Rounding::same(radius.max(0.0));
        self
    }

    /// Sets the default card corner rounding with per-corner control.
    pub fn with_card_corner_rounding(mut self, rounding: Rounding) -> Self {
        self.card_rounding = rounding;
        self
    }

    /// Sets the default inner padding for card sections.
    pub fn with_card_padding(mut self, padding: f32) -> Self {
        self.card_padding = padding.max(0.0);
        self
    }

    // ── Typography ──

    /// Sets the default title text color for card sections.
    pub fn with_title_color(mut self, color: Color32) -> Self {
        self.title_color = Some(color);
        self
    }

    /// Sets the default subtitle text color for card sections.
    pub fn with_subtitle_color(mut self, color: Color32) -> Self {
        self.subtitle_color = Some(color);
        self
    }

    /// Sets the title font size in points.
    pub fn with_title_size(mut self, size: f32) -> Self {
        self.title_size = size.max(1.0);
        self
    }

    /// Sets the subtitle font size in points.
    pub fn with_subtitle_size(mut self, size: f32) -> Self {
        self.subtitle_size = size.max(1.0);
        self
    }

    // ── Outer Frame ──

    /// Sets the background fill for the entire `Split` container frame.
    pub fn with_frame_bg(mut self, bg: Color32) -> Self {
        self.frame_bg = Some(bg);
        self
    }

    /// Sets the border stroke for the entire `Split` container frame.
    pub fn with_frame_stroke(mut self, stroke: Stroke) -> Self {
        self.frame_stroke = Some(stroke);
        self
    }

    /// Sets the corner rounding for the outer `Split` frame uniformly.
    pub fn with_frame_rounding(mut self, radius: f32) -> Self {
        self.frame_rounding = Rounding::same(radius.max(0.0));
        self
    }

    /// Sets the corner rounding for the outer `Split` frame with per-corner control.
    pub fn with_frame_corner_rounding(mut self, rounding: Rounding) -> Self {
        self.frame_rounding = rounding;
        self
    }

    /// Sets the padding between the outer frame edge and the sections.
    pub fn with_frame_padding(mut self, padding: f32) -> Self {
        self.frame_padding = padding.max(0.0);
        self
    }
}
