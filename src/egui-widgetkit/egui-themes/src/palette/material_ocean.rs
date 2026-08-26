//! Material Ocean theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Material Ocean (deep oceanic material theme).
pub fn material_ocean() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(15, 17, 26),
        mantle: Color32::from_rgb(9, 11, 16),
        crust: Color32::from_rgb(4, 5, 7),
        surface0: Color32::from_rgb(30, 33, 50),
        surface1: Color32::from_rgb(41, 45, 71),
        surface2: Color32::from_rgb(59, 66, 104),
        overlay0: Color32::from_rgb(78, 86, 132),
        overlay1: Color32::from_rgb(238, 255, 255),
        overlay2: Color32::from_rgb(238, 255, 255),
        text: Color32::from_rgb(166, 172, 205),
        subtext0: Color32::from_rgb(118, 124, 157),
        subtext1: Color32::from_rgb(238, 255, 255),
        accent: Color32::from_rgb(128, 203, 196), // Teal
        success: Color32::from_rgb(195, 232, 141), // Green
        warning: Color32::from_rgb(255, 203, 107), // Yellow
        danger: Color32::from_rgb(240, 113, 120), // Red
        info: Color32::from_rgb(130, 170, 255), // Blue
        info_alt: Color32::from_rgb(247, 140, 108), // Orange
        sys_controls: Color32::from_rgb(199, 146, 234), // Purple
        on_accent: Color32::from_rgb(15, 17, 26),
        on_surface: Color32::from_rgb(166, 172, 205),
        on_success: Color32::from_rgb(15, 17, 26),
        on_warning: Color32::from_rgb(15, 17, 26),
        on_danger: Color32::from_rgb(15, 17, 26),
        on_info: Color32::from_rgb(15, 17, 26),
        swatches: vec![
            Color32::from_rgb(240, 113, 120),
            Color32::from_rgb(247, 140, 108),
            Color32::from_rgb(255, 203, 107),
            Color32::from_rgb(195, 232, 141),
            Color32::from_rgb(128, 203, 196),
            Color32::from_rgb(130, 170, 255),
            Color32::from_rgb(199, 146, 234),
        ],
    }
}
