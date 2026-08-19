//! Bézier-rounded boundary polygon construction for deformed 4-corner spring highlights.

use egui::{Pos2, Rounding, Vec2};

/// Cubic Bézier circle-arc approximation constant: $\kappa = \frac{4}{3}(\sqrt{2} - 1) \approx 0.55228475$.
pub const BEZIER_KAPPA: f32 = 0.55228475;

/// Subdivisions per corner arc for smooth rendering.
pub const DEFAULT_ARC_SEGMENTS: usize = 8;

/// Builds a closed polygon of vertices from 4 animated corners using cubic Bézier arcs with per-corner rounding.
///
/// Ensures all 4 rounded corners remain intact, smooth, and convex across asymmetric shapes, dynamic deformation, and resizing.
pub fn build_bezier_boundary(
    corners: &[Pos2; 4],
    rounding: Rounding,
    arc_segments: usize,
) -> Vec<Pos2> {
    let corner_radii = [rounding.nw, rounding.ne, rounding.se, rounding.sw];
    let segments = arc_segments.max(2);
    let mut points = Vec::with_capacity(4 * (segments + 1));

    for i in 0..4 {
        let r_corner = corner_radii[i].max(0.0);
        let p_prev = corners[(i + 3) % 4];
        let p_curr = corners[i];
        let p_next = corners[(i + 1) % 4];

        if r_corner <= 0.001 {
            points.push(p_curr);
            continue;
        }

        let d_prev = p_curr - p_prev;
        let len_prev = d_prev.length();
        let u_prev = if len_prev > 1e-4 { d_prev / len_prev } else { Vec2::ZERO };

        let d_next = p_next - p_curr;
        let len_next = d_next.length();
        let u_next = if len_next > 1e-4 { d_next / len_next } else { Vec2::ZERO };

        // Corner rounding invariant: ensure radius does not exceed half edge length
        let r_eff = r_corner.min(len_prev.min(len_next) * 0.5);

        let a = p_curr - u_prev * r_eff;
        let b = p_curr + u_next * r_eff;

        // Cubic Bézier control points using kappa circle-arc factor
        let cp1 = a + u_prev * (BEZIER_KAPPA * r_eff);
        let cp2 = b - u_next * (BEZIER_KAPPA * r_eff);

        for step in 0..=segments {
            let t = step as f32 / segments as f32;
            let omt = 1.0 - t;
            let pt = Pos2::new(
                omt * omt * omt * a.x
                    + 3.0 * omt * omt * t * cp1.x
                    + 3.0 * omt * t * t * cp2.x
                    + t * t * t * b.x,
                omt * omt * omt * a.y
                    + 3.0 * omt * omt * t * cp1.y
                    + 3.0 * omt * t * t * cp2.y
                    + t * t * t * b.y,
            );
            points.push(pt);
        }
    }

    points
}
