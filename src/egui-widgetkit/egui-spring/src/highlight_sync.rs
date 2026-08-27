//! Highlight color synchronization utilities.
//!
//! Provides helper functions for smoothly interpolating a [`HighlightGroup`] layer's
//! stroke color toward a target, and for batch-updating highlight physics and painting.
//!
//! These utilities extract the common "retarget → update → paint" cycle that every
//! vim-navigated UI performs each frame, reducing boilerplate while preserving full
//! control over what the target color is (app-specific) and when repaints are requested.
//!
//! # State Ownership
//!
//! These functions operate on the caller-owned [`HighlightGroup`] by `&mut` reference.
//! They introduce no persistent state of their own.

use egui::{Color32, Stroke};

use crate::group::HighlightGroup;

/// Smoothly interpolates a highlight layer's stroke color toward `target_color`.
///
/// Uses exponential smoothing at the given `speed` (higher = faster convergence).
/// A typical value is `14.0` for responsive color transitions.
///
/// Returns `true` if the color has not yet converged (the caller should call
/// `ctx.request_repaint()` to continue the animation). Returns `false` when
/// the stroke color has reached the target.
///
/// # Arguments
///
/// - `group`: The highlight group containing the layer to update.
/// - `key`: The key identifying which layer to update.
/// - `target_color`: The color to interpolate toward.
/// - `dt`: Frame delta time in seconds (from `ui.input(|i| i.stable_dt)`).
/// - `speed`: Interpolation speed multiplier (e.g., `14.0`).
///
/// # Returns
///
/// - `true` if still interpolating (caller should request repaint).
/// - `false` if the color has converged or the layer doesn't exist.
///
/// # Example
///
/// ```rust,ignore
/// use egui_spring::highlight_sync::sync_highlight_stroke_color;
///
/// let still_animating = sync_highlight_stroke_color(
///     &mut highlights,
///     &HighlightKey::Item,
///     target_color,
///     dt,
///     14.0,
/// );
/// if still_animating {
///     ui.ctx().request_repaint();
/// }
/// ```
pub fn sync_highlight_stroke_color<K>(
    group: &mut HighlightGroup<K>,
    key: &K,
    target_color: Color32,
    dt: f32,
    speed: f32,
) -> bool
where
    K: Ord + Clone,
{
    let layer = match group.get_mut(key) {
        Some(l) => l,
        None => return false,
    };

    let cur_col = layer.stroke.color;
    let smoothed = egui_themes::lerp_color(cur_col, target_color, (dt * speed).clamp(0.0, 1.0));
    layer.stroke = Stroke::new(layer.stroke.width, smoothed);

    smoothed != target_color
}

/// Updates a highlight layer's fill color to `target_fill` (instant, no interpolation).
///
/// This is a convenience for the common pattern of keeping fill transparent or
/// syncing it to a palette token without animation.
pub fn set_highlight_fill<K>(
    group: &mut HighlightGroup<K>,
    key: &K,
    target_fill: Color32,
) where
    K: Ord + Clone,
{
    if let Some(layer) = group.get_mut(key) {
        layer.fill_color = target_fill;
    }
}

/// Convenience: updates all layers, paints all layers, and returns whether any
/// layer is still animating (not settled).
///
/// This replaces the common 3-line pattern:
/// ```rust,ignore
/// highlights.update(dt);
/// if !highlights.is_settled() { ctx.request_repaint(); }
/// highlights.paint_all(ui.painter());
/// ```
///
/// Returns `true` if any highlight is still in motion (caller should request repaint).
pub fn update_and_paint<K>(
    group: &mut HighlightGroup<K>,
    dt: f32,
    painter: &egui::Painter,
) -> bool
where
    K: Ord + Clone,
{
    group.update(dt);
    let animating = !group.is_settled();
    group.paint_all(painter);
    animating
}
