use eframe::egui;
use super::state::{ActiveScene, TestAppState};
use crate::scenes::{combined_demo, layout_demo, nav_stack_demo, spring_demo, theme_demo, vim_nav_demo};

impl eframe::App for TestAppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Dynamically enforce minimum window size based on active scene layout calculation
        let dynamic_min_size = match self.active_scene {
            ActiveScene::Layout => layout_demo::min_layout_size(),
            ActiveScene::Spring => egui::Vec2::new(600.0, 420.0),
            _ => egui::Vec2::new(400.0, 200.0),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(dynamic_min_size));

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_scene, ActiveScene::Layout, "Layout");
                ui.selectable_value(&mut self.active_scene, ActiveScene::Spring, "Spring");
                ui.selectable_value(&mut self.active_scene, ActiveScene::VimNav, "Vim Nav");
                ui.selectable_value(&mut self.active_scene, ActiveScene::NavStack, "Nav Stack");
                ui.selectable_value(&mut self.active_scene, ActiveScene::Theme, "Theme");
                ui.selectable_value(&mut self.active_scene, ActiveScene::Combined, "Combined");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_scene {
                ActiveScene::Layout => layout_demo::show(ui),
                ActiveScene::Spring => spring_demo::show(ui, &mut self.spring_demo),
                ActiveScene::VimNav => vim_nav_demo::show(ui, &mut self.vim_nav_demo),
                ActiveScene::NavStack => nav_stack_demo::show(ui),
                ActiveScene::Theme => theme_demo::show(ui),
                ActiveScene::Combined => combined_demo::show(ui),
            }
        });
    }
}
