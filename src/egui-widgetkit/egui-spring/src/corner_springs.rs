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
    pub fn new(pos: Pos2, stiffness: f32, damping: f32) -> Self {
        let params = SpringParams::new(stiffness, damping);
        Self {
            x: Spring::new(pos.x, params),
            y: Spring::new(pos.y, params),
        }
    }

    /// Advances the point simulation towards target with overridden stiffness and damping.
    pub fn update(&mut self, target: Pos2, dt: f32, stiffness: f32, damping: f32) {
        self.x.params = SpringParams::new(stiffness, damping);
        self.y.params = SpringParams::new(stiffness, damping);
        self.x.set_target(target.x);
        self.y.set_target(target.y);
        self.x.update(dt);
        self.y.update(dt);
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
/// Corners leading in the travel direction stretch faster, while trailing corners lag,
/// producing the signature Neovide / OpenRGB elastic smear effect.
#[derive(Clone, Debug, PartialEq)]
pub struct CornerSprings {
    /// Four corner points: [Top-Left, Top-Right, Bottom-Right, Bottom-Left].
    pub corners: [SpringPoint; 4],
    /// Base natural frequency $\omega_0$ (stiffness).
    pub base_stiffness: f32,
    /// Base dimensionless damping ratio $\zeta$.
    pub base_damping: f32,
    /// Target rectangle equilibrium.
    pub target_rect: Rect,
    /// Whether the corners have been initialized to their first target.
    pub initialized: bool,
}

impl Default for CornerSprings {
    fn default() -> Self {
        Self::new(Rect::ZERO, 22.0, 0.65)
    }
}

impl CornerSprings {
    /// Creates a new `CornerSprings` set targeted at `rect`.
    pub fn new(rect: Rect, base_stiffness: f32, base_damping: f32) -> Self {
        let dummy = Pos2::ZERO;
        Self {
            corners: [
                SpringPoint::new(dummy, base_stiffness, base_damping),
                SpringPoint::new(dummy, base_stiffness, base_damping),
                SpringPoint::new(dummy, base_stiffness, base_damping),
                SpringPoint::new(dummy, base_stiffness, base_damping),
            ],
            base_stiffness,
            base_damping,
            target_rect: rect,
            initialized: false,
        }
    }

    /// Retargets all four corner springs to a new target rectangle.
    pub fn set_target(&mut self, target: Rect) {
        self.target_rect = target;
        if !self.initialized {
            self.reset(target);
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

    /// Advances the corner physics towards `target_rect` using exact OpenRGB/Neovide elastic smear math.
    pub fn update(&mut self, target: Rect, dt: f32) {
        let target_corners = [
            target.left_top(),
            target.right_top(),
            target.right_bottom(),
            target.left_bottom(),
        ];

        if !self.initialized {
            for i in 0..4 {
                self.corners[i].reset(target_corners[i]);
            }
            self.initialized = true;
            return;
        }

        // Calculate travel direction based on centers
        let current_center = self.center();
        let target_center = target.center();
        let travel_vec = target_center - current_center;
        let travel_dist = travel_vec.length();

        // When resizing / morphing between different sizes (e.g. caret <-> card), expand/shrink symmetrically without directional bias
        let current_size = (self.corners[2].pos() - self.corners[0].pos()).abs();
        let target_size = target.size();
        let is_resizing = (current_size.x - target_size.x).abs() > 4.0 || (current_size.y - target_size.y).abs() > 4.0;

        let travel_dir = if travel_dist > 10.0 && !is_resizing {
            travel_vec / travel_dist
        } else {
            Vec2::ZERO
        };

        // Standard local corner direction vectors relative to center
        let corner_dirs = [
            Vec2::new(-1.0, -1.0), // TL
            Vec2::new(1.0, -1.0),  // TR
            Vec2::new(1.0, 1.0),   // BR
            Vec2::new(-1.0, 1.0),  // BL
        ];

        // Logarithmic scaling factor for the damping ratio based on travel distance.
        // We want the bounce (overshoot) to scale sub-linearly with distance.
        let d0 = 30.0;
        let log_factor = if travel_dist > d0 {
            (1.0 + d0).ln() / (1.0 + travel_dist).ln()
        } else {
            1.0
        };

        // The effective base damping ratio for this step (approaches 1.0 for large travel distances)
        let step_base_damping = 1.0 - (1.0 - self.base_damping) * log_factor;

        for i in 0..4 {
            let mut stiffness = self.base_stiffness;
            let mut damping = step_base_damping;

            if travel_dir != Vec2::ZERO {
                // Dot product of corner relative direction and travel direction.
                // Positive means this corner is leading the motion.
                let norm_corner_dir = corner_dirs[i].normalized();
                let alignment = travel_dir.dot(norm_corner_dir);

                // Smoothly scale stiffness based on alignment (avoiding step discontinuities)
                let factor = if alignment > 0.0 {
                    1.0 + alignment * 0.6
                } else {
                    1.0 + alignment * 0.3
                };
                stiffness = self.base_stiffness * factor;

                // Decrease damping ratio slightly for leading corners to let them stretch and bounce dynamically
                if alignment > 0.0 {
                    damping = step_base_damping * (1.0 - alignment * 0.15 * log_factor);
                }
            }

            self.corners[i].update(target_corners[i], dt, stiffness, damping);
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
