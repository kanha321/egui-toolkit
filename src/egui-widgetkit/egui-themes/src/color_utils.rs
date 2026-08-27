//! Color utility functions for theme-aware UI components.
//!
//! These functions operate on [`egui::Color32`] values and are used throughout
//! `egui-widgetkit` for smooth color transitions, contrast resolution, and
//! palette-driven styling.

use egui::Color32;

/// Linearly interpolates between two [`Color32`] values in premultiplied RGBA space.
///
/// The interpolation factor `t` is clamped to `[0.0, 1.0]`:
/// - `t = 0.0` → returns `a`
/// - `t = 1.0` → returns `b`
/// - `t = 0.5` → returns the midpoint
///
/// # Example
///
/// ```
/// use egui::Color32;
/// use egui_themes::lerp_color;
///
/// let red = Color32::from_rgb(255, 0, 0);
/// let blue = Color32::from_rgb(0, 0, 255);
/// let purple = lerp_color(red, blue, 0.5);
/// assert_eq!(purple.r(), 127);
/// assert_eq!(purple.b(), 127);
/// ```
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_at_zero_returns_a() {
        let a = Color32::from_rgb(100, 50, 200);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(lerp_color(a, b, 0.0), a);
    }

    #[test]
    fn lerp_at_one_returns_b() {
        let a = Color32::from_rgb(100, 50, 200);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(lerp_color(a, b, 1.0), b);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        let mid = lerp_color(a, b, 0.5);
        assert_eq!(mid.r(), 100);
        assert_eq!(mid.g(), 50);
        assert_eq!(mid.b(), 25);
    }

    #[test]
    fn lerp_clamps_t() {
        let a = Color32::from_rgb(100, 100, 100);
        let b = Color32::from_rgb(200, 200, 200);
        assert_eq!(lerp_color(a, b, -1.0), a);
        assert_eq!(lerp_color(a, b, 2.0), b);
    }
}
