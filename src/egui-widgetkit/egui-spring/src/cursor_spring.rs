//! Spring-animated fluid cursor with 4-corner directional lag and highlight morphing.
//!
//! Provides [`SpringCursor`], which simulates an elastic quadrilateral cursor driven by
//! four independent analytical corner springs ([`CornerSprings`]) using `spring-core`.
//!
//! # Macro-to-Micro Focus Morphing
//!
//! When focusing a text widget, [`SpringCursor::spawn_from`] initializes the 4 corners at the
//! exact outer widget bounding box and fluidly contracts them into the character caret position,
//! creating the visual effect of an energetic highlight peeling off from behind the card frame.

use egui::{pos2, Color32, Context, Painter, Pos2, Rect, Rounding, Shape, Stroke};
use spring_core::{Spring, SpringParams};

use crate::corner_springs::CornerSprings;

/// A spring-driven fluid cursor with 4-corner directional lag and macro-to-micro morphing.
#[derive(Clone, Debug, PartialEq)]
pub struct SpringCursor {
    /// Four independent corner springs driving the elastic quad polygon.
    pub corners: CornerSprings,
    /// Base/default spring parameters for normal cursor navigation and typing.
    pub base_params: SpringParams,
    /// Whether an outline-to-cursor morph is currently in flight.
    pub is_morphing: bool,
    /// Spring driving cursor opacity ($0.0 \to 1.0$).
    pub alpha_spring: Spring,
    /// Spring driving corner rounding ($6.0 \to 1.5$ during morphing).
    pub rounding_spring: Spring,
    /// Spring driving mode shape morphing (e.g. Block $\leftrightarrow$ Bar $\leftrightarrow$ Underline).
    pub mode_spring: Spring,
    /// Last target rectangle assigned to the cursor.
    pub target_rect: Rect,
    /// Whether the cursor is currently active/focused.
    pub active: bool,
    /// Whether the cursor was focused in the previous frame.
    was_active: bool,
}

impl Default for SpringCursor {
    fn default() -> Self {
        Self::new(SpringParams::new(24.0, 0.65))
    }
}

impl SpringCursor {
    /// Creates a new `SpringCursor` configured with the given spring parameters.
    pub fn new(params: SpringParams) -> Self {
        Self {
            corners: CornerSprings::new(Rect::ZERO, params.angular_frequency, params.damping_ratio),
            base_params: params,
            is_morphing: false,
            alpha_spring: Spring::new(0.0, SpringParams::new(28.0, 0.70)),
            rounding_spring: Spring::new(2.0, SpringParams::new(24.0, 0.65)),
            mode_spring: Spring::new(0.0, SpringParams::new(30.0, 0.60)),
            target_rect: Rect::ZERO,
            active: false,
            was_active: false,
        }
    }

    /// Configures spring stiffness and damping across all sub-springs.
    pub fn set_spring_params(&mut self, params: SpringParams) {
        self.base_params = params;
        if !self.is_morphing {
            self.corners.base_stiffness = params.angular_frequency;
            self.corners.base_damping = params.damping_ratio;
        }
    }

    /// Spawns the cursor by peeling off from an origin bounding box (e.g. outer widget highlight).
    ///
    /// Engages high-frequency [`SpringParams::snappy`] exclusively for the duration of the
    /// macro-to-micro collapse, then automatically reverts to [`Self::base_params`].
    pub fn spawn_from(
        &mut self,
        origin_rect: Rect,
        origin_rounding: f32,
        target_rect: Rect,
        target_rounding: f32,
    ) {
        self.is_morphing = true;
        let snappy = SpringParams::snappy();
        self.corners.base_stiffness = snappy.angular_frequency;
        self.corners.base_damping = snappy.damping_ratio;

        self.corners.reset(origin_rect);
        self.rounding_spring.reset(origin_rounding);
        self.alpha_spring.reset(1.0);

        self.target_rect = target_rect;
        self.corners.set_target(target_rect);
        self.rounding_spring.set_target(target_rounding);
        self.alpha_spring.set_target(1.0);
        self.active = true;
        self.was_active = true;
    }

    /// Retargets the cursor to a new character bounding box inside the text buffer.
    pub fn set_target(&mut self, target_rect: Rect, target_rounding: f32) {
        self.target_rect = target_rect;
        self.corners.set_target(target_rect);
        self.rounding_spring.set_target(target_rounding);
        self.alpha_spring.set_target(1.0);
        self.active = true;
    }

    /// Exits text focus, expanding the cursor outwards towards the parent bounding box as it fades.
    ///
    /// Uses [`SpringParams::gentle`] for smooth, silky outward flight back into the highlight frame.
    pub fn exit_to(&mut self, parent_rect: Rect, parent_rounding: f32) {
        self.target_rect = parent_rect;
        self.corners.set_target(parent_rect);
        self.rounding_spring.set_target(parent_rounding);
        self.alpha_spring.set_target(0.0);
        self.active = false;
        self.is_morphing = false;

        let gentle = SpringParams::gentle();
        self.corners.base_stiffness = gentle.angular_frequency;
        self.corners.base_damping = gentle.damping_ratio;
        self.alpha_spring.params = SpringParams::new(gentle.angular_frequency, gentle.damping_ratio);
    }

    /// Advances the cursor simulation towards its target.
    ///
    /// Requests a continuous frame repaint while any corner or parameter spring is moving.
    pub fn update(&mut self, target_rect: Rect, is_focused: bool, dt: f32, ctx: &Context) {
        if is_focused && !self.was_active {
            // If newly focused without explicit spawn_from, initialize smoothly
            if !self.corners.initialized {
                self.corners.reset(target_rect);
            }
            self.alpha_spring.set_target(1.0);
            self.active = true;
        } else if !is_focused && self.was_active {
            self.alpha_spring.set_target(0.0);
            self.active = false;
            self.is_morphing = false;
            let gentle = SpringParams::gentle();
            self.corners.base_stiffness = gentle.angular_frequency;
            self.corners.base_damping = gentle.damping_ratio;
            self.alpha_spring.params = SpringParams::new(gentle.angular_frequency, gentle.damping_ratio);
        }
        self.was_active = is_focused;

        if is_focused {
            self.target_rect = target_rect;
            self.corners.update(target_rect, dt);

            // Once the outline-to-cursor morph settles at the target caret rect,
            // restore the normal base spring parameters for text editing and typing.
            if self.is_morphing {
                let pts = self.corners.positions();
                let current_w = (pts[1].x - pts[0].x).abs();
                let target_w = target_rect.width().max(1.0);
                if self.corners.is_settled() || (current_w - target_w).abs() < 2.0 {
                    self.is_morphing = false;
                    self.corners.base_stiffness = self.base_params.angular_frequency;
                    self.corners.base_damping = self.base_params.damping_ratio;
                }
            }
        } else if !self.is_settled() {
            // Continue animating outward exit
            let current_target = self.target_rect;
            self.corners.update(current_target, dt);
        }

        self.alpha_spring.update(dt);
        self.rounding_spring.update(dt);
        self.mode_spring.update(dt);

        if !self.is_settled() {
            ctx.request_repaint();
        }
    }

    /// Returns `true` if all corner springs, opacity, and morph parameters have settled.
    pub fn is_settled(&self) -> bool {
        (!self.active || (self.corners.initialized && self.corners.is_settled()))
            && self.alpha_spring.is_settled()
            && self.rounding_spring.is_settled()
            && self.mode_spring.is_settled()
    }

    /// Returns the current evaluated corner positions: `[Top-Left, Top-Right, Bottom-Right, Bottom-Left]`.
    pub fn corner_positions(&self) -> [Pos2; 4] {
        self.corners.positions()
    }

    /// Computes the bounding rectangle encompassing all 4 animated corners.
    pub fn bounding_rect(&self) -> Rect {
        let pts = self.corner_positions();
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in pts {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y))
    }

    /// Paints the morphing, elastic cursor polygon onto the given egui painter.
    pub fn paint(
        &self,
        painter: &Painter,
        fill_color: Color32,
        stroke: Stroke,
    ) {
        let alpha = self.alpha_spring.value().clamp(0.0, 1.0);
        if alpha <= 0.005 {
            return;
        }

        let pts = self.corner_positions();
        let current_w = (pts[1].x - pts[0].x).abs().max(1.0);

        // Continuous transition between solid caret and outline highlight:
        // When current_w <= 6.0px (caret scale), fill_factor is 1.0 (solid fill).
        // As current_w expands beyond 6.0px towards 26.0px, fill_factor fades to 0.0 (outline stroke).
        let fill_factor = (1.0 - (current_w - 6.0).max(0.0) / 20.0).clamp(0.0, 1.0);

        let effective_fill_alpha = alpha * fill_factor;

        let dynamic_fill = Color32::from_rgba_premultiplied(
            (fill_color.r() as f32 * effective_fill_alpha) as u8,
            (fill_color.g() as f32 * effective_fill_alpha) as u8,
            (fill_color.b() as f32 * effective_fill_alpha) as u8,
            (fill_color.a() as f32 * effective_fill_alpha) as u8,
        );

        let stroke_w = if current_w > 12.0 {
            stroke.width.max(1.5)
        } else {
            stroke.width
        };

        let dynamic_stroke = Stroke::new(
            stroke_w,
            Color32::from_rgba_premultiplied(
                (stroke.color.r() as f32 * alpha) as u8,
                (stroke.color.g() as f32 * alpha) as u8,
                (stroke.color.b() as f32 * alpha) as u8,
                (stroke.color.a() as f32 * alpha) as u8,
            ),
        );

        let rounding = self.rounding_spring.value().max(1.5);
        let polygon_points = crate::bezier::build_bezier_boundary(&pts, Rounding::same(rounding), 8);

        if polygon_points.len() >= 3 {
            if dynamic_fill.a() > 0 {
                painter.add(Shape::convex_polygon(
                    polygon_points.clone(),
                    dynamic_fill,
                    Stroke::NONE,
                ));
            }
            if dynamic_stroke.width > 0.0 && dynamic_stroke.color.a() > 0 {
                painter.add(Shape::closed_line(
                    polygon_points,
                    dynamic_stroke,
                ));
            }
        }
    }
}
