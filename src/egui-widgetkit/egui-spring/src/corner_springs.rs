use spring_core::{Spring, SpringParams};

#[derive(Clone, Debug)]
pub struct CornerSprings {
    pub tl: (Spring, Spring),
    pub tr: (Spring, Spring),
    pub br: (Spring, Spring),
    pub bl: (Spring, Spring),
}

impl CornerSprings {
    pub fn new(params: SpringParams) -> Self {
        Self {
            tl: (Spring::new(0.0, params), Spring::new(0.0, params)),
            tr: (Spring::new(0.0, params), Spring::new(0.0, params)),
            br: (Spring::new(0.0, params), Spring::new(0.0, params)),
            bl: (Spring::new(0.0, params), Spring::new(0.0, params)),
        }
    }
}
