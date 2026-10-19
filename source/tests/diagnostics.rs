use scrollpilot::diagnostics::{Diagnostics, InputReason, Routing, TargetReason};

#[test]
fn wheel_routing_honors_mouse_position_and_fails_closed_for_unknown_values() {
    assert_eq!(Routing::from_raw(Some(0)).select(10, 20), Some(10));
    assert_eq!(Routing::from_raw(Some(1)).select(10, 20), Some(10));
    assert_eq!(Routing::from_raw(Some(2)).select(10, 20), Some(20));
    assert_eq!(Routing::from_raw(Some(2)).select(10, 0), None);
    assert_eq!(Routing::from_raw(Some(3)).select(10, 20), None);
    assert_eq!(Routing::from_raw(None).select(10, 20), None);
    // Hybrid needs both destinations to agree: desktop and Store routing differ.
    assert!(!Routing::Hybrid.safe(10, 10, 20));
    assert!(Routing::MousePosition.safe(20, 10, 20));
    assert!(!Routing::Focus.safe(10, 10, 20));
}

#[test]
fn ready_is_not_delivery_and_accepted_input_is_not_emitted_output() {
    let d = Diagnostics::default();
    d.set_target("reader.exe".into(), TargetReason::Ready);
    let (title, _) = d.status_text(true, true, "", 0);
    assert_eq!(title, "Ready");
    d.record_input(InputReason::Accepted, 100);
    assert_eq!(d.status_text(true, true, "", 101).0, "Input accepted");
    d.record_output(110);
    let (title, detail) = d.status_text(true, true, "", 111);
    assert_eq!(title, "Smoothing");
    assert!(detail.contains("reader.exe"));
    assert!(detail.contains("Output 1"));
    assert_eq!(d.status_text(true, true, "", 2000).0, "Ready");
}

#[test]
fn bypass_reasons_and_tagged_emissions_are_counted_separately() {
    let d = Diagnostics::default();
    d.set_target("reader.exe".into(), TargetReason::Ready);
    d.record_input(InputReason::Accepted, 20);
    d.record_output(21);
    d.record_input(InputReason::Tagged, 22);
    assert_eq!(d.status_text(true, true, "", 23).0, "Smoothing");
    for (reason, text) in [
        (InputReason::Precision, "Precision wheel delta"),
        (InputReason::Modifiers, "Modifier/button held"),
        (InputReason::ExternalInjected, "Externally injected input"),
        (InputReason::TargetChanged, "Target changed"),
        (InputReason::Busy, "Engine busy"),
        (
            InputReason::HorizontalDisabled,
            "Horizontal scrolling disabled",
        ),
    ] {
        d.record_input(reason, 25);
        let (title, detail) = d.status_text(true, true, "", 26);
        assert_eq!(title, "Passing through");
        assert!(detail.contains(text), "{detail}");
    }
    let counts = d.counts();
    assert_eq!(counts, (1, 6, 1));
}

#[test]
fn target_changes_and_disable_clear_active_delivery_claims() {
    let d = Diagnostics::default();
    d.set_target("first.exe".into(), TargetReason::Ready);
    d.record_output(100);
    d.set_target("second.exe".into(), TargetReason::Ready);
    assert_eq!(d.status_text(true, true, "", 101).0, "Ready");
    for (reason, text) in [
        (TargetReason::Excluded, "Excluded app"),
        (TargetReason::HigherIntegrity, "higher integrity"),
        (TargetReason::UnknownProcess, "identity unavailable"),
        (TargetReason::UnknownIntegrity, "integrity unavailable"),
        (TargetReason::NoTarget, "No eligible window"),
        (TargetReason::RoutingUnavailable, "routing unavailable"),
    ] {
        d.set_target("second.exe".into(), reason);
        let (title, detail) = d.status_text(true, true, "", 101);
        assert_eq!(title, "Unavailable");
        assert!(detail.contains(text), "{detail}");
    }
    assert_eq!(d.status_text(false, true, "", 101).0, "Paused");
    assert_eq!(d.status_text(true, false, "", 101).0, "Unavailable");
    assert_eq!(
        d.status_text(false, true, "output failed", 101).0,
        "Paused — needs attention"
    );
}

#[test]
fn unchanged_target_refresh_keeps_recent_output_but_explicit_invalidation_clears_it() {
    let d = Diagnostics::default();
    d.set_target("reader.exe".into(), TargetReason::Ready);
    d.record_output(100);
    d.set_target("reader.exe".into(), TargetReason::Ready);
    assert_eq!(d.status_text(true, true, "", 101).0, "Smoothing");
    d.invalidate();
    assert_eq!(d.status_text(true, true, "", 101).0, "Ready");
}
