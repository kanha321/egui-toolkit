//! Monokai Pro theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Monokai Pro (warm, high-contrast dark theme).
pub fn monokai_pro() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(45, 42, 46),
        mantle: Color32::from_rgb(34, 31, 34),
        crust: Color32::from_rgb(25, 24, 26),
        surface0: Color32::from_rgb(64, 62, 65),
        surface1: Color32::from_rgb(91, 89, 92),
        surface2: Color32::from_rgb(114, 112, 114),
        overlay0: Color32::from_rgb(114, 112, 114),
        overlay1: Color32::from_rgb(252, 252, 250),
        overlay2: Color32::from_rgb(252, 252, 250),
        text: Color32::from_rgb(252, 252, 250),
        subtext0: Color32::from_rgb(147, 146, 147),
        subtext1: Color32::from_rgb(252, 252, 250),
        accent: Color32::from_rgb(171, 157, 242), // Purple
        success: Color32::from_rgb(169, 220, 118), // Green
        warning: Color32::from_rgb(255, 216, 102), // Yellow
        danger: Color32::from_rgb(255, 97, 136), // Red
        info: Color32::from_rgb(120, 220, 232), // Cyan
        info_alt: Color32::from_rgb(252, 152, 103), // Orange
        sys_controls: Color32::from_rgb(171, 157, 242),
        on_accent: Color32::from_rgb(34, 31, 34),
        on_surface: Color32::from_rgb(252, 252, 250),
        on_success: Color32::from_rgb(34, 31, 34),
        on_warning: Color32::from_rgb(34, 31, 34),
        on_danger: Color32::from_rgb(34, 31, 34),
        on_info: Color32::from_rgb(34, 31, 34),
        swatches: vec![
            Color32::from_rgb(255, 97, 136),
            Color32::from_rgb(252, 152, 103),
            Color32::from_rgb(255, 216, 102),
            Color32::from_rgb(169, 220, 118),
            Color32::from_rgb(120, 220, 232),
            Color32::from_rgb(171, 157, 242),
        ],
    }
}
