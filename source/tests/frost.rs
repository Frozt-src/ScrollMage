use scrollpilot::frost::blur_tint;

#[test]
fn background_is_blurred_before_tint_instead_of_merely_transparent() {
    let mut pixels: Vec<u32> = (0..64 * 32)
        .map(|i| {
            if (i % 64 + i / 64) % 2 == 0 {
                0
            } else {
                0xffffff
            }
        })
        .collect();
    blur_tint(&mut pixels, 64, 32, 4, 0xfaf7fb);
    let a = pixels[16 * 64 + 30];
    let b = pixels[16 * 64 + 31];
    for shift in [0, 8, 16] {
        assert!(
            (((a >> shift) & 255) as i32 - ((b >> shift) & 255) as i32).abs() <= 1,
            "High frequency details must be blurred, not visible through a flat tint"
        );
    }
    assert!(pixels.iter().all(|p| p >> 24 == 255));
}

#[test]
fn uniform_background_retains_fifteen_percent_under_the_tint() {
    let mut dark = vec![0; 8 * 8];
    let mut light = vec![0xffffff; 8 * 8];
    blur_tint(&mut dark, 8, 8, 3, 0xffffff);
    blur_tint(&mut light, 8, 8, 3, 0xffffff);
    assert_eq!(dark[0] & 255, 217);
    assert_eq!(light[0] & 255, 255);
    let mut single = vec![0x123456];
    blur_tint(&mut single, 1, 1, 8, 0xffffff);
    assert_eq!(single.len(), 1);
    assert_eq!(single[0] >> 24, 255);
}
