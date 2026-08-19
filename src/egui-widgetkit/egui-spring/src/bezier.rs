use egui::Pos2;

pub const KAPPA: f32 = 0.55228475;

pub fn rounded_contour(corners: [Pos2; 4], _rounding: f32) -> Vec<Pos2> {
    corners.to_vec()
}
