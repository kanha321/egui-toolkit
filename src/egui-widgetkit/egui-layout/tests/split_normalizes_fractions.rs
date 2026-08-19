use egui_layout::{Size, Split};

#[test]
fn test_fractions_not_summing_to_one() {
    let fractions = [1.0, 2.0, 1.0];
    let available = 420.0;
    let spacing = 10.0;
    // 3 sections => 2 gaps of 10 = 20. Usable space = 400.
    // 1 : 2 : 1 => 100, 200, 100
    let sizes = Split::compute_sizes(available, spacing, &fractions);
    assert_eq!(sizes.len(), 3);
    assert!((sizes[0] - 100.0).abs() < 1e-3);
    assert!((sizes[1] - 200.0).abs() < 1e-3);
    assert!((sizes[2] - 100.0).abs() < 1e-3);

    let total: f32 = sizes.iter().sum::<f32>() + (sizes.len() as f32 - 1.0) * spacing;
    assert!((total - available).abs() < 1e-3);
}

#[test]
fn test_odd_fraction_set_and_remainder_absorption() {
    let fractions = [0.33, 0.33, 0.34];
    let available = 1000.0;
    let spacing = 4.0;
    let sizes = Split::compute_sizes(available, spacing, &fractions);
    assert_eq!(sizes.len(), 3);

    let total: f32 = sizes.iter().sum::<f32>() + 2.0 * spacing;
    // Remainder absorption must guarantee exact total match
    assert_eq!(total, available);
}

#[test]
fn test_single_section() {
    let fractions = [0.5];
    let available = 500.0;
    let spacing = 10.0;
    let sizes = Split::compute_sizes(available, spacing, &fractions);
    assert_eq!(sizes.len(), 1);
    assert_eq!(sizes[0], available);
}

#[test]
fn test_all_zero_fractions_graceful_fallback() {
    let fractions = [0.0, 0.0, 0.0, 0.0];
    let available = 430.0;
    let spacing = 10.0;
    // 4 sections => 3 gaps of 10 = 30. Usable space = 400.
    // Equal fallback => 100 each
    let sizes = Split::compute_sizes(available, spacing, &fractions);
    assert_eq!(sizes.len(), 4);
    assert!((sizes[0] - 100.0).abs() < 1e-3);
    assert!((sizes[1] - 100.0).abs() < 1e-3);
    assert!((sizes[2] - 100.0).abs() < 1e-3);
    assert!((sizes[3] - 100.0).abs() < 1e-3);
}

#[test]
fn test_negative_fractions_clamped() {
    let fractions = [-1.0, 2.0, 0.0];
    let available = 210.0;
    let spacing = 10.0;
    // -1.0 clamped to 0.0, 0.0 is 0.0. Usable = 190.
    // 2.0 takes 100% of usable space.
    let sizes = Split::compute_sizes(available, spacing, &fractions);
    assert_eq!(sizes.len(), 3);
    assert_eq!(sizes[0], 0.0);
    assert_eq!(sizes[1], 190.0);
    assert_eq!(sizes[2], 0.0);
}

#[test]
fn test_min_size_constraint_prevents_clipping() {
    // 142px container, 6px spacing => 136px usable
    // Header is 20% (normally 27.2px), but has min 50px floor!
    // Remainder workspace absorbs the remaining 86px
    let policies = [
        Size::fraction(0.20).min_size(50.0),
        Size::remainder(),
    ];
    let available = 142.0;
    let spacing = 6.0;
    let sizes = Split::compute_sizes_with_policy(available, spacing, &policies);

    assert_eq!(sizes.len(), 2);
    assert!((sizes[0] - 50.0).abs() < 1e-3, "Header must not shrink below 50px min constraint");
    assert!((sizes[1] - 86.0).abs() < 1e-3, "Workspace must absorb the remaining space");

    let total: f32 = sizes[0] + sizes[1] + spacing;
    assert_eq!(total, available);
}

#[test]
fn test_fixed_and_remainder_policies() {
    let policies = [
        Size::exact(48.0),
        Size::remainder(),
    ];
    let available = 300.0;
    let spacing = 10.0;
    // 300 - 10 = 290 usable => 48 fixed + 242 remainder
    let sizes = Split::compute_sizes_with_policy(available, spacing, &policies);

    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0], 48.0);
    assert_eq!(sizes[1], 242.0);
}

#[test]
fn test_compute_min_length() {
    let policies = [
        Size::fraction(0.24).min_size(150.0),
        Size::exact(48.0),
        Size::fraction(0.35).min_size(160.0),
    ];
    let spacing = 6.0;
    // 150 + 48 + 160 + (3 - 1) * 6 = 358 + 12 = 370.0
    let min_len = Split::compute_min_length(spacing, &policies);
    assert_eq!(min_len, 370.0);
}

