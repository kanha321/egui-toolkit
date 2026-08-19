use egui::{Pos2, Rect, Rounding};
use egui_spring::{build_bezier_boundary, CornerSprings, SpringRect};

#[test]
fn test_corner_springs_settling() {
    let mut corners = CornerSprings::new(Rect::ZERO, 28.0, 0.85);
    let target = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(200.0, 50.0));
    corners.set_target(target);

    let dt = 1.0 / 60.0;
    let mut steps = 0;
    while !corners.is_settled() && steps < 600 {
        corners.update(target, dt);
        steps += 1;
    }

    assert!(corners.is_settled(), "Corner springs must settle within 10 seconds");
    let pts = corners.positions();

    assert!((pts[0].x - target.left_top().x).abs() < 1e-2);
    assert!((pts[0].y - target.left_top().y).abs() < 1e-2);
    assert!((pts[2].x - target.right_bottom().x).abs() < 1e-2);
    assert!((pts[2].y - target.right_bottom().y).abs() < 1e-2);
}

#[test]
fn test_bezier_boundary_generation() {
    let corners = [
        Pos2::new(0.0, 0.0),
        Pos2::new(100.0, 0.0),
        Pos2::new(100.0, 50.0),
        Pos2::new(0.0, 50.0),
    ];

    let points = build_bezier_boundary(&corners, Rounding::same(8.0), 8);
    // 4 corners * 9 points per corner = 36 points
    assert_eq!(points.len(), 36);

    for pt in &points {
        assert!(!pt.x.is_nan());
        assert!(!pt.y.is_nan());
        assert!(pt.x >= -0.01 && pt.x <= 100.01);
        assert!(pt.y >= -0.01 && pt.y <= 50.01);
    }
}

#[test]
fn test_bezier_asymmetric_rounding() {
    let corners = [
        Pos2::new(0.0, 0.0),
        Pos2::new(100.0, 0.0),
        Pos2::new(100.0, 50.0),
        Pos2::new(0.0, 50.0),
    ];

    let asymmetric = Rounding { nw: 20.0, ne: 0.0, se: 20.0, sw: 0.0 };
    let points = build_bezier_boundary(&corners, asymmetric, 8);
    assert!(points.len() > 4);
    for pt in &points {
        assert!(!pt.x.is_nan());
        assert!(!pt.y.is_nan());
    }
}

#[test]
fn test_spring_rect_bounding_rect() {
    let target = Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(120.0, 80.0));
    let mut rect = SpringRect::new(target).with_padding(0.0);
    rect.reset(target);

    assert!(rect.is_settled());
    let bbox = rect.current_bounding_rect();
    assert!((bbox.min.x - target.min.x).abs() < 1e-3);
    assert!((bbox.max.x - target.max.x).abs() < 1e-3);
}
