//! The Werelupe King boss (SPEC-0019; behaviour class 32): the pre-check (the hit flinch, the
//! catch, triumph, the phases and death), the reaction choices on the hero's attack edge (block
//! with its overhead counter, the jump ramp, the roll and its counter-attack), the timed attacks
//! (boomerang, flame wave, swing, lunge, combo), the movement (walk, hop back, orbit), the shared
//! once-per-attack hit, the jump flight, the boomerang flight and the five flame rings. Its
//! phase events 4 and 6 and death flag 273 drive the arena script (SPEC-0039 pack 1246).
//!
//! The host measures the reach test, the hand point and the ground, and applies the requests.
//! Every tuning value is read from the boot data at the addresses of SPEC-0019.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning words (SPEC-0019 Layout).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub block_chance: [i32; 3],
    pub jump_distance: f32,
    pub jump_chance: (i32, i32),
    pub roll_chance: i32,
    pub roll_counter_distance: f32,
    pub roll_counter_chance: i32,
    pub overhead_distance: f32,
    pub overhead_chance: i32,
    pub boomerang_range: (f32, f32),
    pub boomerang_cooldown: i32,
    pub boomerang_chance: [i32; 3],
    pub boomerang_speed: f32,
    pub boomerang_damage: i32,
    pub flame_range: (f32, f32),
    pub flame_cooldown: i32,
    pub flame_chance: [i32; 3],
    pub flame_wait: (i32, i32),
    pub flame_damage: i32,
    pub swing_distance: f32,
    pub swing_cooldown: i32,
    pub swing_chance: [i32; 3],
    pub swing_damage: i32,
    pub lunge_distance: f32,
    pub lunge_cooldown: i32,
    pub lunge_chance: [i32; 3],
    pub lunge_damage: i32,
    pub combo_distance: f32,
    pub combo_cooldown: i32,
    pub combo_chance: [i32; 3],
    pub combo_damage: [i32; 3],
    pub walk_distance: f32,
    pub orbit_distance: f32,
    pub hop_distance: f32,
    pub hop_chance: i32,
    pub overhead_damage: i32,
    pub phase2: i32,
    pub phase3: i32,
    pub tick_rate: f32,
    pub jump_step: f32,
    pub boomerang_timeout: f32,
    pub far_weight: f32,
}

/// The boot addresses of SPEC-0019 Layout.
pub const BLOCK_CHANCE: u32 = 0x002D_B0E0;
pub const JUMP_DISTANCE: u32 = 0x002D_B0EC;
pub const JUMP_CHANCE: u32 = 0x002D_B0F0;
pub const ROLL_CHANCE: u32 = 0x002D_B0F8;
pub const ROLL_COUNTER: u32 = 0x002D_B0FC;
pub const OVERHEAD: u32 = 0x002D_B104;
pub const BOOMERANG_RANGE: u32 = 0x002D_B10C;
pub const BOOMERANG_COOLDOWN: u32 = 0x002D_B114;
pub const BOOMERANG_CHANCE: u32 = 0x002D_B118;
pub const FLAME_RANGE: u32 = 0x002D_B124;
pub const FLAME_COOLDOWN: u32 = 0x002D_B12C;
pub const FLAME_CHANCE: u32 = 0x002D_B130;
pub const FLAME_WAIT: u32 = 0x002D_B13C;
pub const SWING: u32 = 0x002D_B144;
pub const LUNGE: u32 = 0x002D_B158;
pub const COMBO: u32 = 0x002D_B16C;
pub const MOVEMENT: u32 = 0x002D_B180;
pub const DAMAGES: u32 = 0x002D_B190;
pub const PHASES: u32 = 0x002D_B1B0;
pub const BOOMERANG_SPEED: u32 = 0x002D_B1B8;
pub const SHARED: u32 = 0x002D_B1C0;

/// The flame ring's life in ticks (0.8 s; the effect's ±0.02 s jitter is not drawn here).
pub const RING_LIFE: i32 = 48;

/// A flame ring's radius and height after `ticks` effect ticks (effect 415's size rule).
pub fn ring_size(ticks: i32) -> (f32, f32) {
    let l = RING_LIFE as f32;
    let r = 0.5 + 24.5 * ticks as f32 / l;
    let mut h = 6.2963;
    for k in 1..=ticks {
        let f = k as f32 / l;
        h += if f <= 0.052_08 { -64.0 } else { -2.520_15 } / l;
    }
    (r, h)
}

/// The boss states (SPEC-0019 States, numbered 0 to 20).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Flinch(u32),
    Catch,
    BigLaugh,
    Triumph,
    Phase2,
    Phase3,
    /// The block: holding 0x19, or in the recovery 0x1D.
    Block { recovery: bool },
    /// The jump: launch (6), air (7) and landing (8).
    Jump { part: u8, velocity: Vec3 },
    Roll(u32),
    /// The swing and the roll counter-attack (0x17 or 0x16).
    Swing(u32),
    Overhead,
    Throw,
    Flame { wave: u8, wait: i32, fired: bool, playing: bool },
    Lunge,
    /// The combo's animation (0x16, 0x17, 0x18) and the closing animation 1.
    Combo(u32),
    Walk,
    HopBack,
    Orbit { right: bool },
    Death,
}

/// The boomerang (SPEC-0019 Boomerang flight).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boomerang {
    pub position: Vec3,
    pub direction: Vec3,
    /// 0 waypoint A, 1 the hero, 2 waypoint B, 3 the hand.
    pub phase: u8,
    pub a: Vec3,
    pub b: Vec3,
    pub timer: i32,
    pub hit: bool,
}

/// A flame object: its ring centre, age in ticks and hit flag.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ring {
    pub active: bool,
    pub centre: Vec3,
    pub age: i32,
    pub hit: bool,
}

/// What the boss sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero is guarding (the guard branch of the shared hit).
    pub hero_guarding: bool,
    /// The hero is down (knocked out).
    pub hero_down: bool,
    /// The hero's reaction trigger (in an attack swing) and its attack action state (13).
    pub hero_attacking: bool,
    pub hero_attack_state: bool,
    /// The boss was hit this tick (the reaction word).
    pub was_hit: bool,
    /// The reach test of the shared hit (the host's measurement).
    pub reach: bool,
    pub frame_bits: u32,
    pub clip: (u32, f32, bool),
    pub hp: i32,
    pub max_hp: i32,
    /// The hand point (the boomerang's release and catch point).
    pub hand: Vec3,
    pub ground: &'a dyn Fn(f32, f32) -> f32,
}

/// How the host moves the boss (as for Gnarfas: root motion steered, a direct move, or none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Steer { mode: u8, target: Vec3, weight: f32 },
    Fly(Vec3),
    Still,
}

/// What it asks for this tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    pub face: Option<f32>,
    pub travel: Move,
    /// A hit on the hero: damage and the blocked flag.
    pub hero_damage: Option<(i32, bool)>,
    /// Raw unblockable damage (the boomerang and the flame rings).
    pub raw_damage: Vec<i32>,
    pub set_level: Vec<u16>,
    pub set_flag: Vec<u16>,
    /// The rings started this tick (their centres), for the ring effect 415.
    pub rings: Vec<Vec3>,
    /// The no-gravity flag of the jump.
    pub no_gravity: bool,
}

impl Default for Act {
    fn default() -> Self {
        Self { anim: None, face: None, travel: Move::Steer { mode: 255, target: [0.0; 3], weight: 0.0 }, hero_damage: None, raw_damage: Vec::new(), set_level: Vec::new(), set_flag: Vec::new(), rings: Vec::new(), no_gravity: false }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct King {
    pub state: State,
    /// The phase field: 0, 1, 2 for phases 1 to 3.
    pub phase: usize,
    pub ready: bool,
    pub home: Vec3,
    pub boomerang: Option<Boomerang>,
    pub rings: [Ring; 5],
    counters: [i32; 5],
    once: bool,
    swing_toggle: bool,
    hop_timer: i32,
    trigger_was: bool,
    started: bool,
    /// The one-shot blocked word (+0x8BC): set by the hero's swing blocked by the guarding King,
    /// read and cleared by the block state.
    pub blocked: bool,
    /// The first damaging hero hit has been doubled.
    doubled: bool,
}

/// The attack counters: boomerang, flame wave, swing, lunge, combo.
const BOOMERANG: usize = 0;
const FLAME: usize = 1;
const SWING_C: usize = 2;
const LUNGE_C: usize = 3;
const COMBO_C: usize = 4;

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn len(a: Vec3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn norm(a: Vec3) -> Vec3 {
    let l = len(a).max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}

/// A roll 0 to 100 inclusive (SPEC-0019 Roll and coin rules).
fn roll(rng: &mut Rng) -> i32 {
    rng.range(0, 100)
}

impl King {
    /// The controller start: idle, ready, phase 1, home at the start position, the rings empty.
    pub fn new(home: Vec3) -> Self {
        Self { state: State::Idle, phase: 0, ready: true, home, boomerang: None, rings: [Ring::default(); 5], counters: [0; 5], once: false, swing_toggle: false, hop_timer: 0, trigger_was: false, started: true, blocked: false, doubled: false }
    }

    /// Hurt side (SPEC-0019 G11, G14): the King cannot be hit during the flinch, the catch, the
    /// roll and the jump (nor while held, which the host tests).
    pub fn hittable(&self) -> bool {
        !matches!(self.state, State::Flinch(_) | State::Catch | State::Roll(_) | State::Jump { .. })
    }

    /// Guarding (base state 20): the block, holding or in its recovery.
    pub fn guarding(&self) -> bool {
        matches!(self.state, State::Block { .. })
    }

    /// The damage a hero hit takes: the first damaging hit of the fight is doubled.
    pub fn hit_damage(&mut self, damage: i32) -> i32 {
        if damage > 0 && !self.doubled {
            self.doubled = true;
            return 2 * damage;
        }
        damage.max(0)
    }

    /// The attack state gate: idle, hit flinch, walk, hop back and the orbits.
    fn free(&self) -> bool {
        matches!(self.state, State::Idle | State::Flinch(_) | State::Walk | State::HopBack | State::Orbit { .. })
    }

    fn orbiting(&self) -> bool {
        matches!(self.state, State::Orbit { .. })
    }

    /// The side vector s = normalize(u x up) = (-u.z, 0, u.x) for the hero direction u.
    pub fn side(position: Vec3, hero: Vec3) -> Vec3 {
        let u = norm([hero[0] - position[0], 0.0, hero[2] - position[2]]);
        [-u[2], 0.0, u[0]]
    }

    /// The jump's start chance at home distance d (the 15 to 28 ramp, uncapped).
    pub fn jump_chance(w: &Words, d: f32) -> f32 {
        w.jump_chance.0 as f32 + (d - 15.0) / 13.0 * (w.jump_chance.1 - w.jump_chance.0) as f32
    }

    /// One tick.
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let _ = self.started;
        let d = hdist(v.position, v.hero);
        let p = self.phase;
        // Pre-check: the flinch, the catch, triumph, then health.
        if v.was_hit {
            self.state = State::Flinch([0x0B, 0x0C, 0x0D][rng.range(0, 2) as usize]);
        }
        if self.boomerang.is_some_and(|b| b.phase == 4) {
            self.boomerang = None;
            self.state = State::Catch;
        }
        if v.hero_down && self.free() && self.state != State::Triumph {
            self.state = State::Triumph;
        }
        let f = if v.max_hp > 0 { v.hp as f32 / v.max_hp as f32 } else { 0.0 };
        if f <= 0.0 {
            self.state = State::Death;
        } else if f < w.phase3 as f32 / 100.0 && self.phase != 2 && self.state != State::Phase3 {
            self.state = State::Phase3;
        } else if f < w.phase2 as f32 / 100.0 && self.phase != 1 && self.state != State::Phase2 && f >= w.phase3 as f32 / 100.0 {
            self.state = State::Phase2;
        }
        // The cooldown counters run in any state (the flame counter stops at 601).
        for (k, c) in self.counters.iter_mut().enumerate() {
            if k != FLAME || *c <= 600 {
                *c += 1;
            }
        }
        // The reaction choices on the hero's rising edge: block, jump, roll.
        let edge = v.hero_attacking && !self.trigger_was;
        self.trigger_was = v.hero_attacking;
        if edge {
            if self.free() && self.ready && roll(rng) < w.block_chance[p] {
                self.state = State::Block { recovery: false };
            }
            let home_d = hdist(self.home, v.position);
            if self.free() && self.ready && d < w.jump_distance && home_d > 15.0 && (roll(rng) as f32) < Self::jump_chance(w, home_d) {
                self.state = State::Jump { part: 0, velocity: [0.0; 3] };
            }
            if self.free() && roll(rng) < w.roll_chance {
                self.state = State::Roll(if rng.range(0, 1) == 0 { 9 } else { 10 });
            }
        }
        // The attacks: boomerang, flame wave, swing, lunge, combo.
        let open = |s: &Self| s.free() && s.ready;
        if open(self) && self.counters[BOOMERANG] > w.boomerang_cooldown * 60 && d > w.boomerang_range.0 && d < w.boomerang_range.1 && roll(rng) < w.boomerang_chance[p] {
            self.state = State::Throw;
        }
        if open(self) && self.counters[FLAME] > w.flame_cooldown * 60 && d > w.flame_range.0 && d < w.flame_range.1 && roll(rng) < w.flame_chance[p] {
            self.state = State::Flame { wave: 0, wait: 0, fired: false, playing: true };
            for r in &mut self.rings {
                r.hit = false;
            }
        }
        if open(self) && self.counters[SWING_C] > w.swing_cooldown * 60 && d < w.swing_distance && roll(rng) < w.swing_chance[p] {
            self.counters[SWING_C] = 0;
            self.once = false;
            self.swing_toggle = !self.swing_toggle;
            self.state = State::Swing(if self.swing_toggle { 0x17 } else { 0x16 });
        }
        if open(self) && self.counters[LUNGE_C] > w.lunge_cooldown * 60 && d < w.lunge_distance && roll(rng) < w.lunge_chance[p] {
            self.counters[LUNGE_C] = 0;
            self.once = false;
            self.state = State::Lunge;
        }
        if open(self) && self.counters[COMBO_C] > w.combo_cooldown * 60 && d < w.combo_distance && roll(rng) < w.combo_chance[p] {
            self.counters[COMBO_C] = 0;
            self.once = false;
            self.state = State::Combo(0x16);
        }
        // The movement (free states, not flinching).
        if self.free() && !matches!(self.state, State::Flinch(_)) {
            if d > w.walk_distance {
                self.state = State::Walk;
            } else {
                self.hop_timer += 1;
                if d < w.hop_distance && !self.ready && self.hop_timer > 60 {
                    self.hop_timer = 0;
                    if roll(rng) < w.hop_chance {
                        self.state = State::HopBack;
                    }
                } else if d < w.orbit_distance && !self.orbiting() && self.state != State::HopBack {
                    self.state = State::Orbit { right: rng.range(0, 1) == 0 };
                }
            }
        }
        // The state's behaviour.
        let ended = |id: u32| v.clip.0 == id && v.clip.2;
        let window = |id: u32| v.clip.0 == id && v.clip.1 < 0.99;
        let shared = |s: &mut Self, a: &mut Act, damage: i32, id: u32| {
            if window(id) && !s.once && v.frame_bits & 0x1F != 0 && v.reach {
                s.once = true;
                let facing = v.hero_facing[0] * v.facing[0] + v.hero_facing[2] * v.facing[2];
                a.hero_damage = Some(if v.hero_guarding && facing <= -0.5 { (0, true) } else { (damage, false) });
            }
        };
        match self.state {
            State::Idle => {
                a.anim = Some(0);
                a.face = Some(0.05);
            }
            State::Flinch(id) => {
                a.anim = Some(id);
                if ended(id) {
                    self.state = State::Idle;
                }
            }
            State::Catch => {
                a.anim = Some(0x14);
                if ended(0x14) {
                    self.ready = true;
                    self.counters[BOOMERANG] = 0;
                    self.state = State::Idle;
                }
            }
            State::BigLaugh => self.state = State::Idle,
            State::Triumph => {
                a.anim = Some(0x12);
                if ended(0x12) {
                    self.state = State::Idle;
                }
            }
            State::Phase2 | State::Phase3 => {
                let three = self.state == State::Phase3;
                self.phase = if three { 2 } else { 1 };
                a.set_level.push(if three { 6 } else { 4 });
                a.anim = Some(0);
                self.state = State::Idle;
            }
            State::Block { recovery } => {
                a.face = Some(0.05);
                if recovery {
                    a.anim = Some(0x1D);
                    if ended(0x1D) {
                        self.state = State::Idle;
                    }
                } else if std::mem::take(&mut self.blocked) {
                    // The one-shot blocked word (+0x8BC), set by the hero's blocked swing: the
                    // recovery is requested on the next tick.
                    self.state = State::Block { recovery: true };
                    a.anim = Some(0x19);
                } else if v.hero_attacking {
                    a.anim = Some(0x19);
                } else if roll(rng) < w.overhead_chance && d < w.overhead_distance && v.hero_attack_state {
                    self.once = false;
                    self.state = State::Overhead;
                } else {
                    self.state = State::Idle;
                }
            }
            State::Jump { part, velocity } => {
                a.travel = Move::Still;
                a.no_gravity = part != 2;
                match part {
                    0 => {
                        a.anim = Some(6);
                        a.face = Some(0.2);
                        if ended(6) {
                            let vy = w.tick_rate * w.jump_step * 0.5;
                            let vel = [(self.home[0] - v.position[0]) / w.tick_rate, vy, (self.home[2] - v.position[2]) / w.tick_rate];
                            self.state = State::Jump { part: 1, velocity: vel };
                        }
                    }
                    1 => {
                        a.anim = Some(7);
                        a.face = Some(0.2);
                        if len([self.home[0] - v.position[0], self.home[1] - v.position[1], self.home[2] - v.position[2]]) >= 3.0 {
                            let vel = [velocity[0], velocity[1] - w.jump_step, velocity[2]];
                            a.travel = Move::Fly(vel);
                            self.state = State::Jump { part: 1, velocity: vel };
                        } else {
                            self.state = State::Jump { part: 2, velocity: [0.0; 3] };
                            a.anim = Some(8);
                            a.no_gravity = true;
                        }
                    }
                    _ => {
                        a.anim = Some(8);
                        if ended(8) {
                            self.state = State::Idle;
                        }
                    }
                }
            }
            State::Roll(id) => {
                a.anim = Some(id);
                if ended(id) {
                    // The counter-attack test is "at most" the chance, so 100 always passes; it
                    // does not clear the once flag.
                    self.state = if d < w.roll_counter_distance && roll(rng) <= w.roll_counter_chance { State::Swing(0x16) } else { State::Idle };
                }
            }
            State::Swing(id) => {
                a.anim = Some(id);
                shared(self, &mut a, w.swing_damage, id);
                if ended(id) {
                    self.state = State::Idle;
                }
            }
            State::Overhead => {
                a.anim = Some(0x18);
                shared(self, &mut a, w.overhead_damage, 0x18);
                if ended(0x18) {
                    self.state = State::Idle;
                }
            }
            State::Lunge => {
                a.anim = Some(0x15);
                shared(self, &mut a, w.lunge_damage, 0x15);
                if ended(0x15) {
                    self.state = State::Idle;
                }
            }
            State::Combo(id) => {
                a.anim = Some(id);
                if id == 1 {
                    if ended(1) {
                        self.state = State::Idle;
                    }
                } else {
                    let k = (id - 0x16) as usize;
                    shared(self, &mut a, w.combo_damage[k.min(2)], id);
                    a.face = Some(1.0);
                    if v.clip.0 == id && v.clip.1 >= 0.99 {
                        if id < 0x18 {
                            self.once = false;
                            self.state = State::Combo(id + 1);
                        } else {
                            self.state = State::Combo(1);
                        }
                    }
                }
            }
            State::Throw => {
                a.anim = Some(0x13);
                if v.clip.0 == 0x13 && v.frame_bits & 0x20 != 0 && self.ready {
                    self.ready = false;
                    let fwd = norm([v.facing[0], 0.0, v.facing[2]]);
                    let right = [fwd[2], 0.0, -fwd[0]];
                    let dd = d;
                    let u = norm([v.hero[0] - v.position[0], 0.0, v.hero[2] - v.position[2]]);
                    let s = [-u[2], 0.0, u[0]];
                    let at = |k: f32| [v.hand[0] + 0.5 * dd * u[0] + k * 0.4 * dd * s[0], v.hand[1], v.hand[2] + 0.5 * dd * u[2] + k * 0.4 * dd * s[2]];
                    self.boomerang = Some(Boomerang {
                        position: v.hand,
                        direction: norm([0.5 * fwd[0] - 0.5 * right[0], 0.0, 0.5 * fwd[2] - 0.5 * right[2]]),
                        phase: 0,
                        a: at(1.0),
                        b: at(-1.0),
                        timer: (w.boomerang_timeout * w.tick_rate) as i32,
                        hit: false,
                    });
                }
                if ended(0x13) {
                    self.state = State::Idle;
                }
            }
            State::Flame { wave, wait, fired, playing } => {
                a.face = Some(0.05);
                let last = if p == 2 { 4 } else { 3 };
                if playing {
                    a.anim = Some(0x1A);
                    if ended(0x1A) {
                        if wave < last {
                            self.state = State::Flame { wave: wave + 1, wait: rng.range(w.flame_wait.0, w.flame_wait.1), fired: false, playing: false };
                            a.anim = Some(0);
                        } else {
                            self.counters[FLAME] = 0;
                            self.state = State::Idle;
                            a.anim = Some(0);
                        }
                    } else if v.clip.0 == 0x1A && v.frame_bits & 0x400 != 0 && !fired {
                        let k = wave as usize;
                        self.rings[k] = Ring { active: true, centre: v.position, age: 0, hit: self.rings[k].hit };
                        a.rings.push(v.position);
                        self.state = State::Flame { wave, wait, fired: true, playing };
                    }
                } else {
                    a.anim = Some(0);
                    if wait < 1 {
                        self.state = State::Flame { wave, wait, fired, playing: true };
                        a.anim = Some(0x1A);
                    } else {
                        self.state = State::Flame { wave, wait: wait - 1, fired, playing };
                    }
                }
            }
            State::Walk => {
                a.anim = Some(2);
                a.travel = Move::Steer { mode: 0, target: v.hero, weight: 0.3 };
                if d < w.walk_distance {
                    self.state = State::Idle;
                }
            }
            State::HopBack => {
                a.anim = Some(5);
                if ended(5) {
                    self.state = State::Idle;
                }
            }
            State::Orbit { right } => {
                let s = Self::side(v.position, v.hero);
                let k = if right { 5.0 } else { -5.0 };
                a.anim = Some(if right { 3 } else { 4 });
                a.travel = Move::Steer { mode: if right { 5 } else { 6 }, target: [v.position[0] + k * s[0], v.position[1], v.position[2] + k * s[2]], weight: 0.1 };
            }
            State::Death => {
                a.anim = Some(0);
                a.travel = Move::Still;
                a.set_flag.push(273);
            }
        }
        // The boomerang's flight and the flame rings (every tick, in every state).
        self.flight(v, w, &mut a);
        for r in &mut self.rings {
            if !r.active {
                continue;
            }
            r.age += 1;
            if r.age > RING_LIFE {
                r.active = false;
                continue;
            }
            let (rr, hh) = ring_size(r.age);
            let dd = hdist(r.centre, v.hero);
            if !r.hit && dd > rr - 0.3 && dd < rr + 0.3 && v.hero[1] - r.centre[1] < hh {
                r.hit = true;
                a.raw_damage.push(w.flame_damage);
            }
        }
        a
    }

    /// The boomerang's per-tick flight: the steering weights by distance, the 0.7 step, the
    /// ground floor, the hit and the targets in order (A, the hero, B, the hand).
    fn flight(&mut self, v: &View, w: &Words, a: &mut Act) {
        let Some(mut b) = self.boomerang else { return };
        if b.phase >= 4 {
            return;
        }
        b.timer -= 1;
        if b.timer <= 0 {
            // The timeout: back to the hand without a catch; the ready flag restored.
            self.boomerang = None;
            self.ready = true;
            return;
        }
        let target = match b.phase {
            0 => b.a,
            1 => [v.hero[0], (v.ground)(v.hero[0], v.hero[2]) + 2.0, v.hero[2]],
            2 => b.b,
            _ => v.hand,
        };
        let off = [target[0] - b.position[0], target[1] - b.position[1], target[2] - b.position[2]];
        let r = len(off);
        if r < 0.5 {
            b.position = target;
            b.phase += 1;
            if b.phase == 2 {
                let dd = hdist(v.position, v.hero);
                let u = norm([v.hero[0] - v.position[0], 0.0, v.hero[2] - v.position[2]]);
                let s = [-u[2], 0.0, u[0]];
                b.b = [v.hand[0] + 0.5 * dd * u[0] - 0.4 * dd * s[0], v.hand[1], v.hand[2] + 0.5 * dd * u[2] - 0.4 * dd * s[2]];
            }
        } else {
            let t = [off[0] / r, off[1] / r, off[2] / r];
            let k = Self::steer_weight(r, w);
            b.direction = norm([(1.0 - k) * b.direction[0] + k * t[0], (1.0 - k) * b.direction[1] + k * t[1], (1.0 - k) * b.direction[2] + k * t[2]]);
            b.position = [b.position[0] + w.boomerang_speed * b.direction[0], b.position[1] + w.boomerang_speed * b.direction[1], b.position[2] + w.boomerang_speed * b.direction[2]];
            let g = (v.ground)(b.position[0], b.position[2]) + 1.0;
            if b.position[1] < g {
                b.position[1] = g;
                b.direction[1] = b.direction[1].max(0.0);
            }
        }
        // The hit (the hero within 65; a 2.5-radius disc against the hero's body, taken here as
        // a 1.0 by 3.5 upright cylinder): 5 raw, once per throw.
        if !b.hit && len([v.hero[0] - b.position[0], v.hero[1] - b.position[1], v.hero[2] - b.position[2]]) <= 65.0 {
            let dy = b.position[1] - v.hero[1];
            if hdist(b.position, v.hero) <= 2.5 + 1.0 && (-0.125..=3.5 + 0.125).contains(&dy) {
                b.hit = true;
                a.raw_damage.push(w.boomerang_damage);
            }
        }
        self.boomerang = Some(b);
    }

    /// The boomerang's steering weight by the distance to its target.
    pub fn steer_weight(r: f32, w: &Words) -> f32 {
        if r < 5.0 { 1.0 } else if r < 10.0 { 0.25 } else { w.far_weight }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn words() -> Words {
        Words {
            block_chance: [25, 35, 75],
            jump_distance: 5.0,
            jump_chance: (5, 80),
            roll_chance: 50,
            roll_counter_distance: 5.0,
            roll_counter_chance: 100,
            overhead_distance: 5.0,
            overhead_chance: 25,
            boomerang_range: (15.0, 40.0),
            boomerang_cooldown: 8,
            boomerang_chance: [90, 50, 25],
            boomerang_speed: 0.7,
            boomerang_damage: 5,
            flame_range: (10.0, 40.0),
            flame_cooldown: 10,
            flame_chance: [0, 50, 75],
            flame_wait: (0, 15),
            flame_damage: 15,
            swing_distance: 10.0,
            swing_cooldown: 10,
            swing_chance: [60, 46, 20],
            swing_damage: 10,
            lunge_distance: 10.0,
            lunge_cooldown: 8,
            lunge_chance: [30, 40, 50],
            lunge_damage: 10,
            combo_distance: 10.0,
            combo_cooldown: 10,
            combo_chance: [10, 20, 30],
            combo_damage: [10, 10, 10],
            walk_distance: 25.0,
            orbit_distance: 10.0,
            hop_distance: 5.0,
            hop_chance: 50,
            overhead_damage: 15,
            phase2: 75,
            phase3: 37,
            tick_rate: 60.0,
            jump_step: 0.05,
            boomerang_timeout: 10.0,
            far_weight: 0.1,
        }
    }

    fn flat(_: f32, _: f32) -> f32 {
        0.0
    }

    #[test]
    fn hurt_side() {
        // SPEC-0019 G14: not hittable in the flinch, catch, roll and jump; the block guards; the
        // first damaging hit is doubled; the blocked word sends the block to its recovery.
        let mut k = King::new([0.0; 3]);
        assert!(k.hittable() && !k.guarding());
        assert_eq!((k.hit_damage(0), k.hit_damage(7), k.hit_damage(7)), (0, 14, 7));
        for s in [State::Flinch(0x0B), State::Catch, State::Roll(0x14), State::Jump { part: 7, velocity: [0.0; 3] }] {
            k.state = s;
            assert!(!k.hittable());
        }
        k.state = State::Block { recovery: false };
        assert!(k.hittable() && k.guarding());
        k.blocked = true;
        let mut rng = Rng::seeded(1);
        k.tick(&view([0.0, 0.0, 5.0], 100), &words(), &mut rng);
        assert_eq!((k.state, k.blocked), (State::Block { recovery: true }, false));
    }

    fn view(hero: Vec3, hp: i32) -> View<'static> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero,
            hero_facing: [0.0, 0.0, -1.0],
            hero_guarding: false,
            hero_down: false,
            hero_attacking: false,
            hero_attack_state: false,
            was_hit: false,
            reach: false,
            frame_bits: 0,
            clip: (0, 0.0, false),
            hp,
            max_hp: 100,
            hand: [0.0, 2.0, 0.0],
            ground: &flat,
        }
    }

    #[test]
    fn phases_and_death() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let mut k = King::new([0.0; 3]);
        // 80: phase 1; 74: the phase 2 entry (event 4), idle the same tick; 36: phase 3 (event
        // 6); a phase-3 boss at 60 re-enters phase 2 (the quirk); 37 is not below 0.37.
        assert!(k.tick(&view([0.0, 0.0, 15.0], 80), &w, &mut rng).set_level.is_empty());
        let a = k.tick(&view([0.0, 0.0, 15.0], 74), &w, &mut rng);
        assert_eq!((a.set_level, k.phase), (vec![4], 1));
        assert!(!matches!(k.state, State::Phase2));
        assert_eq!(k.tick(&view([0.0, 0.0, 15.0], 36), &w, &mut rng).set_level, vec![6]);
        assert_eq!(k.tick(&view([0.0, 0.0, 15.0], 60), &w, &mut rng).set_level, vec![4]);
        let mut j = King::new([0.0; 3]);
        assert!(j.tick(&view([0.0, 0.0, 15.0], 37), &w, &mut rng).set_level.contains(&4));
        // Health 0: death and flag 273.
        assert!(k.tick(&view([0.0, 0.0, 15.0], 0), &w, &mut rng).set_flag.contains(&273));
    }

    #[test]
    fn rings_and_jump_and_side() {
        // The ring growth vector (life 0.8 s).
        let (r, h) = ring_size(1);
        assert!((r - 1.0104).abs() < 1e-3 && (h - 4.9630).abs() < 1e-3);
        let (r, h) = ring_size(3);
        assert!((r - 2.0313).abs() < 1e-3 && (h - 3.5771).abs() < 1e-3);
        let (r, h) = ring_size(48);
        assert!((r - 25.0).abs() < 1e-3 && (h - 1.2145).abs() < 2e-3);
        // A still hero 12 units out is touched on tick 22 once.
        let w = words();
        let mut rng = Rng::seeded(2);
        let mut k = King::new([0.0; 3]);
        k.state = State::Walk;
        k.rings[0] = Ring { active: true, centre: [0.0; 3], age: 21, hit: false };
        let mut v = view([12.0, 0.0, 0.0], 100);
        v.position = [100.0, 0.0, 0.0];
        assert_eq!(k.tick(&v, &w, &mut rng).raw_damage, vec![15]);
        assert!(k.tick(&v, &w, &mut rng).raw_damage.is_empty());
        // The jump flight vector: home 30 along +x: v = (0.5, 1.5, 0).
        let mut k = King::new([30.0, 0.0, 0.0]);
        k.state = State::Jump { part: 0, velocity: [0.0; 3] };
        let mut v = view([2.0, 0.0, 0.0], 100);
        v.clip = (6, 1.0, true);
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Jump { part: 1, velocity: [0.5, 1.5, 0.0] });
        // The orbit side: u = (0, 0, 1) gives (-1, 0, 0).
        assert_eq!(King::side([0.0; 3], [0.0, 0.0, 3.0]), [-1.0, 0.0, 0.0]);
        // The jump ramp: 21 units from home, about 39.6.
        assert!((King::jump_chance(&w, 21.0) - 39.615).abs() < 0.01);
        // The boomerang steering weights.
        assert_eq!([King::steer_weight(4.0, &w), King::steer_weight(7.0, &w), King::steer_weight(12.0, &w)], [1.0, 0.25, 0.1]);
    }

    #[test]
    fn shared_hit_and_combo() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let mut k = King::new([0.0; 3]);
        k.state = State::Swing(0x17);
        let mut v = view([0.0, 0.0, 3.0], 100);
        v.clip = (0x17, 0.3, false);
        v.frame_bits = 1;
        v.reach = true;
        // Once per attack.
        assert_eq!(k.tick(&v, &w, &mut rng).hero_damage, Some((10, false)));
        assert_eq!(k.tick(&v, &w, &mut rng).hero_damage, None);
        // The guard at dot -0.5 exactly blocks with 0.
        let mut k = King::new([0.0; 3]);
        k.state = State::Overhead;
        let mut v2 = view([0.0, 0.0, 3.0], 100);
        v2.clip = (0x18, 0.3, false);
        v2.frame_bits = 1;
        v2.reach = true;
        v2.hero_guarding = true;
        v2.hero_facing = [0.0, 0.0, -0.5];
        assert_eq!(k.tick(&v2, &w, &mut rng).hero_damage, Some((0, true)));
        // The combo: a hit in each animation; the flag cleared at the transitions.
        let mut k = King::new([0.0; 3]);
        k.state = State::Combo(0x16);
        let mut total = 0;
        for id in [0x16, 0x17, 0x18] {
            let mut v = view([0.0, 0.0, 3.0], 100);
            v.frame_bits = 1;
            v.reach = true;
            v.clip = (id, 0.5, false);
            total += k.tick(&v, &w, &mut rng).hero_damage.map_or(0, |d| d.0);
            total += k.tick(&v, &w, &mut rng).hero_damage.map_or(0, |d| d.0);
            v.clip = (id, 0.99, false);
            k.tick(&v, &w, &mut rng);
        }
        assert_eq!((total, k.state), (30, State::Combo(1)));
        // A roll ending within 5 units is always countered, with the once flag left as it was.
        let mut k = King::new([0.0; 3]);
        k.state = State::Roll(9);
        let mut v = view([0.0, 0.0, 4.9], 100);
        v.clip = (9, 1.0, true);
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Swing(0x16));
    }

    #[test]
    fn walk_and_flame_gate() {
        let w = words();
        let mut rng = Rng::seeded(4);
        // 26 units: walk; 25: no walk.
        let mut k = King::new([0.0; 3]);
        k.tick(&view([0.0, 0.0, 26.0], 100), &w, &mut rng);
        assert_eq!(k.state, State::Walk);
        let mut k = King::new([0.0; 3]);
        k.tick(&view([0.0, 0.0, 25.0], 100), &w, &mut rng);
        assert_ne!(k.state, State::Walk);
        // No flame wave in phase 1 whatever the counter; none while a boomerang is out.
        let mut k = King::new([0.0; 3]);
        k.counters[FLAME] = 601;
        k.counters[BOOMERANG] = 0;
        for _ in 0..50 {
            k.tick(&view([0.0, 0.0, 20.0], 100), &w, &mut rng);
            assert!(!matches!(k.state, State::Flame { .. }));
        }
    }
}
