//! Showcase scene exercising `egui-themes`.
//!
//! Demonstrates:
//! - Single source of truth palette architecture (`ThemePalette`).
//! - 12 curated theme presets (Catppuccin 4 flavors, Tokyo Night, Dracula, Monokai Pro, Gruvbox, etc.).
//! - Smooth exponential lerp theme morphing (`ThemeState::update`).
//! - Live mock component preview widget (`ThemePreview`).
//! - Responsive swatch gallery card grid (`ThemeGallery`).
//! - Live semantic token inspection & custom color overriding.

use egui::{RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use egui_layout::{Section, Split};
use egui_themes::{ThemeGallery, ThemePreset, ThemePreview, ThemeState, ThemeToken};

pub fn show(ui: &mut Ui, state: &mut ThemeState) {
    let current_palette = state.current.clone();
    let active_preset = state.active_preset;
    let mut morph_speed = state.morph_speed;

    let mut custom_palette_change = None;
    let mut selected_preset_change = None;
    let mut reset_requested = false;

    Split::horizontal()
        .spacing(10.0)
        // Left Column: Live Preview & Token Inspector
        .add_section(
            Section::fraction(0.48)
                .min_size_2d(300.0, 200.0)
                .card()
                .title("🎨 Live Theme Preview")
                .subtitle("Real-time component preview & semantic token table")
                .padding(10.0)
                .content(|ui| {
                    ScrollArea::vertical()
                        .id_source("theme_left_scroll")
                        .show(ui, |ui| {
                            // 1. Live Component Preview
                            ui.label(RichText::new("Mock Application Preview:").strong());
                            ui.add_space(4.0);
                            ThemePreview::new()
                                .height(160.0)
                                .show(ui, &current_palette);

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // 2. Semantic Token Swatch Table
                            ui.label(RichText::new("Active Palette Tokens (Source of Truth):").strong());
                            ui.add_space(4.0);

                            let categories = ["Background", "Surfaces", "Overlays", "Typography", "Semantic"];
                            for cat in categories {
                                ui.collapsing(RichText::new(cat).strong(), |ui| {
                                    for token in ThemeToken::all() {
                                        if token.category() == cat {
                                            let mut color = current_palette.get(*token);
                                            ui.horizontal(|ui| {
                                                let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
                                                ui.painter().rect_filled(rect, Rounding::same(3.0), color);
                                                ui.painter().rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.0, current_palette.surface1));

                                                ui.label(token.name());
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.color_edit_button_srgba(&mut color).changed() {
                                                        let mut modified = current_palette.clone();
                                                        modified.set(*token, color);
                                                        custom_palette_change = Some(modified);
                                                    }
                                                    ui.monospace(format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()));
                                                });
                                            });
                                        }
                                    }
                                });
                            }
                        });
                }),
        )
        // Right Column: Presets Swatch Gallery & Controls
        .add_section(
            Section::remainder()
                .min_size_2d(280.0, 200.0)
                .card()
                .title("🏛 Curated Theme Presets")
                .subtitle("12 designer palettes with smooth morphing")
                .padding(10.0)
                .content(|ui| {
                    ScrollArea::vertical()
                        .id_source("theme_right_scroll")
                        .show(ui, |ui| {
                            // Controls bar
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Morph Speed:").strong());
                                ui.add(egui::Slider::new(&mut morph_speed, 1.0..=25.0).text("x"));

                                if ui.button("Reset Palette").clicked() {
                                    reset_requested = true;
                                }
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // Responsive Theme Gallery Card Grid
                            if let Some(new_preset) = ThemeGallery::new()
                                .card_min_width(150.0)
                                .card_height(52.0)
                                .show(ui, active_preset)
                            {
                                selected_preset_change = Some(new_preset);
                            }

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // Active Swatches Strip
                            ui.label(RichText::new("Designer Swatches Strip:").strong());
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                for (i, swatch) in current_palette.swatches.iter().enumerate() {
                                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, Rounding::same(4.0), *swatch);
                                    ui.painter().rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, current_palette.surface1));
                                    resp.on_hover_text(format!("Swatch #{}: RGB({}, {}, {})", i, swatch.r(), swatch.g(), swatch.b()));
                                }
                            });
                        });
                }),
        )
        .show(ui);

    // Apply mutations safely after rendering
    state.morph_speed = morph_speed;
    if let Some(modified) = custom_palette_change {
        state.set_palette(modified);
    }
    if let Some(preset) = selected_preset_change {
        state.set_preset(preset);
    }
    if reset_requested {
        if let Some(preset) = state.active_preset {
            state.set_preset_instant(preset);
        } else {
            state.set_preset(ThemePreset::CatppuccinMocha);
        }
    }
}
