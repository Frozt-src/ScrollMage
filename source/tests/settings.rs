use scrollpilot::settings::{load, save, Settings};
fn path(name: &str) -> std::path::PathBuf {
    let p = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("settings-tests");
    std::fs::create_dir_all(&p).unwrap();
    p.join(format!("{}-{name}.json", std::process::id()))
}
#[test]
fn roundtrip_preserves_user_tuning_and_exclusions() {
    let p = path("roundtrip");
    let s = Settings {
        omega: 40.,
        distance: 250.,
        exclusions: vec!["Game Client.exe".into()],
        ..Settings::default()
    };
    save(&p, &s).unwrap();
    let t = load(&p).unwrap();
    assert_eq!(t.omega, 40.);
    assert_eq!(t.distance, 250.);
    assert!(t.excluded("game client.EXE"));
    std::fs::remove_file(p).unwrap();
}
#[test]
fn corrupt_settings_return_error_without_overwriting_bytes() {
    let p = path("corrupt");
    std::fs::write(&p, b"{broken").unwrap();
    assert!(load(&p).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), b"{broken");
    std::fs::remove_file(p).unwrap();
}
#[test]
fn invalid_save_does_not_replace_existing_settings() {
    let p = path("invalid");
    std::fs::write(&p, b"original").unwrap();
    assert!(save(
        &p,
        &Settings {
            distance: -1.,
            ..Settings::default()
        }
    )
    .is_err());
    assert_eq!(std::fs::read(&p).unwrap(), b"original");
    std::fs::remove_file(p).unwrap();
}

#[test]
fn legacy_tuning_loads_without_overwriting_the_original_file() {
    let p = path("legacy");
    let bytes = br#"{"duration_ms":360,"pulse":true,"pulse_scale":3,"distance":180,"acceleration_max":2,"exclusions":["Game.exe"]}"#;
    std::fs::write(&p, bytes).unwrap();
    let s = load(&p).unwrap();
    assert_eq!(s.distance, 180.);
    assert_eq!(s.acceleration_max, 2.);
    assert_eq!(s.omega, 30.);
    assert!(s.excluded("game.exe"));
    assert_eq!(std::fs::read(&p).unwrap(), bytes);
    std::fs::remove_file(p).unwrap();
}

#[test]
fn removed_profiles_migrate_bypass_to_visible_exclusions_without_hidden_tuning() {
    let p = path("removed-profiles");
    let bytes = br#"{"distance":180,"exclusions":["FIREFOX.exe"],"profiles":[{"executable":"firefox.exe","distance":120,"omega":30,"bypass":true},{"executable":"browser.exe","distance":120,"omega":30,"bypass":true},{"executable":"editor.exe","distance":240,"omega":40,"bypass":false}]}"#;
    std::fs::write(&p, bytes).unwrap();
    let settings = load(&p).unwrap();
    assert_eq!(settings.exclusions, vec!["FIREFOX.exe", "browser.exe"]);
    assert!(settings.profiles.is_empty());
    assert_eq!(settings.for_app("editor.exe").distance, 180.);
    assert_eq!(std::fs::read(&p).unwrap(), bytes);
    save(&p, &settings).unwrap();
    let saved = std::fs::read_to_string(&p).unwrap();
    assert!(!saved.contains("profiles"));
    assert!(load(&p).unwrap().excluded("browser.exe"));
    std::fs::remove_file(p).unwrap();
}

#[test]
fn preset_identity_survives_save_without_overwriting_manual_tuning() {
    let p = path("preset-identity");
    let bytes = br#"{"preset":"macos_style","distance":156,"omega":26,"exclusions":["game.exe"],"reverse":true}"#;
    std::fs::write(&p, bytes).unwrap();
    let s = load(&p).unwrap();
    save(&p, &s).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
    assert_eq!(json["preset"], "macos_style");
    assert_eq!(json["distance"], 156.0);
    assert_eq!(json["omega"], 26.0);
    assert_eq!(json["reverse"], true);
    assert_eq!(json["exclusions"][0], "game.exe");
    std::fs::remove_file(p).unwrap();
}

#[test]
fn selecting_presets_changes_motion_and_keeps_user_app_choices() {
    use scrollpilot::{engine::Engine, settings::Preset};
    let mut s = Settings {
        reverse: true,
        horizontal: false,
        exclusions: vec!["work.exe".into()],
        ..Settings::default()
    };
    let mut peaks = Vec::new();
    for preset in [Preset::Default, Preset::MacosStyle, Preset::SuperSmooth] {
        s.apply_preset(preset);
        s.validate().unwrap();
        assert!(s.reverse && !s.horizontal && s.excluded("WORK.exe"));
        let mut engine = Engine::default();
        assert!(engine.push(0., 120, false, 7, &s));
        let outputs: Vec<_> = (0..=250)
            .map(|i| engine.tick(f64::from(i) * 8., 7)[0])
            .collect();
        assert!(
            outputs.iter().all(|v| *v >= 0),
            "No overshoot or reversal during one notch"
        );
        assert_eq!(
            outputs.iter().sum::<i32>(),
            120,
            "Smoother presets must conserve distance"
        );
        assert!(!engine.active());
        peaks.push(*outputs.iter().max().unwrap());
    }
    assert!(
        peaks[2] < peaks[0],
        "Super smooth must spread the motion into smaller output steps"
    );
    assert!(peaks[1] <= peaks[0]);
}
