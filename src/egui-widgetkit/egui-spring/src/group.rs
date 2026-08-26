//! Multi-layer highlight manager for managing any number of independent spring highlights.

use std::collections::BTreeMap;
use egui::{Painter, Rect, Rounding, Ui};

use crate::config::HighlightConfig;
use crate::spring_rect::SpringRect;

/// Manages an arbitrary number of independent `SpringRect` highlights, each with its own
/// unique settings, motion physics, color styling, and target bounding rect.
///
/// # State Ownership Contract
///
/// `HighlightGroup<K>` is a plain value struct owned by the consuming application
/// across frames (`CODING_RULES §2`). There is zero hidden static or global state.
///
/// # Quickstart
///
/// ```rust
/// use egui_spring::{HighlightGroup, HighlightConfig, MotionPhysics};
/// use egui::{Color32, Rect, Stroke};
///
/// let mut highlights = HighlightGroup::new();
/// highlights.add("outer", HighlightConfig::new().with_motion(MotionPhysics::Gentle));
/// highlights.add("inner", HighlightConfig::new().with_motion(MotionPhysics::Snappy));
/// highlights.add("cursor", HighlightConfig::new().with_motion(MotionPhysics::Off));
///
/// // Update targets
/// highlights.set_target(&"outer", Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightGroup<K: Ord + Clone> {
    layers: BTreeMap<K, SpringRect>,
}

impl<K: Ord + Clone> Default for HighlightGroup<K> {
    fn default() -> Self {
        Self {
            layers: BTreeMap::new(),
        }
    }
}

impl<K: Ord + Clone> HighlightGroup<K> {
    /// Creates an empty `HighlightGroup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new highlight configured with `HighlightConfig`.
    pub fn add(&mut self, key: K, config: HighlightConfig) -> &mut SpringRect {
        let rect = SpringRect::from_config(Rect::ZERO, &config);
        self.layers.insert(key.clone(), rect);
        self.layers.get_mut(&key).expect("just inserted")
    }

    /// Inserts an existing `SpringRect` under `key`.
    pub fn insert(&mut self, key: K, rect: SpringRect) -> &mut SpringRect {
        self.layers.insert(key.clone(), rect);
        self.layers.get_mut(&key).expect("just inserted")
    }

    /// Builder helper to chain insertion of a highlight layer.
    pub fn with_layer(mut self, key: K, rect: SpringRect) -> Self {
        self.insert(key, rect);
        self
    }

    /// Builder helper to chain insertion of a highlight configured via `HighlightConfig`.
    pub fn with_config(mut self, key: K, config: HighlightConfig) -> Self {
        self.add(key, config);
        self
    }

    /// Returns a reference to the `SpringRect` for `key`, if it exists.
    pub fn get(&self, key: &K) -> Option<&SpringRect> {
        self.layers.get(key)
    }

    /// Returns a mutable reference to the `SpringRect` for `key`, if it exists.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut SpringRect> {
        self.layers.get_mut(key)
    }

    /// Removes a highlight layer by key.
    pub fn remove(&mut self, key: &K) -> Option<SpringRect> {
        self.layers.remove(key)
    }

    /// Returns the number of active highlight layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Returns `true` if there are no highlight layers registered.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Sets the target bounding rect for a specific highlight layer.
    pub fn set_target(&mut self, key: &K, target: Rect) {
        if let Some(layer) = self.layers.get_mut(key) {
            layer.set_target(target);
        }
    }

    /// Teleports a specific highlight layer immediately to `rect` and `rounding` without animation.
    pub fn reset_layer(&mut self, key: &K, rect: Rect, rounding: Rounding) {
        if let Some(layer) = self.layers.get_mut(key) {
            layer.reset_to(rect, rounding);
        }
    }

    /// Sets the target bounding rect and uniform rounding for a specific highlight layer.
    pub fn set_target_with_rounding(&mut self, key: &K, target: Rect, rounding: f32) {
        if let Some(layer) = self.layers.get_mut(key) {
            layer.set_target_with_rounding(target, rounding);
        }
    }

    /// Sets the target bounding rect and asymmetric per-corner rounding for a specific highlight layer.
    pub fn set_target_with_corner_rounding(&mut self, key: &K, target: Rect, rounding: Rounding) {
        if let Some(layer) = self.layers.get_mut(key) {
            layer.set_target_with_corner_rounding(target, rounding);
        }
    }

    /// Advances physics on all highlight layers by delta time `dt`.
    pub fn update(&mut self, dt: f32) {
        for layer in self.layers.values_mut() {
            layer.update(dt);
        }
    }

    /// Returns `true` if ALL highlight layers in this group are settled.
    pub fn is_settled(&self) -> bool {
        self.layers.values().all(|layer| layer.is_settled())
    }

    /// Renders a single highlight layer by key to `Painter`.
    pub fn paint_layer(&self, key: &K, painter: &Painter) {
        if let Some(layer) = self.layers.get(key) {
            layer.paint(painter);
        }
    }

    /// Renders all highlight layers to `Painter` in key order.
    pub fn paint_all(&self, painter: &Painter) {
        for layer in self.layers.values() {
            layer.paint(painter);
        }
    }

    /// Advances physics on all layers and requests repaint until all layers have settled.
    ///
    /// Renders all layers to the `ui` painter.
    pub fn show(&mut self, ui: &mut Ui) {
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        self.update(dt);

        if !self.is_settled() {
            ui.ctx().request_repaint();
        }

        self.paint_all(ui.painter());
    }

    /// Provides an iterator over all `(key, SpringRect)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &SpringRect)> {
        self.layers.iter()
    }

    /// Provides a mutable iterator over all `(key, SpringRect)` pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut SpringRect)> {
        self.layers.iter_mut()
    }
}
