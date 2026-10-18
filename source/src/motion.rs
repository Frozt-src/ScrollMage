/// Exact critical damping for a fixed target over dt seconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spring {
    pub position: f64,
    pub target: f64,
    pub velocity: f64,
}
impl Spring {
    pub fn step(&mut self, dt: f64, omega: f64) {
        if !dt.is_finite() || dt <= 0. || !omega.is_finite() || omega <= 0. {
            return;
        }
        let y = self.position - self.target;
        let p = self.velocity + omega * y;
        let e = (-omega * dt).exp();
        self.position = self.target + (y + p * dt) * e;
        self.velocity = (self.velocity - omega * p * dt) * e;
    }
}
pub fn momentum(position: &mut f64, velocity: &mut f64, dt: f64, friction: f64) {
    if !dt.is_finite() || dt <= 0. || !friction.is_finite() || friction <= 0. {
        return;
    }
    let decay = (-friction * dt).exp();
    *position += *velocity * (1. - decay) / friction;
    *velocity *= decay;
    if velocity.abs() < 5. {
        *velocity = 0.;
    }
}
pub fn rubber_band(distance: f64, viewport: f64) -> f64 {
    if !distance.is_finite() || !viewport.is_finite() || viewport <= 0. {
        return 0.;
    }
    (0.55 * distance / (1. + distance.abs() / (0.25 * viewport)))
        .clamp(-0.12 * viewport, 0.12 * viewport)
}
/// Owned content only. OS-provided momentum must never start a second local decay.
#[derive(Clone, Copy, Debug)]
pub struct GestureAxis {
    pub position: f64,
    pub maximum: f64,
    pub viewport: f64,
    velocity: f64,
    excess: f64,
    recovery: Spring,
    dragging: bool,
    local_momentum: bool,
}
impl GestureAxis {
    pub fn new(position: f64, maximum: f64, viewport: f64) -> Self {
        Self {
            position,
            maximum,
            viewport,
            velocity: 0.,
            excess: 0.,
            recovery: Spring::default(),
            dragging: false,
            local_momentum: false,
        }
    }
    pub fn begin(&mut self) {
        self.dragging = true;
        self.velocity = 0.;
        self.local_momentum = false;
        let shown = self.recovery.position;
        self.excess = if shown == 0. {
            0.
        } else {
            shown / (0.55 - shown.abs() / (0.25 * self.viewport))
        };
    }
    pub fn change(&mut self, delta: f64, dt: f64) {
        if !delta.is_finite() {
            return;
        }
        let raw = self.position + self.excess + delta;
        self.position = raw.clamp(0., self.maximum);
        self.excess = raw - self.position;
        self.recovery.position = rubber_band(self.excess, self.viewport);
        if dt.is_finite() && dt > 0. {
            self.velocity = delta / dt;
        }
    }
    pub fn end(&mut self, os_supplies_momentum: bool) {
        self.dragging = false;
        self.local_momentum = !os_supplies_momentum && self.excess == 0.;
        if !self.local_momentum {
            self.velocity = 0.;
        }
        self.recovery.target = 0.;
        self.recovery.velocity = 0.;
    }
    pub fn cancel(&mut self) {
        self.end(true);
    }
    pub fn step(&mut self, dt: f64, friction: f64, omega: f64) {
        if self.dragging {
            return;
        }
        if self.local_momentum {
            momentum(&mut self.position, &mut self.velocity, dt, friction);
            let clamped = self.position.clamp(0., self.maximum);
            if clamped != self.position {
                self.recovery.position = rubber_band(self.position - clamped, self.viewport);
                self.position = clamped;
                self.velocity = 0.;
            }
            self.local_momentum = self.velocity != 0.;
        }
        self.recovery.step(dt, omega);
        if self.recovery.position.abs() < 0.01 && self.recovery.velocity.abs() < 0.1 {
            self.recovery = Spring::default();
            self.excess = 0.;
        }
    }
    pub fn rendered(&self) -> f64 {
        self.position + self.recovery.position
    }
    pub fn active(&self) -> bool {
        self.local_momentum || (!self.dragging && self.recovery.position != 0.)
    }
}
