//! Night Owl theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Night Owl (deep blue nighttime coding theme).
pub fn night_owl() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(1, 22, 39),
        mantle: Color32::from_rgb(1, 14, 23),
        crust: Color32::from_rgb(1, 10, 16),
        surface0: Color32::from_rgb(11, 41, 66),
        surface1: Color32::from_rgb(16, 61, 97),
        surface2: Color32::from_rgb(21, 79, 125),
        overlay0: Color32::from_rgb(99, 119, 119),
        overlay1: Color32::from_rgb(214, 222, 235),
        overlay2: Color32::from_rgb(214, 222, 235),
        text: Color32::from_rgb(214, 222, 235),
        subtext0: Color32::from_rgb(127, 219, 202),
        subtext1: Color32::from_rgb(214, 222, 235),
        accent: Color32::from_rgb(199, 146, 234), // Purple
        success: Color32::from_rgb(127, 219, 202), // Teal/Green
        warning: Color32::from_rgb(236, 196, 141), // Yellow
        danger: Color32::from_rgb(239, 83, 80), // Red
        info: Color32::from_rgb(130, 170, 255), // Blue
        info_alt: Color32::from_rgb(247, 140, 108), // Orange
        sys_controls: Color32::from_rgb(130, 170, 255),
        swatches: vec![
            Color32::from_rgb(239, 83, 80),
            Color32::from_rgb(247, 140, 108),
            Color32::from_rgb(236, 196, 141),
            Color32::from_rgb(127, 219, 202),
            Color32::from_rgb(130, 170, 255),
            Color32::from_rgb(199, 146, 234),
        ],
    }
}
