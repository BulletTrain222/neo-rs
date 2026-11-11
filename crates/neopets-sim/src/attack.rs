//! Tor's attack step, hero state 13 (SPEC-0036 Attack input (Tor), Charge completion, Combo
//! windows, Attack router): the hold with its 50-tick power threshold, the three-swing combo with
//! its press windows, the chained variant (at most 3 repeats), the power step and the requests
//! each sub-state makes. Sub-states end when their clip reaches progress 0.99.
//! The gesture starts (SPEC-0036 Attack router: gesture mode 1 the flick lunge, sub-state 9; 2
//! the clockwise attack, 7; 3 the counter-clockwise attack, 8) wait for their clip's end.
//! The stick step of sub-states 7, 8 and 10 is `hero::StickStep`, set by the host.
//! Not covered: Roberta.

/// Animation ids Tor requests (main family).
pub const HOLD_START: u32 = 22;
pub const HOLD_LOOP: u32 = 13;
pub const SWING_1: u32 = 14;
pub const SWING_2: u32 = 15;
pub const FINISHER: u32 = 16;
pub const CHAINED: u32 = 17;
pub const POWER_NO_MOTE: u32 = 0x1B;
pub const CLOCKWISE: u32 = 19;
pub const COUNTER_CLOCKWISE: u32 = 18;
pub const LUNGE: u32 = 20;
/// The sword bounce of sub-state 13 (SPEC-0036 Tor's arm table; SPEC-0048 Element recoil).
pub const RECOIL: u32 = 26;
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
    /// The Mote in word 2, which picks the power clip (the host's word).
    pub mote: Option<u16>,
    /// This build's endless-spin cheat (not the original's): the clockwise and counter-clockwise
    /// attacks start again at their end while attack is held.
    pub endless_spin: bool,
}

/// SPEC-0036 Attack animation choice, Tor.
pub fn power_clip(mote: Option<u16>) -> u32 {
    match mote {
        Some(161 | 167) => 0x1C,
        Some(162) => 0x12,
        Some(163 | 165) => 0x1E,
        Some(169 | 171) => 0x1D,
        Some(172 | 173) => 0x1F,
        _ => POWER_NO_MOTE,
    }
}

impl TorAttack {
    /// The attack start (the R2 or Circle press edge entering state 13): the reset, then the
    /// hold (gesture mode 0).
    pub fn start() -> Self {
        Self::start_with(0)
    }

    /// The attack start with the gesture mode word M: 0 the hold, 1 the flick lunge (9), 2 the
    /// clockwise attack (7), 3 the counter-clockwise attack (8).
    pub fn start_with(mode: u8) -> Self {
        let sub = match mode {
            1 => 9,
            2 => 7,
            3 => 8,
            _ => 1,
        };
        Self { sub, cleared: true, ..Default::default() }
    }

    fn go(&mut self, sub: u8) {
        self.sub = sub;
        self.f = 0;
        self.cleared = true;
    }

    pub fn active(&self) -> bool {
        self.sub != 0
    }

    /// The recoil request (SPEC-0048 Element recoil; SPEC-0028 Blade recoil): sub-state 13,
    /// which requests the bounce (26) and ends with it. A request while already recoiling does
    /// nothing.
    pub fn recoil(&mut self) {
        if self.sub != 13 {
            self.go(13);
            self.restart = true;
        }
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
                if done(self, power_clip(self.mote)) {
                    self.go(0);
                }
            }
            13 => {
                if done(self, RECOIL) {
                    self.go(0);
                }
            }
            7 | 8 | 9 => {
                let id = match self.sub {
                    7 => CLOCKWISE,
                    8 => COUNTER_CLOCKWISE,
                    _ => LUNGE,
                };
                if done(self, id) {
                    if self.endless_spin && held && self.sub != 9 {
                        let sub = self.sub;
                        self.go(sub);
                        self.restart = true;
                    } else {
                        self.go(0);
                    }
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
            7 => CLOCKWISE,
            8 => COUNTER_CLOCKWISE,
            9 => LUNGE,
            13 => RECOIL,
            _ => power_clip(self.mote),
        };
        Some(Request { action, restart: std::mem::take(&mut self.restart) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoil_bounce() {
        // A recoil mid-swing: sub-state 13 requests 26 (restarted), ends when it finishes, and a
        // second request while recoiling changes nothing.
        let mut a = TorAttack::start();
        a.tick(false, false, Clip { action: HOLD_START, progress: 0.1 });
        assert_eq!(a.sub, 2);
        a.recoil();
        let r = a.tick(false, false, Clip { action: SWING_1, progress: 0.3 }).unwrap();
        assert_eq!((a.sub, r.action, r.restart), (13, RECOIL, true));
        a.recoil();
        assert_eq!(a.tick(false, false, Clip { action: RECOIL, progress: 0.5 }).unwrap(), Request { action: RECOIL, restart: false });
        assert!(a.tick(false, false, Clip { action: RECOIL, progress: 1.0 }).is_none());
    }

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
    fn gesture_starts() {
        // Mode 2 starts the clockwise attack (request 19) and returns to the stance at its end.
        let mut a = TorAttack::start_with(2);
        assert_eq!(a.sub, 7);
        let r = a.tick(true, false, Clip { action: 0, progress: 0.0 }).unwrap();
        assert_eq!(r.action, CLOCKWISE);
        assert!(a.tick(true, false, Clip { action: CLOCKWISE, progress: 0.99 }).is_none());
        assert_eq!((TorAttack::start_with(3).sub, TorAttack::start_with(1).sub, TorAttack::start_with(0).sub), (8, 9, 1));
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
