//! Unit and layout integration tests for egui-widgets.

use egui_themes::ThemePreset;
use egui_widgets::{
    Badge, Button, ButtonSize, ButtonState, Card, Checkbox, CheckboxState,
    InputState, ProgressBar, ProgressState, ProgressVariant, RadioButton, SegmentedTabs, Slider,
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
    let _rb = RadioButton::new(2, &mut selected_option).label("Option 2");
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
    let mut query = String::from("antigravity");
    let mut state = InputState::default();
    assert!(state.is_settled());

    let _input = TextInput::new(&mut query)
        .placeholder("Type here...")
        .icon("🔍")
        .clear_button(true)
        .with_state(&mut state);
}

#[test]
fn test_card_builder() {
    let _card = Card::new()
        .title("Metrics")
        .subtitle("Live stats")
        .interactive(true);
}
