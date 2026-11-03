//! The Endless Staircase's looping-door puzzle (SPEC-0028 Pack 1299 looping-door puzzle; pack 1299, pads of class 500
//! and 501): the state, the state table's move, the display mask, the object offset and the spin rates.

/// The state table's address and length (type 0 needs a crossing to pad A, type 1 to pad B).
pub const TYPE_TABLE: u32 = 0x002E_B410;
pub const TYPE_WORDS: usize = 7;
/// The spin rates of placed objects 0 to 5, degrees per second.
pub const SPIN: [f32; 6] = [15.0, -15.0, 22.0, -18.0, 10.0, 18.0];
/// The display mask at build.
pub const START_MASK: u32 = 0x80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pad {
    A,
    B,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Puzzle {
    types: Vec<u32>,
    pub state: usize,
    pub complete: bool,
    pub accumulator: f32,
    pub mask: u32,
}

impl Puzzle {
    pub fn new(types: Vec<u32>) -> Self {
        Self { types, state: 0, complete: false, accumulator: 0.0, mask: START_MASK }
    }

    /// No teleport past state 5, at state 0 to pad A (the way back out), or at state 5 to pad B (the way on).
    pub fn teleports(&self, to: Pad) -> bool {
        !(self.state > 5 || (self.state == 0 && to == Pad::A) || (self.state == 5 && to == Pad::B))
    }

    fn word(&self, s: usize) -> u32 {
        self.types.get(s).copied().unwrap_or(0)
    }

    /// A teleporting crossing: the state moves (a match goes up, a miss comes down), the mask follows the new state
    /// and the vertical change joins the accumulator.
    pub fn cross(&mut self, to: Pad, dy: f32) {
        let needs = if self.word(self.state) == 0 { Pad::A } else { Pad::B };
        if to == needs {
            self.state += 1;
            self.complete = self.state > 5;
        } else {
            self.state = self.state.saturating_sub(1);
        }
        let i = if self.word(self.state) == 0 { self.state + 2 } else { self.state + 8 };
        self.mask = 1 << (i - 1);
        self.accumulator += dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disc_like() -> Puzzle {
        Puzzle::new(vec![1, 0, 1, 1, 0, 1, 0])
    }

    /// SPEC-0028 test vector: B, A, B, B, A give states 1 to 5 and these masks.
    #[test]
    fn the_solution_and_its_masks() {
        let mut p = disc_like();
        assert_eq!(p.mask, 0x80);
        for (to, s, mask) in [(Pad::B, 1, 0x4), (Pad::A, 2, 0x200), (Pad::B, 3, 0x400), (Pad::B, 4, 0x20), (Pad::A, 5, 0x1000)] {
            assert!(p.teleports(to));
            p.cross(to, 18.0);
            assert_eq!((p.state, p.mask), (s, mask));
        }
        assert!(!p.complete);
        assert_eq!(p.accumulator, 90.0);
    }

    #[test]
    fn the_ends_do_not_teleport() {
        let mut p = disc_like();
        assert!(!p.teleports(Pad::A));
        assert!(p.teleports(Pad::B));
        p.state = 5;
        assert!(!p.teleports(Pad::B));
        assert!(p.teleports(Pad::A));
        p.cross(Pad::A, -18.0);
        assert_eq!(p.state, 4);
    }

    #[test]
    fn a_wrong_crossing_steps_back_and_never_below_zero() {
        let mut p = disc_like();
        p.cross(Pad::B, 18.0);
        assert_eq!(p.state, 1);
        // State 1 needs A: B misses.
        p.cross(Pad::B, 18.0);
        assert_eq!(p.state, 0);
        // State 0 needs B: a miss stays at 0.
        p.cross(Pad::A, -18.0);
        assert_eq!(p.state, 0);
    }
}
