use scrollpilot::motion::{momentum, rubber_band, GestureAxis, Spring};
use scrollpilot::settings::{format_profiles, parse_profiles, Preset, Settings};

#[test]
fn critical_follower_is_independent_of_frame_partition() {
    let start = Spring {
        position: 2.25,
        target: 120.25,
        velocity: 41.,
    };
    let mut one = start;
    one.step(0.2, 30.);
    for hz in [60., 120., 144., 240.] {
        let mut many = start;
        let frames = (hz * 0.2_f64).floor() as usize;
        for _ in 0..frames {
            many.step(1. / hz, 30.);
        }
        many.step(0.2 - frames as f64 / hz, 30.);
        assert!((one.position - many.position).abs() < 1e-10);
        assert!((one.velocity - many.velocity).abs() < 1e-9);
    }
}
#[test]
fn momentum_matches_closed_form_without_double_decay() {
    let (mut x, mut v) = (0., 1000.);
    momentum(&mut x, &mut v, 0.2, 4.);
    assert!((x - 1000. / 4. * (1. - (-0.8_f64).exp())).abs() < 1e-10);
    assert!((v - 1000. * (-0.8_f64).exp()).abs() < 1e-10);
    let mut os = GestureAxis::new(500., 2400., 200.);
    os.begin();
    os.change(20.25, 0.02);
    os.end(true);
    os.step(0.2, 4., 22.);
    assert_eq!(os.position, 520.25);
    assert!(!os.active());
}
#[test]
fn gesture_begins_directly_and_retouch_interrupts_inertia() {
    let mut g = GestureAxis::new(500., 2400., 200.);
    g.begin();
    g.change(20.25, 0.02);
    assert_eq!(g.rendered(), 520.25);
    g.end(false);
    g.step(0.05, 4., 22.);
    assert!(g.position > 520.25);
    g.begin();
    let stopped = g.position;
    g.step(0.2, 4., 22.);
    assert_eq!(g.position, stopped);
    g.change(-2.5, 0.02);
    assert_eq!(g.position, stopped - 2.5);
    g.cancel();
    g.step(0.2, 4., 22.);
    assert_eq!(g.position, stopped - 2.5);
}
#[test]
fn elastic_edges_are_symmetric_capped_and_return_without_oscillation() {
    for d in [0.1, 1., 10., 100., 1e6] {
        assert_eq!(rubber_band(-d, 200.), -rubber_band(d, 200.));
        assert!(rubber_band(d, 200.) <= 24.);
    }
    let mut g = GestureAxis::new(0., 1000., 200.);
    g.begin();
    g.change(-100., 0.02);
    assert_eq!(g.position, 0.);
    let displayed = g.rendered();
    g.end(false);
    g.step(0.02, 4., 22.);
    assert!(g.rendered() > displayed && g.rendered() < 0.);
    g.begin();
    let before = g.rendered();
    g.change(0., 0.02);
    assert!((before - g.rendered()).abs() < 1e-10);
    g.cancel();
    for _ in 0..120 {
        g.step(1. / 120., 4., 22.);
        assert!(g.rendered() <= 0.);
    }
    assert_eq!(g.rendered(), 0.);
    assert!(!g.active());
}
#[test]
fn profiles_override_only_the_selected_app_and_reject_ambiguity() {
    let profiles = parse_profiles("Editor.exe | 72 | 35\nbrowser.exe | off").unwrap();
    assert_eq!(
        parse_profiles(&format_profiles(&profiles)).unwrap().len(),
        2
    );
    let s = Settings {
        profiles,
        ..Settings::default()
    };
    assert_eq!(s.for_app("EDITOR.EXE").distance, 72.);
    assert_eq!(s.for_app("EDITOR.EXE").omega, 35.);
    assert_eq!(s.for_app("other.exe").omega, 30.);
    assert!(s.excluded("Browser.EXE"));
    for input in [
        "a.exe | 30 | NaN",
        "a.exe | 1000 | 30",
        "a.exe | 120 | 30\nA.exe | off",
        "C:\\a.exe | off",
        "C:a.exe | off",
        "*.exe | off",
        "a.exe |",
        ".exe | off",
    ] {
        assert!(parse_profiles(input).is_err(), "{input}");
    }
}

#[test]
fn fixed_profiles_roundtrip_with_preset_identity() {
    let profiles = parse_profiles("Editor.exe | macos_style\nbrowser.exe | super_smooth").unwrap();
    assert_eq!(profiles[0].preset, Some(Preset::MacosStyle));
    assert_eq!(profiles[1].preset, Some(Preset::SuperSmooth));
    let reparsed = parse_profiles(&format_profiles(&profiles)).unwrap();
    assert_eq!(reparsed[0].preset, Some(Preset::MacosStyle));
    assert_eq!(reparsed[1].preset, Some(Preset::SuperSmooth));
    let s = Settings {
        acceleration_ms: 300.,
        acceleration_max: 7.,
        profiles,
        ..Settings::default()
    };
    assert_eq!(
        s.for_app("editor.exe").acceleration_max,
        Preset::MacosStyle.tuning().acceleration_max
    );
    assert_eq!(
        s.for_app("browser.exe").acceleration_max,
        Preset::SuperSmooth.tuning().acceleration_max
    );
}
