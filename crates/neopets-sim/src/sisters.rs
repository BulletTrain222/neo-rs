//! The Faerie Sisters (SPEC-0011; behaviour classes 13, 15 and 16): the shared bookkeeping (the
//! alive count, the shield, level event 2 once), each sister's state machine (idle, the orbit,
//! the two dodges, the energy ball, the status and special missiles, the flinch and death),
//! taking damage through the shield, the chooser bands, the speed scale P = 2 - f, and the
//! alternate loops of level flag 0 (idle, move and the alternate energy ball; loop A's word
//! overrides). The group (dome) attack never starts in the shipped game and is not built.
//!
//! The host measures what needs the world (the hero's hits, the hero projectiles near a sister,
//! the clip and its event bits, the blocked flag) and applies the requests: the clip, the travel,
//! the launches into the enemy projectile pool, the effects and the level events. Every tuning
//! value is read from the boot data block at 0x2DAAC0 (SPEC-0011 Layout).

use crate::projectile::Setup;
use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The leach drain word (50 MP on disc; SPEC-0011 Leach), outside the tuning block.
pub const LEACH_DRAIN: u32 = 0x002D_D458;

/// The tuning block: 56 words from 0x2DAAC0 to 0x2DAB9F.
pub const BASE: u32 = 0x002D_AAC0;
pub const WORDS: usize = 56;

/// The tuning words (SPEC-0011 Layout). The arrays indexed by alive count hold the values for 1,
/// 2 and 3 alive; the "label" arrays hold labels 1, 2 and 3 (label 1 is used with 3 alive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub dodge_missile: [i32; 3],
    pub dodge_melee: [i32; 3],
    pub energy: [(f32, f32); 3],
    pub status: [(f32, f32); 3],
    pub special: [(f32, f32); 3],
    /// The group-attack interval (0x2DAB28 to 0x2DAB3C), drawn only at a sister's start.
    pub group: [(f32, f32); 3],
    pub projectile_speed: [f32; 3],
    pub min_distance: f32,
    pub max_distance: f32,
    pub movement_speed: f32,
    pub ball_damage: i32,
    /// Damage and chance by status kind: 0 poison, 1 curse, 2 slow.
    pub status_damage: [i32; 3],
    pub status_chance: [i32; 3],
    /// Damage by special kind: 0 thorn, 1 leach, 2 Mote eater.
    pub special_damage: [i32; 3],
    /// The leach's MP drain (word 0x2DD458).
    pub leach_drain: i32,
    /// ThornAttackMaxHits (0x2DAB84): the strikes that remove a thorn hazard.
    pub thorn_max_hits: i32,
}

impl Words {
    /// The words from the block read at `BASE`, and the leach drain word.
    pub fn read(b: &[u32], leach_drain: u32) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let i = |at: u32| w(at) as i32;
        let f = |at: u32| f32::from_bits(w(at));
        let pairs = |at: u32| [0, 1, 2].map(|k| (f(at + 8 * k), f(at + 8 * k + 4)));
        Some(Self {
            dodge_missile: [0, 1, 2].map(|k| i(0x002D_AAC8 + 4 * k)),
            dodge_melee: [0, 1, 2].map(|k| i(0x002D_AAD4 + 4 * k)),
            energy: pairs(0x002D_AAE0),
            status: pairs(0x002D_AAF8),
            special: pairs(0x002D_AB10),
            group: pairs(0x002D_AB28),
            projectile_speed: [0, 1, 2].map(|k| f(0x002D_AB94 + 4 * k)),
            min_distance: f(0x002D_AB48),
            max_distance: f(0x002D_AB4C),
            movement_speed: f(0x002D_AB50),
            ball_damage: i(0x002D_AB58),
            // Slow 0x2DAB5C, poison 0x2DAB64, curse 0x2DAB6C (damage, then chance).
            status_damage: [i(0x002D_AB64), i(0x002D_AB6C), i(0x002D_AB5C)],
            status_chance: [i(0x002D_AB68), i(0x002D_AB70), i(0x002D_AB60)],
            special_damage: [i(0x002D_AB80), i(0x002D_AB7C), i(0x002D_AB74)],
            leach_drain: leach_drain as i32,
            thorn_max_hits: i(0x002D_AB84),
        })
    }

    /// Alternate loop A's start (Vanity, class 16): movement speed 10, projectileSpeed1 0.5 and
    /// energyBallTime label 1 1.5 to 3 s, never restored.
    pub fn alternate_a(&mut self) {
        self.movement_speed = 10.0;
        self.projectile_speed[0] = 0.5;
        self.energy[0] = (1.5, 3.0);
    }
}

/// The alive count clamped to 1..=3, as an index into the by-alive arrays (alive - 1) and the
/// reversed label arrays (3 - alive).
fn by_alive(alive: i32) -> usize {
    (alive.clamp(1, 3) - 1) as usize
}

fn label(alive: i32) -> usize {
    (3 - alive.clamp(1, 3)) as usize
}

/// The main-loop projectile speed label (R-0065): 0.9 with 1 alive, 0.75 with 2, and the 3-alive
/// word otherwise (3 alive, and the boot value -1 and 0).
fn speed_label(alive: i32) -> usize {
    match alive {
        1 => 2,
        2 => 1,
        _ => 0,
    }
}

/// The path step's speed in units per tick: 12.5 with 1 alive, 10.0 with 2, else the word.
pub fn move_speed(w: &Words, alive: i32) -> f32 {
    (match alive {
        1 => 12.5,
        2 => 10.0,
        _ => w.movement_speed,
    }) / 60.0
}

/// The state shared by the three sisters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Group {
    /// The alive count (-1 until the first recount).
    pub alive: i32,
    pub shielded: bool,
    ended: bool,
}

impl Default for Group {
    fn default() -> Self {
        Self { alive: -1, shielded: true, ended: false }
    }
}

impl Group {
    /// The once-per-pass recount; returns true on the pass that should post level event 2 (all
    /// sisters dead), once.
    pub fn recount(&mut self, alive: i32) -> bool {
        self.alive = alive;
        if alive == 0 && !self.ended {
            self.ended = true;
            return true;
        }
        false
    }
}

/// The controller states (SPEC-0011 Controller state numbers 0 to 6, 8 and 9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Move,
    DodgeMissile(u32),
    DodgeMelee(u32),
    Ball { fired: bool },
    Status { fired: bool },
    Special { fired: bool },
    Flinch(u32),
    Dead,
}

/// What a sister sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub hero: Vec3,
    pub hp: i32,
    pub max_hp: i32,
    /// The hero's hit on her this tick: damage, and whether it was a shot (else the swing).
    pub hit: Option<(i32, bool)>,
    /// The hero projectiles (up to 6) passing the proximity test.
    pub missiles_near: usize,
    /// The playing clip: action and whether it has ended; the frame's event bits.
    pub clip: (u32, bool),
    /// The playing clip's progress (0 to 1).
    pub progress: f32,
    pub frame_bits: u32,
    /// The body contact or navmesh blocked flag.
    pub blocked: bool,
    pub level: &'a dyn Fn(u16) -> bool,
    /// The alternate loops' avoid point: the far centroid pick of her mesh, 10 from the hero
    /// (SPEC-0056; SPEC-0011 Movement speed), offered while the hero is within 5.0.
    pub far: Option<Vec3>,
}

/// What a sister asks for this tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// The speed scale P = 2 - f for this tick's clip rate.
    pub rate: f32,
    /// Turn toward the hero at this weight.
    pub face: Option<f32>,
    /// Move toward this point at this speed (units per tick).
    pub travel: Option<(Vec3, f32)>,
    pub launch: Option<Setup>,
    pub set_level: Vec<u16>,
    /// A one-shot effect at the sister, raised by this height.
    pub effect: Option<(i32, f32)>,
    pub hp_loss: i32,
    /// The death animation has ended: the sister is removed.
    pub remove: bool,
    /// Alternate loop A has started: the words are overwritten (`Words::alternate_a`).
    pub override_words: bool,
    /// A shielded melee hit (the hero's hit reaction 13).
    pub shield_melee: bool,
}

impl Default for Act {
    fn default() -> Self {
        Self { anim: None, rate: 1.0, face: None, travel: None, launch: None, set_level: Vec::new(), effect: None, hp_loss: 0, remove: false, override_words: false, shield_melee: false }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sister {
    pub class: u8,
    /// The kind selector: 0, 1 or 2 for classes 13, 15 and 16.
    pub kind: u8,
    pub state: State,
    pub energy: i32,
    pub status: i32,
    pub special: i32,
    pub dodge: i32,
    /// The orbit's target radius while moving radially, and the sideways direction.
    pub radius: Option<f32>,
    pub dir: f32,
    /// The damage-taken sum (read only by the unreachable dome start).
    pub taken: i32,
    pub alternate: bool,
    /// The controller start has run, and the group-attack interval it drew (the dome never
    /// starts, so nothing reads it).
    started: bool,
    group: i32,
    /// The alternate loops' avoiding mode (+0x30 = 2) and its point.
    pub avoid: Option<Vec3>,
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

/// A roll of 0 to 100 inclusive below the chance (SPEC-0019 Roll and coin rules).
fn roll(rng: &mut Rng, chance: i32) -> bool {
    rng.range(0, 100) < chance
}

/// The launch from the sister's point raised 2.5 toward the hero raised 2.0 (the alternate
/// loop's points, used for the main loop too).
fn aim(position: Vec3, hero: Vec3) -> (Vec3, Vec3) {
    let start = [position[0], position[1] + 2.5, position[2]];
    let to = [hero[0] - start[0], hero[1] + 2.0 - start[1], hero[2] - start[2]];
    let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-6);
    (start, to.map(|c| c / l))
}

/// One projectile of the shared pool: behaviour kind, visual kind, element, damage, status (id
/// and chance; the hero's default duration), lifetime, flight effect, speed and owner.
#[allow(clippy::too_many_arguments)]
fn shot(kind: i32, visual: i32, element: i32, damage: i32, status: (i32, i32), lifetime: i32, flight: i32, speed: f32, owner: u64, v: &View) -> Setup {
    let (start, d) = aim(v.position, v.hero);
    Setup {
        kind,
        visual,
        element,
        owner: Some(owner),
        start,
        velocity: d.map(|c| c * speed),
        lifetime,
        damage: damage as f32,
        radius: 1.0,
        spin: 0.0,
        area_radius: 0.0,
        area_duration: 0,
        key: 0,
        ignore_world: false,
        status: (status.0, status.1, -1),
        effects: [flight, -1, -1, -1, -1, -1],
        trail: -1,
    }
}

impl Sister {
    /// A sister of class 13, 15 or 16 at its start: idle, the cooldowns at 0.
    pub fn new(class: u8) -> Self {
        let kind = match class {
            15 => 1,
            16 => 2,
            _ => 0,
        };
        Self { class, kind, state: State::Idle, energy: 0, status: 0, special: 0, dodge: 0, radius: None, dir: 1.0, taken: 0, alternate: false, started: false, group: 0, avoid: None }
    }

    /// The controller start (R-0065): energy, status, group and special drawn in that order by
    /// the alive word, the dodge cooldown 0. Energy treats alive -1 like 3 alive; status, group
    /// and special are 0 for alive -1 and 0; alive 0 gives 0 for all four.
    fn start(&mut self, alive: i32, w: &Words, rng: &mut Rng) {
        self.started = true;
        self.dodge = 0;
        let zero = (0.0, 0.0);
        let by = |pairs: &[(f32, f32); 3]| if alive >= 1 { pairs[label(alive)] } else { zero };
        self.energy = draw(rng, if alive == 0 { zero } else { w.energy[label(if alive < 0 { 3 } else { alive })] });
        self.status = draw(rng, by(&w.status));
        self.group = draw(rng, by(&w.group));
        self.special = draw(rng, by(&w.special));
    }

    pub fn dead(&self) -> bool {
        self.state == State::Dead
    }

    /// One tick of the sister's class routine. `owner` is the sister's id for her projectiles.
    pub fn tick(&mut self, v: &View, g: &mut Group, w: &mut Words, owner: u64, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let f = if v.max_hp > 0 { v.hp as f32 / v.max_hp as f32 } else { 0.0 };
        a.rate = 2.0 - f;
        if (v.level)(0) {
            return self.alternate_tick(v, g, w, owner, rng, a);
        }
        let alive = g.alive;
        if !self.started {
            self.start(alive, w, rng);
        }
        // Taking damage (not in the dodges, the flinch or death).
        let immune = matches!(self.state, State::DodgeMissile(_) | State::DodgeMelee(_) | State::Flinch(_) | State::Dead);
        if let (Some((d, is_shot)), false) = (v.hit, immune) {
            if g.shielded {
                if is_shot {
                    g.shielded = false;
                    a.set_level.push(1);
                } else {
                    a.effect = Some((4008, 0.0));
                    a.shield_melee = true;
                }
            } else {
                self.taken += d;
                if d > v.hp {
                    self.state = State::Dead;
                } else {
                    a.effect = Some((222, 1.5));
                    a.hp_loss = d;
                    self.state = if v.hp - d < 1 { State::Dead } else { State::Flinch(rng.range(23, 24) as u32) };
                }
            }
        }
        // The cooldowns, unless level state 4 is set.
        if !(v.level)(4) {
            for t in [&mut self.energy, &mut self.status, &mut self.special] {
                *t -= 1;
            }
        }
        self.dodge = (self.dodge - 1).max(0);
        // The dodges.
        if !self.dead() && self.dodge == 0 {
            let near = ((v.hero[0] - v.position[0]).powi(2) + (v.hero[1] - v.position[1]).powi(2) + (v.hero[2] - v.position[2]).powi(2)).sqrt() < 7.0;
            let dodged = if near && roll(rng, w.dodge_melee[by_alive(alive)]) {
                Some(State::DodgeMelee(rng.range(15, 16) as u32))
            } else if (0..v.missiles_near.min(6)).any(|_| roll(rng, w.dodge_missile[by_alive(alive)])) {
                Some(State::DodgeMissile(rng.range(13, 14) as u32))
            } else {
                None
            };
            if let Some(s) = dodged {
                self.state = s;
                self.dodge = 60 * rng.range(1, 3);
            }
        }
        // The chooser (idle or move).
        if matches!(self.state, State::Idle | State::Move) {
            let next = if f < 0.33 && self.special <= 0 {
                State::Special { fired: false }
            } else if f < 0.66 && self.status < 1 {
                State::Status { fired: false }
            } else if self.energy < 1 {
                State::Ball { fired: false }
            } else {
                State::Move
            };
            if next == State::Move && self.state != State::Move {
                self.radius = None;
                self.dir = if rng.range(0, 1) == 0 { 1.0 } else { -1.0 };
            }
            self.state = next;
        }
        let ended = |anim: u32| v.clip.0 == anim && v.clip.1;
        // The release: the event bit 0x20 or progress above 0.5, whichever comes first (R-0065).
        let release = |anim: u32| v.clip.0 == anim && (v.frame_bits & 0x20 != 0 || v.progress > 0.5);
        a.face = Some(0.25);
        match self.state {
            State::Idle => a.anim = Some(1),
            State::Move => {
                a.anim = Some(1);
                a.travel = self.orbit(v, w, alive, rng);
            }
            State::DodgeMissile(n) | State::DodgeMelee(n) | State::Flinch(n) => {
                a.anim = Some(n);
                a.face = None;
                if ended(n) {
                    self.state = State::Idle;
                }
            }
            State::Ball { fired } => {
                a.anim = Some(17);
                if release(17) && !fired {
                    a.launch = Some(shot(10, 16, -1, w.ball_damage, (-1, 0), 1000, 460, w.projectile_speed[speed_label(alive)], owner, v));
                    self.state = State::Ball { fired: true };
                }
                if ended(17) {
                    self.state = State::Idle;
                    self.energy = draw(rng, w.energy[label(alive)]);
                }
            }
            State::Status { fired } => {
                a.anim = Some(18);
                if release(18) && !fired {
                    let k = self.kind as usize;
                    let (kind, visual, status) = [(12, 18, 2), (13, 19, 1), (11, 17, 3)][k];
                    a.launch = Some(shot(kind, visual, -1, w.status_damage[k], (status, w.status_chance[k]), 1000, -1, w.projectile_speed[speed_label(alive)], owner, v));
                    self.state = State::Status { fired: true };
                }
                if ended(18) {
                    self.state = State::Idle;
                    self.status = draw(rng, w.status[label(alive)]);
                }
            }
            State::Special { fired } => {
                a.anim = Some(19);
                if release(19) && !fired {
                    let k = self.kind as usize;
                    let (kind, visual) = [(16, 22), (15, 21), (14, 20)][k];
                    a.launch = Some(shot(kind, visual, -1, w.special_damage[k], (-1, 0), 1000, -1, w.projectile_speed[speed_label(alive)], owner, v));
                    self.state = State::Special { fired: true };
                }
                if ended(19) {
                    self.state = State::Idle;
                    self.special = draw(rng, w.special[label(alive)]);
                }
            }
            State::Dead => {
                a.anim = Some(26);
                a.face = None;
                a.remove = ended(26);
            }
        }
        a
    }

    /// The move state: the radial approach to a radius in the band, then the sideways circle
    /// with the direction flipped on every blocked tick. Returns the waypoint and speed.
    fn orbit(&mut self, v: &View, w: &Words, alive: i32, rng: &mut Rng) -> Option<(Vec3, f32)> {
        let d = hdist(v.position, v.hero);
        let speed = move_speed(w, alive);
        if self.radius.is_none() && (d < w.min_distance || d > w.max_distance) {
            self.radius = Some(rng.frange(w.min_distance, w.max_distance));
        }
        let u = if d > 1e-6 { [(v.position[0] - v.hero[0]) / d, 0.0, (v.position[2] - v.hero[2]) / d] } else { [1.0, 0.0, 0.0] };
        if let Some(r) = self.radius {
            if (d - r).abs() < 0.5 {
                self.radius = None;
            } else {
                return Some(([v.hero[0] + u[0] * r, v.position[1], v.hero[2] + u[2] * r], speed));
            }
        }
        if v.blocked {
            self.dir = -self.dir;
        }
        let t = [-u[2] * self.dir, 0.0, u[0] * self.dir];
        Some(([v.position[0] + 0.1 * t[0], v.position[1], v.position[2] + 0.1 * t[2]], speed))
    }

    /// The alternate loops (level flag 0): idle, move and the alternate energy ball. Loop A
    /// (class 16) writes 3 to the alive word and overrides the words at its start.
    fn alternate_tick(&mut self, v: &View, g: &mut Group, w: &mut Words, owner: u64, rng: &mut Rng, mut a: Act) -> Act {
        if !self.alternate {
            self.alternate = true;
            self.state = State::Idle;
            if self.class == 16 {
                g.alive = 3;
                w.alternate_a();
                a.override_words = true;
            }
        }
        if !(v.level)(4) {
            self.energy -= 1;
        }
        let alive = g.alive;
        // SPEC-0011 Movement speed: the avoiding mode starts when the hero is within 5.0 of her
        // horizontally and ends within 2.0 of its point or when blocked; meanwhile she runs to
        // the point at twice the movement speed and makes no attack decision.
        if self.avoid.is_none() && matches!(self.state, State::Idle | State::Move) && hdist(v.position, v.hero) < 5.0 {
            self.avoid = v.far;
        }
        if let Some(p) = self.avoid {
            if hdist(v.position, p) < 2.0 || v.blocked {
                self.avoid = None;
            } else {
                self.state = State::Move;
                a.anim = Some(1);
                a.travel = Some((p, 2.0 * move_speed(w, alive)));
                return a;
            }
        }
        if matches!(self.state, State::Idle | State::Move) {
            if self.energy < 1 {
                // The target roll (a target entity is not built, so the flag has no use here).
                let _ = rng.range(0, 100);
                self.state = State::Ball { fired: false };
            } else {
                if self.state != State::Move {
                    self.radius = None;
                    self.dir = if rng.range(0, 1) == 0 { 1.0 } else { -1.0 };
                }
                self.state = State::Move;
            }
        }
        match self.state {
            State::Move => {
                a.anim = Some(1);
                a.travel = self.orbit(v, w, alive, rng);
            }
            State::Ball { fired } => {
                a.anim = Some(17);
                a.face = Some(0.25);
                if v.clip.0 == 17 && v.frame_bits & 0x20 != 0 && !fired {
                    a.launch = Some(shot(10, 16, 6, w.ball_damage, (-1, 0), 240, 460, w.projectile_speed[0], owner, v));
                    self.state = State::Ball { fired: true };
                }
                if v.clip.0 == 17 && v.clip.1 {
                    self.state = State::Idle;
                    self.energy = 2 * draw(rng, w.energy[label(alive)]);
                }
            }
            _ => {
                a.anim = Some(1);
                self.state = State::Idle;
            }
        }
        a
    }
}

/// The Mote-eater drain words (SPEC-0034 3.13): 7 from 0x2DCAC8 and the 14-word per-Mote
/// effect table at 0x2DCAE8.
pub const DRAIN_WORDS: u32 = 0x002D_CAC8;
pub const DRAIN_TABLE: u32 = 0x002D_CAE8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrainWords {
    pub seconds: f32,
    pub body_height: f32,
    pub slot_height: f32,
    pub spread: f32,
    pub scale: f32,
    pub charge_switch: i32,
    pub charge: i32,
    pub effects: [i32; 14],
}

impl DrainWords {
    pub fn read(w: &[u32], table: &[u32]) -> Option<Self> {
        if w.len() < 7 || table.len() < 14 {
            return None;
        }
        let f = |k: usize| f32::from_bits(w[k]);
        Some(Self {
            seconds: f(0),
            body_height: f(1),
            slot_height: f(2),
            spread: f(3),
            scale: f(4),
            charge_switch: w[5] as i32,
            charge: w[6] as i32,
            effects: std::array::from_fn(|k| table[k] as i32),
        })
    }
}

/// The Mote-eater drain (SPEC-0034 3.13): the slots' Mote ids and effects, the countdown, the
/// period and the revealed count.
#[derive(Clone, Debug, PartialEq)]
pub struct Drain {
    pub slots: Vec<(u16, i32)>,
    pub countdown: i32,
    pub period: i32,
    pub revealed: usize,
}

/// What one drain call does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrainStep {
    /// Nothing new (the effects follow the hero).
    Follow,
    /// Slot k is revealed at the offset: its Mote, its effect, and the charge it uses (0: none).
    Reveal { slot: usize, mote: u16, effect: i32, offset: Vec3, charge: i32 },
    /// The countdown reached 0: everything stops.
    Stop,
}

impl Drain {
    /// The start: 8 picks from the twelve Motes 160 to 171 (a repeat steps to (pick mod 12) + 1;
    /// a walk past 11 gives 12, which adds no slot), each slot's effect from the table, the
    /// countdown trunc(seconds x 60) + 1 and the period countdown / slots (at least 1).
    pub fn start(w: &DrainWords, rng: &mut Rng) -> Self {
        let mut picks: Vec<i32> = Vec::new();
        for _ in 0..8 {
            let mut p = rng.range(0, 11);
            while picks.contains(&p) {
                p = (p % 12) + 1;
            }
            picks.push(p);
        }
        let slots: Vec<(u16, i32)> = picks.iter().filter(|&&p| p < 12).map(|&p| (160 + p as u16, w.effects[p as usize])).collect();
        let countdown = (w.seconds * 60.0) as i32 + 1;
        let period = (countdown / slots.len().max(1) as i32).max(1);
        Self { slots, countdown, period, revealed: 0 }
    }

    /// One call (each sister loop still stepped makes one per tick).
    pub fn tick(&mut self, w: &DrainWords, rng: &mut Rng) -> DrainStep {
        self.countdown -= 1;
        if self.countdown <= 0 {
            return DrainStep::Stop;
        }
        if self.countdown % self.period == 0 && self.revealed < 8 && self.revealed < self.slots.len() {
            let k = self.revealed;
            self.revealed += 1;
            let a = w.spread;
            let offset = [rng.frange(-a, a), w.slot_height, rng.frange(-a, a)];
            let (mote, effect) = self.slots[k];
            return DrainStep::Reveal { slot: k, mote, effect, offset, charge: if w.charge_switch != 0 { w.charge } else { 0 } };
        }
        DrainStep::Follow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disc values of SPEC-0011 Layout as a synthetic block.
    fn block() -> Vec<u32> {
        let mut b = vec![0u32; WORDS];
        let mut put = |at: u32, v: u32| b[((at - BASE) / 4) as usize] = v;
        let fl = |x: f32| x.to_bits();
        for (k, (m, e)) in [(50, 40), (60, 50), (75, 60)].into_iter().enumerate() {
            put(0x002D_AAC8 + 4 * k as u32, m);
            put(0x002D_AAD4 + 4 * k as u32, e);
        }
        for k in 0..3u32 {
            put(0x002D_AAE0 + 8 * k, fl(2.5));
            put(0x002D_AAE4 + 8 * k, fl(5.0));
            put(0x002D_AAF8 + 8 * k, fl(1.75));
            put(0x002D_AAFC + 8 * k, fl(4.0));
            put(0x002D_AB10 + 8 * k, fl(1.25));
            put(0x002D_AB14 + 8 * k, fl(3.0));
        }
        for (k, s) in [0.5f32, 0.75, 0.9].into_iter().enumerate() {
            put(0x002D_AB94 + 4 * k as u32, fl(s));
        }
        put(0x002D_AB48, fl(15.0));
        put(0x002D_AB4C, fl(30.0));
        put(0x002D_AB50, fl(7.5));
        put(0x002D_AB58, 25);
        for at in [0x002D_AB5C, 0x002D_AB64, 0x002D_AB6C] {
            put(at, 10);
            put(at + 4, 100);
        }
        put(0x002D_AB74, 20);
        put(0x002D_AB7C, 50);
        put(0x002D_AB80, 10);
        b
    }

    fn view(level: &dyn Fn(u16) -> bool, hp: i32) -> View<'_> {
        View { position: [20.0, 0.0, 0.0], hero: [0.0; 3], hp, max_hp: 100, hit: None, missiles_near: 0, clip: (1, false), progress: 0.0, frame_bits: 0, blocked: false, level, far: None }
    }

    #[test]
    fn words_and_alive_tables() {
        let w = Words::read(&block(), 50).unwrap();
        // 3 alive: missile dodge 75, energy 150 to 300 ticks, speed 0.5, movement 7.5.
        assert_eq!(w.dodge_missile[by_alive(3)], 75);
        assert_eq!((ticks(w.energy[label(3)].0), ticks(w.energy[label(3)].1)), (150, 300));
        assert_eq!(w.projectile_speed[label(3)], 0.5);
        assert!((move_speed(&w, 3) - 0.125).abs() < 1e-6);
        // 1 alive: missile dodge 50, speed 0.9, 12.5 / 60; 2 alive: 0.75, 10 / 60.
        assert_eq!(w.dodge_missile[by_alive(1)], 50);
        assert_eq!(w.projectile_speed[label(1)], 0.9);
        assert!((move_speed(&w, 1) - 12.5 / 60.0).abs() < 1e-6);
        assert_eq!(w.projectile_speed[label(2)], 0.75);
        assert!((move_speed(&w, 2) - 10.0 / 60.0).abs() < 1e-6);
        // Alternate loop A: 3 alive moves at 10.
        let mut a = w;
        a.alternate_a();
        assert!((move_speed(&a, 3) - 10.0 / 60.0).abs() < 1e-6);
        assert_eq!((a.energy[0], a.projectile_speed[0]), ((1.5, 3.0), 0.5));
        // The kind selector by class.
        assert_eq!([13, 15, 16].map(|c| Sister::new(c).kind), [0, 1, 2]);
    }

    #[test]
    fn start_draws_by_the_alive_word() {
        let w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(4);
        // The boot value -1: energy 150 to 300 ticks (like 3 alive), the others 0.
        let mut s = Sister::new(13);
        s.start(-1, &w, &mut rng);
        assert!((150..=300).contains(&s.energy), "{}", s.energy);
        assert_eq!((s.status, s.group, s.special, s.dodge), (0, 0, 0, 0));
        // Alive 0: all four 0.
        let mut s = Sister::new(13);
        s.start(0, &w, &mut rng);
        assert_eq!((s.energy, s.status, s.group, s.special), (0, 0, 0, 0));
        // Three alive: status 105 to 240, special 75 to 180.
        let mut s = Sister::new(13);
        s.start(3, &w, &mut rng);
        assert!((105..=240).contains(&s.status) && (75..=180).contains(&s.special));
        // The main-loop speed: 0.5 for -1, 0 and 3 alive.
        assert_eq!([-1, 0, 3, 2, 1].map(|n| w.projectile_speed[speed_label(n)]), [0.5, 0.5, 0.5, 0.75, 0.9]);
    }

    #[test]
    fn chooser_bands() {
        let mut w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        let mut g = Group::default();
        g.recount(3);
        let pick = |s: &mut Sister, hp: i32, w: &mut Words, g: &mut Group, rng: &mut Rng| {
            s.tick(&view(&no, hp), g, w, 1, rng);
            s.state
        };
        // f 0.5 with the status cooldown at 0 (it reaches -1 this tick): status missile.
        let mut s = Sister { started: true, ..Sister::new(13) };
        (s.energy, s.special) = (100, 100);
        assert_eq!(pick(&mut s, 50, &mut w, &mut g, &mut rng), State::Status { fired: false });
        // f 0.8 with the energy cooldown at 30: move.
        let mut s = Sister { started: true, ..Sister::new(13) };
        s.energy = 30;
        assert_eq!(pick(&mut s, 80, &mut w, &mut g, &mut rng), State::Move);
        // f 0.30: special at 0; special 5 and status 0: status; all above 0: move.
        let mut s = Sister { started: true, ..Sister::new(13) };
        s.energy = 100;
        assert_eq!(pick(&mut s, 30, &mut w, &mut g, &mut rng), State::Special { fired: false });
        let mut s = Sister { started: true, ..Sister::new(13) };
        (s.special, s.energy) = (6, 100);
        assert_eq!(pick(&mut s, 30, &mut w, &mut g, &mut rng), State::Status { fired: false });
        let mut s = Sister { started: true, ..Sister::new(13) };
        (s.special, s.status, s.energy) = (6, 6, 6);
        assert_eq!(pick(&mut s, 30, &mut w, &mut g, &mut rng), State::Move);
        // Level state 4 holds the cooldowns.
        let four = |f: u16| f == 4;
        let mut s = Sister { started: true, ..Sister::new(13) };
        s.energy = 1;
        s.tick(&view(&four, 100), &mut g, &mut w, 1, &mut rng);
        assert_eq!((s.energy, s.state), (1, State::Move));
    }

    #[test]
    fn shield_and_damage() {
        let mut w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(2);
        let no = |_: u16| false;
        let mut g = Group::default();
        g.recount(3);
        let hit = |s: &mut Sister, g: &mut Group, w: &mut Words, rng: &mut Rng, hp: i32, d: i32, is_shot: bool| {
            let mut v = view(&no, hp);
            v.hit = Some((d, is_shot));
            s.energy = 100;
            s.tick(&v, g, w, 1, rng)
        };
        // Shielded melee 20: no damage, effect 4008, reaction 13.
        let mut s = Sister::new(15);
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 20, false);
        assert_eq!((a.hp_loss, a.effect, a.shield_melee, g.shielded), (0, Some((4008, 0.0)), true, true));
        // Shielded shot 10: no damage, level event 1, the shield off for all.
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 10, true);
        assert_eq!((a.hp_loss, a.set_level.clone(), g.shielded), (0, vec![1], false));
        // Unshielded: 10 on 30: flinch, 20 left, sum +10, effect 222 at 1.5.
        let mut s = Sister::new(15);
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 10, true);
        assert!(matches!(s.state, State::Flinch(23 | 24)));
        assert_eq!((a.hp_loss, s.taken, a.effect), (10, 10, Some((222, 1.5))));
        // 31 on 30: dead at once, no flinch or effect; 30 on 30: health 0, then dead.
        let mut s = Sister::new(15);
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 31, true);
        assert_eq!((s.state, a.hp_loss, a.effect), (State::Dead, 0, None));
        let mut s = Sister::new(15);
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 30, true);
        assert_eq!((s.state, a.hp_loss), (State::Dead, 30));
        // A flinching sister takes no hit.
        let mut s = Sister::new(15);
        s.state = State::Flinch(23);
        let a = hit(&mut s, &mut g, &mut w, &mut rng, 30, 10, true);
        assert_eq!(a.hp_loss, 0);
        // The group posts event 2 once.
        assert!(!g.recount(1));
        assert!(g.recount(0));
        assert!(!g.recount(0));
    }

    #[test]
    fn orbit_and_launches() {
        let mut w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        let mut g = Group::default();
        g.recount(2);
        // At 20 within the band: no new radius; the waypoint is 0.1 along the side at 10 / 60.
        let mut s = Sister::new(16);
        s.energy = 100;
        let a = s.tick(&view(&no, 100), &mut g, &mut w, 1, &mut rng);
        assert_eq!(s.radius, None);
        let (p, speed) = a.travel.unwrap();
        assert!((speed - 10.0 / 60.0).abs() < 1e-6);
        assert!((p[0] - 20.0).abs() < 1e-6 && (p[2].abs() - 0.1).abs() < 1e-6);
        // Blocked: the side flips.
        let dir = s.dir;
        let mut v = view(&no, 100);
        v.blocked = true;
        s.tick(&v, &mut g, &mut w, 1, &mut rng);
        assert_eq!(s.dir, -dir);
        // At 14: outside the band, a radius in [15, 30].
        let mut s = Sister::new(16);
        s.energy = 100;
        let mut v = view(&no, 100);
        v.position = [14.0, 0.0, 0.0];
        s.tick(&v, &mut g, &mut w, 1, &mut rng);
        assert!(s.radius.is_some_and(|r| (15.0..=30.0).contains(&r)));
        // The energy ball releases once on event 0x20, at 0.75 with 2 alive, 25 damage.
        let mut s = Sister::new(13);
        s.state = State::Ball { fired: false };
        let mut v = view(&no, 100);
        v.clip = (17, false);
        v.frame_bits = 0x21;
        let a = s.tick(&v, &mut g, &mut w, 7, &mut rng);
        let shot = a.launch.unwrap();
        assert_eq!((shot.kind, shot.visual, shot.damage, shot.lifetime, shot.owner, shot.effects[0]), (10, 16, 25.0, 1000, Some(7), 460));
        let sp = (shot.velocity[0].powi(2) + shot.velocity[1].powi(2) + shot.velocity[2].powi(2)).sqrt();
        assert!((sp - 0.75).abs() < 1e-5);
        assert!(s.tick(&v, &mut g, &mut w, 7, &mut rng).launch.is_none());
        // Its end sets the energy cooldown from the 2-alive label.
        v.clip = (17, true);
        v.frame_bits = 0;
        s.tick(&v, &mut g, &mut w, 7, &mut rng);
        assert_eq!(s.state, State::Idle);
        assert!((150..=300).contains(&s.energy));
        // Vanity's status missile is slow (kind 11, visual 17, status 3 at 100).
        let mut s = Sister::new(16);
        s.state = State::Status { fired: false };
        v.clip = (18, false);
        v.frame_bits = 0x20;
        let m = s.tick(&v, &mut g, &mut w, 7, &mut rng).launch.unwrap();
        assert_eq!((m.kind, m.visual, m.damage, m.status.0, m.status.1), (11, 17, 10.0, 3, 100));
    }

    #[test]
    fn mote_eater_drain() {
        let w = DrainWords::read(&[30.0f32.to_bits(), 4.5f32.to_bits(), 4.0f32.to_bits(), 0.0f32.to_bits(), 0.5f32.to_bits(), 1, 20], &[6000, 6000, 6001, 6001, 6002, 6002, 6003, 6003, 6004, 6004, 6005, 6005, u32::MAX, u32::MAX]).unwrap();
        let mut rng = Rng::seeded(9);
        // 8 slots: reveals on calls 1, 226, ..., 1576; the stop on call 1801.
        let mut d = Drain { slots: (0..8).map(|k| (160 + k, 6000)).collect(), countdown: 1801, period: 225, revealed: 0 };
        let mut reveals = Vec::new();
        let mut stop = 0;
        for call in 1..=1801 {
            match d.tick(&w, &mut rng) {
                DrainStep::Reveal { charge, offset, .. } => {
                    assert_eq!((charge, offset), (20, [0.0, 4.0, 0.0]));
                    reveals.push(call);
                }
                DrainStep::Stop => {
                    stop = call;
                    break;
                }
                DrainStep::Follow => {}
            }
        }
        assert_eq!(reveals, vec![1, 226, 451, 676, 901, 1126, 1351, 1576]);
        assert_eq!(stop, 1801);
        // 7 slots (period 257): the first reveal on call 2, the last on 1544.
        let mut d = Drain { slots: (0..7).map(|k| (160 + k, 6000)).collect(), countdown: 1801, period: 257, revealed: 0 };
        let calls: Vec<i32> = (1..=1801).filter(|_| matches!(d.tick(&w, &mut rng), DrainStep::Reveal { .. })).collect();
        assert_eq!((calls[0], *calls.last().unwrap()), (2, 1544));
        // A start: 7 or 8 distinct Motes, the period from the slot count.
        for seed in 0..20 {
            let d = Drain::start(&w, &mut Rng::seeded(seed));
            let mut ids: Vec<u16> = d.slots.iter().map(|s| s.0).collect();
            ids.dedup();
            assert!((7..=8).contains(&d.slots.len()) && d.slots.iter().all(|s| (160..=171).contains(&s.0)));
            assert_eq!((d.countdown, d.period), (1801, 1801 / d.slots.len() as i32));
        }
    }

    #[test]
    fn alternate_avoid() {
        // SPEC-0011 Movement speed: in an alternate loop a hero within 5.0 starts the avoiding
        // mode: she runs to the far point at twice the movement speed and makes no decision,
        // until within 2.0 of it.
        let mut w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(4);
        let zero = |f: u16| f == 0;
        let mut g = Group::default();
        g.recount(1);
        let mut s = Sister::new(15);
        s.energy = 100;
        let mut v = view(&zero, 100);
        v.hero = [17.0, 0.0, 0.0];
        v.far = Some([40.0, 0.0, 0.0]);
        let a = s.tick(&v, &mut g, &mut w, 1, &mut rng);
        let speed = move_speed(&w, g.alive);
        assert_eq!(a.travel, Some(([40.0, 0.0, 0.0], 2.0 * speed)));
        assert_eq!(s.avoid, Some([40.0, 0.0, 0.0]));
        // Within 2.0 of the point the mode ends.
        v.position = [39.0, 0.0, 0.0];
        v.hero = [30.0, 0.0, 0.0];
        s.tick(&v, &mut g, &mut w, 1, &mut rng);
        assert_eq!(s.avoid, None);
    }

    #[test]
    fn alternate_loop() {
        let mut w = Words::read(&block(), 50).unwrap();
        let mut rng = Rng::seeded(4);
        let zero = |f: u16| f == 0;
        let mut g = Group::default();
        g.recount(1);
        // Loop A: alive 3 and the overrides at its start; an energy ball at once.
        let mut s = Sister::new(16);
        let mut v = view(&zero, 100);
        let a = s.tick(&v, &mut g, &mut w, 1, &mut rng);
        assert!(a.override_words);
        assert_eq!((g.alive, w.movement_speed, s.state), (3, 10.0, State::Ball { fired: false }));
        // One release per animation: element 6, life 240, speed 0.5.
        v.clip = (17, false);
        v.frame_bits = 0x20;
        let b = s.tick(&v, &mut g, &mut w, 1, &mut rng).launch.unwrap();
        assert_eq!((b.element, b.lifetime, b.damage), (6, 240, 25.0));
        assert!(s.tick(&v, &mut g, &mut w, 1, &mut rng).launch.is_none());
        // The end: twice a draw of the label-1 cooldown (now 1.5 to 3 s: 90 to 180 ticks).
        v.clip = (17, true);
        s.tick(&v, &mut g, &mut w, 1, &mut rng);
        assert!(s.energy % 2 == 0 && (180..=360).contains(&s.energy));
        // Loop B (class 15) leaves the alive word.
        let mut g = Group::default();
        g.recount(2);
        let mut s = Sister::new(15);
        assert!(!s.tick(&view(&zero, 100), &mut g, &mut w, 1, &mut rng).override_words);
        assert_eq!(g.alive, 2);
    }
}
