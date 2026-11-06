//! Motion, after the design guide's fluid interfaces: one spring model, set by a damping ratio and a response, drives
//! every movement; a retarget keeps position and velocity, so motion is interruptible without a jump; the projection
//! turns a flick's velocity into where it comes to rest.

/// A damped spring toward `target`, with unit mass: stiffness (2π / response)² and damping 4π · ratio / response.
/// `max_speed` (cells per second) keeps a step at most one cell per frame. With `instant` (Reduced motion) it jumps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub pos: f64,
    pub vel: f64,
    pub target: f64,
    pub damping: f64,
    pub response: f64,
    pub max_speed: f64,
}

/// A spring's tuning: damping ratio (1.0 is critical: no overshoot) and response in seconds.
#[derive(Clone, Copy, Debug)]
pub struct Tuning {
    pub damping: f64,
    pub response: f64,
}

impl Spring {
    /// A spring at rest at `pos`.
    pub fn new(t: Tuning, pos: f64, max_speed: f64) -> Self {
        Self {
            pos,
            vel: 0.0,
            target: pos,
            damping: t.damping,
            response: t.response,
            max_speed,
        }
    }

    /// A new target; position and velocity carry on from where they are (velocity handoff).
    pub fn retarget(&mut self, target: f64) {
        self.target = target;
    }

    /// Jumps to `pos` at rest.
    pub fn snap(&mut self, pos: f64) {
        *self = Self {
            pos,
            vel: 0.0,
            target: pos,
            ..*self
        };
    }

    /// Whether it rests at its target.
    pub fn settled(&self) -> bool {
        self.pos == self.target && self.vel == 0.0
    }

    /// Advances by `dt` seconds (in 1 ms substeps, semi-implicit Euler); `instant` reaches the target at once.
    pub fn step(&mut self, dt: f64, instant: bool) {
        if instant {
            self.snap(self.target);
            return;
        }
        let w = 2.0 * std::f64::consts::PI / self.response;
        let (k, c) = (w * w, 2.0 * self.damping * w);
        let n = (dt / 0.001).ceil().max(1.0) as usize;
        let h = dt / n as f64;
        for _ in 0..n {
            self.vel += (-k * (self.pos - self.target) - c * self.vel) * h;
            self.vel = self.vel.clamp(-self.max_speed, self.max_speed);
            self.pos += self.vel * h;
        }
        if (self.pos - self.target).abs() < 0.01 && self.vel.abs() < 0.1 {
            self.snap(self.target);
        }
    }

    /// The position rounded to a whole cell, as drawn.
    pub fn cell(&self) -> i32 {
        self.pos.round() as i32
    }
}

/// The guide's projection: how far a movement at `velocity` (units per second) goes on before it rests, with a
/// deceleration rate per millisecond (UIKit's 0.99 is "fast").
pub fn projection(velocity: f64, deceleration: f64) -> f64 {
    velocity / 1000.0 * deceleration / (1.0 - deceleration)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning {
        damping: 1.0,
        response: 0.3,
    };

    #[test]
    fn critical_damping_never_overshoots_and_settles() {
        let mut s = Spring::new(T, 6.0, f64::INFINITY);
        s.retarget(0.0);
        let mut min = f64::MAX;
        for _ in 0..120 {
            s.step(1.0 / 60.0, false);
            min = min.min(s.pos);
        }
        assert!(min >= 0.0, "overshot to {min}");
        assert!(s.settled());
    }

    #[test]
    fn a_retarget_keeps_position_and_velocity() {
        let mut s = Spring::new(T, 6.0, f64::INFINITY);
        s.retarget(0.0);
        for _ in 0..6 {
            s.step(1.0 / 60.0, false);
        }
        let (pos, vel) = (s.pos, s.vel);
        assert!(vel < 0.0);
        s.retarget(12.0);
        assert_eq!((s.pos, s.vel), (pos, vel));
        s.step(1.0 / 60.0, false);
        assert!((s.pos - pos).abs() < 1.0, "jumped from {pos} to {}", s.pos);
    }

    #[test]
    fn reduced_motion_arrives_in_one_frame_and_speed_is_capped() {
        let mut s = Spring::new(T, 6.0, 60.0);
        s.retarget(0.0);
        s.step(1.0 / 60.0, true);
        assert!(s.settled() && s.pos == 0.0);
        let mut s = Spring::new(
            Tuning {
                damping: 1.0,
                response: 0.15,
            },
            0.0,
            60.0,
        );
        s.retarget(20.0);
        let mut last = 0.0;
        for _ in 0..120 {
            s.step(1.0 / 60.0, false);
            assert!(
                s.pos - last <= 1.0 + 1e-9,
                "stepped {} cells in a frame",
                s.pos - last
            );
            last = s.pos;
        }
    }

    #[test]
    fn projection_matches_the_guide() {
        assert!((projection(1000.0, 0.99) - 99.0).abs() < 1e-9);
    }
}
