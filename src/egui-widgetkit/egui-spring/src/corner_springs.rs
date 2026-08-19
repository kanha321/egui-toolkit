//! Four independent spring-animated corner points for elastic selection highlights.

use egui::{Pos2, Rect, Vec2};
use spring_core::{Spring, SpringParams};

/// Index constants for the 4 corners in clockwise order starting from Top-Left.
pub const CORNER_TOP_LEFT: usize = 0;
pub const CORNER_TOP_RIGHT: usize = 1;
pub const CORNER_BOTTOM_RIGHT: usize = 2;
pub const CORNER_BOTTOM_LEFT: usize = 3;

/// A 2D point animated by two independent 1D analytical springs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringPoint {
    pub x: Spring,
    pub y: Spring,
}

impl SpringPoint {
    /// Creates a new `SpringPoint` at the specified position.
    pub fn new(pos: Pos2, params: SpringParams) -> Self {
        Self {
            x: Spring::new(pos.x, params),
            y: Spring::new(pos.y, params),
        }
    }

    /// Advances the point simulation by delta time `dt`.
    pub fn update(&mut self, dt: f32) {
        self.x.update(dt);
        self.y.update(dt);
    }

    /// Sets the target position without zeroing existing velocity.
    pub fn set_target(&mut self, target: Pos2) {
        self.x.set_target(target.x);
        self.y.set_target(target.y);
    }

    /// Teleports the point immediately and zeroes velocity.
    pub fn reset(&mut self, pos: Pos2) {
        self.x.reset(pos.x);
        self.y.reset(pos.y);
    }

    /// Checks if both X and Y springs are settled.
    pub fn is_settled(&self) -> bool {
        self.x.is_settled() && self.y.is_settled()
    }

    /// Returns the current evaluated 2D position.
    pub fn pos(&self) -> Pos2 {
        Pos2::new(self.x.value(), self.y.value())
    }

    /// Returns current 2D velocity vector.
    pub fn velocity(&self) -> Vec2 {
        Vec2::new(self.x.velocity(), self.y.velocity())
    }
}

/// A set of four independent corner springs driving an elastic rectangle.
///
/// Corners leading in the travel direction stretch forward while trailing corners lag,
/// producing the natural physical "elastic smear" effect.
#[derive(Clone, Debug, PartialEq)]
pub struct CornerSprings {
    /// Four corner points: [Top-Left, Top-Right, Bottom-Right, Bottom-Left].
    pub corners: [SpringPoint; 4],
    /// Base physical spring parameters.
    pub params: SpringParams,
    /// Target rectangle equilibrium.
    pub target_rect: Rect,
    /// Whether the corners have been initialized to their first target.
    pub initialized: bool,
    /// Maximum allowed stretch/smear deformation clamp in pixels.
    pub max_smear: f32,
}

impl Default for CornerSprings {
    fn default() -> Self {
        Self::new(Rect::ZERO, SpringParams::snappy())
    }
}

impl CornerSprings {
    /// Creates a new `CornerSprings` set targeted at `rect`.
    pub fn new(rect: Rect, params: SpringParams) -> Self {
        let dummy = Pos2::ZERO;
        Self {
            corners: [
                SpringPoint::new(dummy, params),
                SpringPoint::new(dummy, params),
                SpringPoint::new(dummy, params),
                SpringPoint::new(dummy, params),
            ],
            params,
            target_rect: rect,
            initialized: false,
            max_smear: 40.0,
        }
    }

    /// Retargets all four corner springs to a new target rectangle.
    pub fn set_target(&mut self, target: Rect) {
        self.target_rect = target;
        if !self.initialized {
            self.reset(target);
            return;
        }

        let target_corners = [
            target.left_top(),
            target.right_top(),
            target.right_bottom(),
            target.left_bottom(),
        ];

        for i in 0..4 {
            self.corners[i].set_target(target_corners[i]);
        }
    }

    /// Teleports all 4 corners directly to `rect` and clears momentum.
    pub fn reset(&mut self, target: Rect) {
        self.target_rect = target;
        let target_corners = [
            target.left_top(),
            target.right_top(),
            target.right_bottom(),
            target.left_bottom(),
        ];

        for i in 0..4 {
            self.corners[i].reset(target_corners[i]);
        }
        self.initialized = true;
    }

    /// Advances the corner physics towards `target_rect` by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if !self.initialized {
            self.reset(self.target_rect);
            return;
        }

        let target_corners = [
            self.target_rect.left_top(),
            self.target_rect.right_top(),
            self.target_rect.right_bottom(),
            self.target_rect.left_bottom(),
        ];

        let current_center = self.center();
        let target_center = self.target_rect.center();
        let travel_vec = target_center - current_center;
        let travel_dist = travel_vec.length();

        let travel_dir = if travel_dist > 5.0 {
            travel_vec / travel_dist
        } else {
            Vec2::ZERO
        };

        // Corner direction vectors relative to center (clockwise: TL, TR, BR, BL)
        let corner_dirs = [
            Vec2::new(-1.0, -1.0).normalized(),
            Vec2::new(1.0, -1.0).normalized(),
            Vec2::new(1.0, 1.0).normalized(),
            Vec2::new(-1.0, 1.0).normalized(),
        ];

        // Sub-linear logarithmic damping scaling over large distances
        let d0 = 30.0;
        let log_factor = if travel_dist > d0 {
            (1.0 + d0).ln() / (1.0 + travel_dist).ln()
        } else {
            1.0
        };

        let step_base_damping = 1.0 - (1.0 - self.params.damping_ratio) * log_factor;

        for i in 0..4 {
            let mut corner_params = self.params;

            if travel_dir != Vec2::ZERO {
                let alignment = travel_dir.dot(corner_dirs[i]);

                // Leading corners stretch forward smoothly without slowing trailing corners
                let factor = if alignment > 0.0 {
                    1.0 + alignment * 0.45
                } else {
                    1.0
                };
                corner_params.angular_frequency = self.params.angular_frequency * factor;

                // Slightly lower damping on leading corners to enhance dynamic elastic stretch
                if alignment > 0.0 {
                    corner_params.damping_ratio = (step_base_damping * (1.0 - alignment * 0.15 * log_factor)).max(0.1);
                } else {
                    corner_params.damping_ratio = step_base_damping;
                }
            }

            self.corners[i].x.params = corner_params;
            self.corners[i].y.params = corner_params;
            self.corners[i].set_target(target_corners[i]);
            self.corners[i].update(dt);
        }
    }

    /// Computes the geometric centroid of the four animated corners.
    pub fn center(&self) -> Pos2 {
        let mut x = 0.0;
        let mut y = 0.0;
        for c in &self.corners {
            let p = c.pos();
            x += p.x;
            y += p.y;
        }
        Pos2::new(x * 0.25, y * 0.25)
    }

    /// Checks if all 4 corners have settled at their target rect positions.
    pub fn is_settled(&self) -> bool {
        self.initialized && self.corners.iter().all(|c| c.is_settled())
    }

    /// Returns the current positions of the 4 corners: [TL, TR, BR, BL].
    pub fn positions(&self) -> [Pos2; 4] {
        [
            self.corners[0].pos(),
            self.corners[1].pos(),
            self.corners[2].pos(),
            self.corners[3].pos(),
        ]
    }
}
