use crate::settings::{Settings, WheelTuning};

// One momentum model for every preset. Each wheel notch adds distance to a
// remaining budget that drains exponentially: output starts at full speed and
// eases out, like a flywheel with friction. The tuning's response sets the
// friction. Super smooth's free-spin is the same model with a low "glide"
// friction while a flick is coasting; once the glide slows to a crawl the
// normal friction takes over and finishes the remaining distance.
//
// Draining a budget, instead of integrating a velocity, keeps every scroll
// exact: the units sent always add up to the distance that was requested.

/// Normal friction per second, as a share of the tuning's response value.
const FRICTION_PER_RESPONSE: f64 = 0.5;
/// A flick: a same-direction notch within this many milliseconds of the last.
const FLICK_WINDOW_MS: f64 = 80.;
/// Friction per second while a flick is gliding.
const GLIDE_FRICTION: f64 = 1.2;
/// The glide ends, and normal friction resumes, below this speed (units/s).
const GLIDE_STOP_SPEED: f64 = 30.;
const GLIDE_MAX_SPEED: f64 = 18000.;
/// Do not dump missed glide into an app after a worker stall this long.
const MAX_GLIDE_FRAME_SECONDS: f64 = 0.25;
/// Below this many units of remaining budget, finish the scroll at once.
const SETTLE_UNITS: f64 = 0.5;
/// Largest remaining budget per axis, in wheel units.
const MAX_BUDGET: f64 = 16384.;

#[derive(Default)]
struct Axis {
    /// Distance still to travel. Its sign is the scroll direction.
    remaining: f64,
    /// Travelled distance not yet sent, below one whole wheel unit.
    unsent: f64,
    friction: f64,
    gliding: bool,
    last_time: Option<f64>,
    last_input: Option<(f64, i32)>,
}
impl Axis {
    fn advance(&mut self, now: f64) {
        if let Some(then) = self.last_time {
            let dt = ((now - then) / 1000.).max(0.);
            if self.gliding && dt > MAX_GLIDE_FRAME_SECONDS {
                self.remaining = 0.;
                self.gliding = false;
            }
            if self.remaining != 0. {
                let friction = if self.gliding {
                    GLIDE_FRICTION
                } else {
                    self.friction
                };
                // Exact for any frame length, so the frame rate cannot change
                // how far a scroll travels.
                let travel = self.remaining * -(-friction * dt).exp_m1();
                self.remaining -= travel;
                self.unsent += travel;
                if self.gliding && self.remaining.abs() * GLIDE_FRICTION < GLIDE_STOP_SPEED {
                    self.gliding = false;
                }
                if !self.gliding && self.remaining.abs() < SETTLE_UNITS {
                    self.unsent += self.remaining;
                    self.remaining = 0.;
                }
            }
        }
        self.last_time = Some(now);
    }
    fn active(&self) -> bool {
        self.remaining != 0.
    }
}

/// Two constant-size followers. Distances remain f64 until the wheel output boundary.
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
            // Reversing brakes at once: drop the old motion entirely.
            *axis = Axis {
                last_time: axis.last_time,
                ..Axis::default()
            };
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
        if !distance.is_finite() || (axis.remaining + distance).abs() > MAX_BUDGET {
            return false;
        }
        axis.remaining += distance;
        axis.friction = s.omega * FRICTION_PER_RESPONSE;
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
                    .min(GLIDE_MAX_SPEED);
                let glide = (speed / GLIDE_FRICTION).min(MAX_BUDGET);
                axis.remaining = f64::from(direction) * axis.remaining.abs().max(glide);
                axis.gliding = true;
            }
        } else {
            axis.gliding = false;
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
            let p = axis.unsent;
            *out = if (p - p.round()).abs() < 1e-9 {
                p.round()
            } else {
                p.trunc()
            } as i32;
            axis.unsent -= f64::from(*out);
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
