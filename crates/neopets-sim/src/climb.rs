//! Climb panels (SPEC-0033 Behavior 51, state 21, and Behavior 22A, the wall-panel grab): the
//! grab over the type-10 records of type -2, the approach (sub-mode 1), the free climb with its
//! panel hold and hand-offs (2), the straddles of stacked and side-by-side panels (13 to 16), the
//! top-edge hang and the climb over the top (17 and 7), the corners (8, 9, 11 and 12) and the
//! step off (0). The climb clips' root motion moves the hero.
//!
//! The ceiling grab and hang (18 and 4), the overhang paths (3, 5 and 6) and the straddles' other axis
//! are here too.
//!
//! Not covered: sub-mode 10 (the approach's branch with the hero's back to the panel, which the grab's
//! facing test refuses). Interpretations (the spec gives these as Medium or leaves them open): the
//! approach holds the hero on the panel and plays 121 to its end; a corner ends with the hero's position
//! along the new panel clamped into [r, LU - r].

use crate::hero::Vec3;

fn dot3(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A plane value: the four-lane dot product with the point's fourth lane 1.
fn plane(q: [f32; 4], p: Vec3) -> f32 {
    q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
}

fn v3(q: [f32; 4]) -> Vec3 {
    [q[0], q[1], q[2]]
}

/// A type-10 record (SPEC-0006 Type 10) with its neighbour links: 0 below, 1 the side at u = LU,
/// 2 above, 3 the side at u = 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panel {
    pub normal: [f32; 4],
    pub u: [f32; 4],
    pub v: [f32; 4],
    pub lu: f32,
    pub lv: f32,
    pub kind: i8,
    pub links: [Option<usize>; 4],
}

impl Panel {
    pub fn n(&self) -> Vec3 {
        v3(self.normal)
    }
}

/// The hero radius and height the climb uses.
pub const RADIUS: f32 = 1.175;
pub const HEIGHT: f32 = 3.5;

/// The wall-panel grab (Behavior 22A): the first type -2 record whose face the hero's facing and
/// stick direction both point into (dot below -0.8), with the hero within -r to 1.25 r of the
/// plane, a to LU - b along U (a = 1.0, or -1.0 with a link below; b = 1.0, or -1.0 with a link
/// on the u = 0 side) and -1.0 to LV - 1.0 along V, and the stick pushed above 0.5.
pub fn grab_wall(panels: &[Panel], hero: Vec3, facing: Vec3, stick: Vec3, speed: f32) -> Option<usize> {
    if speed <= 0.5 {
        return None;
    }
    panels.iter().position(|r| {
        let n = r.n();
        let a = if r.links[0].is_some() { -1.0 } else { 1.0 };
        let b = if r.links[3].is_some() { -1.0 } else { 1.0 };
        let (d, x, y) = (plane(r.normal, hero), plane(r.u, hero), plane(r.v, hero));
        r.kind == -2 && dot3(facing, n) < -0.8 && dot3(stick, n) < -0.8 && -RADIUS < d && d < 1.25 * RADIUS && a <= x && x <= r.lu - b && (-1.0..=r.lv - 1.0).contains(&y)
    })
}

/// The ceiling-panel grab (Behavior 22A; every air tick): the first record whose normal's vertical part is
/// below -0.99 with 0 < d < H + 1.0, a to LU - b along U and -1.0 to LV - 1.0 along V. No facing or stick test.
pub fn grab_ceiling(panels: &[Panel], hero: Vec3) -> Option<usize> {
    panels.iter().position(|r| {
        let a = if r.links[0].is_some() { -1.0 } else { 1.0 };
        let b = if r.links[3].is_some() { -1.0 } else { 1.0 };
        let (d, x, y) = (plane(r.normal, hero), plane(r.u, hero), plane(r.v, hero));
        r.normal[1] < -0.99 && 0.0 < d && d < HEIGHT + 1.0 && a <= x && x <= r.lu - b && -1.0 <= y && y <= r.lv - 1.0
    })
}

/// What the climb asks of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    pub request: Option<u32>,
    /// The clip rate (1.0 unless a move sets it).
    pub rate: f32,
    /// The steering target (minus the target's normal) and its rate.
    pub facing: Option<(Vec3, f32)>,
    /// A position the hold or a fixed step moves the hero to.
    pub place: Option<Vec3>,
    /// The grab cooldown to set.
    pub cooldown: Option<u32>,
    /// Leave to state 1.
    pub leave: bool,
}

/// What the climb reads this tick: the hero position, the stick's forward and side values
/// (s above 0 moves toward u = 0), the ground gap, the current clip (action, progress) and the
/// grab cooldown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub hero: Vec3,
    pub f: f32,
    pub s: f32,
    pub gap: f32,
    pub clip: (u32, f32),
    pub cooldown: u32,
    /// Sub-mode 4: the hero's forward row, the stick's heading and its speed value (0 in the dead zone).
    pub facing: Vec3,
    pub heading: Vec3,
    pub speed: f32,
}

/// State 21: the sub-mode (the shared word +0x9BC), the target T, the queued panel Q, the
/// partner P and the return word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climb {
    pub sub: u8,
    pub target: usize,
    pub queued: Option<usize>,
    pub partner: Option<usize>,
    ret: u8,
    first: bool,
}

impl Climb {
    /// The wall grab's entry: sub-mode 1 on the panel.
    pub fn new(target: usize) -> Self {
        Self { sub: 1, target, queued: None, partner: None, ret: 0, first: false }
    }

    /// The ceiling grab's entry: sub-mode 18 on the panel.
    pub fn ceiling(target: usize) -> Self {
        Self { sub: 18, target, queued: None, partner: None, ret: 0, first: false }
    }

    /// State 38's hand-over: the top-edge hang of the panel (sub-mode 17).
    pub fn from_drop(target: usize) -> Self {
        Self { sub: 17, target, queued: None, partner: None, ret: 0, first: false }
    }

    /// The panel hold: along n until n . (p, 1) = r, steering to -n.
    fn hold(t: &Panel, p: Vec3, step: &mut Step) -> Vec3 {
        let n = t.n();
        let d = plane(t.normal, p);
        let q = std::array::from_fn(|k| p[k] + (RADIUS - d) * n[k]);
        step.place = Some(q);
        step.facing = Some((n.map(|c| -c), 1.0));
        q
    }

    /// Sub-mode 4's near edges of a panel, by link number: within r of each of its own four edges.
    fn near_edges(t: &Panel, p: Vec3) -> [bool; 4] {
        let (u, h) = (plane(t.u, p), plane(t.v, p));
        [h <= RADIUS, u >= t.lu - RADIUS, h > t.lv - RADIUS, u <= RADIUS]
    }

    /// Sub-mode 4's edge probe: no near edge without a link, or with a link out of the panel's plane.
    fn edge_clear(t: &Panel, panels: &[Panel], p: Vec3) -> bool {
        let near = Self::near_edges(t, p);
        (0..4).all(|k| !near[k] || t.links[k].filter(|&i| i < panels.len()).is_some_and(|i| dot3(panels[i].n(), t.n()) >= 0.99))
    }

    /// Sub-mode 4's Q: the first near edge with a link, in the order 0, 2, 3, 1.
    fn edge_pick(t: &Panel, panels: &[Panel], p: Vec3) -> Option<usize> {
        let near = Self::near_edges(t, p);
        [0, 2, 3, 1].into_iter().filter(|&k| near[k]).find_map(|k| t.links[k].filter(|&i| i < panels.len()))
    }

    fn enter(&mut self, sub: u8) {
        self.sub = sub;
        self.first = true;
    }

    /// One tick of state 21.
    pub fn tick(&mut self, panels: &[Panel], v: View) -> Step {
        let mut step = Step { rate: 1.0, ..Step::default() };
        let Some(t) = panels.get(self.target).copied() else {
            step.leave = true;
            return step;
        };
        let p = v.hero;
        let (f, s) = (v.f, v.s);
        let (ut, ht) = (plane(t.u, p), plane(t.v, p));
        let link = |r: &Panel, k: usize| r.links[k].filter(|&i| i < panels.len());
        let ny = |i: usize| panels[i].normal[1];
        let ndot = |a: usize, b: usize| dot3(panels[a].n(), panels[b].n());
        let cross_y = |a: Vec3, b: Vec3| a[2] * b[0] - a[0] * b[2];
        let (action, progress) = v.clip;
        let first = std::mem::take(&mut self.first);
        match self.sub {
            0 => step.leave = true,
            1 => {
                // The approach: the hold, the grab clip 121, then the free climb.
                Self::hold(&t, p, &mut step);
                step.request = Some(121);
                if action == 121 && progress >= 0.99 {
                    self.enter(2);
                    step.request = Some(49);
                }
            }
            2 => {
                Self::hold(&t, p, &mut step);
                step.facing = Some((t.n().map(|c| -c), 0.5));
                step.request = Some(49);
                if f == 0.0 && s == 0.0 {
                    return step;
                }
                if s.abs() < f.abs() {
                    if f > 0.0 {
                        if ht < t.lv - HEIGHT {
                            step.request = Some(56);
                            step.rate = f.abs().max(0.3);
                        } else if let Some(q) = link(&t, 2).filter(|_| v.cooldown == 0) {
                            let c = ndot(q, self.target);
                            self.queued = Some(q);
                            if c > 0.99 {
                                self.partner = Some(self.target);
                                self.ret = 2;
                                self.enter(13);
                            } else if ny(q) > 0.95 {
                                // A top face: hang from T's top edge (Q stays queued).
                                step.cooldown = Some(20);
                                self.enter(17);
                            } else if ny(q) < 0.95 {
                                self.enter(3);
                            }
                        }
                    } else if ht > 0.5 && v.gap > 0.0 {
                        step.request = Some(45);
                        step.rate = f.abs().max(0.3);
                    } else if let Some(q) = link(&t, 0).filter(|_| v.cooldown == 0) {
                        let c = ndot(q, self.target);
                        if c > 0.99 {
                            self.queued = Some(q);
                            self.partner = Some(self.target);
                            self.ret = 2;
                            self.enter(14);
                        } else if ny(q) > 0.95 {
                            self.enter(0);
                            step.leave = true;
                        } else if ny(q) < 0.95 {
                            self.queued = Some(q);
                            self.enter(6);
                        }
                    }
                } else if s > 0.0 {
                    if ut > RADIUS {
                        step.request = Some(52);
                        step.rate = s.abs().max(0.3);
                    } else if let Some(q) = link(&t, 3) {
                        self.queued = Some(q);
                        if ndot(q, self.target) > 0.25 {
                            self.partner = Some(self.target);
                            self.ret = 2;
                            self.enter(15);
                        } else {
                            self.enter(if cross_y(t.n(), panels[q].n()) >= 0.0 { 12 } else { 9 });
                        }
                    }
                } else if ut < t.lu - RADIUS {
                    step.request = Some(53);
                    step.rate = s.abs().max(0.3);
                } else if let Some(q) = link(&t, 1) {
                    self.queued = Some(q);
                    if ndot(q, self.target) > 0.25 {
                        self.partner = Some(self.target);
                        self.ret = 16;
                        self.enter(16);
                    } else {
                        self.enter(if cross_y(t.n(), panels[q].n()) <= 0.0 { 11 } else { 8 });
                    }
                }
            }
            3 => {
                // Back off from the overhang along nT, then the queued panel is the target (kept queued).
                let d = plane(t.normal, p);
                let along = (1.25 - d).max(0.0) + 0.05;
                let n = t.n();
                step.place = Some(std::array::from_fn(|k| p[k] + along * n[k]));
                step.request = Some(49);
                if let Some(q) = self.queued {
                    self.target = q;
                }
                step.cooldown = Some(120);
                self.enter(4);
            }
            4 => {
                // The panel walk: the top held at the plane, the stick steering at weight 1.0.
                let d = plane(t.normal, p);
                let held = [p[0], p[1] + d - HEIGHT, p[2]];
                step.place = Some(held);
                if v.speed <= 0.0 {
                    step.request = Some(48);
                    return step;
                }
                step.facing = Some((v.heading, 1.0));
                let walk = |step: &mut Step| {
                    step.request = Some(46);
                    step.rate = v.speed.max(0.3);
                };
                if Self::edge_clear(&t, panels, held) {
                    walk(&mut step);
                    // A link in the same plane: the seam is crossed at once, with no cooldown test.
                    let q = Self::edge_pick(&t, panels, held);
                    self.queued = q;
                    if let Some(q) = q.filter(|&q| Self::near_edges(&panels[q], held) != [true; 4]) {
                        self.target = q;
                        self.queued = None;
                    }
                    return step;
                }
                let b = v.facing;
                let ahead = std::array::from_fn(|k| held[k] + 1.5 * b[k]);
                if Self::edge_clear(&t, panels, ahead) {
                    walk(&mut step);
                    return step;
                }
                let q = Self::edge_pick(&t, panels, held);
                self.queued = q;
                step.request = Some(48);
                let Some(q) = q.filter(|_| v.cooldown == 0) else { return step };
                let r = panels[q];
                let (hq, along) = (plane(r.v, held), dot3(b, r.n()));
                if hq > 0.5 * r.lv {
                    if along < -0.75 {
                        let n = r.n();
                        step.place = Some(std::array::from_fn(|k| held[k] - 0.05 * n[k]));
                        step.facing = Some((n.map(|c| -c), 1.0));
                        step.request = Some(49);
                        step.cooldown = Some(120);
                        self.target = q;
                        self.enter(2);
                    }
                } else if hq > -4.0 && along > 0.75 {
                    step.facing = Some((r.n(), 1.0));
                    step.request = None;
                    self.enter(5);
                }
            }
            5 => {
                // Down over the top edge onto Q: raised while below it, then the free climb on it.
                step.request = Some(57);
                let Some(q) = self.queued else {
                    self.enter(4);
                    return step;
                };
                let r = panels[q];
                if action == 57 {
                    if plane(r.v, p) < 0.0 {
                        step.place = Some([p[0], p[1] + 0.05, p[2]]);
                    } else if progress >= 0.99 {
                        self.target = q;
                        self.queued = None;
                        Self::hold(&r, p, &mut step);
                        step.request = Some(49);
                        step.cooldown = Some(120);
                        self.enter(2);
                    }
                }
            }
            6 => {
                // Under the wall's lower edge: clip 54 past progress 0.95, then 48 on Q.
                step.request = Some(54);
                if action == 54 && progress > 0.95 {
                    step.request = Some(48);
                    if let Some(q) = self.queued {
                        self.target = q;
                    }
                    step.cooldown = Some(120);
                    self.enter(4);
                }
            }
            18 => {
                // The ceiling entry's one-shot correction: N . P becomes H.
                let d = plane(t.normal, p) - HEIGHT;
                let n = t.n();
                step.place = Some(std::array::from_fn(|k| p[k] - d * n[k]));
                step.request = Some(48);
                step.cooldown = Some(120);
                self.enter(4);
            }
            7 => {
                // The climb over the top: 59 to progress 0.95, then off to state 1.
                step.request = Some(59);
                if action == 59 && progress >= 0.95 {
                    step.cooldown = Some(120);
                    self.enter(0);
                    step.leave = true;
                }
            }
            8 | 9 | 11 | 12 => {
                // A corner: its clip to the end, then the queued panel with the hold.
                let clip = match self.sub {
                    8 => 44,
                    9 => 43,
                    11 => 51,
                    _ => 50,
                };
                step.request = Some(clip);
                if action == clip && progress >= 0.99 {
                    if let Some(q) = self.queued.take() {
                        self.target = q;
                        let r = panels[q];
                        let u = plane(r.u, p);
                        let du = u.clamp(RADIUS, (r.lu - RADIUS).max(RADIUS)) - u;
                        let moved = std::array::from_fn(|k| p[k] + du * r.u[k]);
                        Self::hold(&r, moved, &mut step);
                    }
                    step.cooldown = Some(0);
                    step.request = Some(49);
                    let back = if self.ret == 13 { 13 } else { 2 };
                    self.sub = back;
                }
            }
            13 | 14 => {
                if first {
                    if let Some(q) = self.queued.take() {
                        self.target = q;
                    }
                    return step;
                }
                Self::hold(&t, p, &mut step);
                let Some(pp) = self.partner else {
                    self.sub = 2;
                    return step;
                };
                let pr = panels[pp];
                let hp = plane(pr.v, p);
                if 1.0 < hp && hp < pr.lv - HEIGHT {
                    self.target = pp;
                    self.partner = None;
                    self.sub = 2;
                } else if 1.0 < ht && ht < t.lv - HEIGHT {
                    self.partner = None;
                    self.sub = 2;
                } else if s.abs() < f.abs() && f > 0.0 {
                    step.request = Some(56);
                } else if s.abs() < f.abs() && f < 0.0 {
                    step.request = Some(45);
                } else if s == 0.0 {
                    step.request = Some(49);
                } else {
                    // Sideways: while inside T, move; at the edge both panels need a neighbour on that side.
                    let toward_zero = s > 0.0;
                    let (k, clip) = if toward_zero { (3, 52) } else { (1, 53) };
                    let inside = if toward_zero { ut > RADIUS } else { ut < t.lu - RADIUS };
                    step.rate = s.abs().max(0.3);
                    if inside {
                        step.request = Some(clip);
                        return step;
                    }
                    step.request = Some(49);
                    let (Some(q), Some(q2)) = (link(&t, k), link(&pr, k)) else {
                        self.queued = None;
                        return step;
                    };
                    if ndot(q2, q).abs() < 0.95 {
                        self.queued = None;
                        return step;
                    }
                    if ndot(q, self.target) > 0.25 {
                        self.queued = None;
                        let before = if toward_zero { ut >= 0.0 } else { ut <= t.lu };
                        if before {
                            step.request = Some(clip);
                        } else {
                            self.target = q;
                            self.partner = Some(q2);
                            Self::hold(&panels[q], p, &mut step);
                        }
                    } else {
                        // A corner, with the return word 13.
                        self.queued = Some(q);
                        self.partner = Some(q2);
                        self.ret = 13;
                        let cy = cross_y(t.n(), panels[q].n());
                        self.enter(if toward_zero {
                            if cy >= 0.0 { 12 } else { 9 }
                        } else if cy <= 0.0 {
                            11
                        } else {
                            8
                        });
                    }
                }
            }
            15 | 16 => {
                if first {
                    if let Some(q) = self.queued.take() {
                        self.target = q;
                    }
                    return step;
                }
                let pr = self.partner.map(|i| panels[i]);
                if -RADIUS < ut && ut < t.lu + RADIUS {
                    self.partner = None;
                    Self::hold(&t, p, &mut step);
                    self.sub = 2;
                } else if let Some(pr) = pr.filter(|pr| {
                    let up = plane(pr.u, p);
                    RADIUS < up && up < pr.lu - RADIUS
                }) {
                    self.target = self.partner.take().unwrap_or(self.target);
                    Self::hold(&pr, p, &mut step);
                    self.sub = 2;
                } else if f.abs() < s.abs() {
                    step.request = Some(if s > 0.0 { 52 } else { 53 });
                    step.rate = s.abs().max(0.3);
                } else if f == 0.0 {
                    step.request = Some(49);
                } else {
                    // Up or down, while inside T; at its end both panels need a neighbour on that side.
                    let up = f > 0.0;
                    let inside = if up { ht < t.lv - HEIGHT } else { ht > 0.5 && v.gap > 0.0 };
                    let clip = if up { 56 } else { 45 };
                    step.rate = f.abs().max(0.3);
                    if inside {
                        step.request = Some(clip);
                        return step;
                    }
                    let k = if up { 2 } else { 0 };
                    step.request = Some(49);
                    let Some(q) = link(&t, k).filter(|_| v.cooldown == 0) else { return step };
                    let Some(q2) = pr.and_then(|pr| link(&pr, k)) else {
                        self.queued = None;
                        step.request = None;
                        return step;
                    };
                    self.queued = Some(q);
                    if ndot(q2, q) < 0.95 {
                        self.queued = None;
                        return step;
                    }
                    if ndot(q, self.target) <= 0.99 {
                        if ny(q) > 0.95 {
                            if up {
                                self.enter(17);
                            } else {
                                self.enter(0);
                                step.leave = true;
                            }
                        } else if ny(q) < 0.95 {
                            self.enter(if up { 3 } else { 6 });
                        }
                    } else {
                        step.request = Some(clip);
                        if if up { ht < t.lv } else { ht < 0.0 } {
                            self.target = q;
                            self.partner = Some(q2);
                        }
                        self.queued = None;
                    }
                }
            }
            17 => {
                // The top-edge hang: r - 0.7 from the wall, the hands at the top edge.
                let n = t.n();
                let d = plane(t.normal, p);
                let mut q: Vec3 = std::array::from_fn(|k| p[k] + (RADIUS - 0.7 - d) * n[k]);
                let vy = t.v[1];
                if vy.abs() > 1e-6 {
                    let e = p[1] + (t.lv - ht) / vy;
                    q[1] = e - 0.25 - HEIGHT + 0.1;
                }
                step.place = Some(q);
                step.facing = Some((n.map(|c| -c), 1.0));
                step.request = Some(73);
                // The stick is read only when both panel-projected values are non-zero (Top-edge
                // hang; test vector 148): the host passes the projections in this sub-mode.
                if f == 0.0 || s == 0.0 {
                    return step;
                }
                if s.abs() < f.abs() {
                    if f > 0.2 && v.cooldown == 0 {
                        self.enter(7);
                        step.request = Some(59);
                    } else if f < -0.2 {
                        self.sub = 2;
                    }
                } else if s > 0.0 {
                    if ut > RADIUS {
                        step.request = Some(60);
                    } else if let Some(nq) = link(&t, 3).filter(|&nq| ndot(nq, self.target) > 0.25) {
                        if ut > 0.0 {
                            step.request = Some(60);
                        } else {
                            self.target = nq;
                            let r = panels[nq];
                            let moved: Vec3 = std::array::from_fn(|k| q[k] - 0.05 * r.u[k]);
                            step.place = Some(moved);
                            step.request = Some(61);
                        }
                    }
                } else if ut < t.lu - RADIUS {
                    step.request = Some(61);
                } else if let Some(nq) = link(&t, 1).filter(|&nq| ndot(nq, self.target) > 0.25) {
                    if ut < t.lu {
                        step.request = Some(61);
                    } else {
                        self.target = nq;
                        let r = panels[nq];
                        let moved: Vec3 = std::array::from_fn(|k| q[k] + 0.05 * r.u[k]);
                        step.place = Some(moved);
                        step.request = Some(61);
                    }
                }
            }
            _ => {
                // Sub-mode 10 is not covered: back to the free climb.
                self.sub = 2;
                step.request = Some(49);
            }
        }
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall facing +Z (the face at z = 0), U along -X from x = 0 (so u = -x), V up from
    /// y = 0, 4 wide and 10 high, with a top face linked above it.
    /// SPEC-0033 test vectors 194 and 195: two ceiling panels in a plane, and a wall at the first's link 0.
    fn panel_walk_set() -> Vec<Panel> {
        let a = Panel { normal: [0.0, -1.0, 0.0, 7.883], u: [1.0, 0.0, 0.0, 83.066], v: [0.0, 0.0, 1.0, -366.381], lu: 8.934, lv: 7.216, kind: 0, links: [Some(2), Some(1), None, None] };
        let b = Panel { u: [1.0, 0.0, 0.0, 74.131], links: [None; 4], ..a };
        let wall = Panel { normal: [0.0, 0.0, 1.0, -367.556], u: [1.0, 0.0, 0.0, 83.066], v: [0.0, 1.0, 0.0, 1.051], lu: 8.934, lv: 8.934, kind: -2, links: [None; 4] };
        vec![a, b, wall]
    }

    fn panel_view(hero: Vec3, facing: Vec3, cooldown: u32) -> View {
        View { hero, f: 0.0, s: 0.0, gap: 1.0, clip: (0, 0.0), cooldown, facing, heading: facing, speed: 1.0 }
    }

    #[test]
    fn panel_walk_crosses_a_seam_in_the_plane() {
        let set = panel_walk_set();
        let mut c = Climb { sub: 4, target: 0, queued: None, partner: None, ret: 0, first: false };
        let s = c.tick(&set, panel_view([-75.30, 4.383, 370.0], [-1.0, 0.0, 0.0], 80));
        assert_eq!((s.request, c.target, c.queued), (Some(46), 1, None));
        // Clear of every edge: no Q.
        let mut c = Climb { sub: 4, target: 0, queued: None, partner: None, ret: 0, first: false };
        let s = c.tick(&set, panel_view([-75.40, 4.383, 370.0], [-1.0, 0.0, 0.0], 0));
        assert_eq!((s.request, c.target, c.queued), (Some(46), 0, None));
    }

    #[test]
    fn panel_walk_backs_onto_a_wall() {
        let set = panel_walk_set();
        let hero = [-79.0, 4.383, 367.40];
        let mut c = Climb { sub: 4, target: 0, queued: None, partner: None, ret: 0, first: false };
        let s = c.tick(&set, panel_view(hero, [0.0, 0.0, -1.0], 0));
        assert_eq!((s.request, c.sub, c.target, s.cooldown), (Some(49), 2, 2, Some(120)));
        assert!((s.place.unwrap()[2] - 367.35).abs() < 1e-3);
        // The cooldown holds him; facing away he walks.
        let mut c = Climb { sub: 4, target: 0, queued: None, partner: None, ret: 0, first: false };
        assert_eq!(c.tick(&set, panel_view(hero, [0.0, 0.0, -1.0], 80)).request, Some(48));
        assert_eq!((c.sub, c.target), (4, 0));
        let mut c = Climb { sub: 4, target: 0, queued: None, partner: None, ret: 0, first: false };
        assert_eq!(c.tick(&set, panel_view(hero, [0.0, 0.0, 1.0], 0)).request, Some(46));
        assert_eq!((c.sub, c.target), (4, 0));
    }

    fn wall() -> Vec<Panel> {
        vec![
            Panel { normal: [0.0, 0.0, 1.0, 0.0], u: [-1.0, 0.0, 0.0, 0.0], v: [0.0, 1.0, 0.0, 0.0], lu: 4.0, lv: 10.0, kind: -2, links: [None, None, Some(1), None] },
            Panel { normal: [0.0, 1.0, 0.0, -10.0], u: [-1.0, 0.0, 0.0, 0.0], v: [0.0, 0.0, -1.0, 0.0], lu: 4.0, lv: 2.0, kind: -2, links: [Some(0), None, None, None] },
        ]
    }

    /// Behavior 22A's ceiling window, the entry's correction, then a walk to the linked wall (pack 1080's
    /// shape: a ceiling at y 20, its link 1 a wall the hero meets facing it).
    #[test]
    fn ceiling_grab_and_hand_off() {
        let ceil = Panel { normal: [0.0, -1.0, 0.0, 20.0], u: [1.0, 0.0, 0.0, 0.0], v: [0.0, 0.0, 1.0, 0.0], lu: 50.0, lv: 4.0, kind: -2, links: [None, Some(1), None, None] };
        let wall = Panel { normal: [-1.0, 0.0, 0.0, 50.0], u: [0.0, 0.0, 1.0, 0.0], v: [0.0, 1.0, 0.0, -10.0], lu: 4.0, lv: 10.0, kind: -2, links: [None; 4] };
        let panels = [ceil, wall];
        assert_eq!(grab_ceiling(&panels, [10.0, 15.6, 2.0]), Some(0));
        assert_eq!(grab_ceiling(&panels, [10.0, 15.4, 2.0]), None, "more than H + 1.0 below");
        assert_eq!(grab_ceiling(&panels, [10.0, 15.6, 3.5]), None, "past LV - 1.0");
        let mut c = Climb::ceiling(0);
        let hero = [10.0, 15.6, 2.0];
        let step = c.tick(&panels, view(hero, 0.0, 0.0, (0, 0.0)));
        assert_eq!((c.sub, step.request, step.cooldown), (4, Some(48), Some(120)));
        assert!((step.place.unwrap()[1] - 16.5).abs() < 1e-5, "{:?}", step.place);
        let mut go = view([10.0, 16.5, 2.0], 1.0, 0.0, (48, 0.0));
        go.facing = [1.0, 0.0, 0.0];
        go.heading = [1.0, 0.0, 0.0];
        go.speed = 1.0;
        let step = c.tick(&panels, go);
        assert_eq!((c.sub, step.request), (4, Some(46)));
        go.hero = [49.5, 16.5, 2.0];
        let step = c.tick(&panels, go);
        assert_eq!((c.sub, c.target, step.request, step.cooldown), (2, 1, Some(49), Some(120)));
        // No way on: the hero holds where he is; nothing clamps him.
        let mut c = Climb::ceiling(0);
        c.sub = 4;
        go.facing = [-1.0, 0.0, 0.0];
        go.hero = [0.5, 16.5, 2.0];
        let step = c.tick(&panels, go);
        assert_eq!((step.request, step.place), (Some(48), Some([0.5, 16.5, 2.0])));
    }

    fn view(hero: Vec3, f: f32, s: f32, clip: (u32, f32)) -> View {
        View { hero, f, s, gap: 1.0, clip, cooldown: 0, facing: [0.0, 0.0, 1.0], heading: [0.0; 3], speed: 0.0 }
    }

    #[test]
    fn grab_gates() {
        let w = wall();
        let into = [0.0, 0.0, -1.0];
        assert_eq!(grab_wall(&w, [-2.0, 0.0, 1.0], into, into, 1.0), Some(0));
        // Facing away, too far out, past the sides or the stick under 0.5: no grab.
        assert_eq!(grab_wall(&w, [-2.0, 0.0, 1.0], [0.0, 0.0, 1.0], into, 1.0), None);
        assert_eq!(grab_wall(&w, [-2.0, 0.0, 1.6], into, into, 1.0), None);
        assert_eq!(grab_wall(&w, [-0.5, 0.0, 1.0], into, into, 1.0), None);
        assert_eq!(grab_wall(&w, [-2.0, 0.0, 1.0], into, into, 0.5), None);
    }

    #[test]
    fn climb_to_the_top_and_over() {
        let w = wall();
        let mut c = Climb::new(0);
        // The approach holds the hero at r from the face and plays 121 to its end.
        let s = c.tick(&w, view([-2.0, 0.0, 2.0], 0.0, 0.0, (0, 0.0)));
        assert_eq!(s.request, Some(121));
        assert!((s.place.unwrap()[2] - RADIUS).abs() < 1e-6);
        c.tick(&w, view([-2.0, 0.0, RADIUS], 0.0, 0.0, (121, 0.99)));
        assert_eq!(c.sub, 2);
        // Up below the top climbs (56 at max(f, 0.3)); at LV - H the top face hands to 17.
        let s = c.tick(&w, view([-2.0, 3.0, RADIUS], 0.2, 0.0, (49, 0.5)));
        assert_eq!((s.request, s.rate), (Some(56), 0.3));
        let s = c.tick(&w, view([-2.0, 6.6, RADIUS], 1.0, 0.0, (56, 0.5)));
        assert_eq!((c.sub, c.target, c.queued, s.cooldown), (17, 0, Some(1), Some(20)));
        // The hang: r - 0.7 from the wall, height E - 0.25 - H + 0.1 with E the top edge.
        let mut c = Climb { sub: 17, target: 0, queued: None, partner: None, ret: 0, first: false };
        let s = c.tick(&w, view([-2.0, 6.6, RADIUS], 0.0, 0.0, (73, 0.5)));
        let q = s.place.unwrap();
        assert!((q[2] - (RADIUS - 0.7)).abs() < 1e-5 && (q[1] - (10.0 - 0.25 - HEIGHT + 0.1)).abs() < 1e-5, "{q:?}");
        // Forward climbs over with 59, then leaves at 0.95.
        // Test vector 148: a zero side value blocks; f 0.5, s 0.1 climbs over (cooldown 0).
        c.tick(&w, view(q, 0.5, 0.0, (73, 0.5)));
        assert_eq!(c.sub, 17);
        c.tick(&w, view(q, 0.5, 0.1, (73, 0.5)));
        assert_eq!(c.sub, 7);
        assert!(c.tick(&w, view(q, 0.0, 0.0, (59, 0.95))).leave);
    }

    #[test]
    fn sideways_and_down() {
        let w = wall();
        let mut c = Climb { sub: 2, target: 0, queued: None, partner: None, ret: 0, first: false };
        // s above 0 moves toward u = 0 (52) while u > r; s below 0 toward LU (53).
        assert_eq!(c.tick(&w, view([-2.0, 3.0, RADIUS], 0.0, 0.5, (49, 0.5))).request, Some(52));
        assert_eq!(c.tick(&w, view([-2.0, 3.0, RADIUS], 0.0, -0.5, (49, 0.5))).request, Some(53));
        // At the u = 0 edge with no link: the idle request.
        assert_eq!(c.tick(&w, view([-1.0, 3.0, RADIUS], 0.0, 0.5, (49, 0.5))).request, Some(49));
        // Down above 0.5 descends (45); at the bottom with no panel below, the idle request (the
        // player lets go with Stop Climbing).
        assert_eq!(c.tick(&w, view([-2.0, 3.0, RADIUS], -1.0, 0.0, (49, 0.5))).request, Some(45));
        let mut v = view([-2.0, 0.3, RADIUS], -1.0, 0.0, (49, 0.5));
        v.gap = 0.0;
        let s = c.tick(&w, v);
        assert!(!s.leave && s.request == Some(49));
    }
}
