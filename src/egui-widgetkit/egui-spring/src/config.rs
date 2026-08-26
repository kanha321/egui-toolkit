//! Configuration descriptor for spring highlight styling and motion physics.

use egui::{Color32, Rounding, Stroke};
use egui_themes::ThemePalette;
use spring_core::MotionPhysics;

/// Reusable configuration bundle for a highlight's physics, colors, and geometry.
///
/// Can be cloned, shared, and applied to any `SpringRect` or used to instantiate one.
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightConfig {
    /// Motion physics preset, custom params, or `Off`.
    pub motion: MotionPhysics,
    /// Fill color for highlight background interior.
    pub fill: Color32,
    /// Border stroke width and color.
    pub stroke: Stroke,
    /// Per-corner rounding radius.
    pub rounding: Rounding,
    /// Bounding box expansion padding in pixels.
    pub padding: f32,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            motion: MotionPhysics::Default,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, Color32::from_rgb(0, 255, 136)),
            rounding: Rounding::same(6.0),
            padding: 3.0,
        }
    }
}

impl HighlightConfig {
    /// Creates a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new configuration whose stroke derives from the palette's accent color.
    pub fn from_palette(palette: &ThemePalette) -> Self {
        let accent = palette.accent;
        Self {
            motion: MotionPhysics::Default,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, accent),
            rounding: Rounding::same(6.0),
            padding: 3.0,
        }
    }

    /// Sets the motion physics mode.
    pub fn with_motion(mut self, motion: MotionPhysics) -> Self {
        self.motion = motion;
        self
    }

    /// Sets the interior fill color.
    pub fn with_fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the border stroke.
    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Sets uniform corner rounding.
    pub fn with_rounding(mut self, rounding: f32) -> Self {
        self.rounding = Rounding::same(rounding.max(0.0));
        self
    }

    /// Sets asymmetric per-corner rounding.
    pub fn with_corner_rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    /// Sets target bounding padding.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}
