//! The Gelert Assassin boss (SPEC-0013; the glade fight of world pack 985, its clones and the
//! 753 ambush copy): the 97 tuning words and the other data constants, the per-tick order
//! (timers, the script gate, the HP bands with their script events, the missile and melee
//! reactions, the attack choice, the strafe and approach rules, the shadow-slip gate, the hit
//! reaction, the state handler and the elapsed-time counter), the melee attacks with their
//! windows, block test and once-per-attack rule, the dagger throw, the shadow slip with its
//! chooser and landing search, the shadow attacks (just appear, back-stab, shadow dagger, light
//! arc with its balls and missile wall, healing, the mirror attack and the rainbow dash) and the
//! clone states.
//!
//! The host measures what needs the world: the weapon touch, the melee reaction's facing and
//! reach tests, the missile threat and the hits on the boss, the navmesh blocked flag, the
//! walkable test of a landing point and the rainbow locators. The dormant settle rule (4.14) and
//! the unreachable shadow thrust-lunge are left out.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The 97 tuning words from 0x2DABD8.
pub const BASE: u32 = 0x002D_ABD8;
pub const WORDS: usize = 97;
/// The other data constants read (SPEC-0013 Other constants): their addresses.
pub const HIT_LIMIT_AT: u32 = 0x002D_AD5C;
pub const HERO_RATIO_AT: u32 = 0x002D_AD60;
pub const DAGGER_SPEED_AT: u32 = 0x002D_AD70;
pub const DAGGER_DAMAGE_AT: u32 = 0x002D_AD78;
pub const LUNGE_DAMAGE_AT: u32 = 0x002D_AD80;
pub const SWING_DAMAGE_AT: u32 = 0x002D_AD84;
pub const ARC_LOOP_AT: u32 = 0x002D_AD88;
pub const ARC_HEIGHT_AT: u32 = 0x002D_AD8C;
pub const RAINBOW_DAMAGE_AT: u32 = 0x002D_AD90;

/// The locomotion slot values (12, 12, 12, -1, -1, 15, 14): approach entry 12, strafe right
/// (mode 5) entry 14, strafe left (mode 6) entry 15.
pub const SLOTS: [i32; 7] = [12, 12, 12, -1, -1, 15, 14];

/// The tuning words and the data constants.
#[derive(Clone, Debug, PartialEq)]
pub struct Words {
    pub tuning: Vec<i32>,
    pub hit_limit: i32,
    pub hero_ratio: f32,
    pub dagger_speed: f32,
    pub dagger_damage: i32,
    pub lunge_damage: i32,
    pub swing_damage: i32,
    pub arc_loop: i32,
    pub arc_height: f32,
    pub rainbow_damage: i32,
}

impl Words {
    /// The tuning words (97, from BASE) and a reader of single data words by address.
    pub fn read(tuning: &[u32], word: &dyn Fn(u32) -> Option<u32>) -> Option<Self> {
        if tuning.len() < WORDS {
            return None;
        }
        Some(Self {
            tuning: tuning[..WORDS].iter().map(|&w| w as i32).collect(),
            hit_limit: word(HIT_LIMIT_AT)? as i32,
            hero_ratio: f32::from_bits(word(HERO_RATIO_AT)?),
            dagger_speed: f32::from_bits(word(DAGGER_SPEED_AT)?),
            dagger_damage: word(DAGGER_DAMAGE_AT)? as i32,
            lunge_damage: word(LUNGE_DAMAGE_AT)? as i32,
            swing_damage: word(SWING_DAMAGE_AT)? as i32,
            arc_loop: word(ARC_LOOP_AT)? as i32,
            arc_height: f32::from_bits(word(ARC_HEIGHT_AT)?),
            rainbow_damage: word(RAINBOW_DAMAGE_AT)? as i32,
        })
    }

    /// The tuning word at a data address (0 outside the block).
    pub fn at(&self, addr: u32) -> i32 {
        addr.checked_sub(BASE).map(|o| (o / 4) as usize).and_then(|i| self.tuning.get(i)).copied().unwrap_or(0)
    }

    /// A pattern-indexed (min, max) pair from a group address; (0, 1) outside 1 to 4 (the
    /// shadow timer's out-of-range rule).
    fn pair(&self, group: u32, p: u8) -> (i32, i32) {
        if !(1..=4).contains(&p) {
            return (0, 1);
        }
        let a = group + 8 * (p as u32 - 1);
        (self.at(a), self.at(a + 4))
    }

    /// A pattern-indexed single value; 0 outside 1 to 4.
    fn per(&self, group: u32, p: u8) -> i32 {
        if !(1..=4).contains(&p) {
            return 0;
        }
        self.at(group + 4 * (p as u32 - 1))
    }
}

/// The boss's states (the spec's names).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Approach,
    StrafeLeft,
    StrafeRight,
    BackUp,
    Flinch,
    /// The superior-sneer, angry and injured stances, waiting for script bit 2, 4 or 6.
    Stance(u16),
    Wait,
    EvilLaugh,
    DodgeMissile,
    Deflect,
    DodgeMelee,
    JumpThrust,
    SwordSwing,
    ThrustLunge,
    SwordBlocked,
    DaggerThrow,
    /// Shadow slip: phase 0 vanish, 1 choose, 2 reappear (with the chosen attack).
    Slip(u8, Shadow),
    JustAppear,
    BackStab,
    ShadowDagger,
    /// Light arc phases: 0 wait (36), 1 windup (37), 2 loop (38), 3 end (39).
    LightArc(u8),
    /// Healing phases 0 to 3.
    Heal(u8),
    /// Mirror attack: 0 animation 41, 1 animation 42.
    Mirror(u8),
    /// Rainbow dash: the point index (0 to 7), then 8 the end animation.
    Rainbow(u8),
    StartInvisible,
    Death,
    /// A clone waiting hidden (base state 42), or woken and running its mirror handler.
    CloneWait,
    CloneWake,
}

/// The shadow attacks the chooser picks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shadow {
    None,
    JustAppear,
    BackStab,
    Dagger,
    LightArc,
    Heal,
    Mirror,
    Rainbow,
}

impl State {
    /// Decision-ready: idle or strafing (4.4).
    pub fn ready(self) -> bool {
        matches!(self, State::Idle | State::Approach | State::StrafeLeft | State::StrafeRight | State::BackUp)
    }

    /// The states that allow reactions (4.5).
    fn reacts(self) -> bool {
        matches!(self, State::Idle | State::Approach | State::StrafeLeft | State::StrafeRight | State::BackUp | State::Flinch | State::Stance(_) | State::DaggerThrow)
    }

    /// The hittable states (4.13).
    pub fn hittable(self) -> bool {
        matches!(
            self,
            State::Idle
                | State::EvilLaugh
                | State::DaggerThrow
                | State::ThrustLunge
                | State::SwordBlocked
                | State::ShadowDagger
                | State::Mirror(_)
                | State::JustAppear
                | State::Approach
                | State::StrafeLeft
                | State::StrafeRight
                | State::BackUp
                | State::Heal(0 | 1 | 3)
                | State::LightArc(0 | 2 | 3)
                | State::Rainbow(_)
        )
    }
}

/// What the boss sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    pub hero_guard: bool,
    /// The hero is attacking (the melee-reaction latch clears when not).
    pub hero_attacking: bool,
    pub hero_anim: u32,
    pub hero_hp_ratio: f32,
    pub hero_radius: f32,
    pub hero_height: f32,
    pub hp: i32,
    pub max_hp: i32,
    /// The playing clip, its progress and whether it ended.
    pub clip: (u32, f32, bool),
    /// The clip's integer frame and event bits.
    pub frame: u32,
    pub bits: u32,
    /// Weapon 1 touches the hero's bone capsules this tick.
    pub weapon_touch: bool,
    /// The melee reaction's facing and reach tests pass.
    pub melee_reach: bool,
    /// A new hero missile's 180-tick path crosses the boss; a missile within 10 units.
    pub missile_threat: bool,
    pub missile_near: bool,
    /// Hits on the boss this tick: a melee hit, and the number of missile slots that hit.
    pub hit_melee: bool,
    pub hit_missiles: u32,
    /// The hero hit damage of this tick's hit (melee at scale 1.0, else the last overlapping
    /// shot slot's damage query), before the time factor.
    pub hit_damage: i32,
    /// The navmesh blocked flag (SPEC-0056).
    pub nav_blocked: bool,
    pub level: &'a dyn Fn(u16) -> bool,
    pub walkable: &'a dyn Fn(Vec3) -> bool,
    /// The rainbow locators 200 to 207.
    pub rainbow: [Option<Vec3>; 8],
    /// Live clones (for the shadow-slip re-arm rule).
    pub clones_alive: usize,
}

/// A dagger launch: the aim point, the speed and damage, and the lifetime in ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dagger {
    pub aim: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
}

/// What the boss asks for this tick.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Act {
    pub anim: Option<u32>,
    pub rate: f32,
    /// Turn toward the hero at this weight.
    pub face: Option<f32>,
    /// A locomotion mode toward a target (0 approach, 5 and 6 sideways).
    pub travel: Option<(u8, Vec3)>,
    /// A straight horizontal move this tick (the rainbow dash).
    pub step: Option<Vec3>,
    /// Placed at a point (the reappear).
    pub place: Option<Vec3>,
    /// An unreduced hit on the hero.
    pub hero_damage: Option<i32>,
    pub daggers: Vec<Dagger>,
    pub set_level: Vec<u16>,
    /// One-shot effects at the boss (222 the flinch flash).
    pub effects: Vec<u32>,
    /// The heal aura 428 started (true) or stopped (false).
    pub aura: Option<bool>,
    /// HP change from healing (may be negative).
    pub heal: i32,
    /// HP taken by a hit.
    pub hp_loss: i32,
    pub wake_clones: bool,
    /// The light-arc balls this tick and whether the missile wall runs.
    pub arc: Option<(Vec3, Vec3)>,
    /// The fade alpha while start-invisible plays (0 to 128).
    pub fade: Option<u8>,
    /// A clone was hit: back to the hidden wait with effect 559.
    pub clone_reset: bool,
}

/// The boss record.
#[derive(Clone, Debug, PartialEq)]
pub struct Gelert {
    pub state: State,
    pub pattern: u8,
    pub clone: bool,
    pub scripted: bool,
    hits: i32,
    t_slip: i32,
    t_attack: i32,
    t_backup: i32,
    elapsed: i32,
    once: bool,
    hit_sent: bool,
    pub invisible: bool,
    slip_latch: bool,
    melee_latch: bool,
    laugh_pending: bool,
    last_frame: Option<u32>,
    started: bool,
    heal_count: i32,
    arc_dir: Vec3,
    arc_start: Vec3,
    arc_ticks: i32,
    arc_hit: bool,
    order: [usize; 8],
    rainbow_hit: bool,
    pub clone_mode: bool,
    prev_clip: u32,
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn hdir(from: Vec3, to: Vec3) -> Vec3 {
    let d = hdist(from, to).max(1e-6);
    [(to[0] - from[0]) / d, 0.0, (to[2] - from[2]) / d]
}

fn roll(rng: &mut Rng) -> i32 {
    rng.range(0, 100)
}

/// The timer value: trunc(rand(min, max) x 60).
fn timer(rng: &mut Rng, (lo, hi): (i32, i32)) -> i32 {
    (rng.frange(lo as f32, hi as f32) * 60.0) as i32
}

impl Gelert {
    /// The record at creation: pattern 1, the pattern-1 timers, the back-up cooldown 60, the
    /// script-control flag copied from flag 258. A clone starts hidden.
    pub fn new(w: &Words, scripted: bool, clone: bool, rng: &mut Rng) -> Self {
        let t_slip = timer(rng, w.pair(0x002D_ABD8, 1));
        let t_attack = timer(rng, w.pair(0x002D_AC18, 1));
        Self {
            state: if clone { State::CloneWait } else { State::Idle },
            pattern: 1,
            clone,
            scripted,
            hits: 0,
            t_slip,
            t_attack,
            t_backup: 60,
            elapsed: 0,
            once: false,
            hit_sent: false,
            invisible: false,
            slip_latch: false,
            melee_latch: false,
            laugh_pending: false,
            last_frame: None,
            started: false,
            heal_count: 0,
            arc_dir: [0.0, 0.0, 1.0],
            arc_start: [0.0; 3],
            arc_ticks: 0,
            arc_hit: false,
            order: [0, 1, 2, 3, 4, 5, 6, 7],
            rainbow_hit: false,
            clone_mode: false,
            prev_clip: u32::MAX,
        }
    }

    /// The pattern of an HP fraction (only ever rising).
    pub fn band(f: f32) -> u8 {
        if f >= 0.75 {
            1
        } else if f >= 0.5 {
            2
        } else if f >= 0.25 {
            3
        } else {
            4
        }
    }

    fn enter(&mut self, s: State) {
        self.state = s;
        self.started = false;
        self.once = false;
        self.last_frame = None;
    }

    fn rearm_attack(&mut self, w: &Words, rng: &mut Rng) {
        self.t_attack = timer(rng, w.pair(0x002D_AC18, self.pattern));
    }

    fn rearm_slip(&mut self, w: &Words, v: &View, rng: &mut Rng) {
        if self.clone_mode && v.clones_alive > 0 {
            return;
        }
        self.clone_mode = false;
        self.t_slip = timer(rng, w.pair(0x002D_ABD8, self.pattern));
    }

    /// The clone's wake (4.10): from the hidden wait to its mirror handler.
    pub fn wake(&mut self) {
        if self.state == State::CloneWait {
            self.enter(State::CloneWake);
        }
    }

    /// G7: the damage a flinching hit takes: trunc(hero hit damage x (1 + 4 x elapsed / 7200)),
    /// the factor a float, unclamped (2.0 at 15 s, 5.0 at 60 s since the last reset).
    pub fn taken(&self, damage: i32) -> i32 {
        (damage.max(0) as f32 * (1.0 + 4.0 * self.elapsed as f32 / 7200.0)) as i32
    }

    /// One tick (4.1).
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act { rate: 1.0, ..Act::default() };
        // The clip that played on the previous tick; a new request this tick restarts it.
        let clip_now = v.clip.0;
        let ended = |n: u32| v.clip.0 == n && v.clip.2;
        // 1. Timers.
        self.t_slip = (self.t_slip - 1).max(0);
        self.t_attack = (self.t_attack - 1).max(0);
        self.t_backup = (self.t_backup - 1).max(0);
        if v.nav_blocked {
            self.slip_latch = true;
        }
        if self.state == State::Death {
            a.anim = Some(clip_now);
            return a;
        }
        // Clones: the hidden wait and the wake.
        if self.state == State::CloneWait {
            return a;
        }
        // 3. The script gate.
        if self.scripted {
            if !(v.level)(0) {
                if self.state != State::Wait {
                    self.enter(State::Wait);
                }
                a.anim = Some(24);
                return a;
            }
            if self.state == State::Wait {
                self.enter(State::Idle);
            }
        } else if !self.clone && (self.hits > w.hit_limit || v.hero_hp_ratio < w.hero_ratio) {
            a.set_level.push(0);
        }
        // The evil-laugh roll.
        if std::mem::take(&mut self.laugh_pending) && roll(rng) < w.at(0x002D_AC38) {
            self.enter(State::EvilLaugh);
        }
        // The HP band check (decision-ready only).
        if self.state.ready() && !self.clone {
            if v.hp <= 0 {
                self.enter(State::Death);
                a.set_level.push(7);
                a.anim = Some(clip_now);
                return a;
            }
            let p = Self::band(v.hp as f32 / v.max_hp.max(1) as f32).max(self.pattern);
            if p > self.pattern {
                self.pattern = p;
                self.elapsed = 0;
                let (event, bit) = match p {
                    2 => (1, 2),
                    3 => (3, 4),
                    _ => (5, 6),
                };
                a.set_level.push(event);
                self.enter(State::Stance(bit));
            }
        }
        if let State::Stance(bit) = self.state {
            if (v.level)(bit) {
                self.enter(State::Idle);
                self.elapsed = 0;
            }
        }
        let d = hdist(v.position, v.hero);
        let p = self.pattern;
        // 4. Reactions.
        if self.state.reacts() {
            if v.missile_threat {
                let dodge = if self.clone { w.at(0x002D_AC4C) } else { w.per(0x002D_AC3C, p) };
                if roll(rng) < dodge {
                    self.enter(State::DodgeMissile);
                    a.anim = Some(if rng.range(0, 1) == 1 { 19 } else { 18 });
                } else if roll(rng) < w.per(0x002D_AC50, p) {
                    self.enter(State::Deflect);
                }
            }
            if !v.hero_attacking {
                self.melee_latch = false;
            } else if v.melee_reach && !self.melee_latch && self.state.reacts() {
                self.melee_latch = true;
                let dodge = if self.clone { w.at(0x002D_AC70) } else { w.per(0x002D_AC60, p) };
                let jump = if self.clone { w.at(0x002D_AC84) } else { w.per(0x002D_AC74, p) };
                if roll(rng) < dodge {
                    self.enter(State::DodgeMelee);
                } else if roll(rng) < jump {
                    self.enter(State::JumpThrust);
                }
            }
        } else {
            self.melee_latch = false;
        }
        // 5. The attack choice.
        if self.state.ready() && self.t_attack == 0 {
            if d > w.at(0x002D_AC8C) as f32 && d < w.at(0x002D_AC88) as f32 && !self.invisible && !self.clone && roll(rng) < w.at(0x002D_AC90) {
                self.enter(State::DaggerThrow);
            } else if d < w.at(0x002D_AC94) as f32 {
                if d <= 10.0 {
                    let s = if roll(rng) >= w.at(0x002D_AC98) { State::ThrustLunge } else { State::SwordSwing };
                    self.enter(s);
                } else {
                    self.enter(State::ThrustLunge);
                }
            }
        }
        // 6. Strafe, approach and the shadow-slip gate.
        if self.state.ready() {
            let outer = w.at(0x002D_AC9C) as f32;
            if d > outer {
                if self.state != State::Approach {
                    self.state = State::Approach;
                }
            } else if d >= w.at(0x002D_ACA8) as f32 {
                if !matches!(self.state, State::BackUp | State::StrafeLeft | State::StrafeRight) {
                    self.state = if rng.range(0, 1) == 0 { State::StrafeRight } else { State::StrafeLeft };
                }
            } else if self.state != State::BackUp {
                if self.t_backup == 0 {
                    self.t_backup = 60;
                    if roll(rng) < w.at(0x002D_ACAC) {
                        self.enter(State::BackUp);
                    }
                } else if matches!(self.state, State::Idle | State::Approach) {
                    self.state = if rng.range(0, 1) == 0 { State::StrafeRight } else { State::StrafeLeft };
                }
            }
            if !self.clone && (self.slip_latch || (self.t_slip == 0 && self.scripted && !self.invisible && !self.clone_mode)) {
                self.enter(State::Slip(0, Shadow::None));
            }
        }
        // 7. Hits on the boss and the reaction.
        if (v.hit_melee || v.hit_missiles > 0) && self.state.hittable() {
            if self.clone {
                self.enter(State::CloneWait);
                a.clone_reset = true;
                return a;
            }
            if !self.scripted {
                self.hits += v.hit_missiles as i32;
            }
            match self.state {
                State::SwordBlocked if roll(rng) < 50 => self.enter(State::Slip(0, Shadow::None)),
                State::Heal(0 | 1 | 3) => {
                    a.aura = Some(false);
                    self.enter(State::Heal(2));
                }
                State::LightArc(_) => {
                    a.hp_loss = self.taken(v.hit_damage);
                    self.enter(State::Flinch);
                    a.anim = Some(if rng.range(0, 1) == 1 { 17 } else { 16 });
                    a.effects.push(222);
                }
                _ => {
                    a.hp_loss = self.taken(v.hit_damage);
                    self.enter(State::Flinch);
                    a.anim = Some(if rng.range(0, 1) == 1 { 17 } else { 16 });
                    a.effects.push(222);
                    if !self.scripted {
                        self.hits += 1;
                    }
                }
            }
        }
        // 8. The state handler.
        self.handle(v, w, rng, d, &mut a, &ended);
        // 9. The elapsed-time counter.
        if self.state != State::Wait {
            self.elapsed += if self.elapsed < 7200 { 2 } else { 1 };
        }
        self.prev_clip = a.anim.unwrap_or(clip_now);
        a
    }

    /// The melee window test with the block test and the once-per-attack rule. Returns true
    /// when the attack was blocked (the state is then sword-blocked).
    fn melee(&mut self, v: &View, damage: i32, a: &mut Act) -> bool {
        if self.once || v.bits & 1 == 0 || !v.weapon_touch {
            return false;
        }
        let dot = v.hero_facing[0] * v.facing[0] + v.hero_facing[2] * v.facing[2];
        if v.hero_guard && dot < -0.5 {
            self.enter(State::SwordBlocked);
            return true;
        }
        a.hero_damage = Some(damage);
        self.once = true;
        self.hit_sent = true;
        false
    }

    /// The end of a lunge or sword swing: idle, the invisibility rule, the attack timer.
    fn end_attack(&mut self, w: &Words, rng: &mut Rng) {
        if self.invisible && self.hit_sent {
            self.enter(State::StartInvisible);
        } else {
            self.invisible = false;
            self.enter(State::Idle);
        }
        self.hit_sent = false;
        self.rearm_attack(w, rng);
    }

    #[allow(clippy::too_many_arguments)]
    fn handle(&mut self, v: &View, w: &Words, rng: &mut Rng, d: f32, a: &mut Act, ended: &dyn Fn(u32) -> bool) {
        let toward_side = |left: bool| {
            let to = hdir(v.position, v.hero);
            let side = if left { [-to[2], 0.0, to[0]] } else { [to[2], 0.0, -to[0]] };
            [v.position[0] + 3.0 * side[0], v.position[1], v.position[2] + 3.0 * side[2]]
        };
        let first = !std::mem::replace(&mut self.started, true);
        match self.state {
            State::Idle => a.anim = Some(0),
            State::Approach => {
                a.travel = Some((0, v.hero));
            }
            State::StrafeRight => a.travel = Some((5, toward_side(false))),
            State::StrafeLeft => a.travel = Some((6, toward_side(true))),
            State::BackUp => {
                a.anim = Some(13);
                a.face = Some(0.5);
                if !first && ended(13) {
                    self.enter(State::Idle);
                }
            }
            State::Flinch => {
                if a.anim.is_none() {
                    a.anim = Some(if matches!(v.clip.0, 16 | 17) { v.clip.0 } else { 16 });
                }
                if !first && (ended(16) || ended(17)) {
                    self.enter(State::Idle);
                }
            }
            State::Stance(_) => {}
            State::Wait => a.anim = Some(24),
            State::EvilLaugh => {
                a.anim = Some(23);
                if !first && ended(23) {
                    self.enter(State::Idle);
                }
            }
            State::DodgeMissile => {
                if a.anim.is_none() {
                    a.anim = Some(if matches!(v.clip.0, 18 | 19) { v.clip.0 } else { 18 });
                }
                if !first && (ended(18) || ended(19)) {
                    self.enter(State::Idle);
                }
            }
            State::Deflect => {
                if v.missile_near {
                    a.anim = Some(21);
                } else {
                    a.anim = Some(0);
                    self.enter(State::Idle);
                }
            }
            State::DodgeMelee => {
                a.anim = Some(20);
                if !first && ended(20) {
                    if !self.clone && roll(rng) < 50 {
                        self.enter(State::Slip(0, Shadow::None));
                    } else {
                        self.enter(State::Idle);
                    }
                }
            }
            State::JumpThrust => {
                a.anim = Some(29);
                if first {
                    self.once = false;
                }
                if self.melee(v, 5, a) {
                    return;
                }
                if !first && ended(29) {
                    self.enter(State::Idle);
                }
            }
            State::ThrustLunge => {
                a.anim = Some(28);
                if first {
                    a.face = Some(1.0);
                }
                if self.melee(v, w.lunge_damage, a) {
                    return;
                }
                if !first && ended(28) {
                    self.end_attack(w, rng);
                }
            }
            State::SwordSwing => {
                a.anim = Some(26);
                a.face = Some(0.5);
                if self.melee(v, w.swing_damage, a) {
                    return;
                }
                if !first && ended(26) {
                    self.end_attack(w, rng);
                }
            }
            State::SwordBlocked => {
                a.anim = Some(27);
                if !first && ended(27) {
                    self.enter(State::Idle);
                }
            }
            State::DaggerThrow => {
                a.anim = Some(31);
                a.face = Some(0.5);
                if v.clip.0 == 31 && !v.clip.2 && v.hero_anim == 32 {
                    self.laugh_pending = true;
                }
                if !first && v.clip.0 == 31 && v.bits & 0x20 != 0 && self.last_frame != Some(v.frame) {
                    self.last_frame = Some(v.frame);
                    a.daggers.push(Dagger { aim: [v.hero[0], v.hero[1] + 2.5, v.hero[2]], speed: w.dagger_speed, damage: w.dagger_damage as f32, radius: 1.25 });
                }
                if !first && ended(31) {
                    self.enter(State::Idle);
                    self.rearm_attack(w, rng);
                }
            }
            State::Slip(phase, chosen) => self.slip(v, w, rng, phase, chosen, a, first, ended),
            State::JustAppear => {
                // Animation 22 only when the roll made at the reappear was above 74.
                if first && roll(rng) <= 74 {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                    return;
                }
                a.anim = Some(22);
                if !first && ended(22) {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                }
            }
            State::BackStab => {
                a.anim = Some(30);
                a.rate = 2.0;
                if !self.once && v.clip.0 == 30 && d < w.at(0x002D_AD0C) as f32 {
                    self.once = true;
                    a.hero_damage = Some(5);
                }
                if !first && ended(30) {
                    self.enter(State::Idle);
                    self.rearm_attack(w, rng);
                    self.rearm_slip(w, v, rng);
                }
            }
            State::ShadowDagger => {
                a.anim = Some(31);
                a.face = Some(0.5);
                if !first && v.clip.0 == 31 && v.bits & 0x20 != 0 {
                    a.daggers.push(Dagger { aim: [v.hero[0], v.hero[1] + 2.5, v.hero[2]], speed: 5.0, damage: 3.0, radius: 1.25 });
                }
                if !first && ended(31) {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                }
            }
            State::LightArc(phase) => self.light_arc(v, w, rng, phase, a, first, ended),
            State::Heal(phase) => self.heal(v, w, rng, phase, a, first, ended),
            State::Mirror(phase) => {
                a.rate = if phase == 0 { 2.0 } else { 1.0 };
                a.anim = Some(41 + phase as u32);
                if phase == 0 && !first && ended(41) {
                    self.state = State::Mirror(1);
                    self.started = false;
                } else if phase == 1 {
                    if v.clip.0 == 42 && v.clip.1 > 0.5 && !self.clone {
                        a.wake_clones = true;
                    }
                    if !first && ended(42) {
                        self.enter(State::Idle);
                        if !self.clone {
                            self.clone_mode = true;
                        }
                    }
                }
            }
            State::Rainbow(k) => self.rainbow(v, w, rng, k, a, first, ended),
            State::StartInvisible => {
                a.anim = Some(40);
                a.fade = Some(((1.0 - v.clip.1.clamp(0.0, 1.0)) * 128.0) as u8);
                if !first && ended(40) {
                    self.invisible = true;
                    self.enter(State::Idle);
                }
            }
            State::CloneWake => {
                // The woken clone runs the mirror handler from its second animation.
                self.state = State::Mirror(1);
                self.started = false;
                a.anim = Some(42);
            }
            State::Death | State::CloneWait => {}
        }
    }

    /// Shadow slip (4.7).
    #[allow(clippy::too_many_arguments)]
    fn slip(&mut self, v: &View, w: &Words, rng: &mut Rng, phase: u8, chosen: Shadow, a: &mut Act, first: bool, ended: &dyn Fn(u32) -> bool) {
        match phase {
            0 => {
                a.anim = Some(24);
                if !first && ended(24) {
                    self.state = State::Slip(1, Shadow::None);
                }
            }
            1 => {
                let mut pick = self.choose(v, w, rng);
                // The landing point.
                let (lo, hi) = match pick {
                    Shadow::Dagger => (w.at(0x002D_AD14), w.at(0x002D_AD14)),
                    Shadow::Heal => (w.at(0x002D_AD28), w.at(0x002D_AD2C)),
                    Shadow::LightArc => (w.at(0x002D_AD34), w.at(0x002D_AD38)),
                    _ => (w.at(0x002D_AD3C), w.at(0x002D_AD40)),
                };
                let land = if pick == Shadow::BackStab {
                    let f = [v.hero_facing[0], 0.0, v.hero_facing[2]];
                    let p = [v.hero[0] - 3.0 * f[0], v.hero[1], v.hero[2] - 3.0 * f[2]];
                    if (v.walkable)(p) {
                        p
                    } else {
                        pick = Shadow::Dagger;
                        v.hero
                    }
                } else {
                    let mut found = None;
                    for _ in 0..10 {
                        let ang = rng.frange(0.0, std::f32::consts::TAU);
                        let r = rng.frange(lo as f32, hi as f32);
                        let p = [v.hero[0] + r * ang.cos(), v.hero[1], v.hero[2] + r * ang.sin()];
                        if (v.walkable)(p) {
                            found = Some(p);
                            break;
                        }
                    }
                    found.unwrap_or_else(|| {
                        pick = Shadow::JustAppear;
                        v.hero
                    })
                };
                a.place = Some(land);
                a.face = Some(1.0);
                self.slip_latch = false;
                self.state = State::Slip(2, pick);
                self.started = false;
            }
            _ => {
                a.anim = Some(25);
                if matches!(chosen, Shadow::BackStab | Shadow::Mirror) {
                    a.rate = 2.0;
                }
                if !first && ended(25) {
                    let next = match chosen {
                        Shadow::BackStab => State::BackStab,
                        Shadow::Dagger => State::ShadowDagger,
                        Shadow::LightArc => State::LightArc(0),
                        Shadow::Heal => State::Heal(0),
                        Shadow::Mirror => State::Mirror(0),
                        Shadow::Rainbow => State::Rainbow(0),
                        _ => State::JustAppear,
                    };
                    self.enter(next);
                }
            }
        }
    }

    /// The shadow-attack chooser (4.8).
    pub fn choose(&self, v: &View, w: &Words, rng: &mut Rng) -> Shadow {
        let p = self.pattern;
        if p <= 1 {
            return Shadow::JustAppear;
        }
        let m = w.per(0x002D_ACB0, p);
        let dg = if self.clone { w.at(0x002D_ACD4) } else { w.per(0x002D_ACC4, p) };
        let h = w.per(0x002D_ACE4, p);
        let mi = if p >= 3 { w.at(0x002D_ACF4 + 4 * (p as u32 - 2)) } else { 0 };
        let rd = if p == 4 { w.at(0x002D_AD04) } else { 0 };
        let r = roll(rng);
        let max = v.max_hp.max(1) as f32;
        let cur = v.hp as f32;
        if r < m {
            Shadow::BackStab
        } else if r < m + dg {
            Shadow::Dagger
        } else if r < m + dg + h {
            if p == 2 {
                Shadow::LightArc
            } else {
                let enough = if p == 3 { (cur - 0.25 * max) / (0.25 * max) >= 0.5 } else { cur / (0.25 * max) >= 0.5 };
                if enough || roll(rng) >= 50 { Shadow::LightArc } else { Shadow::Heal }
            }
        } else if p >= 3 && r < m + dg + h + mi {
            Shadow::Mirror
        } else if p == 4 && r < m + dg + h + mi + rd {
            Shadow::Rainbow
        } else {
            Shadow::JustAppear
        }
    }

    /// The light arc (4.8).
    #[allow(clippy::too_many_arguments)]
    fn light_arc(&mut self, v: &View, w: &Words, rng: &mut Rng, phase: u8, a: &mut Act, first: bool, ended: &dyn Fn(u32) -> bool) {
        match phase {
            0 => {
                a.anim = Some(36);
                if !first && ended(36) {
                    self.state = State::LightArc(1);
                    self.started = false;
                }
            }
            1 => {
                a.anim = Some(37);
                if !first && ended(37) {
                    self.arc_dir = hdir(v.position, v.hero);
                    self.arc_start = [v.position[0], v.position[1] + w.arc_height, v.position[2]];
                    self.arc_ticks = 0;
                    self.arc_hit = false;
                    self.state = State::LightArc(2);
                    self.started = false;
                }
            }
            2 => {
                a.anim = Some(38);
                let (dd, l) = (self.arc_dir, [-self.arc_dir[2], 0.0, self.arc_dir[0]]);
                let s = std::f32::consts::FRAC_1_SQRT_2;
                let ua = [(dd[0] + l[0]) * s, 0.0, (dd[2] + l[2]) * s];
                let ub = [(dd[0] - l[0]) * s, 0.0, (dd[2] - l[2]) * s];
                let speed = w.at(0x002D_AD48) as f32 / 60.0;
                let k = self.arc_ticks as f32 * speed;
                let ball = |u: Vec3| [self.arc_start[0] + k * u[0], self.arc_start[1], self.arc_start[2] + k * u[2]];
                let (pa, pb) = (ball(ua), ball(ub));
                a.arc = Some((pa, pb));
                // Each ball a sphere of radius 2.0 against the hero's body cylinder.
                let touches = |p: Vec3| hdist(p, v.hero) <= 2.0 + v.hero_radius && p[1] + 2.0 >= v.hero[1] && p[1] - 2.0 <= v.hero[1] + v.hero_height;
                if !self.arc_hit && (touches(pa) || touches(pb)) {
                    self.arc_hit = true;
                    a.hero_damage = Some(w.at(0x002D_AD50));
                }
                self.arc_ticks += 1;
                if self.arc_hit || self.arc_ticks >= w.arc_loop {
                    self.state = State::LightArc(3);
                    self.started = false;
                }
            }
            _ => {
                a.anim = Some(39);
                if !first && ended(39) {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                }
            }
        }
    }

    /// Healing (4.9).
    #[allow(clippy::too_many_arguments)]
    fn heal(&mut self, v: &View, w: &Words, rng: &mut Rng, phase: u8, a: &mut Act, first: bool, ended: &dyn Fn(u32) -> bool) {
        match phase {
            0 => {
                a.anim = Some(32);
                if first {
                    a.aura = Some(true);
                }
                if !first && ended(32) {
                    self.state = State::Heal(1);
                    self.started = false;
                }
            }
            1 => {
                a.anim = Some(33);
                if first {
                    self.heal_count = w.at(0x002D_AD30);
                    return;
                }
                if self.heal_count > 0 {
                    self.heal_count -= 1;
                    return;
                }
                let max = v.max_hp.max(1);
                let step = (0.01 * max as f32) as i32;
                let target = max as f32 / self.pattern.max(1) as f32;
                if target < (v.hp + step) as f32 {
                    a.heal = (target - v.hp as f32) as i32;
                    a.aura = Some(false);
                    self.state = State::Heal(3);
                    self.started = false;
                } else {
                    a.heal = step;
                    self.heal_count = w.at(0x002D_AD30);
                }
            }
            2 => {
                a.anim = Some(34);
                if !first && ended(34) {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                }
            }
            _ => {
                a.anim = Some(35);
                if !first && ended(35) {
                    self.enter(State::Idle);
                    self.rearm_slip(w, v, rng);
                }
            }
        }
    }

    /// The rainbow dash (4.8).
    #[allow(clippy::too_many_arguments)]
    fn rainbow(&mut self, v: &View, w: &Words, rng: &mut Rng, k: u8, a: &mut Act, first: bool, ended: &dyn Fn(u32) -> bool) {
        if k == 0 && first {
            let mut used = [false; 8];
            for slot in 0..8 {
                let r = (rng.next_u32() % 7) as usize;
                let mut i = r;
                while used[i] {
                    i = (i + 1) % 8;
                }
                used[i] = true;
                self.order[slot] = i;
            }
            self.rainbow_hit = false;
        }
        if k >= 8 {
            a.anim = Some(45);
            if !first && ended(45) {
                self.enter(State::Idle);
                self.rearm_slip(w, v, rng);
                // The pattern is recomputed.
                let p = Self::band(v.hp as f32 / v.max_hp.max(1) as f32);
                self.pattern = self.pattern.max(p);
            }
            return;
        }
        a.anim = Some(44);
        a.face = Some(1.0);
        let target = v.rainbow[self.order[k as usize]].unwrap_or(v.position);
        let step = w.at(0x002D_AD54) as f32 / 60.0;
        let rem = hdist(v.position, target);
        if rem <= step {
            a.place = Some([target[0], v.position[1], target[2]]);
            self.rainbow_hit = false;
            self.state = State::Rainbow(k + 1);
            self.started = k + 1 < 8;
        } else {
            let dir = hdir(v.position, target);
            a.step = Some([dir[0] * step, 0.0, dir[2] * step]);
        }
        if !self.rainbow_hit && hdist(v.position, v.hero) < w.at(0x002D_AD58) as f32 {
            self.rainbow_hit = true;
            a.hero_damage = Some(w.rainbow_damage);
        }
    }
}

/// The clone slot offsets along the boss's side axis: +5, +10, -5, -10.
pub const CLONE_OFFSETS: [f32; 4] = [5.0, 10.0, -5.0, -10.0];

/// A clone's place: the boss's position plus the offset along (f.z, 0, -f.x).
pub fn clone_place(boss: Vec3, facing: Vec3, slot: usize) -> Vec3 {
    let l = facing[0].hypot(facing[2]).max(1e-6);
    let f = [facing[0] / l, 0.0, facing[2] / l];
    let o = CLONE_OFFSETS[slot.min(3)];
    [boss[0] + o * f[2], boss[1], boss[2] - o * f[0]]
}

/// The light-arc missile wall: a hero missile at P with step s is removed when it is on the n
/// side of A and, 10 ticks on, past B's side (n = (-u.z, 0, u.x), u from A to B).
pub fn wall_stops(a: Vec3, b: Vec3, p: Vec3, s: Vec3) -> bool {
    let u = hdir(a, b);
    let n = [-u[2], 0.0, u[0]];
    let q = [p[0] + 10.0 * s[0], p[1] + 10.0 * s[1], p[2] + 10.0 * s[2]];
    let ap = hdir(a, p);
    let bq = hdir(b, q);
    n[0] * ap[0] + n[2] * ap[2] > 0.0 && n[0] * bq[0] + n[2] * bq[2] < 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic tuning words with the spec's values.
    fn words() -> Words {
        let mut t = vec![0i32; WORDS];
        let mut set = |addr: u32, v: i32| t[((addr - BASE) / 4) as usize] = v;
        for (p, (lo, hi)) in [(8, 15), (4, 8), (3, 6), (2, 5)].into_iter().enumerate() {
            set(0x002D_ABD8 + 8 * p as u32, lo);
            set(0x002D_ABDC + 8 * p as u32, hi);
        }
        for (p, (lo, hi)) in [(2, 4), (2, 3), (1, 3), (1, 2)].into_iter().enumerate() {
            set(0x002D_AC18 + 8 * p as u32, lo);
            set(0x002D_AC1C + 8 * p as u32, hi);
        }
        set(0x002D_AC38, 75);
        for (a, v) in [(0x002D_AC88, 40), (0x002D_AC8C, 20), (0x002D_AC90, 30), (0x002D_AC94, 15), (0x002D_AC98, 60), (0x002D_AC9C, 20), (0x002D_ACA8, 15), (0x002D_ACAC, 100)] {
            set(a, v);
        }
        for (a, v) in [(0x002D_ACB4, 33), (0x002D_ACB8, 25), (0x002D_ACBC, 4), (0x002D_ACC8, 33), (0x002D_ACCC, 25), (0x002D_ACD0, 4), (0x002D_ACD4, 25), (0x002D_ACE4, 30), (0x002D_ACE8, 33), (0x002D_ACEC, 25), (0x002D_ACF0, 32), (0x002D_ACF8, 10), (0x002D_ACFC, 10), (0x002D_AD04, 50)] {
            set(a, v);
        }
        set(0x002D_AD30, 5);
        set(0x002D_AD48, 20);
        set(0x002D_AD50, 20);
        set(0x002D_AD54, 40);
        set(0x002D_AD58, 8);
        set(0x002D_AD0C, 6);
        Words { tuning: t, hit_limit: 2, hero_ratio: 0.55, dagger_speed: 1.5, dagger_damage: 10, lunge_damage: 25, swing_damage: 20, arc_loop: 200, arc_height: 1.5, rainbow_damage: 15 }
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, walk: &'a dyn Fn(Vec3) -> bool) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, 30.0],
            hero_facing: [0.0, 0.0, -1.0],
            hero_guard: false,
            hero_attacking: false,
            hero_anim: 0,
            hero_hp_ratio: 1.0,
            hero_radius: 1.0,
            hero_height: 3.5,
            hp: 800,
            max_hp: 800,
            clip: (0, 0.0, false),
            frame: 0,
            bits: 0,
            weapon_touch: false,
            melee_reach: false,
            missile_threat: false,
            missile_near: false,
            hit_melee: false,
            hit_missiles: 0,
            hit_damage: 0,
            nav_blocked: false,
            level,
            walkable: walk,
            rainbow: [None; 8],
            clones_alive: 0,
        }
    }

    #[test]
    fn damage_time_factor() {
        // SPEC-0013 G7: a hit of 15 takes 15 at the start, 30 at 15 s (900 ticks at 2 a tick:
        // counter 1800), 75 at 60 s (counter 7200).
        let w = words();
        let mut rng = Rng::seeded(1);
        let mut g = Gelert::new(&w, false, false, &mut rng);
        assert_eq!(g.taken(15), 15);
        g.elapsed = 1800;
        assert_eq!(g.taken(15), 30);
        g.elapsed = 7200;
        assert_eq!(g.taken(15), 75);
    }

    #[test]
    fn bands_and_events() {
        // HP bands vectors: 600 stays 1, 599 pattern 2 (event 1), 150 from pattern 1 to 4 (event 5).
        assert_eq!((Gelert::band(0.75), Gelert::band(599.0 / 800.0), Gelert::band(0.1875)), (1, 2, 4));
        let w = words();
        let mut rng = Rng::seeded(1);
        let (lv, wk) = (|b: u16| b == 0, |_: Vec3| true);
        let mut g = Gelert::new(&w, false, false, &mut rng);
        let a = g.tick(&View { hp: 150, ..view(&lv, &wk) }, &w, &mut rng);
        assert_eq!((g.pattern, g.state, a.set_level.contains(&5)), (4, State::Stance(6), true));
        // At pattern 4, HP 400 stays 4.
        g.state = State::Idle;
        g.tick(&View { hp: 400, ..view(&lv, &wk) }, &w, &mut rng);
        assert_eq!(g.pattern, 4);
    }

    #[test]
    fn script_gate_and_timers() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let wk = |_: Vec3| true;
        let off = |_: u16| false;
        let mut g = Gelert::new(&w, true, false, &mut rng);
        // Start-up timers: 480 to 900 and 120 to 240 ticks.
        assert!((480..=900).contains(&g.t_slip) && (120..=240).contains(&g.t_attack));
        let a = g.tick(&view(&off, &wk), &w, &mut rng);
        assert_eq!((g.state, a.anim), (State::Wait, Some(24)));
        let on = |b: u16| b == 0;
        g.tick(&view(&on, &wk), &w, &mut rng);
        assert_eq!(g.state, State::Approach, "released, d = 30 approaches");
    }

    #[test]
    fn attack_choice_and_strafe() {
        let w = words();
        let (lv, wk) = (|_: u16| true, |_: Vec3| true);
        let mut rng = Rng::seeded(3);
        // d = 12: thrust lunge (10 < d < 15 always lunges).
        let mut g = Gelert::new(&w, false, false, &mut rng);
        g.t_attack = 1;
        g.tick(&View { hero: [0.0, 0.0, 12.0], ..view(&lv, &wk) }, &w, &mut rng);
        assert_eq!(g.state, State::ThrustLunge);
        // d = 18 with the timer running: a strafe.
        let mut g = Gelert::new(&w, false, false, &mut rng);
        g.tick(&View { hero: [0.0, 0.0, 18.0], ..view(&lv, &wk) }, &w, &mut rng);
        assert!(matches!(g.state, State::StrafeLeft | State::StrafeRight));
    }

    #[test]
    fn melee_block_and_once() {
        let w = words();
        let (lv, wk) = (|_: u16| true, |_: Vec3| true);
        let mut rng = Rng::seeded(4);
        let mut g = Gelert::new(&w, false, false, &mut rng);
        g.enter(State::SwordSwing);
        g.started = true;
        g.t_attack = 999;
        let base = View { hero: [0.0, 0.0, 3.0], weapon_touch: true, bits: 1, clip: (26, 0.3, false), ..view(&lv, &wk) };
        // Not guarding: 20 once.
        let a = g.tick(&base, &w, &mut rng);
        assert_eq!(a.hero_damage, Some(20));
        assert_eq!(g.tick(&base, &w, &mut rng).hero_damage, None);
        // Guarding, facing each other (dot -1 < -0.5): blocked.
        let mut g = Gelert::new(&w, false, false, &mut rng);
        g.enter(State::ThrustLunge);
        g.started = true;
        g.t_attack = 999;
        let a = g.tick(&View { hero_guard: true, clip: (28, 0.3, false), ..base }, &w, &mut rng);
        assert_eq!((a.hero_damage, g.state), (None, State::SwordBlocked));
    }

    #[test]
    fn chooser_vectors() {
        let w = words();
        let (lv, wk) = (|_: u16| true, |_: Vec3| true);
        let mut rng = Rng::seeded(5);
        let mut g = Gelert::new(&w, false, false, &mut rng);
        // Pattern 1 always just-appears.
        assert_eq!(g.choose(&view(&lv, &wk), &w, &mut rng), Shadow::JustAppear);
        // Over many rolls pattern 4 reaches every bucket.
        g.pattern = 4;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            seen.insert(format!("{:?}", g.choose(&View { hp: 50, ..view(&lv, &wk) }, &w, &mut rng)));
        }
        assert_eq!(seen.len(), 7, "{seen:?}");
    }

    #[test]
    fn heal_arithmetic() {
        // Max 900, pattern 3, target 300, HP 150: 16 full heals of 9 to 294, then +6 and phase 3.
        let w = words();
        let (lv, wk) = (|_: u16| true, |_: Vec3| true);
        let mut rng = Rng::seeded(6);
        let mut g = Gelert::new(&w, false, false, &mut rng);
        g.pattern = 3;
        g.enter(State::Heal(1));
        let mut hp = 150;
        let mut heals = 0;
        for _ in 0..200 {
            let a = g.tick(&View { hp, max_hp: 900, hero: [0.0, 0.0, 30.0], ..view(&lv, &wk) }, &w, &mut rng);
            if a.heal != 0 {
                hp += a.heal;
                heals += 1;
            }
            if g.state != State::Heal(1) {
                break;
            }
        }
        assert_eq!((hp, heals, g.state), (300, 17, State::Heal(3)));
    }

    #[test]
    fn clone_places_and_wall() {
        assert_eq!(clone_place([10.0, 2.0, 20.0], [0.0, 0.0, 1.0], 0), [15.0, 2.0, 20.0]);
        assert_eq!(clone_place([0.0; 3], [1.0, 0.0, 0.0], 1), [0.0, 0.0, -10.0]);
        // The missile-wall vectors.
        let (a, b) = ([0.0; 3], [10.0, 0.0, 0.0]);
        assert!(wall_stops(a, b, [5.0, 1.0, 2.0], [0.0, 0.0, -0.5]));
        assert!(!wall_stops(a, b, [5.0, 1.0, 2.0], [0.0, 0.0, -0.1]));
        assert!(!wall_stops(a, b, [5.0, 1.0, -2.0], [0.0, 0.0, 0.5]));
        assert!(wall_stops(a, b, [50.0, 1.0, 2.0], [0.0, 0.0, -0.5]));
    }

    #[test]
    fn rainbow_order() {
        // Draws 3, 1, 3, 0, 6, 6, 6, 5 give 3, 1, 4, 0, 6, 7, 2, 5 (the cyclic first-unused rule).
        let draws = [3usize, 1, 3, 0, 6, 6, 6, 5];
        let mut used = [false; 8];
        let mut order = Vec::new();
        for r in draws {
            let mut i = r;
            while used[i] {
                i = (i + 1) % 8;
            }
            used[i] = true;
            order.push(i);
        }
        assert_eq!(order, vec![3, 1, 4, 0, 6, 7, 2, 5]);
    }
}
