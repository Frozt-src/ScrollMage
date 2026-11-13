use scrollpilot::{engine::Engine, settings::Settings};

// Default response 30 gives a friction of 15 per second: a notch's remaining
// distance falls to e^(-15 t) of what it was, and the rest has been sent.

#[test]
fn wheel_notch_drains_exponentially_from_full_speed() {
    let s = Settings {
        distance: 120.,
        acceleration_max: 1.,
        ..Settings::default()
    };
    let mut engine = Engine::default();
    assert!(engine.push(0., 120, false, 1, &s));
    let expected = (120. * (1. - (-1.5f64).exp())).trunc() as i32;
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
fn a_second_notch_adds_to_the_motion_already_under_way() {
    let s = Settings {
        acceleration_max: 1.,
        ..Settings::default()
    };
    let mut engine = Engine::default();
    engine.push(0., 120, false, 1, &s);
    engine.push(50., 120, false, 1, &s);
    // Each notch drains independently from the moment it arrived.
    let expected = (240. - 120. * (-1.5f64).exp() - 120. * (-0.75f64).exp()).trunc() as i32;
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
