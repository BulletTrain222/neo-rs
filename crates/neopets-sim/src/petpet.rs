//! Petpets (SPEC-0047): the pet type to kind tables, the kind multipliers and the meal time
//! update, the passive bonuses to the hero, the pending save values and the summoned petpet's
//! class 34 controller (state choice, idle-hero follow, moving follow with its three gaits, the
//! low-time one-shot and the leave).
//!
//! Not covered: the class 33 wild petpet loop, class 20, the feeding interaction and the food
//! reservation (fed food, strike and fallback), the bumped recovery of class 34 state 3 (it
//! holds animation 0) and the nearby-character response of state 1 (its filter is open).

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The primary pet type table: the kind of a pet type id (activation, restore, the query).
pub fn kind(type_id: i32) -> Option<u8> {
    match type_id {
        450 => Some(0),
        703 => Some(1),
        1314 => Some(3),
        1460 => Some(6),
        2090 => Some(7),
        2094 => Some(8),
        _ => None,
    }
}

/// The second table (a character check only): kinds 2, 4, 5 and 9.
pub fn second_kind(type_id: i32) -> Option<u8> {
    match type_id {
        805 => Some(2),
        1350 => Some(4),
        1368 => Some(5),
        2335 => Some(9),
        _ => None,
    }
}

/// The archive package of kind k: resource 534 + k.
pub fn package(kind: u8) -> u32 {
    534 + kind as u32
}

/// The membership sets A, B and C (boot data 0x315C88, 0x315CC8 and 0x315CA8; -1 unused).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sets {
    pub a: Vec<i32>,
    pub b: Vec<i32>,
    pub c: Vec<i32>,
}

pub const SET_A: u32 = 0x0031_5C88;
pub const SET_B: u32 = 0x0031_5CC8;
pub const SET_C: u32 = 0x0031_5CA8;

impl Sets {
    fn has(set: &[i32], code: i32) -> bool {
        code != -1 && set.contains(&code)
    }

    /// The kind multiplier for a target type code (Layout, Kind multipliers).
    pub fn multiplier(&self, kind: u8, code: i32) -> f32 {
        let a = Self::has(&self.a, code);
        let b = Self::has(&self.b, code);
        let c = Self::has(&self.c, code);
        match kind {
            0 if a || c => 1.5,
            1 if b || c => 1.5,
            3 if b || c => 1.5,
            3 if a || c => 0.75,
            6 if a || c => 1.5,
            6 if b || c => 0.75,
            7 if a || b || c => 0.5,
            _ => 1.0,
        }
    }
}

/// A meal's time in ticks: trunc(v x 60 x 60) for a value v in minutes.
pub fn meal_ticks(v: f32) -> i32 {
    (v * 3600.0) as i32
}

/// The words of the passive bonuses (0x2DD8DC to 0x2DD8E8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bonus {
    pub damage: f32,
    pub intake: f32,
    pub hp: f32,
    pub mp: f32,
}

pub const BONUS_AT: u32 = 0x002D_D8DC;

impl Default for Bonus {
    fn default() -> Self {
        Self { damage: 0.0, intake: 0.0, hp: 1.0, mp: 1.0 }
    }
}

/// The live petpet's presence for the bonuses, the query, the save and the meter: a kind and T
/// of at least 1.
fn present(kind: Option<u8>, t: i32) -> Option<u8> {
    kind.filter(|_| t >= 1)
}

impl Bonus {
    /// Factor E of the hero's outgoing damage: 1 + 0.25 for kinds 0 and 7.
    pub fn damage_factor(&self, kind: Option<u8>, t: i32) -> f32 {
        if matches!(present(kind, t), Some(0 | 7)) { 1.0 + self.damage } else { 1.0 }
    }

    /// The intake factor on enemy hits taken: 1 - 0.1 for kinds 1 and 7.
    pub fn intake_factor(&self, kind: Option<u8>, t: i32) -> f32 {
        if matches!(present(kind, t), Some(1 | 7)) { 1.0 - self.intake } else { 1.0 }
    }

    /// The regeneration test: T at least 1, the kind test, and T mod trunc(60 c) = 0.
    fn regen(kind: Option<u8>, t: i32, c: f32, kinds: [u8; 2]) -> i32 {
        let n = (60.0 * c) as i32;
        match present(kind, t) {
            Some(k) if kinds.contains(&k) && n > 0 && t % n == 0 => 1,
            _ => 0,
        }
    }

    /// The petpet hit point this hero update (kinds 6 and 7).
    pub fn hp_point(&self, kind: Option<u8>, t: i32) -> i32 {
        Self::regen(kind, t, self.hp, [6, 7])
    }

    /// The petpet magic point this hero update (kinds 3 and 7).
    pub fn mp_point(&self, kind: Option<u8>, t: i32) -> i32 {
        Self::regen(kind, t, self.mp, [3, 7])
    }
}

/// The save fields (slot 0x214, 0x218, 0x21C) of a live petpet: the type is -1 with no pet or T
/// below 1; T is 0 with no pet.
pub fn save_fields(live: Option<(i32, i32, i32)>) -> (i32, i32, i32) {
    match live {
        Some((ty, t, f)) if t >= 1 => (ty, t, f),
        Some((_, t, f)) => (-1, t.max(0), f),
        None => (-1, 0, 0),
    }
}

/// A fresh spawn's point: 7 units ahead of the hero along its facing.
pub fn spawn_point(hero: Vec3, facing: Vec3) -> Vec3 {
    [hero[0] + 7.0 * facing[0], hero[1], hero[2] + 7.0 * facing[2]]
}

/// What the hero and world look like to the controller this tick.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero's right vector (the second basis vector of the moving follow; Medium).
    pub hero_right: Vec3,
    /// The hero is idle (state 1 is chosen) rather than moving (state 2).
    pub hero_idle: bool,
    /// The game-state code is 0 (T counts down).
    pub counting: bool,
    pub contact: bool,
    /// The playing clip has ended.
    pub clip_done: bool,
}

/// What the controller asks for this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Walk toward a point, turning with a weight.
    pub toward: Option<(Vec3, f32)>,
    /// The petpet leaves (it is removed).
    pub leave: bool,
}

/// The summoned petpet (class 34).
#[derive(Clone, Debug, PartialEq)]
pub struct Live {
    pub type_id: i32,
    pub kind: u8,
    /// The remaining time T and the full time F in ticks.
    pub t: i32,
    pub f: i32,
    pub state: u8,
    low_done: bool,
    gait: u8,
    wait: Option<i32>,
    point: Option<Vec3>,
    pub pending_event: bool,
}

/// The low-time mark: the word 0x2DB1D8 (0.5 minute) x 60 x 60.
pub const LOW_TIME_WORD: u32 = 0x002D_B1D8;

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl Live {
    pub fn new(type_id: i32, t: i32, f: i32) -> Option<Self> {
        Some(Self { type_id, kind: kind(type_id)?, t, f, state: 0, low_done: false, gait: 0, wait: None, point: None, pending_event: false })
    }

    /// A petpet more than 60 units from the hero (after a world change) is placed again.
    pub fn far(at: Vec3, hero: Vec3) -> bool {
        hdist(at, hero) > 60.0
    }

    /// Present for the bonuses, the query, the save and the meter.
    pub fn present(&self) -> Option<u8> {
        present(Some(self.kind), self.t)
    }

    /// A meal of value v minutes: T and F become trunc(v x 3600) when that is above T; the
    /// low-time latch clears.
    pub fn eat(&mut self, v: f32) {
        let n = meal_ticks(v);
        if n > self.t {
            self.t = n;
            self.f = n;
        }
        self.low_done = false;
    }

    /// The HUD meter width: round half away from zero of T x (60 / F); 0 with F = 0.
    pub fn meter(&self) -> i32 {
        if self.f == 0 || self.t < 1 {
            return 0;
        }
        (self.t as f32 * (60.0 / self.f as f32)).round() as i32
    }

    /// One class 34 tick: the state choice, then the state body, then T falls by 1 while the
    /// game-state code is 0.
    pub fn tick(&mut self, v: &View, low_mark: i32, rng: &mut Rng) -> Act {
        // State choice.
        if v.contact && self.state != 5 {
            self.state = 3;
        } else if std::mem::take(&mut self.pending_event) {
            self.state = 4;
        } else if !matches!(self.state, 4 | 6) {
            if self.t >= low_mark || self.low_done {
                if self.t < 1 {
                    self.state = 5;
                } else {
                    let s = if v.hero_idle { 1 } else { 2 };
                    if s == 1 && self.state != 1 {
                        self.point = None;
                        self.wait = None;
                    }
                    self.state = s;
                }
            } else {
                self.state = 6;
                self.low_done = true;
            }
        }
        let mut a = Act::default();
        match self.state {
            1 => self.follow_idle(v, low_mark, rng, &mut a),
            2 => self.follow_moving(v, &mut a),
            3 => a.anim = Some(0),
            4 => {
                // Eating in place; no food is reserved here, so the state ends at once.
                a.anim = Some(4);
                self.state = 1;
            }
            5 => {
                let to = [v.hero[0] - v.position[0], 0.0, v.hero[2] - v.position[2]];
                let l = (to[0] * to[0] + to[2] * to[2]).sqrt().max(1e-6);
                let dot = (v.facing[0] * to[0] + v.facing[2] * to[2]) / l;
                // Removed once it faces away from the hero or touches something; how it turns
                // away is not given, so it holds animation 0 until then.
                a.leave = dot < 0.5 || v.contact;
                a.anim = Some(0);
            }
            6 => {
                a.anim = Some(7);
                if v.clip_done {
                    self.state = 1;
                }
            }
            _ => a.anim = Some(0),
        }
        if v.counting {
            self.t -= 1;
        }
        a
    }

    /// State 1: walk to the stored point while 1.0 or more away (animation 2, weight 0.1, the
    /// wait cleared); below 1.0 wait 180 to 600 ticks, then pick a new point 3 to 10 units off
    /// the hero on each horizontal axis with random signs.
    fn follow_idle(&mut self, v: &View, low_mark: i32, rng: &mut Rng, a: &mut Act) {
        let point = *self.point.get_or_insert_with(|| Self::near(v.hero, rng));
        if hdist(v.position, point) >= 1.0 {
            a.toward = Some((point, 0.1));
            a.anim = Some(2);
            self.wait = None;
            return;
        }
        match self.wait {
            None => {
                self.wait = Some(rng.range(180, 600));
                a.anim = Some(0);
            }
            Some(w) => {
                let w = w - 1;
                if w < 1 {
                    self.point = Some(Self::near(v.hero, rng));
                    self.wait = None;
                } else {
                    self.wait = Some(w);
                }
                let r = self.t as f32 / low_mark.max(1) as f32;
                a.anim = Some(if r < 1.0 {
                    7
                } else if (rng.range(0, 100) as f32) > 200.0 * (1.0 - r).abs() {
                    1
                } else {
                    6
                });
            }
        }
    }

    fn near(hero: Vec3, rng: &mut Rng) -> Vec3 {
        let mut off = || {
            let m = rng.frange(3.0, 10.0);
            if rng.range(0, 1) == 0 { -m } else { m }
        };
        let (x, z) = (off(), off());
        [hero[0] + x, hero[1], hero[2] + z]
    }

    /// State 2: the point 4.0 along the hero's facing and 1.5 along its right, with the three
    /// gaits (Medium for the two basis vectors).
    fn follow_moving(&mut self, v: &View, a: &mut Act) {
        let t = [
            v.hero[0] + 4.0 * v.hero_facing[0] + 1.5 * v.hero_right[0],
            v.hero[1],
            v.hero[2] + 4.0 * v.hero_facing[2] + 1.5 * v.hero_right[2],
        ];
        let (anim, gait) = gait(self.gait, hdist(v.position, t));
        self.gait = gait;
        a.anim = Some(anim);
        if gait != 0 {
            a.toward = Some((t, 0.1));
        }
    }
}

/// The moving follow's gait rule: (animation, next gait) for the current gait and distance.
pub fn gait(g: u8, d: f32) -> (u32, u8) {
    match g {
        0 if d <= 3.0 => (0, 0),
        0 => (2, 1),
        1 if d > 7.0 => (3, 2),
        1 if d < 1.5 => (0, 0),
        1 => (2, 1),
        _ if d < 5.0 => (2, 1),
        _ => (3, 2),
    }
}

/// The pending values the save load fills and the scene start reads (pet manager +0x40 and
/// +0x78 to +0x80).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pending {
    pub flag: bool,
    pub type_id: i32,
    pub t: i32,
    pub f: i32,
}

impl Default for Pending {
    fn default() -> Self {
        Self { flag: false, type_id: -1, t: 0, f: 0 }
    }
}

/// What the scene start does with the pending values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Restore {
    Nothing,
    Deactivate,
    /// Activate by type (a fresh spawn) with T and F.
    Spawn { type_id: i32, t: i32, f: i32 },
}

impl Pending {
    /// The save load: the slot's three fields become pending, with the load flag.
    pub fn load(type_id: i32, t: i32, f: i32) -> Self {
        Self { flag: true, type_id, t, f }
    }

    /// The scene start: the flag is cleared; a negative type deactivates (the pending values
    /// stay); a type with a kind spawns fresh and the pending values clear; a type with no kind
    /// spawns nothing (and the values stay).
    pub fn scene_start(&mut self) -> Restore {
        if !std::mem::take(&mut self.flag) {
            return Restore::Nothing;
        }
        if self.type_id < 0 {
            return Restore::Deactivate;
        }
        if kind(self.type_id).is_none() {
            return Restore::Nothing;
        }
        let r = Restore::Spawn { type_id: self.type_id, t: self.t, f: self.f };
        *self = Self::default();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sets() -> Sets {
        Sets { a: vec![150, 146, 156, 58, -1, -1, -1, -1], b: vec![149, 157, 147, 59, -1, -1, -1, -1], c: vec![153, 155, 152, 57, 56, 55, 50, -1] }
    }

    #[test]
    fn kinds_and_multipliers() {
        // SPEC-0047 vectors 5 to 14 (the multiplier parts; synthetic sets as in Layout).
        assert_eq!((kind(703), kind(4660), second_kind(805), kind(805)), (Some(1), None, Some(2), None));
        let s = sets();
        assert_eq!(s.multiplier(0, 146), 1.5);
        assert_eq!(s.multiplier(1, 146), 1.0);
        assert_eq!(s.multiplier(7, 146), 0.5);
        assert_eq!(s.multiplier(3, 147), 1.5);
        assert_eq!(s.multiplier(3, 150), 0.75);
        assert_eq!(s.multiplier(6, 150), 1.5);
        assert_eq!(s.multiplier(6, 152), 1.5);
        assert_eq!(s.multiplier(7, 155), 0.5);
        assert_eq!(s.multiplier(8, 152), 1.0);
        assert_eq!(s.multiplier(0, -1), 1.0);
        // Vector 6: 5.0 x 1.5 minutes = 27000 ticks; vector 12: 60 x 1.5 = 324000.
        assert_eq!((meal_ticks(7.5), meal_ticks(90.0)), (27000, 324000));
    }

    #[test]
    fn meals_save_and_restore() {
        // Vector 17: T 27000 with a candidate of 18000 stays; 54000 raises T and F.
        let mut p = Live::new(450, 27000, 27000).unwrap();
        p.eat(5.0);
        assert_eq!((p.t, p.f), (27000, 27000));
        p.eat(15.0);
        assert_eq!((p.t, p.f, p.meter()), (54000, 54000, 60));
        // Vectors 2 and 18: save fields.
        assert_eq!(save_fields(Some((450, 27000, 27000))), (450, 27000, 27000));
        assert_eq!(save_fields(None), (-1, 0, 0));
        assert_eq!(save_fields(Some((450, 0, 500))), (-1, 0, 500));
        // Vectors 1, 3, 4: the scene start.
        let mut n = Pending::default();
        assert_eq!(n.scene_start(), Restore::Nothing);
        let mut n = Pending::load(703, 27000, 27000);
        assert_eq!(n.scene_start(), Restore::Spawn { type_id: 703, t: 27000, f: 27000 });
        assert_eq!(n, Pending::default());
        let mut n = Pending::load(-1, 0, 0);
        assert_eq!(n.scene_start(), Restore::Deactivate);
        let mut n = Pending::load(805, 100, 100);
        assert_eq!(n.scene_start(), Restore::Nothing);
        // Vector 23: a hero at the origin facing +Z gives a petpet at (0, 0, 7).
        assert_eq!(spawn_point([0.0; 3], [0.0, 0.0, 1.0]), [0.0, 0.0, 7.0]);
        // Meter: 0 with F = 0.
        assert_eq!(Live::new(450, 0, 0).unwrap().meter(), 0);
    }

    #[test]
    fn passive_bonuses() {
        let b = Bonus { damage: 0.25, intake: 0.1, hp: 1.0, mp: 1.0 };
        // Vectors 28, 29.
        assert_eq!((10.0 * b.damage_factor(Some(0), 100), 10.0 * b.damage_factor(Some(1), 100), 10.0 * b.damage_factor(None, 100)), (12.5, 10.0, 10.0));
        assert_eq!((20.0 * b.intake_factor(Some(7), 100), 20.0 * b.intake_factor(Some(0), 100)), (18.0, 20.0));
        assert_eq!(b.damage_factor(Some(0), 0), 1.0, "T below 1: no pet");
        // Vector 32: T 60 passes, 59, 0, -1 and 1 fail.
        assert_eq!([60, 59, 0, -1, 1].map(|t| b.hp_point(Some(6), t)), [1, 0, 0, 0, 0]);
        assert_eq!((b.mp_point(Some(3), 120), b.mp_point(Some(6), 120), b.hp_point(Some(7), 120)), (1, 0, 1));
        // Vector 31: 300 passing values from 18000 down to 1.
        assert_eq!((1..=18000).filter(|&t| b.hp_point(Some(6), t) == 1).count(), 300);
    }

    #[test]
    fn class_34_controller() {
        let mut rng = Rng::seeded(3);
        let v = View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, 5.0],
            hero_facing: [0.0, 0.0, 1.0],
            hero_right: [1.0, 0.0, 0.0],
            hero_idle: true,
            counting: true,
            contact: false,
            clip_done: false,
        };
        // Vector 34: T = 1 keeps following; the next counting tick T = 0 and the pet leaves.
        let mut p = Live::new(450, 1, 27000).unwrap();
        p.low_done = true;
        p.tick(&v, 1800, &mut rng);
        assert_eq!((p.state, p.t), (1, 0));
        p.tick(&v, 1800, &mut rng);
        assert_eq!(p.state, 5);
        // Vector 35: contact beats a pending event, which stays set.
        let mut p = Live::new(450, 5000, 27000).unwrap();
        p.pending_event = true;
        p.tick(&View { contact: true, ..v }, 1800, &mut rng);
        assert_eq!((p.state, p.pending_event), (3, true));
        // Vector 38: below the low mark the first time: state 6 (animation 7) once.
        let mut p = Live::new(450, 1700, 27000).unwrap();
        let a = p.tick(&v, 1800, &mut rng);
        assert_eq!((p.state, a.anim), (6, Some(7)));
        p.tick(&View { clip_done: true, ..v }, 1800, &mut rng);
        p.tick(&v, 1800, &mut rng);
        assert_eq!(p.state, 1);
        // Freeze: T does not change while the game-state code is not 0.
        let t = p.t;
        p.tick(&View { counting: false, ..v }, 1800, &mut rng);
        assert_eq!(p.t, t);
    }

    #[test]
    fn moving_follow_gaits() {
        // Vectors 48 and 49 (the second one numbered 49).
        assert_eq!((gait(0, 3.0), gait(0, 3.01)), ((0, 0), (2, 1)));
        assert_eq!((gait(1, 1.5), gait(1, 7.0), gait(1, 1.49), gait(1, 7.01)), ((2, 1), (2, 1), (0, 0), (3, 2)));
        assert_eq!((gait(2, 5.0), gait(2, 4.99)), ((3, 2), (2, 1)));
    }
}
