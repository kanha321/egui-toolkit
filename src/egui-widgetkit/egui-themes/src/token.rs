use egui::Color32;

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeToken {
    pub name: String,
    pub color: Color32,
}
