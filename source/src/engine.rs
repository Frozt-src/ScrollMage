use crate::{
    motion::Spring,
    settings::{Settings, WheelTuning},
};

// Software free-spin: fast consecutive notches launch a bounded exponential coast.
// These are fixed Super smooth behavior, independent of the drag-preview friction.
const FLICK_WINDOW_MS: f64 = 80.;
const COAST_DECAY: f64 = 1.2;
const COAST_MAX_SPEED: f64 = 18000.;
const COAST_STOP_SPEED: f64 = 30.;
const MAX_COAST_FRAME_SECONDS: f64 = 0.25;

#[derive(Default)]
struct Axis {
    spring: Spring,
    last_time: Option<f64>,
    last_input: Option<(f64, i32)>,
    omega: f64,
    coast_velocity: f64,
}
impl Axis {
    fn advance(&mut self, now: f64) {
        if let Some(then) = self.last_time {
            let dt = ((now - then) / 1000.).max(0.);
            if self.coast_velocity != 0. {
                if dt > MAX_COAST_FRAME_SECONDS {
                    // Do not dump missed momentum into an app after a worker stall.
                    self.coast_velocity = 0.;
                } else {
                    let remaining =
                        (self.coast_velocity.abs() / COAST_STOP_SPEED).ln() / COAST_DECAY;
                    let elapsed = dt.min(remaining.max(0.));
                    let travel =
                        self.coast_velocity * -(-COAST_DECAY * elapsed).exp_m1() / COAST_DECAY;
                    // Translating both coordinates leaves the finite impulse's exact
                    // spring response intact, while coasting is frame-rate independent.
                    self.spring.position += travel;
                    self.spring.target += travel;
                    self.coast_velocity = if dt >= remaining {
                        0.
                    } else {
                        self.coast_velocity * (-COAST_DECAY * dt).exp()
                    };
                }
            }
            self.spring.step(dt, self.omega);
            if (self.spring.target - self.spring.position).abs() < 0.00001
                && self.spring.velocity.abs() < 0.005
            {
                self.spring.position = self.spring.target;
                self.spring.velocity = 0.;
            }
        }
        self.last_time = Some(now);
    }
    fn active(&self) -> bool {
        self.coast_velocity != 0.
            || self.spring.position != self.spring.target
            || self.spring.velocity != 0.
    }
}

/// Two constant-size followers. Coordinates remain f64 until the wheel output boundary.
#[derive(Default)]
pub struct Engine {
    axes: [Axis; 2],
    target: usize,
}
impl Engine {
    pub fn push(
        &mut self,
        now: f64,
        delta: i32,
        horizontal: bool,
        target: usize,
        s: &Settings,
    ) -> bool {
        self.push_tuned(now, delta, horizontal, target, s.wheel())
    }
    pub fn push_tuned(
        &mut self,
        now: f64,
        delta: i32,
        horizontal: bool,
        target: usize,
        s: WheelTuning,
    ) -> bool {
        if !now.is_finite()
            || delta == 0
            || target == 0
            || self
                .axes
                .iter()
                .any(|a| a.last_time.is_some_and(|t| now < t))
        {
            return false;
        }
        if target != self.target {
            self.clear();
            self.target = target;
        }
        let axis = &mut self.axes[usize::from(horizontal)];
        axis.advance(now);
        let direction = delta.signum();
        if axis.last_input.is_some_and(|(_, d)| d != direction) {
            axis.spring = Spring::default();
            axis.coast_velocity = 0.;
            axis.last_input = None;
        }
        let gain = axis.last_input.map_or(1., |(then, _)| {
            let dt = (now - then).max(1.);
            if dt < s.acceleration_ms {
                ((s.acceleration_ms / dt + 1.) * 0.5).clamp(1., s.acceleration_max)
            } else {
                1.
            }
        });
        let distance = s.distance * f64::from(delta) / 120. * gain;
        if !distance.is_finite() || (axis.spring.target + distance).abs() > 16384. {
            return false;
        }
        axis.spring.target += distance;
        axis.omega = s.omega;
        if s.free_spin {
            let interval = axis
                .last_input
                .and_then(|(then, _)| {
                    let dt = now - then;
                    (dt <= FLICK_WINDOW_MS).then_some(dt.max(8.))
                })
                .or_else(|| (delta.unsigned_abs() >= 240).then_some(40.));
            if let Some(interval) = interval {
                let speed = (s.distance * f64::from(delta).abs() / 120. * 1000. / interval * 1.5)
                    .min(COAST_MAX_SPEED);
                axis.coast_velocity = f64::from(direction) * axis.coast_velocity.abs().max(speed);
            }
        } else {
            axis.coast_velocity = 0.;
        }
        axis.last_input = Some((now, direction));
        true
    }
    pub fn tick(&mut self, now: f64, target: usize) -> [i32; 2] {
        if target != self.target || !now.is_finite() {
            self.clear();
            return [0, 0];
        }
        let mut output = [0; 2];
        for (axis, out) in self.axes.iter_mut().zip(&mut output) {
            if axis.last_time.is_some_and(|t| now < t) {
                continue;
            }
            axis.advance(now);
            let p = axis.spring.position;
            *out = if (p - p.round()).abs() < 1e-9 {
                p.round()
            } else {
                p.trunc()
            } as i32;
            axis.spring.position -= f64::from(*out);
            axis.spring.target -= f64::from(*out);
        }
        output
    }
    pub fn active(&self) -> bool {
        self.axes.iter().any(Axis::active)
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
