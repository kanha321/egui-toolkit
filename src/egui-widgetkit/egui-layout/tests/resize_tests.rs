use egui_layout::SplitState;
use spring_core::{Spring, SpringParams};

#[test]
fn test_split_state_default() {
    let state = SplitState::default();
    assert!(state.overrides.is_empty());
    assert!(state.springs.is_empty());
    assert_eq!(state.active_divider, None);
    assert_eq!(state.section_count, 0);
    assert!(!state.initialized);
    assert!(!state.is_animating());
}

#[test]
fn test_split_state_reset_clears_overrides() {
    let mut state = SplitState::default();
    state.overrides = vec![Some(100.0), Some(200.0), None];
    
    state.reset();
    
    assert_eq!(state.overrides, vec![None, None, None]);
}

#[test]
fn test_split_state_reset_section() {
    let mut state = SplitState::default();
    state.overrides = vec![Some(100.0), Some(200.0), Some(300.0)];
    
    state.reset_section(1);
    
    assert_eq!(state.overrides, vec![Some(100.0), None, Some(300.0)]);
}

#[test]
fn test_split_state_is_animating() {
    let mut state = SplitState::default();
    let mut spring = Spring::new(0.0, SpringParams::default());
    spring.reset(100.0);
    spring.set_target(100.0); // Settled
    
    state.springs = vec![spring];
    assert!(!state.is_animating());

    state.springs[0].set_target(200.0);
    assert!(state.is_animating());
}

#[test]
fn test_divider_visibility_modes() {
    use egui_layout::{DividerVisibility, Split, SplitStyle};

    let style = SplitStyle::default();
    assert_eq!(style.divider_visibility, DividerVisibility::HoverOnly);

    let style_vis = SplitStyle::default().with_divider_always_visible();
    assert_eq!(style_vis.divider_visibility, DividerVisibility::Visible);

    let style_hidden = SplitStyle::default().with_divider_hidden();
    assert_eq!(style_hidden.divider_visibility, DividerVisibility::Hidden);

    let style_hover = SplitStyle::default().with_divider_hover_only();
    assert_eq!(style_hover.divider_visibility, DividerVisibility::HoverOnly);

    let split = Split::horizontal().divider_always_visible();
    assert_eq!(split.get_style().divider_visibility, DividerVisibility::Visible);

    let split_hidden = Split::horizontal().divider_hidden();
    assert_eq!(split_hidden.get_style().divider_visibility, DividerVisibility::Hidden);

    let split_hover = Split::horizontal().divider_hover_only();
    assert_eq!(split_hover.get_style().divider_visibility, DividerVisibility::HoverOnly);
}
