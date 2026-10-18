use scrollpilot::branding::{pill, should_animate, CYCLE};

#[test]
fn pill_stretches_but_stays_inside_the_final_letter() {
    let mut tallest = 0f64;
    for i in 0..=4000 {
        let p = pill(i as f64 / 1000., true);
        assert!(p.x >= 439. && p.x + p.width <= 456.00001);
        assert!(p.y >= 6.99999 && p.y + p.height <= 130.00001);
        assert!(p.width > 12. && p.height >= 53.99999);
        tallest = tallest.max(p.height);
    }
    assert!(tallest > 70.);
    assert!(pill(0., true).y < pill(2., true).y);
    assert_eq!(pill(0., true), pill(CYCLE, true));
}
#[test]
fn reduced_motion_uses_the_static_reference_pill() {
    let rest = pill(0., false);
    assert_eq!(rest.height, 54.);
    assert_eq!(rest.y, 41.5);
    assert_eq!(rest, pill(900., false));
    assert_eq!(rest, pill(f64::NAN, true));
    assert!(!should_animate(false, false, false));
    assert!(!should_animate(true, true, false));
    assert!(!should_animate(true, false, true));
    assert!(should_animate(true, false, false));
}
#[test]
fn loop_seams_do_not_jump_visibly() {
    for t in [0.35, 2., 2.35, 4.] {
        let a = pill(t - 0.000001, true);
        let b = pill(t + 0.000001, true);
        assert!((a.y - b.y).abs() < 0.002);
        assert!((a.height - b.height).abs() < 0.002);
    }
}
