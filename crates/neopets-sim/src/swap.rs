//! Hero switching (SPEC-0053): the swap mode lock and rule M, the request with its refusal
//! rules, the L1 and R1 chord, the switch-ready gate and the swap task, which the host runs one
//! tick at a time and which tells the host what to do at each step.
//!
//! Not covered: the one-shot states 0x33 of the outgoing and incoming heroes (their animation
//! is not given; the task does not wait on them), the sound settle, the HUD portrait swing, and
//! mode 4's own-spot placement (mode 4 has no writer).

/// Rule M (boot, new game and start-point loads): flag 220 clear gives mode 0; 220 set and 248
/// clear gives 1; both give 2.
pub fn rule_m(f220: bool, f248: bool) -> u32 {
    match (f220, f248) {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    }
}

/// Whether the mode allows a request while hero `selected` is selected (0 Tor, 1 Roberta).
pub fn allowed(mode: u32, selected: usize) -> bool {
    match mode {
        3 | 5 => false,
        0 => selected != 0,
        1 => selected != 1,
        _ => true,
    }
}

/// The chord: one of L1 and R1 newly pressed with the other held, or both newly pressed.
pub fn chord(l1_held: bool, l1_new: bool, r1_held: bool, r1_new: bool) -> bool {
    (l1_new && (r1_held || r1_new)) || (r1_new && l1_held)
}

/// The switch-ready gate's inputs for the selected hero (SPEC-0053 3.12 and 3.13).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ready {
    pub dialogue: bool,
    pub quick_open: bool,
    /// The game-state code is 0 (no swap in progress, no box or picker).
    pub code_zero: bool,
    pub state: u8,
    /// The height above the ground, and the grounded word.
    pub height: f32,
    pub grounded: bool,
    /// The stick speed value v.
    pub v: f32,
    pub stunned: bool,
    /// The screen effect is moving (a fade or bar sweep is running).
    pub fade_moving: bool,
}

/// The switch-ready gate (3.12): state 1 or 20, on the ground, the stick at rest, not stunned,
/// no box, the quick panel closed, game-state code 0 and no fade running.
pub fn ready(r: &Ready) -> bool {
    !r.dialogue
        && !r.quick_open
        && r.code_zero
        && matches!(r.state, 1 | 20)
        && (r.height < 0.15 || r.grounded)
        && r.v == 0.0
        && !r.stunned
        && !r.fade_moving
}

/// What the host does at a task step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Reset the hero attack pools; with the first flag, sound 0xB0 and the first swap effect
    /// (4025, 0xFB9) at the outgoing hero; with the second, fade to white over 37 ticks.
    Start { flash: bool, effects: bool },
    /// Exchange the heroes: bind the incoming hero at the outgoing hero's place, select it,
    /// clear its status timers and empty its target list.
    Exchange,
    /// The end flags (step 10): with the first flag, the second effect stopped (it never
    /// started) and the third (4027, 0xFBB) started; with the second, fade back from white over
    /// 8 ticks.
    EndFlags { flash: bool, effects: bool },
    /// Set the incoming hero's state to the carried state; with the extra flag, the six HUD
    /// visibility timers go to 240.
    Carry { state: u8, extra: bool },
    /// The in-progress flag clears.
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Start,
    /// Waiting at least three ticks and until the fade has reached its target.
    Settle(u32),
    Exchange,
    FadeIn,
    Carry,
    Hold(u32),
}

/// The swap task (SPEC-0053 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Task {
    pub target: usize,
    pub extra: bool,
    pub effects: bool,
    pub flash: bool,
    carried: u8,
    phase: Phase,
}

/// The hold: trunc(0.7 x 60) with the console's truncating multiply, 41 ticks.
pub fn hold_ticks() -> u32 {
    ps2_float::fpu::mul(0.7, 60.0) as u32
}

impl Task {
    /// Accept a request: the carried state is the outgoing state, or 1 when the outgoing hero
    /// is dead or in state 0x12 or 0x13.
    pub fn new(target: usize, extra: bool, effects: bool, flash: bool, outgoing_state: u8, outgoing_dead: bool) -> Self {
        let carried = if outgoing_dead || matches!(outgoing_state, 0x12 | 0x13 | 16) { 1 } else { outgoing_state };
        Self { target, extra, effects, flash, carried, phase: Phase::Start }
    }

    /// One tick: `fade_done` is the fade alpha at its target. Returns the step the host runs.
    pub fn tick(&mut self, fade_done: bool) -> Option<Step> {
        match self.phase {
            Phase::Start => {
                self.phase = Phase::Settle(0);
                Some(Step::Start { flash: self.flash, effects: self.effects })
            }
            Phase::Settle(n) => {
                if n + 1 >= 3 && fade_done {
                    self.phase = Phase::Exchange;
                } else {
                    self.phase = Phase::Settle(n + 1);
                }
                None
            }
            Phase::Exchange => {
                self.phase = if self.flash || self.effects { Phase::FadeIn } else { Phase::Carry };
                Some(Step::Exchange)
            }
            Phase::FadeIn => {
                self.phase = Phase::Carry;
                Some(Step::EndFlags { flash: self.flash, effects: self.effects })
            }
            Phase::Carry => {
                self.phase = Phase::Hold(0);
                Some(Step::Carry { state: self.carried, extra: self.extra })
            }
            Phase::Hold(n) => {
                if n + 1 >= hold_ticks() {
                    return Some(Step::Done);
                }
                self.phase = Phase::Hold(n + 1);
                None
            }
        }
    }
}

/// The act-start setup by stage (3.7): the swap mode, the hero to select (None: unchanged; the
/// bool is whether it is a normal switch request), the story flag and the heal mask (bit 1 Tor,
/// bit 2 Roberta).
pub fn act_start(stage: i32) -> (u32, Option<(usize, bool)>, Option<u16>, u8) {
    match stage {
        1 => (0, Some((0, true)), None, 0),
        2 => (1, Some((1, false)), Some(220), 2),
        3 => (2, Some((0, false)), Some(248), 3),
        4 => (2, None, Some(332), 3),
        _ => (2, None, None, 3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_and_rule_m() {
        assert_eq!((rule_m(false, true), rule_m(true, false), rule_m(true, true)), (0, 1, 2));
        assert!(!allowed(0, 0) && allowed(0, 1) && !allowed(1, 1) && allowed(1, 0));
        assert!(allowed(2, 0) && allowed(4, 1) && !allowed(3, 0) && !allowed(5, 1));
    }

    #[test]
    fn chord_reading() {
        assert!(chord(true, false, true, true) && chord(true, true, true, false) && chord(true, true, true, true));
        assert!(!chord(true, false, true, false), "both held, no new press");
        assert!(!chord(false, false, true, true), "R1 alone");
    }

    #[test]
    fn ready_gate() {
        let base = Ready { code_zero: true, state: 1, ..Ready::default() };
        assert!(ready(&base));
        assert!(ready(&Ready { state: 20, ..base }));
        assert!(!ready(&Ready { state: 3, ..base }));
        assert!(!ready(&Ready { v: 0.01, ..base }));
        assert!(!ready(&Ready { height: 0.2, ..base }));
        assert!(ready(&Ready { height: 0.2, grounded: true, ..base }));
        assert!(!ready(&Ready { stunned: true, ..base }));
        assert!(!ready(&Ready { fade_moving: true, ..base }));
        assert!(!ready(&Ready { code_zero: false, ..base }));
    }

    #[test]
    fn task_order_and_hold() {
        // The hold is 41 ticks (0.7 x 60 truncated by the console multiply).
        assert_eq!(hold_ticks(), 41);
        // A dead outgoing hero carries state 1.
        let mut t = Task::new(1, true, true, true, 16, true);
        assert_eq!(t.tick(true), Some(Step::Start { flash: true, effects: true }));
        let mut steps = Vec::new();
        for _ in 0..80 {
            if let Some(s) = t.tick(true) {
                steps.push(s);
                if s == Step::Done {
                    break;
                }
            }
        }
        assert_eq!(steps, vec![Step::Exchange, Step::EndFlags { flash: true, effects: true }, Step::Carry { state: 1, extra: true }, Step::Done]);
        // Without the flash the hold is exactly 41 ticks after the carry.
        let mut t = Task::new(0, false, false, false, 3, false);
        t.tick(true);
        let mut n = 0;
        let mut after_carry = None;
        loop {
            n += 1;
            match t.tick(true) {
                Some(Step::Carry { state, .. }) => {
                    assert_eq!(state, 3);
                    after_carry = Some(n);
                }
                Some(Step::Done) => break,
                _ => {}
            }
        }
        assert_eq!(n - after_carry.unwrap(), 41);
    }

    #[test]
    fn act_start_rows() {
        assert_eq!(act_start(2), (1, Some((1, false)), Some(220), 2));
        assert_eq!(act_start(3).0, 2);
        assert_eq!(act_start(9), (2, None, None, 3));
    }
}
