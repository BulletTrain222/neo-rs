//! Tor's attack step, hero state 13 (SPEC-0036 Attack input (Tor), Charge completion, Combo
//! windows, Attack router): the hold with its 50-tick power threshold, the three-swing combo with
//! its press windows, the chained variant (at most 3 repeats), the power step and the requests
//! each sub-state makes. Sub-states end when their clip reaches progress 0.99.
//! Not covered: the stick-gesture starts (sub-states 7, 8, 9), the power-ready check, Mote
//! choice of the power clip (no Mote: 0x1B), sounds and voices, Roberta.

/// Animation ids Tor requests (main family).
pub const HOLD_START: u32 = 22;
pub const HOLD_LOOP: u32 = 13;
pub const SWING_1: u32 = 14;
pub const SWING_2: u32 = 15;
pub const FINISHER: u32 = 16;
pub const CHAINED: u32 = 17;
pub const POWER_NO_MOTE: u32 = 0x1B;
const POWER_THRESHOLD: u32 = 50;

/// The playing clip as the attack step reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub action: u32,
    pub progress: f32,
}

/// What the step asks for this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub action: u32,
    /// Restart the clip even when the id is already current (chained repeats).
    pub restart: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TorAttack {
    /// Sub-state (+0xF90); 0 means the attack has ended (back to the stance).
    pub sub: u8,
    /// Sub-state tick counter F (+0xF8C), 0 on every transition.
    pub f: u32,
    repeats: u8,
    closed: bool,
    restart: bool,
    /// The hit list is cleared on every transition (the caller reads and resets this).
    pub cleared: bool,
}

impl TorAttack {
    /// The attack start (the R2 or Circle press edge entering state 13): the reset, then the
    /// hold (gesture mode 0).
    pub fn start() -> Self {
        Self { sub: 1, cleared: true, ..Default::default() }
    }

    fn go(&mut self, sub: u8) {
        self.sub = sub;
        self.f = 0;
        self.cleared = true;
    }

    pub fn active(&self) -> bool {
        self.sub != 0
    }

    /// One tick: `held` is the attack button held, `pressed` its press edge this tick; `clip`
    /// the playing main clip. Returns the request (None when the attack has just ended: the
    /// state returns to 1 and the idle is requested by the stance).
    pub fn tick(&mut self, held: bool, pressed: bool, clip: Clip) -> Option<Request> {
        let done = |s: &Self, id: u32| s.f != 0 && clip.action == id && clip.progress >= 0.99;
        match self.sub {
            1 => {
                if held {
                    self.f += 1;
                } else if self.f >= POWER_THRESHOLD {
                    self.go(10);
                } else {
                    self.go(2);
                }
            }
            2 | 3 => {
                let (lo, hi, next, id) = if self.sub == 2 { (20, 33, 3, SWING_1) } else { (17, 30, 4, SWING_2) };
                if pressed && !self.closed {
                    if self.f < lo {
                        self.go(6);
                        self.restart = true;
                    } else if self.f <= hi {
                        self.go(next);
                    } else {
                        self.closed = true;
                    }
                } else if done(self, id) {
                    self.go(0);
                }
            }
            4 => {
                if done(self, FINISHER) {
                    self.go(0);
                }
            }
            6 => {
                if pressed && self.repeats <= 2 && self.f > 10 && self.f < 20 {
                    self.repeats += 1;
                    self.go(6);
                    self.restart = true;
                } else if done(self, CHAINED) {
                    self.go(0);
                }
            }
            10 => {
                if done(self, POWER_NO_MOTE) {
                    self.go(0);
                }
            }
            _ => self.go(0),
        }
        if self.sub != 1 && self.sub != 0 {
            self.f += 1;
        }
        let action = match self.sub {
            0 => return None,
            1 => {
                if clip.action == HOLD_START && clip.progress >= 0.99 || clip.action == HOLD_LOOP {
                    HOLD_LOOP
                } else {
                    HOLD_START
                }
            }
            2 => SWING_1,
            3 => SWING_2,
            4 => FINISHER,
            6 => CHAINED,
            _ => POWER_NO_MOTE,
        };
        Some(Request { action, restart: std::mem::take(&mut self.restart) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the step with a clip model: a request starts its clip (or restarts it), and the
    /// clip's progress advances by 1 / length per tick. Returns (tick, sub-state, request) at
    /// each change.
    fn run(presses: &[u32], release: u32, lengths: &[(u32, u32)], ticks: u32) -> Vec<(u32, u8, u32)> {
        let len = |id: u32| lengths.iter().find(|l| l.0 == id).map_or(40, |l| l.1) as f32;
        let mut a = TorAttack::start();
        let mut clip = Clip { action: HOLD_START, progress: 0.0 };
        let mut age = 0u32;
        let mut log = vec![(0, a.sub, HOLD_START)];
        for t in 1..=ticks {
            let held = t < release;
            let pressed = presses.contains(&t);
            let before = a.sub;
            let req = a.tick(held, pressed, clip);
            let Some(req) = req else {
                log.push((t, 0, 0));
                break;
            };
            if req.action != clip.action || req.restart {
                clip = Clip { action: req.action, progress: 0.0 };
                age = 0;
            }
            age += 1;
            clip.progress = age as f32 / len(clip.action);
            if a.sub != before {
                log.push((t, a.sub, req.action));
            }
        }
        log
    }

    #[test]
    fn three_swing_combo_follows_the_spec_vector() {
        // SPEC-0036: release at t10, presses at t35 and t61: t10 sub-state 2 (request 14), t35
        // sub-state 3 (15), t61 sub-state 4 (16), back to the stance at t109.
        let log = run(&[35, 61], 10, &[(14, 44), (15, 60), (16, 48), (22, 36)], 200);
        assert_eq!(log[1], (10, 2, 14));
        assert_eq!(log[2], (35, 3, 15));
        assert_eq!(log[3], (61, 4, 16));
        let end = log.last().unwrap();
        assert!(end.1 == 0 && (108..=110).contains(&end.0), "{log:?}");
    }

    #[test]
    fn chained_variant_repeats_at_most_three_times() {
        // Release at t10, presses at t20, t36, t52 (and t68, refused): repeats then done near
        // t104 (52 ticks after the last restart).
        let log = run(&[20, 36, 52, 68, 84], 10, &[(17, 52), (14, 44)], 200);
        assert_eq!(log[2], (20, 6, 17));
        let repeats = log.iter().filter(|e| e.1 == 6).count();
        assert_eq!(repeats, 1, "sub-state 6 is entered once; repeats stay in 6: {log:?}");
        let end = log.last().unwrap();
        assert!(end.1 == 0 && (119..=122).contains(&end.0), "{log:?}");
    }

    #[test]
    fn held_sixty_ticks_is_a_power_step() {
        // Held 60 ticks: request 13 once 22 has played (t36), sub-state 10 with 0x1B at t60,
        // done near t104.
        let log = run(&[], 60, &[(22, 36), (0x1B, 44)], 200);
        assert_eq!(log[1].1, 10);
        assert_eq!(log[1].2, POWER_NO_MOTE);
        assert_eq!(log[1].0, 60);
        let end = log.last().unwrap();
        assert!(end.1 == 0 && (103..=105).contains(&end.0), "{log:?}");
    }

    #[test]
    fn a_late_press_closes_the_chain() {
        // A press 34 ticks into sub-state 2 closes the chain: no second swing.
        let log = run(&[44], 10, &[(14, 44)], 200);
        assert!(!log.iter().any(|e| e.1 == 3), "{log:?}");
    }
}
