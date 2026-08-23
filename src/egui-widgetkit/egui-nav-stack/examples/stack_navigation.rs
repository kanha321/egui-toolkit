//! Minimal standalone sanity-check example for `egui-nav-stack`.
//!
//! Demonstrates 3 simple placeholder destinations (`Home`, `List`, `Detail`),
//! push/pop navigation, and safe post-render stack mutation.

use eframe::egui::{self, CentralPanel, Color32, RichText, TopBottomPanel};
use egui_nav_stack::{NavAction, NavDisplay, NavStack};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Screen {
    Home,
    List,
    Detail(usize),
}

struct StackNavApp {
    stack: NavStack<Screen>,
}

impl Default for StackNavApp {
    fn default() -> Self {
        Self {
            stack: NavStack::new(Screen::Home),
        }
    }
}

impl eframe::App for StackNavApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("egui-nav-stack Minimal Example");
                ui.separator();
                ui.label(format!("Stack depth: {}", self.stack.len()));

                if self.stack.can_pop() && ui.button("⬅ Back").clicked() {
                    self.stack.pop();
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            NavDisplay::new(&self.stack)
                .show(ui, |screen, ui| match screen {
                    Screen::Home => {
                        ui.label(RichText::new("🏠 Home Screen").size(20.0).strong());
                        ui.add_space(8.0);
                        ui.label("Welcome to the navigation stack demo.");
                        ui.add_space(16.0);
                        if ui.button("Browse Items ➔").clicked() {
                            Some(NavAction::Push(Screen::List))
                        } else {
                            None
                        }
                    }
                    Screen::List => {
                        ui.label(RichText::new("📋 Items List").size(20.0).strong());
                        ui.add_space(8.0);
                        let mut action = None;
                        for i in 1..=4 {
                            if ui.button(format!("Open Item #{}", i)).clicked() {
                                action = Some(NavAction::Push(Screen::Detail(i)));
                            }
                        }
                        ui.add_space(12.0);
                        if ui.button("⬅ Back to Home").clicked() {
                            action = Some(NavAction::Pop);
                        }
                        action
                    }
                    Screen::Detail(id) => {
                        ui.label(RichText::new(format!("🔍 Item #{} Details", id)).size(20.0).strong().color(Color32::from_rgb(137, 180, 250)));
                        ui.add_space(8.0);
                        ui.label(format!("This is the detailed view for item #{}.", id));
                        ui.add_space(16.0);
                        if ui.button("⬅ Back to List").clicked() {
                            Some(NavAction::Pop)
                        } else if ui.button("🏠 Pop to Root (Home)").clicked() {
                            Some(NavAction::PopToRoot)
                        } else {
                            None
                        }
                    }
                })
                .apply_to(&mut self.stack);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui-nav-stack example",
        options,
        Box::new(|_cc| Box::new(StackNavApp::default())),
    )
}
