//! The Black Knight boss (SPEC-0012; the controller of world pack 989): the per-fight record
//! (state, phase, speed multiplier, counter flag, the six weak points, the charge references),
//! the flag decision (laugh, angry and enraged starts, loosen neck, en garde), the defeat test
//! with level state 0, parry and deflect with their accumulators, the attack selection (the
//! opening kick, kick, slow and angry swings, the phase-2 group, the balcony spin), the idle
//! movement (turns, approach, orbit), the swings with their block, the four enraged attacks with
//! their 600-tick stun windows and weak-point hits, the flying sword's sub-states, the glow
//! pulse and the health display's quirk.
//!
//! The host measures what needs the world: the weapon tests, the weak-point overlaps, the shot
//! overlap, the walk-mesh edge and body contact flags, the hero's place on the knight's mesh,
//! the high swing's second measure, the ground under the knight and the sword's hero test.
//! The flying sword's orbit offset is built from the knight's facing (the spec's pose-derived
//! basis is Low), so its world path is approximate; its counts, height, homing, return and
//! hit rules follow the spec. Every tuning value is read from the boot data at the spec's
//! addresses.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The knight's locomotion slot values (SPEC-0012; R-0067): mode 0 plays entry 3 (clip 40), mode 5
/// entry 4 (clip 38, positive sideways root: it moves to its left) and mode 6 entry 5 (clip 39,
/// to its right).
pub const SLOTS: [i32; 7] = [3, 3, 3, -1, -1, 5, 4];

/// The tuning block: 56 words from 0x2DA3E8 to 0x2DA4C7.
pub const BASE: u32 = 0x002D_A3E8;
pub const WORDS: usize = 56;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub laugh: i32,
    pub loosen: i32,
    pub en_garde: i32,
    pub parry_increase: i32,
    pub deflect_increase: i32,
    pub deflect_stance: (i32, i32),
    pub counter: i32,
    pub kick_range: f32,
    pub kick_chance: i32,
    pub kick_damage: i32,
    pub slow_range: f32,
    pub slow_damage: i32,
    pub angry_range: f32,
    pub angry_damage: i32,
    pub charge_range: f32,
    pub charge_damage: i32,
    pub charge_stun: f32,
    pub high_range: f32,
    pub high_damage: i32,
    pub high_stun: f32,
    pub floor_damage: i32,
    pub floor_stun: f32,
    pub spin_stun: f32,
    pub turn_threshold: f32,
    pub close: f32,
    pub orbit: f32,
    pub sword_damage: i32,
    pub sword_speed: f32,
    pub parts: [i32; 6],
    pub balcony: i32,
    pub knock: f32,
    pub launch_threshold: f32,
    pub spin_step: f32,
    pub spin_accel: f32,
    pub path_rate: f32,
    pub revolutions: i32,
    pub bounce_speed: f32,
    pub bounce_limit: f32,
    pub min_height: f32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let i = |at: u32| w(at) as i32;
        let f = |at: u32| f32::from_bits(w(at));
        Some(Self {
            laugh: i(0x002D_A3E8),
            loosen: i(0x002D_A3EC),
            en_garde: i(0x002D_A3F0),
            parry_increase: i(0x002D_A3F4),
            deflect_increase: i(0x002D_A3F8),
            deflect_stance: (i(0x002D_A3FC), i(0x002D_A400)),
            counter: i(0x002D_A404),
            kick_range: i(0x002D_A408) as f32,
            kick_chance: i(0x002D_A40C),
            kick_damage: i(0x002D_A410),
            slow_range: i(0x002D_A414) as f32,
            slow_damage: i(0x002D_A418),
            angry_range: i(0x002D_A41C) as f32,
            angry_damage: i(0x002D_A420),
            charge_range: i(0x002D_A424) as f32,
            charge_damage: i(0x002D_A42C),
            charge_stun: f(0x002D_A430),
            high_range: i(0x002D_A434) as f32,
            high_damage: i(0x002D_A438),
            high_stun: f(0x002D_A43C),
            floor_damage: i(0x002D_A440),
            floor_stun: f(0x002D_A444),
            spin_stun: f(0x002D_A448),
            turn_threshold: i(0x002D_A44C) as f32,
            close: i(0x002D_A450) as f32,
            orbit: i(0x002D_A458) as f32,
            sword_damage: i(0x002D_A45C),
            sword_speed: i(0x002D_A460) as f32,
            parts: std::array::from_fn(|k| i(0x002D_A468 + 4 * k as u32)),
            balcony: i(0x002D_A480),
            knock: f(0x002D_A484),
            launch_threshold: f(0x002D_A488),
            spin_step: f(0x002D_A48C),
            spin_accel: f(0x002D_A494),
            path_rate: f(0x002D_A4A4),
            revolutions: i(0x002D_A4B8),
            bounce_speed: f(0x002D_A4BC),
            bounce_limit: f(0x002D_A4C0),
            min_height: f(0x002D_A4C4),
        })
    }
}

/// The weak points: left knee, right knee, left under-arm, right under-arm, back, neck.
pub const LEFT_KNEE: usize = 0;
pub const RIGHT_KNEE: usize = 1;
pub const LEFT_ARM: usize = 2;
pub const RIGHT_ARM: usize = 3;
pub const BACK: usize = 4;
pub const NECK: usize = 5;

/// The health display's current value: LK + RK + LA + RK + B + N (the right knee twice, the
/// right under-arm never; reproduced).
pub fn display_hp(p: &[i32; 6]) -> i32 {
    p[LEFT_KNEE] + p[RIGHT_KNEE] + p[LEFT_ARM] + p[RIGHT_KNEE] + p[BACK] + p[NECK]
}

/// A knock-back velocity: unit(9 d + 12 up) x scale.
pub fn knock(d: Vec3, scale: f32) -> Vec3 {
    let v = [9.0 * d[0], 12.0, 9.0 * d[2]];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|c| c / l * scale)
}

/// The charge's direction test: the reference by the larger lane of the horizontal unit
/// direction (A x, B z) and whether |reference - A| is below the range.
pub fn charge_passes(refs: &[f32; 4], a: f32, b: f32, range: f32) -> bool {
    let r = if b.abs() < a.abs() { if a > 0.0 { refs[0] } else { refs[1] } } else if b > 0.0 { refs[2] } else { refs[3] };
    (r - a).abs() < range
}

/// One weak point's glow pulse: the colour (r, g, b, a) and the direction (up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub colour: [f32; 4],
    pub up: bool,
}

impl Default for Glow {
    fn default() -> Self {
        Self { colour: [0.0, 0.0, 0.0, 1.0], up: true }
    }
}

impl Glow {
    /// One tick: unlit resets to black going up; lit ramps r and b by 0.01, turning at 1.0 and 0.
    pub fn step(&mut self, lit: bool) {
        if !lit {
            *self = Self::default();
            return;
        }
        if self.up {
            if self.colour[0] >= 1.0 {
                self.up = false;
            } else {
                self.colour[0] += 0.01;
                self.colour[2] += 0.01;
            }
        } else if self.colour[0] <= 0.0 {
            self.up = true;
        } else {
            self.colour[0] -= 0.01;
            self.colour[2] -= 0.01;
        }
    }
}

/// The flying sword (one per knight) with the shared orbit values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sword {
    pub sub: u8,
    pub position: Vec3,
    pub angle: f32,
    pub bounce: f32,
    bounce_up: bool,
    pub latch: bool,
    pub returned: bool,
    pub step: f32,
    pub rate: f32,
    pub path: f32,
    pub count: i32,
    pub height: f32,
    lift: f32,
}

impl Default for Sword {
    fn default() -> Self {
        Self { sub: 5, position: [0.0; 3], angle: 0.0, bounce: 0.0, bounce_up: true, latch: false, returned: false, step: 0.0, rate: 0.0, path: 0.0, count: 0, height: 0.0, lift: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Laugh,
    AngryStart,
    EnragedStart,
    LoosenNeck,
    EnGarde,
    Parry,
    /// The deflect stance: ticks left and the strike animation playing.
    Deflect(i32, Option<u32>),
    Kick,
    SlowSwing { blocked: bool },
    AngrySwing { blocked: bool },
    /// An enraged attack's step: 0 the first, then as the spec's tables.
    Charge(u8),
    HighSwing(u8),
    FloorStrike(u8),
    Spin(u8),
    TurnToward,
    TurnAway,
    Approach,
    Orbit,
    Defeated,
}

impl State {
    pub fn ready(self) -> bool {
        matches!(self, State::Idle | State::TurnToward | State::TurnAway | State::Approach | State::Orbit)
    }
}

/// What the knight sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero index (0 Tor, 1 Roberta), its attack-state predicate and its guard predicate.
    pub hero_index: u8,
    pub hero_attack: bool,
    pub hero_guard: bool,
    /// The strike-active predicate (SPEC-0012 G9): Tor in action state 13 with sub-state 11 or
    /// 12, or in action state 7 in air phase 8 or 9; Roberta in state 13 with sub-state 5 or 12.
    pub hero_strike: bool,
    /// Roberta's shot slots: any active; the first active one overlaps the knight.
    pub shots: bool,
    pub shot_overlap: bool,
    /// The monster weapon test of weapons 1 to 5 this tick.
    pub weapons: [bool; 5],
    pub frame_bits: u32,
    pub clip: (u32, bool),
    /// The hero found on the knight's ground mesh (the batched and the full-scan locates).
    pub hero_on_mesh: bool,
    pub hero_on_mesh_full: bool,
    /// The body contact and walk-mesh edge-contact flags.
    pub contact: bool,
    pub edge: bool,
    /// The weak-point test of a part: the hero's body cylinder touches it (melee).
    pub part_hit: &'a dyn Fn(usize) -> bool,
    /// The weak-point test of a part by an armed hero shot (tested whatever the gate).
    pub part_shot: &'a dyn Fn(usize) -> bool,
    /// The hero's damage query for the knight (argument 1.0), rounded down.
    pub hero_damage: i32,
    /// The high swing's second measure (below 10.0 to swing).
    pub second_measure: f32,
    /// The ground under the knight (probed 8.0 above it); the hand and return points.
    pub ground: Option<f32>,
    pub hand: Vec3,
    pub return_point: Vec3,
    /// The flying sword's cylinder touches the hero at this point.
    pub sword_hit: &'a dyn Fn(Vec3) -> bool,
    pub level: &'a dyn Fn(u16) -> bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Act {
    pub anim: Option<u32>,
    /// Turn toward the hero at this weight (negative: away).
    pub face: Option<f32>,
    pub travel: Option<(u8, Vec3)>,
    /// A hit on the hero (unreduced) with its knock-back velocity.
    pub hero_damage: Option<(i32, Option<Vec3>)>,
    pub set_level: Vec<u16>,
    /// A weak-point hit: the part (effect 305 at its frame).
    pub part_effect: Option<usize>,
    pub consume_shot: bool,
    /// The flying sword's position while it flies.
    pub sword: Option<Vec3>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Knight {
    pub state: State,
    pub phase: u8,
    pub speed: f32,
    counter: bool,
    pub parts: [i32; 6],
    refs: [f32; 4],
    laugh: bool,
    angry: bool,
    enraged: bool,
    loosen: bool,
    en_garde: bool,
    stun: i32,
    hit: bool,
    orbit_side: u8,
    orbit_timer: i32,
    parry: i32,
    deflect: i32,
    latch: bool,
    lit: [bool; 6],
    pub glow: [Glow; 6],
    pub sword: Sword,
    defeated_sent: bool,
    /// This tick's stun gate passed (base state 1 for the tick: neither hittable nor guarding,
    /// so the hero's swing passes through; otherwise the knight guards and blocks it; G9, G15).
    pub exposed: bool,
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn hdir(from: Vec3, to: Vec3) -> Vec3 {
    let d = hdist(from, to).max(1e-6);
    [(to[0] - from[0]) / d, 0.0, (to[2] - from[2]) / d]
}

fn roll(rng: &mut Rng, chance: i32) -> bool {
    rng.range(0, 100) < chance
}

fn toward(p: Vec3, t: Vec3, step: f32) -> (Vec3, f32) {
    let d = [t[0] - p[0], t[1] - p[1], t[2] - p[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if l <= step || l == 0.0 { (t, l) } else { (std::array::from_fn(|k| p[k] + d[k] * step / l), l) }
}

impl Knight {
    /// The per-fight record at its start.
    pub fn new(w: &Words) -> Self {
        Self {
            state: State::Idle,
            phase: 0,
            speed: 1.0,
            counter: true,
            parts: w.parts,
            refs: [20.0, -20.0, 60.0, -60.0],
            laugh: false,
            angry: false,
            enraged: false,
            loosen: false,
            en_garde: false,
            stun: 0,
            hit: false,
            orbit_side: 0,
            orbit_timer: 30,
            parry: 0,
            deflect: 0,
            latch: false,
            lit: [false; 6],
            glow: [Glow::default(); 6],
            sword: Sword::default(),
            defeated_sent: false,
            exposed: false,
        }
    }

    pub fn defeated(&self) -> bool {
        self.parts.iter().all(|&p| p <= 0)
    }

    fn select(&mut self, s: State) {
        self.hit = false;
        self.state = s;
    }

    /// One tick (SPEC-0012 Per-tick order).
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
        self.exposed = false;
        // 1. The glow, from the flags of the previous tick.
        for k in 0..6 {
            self.glow[k].step(self.lit[k]);
        }
        self.lit = [false; 6];
        // 2. The flags, then the defeat test.
        if self.state.ready() {
            if self.laugh {
                self.laugh = false;
                if roll(rng, w.laugh) {
                    self.state = State::Laugh;
                }
            } else if self.angry {
                self.angry = false;
                self.phase = 1;
                self.state = State::AngryStart;
            } else if self.enraged {
                self.enraged = false;
                self.phase = 2;
                self.state = State::EnragedStart;
            } else if self.loosen {
                self.loosen = false;
                if roll(rng, w.loosen) {
                    self.state = State::LoosenNeck;
                }
            } else if self.en_garde {
                self.en_garde = false;
                self.state = State::EnGarde;
                self.speed *= (w.en_garde + 100) as f32 / 100.0;
            }
        }
        if self.defeated() {
            self.state = State::Defeated;
            if !(v.level)(0) && !self.defeated_sent {
                self.defeated_sent = true;
                a.set_level.push(0);
            }
        }
        let d = hdist(v.position, v.hero);
        let to_hero = hdir(v.position, v.hero);
        // 3. Parry (Tor) and deflect (Roberta), in a ready state.
        if self.state.ready() {
            if v.hero_index == 0 {
                if !v.hero_attack {
                    self.latch = false;
                } else if !self.latch {
                    self.latch = true;
                    if rng.range(0, 100) < self.parry {
                        self.parry = 0;
                        self.state = State::Parry;
                    } else {
                        self.parry += w.parry_increase;
                    }
                }
            } else if v.hero_index == 1 {
                if !v.shots {
                    self.latch = false;
                } else if !self.latch {
                    self.latch = true;
                    if v.shot_overlap {
                        if rng.range(0, 100) < self.deflect {
                            self.deflect = 0;
                            let t = rng.range(60 * w.deflect_stance.0, 60 * w.deflect_stance.1);
                            self.state = State::Deflect(t, None);
                        } else {
                            self.deflect += w.deflect_increase;
                        }
                    }
                }
            }
        }
        // 4. The attack selection.
        if self.state.ready() {
            self.choose(v, w, d, to_hero, rng);
        }
        // 5. The idle movement.
        if self.state.ready() {
            self.movement(v, w, d, to_hero, rng);
        }
        // 6. The state handler.
        self.run(v, w, d, to_hero, rng, &mut a);
        a
    }

    fn choose(&mut self, v: &View, w: &Words, d: f32, to_hero: Vec3, rng: &mut Rng) {
        if self.counter {
            self.counter = false;
            if roll(rng, w.counter) {
                return self.select(State::Kick);
            }
        }
        if v.hero_guard && d < w.kick_range && roll(rng, w.kick_chance) {
            return self.select(State::Kick);
        }
        if self.phase == 0 && d < w.slow_range {
            return self.select(State::SlowSwing { blocked: false });
        }
        if self.phase == 1 && d < w.angry_range {
            return self.select(State::AngrySwing { blocked: false });
        }
        if self.phase == 2 {
            let p = self.parts;
            if (p[LEFT_KNEE] > 0 || p[RIGHT_KNEE] > 0) && charge_passes(&self.refs, to_hero[0], to_hero[2], w.charge_range) {
                return self.select(State::Charge(0));
            }
            if d < w.high_range && v.second_measure < 10.0 && (p[LEFT_ARM] > 0 || p[RIGHT_ARM] > 0) {
                return self.select(State::HighSwing(0));
            }
            if p[BACK] > 0 {
                return self.select(State::FloorStrike(0));
            }
            if p[..5].iter().all(|&x| x <= 0) {
                return self.select(State::Spin(0));
            }
        }
        if roll(rng, w.balcony) && !v.hero_on_mesh_full {
            self.select(State::Spin(0));
        }
    }

    fn movement(&mut self, v: &View, w: &Words, d: f32, to_hero: Vec3, rng: &mut Rng) {
        let cos = to_hero[0] * v.facing[0] + to_hero[2] * v.facing[2];
        let right = [-v.facing[2], 0.0, v.facing[0]];
        match self.state {
            State::Idle => {
                if cos < w.turn_threshold.to_radians().cos() {
                    let on_right = to_hero[0] * right[0] + to_hero[2] * right[2] > 0.0;
                    self.state = if on_right { State::TurnToward } else { State::TurnAway };
                } else if !v.hero_on_mesh {
                } else if d > w.close {
                    self.state = State::Approach;
                } else if d <= w.orbit {
                    self.state = State::Orbit;
                    self.orbit_side = rng.range(0, 1) as u8;
                    self.orbit_timer = 30;
                }
            }
            State::TurnToward | State::TurnAway => {
                if cos > 0.99 {
                    self.state = State::Idle;
                }
            }
            State::Approach => {
                if d < w.close {
                    self.state = State::Idle;
                }
            }
            State::Orbit => {
                if d > 18.0 || !v.hero_on_mesh {
                    self.state = State::Idle;
                }
            }
            _ => {}
        }
    }

    /// The stun step of an enraged attack: the countdown and the part tests. Returns whether the
    /// countdown ended this tick (no part test then).
    fn stun_tick(&mut self, v: &View, parts: &[usize], reaction: u32, before: bool, a: &mut Act) -> bool {
        if self.stun < 1 {
            return true;
        }
        if before {
            self.stun -= 1;
        }
        for &k in parts {
            if self.parts[k] > 0 {
                self.lit[k] = true;
            }
        }
        // The gate for the melee part test (G9 step 5): knees, the hero attacking in front of
        // the knight (dot with its forward axis above 0.25); the back, attacking and behind
        // (below -0.25); the under-arms, striking and on that arm's side (the side axis is +X
        // when the knight faces +Z: the left arm above 0.25, the right below -0.25), with the
        // arm's part above 0; the neck, striking and behind. A passing gate exposes the knight
        // for the tick; an armed shot is tested whatever the gate.
        let dir = hdir(v.position, v.hero);
        let f = {
            let l = v.facing[0].hypot(v.facing[2]).max(1e-6);
            [v.facing[0] / l, 0.0, v.facing[2] / l]
        };
        let (front, side) = (dir[0] * f[0] + dir[2] * f[2], dir[0] * f[2] - dir[2] * f[0]);
        let gate = |k: usize, parts: &[i32; 6]| match k {
            LEFT_KNEE | RIGHT_KNEE => v.hero_attack && front > 0.25,
            BACK => v.hero_attack && front < -0.25,
            NECK => v.hero_strike && front < -0.25,
            LEFT_ARM => v.hero_strike && side > 0.25 && parts[k] > 0,
            _ => v.hero_strike && side < -0.25 && parts[k] > 0,
        };
        let open: Vec<bool> = parts.iter().map(|&k| gate(k, &self.parts)).collect();
        self.exposed = open.iter().any(|&o| o);
        for (n, &k) in parts.iter().enumerate() {
            if self.parts[k] > 0 && ((open[n] && (v.part_hit)(k)) || (v.part_shot)(k)) {
                self.parts[k] = if self.parts[k] < v.hero_damage { 0 } else { self.parts[k] - v.hero_damage };
                self.hit = true;
                a.part_effect = Some(k);
                a.anim = Some(reaction);
                break;
            }
        }
        if !before {
            self.stun -= 1;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn run(&mut self, v: &View, w: &Words, _d: f32, to_hero: Vec3, rng: &mut Rng, a: &mut Act) {
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let window = |n: u32, k: usize| v.clip.0 == n && v.frame_bits & (1 << k) != 0 && v.weapons[k];
        let blocked = |v: &View| v.hero_guard && v.hero_facing[0] * v.facing[0] + v.hero_facing[2] * v.facing[2] < -0.5;
        let ticks = |s: f32| (s * 60.0) as i32;
        match self.state {
            State::Idle => a.anim = Some(0),
            State::Defeated => a.anim = Some(43),
            State::Laugh | State::AngryStart | State::EnragedStart | State::LoosenNeck | State::EnGarde | State::Parry => {
                let n = match self.state {
                    State::Laugh => 12,
                    State::AngryStart => 13,
                    State::EnragedStart => 14,
                    State::LoosenNeck => 1,
                    State::EnGarde => 2,
                    _ => 8,
                };
                a.anim = Some(n);
                if ended(n) {
                    if self.state == State::Parry {
                        self.loosen = true;
                    }
                    self.state = State::Idle;
                }
            }
            State::Deflect(t, strike) => {
                let t = t - 1;
                match strike {
                    Some(n) => {
                        a.anim = Some(n);
                        self.state = if ended(n) { if t <= 0 { State::Idle } else { State::Deflect(t, None) } } else { State::Deflect(t, strike) };
                    }
                    None => {
                        a.anim = Some(9);
                        self.state = if t <= 0 { State::Idle } else if v.hero_attack || v.shots { State::Deflect(t, Some(if rng.range(0, 1) == 0 { 10 } else { 11 })) } else { State::Deflect(t, None) };
                    }
                }
            }
            State::Kick => {
                a.anim = Some(17);
                if window(17, 4) && !self.hit {
                    self.hit = true;
                    a.hero_damage = Some((w.kick_damage, Some(knock(v.facing, w.knock))));
                }
                if ended(17) {
                    self.state = State::Idle;
                }
            }
            State::SlowSwing { blocked: b } | State::AngrySwing { blocked: b } => {
                let slow = matches!(self.state, State::SlowSwing { .. });
                let (swing, recovery, damage) = if slow { (15, 16, w.slow_damage) } else { (18, 19, w.angry_damage) };
                if slow {
                    a.face = Some(0.25);
                }
                if b {
                    a.anim = Some(recovery);
                    if ended(recovery) {
                        self.state = State::Idle;
                        if slow { self.angry = true } else { self.enraged = true }
                    }
                } else {
                    a.anim = Some(swing);
                    if window(swing, 0) && !self.hit {
                        if blocked(v) {
                            self.state = if slow { State::SlowSwing { blocked: true } } else { State::AngrySwing { blocked: true } };
                            return;
                        }
                        self.hit = true;
                        a.hero_damage = Some((damage, None));
                        self.phase = 0;
                    }
                    if ended(swing) {
                        self.state = State::Idle;
                        match (slow, self.hit) {
                            (true, true) => self.laugh = true,
                            (true, false) => self.angry = true,
                            (false, false) => self.enraged = true,
                            _ => {}
                        }
                    }
                }
            }
            State::Charge(step) => {
                let next = match step {
                    0 => {
                        a.anim = Some(26);
                        a.face = Some(0.25);
                        if ended(26) { 1 } else { 0 }
                    }
                    1 => {
                        a.anim = Some(27);
                        a.travel = Some((255, v.position));
                        if v.edge { 3 } else if v.contact { 2 } else { 1 }
                    }
                    2 => {
                        a.anim = Some(28);
                        if v.clip.0 == 28 && v.frame_bits & 0x1F != 0 && !self.hit {
                            self.hit = true;
                            self.phase = 0;
                            a.hero_damage = Some((w.charge_damage, Some(knock(to_hero, w.knock))));
                        }
                        if ended(28) {
                            self.state = State::Idle;
                            return;
                        }
                        2
                    }
                    3 => {
                        a.anim = Some(29);
                        if ended(29) {
                            self.stun = ticks(w.charge_stun);
                            4
                        } else {
                            3
                        }
                    }
                    4 => {
                        a.anim = Some(30);
                        if self.stun_tick(v, &[RIGHT_KNEE, LEFT_KNEE], 45, true, a) {
                            5
                        } else if self.parts[LEFT_KNEE] <= 0 && self.parts[RIGHT_KNEE] <= 0 {
                            6
                        } else {
                            4
                        }
                    }
                    5 | 6 => {
                        let n = if step == 5 { 31 } else { 32 };
                        a.anim = Some(n);
                        if ended(n) {
                            self.phase = 0;
                            if step == 6 {
                                self.en_garde = true;
                            }
                            self.state = State::Idle;
                            return;
                        }
                        step
                    }
                    _ => 0,
                };
                self.state = State::Charge(next);
            }
            State::HighSwing(step) => {
                let next = match step {
                    0 => {
                        a.anim = Some(33);
                        if window(33, 1) && !self.hit {
                            self.hit = true;
                            a.hero_damage = Some((w.high_damage, None));
                            1
                        } else if ended(33) {
                            2
                        } else {
                            0
                        }
                    }
                    1 => {
                        a.anim = Some(34);
                        if ended(34) {
                            self.phase = 0;
                            self.state = State::Idle;
                            return;
                        }
                        1
                    }
                    2 => {
                        a.anim = Some(35);
                        if ended(35) {
                            self.stun = ticks(w.high_stun);
                            3
                        } else {
                            2
                        }
                    }
                    3 => {
                        a.anim = Some(36);
                        if self.stun_tick(v, &[RIGHT_ARM, LEFT_ARM], 46, false, a) {
                            4
                        } else if self.parts[RIGHT_ARM] <= 0 {
                            5
                        } else {
                            3
                        }
                    }
                    4 | 5 => {
                        let n = if step == 4 { 37 } else { 38 };
                        a.anim = Some(n);
                        if ended(n) {
                            self.phase = 0;
                            if step == 5 {
                                self.en_garde = true;
                            }
                            self.state = State::Idle;
                            return;
                        }
                        step
                    }
                    _ => 0,
                };
                self.state = State::HighSwing(next);
            }
            State::FloorStrike(step) => {
                let next = match step {
                    0 => {
                        a.anim = Some(20);
                        if window(20, 0) && !self.hit {
                            self.hit = true;
                            a.hero_damage = Some((w.floor_damage, None));
                            1
                        } else if ended(20) {
                            2
                        } else {
                            0
                        }
                    }
                    1 => {
                        a.anim = Some(21);
                        if ended(21) {
                            self.phase = 0;
                            self.state = State::Idle;
                            return;
                        }
                        1
                    }
                    2 => {
                        a.anim = Some(22);
                        if ended(22) {
                            self.stun = ticks(w.floor_stun);
                            3
                        } else {
                            2
                        }
                    }
                    3 => {
                        a.anim = Some(23);
                        if self.stun_tick(v, &[BACK], 44, true, a) {
                            4
                        } else if self.parts[BACK] <= 0 {
                            5
                        } else {
                            3
                        }
                    }
                    4 | 5 => {
                        let n = if step == 4 { 24 } else { 38 };
                        a.anim = Some(n);
                        if ended(n) {
                            self.phase = 0;
                            if step == 5 {
                                self.en_garde = true;
                            }
                            self.state = State::Idle;
                            return;
                        }
                        step
                    }
                    _ => 0,
                };
                self.state = State::FloorStrike(next);
            }
            State::Spin(step) => {
                let next = match step {
                    0 => {
                        a.anim = Some(39);
                        if ended(39) {
                            self.sword.sub = 0;
                            self.sword.returned = false;
                            1
                        } else {
                            0
                        }
                    }
                    1 => {
                        a.anim = Some(40);
                        if self.sword.returned { 2 } else { 1 }
                    }
                    2 => {
                        a.anim = Some(41);
                        if ended(41) {
                            if self.parts[..5].iter().any(|&x| x > 0) {
                                self.phase = 0;
                                self.state = State::Idle;
                                return;
                            }
                            self.stun = ticks(w.spin_stun);
                            3
                        } else {
                            2
                        }
                    }
                    3 => {
                        a.anim = Some(42);
                        if self.stun_tick(v, &[NECK], 47, true, a) {
                            self.phase = 0;
                            self.state = State::Idle;
                            return;
                        } else if self.parts[NECK] <= 0 {
                            4
                        } else {
                            3
                        }
                    }
                    _ => {
                        a.anim = Some(43);
                        if ended(43) {
                            self.phase = 0;
                            self.state = State::Idle;
                            return;
                        }
                        4
                    }
                };
                self.state = State::Spin(next);
                if step != 0 {
                    a.sword = self.sword_tick(v, w, a);
                }
            }
            State::TurnToward => {
                a.anim = Some(7);
                a.face = Some(0.05);
            }
            State::TurnAway => {
                a.anim = Some(6);
                a.face = Some(-0.05);
            }
            State::Approach => a.travel = Some((0, v.hero)),
            State::Orbit => {
                self.orbit_timer -= 1;
                if self.orbit_timer <= 0 {
                    self.orbit_timer = 30;
                    self.orbit_side ^= 1;
                }
                let s = [-to_hero[2], 0.0, to_hero[0]];
                let k = if self.orbit_side == 0 { 3.0 } else { -3.0 };
                a.travel = Some((5 + self.orbit_side, [v.position[0] + k * s[0], v.position[1], v.position[2] + k * s[2]]));
            }
        }
    }

    /// One sub-state of the flying sword (it runs in the spin after its spin-up).
    fn sword_tick(&mut self, v: &View, w: &Words, a: &mut Act) -> Option<Vec3> {
        let s = &mut self.sword;
        let step = w.sword_speed / 60.0;
        match s.sub {
            0 => {
                s.position = v.hand;
                s.sub = 1;
                s.lift = 1.0;
                Some(s.position)
            }
            1 => {
                // The launch: the shared resets, the sword turning upward until its vertical
                // direction component falls below the threshold (taken as a 0.05 drop per tick).
                s.count = 0;
                s.step = w.spin_step;
                s.rate = 0.0;
                s.path = 0.0;
                s.position[1] += 0.2;
                s.lift -= 0.05;
                if s.lift < w.launch_threshold {
                    s.sub = 2;
                    s.angle = 0.0;
                    s.height = s.position[1];
                }
                Some(s.position)
            }
            2 => {
                s.rate += w.path_rate;
                s.step += w.spin_accel;
                if s.bounce_up {
                    s.bounce += w.bounce_speed;
                    if s.bounce > w.bounce_limit {
                        s.bounce_up = false;
                    }
                } else {
                    s.bounce -= w.bounce_speed;
                    if s.bounce < -w.bounce_limit {
                        s.bounce_up = true;
                    }
                }
                s.angle += s.step;
                s.path += s.rate;
                if s.count > 0 {
                    let target = (v.ground.unwrap_or(v.position[1]) + 2.0).max(w.min_height);
                    s.height = toward([0.0, s.height, 0.0], [0.0, target, 0.0], 0.05).0[1];
                }
                let f = v.facing;
                let r = [-f[2], 0.0, f[0]];
                let dir = [s.angle.cos() * r[0] + s.angle.sin() * f[0], s.bounce, s.angle.cos() * r[2] + s.angle.sin() * f[2]];
                let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-6);
                s.position = [v.position[0] + 3.0 * dir[0] / l, if s.count > 0 { s.height } else { v.position[1] + 3.0 * dir[1] / l + 2.0 }, v.position[2] + 3.0 * dir[2] / l];
                let target = (v.ground.unwrap_or(v.position[1]) + 2.0).max(w.min_height);
                let on_target = (s.height - target).abs() <= 0.05;
                if s.angle >= std::f32::consts::TAU {
                    if (s.count < w.revolutions || !on_target) && s.count < 14 {
                        s.count += 1;
                        s.angle -= std::f32::consts::TAU;
                        s.latch = false;
                        return Some(s.position);
                    }
                    if s.count >= w.revolutions {
                        s.sub = 3;
                    }
                }
                if !s.latch && (v.sword_hit)(s.position) {
                    s.latch = true;
                    a.hero_damage = Some((w.sword_damage, Some(knock(v.facing, w.knock))));
                }
                Some(s.position)
            }
            3 => {
                s.position = toward(s.position, [v.hero[0], v.hero[1] + 1.5, v.hero[2]], step).0;
                if (v.sword_hit)(s.position) {
                    if !s.latch {
                        a.hero_damage = Some((w.sword_damage, None));
                    }
                    s.latch = true;
                    s.sub = 4;
                }
                Some(s.position)
            }
            4 => {
                let (p, gap) = toward(s.position, v.return_point, step);
                if gap < step {
                    s.returned = true;
                    s.sub = 5;
                    s.position = v.hand;
                } else {
                    s.position = p;
                }
                Some(s.position)
            }
            _ => {
                s.position = v.hand;
                None
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
        for (at, x) in [(0x002D_A3E8, 25u32), (0x002D_A3EC, 25), (0x002D_A3F0, 25), (0x002D_A3F4, 50), (0x002D_A3F8, 20), (0x002D_A3FC, 4), (0x002D_A400, 6), (0x002D_A404, 40), (0x002D_A408, 5), (0x002D_A40C, 70), (0x002D_A410, 20), (0x002D_A414, 10), (0x002D_A418, 20), (0x002D_A41C, 11), (0x002D_A420, 20), (0x002D_A424, 30), (0x002D_A42C, 20), (0x002D_A434, 11), (0x002D_A438, 20), (0x002D_A440, 20), (0x002D_A44C, 90), (0x002D_A450, 10), (0x002D_A458, 10), (0x002D_A45C, 15), (0x002D_A460, 75), (0x002D_A480, 30), (0x002D_A4B8, 9)] {
            put(at, x);
        }
        for (k, hp) in [50u32, 50, 50, 50, 50, 100].into_iter().enumerate() {
            put(0x002D_A468 + 4 * k as u32, hp);
        }
        for (at, x) in [(0x002D_A430, 10.0f32), (0x002D_A43C, 10.0), (0x002D_A444, 10.0), (0x002D_A448, 10.0), (0x002D_A484, 20.0), (0x002D_A488, 0.4), (0x002D_A48C, 0.05), (0x002D_A494, 0.00075), (0x002D_A4A4, 0.002), (0x002D_A4BC, 0.04), (0x002D_A4C0, 0.5), (0x002D_A4C4, 4.0)] {
            put(at, x.to_bits());
        }
        Words::read(&b).unwrap()
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, part: &'a dyn Fn(usize) -> bool, sword: &'a dyn Fn(Vec3) -> bool, d: f32) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, d],
            hero_facing: [0.0, 0.0, -1.0],
            hero_index: 0,
            hero_attack: false,
            hero_guard: false,
            hero_strike: false,
            shots: false,
            shot_overlap: false,
            weapons: [false; 5],
            frame_bits: 0,
            clip: (0, false),
            hero_on_mesh: true,
            hero_on_mesh_full: true,
            contact: false,
            edge: false,
            part_hit: part,
            part_shot: &|_| false,
            hero_damage: 12,
            second_measure: 5.0,
            ground: Some(0.0),
            hand: [0.0, 2.0, 1.0],
            return_point: [0.0, 2.0, 1.0],
            sword_hit: sword,
            level,
        }
    }

    #[test]
    fn display_defeat_and_tests() {
        assert_eq!(display_hp(&[50, 10, 50, 0, 50, 100]), 270);
        let w = words();
        let k = Knight { parts: [0, -5, 0, 0, 0, 0], ..Knight::new(&w) };
        assert!(k.defeated());
        let k = Knight { parts: [0, -5, 0, 0, 0, 1], ..Knight::new(&w) };
        assert!(!k.defeated());
        // The charge references: (0.8, 0.3) +20 passes; (0.2, 0.9) +60 fails; (-0.6, 0.1) -20.
        let r = [20.0, -20.0, 60.0, -60.0];
        assert!(charge_passes(&r, 0.8, 0.3, 30.0) && !charge_passes(&r, 0.2, 0.9, 30.0) && charge_passes(&r, -0.6, 0.1, 30.0));
        // The knock-back: unit(9 d + 12 up) x 20.
        let v = knock([1.0, 0.0, 0.0], 20.0);
        assert!((v[0] - 12.0).abs() < 1e-4 && (v[1] - 16.0).abs() < 1e-4);
        // The glow: lit from tick 1 shows from tick 2, r and b step 0.01.
        let mut g = Glow::default();
        g.step(true);
        assert!((g.colour[0] - 0.01).abs() < 1e-6 && g.colour[1] == 0.0);
        g.step(false);
        assert_eq!(g, Glow::default());
    }

    #[test]
    fn selection_flags_and_defeat() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        let miss = |_: usize| false;
        let nosword = |_: Vec3| false;
        // The first ready tick: the counter flag is cleared; a slow swing at 9.9 in phase 0.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.tick(&view(&no, &miss, &nosword, 9.9), &w, &mut rng);
        assert_eq!(k.state, State::SlowSwing { blocked: false });
        let mut k = Knight::new(&w);
        k.counter = false;
        k.tick(&view(&no, &miss, &nosword, 10.0), &w, &mut rng);
        assert!(!matches!(k.state, State::SlowSwing { .. }));
        // A blocked slow swing goes to recovery 16 with no damage, then the angry start.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.state = State::SlowSwing { blocked: false };
        let mut v = view(&no, &miss, &nosword, 5.0);
        v.clip = (15, false);
        v.frame_bits = 1;
        v.weapons[0] = true;
        v.hero_guard = true;
        let a = k.tick(&v, &w, &mut rng);
        assert_eq!((a.hero_damage, k.state), (None, State::SlowSwing { blocked: true }));
        let mut v = view(&no, &miss, &nosword, 5.0);
        v.clip = (16, true);
        k.tick(&v, &w, &mut rng);
        k.tick(&view(&no, &miss, &nosword, 20.0), &w, &mut rng);
        assert_eq!((k.phase, k.state), (1, State::AngryStart));
        // Phase 2 order: a knee up and the charge test passing: the charge.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.phase = 2;
        let mut v = view(&no, &miss, &nosword, 20.0);
        v.hero = [16.0, 0.0, 6.0];
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Charge(0));
        // Only the neck left: the sword spin.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.phase = 2;
        k.parts = [0, 0, 0, 0, 0, 50];
        k.tick(&view(&no, &miss, &nosword, 20.0), &w, &mut rng);
        assert_eq!(k.state, State::Spin(0));
        // The defeat: state 19 and level state 0 once.
        let mut k = Knight::new(&w);
        k.parts = [0; 6];
        assert_eq!(k.tick(&view(&no, &miss, &nosword, 20.0), &w, &mut rng).set_level, vec![0]);
        assert!(k.tick(&view(&no, &miss, &nosword, 20.0), &w, &mut rng).set_level.is_empty());
        assert_eq!(k.state, State::Defeated);
    }

    #[test]
    fn charge_stun_and_parts() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let no = |_: u16| false;
        let miss = |_: usize| false;
        let nosword = |_: Vec3| false;
        // The run: contact without edge: the impact (20 at any distance); edge: the stun start.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.state = State::Charge(1);
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.contact = true;
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Charge(2));
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.clip = (28, false);
        v.frame_bits = 0x4;
        let a = k.tick(&v, &w, &mut rng);
        assert_eq!(a.hero_damage.map(|h| h.0), Some(20));
        let mut k = Knight::new(&w);
        k.state = State::Charge(1);
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.edge = true;
        v.contact = true;
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Charge(3));
        // The stun: 600 exposed ticks; a hit on the right knee takes the damage (floored at 0).
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.clip = (29, true);
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Charge(4));
        let right = |p: usize| p == RIGHT_KNEE;
        // The hero in front but not attacking fails the knee gate: blocked, nothing lost.
        let a = k.tick(&view(&no, &right, &nosword, 3.0), &w, &mut rng);
        assert_eq!((k.parts[RIGHT_KNEE], a.part_effect, k.exposed), (50, None, false));
        let mut v = view(&no, &right, &nosword, 3.0);
        v.hero_attack = true;
        let a = k.tick(&v, &w, &mut rng);
        assert_eq!((k.parts[RIGHT_KNEE], a.part_effect, a.anim, k.exposed), (38, Some(RIGHT_KNEE), Some(45), true));
        // Attacking from behind fails the knee gate too.
        let mut v = view(&no, &right, &nosword, -3.0);
        v.hero_attack = true;
        k.tick(&v, &w, &mut rng);
        assert!(!k.exposed && k.parts[RIGHT_KNEE] == 38);
        // 597 more ticks expose; the 601st ends the stun with no test.
        for _ in 0..597 {
            k.tick(&view(&no, &miss, &nosword, 30.0), &w, &mut rng);
        }
        assert_eq!(k.state, State::Charge(4));
        k.tick(&view(&no, &right, &nosword, 30.0), &w, &mut rng);
        assert_eq!((k.state, k.parts[RIGHT_KNEE]), (State::Charge(5), 38));
        // The weak-point floor: 5 HP taking 20 leaves 0 (the hero attacking from behind).
        let mut k = Knight::new(&w);
        k.state = State::FloorStrike(3);
        k.stun = 600;
        k.parts[BACK] = 5;
        let back = |p: usize| p == BACK;
        let mut v = view(&no, &back, &nosword, -3.0);
        v.hero_attack = true;
        v.hero_damage = 20;
        k.tick(&v, &w, &mut rng);
        assert_eq!((k.parts[BACK], k.state), (0, State::FloorStrike(5)));
    }

    #[test]
    fn parry_deflect_and_movement() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        let miss = |_: usize| false;
        let nosword = |_: Vec3| false;
        // Parry: accumulator 0 never passes and grows by 50.
        let mut k = Knight::new(&w);
        k.counter = false;
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.hero_attack = true;
        k.tick(&v, &w, &mut rng);
        assert_eq!((k.parry, k.latch), (50, true));
        // Deflect (Roberta): an overlapping first slot with accumulator 0 fails and grows by 20.
        let mut k = Knight::new(&w);
        k.counter = false;
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.hero_index = 1;
        v.shots = true;
        v.shot_overlap = true;
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.deflect, 20);
        // Turn: the hero 100 degrees off on the right: turn toward; 80 degrees at 12: approach.
        let mut k = Knight::new(&w);
        k.counter = false;
        let mut v = view(&no, &miss, &nosword, 12.0);
        let a = 100f32.to_radians();
        v.hero = [-12.0 * a.sin(), 0.0, 12.0 * a.cos()];
        k.tick(&v, &w, &mut rng);
        assert!(matches!(k.state, State::TurnToward | State::TurnAway));
        let mut k = Knight::new(&w);
        k.counter = false;
        let mut v = view(&no, &miss, &nosword, 12.0);
        let a = 80f32.to_radians();
        v.hero = [12.0 * a.sin(), 0.0, 12.0 * a.cos()];
        v.second_measure = 50.0;
        k.phase = 1;
        k.tick(&v, &w, &mut rng);
        assert_eq!(k.state, State::Approach);
    }

    #[test]
    fn flying_sword() {
        let w = words();
        let mut rng = Rng::seeded(4);
        let no = |_: u16| false;
        let miss = |_: usize| false;
        let nosword = |_: Vec3| false;
        // The spin-up's end resets the sword; the wait runs it through attach, launch and the
        // orbit; homing starts at the 10th full turn (on target), then the return sets the flag.
        let mut k = Knight::new(&w);
        k.counter = false;
        k.state = State::Spin(0);
        let mut v = view(&no, &miss, &nosword, 30.0);
        v.clip = (39, true);
        k.tick(&v, &w, &mut rng);
        assert_eq!((k.state, k.sword.sub), (State::Spin(1), 0));
        let mut homing = None;
        for t in 0..2000 {
            k.tick(&view(&no, &miss, &nosword, 30.0), &w, &mut rng);
            if k.sword.sub == 3 && homing.is_none() {
                homing = Some((t, k.sword.count));
            }
            if k.sword.returned {
                break;
            }
        }
        let (_, count) = homing.expect("the sword homes");
        assert!((9..=14).contains(&count));
        // The homing sword never touches the hero here, so it keeps homing (no return).
        assert!(!k.sword.returned);
        // A homing sword that touches the hero hits for 15 and returns; within 1.25 of the
        // return point it sets the flag.
        let touch = |_: Vec3| true;
        let a = k.tick(&view(&no, &miss, &touch, 30.0), &w, &mut rng);
        assert_eq!((a.hero_damage.map(|h| h.0), k.sword.sub), (Some(15), 4));
        for _ in 0..200 {
            k.tick(&view(&no, &miss, &nosword, 30.0), &w, &mut rng);
            if k.sword.returned {
                break;
            }
        }
        assert!(k.sword.returned);
        k.tick(&view(&no, &miss, &nosword, 30.0), &w, &mut rng);
        assert_eq!(k.state, State::Spin(2));
    }
}
