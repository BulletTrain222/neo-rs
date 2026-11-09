//! The follow camera's eye pass (SPEC-0027 Walls and the stuck timers, Camera collision and
//! occlusion, Eye pass per-primitive routines, Eye pass failure flag, Side-swing event): after
//! the spring step the eye (a sphere of radius 1.25) is pushed out of the movable blocks, the
//! frame records of its cell (type 6 skipped by the caller), the cell's list A primitives and the
//! near tree trunks, in that order. Each contact adds its normal to the contact list (40-tick
//! entries, at most 19); then the routine settles: a sideways nudge when the line to the focus is
//! clear, else the two side probes, which post the side-swing event or set the failure flag. The
//! failure flag drives the stuck timer: the fourth consecutive failing tick runs the recentre.
//!
//! The caller applies the mask and class gates and picks the primitives (see the app). Medium in
//! the spec: the cylinder, zero-radius and trunk settling. This build's choices: the eye motion
//! direction (0x100) is the spring displacement of the tick; the zero-radius floor and ceiling
//! push (|axis y| above 0.85) moves the eye to 1.25 off the plane (the spec gives no depth); the
//! physics-block step divisor is 1; the clear test of the cylinder routine uses the closest
//! approach of the segment to the axis.

use crate::contact::{push_grown, BoxShape, WallEdge};
use crate::hero::Vec3;

/// The eye push radius (0x2DB400).
pub const EYE_RADIUS: f32 = 1.25;
/// Contact-list entry lifetime (0x2DB40C) and capacity.
pub const CONTACT_LIFE: u32 = 40;
pub const CONTACT_CAPACITY: usize = 19;
/// The side-probe length (0x2DB3FC).
pub const PROBE_LENGTH: f32 = 10.0;
/// The side-swing event the eye pass posts: rate (0x2DB404) and duration (0x2DB408).
pub const SWING_RATE: f32 = 0.01;
pub const SWING_TICKS: u32 = 10;
/// (int)(0.05 x tick rate 60): the stuck counter must exceed this.
pub const STUCK_LIMIT: u32 = 3;

/// One primitive the eye pass tests, in world space, gates already applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EyePrim {
    /// A frame record (wall, ledge, ground or ceiling slab).
    Wall(WallEdge),
    /// List A kind 1.
    Sphere { centre: Vec3, radius: f32 },
    /// List A kind 3: base, unit axis, length and radius (a flat-ended finite cylinder).
    Cylinder { base: Vec3, axis: Vec3, length: f32, radius: f32 },
    /// List A kinds 4 and 7 with the third extent at or below 0: axes V, A and the normal B with
    /// the extents along V and A.
    Panel { origin: Vec3, axes: [Vec3; 3], extents: [f32; 3] },
    /// List A kinds 4 and 7 with a positive third extent, and the active movable blocks.
    Box(BoxShape),
    /// A near-tree plant: an upright cylinder.
    Trunk { base: Vec3, length: f32, radius: f32 },
}

/// The eye pass words of the camera record (0x280 to 0x510) and the side-swing pair (0x1C8, 0x1CC).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EyeState {
    /// Contact normals with their remaining lifetimes.
    pub contacts: Vec<(Vec3, u32)>,
    pub prev_failure: bool,
    pub failure: bool,
    pub latch: bool,
    pub stuck: u32,
    /// The side-swing increment and its remaining ticks.
    pub swing: f32,
    pub swing_ticks: u32,
}

/// What the routines read besides the eye.
pub struct EyeCtx<'a> {
    pub focus: Vec3,
    /// The camera's forward row (0x20) and first row (0x00).
    pub forward: Vec3,
    pub first: Vec3,
    /// The focus hero is moving (the class test of the nudge and the ageing).
    pub moving: bool,
    /// The hold word (0x26C) is nonzero.
    pub hold: bool,
    /// The eye motion direction (0x100).
    pub heading: Vec3,
    /// The line-of-sight sweep: true when the segment is clear.
    pub clear: &'a dyn Fn(Vec3, Vec3) -> bool,
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: Vec3, k: f32) -> Vec3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn len(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}
fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 1e-12 { scale(a, 1.0 / l) } else { [0.0; 3] }
}
fn neg(a: Vec3) -> Vec3 {
    scale(a, -1.0)
}
fn plane(q: [f32; 4], p: Vec3) -> f32 {
    q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
}

impl EyeState {
    /// Start of the pass: the contact list ages (only while the hero moves or the hold is set) and
    /// the failure flag clears.
    pub fn begin(&mut self, active: bool) {
        if active {
            for c in &mut self.contacts {
                c.1 = c.1.saturating_sub(1);
            }
            self.contacts.retain(|c| c.1 > 0);
        }
        self.failure = false;
    }

    /// The contact-add routine: a normal within 0.999 of a stored one refreshes it; else it is
    /// appended while the list has room.
    pub fn add_contact(&mut self, n: Vec3) {
        if let Some(c) = self.contacts.iter_mut().find(|c| dot(c.0, n) > 0.999) {
            c.1 = CONTACT_LIFE;
        } else if self.contacts.len() < CONTACT_CAPACITY {
            self.contacts.push((n, CONTACT_LIFE));
        }
    }

    fn flagged(&self) -> bool {
        self.prev_failure || self.failure || self.latch
    }

    /// The contact: the normal is listed and the eye moved by it times the depth. True when a
    /// flag was already set (the routine stops).
    fn contact(&mut self, eye: &mut Vec3, n: Vec3, depth: f32) -> bool {
        self.add_contact(n);
        *eye = add(*eye, scale(n, depth));
        self.flagged()
    }

    /// The contact normal nudge.
    fn nudge(&self, eye: &mut Vec3, dir: Vec3, amount: f32, ctx: &EyeCtx) {
        if !(ctx.moving || ctx.hold) || self.stuck != 0 {
            return;
        }
        if self.contacts.iter().any(|c| dot(c.0, dir) <= 0.0) {
            return;
        }
        *eye = add(*eye, scale(dir, amount));
    }

    /// The side probes along D then minus D (the second with its inverted sight test), then the
    /// event or the failure.
    fn probes(&mut self, eye: Vec3, d: Vec3, ctx: &EyeCtx) {
        let accept = |v: Vec3, sight_clear: bool| {
            let far = add(eye, scale(v, PROBE_LENGTH));
            (ctx.clear)(eye, far) && (ctx.clear)(far, ctx.focus) == sight_clear
        };
        let dir = if accept(d, true) {
            Some(d)
        } else if accept(neg(d), false) {
            Some(neg(d))
        } else {
            None
        };
        match dir {
            Some(v) if !ctx.hold => self.post(v, ctx.forward, ctx.first),
            Some(_) => {}
            None => self.failure = true,
        }
    }

    /// The side-swing event with the direction flag 1.
    pub fn post(&mut self, n: Vec3, forward: Vec3, first: Vec3) {
        if dot(n, forward) > 0.0 {
            self.swing = if dot(n, first) < 0.0 { -SWING_RATE } else { SWING_RATE };
            self.swing_ticks = SWING_TICKS;
        }
    }

    /// The spin term of the event for this tick (the increment while the counter runs).
    pub fn swing_step(&mut self) -> f32 {
        if self.swing_ticks > 0 {
            self.swing_ticks -= 1;
            self.swing
        } else {
            0.0
        }
    }

    /// End of the pass (Eye pass failure flag step 3 and 4). Returns true when the recovery
    /// (recentre) pass must run.
    pub fn finish(&mut self, eye: &mut Vec3, prev_eye: Vec3) -> bool {
        let mut recover = false;
        if !self.failure && !self.latch {
            self.stuck = 0;
        } else {
            if self.contacts.len() >= 2 || self.latch {
                *eye = prev_eye;
                self.latch = true;
            }
            self.stuck += 1;
            if self.stuck > STUCK_LIMIT {
                recover = true;
                self.stuck = 0;
                self.latch = false;
                self.failure = false;
                self.prev_failure = false;
            }
        }
        self.prev_failure = self.failure;
        recover
    }

    /// The contact-block reset (it does not touch the contact list).
    pub fn reset_flags(&mut self) {
        self.prev_failure = false;
        self.failure = false;
        self.latch = false;
        self.stuck = 0;
    }
}

/// The eye pass over the primitives in order. `velocity` is the spring displacement (0x110).
pub fn eye_pass(state: &mut EyeState, eye: &mut Vec3, velocity: &mut Vec3, prims: &[EyePrim], ctx: &EyeCtx) {
    state.begin(ctx.moving || ctx.hold);
    for p in prims {
        match *p {
            EyePrim::Wall(w) => wall(state, eye, velocity, &w, ctx),
            EyePrim::Sphere { centre, radius } => sphere(state, eye, centre, radius, ctx),
            EyePrim::Cylinder { base, axis, length, radius } => cylinder(state, eye, base, unit(axis), length, radius, ctx),
            EyePrim::Trunk { base, length, radius } => cylinder(state, eye, base, [0.0, 1.0, 0.0], length, radius, ctx),
            EyePrim::Panel { origin, axes, extents } => panel(state, eye, velocity, origin, axes, extents, ctx),
            EyePrim::Box(b) => boxed(state, eye, &b, ctx),
        }
    }
}

/// The camera plane test of a frame record: the hit normal and depth (0 for ground and ceiling).
pub fn plane_test(w: &WallEdge, p: Vec3, velocity: Vec3) -> Option<(Vec3, f32)> {
    let r = EYE_RADIUS;
    if w.top[1] == 0.0 {
        return None;
    }
    let n = [w.face[0], w.face[1], w.face[2]];
    let b = sub(p, velocity);
    let (dp, db) = (plane(w.face, p), plane(w.face, b));
    let run = |q: Vec3| (-r..=w.length + r).contains(&plane(w.run, q));
    if !(run(p) || run(b)) {
        return None;
    }
    let y = p[1];
    let window = |top_cut: f32| {
        let at = |q: Vec3| {
            let t = w.edge_height(q);
            t - w.height - r < y && y < t - top_cut
        };
        at(p) || at(b)
    };
    if n[1].abs() <= 0.9 {
        if !((dp > 0.0 || db > 0.0) && dp <= r) || !window(0.3) {
            return None;
        }
        Some((n, r - dp))
    } else {
        let touch = if n[1] > 0.0 { dp < 0.05 } else { dp < r };
        (touch && window(0.0)).then_some((n, 0.0))
    }
}

/// The record's rectangle crossed by the segment (two-sided).
fn wall_crossed(w: &WallEdge, a: Vec3, c: Vec3) -> bool {
    let (sa, sc) = (plane(w.face, a), plane(w.face, c));
    if sa * sc >= 0.0 || w.top[1] == 0.0 {
        return false;
    }
    let k = sa / (sa - sc);
    let x = add(a, scale(sub(c, a), k));
    let s = plane(w.run, x);
    let top = w.edge_height(x) + 0.25;
    (0.0..=w.length).contains(&s) && x[1] <= top && x[1] >= top - w.height
}

fn wall(st: &mut EyeState, eye: &mut Vec3, velocity: &mut Vec3, w: &WallEdge, ctx: &EyeCtx) {
    let Some((n, depth)) = plane_test(w, *eye, *velocity) else { return };
    if st.contact(eye, n, depth) {
        return;
    }
    let start = [ctx.focus[0], ctx.focus[1] - w.top[1].abs(), ctx.focus[2]];
    let mut d = unit([w.run[0], w.run[1], w.run[2]]);
    if !wall_crossed(w, start, *eye) {
        if dot(d, ctx.heading) <= 0.0 {
            d = neg(d);
        }
        st.nudge(eye, d, depth, ctx);
    } else {
        st.probes(*eye, d, ctx);
    }
    if (ctx.moving || ctx.hold) && !st.failure && !st.latch {
        *velocity = sub(*velocity, scale(n, dot(*velocity, n)));
    }
}

/// The segment against a horizontal circle (height ignored): true when it crosses.
fn crosses_circle(a: Vec3, c: Vec3, q: Vec3, r: f32) -> bool {
    let d2 = |p: Vec3| (p[0] - q[0]).powi(2) + (p[2] - q[2]).powi(2);
    let (ia, ic) = (d2(a) < r * r, d2(c) < r * r);
    if ia || ic {
        return ia != ic;
    }
    let (dx, dz) = (c[0] - a[0], c[2] - a[2]);
    let l2 = dx * dx + dz * dz;
    let t = if l2 > 1e-9 { (((q[0] - a[0]) * dx + (q[2] - a[2]) * dz) / l2).clamp(0.0, 1.0) } else { 0.0 };
    d2([a[0] + t * dx, 0.0, a[2] + t * dz]) < r * r
}

/// The horizontal slide axis about a centre: normalise(unit(eye -> centre) x up).
fn around(eye: Vec3, centre: Vec3) -> Vec3 {
    let u = unit(sub(centre, eye));
    unit([-u[2], 0.0, u[0]])
}

fn sphere(st: &mut EyeState, eye: &mut Vec3, centre: Vec3, radius: f32, ctx: &EyeCtx) {
    let v = sub(*eye, centre);
    let dist = len(v);
    let reach = radius + EYE_RADIUS;
    if dist >= reach || dist <= 0.0 {
        return;
    }
    let depth = reach - dist;
    if st.contact(eye, scale(v, 1.0 / dist), depth) {
        return;
    }
    let mut d = around(*eye, centre);
    if crosses_circle(*eye, ctx.focus, centre, radius) {
        st.probes(*eye, d, ctx);
    } else {
        if dot(d, ctx.heading) <= 0.0 {
            d = neg(d);
        }
        st.nudge(eye, d, depth, ctx);
    }
}

/// The eye sphere against a flat-ended finite cylinder: the outward normal and depth.
pub fn cylinder_contact(p: Vec3, base: Vec3, axis: Vec3, length: f32, radius: f32) -> Option<(Vec3, f32)> {
    let r = EYE_RADIUS;
    let rel = sub(p, base);
    let h = dot(rel, axis);
    let radial = sub(rel, scale(axis, h));
    let d = len(radial);
    let out = if d > 1e-6 { scale(radial, 1.0 / d) } else { [1.0, 0.0, 0.0] };
    if (0.0..=length).contains(&h) {
        return (d < radius + r).then(|| (out, radius + r - d));
    }
    let beyond = if h < 0.0 { h } else { h - length };
    if d <= radius {
        return (beyond.abs() < r).then(|| (scale(axis, beyond.signum()), r - beyond.abs()));
    }
    let e = ((d - radius).powi(2) + beyond * beyond).sqrt();
    (e < r && e > 0.0).then(|| (unit(add(scale(out, d - radius), scale(axis, beyond))), r - e))
}

/// The closest distance between segment a-c and the cylinder's axis segment.
fn segment_axis_distance(a: Vec3, c: Vec3, base: Vec3, axis: Vec3, length: f32) -> f32 {
    let u = sub(c, a);
    let v = scale(axis, length);
    let w = sub(a, base);
    let (uu, uv, vv, uw, vw) = (dot(u, u), dot(u, v), dot(v, v), dot(u, w), dot(v, w));
    let den = uu * vv - uv * uv;
    let mut s = if den > 1e-9 { ((uv * vw - vv * uw) / den).clamp(0.0, 1.0) } else { 0.0 };
    let mut t = if vv > 1e-9 { ((uv * s + vw) / vv).clamp(0.0, 1.0) } else { 0.0 };
    if uu > 1e-9 {
        s = ((uv * t - uw) / uu).clamp(0.0, 1.0);
    }
    if vv > 1e-9 {
        t = ((uv * s + vw) / vv).clamp(0.0, 1.0);
    }
    len(sub(add(a, scale(u, s)), add(base, scale(v, t))))
}

fn cylinder(st: &mut EyeState, eye: &mut Vec3, base: Vec3, axis: Vec3, length: f32, radius: f32, ctx: &EyeCtx) {
    let Some((n, depth)) = cylinder_contact(*eye, base, axis, length, radius) else { return };
    if st.contact(eye, n, depth) {
        return;
    }
    let d = around(*eye, base);
    let near = (base[0] - eye[0]).powi(2) + (base[2] - eye[2]).powi(2) < 2500.0;
    if near && segment_axis_distance(*eye, ctx.focus, base, axis, length) < radius {
        st.probes(*eye, d, ctx);
    } else {
        let d = if dot(ctx.heading, d) > 0.0 { d } else { neg(d) };
        st.nudge(eye, d, depth, ctx);
    }
}

/// The segment crosses the flat rectangle (two-sided).
fn crosses_panel(a: Vec3, c: Vec3, origin: Vec3, axes: [Vec3; 3], extents: [f32; 3]) -> bool {
    let n = axes[2];
    let (sa, sc) = (dot(n, sub(a, origin)), dot(n, sub(c, origin)));
    if sa * sc >= 0.0 {
        return false;
    }
    let x = sub(add(a, scale(sub(c, a), sa / (sa - sc))), origin);
    let within = |u: f32, e: f32| u >= e.min(0.0) && u <= e.max(0.0);
    within(dot(axes[0], x), extents[0]) && within(dot(axes[1], x), extents[1])
}

fn panel(st: &mut EyeState, eye: &mut Vec3, velocity: &mut Vec3, origin: Vec3, axes: [Vec3; 3], extents: [f32; 3], ctx: &EyeCtx) {
    let r = EYE_RADIUS;
    let n = axes[2];
    let q = sub(*eye, *velocity);
    let in_a = |x: Vec3| (-r..=extents[1] + r).contains(&dot(axes[1], sub(x, origin)));
    let in_v = |x: Vec3, top: f32| (-r..=top).contains(&dot(axes[0], sub(x, origin)));
    if n[1].abs() > 0.85 {
        // The floor and ceiling push: no list entry, event or failure.
        let s = dot(n, sub(*eye, origin));
        if (in_a(*eye) || in_a(q)) && (in_v(*eye, extents[0] + r) || in_v(q, extents[0] + r)) && s > -r && s < r {
            *eye = add(*eye, scale(n, r - s));
        }
        return;
    }
    let s = dot(n, sub(*eye, origin));
    let t = dot(n, sub(q, origin));
    if s < 0.0 && t < 0.0 && n[1] < 0.9 {
        return;
    }
    let touching = (s <= r && r < t) || (r <= s && t < r) || s.abs() <= r;
    if !touching || !(in_a(*eye) || in_a(q)) || !(in_v(*eye, extents[0] - 0.01) || in_v(q, extents[0] - 0.01)) {
        return;
    }
    let depth = r - s;
    if st.contact(eye, n, depth) {
        return;
    }
    let a = unit(axes[1]);
    if crosses_panel(*eye, ctx.focus, origin, axes, extents) {
        st.probes(*eye, a, ctx);
    } else {
        let d = if dot(ctx.heading, a) > 0.0 { a } else { neg(a) };
        st.nudge(eye, d, depth, ctx);
    }
    if st.flagged() {
        return;
    }
    let l = len(*velocity);
    let slid = sub(*velocity, scale(n, dot(*velocity, n)));
    *velocity = scale(unit(slid), l);
}

/// The segment crosses the box (slab test in the box's frame).
pub fn crosses_box(a: Vec3, c: Vec3, b: &BoxShape) -> bool {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let ax = b.axes[k];
        let pa = dot(ax, sub(a, b.origin));
        let pd = dot(ax, sub(c, a));
        let (lo, hi) = (b.extents[k].min(0.0), b.extents[k].max(0.0));
        if pd.abs() < 1e-9 {
            if pa < lo || pa > hi {
                return false;
            }
        } else {
            let (mut u0, mut u1) = ((lo - pa) / pd, (hi - pa) / pd);
            if u0 > u1 {
                std::mem::swap(&mut u0, &mut u1);
            }
            t0 = t0.max(u0);
            t1 = t1.min(u1);
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

fn boxed(st: &mut EyeState, eye: &mut Vec3, b: &BoxShape, ctx: &EyeCtx) {
    let Some((face, n, dist)) = push_grown(*eye, b, EYE_RADIUS, true) else { return };
    if st.contact(eye, n, dist) {
        return;
    }
    let (a, bb) = (b.axes[1], b.axes[2]);
    let s = match face {
        1 | 4 => a,
        2 | 3 => neg(a),
        5 => neg(bb),
        _ => bb,
    };
    if crosses_box(*eye, ctx.focus, b) {
        return;
    }
    let s = if dot(s, ctx.heading) > 0.0 { s } else { neg(s) };
    st.nudge(eye, s, dist, ctx);
}

/// The hero body radius the occlusion clip reads from the focus object's motion record.
pub const CLIP_BODY_RADIUS: f32 = 1.175;

/// The face test of the occlusion clip (SPEC-0027 Camera on world entry, Face test): the eye
/// shifted r_h against the normal, the plane fraction window -0.02 to 1.02, the run widened 0.25
/// and the vertical window from 0.25 below the depth to 0.25 under the top; a hit closer than
/// 1.2 r_h (as coded, ell - t) snaps the eye to the focus.
pub fn clip_face(eye: Vec3, focus: Vec3, w: &WallEdge) -> Option<Vec3> {
    let rh = CLIP_BODY_RADIUS;
    if w.top[1] == 0.0 {
        return None;
    }
    let n = [w.face[0], w.face[1], w.face[2]];
    let shifted = sub(eye, scale(n, rh));
    let (df, de) = (plane(w.face, focus), plane(w.face, shifted));
    if df == de {
        return None;
    }
    let t = df / (df - de);
    if !(-0.02..=1.02).contains(&t) {
        return None;
    }
    let x = add(focus, scale(sub(shifted, focus), t));
    let u = plane(w.run, x);
    let top = w.edge_height(x) + 0.25;
    if !(-0.25..=w.length + 0.25).contains(&u) || x[1] < top - w.height - 0.25 || x[1] > top - 0.25 {
        return None;
    }
    let ell = len(sub(eye, focus));
    Some(if ell - t < 1.2 * rh { focus } else { add(focus, scale(sub(eye, focus), t)) })
}

/// The face test's crossing as a boolean (the line-of-sight sweep's wall groups): the segment
/// a to c crosses the plane at a fraction in [-0.02, 1.02], inside the run widened 0.25 and the
/// vertical window from 0.25 below the depth to 0.25 under the top. Two-sided.
pub fn face_crossed(a: Vec3, c: Vec3, w: &WallEdge) -> bool {
    if w.top[1] == 0.0 {
        return false;
    }
    let (da, dc) = (plane(w.face, a), plane(w.face, c));
    if da == dc {
        return false;
    }
    let t = da / (da - dc);
    if !(-0.02..=1.02).contains(&t) {
        return false;
    }
    let x = add(a, scale(sub(c, a), t));
    let top = w.edge_height(x) + 0.25;
    (-0.25..=w.length + 0.25).contains(&plane(w.run, x)) && x[1] >= top - w.height - 0.25 && x[1] <= top - 0.25
}

/// The ray from s along unit u against a sphere: the nearest root in (0, len), else the far one.
fn ray_sphere(s: Vec3, u: Vec3, l: f32, c: Vec3, r: f32) -> Option<f32> {
    let m = sub(s, c);
    let b = dot(m, u);
    let disc = b * b - (dot(m, m) - r * r);
    if disc < 0.0 {
        return None;
    }
    let q = disc.sqrt();
    [-b - q, -b + q].into_iter().find(|&t| t > 0.0 && t < l)
}

/// The ray against a capped finite cylinder: the entry t0 if in (0, len), else the exit t1.
fn ray_cylinder(s: Vec3, u: Vec3, l: f32, base: Vec3, axis: Vec3, length: f32, radius: f32) -> Option<f32> {
    let m = sub(s, base);
    let (mh, uh) = (dot(m, axis), dot(u, axis));
    let (mr, ur) = (sub(m, scale(axis, mh)), sub(u, scale(axis, uh)));
    let (a, b, c) = (dot(ur, ur), 2.0 * dot(mr, ur), dot(mr, mr) - radius * radius);
    let (mut t0, mut t1) = if a < 1e-9 {
        if c > 0.0 {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }
        let q = disc.sqrt();
        ((-b - q) / (2.0 * a), (-b + q) / (2.0 * a))
    };
    if uh.abs() < 1e-9 {
        if mh < 0.0 || mh > length {
            return None;
        }
    } else {
        let (mut c0, mut c1) = (-mh / uh, (length - mh) / uh);
        if c0 > c1 {
            std::mem::swap(&mut c0, &mut c1);
        }
        t0 = t0.max(c0);
        t1 = t1.min(c1);
    }
    if t0 > t1 {
        return None;
    }
    [t0, t1].into_iter().find(|&t| t > 0.0 && t < l)
}

/// The exact rectangle crossing (fraction in [0, 1], in-plane coordinates in [0, extent]); the
/// plane normal is axes[2], the in-plane axes axes[0] and axes[1].
fn rect_fraction(a: Vec3, c: Vec3, origin: Vec3, axes: [Vec3; 3], extents: [f32; 3]) -> Option<f32> {
    let n = axes[2];
    let (sa, sc) = (dot(n, sub(a, origin)), dot(n, sub(c, origin)));
    if sa == sc {
        return None;
    }
    let t = sa / (sa - sc);
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    let x = sub(add(a, scale(sub(c, a), t)), origin);
    let within = |u: f32, e: f32| u >= e.min(0.0) && u <= e.max(0.0);
    (within(dot(axes[1], x), extents[1]) && within(dot(axes[0], x), extents[0])).then_some(t)
}

/// The box ray test: the smallest fraction of the five faces the segment crosses (the four sides
/// and the bottom; never the top), after the ball prefilter of 20 x the largest extent.
fn box_fraction(s: Vec3, e: Vec3, b: &BoxShape) -> Option<f32> {
    let biggest = b.extents.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let v = sub(e, s);
    let k = (dot(sub(b.origin, s), v) / dot(v, v).max(1e-12)).clamp(0.0, 1.0);
    if len(sub(add(s, scale(v, k)), b.origin)) >= 20.0 * biggest {
        return None;
    }
    // (axis index, high face): +-B, +-A and the bottom (V = 0).
    let faces = [(2usize, true), (2, false), (1, true), (1, false), (0, false)];
    let mut best: Option<f32> = None;
    for (k, high) in faces {
        let n = b.axes[k];
        let o = if high { add(b.origin, scale(n, b.extents[k])) } else { b.origin };
        let others: Vec<usize> = (0..3).filter(|&j| j != k).collect();
        let axes = [b.axes[others[0]], b.axes[others[1]], n];
        let extents = [b.extents[others[0]], b.extents[others[1]], 0.0];
        if let Some(t) = rect_fraction(s, e, o, axes, extents) {
            best = Some(best.map_or(t, |m| m.min(t)));
        }
    }
    best
}

/// The occlusion clip (SPEC-0027 Camera on world entry, Occlusion clip): the eye is pulled along
/// the line toward the focus by each primitive in order, each seeing the eye left by the one
/// before. Frame records by the face test; spheres by three parallel rays 1.5 apart; cylinders
/// and trunks as capped cylinders (within 50 horizontally of the eye); flat rectangles by the
/// exact rectangle crossing from the focus lowered by |V.y| (the zero nudge omitted); boxes by
/// the first of five faces entered. The rounds and boxes start from S = the focus lowered by
/// 1.0. Not built: the wall-face groups and the plate pool.
pub fn occlusion_clip(eye: Vec3, focus: Vec3, prims: &[EyePrim]) -> Vec3 {
    let mut e = eye;
    let down = [focus[0], focus[1] - 1.0, focus[2]];
    let round = |e: Vec3, hit: &dyn Fn(Vec3, Vec3, f32) -> Option<f32>| -> Option<Vec3> {
        let v = sub(e, down);
        let l = len(v);
        if l <= 0.0 {
            return None;
        }
        let u = scale(v, 1.0 / l);
        hit(down, u, l).map(|t| add(down, scale(u, t - 0.05)))
    };
    let near = |e: Vec3, q: Vec3| (q[0] - e[0]).powi(2) + (q[2] - e[2]).powi(2) < 2500.0;
    for p in prims {
        let moved = match *p {
            EyePrim::Wall(w) => clip_face(e, focus, &w),
            EyePrim::Sphere { centre, radius } => round(e, &|s, u, l| {
                let side = scale(unit([-u[2], 0.0, u[0]]), 1.5);
                [s, add(s, side), sub(s, side)].into_iter().find_map(|o| ray_sphere(o, u, l, centre, radius))
            }),
            EyePrim::Cylinder { base, axis, length, radius } if near(e, base) => {
                round(e, &|s, u, l| ray_cylinder(s, u, l, base, unit(axis), length, radius))
            }
            EyePrim::Trunk { base, length, radius } if near(e, base) => {
                round(e, &|s, u, l| ray_cylinder(s, u, l, base, [0.0, 1.0, 0.0], length, radius))
            }
            EyePrim::Panel { origin, axes, extents } => {
                let s = [focus[0], focus[1] - axes[0][1].abs(), focus[2]];
                let shifted = sub(e, scale(axes[2], 0.05));
                rect_fraction(s, shifted, origin, axes, extents).map(|t| add(s, scale(sub(e, s), t)))
            }
            EyePrim::Box(b) => box_fraction(down, e, &b).map(|t| add(down, scale(sub(e, down), t))),
            _ => None,
        };
        if let Some(m) = moved {
            e = m;
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_face_vectors() {
        // SPEC-0027 Test vectors for the clip: plane z = 0, run x 0 to 40, top 20, depth 20.
        let w = wall_z0();
        let f = [10.0, 5.0, 8.0];
        let z = |e: Vec3, f: Vec3| clip_face(e, f, &w).map(|p| p[2]);
        assert!((z([10.0, 5.0, 0.5], f).unwrap() - 1.08357).abs() < 1e-4);
        assert!((z([10.0, 5.0, -3.0], f).unwrap() - 0.77207).abs() < 1e-4);
        assert!((z([10.0, 5.0, 0.0], f).unwrap() - 1.02452).abs() < 1e-4);
        assert!(z([10.0, 5.0, 2.0], f).is_none());
        assert_eq!(clip_face([10.0, 5.0, 0.3], [10.0, 5.0, 1.2], &w), Some([10.0, 5.0, 1.2]));
        assert!((z([10.0, 5.0, 1.0], [10.0, 5.0, 3.5]).unwrap() - 1.11905).abs() < 1e-4);
        assert!(z([50.0, 5.0, -3.0], [50.0, 5.0, 8.0]).is_none());
        // Top cut: true top 20, D = 120.
        let deep = WallEdge { height: 120.0, top: [0.0, 1.0, 0.0, 100.0], ..w };
        assert!(clip_face([10.0, 19.6, -3.0], [10.0, 19.6, 8.0], &deep).is_some());
        assert!(clip_face([10.0, 19.9, -3.0], [10.0, 19.9, 8.0], &deep).is_none());
        // Doorway: A x 0 to 10 and B x 14 to 24.
        let a = WallEdge { length: 10.0, ..w };
        let b = WallEdge { run: [1.0, 0.0, 0.0, -14.0], ..a };
        let door = [EyePrim::Wall(a), EyePrim::Wall(b)];
        assert_eq!(occlusion_clip([12.0, 5.0, -3.0], [12.0, 5.0, 8.0], &door), [12.0, 5.0, -3.0]);
        let e = occlusion_clip([9.0, 5.0, -3.0], [12.0, 5.0, 8.0], &door);
        assert!((e[0] - 10.02875).abs() < 1e-3 && (e[2] - 0.77207).abs() < 1e-4, "{e:?}");
    }

    #[test]
    fn clip_rectangle_box_and_trunk_vectors() {
        let f = [10.0, 5.0, 8.0];
        // Rectangle in z = 0 (normal +z, V vertical): eye (10, 4.72398, 0.03620).
        let axes = [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let panel = EyePrim::Panel { origin: [0.0, -50.0, 0.0], axes, extents: [100.0, 40.0, 0.0] };
        let e = occlusion_clip([10.0, 5.0, -3.0], f, &[panel]);
        assert!((e[1] - 4.72398).abs() < 1e-4 && (e[2] - 0.03620).abs() < 1e-4, "{e:?}");
        // Box x 0 to 4, z 0 to 4, y 0 to 6: eye (2, 4.18182, 4.0).
        let b = BoxShape { origin: [0.0; 3], axes, extents: [6.0, 4.0, 4.0] };
        let e = occlusion_clip([2.0, 4.5, -3.0], [2.0, 5.0, 8.0], &[EyePrim::Box(b)]);
        assert!((e[1] - 4.18182).abs() < 1e-4 && (e[2] - 4.0).abs() < 1e-4, "{e:?}");
        // Trunk: eye (-20, 45, 0), focus (20, 45, 0), base y 40, radius 4.95, height 25: blocked.
        let t = EyePrim::Trunk { base: [0.0, 40.0, 0.0], length: 25.0, radius: 4.95 };
        assert_ne!(occlusion_clip([-20.0, 45.0, 0.0], [20.0, 45.0, 0.0], &[t]), [-20.0, 45.0, 0.0]);
        // 55 away horizontally: skipped.
        assert_eq!(occlusion_clip([-55.0, 45.0, 0.0], [20.0, 45.0, 0.0], &[t]), [-55.0, 45.0, 0.0]);
    }

    fn ctx<'a>(clear: &'a dyn Fn(Vec3, Vec3) -> bool, moving: bool) -> EyeCtx<'a> {
        EyeCtx { focus: [0.0, 3.5, 20.0], forward: [0.0, 0.0, 1.0], first: [1.0, 0.0, 0.0], moving, hold: false, heading: [0.0; 3], clear }
    }

    fn wall_z0() -> WallEdge {
        // Normal (0, 0, 1) through z = 0, run along x from 0 to 40, top plane at 20, depth 20.
        WallEdge { face: [0.0, 0.0, 1.0, 0.0], run: [1.0, 0.0, 0.0, 0.0], top: [0.0, 1.0, 0.0, 0.0], length: 40.0, height: 20.0, kind: 0 }
    }

    #[test]
    fn camera_plane_test_vectors() {
        // SPEC-0027 Camera plane test: T = 19.75, window -1.5 < y < 19.45.
        let w = wall_z0();
        let hit = |p: Vec3, v: Vec3| plane_test(&w, p, v).map(|(_, d)| d);
        assert!((hit([10.0, 5.0, 0.9], [0.0, 0.0, -0.6]).unwrap() - 0.35).abs() < 1e-5);
        assert!((hit([10.0, 5.0, -0.5], [0.0, 0.0, -2.0]).unwrap() - 1.75).abs() < 1e-5);
        assert!(hit([10.0, 5.0, -3.0], [0.0, 0.0, -1.0]).is_none());
        assert!(hit([10.0, 5.0, 0.0], [0.0; 3]).is_none());
        assert!((hit([10.0, 5.0, 0.0], [0.0, 0.0, -0.4]).unwrap() - 1.25).abs() < 1e-5);
        assert!((hit([10.0, 5.0, -2.0], [0.0, 0.0, -3.0]).unwrap() - 3.25).abs() < 1e-5);
        assert!(hit([41.0, 5.0, 0.9], [0.0; 3]).is_some() && hit([45.0, 5.0, 0.9], [0.0; 3]).is_none());
        assert!(hit([10.0, 19.5, 0.9], [0.0; 3]).is_none() && hit([10.0, 19.4, 0.9], [0.0; 3]).is_some());
        assert!(hit([10.0, -1.6, 0.9], [0.0; 3]).is_none() && hit([10.0, -1.4, 0.9], [0.0; 3]).is_some());
    }

    #[test]
    fn doorway_gap_band() {
        // Two panels in z = 0 with a gap of 4 (x 10 to 14): the free band is 11.25 < x < 12.75.
        let a = wall_z0();
        let a = WallEdge { length: 10.0, ..a };
        let b = WallEdge { run: [1.0, 0.0, 0.0, -14.0], ..a };
        let at = |x: f32| plane_test(&a, [x, 5.0, 0.9], [0.0, 0.0, -0.6]).is_some() || plane_test(&b, [x, 5.0, 0.9], [0.0, 0.0, -0.6]).is_some();
        assert!(!at(12.0) && at(11.0) && at(12.8));
    }

    #[test]
    fn box_vector() {
        // Box x 0 to 4, z 0 to 4, y 0 to 6; eye (2, 3, 2.5): the high-z face wins, eye to z 5.25.
        let b = BoxShape { origin: [0.0; 3], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [6.0, 4.0, 4.0] };
        let mut st = EyeState::default();
        let mut eye = [2.0, 3.0, 2.5];
        let clear = |_: Vec3, _: Vec3| true;
        eye_pass(&mut st, &mut eye, &mut [0.0; 3], &[EyePrim::Box(b)], &ctx(&clear, false));
        assert!((eye[2] - 5.25).abs() < 1e-5 && eye[0] == 2.0, "{eye:?}");
        assert_eq!(st.contacts.len(), 1);
    }

    #[test]
    fn sphere_probes_post_or_fail() {
        // A sphere between the eye and the focus: blocked sight line.
        let centre = [0.0, 3.0, 2.0];
        let sight = move |a: Vec3, c: Vec3| !crosses_circle(a, c, centre, 1.0);
        let mut st = EyeState::default();
        let mut eye = [0.0, 3.0, 0.0];
        eye_pass(&mut st, &mut eye, &mut [0.0; 3], &[EyePrim::Sphere { centre, radius: 1.0 }], &ctx(&sight, true));
        // Pushed out to 2.25 from the centre; a side probe clears, the event points along +-x,
        // which has no forward component, so nothing is written; no failure.
        assert!((eye[2] - (2.0 - 2.25)).abs() < 1e-5 && !st.failure);
        // Everything blocked: the failure flag.
        let none = |_: Vec3, _: Vec3| false;
        let mut st = EyeState::default();
        let mut eye = [0.0, 3.0, 0.0];
        eye_pass(&mut st, &mut eye, &mut [0.0; 3], &[EyePrim::Sphere { centre, radius: 1.0 }], &ctx(&none, true));
        assert!(st.failure);
    }

    #[test]
    fn stuck_timer_recovers_on_the_fourth_failing_tick() {
        let mut st = EyeState::default();
        let mut eye = [0.0; 3];
        for tick in 1..=4 {
            st.begin(true);
            st.failure = true;
            let recover = st.finish(&mut eye, [0.0; 3]);
            assert_eq!(recover, tick == 4, "tick {tick}");
        }
        assert_eq!(st.stuck, 0);
        assert!(!st.prev_failure);
    }

    #[test]
    fn side_swing_event_and_contact_list() {
        let mut st = EyeState::default();
        st.post([0.5, 0.0, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!((st.swing, st.swing_ticks), (SWING_RATE, SWING_TICKS));
        st.post([-0.5, 0.0, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!(st.swing, -SWING_RATE);
        let mut quiet = EyeState::default();
        quiet.post([0.5, 0.0, -0.5], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!(quiet.swing_ticks, 0);
        // A near-equal normal refreshes; the list caps at 19; entries age only while active.
        st.add_contact([1.0, 0.0, 0.0]);
        st.begin(true);
        st.add_contact([1.0, 0.0, 0.0001]);
        assert_eq!(st.contacts, vec![([1.0, 0.0, 0.0], CONTACT_LIFE)]);
        for k in 0..30 {
            let a = 0.2 * k as f32;
            st.add_contact([a.cos(), 0.0, a.sin()]);
        }
        assert_eq!(st.contacts.len(), CONTACT_CAPACITY);
        st.begin(false);
        assert!(st.contacts.iter().all(|c| c.1 >= CONTACT_LIFE - 1));
    }

    #[test]
    fn wall_group_face_crossing() {
        // The face of the clip vectors (plane z = 0, run x 0 to 40, top 20, depth 20): a line
        // through it is blocked; over the top cut (y 19.9) or past the run (x 50) it is clear.
        let w = wall_z0();
        assert!(face_crossed([10.0, 5.0, 8.0], [10.0, 5.0, -3.0], &w));
        assert!(!face_crossed([10.0, 19.9, 8.0], [10.0, 19.9, -3.0], &w));
        assert!(!face_crossed([50.0, 5.0, 8.0], [50.0, 5.0, -3.0], &w));
        assert!(!face_crossed([10.0, 5.0, 8.0], [10.0, 5.0, 2.0], &w));
    }
}
