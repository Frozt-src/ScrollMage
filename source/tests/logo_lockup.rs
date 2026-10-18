use scrollpilot::{
    brand_lockup::Lockup,
    logo_motion::{wheel, CYCLE},
};

#[test]
fn redesigned_wheel_stays_inside_its_new_stem_through_the_whole_loop() {
    let mut tallest = 0.0_f64;
    for i in 0..=8000 {
        let p = wheel(i as f64 / 2000., true);
        assert!(p.x >= 1171.5 && p.x + p.width <= 1199.50001);
        assert!(p.y >= 33.99999 && p.y + p.height <= 258.00001);
        assert!(p.width > 20. && p.height >= 105.99999);
        tallest = tallest.max(p.height);
    }
    assert!(tallest > 135.);
    assert!(wheel(0., true).y < wheel(2., true).y);
    assert_eq!(wheel(0., true), wheel(CYCLE, true));
    assert_eq!(wheel(0., false), wheel(123., false));
    assert_eq!(wheel(0., false), wheel(f64::NAN, true));
    for t in [0.35, 2., 2.35, 4.] {
        let a = wheel(t - 0.000001, true);
        let b = wheel(t + 0.000001, true);
        assert!((a.y - b.y).abs() < 0.004 && (a.height - b.height).abs() < 0.004);
    }
}

#[test]
fn only_the_new_final_letter_changes_in_native_logo_frames() {
    let logo = Lockup::new().unwrap();
    let a = logo.rgba(1320, 440, 0., true, 0).unwrap();
    let b = logo.rgba(1320, 440, 2., true, 0).unwrap();
    let mut changes = 0;
    for (index, (first, second)) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if first != second {
            let x = index % 1320;
            let y = index / 1320;
            assert!(
                (1170..1201).contains(&x) && (33..260).contains(&y),
                "Unexpected animated pixel {x},{y}"
            );
            changes += 1;
        }
    }
    assert!(changes > 3000);
    let alpha = |x: usize, y: usize| a[(y * 1320 + x) * 4 + 3];
    assert_eq!(alpha(1186, 70), 0); // Rebuilt wheel opening.
    assert_eq!(alpha(1186, 185), 255); // The reference's old static opening is not left behind.
    assert_eq!(alpha(0, 0), 0);
}
