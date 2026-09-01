use egui_layout::{CollapseMode, Section};

#[test]
fn test_section_collapsible_sets_fields() {
    let mut expanded = true;
    let section = Section::fraction(0.5)
        .collapsible(&mut expanded, CollapseMode::FixedBar(40.0));

    assert!(section.is_card);
    assert!(section.is_collapsible());
    assert!(section.collapse_config.is_some());
}

#[test]
fn test_collapse_mode_variants() {
    let mut expanded = true;
    
    let s1 = Section::new().collapsible(&mut expanded, CollapseMode::Hidden);
    if let Some(config) = s1.collapse_config {
        assert!(matches!(config.mode, CollapseMode::Hidden));
    } else {
        panic!("Missing config");
    }

    let mut expanded2 = true;
    let s2 = Section::new().collapsible(&mut expanded2, CollapseMode::FixedBar(50.0));
    if let Some(config) = s2.collapse_config {
        assert!(matches!(config.mode, CollapseMode::FixedBar(50.0)));
    } else {
        panic!("Missing config");
    }

    let mut expanded3 = true;
    let s3 = Section::new().collapsible(&mut expanded3, CollapseMode::HeaderOnly);
    if let Some(config) = s3.collapse_config {
        assert!(matches!(config.mode, CollapseMode::HeaderOnly));
    } else {
        panic!("Missing config");
    }
}

#[test]
fn test_section_is_collapsed() {
    let mut expanded = true;
    let s = Section::new().collapsible(&mut expanded, CollapseMode::Hidden);
    assert!(!s.is_collapsed());

    // Because Section holds a mutable reference to expanded, we can't easily change it
    // from the outside while Section holds it, but we can test by creating a new one.
    let mut collapsed = false;
    let s2 = Section::new().collapsible(&mut collapsed, CollapseMode::Hidden);
    assert!(s2.is_collapsed());

    let s3 = Section::new();
    assert!(!s3.is_collapsed()); // Non-collapsible is never collapsed
}

#[test]
fn test_section_no_chevron() {
    let mut expanded = true;
    let section = Section::new()
        .collapsible(&mut expanded, CollapseMode::Hidden)
        .no_chevron();

    assert_eq!(section.collapse_config.unwrap().show_chevron, false);
}
