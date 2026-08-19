use egui::Context;
use super::palette::Palette;

#[derive(Clone, Debug)]
pub struct ThemeSwitcher {
    pub active: Palette,
}

impl Default for ThemeSwitcher {
    fn default() -> Self {
        Self { active: Palette::dark() }
    }
}

impl ThemeSwitcher {
    pub fn new(initial: Palette) -> Self {
        Self { active: initial }
    }

    pub fn set(&mut self, palette: Palette, _ctx: &Context) {
        self.active = palette;
    }
}
