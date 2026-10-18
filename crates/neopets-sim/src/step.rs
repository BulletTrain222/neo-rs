/// Turns elapsed host time into a whole number of fixed simulation ticks.
///
/// Time is kept as an exact remainder (nanoseconds × rate), so rates that do not divide a
/// second evenly, such as 60 Hz, never drift. The host supplies elapsed time; this type never
/// reads a clock. The original game's tick rate is not specified yet, so callers choose it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedStep {
    rate_hz: u32,
    max_ticks: u32,
    /// Fraction of the next tick already elapsed, in units of 1 / (rate × 1e9) seconds.
    carry: u128,
    dropped: u64,
}

const NANOS: u128 = 1_000_000_000;

impl FixedStep {
    /// `max_ticks` caps catch-up per call so a stalled host does not trigger a burst of ticks.
    pub fn new(rate_hz: u32, max_ticks: u32) -> Result<Self, &'static str> {
        if rate_hz == 0 || max_ticks == 0 {
            return Err("Tick rate and catch-up limit must be positive");
        }
        Ok(Self {
            rate_hz,
            max_ticks,
            carry: 0,
            dropped: 0,
        })
    }

    pub fn rate_hz(&self) -> u32 {
        self.rate_hz
    }

    /// Ticks that were due but skipped because of the catch-up limit.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Add elapsed time and return how many ticks are now due (at most `max_ticks`).
    pub fn advance(&mut self, elapsed_ns: u64) -> u32 {
        let total = self.carry + elapsed_ns as u128 * self.rate_hz as u128;
        let due = total / NANOS;
        self.carry = total % NANOS;
        if due > self.max_ticks as u128 {
            self.dropped = self.dropped.saturating_add((due - self.max_ticks as u128) as u64);
            return self.max_ticks;
        }
        due as u32
    }

    /// How far the host is between the last tick and the next, in [0, 1), for interpolation.
    pub fn alpha(&self) -> f32 {
        (self.carry as f64 / NANOS as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_hertz_is_exact_over_uneven_frames() {
        let mut step = FixedStep::new(60, 1000).unwrap();
        // One second split into frames that never line up with a 60 Hz tick.
        let frames = [7_000_001, 13_333_333, 16_666_666, 33_000_000, 930_000_000];
        let total: u64 = frames.iter().sum();
        assert_eq!(total, 1_000_000_000);
        let ticks: u32 = frames.iter().map(|&ns| step.advance(ns)).sum();
        assert_eq!(ticks, 60);
        assert_eq!(step.alpha(), 0.0);
    }

    #[test]
    fn carries_partial_ticks() {
        let mut step = FixedStep::new(60, 10).unwrap();
        assert_eq!(step.advance(10_000_000), 0);
        assert!((step.alpha() - 0.6).abs() < 1e-6);
        assert_eq!(step.advance(10_000_000), 1);
        assert!((step.alpha() - 0.2).abs() < 1e-6);
    }

    #[test]
    fn caps_catch_up_and_counts_dropped_ticks() {
        let mut step = FixedStep::new(60, 4).unwrap();
        assert_eq!(step.advance(1_000_000_000), 4);
        assert_eq!(step.dropped(), 56);
        assert_eq!(step.advance(u64::MAX), 4);
        assert!(step.dropped() > 56);
    }

    #[test]
    fn rejects_zero_rate_or_limit() {
        assert!(FixedStep::new(0, 4).is_err());
        assert!(FixedStep::new(60, 0).is_err());
    }

    #[test]
    fn simulation_runs_due_ticks_with_the_given_input() {
        let mut step = FixedStep::new(50, 8).unwrap();
        let mut sim = crate::Simulation::default();
        let input = crate::PlayerInput {
            move_x: 1,
            ..Default::default()
        };
        assert_eq!(sim.run_for(&mut step, 100_000_000, input), 5);
        assert_eq!(sim.tick, 5);
    }
}
