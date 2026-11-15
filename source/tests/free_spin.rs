use scrollpilot::{
    engine::Engine,
    settings::{Preset, Profile, Settings},
};

fn super_smooth() -> Settings {
    let mut settings = Settings::default();
    settings.select_preset(Preset::SuperSmooth);
    settings
}

fn flick(settings: &Settings) -> Engine {
    let mut engine = Engine::default();
    for time in [0., 20., 40.] {
        assert!(engine.push(time, 120, false, 1, settings));
    }
    engine
}

#[test]
fn quick_flick_coasts_fast_after_input_then_stops() {
    let mut engine = flick(&super_smooth());
    let mut total = 0;
    let mut late = 0;
    let mut first_second = 0;
    let mut last_output_ms = 0.;
    for frame in 5..=875 {
        let time = f64::from(frame) * 8.;
        let amount = engine.tick(time, 1)[0];
        assert!(amount >= 0, "A flick must never reverse on its own");
        total += amount;
        if amount != 0 {
            last_output_ms = time;
        }
        if time <= 1040. {
            first_second += amount;
        }
        if (1500. ..2500.).contains(&time) {
            late += amount;
        }
        if time == 2040. {
            assert!(
                engine.active(),
                "A flick must still coast after two seconds"
            );
        }
    }
    assert!(
        first_second > 2400,
        "Flick must traverse quickly, got {first_second} units"
    );
    assert!(
        late > 120,
        "Flick must keep delivering useful momentum, got {late} units"
    );
    assert!(
        (4000..20000).contains(&total),
        "Flick distance must be substantial but bounded: {total}"
    );
    assert!(!engine.active(), "Momentum must eventually sleep");
    assert_eq!(engine.tick(8000., 1), [0, 0]);
    eprintln!("Free-spin trace: first-second={first_second}, later={late}, total={total} wheel units; final output at {last_output_ms}ms");
}

#[test]
fn slow_notches_stay_precise_and_do_not_launch_a_coast() {
    let mut engine = Engine::default();
    let settings = super_smooth();
    let mut total = 0;
    for time in (0..=2400).step_by(8) {
        if [0, 400, 800].contains(&time) {
            assert!(engine.push(f64::from(time), 120, false, 1, &settings));
        }
        total += engine.tick(f64::from(time), 1)[0];
    }
    assert_eq!(total, 360);
    assert!(!engine.active());
}

#[test]
fn one_grouped_flick_packet_can_launch_but_one_notch_cannot() {
    let settings = super_smooth();
    let mut grouped = Engine::default();
    assert!(grouped.push(0., 240, false, 1, &settings));
    let mut late = 0;
    for time in (0..=2500).step_by(8) {
        let out = grouped.tick(f64::from(time), 1)[0];
        if time >= 1500 {
            late += out;
        }
    }
    assert!(late > 120, "A packet containing a flick must coast");
    let mut single = Engine::default();
    single.push(0., 120, false, 1, &settings);
    assert_eq!(
        (0..=2000)
            .step_by(8)
            .map(|t| single.tick(f64::from(t), 1)[0])
            .sum::<i32>(),
        120
    );
    assert!(!single.active());
}

#[test]
fn reversing_brakes_the_coast_before_scrolling_back() {
    let settings = super_smooth();
    let mut engine = flick(&settings);
    for time in (40..=496).step_by(8) {
        engine.tick(f64::from(time), 1);
    }
    assert!(engine.push(504., -120, false, 1, &settings));
    let mut total = 0;
    for time in (504..=2504).step_by(8) {
        let out = engine.tick(f64::from(time), 1)[0];
        assert!(
            out <= 0,
            "Old forward momentum must be discarded immediately"
        );
        total += out;
    }
    assert_eq!(total, -120);
    assert!(!engine.active());
}

#[test]
fn target_change_clear_and_long_stall_cancel_the_coast() {
    let settings = super_smooth();
    let mut engine = flick(&settings);
    assert_eq!(engine.tick(48., 2), [0, 0]);
    assert!(!engine.active());
    let mut engine = flick(&settings);
    engine.clear();
    assert_eq!(engine.tick(48., 1), [0, 0]);
    assert!(!engine.active());
    let mut engine = flick(&settings);
    engine.tick(40., 1);
    let catch_up = engine.tick(1040., 1)[0];
    assert!(
        catch_up <= 360,
        "A stalled worker must not dump a second of coasting: {catch_up}"
    );
    assert!(
        engine.tick(2040., 1)[0] <= 1,
        "Only a final fractional follower unit may remain"
    );
    assert!(!engine.active());
}

#[test]
fn coasting_is_consistent_across_frame_rates() {
    let settings = super_smooth();
    let mut totals = Vec::new();
    for hz in [60., 120., 144., 240.] {
        let mut engine = flick(&settings);
        let mut total = engine.tick(40., 1)[0];
        for frame in 1..=(hz * 7.) as i32 {
            total += engine.tick(40. + f64::from(frame) * 1000. / hz, 1)[0];
        }
        assert!(!engine.active());
        totals.push(total);
    }
    assert!(
        totals[0] > 4000,
        "Coasting must actually be present: {totals:?}"
    );
    assert!(
        totals.iter().all(|t| (t - totals[0]).abs() <= 1),
        "Frame rate must not change flick distance: {totals:?}"
    );
}

#[test]
fn repeated_flicks_are_bounded_and_axes_are_independent() {
    let settings = super_smooth();
    let mut engine = Engine::default();
    let mut total = 0;
    for time in (0..=15000).step_by(8) {
        if time < 8000 && time % 16 == 0 {
            assert!(engine.push(f64::from(time), 120, false, 1, &settings));
        }
        if time == 800 {
            assert!(engine.push(800., -120, true, 1, &settings));
        }
        let out = engine.tick(f64::from(time), 1);
        assert!((0..=512).contains(&out[0]), "Runaway frame: {}", out[0]);
        assert!(out[1] <= 0);
        total += out[0];
    }
    assert!(
        (60000..250000).contains(&total),
        "Bounded sustained traversal: {total}"
    );
    assert!(!engine.active());
}

#[test]
fn free_spin_belongs_only_to_the_selected_super_smooth_profile() {
    for preset in [Preset::Default, Preset::MacosStyle, Preset::Custom] {
        let mut settings = Settings::default();
        settings.select_preset(preset);
        let mut engine = flick(&settings);
        for time in (40..=2400).step_by(8) {
            let out = engine.tick(f64::from(time), 1);
            if time >= 1500 {
                assert_eq!(out, [0, 0]);
            }
        }
        assert!(!engine.active());
    }
    let mut settings = super_smooth();
    settings.profiles.push(Profile {
        executable: "editor.exe".into(),
        preset: None,
        distance: 72.,
        omega: 35.,
        bypass: false,
    });
    let mut custom = Engine::default();
    for time in [0., 20., 40.] {
        custom.push_tuned(time, 120, false, 1, settings.for_app("editor.exe"));
    }
    let total = (40..=2400)
        .step_by(8)
        .map(|t| custom.tick(f64::from(t), 1)[0])
        .sum::<i32>();
    assert_eq!(
        total, 216,
        "Custom per-app tuning must not silently inherit a fling"
    );
    settings.profiles[0].preset = Some(Preset::SuperSmooth);
    let mut fixed = Engine::default();
    for time in [0., 20., 40.] {
        fixed.push_tuned(time, 120, false, 1, settings.for_app("editor.exe"));
    }
    let mut late = 0;
    for time in (40..=2400).step_by(8) {
        let out = fixed.tick(f64::from(time), 1)[0];
        if time >= 1500 {
            late += out;
        }
    }
    assert!(
        late > 120,
        "Fixed app Super smooth must inherit free-spin behavior"
    );
}
