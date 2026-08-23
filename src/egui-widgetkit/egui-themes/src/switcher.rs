//! Theme switcher state management, animated morphing, and egui Context visuals synchronization.

use egui::{Context, Stroke, Visuals};
use crate::palette::{ThemePalette, ThemePreset};

/// App-owned state manager for the active theme, smooth morph transitions, and egui styling.
///
/// # State Ownership
///
/// `ThemeState` is owned directly by the consuming application (e.g. stored in `TestAppState`)
/// and passed by `&` / `&mut` reference (`CODING_RULES §2`).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeState {
    /// The current, dynamically interpolated palette displayed this frame.
    pub current: ThemePalette,
    /// The target palette towards which `current` is morphing.
    pub target: ThemePalette,
    /// The actively selected theme preset (or `None` if custom).
    pub active_preset: Option<ThemePreset>,
    /// Whether a theme morph animation is actively in progress.
    pub animating: bool,
    /// Exponential lerp morphing speed (default `9.0`).
    pub morph_speed: f32,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self::new(ThemePreset::CatppuccinMocha)
    }
}

impl ThemeState {
    /// Creates a new `ThemeState` initialized with the given preset.
    pub fn new(initial_preset: ThemePreset) -> Self {
        let palette = initial_preset.palette();
        Self {
            current: palette.clone(),
            target: palette,
            active_preset: Some(initial_preset),
            animating: false,
            morph_speed: 9.0,
        }
    }

    /// Creates a `ThemeState` initialized with a custom `ThemePalette`.
    pub fn from_palette(palette: ThemePalette) -> Self {
        Self {
            current: palette.clone(),
            target: palette,
            active_preset: None,
            animating: false,
            morph_speed: 9.0,
        }
    }

    /// Sets a new target preset to smoothly morph towards.
    pub fn set_preset(&mut self, preset: ThemePreset) {
        self.active_preset = Some(preset);
        self.target = preset.palette();
        self.animating = true;
    }

    /// Sets a new target preset instantly without animation.
    pub fn set_preset_instant(&mut self, preset: ThemePreset) {
        let palette = preset.palette();
        self.active_preset = Some(preset);
        self.target = palette.clone();
        self.current = palette;
        self.animating = false;
    }

    /// Sets a custom target palette to smoothly morph towards.
    pub fn set_palette(&mut self, palette: ThemePalette) {
        self.active_preset = None;
        self.target = palette;
        self.animating = true;
    }

    /// Advances the smooth theme morphing animation by `dt` seconds and requests a repaint if animating.
    pub fn update(&mut self, dt: f32, ctx: &Context) {
        if self.animating {
            let changed = self.current.interpolate(&self.target, dt, self.morph_speed);
            if changed {
                ctx.request_repaint();
            } else {
                self.animating = false;
            }
        }
    }

    /// Synchronizes the active palette directly with `egui::Context`, instantly styling all native egui widgets.
    pub fn apply_to_ctx(&self, ctx: &Context) {
        let p = &self.current;

        let mut visuals = if p.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        // Window & Panel backgrounds
        visuals.dark_mode = p.dark;
        visuals.panel_fill = p.base;
        visuals.window_fill = p.base;
        visuals.window_stroke = Stroke::new(1.0, p.surface0);
        visuals.extreme_bg_color = p.crust;
        visuals.faint_bg_color = p.mantle;
        visuals.code_bg_color = p.surface0;

        // Widget fills
        visuals.widgets.noninteractive.bg_fill = p.surface0;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.surface1);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text);

        visuals.widgets.inactive.bg_fill = p.surface0;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.surface1);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);

        visuals.widgets.hovered.bg_fill = p.surface1;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.overlay0);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);

        visuals.widgets.active.bg_fill = p.surface2;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, p.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, p.text);

        visuals.widgets.open.bg_fill = p.surface1;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, p.accent);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, p.text);

        // Selection & Accents
        visuals.selection.bg_fill = p.accent.linear_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0, p.accent);
        visuals.hyperlink_color = p.info;
        visuals.warn_fg_color = p.warning;
        visuals.error_fg_color = p.danger;

        ctx.set_visuals(visuals);
    }
}
