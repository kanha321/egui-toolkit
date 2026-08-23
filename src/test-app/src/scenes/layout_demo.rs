use egui::{Color32, Stroke, Ui, Vec2};
use egui_layout::{Section, Split};

/// Inter-section spacing in logical points.
const SECTION_SPACING: f32 = 6.0;

/// Dynamically calculates the minimum window dimensions required to satisfy all nested section constraints.
pub fn min_layout_size() -> Vec2 {
    // Automatically computed using the declarative 2D constraint tree
    let inner_workspace = Split::horizontal()
        .spacing(SECTION_SPACING)
        .add_section(Section::remainder().min_size_2d(140.0, 80.0))
        .add_section(Section::fraction(0.35).min_size_2d(160.0, 80.0));

    let right_area = Split::vertical()
        .spacing(SECTION_SPACING)
        .add_section(Section::fixed(48.0).min_cross(200.0))
        .add_section(Section::remainder().min_size_2d(inner_workspace.min_width(), inner_workspace.min_height()));

    let full_layout = Split::horizontal()
        .spacing(SECTION_SPACING)
        .add_section(Section::fraction(0.24).min_size_2d(150.0, 150.0))
        .add_section(Section::remainder().min_size_2d(right_area.min_width(), right_area.min_height()));

    let min_bounds = full_layout.min_size();
    Vec2::new(min_bounds.x + 24.0, min_bounds.y + 60.0)
}

pub fn show(ui: &mut Ui) {
    Split::horizontal()
        .spacing(SECTION_SPACING)
        // Section 1: Navigation / Explorer (Left column)
        .add_section(
            Section::fraction(0.24)
                .min_size_2d(150.0, 150.0)
                .card()
                .title("📁 Project Explorer")
                .subtitle("Section 1 • Left (24% min 150pt)")
                .title_color(Color32::from_rgb(137, 180, 250))
                .bg(Color32::from_rgb(30, 32, 48))
                .stroke(Stroke::new(1.0, Color32::from_rgb(69, 71, 90)))
                .padding(10.0)
                .content(|ui| {
                    ui.separator();
                    ui.label("📁 src/");
                    ui.label("  📄 lib.rs");
                    ui.label("  📄 split.rs");
                    ui.label("  📄 section.rs");
                    ui.label("  📄 size.rs");
                }),
        )
        // Right Area containing Sections 2, 3, and 4
        .add_section(
            Section::remainder().content(|ui| {
                Split::vertical()
                    .spacing(SECTION_SPACING)
                    // Section 2: Top Command Ribbon (Fixed 48pt height)
                    .add_section(
                        Section::fixed(48.0)
                            .min_cross(200.0)
                            .card()
                            .title("⚡ Command Ribbon")
                            .subtitle("Section 2 • Top (Fixed 48pt)")
                            .title_color(Color32::from_rgb(166, 227, 161))
                            .bg(Color32::from_rgb(36, 39, 58))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(69, 71, 90)))
                            .padding(8.0)
                            .content(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("▶ Run").clicked() {}
                                    if ui.button("⏸ Pause").clicked() {}
                                    if ui.button("🔄 Reload").clicked() {}
                                });
                            }),
                    )
                    // Middle area split horizontally into Canvas & Inspector
                    .add_section(
                        Section::remainder().content(|ui| {
                            Split::horizontal()
                                .spacing(SECTION_SPACING)
                                // Section 3: Central Canvas / Viewport (Flexible remainder)
                                .add_section(
                                    Section::remainder()
                                        .min_size_2d(140.0, 80.0)
                                        .card()
                                        .title("🎨 Design Canvas / Viewport")
                                        .subtitle("Section 3 • Center (Flexible Remainder)")
                                        .title_color(Color32::from_rgb(203, 166, 247))
                                        .bg(Color32::from_rgb(24, 25, 38))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(69, 71, 90)))
                                        .padding(10.0)
                                        .content(|ui| {
                                            ui.separator();
                                            ui.label("Interactive visual canvas area.");
                                            ui.label("Auto-expands to fill all remaining space.");
                                        }),
                                )
                                // Section 4: Property Inspector (Right column)
                                .add_section(
                                    Section::fraction(0.35)
                                        .min_size_2d(160.0, 80.0)
                                        .card()
                                        .title("⚙️ Property Inspector")
                                        .subtitle("Section 4 • Right (35% min 160pt)")
                                        .title_color(Color32::from_rgb(249, 226, 175))
                                        .bg(Color32::from_rgb(32, 34, 52))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(69, 71, 90)))
                                        .padding(10.0)
                                        .content(|ui| {
                                            ui.separator();
                                            ui.label("Width constraint: 35% flex");
                                            ui.label("Minimum width: 160pt");
                                        }),
                                )
                                .show(ui);
                        }),
                    )
                    .show(ui);
            }),
        )
        .show(ui);
}
