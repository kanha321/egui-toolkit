use super::params::SpringParams;
use super::solver::step;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    pub params: SpringParams,
}

impl Spring {
    pub fn new(initial: f32, params: SpringParams) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            params,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let (val, vel) = step(self.value, self.velocity, self.target, &self.params, dt);
        self.value = val;
        self.velocity = vel;
    }

    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() < 1e-3 && self.velocity.abs() < 1e-2
    }
}
