use spring_core::{Spring, SpringParams};

#[test]
fn test_spring_settles_and_clamps_cleanly() {
    let mut spring = Spring::new(0.0, SpringParams::snappy());
    spring.set_target(150.0);

    let dt = 1.0 / 60.0;
    let mut steps = 0;
    let max_steps = 600; // 10 seconds timeout

    while !spring.is_settled() && steps < max_steps {
        spring.update(dt);
        steps += 1;
    }

    assert!(spring.is_settled(), "Spring must settle within 10 seconds");
    assert_eq!(spring.value(), 150.0, "Position must snap exactly to target upon settling");
    assert_eq!(spring.velocity(), 0.0, "Velocity must snap to 0.0 upon settling");
}

#[test]
fn test_passing_through_target_at_speed_does_not_falsely_settle() {
    // Underdamped bouncy spring will oscillate across target multiple times
    let mut spring = Spring::new(0.0, SpringParams::bouncy());
    spring.set_target(100.0);

    let mut saw_target_crossing_with_velocity = false;
    let dt = 0.005;

    for _ in 0..200 {
        let prev_pos = spring.value();
        spring.update(dt);
        let curr_pos = spring.value();

        // Check if crossed target position (100.0) while still moving
        if (prev_pos < 100.0 && curr_pos >= 100.0) || (prev_pos > 100.0 && curr_pos <= 100.0) {
            if spring.velocity().abs() > 5.0 {
                saw_target_crossing_with_velocity = true;
                // is_settled() MUST be false despite being right around target!
                assert!(!spring.is_settled(), "Spring must not report settled while moving through target at speed!");
            }
        }
    }

    assert!(saw_target_crossing_with_velocity, "Bouncy spring must cross target during overshoot");
}

#[test]
fn test_all_presets_settle_consistently() {
    let presets = [
        ("gentle", SpringParams::gentle()),
        ("snappy", SpringParams::snappy()),
        ("bouncy", SpringParams::bouncy()),
    ];

    for (name, params) in presets {
        let mut spring = Spring::new(0.0, params);
        spring.set_target(250.0);

        let dt = 1.0 / 60.0;
        let mut steps = 0;
        while !spring.is_settled() && steps < 1200 {
            spring.update(dt);
            steps += 1;
        }

        assert!(spring.is_settled(), "Preset '{}' must settle cleanly within time limit", name);
        assert_eq!(spring.value(), 250.0, "Preset '{}' position must match target", name);
        assert_eq!(spring.velocity(), 0.0, "Preset '{}' velocity must be 0", name);
    }
}
