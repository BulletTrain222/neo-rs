//! NPC walking (SPEC-0056 Navigation step, Route step, Wall confinement, Blocked flag, Travel by
//! root motion; SPEC-0029 Behavior 2b goal latch): steering toward a walk target over a
//! navigation mesh, the navmesh confinement, and travel by the playing clip's root step.
//! Not covered: the steering look-ahead's obstacles (SPEC-0029 2a; the goal is the walk target
//! unchanged), the batched goal locate's 50-per-tick budget (the goal is located in full each
//! tick), the occupancy quirk, cell list B, the character push and moving objects.

use crate::hero::{Vec3, basis, steer};
use crate::motion::Body;
use crate::nav::NavMesh;

/// What the navigation step asks of the character's animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gait {
    /// Play the movement clip of the current locomotion mode.
    Move,
    /// Play the idle picker.
    Idle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Walker {
    /// The body: position, velocity and the physics of SPEC-0040 Behavior 14A.
    pub body: Body,
    /// Facing (B3) and the move direction the steering writes (the facing follows it).
    pub facing: Vec3,
    pub radius: f32,
    /// Navigation-mesh index (None: no mesh) and the triangles of the step.
    pub mesh: Option<usize>,
    current: Option<usize>,
    last_valid: Option<usize>,
    c0: Option<usize>,
    t0: Option<usize>,
    steer_point: Vec3,
    latch: bool,
    pub arrived: bool,
    /// Blocked flag and timer (set to 120 by the confinement).
    pub blocked: bool,
    pub blocked_timer: u32,
    /// The motion record's lock word (+0x280; SPEC-0056 Lock value): a one-tick flag that skips
    /// the edge guard and the off-mesh restore; cleared at the end of every motion tick.
    pub lock: bool,
    /// The class kind query returns 1 (SPEC-0040 Behavior 14E): the edge guard applies. On for
    /// every walker (no hero walks with one).
    pub edge_guard: bool,
    /// Locomotion modes 3, 4 and 8 steer toward minus the direction (SPEC-0029 Behavior 2,
    /// Steering by mode); set by the caller for the step.
    pub reverse: bool,
    saved: Vec3,
    id: u32,
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

fn dir(a: Vec3, b: Vec3) -> Vec3 {
    [b[0] - a[0], 0.0, b[2] - a[2]]
}

impl Walker {
    pub fn new(position: Vec3, facing: Vec3, radius: f32, weight: f32, id: u32) -> Self {
        Self {
            body: Body::new(position, weight),
            facing,
            radius,
            mesh: None,
            current: None,
            last_valid: None,
            c0: None,
            t0: None,
            steer_point: position,
            latch: true,
            arrived: false,
            blocked: false,
            blocked_timer: 0,
            lock: false,
            edge_guard: true,
            reverse: false,
            saved: position,
            id,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.body.position
    }

    /// Place the body at a point with its velocity cleared (a scripted placement).
    pub fn place(&mut self, p: Vec3) {
        self.body.position = p;
        self.body.velocity = [0.0; 3];
        self.saved = p;
    }

    /// The current walk waypoint (the steer point of the route step).
    pub fn waypoint(&self) -> Vec3 {
        self.steer_point
    }

    /// A fresh block: the confinement pushed during the previous tick.
    pub fn fresh_block(&self) -> bool {
        self.blocked && self.blocked_timer == 120
    }

    /// Mesh assignment: locate the character on the new mesh.
    pub fn assign_mesh(&mut self, mesh: Option<usize>, meshes: &[NavMesh]) {
        if self.mesh == mesh {
            return;
        }
        self.mesh = mesh;
        let p = self.position();
        let m = mesh.and_then(|i| meshes.get(i));
        self.current = m.and_then(|m| m.locate(None, p));
        self.last_valid = self.current;
        self.c0 = None;
        self.t0 = None;
        self.saved = p;
    }

    /// Steering toward a point with a weight.
    pub fn steer_to(&mut self, target: Vec3, w: f32) {
        let d = dir(self.position(), target);
        self.facing = steer(self.facing, if self.reverse { d.map(|v| -v) } else { d }, w);
    }

    /// The goal latch (SPEC-0029 2b): the goal triangle and point for this tick.
    fn goal(&mut self, m: &NavMesh, w: Vec3) -> (Option<usize>, Vec3) {
        match m.locate_full(w) {
            Some(t) => {
                self.latch = true;
                (Some(t), w)
            }
            None if self.latch => match self.last_valid {
                Some(t) => (Some(t), m.centroid(t)),
                None => (None, w),
            },
            None => {
                let j = (self.id % 20) as f32 * 0.2 - 2.0;
                let q = if self.id % 2 == 0 { [w[0] + j, w[1], w[2]] } else { [w[0], w[1], w[2] + j] };
                match m.nearest(q) {
                    Some(t) => {
                        self.current = Some(t);
                        self.last_valid = Some(t);
                        (Some(t), m.centroid(t))
                    }
                    None => (None, w),
                }
            }
        }
    }

    /// One navigation step toward walk target w (SPEC-0056 Navigation step). Returns the gait
    /// to play; sets `arrived` on arrival. With no mesh the character steers straight at w.
    pub fn nav_step(&mut self, meshes: &[NavMesh], w: Vec3, hero: Vec3) -> Gait {
        let p = self.position();
        let Some(m) = self.mesh.and_then(|i| meshes.get(i)) else {
            if hdist(p, w) < 0.25 {
                self.arrived = true;
                return Gait::Idle;
            }
            self.steer_to(w, 0.3);
            return Gait::Move;
        };
        let c = m.locate(self.current, p);
        if c.is_some() {
            self.current = c;
            self.last_valid = c;
        }
        let Some(c) = c else {
            // Off the mesh: steer to the nearest triangle's centroid.
            if let Some(t) = m.nearest(p) {
                self.current = Some(t);
                self.last_valid = Some(t);
                self.steer_to(m.centroid(t), 0.1);
            }
            return Gait::Move;
        };
        let (t, g) = self.goal(m, w);
        let Some(t) = t else { return Gait::Idle };
        let d = hdist(p, g);
        if self.c0 == Some(c) && t == c {
            self.t0 = Some(c);
            if d < 0.25 || self.fresh_block() {
                self.arrived = true;
                self.steer_to(hero, 0.1);
                return Gait::Idle;
            }
            self.steer_to(g, 0.3);
            return Gait::Move;
        }
        if d < 0.25 {
            self.arrived = true;
            return Gait::Idle;
        }
        if self.c0 == Some(c) {
            if self.t0 == Some(t) {
                let k = self.steer_point;
                self.steer_to(k, 0.1);
            } else {
                self.t0 = Some(t);
            }
        } else {
            self.t0 = Some(t);
            self.c0 = Some(c);
        }
        if self.c0 != self.t0 {
            if let Some(e) = m.route(c, t) {
                self.steer_point = m.route_point(c, e, p, g, self.radius + 0.25);
                let k = self.steer_point;
                self.steer_to(k, 0.1);
            }
        } else {
            self.steer_to(g, 0.1);
        }
        Gait::Move
    }

    /// Set a new walk target (the plain setter): the arrival flag is cleared.
    pub fn set_target(&mut self) {
        self.arrived = false;
    }

    /// The motion-record tick of a ground NPC: ground step, travel by root step L through the
    /// facing basis (projected while within 1.0 of the ground), constraint, gravity, the navmesh
    /// confinement, integrator. `probe(x, z, ceiling)` gives the ground height and normal.
    pub fn motion_tick(&mut self, root_step: Vec3, meshes: &[NavMesh], probe: impl Fn(f32, f32, f32) -> (f32, Vec3)) {
        self.body.measure(probe);
        let p0 = self.body.position;
        let [b1, b2, b3] = basis(self.facing);
        let mut d: Vec3 = std::array::from_fn(|k| root_step[0] * b1[k] + root_step[1] * b2[k] + root_step[2] * b3[k]);
        if self.body.gap < 1.0 {
            let n = self.body.normal;
            let dn = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
            d = std::array::from_fn(|k| d[k] - dn * n[k]);
        }
        for k in 0..3 {
            self.body.position[k] += d[k];
        }
        // SPEC-0040 Behavior 14E: with the lock word clear and the ground gap above 2.0 in
        // magnitude, this tick's root step is reversed (P' = P0 - D) and the rest of the pre-step
        // (the ground constraint) is skipped.
        if self.edge_guard && !self.lock && self.body.gap.abs() > 2.0 {
            self.body.position = std::array::from_fn(|k| p0[k] - d[k]);
        } else {
            self.body.constrain();
        }
        self.body.gravity();
        self.confine(meshes, d);
        self.body.integrate();
        self.lock = false;
    }

    /// SPEC-0056 navmesh confinement: the blocked timer; boundary edges of the current triangle
    /// push the position back to the edge's reach (the push removes the into-wall part of the
    /// step); the off-mesh restore puts the character back at its saved position.
    fn confine(&mut self, meshes: &[NavMesh], step: Vec3) {
        if self.blocked {
            self.blocked_timer = self.blocked_timer.saturating_sub(1);
            if self.blocked_timer < 1 {
                self.blocked = false;
            }
        }
        let (Some(m), Some(c)) = (self.mesh.and_then(|i| meshes.get(i)), self.current) else { return };
        let r = self.radius;
        let tri = m.triangles[c];
        for k in 0..3 {
            if m.neighbours[c][k] >= 0 {
                continue;
            }
            let (v0, v1, opp) = (tri[k], tri[(k + 1) % 3], tri[(k + 2) % 3]);
            let (ex, ez) = (v1[0] - v0[0], v1[2] - v0[2]);
            let len = (ex * ex + ez * ez).sqrt();
            if len == 0.0 {
                continue;
            }
            // Inward normal for the positive winding: (−ez, ex) / len.
            let n = [-ez / len, 0.0, ex / len];
            let signed = |q: Vec3| (q[0] - v0[0]) * n[0] + (q[2] - v0[2]) * n[2];
            let along = |q: Vec3| ((q[0] - v0[0]) * ex + (q[2] - v0[2]) * ez) / len;
            let rho = if signed(opp) >= 2.0 * r && v0[0] * n[0] + v0[2] * n[2] >= r { r } else { 0.0 };
            let p = self.position();
            let (d, u) = (signed(p), along(p));
            if d <= rho && (0.0..=len).contains(&u) {
                let depth = (d - rho).abs();
                self.body.position[0] += n[0] * depth;
                self.body.position[2] += n[2] * depth;
                self.remove_into_wall(n);
                self.blocked = true;
                self.blocked_timer = 120;
            } else if (step[0] * step[0] + step[2] * step[2]).sqrt() > 0.1 {
                let q = [p[0] + 0.5 * step[0], p[1], p[2] + 0.5 * step[2]];
                if signed(q) <= rho && (0.0..=len).contains(&along(q)) {
                    self.remove_into_wall(n);
                    self.blocked = true;
                    self.blocked_timer = 120;
                }
            }
        }
        let p = self.position();
        match m.locate(self.current, p) {
            Some(t) => {
                self.current = Some(t);
                self.saved = p;
            }
            None if !self.lock => {
                self.body.position = self.saved;
                self.blocked = true;
                self.blocked_timer = 120;
            }
            None => {}
        }
    }

    fn remove_into_wall(&mut self, n: Vec3) {
        let v = self.body.velocity;
        let into = v[0] * n[0] + v[2] * n[2];
        if into < 0.0 {
            self.body.velocity[0] -= into * n[0];
            self.body.velocity[2] -= into * n[2];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> NavMesh {
        NavMesh {
            kind: 1,
            triangles: vec![
                [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 0.0, 10.0]],
                [[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 0.0, 10.0]],
            ],
            neighbours: vec![[-1, -1, 1], [0, -1, -1]],
            table: vec![0b0000_1000],
        }
    }

    fn flat(_: f32, _: f32, _: f32) -> (f32, Vec3) {
        (0.0, [0.0, 1.0, 0.0])
    }

    #[test]
    fn walks_across_triangles_to_the_target_by_root_motion() {
        let meshes = [square()];
        let mut w = Walker::new([8.0, 0.0, 2.0], [1.0, 0.0, 0.0], 1.0, 5.5, 1);
        w.assign_mesh(Some(0), &meshes);
        let target = [2.0, 0.0, 8.0];
        let mut moved = false;
        for _ in 0..400 {
            let gait = w.nav_step(&meshes, target, [100.0, 0.0, 100.0]);
            let step = if gait == Gait::Move { [0.0, 0.0, 0.1] } else { [0.0; 3] };
            moved |= gait == Gait::Move;
            w.motion_tick(step, &meshes, flat);
            if w.arrived {
                break;
            }
        }
        assert!(moved && w.arrived, "{:?}", w.position());
        assert!(hdist(w.position(), target) < 0.35, "{:?}", w.position());
    }

    #[test]
    fn a_boundary_edge_holds_the_walker_inside() {
        let meshes = [square()];
        // Walking straight at the x = 10 edge (a boundary of triangle 0).
        let mut w = Walker::new([8.0, 0.0, 2.0], [1.0, 0.0, 0.0], 1.0, 5.5, 1);
        w.assign_mesh(Some(0), &meshes);
        for _ in 0..100 {
            w.motion_tick([0.0, 0.0, 0.2], &meshes, flat);
        }
        assert!(w.position()[0] <= 10.0, "{:?}", w.position());
        assert!(w.blocked);
    }

    #[test]
    fn the_edge_guard_reverses_the_step_over_a_drop() {
        // SPEC-0040 Behavior 14E: with the gap above 2.0 the root step D is reversed (P0 - D);
        // with the lock word set it is kept.
        let drop = |_: f32, _: f32, _: f32| (-10.0, [0.0, 1.0, 0.0]);
        let mut w = Walker::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 5.5, 1);
        w.motion_tick([0.0, 0.0, 0.5], &[], drop);
        assert!((w.position()[2] + 0.5).abs() < 1e-4, "{:?}", w.position());
        let mut w = Walker::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 5.5, 1);
        w.lock = true;
        w.motion_tick([0.0, 0.0, 0.5], &[], drop);
        assert!((w.position()[2] - 0.5).abs() < 1e-4 && !w.lock, "{:?}", w.position());
        // On flat ground (gap 0) the step is kept.
        let mut w = Walker::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 5.5, 1);
        w.motion_tick([0.0, 0.0, 0.5], &[], flat);
        assert!((w.position()[2] - 0.5).abs() < 1e-4);
    }
}
