//! Whinny (behavior class 7; SPEC-0029 Behavior 5) and the round-up flags (SPEC-0049 item 6):
//! each Whinny runs one of four modes in priority order (penned, lead, flee, wander); the group
//! count sets flag 107 + p (p penned) whenever the group has three members.
//! "A random point on its mesh" is read as a uniformly drawn triangle's centroid, the flee
//! sub-step B's safe point as the far-centroid pick (uniform draws) with D = 10 from the hero, and the path test
//! of sub-step A as the candidate lying on a triangle of the current mesh (SPEC-0029 Behavior 5;
//! R-0073 answered); the latch is cleared whenever a flee sub-step ends. The navmesh is taken once at
//! start (the first ground mesh holding the Whinny) and set directly only on entering lead (the
//! pen's entrance mesh) and on becoming penned (the inner mesh); R-0013. The group sound is not
//! played.

use crate::hero::Vec3;
use crate::level::GameState;
use crate::nav::NavMesh;
use crate::rng::Rng;
use crate::walker::{Gait, Walker};

/// Animation ids of the Whinny's locomotion slots: idle 0, walk (mode 0) 2, run (mode 2) 3.
const IDLE: u32 = 0;
const WALK: u32 = 2;
const RUN: u32 = 3;

/// A pen (SetWhinnyPenmesh / SetWhinnySwitches): the inner-area and entrance navmesh ids, the
/// pen point (a locator) and the occupied flag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pen {
    pub inner: usize,
    pub entrance: usize,
    pub point: Vec3,
    pub occupied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Wander,
    Flee,
    Lead,
    Penned,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Whinny {
    pub mode: Mode,
    target: Option<Vec3>,
    timer: i32,
    run: bool,
    /// Flee sub-step: 0 none, 1 direct away (A), 2 safe point (B); its tick budget.
    flee_step: u8,
    flee_ticks: u32,
    pen: Option<usize>,
    /// The start navmesh pick has run.
    started: bool,
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

fn random_centroid(m: &NavMesh, rng: &mut Rng) -> Option<Vec3> {
    (!m.is_empty()).then(|| m.centroid(rng.index(m.len() as u32) as usize))
}

impl Default for Whinny {
    fn default() -> Self {
        Self { mode: Mode::Wander, target: None, timer: 0, run: false, flee_step: 0, flee_ticks: 0, pen: None, started: false }
    }
}

impl Whinny {
    pub fn penned(&self) -> bool {
        self.mode == Mode::Penned
    }

    fn inside(meshes: &[NavMesh], mesh: usize, p: Vec3) -> bool {
        meshes.get(mesh).is_some_and(|m| m.locate_full(p).is_some())
    }

    /// One tick: picks the mode, steers the walker and returns the animation id to play.
    pub fn tick(&mut self, w: &mut Walker, meshes: &[NavMesh], pens: &mut [Pen], hero: Vec3, rng: &mut Rng) -> u32 {
        let p = w.position();
        // Start: the first ground mesh (kind 1) holding the Whinny, once; no search later.
        if !self.started {
            self.started = true;
            let pick = meshes.iter().position(|m| m.kind == 1 && m.locate_full(p).is_some());
            w.assign_mesh(pick, meshes);
        }
        let d = hdist(p, hero);
        match self.mode {
            Mode::Penned => self.penned_tick(w, meshes, pens, hero, rng),
            Mode::Lead => self.lead_tick(w, meshes, pens, hero),
            Mode::Flee => {
                // An accepting pen's entrance turns the flight into a lead.
                let accepting = pens.len() == 1 || pens.iter().any(|pen| !pen.occupied);
                if let Some(i) = (0..pens.len()).find(|&i| (pens.len() == 1 || !pens[i].occupied) && accepting && Self::inside(meshes, pens[i].entrance, p)) {
                    pens[i].occupied = true;
                    self.mode = Mode::Lead;
                    self.pen = Some(i);
                    self.timer = 30;
                    self.target = Some(pens[i].point);
                    // The current navmesh is set directly to the entrance mesh (no search).
                    w.assign_mesh(Some(pens[i].entrance), meshes);
                    w.set_target();
                    return self.lead_tick(w, meshes, pens, hero);
                }
                if d > 8.0 {
                    self.mode = Mode::Wander;
                    self.target = None;
                    self.flee_step = 0;
                    return self.wander_tick(w, meshes, pens, hero, rng);
                }
                self.flee_tick(w, meshes, hero, rng)
            }
            Mode::Wander => {
                if d < 5.0 {
                    self.mode = Mode::Flee;
                    self.flee_step = 0;
                    return self.flee_tick(w, meshes, hero, rng);
                }
                self.wander_tick(w, meshes, pens, hero, rng)
            }
        }
    }

    fn wander_tick(&mut self, w: &mut Walker, meshes: &[NavMesh], pens: &mut [Pen], hero: Vec3, rng: &mut Rng) -> u32 {
        let p = w.position();
        if pens.len() == 1 && Self::inside(meshes, pens[0].inner, p) {
            self.mode = Mode::Penned;
            self.pen = Some(0);
            self.target = None;
            w.assign_mesh(Some(pens[0].inner), meshes);
            return IDLE;
        }
        let arrived_out = self.target.is_some_and(|t| hdist(p, t) < 3.0) && self.timer < 1;
        if self.target.is_none() || arrived_out || w.fresh_block() {
            self.target = w.mesh.and_then(|i| meshes.get(i)).and_then(|m| random_centroid(m, rng));
            self.timer = rng.range(120, 400);
            w.set_target();
        }
        let Some(t) = self.target else { return IDLE };
        let dist = hdist(p, t);
        if dist < 3.0 {
            self.timer -= 1;
            return IDLE;
        }
        self.run = dist > 10.0;
        match w.nav_step(meshes, t, hero) {
            Gait::Move => {
                if self.run {
                    RUN
                } else {
                    WALK
                }
            }
            Gait::Idle => IDLE,
        }
    }

    fn flee_tick(&mut self, w: &mut Walker, meshes: &[NavMesh], hero: Vec3, rng: &mut Rng) -> u32 {
        let p = w.position();
        let mesh = w.mesh.and_then(|i| meshes.get(i));
        if self.flee_step == 1 && w.blocked {
            // The blocked latch set during the direct-away walk: abandon it.
            self.flee_step = 0;
            w.blocked = false;
        }
        let done = self.target.is_none_or(|t| hdist(p, t) < if self.flee_step == 1 { 2.5 } else { 2.0 })
            || self.flee_ticks == 0;
        // A sub-step ending by timer or arrival clears the latch.
        if self.flee_step != 0 && done {
            w.blocked = false;
        }
        if self.flee_step == 0 || done {
            let away = [p[0] - hero[0], 0.0, p[2] - hero[2]];
            let len = (away[0] * away[0] + away[2] * away[2]).sqrt().max(1e-6);
            let a = [p[0] + 4.0 * away[0] / len, p[1], p[2] + 4.0 * away[2] / len];
            // The path test: the candidate lies on a triangle of the current mesh (edges
            // inclusive, only that mesh), and the latch is clear.
            let clear = mesh.is_some_and(|m| m.locate_full(a).is_some());
            if clear && !w.blocked {
                self.flee_step = 1;
                self.flee_ticks = 15;
                self.target = Some(a);
            } else {
                // Safe point: the far centroid pick (SPEC-0056; SPEC-0029 Behavior 28): up to T
                // uniform triangle draws, the first centroid more than 10 from the hero, else the
                // last one drawn.
                let safe = mesh.filter(|m| !m.is_empty()).and_then(|m| {
                    let mut last = None;
                    for _ in 0..m.len() {
                        let c = m.centroid(rng.index(m.len() as u32) as usize);
                        last = Some(c);
                        if hdist(c, hero) > 10.0 {
                            return Some(c);
                        }
                    }
                    last
                });
                self.flee_step = 2;
                self.flee_ticks = 25;
                self.target = safe.or(Some(a));
            }
            w.set_target();
        }
        self.flee_ticks = self.flee_ticks.saturating_sub(1);
        let Some(t) = self.target else { return IDLE };
        match w.nav_step(meshes, t, hero) {
            Gait::Move => RUN,
            Gait::Idle => IDLE,
        }
    }

    fn lead_tick(&mut self, w: &mut Walker, meshes: &[NavMesh], pens: &[Pen], hero: Vec3) -> u32 {
        let Some(pen) = self.pen.and_then(|i| pens.get(i)).copied() else {
            self.mode = Mode::Wander;
            return IDLE;
        };
        let p = w.position();
        if hdist(p, pen.point) < 3.0 {
            if Self::inside(meshes, pen.inner, p) {
                self.mode = Mode::Penned;
                self.target = None;
                w.assign_mesh(Some(pen.inner), meshes);
                return IDLE;
            }
            self.timer = 30;
            w.set_target();
        }
        match w.nav_step(meshes, pen.point, hero) {
            Gait::Move => RUN,
            Gait::Idle => IDLE,
        }
    }

    fn penned_tick(&mut self, w: &mut Walker, meshes: &[NavMesh], pens: &[Pen], hero: Vec3, rng: &mut Rng) -> u32 {
        let Some(pen) = self.pen.and_then(|i| pens.get(i)).copied() else { return IDLE };
        let p = w.position();
        if self.timer > 0 {
            self.timer -= 1;
            return IDLE;
        }
        let reached = self.target.is_none_or(|t| hdist(p, t) < 1.5) || w.fresh_block();
        if reached && self.target.is_some() {
            self.target = None;
            self.timer = rng.range(120, 400);
            return IDLE;
        }
        if self.target.is_none() {
            self.target = meshes.get(pen.inner).and_then(|m| random_centroid(m, rng));
            w.set_target();
        }
        let Some(t) = self.target else { return IDLE };
        match w.nav_step(meshes, t, hero) {
            Gait::Move => WALK,
            Gait::Idle => IDLE,
        }
    }
}

/// The group count after one member's update (SPEC-0049 item 6): with three members, set flag
/// 107 + p, p the number penned (a flag already on is not set again).
pub fn group_count(state: &mut GameState, members: usize, penned: usize) {
    if members == 3 {
        state.set(107 + penned.min(3) as u16, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_up_flags_advance_one_member_at_a_time() {
        // SPEC-0049 test vector 8.
        let mut s = GameState::default();
        group_count(&mut s, 3, 0);
        assert!(s.flag(107));
        group_count(&mut s, 3, 1);
        group_count(&mut s, 3, 2);
        assert!(s.flag(108) && s.flag(109) && !s.flag(110));
        group_count(&mut s, 3, 3);
        assert!(s.flag(110));
        // Two members: nothing is set.
        let mut s = GameState::default();
        group_count(&mut s, 2, 2);
        assert!(s.flags.is_empty());
    }

    /// One flat square mesh (as in the walker tests), plus a one-triangle pen inside it.
    fn meshes() -> Vec<NavMesh> {
        let square = NavMesh {
            kind: 1,
            triangles: vec![
                [[0.0, 0.0, 0.0], [40.0, 0.0, 0.0], [40.0, 0.0, 40.0]],
                [[0.0, 0.0, 0.0], [40.0, 0.0, 40.0], [0.0, 0.0, 40.0]],
            ],
            neighbours: vec![[-1, -1, 1], [0, -1, -1]],
            table: vec![0b0000_1000],
        };
        let pen = NavMesh {
            kind: 1,
            triangles: vec![[[30.0, 0.0, 30.0], [40.0, 0.0, 30.0], [40.0, 0.0, 40.0]]],
            neighbours: vec![[-1, -1, -1]],
            table: vec![0],
        };
        vec![pen, square]
    }

    #[test]
    fn modes_follow_the_hero_distance_thresholds() {
        // SPEC-0029: hero at 4.9 → flee; at 5.0 stays in wander. Flee end above 8.0.
        let m = meshes();
        let mut rng = Rng::seeded(1);
        let mut w = Walker::new([10.0, 0.0, 10.0], [1.0, 0.0, 0.0], 1.0, 5.5, 2);
        let mut pens: Vec<Pen> = Vec::new();
        let mut wh = Whinny::default();
        wh.tick(&mut w, &m, &mut pens, [15.0, 0.0, 10.0], &mut rng);
        assert_eq!(wh.mode, Mode::Wander);
        wh.tick(&mut w, &m, &mut pens, [14.9, 0.0, 10.0], &mut rng);
        assert_eq!(wh.mode, Mode::Flee);
        wh.tick(&mut w, &m, &mut pens, [18.0, 0.0, 10.0], &mut rng);
        assert_eq!(wh.mode, Mode::Flee, "8.0 keeps fleeing");
        wh.tick(&mut w, &m, &mut pens, [18.1, 0.0, 10.0], &mut rng);
        assert_eq!(wh.mode, Mode::Wander);
    }

    #[test]
    fn inside_the_inner_area_with_one_pen_is_penned() {
        let m = meshes();
        let mut rng = Rng::seeded(1);
        let mut w = Walker::new([38.0, 0.0, 32.0], [1.0, 0.0, 0.0], 1.0, 5.5, 2);
        let mut pens = vec![Pen { inner: 0, entrance: 0, point: [36.0, 0.0, 34.0], occupied: false }];
        let mut wh = Whinny::default();
        wh.tick(&mut w, &m, &mut pens, [0.0, 0.0, 0.0], &mut rng);
        assert!(wh.penned());
        assert_eq!(w.mesh, Some(0), "becoming penned sets the inner mesh");
    }

    /// SPEC-0029 Behavior 5 test vector: a Whinny on its start mesh that stands inside the
    /// entrance area while fleeing gets the entrance mesh at once and leads to the pen point on
    /// it, even though its start mesh does not hold the pen point.
    #[test]
    fn fleeing_into_the_entrance_leads_on_the_entrance_mesh() {
        let ground = |tris: Vec<[[f32; 3]; 3]>, neighbours, table| NavMesh { kind: 1, triangles: tris, neighbours, table };
        // Start mesh: x 0 to 20. Entrance mesh (kind 0): x 15 to 40, holding the pen point.
        let start = ground(
            vec![[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0], [20.0, 0.0, 20.0]], [[0.0, 0.0, 0.0], [20.0, 0.0, 20.0], [0.0, 0.0, 20.0]]],
            vec![[-1, -1, 1], [0, -1, -1]],
            vec![0b0000_1000],
        );
        let mut entrance = ground(
            vec![[[15.0, 0.0, 0.0], [40.0, 0.0, 0.0], [40.0, 0.0, 20.0]], [[15.0, 0.0, 0.0], [40.0, 0.0, 20.0], [15.0, 0.0, 20.0]]],
            vec![[-1, -1, 1], [0, -1, -1]],
            vec![0b0000_1000],
        );
        entrance.kind = 0;
        let inner = NavMesh { kind: 0, triangles: vec![[[32.0, 0.0, 2.0], [40.0, 0.0, 2.0], [40.0, 0.0, 18.0]]], neighbours: vec![[-1, -1, -1]], table: vec![0] };
        let m = vec![inner, start, entrance];
        let mut rng = Rng::seeded(1);
        let mut w = Walker::new([17.0, 0.0, 10.0], [1.0, 0.0, 0.0], 1.0, 5.5, 2);
        let mut pens = vec![Pen { inner: 0, entrance: 2, point: [37.0, 0.0, 10.0], occupied: false }];
        let mut wh = Whinny::default();
        let hero = [13.0, 0.0, 10.0];
        wh.tick(&mut w, &m, &mut pens, hero, &mut rng);
        assert_eq!(w.mesh, Some(1), "the start pick takes the first ground mesh");
        wh.tick(&mut w, &m, &mut pens, hero, &mut rng);
        assert_eq!(wh.mode, Mode::Lead);
        assert_eq!(w.mesh, Some(2), "the entrance mesh is set directly");
        let mut penned_at = None;
        for t in 0..900 {
            w.motion_tick([0.0, 0.0, 0.3], &m, |_, _, _| (0.0, [0.0, 1.0, 0.0]));
            wh.tick(&mut w, &m, &mut pens, hero, &mut rng);
            if wh.penned() {
                penned_at = Some(t);
                break;
            }
        }
        assert!(penned_at.is_some(), "the lead reaches the pen: {:?} {:?}", wh.mode, w.position());
    }
}
