//! Petpets (SPEC-0047): the pet type to kind tables, the kind multipliers and the meal time
//! update, the food values and the shared food reservation with its fallback, the passive
//! bonuses to the hero, the pending save values, the wild petpet's class 33 loop (state choice,
//! idle, roam, strike with the eat test, hover) and the summoned petpet's class 34 controller
//! (state choice, idle-hero follow, moving follow with its three gaits, eating in place, the
//! low-time one-shot and the leave).
//!
//! Class 20 (the second petpet behaviour) has its idle, approach-and-consume and wander states.
//!
//! Not covered: class 20's state 1 (its item test is open), the bumped recovery of class 34
//! state 3 (it holds animation 0) and the nearby-character response of state 1 (its filter is
//! open).

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

/// The Bagguss (145): the anchor, fed but never eaten.
pub const BAGGUSS: u16 = 145;

/// The food ids the hero can feed (145 to 157 but 148, 151 and 154).
pub fn feedable(id: u16) -> bool {
    (145..=157).contains(&id) && !matches!(id, 148 | 151 | 154)
}

/// The nine base-value words (0x2DB1DC to 0x2DB1FC).
pub const BASE_AT: u32 = 0x002D_B1DC;

/// The base-value word a food id reads (Layout, Base hit value by target type code: which word
/// belongs to which id is open, open question 4; this picks a word of the spec's value).
fn base_word(id: u16) -> Option<usize> {
    match id {
        150 => Some(0),
        146 | 149 | 157 => Some(1),
        147 | 153 | 156 => Some(2),
        155 => Some(7),
        152 => Some(8),
        _ => None,
    }
}

/// The food values: the base words and the membership sets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Food {
    pub base: [f32; 9],
    pub sets: Sets,
}

impl Food {
    /// The hit value of food `id` for a petpet of kind `k`, in minutes (0 for the anchor and
    /// the ids without a value).
    pub fn value(&self, kind: u8, id: u16) -> f32 {
        match base_word(id) {
            Some(w) => self.base[w] * self.sets.multiplier(kind, id as i32),
            None => 0.0,
        }
    }
}

/// The shared food reservation (Strike): the reserved food (a world item's token), the timer in
/// seconds and the slow flag.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reservation {
    pub food: Option<u32>,
    pub timer: f32,
    pub slow: bool,
}

impl Reservation {
    pub fn reserve(&mut self, food: u32) {
        *self = Self { food: Some(food), timer: 0.0, slow: false };
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// The fallback, once per tick while a food is reserved: the timer grows by 1/60 s (1/180 s
    /// while slow); past 5.0 s the reservation clears and the food is returned, to be removed and
    /// given to the live petpet.
    pub fn tick(&mut self) -> Option<u32> {
        let food = self.food?;
        self.timer += if self.slow { 1.0 / 180.0 } else { 1.0 / 60.0 };
        if self.timer > 5.0 {
            self.clear();
            return Some(food);
        }
        None
    }
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
    /// The playing clip and whether it has ended.
    pub clip_action: u32,
    pub clip_done: bool,
    /// The nearest world item of category 12 (position, item id, token).
    pub food: Option<(Vec3, u16, u32)>,
}

/// What the controller asks for this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Walk toward a point, turning with a weight.
    pub toward: Option<(Vec3, f32)>,
    /// The petpet leaves (it is removed).
    pub leave: bool,
    /// Reserve this food (its token) this tick.
    pub reserve: Option<u32>,
    /// The food eaten this tick (token, item id): its value is applied and it is removed.
    pub eat: Option<(u32, u16)>,
    /// Turn toward a point with a rate (no walk).
    pub turn: Option<(Vec3, f32)>,
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
            4 => match v.food {
                // Strike step 3: the timer is read before this tick's reservation, so the test
                // passes and the petpet eats where it stands; when the clip ends the food is
                // applied and the reservation cleared.
                Some((_, id, token)) if id != BAGGUSS => {
                    a.reserve = Some(token);
                    a.anim = Some(4);
                    if v.clip_action == 4 && v.clip_done {
                        a.eat = Some((token, id));
                        self.state = 1;
                    }
                }
                // No food: nothing happens in the step; the petpet returns to following
                // (the exit is not given; Medium).
                _ => {
                    a.anim = Some(0);
                    self.state = 1;
                }
            },
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

/// What a wild petpet (class 33) sees this tick.
#[derive(Clone, Copy, Debug)]
pub struct WildView {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    /// The nearest world item of category 12 (position, item id, token) and whether the
    /// petpet can reach it.
    pub food: Option<(Vec3, u16, u32)>,
    pub reachable: bool,
    /// The nearest anchor (a Bagguss on the ground).
    pub anchor: Option<Vec3>,
    pub clip_action: u32,
    pub clip_done: bool,
}

/// A wild petpet's class 33 loop (State selection, States, Strike).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wild {
    pub state: u8,
    /// Set by the hero's feed (hero state 49).
    pub fed: bool,
    /// The remaining time its meal gave (ticks), carried into the live petpet when it joins.
    pub t: i32,
    point: Option<Vec3>,
    wait: Option<i32>,
    eating: bool,
    one_shot: bool,
}

impl Wild {
    /// One tick: the state choice, then the state body.
    pub fn tick(&mut self, v: &WildView, rng: &mut Rng) -> Act {
        let near_food = v.food.filter(|&(p, id, _)| id != BAGGUSS && self.fed && v.reachable && hdist(p, v.position) <= 10.0);
        if hdist(v.position, v.hero) >= 10.0 {
            // Rule 1 (the test for state 2 on this branch always fails).
            if !matches!(self.state, 4 | 5) {
                self.state = 3;
            }
        } else if near_food.is_some() {
            self.state = 4;
        } else if v.anchor.is_some_and(|p| hdist(p, v.position) <= 5.0) {
            self.state = 5;
        } else if !matches!(self.state, 1 | 4) {
            self.state = 1;
            self.one_shot = true;
        }
        let mut a = Act::default();
        match self.state {
            1 => {
                // Animation 9 once, then idle.
                if self.one_shot {
                    a.anim = Some(9);
                    if v.clip_action == 9 && v.clip_done {
                        self.one_shot = false;
                    }
                } else {
                    a.anim = Some(0);
                }
            }
            3 => self.roam(v, rng, &mut a),
            4 => self.strike(v, &mut a),
            5 => {
                // Hold about 10 units from the anchor (the point after arrival is not
                // recorded).
                let Some(c) = v.anchor else {
                    self.state = 1;
                    a.anim = Some(0);
                    return a;
                };
                let d = [v.position[0] - c[0], 0.0, v.position[2] - c[2]];
                let l = (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-6);
                let p = [c[0] + 10.0 * d[0] / l, v.position[1], c[2] + 10.0 * d[2] / l];
                if hdist(p, v.position) >= 1.0 {
                    a.toward = Some((p, 0.1));
                    a.anim = Some(2);
                } else {
                    a.anim = Some(0);
                }
            }
            _ => a.anim = Some(0),
        }
        a
    }

    /// State 3: a point within 40 units of the hero and a wait; idle while the wait runs, then
    /// walk to the point; on arrival a new point and wait (the wait's range is open; 180 to 600
    /// ticks is used, as class 34's).
    fn roam(&mut self, v: &WildView, rng: &mut Rng, a: &mut Act) {
        let point = match self.point {
            Some(p) => p,
            None => {
                let ang = rng.frange(0.0, std::f32::consts::TAU);
                let r = rng.frange(0.0, 40.0);
                let p = [v.hero[0] + r * ang.cos(), v.hero[1], v.hero[2] + r * ang.sin()];
                self.point = Some(p);
                self.wait = Some(rng.range(180, 600));
                p
            }
        };
        if let Some(w) = self.wait.filter(|&w| w > 0) {
            self.wait = Some(w - 1);
            a.anim = Some(0);
            return;
        }
        if hdist(v.position, point) < 1.0 {
            self.point = None;
            a.anim = Some(0);
            return;
        }
        a.toward = Some((point, 0.1));
        a.anim = Some(2);
    }

    /// State 4 (Strike, steps 1, 2 and 4): reserve the nearest food each tick; eat when facing
    /// it (dot above 0.8) less than 2.0 away, else walk toward it (2.0 or more away, when it can
    /// be reached) or turn toward it (rate 0.1). The eat clip is animation 4; at its end the
    /// food is eaten and the petpet joins.
    fn strike(&mut self, v: &WildView, a: &mut Act) {
        let Some((p, id, token)) = v.food.filter(|&(_, id, _)| id != BAGGUSS) else {
            // No food: nothing happens in the step; the petpet goes back to idle (the exit is
            // not given; Medium).
            self.eating = false;
            self.state = 1;
            a.anim = Some(0);
            return;
        };
        a.reserve = Some(token);
        if self.eating {
            a.anim = Some(4);
            if v.clip_action == 4 && v.clip_done {
                self.eating = false;
                self.fed = false;
                a.eat = Some((token, id));
            }
            return;
        }
        let to = [p[0] - v.position[0], 0.0, p[2] - v.position[2]];
        let d = (to[0] * to[0] + to[2] * to[2]).sqrt();
        let dot = if d > 0.0 { (v.facing[0] * to[0] + v.facing[2] * to[2]) / d } else { 1.0 };
        if dot > 0.8 && d < 2.0 {
            self.eating = true;
            a.anim = Some(4);
        } else if d >= 2.0 {
            if v.reachable {
                a.toward = Some((p, 0.1));
                a.anim = Some(2);
            } else {
                a.anim = Some(0);
            }
        } else {
            a.turn = Some((p, 0.1));
            a.anim = Some(0);
        }
    }

    /// The meal of value v minutes: the time it gives when larger than the current one.
    pub fn eat(&mut self, v: f32) {
        self.t = self.t.max(meal_ticks(v));
    }
}

/// The class 20 wait words: 4.0 and 5.0 seconds (0x2DB2AC, 0x2DB2B0).
pub const CLASS20_WAIT_AT: u32 = 0x002D_B2AC;

/// What a class 20 creature sees: the nearest world item other than the anchor within 25 units
/// (position, token).
#[derive(Clone, Copy, Debug)]
pub struct Class20View {
    pub position: Vec3,
    pub object: Option<(Vec3, u32)>,
    pub clip_action: u32,
    pub clip_done: bool,
}

/// What class 20 asks for this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Class20Act {
    pub anim: Option<u32>,
    pub toward: Option<Vec3>,
    /// The object it latched this tick (its token): it is removed from the world.
    pub consume: Option<u32>,
    /// It is seeking an object (the reservation's slow flag).
    pub seeking: bool,
}

/// Class 20 (SPEC-0047, Class 20): states 0 (idle), 2 (approach and consume) and 3 (wander).
/// State 1 needs the item test, which is open, so it is never chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct Class20 {
    pub state: u8,
    sub: u8,
    wait: i32,
    point: Option<Vec3>,
    latched: bool,
    target: Option<Vec3>,
    /// The reference object's point (the spawn point; Medium).
    home: Vec3,
}

impl Class20 {
    pub fn new(home: Vec3) -> Self {
        Self { state: 0, sub: 0, wait: 0, point: None, latched: false, target: None, home }
    }

    /// One tick. `waits` is the 4 to 5 s range in ticks.
    pub fn tick(&mut self, v: &Class20View, waits: (i32, i32), rng: &mut Rng) -> Class20Act {
        if self.state != 2 {
            if v.object.is_some() {
                self.state = 2;
                self.latched = false;
                self.target = None;
            } else if self.state != 3 {
                self.state = 3;
                self.sub = 1;
            }
        }
        let mut a = Class20Act::default();
        match self.state {
            2 => {
                if self.latched {
                    a.anim = Some(4);
                    if v.clip_action == 4 && v.clip_done {
                        self.state = 3;
                        self.sub = 0;
                        self.wait = rng.range(waits.0, waits.1);
                    }
                    return a;
                }
                let moved = |p: Vec3| self.target.is_some_and(|t| hdist(t, p) > 2.0);
                match v.object {
                    Some((p, _)) if moved(p) => self.to_wander(waits, rng, &mut a),
                    None => self.to_wander(waits, rng, &mut a),
                    Some((p, token)) => {
                        self.target = Some(p);
                        a.seeking = true;
                        if hdist(p, v.position) >= 2.5 {
                            a.toward = Some(p);
                            a.anim = Some(2);
                        } else {
                            self.latched = true;
                            a.consume = Some(token);
                            a.anim = Some(4);
                        }
                    }
                }
            }
            3 => self.wander(v, waits, rng, &mut a),
            _ => a.anim = Some(0),
        }
        a
    }

    fn to_wander(&mut self, waits: (i32, i32), rng: &mut Rng, a: &mut Class20Act) {
        self.state = 3;
        self.sub = 0;
        self.wait = rng.range(waits.0, waits.1);
        a.anim = Some(0);
    }

    /// State 3: sub-step 0 counts the wait down with animation 0, then picks a point within 40
    /// units of the reference object; sub-step 1 walks to it and within 5.0 draws a new wait.
    fn wander(&mut self, v: &Class20View, waits: (i32, i32), rng: &mut Rng, a: &mut Class20Act) {
        if self.sub == 0 {
            a.anim = Some(0);
            self.wait -= 1;
            if self.wait <= 0 {
                let ang = rng.frange(0.0, std::f32::consts::TAU);
                let r = rng.frange(0.0, 40.0);
                self.point = Some([self.home[0] + r * ang.cos(), self.home[1], self.home[2] + r * ang.sin()]);
                self.sub = 1;
            }
            return;
        }
        // A wander entered at sub-step 1 walks to the stored point; with none stored the
        // reference point is used (open question 21).
        let p = *self.point.get_or_insert(self.home);
        if hdist(v.position, p) < 5.0 {
            self.sub = 0;
            self.wait = rng.range(waits.0, waits.1);
            a.anim = Some(0);
        } else {
            a.toward = Some(p);
            a.anim = Some(2);
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
            clip_action: 0,
            clip_done: false,
            food: None,
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
    fn food_values_and_reservation() {
        // Vectors 6 to 16 with the base words as on the disc's values.
        let f = Food { base: [2.0, 5.0, 10.0, 2.0, 5.0, 10.0, 10.0, 30.0, 60.0], sets: sets() };
        let ticks = |k: u8, id: u16| meal_ticks(f.value(k, id));
        assert_eq!([ticks(0, 146), ticks(1, 146), ticks(7, 146), ticks(3, 147), ticks(3, 150), ticks(6, 150), ticks(6, 152), ticks(7, 155), ticks(8, 152)], [27000, 18000, 9000, 54000, 5400, 10800, 324000, 54000, 216000]);
        assert_eq!((f.value(0, BAGGUSS), f.value(0, 148)), (0.0, 0.0));
        assert!(feedable(145) && feedable(157) && !feedable(148) && !feedable(144));
        // Vector 24: the fallback passes 5.0 s on about the 301st tick; a reserve resets it.
        let mut r = Reservation::default();
        r.reserve(7);
        let n = (1..=400).find(|_| r.tick().is_some());
        assert!(matches!(n, Some(300..=302)), "{n:?}");
        for _ in 0..1000 {
            r.reserve(7);
            assert_eq!(r.tick(), None);
        }
    }

    #[test]
    fn class_33_wild_petpet() {
        let mut rng = Rng::seeded(5);
        let v = WildView { position: [0.0; 3], facing: [0.0, 0.0, 1.0], hero: [0.0, 0.0, 8.0], food: None, reachable: true, anchor: None, clip_action: 0, clip_done: false };
        // Vector 21: 10.0 from the hero roams; 9.9 idles; a fed petpet with food 9 away strikes.
        let mut w = Wild::default();
        w.tick(&WildView { hero: [0.0, 0.0, 10.0], ..v }, &mut rng);
        assert_eq!(w.state, 3);
        let mut w = Wild::default();
        w.tick(&WildView { hero: [0.0, 0.0, 9.9], ..v }, &mut rng);
        assert_eq!(w.state, 1);
        let food = Some(([0.0, 0.0, 9.0], 146, 3));
        let mut w = Wild { fed: true, ..Wild::default() };
        w.tick(&WildView { food, ..v }, &mut rng);
        assert_eq!(w.state, 4);
        // Not fed: no strike.
        let mut w = Wild::default();
        w.tick(&WildView { food, ..v }, &mut rng);
        assert_eq!(w.state, 1);
        // Vector 22: an anchor 4 away hovers.
        let mut w = Wild::default();
        w.tick(&WildView { anchor: Some([4.0, 0.0, 0.0]), ..v }, &mut rng);
        assert_eq!(w.state, 5);
        // Vector 42: 1.5 away facing (dot 0.9) eats; dot 0.7 turns; 3.0 away walks.
        let at = |x: f32, z: f32| Some(([x, 0.0, z], 146u16, 3u32));
        let mut w = Wild { fed: true, ..Wild::default() };
        let a = w.tick(&WildView { food: at(0.6538, 1.35), ..v }, &mut rng);
        assert_eq!((a.anim, a.reserve, w.eating), (Some(4), Some(3), true));
        let a = w.tick(&WildView { food: at(0.6538, 1.35), clip_action: 4, clip_done: true, ..v }, &mut rng);
        assert_eq!(a.eat, Some((3, 146)));
        let mut w = Wild { fed: true, ..Wild::default() };
        let a = w.tick(&WildView { food: at(1.071, 1.05), ..v }, &mut rng);
        assert!(a.turn.is_some() && a.eat.is_none() && !w.eating);
        let mut w = Wild { fed: true, ..Wild::default() };
        let a = w.tick(&WildView { food: at(0.0, 3.0), ..v }, &mut rng);
        assert!(a.toward.is_some() && !w.eating);
        // Vector 37: a kind 0 petpet fed a Chokato gets 27000 ticks.
        let f = Food { base: [2.0, 5.0, 10.0, 2.0, 5.0, 10.0, 10.0, 30.0, 60.0], sets: sets() };
        let mut w = Wild::default();
        w.eat(f.value(0, 146));
        assert_eq!(w.t, 27000);
    }

    #[test]
    fn class_34_eats_in_place() {
        // Vector 41: a pending event with food 8.0 away: animation 4 without moving; at the
        // clip's end the food is eaten.
        let mut rng = Rng::seeded(1);
        let v = View { position: [0.0; 3], facing: [0.0, 0.0, 1.0], hero: [0.0, 0.0, 3.0], hero_facing: [0.0, 0.0, 1.0], hero_right: [1.0, 0.0, 0.0], hero_idle: true, counting: true, contact: false, clip_action: 0, clip_done: false, food: Some(([8.0, 0.0, 0.0], 146, 9)) };
        let mut p = Live::new(450, 5000, 27000).unwrap();
        p.pending_event = true;
        let a = p.tick(&v, 1800, &mut rng);
        assert_eq!((p.state, a.anim, a.toward, a.reserve), (4, Some(4), None, Some(9)));
        let a = p.tick(&View { clip_action: 4, clip_done: true, ..v }, 1800, &mut rng);
        assert_eq!((a.eat, p.state), (Some((9, 146)), 1));
    }

    #[test]
    fn class_20() {
        // Vector 25: 2.5 away moves toward the object; 2.4 latches and consumes it; the next
        // tick plays animation 4; when it ends, state 3 sub-step 0 with a 240 to 300 wait.
        let mut rng = Rng::seeded(2);
        let w = (240, 300);
        let v = Class20View { position: [0.0; 3], object: Some(([2.5, 0.0, 0.0], 4)), clip_action: 0, clip_done: false };
        let mut c = Class20::new([0.0; 3]);
        let a = c.tick(&v, w, &mut rng);
        assert_eq!((c.state, a.toward, a.consume), (2, Some([2.5, 0.0, 0.0]), None));
        let a = c.tick(&Class20View { object: Some(([2.4, 0.0, 0.0], 4)), ..v }, w, &mut rng);
        assert_eq!((c.state, a.consume, c.latched), (2, Some(4), true));
        let a = c.tick(&Class20View { object: None, ..v }, w, &mut rng);
        assert_eq!((c.state, a.anim), (2, Some(4)));
        c.tick(&Class20View { object: None, clip_action: 4, clip_done: true, ..v }, w, &mut rng);
        assert!(c.state == 3 && c.sub == 0 && (240..=300).contains(&c.wait));
        // Vector 26: in sub-step 1, 6.0 from the point keeps walking; 4.0 draws a new wait.
        let mut c = Class20::new([0.0; 3]);
        c.state = 3;
        c.sub = 1;
        c.point = Some([6.0, 0.0, 0.0]);
        let a = c.tick(&Class20View { object: None, ..v }, w, &mut rng);
        assert_eq!((c.sub, a.toward), (1, Some([6.0, 0.0, 0.0])));
        c.point = Some([4.0, 0.0, 0.0]);
        c.tick(&Class20View { object: None, ..v }, w, &mut rng);
        assert_eq!(c.sub, 0);
    }

    #[test]
    fn moving_follow_gaits() {
        // Vectors 48 and 49 (the second one numbered 49).
        assert_eq!((gait(0, 3.0), gait(0, 3.01)), ((0, 0), (2, 1)));
        assert_eq!((gait(1, 1.5), gait(1, 7.0), gait(1, 1.49), gait(1, 7.01)), ((2, 1), (2, 1), (0, 0), (3, 2)));
        assert_eq!((gait(2, 5.0), gait(2, 4.99)), ((3, 2), (2, 1)));
    }
}
