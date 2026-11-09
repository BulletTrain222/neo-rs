//! The Miamouse (class 21; SPEC-0029 Behavior 28): the dispatch (notice, sneak cone and the
//! flee entry), wander and pause with its emote, flee, the released and landing state with the
//! cage and field tests, the delivered state, and the global catch switch, meshes and count
//! with the delivery step on the level states.
//!
//! The movement clips are the generic walk (2) and run (3) of the character set (the clip of
//! each movement mode is not named; unverified).

use crate::nav::NavMesh;
use crate::rng::Rng;
use crate::walker::{Gait, Walker};

type Vec3 = [f32; 3];

pub const CLIP_IDLE: u32 = 0;
pub const CLIP_HELD: u32 = 12;
pub const CLIP_RELEASED: u32 = 13;
pub const CLIP_LANDING: u32 = 19;
pub const CLIP_EMOTE: u32 = 20;
const CLIP_WALK: u32 = 2;
const CLIP_RUN: u32 = 3;

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The catch switch, the field and cage mesh ids and the count (global to the game).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Catch {
    pub on: bool,
    pub field: i32,
    pub cage: i32,
    pub count: u32,
}

impl Default for Catch {
    fn default() -> Self {
        Self { on: false, field: -1, cage: -1, count: 0 }
    }
}

impl Catch {
    /// InitMiamouseBehavior(m1, m2): field, cage, catching on, count 0.
    pub fn init(&mut self, field: i32, cage: i32) {
        *self = Self { on: true, field, cage, count: 0 };
    }

    /// Whether a point is in the triangles of mesh `id` (the inside test; -1 or out of range:
    /// never).
    pub fn in_mesh(meshes: &[NavMesh], id: i32, p: Vec3) -> bool {
        usize::try_from(id).ok().and_then(|i| meshes.get(i)).is_some_and(|m| m.locate_full(p).is_some())
    }

    /// The delivery step: count + 1, then the lowest level state k of 0 to 4 that is on gives
    /// state k + 1 on and k off. `level` reads and `set` writes the level states.
    pub fn deliver(&mut self, level: &dyn Fn(u16) -> bool, set: &mut dyn FnMut(u16, bool)) {
        self.count += 1;
        if let Some(k) = (0..=4).find(|&k| level(k)) {
            set(k + 1, true);
            set(k, false);
        }
    }
}

/// The far centroid pick: up to N draws of a random triangle's centroid, stopping at the first
/// more than r from p; the last drawn otherwise; p with no mesh.
pub fn far_pick(m: Option<&NavMesh>, p: Vec3, r: f32, rng: &mut Rng) -> Vec3 {
    let Some(m) = m.filter(|m| !m.is_empty()) else { return p };
    let mut last = p;
    for _ in 0..m.len() {
        last = m.centroid(rng.index(m.len() as u32) as usize);
        if hdist(last, p) > r {
            return last;
        }
    }
    last
}

/// What the Miamouse sees.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub hero: Vec3,
    /// The hero's action state (1 standing; 22, 39 and 40 sneaking).
    pub hero_state: u8,
    pub clip: u32,
    pub progress: f32,
    /// The ground gap (0 with the floor flag set).
    pub gap: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Miamouse {
    /// Main state: 0 idle, 1 wander and pause, 2 flee, 3 released and landing, 4 delivered.
    pub state: u8,
    pub sub: u8,
    timer: i32,
    dest: Option<Vec3>,
    pub catchable: bool,
    pub home: Vec3,
    /// The delivery happened on this tick (the host runs the delivery step).
    pub delivered: bool,
}

impl Miamouse {
    pub fn new(home: Vec3) -> Self {
        Self { catchable: true, home, ..Self::default() }
    }

    /// The flee entry: state 2, a radius-30 far pick of its mesh from the hero.
    fn flee(&mut self, w: &Walker, meshes: &[NavMesh], hero: Vec3, rng: &mut Rng) {
        self.state = 2;
        self.sub = 0;
        self.timer = 0;
        self.dest = Some(far_pick(w.mesh.and_then(|i| meshes.get(i)), hero, 30.0, rng));
    }

    /// One tick: the dispatch, then the state. Returns the clip to play.
    pub fn tick(&mut self, w: &mut Walker, meshes: &[NavMesh], c: &Catch, v: &View, rng: &mut Rng) -> u32 {
        self.delivered = false;
        let p = w.position();
        // Dispatch.
        if v.clip == CLIP_RELEASED {
            self.state = 3;
        }
        if !matches!(self.state, 2..=4) {
            let d = hdist(p, v.hero);
            if v.hero_state != 1 && d < 10.0 {
                let sneaking = matches!(v.hero_state, 22 | 39 | 40);
                let u = [(v.hero[0] - p[0]) / d.max(1e-6), 0.0, (v.hero[2] - p[2]) / d.max(1e-6)];
                let f = w.facing;
                if !sneaking || f[0] * u[0] + f[2] * u[2] > 0.5403 {
                    self.flee(w, meshes, v.hero, rng);
                }
            }
            if self.state == 0 {
                self.state = 1;
                self.sub = 0;
            }
        }
        match self.state {
            1 => {
                self.catchable = true;
                self.wander(w, meshes, v, rng)
            }
            2 => {
                self.catchable = false;
                self.flee_step(w, meshes, v, rng)
            }
            3 => {
                self.catchable = false;
                self.landing(w, meshes, c, v, rng)
            }
            4 => {
                let clip = self.wander(w, meshes, v, rng);
                self.catchable = false;
                clip
            }
            _ => CLIP_IDLE,
        }
    }

    fn wander(&mut self, w: &mut Walker, meshes: &[NavMesh], v: &View, rng: &mut Rng) -> u32 {
        if self.timer > 0 {
            self.timer -= 1;
        }
        match self.sub {
            0 => {
                self.timer = 0;
                self.sub = 1;
                CLIP_IDLE
            }
            1 if self.timer > 0 => {
                if self.timer % 90 == 0 {
                    self.sub = 2;
                }
                CLIP_IDLE
            }
            1 => {
                let m = w.mesh.and_then(|i| meshes.get(i)).filter(|m| !m.is_empty());
                self.dest = Some(m.map_or(self.home, |m| m.centroid(rng.index(m.len() as u32) as usize)));
                w.set_target();
                self.sub = 3;
                CLIP_IDLE
            }
            2 => {
                if v.clip == CLIP_EMOTE && v.progress >= 0.99 {
                    self.sub = 1;
                }
                CLIP_EMOTE
            }
            _ => {
                let t = self.dest.unwrap_or(self.home);
                if hdist(w.position(), t) < 1.0 {
                    self.timer = rng.range(120, 210);
                    self.sub = 1;
                    return CLIP_IDLE;
                }
                match w.nav_step(meshes, t, v.hero) {
                    Gait::Move => CLIP_WALK,
                    Gait::Idle => CLIP_IDLE,
                }
            }
        }
    }

    fn flee_step(&mut self, w: &mut Walker, meshes: &[NavMesh], v: &View, rng: &mut Rng) -> u32 {
        let p = w.position();
        let d = hdist(p, v.hero);
        let mesh = w.mesh.and_then(|i| meshes.get(i));
        let touched = w.blocked;
        match self.sub {
            0 => {
                if touched {
                    w.blocked = false;
                    self.sub = 1;
                    self.dest = Some(far_pick(mesh, v.hero, 10.0, rng));
                    return CLIP_IDLE;
                }
                if d <= 20.0 {
                    let a = [p[0] + 4.0 * (p[0] - v.hero[0]) / d.max(1e-6), p[1], p[2] + 4.0 * (p[2] - v.hero[2]) / d.max(1e-6)];
                    if mesh.is_some_and(|m| m.locate_full(a).is_some()) {
                        self.dest = Some(a);
                    } else {
                        self.sub = 1;
                        self.dest = Some(far_pick(mesh, v.hero, 10.0, rng));
                    }
                } else {
                    self.sub = 2;
                    self.dest = Some(far_pick(mesh, v.hero, 30.0, rng));
                }
            }
            1 | 2 => {
                let arrived = self.dest.is_none_or(|t| hdist(p, t) < 0.5) || touched;
                if arrived {
                    w.blocked = false;
                    if self.sub == 2 {
                        self.state = 0;
                        self.sub = 0;
                        return CLIP_IDLE;
                    }
                    if d < 20.0 {
                        self.sub = 0;
                    } else {
                        self.sub = 2;
                        self.dest = Some(far_pick(mesh, v.hero, 30.0, rng));
                    }
                }
            }
            _ => self.sub = 0,
        }
        let t = self.dest.unwrap_or(p);
        match w.nav_step(meshes, t, v.hero) {
            Gait::Move => CLIP_RUN,
            Gait::Idle => CLIP_IDLE,
        }
    }

    /// The release (from the carry): clip 13, the cage mesh when the held point is in the cage
    /// zone (else the field), placed at the held point + 1.0 x the hero's forward.
    pub fn release(&mut self, w: &mut Walker, meshes: &[NavMesh], c: &Catch, held: Vec3, forward: Vec3) {
        let mesh = if Catch::in_mesh(meshes, c.cage, held) { c.cage } else { c.field };
        w.assign_mesh(usize::try_from(mesh).ok(), meshes);
        w.place([held[0] + forward[0], held[1], held[2] + forward[2]]);
        self.state = 3;
    }

    /// State 3: while clip 13 plays, a cage landing hops between cage centroids; off both meshes
    /// it goes home; with the gap below 0.25 the landing clip 19 plays. Once it has passed 0.99:
    /// in the cage, state 4 and the delivery; else the field mesh and the flee entry.
    fn landing(&mut self, w: &mut Walker, meshes: &[NavMesh], c: &Catch, v: &View, rng: &mut Rng) -> u32 {
        let p = w.position();
        let in_cage = Catch::in_mesh(meshes, c.cage, p);
        let cage = usize::try_from(c.cage).ok().and_then(|i| meshes.get(i));
        if v.clip == CLIP_RELEASED {
            if in_cage {
                w.assign_mesh(usize::try_from(c.cage).ok(), meshes);
                let t = far_pick(cage, p, 3.0, rng);
                w.place([t[0], p[1], t[2]]);
            } else if !Catch::in_mesh(meshes, c.field, p) {
                w.place(self.home);
            }
            return if v.gap < 0.25 { CLIP_LANDING } else { CLIP_RELEASED };
        }
        if v.progress > 0.99 {
            if in_cage {
                w.assign_mesh(usize::try_from(c.cage).ok(), meshes);
                let t = far_pick(cage, p, 3.0, rng);
                w.place([t[0], p[1], t[2]]);
                self.state = 4;
                self.sub = 0;
                self.delivered = true;
                return CLIP_IDLE;
            }
            w.assign_mesh(usize::try_from(c.field).ok(), meshes);
            self.flee(w, meshes, v.hero, rng);
            return CLIP_IDLE;
        }
        v.clip
    }
}

/// The catch prompt's candidate test (SPEC-0029 Behavior 28): catching on, catchable, a
/// horizontal distance below 4.0 and a facing dot above 0.5.
pub fn catchable(c: &Catch, m: &Miamouse, at: Vec3, hero: Vec3, hero_facing: Vec3) -> bool {
    let d = hdist(at, hero);
    if !c.on || !m.catchable || d >= 4.0 || d <= 0.0 {
        return false;
    }
    let u = [(at[0] - hero[0]) / d, (at[2] - hero[2]) / d];
    hero_facing[0] * u[0] + hero_facing[2] * u[1] > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_prompt_vectors() {
        // SPEC-0029 Behavior 28: hero at the origin facing +z, catching on.
        let mut c = Catch::default();
        let m = Miamouse::new([0.0; 3]);
        let f = [0.0, 0.0, 1.0];
        assert!(!catchable(&c, &m, [0.0, 0.0, 3.9], [0.0; 3], f), "catching off");
        c.init(0, 1);
        assert!(catchable(&c, &m, [0.0, 0.0, 3.9], [0.0; 3], f));
        assert!(!catchable(&c, &m, [0.0, 0.0, 4.0], [0.0; 3], f));
        assert!(catchable(&c, &m, [2.0, 0.0, 3.0], [0.0; 3], f));
        assert!(!catchable(&c, &m, [3.0, 0.0, 1.5], [0.0; 3], f));
        let fleeing = Miamouse { catchable: false, ..m };
        assert!(!catchable(&c, &fleeing, [0.0, 0.0, 2.0], [0.0; 3], f));
    }

    #[test]
    fn delivery_step_vectors() {
        // Level states {0}: five deliveries give {1} to {5}, counts 1 to 5; a sixth only counts.
        let mut c = Catch::default();
        c.init(0, 1);
        let states = std::cell::RefCell::new(std::collections::BTreeSet::from([0u16]));
        for n in 1..=6u16 {
            let level = |k: u16| states.borrow().contains(&k);
            let mut set = |k: u16, on: bool| {
                if on {
                    states.borrow_mut().insert(k);
                } else {
                    states.borrow_mut().remove(&k);
                }
            };
            c.deliver(&level, &mut set);
            assert_eq!(*states.borrow(), std::collections::BTreeSet::from([n.min(5)]));
        }
        assert_eq!(c.count, 6);
    }
}
