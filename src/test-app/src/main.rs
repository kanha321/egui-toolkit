#![windows_subsystem = "windows"]

mod app;
mod scenes;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-widgetkit test app")
            .with_inner_size([1120.0, 740.0])
            .with_min_inner_size([900.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "test-app",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "jetbrains_mono_nf".to_owned(),
                egui::FontData::from_static(include_bytes!("../resources/JetBrainsMonoNerdFont-Regular.ttf")),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "jetbrains_mono_nf".to_owned());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "jetbrains_mono_nf".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Box::new(app::TestAppState::default())
        }),
    )
}
