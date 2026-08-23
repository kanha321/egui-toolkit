//! Standalone grid navigation example for `egui-vim-nav`.
//!
//! Run with: `cargo run -p egui-vim-nav --example grid_navigation`

use eframe::egui;
use egui_vim_nav::{FocusGraph, FocusRegion, Navigator, VimKeyHandler};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 400.0])
            .with_title("egui-vim-nav: Grid Navigation"),
        ..Default::default()
    };

    eframe::run_native(
        "grid_navigation",
        options,
        Box::new(|_cc| Box::new(GridNavApp::default())),
    )
}

/// Node IDs for the 3×3 grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Card {
    A, B, C,
    D, E, F,
    G, H, I,
}

impl Card {
    fn label(self) -> &'static str {
        match self {
            Card::A => "A", Card::B => "B", Card::C => "C",
            Card::D => "D", Card::E => "E", Card::F => "F",
            Card::G => "G", Card::H => "H", Card::I => "I",
        }
    }
}

struct GridNavApp {
    graph: FocusGraph<Card>,
    navigator: Navigator<Card>,
    key_handler: VimKeyHandler,
    text_input: String,
}

impl Default for GridNavApp {
    fn default() -> Self {
        let mut graph = FocusGraph::new();
        graph.connect_grid(&[
            &[Card::A, Card::B, Card::C],
            &[Card::D, Card::E, Card::F],
            &[Card::G, Card::H, Card::I],
        ]);

        Self {
            graph,
            navigator: Navigator::new().with_initial_focus(Card::A),
            key_handler: VimKeyHandler::new(),
            text_input: String::new(),
        }
    }
}

impl eframe::App for GridNavApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard navigation
        self.key_handler.handle_input(ctx, &mut self.navigator, &self.graph);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Grid Navigation (HJKL / Arrows)");
            ui.separator();

            // Show current focus
            if let Some(focused) = self.navigator.focused() {
                ui.label(format!("Focused: {:?}", focused));
            } else {
                ui.label("No focus (press any arrow key)");
            }

            ui.add_space(8.0);

            // Text input to test the input guard
            ui.horizontal(|ui| {
                ui.label("Type here (HJKL should not navigate):");
                ui.text_edit_singleline(&mut self.text_input);
            });

            ui.add_space(8.0);

            // Render 3×3 grid of cards
            let cards = [
                [Card::A, Card::B, Card::C],
                [Card::D, Card::E, Card::F],
                [Card::G, Card::H, Card::I],
            ];

            for row in &cards {
                ui.horizontal(|ui| {
                    for &card in row {
                        let resp = FocusRegion::show(
                            ui,
                            &mut self.navigator,
                            &card,
                            |ui, focused| {
                                let size = egui::vec2(80.0, 60.0);
                                let (rect, _) = ui.allocate_exact_size(
                                    size,
                                    egui::Sense::hover(),
                                );

                                let bg = if focused {
                                    egui::Color32::from_rgb(60, 100, 180)
                                } else {
                                    egui::Color32::from_gray(40)
                                };
                                let stroke = if focused {
                                    egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
                                } else {
                                    egui::Stroke::new(1.0, egui::Color32::from_gray(80))
                                };

                                ui.painter().rect(rect, 6.0, bg, stroke);
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    card.label(),
                                    egui::FontId::proportional(18.0),
                                    if focused {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_gray(180)
                                    },
                                );
                            },
                        );
                        let _ = resp;
                    }
                });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label("Keys: H=Left  J=Down  K=Up  L=Right  (also Arrow keys)");
            ui.label("Click a card to focus it. Type in the text box to test input guard.");
        });
    }
}
