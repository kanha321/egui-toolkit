use egui::Color32;
use egui_themes::{ThemePalette, ThemePreset, ThemeState, ThemeToken};

#[test]
fn test_all_12_presets_construct_valid_palettes() {
    let presets = ThemePreset::all();
    assert_eq!(presets.len(), 12);

    for preset in presets {
        let palette = preset.palette();
        assert!(!preset.name().is_empty());

        // Verify key tokens are non-transparent
        assert_eq!(palette.base.a(), 255);
        assert_eq!(palette.text.a(), 255);
        assert_eq!(palette.accent.a(), 255);
        assert!(!palette.swatches.is_empty());

        // Verify dark mode flag
        if *preset == ThemePreset::CatppuccinLatte {
            assert!(!preset.is_dark());
            assert!(!palette.dark);
        } else {
            assert!(preset.is_dark());
            assert!(palette.dark);
        }
    }
}

#[test]
fn test_token_resolution_and_mutation() {
    let mut palette = ThemePalette::catppuccin_mocha();

    // Check getting tokens
    assert_eq!(palette.get(ThemeToken::Base), palette.base);
    assert_eq!(palette.get(ThemeToken::Text), palette.text);
    assert_eq!(palette.get(ThemeToken::Accent), palette.accent);
    assert_eq!(palette.get(ThemeToken::OnAccent), palette.on_accent);
    assert_eq!(palette.get(ThemeToken::OnSurface), palette.on_surface);
    assert_eq!(palette.get(ThemeToken::OnSuccess), palette.on_success);
    assert_eq!(palette.get(ThemeToken::OnWarning), palette.on_warning);
    assert_eq!(palette.get(ThemeToken::OnDanger), palette.on_danger);
    assert_eq!(palette.get(ThemeToken::OnInfo), palette.on_info);

    // Check mutating via token
    let test_color = Color32::from_rgb(123, 45, 67);
    palette.set(ThemeToken::Accent, test_color);
    assert_eq!(palette.accent, test_color);
    assert_eq!(palette.get(ThemeToken::Accent), test_color);

    palette.set(ThemeToken::OnAccent, Color32::WHITE);
    assert_eq!(palette.on_accent, Color32::WHITE);
    assert_eq!(palette.get(ThemeToken::OnAccent), Color32::WHITE);
}

#[test]
fn test_contrast_and_luminance_calculation() {
    let palette = ThemePalette::catppuccin_mocha();

    // Pure black has 0.0 luminance, pure white has 1.0 luminance
    assert_eq!(ThemePalette::relative_luminance(Color32::BLACK), 0.0);
    assert!((ThemePalette::relative_luminance(Color32::WHITE) - 1.0).abs() < 1e-4);

    // Dark background yields light text
    assert_eq!(palette.contrast_on(Color32::from_rgb(10, 10, 10)), palette.text);

    // Bright background yields dark text (crust)
    assert_eq!(palette.contrast_on(Color32::from_rgb(240, 240, 240)), palette.crust);
}

#[test]
fn test_palette_interpolation_converges() {
    let mut current = ThemePalette::catppuccin_mocha();
    let target = ThemePalette::tokyo_night();

    assert_ne!(current.base, target.base);

    // Run interpolation steps
    for _ in 0..100 {
        current.interpolate(&target, 0.016, 12.0);
    }

    // Colors should have converged to target
    assert_eq!(current.base, target.base);
    assert_eq!(current.accent, target.accent);
    assert_eq!(current.text, target.text);
}

#[test]
fn test_theme_state_lifecycle() {
    let mut state = ThemeState::new(ThemePreset::CatppuccinMocha);
    assert_eq!(state.active_preset, Some(ThemePreset::CatppuccinMocha));
    assert!(!state.animating);

    // Change preset
    state.set_preset(ThemePreset::Dracula);
    assert_eq!(state.active_preset, Some(ThemePreset::Dracula));
    assert!(state.animating);
    assert_eq!(state.target, ThemePreset::Dracula.palette());

    // Instant switch
    state.set_preset_instant(ThemePreset::Gruvbox);
    assert_eq!(state.active_preset, Some(ThemePreset::Gruvbox));
    assert!(!state.animating);
    assert_eq!(state.current, ThemePreset::Gruvbox.palette());
}

#[test]
fn test_egui_visuals_sync() {
    let state = ThemeState::new(ThemePreset::CatppuccinMocha);
    egui::__run_test_ctx(|ctx| {
        state.apply_to_ctx(ctx);
        let visuals = ctx.style().visuals.clone();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, state.current.base);
        assert_eq!(visuals.widgets.noninteractive.bg_fill, state.current.surface0);
        assert_eq!(visuals.widgets.hovered.bg_fill, state.current.surface1);
        assert_eq!(visuals.widgets.active.bg_fill, state.current.surface2);
    });
}
