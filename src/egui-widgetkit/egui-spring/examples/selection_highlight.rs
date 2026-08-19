use eframe::egui::{self, Color32, Rect, Rounding, Stroke, Vec2};
use egui_spring::SpringRect;
use spring_core::SpringParams;

struct SelectionHighlightExample {
    spring_rect: SpringRect,
    selected_index: usize,
}

impl Default for SelectionHighlightExample {
    fn default() -> Self {
        Self {
            spring_rect: SpringRect::new(Rect::ZERO)
                .with_fill(Color32::from_rgba_unmultiplied(137, 180, 250, 45))
                .with_stroke(Stroke::new(2.0, Color32::from_rgb(137, 180, 250)))
                .with_rounding(8.0)
                .with_params(SpringParams::snappy()),
            selected_index: 0,
        }
    }
}

impl eframe::App for SelectionHighlightExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        self.spring_rect.update(dt);

        if !self.spring_rect.is_settled() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui-spring Selection Highlight Demo");
            ui.label("Click any card to move the spring highlight with elastic smear physics.");
            ui.add_space(16.0);

            let items = [
                "Dashboard & Metrics",
                "User Management",
                "Device Configurations",
                "Network Telemetry",
                "Security & Audit Logs",
                "System Preferences",
            ];

            let mut target_rect = None;

            for (idx, label) in items.iter().enumerate() {
                let (rect, response) = ui.allocate_exact_size(Vec2::new(320.0, 48.0), egui::Sense::click());

                if response.clicked() {
                    self.selected_index = idx;
                }

                if idx == self.selected_index {
                    target_rect = Some(rect);
                }

                // Render item background card
                ui.painter().rect(
                    rect,
                    Rounding::same(8.0),
                    Color32::from_rgb(30, 30, 46),
                    Stroke::new(1.0, Color32::from_rgb(49, 50, 68)),
                );

                ui.painter().text(
                    rect.left_center() + Vec2::new(16.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    *label,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(205, 214, 244),
                );

                ui.add_space(8.0);
            }

            if let Some(target) = target_rect {
                self.spring_rect.set_target(target);
            }

            // Draw the spring-animated highlight over the active selection
            self.spring_rect.paint(ui.painter());
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-spring Example")
            .with_inner_size([400.0, 460.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui-spring-example",
        options,
        Box::new(|_cc| Box::new(SelectionHighlightExample::default())),
    )
}
