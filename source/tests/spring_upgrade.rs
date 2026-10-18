use scrollpilot::{engine::Engine, settings::Settings};

#[test]
fn wheel_follows_exact_critical_spring_instead_of_restarting_a_pulse() {
    let s = Settings {
        distance: 120.,
        acceleration_max: 1.,
        ..Settings::default()
    };
    let mut engine = Engine::default();
    assert!(engine.push(0., 120, false, 1, &s));
    let expected = (120. * (1. - 4. * (-3f64).exp())).trunc() as i32;
    assert_eq!(engine.tick(100., 1)[0], expected);
}

#[test]
fn fractional_travel_is_preserved_between_separate_notches() {
    let s = Settings {
        distance: 120.25,
        acceleration_max: 1.,
        ..Settings::default()
    };
    let mut engine = Engine::default();
    let mut total = 0;
    for i in 0..4 {
        let time = i as f64 * 1000.;
        assert!(engine.push(time, 120, false, 1, &s));
        total += engine.tick(time + 900., 1)[0];
    }
    assert_eq!(total, 481);
}

#[test]
fn retargeting_preserves_the_existing_velocity() {
    let s = Settings {
        acceleration_max: 1.,
        ..Settings::default()
    };
    let mut engine = Engine::default();
    engine.push(0., 120, false, 1, &s);
    engine.push(50., 120, false, 1, &s);
    let expected =
        (120. * (1. - 4. * (-3f64).exp()) + 120. * (1. - 2.5 * (-1.5f64).exp())).trunc() as i32;
    assert_eq!(engine.tick(100., 1)[0], expected);
}

#[test]
fn pending_debt_is_bounded_and_invalid_time_does_not_poison_state() {
    let s = Settings::default();
    let mut engine = Engine::default();
    assert!(!engine.push(f64::NAN, 120, false, 1, &s));
    for _ in 0..136 {
        assert!(engine.push(0., 120, false, 1, &s));
    }
    assert!(!engine.push(0., 120, false, 1, &s));
    assert_eq!(engine.tick(1000., 1)[0], 16320);
    assert!(!engine.active());
    assert!(!engine.push(999., 120, false, 1, &s));
}
