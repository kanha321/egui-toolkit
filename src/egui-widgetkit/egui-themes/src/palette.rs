use egui::Color32;

#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub name: String,
    pub base: Color32,
    pub mantle: Color32,
    pub accent: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            base: Color32::from_rgb(30, 30, 46),
            mantle: Color32::from_rgb(24, 24, 37),
            accent: Color32::from_rgb(203, 166, 247),
        }
    }
}
