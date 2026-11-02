//! The Undead Draik (SPEC-0029 Behavior 6; behaviour class 12, the Draik Skeleton): the stages
//! (1, 3 and 5; death is 6; level flag 0x11 forces stage 5), the seven cooldowns from the stage
//! tables, the attack choice by stage and distance band, the approach and facing movement, the
//! breath beam (fire or ice: the standoff, windup, beam length and end, the 60-tick hit gap, the
//! reflection to a stun), the spit, claw, wind (with its ground hazards), stomp and bite, the
//! stun hit registration and its latch, the hit reactions, the transitions with their broken
//! parts and level event 0, and the death with level event 12.
//!
//! The host measures what needs the world (the bone points, the hero's weapon and shot overlap
//! with the hit groups, the claw and bite weapon test, the hero's guard and shield Mote) and
//! applies the requests. Every tuning value is read from the boot data block at 0x2DA650.

use crate::projectile::Setup;
use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning block: 265 words from 0x2DA650 to 0x2DAA73.
pub const BASE: u32 = 0x002D_A650;
pub const WORDS: usize = 265;

/// The broken-part bones (left leg, right leg, tail, chest, left arm, right arm, left wing,
/// right wing, head).
pub const PART_BONES: [usize; 9] = [25, 26, 4, 1, 9, 17, 38, 5, 7];
/// Parts broken by the first transition, the second, and death.
pub const FIRST_BREAK: [usize; 3] = [0, 1, 2];
pub const SECOND_BREAK: [usize; 5] = [3, 4, 5, 6, 7];
pub const HEAD: usize = 8;
/// The effect that a breaking part spawns (0xFC0).
pub const BREAK_EFFECT: i32 = 0xFC0;

/// The cooldowns: breath, spit, claw, wind, tail, stomp, bite.
pub const COOLDOWNS: usize = 7;
const BREATH: usize = 0;
const SPIT: usize = 1;
const CLAW: usize = 2;
const WIND: usize = 3;
const STOMP: usize = 5;
const BITE: usize = 6;

/// A 5-entry stage table pair (min and max), indexed by stage - 1.
pub type Table = [(f32, f32); 5];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub cooldown: [Table; COOLDOWNS],
    pub breath_band: Table,
    pub standoff: Table,
    pub abort: [f32; 5],
    pub spit_band: Table,
    pub claw_band: Table,
    pub wind_band: Table,
    pub wind_approach: [f32; 5],
    pub stomp_band: Table,
    pub stomp_approach: [f32; 5],
    pub bite_band: Table,
    /// The approach distance of stages 1, 3 and 5.
    pub approach: [f32; 3],
    pub fire: (f32, f32),
    pub fire_damage: i32,
    pub ice: (f32, f32),
    pub ice_damage: i32,
    pub beam_rate: f32,
    pub wind_hold: (f32, f32),
    pub wind_idle: (f32, f32),
    pub wind_delay: (f32, f32),
    pub hazard_life: (f32, f32),
    pub hazard_throw: f32,
    /// The spit records' speed (fire 0x2DA9E8, ice 0x2DA9F8; units per tick), damage (0x2DA9F4,
    /// 0x2DAA04) and sphere radius (0x2DAA6C).
    pub spit_speed: [f32; 2],
    pub spit_damage: [i32; 2],
    pub spit_radius: f32,
    pub hazard_damage: i32,
    pub bite_damage: i32,
    pub hazard_slide: f32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let f = |at: u32| f32::from_bits(w(at));
        let i = |at: u32| w(at) as i32;
        let table = |lo: u32, hi: u32| -> Table { std::array::from_fn(|k| (f(lo + 4 * k as u32), f(hi + 4 * k as u32))) };
        let five = |at: u32| -> [f32; 5] { std::array::from_fn(|k| f(at + 4 * k as u32)) };
        let pair = |at: u32| (f(at), f(at + 4));
        Some(Self {
            cooldown: [
                table(0x002D_A650, 0x002D_A664),
                table(0x002D_A6DC, 0x002D_A6F0),
                table(0x002D_A72C, 0x002D_A740),
                table(0x002D_A7A4, 0x002D_A7B8),
                table(0x002D_A81C, 0x002D_A830),
                table(0x002D_A894, 0x002D_A8A8),
                table(0x002D_A90C, 0x002D_A920),
            ],
            breath_band: table(0x002D_A678, 0x002D_A68C),
            standoff: table(0x002D_A6A0, 0x002D_A6B4),
            abort: five(0x002D_A6C8),
            spit_band: table(0x002D_A704, 0x002D_A718),
            claw_band: table(0x002D_A754, 0x002D_A768),
            wind_band: table(0x002D_A7CC, 0x002D_A7E0),
            wind_approach: five(0x002D_A808),
            stomp_band: table(0x002D_A8BC, 0x002D_A8D0),
            stomp_approach: five(0x002D_A8F8),
            bite_band: table(0x002D_A934, 0x002D_A948),
            approach: [f(0x002D_A988), f(0x002D_A9A4), f(0x002D_A9C4)],
            fire: pair(0x002D_A9C8),
            fire_damage: i(0x002D_A9D4),
            ice: pair(0x002D_A9D8),
            ice_damage: i(0x002D_A9E4),
            beam_rate: f(0x002D_A9E8),
            wind_hold: pair(0x002D_AA20),
            wind_idle: pair(0x002D_AA2C),
            wind_delay: pair(0x002D_AA34),
            hazard_life: pair(0x002D_AA3C),
            hazard_throw: f(0x002D_AA44),
            spit_speed: [f(0x002D_A9E8), f(0x002D_A9F8)],
            spit_damage: [i(0x002D_A9F4), i(0x002D_AA04)],
            spit_radius: f(0x002D_AA6C),
            hazard_damage: i(0x002D_AA48),
            bite_damage: i(0x002D_AA68),
            hazard_slide: f(0x002D_AA70),
        })
    }
}

/// The breath's steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Breath {
    /// Keeping the standoff: approaching, or backing off toward this point.
    Standoff(Option<Vec3>),
    Windup,
    /// The beam: ticks left, length, end point, ticks to the next hit.
    Beam { left: i32, length: f32, end: Vec3, gap: i32 },
    /// A reflected ball on its way back to the chest.
    Reflected(Vec3),
    Recovery,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Approach,
    Facing,
    Breath { ice: bool, step: Breath },
    Spit { ice: bool, fired: bool },
    Claw { hit: bool },
    /// Wind: 0 approach, 1 the delay, 2 the hold, 3 the idle; the timer; the hazard spawned.
    Wind { step: u8, timer: i32, spawned: bool },
    Stomp { close: bool },
    Bite { close: bool, hit: bool },
    /// Stunned (26 in stage 1, 27 in stage 3) with the ticks left.
    Stun(i32),
    /// The hit reaction, keeping the stun's ticks (none in stage 5).
    Reaction(Option<i32>),
    Transition,
    Dead,
}

/// A ground hazard of the wind (4 slots).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hazard {
    pub position: Vec3,
    pub target: Vec3,
    pub renew: i32,
    pub life: i32,
    pub thrown: bool,
}

pub const HAZARDS: usize = 4;

/// The spit records' effects (SPEC-0029 Spit appearance): (flight, impact) for fire and ice.
pub const SPIT_FX: [(i32, i32); 2] = [(528, 991), (527, 992)];

/// A spit that ended this tick: its record (0 fire, 1 ice), its last position and the damage
/// it dealt to the hero (None when it reached the ground).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpitEnd {
    pub record: usize,
    pub position: Vec3,
    pub damage: Option<i32>,
}

/// A spit record (R-0066): the Draik keeps one for fire and one for ice, outside the enemy
/// pool. It moves at its speed each tick, hurts the hero on a sphere contact and ends there or
/// at the ground; it has no lifetime.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spit {
    pub position: Vec3,
    pub velocity: Vec3,
    pub damage: i32,
}

/// What the Draik sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hp: i32,
    pub max_hp: i32,
    /// The mouth and chest points (the beam start and the reflected ball's end).
    pub mouth: Vec3,
    pub chest: Vec3,
    /// The hero's melee attack is active; a hero shot slot is active.
    pub melee: bool,
    pub shots: bool,
    /// A hit group of this stage passes: by the hero's weapon, by a hero shot.
    pub melee_group: bool,
    pub shot_group: bool,
    /// The claw or bite weapon test touches the hero this tick.
    pub weapon_contact: bool,
    /// The beam touches the hero this tick; the hero would reflect a fire or an ice beam.
    pub beam_touches: &'a dyn Fn(Vec3, Vec3) -> bool,
    /// The hero's forward axis, and whether it guards with a Mote that reflects a fire or an ice
    /// beam (the facing test against the beam is made here).
    pub hero_facing: Vec3,
    pub reflects: [bool; 2],
    pub clip: (u32, bool),
    pub frame_bits: u32,
    pub level: &'a dyn Fn(u16) -> bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// The locomotion clip toward a point (the move direction steers the facing at the weight).
    Toward(Vec3, f32),
    /// Turn toward the hero at the weight with the current clip's own travel.
    Turn(f32),
    Still,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    pub travel: Move,
    /// Damage to the hero (boss melee or beam, unreduced) with the knock-up throw (speed) of a
    /// hazard.
    pub hero_damage: Vec<(i32, Option<(Vec3, f32)>)>,
    pub launch: Option<Setup>,
    pub hp_loss: i32,
    pub set_level: Vec<u16>,
    pub breaks: Vec<usize>,
    /// A counted hero shot is used up.
    pub consume_shot: bool,
    /// The beam from the mouth to this end, for drawing.
    pub beam: Option<(Vec3, f32, bool)>,
}

impl Default for Act {
    fn default() -> Self {
        Self { anim: None, travel: Move::Still, hero_damage: Vec::new(), launch: None, hp_loss: 0, set_level: Vec::new(), breaks: Vec::new(), consume_shot: false, beam: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Draik {
    pub stage: u8,
    pub state: State,
    pub cooldown: [i32; COOLDOWNS],
    pub broken: [bool; 9],
    pub hazards: Vec<Hazard>,
    latch: bool,
    /// The fire and ice spit records.
    pub spits: [Option<Spit>; 2],
}

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

fn draw(rng: &mut Rng, (lo, hi): (f32, f32)) -> i32 {
    rng.range(ticks(lo), ticks(hi))
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len(a: Vec3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Move a point toward a target by a step, landing on it when closer.
fn toward(p: Vec3, t: Vec3, step: f32) -> Vec3 {
    let d = sub(t, p);
    let l = len(d);
    if l <= step || l == 0.0 {
        t
    } else {
        std::array::from_fn(|k| p[k] + d[k] * step / l)
    }
}

/// The beam length step: grow by the rate up to the target, or drop to it at once.
pub fn beam_length(l: f32, target: f32, rate: f32) -> f32 {
    if l < target { (l + rate).min(target) } else { target }
}

fn open(band: (f32, f32), d: f32) -> bool {
    band.0 < d && d < band.1
}

impl Draik {
    /// Guarding for the hero's swing (SPEC-0029 G11: base state 20 with Blocking Angle 2.0):
    /// every tick of stages 1 and 3 outside the stun and the hit reactions; never in stage 5.
    pub fn guarding(&self) -> bool {
        self.stage < 5 && !matches!(self.state, State::Stun(_) | State::Reaction(_) | State::Dead)
    }

    /// The spit records' tick: each moves by its velocity, then hurts the hero and ends when its
    /// sphere touches the hero (`hits`), or ends when its height reaches the ground under it
    /// (`ground`, the probe from 8 above). Returns the spits that ended.
    pub fn spit_step(&mut self, w: &Words, hits: &dyn Fn(Vec3, f32) -> bool, ground: &dyn Fn(Vec3) -> Option<f32>) -> Vec<SpitEnd> {
        let mut out = Vec::new();
        for (record, slot) in self.spits.iter_mut().enumerate() {
            let Some(mut sp) = *slot else { continue };
            sp.position = [sp.position[0] + sp.velocity[0], sp.position[1] + sp.velocity[1], sp.position[2] + sp.velocity[2]];
            *slot = if hits(sp.position, w.spit_radius) {
                out.push(SpitEnd { record, position: sp.position, damage: Some(sp.damage) });
                None
            } else if ground(sp.position).is_some_and(|g| sp.position[1] <= g) {
                out.push(SpitEnd { record, position: sp.position, damage: None });
                None
            } else {
                Some(sp)
            };
        }
        out
    }

    /// Setup: stage 1, every cooldown drawn from the stage-1 range.
    pub fn new(w: &Words, rng: &mut Rng) -> Self {
        let cooldown = std::array::from_fn(|k| draw(rng, w.cooldown[k][0]));
        Self { stage: 1, state: State::Idle, cooldown, broken: [false; 9], hazards: Vec::new(), latch: false, spits: [None; 2] }
    }

    fn s(&self) -> usize {
        (self.stage.clamp(1, 5) - 1) as usize
    }

    /// The stage's animation from a (stage 1, 3, 5) triple.
    fn by_stage(&self, a: [u32; 3]) -> u32 {
        match self.stage {
            1 => a[0],
            3 => a[1],
            _ => a[2],
        }
    }

    fn free(&self) -> bool {
        matches!(self.state, State::Idle | State::Approach | State::Facing)
    }

    fn set(&mut self, k: usize, w: &Words, rng: &mut Rng) {
        self.cooldown[k] = draw(rng, w.cooldown[k][self.s()]);
    }

    /// The attack choice of the stage (6.3), when one starts.
    fn choose(&self, d: f32, w: &Words, rng: &mut Rng) -> Option<State> {
        let s = self.s();
        let ready = |k: usize| self.cooldown[k] == 0;
        let breath = || ready(BREATH) && open(w.breath_band[s], d);
        let spit = || ready(SPIT) && open(w.spit_band[s], d);
        let coin = |rng: &mut Rng| rng.range(0, 1) == 1;
        match self.stage {
            1 => {
                if breath() {
                    return Some(State::Breath { ice: coin(rng), step: Breath::Standoff(None) });
                }
                if spit() {
                    return Some(State::Spit { ice: coin(rng), fired: false });
                }
                if ready(CLAW) && open(w.claw_band[s], d) {
                    return Some(State::Claw { hit: false });
                }
                None
            }
            3 => {
                if breath() {
                    return Some(State::Breath { ice: coin(rng), step: Breath::Standoff(None) });
                }
                if spit() {
                    return Some(State::Spit { ice: coin(rng), fired: false });
                }
                let mut pick = None;
                if ready(WIND) && open(w.wind_band[s], d) {
                    pick = Some(State::Wind { step: 0, timer: 0, spawned: false });
                }
                if ready(STOMP) && open(w.stomp_band[s], d) {
                    pick = Some(State::Stomp { close: false });
                }
                pick
            }
            5 => {
                if spit() {
                    return Some(State::Spit { ice: coin(rng), fired: false });
                }
                let mut pick = None;
                if ready(STOMP) && open(w.stomp_band[s], d) {
                    pick = Some(State::Stomp { close: false });
                }
                if ready(BITE) && open(w.bite_band[s], d) {
                    pick = Some(State::Bite { close: false, hit: false });
                }
                pick
            }
            _ => None,
        }
    }

    /// One tick.
    pub fn tick(&mut self, v: &View, w: &Words, _owner: u64, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let d = hdist(v.position, v.hero);
        let to = [v.hero[0] - v.position[0], 0.0, v.hero[2] - v.position[2]];
        let c = if d > 1e-6 { (to[0] * v.facing[0] + to[2] * v.facing[2]) / d } else { 1.0 };
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let s = self.s();
        for t in &mut self.cooldown {
            *t = (*t - 1).max(0);
        }
        self.hazards_tick(v, w, rng, &mut a);
        // Level flag 0x11 forces stage 5 with every part broken.
        if (v.level)(0x11) && self.stage < 5 {
            self.stage = 5;
            a.breaks.extend((0..9).filter(|&k| !self.broken[k]));
            self.broken = [true; 9];
            self.state = State::Idle;
        }
        let per_hit = (0.025 * v.max_hp as f32) as i32;
        // The hit registration: the stuns of stages 1 and 3 (not during the reaction), any time
        // in stage 5 (not in the reaction, the transition or death).
        let counts = match (self.stage, self.state) {
            (1 | 3, State::Stun(_)) => true,
            (5, st) => !matches!(st, State::Reaction(_) | State::Transition | State::Dead),
            _ => false,
        };
        if counts {
            if !v.melee && !v.shots {
                self.latch = false;
            } else if (!self.latch || v.shots) && (v.melee_group || v.shot_group) {
                self.latch = true;
                a.consume_shot = v.shot_group;
                if self.stage == 5 && v.hp - per_hit <= 0 {
                    a.hp_loss = v.hp;
                    self.state = State::Dead;
                } else {
                    a.hp_loss = per_hit;
                    let stun = if let State::Stun(t) = self.state { Some(t) } else { None };
                    self.state = State::Reaction(stun);
                }
            }
        }
        // The free states: the attack choice, else the movement.
        if self.free() {
            if let Some(next) = self.choose(d, w, rng) {
                self.state = next;
            } else if d >= w.approach[s / 2] {
                self.state = State::Approach;
            } else if self.state == State::Facing && c > 0.95 {
                self.state = State::Idle;
            } else if c <= 0.5 {
                self.state = State::Facing;
            } else if self.state == State::Approach {
                self.state = State::Idle;
            }
        }
        let hero_up = [v.hero[0], v.hero[1] + 1.5, v.hero[2]];
        match self.state {
            State::Idle => a.anim = Some(self.by_stage([1, 26, 49])),
            State::Approach => {
                a.anim = Some(self.by_stage([2, 27, 50]));
                a.travel = Move::Toward(v.hero, 0.25);
            }
            State::Facing => {
                a.anim = Some(self.by_stage([10, 34, 55]));
                a.travel = Move::Turn(0.1);
            }
            State::Breath { ice, step } => {
                let next = match step {
                    Breath::Standoff(back) => {
                        let (lo, hi) = w.standoff[s];
                        if let Some(p) = back {
                            a.anim = Some(self.by_stage([2, 27, 50]));
                            a.travel = Move::Toward(p, 1.0);
                            if hdist(v.position, p) < 1.0 { Breath::Standoff(None) } else { step }
                        } else if d > hi {
                            a.anim = Some(self.by_stage([2, 27, 50]));
                            a.travel = Move::Toward(v.hero, 0.25);
                            step
                        } else if d < lo {
                            // Back off (R-0066): each tick toward the point 5 units behind its own
                            // position, along the horizontal direction from the hero, at its own
                            // height; recomputed every tick until d reaches the low bound.
                            let u = if d > 1e-6 { [-to[0] / d, 0.0, -to[2] / d] } else { [0.0; 3] };
                            a.anim = Some(self.by_stage([2, 27, 50]));
                            a.travel = Move::Toward([v.position[0] + 5.0 * u[0], v.position[1], v.position[2] + 5.0 * u[2]], 1.0);
                            step
                        } else {
                            Breath::Windup
                        }
                    }
                    Breath::Windup => {
                        let n = self.by_stage([3, 28, 28]);
                        a.anim = Some(n);
                        a.travel = Move::Turn(0.1);
                        if ended(n) {
                            Breath::Beam { left: draw(rng, if ice { w.ice } else { w.fire }), length: 0.0, end: v.mouth, gap: 0 }
                        } else {
                            step
                        }
                    }
                    Breath::Beam { left, length, end, gap } => {
                        a.anim = Some(self.by_stage([4, 29, 29]));
                        let end = toward(end, hero_up, 0.25);
                        let length = beam_length(length, len(sub(end, v.mouth)), w.beam_rate);
                        let dir = sub(end, v.mouth);
                        let l = len(dir).max(1e-6);
                        let tip = std::array::from_fn(|k| v.mouth[k] + dir[k] / l * length);
                        a.beam = Some((tip, length, ice));
                        let mut gap = (gap - 1).max(0);
                        let mut next = Breath::Beam { left: left - 1, length, end, gap };
                        if gap == 0 && (v.beam_touches)(v.mouth, tip) {
                            // The hero faces the beam: its forward axis against the beam's direction.
                            let faces = v.hero_facing[0] * -dir[0] + v.hero_facing[2] * -dir[2] > 0.0;
                            if v.reflects[ice as usize] && faces {
                                next = Breath::Reflected(tip);
                            } else {
                                a.hero_damage.push((if ice { w.ice_damage } else { w.fire_damage }, None));
                                gap = 60;
                                next = Breath::Beam { left: left - 1, length, end, gap };
                            }
                        }
                        if matches!(next, Breath::Beam { .. }) && (d >= w.abort[s] || left - 1 <= 0) {
                            Breath::Recovery
                        } else {
                            next
                        }
                    }
                    Breath::Reflected(ball) => {
                        let ball = toward(ball, v.chest, 0.25);
                        a.beam = None;
                        if ball == v.chest {
                            let t = draw(rng, w.cooldown[BREATH][s]);
                            self.cooldown[BREATH] = if self.stage == 3 { 2 * t } else { t };
                            self.state = State::Stun(360);
                            a.anim = Some(self.by_stage([12, 36, 36]));
                            return a;
                        }
                        Breath::Reflected(ball)
                    }
                    Breath::Recovery => {
                        let n = self.by_stage([5, 30, 30]);
                        a.anim = Some(n);
                        if ended(n) {
                            self.set(BREATH, w, rng);
                            self.state = State::Idle;
                            return a;
                        }
                        step
                    }
                };
                self.state = State::Breath { ice, step: next };
            }
            State::Spit { ice, fired } => {
                let n = self.by_stage([6, 31, 52]);
                a.anim = Some(n);
                a.travel = Move::Turn(0.25);
                // Released on the first tick event bit 0x20 is active, once, at the mouth (bone
                // 52), toward the hero raised 2 (R-0066).
                if v.clip.0 == n && v.frame_bits & 0x20 != 0 && !fired {
                    let aim = [v.hero[0], v.hero[1] + 2.0, v.hero[2]];
                    let dir = sub(aim, v.mouth);
                    let l = len(dir).max(1e-6);
                    let k = ice as usize;
                    self.spits[k] = Some(Spit { position: v.mouth, velocity: dir.map(|x| x / l * w.spit_speed[k]), damage: w.spit_damage[k] });
                    self.state = State::Spit { ice, fired: true };
                }
                if ended(n) {
                    self.set(SPIT, w, rng);
                    self.state = State::Idle;
                }
            }
            State::Claw { hit } => {
                a.anim = Some(7);
                if v.clip.0 == 7 && v.weapon_contact && !hit {
                    a.hero_damage.push((20, None));
                    self.state = State::Claw { hit: true };
                }
                if ended(7) {
                    self.set(CLAW, w, rng);
                    self.state = State::Idle;
                }
            }
            State::Wind { step, timer, spawned } => {
                let idle = self.by_stage([1, 26, 49]);
                self.state = match step {
                    0 => {
                        a.anim = Some(self.by_stage([2, 27, 50]));
                        a.travel = Move::Toward(v.hero, 0.25);
                        if d <= w.wind_approach[s] { State::Wind { step: 1, timer: draw(rng, w.wind_delay), spawned } } else { self.state }
                    }
                    1 => {
                        a.anim = Some(idle);
                        a.travel = Move::Turn(0.25);
                        if timer <= 0 { State::Wind { step: 2, timer: draw(rng, w.wind_hold), spawned } } else { State::Wind { step, timer: timer - 1, spawned } }
                    }
                    2 => {
                        a.anim = Some(31);
                        a.travel = Move::Turn(0.25);
                        let mut spawned = spawned;
                        if v.clip.0 == 31 && v.frame_bits != 0 && !spawned && self.hazards.len() < HAZARDS {
                            spawned = true;
                            self.hazards.push(Hazard { position: v.position, target: v.hero, renew: 0, life: draw(rng, w.hazard_life), thrown: false });
                        }
                        // The idle time is used as ticks, not multiplied by 60 (Edge cases).
                        if timer <= 0 { State::Wind { step: 3, timer: rng.range(w.wind_idle.0 as i32, w.wind_idle.1 as i32), spawned } } else { State::Wind { step, timer: timer - 1, spawned } }
                    }
                    _ => {
                        a.anim = Some(idle);
                        if timer <= 0 {
                            self.set(WIND, w, rng);
                            State::Idle
                        } else {
                            State::Wind { step, timer: timer - 1, spawned }
                        }
                    }
                };
            }
            State::Stomp { close } => {
                if !close && d > w.stomp_approach[s] {
                    a.anim = Some(self.by_stage([2, 27, 50]));
                    a.travel = Move::Toward(v.hero, 0.25);
                } else {
                    let n = self.by_stage([32, 32, 53]);
                    a.anim = Some(n);
                    self.state = State::Stomp { close: true };
                    if ended(n) {
                        self.set(STOMP, w, rng);
                        self.state = State::Idle;
                    }
                }
            }
            State::Bite { close, hit } => {
                if !close && d > 6.0 {
                    a.anim = Some(self.by_stage([2, 27, 50]));
                    a.travel = Move::Toward(v.hero, 0.25);
                } else {
                    a.anim = Some(51);
                    let mut hit = hit;
                    if v.clip.0 == 51 && v.weapon_contact && !hit {
                        hit = true;
                        a.hero_damage.push((w.bite_damage, None));
                    }
                    self.state = State::Bite { close: true, hit };
                    if ended(51) {
                        self.set(BITE, w, rng);
                        self.state = State::Idle;
                    }
                }
            }
            State::Stun(t) => {
                a.anim = Some(self.by_stage([12, 36, 36]));
                self.state = if t <= 1 { State::Idle } else { State::Stun(t - 1) };
            }
            State::Reaction(stun) => {
                let n = self.by_stage([23, 46, 54]);
                a.anim = Some(n);
                let stun = stun.map(|t| t - 1);
                self.state = State::Reaction(stun);
                if ended(n) {
                    let f = if v.max_hp > 0 { (v.hp - a.hp_loss) as f32 / v.max_hp as f32 } else { 0.0 };
                    self.state = match self.stage {
                        1 if f <= 0.66 => {
                            a.set_level.push(0);
                            a.breaks.extend(FIRST_BREAK);
                            FIRST_BREAK.iter().for_each(|&k| self.broken[k] = true);
                            State::Transition
                        }
                        3 if f <= 0.33 => {
                            a.set_level.push(0);
                            a.breaks.extend(SECOND_BREAK);
                            SECOND_BREAK.iter().for_each(|&k| self.broken[k] = true);
                            State::Transition
                        }
                        _ => match stun {
                            Some(t) if t > 0 => State::Stun(t),
                            _ => State::Idle,
                        },
                    };
                }
            }
            State::Transition => {
                let n = if self.stage == 1 { 15 } else { 39 };
                a.anim = Some(n);
                if ended(n) {
                    self.stage = if self.stage == 1 { 3 } else { 5 };
                    self.state = State::Idle;
                }
            }
            State::Dead => {
                a.anim = Some(56);
                if ended(56) {
                    if !self.broken[HEAD] {
                        self.broken[HEAD] = true;
                        a.breaks.push(HEAD);
                    }
                    // Repeats each tick while the finished animation is held.
                    a.set_level.push(12);
                    self.stage = 6;
                }
            }
        }
        a
    }

    /// The wind hazards: life, the target renewed every 60 ticks within 5 of the hero, the
    /// slide, and the throw within 4.0 (once per hazard).
    fn hazards_tick(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        for h in &mut self.hazards {
            h.life -= 1;
            h.renew -= 1;
            if h.renew <= 0 {
                h.renew = 60;
                h.target = [v.hero[0] + rng.frange(-5.0, 5.0), v.hero[1], v.hero[2] + rng.frange(-5.0, 5.0)];
            }
            let t = [h.target[0], h.position[1], h.target[2]];
            h.position = toward(h.position, t, w.hazard_slide);
            if !h.thrown && hdist(h.position, v.hero) < 4.0 {
                h.thrown = true;
                let d = hdist(h.position, v.hero).max(1e-6);
                let u = [(v.hero[0] - h.position[0]) / d, 0.0, (v.hero[2] - h.position[2]) / d];
                a.hero_damage.push((w.hazard_damage, Some((u, w.hazard_throw))));
            }
        }
        self.hazards.retain(|h| h.life > 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disc values of SPEC-0029 Behavior 6 as a synthetic block (equal stage entries).
    fn block() -> Vec<u32> {
        let mut b = vec![0u32; WORDS];
        let mut put = |at: u32, v: u32| b[((at - BASE) / 4) as usize] = v;
        let tables: [(u32, u32, f32, f32); 15] = [
            (0x002D_A650, 0x002D_A664, 4.0, 8.0),
            (0x002D_A6DC, 0x002D_A6F0, 1.0, 2.0),
            (0x002D_A72C, 0x002D_A740, 1.0, 3.0),
            (0x002D_A7A4, 0x002D_A7B8, 4.0, 8.0),
            (0x002D_A81C, 0x002D_A830, 2.0, 5.0),
            (0x002D_A894, 0x002D_A8A8, 2.0, 5.0),
            (0x002D_A90C, 0x002D_A920, 2.0, 5.0),
            (0x002D_A678, 0x002D_A68C, 10.0, 20.0),
            (0x002D_A6A0, 0x002D_A6B4, 10.0, 15.0),
            (0x002D_A704, 0x002D_A718, 5.0, 15.0),
            (0x002D_A754, 0x002D_A768, 0.0, 10.0),
            (0x002D_A7CC, 0x002D_A7E0, 5.0, 10.0),
            (0x002D_A8BC, 0x002D_A8D0, 0.0, 5.0),
            (0x002D_A934, 0x002D_A948, 0.0, 10.0),
            (0x002D_A6C8, 0x002D_A6C8, 25.0, 25.0),
        ];
        for (lo, hi, x, y) in tables {
            for k in 0..5 {
                put(lo + 4 * k, x.to_bits());
                put(hi + 4 * k, y.to_bits());
            }
        }
        for (at, v) in [(0x002D_A808, 10.0f32), (0x002D_A8F8, 7.0)] {
            for k in 0..5 {
                put(at + 4 * k, v.to_bits());
            }
        }
        for (at, v) in [(0x002D_A988, 5.0f32), (0x002D_A9A4, 10.0), (0x002D_A9C4, 10.0), (0x002D_A9C8, 2.0), (0x002D_A9CC, 5.0), (0x002D_A9D8, 2.0), (0x002D_A9DC, 5.0), (0x002D_A9E8, 1.0), (0x002D_AA70, 0.1), (0x002D_AA44, 10.0)] {
            put(at, v.to_bits());
        }
        for at in [0x002D_A9D4, 0x002D_A9E4, 0x002D_AA48, 0x002D_AA68] {
            put(at, 10);
        }
        b
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, touch: &'a dyn Fn(Vec3, Vec3) -> bool, d: f32) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, d],
            hp: 1000,
            max_hp: 1000,
            mouth: [0.0, 4.0, 1.0],
            chest: [0.0, 3.0, 0.0],
            melee: false,
            shots: false,
            melee_group: false,
            shot_group: false,
            weapon_contact: false,
            beam_touches: touch,
            hero_facing: [0.0, 0.0, -1.0],
            reflects: [false; 2],
            clip: (1, false),
            frame_bits: 0,
            level,
        }
    }

    #[test]
    fn spit_records_move_hit_and_end_at_the_ground() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(1);
        let mut k = Draik::new(&w, &mut rng);
        k.spits[0] = Some(Spit { position: [0.0, 5.0, 0.0], velocity: [0.0, -1.0, 0.0], damage: 10 });
        let never = |_: Vec3, _: f32| false;
        let ground = |_: Vec3| Some(3.0);
        assert!(k.spit_step(&w, &never, &ground).is_empty());
        assert_eq!(k.spits[0].map(|s| s.position[1]), Some(4.0));
        assert_eq!(k.spit_step(&w, &never, &ground), vec![SpitEnd { record: 0, position: [0.0, 3.0, 0.0], damage: None }]);
        assert!(k.spits[0].is_none(), "at the ground height it ends");
        k.spits[1] = Some(Spit { position: [0.0, 9.0, 0.0], velocity: [1.0, 0.0, 0.0], damage: 10 });
        let always = |_: Vec3, _: f32| true;
        assert_eq!(k.spit_step(&w, &always, &ground), vec![SpitEnd { record: 1, position: [1.0, 9.0, 0.0], damage: Some(10) }]);
        assert!(k.spits[1].is_none());
    }

    #[test]
    fn cooldowns_choice_and_movement() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        let never = |_: Vec3, _: Vec3| false;
        let k = Draik::new(&w, &mut rng);
        assert!((240..=480).contains(&k.cooldown[BREATH]));
        // Stage 1, d 7 with spit and claw ready: spit (its band is tested first).
        let mut k = Draik::new(&w, &mut rng);
        k.cooldown = [100, 1, 1, 100, 100, 100, 100];
        k.tick(&view(&no, &never, 7.0), &w, 1, &mut rng);
        assert!(matches!(k.state, State::Spit { .. }));
        // Stage 5, d 6 with stomp and bite ready: bite.
        let mut k = Draik::new(&w, &mut rng);
        k.stage = 5;
        k.cooldown = [100, 100, 100, 100, 100, 1, 1];
        k.tick(&view(&no, &never, 6.0), &w, 1, &mut rng);
        assert!(matches!(k.state, State::Bite { .. }));
        // Stage 1 movement: approach at 5 or more; facing at cosine 0.5 with d 3, kept at 0.9,
        // ended at 0.96.
        let mut k = Draik::new(&w, &mut rng);
        k.cooldown = [100; 7];
        k.tick(&view(&no, &never, 5.0), &w, 1, &mut rng);
        assert_eq!(k.state, State::Approach);
        let side = |c: f32| {
            let mut v = view(&no, &never, 3.0);
            v.hero = [3.0 * (1.0 - c * c).sqrt(), 0.0, 3.0 * c];
            v
        };
        let mut k = Draik::new(&w, &mut rng);
        k.cooldown = [100; 7];
        k.tick(&side(0.5), &w, 1, &mut rng);
        assert_eq!(k.state, State::Facing);
        k.tick(&side(0.9), &w, 1, &mut rng);
        assert_eq!(k.state, State::Facing);
        k.tick(&side(0.96), &w, 1, &mut rng);
        assert_eq!(k.state, State::Idle);
    }

    #[test]
    fn beam_length_vectors() {
        let mut l = 0.0;
        let seen: Vec<f32> = (0..4).map(|_| {
            l = beam_length(l, 3.4, 1.0);
            l
        }).collect();
        assert_eq!(seen, vec![1.0, 2.0, 3.0, 3.4]);
        assert_eq!(beam_length(3.4, 1.2, 1.0), 1.2);
    }

    /// SPEC-0029 Forcing the wind and the stomp (R-0180): stage 3's vectors; stage 1 has neither.
    #[test]
    fn wind_and_stomp_by_stage() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        let never = |_: Vec3, _: Vec3| false;
        let mut k = Draik::new(&w, &mut rng);
        k.stage = 3;
        k.state = State::Idle;
        k.cooldown = [200; COOLDOWNS];
        (k.cooldown[SPIT], k.cooldown[WIND], k.cooldown[STOMP]) = (40, 0, 150);
        k.tick(&view(&no, &never, 8.0), &w, 1, &mut rng);
        assert!(matches!(k.state, State::Wind { .. }), "{:?}", k.state);
        let mut k = Draik::new(&w, &mut rng);
        k.stage = 3;
        k.state = State::Idle;
        k.cooldown = [200; COOLDOWNS];
        k.cooldown[STOMP] = 0;
        k.tick(&view(&no, &never, 3.0), &w, 1, &mut rng);
        assert!(matches!(k.state, State::Stomp { .. }), "{:?}", k.state);
        // Stage 1 has neither, with both ready.
        let mut k = Draik::new(&w, &mut rng);
        k.state = State::Idle;
        k.cooldown = [200; COOLDOWNS];
        (k.cooldown[WIND], k.cooldown[STOMP]) = (0, 0);
        k.tick(&view(&no, &never, 3.0), &w, 1, &mut rng);
        assert!(!matches!(k.state, State::Wind { .. } | State::Stomp { .. }));
    }

    #[test]
    fn breath_hits_and_reflection() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(2);
        let no = |_: u16| false;
        let always = |_: Vec3, _: Vec3| true;
        // A beam on an unguarded hero: 10, then 60 ticks before the next.
        let mut k = Draik::new(&w, &mut rng);
        k.state = State::Breath { ice: false, step: Breath::Beam { left: 200, length: 0.0, end: [0.0, 4.0, 1.0], gap: 0 } };
        let v = view(&no, &always, 12.0);
        let hits: usize = (0..61).map(|_| k.tick(&v, &w, 1, &mut rng).hero_damage.len()).sum();
        assert_eq!(hits, 2);
        // A reflecting hero: the ball returns at 0.25 per tick, then the 360-tick stun.
        let mut k = Draik::new(&w, &mut rng);
        k.state = State::Breath { ice: true, step: Breath::Beam { left: 200, length: 0.0, end: [0.0, 3.0, 1.0], gap: 0 } };
        let mut v = view(&no, &always, 12.0);
        v.reflects = [false, true];
        k.tick(&v, &w, 1, &mut rng);
        assert!(matches!(k.state, State::Breath { step: Breath::Reflected(_), .. }));
        for _ in 0..20 {
            k.tick(&v, &w, 1, &mut rng);
        }
        assert!(matches!(k.state, State::Stun(_)));
        assert!((240..=480).contains(&k.cooldown[BREATH]));
    }

    #[test]
    fn stun_hits_transitions_and_death() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        let never = |_: Vec3, _: Vec3| false;
        // A stunned stage-1 Draik with max HP 1000 loses 25 per hit; a swing counts once.
        let mut k = Draik::new(&w, &mut rng);
        k.cooldown = [100; 7];
        k.state = State::Stun(300);
        let mut v = view(&no, &never, 3.0);
        v.melee = true;
        v.melee_group = true;
        assert_eq!(k.tick(&v, &w, 1, &mut rng).hp_loss, 25);
        assert!(matches!(k.state, State::Reaction(Some(_))));
        k.state = State::Stun(300);
        assert_eq!(k.tick(&v, &w, 1, &mut rng).hp_loss, 0, "the latch holds a swing");
        v.melee = false;
        v.melee_group = false;
        k.tick(&v, &w, 1, &mut rng);
        // A shot counts each time and is used up.
        k.state = State::Stun(300);
        v.shots = true;
        v.shot_group = true;
        let a = k.tick(&v, &w, 1, &mut rng);
        assert!(a.consume_shot && a.hp_loss == 25);
        // After the stage-1 reaction with HP 660 or less: event 0, legs and tail, stage 3.
        let mut k = Draik::new(&w, &mut rng);
        k.state = State::Reaction(Some(100));
        let mut v = view(&no, &never, 3.0);
        v.hp = 660;
        v.clip = (23, true);
        let a = k.tick(&v, &w, 1, &mut rng);
        assert_eq!((a.set_level.clone(), a.breaks.clone(), k.state), (vec![0], vec![0, 1, 2], State::Transition));
        v.clip = (15, true);
        k.tick(&v, &w, 1, &mut rng);
        assert_eq!((k.stage, k.state), (3, State::Idle));
        // Above 0.66 the stun resumes.
        let mut k = Draik::new(&w, &mut rng);
        k.state = State::Reaction(Some(100));
        v.hp = 700;
        v.clip = (23, true);
        k.tick(&v, &w, 1, &mut rng);
        assert_eq!(k.state, State::Stun(99));
        // Stage 5: a killing hit; the death's end breaks the head and posts 12 each tick.
        let mut k = Draik::new(&w, &mut rng);
        k.stage = 5;
        k.cooldown = [100; 7];
        let mut v = view(&no, &never, 3.0);
        v.hp = 20;
        v.melee = true;
        v.melee_group = true;
        let a = k.tick(&v, &w, 1, &mut rng);
        assert_eq!((a.hp_loss, k.state), (20, State::Dead));
        v.clip = (56, true);
        let a = k.tick(&v, &w, 1, &mut rng);
        assert_eq!((a.breaks.clone(), a.set_level.clone(), k.stage), (vec![HEAD], vec![12], 6));
        assert_eq!(k.tick(&v, &w, 1, &mut rng).set_level, vec![12]);
        // Level flag 0x11: stage 5 with every part broken.
        let mut k = Draik::new(&w, &mut rng);
        let force = |f: u16| f == 0x11;
        let a = k.tick(&view(&force, &never, 30.0), &w, 1, &mut rng);
        assert_eq!((k.stage, a.breaks.len()), (5, 9));
    }

    #[test]
    fn wind_hazard() {
        let w = Words::read(&block()).unwrap();
        let mut rng = Rng::seeded(4);
        let no = |_: u16| false;
        let never = |_: Vec3, _: Vec3| false;
        let mut k = Draik::new(&w, &mut rng);
        k.hazards.push(Hazard { position: [0.0, 0.0, 10.0], target: [0.0, 0.0, 20.0], renew: 60, life: 600, thrown: false });
        // The hazard slides 0.1 per tick; within 4.0 of the hero it throws once for 10 at 10.0.
        let v = view(&no, &never, 30.0);
        let mut k2 = k.clone();
        k2.hazards[0].position = [0.0, 0.0, 26.5];
        let a = k2.tick(&v, &w, 1, &mut rng);
        assert_eq!(a.hero_damage.len(), 1);
        assert_eq!(a.hero_damage[0].0, 10);
        assert_eq!(a.hero_damage[0].1.map(|(_, s)| s), Some(10.0));
        assert!(k2.tick(&v, &w, 1, &mut rng).hero_damage.is_empty());
        k.tick(&v, &w, 1, &mut rng);
        assert!((k.hazards[0].position[2] - 10.1).abs() < 1e-4);
    }
}
