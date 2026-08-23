//! Immediate-mode focus region helper for rendering focusable UI elements.
//!
//! [`FocusRegion`] is a lightweight widget wrapper that renders content,
//! detects whether the given node ID is currently focused, and handles
//! click-and-hover-to-focus. It does **not** modify or auto-infer the
//! [`FocusGraph`](super::FocusGraph) topology — the graph is a separate
//! concern, configured by the consuming application.
//!
//! # Registration pattern
//!
//! This follows the **per-frame immediate-mode pattern**: the consuming
//! application calls `FocusRegion::show()` every frame for each focusable
//! element. Focus state is determined by comparing the node ID against
//! the [`Navigator`](super::Navigator)'s current focus. No persistent
//! registration is needed — if a region stops being drawn, it simply
//! isn't checked that frame.
//!
//! # Stable ID scheme
//!
//! Node IDs are application-supplied values of type `T` (the same generic
//! parameter used by `FocusGraph<T>` and `Navigator<T>`). Because these are
//! stable across frames (e.g. enum variants, `&'static str`, or
//! application-defined IDs), focus is preserved seamlessly between frames.

use std::fmt::Debug;
use std::hash::Hash;

use egui::{Response, Ui, Vec2};

use super::navigator::Navigator;

/// Result of rendering a focusable region.
pub struct FocusRegionResponse<R> {
    /// The interactive egui response with `Sense::click()`.
    pub response: Response,
    /// Whether this region currently has focus.
    pub is_focused: bool,
    /// Whether this region was hovered by the pointer.
    pub is_hovered: bool,
    /// Whether this region was primary-clicked (left click) by the pointer.
    pub is_clicked: bool,
    /// Whether this region was secondary-clicked (right click) by the pointer.
    pub is_secondary_clicked: bool,
    /// The value returned by the content closure.
    pub inner: R,
}

/// Immediate-mode helper for rendering a focusable UI region.
///
/// Wraps content in an `egui::Frame` and handles focus detection,
/// hover-to-focus (when the pointer is moving), primary click-to-focus,
/// and secondary click-to-focus automatically.
/// Does not modify the `FocusGraph` — the graph topology is the consuming
/// application's responsibility.
///
/// # Example
///
/// ```ignore
/// let resp = FocusRegion::show(ui, &mut navigator, &"card_a", |ui, focused| {
///     let stroke = if focused {
///         egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
///     } else {
///         egui::Stroke::NONE
///     };
///     egui::Frame::none().stroke(stroke).show(ui, |ui| {
///         ui.label("Card A");
///     });
/// });
/// ```
pub struct FocusRegion;

impl FocusRegion {
    /// Renders a focusable region and handles primary/secondary click-to-focus and hover-to-focus.
    ///
    /// - `ui`: The parent UI to render into.
    /// - `nav`: The navigator tracking current focus (mutable for click/hover-to-focus).
    /// - `id`: The node ID for this region (must match IDs used in the `FocusGraph`).
    /// - `add_contents`: Closure receiving `(&mut Ui, bool)` where the `bool`
    ///   indicates whether this region is currently focused.
    ///
    /// Returns a [`FocusRegionResponse`] with the interactive egui response, focus state,
    /// primary click state, secondary click state, hover state, and the closure's return value.
    pub fn show<T, R>(
        ui: &mut Ui,
        nav: &mut Navigator<T>,
        id: &T,
        add_contents: impl FnOnce(&mut Ui, bool) -> R,
    ) -> FocusRegionResponse<R>
    where
        T: Clone + Eq + Hash + Debug,
    {
        let is_focused = nav.focused() == Some(id);

        // Wrap in a group to get a response for the entire region
        let group_resp = ui.group(|ui| {
            add_contents(ui, is_focused)
        });

        // Click-and-hover-to-focus: interact with the rect using Sense::click()
        let interact_resp = ui.interact(
            group_resp.response.rect,
            group_resp.response.id.with("focus_interact"),
            egui::Sense::click(),
        );

        let is_clicked = interact_resp.clicked();
        let is_secondary_clicked = interact_resp.secondary_clicked();
        let is_hovered = interact_resp.hovered();
        let pointer_moved = ui.input(|i| i.pointer.delta() != Vec2::ZERO);

        // Update focus when clicked (left or right), or when the pointer is actively moving over the element
        if is_clicked || is_secondary_clicked || (is_hovered && pointer_moved) {
            nav.set_focus(Some(id.clone()));
        }

        FocusRegionResponse {
            response: interact_resp,
            is_focused: nav.focused() == Some(id),
            is_hovered,
            is_clicked,
            is_secondary_clicked,
            inner: group_resp.inner,
        }
    }
}
