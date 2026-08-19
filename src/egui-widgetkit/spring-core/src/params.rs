#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping_ratio: f32,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self::snappy()
    }
}

impl SpringParams {
    pub fn gentle() -> Self {
        Self { stiffness: 120.0, damping_ratio: 1.2 }
    }

    pub fn snappy() -> Self {
        Self { stiffness: 240.0, damping_ratio: 1.0 }
    }

    pub fn bouncy() -> Self {
        Self { stiffness: 300.0, damping_ratio: 0.65 }
    }
}
