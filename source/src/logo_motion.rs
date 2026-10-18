//! Elastic wheel motion in the rebuilt, tail-connected logo's final l.
use crate::branding::Pill;

pub const CYCLE: f64 = 4.;

fn settle(t: f64, response: f64) -> f64 {
    if t >= 1.6 {
        1.
    } else {
        1. - (1. + response * t) * (-response * t).exp()
    }
}

pub fn wheel(time: f64, animate: bool) -> Pill {
    if !animate || !time.is_finite() {
        return Pill {
            x: 1171.5,
            y: 100.,
            width: 28.,
            height: 106.,
        };
    }
    let phase = time.rem_euclid(CYCLE);
    let (top, bottom) = if phase < 0.35 {
        (34., 140.)
    } else if phase < 2. {
        let t = phase - 0.35;
        (34. + 118. * settle(t, 10.), 140. + 118. * settle(t, 18.))
    } else if phase < 2.35 {
        (152., 258.)
    } else {
        let t = phase - 2.35;
        (152. - 118. * settle(t, 18.), 258. - 118. * settle(t, 10.))
    };
    let height = bottom - top;
    let width = 28. * (106. / height).sqrt();
    Pill {
        x: 1185.5 - width / 2.,
        y: top,
        width,
        height,
    }
}
