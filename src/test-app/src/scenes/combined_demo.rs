use std::cell::Cell;
use egui::Ui;
use egui_layout::{CollapseMode, Section, Split, SplitState, SplitStyle};
use spring_core::MotionPhysics;

/// Combined multi-crate showcase demonstrating collapsible + resizable sections
/// with spring-animated transitions.
pub fn show(
    ui: &mut Ui,
    sidebar_open: &mut bool,
    toolbar_open: &mut bool,
    properties_open: &mut bool,
    motion: &mut MotionPhysics,
) {
    let is_sidebar_open = *sidebar_open;
    let is_toolbar_open = *toolbar_open;
    let is_props_open = *properties_open;

    let toggle_sidebar = Cell::new(false);
    let toggle_toolbar = Cell::new(false);
    let toggle_properties = Cell::new(false);
    let reset_layout = Cell::new(false);

    let ts = &toggle_sidebar;
    let tt = &toggle_toolbar;
    let tp = &toggle_properties;
    let tr = &reset_layout;

    let style = SplitStyle::default()
        .with_spacing(6.0)
        .with_card_padding(10.0)
        .with_card_rounding(8.0);

    let active_motion = *motion;

    Split::horizontal()
        .style(style.clone())
        .id_salt("combined_main")
        .motion(active_motion)
        .resizable(true)
        // ── Collapsible Sidebar → 40px icon rail ──
        .add_section(
            Section::fraction(0.22)
                .min_size(40.0)
                .card()
                .title("📁 Explorer")
                .collapsible(sidebar_open, CollapseMode::FixedBar(40.0))
                .collapsed_content(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);
                        if ui.button("📁").on_hover_text("Click to expand Explorer").clicked() {
                            ts.set(true);
                        }
                        ui.add_space(4.0);
                        ui.label("🔍");
                        ui.add_space(4.0);
                        ui.label("⚙");
                    });
                })
                .content(|ui| {
                    ui.separator();
                    ui.label("📁 src/");
                    ui.label("  📄 lib.rs");
                    ui.label("  📄 split.rs");
                    ui.label("  📄 section.rs");
                    ui.label("  📄 collapse.rs");
                    ui.label("  📄 resize.rs");
                    ui.label("📁 tests/");
                    ui.label("  📄 collapse_tests.rs");
                    ui.label("  📄 resize_tests.rs");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label("💡 Collapse to 40px Rail");
                }),
        )
        // ── Right area: nested vertical split ──
        .add_section(
            Section::remainder().content(|ui| {
                Split::vertical()
                    .style(style.clone())
                    .id_salt("combined_right")
                    .motion(active_motion)
                    .resizable(true)
                    // ── Collapsible top toolbar → header only ──
                    .add_section(
                        Section::fixed(80.0)
                            .card()
                            .title("⚡ Toolbar")
                            .collapsible(toolbar_open, CollapseMode::HeaderOnly)
                            .collapsed_content(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("⚡ Expand Toolbar").clicked() {
                                        tt.set(true);
                                    }
                                });
                            })
                            .content(|ui| {
                                ui.add_space(2.0);
                                ui.horizontal_wrapped(|ui| {
                                    let _ = ui.button("▶  Run");
                                    let _ = ui.button("⏸  Pause");
                                    let _ = ui.button("🔄  Restart");
                                    ui.separator();
                                    let _ = ui.button("📋  Copy");
                                    let _ = ui.button("📌  Pin");
                                });
                            }),
                    )
                    // ── Main workspace (nested horizontal) ──
                    .add_section(
                        Section::remainder().content(|ui| {
                            Split::horizontal()
                                .style(style.clone())
                                .id_salt("combined_workspace")
                                .motion(active_motion)
                                .resizable(true)
                                // ── Editor area with animation test controls ──
                                .add_section(
                                    Section::remainder()
                                        .min_size(160.0)
                                        .card()
                                        .title("🎨 Canvas & Animation Controls")
                                        .subtitle("Interactive spring test bench")
                                        .content(|ui| {
                                            ui.add_space(4.0);
                                            ui.heading("Spring Animation Test Controls");
                                            ui.label("Click any button to trigger smooth spring animations across panels:");
                                            ui.add_space(8.0);

                                            ui.group(|ui| {
                                                ui.label(egui::RichText::new("Section Collapse Toggles:").strong());
                                                ui.horizontal_wrapped(|ui| {
                                                    let sb_text = if is_sidebar_open { "📁 Collapse Explorer (40px Rail)" } else { "📁 Expand Explorer" };
                                                    if ui.button(sb_text).clicked() {
                                                        ts.set(true);
                                                    }

                                                    let tb_text = if is_toolbar_open { "⚡ Collapse Toolbar (HeaderOnly)" } else { "⚡ Expand Toolbar" };
                                                    if ui.button(tb_text).clicked() {
                                                        tt.set(true);
                                                    }

                                                    let prop_text = if is_props_open { "⚙ Collapse Properties (Hidden)" } else { "⚙ Expand Properties" };
                                                    if ui.button(prop_text).clicked() {
                                                        tp.set(true);
                                                    }
                                                });
                                            });

                                            ui.add_space(6.0);
                                            ui.group(|ui| {
                                                ui.label(egui::RichText::new("Motion Physics Preset:").strong());
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.selectable_value(motion, MotionPhysics::Responsive, "🎯 Fast & Bouncy (26 / 0.44)");
                                                    ui.selectable_value(motion, MotionPhysics::Bouncy, "🏀 Extra Bouncy (24 / 0.40)");
                                                    ui.selectable_value(motion, MotionPhysics::Snappy, "⚡ Snappy (32 / 0.85)");
                                                    ui.selectable_value(motion, MotionPhysics::Gentle, "🍃 Gentle (18 / 0.90)");
                                                    ui.selectable_value(motion, MotionPhysics::OpenRGB, "🌈 OpenRGB (22 / 0.65)");
                                                    ui.selectable_value(motion, MotionPhysics::Off, "⏱ Instant (Off)");
                                                });
                                            });

                                            ui.add_space(6.0);
                                            ui.group(|ui| {
                                                ui.label(egui::RichText::new("Divider & Layout Tools:").strong());
                                                ui.horizontal(|ui| {
                                                    if ui.button("🔄 Reset Dividers (Spring back)").clicked() {
                                                        tr.set(true);
                                                    }
                                                    ui.label("• Hover/drag dividers between any sections to resize");
                                                });
                                            });

                                            ui.add_space(8.0);
                                            ui.separator();
                                        }),
                                )
                                // ── Properties panel ──
                                .add_section(
                                    Section::fraction(0.30)
                                        .min_size(100.0)
                                        .card()
                                        .title("⚙ Properties")
                                        .collapsible(properties_open, CollapseMode::Hidden)
                                        .content(|ui| {
                                            ui.separator();
                                            ui.label("Position: (0, 0)");
                                            ui.label("Size: 100 × 100");
                                            ui.label("Rotation: 0°");
                                            ui.label("Opacity: 100%");
                                            ui.add_space(8.0);
                                            ui.separator();
                                            ui.label("💡 Collapses completely (Hidden)");
                                        }),
                                )
                                .show(ui);
                        }),
                    )
                    .show(ui);
            }),
        )
        .show(ui);

    if toggle_sidebar.get() {
        *sidebar_open = !*sidebar_open;
    }
    if toggle_toolbar.get() {
        *toolbar_open = !*toolbar_open;
    }
    if toggle_properties.get() {
        *properties_open = !*properties_open;
    }
    if reset_layout.get() {
        let mut s1 = SplitState::load(ui, egui::Id::new("combined_main"));
        s1.reset();
        s1.store(ui, egui::Id::new("combined_main"));

        let mut s2 = SplitState::load(ui, egui::Id::new("combined_right"));
        s2.reset();
        s2.store(ui, egui::Id::new("combined_right"));

        let mut s3 = SplitState::load(ui, egui::Id::new("combined_workspace"));
        s3.reset();
        s3.store(ui, egui::Id::new("combined_workspace"));
    }
}
