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

#[test]
fn test_spring_cursor_morphing_and_lifecycle() {
    use egui_spring::SpringCursor;
    use spring_core::SpringParams;

    let mut cursor = SpringCursor::new(SpringParams::snappy());
    let outer_card = Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(300.0, 40.0));
    let inner_caret = Rect::from_min_size(Pos2::new(120.0, 62.0), egui::vec2(10.0, 16.0));

    // 1. Spawn from outer card (Macro -> Micro morphing)
    cursor.spawn_from(outer_card, 6.0, inner_caret, 1.5);
    assert!(cursor.active);
    assert!(!cursor.is_settled());

    let dt = 1.0 / 60.0;
    let ctx = egui::Context::default();
    let mut steps = 0;

    while !cursor.is_settled() && steps < 600 {
        cursor.update(inner_caret, true, dt, &ctx);
        steps += 1;
    }

    assert!(cursor.is_settled(), "Cursor must settle to inner caret");
    let bounds = cursor.bounding_rect();
    assert!((bounds.min.x - inner_caret.min.x).abs() < 1e-2);
    assert!((bounds.min.y - inner_caret.min.y).abs() < 1e-2);
    assert!((bounds.max.x - inner_caret.max.x).abs() < 1e-2);
    assert!((bounds.max.y - inner_caret.max.y).abs() < 1e-2);

    // 2. Exit to outer card (Micro -> Macro exit)
    cursor.exit_to(outer_card, 6.0);
    assert!(!cursor.active);

    steps = 0;
    while !cursor.is_settled() && steps < 600 {
        cursor.update(outer_card, false, dt, &ctx);
        steps += 1;
    }

    assert!(cursor.is_settled(), "Cursor must settle upon exit");
    assert!(cursor.alpha_spring.value() < 1e-3, "Alpha must fade to 0 on exit");
}

#[test]
fn test_spring_cursor_snappy_morph_and_restore() {
    use egui_spring::SpringCursor;
    use spring_core::SpringParams;

    // Normal typing physics is gentle
    let gentle_params = SpringParams::gentle();
    let mut cursor = SpringCursor::new(gentle_params);
    assert_eq!(cursor.base_params, gentle_params);
    assert!(!cursor.is_morphing);

    let outer_box = Rect::from_min_size(Pos2::new(10.0, 10.0), egui::vec2(250.0, 36.0));
    let target_caret = Rect::from_min_size(Pos2::new(40.0, 20.0), egui::vec2(2.0, 16.0));

    // Spawn from outer box -> triggers Snappy morph physics!
    cursor.spawn_from(outer_box, 6.0, target_caret, 0.5);
    assert!(cursor.is_morphing);
    let snappy = SpringParams::snappy();
    assert_eq!(cursor.corners.base_stiffness, snappy.angular_frequency);
    assert_eq!(cursor.corners.base_damping, snappy.damping_ratio);

    let dt = 1.0 / 60.0;
    let ctx = egui::Context::default();
    let mut steps = 0;
    while cursor.is_morphing && steps < 600 {
        cursor.update(target_caret, true, dt, &ctx);
        steps += 1;
    }

    // Once collapsed to caret, is_morphing becomes false and base physics restored
    assert!(!cursor.is_morphing);
    assert_eq!(cursor.corners.base_stiffness, gentle_params.angular_frequency);
    assert_eq!(cursor.corners.base_damping, gentle_params.damping_ratio);
}

#[test]
fn test_spring_cursor_gentle_exit_physics() {
    use egui_spring::SpringCursor;
    use spring_core::SpringParams;

    let mut cursor = SpringCursor::new(SpringParams::snappy());
    let outer_box = Rect::from_min_size(Pos2::new(10.0, 10.0), egui::vec2(250.0, 36.0));
    let target_caret = Rect::from_min_size(Pos2::new(40.0, 20.0), egui::vec2(2.0, 16.0));

    cursor.spawn_from(outer_box, 6.0, target_caret, 0.5);

    // Call exit_to -> should engage Gentle physics for the outward expansion
    cursor.exit_to(outer_box, 6.0);
    assert!(!cursor.active);
    let gentle = SpringParams::gentle();
    assert_eq!(cursor.corners.base_stiffness, gentle.angular_frequency);
    assert_eq!(cursor.corners.base_damping, gentle.damping_ratio);
    assert_eq!(cursor.alpha_spring.params.angular_frequency, gentle.angular_frequency);
}
