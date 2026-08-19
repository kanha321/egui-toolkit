use egui::{Response, Ui};
use super::switcher::ThemeSwitcher;

pub struct ThemePreview<'a> {
    pub switcher: &'a ThemeSwitcher,
}

impl<'a> ThemePreview<'a> {
    pub fn new(switcher: &'a ThemeSwitcher) -> Self {
        Self { switcher }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}
