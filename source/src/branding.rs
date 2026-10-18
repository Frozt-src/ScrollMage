//! Scroll lettering reconstructed from the supplied image. Coordinates are vector units.
pub const WIDTH: f64 = 460.;
pub const HEIGHT: f64 = 140.;
pub const CYCLE: f64 = 4.;
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Pill {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
pub fn should_animate(visible: bool, minimized: bool, reduced_motion: bool) -> bool {
    visible && !minimized && !reduced_motion
}
fn response(t: f64, omega: f64) -> f64 {
    if t >= 1.6 {
        1.
    } else {
        1. - (1. + omega * t) * (-omega * t).exp()
    }
}
pub fn pill(time: f64, animate: bool) -> Pill {
    if !animate || !time.is_finite() {
        return Pill {
            x: 439.,
            y: 41.5,
            width: 17.,
            height: 54.,
        };
    }
    let phase = time.rem_euclid(CYCLE);
    let (top, bottom) = if phase < 0.35 {
        (7., 61.)
    } else if phase < 2. {
        let t = phase - 0.35;
        (7. + 69. * response(t, 10.), 61. + 69. * response(t, 18.))
    } else if phase < 2.35 {
        (76., 130.)
    } else {
        let t = phase - 2.35;
        (76. - 69. * response(t, 18.), 130. - 69. * response(t, 10.))
    };
    let height = bottom - top;
    let width = 17. * (54. / height).sqrt();
    Pill {
        x: 447.5 - width / 2.,
        y: top,
        width,
        height,
    }
}
#[derive(Clone, Copy)]
pub enum Command {
    Move(f32, f32),
    Line(f32, f32),
    Curve(f32, f32, f32, f32, f32, f32),
    Close,
}
use Command::*;
pub struct Glyph {
    pub name: &'static str,
    pub character: char,
    pub advance: f32,
    pub stroke: f32,
    pub path: &'static [Command],
}
pub const GLYPHS: &[Glyph] = &[
    Glyph {
        name: "s",
        character: 's',
        advance: 96.,
        stroke: 22.,
        path: &[
            Move(59., 51.),
            Curve(42., 39., 20., 42., 15., 60.),
            Curve(5., 87., 67., 76., 62., 107.),
            Curve(57., 130., 27., 132., 12., 120.),
        ],
    },
    Glyph {
        name: "c",
        character: 'c',
        advance: 96.,
        stroke: 22.,
        path: &[
            Move(62., 52.),
            Curve(43., 39., 8., 45., 8., 79.),
            Line(8., 91.),
            Curve(8., 125., 42., 132., 62., 119.),
        ],
    },
    Glyph {
        name: "r",
        character: 'r',
        advance: 74.,
        stroke: 22.,
        path: &[
            Move(9., 125.),
            Line(9., 72.),
            Curve(9., 51., 24., 44., 48., 46.),
        ],
    },
    Glyph {
        name: "o",
        character: 'o',
        advance: 116.,
        stroke: 22.,
        path: &[
            Move(44., 46.),
            Curve(69., 46., 77., 60., 77., 86.),
            Line(77., 90.),
            Curve(77., 116., 64., 126., 44., 126.),
            Curve(20., 126., 10., 113., 10., 91.),
            Line(10., 80.),
            Curve(10., 57., 24., 46., 44., 46.),
            Close,
        ],
    },
    Glyph {
        name: "l",
        character: 'l',
        advance: 54.,
        stroke: 23.,
        path: &[Move(11.5, 11.5), Line(11.5, 125.5)],
    },
];
pub const WORD: &[(usize, f32)] = &[
    (0, 0.),
    (1, 96.),
    (2, 192.),
    (3, 266.),
    (4, 382.),
    (4, 436.),
];
