//! Ladders (SPEC-0033 Behavior 13, state 41, and Behavior 22, the ladder grab test): the grab
//! test over the type-10 ladder records (type -3), the mount, the rung steps by clip, climbing
//! off the top and down, and the jump off. The rung clips' root motion moves the hero.
//!
//! Not covered: the grab cooldown's other users, the record flag of the climb off the top,
//! Roberta's 0.05 step at the top.

use crate::hero::Vec3;

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A plane value: the four-lane dot product with the point's fourth lane 1.
fn plane(q: [f32; 4], p: Vec3) -> f32 {
    q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
}

/// A ladder record: the face plane n, the side axis U and up axis V (with offsets) and the
/// lengths along them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rail {
    pub normal: [f32; 4],
    pub u: [f32; 4],
    pub v: [f32; 4],
    pub lu: f32,
    pub lv: f32,
}

impl Rail {
    fn n3(&self) -> Vec3 {
        [self.normal[0], self.normal[1], self.normal[2]]
    }
}

/// The hero radius and height the grab test and the top use.
pub const RADIUS: f32 = 1.175;
pub const HEIGHT: f32 = 3.5;
/// The jump off: horizontal velocity -0.5 times the facing (0x2DD858), vertical +2.0
/// (0x2DD85C), and the grab cooldown of 30 ticks.
pub const JUMP_BACK: f32 = 0.5;
pub const JUMP_UP: f32 = 2.0;
pub const COOLDOWN: u32 = 30;

/// The ladder grab test (Behavior 22): the first record whose face the hero and the stick (or
/// the facing with the stick centred) both point into (dot below -0.8), with the hero within
/// -r to 1.25 r of the plane, 0 to LU along U and -1.0 to LV - (H + 1.25) along V (LV from
/// the ledge hang), and the stick pushed above 0.1 (not needed from the ledge hang).
pub fn grab(rails: &[Rail], hero: Vec3, facing: Vec3, stick: Vec3, speed: f32, from_hang: bool) -> Option<usize> {
    let u = if speed > 0.0 { stick } else { facing };
    rails.iter().position(|r| {
        let n = r.n3();
        let d = plane(r.normal, hero);
        let x = plane(r.u, hero);
        let y = plane(r.v, hero);
        let top = if from_hang { r.lv } else { r.lv - (HEIGHT + 1.25) };
        dot(facing, n) < -0.8 && dot(u, n) < -0.8 && -RADIUS < d && d < 1.25 * RADIUS && (0.0..=r.lu).contains(&x) && (-1.0..=top).contains(&y) && (from_hang || speed > 0.1)
    })
}

/// What the ladder asks of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    pub request: Option<u32>,
    /// The facing (minus the ladder normal, at rate 1.0).
    pub facing: Vec3,
    /// The mount's position move (each axis at most 0.1 a tick).
    pub place: Option<Vec3>,
    /// Leave the ladder: to state 1 (`jump` false) or a jump off (true).
    pub leave: Option<bool>,
    /// A rung cue (140).
    pub cue: bool,
}

/// State 41's sub-mode (the shared word +0x9BC) and the ladder held.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ladder {
    pub rail: usize,
    pub sub: u8,
    mount_clip: bool,
}

impl Ladder {
    pub fn new(rail: usize) -> Self {
        Self { rail, sub: 1, mount_clip: false }
    }

    /// One tick. `forward` is the stick's forward value (positive pushed up), `jump` the Cross
    /// press edge, `gap` the ground gap.
    pub fn tick(&mut self, rails: &[Rail], hero: Vec3, forward: f32, jump: bool, gap: f32, clip: (u32, f32)) -> Step {
        let Some(r) = rails.get(self.rail).copied() else { return Step { leave: Some(false), ..Step::default() } };
        let n = r.n3();
        let mut step = Step { facing: n.map(|c| -c), ..Step::default() };
        if jump && self.sub != 8 {
            self.sub = 0;
            step.leave = Some(true);
            return step;
        }
        let y = plane(r.v, hero);
        let (action, progress) = clip;
        match self.sub {
            0 => step.leave = Some(false),
            1 => {
                // The mount: 0.9 out from the face, 0.8 along U, up to the nearest rung (0.45 +
                // 1.25 k, the first with the height below it plus 0.6), each at most 0.1 a tick.
                if self.mount_clip {
                    step.request = Some(120);
                    if action == 120 && progress >= 0.99 {
                        self.sub = 2;
                        self.mount_clip = false;
                        step.request = Some(93);
                    }
                    return step;
                }
                let k = ((y - 0.6 - 0.45) / 1.25).floor().max(-1.0) + 1.0;
                let rung = 0.45 + 1.25 * k;
                let targets = [(r.normal, 0.9), (r.u, 0.8), (r.v, rung)];
                let mut p = hero;
                let mut aligned = true;
                for (q, t) in targets {
                    let now = plane(q, p);
                    let delta = (t - now).clamp(-0.1, 0.1);
                    if (t - now).abs() > 1e-4 {
                        aligned &= (t - now).abs() <= 0.1;
                        for (c, a) in p.iter_mut().zip([q[0], q[1], q[2]]) {
                            *c += delta * a;
                        }
                    }
                }
                step.place = Some(p);
                if aligned {
                    if gap <= 1.5 {
                        self.sub = 2;
                        step.request = Some(93);
                    } else {
                        // Mounting from above: 120 with cue 140 until it ends.
                        self.mount_clip = true;
                        step.request = Some(120);
                        step.cue = true;
                    }
                }
            }
            2 | 3 => {
                let a = self.sub == 2;
                step.request = Some(if a { 93 } else { 92 });
                if forward > 0.0 {
                    if y >= r.lv - HEIGHT - if a { 1.3 } else { 1.27 } {
                        self.sub = 8;
                        step.request = Some(59);
                    } else {
                        self.sub = if a { 4 } else { 5 };
                        step.request = Some(if a { 91 } else { 90 });
                    }
                } else if forward < 0.0 {
                    if y <= 0.5 || gap < 0.5 {
                        self.sub = 0;
                        step.leave = Some(false);
                    } else {
                        self.sub = if a { 6 } else { 7 };
                        step.request = Some(if a { 95 } else { 94 });
                    }
                }
            }
            4..=7 => {
                let (clip_id, next, req) = match self.sub {
                    4 => (91, 3, 92),
                    5 => (90, 2, 93),
                    6 => (95, 3, 92),
                    _ => (94, 2, 93),
                };
                step.request = Some(clip_id);
                if action == clip_id && progress >= 0.99 {
                    self.sub = next;
                    step.request = Some(req);
                    step.cue = true;
                }
            }
            _ => {
                step.request = Some(135);
                if action == 135 && progress >= 0.99 {
                    self.sub = 0;
                    step.leave = Some(false);
                }
            }
        }
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ladder facing +Z (normal +Z, the face at z = 0), U along +X from x = 0, V up from y = 0,
    /// 1.6 wide and 8 high.
    fn rail() -> Rail {
        Rail { normal: [0.0, 0.0, 1.0, 0.0], u: [1.0, 0.0, 0.0, 0.0], v: [0.0, 1.0, 0.0, 0.0], lu: 1.6, lv: 8.0 }
    }

    #[test]
    fn grab_gates() {
        let r = [rail()];
        let into = [0.0, 0.0, -1.0];
        assert_eq!(grab(&r, [0.8, 0.0, 1.0], into, into, 1.0, false), Some(0));
        // Facing away, too far out, past the side, too high, or the stick centred: no grab.
        assert_eq!(grab(&r, [0.8, 0.0, 1.0], [0.0, 0.0, 1.0], into, 1.0, false), None);
        assert_eq!(grab(&r, [0.8, 0.0, 1.5], into, into, 1.0, false), None);
        assert_eq!(grab(&r, [1.7, 0.0, 1.0], into, into, 1.0, false), None);
        assert_eq!(grab(&r, [0.8, 3.4, 1.0], into, into, 1.0, false), None);
        assert_eq!(grab(&r, [0.8, 0.0, 1.0], into, into, 0.0, false), None);
        // From the ledge hang the top is LV and no stick is needed.
        assert_eq!(grab(&r, [0.8, 3.4, 1.0], into, into, 0.0, true), Some(0));
    }

    #[test]
    fn mount_and_climb() {
        let r = [rail()];
        let mut l = Ladder::new(0);
        // The mount moves each axis by at most 0.1 toward (0.8 along U, 0.9 out, rung 0.45).
        let mut p = [0.5, 0.0, 1.0];
        let mut s = Step::default();
        for _ in 0..10 {
            s = l.tick(&r, p, 0.0, false, 0.0, (0, 0.0));
            if let Some(q) = s.place {
                p = q;
            }
            if l.sub == 2 {
                break;
            }
        }
        assert_eq!((l.sub, s.request), (2, Some(93)));
        assert!((p[0] - 0.8).abs() < 1e-4 && (p[1] - 0.45).abs() < 1e-4 && (p[2] - 0.9).abs() < 1e-4, "{p:?}");
        // Stick forward: rung step 91, then 92 at 0.99 with the cue.
        assert_eq!(l.tick(&r, p, 1.0, false, 0.0, (93, 0.5)).request, Some(91));
        let s = l.tick(&r, p, 1.0, false, 0.0, (91, 0.99));
        assert_eq!((l.sub, s.request, s.cue), (3, Some(92), true));
        // Near the top (y >= 8 - 3.5 - 1.27): climb off with 59, then 135, then state 1.
        let s = l.tick(&r, [0.8, 3.3, 0.9], 1.0, false, 3.0, (92, 0.5));
        assert_eq!((l.sub, s.request), (8, Some(59)));
        assert_eq!(l.tick(&r, [0.8, 3.3, 0.9], 0.0, false, 3.0, (135, 0.99)).leave, Some(false));
        // Down at the bottom steps off; Cross jumps off.
        let mut l = Ladder::new(0);
        l.sub = 2;
        assert_eq!(l.tick(&r, [0.8, 0.45, 0.9], -1.0, false, 0.0, (93, 0.5)).leave, Some(false));
        let mut l = Ladder::new(0);
        l.sub = 3;
        assert_eq!(l.tick(&r, [0.8, 2.0, 0.9], 0.0, true, 2.0, (92, 0.5)).leave, Some(true));
    }
}
