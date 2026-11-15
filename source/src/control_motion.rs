use crate::motion::Spring;

/// Aim at 0 or 1; without animation, jump there at rest.
fn aim(spring: &mut Spring, on: bool, animate: bool) {
    spring.target = f64::from(on);
    if !animate {
        spring.position = spring.target;
        spring.velocity = 0.;
    }
}
/// Advance `dt` seconds; once close enough, snap to rest. Returns whether it still moves.
fn settle(spring: &mut Spring, dt: f64, omega: f64) -> bool {
    spring.step(dt, omega);
    let active = (spring.position - spring.target).abs() > 0.001 || spring.velocity.abs() > 0.01;
    if !active {
        spring.position = spring.target;
        spring.velocity = 0.;
    }
    active
}

/// Retargetable visual motion. The native checkbox remains the logical state.
#[derive(Clone, Copy, Debug, Default)]
pub struct Toggle(Spring);
impl Toggle {
    pub fn target(&mut self, on: bool, animate: bool) {
        aim(&mut self.0, on, animate);
    }
    pub fn step(&mut self, dt: f64) -> bool {
        settle(&mut self.0, dt, 24.)
    }
    pub fn position(self) -> f64 {
        self.0.position.clamp(0., 1.)
    }
    pub fn stretch(self) -> f64 {
        (self.0.velocity.abs() * 0.45).min(4.)
    }
}

/// Height of the Scroll settings card: 0 shows only the preset row, 1 shows
/// every tuning row. Critically damped both ways: it eases out without
/// overshooting, so the card never wobbles or cuts into the preset row.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reveal(Spring);
impl Reveal {
    pub fn target(&mut self, open: bool, animate: bool) {
        aim(&mut self.0, open, animate);
    }
    pub fn open(self) -> bool {
        self.0.target == 1.
    }
    /// Advance by `dt` seconds (a stall counts as at most 0.1 s); returns
    /// whether the card is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        settle(&mut self.0, dt.clamp(0., 0.1), 18.)
    }
    /// Card height in logical pixels between `collapsed` and `expanded`.
    pub fn height(self, collapsed: i32, expanded: i32) -> i32 {
        collapsed + (f64::from(expanded - collapsed) * self.0.position.max(0.)).round() as i32
    }
}
