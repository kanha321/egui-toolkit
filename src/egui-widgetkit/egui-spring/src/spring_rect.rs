//! Spring-animated highlight widget emitting pure `egui::Shape` primitives.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use spring_core::SpringParams;

use crate::bezier::{build_bezier_boundary, DEFAULT_ARC_SEGMENTS};
use crate::corner_springs::CornerSprings;

/// A spring-animated selection highlight rectangle with Bézier-rounded borders.
///
/// # State Ownership
///
/// `SpringRect` is a plain value struct owned by the consuming application and stored
/// across frames (`CODING_RULES §2`).
#[derive(Clone, Debug, PartialEq)]
pub struct SpringRect {
    /// 4-corner analytical spring bundle.
    pub corners: CornerSprings,
    /// Fill color for the highlight interior.
    pub fill_color: Color32,
    /// Stroke applied to the highlight border.
    pub stroke: Stroke,
    /// Corner rounding radius in pixels.
    pub rounding: f32,
}

impl Default for SpringRect {
    fn default() -> Self {
        Self::new(Rect::ZERO)
    }
}

impl SpringRect {
    /// Creates a new `SpringRect` initialized to `target_rect`.
    pub fn new(target_rect: Rect) -> Self {
        Self {
            corners: CornerSprings::new(target_rect, SpringParams::snappy()),
            fill_color: Color32::from_rgba_unmultiplied(137, 180, 250, 30),
            stroke: Stroke::new(1.5, Color32::from_rgb(137, 180, 250)),
            rounding: 6.0,
        }
    }

    /// Sets the physical spring dynamics parameters.
    pub fn with_params(mut self, params: SpringParams) -> Self {
        self.corners.params = params;
        self
    }

    /// Sets the fill color.
    pub fn with_fill(mut self, fill: Color32) -> Self {
        self.fill_color = fill;
        self
    }

    /// Sets the border stroke.
    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Sets the corner rounding radius.
    pub fn with_rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding.max(0.0);
        self
    }

    /// Retargets the spring highlight to a new bounding rectangle.
    pub fn set_target(&mut self, target: Rect) {
        self.corners.set_target(target);
    }

    /// Instantly snaps the highlight to `target` and clears all momentum.
    pub fn reset(&mut self, target: Rect) {
        self.corners.reset(target);
    }

    /// Advances the spring physics by delta time `dt`.
    pub fn update(&mut self, dt: f32) {
        self.corners.update(dt);
    }

    /// Checks if the animated highlight has settled at its target position.
    pub fn is_settled(&self) -> bool {
        self.corners.is_settled()
    }

    /// Returns the current bounding rect formed by the animated corner positions.
    pub fn current_bounding_rect(&self) -> Rect {
        let pts = self.corners.positions();
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for p in &pts {
            if p.x < min_x { min_x = p.x; }
            if p.x > max_x { max_x = p.x; }
            if p.y < min_y { min_y = p.y; }
            if p.y > max_y { max_y = p.y; }
        }

        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }

    /// Renders the spring highlight onto the given `Painter` using pure `egui::Shape`s (`CODING_RULES §4`).
    pub fn paint(&self, painter: &Painter) {
        let pts = self.corners.positions();
        let polygon_points = build_bezier_boundary(&pts, self.rounding, DEFAULT_ARC_SEGMENTS);

        if polygon_points.len() < 3 {
            return;
        }

        // 1. Fill shape
        if self.fill_color.a() > 0 {
            painter.add(Shape::convex_polygon(
                polygon_points.clone(),
                self.fill_color,
                Stroke::NONE,
            ));
        }

        // 2. Stroke outline
        if self.stroke.width > 0.0 && self.stroke.color.a() > 0 {
            painter.add(Shape::closed_line(
                polygon_points,
                self.stroke,
            ));
        }
    }

    /// Advances physics and renders the highlight in the provided `Ui`.
    ///
    /// Requests continuous repaint while unsettled (`CODING_RULES §4`).
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        self.update(dt);

        if !self.is_settled() {
            ui.ctx().request_repaint();
        }

        let bbox = self.current_bounding_rect();
        let (_rect, response) = ui.allocate_exact_size(
            if bbox.is_positive() { bbox.size() } else { Vec2::ZERO },
            Sense::hover(),
        );

        self.paint(ui.painter());

        response
    }
}
