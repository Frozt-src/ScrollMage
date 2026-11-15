use scrollpilot::branding::should_animate;

#[test]
fn logo_animates_only_when_visible_restored_and_motion_is_allowed() {
    assert!(!should_animate(false, false, false));
    assert!(!should_animate(true, true, false));
    assert!(!should_animate(true, false, true));
    assert!(should_animate(true, false, false));
}
