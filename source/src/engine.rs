use crate::{
    motion::Spring,
    settings::{Settings, WheelTuning},
};

#[derive(Default)]
struct Axis {
    spring: Spring,
    last_time: Option<f64>,
    last_input: Option<(f64, i32)>,
    omega: f64,
}
impl Axis {
    fn advance(&mut self, now: f64) {
        if let Some(then) = self.last_time {
            self.spring.step(((now - then) / 1000.).max(0.), self.omega);
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
        self.spring.position != self.spring.target || self.spring.velocity != 0.
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
