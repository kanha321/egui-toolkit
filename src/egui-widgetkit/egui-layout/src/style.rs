//! Visual styling configuration for `Split` layouts and card sections.

use egui::{Color32, Rounding, Stroke};

/// Visual visibility mode for interactive resize dividers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DividerVisibility {
    /// Dividers are invisible when idle at rest, but become visible with highlights and grip on hover or drag.
    #[default]
    HoverOnly,

    /// Dividers are always drawn, including when idle at rest.
    Visible,

    /// Dividers are never rendered visually (completely hidden), though resizing interactions still function.
    Hidden,
}

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
/// use egui_layout::{SplitStyle, DividerVisibility};
/// use egui::Rounding;
///
/// let style = SplitStyle::default()
///     .with_spacing(8.0)
///     .with_card_rounding(12.0)
///     .with_card_padding(12.0)
///     .with_divider_visibility(DividerVisibility::HoverOnly);
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

    // ── Divider / Resize Handle ──

    /// Width of the interactive hit area for dividers in logical points (default: `8.0`).
    /// The visible divider line can be thinner; this controls the hover/drag target.
    pub divider_hit_width: f32,

    /// Thickness of the visible divider line in logical points (default: `1.0`).
    pub divider_thickness: f32,

    /// Divider line color at rest.
    /// `None` = subtle, derived from `ui.visuals().widgets.noninteractive.bg_stroke.color`.
    pub divider_color: Option<Color32>,

    /// Divider line color on hover.
    /// `None` = accent tint from visuals.
    pub divider_hover_color: Option<Color32>,

    /// Divider line color while actively dragging.
    /// `None` = stronger accent from visuals.
    pub divider_drag_color: Option<Color32>,

    /// Whether to show a small grip indicator on dividers (default: `true`).
    pub divider_show_grip: bool,

    /// Visual visibility mode for dividers (default: [`DividerVisibility::HoverOnly`]).
    /// - `HoverOnly`: invisible at rest, visible on hover/drag.
    /// - `Visible`: always visible.
    /// - `Hidden`: never rendered visually (resizing still functions).
    pub divider_visibility: DividerVisibility,

    // ── Collapse ──

    /// Size of the collapse chevron icon in points (default: `12.0`).
    pub collapse_chevron_size: f32,

    /// Collapse chevron color.
    /// `None` = derived from `ui.visuals().text_color()`.
    pub collapse_chevron_color: Option<Color32>,
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
            divider_hit_width: 8.0,
            divider_thickness: 1.0,
            divider_color: None,
            divider_hover_color: None,
            divider_drag_color: None,
            divider_show_grip: true,
            divider_visibility: DividerVisibility::HoverOnly,
            collapse_chevron_size: 12.0,
            collapse_chevron_color: None,
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

    // ── Divider / Resize Handle ──

    /// Sets the interactive hit area width for dividers.
    pub fn with_divider_hit_width(mut self, px: f32) -> Self {
        self.divider_hit_width = px.max(1.0);
        self
    }

    /// Sets the visible divider line thickness.
    pub fn with_divider_thickness(mut self, px: f32) -> Self {
        self.divider_thickness = px.max(0.0);
        self
    }

    /// Sets the divider line color at rest.
    pub fn with_divider_color(mut self, color: Color32) -> Self {
        self.divider_color = Some(color);
        self
    }

    /// Sets the divider line color on hover.
    pub fn with_divider_hover_color(mut self, color: Color32) -> Self {
        self.divider_hover_color = Some(color);
        self
    }

    /// Sets the divider line color while actively dragging.
    pub fn with_divider_drag_color(mut self, color: Color32) -> Self {
        self.divider_drag_color = Some(color);
        self
    }

    /// Sets whether to show a grip indicator on dividers.
    pub fn with_divider_show_grip(mut self, show: bool) -> Self {
        self.divider_show_grip = show;
        self
    }

    /// Sets the visual visibility mode for resize dividers.
    pub fn with_divider_visibility(mut self, visibility: DividerVisibility) -> Self {
        self.divider_visibility = visibility;
        self
    }

    /// Convenience helper to set dividers to always visible.
    pub fn with_divider_always_visible(mut self) -> Self {
        self.divider_visibility = DividerVisibility::Visible;
        self
    }

    /// Convenience helper to set dividers to visible on hover only.
    pub fn with_divider_hover_only(mut self) -> Self {
        self.divider_visibility = DividerVisibility::HoverOnly;
        self
    }

    /// Convenience helper to completely hide dividers visually.
    pub fn with_divider_hidden(mut self) -> Self {
        self.divider_visibility = DividerVisibility::Hidden;
        self
    }

    /// Sets whether the divider line is visible when idle (true = Visible, false = HoverOnly).
    pub fn with_divider_idle_visible(mut self, visible: bool) -> Self {
        self.divider_visibility = if visible {
            DividerVisibility::Visible
        } else {
            DividerVisibility::HoverOnly
        };
        self
    }

    /// Alias for [`Self::with_divider_idle_visible`].
    pub fn with_divider_visible(self, visible: bool) -> Self {
        self.with_divider_idle_visible(visible)
    }

    // ── Collapse ──

    /// Sets the collapse chevron icon size in points.
    pub fn with_collapse_chevron_size(mut self, size: f32) -> Self {
        self.collapse_chevron_size = size.max(4.0);
        self
    }

    /// Sets the collapse chevron color.
    pub fn with_collapse_chevron_color(mut self, color: Color32) -> Self {
        self.collapse_chevron_color = Some(color);
        self
    }
}
