//! The Gnarfas boss (SPEC-0020; behaviour class 14): the phase hub (event 0, the phase 2 heal,
//! the hold, rage and death), taking damage through the flinch, the missile and melee
//! reactions, the attack order and cooldowns, approach and orbit, the chain whip and rapid
//! strike with the bone-chain test, the head butt, the jump-to-tree sequence and the body slam.
//! The level flags it reads and posts are shared with the level script (SPEC-0041 pack 985).
//!
//! The host measures what needs the world (the hero's hits, the missile paths, the bone-chain
//! overlap, the tree points, the ground gap) and applies the requests. Every tuning value is read
//! from the boot data at the addresses of SPEC-0020.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning words (SPEC-0020 Layout).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub laugh_chance: i32,
    pub rage_percent: i32,
    pub block_chance: [i32; 2],
    pub dodge_chance: i32,
    pub reaction: (f32, f32),
    pub missile_jump_chance: i32,
    pub melee_dodge_chance: i32,
    pub whip_cooldown: (f32, f32),
    pub whip_range: f32,
    pub whip_damage: i32,
    pub rapid_cooldown: (f32, f32),
    pub rapid_range: f32,
    pub rapid_damage: i32,
    pub rapid_exhaust_chance: i32,
    pub power_range: f32,
    pub power_cooldown: (f32, f32),
    pub power_chance: [i32; 2],
    pub power_damage: i32,
    pub jump_hero_chance: i32,
    pub slam_damage: i32,
    pub butt_range: f32,
    pub butt_cooldown: (f32, f32),
    pub butt_chance: [i32; 2],
    pub approach_range: f32,
    pub jump_time: f32,
    pub exhaust: [(f32, f32); 2],
    pub push: f32,
    pub gravity: f32,
    pub run_walk: f32,
}

/// The boot addresses of SPEC-0020 Layout (each a 32-bit word; the pairs are min then max).
pub const LAUGH_CHANCE: u32 = 0x002D_AD98;
pub const RAGE_PERCENT: u32 = 0x002D_ADA4;
pub const BLOCK_CHANCE: u32 = 0x002D_ADAC;
pub const DODGE_CHANCE: u32 = 0x002D_ADB4;
pub const REACTION: u32 = 0x002D_ADB8;
pub const MISSILE_JUMP_CHANCE: u32 = 0x002D_ADC0;
pub const MELEE_DODGE_CHANCE: u32 = 0x002D_ADC4;
pub const WHIP_COOLDOWN: u32 = 0x002D_ADD8;
pub const WHIP_RANGE: u32 = 0x002D_ADE0;
pub const WHIP_DAMAGE: u32 = 0x002D_ADE4;
pub const RAPID_COOLDOWN: u32 = 0x002D_ADF0;
pub const RAPID_RANGE: u32 = 0x002D_ADF8;
pub const RAPID_DAMAGE: u32 = 0x002D_ADFC;
pub const RAPID_EXHAUST: u32 = 0x002D_AE00;
pub const POWER_RANGE: u32 = 0x002D_AE04;
pub const POWER_COOLDOWN: u32 = 0x002D_AE08;
pub const POWER_CHANCE: u32 = 0x002D_AE10;
pub const POWER_DAMAGE: u32 = 0x002D_AE18;
pub const JUMP_HERO_CHANCE: u32 = 0x002D_AE20;
pub const SLAM_DAMAGE: u32 = 0x002D_AE2C;
pub const BUTT_RANGE: u32 = 0x002D_AE30;
pub const BUTT_COOLDOWN: u32 = 0x002D_AE34;
pub const BUTT_CHANCE: u32 = 0x002D_AE3C;
pub const APPROACH_RANGE: u32 = 0x002D_AE48;
pub const JUMP_TIME: u32 = 0x002D_AE54;
pub const EXHAUST: u32 = 0x002D_AE64;
pub const PUSH: u32 = 0x002D_AE78;
pub const GRAVITY: u32 = 0x002D_AE80;
pub const RUN_WALK: u32 = 0x002D_AE8C;

/// The tree points' locator tags (ids 101 to 104 of the boss's level).
pub const TREES: [usize; 4] = [101, 102, 103, 104];

/// The bone runs of the chain test by event bit (0x1, 0x2, 0x4, 0x8): first and last bone; the
/// last is the run's tip, where the chain spark plays.
pub const RUNS: [(u32, usize, usize); 4] = [(0x1, 29, 33), (0x2, 15, 19), (0x4, 46, 49), (0x8, 38, 41)];

/// The chain bone cylinder (radius 0.5, length 1.25 from the posed origin along its Y axis)
/// against the hero's upright cylinder (radius 1.0, length 3.0 from its feet): whether they
/// overlap (the static finite-cylinder test of SPEC-0018, taken here as the distance between
/// the two axis segments against the sum of the radii).
pub fn bone_hits(origin: Vec3, y_axis: Vec3, hero: Vec3) -> bool {
    let end = [origin[0] + 1.25 * y_axis[0], origin[1] + 1.25 * y_axis[1], origin[2] + 1.25 * y_axis[2]];
    let top = [hero[0], hero[1] + 3.0, hero[2]];
    segment_distance(origin, end, hero, top) <= 1.5
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The closest distance between segments p0p1 and q0q1.
pub fn segment_distance(p0: Vec3, p1: Vec3, q0: Vec3, q1: Vec3) -> f32 {
    let (d1, d2, r) = (sub(p1, p0), sub(q1, q0), sub(p0, q0));
    let (a, e, f) = (dot(d1, d1), dot(d2, d2), dot(d2, r));
    let c = dot(d1, r);
    let b = dot(d1, d2);
    let den = a * e - b * b;
    let mut s = if den > 1e-12 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
    let mut t = if e > 1e-12 { (b * s + f) / e } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if a > 1e-12 { (-c / a).clamp(0.0, 1.0) } else { 0.0 };
    } else if t > 1.0 {
        t = 1.0;
        s = if a > 1e-12 { ((b - c) / a).clamp(0.0, 1.0) } else { 0.0 };
    }
    let cp = [p0[0] + d1[0] * s, p0[1] + d1[1] * s, p0[2] + d1[2] * s];
    let cq = [q0[0] + d2[0] * t, q0[1] + d2[1] * t, q0[2] + d2[2] * t];
    dot(sub(cp, cq), sub(cp, cq)).sqrt()
}

/// The boss states (SPEC-0020 States).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Flinch { exhausted: bool, damage: i32 },
    Triumph,
    Rage,
    Death { sent: bool },
    Laugh,
    Whip { hit: bool },
    Rapid { hit: bool },
    Exhaust,
    Power { hit: bool },
    Slam { hit: bool },
    Butt { hit: bool },
    Approach,
    Orbit { side: u8 },
    DodgeMissile,
    BlockMissile,
    DodgeMelee(u8),
    /// The jump sequence: step 0, 1, 2, 3 or 5, the target, the velocity and the perch wait.
    Jump { step: u8, target: Vec3, velocity: Vec3, wait: i32, tree: usize },
}

/// What the boss sees this tick (the host's measurements).
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_hp: i32,
    pub hero_max: i32,
    /// The hero is down: its action state is 16 (the hold, the knockback and the death gate;
    /// not the busy list, which would freeze the boss whenever the hero swings or jumps).
    pub hero_busy: bool,
    /// The hero is guarding and faces Gnarfas (the block test of the chain hits).
    pub hero_blocks: bool,
    /// The hero is in a down state (7 to 10, 0x0B, 0x2B to 0x2E).
    pub hero_down: bool,
    /// The hero started a melee attack this tick (the rising edge).
    pub hero_attack_start: bool,
    /// The hero's hit on Gnarfas this tick (melee or a missile slot), with its damage.
    pub hit: Option<i32>,
    /// The hit was a hero shot only (no melee hit this tick): a block removes it (SPEC-0020 Hero
    /// actions).
    pub shot: bool,
    /// A hero missile's path (10 ticks ahead) touches Gnarfas; and 120 ticks ahead.
    pub missile_near: bool,
    pub missile_ahead: bool,
    /// The bone-chain test of this tick's event bits hit the hero: the tip bone of the run.
    pub chain_hit: Option<usize>,
    /// The current frame's event bits.
    pub frame_bits: u32,
    /// The playing clip: action, progress, ended.
    pub clip: (u32, f32, bool),
    pub hp: i32,
    pub max_hp: i32,
    pub level: &'a dyn Fn(u16) -> bool,
    pub trees: [Option<Vec3>; 4],
    /// Height above the ground.
    pub ground_gap: f32,
}

/// How the host moves the boss this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// By the clip's root motion with the facing turned toward the target at the weight (the
    /// shared steering rule: modes 5 and 6 face across the move, others along it).
    Steer { mode: u8, target: Vec3, weight: f32 },
    /// By this velocity (a jump arc), turning at 0.2 toward the unit direction to the jump's
    /// target (flat in steps 1 and 5, three-dimensional in step 2; SPEC-0020 Step 1, corrected
    /// 2026-11-09: not the direction of the motion).
    Fly(Vec3, Vec3),
    /// No travel.
    Still,
}

/// What it asks for this tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Turn toward the hero at this weight.
    pub face: Option<f32>,
    pub travel: Move,
    /// A hit on the hero: raw damage, blocked, and the knockback strength (0 for none).
    pub hero_damage: Option<(i32, bool, f32)>,
    /// A knockback with no damage (the head butt), when the hero is not busy.
    pub knockback: Option<f32>,
    pub spark: Option<usize>,
    pub hp_loss: i32,
    pub set_level: Vec<u16>,
    /// The phase 2 entry restores the hero's health to its maximum.
    pub heal_hero: bool,
}

impl Default for Act {
    fn default() -> Self {
        Self { anim: None, face: None, travel: Move::Root, hero_damage: None, knockback: None, spark: None, hp_loss: 0, set_level: Vec::new(), heal_hero: false }
    }
}

#[allow(non_upper_case_globals)]
impl Move {
    /// Root motion with no steering (the default of a playing clip).
    pub const Root: Move = Move::Steer { mode: 255, target: [0.0; 3], weight: 0.0 };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gnarfas {
    pub state: State,
    pub phase2: bool,
    rage: bool,
    pub strikes: i32,
    whip: i32,
    rapid: i32,
    power: i32,
    butt: i32,
    jump_timer: i32,
    exhaust: i32,
    reaction: i32,
    flip: i32,
}

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

fn draw(rng: &mut Rng, (lo, hi): (f32, f32)) -> i32 {
    rng.range(ticks(lo), ticks(hi))
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A roll less than the chance: the roll is 0 to 100 inclusive (101 values; SPEC-0019 Roll and coin
/// rules, which SPEC-0020 does not restate).
fn roll(rng: &mut Rng, chance: i32) -> bool {
    rng.range(0, 100) < chance
}

impl Gnarfas {
    /// Setup at spawn: idle, the jump timer at 30 ticks, each cooldown drawn once.
    pub fn new(w: &Words, rng: &mut Rng) -> Self {
        let butt = draw(rng, w.butt_cooldown);
        let whip = draw(rng, w.whip_cooldown);
        let power = draw(rng, w.power_cooldown);
        let rapid = draw(rng, w.rapid_cooldown);
        Self { state: State::Idle, phase2: false, rage: false, strikes: 0, whip, rapid, power, butt, jump_timer: ticks(w.jump_time), exhaust: 0, reaction: 0, flip: 0 }
    }

    fn free(&self) -> bool {
        matches!(self.state, State::Idle | State::Approach | State::Orbit { .. })
    }

    fn percent(hp: i32, max: i32) -> i32 {
        if max <= 0 { 0 } else { (100.0 * hp as f32 / max as f32) as i32 }
    }

    fn jump(v: &View) -> State {
        State::Jump { step: 0, target: v.position, velocity: [0.0; 3], wait: 0, tree: 0 }
    }

    /// One tick.
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let d = hdist(v.position, v.hero);
        let phase = self.phase2 as usize;
        // Taking damage, before the hub (not in the flinch, death and the attacks).
        let immune = matches!(self.state, State::Flinch { .. } | State::Death { .. } | State::Whip { .. } | State::Rapid { .. } | State::Power { .. } | State::Slam { .. } | State::Butt { .. });
        if let (Some(damage), false) = (v.hit, immune) {
            // A shot meeting the missile block is only removed; a melee hit flinches him there too.
            if !(v.shot && self.state == State::BlockMissile) {
                self.state = State::Flinch { exhausted: self.state == State::Exhaust, damage };
            }
        }
        // The timers.
        for t in [&mut self.whip, &mut self.rapid, &mut self.power, &mut self.butt, &mut self.jump_timer, &mut self.exhaust, &mut self.reaction, &mut self.flip] {
            *t = (*t - 1).max(0);
        }
        // The phase hub: event 0, the phase 2 entry, then (free states) the hold, rage, death.
        if !(v.level)(0) && (2 * v.hero_hp < v.hero_max || self.strikes >= 3) {
            a.set_level.push(0);
        }
        if (v.level)(0) && (v.level)(1) && !self.phase2 {
            self.phase2 = true;
            a.heal_hero = true;
        }
        if self.free() {
            let p = Self::percent(v.hp, v.max_hp);
            if v.hero_busy {
                self.state = State::Triumph;
            } else if p <= 0 {
                self.state = State::Death { sent: false };
            } else if p < 100 - w.rage_percent && !self.rage {
                self.state = State::Rage;
            }
        }
        // The reactions, the attack order and the movement (free states only).
        if self.free() {
            if self.reaction == 0 && v.missile_near {
                self.state = if roll(rng, w.block_chance[phase]) {
                    State::BlockMissile
                } else if roll(rng, w.dodge_chance) {
                    State::DodgeMissile
                } else if roll(rng, w.missile_jump_chance) {
                    Self::jump(v)
                } else {
                    self.state
                };
                self.reaction = draw(rng, w.reaction);
            } else if v.hero_attack_start && roll(rng, w.melee_dodge_chance) {
                self.state = State::DodgeMelee(rng.range(0, 2) as u8);
            } else if self.whip == 0 && d < w.whip_range {
                self.state = State::Whip { hit: false };
            } else if self.rapid == 0 && d < w.rapid_range {
                self.state = State::Rapid { hit: false };
            } else if self.power == 0 && d < w.power_range && roll(rng, w.power_chance[phase]) {
                self.state = State::Power { hit: false };
            } else {
                if self.power == 0 && d < w.power_range {
                    self.power = draw(rng, w.power_cooldown);
                }
                let butt = self.butt == 0 && d < w.butt_range;
                if butt && roll(rng, w.butt_chance[phase]) {
                    self.state = State::Butt { hit: false };
                } else {
                    if butt {
                        self.butt = draw(rng, w.butt_cooldown);
                    }
                    let orbiting = matches!(self.state, State::Orbit { .. });
                    if d > w.approach_range {
                        self.state = State::Approach;
                    } else if d >= 6.0 || orbiting {
                        if self.jump_timer == 0 {
                            self.jump_timer = ticks(w.jump_time);
                        }
                    } else {
                        self.state = State::Orbit { side: rng.range(0, 1) as u8 };
                    }
                }
            }
        }
        // The state's behaviour.
        let ended = |id: u32| v.clip.0 == id && v.clip.2;
        let to_hero = {
            let (dx, dz) = (v.hero[0] - v.position[0], v.hero[2] - v.position[2]);
            let l = dx.hypot(dz).max(1e-6);
            [dx / l, 0.0, dz / l]
        };
        match self.state {
            State::Idle => a.anim = Some(3),
            State::Flinch { exhausted, damage } => {
                a.anim = Some(9);
                if ended(9) {
                    a.hp_loss = damage;
                    self.state = if Self::percent(v.hp - damage, v.max_hp) < 1 {
                        State::Death { sent: false }
                    } else if exhausted {
                        State::Exhaust
                    } else {
                        State::Idle
                    };
                }
            }
            State::Triumph => {
                a.anim = Some(1);
                a.face = Some(0.3);
                a.travel = Move::Still;
                if !v.hero_busy {
                    self.state = State::Idle;
                }
            }
            State::Rage => {
                self.rage = true;
                self.state = State::Idle;
            }
            State::Death { sent } => {
                a.anim = Some(2);
                a.travel = Move::Still;
                if ended(2) && !sent {
                    a.set_level.push(2);
                    self.state = State::Death { sent: true };
                }
            }
            // The maniacal laugh is action 0 (SPEC-0020 What a port gets wrong).
            State::Laugh => {
                a.anim = Some(0);
                if ended(0) {
                    self.state = State::Idle;
                }
            }
            State::Whip { hit } | State::Rapid { hit } | State::Power { hit } => {
                let (id, damage) = match self.state {
                    State::Whip { .. } => (0x13, w.whip_damage),
                    State::Rapid { .. } => (0x14, w.rapid_damage),
                    _ => (0x15, w.power_damage),
                };
                a.anim = Some(id);
                a.face = Some((1.0 - v.clip.1.clamp(0.0, 1.0)) * 0.5);
                let mut hit = hit;
                if !hit && v.clip.0 == id {
                    if let Some(tip) = v.chain_hit {
                        hit = true;
                        // The strike count rises on each whip and rapid strike hit, blocked or
                        // not (SPEC-0020 Phase hub, the strike count).
                        if id != 0x15 {
                            self.strikes += 1;
                        }
                        a.spark = Some(tip);
                        // The chain power has no guard test; the others block with full damage.
                        let blocked = id != 0x15 && v.hero_blocks;
                        a.hero_damage = Some((damage, blocked, if blocked || v.hero_busy { 0.0 } else { 1.0 }));
                    }
                }
                self.state = match self.state {
                    State::Whip { .. } => State::Whip { hit },
                    State::Rapid { .. } => State::Rapid { hit },
                    _ => State::Power { hit },
                };
                if ended(id) {
                    self.state = match id {
                        0x13 => {
                            self.whip = draw(rng, w.whip_cooldown);
                            State::Idle
                        }
                        0x14 => {
                            self.rapid = draw(rng, w.rapid_cooldown);
                            if roll(rng, w.rapid_exhaust_chance) {
                                self.exhaust = draw(rng, w.exhaust[phase]);
                                State::Exhaust
                            } else {
                                Self::jump(v)
                            }
                        }
                        _ => {
                            self.exhaust = draw(rng, w.exhaust[phase]);
                            if hit && roll(rng, 30) { State::Laugh } else { State::Exhaust }
                        }
                    };
                }
            }
            State::Exhaust => {
                a.anim = Some(4);
                a.travel = Move::Still;
                if self.exhaust < 1 {
                    self.state = Self::jump(v);
                }
            }
            State::Slam { hit } => {
                a.anim = Some(0x16);
                let mut hit = hit;
                if !hit && d < 5.0 {
                    hit = true;
                    self.strikes += 1;
                    if !v.hero_down {
                        a.hero_damage = Some((w.slam_damage, false, if v.hero_busy { 0.0 } else { 1.0 }));
                    }
                }
                self.state = State::Slam { hit };
                if ended(0x16) {
                    self.state = if hit && roll(rng, w.laugh_chance) { State::Laugh } else { State::Idle };
                }
            }
            State::Butt { hit } => {
                a.anim = Some(0x18);
                let mut hit = hit;
                if !hit && v.frame_bits & 1 != 0 && dot(v.facing, to_hero) > 0.5 && d < 7.0 {
                    hit = true;
                    self.strikes += 1;
                    if !v.hero_busy {
                        a.knockback = Some(2.0);
                    }
                }
                self.state = State::Butt { hit };
                if ended(0x18) {
                    self.butt = draw(rng, w.butt_cooldown);
                    self.state = State::Idle;
                }
            }
            State::Approach => {
                let run = d > w.run_walk;
                a.anim = Some(if run { 6 } else { 5 });
                a.travel = Move::Steer { mode: run as u8, target: v.hero, weight: 0.3 };
                if d < 5.0 {
                    self.state = State::Idle;
                }
            }
            State::Orbit { side } => {
                // The move point 3 units to the side, across the hero direction.
                let perp = if side == 0 { [to_hero[2], 0.0, -to_hero[0]] } else { [-to_hero[2], 0.0, to_hero[0]] };
                let target = [v.position[0] + 3.0 * perp[0], v.position[1], v.position[2] + 3.0 * perp[2]];
                a.anim = Some(if side == 0 { 0x0D } else { 0x0E });
                a.travel = Move::Steer { mode: 5 + side, target, weight: 0.1 };
            }
            State::DodgeMissile => {
                a.anim = Some(0x10);
                if ended(0x10) {
                    self.state = State::Idle;
                }
            }
            State::BlockMissile => {
                a.anim = Some(3);
                a.travel = Move::Still;
                if !v.missile_ahead {
                    self.state = State::Idle;
                }
            }
            State::DodgeMelee(k) => {
                let id = [0x10, 0x11, 0x0F][k.min(2) as usize];
                a.anim = Some(id);
                a.face = Some(0.1);
                if ended(id) {
                    self.state = State::Idle;
                }
            }
            State::Jump { step, target, velocity, wait, tree } => {
                let trees: Vec<(usize, Vec3)> = v.trees.iter().enumerate().filter_map(|(i, t)| t.map(|p| (i, p))).collect();
                if trees.is_empty() {
                    self.state = State::Idle;
                    return a;
                }
                let launch = |to: Vec3| [(to[0] - v.position[0]) / 60.0, (to[1] - v.position[1]) / 60.0 + 1.5, (to[2] - v.position[2]) / 60.0];
                a.travel = Move::Still;
                match step {
                    0 => {
                        a.anim = Some(7);
                        if ended(7) {
                            let &(i, p) = trees.iter().min_by(|x, y| hdist(x.1, v.position).total_cmp(&hdist(y.1, v.position))).unwrap();
                            self.state = State::Jump { step: 1, target: p, velocity: launch(p), wait: 0, tree: i };
                        }
                    }
                    1 | 2 | 5 => {
                        a.anim = Some(if step == 2 && v.clip.0 != 0x0B && !ended(0x0C) { 0x0C } else { 0x0B });
                        let vel = [velocity[0], velocity[1] - w.gravity, velocity[2]];
                        let to = sub(target, v.position);
                        let to = if step == 2 { to } else { [to[0], 0.0, to[2]] };
                        let l = dot(to, to).sqrt();
                        let dir = if l > 0.0 { [to[0] / l, to[1] / l, to[2] / l] } else { [0.0; 3] };
                        a.travel = Move::Fly(vel, dir);
                        let next = [v.position[0] + vel[0], v.position[1] + vel[1], v.position[2] + vel[2]];
                        let dist3 = dot(sub(target, next), sub(target, next)).sqrt();
                        self.state = State::Jump { step, target, velocity: vel, wait, tree };
                        if step == 1 && hdist(next, target) < 3.0 {
                            self.state = State::Jump { step: 3, target, velocity: [0.0; 3], wait: 0, tree };
                        } else if step == 2 && dist3 < 3.0 {
                            self.state = State::Jump { step: 3, target, velocity: [0.0; 3], wait: rng.range(0, 180), tree };
                        } else if step == 5 && vel[1] < 0.0 && v.ground_gap <= 1.0 {
                            self.state = if hdist(next, v.hero) < 5.0 { State::Slam { hit: false } } else { State::Idle };
                        }
                    }
                    _ => {
                        a.anim = Some(10);
                        if wait - 1 <= 0 {
                            if roll(rng, w.jump_hero_chance) {
                                self.state = State::Jump { step: 5, target: v.hero, velocity: launch(v.hero), wait: 0, tree };
                            } else {
                                let others: Vec<&(usize, Vec3)> = trees.iter().filter(|t| t.0 != tree).collect();
                                let pick = if others.is_empty() { trees[0] } else { *others[rng.index(others.len() as u32) as usize] };
                                self.state = State::Jump { step: 2, target: pick.1, velocity: launch(pick.1), wait: 0, tree: pick.0 };
                            }
                        } else {
                            self.state = State::Jump { step: 3, target, velocity, wait: wait - 1, tree };
                        }
                    }
                }
            }
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        Words {
            laugh_chance: 30,
            rage_percent: 75,
            block_chance: [100, 25],
            dodge_chance: 25,
            reaction: (1.0, 3.0),
            missile_jump_chance: 25,
            melee_dodge_chance: 100,
            whip_cooldown: (2.0, 4.0),
            whip_range: 9.0,
            whip_damage: 15,
            rapid_cooldown: (3.0, 6.0),
            rapid_range: 9.0,
            rapid_damage: 15,
            rapid_exhaust_chance: 75,
            power_range: 9.0,
            power_cooldown: (2.0, 6.0),
            power_chance: [0, 0],
            power_damage: 13,
            jump_hero_chance: 50,
            slam_damage: 30,
            butt_range: 15.0,
            butt_cooldown: (4.0, 7.0),
            butt_chance: [40, 25],
            approach_range: 8.0,
            jump_time: 0.5,
            exhaust: [(0.0, 0.0), (2.0, 3.0)],
            push: 8.0,
            gravity: 0.05,
            run_walk: 5.0,
        }
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, hero: Vec3) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero,
            hero_hp: 100,
            hero_max: 100,
            hero_busy: false,
            hero_blocks: false,
            hero_down: false,
            hero_attack_start: false,
            shot: false,
            hit: None,
            missile_near: false,
            missile_ahead: false,
            chain_hit: None,
            frame_bits: 0,
            clip: (3, 0.0, false),
            hp: 1000,
            max_hp: 1000,
            level,
            trees: [None; 4],
            ground_gap: 0.0,
        }
    }

    #[test]
    fn hub_events_and_phase_two() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let mut g = Gnarfas::new(&w, &mut rng);
        let none = |_: u16| false;
        // The hero below half health sends event 0 (while bit 0 is clear).
        let mut v = view(&none, [0.0, 0.0, 20.0]);
        v.hero_hp = 49;
        assert_eq!(g.tick(&v, &w, &mut rng).set_level, vec![0]);
        // Bits 0 and 1 set: phase 2 once, the hero healed.
        let both = |f: u16| f <= 1;
        let v = view(&both, [0.0, 0.0, 20.0]);
        assert!(g.tick(&v, &w, &mut rng).heal_hero && g.phase2);
        assert!(!g.tick(&v, &w, &mut rng).heal_hero);
    }

    #[test]
    fn approach_orbit_and_whip() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let mut g = Gnarfas::new(&w, &mut rng);
        let none = |_: u16| false;
        // Far: approach, running beyond 5 units.
        let a = g.tick(&view(&none, [0.0, 0.0, 20.0]), &w, &mut rng);
        assert_eq!((g.state, a.anim), (State::Approach, Some(6)));
        // Close with the cooldowns zero: the chain whip; its hit is once, blocked at full damage.
        g.whip = 1;
        g.state = State::Idle;
        let mut v = view(&none, [0.0, 0.0, 4.0]);
        g.tick(&v, &w, &mut rng);
        assert_eq!(g.state, State::Whip { hit: false });
        v.clip = (0x13, 0.4, false);
        v.chain_hit = Some(33);
        v.hero_blocks = true;
        let a = g.tick(&v, &w, &mut rng);
        assert_eq!((a.hero_damage, a.spark), (Some((15, true, 0.0)), Some(33)));
        assert_eq!(g.strikes, 1, "a blocked whip hit still counts as a strike");
        assert!(g.tick(&v, &w, &mut rng).hero_damage.is_none());
        assert_eq!(g.strikes, 1, "once per attack");
        v.clip = (0x13, 1.0, true);
        g.tick(&v, &w, &mut rng);
        assert_eq!(g.state, State::Idle);
        // Within 6 units and not orbiting: the orbit (after the other choices fail).
        g.whip = 999;
        g.rapid = 999;
        g.butt = 999;
        g.tick(&view(&none, [0.0, 0.0, 5.5]), &w, &mut rng);
        assert!(matches!(g.state, State::Orbit { .. }));
    }

    #[test]
    fn damage_rage_and_death() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let mut g = Gnarfas::new(&w, &mut rng);
        g.whip = 999;
        g.rapid = 999;
        g.butt = 999;
        let none = |_: u16| false;
        // A hit starts the flinch; the damage lands when its animation ends.
        let mut v = view(&none, [0.0, 0.0, 7.0]);
        v.hit = Some(200);
        g.tick(&v, &w, &mut rng);
        assert!(matches!(g.state, State::Flinch { damage: 200, .. }));
        v.hit = None;
        v.clip = (9, 1.0, true);
        assert_eq!(g.tick(&v, &w, &mut rng).hp_loss, 200);
        // Below 25 percent: rage once; at 0: death, and event 2 when its animation ends.
        let mut v = view(&none, [0.0, 0.0, 7.0]);
        v.hp = 200;
        g.state = State::Idle;
        g.tick(&v, &w, &mut rng);
        assert!(g.rage);
        v.hp = 0;
        g.state = State::Idle;
        g.tick(&v, &w, &mut rng);
        assert_eq!(g.state, State::Death { sent: false });
        v.clip = (2, 1.0, true);
        assert_eq!(g.tick(&v, &w, &mut rng).set_level, vec![2]);
        assert!(g.tick(&v, &w, &mut rng).set_level.is_empty());
    }

    #[test]
    fn jump_slam_and_butt() {
        let w = words();
        let mut rng = Rng::seeded(4);
        let mut g = Gnarfas::new(&w, &mut rng);
        let none = |_: u16| false;
        // The rapid strike's end without exhaustion starts the jump; step 0 then the nearest tree.
        let mut v = view(&none, [0.0, 0.0, 4.0]);
        v.trees = [Some([10.0, 0.0, 0.0]), Some([-40.0, 0.0, 0.0]), None, None];
        g.state = Gnarfas::jump(&v);
        v.clip = (7, 1.0, true);
        g.tick(&v, &w, &mut rng);
        match g.state {
            State::Jump { step: 1, target, velocity, .. } => {
                assert_eq!(target, [10.0, 0.0, 0.0]);
                assert!((velocity[1] - 1.5).abs() < 1e-6 && (velocity[0] - 10.0 / 60.0).abs() < 1e-6);
            }
            s => panic!("{s:?}"),
        }
        // The body slam: one strike within 5 units, 30 damage, a strike counted.
        g.state = State::Slam { hit: false };
        let v = view(&none, [0.0, 0.0, 4.0]);
        let a = g.tick(&v, &w, &mut rng);
        assert_eq!(a.hero_damage, Some((30, false, 1.0)));
        assert_eq!(g.strikes, 1);
        // The head butt: event bit 0x1, the hero ahead and within 7: knockback 2.0, no damage.
        g.state = State::Butt { hit: false };
        let mut v = view(&none, [0.0, 0.0, 6.0]);
        v.frame_bits = 1;
        let a = g.tick(&v, &w, &mut rng);
        assert_eq!((a.knockback, a.hero_damage), (Some(2.0), None));
        assert_eq!(g.strikes, 2);
        // Three strikes send event 0.
        g.strikes = 3;
        g.state = State::Idle;
        assert!(g.tick(&view(&none, [0.0, 0.0, 30.0]), &w, &mut rng).set_level.contains(&0));
    }

    #[test]
    fn bone_cylinder_test() {
        // A bone 1.0 from the hero's axis overlaps (radii 0.5 + 1.0); 2.0 away does not.
        assert!(bone_hits([1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3]));
        assert!(!bone_hits([2.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3]));
        assert!(!bone_hits([0.0, 5.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3]));
    }
}
