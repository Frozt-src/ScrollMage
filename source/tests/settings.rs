use scrollpilot::settings::{load, save, Preset, Profile, Settings};
fn path(name: &str) -> std::path::PathBuf {
    let p = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("settings-tests");
    std::fs::create_dir_all(&p).unwrap();
    p.join(format!("{}-{name}.json", std::process::id()))
}

#[test]
fn concurrent_saves_leave_one_complete_reloadable_file() {
    let p = path("concurrent");
    let writers: Vec<_> = (0..16)
        .map(|index| {
            let p = p.clone();
            std::thread::spawn(move || {
                let result = save(
                    &p,
                    &Settings {
                        distance: 100. + index as f64,
                        exclusions: vec![format!("writer{index}.exe")],
                        ..Settings::default()
                    },
                );
                (index, result.is_ok())
            })
        })
        .collect();
    // Windows may reject a simultaneous replacement with access denied. Such
    // failures must be reported; they must not damage a successful writer's file.
    let successful: Vec<_> = writers
        .into_iter()
        .map(|writer| writer.join().unwrap())
        .filter_map(|(index, ok)| ok.then_some(index))
        .collect();
    assert!(!successful.is_empty());
    let saved = load(&p).unwrap();
    let index = (saved.distance - 100.) as usize;
    assert!(successful.contains(&index));
    assert_eq!(saved.exclusions, vec![format!("writer{index}.exe")]);
    std::fs::remove_file(p).unwrap();
}

#[test]
#[cfg(windows)]
fn locked_destination_preserves_settings_and_recovers_after_release() {
    use std::os::windows::fs::OpenOptionsExt;
    let p = path("locked");
    save(&p, &Settings::default()).unwrap();
    let original = std::fs::read(&p).unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&p)
        .unwrap();
    let updated = Settings {
        distance: 240.,
        ..Settings::default()
    };
    assert!(save(&p, &updated).is_err());
    drop(lock);
    assert_eq!(std::fs::read(&p).unwrap(), original);
    save(&p, &updated).unwrap();
    assert_eq!(load(&p).unwrap().distance, 240.);
    std::fs::remove_file(p).unwrap();
}

#[test]
fn oversized_save_preserves_reloadable_settings() {
    let p = path("oversized-save");
    save(&p, &Settings::default()).unwrap();
    let original = std::fs::read(&p).unwrap();
    let names: Vec<_> = (0..128)
        .map(|i| format!("{i:03}{}.exe", "x".repeat(240)))
        .collect();
    let s = Settings {
        exclusions: names.clone(),
        incompatible_processes: names.clone(),
        profiles: names
            .into_iter()
            .map(|executable| Profile {
                executable,
                preset: None,
                distance: 120.,
                omega: 30.,
                bypass: false,
            })
            .collect(),
        ..Settings::default()
    };
    s.validate().unwrap();
    assert!(serde_json::to_vec_pretty(&s).unwrap().len() > 65536);
    assert!(
        save(&p, &s).is_err(),
        "Save must not write a file the loader will refuse"
    );
    assert_eq!(std::fs::read(&p).unwrap(), original);
    assert!(load(&p).is_ok());
    std::fs::remove_file(p).unwrap();
}

#[test]
fn failed_save_removes_its_temporary_file() {
    let p = path("blocked-replace");
    std::fs::create_dir_all(&p).unwrap();
    let stem = p.file_stem().unwrap().to_string_lossy().into_owned();
    let temps = || {
        std::fs::read_dir(p.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                name.starts_with(&stem) && name.ends_with(".tmp")
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>()
    };
    let before = temps();
    assert!(save(&p, &Settings::default()).is_err());
    let after = temps();
    std::fs::remove_dir(&p).unwrap();
    for temp in after.iter().filter(|temp| !before.contains(temp)) {
        std::fs::remove_file(temp).unwrap();
    }
    assert_eq!(
        after, before,
        "A failed save must clean up only its own temporary file"
    );
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
fn legacy_profiles_survive_loading_and_saving_without_losing_app_tuning() {
    let p = path("removed-profiles");
    let bytes = br#"{"distance":180,"exclusions":["FIREFOX.exe"],"profiles":[{"executable":"firefox.exe","distance":120,"omega":30,"bypass":true},{"executable":"browser.exe","distance":120,"omega":30,"bypass":true},{"executable":"editor.exe","distance":240,"omega":40,"bypass":false}]}"#;
    std::fs::write(&p, bytes).unwrap();
    let settings = load(&p).unwrap();
    assert_eq!(settings.exclusions, vec!["FIREFOX.exe"]);
    assert_eq!(settings.profiles.len(), 3);
    assert_eq!(settings.for_app("editor.exe").distance, 240.);
    assert_eq!(settings.for_app("editor.exe").omega, 40.);
    assert_eq!(std::fs::read(&p).unwrap(), bytes);
    save(&p, &settings).unwrap();
    let saved = std::fs::read_to_string(&p).unwrap();
    assert!(saved.contains("profiles"));
    assert_eq!(load(&p).unwrap().profiles.len(), 3);
    assert!(load(&p).unwrap().excluded("browser.exe"));
    std::fs::remove_file(p).unwrap();
}

#[test]
fn legacy_fixed_preset_is_canonical_and_preserves_manual_tuning() {
    let p = path("preset-identity");
    let bytes = br#"{"preset":"macos_style","distance":156,"omega":26,"exclusions":["game.exe"],"reverse":true}"#;
    std::fs::write(&p, bytes).unwrap();
    let s = load(&p).unwrap();
    save(&p, &s).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
    assert_eq!(json["preset"], "macos_style");
    assert_eq!(json["distance"], 120.0);
    assert_eq!(json["omega"], Preset::MacosStyle.tuning().omega);
    assert_eq!(json["custom_tuning"]["distance"], 156.0);
    assert_eq!(json["custom_tuning"]["omega"], 26.0);
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
        assert!(preset.fixed());
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

#[test]
fn custom_tuning_returns_after_fixed_presets_and_restart() {
    let p = path("custom-memory");
    let mut s = Settings {
        preset: Preset::Custom,
        distance: 240.,
        omega: 42.,
        acceleration_ms: 150.,
        acceleration_max: 2.5,
        friction: 6.,
        elastic_omega: 28.,
        ..Settings::default()
    };
    let edited = s.tuning();
    s.select_preset(Preset::MacosStyle);
    assert_eq!(s.custom_tuning, Some(edited));
    assert!(s.preset.fixed());
    s.select_preset(Preset::Default);
    assert_eq!(s.tuning(), Preset::Default.tuning(), "Default is fixed");
    s.select_preset(Preset::SuperSmooth);
    assert_eq!(s.custom_tuning, Some(edited));
    save(&p, &s).unwrap();
    let mut restored = load(&p).unwrap();
    restored.select_preset(Preset::Custom);
    assert_eq!(restored.tuning(), edited);
    assert!(!restored.preset.fixed());
    restored.distance = 360.;
    restored.select_preset(Preset::Custom);
    assert_eq!(
        restored.distance, 360.,
        "reselecting Custom keeps current edits"
    );
    restored.select_preset(Preset::MacosStyle);
    restored.select_preset(Preset::Custom);
    assert_eq!(restored.distance, 360.);
    std::fs::remove_file(p).unwrap();
}

#[test]
fn saving_fixed_tuning_normalizes_copy_and_preserves_custom() {
    let p = path("fixed-save");
    let mut s = Settings {
        preset: Preset::Custom,
        distance: 220.,
        ..Settings::default()
    };
    let edited = s.tuning();
    s.select_preset(Preset::SuperSmooth);
    let canonical = s.tuning();
    s.distance = 500.;
    save(&p, &s).unwrap();
    assert_eq!(s.distance, 500., "save must not mutate caller");
    let mut loaded = load(&p).unwrap();
    assert_eq!(loaded.tuning(), canonical);
    assert_eq!(loaded.custom_tuning, Some(edited));
    loaded.select_preset(Preset::Custom);
    assert_eq!(loaded.tuning(), edited);
    std::fs::remove_file(p).unwrap();
}

#[test]
fn fixed_profiles_use_full_canonical_wheel_tuning_and_roundtrip() {
    let p = path("fixed-profile");
    let mut s = Settings {
        acceleration_ms: 350.,
        acceleration_max: 8.,
        profiles: vec![Profile {
            executable: "editor.exe".into(),
            distance: 222.,
            omega: 35.,
            preset: Some(Preset::MacosStyle),
            bypass: false,
        }],
        ..Settings::default()
    };
    let mut canonical = Settings::default();
    canonical.select_preset(Preset::MacosStyle);
    assert_eq!(s.for_app("EDITOR.EXE"), canonical.wheel());
    save(&p, &s).unwrap();
    let loaded = load(&p).unwrap();
    assert_eq!(loaded.profiles[0].preset, Some(Preset::MacosStyle));
    assert_eq!(loaded.profiles[0].distance, 120.);
    assert_eq!(loaded.profiles[0].omega, Preset::MacosStyle.tuning().omega);
    assert_eq!(loaded.for_app("editor.exe"), canonical.wheel());
    s.profiles[0].bypass = true;
    assert_eq!(s.for_app("editor.exe"), s.wheel());
    std::fs::remove_file(p).unwrap();
}

#[test]
fn malformed_profiles_and_saved_default_are_rejected_without_replacement() {
    let p = path("bad-profile");
    for profiles in [
        vec![Profile {
            executable: "C:\\game.exe".into(),
            distance: 120.,
            omega: 30.,
            preset: None,
            bypass: false,
        }],
        vec![Profile {
            executable: "a.exe".into(),
            distance: f64::NAN,
            omega: 30.,
            preset: None,
            bypass: false,
        }],
        vec![Profile {
            executable: "a.exe".into(),
            distance: 120.,
            omega: f64::INFINITY,
            preset: Some(Preset::MacosStyle),
            bypass: false,
        }],
        vec![
            Profile {
                executable: "a.exe".into(),
                distance: 120.,
                omega: 30.,
                preset: None,
                bypass: false,
            },
            Profile {
                executable: "A.EXE".into(),
                distance: 120.,
                omega: 30.,
                preset: None,
                bypass: true,
            },
        ],
    ] {
        let s = Settings {
            profiles,
            ..Settings::default()
        };
        std::fs::write(&p, b"original").unwrap();
        assert!(save(&p, &s).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"original");
    }
    let mut s = Settings::default();
    let mut bad = s.tuning();
    bad.acceleration_max = f64::NAN;
    s.custom_tuning = Some(bad);
    assert!(s.validate().is_err());
    std::fs::remove_file(p).unwrap();
}

#[test]
fn custom_starts_from_the_active_preset_the_first_time() {
    let mut s = Settings::default();
    s.select_preset(Preset::MacosStyle);
    s.select_preset(Preset::Custom);
    assert_eq!(s.preset, Preset::Custom);
    assert_eq!(s.tuning(), Preset::MacosStyle.tuning());
    assert!(!s.wheel().free_spin, "Custom never adds free-spin glide");
}

#[test]
fn legacy_edited_default_loads_as_custom_and_old_key_is_read() {
    let p = path("legacy-default");
    std::fs::write(
        &p,
        br#"{"preset":"default","distance":200,"omega":40,"exclusions":[]}"#,
    )
    .unwrap();
    let s = load(&p).unwrap();
    assert_eq!(s.preset, Preset::Custom);
    assert_eq!((s.distance, s.omega), (200., 40.));
    std::fs::write(
        &p,
        br#"{"preset":"super_smooth","default_tuning":{"distance":180,"omega":20,"acceleration_ms":70,"acceleration_max":1,"friction":4,"elastic_omega":22}}"#,
    )
    .unwrap();
    let mut s = load(&p).unwrap();
    assert_eq!(s.custom_tuning.map(|t| t.distance), Some(180.));
    s.select_preset(Preset::Custom);
    assert_eq!(s.distance, 180.);
    std::fs::write(&p, br#"{"preset":"default","exclusions":[]}"#).unwrap();
    assert_eq!(load(&p).unwrap().preset, Preset::Default);
    std::fs::remove_file(p).unwrap();
}
