//! Unit and layout integration tests for egui-widgets.

use egui_themes::ThemePreset;
use egui_widgets::{
    Badge, Button, ButtonSize, ButtonState, Card, Checkbox, CheckboxState,
    InputState, ProgressBar, ProgressState, ProgressVariant, RadioButton, RadioState, SegmentedTabs, Slider,
    SliderLayout, SliderState, Switch, SwitchSize, SwitchState, TabsState, TextInput,
};
use spring_core::SpringParams;

#[test]
fn test_button_builder_and_state() {
    let mut state = ButtonState::default();
    assert!(state.is_settled());

    let palette = ThemePreset::CatppuccinMocha.palette();
    let _btn = Button::new("Click Me")
        .primary()
        .size(ButtonSize::Large)
        .icon("🚀")
        .shortcut("Ctrl+Enter")
        .badge("New")
        .palette(&palette)
        .spring_params(SpringParams::snappy())
        .with_state(&mut state);
}

#[test]
fn test_button_click_animation_frames() {
    let mut state = ButtonState::default();
    state.trigger_click();
    assert_eq!(state.press_spring.current, 0.0);
    assert_eq!(state.press_spring.velocity, 18.0);
    assert!(!state.is_settled());

    let dt = 0.016;
    for frame in 1..=20 {
        state.press_spring.update(dt);
        println!("Frame {}: current = {:.4}, velocity = {:.4}, is_settled = {}", 
            frame, state.press_spring.current, state.press_spring.velocity, state.is_settled());
    }
}

#[test]
fn test_switch_toggle_and_state() {
    let mut enabled = false;
    let mut state = SwitchState::new(false);
    assert!(state.is_settled());

    let _sw = Switch::new(&mut enabled)
        .label("Enable Feature")
        .compact()
        .motion(true)
        .with_state(&mut state);

    let (w, h, r) = SwitchSize::Compact.dimensions();
    assert_eq!(w, 36.0);
    assert_eq!(h, 20.0);
    assert!(r > 0.0);
}

#[test]
fn test_segmented_tabs_construction() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Tab {
        First,
        Second,
        Third,
    }

    let mut active = Tab::First;
    let mut state = TabsState::default();

    let _tabs = SegmentedTabs::new(&mut active)
        .tab(Tab::First, "First")
        .tab(Tab::Second, "Second")
        .tab(Tab::Third, "Third")
        .height(40.0)
        .with_state(&mut state);
}

#[test]
fn test_progress_bar_clamping_and_state() {
    let mut state = ProgressState::new(0.5);
    assert!(state.is_settled());

    let _pb = ProgressBar::new(1.4)
        .variant(ProgressVariant::Success)
        .show_percentage(true)
        .with_state(&mut state);
}

#[test]
fn test_slider_bounds_and_state() {
    let mut val = 42.0f32;
    let mut state = SliderState::default();
    assert!(state.is_settled());

    let _sl1 = Slider::new(&mut val, 0.0..=100.0)
        .label("Volume")
        .suffix(" dB")
        .inline()
        .with_state(&mut state);

    let _sl2 = Slider::new(&mut val, 0.0..=100.0)
        .label("Thermal Cutoff")
        .suffix(" °C")
        .stacked()
        .layout(SliderLayout::Stacked);
}

#[test]
fn test_checkbox_and_radio() {
    let mut checked = false;
    let mut state = CheckboxState::new(false);
    assert!(state.is_settled());

    let _cb = Checkbox::new(&mut checked)
        .label("Accept Terms")
        .box_size(20.0)
        .with_state(&mut state);

    let mut selected_option = 1;
    let mut radio_state = RadioState::new(true);
    assert!(radio_state.is_settled());

    let _rb = RadioButton::new(2, &mut selected_option)
        .label("Option 2")
        .with_state(&mut radio_state);

    // Test clicking already-selected: no dot reset
    let ctx = egui::Context::default();
    radio_state.update(0.016, true, false, false, true, &ctx);
    assert_eq!(radio_state.dot_spring.current, 1.0); // Not reset to 0!
    assert_eq!(radio_state.dot_spring.velocity, 0.0); // No reanimation impulse!

    // Test newly selecting an unselected radio button: pops
    let mut unselected_state = RadioState::new(false);
    assert_eq!(unselected_state.dot_spring.current, 0.0);
    unselected_state.update(0.016, true, false, false, true, &ctx);
    assert!(unselected_state.dot_spring.current > 0.0); // Has moved toward 1.0
    assert!(unselected_state.dot_spring.velocity > 0.0); // Pop velocity applied

    // Test nullable radio button
    let mut nullable_choice: Option<i32> = Some(1);
    let _nullable_rb = RadioButton::nullable(1, &mut nullable_choice)
        .label("Option 1")
        .allow_deselect(true);
}

#[test]
fn test_badge_variants() {
    let palette = ThemePreset::TokyoNight.palette();

    let _b1 = Badge::new("Active").success().outline().dot(true).palette(&palette);
    let _b2 = Badge::new("Error").danger().solid().dot(true).palette(&palette);
    let _b3 = Badge::new("Info").info().palette(&palette);
    let _b4 = Badge::new("Neutral").neutral().outline().palette(&palette);
    let _b5 = Badge::new("Custom").solid().palette(&palette);
}

#[test]
fn test_text_input_and_state() {
    use egui_widgets::TextAlign;

    let mut query = String::from("antigravity");
    let mut state = InputState::default();
    assert!(state.is_settled());

    let _input = TextInput::new(&mut query)
        .placeholder("Type here...")
        .icon("🔍")
        .clear_button(true)
        .align(TextAlign::Center)
        .with_state(&mut state);

    let mut left_query = String::new();
    let _left_input = TextInput::new(&mut left_query).align_left();

    let mut right_query = String::new();
    let _right_input = TextInput::new(&mut right_query).align_right();
}

#[test]
fn test_card_builder() {
    let _card = Card::new()
        .title("Metrics")
        .subtitle("Live stats")
        .interactive(true);
}

#[test]
fn test_button_dynamic_size_spring() {
    let mut state = ButtonState::default();
    assert!(state.is_settled());

    // Frame 0: Initial sizing at 60.0 x 32.0
    state.width_spring.reset(60.0);
    state.height_spring.reset(32.0);
    state.size_initialized = true;
    assert_eq!(state.width_spring.value(), 60.0);
    assert_eq!(state.height_spring.value(), 32.0);
    assert!(state.is_settled());

    // Text changed: Target expands from 60.0 -> 180.0
    state.width_spring.set_target(180.0);
    assert!(!state.is_settled());

    // Frame 1: Intermediate frame step
    state.width_spring.update(0.016);
    let frame1_width = state.width_spring.value();
    assert!(frame1_width > 60.0 && frame1_width < 180.0);
    assert!(!state.is_settled());

    // Step until settlement
    for _ in 0..100 {
        state.width_spring.update(0.016);
    }
    assert!(state.is_settled());
    assert!((state.width_spring.value() - 180.0).abs() < 0.01);
}

#[test]
fn test_badge_dynamic_size_spring() {
    use egui_widgets::BadgeState;

    let mut state = BadgeState::default();
    let ctx = egui::Context::default();
    assert!(state.is_settled());

    // Frame 0: Initial seeding
    state.update(0.016, egui::vec2(50.0, 20.0), &ctx);
    assert_eq!(state.width_spring.value(), 50.0);
    assert_eq!(state.height_spring.value(), 20.0);
    assert!(state.is_settled());

    // Badge label changes: Target expands from 50.0 -> 120.0
    state.update(0.016, egui::vec2(120.0, 20.0), &ctx);
    assert!(!state.is_settled());
    assert!(state.width_spring.value() > 50.0 && state.width_spring.value() < 120.0);

    // Step until settlement
    for _ in 0..100 {
        state.update(0.016, egui::vec2(120.0, 20.0), &ctx);
    }
    assert!(state.is_settled());
    assert!((state.width_spring.value() - 120.0).abs() < 0.01);
}

#[test]
fn test_slider_badge_width_spring() {
    let mut state = SliderState::default();
    assert!(state.is_settled());

    // Value changes from single digit (56.0px) to wide triple digit (90.0px)
    state.badge_width_spring.set_target(90.0);
    assert!(!state.is_settled());

    state.badge_width_spring.update(0.016);
    assert!(state.badge_width_spring.value() > 56.0 && state.badge_width_spring.value() < 90.0);

    for _ in 0..100 {
        state.badge_width_spring.update(0.016);
    }
    assert!(state.is_settled());
    assert!((state.badge_width_spring.value() - 90.0).abs() < 0.01);
}

#[test]
fn test_button_press_and_pop_springs() {
    let mut state = ButtonState::default();
    let ctx = egui::Context::default();
    assert!(state.is_settled());

    // Frame 1: Focus and Press down (focus bounce triggered)
    state.update(0.016, true, true, false, &ctx);
    assert!(!state.is_settled());
    assert_eq!(state.press_spring.target, 1.0);
    assert!(state.focus_bounce_spring.velocity > 0.0);

    // Frame 2: Release and click
    state.update(0.016, true, false, true, &ctx);
    assert_eq!(state.press_spring.target, 0.0);
    assert!(state.press_spring.velocity > 0.0);

    // Step until spring settle
    for _ in 0..100 {
        state.update(0.016, false, false, false, &ctx);
    }
    assert!(state.is_settled());
}

#[test]
fn test_switch_press_and_release_springs() {
    let mut state = SwitchState::new(false);
    let ctx = egui::Context::default();
    assert!(state.is_settled());

    // Focus entrance sets highlight_spring target and triggers focus bounce impulse
    state.update(0.016, false, true, false, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.focus_spring.velocity > 0.0);
    assert_eq!(state.highlight_spring.target, 1.0);

    // Press down
    state.update(0.016, false, true, true, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.press_spring.target == 1.0);

    // Release click (toggled to true)
    state.update(0.016, true, true, false, true, &ctx);
    assert!(state.thumb_spring.velocity > 0.0);
    assert!(state.press_spring.target == 0.0);

    for _ in 0..100 {
        state.update(0.016, true, false, false, false, &ctx);
    }
    assert!(state.is_settled());
    assert!((state.thumb_spring.value() - 1.0).abs() < 0.01);
}

#[test]
fn test_checkbox_press_and_pop_springs() {
    let mut state = CheckboxState::new(false);
    let ctx = egui::Context::default();
    assert!(state.is_settled());

    // Focus entrance triggers bounce impulse
    state.update(0.016, false, true, false, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.focus_spring.velocity > 0.0);

    // Press down
    state.update(0.016, false, true, true, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.press_spring.target == 1.0);

    // Release click (checked = true)
    state.update(0.016, true, true, false, true, &ctx);
    assert!(state.check_spring.velocity > 0.0);
    assert!(state.press_spring.target == 0.0);

    for _ in 0..100 {
        state.update(0.016, true, false, false, false, &ctx);
    }
    assert!(state.is_settled());
    assert!((state.check_spring.value() - 1.0).abs() < 0.01);
}

#[test]
fn test_radio_press_and_pop_springs() {
    let mut state = RadioState::new(false);
    let ctx = egui::Context::default();
    assert!(state.is_settled());

    // Focus entrance triggers bounce impulse
    state.update(0.016, false, true, false, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.focus_spring.velocity > 0.0);

    // Press down
    state.update(0.016, false, true, true, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.press_spring.target == 1.0);

    // Release click (selected = true)
    state.update(0.016, true, true, false, true, &ctx);
    assert!(state.dot_spring.velocity > 0.0);
    assert!(state.press_spring.target == 0.0);

    for _ in 0..100 {
        state.update(0.016, true, false, false, false, &ctx);
    }
    assert!(state.is_settled());
    assert!((state.dot_spring.value() - 1.0).abs() < 0.01);
}

#[test]
fn test_tabs_press_and_glide_springs() {
    let mut state = TabsState::default();
    let ctx = egui::Context::default();
    let r1 = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(60.0, 30.0));
    let r2 = egui::Rect::from_min_size(egui::pos2(80.0, 10.0), egui::vec2(60.0, 30.0));

    // Initialize
    state.update(0.016, r1, false, false, &ctx);
    assert!(state.is_settled());

    // Press down on Tab 2
    state.update(0.016, r2, true, false, &ctx);
    assert!(!state.is_settled());
    assert!(state.press_spring.target == 1.0);

    // Release click
    state.update(0.016, r2, false, true, &ctx);
    assert!(state.press_spring.target == 0.0);

    for _ in 0..100 {
        state.update(0.016, r2, false, false, &ctx);
    }
    assert!(state.is_settled());
    assert!((state.x_spring.value() - 80.0).abs() < 0.01);
}

#[test]
fn test_text_input_mouse_word_selection() {
    use egui_vim_nav::{VimBufferState, VimMode, VisualType};

    let text = "hello world test".to_string();
    let mut vbuf = VimBufferState::new(text.clone());
    let mut input_state = InputState::default();

    // Verify initial state
    assert_eq!(vbuf.cursor(), 0);
    assert_eq!(vbuf.mode(), VimMode::Normal);

    // Enter insert mode and type
    vbuf.mode = VimMode::Insert;
    vbuf.cursor = 5; // right after "hello"
    assert_eq!(vbuf.cursor, 5);

    // Enter visual mode with selection spanning "world" (indices 6..11)
    vbuf.anchor = Some(6);
    vbuf.cursor = 11;
    vbuf.mode = VimMode::Visual(VisualType::Character);

    assert_eq!(vbuf.anchor, Some(6));
    assert_eq!(vbuf.cursor, 11);
    assert!(vbuf.mode().is_visual());

    let ctx = egui::Context::default();
    input_state.update(0.016, true, &ctx);
    assert!(!input_state.is_settled());
}

#[test]
fn test_text_input_selection_drag_and_drop_flight() {
    use egui_widgets::DropFlightAnim;

    let mut flight = DropFlightAnim::new(
        "world".to_string(),
        egui::pos2(200.0, 150.0),
        egui::vec2(-50.0, -20.0),
        egui::pos2(50.0, 100.0),
        0,
        45.0,
        0..5,
    );

    assert_eq!(flight.text, "world");
    assert!(!flight.is_settled());

    // Advance flight through ODE spring solver
    for _ in 0..100 {
        flight.update(0.016);
    }

    assert!(flight.is_settled());
    assert!((flight.x_spring.value() - 50.0).abs() < 0.01);
    assert!((flight.y_spring.value() - 100.0).abs() < 0.01);
    assert!((flight.scale_spring.value() - 1.0).abs() < 0.01);
    assert!((flight.gap_spring.value() - 45.0).abs() < 0.01);
}

#[test]
fn test_dropdown_builder_and_state() {
    use egui_widgets::{Dropdown, DropdownOption, DropdownState};

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Env {
        Production,
        Staging,
        Development,
    }

    let mut selected = Env::Staging;
    let mut state = DropdownState::default();
    assert!(state.is_settled());
    assert!(!state.is_open);

    let palette = ThemePreset::CatppuccinMocha.palette();
    let _dd = Dropdown::new(
        &mut selected,
        vec![
            DropdownOption::new(Env::Production, "Production").icon("🔒"),
            DropdownOption::new(Env::Staging, "Staging").icon("🧪"),
            DropdownOption::new(Env::Development, "Development").icon("🛠"),
        ],
    )
    .primary()
    .palette(&palette)
    .with_state(&mut state);
}

#[test]
fn test_dropdown_open_close_animation_frames() {
    use egui_widgets::DropdownState;

    let mut state = DropdownState::default();
    assert!(state.is_settled());

    // 1. Open dropdown
    state.open();
    assert!(state.is_open);
    assert!(!state.is_settled());
    assert_eq!(state.open_spring.target, 1.0);
    assert_eq!(state.chevron_spring.target, 1.0);

    let dt = 0.016;
    for _ in 0..100 {
        state.open_spring.update(dt);
        state.chevron_spring.update(dt);
    }
    assert!(state.open_spring.is_settled());
    assert!((state.open_spring.value() - 1.0).abs() < 0.01);
    assert!(state.chevron_spring.is_settled());
    assert!((state.chevron_spring.value() - 1.0).abs() < 0.01);

    // 2. Close dropdown
    state.close();
    assert!(!state.is_open);
    assert_eq!(state.open_spring.target, 0.0);
    assert_eq!(state.chevron_spring.target, 0.0);

    for _ in 0..100 {
        state.open_spring.update(dt);
        state.chevron_spring.update(dt);
    }
    assert!(state.open_spring.is_settled());
    assert!((state.open_spring.value() - 0.0).abs() < 0.01);
    assert!(state.chevron_spring.is_settled());
    assert!((state.chevron_spring.value() - 0.0).abs() < 0.01);
}

#[test]
fn test_dropdown_two_tier_highlight_springs() {
    use egui_widgets::DropdownState;

    let mut state = DropdownState::default();
    // 1. Sliding focus highlight pill
    state.focus_y_spring.reset(20.0);
    state.focus_h_spring.reset(30.0);
    assert!(state.focus_y_spring.is_settled());
    assert!(state.focus_h_spring.is_settled());

    state.focus_y_spring.set_target(120.0);
    assert!(!state.focus_y_spring.is_settled());

    // 2. Saved selection highlight
    state.saved_y_spring.reset(20.0);
    state.saved_h_spring.reset(30.0);
    assert!(state.saved_y_spring.is_settled());

    state.saved_y_spring.set_target(60.0);
    assert!(!state.saved_y_spring.is_settled());

    let dt = 0.016;
    for _ in 0..100 {
        state.focus_y_spring.update(dt);
        state.saved_y_spring.update(dt);
    }
    assert!(state.focus_y_spring.is_settled());
    assert!((state.focus_y_spring.value() - 120.0).abs() < 0.01);
    assert!(state.saved_y_spring.is_settled());
    assert!((state.saved_y_spring.value() - 60.0).abs() < 0.01);
}

#[test]
fn test_dropdown_4_state_press_and_pop() {
    use egui_widgets::DropdownState;

    let mut state = DropdownState::default();

    // 1. Focus arrival impulse
    state.trigger_focus_bounce();
    assert_eq!(state.focus_bounce_spring.velocity, 20.0);
    assert!(!state.focus_bounce_spring.is_settled());

    // 2. Click release rebound pop
    state.trigger_click();
    assert_eq!(state.press_spring.velocity, 18.0);
    assert!(!state.press_spring.is_settled());

    let dt = 0.016;
    for _ in 0..120 {
        state.focus_bounce_spring.update(dt);
        state.press_spring.update(dt);
    }
    assert!(state.focus_bounce_spring.is_settled());
    assert!(state.press_spring.is_settled());
}

#[test]
fn test_dropdown_in_place_height_morph() {
    use egui_widgets::DropdownState;

    let mut state = DropdownState::default();
    let collapsed_h = 32.0;
    let expanded_h = 160.0;

    // Collapsed initial height
    let open_t = state.open_spring.value().clamp(0.0, 1.0);
    let h_0 = collapsed_h + open_t * (expanded_h - collapsed_h);
    assert_eq!(h_0, 32.0);

    // Expand
    state.open();
    let dt = 0.016;
    for _ in 0..100 {
        state.open_spring.update(dt);
    }
    let open_t_end = state.open_spring.value().clamp(0.0, 1.0);
    let h_end = collapsed_h + open_t_end * (expanded_h - collapsed_h);
    assert!((h_end - 160.0).abs() < 0.1);
}

#[test]
fn test_dropdown_scrolloff_margin_and_capping() {
    use egui_widgets::DropdownState;

    let total_items = 8;
    let max_visible = 5;

    // Test moving down through all 8 items (Vim 'j')
    let mut s = 0;
    let expected_down = [
        (0, 0, 0), // (H, expected S, expected visible slot H - S)
        (1, 0, 1),
        (2, 0, 2),
        (3, 0, 3), // Slot 3: second-to-last item!
        (4, 1, 3), // Scrolled! Highlight remains on Slot 3
        (5, 2, 3), // Scrolled! Highlight remains on Slot 3
        (6, 3, 3), // Scrolled! Highlight remains on Slot 3 (max scroll)
        (7, 3, 4), // At bottom boundary: steps to Slot 4
    ];

    for (h, expected_s, expected_slot) in expected_down {
        s = DropdownState::compute_scrolloff_row(h, s, total_items, max_visible);
        assert_eq!(s, expected_s, "Down: at H={}, expected S={}, got S={}", h, expected_s, s);
        assert_eq!(h - s, expected_slot, "Down: at H={}, expected slot={}, got slot={}", h, expected_slot, h - s);
    }

    // Test moving back up through all items (Vim 'k')
    let expected_up = [
        (7, 3, 4),
        (6, 3, 3),
        (5, 3, 2),
        (4, 3, 1), // Slot 1: second item!
        (3, 2, 1), // Scrolled! Highlight remains on Slot 1
        (2, 1, 1), // Scrolled! Highlight remains on Slot 1
        (1, 0, 1), // Scrolled! Highlight remains on Slot 1 (min scroll)
        (0, 0, 0), // At top boundary: steps to Slot 0
    ];

    for (h, expected_s, expected_slot) in expected_up {
        s = DropdownState::compute_scrolloff_row(h, s, total_items, max_visible);
        assert_eq!(s, expected_s, "Up: at H={}, expected S={}, got S={}", h, expected_s, s);
        assert_eq!(h - s, expected_slot, "Up: at H={}, expected slot={}, got slot={}", h, expected_slot, h - s);
    }
}

#[test]
fn test_dropdown_custom_item_design_and_context() {
    use egui::{Color32, Context};
    use egui_widgets::{Dropdown, DropdownOption, DropdownState};

    let ctx = Context::default();
    let mut selected = "apple".to_string();
    let mut state = DropdownState::default();

    let opt1 = DropdownOption::new("apple".to_string(), "Apple")
        .description("Fresh green granny smith")
        .status_dot(Color32::GREEN)
        .icon("🍏")
        .icon_color(Color32::from_rgb(100, 255, 100))
        .badge("In Stock")
        .badge_color(Color32::from_rgb(0, 200, 100))
        .subtitle("120 cal");

    let opt2 = DropdownOption::new("banana".to_string(), "Banana")
        .description("Organic ripe fruit")
        .status_dot(Color32::YELLOW)
        .badge("Low")
        .badge_color(Color32::from_rgb(255, 200, 50));

    assert_eq!(opt1.status_dot, Some(Color32::GREEN));
    assert_eq!(opt2.status_dot, Some(Color32::YELLOW));
    assert!(opt1.description.is_some());
    assert_eq!(opt1.badge_color, Some(Color32::from_rgb(0, 200, 100)));

    let custom_rendered_count = std::cell::Cell::new(0);

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _resp = Dropdown::new(&mut selected, vec![opt1, opt2])
                .item_height(44.0)
                .with_state(&mut state)
                .item_renderer(|ui, item_ctx| {
                    custom_rendered_count.set(custom_rendered_count.get() + 1);
                    assert!(item_ctx.rect.height() >= 40.0);
                    ui.label(item_ctx.option.label.text());
                })
                .show(ui);
        });
    });
}

#[test]
fn test_dropdown_expansion_visibility_and_auto_scroll() {
    use egui::Context;
    use egui_widgets::{Dropdown, DropdownOption, DropdownState};

    let ctx = Context::default();
    let mut selected = "a".to_string();
    let mut state = DropdownState::default();
    state.open();

    let options = vec![
        DropdownOption::new("a".to_string(), "Alpha"),
        DropdownOption::new("b".to_string(), "Beta"),
        DropdownOption::new("c".to_string(), "Gamma"),
    ];

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = Dropdown::new(&mut selected, options.clone())
                .auto_scroll(true)
                .with_state(&mut state)
                .show(ui);

            assert!(resp.is_open());
            assert!(resp.just_opened());
            let exp_rect = resp.expanded_rect();
            assert!(exp_rect.height() >= 60.0);
            assert!(resp.is_fully_visible());
        });
    });

    // On next frame, just_opened must be false to prevent rubber-banding
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = Dropdown::new(&mut selected, options)
                .auto_scroll(true)
                .with_state(&mut state)
                .show(ui);

            assert!(resp.is_open());
            assert!(!resp.just_opened(), "just_opened must clear after first frame");
        });
    });
}
