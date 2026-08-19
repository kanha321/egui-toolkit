mod app;
mod scenes;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-widgetkit test app")
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size(scenes::layout_demo::min_layout_size()),
        ..Default::default()
    };

    eframe::run_native(
        "test-app",
        options,
        Box::new(|_cc| Box::new(app::TestAppState::default())),
    )
}
