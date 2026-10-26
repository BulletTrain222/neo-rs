//! The Ixi Chief boss (SPEC-0014; behaviour class 11): the fight mode chosen once (game states
//! 358 and 60), the stages of mode 0 (the dormant hold, the transition at half health, death
//! with level flag 3), the lava events of mode 1 (and the second that can never fire), the hold
//! while level flag 3 is set, the missile and melee reactions, the incoming hits through the
//! get-hit state, the attack chooser (counter, charge, forward swipe, wide swipe, buck kick,
//! whirl), the movement band, the orbit, the charge's sub-steps, the push and the damage scale.
//!
//! The movement states ask for a locomotion mode (0 rush, 1 strafe close, 3 strafe back, 5 and 6
//! orbit) and a target; each mode plays its slot entry (SLOTS; R-0067). The host measures the missile contact, the hero's hits, the weapon tests and the
//! navigation check. Every tuning value is read from the boot data at the addresses of SPEC-0014.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The Ixi Chief's locomotion slot values (SPEC-0014; R-0067), set before every state handler:
/// modes 0, 1, 2, 3 and 4 play entries 9, 10, 30, 14 and 15; mode 5 entry 16 (clip 27, it
/// circles to its left) and mode 6 entry 17 (clip 28, to its right).
pub const SLOTS: [i32; 7] = [9, 10, 30, 14, 15, 17, 16];

/// The tuning block: 47 words from 0x2DAE90 to 0x2DAF4B (the base damage scale is the last).
pub const BASE: u32 = 0x002D_AE90;
pub const WORDS: usize = 47;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub lava: [f32; 2],
    pub missile_cooldown: (f32, f32),
    pub missile_block: i32,
    pub missile_dodge: i32,
    pub melee_block: i32,
    pub melee_dodge: i32,
    pub counter_damage: i32,
    pub retreat: (f32, f32),
    pub charge_cooldown: (f32, f32),
    pub slam_chance: i32,
    pub slam_damage: i32,
    pub forward_range: f32,
    pub forward_cooldown: (f32, f32),
    pub forward_chance: i32,
    pub forward_damage: i32,
    pub wide_cooldown: (f32, f32),
    pub wide_chance: [i32; 2],
    pub wide_damage: i32,
    pub buck_range: f32,
    pub buck_damage: i32,
    pub whirl_trigger: f32,
    pub whirl_hit: f32,
    pub whirl_push: f32,
    pub whirl_cooldown: (f32, f32),
    pub whirl_chance: i32,
    pub whirl_damage: i32,
    pub rush: f32,
    pub strafe_close: f32,
    pub strafe_back: f32,
    pub base_scale: f32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let f = |at: u32| f32::from_bits(w(at));
        let i = |at: u32| w(at) as i32;
        let pair = |at: u32| (f(at), f(at + 4));
        Some(Self {
            lava: [f(0x002D_AE90), f(0x002D_AE94)],
            missile_cooldown: pair(0x002D_AE98),
            missile_block: i(0x002D_AEA0),
            missile_dodge: i(0x002D_AEA4),
            melee_block: i(0x002D_AEAC),
            melee_dodge: i(0x002D_AEB4),
            counter_damage: i(0x002D_AEBC),
            retreat: pair(0x002D_AEC0),
            charge_cooldown: pair(0x002D_AEC8),
            slam_chance: i(0x002D_AED0),
            slam_damage: i(0x002D_AED8),
            forward_range: f(0x002D_AEE4),
            forward_cooldown: pair(0x002D_AEE8),
            forward_chance: i(0x002D_AEF0),
            forward_damage: i(0x002D_AEF4),
            wide_cooldown: pair(0x002D_AEF8),
            wide_chance: [i(0x002D_AF00), i(0x002D_AF04)],
            wide_damage: i(0x002D_AF08),
            buck_range: f(0x002D_AF0C),
            buck_damage: i(0x002D_AF14),
            whirl_trigger: f(0x002D_AF18),
            whirl_hit: f(0x002D_AF1C),
            whirl_push: f(0x002D_AF20),
            whirl_cooldown: pair(0x002D_AF24),
            whirl_chance: i(0x002D_AF2C),
            whirl_damage: i(0x002D_AF30),
            rush: f(0x002D_AF38),
            strafe_close: f(0x002D_AF3C),
            strafe_back: f(0x002D_AF40),
            base_scale: f(0x002D_AF48),
        })
    }
}

/// The fight mode from the two story flags (game state 358 = 0x166, then 60 = 0x3C).
pub fn fight_mode(f358: bool, f60: bool) -> u8 {
    if f358 {
        2
    } else if f60 {
        1
    } else {
        0
    }
}

/// The damage of an attack: trunc(tuned x scale).
pub fn damage(tuned: i32, scale: f32) -> i32 {
    (tuned as f32 * scale) as i32
}

/// A push of the hero: K x normalize(9 d + (0, 12, 0)) for the horizontal unit direction d from
/// the boss to the hero (hero state 0x30).
pub fn push(d: Vec3, k: f32) -> Vec3 {
    let v = [9.0 * d[0], 12.0, 9.0 * d[2]];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|c| c / l * k)
}

/// The orbit point: 3.0 to the side s = (-d.z, 0, d.x) of the boss (direction 0), or the other.
pub fn orbit_point(p: Vec3, d: Vec3, dir: u8) -> Vec3 {
    let s = [-d[2], 0.0, d[0]];
    let k = if dir == 0 { 3.0 } else { -3.0 };
    [p[0] + k * s[0], p[1], p[2] + k * s[2]]
}

/// The charge's three candidates: the hero plus R along its forward, its side (f.z, 0, -f.x)
/// and the opposite side.
pub fn charge_candidates(hero: Vec3, f: Vec3, r: f32) -> [Vec3; 3] {
    let side = [f[2], 0.0, -f[0]];
    [
        [hero[0] + r * f[0], hero[1], hero[2] + r * f[2]],
        [hero[0] + r * side[0], hero[1], hero[2] + r * side[2]],
        [hero[0] - r * side[0], hero[1], hero[2] - r * side[2]],
    ]
}

/// The movement band's choice (0 rush, 1 strafe close, 3 strafe back, 5 orbit; None: keep).
pub fn band(d: f32, w: &Words, current: Option<u8>) -> Option<u8> {
    if d > w.rush {
        Some(0)
    } else if d > w.strafe_close {
        if d < 13.0 { Some(1) } else { None }
    } else if d < w.strafe_back {
        Some(3)
    } else if current == Some(5) || (current == Some(1) && d > 8.0) {
        None
    } else {
        Some(5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Hold,
    Idle,
    Transition,
    Death,
    BlockMissile,
    DodgeMissile(u8),
    DodgeMelee(u8),
    BlockMelee,
    Counter,
    /// The charge: its sub-step and target.
    Charge(u8, Vec3),
    ForwardSwipe,
    WideSwipe,
    Whirl,
    BuckKick,
    /// A movement state by locomotion mode (0, 1, 3, 5 or 6).
    Move(u8),
    GetHit,
    Lava,
}

pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero's attack-active word, its guard, its action state.
    pub hero_attacking: bool,
    pub hero_guard: bool,
    pub hero_state: u8,
    pub hp: i32,
    pub max_hp: i32,
    /// A hero projectile's 120-tick path meets the boss.
    pub missile_contact: bool,
    /// The hero's melee hit (its damage) and the summed damage of the shots that hit.
    pub melee_hit: Option<i32>,
    pub shot_hit: Option<i32>,
    /// The monster weapon test of weapons 1 to 5 against the hero this tick.
    pub weapons: [bool; 5],
    pub frame_bits: u32,
    pub clip: (u32, bool),
    /// The run past is blocked; the hero is in its knocked state.
    pub blocked: bool,
    pub hero_knocked: bool,
    pub valid: &'a dyn Fn(Vec3) -> bool,
    pub level: &'a dyn Fn(u16) -> bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Act {
    pub anim: Option<u32>,
    /// Turn toward the hero at a weight.
    pub face: Option<f32>,
    /// Travel by a locomotion mode toward a point.
    pub travel: Option<(u8, Vec3)>,
    /// A hit on the hero: damage and the blocked flag (unreduced boss melee).
    pub hero_damage: Option<(i32, bool)>,
    /// A push of the hero (velocity; hero state 0x30).
    pub push: Option<Vec3>,
    pub set_level: Vec<u16>,
    pub hp_loss: i32,
    /// The hit flash 222 at the boss.
    pub flash: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ixi {
    pub state: State,
    pub mode: u8,
    pub stage: u8,
    pub scale: f32,
    pub lava: [bool; 2],
    cool: [i32; 5],
    hit: bool,
    melee: bool,
    orbit_dir: u8,
    orbit_timer: i32,
    retreat: f32,
}

const MISSILE: usize = 0;
const CHARGE: usize = 1;
const FORWARD: usize = 2;
const WIDE: usize = 3;
const WHIRL: usize = 4;

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

fn draw(rng: &mut Rng, (lo, hi): (f32, f32)) -> i32 {
    rng.range(ticks(lo), ticks(hi))
}

fn roll(rng: &mut Rng, chance: i32) -> bool {
    rng.range(0, 100) < chance
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn hdir(from: Vec3, to: Vec3) -> Vec3 {
    let d = hdist(from, to).max(1e-6);
    [(to[0] - from[0]) / d, 0.0, (to[2] - from[2]) / d]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Ixi {
    /// The controller's start: the mode from the flags, the scale (the base word in mode 0, 1.0
    /// otherwise); mode 0 starts in the dormant hold.
    pub fn new(mode: u8, w: &Words) -> Self {
        let scale = if mode == 0 { w.base_scale } else { 1.0 };
        Self { state: if mode == 0 { State::Hold } else { State::Idle }, mode, stage: 0, scale, lava: [false; 2], cool: [0; 5], hit: false, melee: false, orbit_dir: 0, orbit_timer: 30, retreat: 0.0 }
    }

    fn free(&self) -> bool {
        matches!(self.state, State::Idle | State::Move(_))
    }

    fn reacts(&self) -> bool {
        self.free() || matches!(self.state, State::Counter | State::ForwardSwipe | State::WideSwipe | State::Whirl | State::BuckKick) || matches!(self.state, State::Charge(s, _) if !matches!(s, 1 | 5))
    }

    fn enter(&mut self, s: State) {
        self.state = s;
    }

    /// One tick (SPEC-0014 Each tick, in order).
    /// One tick; while it travels by a locomotion mode without its own clip, the clip is the
    /// mode's slot entry (SLOTS, R-0067).
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = self.tick_inner(v, w, rng);
        if let (None, Some((mode, _))) = (a.anim, a.travel) {
            a.anim = crate::enemy::slot_entry(&SLOTS, mode);
        }
        a
    }

    fn tick_inner(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        for c in &mut self.cool {
            if *c > 0 {
                *c -= 1;
            }
        }
        let f = if v.max_hp > 0 { v.hp as f32 / v.max_hp as f32 } else { 0.0 };
        self.stage_rules(v, w, f, rng, &mut a);
        let d = hdist(v.position, v.hero);
        let to_hero = hdir(v.position, v.hero);
        let in_front = dot(v.facing, to_hero) > 0.5;
        let hero_faces = dot(v.hero_facing, hdir(v.hero, v.position)) > 0.5;
        if !v.hero_attacking {
            self.melee = false;
        }
        if self.reacts() {
            // The missile reaction.
            if self.cool[MISSILE] == 0 && v.missile_contact {
                if roll(rng, w.missile_block) {
                    self.enter(State::BlockMissile);
                } else if roll(rng, w.missile_dodge) {
                    self.enter(State::DodgeMissile(rng.range(0, 1) as u8));
                }
                self.cool[MISSILE] = draw(rng, w.missile_cooldown);
            }
            // The melee reaction.
            if v.hero_attacking && !self.melee && d < 7.0 && hero_faces {
                self.melee = true;
                if roll(rng, w.melee_dodge) {
                    self.enter(State::DodgeMelee(rng.range(0, 1) as u8));
                } else if roll(rng, w.melee_block) {
                    self.enter(State::BlockMelee);
                }
            }
        }
        // The incoming hits (none in the hold, the lava or death).
        if !matches!(self.state, State::Hold | State::Lava | State::Death) {
            let shielded = matches!(self.state, State::DodgeMelee(_) | State::BlockMelee | State::ForwardSwipe | State::GetHit) && in_front;
            let melee = v.melee_hit.filter(|_| !matches!(self.state, State::GetHit | State::Transition) && !shielded);
            let total = melee.unwrap_or(0) + v.shot_hit.unwrap_or(0);
            if melee.is_some() || v.shot_hit.is_some() {
                a.flash = true;
                a.hp_loss = total;
                self.enter(State::GetHit);
            }
        }
        if self.free() {
            self.choose(v, w, d, rng);
        }
        if self.free() {
            if let Some(m) = band(d, w, if let State::Move(m) = self.state { Some(if m == 6 { 5 } else { m }) } else { None }) {
                if m == 5 && !matches!(self.state, State::Move(5 | 6)) {
                    self.orbit_dir = rng.range(0, 1) as u8;
                    self.orbit_timer = 30;
                }
                if !(m == 5 && matches!(self.state, State::Move(5 | 6))) {
                    self.enter(State::Move(if m == 5 { 5 + self.orbit_dir } else { m }));
                }
            }
        }
        self.run(v, w, d, to_hero, rng, &mut a);
        a
    }

    fn stage_rules(&mut self, v: &View, w: &Words, f: f32, rng: &mut Rng, a: &mut Act) {
        if self.state == State::Death {
            return;
        }
        match self.mode {
            0 => match self.stage {
                0 => {
                    self.state = State::Hold;
                    if (v.level)(0) {
                        self.stage = 1;
                        self.state = State::Idle;
                        self.cool[CHARGE] = draw(rng, w.charge_cooldown);
                        self.cool[FORWARD] = draw(rng, w.forward_cooldown);
                        self.cool[WIDE] = draw(rng, w.wide_cooldown);
                    }
                }
                1 => {
                    if self.state == State::Transition {
                        if (v.level)(2) {
                            self.state = State::Idle;
                            self.stage = 2;
                        }
                    } else if f < 0.5 {
                        self.state = State::Transition;
                        a.set_level.push(1);
                    }
                }
                _ => {
                    if f <= 0.0 {
                        self.state = State::Death;
                        a.set_level.push(3);
                    }
                }
            },
            mode => {
                if mode == 1 {
                    if !self.lava[0] && f < w.lava[0] {
                        self.state = State::Lava;
                        self.lava[0] = true;
                    } else if self.lava[0] && !self.lava[1] && f < w.lava[1] {
                        self.state = State::Lava;
                        self.lava[1] = true;
                    }
                }
                if f <= 0.0 {
                    self.state = State::Death;
                    if mode == 1 {
                        a.set_level.push(0);
                    }
                    return;
                }
                if (v.level)(3) {
                    self.state = State::Hold;
                } else if self.state == State::Hold {
                    self.state = State::Idle;
                }
            }
        }
    }

    fn choose(&mut self, v: &View, w: &Words, d: f32, rng: &mut Rng) {
        let behind = dot(v.facing, hdir(v.position, v.hero)) < -0.5;
        let strafing = matches!(v.hero_state, 0x20 | 0x2B..=0x2E);
        let next = if v.hero_attacking && d < 6.0 {
            Some(State::Counter)
        } else if self.cool[CHARGE] == 0 {
            self.retreat = rng.frange(w.retreat.0, w.retreat.1);
            Some(State::Charge(0, v.position))
        } else if self.cool[FORWARD] == 0 && d < w.forward_range {
            if roll(rng, w.forward_chance) {
                Some(State::ForwardSwipe)
            } else {
                self.cool[FORWARD] = draw(rng, w.forward_cooldown);
                None
            }
        } else if self.cool[WIDE] == 0 && d < w.forward_range && strafing {
            if roll(rng, w.wide_chance[(self.mode > 0) as usize]) {
                Some(State::WideSwipe)
            } else {
                self.cool[WIDE] = draw(rng, w.wide_cooldown);
                None
            }
        } else if behind && d < w.buck_range {
            Some(State::BuckKick)
        } else if self.mode == 1 && self.cool[WHIRL] == 0 {
            if !roll(rng, w.whirl_chance) {
                self.cool[WHIRL] = draw(rng, w.whirl_cooldown);
                None
            } else if d < w.whirl_trigger {
                Some(State::Whirl)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(s) = next {
            self.hit = false;
            self.enter(s);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run(&mut self, v: &View, w: &Words, d: f32, to_hero: Vec3, rng: &mut Rng, a: &mut Act) {
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let hit_now = |n: u32, weapons: &[usize]| v.clip.0 == n && weapons.iter().any(|&k| v.frame_bits & (1 << k) != 0 && v.weapons[k]);
        match self.state {
            State::Hold | State::Idle | State::Transition | State::Death => a.anim = Some(7),
            State::Lava => {
                a.anim = Some(1);
                self.lava[1] = true;
                if v.clip.0 == 1 && v.frame_bits & 0x1F != 0 {
                    a.set_level.push(2);
                }
                if ended(1) {
                    self.enter(State::Idle);
                }
            }
            State::BlockMissile => {
                a.anim = Some(0x14);
                if !v.missile_contact {
                    self.enter(State::Idle);
                }
            }
            State::DodgeMissile(dir) | State::DodgeMelee(dir) => {
                let n = if dir == 0 { 0x13 } else { 0x14 };
                a.anim = Some(n);
                if ended(n) {
                    self.enter(State::Idle);
                }
            }
            State::BlockMelee => {
                a.anim = Some(0x14);
                if !v.hero_attacking {
                    self.enter(State::Idle);
                }
            }
            State::Counter => {
                a.anim = Some(0x19);
                if hit_now(0x19, &[0]) && !self.hit {
                    self.hit = true;
                    a.hero_damage = Some((damage(w.counter_damage, self.scale), false));
                }
                if ended(0x19) {
                    self.enter(State::Idle);
                }
            }
            State::ForwardSwipe => {
                a.anim = Some(0x18);
                if hit_now(0x18, &[0]) && !self.hit {
                    self.hit = true;
                    let faces = dot(v.hero_facing, hdir(v.hero, v.position)) > 0.5;
                    a.hero_damage = Some(if v.hero_guard && faces { (0, true) } else { (damage(w.forward_damage, self.scale), false) });
                }
                if ended(0x18) {
                    self.enter(State::Idle);
                    self.cool[FORWARD] = draw(rng, w.forward_cooldown);
                }
            }
            State::WideSwipe => {
                a.anim = Some(0x1A);
                a.face = Some(0.25);
                if hit_now(0x1A, &[0]) && !self.hit {
                    self.hit = true;
                    a.hero_damage = Some((damage(w.wide_damage, self.scale), false));
                }
                if ended(0x1A) {
                    self.enter(State::Idle);
                    self.cool[WIDE] = draw(rng, w.wide_cooldown);
                }
            }
            State::Whirl => {
                a.anim = Some(0x1B);
                if v.clip.0 == 0x1B && v.frame_bits & 0x1 != 0 && !self.hit && d < w.whirl_hit {
                    self.hit = true;
                    a.hero_damage = Some((damage(w.whirl_damage, self.scale), false));
                    a.push = Some(push(to_hero, w.whirl_push));
                }
                if ended(0x1B) {
                    self.enter(State::Idle);
                    self.cool[WHIRL] = draw(rng, w.whirl_cooldown);
                }
            }
            State::BuckKick => {
                a.anim = Some(0x1D);
                if hit_now(0x1D, &[3, 4]) && !self.hit {
                    self.hit = true;
                    a.hero_damage = Some((damage(w.buck_damage, self.scale), false));
                }
                if ended(0x1D) {
                    self.enter(State::Idle);
                }
            }
            State::GetHit => {
                a.anim = Some(6);
                if ended(6) {
                    if rng.range(0, 100) < 25 {
                        self.hit = false;
                        self.enter(State::ForwardSwipe);
                    } else {
                        self.enter(State::Idle);
                    }
                }
            }
            State::Move(m) => {
                let target = match m {
                    0 | 1 => v.hero,
                    3 => [v.position[0] - 2.0 * to_hero[0], v.position[1], v.position[2] - 2.0 * to_hero[2]],
                    _ => {
                        self.orbit_timer -= 1;
                        if self.orbit_timer <= 0 {
                            self.orbit_timer = 30;
                            self.orbit_dir ^= 1;
                            self.state = State::Move(5 + self.orbit_dir);
                        }
                        let p = orbit_point(v.position, to_hero, self.orbit_dir);
                        if (v.valid)(p) { p } else { v.position }
                    }
                };
                let mode = if let State::Move(m) = self.state { m } else { m };
                a.travel = Some((mode, target));
            }
            State::Charge(step, target) => self.charge(v, w, d, to_hero, step, target, rng, a),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn charge(&mut self, v: &View, w: &Words, d: f32, to_hero: Vec3, step: u8, target: Vec3, rng: &mut Rng, a: &mut Act) {
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let fail = |s: &mut Self, rng: &mut Rng| {
            s.cool[CHARGE] = draw(rng, w.charge_cooldown);
            s.enter(State::Idle);
        };
        match step {
            0 => match charge_candidates(v.hero, v.hero_facing, self.retreat).into_iter().find(|&p| (v.valid)(p)) {
                Some(t) => self.state = State::Charge(1, t),
                None => fail(self, rng),
            },
            1 => {
                // The charge's own slot write: slot 0 plays entry 30 (clip 9, 20.0 units/s).
                a.anim = Some(30);
                a.travel = Some((0, target));
                if hdist(v.position, target) < 1.0 {
                    self.state = State::Charge(2, target);
                }
            }
            2 => {
                a.anim = Some(7);
                a.face = Some(0.1);
                if ended(7) {
                    self.state = State::Charge(3, target);
                }
            }
            3 => {
                a.anim = Some(0x15);
                a.face = Some(1.0);
                if ended(0x15) {
                    self.state = State::Charge(4, target);
                }
            }
            4 => {
                if !(v.valid)(v.hero) {
                    fail(self, rng);
                    return;
                }
                a.anim = Some(0x16);
                a.travel = Some((0, v.hero));
                if d < 6.0 {
                    if rng.range(0, 100) < w.slam_chance {
                        self.state = State::Charge(6, target);
                    } else {
                        self.hit = false;
                        self.state = State::Charge(5, target);
                    }
                }
            }
            5 => {
                a.anim = Some(0x16);
                a.travel = Some((0, [v.position[0] + v.facing[0], v.position[1], v.position[2] + v.facing[2]]));
                if d > 7.5 || v.blocked {
                    fail(self, rng);
                } else if !v.hero_knocked && d < 2.0 {
                    a.push = Some(push(to_hero, 5.0));
                }
            }
            _ => {
                a.anim = Some(0x17);
                if v.clip.0 == 0x17 && v.frame_bits & 0x1 != 0 && v.weapons[0] && !self.hit {
                    self.hit = true;
                    a.hero_damage = Some((damage(w.slam_damage, self.scale), false));
                }
                if ended(0x17) {
                    fail(self, rng);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        let mut b = vec![0u32; WORDS];
        let mut put = |at: u32, v: u32| b[((at - BASE) / 4) as usize] = v;
        for (at, x) in [(0x002D_AE90, 0.66f32), (0x002D_AE94, 0.33), (0x002D_AE98, 0.5), (0x002D_AE9C, 1.0), (0x002D_AEC0, 10.0), (0x002D_AEC4, 20.0), (0x002D_AEC8, 1.0), (0x002D_AECC, 3.0), (0x002D_AEE4, 7.0), (0x002D_AEE8, 1.0), (0x002D_AEEC, 3.0), (0x002D_AEF8, 1.0), (0x002D_AEFC, 3.0), (0x002D_AF0C, 4.0), (0x002D_AF18, 15.0), (0x002D_AF1C, 12.0), (0x002D_AF20, 5.0), (0x002D_AF24, 2.0), (0x002D_AF28, 4.0), (0x002D_AF38, 15.0), (0x002D_AF3C, 10.0), (0x002D_AF40, 5.0), (0x002D_AF48, 0.25)] {
            put(at, x.to_bits());
        }
        for (at, x) in [(0x002D_AEA0, 25u32), (0x002D_AEA4, 25), (0x002D_AEAC, 25), (0x002D_AEB4, 25), (0x002D_AEBC, 25), (0x002D_AED0, 40), (0x002D_AED8, 100), (0x002D_AEF0, 50), (0x002D_AEF4, 50), (0x002D_AF00, 25), (0x002D_AF04, 50), (0x002D_AF08, 35), (0x002D_AF14, 25), (0x002D_AF2C, 40), (0x002D_AF30, 100)] {
            put(at, x);
        }
        Words::read(&b).unwrap()
    }

    fn view<'a>(valid: &'a dyn Fn(Vec3) -> bool, level: &'a dyn Fn(u16) -> bool, hp: i32) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, 30.0],
            hero_facing: [0.0, 0.0, -1.0],
            hero_attacking: false,
            hero_guard: false,
            hero_state: 1,
            hp,
            max_hp: 100,
            missile_contact: false,
            melee_hit: None,
            shot_hit: None,
            weapons: [false; 5],
            frame_bits: 0,
            clip: (7, false),
            blocked: false,
            hero_knocked: false,
            valid,
            level,
        }
    }

    #[test]
    fn modes_damage_and_shapes() {
        let w = words();
        assert_eq!([fight_mode(true, true), fight_mode(false, true), fight_mode(false, false)], [2, 1, 0]);
        // Damages at scale 0.25: counter 6, forward 12, wide 8, whirl 25, slam 25, buck 6.
        let s = w.base_scale;
        assert_eq!([w.counter_damage, w.forward_damage, w.wide_damage, w.whirl_damage, w.slam_damage, w.buck_damage].map(|t| damage(t, s)), [6, 12, 8, 25, 25, 6]);
        assert_eq!(damage(25, 1.0), 25);
        // The push: K 5 along +x gives (3, 4, 0).
        let p = push([1.0, 0.0, 0.0], 5.0);
        assert!((p[0] - 3.0).abs() < 1e-5 && (p[1] - 4.0).abs() < 1e-5 && p[2] == 0.0);
        // The orbit side: boss at the origin, hero at (0, 0, 10): direction 0 to (-3, 0, 0).
        assert_eq!(orbit_point([0.0; 3], [0.0, 0.0, 1.0], 0), [-3.0, 0.0, 0.0]);
        assert_eq!(orbit_point([0.0; 3], [0.0, 0.0, 1.0], 1), [3.0, 0.0, 0.0]);
        // The charge candidates: hero at the origin facing +z: (0, 0, R), (R, 0, 0), (-R, 0, 0).
        assert_eq!(charge_candidates([0.0; 3], [0.0, 0.0, 1.0], 12.0), [[0.0, 0.0, 12.0], [12.0, 0.0, 0.0], [-12.0, 0.0, 0.0]]);
        // The movement band: 16 rush, 12 strafe close, 4 strafe back, 14 keep, 9 orbit.
        assert_eq!([16.0, 12.0, 4.0, 14.0, 9.0].map(|d| band(d, &w, None)), [Some(0), Some(1), Some(3), None, Some(5)]);
        assert_eq!(band(9.0, &w, Some(1)), None);
        assert_eq!(band(7.5, &w, Some(1)), Some(5));
    }

    #[test]
    fn stages_and_lava() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let valid = |_: Vec3| true;
        let no = |_: u16| false;
        let zero = |f: u16| f == 0;
        // Mode 0: the dormant hold until level flag 0; stage 1's transition below 0.5 strictly.
        let mut k = Ixi::new(0, &w);
        k.tick(&view(&valid, &no, 100), &w, &mut rng);
        assert_eq!((k.state, k.stage), (State::Hold, 0));
        k.tick(&view(&valid, &zero, 100), &w, &mut rng);
        assert_eq!(k.stage, 1);
        let a = k.tick(&view(&valid, &no, 50), &w, &mut rng);
        assert!(a.set_level.is_empty());
        let a = k.tick(&view(&valid, &no, 49), &w, &mut rng);
        assert_eq!((k.state, a.set_level), (State::Transition, vec![1]));
        // Mode 1: lava at 0.65 (both latches set on its first tick); never a second lava.
        let mut k = Ixi::new(1, &w);
        k.tick(&view(&valid, &no, 66), &w, &mut rng);
        assert!(!k.lava[0]);
        k.tick(&view(&valid, &no, 65), &w, &mut rng);
        assert_eq!((k.state, k.lava), (State::Lava, [true, true]));
        let mut v = view(&valid, &no, 32);
        v.clip = (1, true);
        k.tick(&v, &w, &mut rng);
        k.tick(&view(&valid, &no, 32), &w, &mut rng);
        assert_ne!(k.state, State::Lava);
        // Mode 2: no lava; death without a flag. Mode 1's death sets level flag 0.
        let mut k = Ixi::new(2, &w);
        k.tick(&view(&valid, &no, 10), &w, &mut rng);
        assert_ne!(k.state, State::Lava);
        assert!(k.tick(&view(&valid, &no, 0), &w, &mut rng).set_level.is_empty());
        let mut k = Ixi::new(1, &w);
        k.lava = [true, true];
        assert_eq!(k.tick(&view(&valid, &no, 0), &w, &mut rng).set_level, vec![0]);
        // The hold while level flag 3 is set (modes 1 and 2).
        let three = |f: u16| f == 3;
        let mut k = Ixi::new(1, &w);
        k.tick(&view(&valid, &three, 100), &w, &mut rng);
        assert_eq!(k.state, State::Hold);
        k.tick(&view(&valid, &no, 100), &w, &mut rng);
        assert_ne!(k.state, State::Hold);
    }

    #[test]
    fn attacks_hits_and_charge() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let valid = |_: Vec3| true;
        let no = |_: u16| false;
        // The forward swipe's block: a guarding hero facing it (0.6) takes 0 with the flag.
        let mut k = Ixi::new(1, &w);
        k.state = State::ForwardSwipe;
        let mut v = view(&valid, &no, 100);
        v.hero = [0.0, 0.0, 5.0];
        v.clip = (0x18, false);
        v.frame_bits = 1;
        v.weapons[0] = true;
        v.hero_guard = true;
        let a = k.tick(&v, &w, &mut rng);
        assert_eq!(a.hero_damage, Some((0, true)));
        // The whirl: 11.9 away 100 once with the push; 12.0 nothing.
        for (z, hit) in [(11.9, true), (12.0, false)] {
            let mut k = Ixi::new(1, &w);
            k.state = State::Whirl;
            let mut v = view(&valid, &no, 100);
            v.hero = [0.0, 0.0, z];
            v.clip = (0x1B, false);
            v.frame_bits = 1;
            let a = k.tick(&v, &w, &mut rng);
            assert_eq!(a.hero_damage.is_some(), hit);
            assert_eq!(a.push.is_some(), hit);
        }
        // A counted hit: the flash, the damage, the get-hit state; none in the hold.
        let mut k = Ixi::new(1, &w);
        let mut v = view(&valid, &no, 100);
        v.shot_hit = Some(10);
        let a = k.tick(&v, &w, &mut rng);
        assert!(a.flash && a.hp_loss == 10 && k.state == State::GetHit);
        let mut k = Ixi::new(1, &w);
        let three = |f: u16| f == 3;
        let mut v = view(&valid, &three, 100);
        v.shot_hit = Some(10);
        assert_eq!(k.tick(&v, &w, &mut rng).hp_loss, 0);
        // The charge: no distance check; sub-step 0 takes the first valid candidate.
        let mut k = Ixi::new(1, &w);
        k.tick(&view(&valid, &no, 100), &w, &mut rng);
        assert!(matches!(k.state, State::Charge(1, _)), "{:?}", k.state);
        // No valid candidate: the cooldown and idle.
        let invalid = |_: Vec3| false;
        let mut k = Ixi::new(1, &w);
        k.tick(&view(&invalid, &no, 100), &w, &mut rng);
        assert!(!matches!(k.state, State::Charge(..)));
        // The run past ends beyond 7.5.
        let mut k = Ixi::new(1, &w);
        k.cool = [100; 5];
        k.state = State::Charge(5, [0.0; 3]);
        let mut v = view(&valid, &no, 100);
        v.hero = [0.0, 0.0, 7.6];
        k.tick(&v, &w, &mut rng);
        assert!(!matches!(k.state, State::Charge(5, _)));
        // The wide swipe needs the hero strafing (0x2B eligible, 0x10 not).
        let mut k = Ixi::new(1, &w);
        k.cool = [100, 100, 100, 0, 100];
        let mut v = view(&valid, &no, 100);
        v.hero = [0.0, 0.0, 6.0];
        v.hero_state = 0x10;
        k.tick(&v, &w, &mut rng);
        assert_ne!(k.state, State::WideSwipe);
    }
}
