/// Blur the app-owned background, then mix in an opaque tint.
pub fn blur_tint(pixels: &mut [u32], width: usize, height: usize, radius: usize, tint: u32) {
    if width == 0 || height == 0 || width.checked_mul(height) != Some(pixels.len()) {
        return;
    }
    let radius = radius.clamp(1, 32);
    let mut scratch = vec![0; pixels.len()];
    // Three separable box passes approximate a Gaussian at linear cost.
    for _ in 0..3 {
        axis(pixels, &mut scratch, width, height, radius, true);
        axis(&scratch, pixels, width, height, radius, false);
    }
    for pixel in pixels {
        let mut mixed = 0xff000000;
        for shift in [0, 8, 16] {
            mixed |=
                (((*pixel >> shift & 255) * 38 + (tint >> shift & 255) * 217 + 127) / 255) << shift;
        }
        *pixel = mixed;
    }
}

fn axis(
    input: &[u32],
    output: &mut [u32],
    width: usize,
    height: usize,
    radius: usize,
    horizontal: bool,
) {
    let (lines, length, stride) = if horizontal {
        (height, width, 1)
    } else {
        (width, height, width)
    };
    let count = (2 * radius + 1) as u64;
    for line in 0..lines {
        let start = if horizontal { line * width } else { line };
        let mut sums = [0u64; 3];
        for k in 0..2 * radius + 1 {
            let position = k.saturating_sub(radius).min(length - 1);
            let pixel = input[start + position * stride];
            for (channel, sum) in sums.iter_mut().enumerate() {
                *sum += u64::from(pixel >> (channel * 8) & 255);
            }
        }
        for x in 0..length {
            let mut pixel = 0;
            for (channel, sum) in sums.iter().enumerate() {
                pixel |= (((*sum + count / 2) / count) as u32) << (channel * 8);
            }
            output[start + x * stride] = pixel;
            let old = input[start + x.saturating_sub(radius) * stride];
            let next = input[start + (x + radius + 1).min(length - 1) * stride];
            for (channel, sum) in sums.iter_mut().enumerate() {
                *sum = *sum - u64::from(old >> (channel * 8) & 255)
                    + u64::from(next >> (channel * 8) & 255);
            }
        }
    }
}
