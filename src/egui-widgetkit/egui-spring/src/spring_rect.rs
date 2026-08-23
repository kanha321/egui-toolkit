//! Spring-animated highlight widget emitting pure `egui::Shape` primitives.

use egui::{Color32, Painter, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui, Vec2};
use spring_core::{MotionPhysics, Spring, SpringParams};

use crate::bezier::{build_bezier_boundary, DEFAULT_ARC_SEGMENTS};
use crate::corner_springs::CornerSprings;

/// A spring-animated selection highlight rectangle with Bézier-rounded borders
/// and flight-synchronized shape morphing across regular and irregular geometric shapes.
///
/// # State Ownership
///
/// `SpringRect` is a plain value struct owned by the consuming application and stored
/// across frames (`CODING_RULES §2`).
#[derive(Clone, Debug, PartialEq)]
pub struct SpringRect {
    /// Active motion physics mode (presets, custom, or `Off`).
    pub motion: MotionPhysics,
    /// 4-corner analytical spring bundle.
    pub corners: CornerSprings,
    /// Alpha opacity fade-in spring.
    pub alpha_spring: Spring,
    /// Fill color for the highlight interior.
    pub fill_color: Color32,
    /// Stroke applied to the highlight border.
    pub stroke: Stroke,
    /// Target corner rounding per-corner (NW, NE, SE, SW).
    pub target_rounding: Rounding,
    /// Starting rounding per-corner at beginning of flight.
    pub start_rounding: Rounding,
    /// Current interpolated rounding per-corner.
    pub current_rounding: Rounding,
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
    /// Creates a new `SpringRect` initialized to `target_rect` with default spring dynamics.
    pub fn new(target_rect: Rect) -> Self {
        let default_rounding = Rounding::same(6.0);
        let default_motion = MotionPhysics::Default;
        let default_params = default_motion.to_params().unwrap_or_default();
        Self {
            motion: default_motion,
            corners: CornerSprings::new(target_rect, default_params.angular_frequency, default_params.damping_ratio),
            alpha_spring: Spring::new(1.0, SpringParams::new(24.0, 0.75)),
            fill_color: Color32::from_rgba_unmultiplied(0, 255, 136, 14),
            stroke: Stroke::new(1.5, Color32::from_rgb(0, 255, 136)),
            target_rounding: default_rounding,
            start_rounding: default_rounding,
            current_rounding: default_rounding,
            start_center: target_rect.center(),
            target_center: target_rect.center(),
            padding: 3.0,
        }
    }

    /// Creates a new `SpringRect` configured with the provided `HighlightConfig`.
    pub fn from_config(target_rect: Rect, config: &crate::config::HighlightConfig) -> Self {
        let mut rect = Self::new(target_rect);
        rect.apply_config(config);
        rect
    }

    /// Applies a `HighlightConfig` directly to this `SpringRect`.
    pub fn apply_config(&mut self, config: &crate::config::HighlightConfig) {
        self.set_motion(config.motion);
        self.fill_color = config.fill;
        self.stroke = config.stroke;
        self.target_rounding = config.rounding;
        self.current_rounding = config.rounding;
        self.start_rounding = config.rounding;
        self.padding = config.padding;
    }

    /// Exports the current styling and motion parameters as a `HighlightConfig`.
    pub fn to_config(&self) -> crate::config::HighlightConfig {
        crate::config::HighlightConfig {
            motion: self.motion,
            fill: self.fill_color,
            stroke: self.stroke,
            rounding: self.target_rounding,
            padding: self.padding,
        }
    }

    /// Creates an instant highlight with motion physics turned `Off`.
    pub fn off(target_rect: Rect) -> Self {
        Self::new(target_rect).with_motion(MotionPhysics::Off)
    }

    /// Creates a highlight configured with the `Gentle` spring preset.
    pub fn gentle(target_rect: Rect) -> Self {
        Self::new(target_rect).with_motion(MotionPhysics::Gentle)
    }

    /// Creates a highlight configured with the `Snappy` spring preset.
    pub fn snappy(target_rect: Rect) -> Self {
        Self::new(target_rect).with_motion(MotionPhysics::Snappy)
    }

    /// Creates a highlight configured with the `Bouncy` spring preset.
    pub fn bouncy(target_rect: Rect) -> Self {
        Self::new(target_rect).with_motion(MotionPhysics::Bouncy)
    }

    /// Creates a highlight configured with the `OpenRGB` spring preset.
    pub fn openrgb(target_rect: Rect) -> Self {
        Self::new(target_rect).with_motion(MotionPhysics::OpenRGB)
    }

    /// Sets the motion physics mode (e.g. `MotionPhysics::Gentle`, `MotionPhysics::Snappy`, `MotionPhysics::Off`).
    pub fn with_motion(mut self, motion: MotionPhysics) -> Self {
        self.set_motion(motion);
        self
    }

    /// Dynamically updates the motion physics mode at runtime.
    pub fn set_motion(&mut self, motion: MotionPhysics) {
        self.motion = motion;
        if let Some(params) = motion.to_params() {
            self.corners.base_stiffness = params.angular_frequency;
            self.corners.base_damping = params.damping_ratio;
        } else {
            // When Off, instantly snap to current target rect and rounding
            let target = self.corners.target_rect;
            self.corners.reset(target);
            self.start_center = target.center();
            self.target_center = target.center();
            self.current_rounding = self.target_rounding;
            self.start_rounding = self.target_rounding;
            self.alpha_spring.reset(1.0);
        }
    }

    /// Sets the base physical spring dynamics parameters.
    pub fn with_params(mut self, params: SpringParams) -> Self {
        self.set_params(params);
        self
    }

    /// Dynamically updates the spring physical parameters at runtime.
    pub fn set_params(&mut self, params: SpringParams) {
        self.corners.base_stiffness = params.angular_frequency;
        self.corners.base_damping = params.damping_ratio;
        self.motion = MotionPhysics::Custom(params);
    }

    /// Sets the fill color.
    pub fn with_fill(mut self, fill: Color32) -> Self {
        self.fill_color = fill;
        self
    }

    /// Dynamically updates the fill color at runtime.
    pub fn set_fill(&mut self, fill: Color32) {
        self.fill_color = fill;
    }

    /// Sets the border stroke.
    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Dynamically updates the border stroke at runtime.
    pub fn set_stroke(&mut self, stroke: Stroke) {
        self.stroke = stroke;
    }

    /// Sets uniform corner rounding radius.
    pub fn with_rounding(mut self, rounding: f32) -> Self {
        let r = Rounding::same(rounding.max(0.0));
        self.target_rounding = r;
        self.current_rounding = r;
        self.start_rounding = r;
        self
    }

    /// Sets per-corner rounding for asymmetric / irregular shapes.
    pub fn with_corner_rounding(mut self, rounding: Rounding) -> Self {
        self.target_rounding = rounding;
        self.current_rounding = rounding;
        self.start_rounding = rounding;
        self
    }

    /// Sets the target bounding expansion padding (default: 3.0 px).
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Dynamically updates the padding expansion at runtime.
    pub fn set_padding(&mut self, padding: f32) {
        self.padding = padding;
    }

    /// Retargets the spring highlight to a new bounding rectangle.
    pub fn set_target(&mut self, target: Rect) {
        self.set_target_with_corner_rounding(target, self.target_rounding);
    }

    /// Retargets the spring highlight with uniform target rounding for morphing.
    pub fn set_target_with_rounding(&mut self, target: Rect, rounding: f32) {
        self.set_target_with_corner_rounding(target, Rounding::same(rounding));
    }

    /// Retargets the spring highlight with asymmetric per-corner rounding for morphing.
    pub fn set_target_with_corner_rounding(&mut self, target: Rect, rounding: Rounding) {
        let padded = target.expand(self.padding);

        if self.motion.is_off() {
            self.corners.reset(padded);
            self.corners.target_rect = padded;
            self.target_rounding = rounding;
            self.current_rounding = rounding;
            self.start_rounding = rounding;
            self.start_center = padded.center();
            self.target_center = padded.center();
            self.alpha_spring.reset(1.0);
            return;
        }

        let new_target_center = padded.center();

        let center_delta = (new_target_center - self.target_center).length();
        let rounding_delta = (rounding.nw - self.target_rounding.nw).abs()
            + (rounding.ne - self.target_rounding.ne).abs()
            + (rounding.se - self.target_rounding.se).abs()
            + (rounding.sw - self.target_rounding.sw).abs();

        if center_delta > 1.0 || rounding_delta > 0.1 {
            self.start_center = self.corners.center();
            self.start_rounding = self.current_rounding;
            self.target_center = new_target_center;
            self.target_rounding = rounding;
        }

        self.corners.target_rect = padded;
        if !self.corners.initialized {
            self.reset(target);
            self.current_rounding = rounding;
            self.target_rounding = rounding;
            self.start_rounding = rounding;
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
        if self.motion.is_off() {
            let padded_target = self.corners.target_rect;
            self.corners.reset(padded_target);
            self.current_rounding = self.target_rounding;
            self.alpha_spring.reset(1.0);
            return;
        }

        let padded_target = self.corners.target_rect;
        self.corners.update(padded_target, dt);
        self.alpha_spring.update(dt);

        // Morph rounding per-corner smoothly in direct proportion to spatial flight progress
        let total_dist = (self.target_center - self.start_center).length();
        if total_dist > 2.0 {
            let current_dist = (self.target_center - self.corners.center()).length();
            let raw_progress = (1.0 - (current_dist / total_dist)).clamp(0.0, 1.0);
            // Hermite smoothstep for natural organic curve interpolation
            let smooth_progress = raw_progress * raw_progress * (3.0 - 2.0 * raw_progress);
            self.current_rounding = Rounding {
                nw: self.start_rounding.nw + (self.target_rounding.nw - self.start_rounding.nw) * smooth_progress,
                ne: self.start_rounding.ne + (self.target_rounding.ne - self.start_rounding.ne) * smooth_progress,
                se: self.start_rounding.se + (self.target_rounding.se - self.start_rounding.se) * smooth_progress,
                sw: self.start_rounding.sw + (self.target_rounding.sw - self.start_rounding.sw) * smooth_progress,
            };
        } else {
            self.current_rounding = self.target_rounding;
        }
    }

    /// Checks if the animated highlight has settled at its target position.
    pub fn is_settled(&self) -> bool {
        if self.motion.is_off() {
            true
        } else {
            self.corners.is_settled() && self.alpha_spring.is_settled()
        }
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
