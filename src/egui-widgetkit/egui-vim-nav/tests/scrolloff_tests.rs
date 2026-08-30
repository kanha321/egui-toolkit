use egui::{pos2, vec2, Rect};
use egui_vim_nav::{compute_scrolloff_delta, Direction, NavDirection, Scrolloff};
use spring_core::SpringParams;

#[test]
fn test_scrolloff_comfort_zone() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    // Widget comfortably in the middle (between y=36 and y=564)
    let target = Rect::from_min_size(pos2(10.0, 200.0), vec2(380.0, 40.0));

    let delta = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Down);
    assert_eq!(delta, 0.0);

    let delta_up = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Up);
    assert_eq!(delta_up, 0.0);
}

#[test]
fn test_scrolloff_bottom_clipped_normal() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    // Bottom threshold is 600 - 36 = 564. Target bottom is at 584 (overshoot = 20px)
    let target = Rect::from_min_size(pos2(10.0, 544.0), vec2(380.0, 40.0));

    let delta = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Down);
    assert_eq!(delta, 20.0);
}

#[test]
fn test_scrolloff_top_clipped_normal() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    // Top threshold is 0 + 36 = 36. Target top is at 16 (overshoot = 20px)
    let target = Rect::from_min_size(pos2(10.0, 16.0), vec2(380.0, 40.0));

    let delta = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Up);
    assert_eq!(delta, -20.0);
}

#[test]
fn test_scrolloff_oversized_widget_tiebreaker() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    // Available comfort zone height is 600 - 36 - 36 = 528.
    // Target height is 560 (top at 20, bottom at 580) -> clips BOTH top (<36) and bottom (>564)!
    let target = Rect::from_min_size(pos2(10.0, 20.0), vec2(380.0, 560.0));

    // Moving Down -> prioritizes bottom edge
    let delta_down = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Down);
    assert_eq!(delta_down, 580.0 - 564.0); // +16.0

    // Moving Up -> prioritizes top header
    let delta_up = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Up);
    assert_eq!(delta_up, -(36.0 - 20.0)); // -16.0

    // Unknown (mouse click / tab) -> prioritizes top header
    let delta_unk = compute_scrolloff_delta(target, viewport, 36.0, 36.0, NavDirection::Unknown);
    assert_eq!(delta_unk, -(36.0 - 20.0)); // -16.0
}

#[test]
fn test_scrolloff_state_transition_freezing() {
    let mut scrolloff = Scrolloff::<usize>::new()
        .with_margins(36.0, 36.0)
        .with_spring_params(SpringParams::new(24.0, 1.0));

    // Frame 1: Keypress Down transitions to widget 1
    scrolloff.record_nav_event(Some(1), Some(Direction::Down));
    assert_eq!(scrolloff.active_direction, NavDirection::Down);
    assert!(scrolloff.trigger_scrolloff);

    // Simulate Phase 3 running on Frame 1
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    let target = Rect::from_min_size(pos2(10.0, 570.0), vec2(380.0, 30.0));
    scrolloff.adjust_for_target(target, viewport, 2000.0);
    assert!(!scrolloff.trigger_scrolloff);
    assert_eq!(scrolloff.target_offset, 600.0 - 564.0); // 36.0

    // Frame 2: Mid-glide frame (no keypress, same widget)
    scrolloff.record_nav_event(Some(1), None);
    assert_eq!(scrolloff.active_direction, NavDirection::Down); // direction remains frozen!
    assert!(!scrolloff.trigger_scrolloff); // does not re-trigger!

    // Frame 3: Mouse click on widget 2 (no keypress, transition to widget 2)
    scrolloff.record_nav_event(Some(2), None);
    assert_eq!(scrolloff.active_direction, NavDirection::Unknown); // resets to Unknown
    assert!(!scrolloff.trigger_scrolloff, "mouse click should NOT trigger autoscroll by default");

    // Frame 4: When explicitly enabled, pointer focus DOES trigger autoscroll
    let mut pointer_scrolloff = Scrolloff::<usize>::new().with_autoscroll_on_pointer(true);
    pointer_scrolloff.record_nav_event(Some(2), None);
    assert!(pointer_scrolloff.trigger_scrolloff);
}

#[test]
fn test_scrolloff_max_content_clamping() {
    let mut scrolloff = Scrolloff::<usize>::new();
    scrolloff.record_nav_event(Some(1), Some(Direction::Down));

    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    let target = Rect::from_min_size(pos2(10.0, 900.0), vec2(380.0, 30.0));
    let content_height = 800.0; // max scroll = 800 - 600 = 200

    scrolloff.adjust_for_target(target, viewport, content_height);
    assert_eq!(scrolloff.target_offset, 200.0); // clamped cleanly to 200.0
}

#[test]
fn test_scrolloff_ensure_visible() {
    let mut scrolloff = Scrolloff::<usize>::new().with_margins(50.0, 50.0);

    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 600.0));
    // An expanded dropdown extending to Y=580 (within viewport 600, but past margin 550)
    let expanded_target = Rect::from_min_size(pos2(10.0, 300.0), vec2(380.0, 280.0)); // bottom = 580.0
    let content_height = 2000.0;

    let adjusted = scrolloff.ensure_visible(expanded_target, viewport, content_height);
    assert!(adjusted);
    // Delta should be 580.0 - (600.0 - 50.0) = 30.0
    assert_eq!(scrolloff.target_offset, 30.0);

    // Calling it again when already visible with margin returns false
    let target_in_view = Rect::from_min_size(pos2(10.0, 100.0), vec2(380.0, 200.0));
    let adjusted2 = scrolloff.ensure_visible(target_in_view, viewport, content_height);
    assert!(!adjusted2);
}
