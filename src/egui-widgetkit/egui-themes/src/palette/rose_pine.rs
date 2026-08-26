//! Rosé Pine theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Rosé Pine (soho vibes warm aesthetic theme).
pub fn rose_pine() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(25, 23, 36),
        mantle: Color32::from_rgb(31, 29, 46),
        crust: Color32::from_rgb(18, 16, 26),
        surface0: Color32::from_rgb(38, 35, 58),
        surface1: Color32::from_rgb(64, 61, 82),
        surface2: Color32::from_rgb(82, 79, 103),
        overlay0: Color32::from_rgb(110, 106, 134),
        overlay1: Color32::from_rgb(144, 140, 170),
        overlay2: Color32::from_rgb(144, 140, 170),
        text: Color32::from_rgb(224, 222, 244),
        subtext0: Color32::from_rgb(144, 140, 170),
        subtext1: Color32::from_rgb(224, 222, 244),
        accent: Color32::from_rgb(196, 167, 231), // Iris
        success: Color32::from_rgb(49, 116, 143), // Pine
        warning: Color32::from_rgb(246, 193, 119), // Gold
        danger: Color32::from_rgb(235, 111, 146), // Love
        info: Color32::from_rgb(156, 207, 216), // Foam
        info_alt: Color32::from_rgb(235, 188, 186), // Rose
        sys_controls: Color32::from_rgb(196, 167, 231), // Iris
        on_accent: Color32::from_rgb(18, 16, 26),
        on_surface: Color32::from_rgb(224, 222, 244),
        on_success: Color32::from_rgb(224, 222, 244),
        on_warning: Color32::from_rgb(18, 16, 26),
        on_danger: Color32::from_rgb(18, 16, 26),
        on_info: Color32::from_rgb(18, 16, 26),
        swatches: vec![
            Color32::from_rgb(235, 111, 146),
            Color32::from_rgb(246, 193, 119),
            Color32::from_rgb(235, 188, 186),
            Color32::from_rgb(49, 116, 143),
            Color32::from_rgb(156, 207, 216),
            Color32::from_rgb(196, 167, 231),
            Color32::from_rgb(224, 222, 244),
        ],
    }
}
