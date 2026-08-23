//! Responsive swatch gallery card grid for selecting theme presets.

use egui::{
    Align, Align2, FontId, Layout, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use crate::palette::ThemePreset;

/// An immediate-mode gallery widget displaying all theme presets in a responsive card grid.
///
/// Each preset card displays:
/// - Preset name and mode badge (Dark / Light).
/// - Active selection indicator dot.
/// - Bottom designer color palette strip with rounded corners matching the card frame.
#[derive(Clone, Debug)]
pub struct ThemeGallery {
    card_min_width: f32,
    card_height: f32,
    spacing: Vec2,
}

impl Default for ThemeGallery {
    fn default() -> Self {
        Self {
            card_min_width: 170.0,
            card_height: 54.0,
            spacing: Vec2::new(10.0, 10.0),
        }
    }
}

impl ThemeGallery {
    /// Creates a new `ThemeGallery` with standard defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum card width for responsive column wrapping (default `170.0`).
    pub fn card_min_width(mut self, width: f32) -> Self {
        self.card_min_width = width;
        self
    }

    /// Sets the card height (default `54.0`).
    pub fn card_height(mut self, height: f32) -> Self {
        self.card_height = height;
        self
    }

    /// Sets the item spacing between cards (default `(10.0, 10.0)`).
    pub fn spacing(mut self, spacing: Vec2) -> Self {
        self.spacing = spacing;
        self
    }

    /// Renders the theme gallery grid.
    ///
    /// Returns `Some(preset)` if a card was clicked this frame.
    pub fn show(self, ui: &mut Ui, active_preset: Option<ThemePreset>) -> Option<ThemePreset> {
        let mut clicked_preset = None;
        let presets = ThemePreset::all();
        let total_items = presets.len();

        let available_w = ui.available_width();
        let n_cols = (((available_w + self.spacing.x) / (self.card_min_width + self.spacing.x))
            .floor() as usize)
            .max(1);
        let col_width = (available_w - (n_cols - 1) as f32 * self.spacing.x) / n_cols as f32;
        let rows = (total_items as f32 / n_cols as f32).ceil() as usize;

        for row in 0..rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                for col in 0..n_cols {
                    let idx = row * n_cols + col;
                    if idx >= total_items {
                        break;
                    }

                    let preset = presets[idx];
                    let is_selected = active_preset == Some(preset);
                    let palette = preset.palette();

                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(col_width, self.card_height),
                        Sense::click(),
                    );

                    let is_hovered = response.hovered();

                    // Colors
                    let bg_color = if is_hovered {
                        palette.surface0
                    } else {
                        palette.mantle
                    };

                    let stroke_color = if is_selected {
                        palette.success
                    } else if is_hovered {
                        palette.accent
                    } else {
                        palette.surface0
                    };

                    // Draw card background & border
                    ui.painter()
                        .rect_filled(rect, Rounding::same(6.0), bg_color);
                    ui.painter().rect_stroke(
                        rect,
                        Rounding::same(6.0),
                        Stroke::new(if is_selected { 1.5 } else { 1.0 }, stroke_color),
                    );

                    // Draw card text & active dot
                    let inner_rect = rect.shrink2(Vec2::new(8.0, 6.0));
                    let mut text_ui = ui.child_ui(inner_rect, Layout::top_down(Align::Min));

                    text_ui.horizontal(|ui| {
                        if is_selected {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(6.0, 6.0), Sense::hover());
                            ui.painter()
                                .circle_filled(dot_rect.center(), 3.0, palette.success);
                            ui.add_space(3.0);
                        }

                        let text_color = if is_selected {
                            palette.success
                        } else if is_hovered {
                            palette.accent
                        } else {
                            palette.text
                        };

                        ui.painter().text(
                            ui.cursor().min + Vec2::new(0.0, 2.0),
                            Align2::LEFT_TOP,
                            preset.name(),
                            FontId::proportional(11.0),
                            text_color,
                        );
                    });

                    // Draw designer color palette strip strictly along bottom edge
                    let palette_h = 5.0;
                    let border_thick = 1.0;
                    let palette_rect = Rect::from_min_max(
                        Pos2::new(rect.left() + border_thick, rect.bottom() - border_thick - palette_h),
                        Pos2::new(rect.right() - border_thick, rect.bottom() - border_thick),
                    );

                    let strip_colors = [
                        palette.base,
                        palette.accent,
                        palette.success,
                        palette.warning,
                        palette.danger,
                        palette.info,
                    ];
                    let n_colors = strip_colors.len();
                    let seg_w = palette_rect.width() / n_colors as f32;

                    for (c_idx, color) in strip_colors.into_iter().enumerate() {
                        let seg_rect = Rect::from_min_max(
                            Pos2::new(palette_rect.left() + c_idx as f32 * seg_w, palette_rect.top()),
                            Pos2::new(palette_rect.left() + (c_idx + 1) as f32 * seg_w, palette_rect.bottom()),
                        );
                        let rounding = if c_idx == 0 {
                            Rounding {
                                nw: 0.0,
                                ne: 0.0,
                                sw: 5.0,
                                se: 0.0,
                            }
                        } else if c_idx == n_colors - 1 {
                            Rounding {
                                nw: 0.0,
                                ne: 0.0,
                                sw: 0.0,
                                se: 5.0,
                            }
                        } else {
                            Rounding::ZERO
                        };
                        ui.painter().rect_filled(seg_rect, rounding, color);
                    }

                    if response.clicked() {
                        clicked_preset = Some(preset);
                    }

                    if col < n_cols - 1 {
                        ui.add_space(self.spacing.x);
                    }
                }
            });

            ui.add_space(self.spacing.y);
        }

        clicked_preset
    }
}
