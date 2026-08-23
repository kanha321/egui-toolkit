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

            // 1. Primary Text Font (JetBrains Mono)
            fonts.font_data.insert(
                "jetbrains_mono".to_owned(),
                egui::FontData::from_static(include_bytes!("../resources/JetBrainsMonoNerdFont-Regular.ttf")),
            );

            // 2. Separate Dedicated Fallback Font for Nerd Font Symbols / Icons
            fonts.font_data.insert(
                "symbols_nerd_font".to_owned(),
                egui::FontData::from_static(include_bytes!("../resources/SymbolsNerdFont-Regular.ttf")),
            );

            // Configure Proportional family: primary text font first, symbols fallback second
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                vec.insert(0, "jetbrains_mono".to_owned());
                vec.push("symbols_nerd_font".to_owned());
            }

            // Configure Monospace family: primary text font first, symbols fallback second
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.insert(0, "jetbrains_mono".to_owned());
                vec.push("symbols_nerd_font".to_owned());
            }

            cc.egui_ctx.set_fonts(fonts);

            Box::new(app::TestAppState::default())
        }),
    )
}
