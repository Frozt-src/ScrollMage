use scrollpilot::diagnostics::Routing;

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
