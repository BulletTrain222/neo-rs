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
//! (side offset below 0.001) and the +0x270 lift while 113 plays (its role is not established).
//! The leap to a parallel line (modes 2 and 3), request 113 with Cross held, and (in the host)
//! cue 141 and the touched-flag exit to state 48 are built.

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

/// The hang on line `line` (mode 0 attach, 1 slide, 2 launch, 3 flight).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hang {
    pub line: usize,
    pub mode: u8,
    /// The leap's target line, side word (0 the -B1 side, 1 the +B1 side), side velocity per
    /// tick and vertical velocity per tick.
    pub target: Option<usize>,
    pub side: u8,
    side_vel: Vec3,
    vy: f32,
    /// Set by the host before each tick: Cross held (request 113 in the slide) and the hero's
    /// current clip (the attach keeps a playing 115 or 117).
    pub cross: bool,
    pub playing: u32,
}

/// The leap search (Behavior 9, Search): the nearest other line by side distance whose axis is
/// within 0.75 (dot) of the current one's, on the chosen side of the hero (side 0: -B1, 1: +B1),
/// whose end B' is ahead along the facing `f`, whose span holds the hero, within 20.0 sideways
/// and 8.0 in height of its start.
pub fn search(lines: &[Line], current: usize, p: Vec3, f: Vec3, b1: Vec3, side: u8) -> Option<usize> {
    let c = lines.get(current)?;
    let mut best: Option<(usize, f32)> = None;
    for (k, l) in lines.iter().enumerate() {
        if k == current || dot(l.u, c.u) <= 0.75 {
            continue;
        }
        let off = [l.a[0] - p[0], 0.0, l.a[2] - p[2]];
        let s = dot(off, b1);
        if (side == 0 && s >= 0.0) || (side == 1 && s <= 0.0) {
            continue;
        }
        let b = l.b();
        if dot([b[0] - p[0], b[1] - p[1], b[2] - p[2]], f) <= 0.0 {
            continue;
        }
        let t = dot([p[0] - l.a[0], p[1] - l.a[1], p[2] - l.a[2]], l.u);
        if !(t > 0.0 && t < l.length) || s.abs() >= 20.0 || (l.a[1] - p[1]).abs() >= 8.0 {
            continue;
        }
        if best.is_none_or(|(_, d)| s.abs() < d) {
            best = Some((k, s.abs()));
        }
    }
    best.map(|b| b.0)
}

impl Hang {
    pub fn new(line: usize) -> Self {
        Self { line, mode: 0, target: None, side: 0, side_vel: [0.0; 3], vy: 0.0, cross: false, playing: 0 }
    }

    /// The search in mode 1 (Cross held with the side stick): a found line becomes the pending
    /// target and the launch (mode 2) runs this tick. Returns the launch request (114 or 116).
    pub fn seek(&mut self, lines: &[Line], p: Vec3, f: Vec3, b1: Vec3, side: u8) -> Option<u32> {
        if self.mode != 1 {
            return None;
        }
        let k = search(lines, self.line, p, f, b1, side)?;
        let a = lines[k].a;
        let s = dot([a[0] - p[0], 0.0, a[2] - p[2]], b1);
        self.target = Some(k);
        self.side = side;
        self.side_vel = b1.map(|c| c * s / 40.0);
        // 40 x 40 x 0.5 x 0.02 / 40.
        self.vy = 0.4;
        self.mode = 2;
        Some(if side == 0 { 114 } else { 116 })
    }

    /// One tick at hero position `p` with the ground gap `gap` and the Square press `square`.
    pub fn tick(&mut self, lines: &[Line], p: Vec3, gap: f32, square: bool) -> Step {
        let Some(l) = lines.get(self.line) else { return Step { let_go: true, ..Step::default() } };
        let hang = DROP + l.radius;
        let fall = if self.side == 0 { 115 } else { 117 };
        match self.mode {
            2 => {
                // The launch: one slide step; the flight starts next tick.
                self.mode = 3;
                return Step { place: Some(std::array::from_fn(|k| p[k] + STEP * l.u[k])), request: if self.side == 0 { 114 } else { 116 }, let_go: false };
            }
            3 => {
                let Some(t) = self.target.and_then(|t| lines.get(t)) else { return Step { let_go: true, ..Step::default() } };
                let mut q: Vec3 = std::array::from_fn(|k| p[k] + STEP * l.u[k] + self.side_vel[k]);
                q[1] += self.vy;
                self.vy -= 0.02;
                // The side offset still to close, along the side axis of the leap.
                let b1 = self.side_vel;
                let bl = dot(b1, b1).sqrt().max(1e-6);
                let gapside = dot([t.a[0] - q[0], 0.0, t.a[2] - q[2]], b1.map(|c| c / bl));
                let request = if self.vy < -0.25 { fall } else if self.side == 0 { 114 } else { 116 };
                if gapside.abs() <= 0.1 || gapside < 0.0 {
                    self.line = self.target.take().unwrap_or(self.line);
                    self.mode = 0;
                    return Step { place: Some(q), request: fall, let_go: false };
                }
                return Step { place: Some(q), request, let_go: false };
            }
            _ => {}
        }
        match self.mode {
            0 => {
                let mut q = l.foot(p);
                q[1] -= hang;
                self.mode = 1;
                // Request 112 unless a leap's falling clip (115 or 117) is playing.
                let request = if matches!(self.playing, 115 | 117) { self.playing } else { 112 };
                Step { place: Some(q), request, let_go: false }
            }
            _ => {
                let q: Vec3 = std::array::from_fn(|k| p[k] + STEP * l.u[k]);
                // The line end: still in the air and |P - A| at most L.
                let from_a = std::array::from_fn::<f32, 3, _>(|k| q[k] - l.a[k]);
                let within = dot(from_a, from_a).sqrt() <= l.length;
                if square || gap <= 0.0 || !within {
                    return Step { place: None, request: 0, let_go: true };
                }
                Step { place: Some(q), request: if self.cross { 113 } else { 112 }, let_go: false }
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
        // Cross held: request 113.
        h.cross = true;
        assert_eq!(h.tick(&ls, p, 5.0, false).request, 113);
        h.cross = false;
        // Square, ground contact and the line end let go.
        assert!(h.tick(&ls, p, 5.0, true).let_go);
        assert!(h.tick(&ls, p, 0.0, false).let_go);
        assert!(h.tick(&ls, [99.8, -0.45, 0.0], 5.0, false).let_go);
    }

    #[test]
    fn leap_to_a_parallel_line() {
        // Two level lines along +x, 15 apart in z; the hero on line 0 facing +x (B1 = +z... the
        // side axis up x f = (0, 0, -1) for f = +x), seeking on the +B1 side finds nothing, on
        // the -B1 side (z > 0) the second line.
        let ls = [line(), Line { a: [0.0, 3.0, 15.0], ..line() }];
        let mut h = Hang::new(0);
        let p0 = h.tick(&ls, [10.0, 0.0, 0.0], 5.0, false).place.unwrap();
        let (f, b1) = ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        assert_eq!(search(&ls, 0, p0, f, b1, 1), None);
        assert_eq!(h.seek(&ls, p0, f, b1, 0), Some(114));
        // The launch tick, then 40 flight ticks close the 15-unit gap; the hero then attaches.
        let mut p = h.tick(&ls, p0, 5.0, false).place.unwrap();
        let mut n = 0;
        while h.mode == 3 && n < 60 {
            p = h.tick(&ls, p, 5.0, false).place.unwrap();
            n += 1;
        }
        assert_eq!((h.line, h.mode), (1, 0));
        assert!((38..=41).contains(&n), "{n}");
        let q = h.tick(&ls, p, 5.0, false).place.unwrap();
        assert!((q[2] - 15.0).abs() < 1e-4 && (q[1] - (3.0 - 3.45)).abs() < 1e-4);
    }
}
