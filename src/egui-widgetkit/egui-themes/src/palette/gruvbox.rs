//! Gruvbox theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Gruvbox (retro groove warm earthy theme).
pub fn gruvbox() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(40, 40, 40),
        mantle: Color32::from_rgb(29, 32, 33),
        crust: Color32::from_rgb(29, 32, 33),
        surface0: Color32::from_rgb(60, 56, 54),
        surface1: Color32::from_rgb(80, 73, 69),
        surface2: Color32::from_rgb(102, 92, 84),
        overlay0: Color32::from_rgb(124, 111, 100),
        overlay1: Color32::from_rgb(146, 131, 116),
        overlay2: Color32::from_rgb(168, 153, 132),
        text: Color32::from_rgb(235, 219, 178),
        subtext0: Color32::from_rgb(189, 174, 147),
        subtext1: Color32::from_rgb(213, 196, 161),
        accent: Color32::from_rgb(211, 134, 155), // Purple (d3869b)
        success: Color32::from_rgb(184, 187, 38), // Green (b8bb26)
        warning: Color32::from_rgb(250, 189, 47), // Yellow (fabd2f)
        danger: Color32::from_rgb(251, 73, 52), // Red (fb4934)
        info: Color32::from_rgb(131, 165, 152), // Blue (83a598)
        info_alt: Color32::from_rgb(142, 192, 124), // Aqua (8ec07c)
        sys_controls: Color32::from_rgb(254, 128, 25), // Orange (fe8019)
        on_accent: Color32::from_rgb(29, 32, 33),
        on_surface: Color32::from_rgb(235, 219, 178),
        on_success: Color32::from_rgb(29, 32, 33),
        on_warning: Color32::from_rgb(29, 32, 33),
        on_danger: Color32::from_rgb(29, 32, 33),
        on_info: Color32::from_rgb(29, 32, 33),
        swatches: vec![
            Color32::from_rgb(251, 73, 52),
            Color32::from_rgb(254, 128, 25),
            Color32::from_rgb(250, 189, 47),
            Color32::from_rgb(184, 187, 38),
            Color32::from_rgb(142, 192, 124),
            Color32::from_rgb(131, 165, 152),
            Color32::from_rgb(211, 134, 155),
            Color32::from_rgb(168, 153, 132),
            Color32::from_rgb(235, 219, 178),
        ],
    }
}
