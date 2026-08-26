//! One Dark Pro theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// One Dark Pro (classic Atom/VSCode editor theme).
pub fn one_dark_pro() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(40, 44, 52),
        mantle: Color32::from_rgb(33, 37, 43),
        crust: Color32::from_rgb(30, 34, 39),
        surface0: Color32::from_rgb(53, 59, 69),
        surface1: Color32::from_rgb(62, 68, 81),
        surface2: Color32::from_rgb(75, 82, 99),
        overlay0: Color32::from_rgb(92, 99, 112),
        overlay1: Color32::from_rgb(171, 178, 191),
        overlay2: Color32::from_rgb(171, 178, 191),
        text: Color32::from_rgb(171, 178, 191),
        subtext0: Color32::from_rgb(130, 137, 151),
        subtext1: Color32::from_rgb(171, 178, 191),
        accent: Color32::from_rgb(198, 120, 221), // Purple
        success: Color32::from_rgb(152, 195, 121), // Green
        warning: Color32::from_rgb(229, 192, 123), // Yellow
        danger: Color32::from_rgb(224, 108, 117), // Red
        info: Color32::from_rgb(97, 175, 239), // Blue
        info_alt: Color32::from_rgb(86, 182, 194), // Cyan
        sys_controls: Color32::from_rgb(97, 175, 239),
        on_accent: Color32::from_rgb(33, 37, 43),
        on_surface: Color32::from_rgb(171, 178, 191),
        on_success: Color32::from_rgb(33, 37, 43),
        on_warning: Color32::from_rgb(33, 37, 43),
        on_danger: Color32::from_rgb(33, 37, 43),
        on_info: Color32::from_rgb(33, 37, 43),
        swatches: vec![
            Color32::from_rgb(224, 108, 117),
            Color32::from_rgb(209, 154, 102),
            Color32::from_rgb(229, 192, 123),
            Color32::from_rgb(152, 195, 121),
            Color32::from_rgb(86, 182, 194),
            Color32::from_rgb(97, 175, 239),
            Color32::from_rgb(198, 120, 221),
        ],
    }
}
