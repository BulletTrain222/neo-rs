//! Grab lines (SPEC-0033 Behavior 9, state 28): a cylinder of the area's type-11 list whose class
//! word has bit 1 is a slide line from A (its base) to B = A + L u. A jump into the first half of
//! a line hangs the hero 3.25 + r below it, slides it 0.5 a tick toward B, and lets go at the
//! line end, on Square or on ground contact, leaving at the slide speed (u times 15.0) into air
//! phase 5.
//!
//! The reach probe is the spec's measured result rather than its three probe segments (whose
//! combination is Medium): a hero is caught when the line, taken as centred on A (so only its
//! first half, plus 0.5, counts), is ahead of A along the flattened axis, within 1.5 of the hero
//! sideways, and inside the height band for that side offset (1.5 to 4.0 above the hero up to
//! 1.0 sideways, 1.0 to 4.5 up to 1.25, -0.5 to 6.0 up to 1.5). Not built: the exact-plane case
//! (side offset below 0.001), the leap to a parallel line (modes 2 and 3; Cross with the side
//! stick), request 113 with Cross held, cue 141, and the touched-flag exit to state 48.

use crate::hero::Vec3;

pub const STATE: u8 = 28;
/// The slide step per tick and the hang drop below the line (word 0x2DD7B8).
pub const STEP: f32 = 0.5;
pub const DROP: f32 = 3.25;
/// The cooldown after the attach and the let-go (the frame-rate word).
pub const COOLDOWN: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    /// The start point A (the cylinder base).
    pub a: Vec3,
    /// The unit axis u.
    pub u: Vec3,
    pub length: f32,
    pub radius: f32,
    /// The entry's gate word (+0x1A): 0, or tested against the world mask.
    pub mask: u16,
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Line {
    pub fn b(&self) -> Vec3 {
        std::array::from_fn(|k| self.a[k] + self.length * self.u[k])
    }

    /// The measured reach (see the module note) for a hero at `p`.
    pub fn reaches(&self, p: Vec3) -> bool {
        let flat = [self.u[0], 0.0, self.u[2]];
        let fl = dot(flat, flat).sqrt();
        if fl < 1e-6 {
            return false;
        }
        let f = flat.map(|c| c / fl);
        let c = [p[0], p[1] + 2.75, p[2]];
        let d = [c[0] - self.a[0], 0.0, c[2] - self.a[2]];
        let along = dot(d, f);
        // Forward of A, and inside the tested half (the line cylinder centred on A, with the
        // hero cylinder's 1.5 radius).
        if along <= 0.0 || along >= 0.5 * self.length * fl + 0.5 {
            return false;
        }
        let side = (d[0] - along * f[0]).hypot(d[2] - along * f[2]);
        // The line's height at the hero's place along it, above the hero's position.
        let s = along / fl;
        let h = self.a[1] + s * self.u[1] - p[1];
        let band = if side <= 1.0 {
            (1.5, 4.0)
        } else if side <= 1.25 {
            (1.0, 4.5)
        } else if side <= 1.5 {
            (-0.5, 6.0)
        } else {
            return false;
        };
        h >= band.0 && h <= band.1
    }

    /// The point on the line nearest `p` (A + ((P - A) . u) u).
    pub fn foot(&self, p: Vec3) -> Vec3 {
        let t = dot(std::array::from_fn(|k| p[k] - self.a[k]), self.u);
        std::array::from_fn(|k| self.a[k] + t * self.u[k])
    }
}

/// What the hang asks of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    pub place: Option<Vec3>,
    pub request: u32,
    /// Let go: leave at u times 15.0 into air phase 5.
    pub let_go: bool,
}

/// The hang on line `line` (mode 0 attach, 1 slide).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hang {
    pub line: usize,
    pub mode: u8,
}

impl Hang {
    pub fn new(line: usize) -> Self {
        Self { line, mode: 0 }
    }

    /// One tick at hero position `p` with the ground gap `gap` and the Square press `square`.
    pub fn tick(&mut self, lines: &[Line], p: Vec3, gap: f32, square: bool) -> Step {
        let Some(l) = lines.get(self.line) else { return Step { let_go: true, ..Step::default() } };
        let hang = DROP + l.radius;
        match self.mode {
            0 => {
                let mut q = l.foot(p);
                q[1] -= hang;
                self.mode = 1;
                Step { place: Some(q), request: 112, let_go: false }
            }
            _ => {
                let q: Vec3 = std::array::from_fn(|k| p[k] + STEP * l.u[k]);
                // The line end: still in the air and |P - A| at most L.
                let from_a = std::array::from_fn::<f32, 3, _>(|k| q[k] - l.a[k]);
                let within = dot(from_a, from_a).sqrt() <= l.length;
                if square || gap <= 0.0 || !within {
                    return Step { place: None, request: 0, let_go: true };
                }
                Step { place: Some(q), request: 112, let_go: false }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A level line along +x from the origin, 100 long, radius 0.2, 3.0 up.
    fn line() -> Line {
        Line { a: [0.0, 3.0, 0.0], u: [1.0, 0.0, 0.0], length: 100.0, radius: 0.2, mask: 0 }
    }

    #[test]
    fn reach_bands() {
        let l = line();
        // On the first half, 0.5 to the side, the line 3.0 above: caught; past the middle: not.
        assert!(l.reaches([20.0, 0.0, 0.5]));
        assert!(!l.reaches([51.0, 0.0, 0.5]));
        assert!(!l.reaches([-1.0, 0.0, 0.5]));
        // 2.0 to the side: not; the line 1.0 above (below the 1.5 band at 0.5): not.
        assert!(!l.reaches([20.0, 0.0, 2.0]));
        assert!(!l.reaches([20.0, 2.0, 0.5]));
        // At 1.45 to the side the band is -0.5 to 6.0.
        assert!(l.reaches([20.0, 2.5, 1.45]));
    }

    #[test]
    fn attach_slide_and_let_go() {
        let ls = [line()];
        let mut h = Hang::new(0);
        let s = h.tick(&ls, [10.0, 0.5, 0.4], 5.0, false);
        let p = s.place.unwrap();
        assert!((p[0] - 10.0).abs() < 1e-5 && (p[1] - (3.0 - 3.45)).abs() < 1e-5 && p[2].abs() < 1e-5);
        assert_eq!(h.mode, 1);
        let s = h.tick(&ls, p, 5.0, false);
        assert!((s.place.unwrap()[0] - 10.5).abs() < 1e-5);
        // Square, ground contact and the line end let go.
        assert!(h.tick(&ls, p, 5.0, true).let_go);
        assert!(h.tick(&ls, p, 0.0, false).let_go);
        assert!(h.tick(&ls, [99.8, -0.45, 0.0], 5.0, false).let_go);
    }
}
