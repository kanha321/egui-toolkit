//! Live interactive and animated mock component preview widget.

use egui::{
    Align, Align2, Color32, FontId, Layout, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui,
    Vec2,
};
use crate::palette::ThemePalette;

/// An immediate-mode widget rendering a live, animated mock application preview of a [`ThemePalette`].
///
/// Displays a scaled mock interface consisting of:
/// - macOS-style window traffic light dots.
/// - Mock sidebar navigation column.
/// - System options card with toggle switch.
/// - Controls card with pulsating status dot, interactive slider, and animated spectrum visualizer.
#[derive(Clone, Debug, Default)]
pub struct ThemePreview {
    height: Option<f32>,
}

impl ThemePreview {
    /// Creates a new `ThemePreview` widget.
    pub fn new() -> Self {
        Self { height: None }
    }

    /// Sets an explicit height for the preview container. If not specified, calculates a responsive height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Renders the preview widget using the given palette colors.
    pub fn show(self, ui: &mut Ui, palette: &ThemePalette) -> Response {
        let available_w = ui.available_width();
        let preview_height = self
            .height
            .unwrap_or_else(|| (available_w * 0.30).clamp(140.0, 240.0));
        let scale = preview_height / 160.0;
        let time = ui.input(|i| i.time);

        let (total_rect, response) =
            ui.allocate_exact_size(Vec2::new(available_w, preview_height), Sense::hover());

        // Container frame
        ui.painter()
            .rect_filled(total_rect, Rounding::same(8.0), palette.crust);
        ui.painter().rect_stroke(
            total_rect,
            Rounding::same(8.0),
            Stroke::new(1.0, palette.surface1),
        );

        // 1. Mock Header Bar
        let header_h = 22.0 * scale;
        let header_rect = Rect::from_min_size(total_rect.min, Vec2::new(total_rect.width(), header_h));

        // macOS Window control dots
        ui.painter().circle_filled(
            header_rect.min + Vec2::new(12.0 * scale, 11.0 * scale),
            4.0 * scale,
            Color32::from_rgb(255, 95, 86),
        );
        ui.painter().circle_filled(
            header_rect.min + Vec2::new(24.0 * scale, 11.0 * scale),
            4.0 * scale,
            Color32::from_rgb(255, 189, 46),
        );
        ui.painter().circle_filled(
            header_rect.min + Vec2::new(36.0 * scale, 11.0 * scale),
            4.0 * scale,
            Color32::from_rgb(40, 201, 64),
        );

        // Title
        let font_id = FontId::monospace(10.0 * scale);
        ui.painter().text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            "LIVE COMPONENT PREVIEW",
            font_id,
            palette.overlay1,
        );

        // Header bottom divider
        ui.painter().line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            Stroke::new(1.0, palette.surface1),
        );

        // 2. Lower Area Layout (Sidebar + Content)
        let lower_rect = Rect::from_min_max(
            header_rect.left_bottom(),
            total_rect.right_bottom(),
        );

        let mut child_ui = ui.child_ui(lower_rect, Layout::left_to_right(Align::Min));
        child_ui.set_clip_rect(child_ui.clip_rect().intersect(lower_rect));

        // 3. Mock Left Sidebar
        let sidebar_w = 115.0 * scale;
        let sidebar_rect = Rect::from_min_size(lower_rect.min, Vec2::new(sidebar_w, lower_rect.height()));
        child_ui.painter().rect_filled(
            sidebar_rect,
            Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 7.0,
                se: 0.0,
            },
            palette.mantle,
        );
        child_ui.painter().line_segment(
            [sidebar_rect.right_top(), sidebar_rect.right_bottom()],
            Stroke::new(1.0, palette.surface1),
        );

        let mut side_ui = child_ui.child_ui(sidebar_rect.shrink(6.0 * scale), Layout::top_down(Align::Min));
        side_ui.vertical(|ui| {
            ui.painter().text(
                ui.cursor().min + Vec2::new(0.0, 4.0 * scale),
                Align2::LEFT_TOP,
                "NAVIGATION",
                FontId::proportional(8.0 * scale),
                palette.info,
            );
            ui.add_space(16.0 * scale);

            ui.painter().text(
                ui.cursor().min + Vec2::new(4.0 * scale, 0.0),
                Align2::LEFT_TOP,
                "📁 Dashboard",
                FontId::proportional(9.0 * scale),
                palette.text,
            );
            ui.add_space(14.0 * scale);

            // Active item highlight
            let active_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(sidebar_w - 16.0 * scale, 16.0 * scale));
            ui.painter().rect_filled(active_rect, Rounding::same(3.0 * scale), palette.surface0);
            ui.painter().text(
                active_rect.left_center() + Vec2::new(6.0 * scale, 0.0),
                Align2::LEFT_CENTER,
                "⚡ Live Views",
                FontId::proportional(9.0 * scale),
                palette.accent,
            );
            ui.add_space(20.0 * scale);

            ui.painter().text(
                ui.cursor().min + Vec2::new(4.0 * scale, 0.0),
                Align2::LEFT_TOP,
                "⚙ Settings",
                FontId::proportional(9.0 * scale),
                palette.subtext0,
            );
        });

        // 4. Mock Right Content Area
        let content_rect = Rect::from_min_max(
            Pos2::new(sidebar_rect.right(), lower_rect.top()),
            lower_rect.right_bottom(),
        );
        let mut main_ui = child_ui.child_ui(content_rect.shrink(8.0 * scale), Layout::top_down(Align::Min));

        main_ui.vertical(|ui| {
            // Card 1: System Options
            let card1_h = 32.0 * scale;
            let (card1_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), card1_h), Sense::hover());
            ui.painter().rect_filled(card1_rect, Rounding::same(5.0 * scale), palette.mantle);
            ui.painter().rect_stroke(card1_rect, Rounding::same(5.0 * scale), Stroke::new(1.0, palette.surface0));

            // Card 1 Header & Switch
            ui.painter().text(
                card1_rect.left_center() + Vec2::new(8.0 * scale, 0.0),
                Align2::LEFT_CENTER,
                "⚙ System Service",
                FontId::proportional(9.0 * scale),
                palette.sys_controls,
            );

            // Mock switch
            let sw_rect = Rect::from_min_size(
                card1_rect.right_center() + Vec2::new(-24.0 * scale, -6.0 * scale),
                Vec2::new(18.0 * scale, 12.0 * scale),
            );
            ui.painter().rect_filled(sw_rect, Rounding::same(6.0 * scale), palette.success.linear_multiply(0.35));
            ui.painter().rect_stroke(sw_rect, Rounding::same(6.0 * scale), Stroke::new(1.0, palette.success));
            ui.painter().circle_filled(
                Pos2::new(sw_rect.right() - 6.0 * scale, sw_rect.center().y),
                4.0 * scale,
                palette.success,
            );

            ui.add_space(6.0 * scale);

            // Card 2: Effect Controls & Spectrum Visualizer
            let card2_h = ui.available_height().max(40.0 * scale);
            let (card2_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), card2_h), Sense::hover());
            ui.painter().rect_filled(card2_rect, Rounding::same(5.0 * scale), palette.mantle);
            ui.painter().rect_stroke(card2_rect, Rounding::same(5.0 * scale), Stroke::new(1.0, palette.surface0));

            // Card 2 Header with Pulsing status indicator
            ui.painter().text(
                card2_rect.left_top() + Vec2::new(8.0 * scale, 8.0 * scale),
                Align2::LEFT_TOP,
                "🎛 Physics Visualizer",
                FontId::proportional(9.0 * scale),
                palette.warning,
            );

            // Pulsing dot
            let pulse = (time * 4.0).sin().abs() as f32;
            let dot_center = card2_rect.right_top() + Vec2::new(-14.0 * scale, 12.0 * scale);
            ui.painter().circle_filled(
                dot_center,
                (2.0 + pulse * 2.0) * scale,
                palette.success.linear_multiply(0.3),
            );
            ui.painter().circle_filled(dot_center, 2.5 * scale, palette.success);

            // Slider track & thumb
            let slider_y = card2_rect.center().y + 8.0 * scale;
            let slider_left = card2_rect.left() + 8.0 * scale;
            let slider_w = (card2_rect.width() * 0.40).clamp(40.0 * scale, 90.0 * scale);
            ui.painter().line_segment(
                [Pos2::new(slider_left, slider_y), Pos2::new(slider_left + slider_w, slider_y)],
                Stroke::new(2.0 * scale, palette.surface1),
            );
            ui.painter().line_segment(
                [Pos2::new(slider_left, slider_y), Pos2::new(slider_left + slider_w * 0.65, slider_y)],
                Stroke::new(2.0 * scale, palette.accent),
            );
            ui.painter().circle_filled(
                Pos2::new(slider_left + slider_w * 0.65, slider_y),
                4.0 * scale,
                palette.accent,
            );

            // Spectrum Equalizer Bars
            let n_bars = 10;
            let bar_w = 3.0 * scale;
            let spacing = 2.0 * scale;
            let spec_w = n_bars as f32 * bar_w + (n_bars - 1) as f32 * spacing;
            let spec_h = 16.0 * scale;
            let spec_x = card2_rect.right() - spec_w - 8.0 * scale;
            let spec_base_y = card2_rect.bottom() - 8.0 * scale;

            for i in 0..n_bars {
                let wave = (time * 5.0 + i as f64 * 0.75).sin() * 0.45
                    + (time * 2.5 - i as f64 * 0.35).cos() * 0.35
                    + 0.20;
                let val = (wave as f32).clamp(0.08, 1.0);
                let bar_h = val * spec_h;

                let bx = spec_x + i as f32 * (bar_w + spacing);
                let b_rect = Rect::from_min_max(
                    Pos2::new(bx, spec_base_y - bar_h),
                    Pos2::new(bx + bar_w, spec_base_y),
                );
                ui.painter().rect_filled(b_rect, Rounding::same(1.0 * scale), palette.accent);
            }
        });

        // Continuous repaint for smooth mock animations
        ui.ctx().request_repaint();

        response
    }
}
