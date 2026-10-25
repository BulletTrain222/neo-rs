//! The left-stick gesture machine (SPEC-0033 Behavior 52): a back-then-forward flick, a
//! clockwise turn or a counter-clockwise turn sets the gesture mode word M (1, 2 or 3) for a
//! short hold, which makes the next attack press start a gesture variant (SPEC-0036 Attack
//! router).

/// Per-hero values: the counter-clockwise step count N and the spin hold W (Tor: the words
/// 0x2DD7EC and 0x2DD7E8; Roberta: 3 and 10), and the flick hold (0x2DD7E4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Words {
    pub n: u32,
    pub w: u32,
    pub flick: u32,
}

pub const FLICK_HOLD: u32 = 0x002D_D7E4;
pub const TOR_SPIN_HOLD: u32 = 0x002D_D7E8;
pub const TOR_CCW_STEPS: u32 = 0x002D_D7EC;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Idle,
    Back,
    Flick,
    Track,
    Ccw,
    Cw,
    SpinA,
    SpinB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    None,
    Ccw,
    Cw,
    Reset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gesture {
    state: State,
    /// The stored quadrant Q.
    q: u8,
    turns: u32,
    hold: u32,
    /// The gesture mode word M.
    pub mode: u8,
}

/// The quadrant of a stick push: 0 up-right, 1 up-left, 2 down-left, 3 any other.
fn quadrant(f: f32, s: f32) -> u8 {
    if f > 0.0 && s < 0.0 {
        0
    } else if f > 0.0 && s > 0.0 {
        1
    } else if f < 0.0 && s > 0.0 {
        2
    } else {
        3
    }
}

impl Gesture {
    /// The reset while the hero is busy (or the game-state word is above 0): back to Idle with
    /// the counters zeroed; M keeps its value.
    pub fn reset(&mut self) {
        let mode = self.mode;
        *self = Self { mode, ..Self::default() };
    }

    /// Classify a step against the stored quadrant (a step needs F and S both non-zero).
    fn step(&mut self, f: f32, s: f32) -> Step {
        if f == 0.0 || s == 0.0 {
            return Step::None;
        }
        let q = quadrant(f, s);
        let old = self.q;
        if q == old {
            Step::None
        } else if q == (old + 1) % 4 {
            self.q = q;
            Step::Ccw
        } else if old == (q + 1) % 4 {
            // The step 0 to 3 stores Q := 2 (quirk).
            self.q = if old == 0 && q == 3 { 2 } else { q };
            Step::Cw
        } else {
            Step::Reset
        }
    }

    /// One tick with the dead-zoned forward F (up positive) and side S (left positive) values.
    pub fn tick(&mut self, f: f32, s: f32, w: &Words) {
        let centred = f == 0.0 && s == 0.0;
        match self.state {
            State::Idle | State::Back => {
                self.mode = 0;
                if s.abs() > 30.0 {
                    self.q = quadrant(f, s);
                    self.state = State::Track;
                } else if self.state == State::Idle && f < 0.0 {
                    self.state = State::Back;
                } else if self.state == State::Back && f > 0.0 {
                    self.hold = 0;
                    self.state = State::Flick;
                }
            }
            State::Flick => self.held(1, w.flick),
            State::Track => {
                self.turns = 0;
                match self.step(f, s) {
                    _ if centred => self.state = State::Idle,
                    Step::Ccw => self.state = State::Ccw,
                    Step::Cw => self.state = State::Cw,
                    Step::Reset => self.state = State::Idle,
                    Step::None => {}
                }
            }
            State::Ccw => match self.step(f, s) {
                _ if centred => self.state = State::Idle,
                Step::Ccw => {
                    self.hold = 0;
                    self.turns += 1;
                    if self.turns >= w.n {
                        self.state = State::SpinB;
                    }
                }
                Step::Cw => {
                    self.turns = 0;
                    self.state = State::Cw;
                }
                Step::Reset => self.state = State::Idle,
                Step::None => {}
            },
            State::Cw => match self.step(f, s) {
                _ if centred => self.state = State::Idle,
                Step::Ccw => {
                    self.turns = 0;
                    self.state = State::Ccw;
                }
                Step::Cw => {
                    // N is not tested here (quirk).
                    self.hold = 0;
                    self.turns += 1;
                    self.state = State::SpinA;
                }
                Step::Reset => self.state = State::Idle,
                Step::None => {}
            },
            State::SpinA => self.held(2, w.w),
            State::SpinB => self.held(3, w.w),
        }
    }

    /// A hold state: M for `ticks` ticks, then Idle.
    fn held(&mut self, mode: u8, ticks: u32) {
        if self.hold < ticks {
            self.mode = mode;
            self.hold += 1;
        } else {
            self.mode = 0;
            self.state = State::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOR: Words = Words { n: 1, w: 25, flick: 15 };
    const ROBERTA: Words = Words { n: 3, w: 10, flick: 15 };

    fn run(g: &mut Gesture, w: &Words, path: &[(f32, f32)]) {
        for &(f, s) in path {
            g.tick(f, s, w);
        }
    }

    #[test]
    fn flick_sets_mode_1_for_15_ticks() {
        let mut g = Gesture::default();
        run(&mut g, &TOR, &[(-50.0, 0.0), (50.0, 0.0)]);
        let modes: Vec<u8> = (0..17).map(|_| {
            g.tick(0.0, 0.0, &TOR);
            g.mode
        }).collect();
        assert_eq!(modes.iter().filter(|&&m| m == 1).count(), 15);
        assert_eq!(*modes.last().unwrap(), 0);
    }

    #[test]
    fn clockwise_needs_three_quadrants_counter_clockwise_by_n() {
        // Clockwise: up-left (1), up-right (0), then down (3): two CW steps.
        let mut g = Gesture::default();
        run(&mut g, &TOR, &[(50.0, 50.0), (50.0, -50.0), (-50.0, -50.0)]);
        g.tick(-50.0, -50.0, &TOR);
        assert_eq!(g.mode, 2);
        // Tor counter-clockwise: up-right (0), up-left (1), down-left (2): 1 + N = 2 steps.
        let mut g = Gesture::default();
        run(&mut g, &TOR, &[(50.0, -50.0), (50.0, 50.0), (-50.0, 50.0)]);
        g.tick(-50.0, 50.0, &TOR);
        assert_eq!(g.mode, 3);
        // Roberta needs 4 steps: three quadrants are not enough.
        let mut g = Gesture::default();
        run(&mut g, &ROBERTA, &[(50.0, -50.0), (50.0, 50.0), (-50.0, 50.0), (-50.0, 50.0)]);
        assert_eq!(g.mode, 0);
        // A centred stick returns to Idle; the busy reset keeps M.
        let mut g = Gesture::default();
        run(&mut g, &TOR, &[(50.0, 50.0), (50.0, -50.0), (-50.0, -50.0), (-50.0, -50.0)]);
        g.reset();
        assert_eq!(g.mode, 2);
    }
}
