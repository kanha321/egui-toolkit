use super::params::SpringParams;

pub fn step(
    current: f32,
    velocity: f32,
    target: f32,
    _params: &SpringParams,
    dt: f32,
) -> (f32, f32) {
    if dt <= 0.0 {
        return (current, velocity);
    }
    (target, 0.0)
}
