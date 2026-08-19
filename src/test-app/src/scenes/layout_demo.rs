use egui::{Color32, Rounding, Stroke, Ui, Vec2};
use egui_layout::{Size, Split};

/// Spacing between split sections in logical points.
const SECTION_SPACING: f32 = 6.0;

/// Sidebar sizing policy: 24% width, minimum 150pt.
const SIDEBAR_SIZE: Size = Size::Fraction {
    fraction: 0.24,
    min: Some(150.0),
    max: None,
};

/// Top Command Ribbon sizing policy: fixed 48pt height.
const RIBBON_SIZE: Size = Size::Exact(48.0);

/// Inspector sizing policy: 35% width, minimum 160pt.
const INSPECTOR_SIZE: Size = Size::Fraction {
    fraction: 0.35,
    min: Some(160.0),
    max: None,
};

/// Canvas minimum size constraints.
const CANVAS_MIN_WIDTH: f32 = 140.0;
const CANVAS_MIN_HEIGHT: f32 = 80.0;

/// Dynamically calculates the minimum window dimensions required to satisfy all nested section constraints.
pub fn min_layout_size() -> Vec2 {
    // Width: Sidebar + Spacing + (Canvas min + Spacing + Inspector min)
    let workspace_min_w = Split::compute_min_length(
        SECTION_SPACING,
        &[
            Size::remainder().min_size(CANVAS_MIN_WIDTH),
            INSPECTOR_SIZE,
        ],
    );
    let total_min_w = Split::compute_min_length(
        SECTION_SPACING,
        &[
            SIDEBAR_SIZE,
            Size::remainder().min_size(workspace_min_w),
        ],
    );

    // Height: Ribbon + Spacing + Canvas min height + Top Navigation Bar (~36pt) + Padding (~24pt)
    let total_min_h = Split::compute_min_length(
        SECTION_SPACING,
        &[
            RIBBON_SIZE,
            Size::remainder().min_size(CANVAS_MIN_HEIGHT),
        ],
    ) + 60.0; // Margin + Top panel overhead

    Vec2::new(total_min_w + 24.0, total_min_h)
}

pub fn show(ui: &mut Ui) {
    Split::horizontal()
        .spacing(SECTION_SPACING)
        // Section 1: Navigation / Explorer (Left column)
        .section_custom(SIDEBAR_SIZE, |ui| {
            draw_section_card(
                ui,
                "📁 Project Explorer",
                "Section 1 • Left (24% min 150pt)",
                Color32::from_rgb(30, 32, 48),
                Color32::from_rgb(137, 180, 250),
            );
        })
        // Right Area containing Sections 2, 3, and 4
        .section_remainder(|ui| {
            Split::vertical()
                .spacing(SECTION_SPACING)
                // Section 2: Top Command Ribbon
                .section_custom(RIBBON_SIZE, |ui| {
                    draw_section_card(
                        ui,
                        "⚡ Command Ribbon",
                        "Section 2 • Top (Fixed 48pt)",
                        Color32::from_rgb(36, 39, 58),
                        Color32::from_rgb(166, 227, 161),
                    );
                })
                // Middle area split horizontally into Canvas & Inspector
                .section_remainder(|ui| {
                    Split::horizontal()
                        .spacing(SECTION_SPACING)
                        // Section 3: Central Canvas / Viewport (Flexible remainder)
                        .section_remainder(|ui| {
                            draw_section_card(
                                ui,
                                "🎨 Design Canvas / Viewport",
                                "Section 3 • Center (Flexible Remainder)",
                                Color32::from_rgb(24, 25, 38),
                                Color32::from_rgb(203, 166, 247),
                            );
                        })
                        // Section 4: Property Inspector (Right column)
                        .section_custom(INSPECTOR_SIZE, |ui| {
                            draw_section_card(
                                ui,
                                "⚙️ Property Inspector",
                                "Section 4 • Right (35% min 160pt)",
                                Color32::from_rgb(32, 34, 52),
                                Color32::from_rgb(249, 226, 175),
                            );
                        })
                        .show(ui);
                })
                .show(ui);
        })
        .show(ui);
}

fn draw_section_card(ui: &mut Ui, title: &str, subtitle: &str, bg: Color32, accent: Color32) {
    let available_rect = ui.available_rect_before_wrap();
    if available_rect.width() <= 0.0 || available_rect.height() <= 0.0 {
        return;
    }

    // Always render the card background and border with all 4 rounded corners intact
    ui.painter().rect(
        available_rect,
        Rounding::same(8.0),
        bg,
        Stroke::new(1.0, Color32::from_rgb(69, 71, 90)),
    );

    // Inner content area with inset margin
    let content_rect = available_rect.shrink(10.0);
    if content_rect.width() > 0.0 && content_rect.height() > 0.0 {
        let mut child_ui = ui.child_ui(content_rect, egui::Layout::top_down(egui::Align::Min));
        child_ui.set_clip_rect(child_ui.clip_rect().intersect(content_rect));

        child_ui.colored_label(accent, egui::RichText::new(title).strong().size(13.0));
        if content_rect.height() > 24.0 {
            child_ui.label(egui::RichText::new(subtitle).size(10.5).color(Color32::from_rgb(166, 173, 200)));
        }
        if content_rect.height() > 44.0 {
            child_ui.separator();
            child_ui.label(format!("Size: {:.0} x {:.0} pt", available_rect.width(), available_rect.height()));
        }
    }
}
