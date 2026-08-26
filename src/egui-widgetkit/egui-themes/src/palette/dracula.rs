//! Dracula theme palette definition.

use egui::Color32;
use crate::palette::ThemePalette;

/// Dracula (vibrant dark gothic theme).
pub fn dracula() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(40, 42, 54),
        mantle: Color32::from_rgb(30, 31, 41),
        crust: Color32::from_rgb(25, 26, 33),
        surface0: Color32::from_rgb(68, 71, 90),
        surface1: Color32::from_rgb(98, 114, 164),
        surface2: Color32::from_rgb(98, 114, 164),
        overlay0: Color32::from_rgb(98, 114, 164),
        overlay1: Color32::from_rgb(248, 248, 242),
        overlay2: Color32::from_rgb(248, 248, 242),
        text: Color32::from_rgb(248, 248, 242),
        subtext0: Color32::from_rgb(189, 147, 249),
        subtext1: Color32::from_rgb(248, 248, 242),
        accent: Color32::from_rgb(189, 147, 249), // Purple
        success: Color32::from_rgb(80, 250, 123), // Green
        warning: Color32::from_rgb(255, 184, 108), // Orange
        danger: Color32::from_rgb(255, 85, 85), // Red
        info: Color32::from_rgb(139, 233, 253), // Cyan
        info_alt: Color32::from_rgb(255, 121, 198), // Pink
        sys_controls: Color32::from_rgb(189, 147, 249),
        on_accent: Color32::from_rgb(40, 42, 54),
        on_surface: Color32::from_rgb(248, 248, 242),
        on_success: Color32::from_rgb(40, 42, 54),
        on_warning: Color32::from_rgb(40, 42, 54),
        on_danger: Color32::from_rgb(40, 42, 54),
        on_info: Color32::from_rgb(40, 42, 54),
        swatches: vec![
            Color32::from_rgb(255, 85, 85),
            Color32::from_rgb(255, 184, 108),
            Color32::from_rgb(241, 250, 140),
            Color32::from_rgb(80, 250, 123),
            Color32::from_rgb(139, 233, 253),
            Color32::from_rgb(189, 147, 249),
            Color32::from_rgb(255, 121, 198),
        ],
    }
}
