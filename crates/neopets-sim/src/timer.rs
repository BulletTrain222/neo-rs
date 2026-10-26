//! The on-screen countdown timer (SPEC-0023 section 31; SPEC-0030 rows 244 to 249): one
//! engine-wide timer with a visible and a running flag, a value in seconds, a limit and a
//! direction, stepped once per in-play tick while visible.

/// The frame step (1/60 s, float32).
pub const STEP: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Timer {
    pub visible: bool,
    pub running: bool,
    pub value: f32,
    pub limit: f32,
    /// Counting up (direction not 0).
    pub up: bool,
}

impl Timer {
    /// ActivateTimer(start, limit, direction): visible and running with value = start (no clamp).
    pub fn activate(&mut self, start: f32, limit: f32, up: bool) {
        self.value = start;
        self.limit = limit;
        self.up = up;
        self.visible = true;
        self.running = true;
    }

    /// DeactivateTimer: visible off; running and the value are kept.
    pub fn deactivate(&mut self) {
        self.visible = false;
    }

    /// One in-play tick: while visible and running, the step and then the clamp (a limit of 0 or
    /// more is a floor counting down and a cap counting up; a negative limit is none).
    pub fn tick(&mut self) {
        if !self.visible || !self.running {
            return;
        }
        if self.up {
            self.value += STEP;
            if self.limit >= 0.0 && self.value > self.limit {
                self.value = self.limit;
            }
        } else {
            self.value -= STEP;
            if self.limit >= 0.0 && self.value < self.limit {
                self.value = self.limit;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_and_gates() {
        let mut t = Timer::default();
        // Counting up with a limit of 0 pins the value at 0 from the first running tick.
        t.activate(0.0, 0.0, true);
        t.tick();
        assert_eq!(t.value, 0.0);
        // A start beyond the limit is kept until the first running tick.
        t.activate(5.0, 2.0, true);
        assert_eq!(t.value, 5.0);
        t.tick();
        assert_eq!(t.value, 2.0);
        // Counting down stops at its floor; hidden it freezes.
        t.activate(0.02, 0.0, false);
        t.tick();
        t.tick();
        assert_eq!(t.value, 0.0);
        t.activate(10.0, -1.0, false);
        t.deactivate();
        t.tick();
        assert_eq!(t.value, 10.0);
        // 60 ticks counting up move about one second.
        t.activate(0.0, 600.0, true);
        for _ in 0..60 {
            t.tick();
        }
        assert!((t.value - 1.0).abs() < 1e-4);
    }
}
