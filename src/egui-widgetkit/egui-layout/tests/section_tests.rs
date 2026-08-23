use egui::{Color32, Rounding, Stroke, Vec2};
use egui_layout::{Section, Split};

#[test]
fn test_section_builder_defaults_and_setters() {
    let s = Section::fraction(0.25)
        .min_size(120.0)
        .max_size(400.0)
        .min_cross(80.0)
        .max_cross(500.0)
        .card()
        .title("Test Section")
        .subtitle("Subtitle")
        .bg(Color32::from_rgb(10, 20, 30))
        .stroke(Stroke::new(2.0, Color32::RED))
        .rounding(Rounding::same(12.0))
        .padding(14.0);

    assert_eq!(s.primary_min(), 120.0);
    assert_eq!(s.cross_min(), 80.0);
    assert!(s.is_card);
    assert_eq!(s.title.as_deref(), Some("Test Section"));
    assert_eq!(s.subtitle.as_deref(), Some("Subtitle"));
    assert_eq!(s.bg, Some(Color32::from_rgb(10, 20, 30)));
    assert_eq!(s.padding, Some(14.0));
}

#[test]
fn test_split_min_size_2d_horizontal() {
    let split = Split::horizontal()
        .spacing(6.0)
        .add_section(Section::fraction(0.3).min_size_2d(100.0, 50.0))
        .add_section(Section::fixed(200.0).min_cross(120.0))
        .add_section(Section::remainder().min_size_2d(150.0, 80.0));

    // Primary length (width): 100 + 200 + 150 + (2 * 6.0) = 462.0
    // Cross length (height): max(50.0, 120.0, 80.0) = 120.0
    let min_size = split.min_size();
    assert_eq!(min_size.x, 462.0);
    assert_eq!(min_size.y, 120.0);
    assert_eq!(split.min_width(), 462.0);
    assert_eq!(split.min_height(), 120.0);
}

#[test]
fn test_split_min_size_2d_vertical() {
    let split = Split::vertical()
        .spacing(4.0)
        .add_section(Section::fixed(48.0).min_cross(150.0))
        .add_section(Section::fraction(0.7).min_size_2d(200.0, 300.0));

    // Primary length (height): 48 + 200 + (1 * 4.0) = 252.0
    // Cross length (width): max(150.0, 300.0) = 300.0
    let min_size = split.min_size();
    assert_eq!(min_size.x, 300.0);
    assert_eq!(min_size.y, 252.0);
    assert_eq!(split.min_width(), 300.0);
    assert_eq!(split.min_height(), 252.0);
}

#[test]
fn test_split_sections_equal_generator() {
    let split = Split::horizontal()
        .spacing(5.0)
        .sections_equal(4, |_idx, _ui| {});

    assert_eq!(split.len(), 4);
    let sizes = Split::compute_sizes(415.0, 5.0, &[0.25, 0.25, 0.25, 0.25]);
    // 415 - (3 * 5) = 400. Each gets 100.0
    assert_eq!(sizes, vec![100.0, 100.0, 100.0, 100.0]);
}

#[test]
fn test_split_sections_proportional_generator() {
    let split = Split::vertical()
        .spacing(10.0)
        .sections_proportional(&[1.0, 2.0, 1.0], |_idx, _fraction, _ui| {});

    assert_eq!(split.len(), 3);
    let sizes = Split::compute_sizes(420.0, 10.0, &[1.0, 2.0, 1.0]);
    // 420 - 20 = 400. 1/4 = 100, 2/4 = 200, 1/4 = 100
    assert_eq!(sizes, vec![100.0, 200.0, 100.0]);
}

#[test]
fn test_nested_split_min_size_propagation() {
    // Inner workspace split
    let inner = Split::horizontal()
        .spacing(6.0)
        .add_section(Section::remainder().min_size_2d(140.0, 80.0))
        .add_section(Section::fraction(0.35).min_size_2d(160.0, 100.0));

    let inner_min = inner.min_size();
    // Inner width: 140 + 160 + 6 = 306.0
    // Inner height: max(80, 100) = 100.0
    assert_eq!(inner_min, Vec2::new(306.0, 100.0));

    // Outer split nesting the inner split's min bounds
    let outer = Split::horizontal()
        .spacing(6.0)
        .add_section(Section::fraction(0.24).min_size_2d(150.0, 120.0))
        .add_section(Section::remainder().min_size_2d(inner_min.x, inner_min.y));

    let outer_min = outer.min_size();
    // Outer width: 150 + 306 + 6 = 462.0
    // Outer height: max(120, 100) = 120.0
    assert_eq!(outer_min, Vec2::new(462.0, 120.0));
}
