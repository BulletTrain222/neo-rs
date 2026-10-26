//! The on-screen countdown timer (SPEC-0023 section 31; SPEC-0030 rows 244 to 249): one
//! engine-wide timer with a visible and a running flag, a value in seconds, a limit and a
//! direction, stepped once per in-play tick while visible.

/// The frame step (1/60 s, float32).
pub const STEP: f32 = 1.0 / 60.0;

/// The default colour word (0x2DD040): 128 per channel.
pub const DEFAULT_COLOUR: [u8; 4] = [128; 4];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    pub visible: bool,
    pub running: bool,
    pub value: f32,
    pub limit: f32,
    /// Counting up (direction not 0).
    pub up: bool,
    /// The colour written to element 0xD8 each tick (Activate's or SetTimerColor's).
    pub colour: [u8; 4],
}

impl Default for Timer {
    fn default() -> Self {
        Self { visible: false, running: false, value: 0.0, limit: 0.0, up: false, colour: DEFAULT_COLOUR }
    }
}

impl Timer {
    /// ActivateTimer(start, limit, direction): visible and running with value = start (no clamp).
    pub fn activate(&mut self, start: f32, limit: f32, up: bool) {
        self.value = start;
        self.limit = limit;
        self.up = up;
        self.colour = DEFAULT_COLOUR;
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

/// The 0xD8 text of a value (SPEC-0023 section 13): hundredths h = round half away from zero
/// (value x 100), unsigned and clamped to 59 999, shown as M:SS:CC.
pub fn text(value: f32) -> String {
    let r = (value * 100.0).round() as i64;
    let h = if r < 0 { 59_999 } else { (r as u64).min(59_999) };
    format!("{}:{:02}:{:02}", h / 6000, (h / 100) % 60, h % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_text() {
        assert_eq!(text(65.5), "1:05:50");
        assert_eq!(text(0.25), "0:00:25");
        assert_eq!(text(-1.0), "9:59:99");
        assert_eq!(text(1000.0), "9:59:99");
    }

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
