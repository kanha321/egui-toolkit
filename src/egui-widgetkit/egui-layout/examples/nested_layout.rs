//! Isolated visual sanity check for `egui-layout`.
//!
//! Run with:
//! ```bash
//! cargo run -p egui-layout --example nested_layout
//! ```

use eframe::egui;
use egui::{Color32, Frame, Margin, Rounding, Stroke};
use egui_layout::Split;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-layout — Nested Layout Example")
            .with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-layout demo",
        options,
        Box::new(|_cc| Box::new(NestedLayoutApp::default())),
    )
}

struct NestedLayoutApp {
    spacing: f32,
    sidebar_fraction: f32,
    header_fraction: f32,
}

impl Default for NestedLayoutApp {
    fn default() -> Self {
        Self {
            spacing: 6.0,
            sidebar_fraction: 0.30,
            header_fraction: 0.25,
        }
    }
}

impl eframe::App for NestedLayoutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Spacing (px):");
                ui.add(egui::Slider::new(&mut self.spacing, 0.0..=20.0).suffix("px"));
                ui.separator();
                ui.label("Sidebar Fraction:");
                ui.add(egui::Slider::new(&mut self.sidebar_fraction, 0.1..=0.9));
                ui.separator();
                ui.label("Header Fraction:");
                ui.add(egui::Slider::new(&mut self.header_fraction, 0.1..=0.9));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            Split::horizontal()
                .resizable(true)
                .spacing(self.spacing)
                // Left Column
                .section(self.sidebar_fraction, |ui| {
                    draw_card(ui, "Sidebar (Master List)", Color32::from_rgb(30, 32, 48));
                })
                // Right Column (Nested vertical split)
                .section(1.0 - self.sidebar_fraction, |ui| {
                    Split::vertical()
                        .resizable(true)
                        .spacing(self.spacing)
                        .section(self.header_fraction, |ui| {
                            draw_card(ui, "Top Controls / Metrics", Color32::from_rgb(40, 44, 60));
                        })
                        .section(1.0 - self.header_fraction, |ui| {
                            draw_card(ui, "Main Workspace / Content", Color32::from_rgb(24, 26, 38));
                        })
                        .show(ui);
                })
                .show(ui);
        });
    }
}

fn draw_card(ui: &mut egui::Ui, title: &str, fill: Color32) {
    Frame::none()
        .fill(fill)
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 85)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());
            ui.heading(title);
            ui.label(format!("Available Dimensions: {:.0} x {:.0} pt", ui.available_width(), ui.available_height()));
        });
}
