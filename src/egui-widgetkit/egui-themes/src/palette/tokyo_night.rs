//! Tokyo Night theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Tokyo Night (clean, neon dark cyber theme).
pub fn tokyo_night() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(26, 27, 38),
        mantle: Color32::from_rgb(22, 22, 30),
        crust: Color32::from_rgb(22, 22, 30),
        surface0: Color32::from_rgb(41, 46, 66),
        surface1: Color32::from_rgb(56, 62, 90),
        surface2: Color32::from_rgb(71, 82, 122),
        overlay0: Color32::from_rgb(86, 95, 137),
        overlay1: Color32::from_rgb(101, 117, 172),
        overlay2: Color32::from_rgb(169, 177, 214),
        text: Color32::from_rgb(192, 202, 245),
        subtext0: Color32::from_rgb(154, 165, 218),
        subtext1: Color32::from_rgb(169, 177, 214),
        accent: Color32::from_rgb(187, 154, 247), // Purple
        success: Color32::from_rgb(158, 206, 106), // Green
        warning: Color32::from_rgb(224, 175, 104), // Yellow
        danger: Color32::from_rgb(247, 118, 142), // Red
        info: Color32::from_rgb(122, 162, 247), // Blue
        info_alt: Color32::from_rgb(115, 218, 202), // Teal
        sys_controls: Color32::from_rgb(42, 195, 222), // Cyan
        swatches: vec![
            Color32::from_rgb(247, 118, 142),
            Color32::from_rgb(255, 158, 100),
            Color32::from_rgb(224, 175, 104),
            Color32::from_rgb(158, 206, 106),
            Color32::from_rgb(115, 218, 202),
            Color32::from_rgb(122, 162, 247),
            Color32::from_rgb(187, 154, 247),
            Color32::from_rgb(255, 0, 127),
            Color32::from_rgb(192, 202, 245),
        ],
    }
}
