//! NPC walking (SPEC-0056 Navigation step, Route step, Wall confinement, Blocked flag, Travel by
//! root motion; SPEC-0029 Behavior 2b goal latch): steering toward a walk target over a
//! navigation mesh, the navmesh confinement, and travel by the playing clip's root step.
//! The steering look-ahead (SPEC-0029 2a) shifts the goal around the host's `avoid` list.
//! Not covered: the batched goal locate's 50-per-tick budget (the goal is located in full each
//! tick), the occupancy quirk, cell list B, the character push and moving objects.

use crate::hero::{Vec3, basis, steer};
use crate::motion::Body;
use crate::nav::NavMesh;

/// An obstacle of the steering look-ahead: a vertical finite cylinder, with the character it is (0: none), which
/// that character's own walk skips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub point: Vec3,
    pub radius: f32,
    pub length: f32,
    pub owner: u32,
    /// Both lanes are tested (the kind-3 cylinders); blocks, grid-cell entries and characters
    /// test only the first lane, P + s to W + s (SPEC-0029 2a quirk).
    pub both_lanes: bool,
}

/// The radius and length of a block's, or another character's, obstacle.
pub const OBSTACLE_RADIUS: f32 = 5.0;
pub const OBSTACLE_LENGTH: f32 = 20.0;

/// A segment against a vertical cylinder centred on its point (flat ends).
fn segment_meets(a: Vec3, b: Vec3, o: &Obstacle) -> bool {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let l = (dx * dx + dz * dz).max(1e-9);
    let t = (((o.point[0] - a[0]) * dx + (o.point[2] - a[2]) * dz) / l).clamp(0.0, 1.0);
    let (ex, ez) = (a[0] + t * dx - o.point[0], a[2] + t * dz - o.point[2]);
    let y = a[1] + t * (b[1] - a[1]);
    (ex * ex + ez * ez).sqrt() <= o.radius && (y - o.point[1]).abs() <= 0.5 * o.length
}

/// SPEC-0029 2a: the goal point for the walk target w. The two lanes one unit to each side of the straight line are
/// tested against the obstacles; of the hits, the one nearest p is passed on the side given by the hero's position
/// relative to it, by R + 5.0, on the mesh, or else w.
pub fn look_ahead(p: Vec3, w: Vec3, hero: Vec3, avoid: &[Obstacle], own: u32, on_mesh: impl Fn(Vec3) -> bool) -> Vec3 {
    let (dx, dz) = (w[0] - p[0], w[2] - p[2]);
    let l = dx.hypot(dz);
    if l <= 1e-6 {
        return w;
    }
    // s = unit(d x up).
    let s = [-dz / l, 0.0, dx / l];
    let lane = |k: f32, q: Vec3| [q[0] + k * s[0], q[1], q[2] + k * s[2]];
    let hit = avoid
        .iter()
        .filter(|o| o.owner != own || own == 0)
        .filter(|o| segment_meets(lane(1.0, p), lane(1.0, w), o) || (o.both_lanes && segment_meets(lane(-1.0, p), lane(-1.0, w), o)))
        .min_by(|a, b| hdist(a.point, p).total_cmp(&hdist(b.point, p)));
    let Some(o) = hit else { return w };
    let (ux, uz) = (o.point[0] - hero[0], o.point[2] - hero[2]);
    let m = ux.hypot(uz).max(1e-6);
    let n = [-uz / m, 0.0, ux / m];
    let d = o.radius + 5.0;
    let first = [o.point[0] + d * n[0], w[1], o.point[2] + d * n[2]];
    if on_mesh(first) {
        return first;
    }
    let second = [o.point[0] - d * n[0], w[1], o.point[2] - d * n[2]];
    if on_mesh(second) { second } else { w }
}

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
    /// How far the last motion tick moved the character across the ground (after the navmesh
    /// confinement): the enemy's speed of SPEC-0038 sequencer state 7 (R-0252).
    pub last_step: f32,
    /// Facing (B3) and the move direction the steering writes (the facing follows it).
    pub facing: Vec3,
    pub radius: f32,
    /// The obstacles of the steering look-ahead this tick (the host's list), and its own id there.
    pub avoid: std::sync::Arc<Vec<Obstacle>>,
    pub avoid_id: u32,
    /// Navigation-mesh index (None: no mesh) and the triangles of the step.
    pub mesh: Option<usize>,
    current: Option<usize>,
    last_valid: Option<usize>,
    c0: Option<usize>,
    t0: Option<usize>,
    steer_point: Vec3,
    latch: bool,
    /// The goal locate's misses since it last held, and the last good goal: while the latch holds the goal stays there;
    /// it clears after ceil(T / 50) misses, the batch that ends at the mesh's last triangle (SPEC-0056 TV-0056-R228a).
    misses: usize,
    last_goal: Option<Vec3>,
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
    /// A can-fly 1 enemy in flight (SPEC-0040 Behavior 14D item 7): the no-gravity flag, which
    /// also skips the ground projection; its height comes from the caller's height keeper.
    pub float: bool,
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
            last_step: 0.0,
            current: None,
            last_valid: None,
            c0: None,
            t0: None,
            steer_point: position,
            latch: true,
            misses: 0,
            last_goal: None,
            arrived: false,
            blocked: false,
            blocked_timer: 0,
            lock: false,
            edge_guard: true,
            float: false,
            reverse: false,
            saved: position,
            id,
            avoid: Default::default(),
            avoid_id: id,
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
                self.misses = 0;
                self.last_goal = Some(w);
                (Some(t), w)
            }
            None if self.latch => {
                self.misses += 1;
                if self.misses >= m.len().div_ceil(50) {
                    self.latch = false;
                }
                match (self.last_goal, self.last_valid) {
                    (Some(g), t) => (m.locate_full(g).or(t), g),
                    (None, Some(t)) => (Some(t), m.centroid(t)),
                    (None, None) => (None, w),
                }
            }
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

    /// The debug tools' inspector line.
    pub fn debug(&self) -> String {
        format!("mesh {:?} triangle {:?} goal latch {} misses {} last goal {:?} arrived {} blocked {} timer {}", self.mesh, self.current, self.latch, self.misses, self.last_goal, self.arrived, self.blocked, self.blocked_timer)
    }

    /// SPEC-0056 Wander step 3: the walker is settled (its goal in its own triangle) within 2.5 of the
    /// plane of one of that triangle's boundary edges, or the goal latch has cleared.
    pub fn wander_edge(&self, meshes: &[NavMesh]) -> bool {
        let m = self.mesh.and_then(|i| meshes.get(i));
        let (Some(m), Some(c)) = (m, self.current.filter(|&c| m.is_some_and(|m| c < m.len()))) else { return false };
        if !self.latch {
            return true;
        }
        if self.c0 != Some(c) || self.t0 != Some(c) {
            return false;
        }
        let (tri, p) = (m.triangles[c], self.position());
        (0..3).any(|k| {
            let (v0, v1) = (tri[k], tri[(k + 1) % 3]);
            let (ex, ez) = (v1[0] - v0[0], v1[2] - v0[2]);
            let l = ex.hypot(ez);
            m.neighbours[c][k] < 0 && l > 0.0 && ((p[0] - v0[0]) * -ez + (p[2] - v0[2]) * ex) / l < 2.5
        })
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
        let w = if self.avoid.is_empty() { w } else { look_ahead(p, w, hero, &self.avoid, self.avoid_id, |q| m.locate(None, q).is_some()) };
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
        if self.float {
            // No ground projection and no gravity while flying.
        } else if self.edge_guard && !self.lock && self.body.gap.abs() > 2.0 {
            self.body.position = std::array::from_fn(|k| p0[k] - d[k]);
        } else {
            self.body.constrain();
        }
        if !self.float {
            self.body.gravity();
        }
        self.confine(meshes, d);
        self.body.integrate();
        self.lock = false;
        let q = self.body.position;
        self.last_step = (q[0] - p0[0]).hypot(q[2] - p0[2]);
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
        let m = self.mesh.and_then(|i| meshes.get(i));
        let (Some(m), Some(c)) = (m, self.current.filter(|&c| m.is_some_and(|m| c < m.len()))) else { return };
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
            // The reach's three tests (R-0194): the opposite vertex, v0 and the second vertex with the
            // next edge's neighbour index as its w.
            let outside = v0[0] * n[0] + v0[2] * n[2];
            let next = m.neighbours[c][(k + 1) % 3] as f32;
            let rho = if signed(opp) >= 2.0 * r && outside >= r && outside * (1.0 - next) >= 2.0 * r { r } else { 0.0 };
            let p = self.position();
            let (d, u) = (signed(p), along(p));
            if d <= rho && (0.0..=len).contains(&u) {
                let depth = (d - rho).abs();
                let v = self.body.velocity;
                // Within 0.05 with nothing moving into the wall, the response changes nothing.
                if depth > 0.05 || step[0] * n[0] + step[2] * n[2] < 0.0 || v[0] * n[0] + v[2] * n[2] < 0.0 {
                    self.body.position[0] += n[0] * depth;
                    self.body.position[2] += n[2] * depth;
                    self.remove_into_wall(n);
                }
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

    /// A triangle number from another world's mesh is ignored, not read past this mesh's end.
    #[test]
    fn a_stale_triangle_is_ignored() {
        let m = NavMesh { neighbours: vec![[-1, -1, -1]], triangles: vec![[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 0.0, 10.0]]], ..Default::default() };
        let meshes = [m];
        let mut w = Walker::new([1.0, 0.0, 1.0], [0.0, 0.0, 1.0], 1.0, 1.0, 0);
        w.mesh = Some(0);
        w.current = Some(18);
        w.latch = true;
        assert!(!w.wander_edge(&meshes));
        w.confine(&meshes, [0.0; 3]);
    }

    /// TV-0056-R228a's rule: a goal off the mesh keeps the last good goal for ceil(T / 50) misses, then the nearest triangle
    /// is searched from the goal, not from the walker.
    #[test]
    fn an_off_mesh_goal_searches_from_the_goal() {
        let tri = |x: f32| [[x, 0.0, 0.0], [x + 3.0, 0.0, 0.0], [x, 0.0, 3.0]];
        let m = NavMesh { neighbours: vec![[-1, -1, -1]; 2], triangles: vec![tri(0.0), tri(100.0)], ..Default::default() };
        let mut w = Walker::new([101.0, 0.0, 1.0], [0.0, 0.0, 1.0], 1.0, 1.0, 10);
        let inside = [101.0, 0.0, 1.0];
        assert_eq!(w.goal(&m, inside).1, inside);
        // One miss for a mesh of two triangles, with the last good goal kept.
        assert_eq!(w.goal(&m, [-50.0, 0.0, 1.0]).1, inside);
        // Then the centroid of the triangle nearest the goal, far from the walker.
        assert_eq!(w.goal(&m, [-50.0, 0.0, 1.0]).1, m.centroid(0));
    }

    /// TV-0056-R194e: a reach needs the next edge too.
    #[test]
    fn reach_needs_the_next_edge() {
        for (next, pushed) in [(-1, true), (0, true), (1, false), (3, false)] {
            let m = NavMesh { neighbours: vec![[-1, next, -1]], triangles: vec![[[100.0, 0.0, 100.0], [200.0, 0.0, 100.0], [100.0, 0.0, 200.0]]], ..square() };
            let meshes = [m];
            let mut w = Walker::new([150.0, 0.0, 100.5], [0.0, 0.0, 1.0], 1.175, 1.0, 0);
            w.mesh = Some(0);
            w.current = Some(0);
            w.confine(&meshes, [0.0; 3]);
            assert_eq!(w.blocked, pushed, "next {next}");
            assert_eq!(w.position()[2] > 101.0, pushed, "next {next}");
        }
    }

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

    /// SPEC-0029 2a: the side comes from the hero's position, and the mesh decides between the candidates.
    #[test]
    fn look_ahead_shifts_around_the_nearest_obstacle() {
        let o = |x: f32, z: f32, owner: u32| Obstacle { point: [x, 0.0, z], radius: OBSTACLE_RADIUS, length: OBSTACLE_LENGTH, owner, both_lanes: true };
        let (p, w, hero) = ([0.0; 3], [0.0, 0.0, 30.0], [0.0, 0.0, 40.0]);
        // Clear lanes: w unchanged.
        assert_eq!(look_ahead(p, w, hero, &[o(20.0, 15.0, 2)], 1, |_| true), w);
        // Nearest of two; u = (0, -1) from the hero's side, n = (1, 0): the first candidate is O + 10 n.
        assert_eq!(look_ahead(p, w, hero, &[o(0.0, 20.0, 2), o(0.0, 10.0, 3)], 1, |_| true), [10.0, 0.0, 10.0]);
        // Off the mesh: the second, then w.
        assert_eq!(look_ahead(p, w, hero, &[o(0.0, 10.0, 3)], 1, |q| q[0] < 0.0), [-10.0, 0.0, 10.0]);
        assert_eq!(look_ahead(p, w, hero, &[o(0.0, 10.0, 3)], 1, |_| false), w);
        // Its own entry is skipped; so is one above the lanes.
        assert_eq!(look_ahead(p, w, hero, &[o(0.0, 10.0, 1)], 1, |_| true), w);
        let high = Obstacle { point: [0.0, 30.0, 10.0], radius: 5.0, length: 20.0, owner: 2, both_lanes: true };
        assert_eq!(look_ahead(p, w, hero, &[high], 1, |_| true), w);
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
