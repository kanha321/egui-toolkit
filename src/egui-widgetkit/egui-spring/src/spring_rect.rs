//! Spring-animated highlight widget emitting pure `egui::Shape` primitives.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use spring_core::{Spring, SpringParams};

use crate::bezier::{build_bezier_boundary, DEFAULT_ARC_SEGMENTS};
use crate::corner_springs::CornerSprings;

/// A spring-animated selection highlight rectangle with Bézier-rounded borders
/// and flight-synchronized shape morphing (OpenRGB / Neovide design).
///
/// # State Ownership
///
/// `SpringRect` is a plain value struct owned by the consuming application and stored
/// across frames (`CODING_RULES §2`).
#[derive(Clone, Debug, PartialEq)]
pub struct SpringRect {
    /// 4-corner analytical spring bundle.
    pub corners: CornerSprings,
    /// Alpha opacity fade-in spring.
    pub alpha_spring: Spring,
    /// Fill color for the highlight interior.
    pub fill_color: Color32,
    /// Stroke applied to the highlight border.
    pub stroke: Stroke,
    /// Target corner rounding radius in pixels.
    pub target_rounding: f32,
    /// Starting rounding radius at beginning of flight.
    pub start_rounding: f32,
    /// Current interpolated rounding radius.
    pub current_rounding: f32,
    /// Flight start centroid position.
    pub start_center: Pos2,
    /// Flight target centroid position.
    pub target_center: Pos2,
    /// Padding expansion added around target rect (default: `3.0` px).
    pub padding: f32,
}

impl Default for SpringRect {
    fn default() -> Self {
        Self::new(Rect::ZERO)
    }
}

impl SpringRect {
    /// Creates a new `SpringRect` initialized to `target_rect` with OpenRGB/Neovide defaults.
    pub fn new(target_rect: Rect) -> Self {
        let default_rounding = 6.0;
        Self {
            corners: CornerSprings::new(target_rect, 22.0, 0.65),
            alpha_spring: Spring::new(1.0, SpringParams::new(24.0, 0.75)),
            fill_color: Color32::from_rgba_unmultiplied(0, 255, 136, 16),
            stroke: Stroke::new(1.5, Color32::from_rgb(0, 255, 136)),
            target_rounding: default_rounding,
            start_rounding: default_rounding,
            current_rounding: default_rounding,
            start_center: target_rect.center(),
            target_center: target_rect.center(),
            padding: 3.0,
        }
    }

    /// Sets the base physical spring dynamics parameters.
    pub fn with_params(mut self, params: SpringParams) -> Self {
        self.corners.base_stiffness = params.angular_frequency;
        self.corners.base_damping = params.damping_ratio;
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
        self.target_rounding = rounding.max(0.0);
        self.current_rounding = rounding.max(0.0);
        self.start_rounding = rounding.max(0.0);
        self
    }

    /// Sets the target bounding expansion padding (default: 3.0 px).
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Retargets the spring highlight to a new bounding rectangle.
    pub fn set_target(&mut self, target: Rect) {
        let padded = target.expand(self.padding);
        let new_target_center = padded.center();

        if (new_target_center - self.target_center).length() > 1.0 {
            self.start_center = self.corners.center();
            self.start_rounding = self.current_rounding;
            self.target_center = new_target_center;
        }

        self.corners.target_rect = padded;
        if !self.corners.initialized {
            self.reset(target);
        }
    }

    /// Instantly snaps the highlight to `target` and clears all momentum.
    pub fn reset(&mut self, target: Rect) {
        let padded = target.expand(self.padding);
        self.corners.reset(padded);
        self.start_center = padded.center();
        self.target_center = padded.center();
        self.current_rounding = self.target_rounding;
        self.start_rounding = self.target_rounding;
        self.alpha_spring.reset(1.0);
    }

    /// Advances the spring physics by delta time `dt`.
    pub fn update(&mut self, dt: f32) {
        let padded_target = self.corners.target_rect;
        self.corners.update(padded_target, dt);
        self.alpha_spring.update(dt);

        // Morph rounding smoothly in direct proportion to spatial flight progress
        let total_dist = (self.target_center - self.start_center).length();
        if total_dist > 2.0 {
            let current_dist = (self.target_center - self.corners.center()).length();
            let raw_progress = (1.0 - (current_dist / total_dist)).clamp(0.0, 1.0);
            // Hermite smoothstep for natural organic curve interpolation
            let smooth_progress = raw_progress * raw_progress * (3.0 - 2.0 * raw_progress);
            self.current_rounding = self.start_rounding
                + (self.target_rounding - self.start_rounding) * smooth_progress;
        } else {
            self.current_rounding = self.target_rounding;
        }
    }

    /// Checks if the animated highlight has settled at its target position.
    pub fn is_settled(&self) -> bool {
        self.corners.is_settled() && self.alpha_spring.is_settled()
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
        let polygon_points = build_bezier_boundary(&pts, self.current_rounding, DEFAULT_ARC_SEGMENTS);

        if polygon_points.len() < 3 {
            return;
        }

        let alpha_factor = self.alpha_spring.value().clamp(0.0, 1.0);

        // 1. Fill shape
        if self.fill_color.a() > 0 {
            let fill_a = (self.fill_color.a() as f32 * alpha_factor) as u8;
            let fill = Color32::from_rgba_unmultiplied(
                self.fill_color.r(),
                self.fill_color.g(),
                self.fill_color.b(),
                fill_a,
            );
            painter.add(Shape::convex_polygon(
                polygon_points.clone(),
                fill,
                Stroke::NONE,
            ));
        }

        // 2. Stroke outline
        if self.stroke.width > 0.0 && self.stroke.color.a() > 0 {
            let stroke_a = (self.stroke.color.a() as f32 * alpha_factor) as u8;
            let stroke_color = Color32::from_rgba_unmultiplied(
                self.stroke.color.r(),
                self.stroke.color.g(),
                self.stroke.color.b(),
                stroke_a,
            );
            painter.add(Shape::closed_line(
                polygon_points,
                Stroke::new(self.stroke.width, stroke_color),
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
