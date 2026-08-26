//! Complete single-source-of-truth palette definition and operations.

use egui::Color32;
use crate::palette::{
    catppuccin, dracula, gruvbox, material_ocean, monokai_pro, night_owl, one_dark_pro, rose_pine,
    tokyo_night,
};
use crate::token::ThemeToken;

/// A complete, concrete color palette acting as the **single source of truth** for UI colors.
///
/// # State Ownership
///
/// `ThemePalette` is a plain value struct owned by the consuming application and stored in state
/// (`CODING_RULES §2`).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemePalette {
    /// Whether this palette is designed for dark mode.
    pub dark: bool,

    // ── Background Layers ──
    pub base: Color32,
    pub mantle: Color32,
    pub crust: Color32,
    pub surface0: Color32,
    pub surface1: Color32,
    pub surface2: Color32,
    pub overlay0: Color32,
    pub overlay1: Color32,
    pub overlay2: Color32,

    // ── Typography ──
    pub text: Color32,
    pub subtext0: Color32,
    pub subtext1: Color32,

    // ── Semantic Roles ──
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub info_alt: Color32,
    pub sys_controls: Color32,

    // ── On-Color Contrast Tokens (Material 3 paired roles) ──
    pub on_accent: Color32,
    pub on_surface: Color32,
    pub on_success: Color32,
    pub on_warning: Color32,
    pub on_danger: Color32,
    pub on_info: Color32,

    // ── Designer Swatches Strip ──
    pub swatches: Vec<Color32>,
}

impl Default for ThemePalette {
    fn default() -> Self {
        catppuccin::mocha()
    }
}

impl ThemePalette {
    /// Retrieves the color corresponding to the given [`ThemeToken`].
    pub fn get(&self, token: ThemeToken) -> Color32 {
        match token {
            ThemeToken::Base => self.base,
            ThemeToken::Mantle => self.mantle,
            ThemeToken::Crust => self.crust,
            ThemeToken::Surface0 => self.surface0,
            ThemeToken::Surface1 => self.surface1,
            ThemeToken::Surface2 => self.surface2,
            ThemeToken::Overlay0 => self.overlay0,
            ThemeToken::Overlay1 => self.overlay1,
            ThemeToken::Overlay2 => self.overlay2,
            ThemeToken::Text => self.text,
            ThemeToken::Subtext0 => self.subtext0,
            ThemeToken::Subtext1 => self.subtext1,
            ThemeToken::Accent => self.accent,
            ThemeToken::Success => self.success,
            ThemeToken::Warning => self.warning,
            ThemeToken::Danger => self.danger,
            ThemeToken::Info => self.info,
            ThemeToken::InfoAlt => self.info_alt,
            ThemeToken::SysControls => self.sys_controls,
            ThemeToken::OnAccent => self.on_accent,
            ThemeToken::OnSurface => self.on_surface,
            ThemeToken::OnSuccess => self.on_success,
            ThemeToken::OnWarning => self.on_warning,
            ThemeToken::OnDanger => self.on_danger,
            ThemeToken::OnInfo => self.on_info,
        }
    }

    /// Sets the color corresponding to the given [`ThemeToken`].
    pub fn set(&mut self, token: ThemeToken, color: Color32) {
        match token {
            ThemeToken::Base => self.base = color,
            ThemeToken::Mantle => self.mantle = color,
            ThemeToken::Crust => self.crust = color,
            ThemeToken::Surface0 => self.surface0 = color,
            ThemeToken::Surface1 => self.surface1 = color,
            ThemeToken::Surface2 => self.surface2 = color,
            ThemeToken::Overlay0 => self.overlay0 = color,
            ThemeToken::Overlay1 => self.overlay1 = color,
            ThemeToken::Overlay2 => self.overlay2 = color,
            ThemeToken::Text => self.text = color,
            ThemeToken::Subtext0 => self.subtext0 = color,
            ThemeToken::Subtext1 => self.subtext1 = color,
            ThemeToken::Accent => self.accent = color,
            ThemeToken::Success => self.success = color,
            ThemeToken::Warning => self.warning = color,
            ThemeToken::Danger => self.danger = color,
            ThemeToken::Info => self.info = color,
            ThemeToken::InfoAlt => self.info_alt = color,
            ThemeToken::SysControls => self.sys_controls = color,
            ThemeToken::OnAccent => self.on_accent = color,
            ThemeToken::OnSurface => self.on_surface = color,
            ThemeToken::OnSuccess => self.on_success = color,
            ThemeToken::OnWarning => self.on_warning = color,
            ThemeToken::OnDanger => self.on_danger = color,
            ThemeToken::OnInfo => self.on_info = color,
        }
    }

    /// Calculates the W3C relative luminance of an sRGB color ($0.0 \dots 1.0$).
    pub fn relative_luminance(c: Color32) -> f32 {
        fn srgb_to_linear(val: u8) -> f32 {
            let v = val as f32 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * srgb_to_linear(c.r()) + 0.7152 * srgb_to_linear(c.g()) + 0.0722 * srgb_to_linear(c.b())
    }

    /// Automatically returns either high-contrast dark (`self.crust`) or light (`self.text`) text
    /// to guarantee maximum WCAG readability on top of any arbitrary background color.
    pub fn contrast_on(&self, bg: Color32) -> Color32 {
        if Self::relative_luminance(bg) > 0.38 {
            self.crust
        } else {
            self.text
        }
    }

    /// Smoothly interpolates (exponential lerp) all colors in `self` towards `target` over delta time `dt`.
    ///
    /// Returns `true` if any color changed during this step (meaning animation is still running).
    pub fn interpolate(&mut self, target: &ThemePalette, dt: f32, speed: f32) -> bool {
        let factor = (1.0 - (-speed * dt).exp()).clamp(0.0, 1.0);
        let mut changed = false;

        fn lerp_val(cur: u8, tgt: u8, factor: f32) -> u8 {
            if cur == tgt {
                return tgt;
            }
            let diff = tgt as f32 - cur as f32;
            let delta = diff * factor;
            if delta.abs() < 0.5 {
                if diff > 0.0 {
                    cur.saturating_add(1).min(tgt)
                } else {
                    cur.saturating_sub(1).max(tgt)
                }
            } else {
                (cur as f32 + delta).round().clamp(0.0, 255.0) as u8
            }
        }

        fn lerp_color(cur: Color32, tgt: Color32, factor: f32, changed: &mut bool) -> Color32 {
            let next = Color32::from_rgba_unmultiplied(
                lerp_val(cur.r(), tgt.r(), factor),
                lerp_val(cur.g(), tgt.g(), factor),
                lerp_val(cur.b(), tgt.b(), factor),
                lerp_val(cur.a(), tgt.a(), factor),
            );
            if next != cur {
                *changed = true;
            }
            next
        }

        self.dark = target.dark;
        self.base = lerp_color(self.base, target.base, factor, &mut changed);
        self.mantle = lerp_color(self.mantle, target.mantle, factor, &mut changed);
        self.crust = lerp_color(self.crust, target.crust, factor, &mut changed);
        self.surface0 = lerp_color(self.surface0, target.surface0, factor, &mut changed);
        self.surface1 = lerp_color(self.surface1, target.surface1, factor, &mut changed);
        self.surface2 = lerp_color(self.surface2, target.surface2, factor, &mut changed);
        self.overlay0 = lerp_color(self.overlay0, target.overlay0, factor, &mut changed);
        self.overlay1 = lerp_color(self.overlay1, target.overlay1, factor, &mut changed);
        self.overlay2 = lerp_color(self.overlay2, target.overlay2, factor, &mut changed);
        self.text = lerp_color(self.text, target.text, factor, &mut changed);
        self.subtext0 = lerp_color(self.subtext0, target.subtext0, factor, &mut changed);
        self.subtext1 = lerp_color(self.subtext1, target.subtext1, factor, &mut changed);
        self.accent = lerp_color(self.accent, target.accent, factor, &mut changed);
        self.success = lerp_color(self.success, target.success, factor, &mut changed);
        self.warning = lerp_color(self.warning, target.warning, factor, &mut changed);
        self.danger = lerp_color(self.danger, target.danger, factor, &mut changed);
        self.info = lerp_color(self.info, target.info, factor, &mut changed);
        self.info_alt = lerp_color(self.info_alt, target.info_alt, factor, &mut changed);
        self.sys_controls = lerp_color(self.sys_controls, target.sys_controls, factor, &mut changed);
        self.on_accent = lerp_color(self.on_accent, target.on_accent, factor, &mut changed);
        self.on_surface = lerp_color(self.on_surface, target.on_surface, factor, &mut changed);
        self.on_success = lerp_color(self.on_success, target.on_success, factor, &mut changed);
        self.on_warning = lerp_color(self.on_warning, target.on_warning, factor, &mut changed);
        self.on_danger = lerp_color(self.on_danger, target.on_danger, factor, &mut changed);
        self.on_info = lerp_color(self.on_info, target.on_info, factor, &mut changed);

        // Interpolate swatches
        let n = self.swatches.len().min(target.swatches.len());
        for i in 0..n {
            self.swatches[i] = lerp_color(self.swatches[i], target.swatches[i], factor, &mut changed);
        }

        changed
    }

    // ── Direct Constructors ──

    /// Catppuccin Mocha preset.
    pub fn catppuccin_mocha() -> Self {
        catppuccin::mocha()
    }

    /// Catppuccin Macchiato preset.
    pub fn catppuccin_macchiato() -> Self {
        catppuccin::macchiato()
    }

    /// Catppuccin Frappé preset.
    pub fn catppuccin_frappe() -> Self {
        catppuccin::frappe()
    }

    /// Catppuccin Latte (light mode) preset.
    pub fn catppuccin_latte() -> Self {
        catppuccin::latte()
    }

    /// Tokyo Night preset.
    pub fn tokyo_night() -> Self {
        tokyo_night::tokyo_night()
    }

    /// One Dark Pro preset.
    pub fn one_dark_pro() -> Self {
        one_dark_pro::one_dark_pro()
    }

    /// Dracula preset.
    pub fn dracula() -> Self {
        dracula::dracula()
    }

    /// Monokai Pro preset.
    pub fn monokai_pro() -> Self {
        monokai_pro::monokai_pro()
    }

    /// Night Owl preset.
    pub fn night_owl() -> Self {
        night_owl::night_owl()
    }

    /// Material Ocean preset.
    pub fn material_ocean() -> Self {
        material_ocean::material_ocean()
    }

    /// Rosé Pine preset.
    pub fn rose_pine() -> Self {
        rose_pine::rose_pine()
    }

    /// Gruvbox preset.
    pub fn gruvbox() -> Self {
        gruvbox::gruvbox()
    }
}
