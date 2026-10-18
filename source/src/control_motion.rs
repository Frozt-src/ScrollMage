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
