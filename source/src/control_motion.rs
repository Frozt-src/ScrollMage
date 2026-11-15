use crate::motion::Spring;

/// Retargetable visual motion. The native checkbox remains the logical state.
#[derive(Clone, Copy, Debug, Default)]
pub struct Toggle(Spring);
impl Toggle {
    pub fn target(&mut self, on: bool, animate: bool) {
        self.0.target = f64::from(on);
        if !animate {
            self.0.position = self.0.target;
            self.0.velocity = 0.;
        }
    }
    pub fn step(&mut self, dt: f64) -> bool {
        self.0.step(dt, 24.);
        let active =
            (self.0.position - self.0.target).abs() > 0.001 || self.0.velocity.abs() > 0.01;
        if !active {
            self.0.position = self.0.target;
            self.0.velocity = 0.;
        }
        active
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
pub struct Reveal {
    position: f64,
    velocity: f64,
    target: f64,
}
impl Reveal {
    pub fn target(&mut self, open: bool, animate: bool) {
        self.target = f64::from(open);
        if !animate {
            self.position = self.target;
            self.velocity = 0.;
        }
    }
    pub fn open(self) -> bool {
        self.target == 1.
    }
    /// Advance by `dt` seconds; returns whether the card is still moving.
    pub fn step(&mut self, dt: f64) -> bool {
        let omega = 18.;
        let mut left = dt.clamp(0., 0.1);
        while left > 0. {
            let h = left.min(1. / 240.);
            let accel = omega * omega * (self.target - self.position) - 2. * omega * self.velocity;
            self.velocity += accel * h;
            self.position += self.velocity * h;
            left -= h;
        }
        let active = (self.position - self.target).abs() > 0.001 || self.velocity.abs() > 0.01;
        if !active {
            self.position = self.target;
            self.velocity = 0.;
        }
        active
    }
    /// Card height in logical pixels between `collapsed` and `expanded`.
    pub fn height(self, collapsed: i32, expanded: i32) -> i32 {
        collapsed + (f64::from(expanded - collapsed) * self.position.max(0.)).round() as i32
    }
}
