//! Mortog (behaviour class 39; SPEC-0029 Behavior 3): idle, wander, flee and hold, with the
//! level's zone (InitMortogBehavior: the centre A at a locator and the squared radius L from two
//! locators, the hold range; ChangeMortogTimes), the shared zone count (NumMortogsOnTarget), the
//! run rule, the hold end's goal with no position term and the wander zone rule with 2L and 3L.
//! "Pick a random point for wandering" is read as the Whinny's rule, a uniformly drawn triangle
//! centroid of its mesh, and the flee path test as the away point lying on its mesh (Medium).

use crate::hero::Vec3;
use crate::nav::NavMesh;
use crate::rng::Rng;
use crate::walker::{Gait, Walker};

const IDLE: u32 = 0;
const WALK: u32 = 2;
const RUN: u32 = 3;

/// The tuning words: 11 from 0x2DB214.
pub const BASE: u32 = 0x002D_B214;
pub const WORDS: usize = 11;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub hold: (f32, f32),
    pub idle: (i32, i32),
    pub arrival: f32,
    pub notice: f32,
    pub flee: f32,
    pub flee_distance: f32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let f = |at: u32| f32::from_bits(w(at));
        Some(Self {
            hold: (f(0x002D_B21C), f(0x002D_B220)),
            idle: (w(0x002D_B228) as i32, w(0x002D_B22C) as i32),
            arrival: f(0x002D_B230),
            notice: f(0x002D_B234),
            flee: f(0x002D_B238),
            flee_distance: f(0x002D_B23C),
        })
    }
}

/// The zone the level sets (SPEC-0030 rows 83 and 84).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub centre: Vec3,
    /// The squared horizontal radius L.
    pub l: f32,
    pub hold: (f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Wander,
    Flee,
    Hold,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mortog {
    pub state: State,
    idle: i32,
    hold: i32,
    goal: Option<Vec3>,
}

impl Default for Mortog {
    fn default() -> Self {
        Self { state: State::Idle, idle: 0, hold: 0, goal: None }
    }
}

fn hsq(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)
}

/// The three-dimensional unit vector from A to P.
fn unit3(p: Vec3, a: Vec3) -> Vec3 {
    let d = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if l > 0.0 { d.map(|c| c / l) } else { [0.0; 3] }
}

/// The hold end's move goal: k x unit(P - A), with no position term.
pub fn hold_goal(p: Vec3, a: Vec3, k: i32) -> Vec3 {
    unit3(p, a).map(|c| c * k as f32)
}

/// The wander zone rule: with the look-ahead 6 units along the facing below 2L (squared,
/// horizontal) of A, the goal P + 3L x unit(P - A); else none.
pub fn zone_goal(p: Vec3, facing: Vec3, z: &Zone) -> Option<Vec3> {
    let look = [p[0] + 6.0 * facing[0], p[1] + 6.0 * facing[1], p[2] + 6.0 * facing[2]];
    if hsq(look, z.centre) < 2.0 * z.l {
        let u = unit3(p, z.centre);
        Some(std::array::from_fn(|k| p[k] + 3.0 * z.l * u[k]))
    } else {
        None
    }
}

impl Mortog {
    fn go(&mut self, w: &mut Walker, meshes: &[NavMesh], t: Vec3, hero: Vec3, clip: u32) -> u32 {
        match w.nav_step(meshes, t, hero) {
            Gait::Move => clip,
            Gait::Idle => IDLE,
        }
    }

    fn to_idle(&mut self, words: &Words, rng: &mut Rng) {
        self.state = State::Idle;
        self.idle = 60 * rng.range(words.idle.0, words.idle.1);
    }

    /// One tick: the state, the walker's steering; returns the animation id.
    pub fn tick(&mut self, w: &mut Walker, meshes: &[NavMesh], zone: &Zone, words: &Words, count: &mut u8, hero: Vec3, rng: &mut Rng) -> u32 {
        let p = w.position();
        // The walker's mesh follows its position (re-picked when it leaves it).
        if w.mesh.and_then(|i| meshes.get(i)).is_none_or(|m| m.locate_full(p).is_none()) {
            let pick = meshes.iter().position(|m| m.kind == 1 && m.locate_full(p).is_some());
            if pick.is_some() {
                w.assign_mesh(pick, meshes);
            }
        }
        let hero_sq = hsq(p, hero);
        match self.state {
            State::Idle => {
                self.idle -= 1;
                if hero_sq < words.notice {
                    self.state = State::Flee;
                } else if self.idle < 1 {
                    self.goal = w.mesh.and_then(|i| meshes.get(i)).filter(|m| !m.is_empty()).map(|m| m.centroid(rng.index(m.len() as u32) as usize));
                    self.state = State::Wander;
                    w.set_target();
                }
                IDLE
            }
            State::Wander => {
                if hero_sq < words.notice {
                    self.state = State::Flee;
                    return IDLE;
                }
                let Some(goal) = self.goal else {
                    self.to_idle(words, rng);
                    return IDLE;
                };
                if hsq(p, goal) < words.arrival {
                    self.to_idle(words, rng);
                    return IDLE;
                }
                if let Some(g) = zone_goal(p, w.facing, zone) {
                    self.goal = Some(g);
                    w.set_target();
                }
                let g = self.goal.unwrap_or(goal);
                self.go(w, meshes, g, hero, WALK)
            }
            State::Flee => self.flee(w, meshes, zone, words, count, hero, rng),
            State::Hold => {
                self.hold -= 1;
                if self.hold < 1 && zone.hold.1 != 0.0 {
                    self.goal = Some(hold_goal(p, zone.centre, rng.range(4, 8)));
                    self.state = State::Wander;
                    *count = count.saturating_sub(1);
                    w.set_target();
                }
                IDLE
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn flee(&mut self, w: &mut Walker, meshes: &[NavMesh], zone: &Zone, words: &Words, count: &mut u8, hero: Vec3, rng: &mut Rng) -> u32 {
        let p = w.position();
        // The zone test, then the flee step in the same tick.
        if hsq(p, zone.centre) < zone.l {
            self.state = State::Hold;
            self.hold = (60.0 * rng.frange(zone.hold.0, zone.hold.1)).round() as i32;
            *count = count.saturating_add(1);
        }
        let hero_sq = hsq(p, hero);
        let away = {
            let d = [p[0] - hero[0], p[2] - hero[2]];
            let l = d[0].hypot(d[1]).max(1e-6);
            [p[0] + words.flee_distance * d[0] / l, p[1], p[2] + words.flee_distance * d[1] / l]
        };
        let clear = w.mesh.and_then(|i| meshes.get(i)).is_some_and(|m| m.locate_full(away).is_some());
        if hero_sq < words.flee && clear {
            w.set_target();
            let clip = if hero_sq > 112.5 { WALK } else { RUN };
            if self.state == State::Hold {
                return IDLE;
            }
            return self.go(w, meshes, away, hero, clip);
        }
        self.to_idle(words, rng);
        IDLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_end_and_zone_rule() {
        // The hold end has no position term.
        let g = hold_goal([100.0, 0.0, 50.0], [96.0, 0.0, 47.0], 4);
        assert!((g[0] - 3.2).abs() < 1e-5 && (g[2] - 2.4).abs() < 1e-5);
        let g = hold_goal([3.0, 4.0, 0.0], [0.0; 3], 5);
        assert!((g[0] - 3.0).abs() < 1e-5 && (g[1] - 4.0).abs() < 1e-5);
        // The zone rule: L 25, A at the origin, P (4, 0, 0): facing +x no change; facing -x the
        // goal (79, 0, 0).
        let z = Zone { centre: [0.0; 3], l: 25.0, hold: (10.0, 20.0) };
        assert_eq!(zone_goal([4.0, 0.0, 0.0], [1.0, 0.0, 0.0], &z), None);
        assert_eq!(zone_goal([4.0, 0.0, 0.0], [-1.0, 0.0, 0.0], &z), Some([79.0, 0.0, 0.0]));
        // L 4, the look-ahead at 7.9 below 2L = 8: P plus 12 along the unit from A.
        let z = Zone { centre: [0.0; 3], l: 4.0, hold: (10.0, 20.0) };
        assert_eq!(zone_goal([8.0, 0.0, 0.0], [-1.0, 0.0, 0.0], &z), Some([20.0, 0.0, 0.0]));
    }

    #[test]
    fn zone_count_and_holds() {
        let words = Words::read(&{
            let mut b = vec![0u32; WORDS];
            let mut put = |at: u32, v: u32| b[((at - BASE) / 4) as usize] = v;
            put(0x002D_B21C, 10.0f32.to_bits());
            put(0x002D_B220, 20.0f32.to_bits());
            put(0x002D_B228, 2);
            put(0x002D_B22C, 8);
            for (at, v) in [(0x002D_B230, 4.0f32), (0x002D_B234, 25.0), (0x002D_B238, 225.0), (0x002D_B23C, 8.0)] {
                put(at, v.to_bits());
            }
            b
        })
        .unwrap();
        let mut rng = Rng::seeded(1);
        let meshes: Vec<NavMesh> = Vec::new();
        let zone = Zone { centre: [0.0; 3], l: 25.0, hold: (12.5, 12.5) };
        // A fleeing Mortog at squared distance 24.99 holds (750 ticks at 12.5 s); 25.0 does not.
        let mut w = Walker::new([4.999, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, 1.0, 1);
        let mut m = Mortog { state: State::Flee, ..Mortog::default() };
        let mut count = 0;
        m.tick(&mut w, &meshes, &zone, &words, &mut count, [5.0, 0.0, 3.0], &mut rng);
        assert_eq!((m.hold, count), (750, 1));
        // The flee step has no clear path (no mesh): the state goes idle with the count kept.
        assert_eq!(m.state, State::Idle);
        let mut w = Walker::new([5.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, 1.0, 1);
        let mut m = Mortog { state: State::Flee, ..Mortog::default() };
        let mut count = 0;
        m.tick(&mut w, &meshes, &zone, &words, &mut count, [5.0, 0.0, 3.0], &mut rng);
        assert_eq!(count, 0);
        // A hold ends with the launch and the count falls; with the hold maximum 0 it never ends.
        let mut m = Mortog { state: State::Hold, hold: 1, ..Mortog::default() };
        let mut count = 1;
        m.tick(&mut w, &meshes, &zone, &words, &mut count, [90.0, 0.0, 0.0], &mut rng);
        assert_eq!((m.state, count), (State::Wander, 0));
        let stuck = Zone { hold: (0.0, 0.0), ..zone };
        let mut m = Mortog { state: State::Hold, hold: 1, ..Mortog::default() };
        let mut count = 1;
        for _ in 0..10 {
            m.tick(&mut w, &meshes, &stuck, &words, &mut count, [90.0, 0.0, 0.0], &mut rng);
        }
        assert_eq!((m.state, count), (State::Hold, 1));
    }
}
