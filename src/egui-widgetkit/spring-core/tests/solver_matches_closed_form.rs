use spring_core::{solve_step, SpringParams};

#[test]
fn test_underdamped_closed_form_exact_values() {
    // x0 = 10.0, v0 = 0.0, target = 0.0, w0 = 10.0, zeta = 0.5
    // gamma = 10 * sqrt(0.75) = 8.660254
    // at t = 0.1s:
    // decay = exp(-5 * 0.1) = exp(-0.5) = 0.60653066
    // gamma * dt = 0.8660254
    // cos(0.8660254) = 0.6476382, sin(0.8660254) = 0.761947
    // term = (10 * cos + (50 / gamma) * sin) * decay = 6.597002
    let params = SpringParams::new(10.0, 0.5);
    let state = solve_step(10.0, 0.0, 0.0, 0.1, params);

    assert!((state.position - 6.597002).abs() < 1e-4, "Underdamped position mismatch: {}", state.position);
}

#[test]
fn test_critically_damped_closed_form() {
    // x0 = 10.0, v0 = 0.0, target = 0.0, w0 = 10.0, zeta = 1.0
    // at t = 0.1s:
    // decay = exp(-10 * 0.1) = exp(-1.0) = 0.36787944
    // b = 0 + 10 * 10 = 100
    // x(0.1) = (10 + 100 * 0.1) * exp(-1.0) = 20 * 0.36787944 = 7.3575888
    let params = SpringParams::new(10.0, 1.0);
    let state = solve_step(10.0, 0.0, 0.0, 0.1, params);

    assert!((state.position - 7.3575888).abs() < 1e-4, "Critically damped position mismatch: {}", state.position);
}

#[test]
fn test_overdamped_closed_form() {
    // x0 = 10.0, v0 = 0.0, target = 0.0, w0 = 10.0, zeta = 2.0
    let params = SpringParams::new(10.0, 2.0);
    let state = solve_step(10.0, 0.0, 0.0, 0.05, params);

    assert!(state.position > 0.0 && state.position < 10.0);
    assert!(state.velocity < 0.0, "Velocity must be directed toward target");
}

#[test]
fn test_framerate_independence_step_invariance() {
    // An analytical closed-form solver must produce the same result whether stepped
    // in one 100ms step or 100 steps of 1ms!
    let params = SpringParams::bouncy();
    let initial_pos = 0.0;
    let initial_vel = 50.0;
    let target = 100.0;

    // 1 single large step of 0.1s
    let single_step = solve_step(initial_pos, initial_vel, target, 0.1, params);

    // 100 small steps of 0.001s
    let mut pos = initial_pos;
    let mut vel = initial_vel;
    for _ in 0..100 {
        let next = solve_step(pos, vel, target, 0.001, params);
        pos = next.position;
        vel = next.velocity;
    }

    assert!((single_step.position - pos).abs() < 1e-3, "Position must match across dt discretizations: single={}, multi={}", single_step.position, pos);
    assert!((single_step.velocity - vel).abs() < 1e-2, "Velocity must match across dt discretizations: single={}, multi={}", single_step.velocity, vel);
}

#[test]
fn test_critical_damping_epsilon_band_continuity() {
    let target = 100.0;
    let dt = 0.05;

    // Slightly below critical
    let p_under = SpringParams::new(20.0, 0.99995);
    let s_under = solve_step(0.0, 0.0, target, dt, p_under);

    // Exactly critical
    let p_crit = SpringParams::new(20.0, 1.0);
    let s_crit = solve_step(0.0, 0.0, target, dt, p_crit);

    // Slightly above critical
    let p_over = SpringParams::new(20.0, 1.00005);
    let s_over = solve_step(0.0, 0.0, target, dt, p_over);

    // The transitions must be seamless with no discontinuity or NaN
    assert!(!s_under.position.is_nan());
    assert!(!s_crit.position.is_nan());
    assert!(!s_over.position.is_nan());

    assert!((s_under.position - s_crit.position).abs() < 1e-3);
    assert!((s_over.position - s_crit.position).abs() < 1e-3);
}
