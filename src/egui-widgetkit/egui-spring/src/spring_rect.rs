use egui::{Rect, Response, Ui};
use spring_core::SpringParams;
use super::corner_springs::CornerSprings;

#[derive(Clone, Debug)]
pub struct SpringRect {
    pub corners: CornerSprings,
    pub target: Rect,
}

impl SpringRect {
    pub fn new(target: Rect) -> Self {
        Self {
            corners: CornerSprings::new(SpringParams::snappy()),
            target,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}
