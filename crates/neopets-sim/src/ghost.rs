//! Ghosts (SPEC-0029 Behavior 33): any region character whose monster row has the Ghost Flag cycles
//! between a solid phase and a ghost phase, fading over 60 ticks. A hit in the ghost phase is absorbed
//! and reveals the character for a while. It is targetable only once its solid fade-in has ended,
//! while it attacks, or while it is revealed.

use crate::rng::Rng;

/// The row's ghost words (+0x13C to +0x158).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Words {
    pub solid: (i32, i32),
    pub ghost: (i32, i32),
    pub opacity: (f32, f32),
    pub reveal: (i32, i32),
}

const FADE: i32 = 60;

#[derive(Clone, Debug, PartialEq)]
pub struct Ghost {
    w: Words,
    solid: bool,
    phase: i32,
    fade: i32,
    length: i32,
    revealed: bool,
    reveal: i32,
    /// The hero's auto-target and lock-on may pick it this tick.
    pub targetable: bool,
    pub opacity: f32,
}

impl Ghost {
    pub fn new(w: Words) -> Self {
        Self { w, solid: true, phase: -1, fade: -1, length: 0, revealed: false, reveal: 0, targetable: false, opacity: w.opacity.1 }
    }

    fn rising(&self) -> f32 {
        let (lo, hi) = self.w.opacity;
        if self.length > 0 && self.fade > 0 { hi - (self.fade as f32 / self.length as f32) * (hi - lo) } else { hi }
    }

    fn falling(&self) -> f32 {
        let (lo, hi) = self.w.opacity;
        if self.length > 0 && self.fade > 0 { lo + (self.fade as f32 / self.length as f32) * (hi - lo) } else { lo }
    }

    fn fade_down(&mut self) {
        if self.fade > 0 {
            self.fade -= 1;
        }
    }

    /// One tick, before the character's status and AI step. `attacking`: agro with the sequencer in
    /// states 0 to 3 or 14. `hidden`: retreated in state 14, or emerging.
    pub fn tick(&mut self, attacking: bool, hidden: bool, rng: &mut Rng) {
        self.targetable = false;
        if self.revealed {
            self.targetable = true;
            if self.reveal < 1 {
                (self.solid, self.phase, self.fade, self.length, self.revealed) = (true, -1, 0, 0, false);
                self.opacity = self.w.opacity.1;
            } else {
                self.reveal -= 1;
                self.fade_down();
                self.opacity = self.rising();
            }
            return;
        }
        if attacking {
            self.targetable = true;
            self.fade_down();
            self.opacity = self.rising();
            self.solid = true;
            return;
        }
        if hidden {
            if self.fade < 1 {
                self.fade = 0;
                self.length = FADE;
            } else {
                self.fade -= 1;
            }
            self.opacity = self.falling();
            self.solid = false;
            return;
        }
        self.fade_down();
        if self.phase > 0 {
            self.phase -= 1;
        }
        let draw = |r: (i32, i32), rng: &mut Rng| rng.range(r.0.min(r.1), r.0.max(r.1));
        if self.fade < 1 && self.phase < 0 {
            self.phase = draw(if self.solid { self.w.solid } else { self.w.ghost }, rng);
        } else if self.phase == 0 {
            self.solid = !self.solid;
            (self.phase, self.length, self.fade) = (-1, FADE, FADE);
        }
        if self.solid {
            self.opacity = self.rising();
            self.targetable = self.fade <= 0;
        } else {
            self.opacity = self.falling();
        }
    }

    /// The hit rule, called by the take-hit: in the ghost phase, not revealed, the hit is absorbed
    /// (true) and reveals the character.
    pub fn absorb(&mut self, rng: &mut Rng) -> bool {
        if self.solid || self.revealed {
            return false;
        }
        self.revealed = true;
        self.reveal = rng.range(self.w.reveal.0.min(self.w.reveal.1), self.w.reveal.0.max(self.w.reveal.1));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Words = Words { solid: (5, 5), ghost: (10, 10), opacity: (0.2, 0.6), reveal: (3, 3) };

    #[test]
    fn solid_then_fades_to_ghost_and_back() {
        let mut r = Rng::seeded(1);
        let mut g = Ghost::new(W);
        g.tick(false, false, &mut r);
        assert!(g.targetable && g.solid, "solid and targetable from reset");
        assert!((g.opacity - 0.6).abs() < 1e-6);
        // The draw of 5 runs out: the fade to the ghost phase starts, no longer targetable.
        let mut n = 1;
        while g.solid {
            g.tick(false, false, &mut r);
            n += 1;
            assert!(n < 20);
        }
        assert_eq!(n, 6);
        assert!(!g.targetable);
        assert!((g.opacity - 0.6).abs() < 1e-6, "the fade starts from the top");
        for _ in 0..60 {
            g.tick(false, false, &mut r);
        }
        assert!((g.opacity - 0.2).abs() < 1e-6);
        assert!(!g.solid);
        // Fading back in: targetable only once the fade has ended.
        while !g.solid {
            g.tick(false, false, &mut r);
        }
        assert!(!g.targetable);
        for _ in 0..59 {
            g.tick(false, false, &mut r);
            assert!(!g.targetable);
        }
        g.tick(false, false, &mut r);
        assert!(g.targetable);
    }

    #[test]
    fn a_ghost_phase_hit_is_absorbed_and_reveals() {
        let mut r = Rng::seeded(2);
        let mut g = Ghost::new(W);
        assert!(!g.absorb(&mut r), "a solid character takes the hit");
        g.solid = false;
        assert!(g.absorb(&mut r));
        assert!(!g.absorb(&mut r), "a revealed character takes the hit");
        g.tick(false, false, &mut r);
        assert!(g.targetable);
        for _ in 0..3 {
            g.tick(false, false, &mut r);
        }
        assert!(g.solid && !g.revealed, "the reveal ends solid");
        assert!((g.opacity - 0.6).abs() < 1e-6);
    }

    #[test]
    fn an_attack_makes_it_solid_and_hiding_fades_it() {
        let mut r = Rng::seeded(3);
        let mut g = Ghost::new(W);
        g.solid = false;
        g.tick(true, false, &mut r);
        assert!(g.solid && g.targetable);
        g.tick(false, true, &mut r);
        assert!(!g.solid && !g.targetable);
        assert!((g.opacity - 0.2).abs() < 1e-6);
    }
}
