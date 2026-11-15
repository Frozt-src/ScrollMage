//! A capsule in logo vector units, and when the logo may animate.
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
