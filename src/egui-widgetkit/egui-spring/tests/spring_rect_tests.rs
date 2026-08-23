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

#[test]
fn test_spring_rect_motion_physics_off() {
    let target1 = Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(50.0, 50.0));
    let target2 = Rect::from_min_size(Pos2::new(200.0, 300.0), egui::vec2(100.0, 80.0));

    let mut rect = SpringRect::off(target1).with_padding(0.0);
    assert!(rect.is_settled(), "Off highlight must be settled immediately");

    // Retargeting when Off must immediately update bounds without needing animation steps
    rect.set_target(target2);
    assert!(rect.is_settled(), "Off highlight must stay settled when retargeted");
    let bbox = rect.current_bounding_rect();
    assert!((bbox.min.x - target2.min.x).abs() < 1e-3);
    assert!((bbox.min.y - target2.min.y).abs() < 1e-3);
    assert!((bbox.max.x - target2.max.x).abs() < 1e-3);
    assert!((bbox.max.y - target2.max.y).abs() < 1e-3);
}

#[test]
fn test_highlight_group_independent_settings() {
    use egui::{Color32, Stroke};
    use egui_spring::{HighlightConfig, HighlightGroup, MotionPhysics};

    let mut group: HighlightGroup<&'static str> = HighlightGroup::new();

    // Layer 1: Gentle Blue outer panel
    group.add(
        "outer",
        HighlightConfig::new()
            .with_motion(MotionPhysics::Gentle)
            .with_stroke(Stroke::new(2.0, Color32::BLUE))
            .with_padding(4.0),
    );

    // Layer 2: Snappy Green item
    group.add(
        "inner",
        HighlightConfig::new()
            .with_motion(MotionPhysics::Snappy)
            .with_stroke(Stroke::new(1.5, Color32::GREEN))
            .with_padding(2.0),
    );

    // Layer 3: Instant/Off Yellow cursor
    group.add(
        "cursor",
        HighlightConfig::new()
            .with_motion(MotionPhysics::Off)
            .with_stroke(Stroke::new(1.0, Color32::YELLOW)),
    );

    assert_eq!(group.len(), 3);
    assert_eq!(group.get(&"outer").unwrap().motion, MotionPhysics::Gentle);
    assert_eq!(group.get(&"inner").unwrap().motion, MotionPhysics::Snappy);
    assert_eq!(group.get(&"cursor").unwrap().motion, MotionPhysics::Off);

    // Retarget all independently
    let r1 = Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(300.0, 300.0));
    let r2 = Rect::from_min_size(Pos2::new(10.0, 10.0), egui::vec2(80.0, 40.0));
    let r3 = Rect::from_min_size(Pos2::new(50.0, 20.0), egui::vec2(20.0, 20.0));

    group.set_target(&"outer", r1);
    group.set_target(&"inner", r2);
    group.set_target(&"cursor", r3);

    // Cursor is Off, so it's settled immediately
    assert!(group.get(&"cursor").unwrap().is_settled());

    // Update group by 600 steps to let all settle
    let mut steps = 0;
    while !group.is_settled() && steps < 600 {
        group.update(1.0 / 60.0);
        steps += 1;
    }

    assert!(group.is_settled(), "All independent highlights must settle");
}
