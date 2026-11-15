use scrollpilot::{engine::Engine, settings::Settings};
fn unaccelerated() -> Settings {
    Settings {
        acceleration_max: 1.,
        preset: scrollpilot::settings::Preset::Custom,
        ..Settings::default()
    }
}
#[test]
fn full_impulse_conserves_wheel_units() {
    let mut e = Engine::default();
    assert!(e.push(0., 120, false, 1, &unaccelerated()));
    let sum: i32 = (0..=125).map(|i| e.tick(i as f64 * 8., 1)[0]).sum();
    assert_eq!(sum, 120);
    assert!(!e.active());
}
#[test]
fn negative_and_horizontal_distances_are_conserved() {
    let mut e = Engine::default();
    assert!(e.push(0., -120, true, 1, &unaccelerated()));
    let sums = (0..=125).fold([0, 0], |a, i| {
        let b = e.tick(i as f64 * 8., 1);
        [a[0] + b[0], a[1] + b[1]]
    });
    assert_eq!(sums, [0, -120]);
}
#[test]
fn switching_target_discards_old_momentum() {
    let mut e = Engine::default();
    assert!(e.push(0., 120, false, 1, &unaccelerated()));
    assert!(e.active());
    assert_eq!(e.tick(30., 2), [0, 0]);
    assert!(!e.active());
}
#[test]
fn reversal_stops_pending_direction() {
    let mut e = Engine::default();
    e.push(0., 120, false, 1, &unaccelerated());
    e.push(10., -120, false, 1, &unaccelerated());
    assert_eq!(e.tick(1000., 1)[0], -120);
}
#[test]
fn acceleration_caps_even_at_same_timestamp() {
    let mut e = Engine::default();
    let s = Settings::default();
    e.push(0., 120, false, 1, &s);
    e.push(0., 120, false, 1, &s);
    assert_eq!(e.tick(1000., 1)[0], 240);
}
#[test]
fn independent_axes_do_not_cancel_one_another() {
    let mut e = Engine::default();
    e.push(0., 120, false, 1, &unaccelerated());
    e.push(10., 120, true, 1, &unaccelerated());
    assert_eq!(e.tick(1000., 1), [120, 120]);
}
#[test]
fn fractional_outputs_accumulate_across_frames() {
    let mut e = Engine::default();
    let s = Settings {
        distance: 121.,
        ..unaccelerated()
    };
    e.push(0., 120, false, 1, &s);
    let sum: i32 = (0..=1000).map(|i| e.tick(i as f64, 1)[0]).sum();
    assert_eq!(sum, 121);
}
#[test]
fn invalid_settings_are_rejected() {
    for duration in [f64::NAN, 0., 1e9] {
        assert!(Settings {
            omega: duration,
            preset: scrollpilot::settings::Preset::Custom,
            ..Settings::default()
        }
        .validate()
        .is_err());
    }
    assert!(Settings {
        distance: -1.,
        preset: scrollpilot::settings::Preset::Custom,
        ..Settings::default()
    }
    .validate()
    .is_err());
}
#[test]
fn exclusions_ignore_case_and_preserve_spaces() {
    assert!(Settings::default().excluded("diablo iv.EXE"));
    assert!(!Settings::default().excluded("editor.exe"));
}
