use egui::Ui;
use egui_layout::{Section, Split, SplitStyle};
use egui_themes::ThemePalette;

/// Inter-section spacing in logical points.
const SECTION_SPACING: f32 = 6.0;

/// Dynamically calculates the minimum window dimensions required to satisfy all nested section constraints.
pub fn min_layout_size() -> egui::Vec2 {
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
    egui::Vec2::new(min_bounds.x + 24.0, min_bounds.y + 60.0)
}

pub fn show(ui: &mut Ui, palette: &ThemePalette) {
    // Global style for all sections — developers configure once, all cards inherit
    let style = SplitStyle::default()
        .with_spacing(SECTION_SPACING)
        .with_card_rounding(10.0)
        .with_card_padding(10.0);

    Split::horizontal()
        .style(style.clone())
        // Section 1: Navigation / Explorer (Left column)
        .add_section(
            Section::fraction(0.24)
                .min_size_2d(150.0, 150.0)
                .card()
                .title("📁 Project Explorer")
                .subtitle("Section 1 • Left (24% min 150pt)")
                .title_color(palette.info)
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
                    .style(style.clone())
                    // Section 2: Top Command Ribbon (Fixed 48pt height)
                    .add_section(
                        Section::fixed(48.0)
                            .min_cross(200.0)
                            .card()
                            .title("⚡ Command Ribbon")
                            .subtitle("Section 2 • Top (Fixed 48pt)")
                            .title_color(palette.success)
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
                                .style(style.clone())
                                // Section 3: Central Canvas / Viewport (Flexible remainder)
                                .add_section(
                                    Section::remainder()
                                        .min_size_2d(140.0, 80.0)
                                        .card()
                                        .title("🎨 Design Canvas / Viewport")
                                        .subtitle("Section 3 • Center (Flexible Remainder)")
                                        .title_color(palette.accent)
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
                                        .title_color(palette.warning)
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
