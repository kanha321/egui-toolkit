//! Standalone example demonstrating `egui-themes` palette switching and live component preview.
//!
//! Run with:
//! ```bash
//! cargo run -p egui-themes --example theme_switcher
//! ```

use eframe::egui;
use egui_themes::{ThemeGallery, ThemePreset, ThemePreview, ThemeState, ThemeToken};

struct ThemeSwitcherApp {
    theme: ThemeState,
    custom_accent: egui::Color32,
}

impl Default for ThemeSwitcherApp {
    fn default() -> Self {
        let theme = ThemeState::new(ThemePreset::CatppuccinMocha);
        let custom_accent = theme.current.accent;
        Self {
            theme,
            custom_accent,
        }
    }
}

impl eframe::App for ThemeSwitcherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt.min(0.1));
        self.theme.update(dt, ctx);
        self.theme.apply_to_ctx(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎨 egui-themes — Single Source of Truth Theme Engine");
            ui.label("Curated palette presets, smooth animated morphing, and live component preview.");
            ui.add_space(8.0);

            // 1. Live Component Preview Widget
            ui.group(|ui| {
                ui.label(egui::RichText::new("Live Component Preview").strong());
                ThemePreview::new()
                    .height(180.0)
                    .show(ui, &self.theme.current);
            });

            ui.add_space(10.0);

            // 2. Responsive Theme Presets Gallery Grid
            ui.label(egui::RichText::new("Curated Theme Presets:").strong());
            if let Some(new_preset) = ThemeGallery::new().show(ui, self.theme.active_preset) {
                self.theme.set_preset(new_preset);
                self.custom_accent = self.theme.target.accent;
            }

            ui.add_space(10.0);
            ui.separator();

            // 3. Custom Token Tweaker
            ui.horizontal(|ui| {
                ui.label("Custom Accent Color:");
                if ui.color_edit_button_srgba(&mut self.custom_accent).changed() {
                    let mut modified = self.theme.current.clone();
                    modified.set(ThemeToken::Accent, self.custom_accent);
                    self.theme.set_palette(modified);
                }

                if ui.button("Reset to Preset").clicked() {
                    if let Some(preset) = self.theme.active_preset {
                        self.theme.set_preset(preset);
                    } else {
                        self.theme.set_preset(ThemePreset::CatppuccinMocha);
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 680.0])
            .with_min_inner_size([480.0, 400.0])
            .with_title("egui-themes — Theme Switcher Example"),
        ..Default::default()
    };

    eframe::run_native(
        "egui-themes Example",
        options,
        Box::new(|_cc| Box::new(ThemeSwitcherApp::default())),
    )
}
