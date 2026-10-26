//! The prize wheel (SPEC-0059 section 2): the copy values, the light show, the new pattern, the
//! spin with its float32 rate, the result cell and the mask word.
//!
//! Not covered: the 758 and 1358 copies' triggers (their modules are not built) and the music
//! override.

/// One wheel copy's values (SPEC-0059 Layout, Prize wheel copies).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Copy {
    pub fee: i32,
    pub too_poor: u16,
    pub start_prompt: u16,
    pub go_on_prompts: [u16; 4],
    pub go_on_line: u16,
    pub cash_out: [u16; 4],
    pub five_win: u16,
    pub loss: [u16; 5],
    pub payouts: [i32; 5],
    /// Loss, cash-out and five-win flags.
    pub flags: Option<[u16; 3]>,
    /// Win and loss sounds.
    pub sounds: (i32, i32),
    /// The mask bit of cell i is 2^(i + shift).
    pub shift: u32,
}

/// The pack 1023 copy (trigger 36).
pub const VILLAGE: Copy = Copy {
    fee: 15,
    too_poor: 1042,
    start_prompt: 1026,
    go_on_prompts: [1027, 1028, 1029, 1030],
    go_on_line: 1032,
    cash_out: [1033, 1034, 1035, 1036],
    five_win: 1031,
    loss: [1037, 1038, 1039, 1040, 1041],
    payouts: [20, 40, 60, 80, 160],
    flags: Some([478, 479, 480]),
    sounds: (11, 10),
    shift: 2,
};

/// The pack 1358 copy (trigger 0, registered with flag 102).
pub const PALACE: Copy = Copy {
    fee: 100,
    too_poor: 109,
    start_prompt: 93,
    go_on_prompts: [94, 95, 96, 97],
    go_on_line: 99,
    cash_out: [100, 101, 102, 103],
    five_win: 98,
    loss: [104, 105, 106, 107, 108],
    payouts: [140, 180, 250, 400, 700],
    flags: None,
    sounds: (7, 6),
    shift: 0,
};

/// The pack 758 copy (trigger 3).
pub const MARKET: Copy = Copy {
    fee: 25,
    too_poor: 335,
    start_prompt: 319,
    go_on_prompts: [320, 321, 322, 323],
    go_on_line: 325,
    cash_out: [326, 327, 328, 329],
    five_win: 324,
    loss: [330, 331, 332, 333, 334],
    payouts: [45, 65, 105, 175, 325],
    flags: Some([536, 537, 538]),
    sounds: (9, 8),
    shift: 0,
};

/// The wheel copy of a pack (1358, 758, else 1023's).
pub fn copy_of(pack: u32) -> Copy {
    match pack {
        1358 => PALACE,
        758 => MARKET,
        _ => VILLAGE,
    }
}

/// The wheel's game words: stage G (0 new, 1 won, -1 lost), round r, angle A (degrees) and the
/// lit cells.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wheel {
    pub g: i32,
    pub r: u32,
    pub angle: f32,
    pub lit: [bool; 8],
}

/// The coins that pay n NP: the largest coin at most n first (139, 138, 137, 136, 135 for 100,
/// 25, 10, 5 and 1 NP; SPEC-0059 Common rules).
pub fn pay(mut n: i32) -> Vec<u16> {
    let mut out = Vec::new();
    for (id, v) in [(139u16, 100), (138, 25), (137, 10), (136, 5), (135, 1)] {
        while n >= v {
            out.push(id);
            n -= v;
        }
    }
    out
}

impl Wheel {
    /// The mask word M: a dark cell sets its bit.
    pub fn mask(&self, shift: u32) -> u32 {
        (0..8).filter(|&i| !self.lit[i]).fold(0, |m, i| m | 1 << (i as u32 + shift))
    }

    /// The light show at tick t (0 to 69): even cells lit for 5 ticks, then odd cells.
    pub fn light_show(&mut self, t: u32) {
        let even = (t / 5) % 2 == 0;
        self.lit = std::array::from_fn(|i| (i % 2 == 0) == even);
    }

    /// A new pattern: all lit, then three distinct cells dark, drawn by `draw` (Random(0, 7);
    /// a repeat is drawn again).
    pub fn new_pattern(&mut self, draw: &mut dyn FnMut() -> i32) {
        self.lit = [true; 8];
        let mut dark = Vec::new();
        while dark.len() < 3 {
            let c = draw().clamp(0, 7) as usize;
            if !dark.contains(&c) {
                dark.push(c);
                self.lit[c] = false;
            }
        }
    }

    /// The cell under the pointer: 7 - floor(A / 45).
    pub fn cell(&self) -> usize {
        7 - ((self.angle / 45.0) as usize).min(7)
    }

    /// The result: a win darkens the cell and sets G to 1; a loss sets G to -1. Returns the win.
    pub fn result(&mut self) -> bool {
        let c = self.cell();
        if self.lit[c] {
            self.lit[c] = false;
            self.g = 1;
            true
        } else {
            self.g = -1;
            false
        }
    }
}

/// A spin in progress: the turn rate and whether the stop press was seen.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spin {
    pub rate: f32,
    pub stopped: bool,
}

impl Spin {
    /// One tick: below the full rate the rate grows by 0.1 (float32); at it the stop prompt
    /// shows and `square` is tested; then A advances (wrapping above 360). Returns whether the
    /// stop prompt shows this tick. The spin ends after the tick the press was seen.
    pub fn tick(&mut self, wheel: &mut Wheel, square: bool) -> bool {
        let mut prompt = false;
        if self.rate < 9.0 {
            self.rate += 0.1;
        } else {
            prompt = true;
            if square {
                self.stopped = true;
            }
        }
        wheel.angle += self.rate;
        if wheel.angle > 360.0 {
            wheel.angle -= 360.0;
        }
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-0059: the full rate is 9.0999985 after 91 additions, reached on tick 91 with A
    /// advanced 418.60 degrees (unwrapped); the prompt first shows on tick 92.
    #[test]
    fn spin_rate_in_single_precision() {
        let mut w = Wheel::default();
        let mut s = Spin::default();
        let mut total = 0.0f32;
        let mut first_prompt = None;
        for tick in 1..=92 {
            let before = w.angle;
            if s.tick(&mut w, false) && first_prompt.is_none() {
                first_prompt = Some(tick);
            }
            let step = if w.angle < before { w.angle + 360.0 - before } else { w.angle - before };
            total += step;
            if tick == 91 {
                assert!((s.rate - 9.0999985).abs() < 1e-6, "{}", s.rate);
                assert!((total - 418.60).abs() < 0.05, "{total}");
            }
        }
        assert_eq!(first_prompt, Some(92));
    }

    #[test]
    fn cells_mask_pattern_and_pay() {
        let mut w = Wheel { angle: 10.0, ..Default::default() };
        assert_eq!(w.cell(), 7);
        w.angle = 350.0;
        assert_eq!(w.cell(), 0);
        let mut draws = [2, 2, 5, 7].into_iter();
        w.new_pattern(&mut || draws.next().unwrap());
        assert_eq!(w.lit.iter().filter(|&&l| l).count(), 5);
        assert_eq!(w.mask(2), (1 << 4) | (1 << 7) | (1 << 9));
        w.light_show(0);
        assert_eq!(w.lit, [true, false, true, false, true, false, true, false]);
        w.light_show(69);
        assert_eq!(w.lit, [false, true, false, true, false, true, false, true]);
        w.lit = [true; 8];
        w.angle = 350.0;
        assert!(w.result() && w.g == 1 && !w.lit[0]);
        assert!(!w.result() && w.g == -1);
        assert_eq!(pay(160), vec![139, 138, 138, 137]);
        assert_eq!(pay(20), vec![137, 137]);
    }
}
