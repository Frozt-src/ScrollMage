use scrollpilot::control_motion::Toggle;

#[test]
fn toggles_retarget_without_jumping_and_settle_in_both_directions() {
    let mut toggle = Toggle::default();
    toggle.target(true, true);
    assert!(toggle.step(0.06));
    let midway = toggle.position();
    assert!(midway > 0. && midway < 1. && toggle.stretch() > 0.);
    toggle.target(false, true);
    assert_eq!(toggle.position(), midway);
    for _ in 0..120 {
        toggle.step(1. / 120.);
    }
    assert_eq!(toggle.position(), 0.);
    assert_eq!(toggle.stretch(), 0.);
    toggle.target(true, true);
    for _ in 0..60 {
        toggle.step(1. / 60.);
    }
    assert_eq!(toggle.position(), 1.);
    toggle.target(false, false);
    assert_eq!(toggle.position(), 0.);
    assert_eq!(toggle.stretch(), 0.);
}

#[test]
fn toggle_motion_does_not_depend_on_refresh_rate() {
    let mut slow = Toggle::default();
    let mut fast = slow;
    slow.target(true, true);
    fast.target(true, true);
    for _ in 0..6 {
        slow.step(1. / 30.);
    }
    for _ in 0..24 {
        fast.step(1. / 120.);
    }
    assert!((slow.position() - fast.position()).abs() < 1e-12);
}

#[test]
fn settings_reveal_glides_open_with_minimal_give_and_closes_without_overshoot() {
    use scrollpilot::control_motion::Reveal;
    let (shut, open) = (80, 320);
    let mut r = Reveal::default();
    r.target(true, true);
    let mut peak = 0;
    let mut frames = 0;
    while r.step(1. / 60.) {
        peak = peak.max(r.height(shut, open));
        frames += 1;
        assert!(frames < 60, "Reveal must settle within a second");
    }
    assert_eq!(r.height(shut, open), open);
    assert!(peak <= open + 4, "Opening must not bounce visibly: {peak}");
    r.target(false, true);
    let mut low = open;
    while r.step(1. / 60.) {
        low = low.min(r.height(shut, open));
    }
    assert_eq!(r.height(shut, open), shut);
    assert!(low >= shut, "Closing never cuts into the preset row");
    r.target(true, false);
    assert_eq!(r.height(shut, open), open, "Without animation it snaps");
}
