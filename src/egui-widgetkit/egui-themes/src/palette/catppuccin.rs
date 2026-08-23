//! Catppuccin theme family palette definitions (Mocha, Macchiato, Frappé, Latte).

use egui::Color32;
use crate::palette::ThemePalette;

/// Catppuccin Mocha (darkest pastel soothing theme).
pub fn mocha() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(30, 30, 46),
        mantle: Color32::from_rgb(24, 24, 37),
        crust: Color32::from_rgb(17, 17, 27),
        surface0: Color32::from_rgb(49, 50, 68),
        surface1: Color32::from_rgb(69, 71, 90),
        surface2: Color32::from_rgb(88, 91, 112),
        overlay0: Color32::from_rgb(108, 112, 134),
        overlay1: Color32::from_rgb(127, 132, 156),
        overlay2: Color32::from_rgb(147, 153, 178),
        text: Color32::from_rgb(205, 214, 244),
        subtext0: Color32::from_rgb(166, 173, 200),
        subtext1: Color32::from_rgb(186, 194, 222),
        accent: Color32::from_rgb(203, 166, 247), // Mauve
        success: Color32::from_rgb(166, 227, 161), // Green
        warning: Color32::from_rgb(250, 179, 135), // Peach
        danger: Color32::from_rgb(243, 139, 168), // Red
        info: Color32::from_rgb(137, 180, 250), // Blue
        info_alt: Color32::from_rgb(148, 226, 213), // Teal
        sys_controls: Color32::from_rgb(116, 199, 236), // Sapphire
        swatches: vec![
            Color32::from_rgb(243, 139, 168),
            Color32::from_rgb(250, 179, 135),
            Color32::from_rgb(249, 226, 175),
            Color32::from_rgb(166, 227, 161),
            Color32::from_rgb(148, 226, 213),
            Color32::from_rgb(137, 180, 250),
            Color32::from_rgb(203, 166, 247),
            Color32::from_rgb(245, 194, 231),
            Color32::from_rgb(245, 224, 220),
        ],
    }
}

/// Catppuccin Macchiato (medium dark pastel theme).
pub fn macchiato() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(36, 39, 58),
        mantle: Color32::from_rgb(30, 32, 48),
        crust: Color32::from_rgb(24, 25, 38),
        surface0: Color32::from_rgb(54, 57, 79),
        surface1: Color32::from_rgb(73, 77, 100),
        surface2: Color32::from_rgb(91, 96, 120),
        overlay0: Color32::from_rgb(110, 115, 141),
        overlay1: Color32::from_rgb(128, 135, 162),
        overlay2: Color32::from_rgb(147, 154, 183),
        text: Color32::from_rgb(202, 211, 245),
        subtext0: Color32::from_rgb(165, 173, 203),
        subtext1: Color32::from_rgb(184, 192, 224),
        accent: Color32::from_rgb(198, 160, 246), // Mauve
        success: Color32::from_rgb(166, 218, 149), // Green
        warning: Color32::from_rgb(245, 169, 127), // Peach
        danger: Color32::from_rgb(237, 135, 150), // Red
        info: Color32::from_rgb(138, 173, 244), // Blue
        info_alt: Color32::from_rgb(139, 213, 202), // Teal
        sys_controls: Color32::from_rgb(238, 212, 159), // Yellow
        swatches: vec![
            Color32::from_rgb(237, 135, 150),
            Color32::from_rgb(245, 169, 127),
            Color32::from_rgb(238, 212, 159),
            Color32::from_rgb(166, 218, 149),
            Color32::from_rgb(139, 213, 202),
            Color32::from_rgb(138, 173, 244),
            Color32::from_rgb(198, 160, 246),
        ],
    }
}

/// Catppuccin Frappé (muted dark pastel theme).
pub fn frappe() -> ThemePalette {
    ThemePalette {
        dark: true,
        base: Color32::from_rgb(48, 52, 70),
        mantle: Color32::from_rgb(41, 44, 60),
        crust: Color32::from_rgb(35, 38, 52),
        surface0: Color32::from_rgb(65, 69, 89),
        surface1: Color32::from_rgb(81, 85, 104),
        surface2: Color32::from_rgb(98, 104, 128),
        overlay0: Color32::from_rgb(115, 121, 148),
        overlay1: Color32::from_rgb(131, 139, 167),
        overlay2: Color32::from_rgb(148, 156, 187),
        text: Color32::from_rgb(198, 208, 245),
        subtext0: Color32::from_rgb(165, 173, 206),
        subtext1: Color32::from_rgb(181, 191, 226),
        accent: Color32::from_rgb(202, 158, 230), // Mauve
        success: Color32::from_rgb(166, 209, 137), // Green
        warning: Color32::from_rgb(239, 159, 118), // Peach
        danger: Color32::from_rgb(231, 130, 132), // Red
        info: Color32::from_rgb(140, 170, 238), // Blue
        info_alt: Color32::from_rgb(129, 200, 190), // Teal
        sys_controls: Color32::from_rgb(186, 187, 241), // Lavender
        swatches: vec![
            Color32::from_rgb(231, 130, 132),
            Color32::from_rgb(239, 159, 118),
            Color32::from_rgb(229, 200, 144),
            Color32::from_rgb(166, 209, 137),
            Color32::from_rgb(129, 200, 190),
            Color32::from_rgb(140, 170, 238),
            Color32::from_rgb(202, 158, 230),
            Color32::from_rgb(186, 187, 241),
            Color32::from_rgb(198, 208, 245),
        ],
    }
}

/// Catppuccin Latte (light mode pastel theme).
pub fn latte() -> ThemePalette {
    ThemePalette {
        dark: false,
        base: Color32::from_rgb(239, 241, 245),
        mantle: Color32::from_rgb(230, 233, 239),
        crust: Color32::from_rgb(220, 224, 232),
        surface0: Color32::from_rgb(204, 208, 218),
        surface1: Color32::from_rgb(188, 192, 204),
        surface2: Color32::from_rgb(172, 176, 190),
        overlay0: Color32::from_rgb(156, 160, 176),
        overlay1: Color32::from_rgb(140, 143, 161),
        overlay2: Color32::from_rgb(124, 127, 147),
        text: Color32::from_rgb(76, 79, 105),
        subtext0: Color32::from_rgb(108, 111, 133),
        subtext1: Color32::from_rgb(92, 95, 119),
        accent: Color32::from_rgb(136, 57, 239), // Mauve
        success: Color32::from_rgb(64, 160, 43), // Green
        warning: Color32::from_rgb(223, 142, 29), // Yellow
        danger: Color32::from_rgb(210, 15, 57), // Red
        info: Color32::from_rgb(30, 102, 245), // Blue
        info_alt: Color32::from_rgb(23, 146, 153), // Teal
        sys_controls: Color32::from_rgb(114, 135, 253), // Lavender
        swatches: vec![
            Color32::from_rgb(210, 15, 57),
            Color32::from_rgb(254, 100, 27),
            Color32::from_rgb(223, 142, 29),
            Color32::from_rgb(64, 160, 43),
            Color32::from_rgb(23, 146, 153),
            Color32::from_rgb(30, 102, 245),
            Color32::from_rgb(136, 57, 239),
            Color32::from_rgb(114, 135, 253),
            Color32::from_rgb(76, 79, 105),
        ],
    }
}
