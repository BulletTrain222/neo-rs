//! Common enemies, class 17 (SPEC-0038): the think loop's pre-step (awareness countdown, alert
//! flag, per-state switch), the notice check (Behavior 3) with the band factors (Behavior 18),
//! the state setter (Behavior 5), the idle picker (Behavior 8), the agro machine (Behavior 44:
//! set-up, rush, keep distance, strafe, search after lost sight, hit reactions), the normal
//! attack choice (Behavior 7), the distance draws (Behavior 4), the attack sequencer (Behavior 13:
//! melee, charge and loop states), the agro tail with the leash (Behavior 23), the chase stop
//! (Behavior 48.6), the take-hit hook (Behavior 37) and the dead state (Behaviors 34 and 36).
//!
//! Not covered: the alarm groups and the group alert (Behaviors 3 and 10), socialize, the global
//! AI pause (state 20), the retarget countdown and flyer re-launch (state 22 is played but never
//! entered), the map search (Behaviors 9, 17, 18, 39, 42, 43), the named-target state 9 (it
//! stands and notices), the alarm-group emerge of state 8, wandering in
//! states 5 to 7 (they stand and notice), retreat home (state 14) and the distance-hide gate,
//! the melee and missile defences (outers 5 to 8), the counter-attack choice (Behavior 14), the
//! confused outer 11, the attack manager and the wait in line (outer 12), projectiles (missile
//! slots play their strike with no shot), the grab-and-throw, statuses, theft and Mote drain,
//! ghosts, reanimation (state 16 of rows 224, 226, 227), drops and coins, and flyers (the AI is
//! not run for rows that can fly). The alerted search (state 2) only aims at the last-seen point
//! until its countdown ends (R-0015).

use crate::hero::Vec3;
use crate::rng::Rng;

pub const MELEE: u16 = 0;
pub const MISSILE: u16 = 1;
pub const UNUSED: u16 = 0xFFFF;

/// One attack slot k of the monster row (SPEC-0038 Layout section 2, offsets + 4k or + 2k).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slot {
    pub kind: u16,
    pub orient: bool,
    pub trigger_max: f32,
    pub trigger_min: f32,
    pub chance: f32,
    pub special: f32,
    pub wind_speed: f32,
    pub strike_speed: f32,
    pub recover_speed: f32,
    pub arc: f32,
    pub range: f32,
    pub delay_max: f32,
    pub delay_min: f32,
    pub charge: i32,
    pub charge_max: f32,
    pub charge_min: f32,
    pub damage: f32,
    pub knockback: f32,
    pub flags: u32,
    pub spawn_cap: u16,
    pub loopable: bool,
    pub loop_min: i32,
    pub loop_max: i32,
    pub loop_hold: f32,
}

/// The monster-row values class 17 reads (SPEC-0038 Layout section 2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params {
    pub approach: u16,
    pub hostile: bool,
    pub display_default: bool,
    pub can_fly: i32,
    pub search_max: f32,
    pub search_min: f32,
    pub attack_timer_max: f32,
    pub attack_timer_min: f32,
    pub approach_timer_max: f32,
    pub approach_timer_min: f32,
    pub turn: f32,
    pub distance_hide: bool,
    pub flying_hit: i32,
    pub anger: bool,
    pub view_angle: f32,
    pub awareness: i32,
    pub awareness_timer: f32,
    pub chance_far: f32,
    pub chance_near: f32,
    pub range_far: f32,
    pub range_near: f32,
    pub always_sight: bool,
    pub rush: (f32, f32),
    pub desired: (f32, f32),
    pub retreat: (f32, f32),
    pub strafe: (f32, f32),
    pub backbiter: f32,
    pub disappear: i32,
    pub slots: [Slot; 4],
}

/// What the enemy sees this tick (computed by the caller from the world).
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero is attackable (not dead, held, blocking or hit-locked; Behavior 7, step 1).
    pub hero_attackable: bool,
    /// The band factors F1 and F2 for the hero's action state (Behavior 18).
    pub bands: (f32, f32),
    /// The sight line to the hero is clear.
    pub sight: bool,
    /// A navmesh route to the hero exists, and the hero is on the walk mesh.
    pub route: bool,
    pub hero_on_mesh: bool,
    /// The current walk waypoint (leash, waypoint arrival tests).
    pub waypoint: Vec3,
    /// The blocked flag (0xCC4) and the enemy's speed this tick (units per tick).
    pub blocked: bool,
    pub speed: f32,
    /// A weapon volume touches one of the hero's bone capsules this tick (the strike test).
    pub contact: bool,
    /// Own record radius plus the hero's.
    pub radii: f32,
    pub clip: Clip,
    /// The enemy's animation table holds an id.
    pub has: &'a dyn Fn(u32) -> bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub action: u32,
    pub progress: f32,
    pub finished: bool,
}

impl Clip {
    /// "Plays to its end": the clip progress gate at 0.99 (SPEC-0005 Clip progress), which a
    /// looping clip can also pass; a finished one-shot reads 1.0.
    pub fn ended(&self) -> bool {
        self.finished || self.progress >= 0.99
    }
}

/// How the enemy moves this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Go {
    /// A forward gait: navigate toward the point (the move direction steers the facing).
    Toward(Vec3),
    /// A back or side gait: the clip's own root travel carries the enemy while it faces the
    /// hero (R-0015).
    Facing(Vec3),
}

/// A strike on the hero by slot k (Behavior 13, strike test or loop damage).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub slot: usize,
    pub damage: f32,
    pub knockback: f32,
    pub unblockable: bool,
    pub force_blocked: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Restart the requested clip even when it plays (the take-hit force-start).
    pub force: bool,
    /// The per-tick speed scale P stored by the sequencer (the clip rate this tick).
    pub rate: Option<f32>,
    pub go: Option<Go>,
    /// Turn toward a point with a weight.
    pub face: Option<(Vec3, f32)>,
    pub strike: Option<Strike>,
    /// Discard this tick's root step (chase stop).
    pub stop: bool,
    /// Sink the corpse by this much along world up.
    pub sink: f32,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}

fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 0.0 { a.map(|c| c / l) } else { a }
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn along(p: Vec3, d: Vec3, s: f32) -> Vec3 {
    [p[0] + d[0] * s, p[1] + d[1] * s, p[2] + d[2] * s]
}

/// Behavior 18: the band factors F1 and F2 by hero action state, from the two boot tables of
/// seven words each.
pub fn band_factors(state: u8, f1: &[f32], f2: &[f32]) -> (f32, f32) {
    let i = match state {
        2 | 5 => 0,
        3 | 9 => 1,
        4 | 6 | 10 => 2,
        21 | 41 => 3,
        23 => 4,
        22 => 5,
        1 => 6,
        _ => return (1.0, 0.5),
    };
    (f1.get(i).copied().unwrap_or(1.0), f2.get(i).copied().unwrap_or(0.5))
}

/// Slot animation ids: A = 33 + 5k (wind-up) to E = 37 + 5k (recovery).
fn slot_anim(k: usize, n: u32) -> u32 {
    33 + 5 * k as u32 + n
}

/// The spawn record's words read by class 17 (SPEC-0038 Layout section 1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spawn {
    /// The default-state word (u16 at 0x84).
    pub default_state: u16,
    /// Home point (the placement position), the emerge distance (f32 at 0xC0) and the selector
    /// word (0xC8: 0 to 4 pick the emerge and hide animations).
    pub home: Vec3,
    pub emerge_distance: f32,
    pub selector: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    pub p: Params,
    pub spawn: Spawn,
    /// The spawn record's default-state word (u16 at 0x84).
    pub default_state: u16,
    /// State 8's sub-state (0xBC0).
    pub emerge: u32,
    pub state: u32,
    pub previous: u32,
    /// Base actor state 16 (dead), the disappear countdown and the corpse's release.
    pub dead: bool,
    pub disappear: i32,
    pub gone: bool,
    awareness: i32,
    alert: bool,
    cooldown: [i32; 4],
    recovery: i32,
    pub selected: Option<usize>,
    desired: f32,
    retreat: f32,
    charge: [f32; 4],
    search: i32,
    last_seen: Vec3,
    approach_dir: Vec3,
    approach_timer: i32,
    pub outer: u32,
    pub inner: u32,
    pub seq: u32,
    hold: i32,
    lock: i32,
    route: bool,
    leash: u32,
    gaits: [u32; 7],
    react: u32,
    react_phase: u32,
    stun: i32,
    stun_type: i32,
    counter: u32,
    anger_count: u32,
    pub hit_recovery: u32,
    start: Vec3,
    ticks: u32,
    loop_count: i32,
    reentered: bool,
    /// The hero is on this enemy's hit list (cleared at each attack start).
    hit: bool,
    idle_timer: i32,
    idle_variant: Option<u32>,
    through: Option<u32>,
    through_seen: bool,
    force: bool,
}

impl Enemy {
    /// Creation: the distance draws (Behavior 4), the idle timer, and the think loop's start
    /// (Behavior 1, step 1: the setter with 0; state 8 when the default-state word is 8).
    pub fn new(p: Params, spawn: Spawn, rng: &mut Rng) -> Self {
        let default_state = spawn.default_state;
        let mut e = Self {
            p,
            spawn,
            default_state,
            emerge: 0,
            state: 0,
            previous: 0,
            dead: false,
            disappear: 0,
            gone: false,
            awareness: -1,
            alert: false,
            cooldown: [0; 4],
            recovery: 0,
            selected: None,
            desired: 0.0,
            retreat: 0.0,
            charge: [0.0; 4],
            search: 0,
            last_seen: [0.0; 3],
            approach_dir: [0.0; 3],
            approach_timer: 0,
            outer: 0,
            inner: 0,
            seq: 0,
            hold: 0,
            lock: 0,
            route: true,
            leash: 0,
            gaits: [0; 7],
            react: 0,
            react_phase: 0,
            stun: 0,
            stun_type: 0,
            counter: 0,
            anger_count: 0,
            hit_recovery: 0,
            start: [0.0; 3],
            ticks: 0,
            loop_count: 0,
            reentered: false,
            hit: false,
            idle_timer: 0,
            idle_variant: None,
            through: None,
            through_seen: false,
            force: false,
        };
        e.draw_distances(rng);
        e.idle_timer = rng.range(120, 600);
        e.state = if default_state == 8 { 8 } else { e.default_case() };
        e
    }

    /// Behavior 4: the desired distance from the approach band (f = int[20, 80] / 100), the
    /// retreat distance, and one charge distance per slot from the selected slot's range (slot 0
    /// when none is selected).
    fn draw_distances(&mut self, rng: &mut Rng) {
        let p = &self.p;
        let (lo, hi) = match p.approach {
            1 => p.rush,
            3 => p.desired,
            4 => p.strafe,
            _ => (5.0, 15.0),
        };
        let f = rng.range(20, 80) as f32 / 100.0;
        self.desired = lo + (hi - lo) * f;
        self.retreat = rng.frange(p.retreat.0, p.retreat.1);
        let s = p.slots[self.selected.unwrap_or(0)];
        for k in 0..4 {
            self.charge[k] = rng.frange(s.charge_min, s.charge_max);
        }
    }

    fn default_case(&self) -> u32 {
        if self.p.display_default || self.default_state == 0 { 0 } else { 23 }
    }

    /// The setter (Behavior 5): saves the previous state, runs the entry work and stores the
    /// state. Value 0 is the default case.
    fn set(&mut self, s: u32, v: &View, rng: &mut Rng, a: &mut Act) {
        let mut s = if s == 0 { self.default_case() } else { s };
        match s {
            1 => {
                self.approach_dir = unit([v.hero[0] - v.position[0], 0.0, v.hero[2] - v.position[2]]);
                self.approach_timer = rng.frange(self.p.approach_timer_min, self.p.approach_timer_max) as i32;
            }
            2 | 21 => {
                if s == 2 && !self.p.display_default {
                    s = 21;
                }
                self.search = rng.frange(self.p.search_min, self.p.search_max) as i32;
                self.last_seen = v.hero;
            }
            10 => {
                if (v.has)(73) {
                    a.anim = Some(73);
                } else {
                    s = 0;
                }
            }
            19 => {
                self.outer = 0;
                self.recovery = 0;
                self.cooldown = [0; 4];
            }
            _ => {}
        }
        if s == 8 {
            self.emerge = 0;
        }
        if s != self.state {
            self.previous = self.state;
        }
        self.state = s;
        self.through = None;
    }

    /// Behavior 40: state 8's pre-step (the emerge).
    fn emerge_step(&mut self, v: &View, rng: &mut Rng, a: &mut Act) {
        let sel = self.spawn.selector;
        match self.emerge {
            0 => {
                if len(sub(v.hero, v.position)) < self.spawn.emerge_distance * v.bands.0 {
                    self.emerge = 1;
                }
            }
            1 => {
                let id = if self.p.can_fly != 0 {
                    165
                } else {
                    match sel {
                        1..=4 => 132 + sel,
                        _ => 10,
                    }
                };
                if !(v.has)(id) {
                    self.set(19, v, rng, a);
                    return;
                }
                self.through(id, v, a);
                self.emerge = 2;
            }
            _ => match self.through {
                Some(id) => {
                    if self.through(id, v, a) {
                        self.set(19, v, rng, a);
                    }
                }
                None => self.set(19, v, rng, a),
            },
        }
    }

    /// Behavior 40: state 8's handler in sub 0 (hide at home, or walk home with gait 0).
    fn hide_handler(&self, v: &View, a: &mut Act) {
        if self.emerge != 0 {
            return;
        }
        if hdist(v.position, self.spawn.home) <= 0.35 || self.p.distance_hide {
            let sel = self.spawn.selector;
            a.anim = Some(match sel {
                1..=4 => 127 + sel,
                _ => 9,
            });
        } else {
            a.anim = Some(2);
            a.go = Some(Go::Toward(self.spawn.home));
        }
    }

    /// Behavior 3: the notice check (no alarm-group gate, no socialize halving, no cloak).
    fn notice(&self, v: &View, rng: &mut Rng) -> bool {
        let p = &self.p;
        if !p.hostile || self.awareness != 0 {
            return false;
        }
        if rng.range(1, 100) > p.awareness {
            return false;
        }
        let (f1, f2) = v.bands;
        let (mut near, mut far) = (p.range_near * f1, p.range_far * f1);
        if !v.sight && !p.always_sight {
            (near, far) = (p.range_near * f2, p.range_far * f2);
        }
        let d = len(sub(v.hero, v.position));
        if d < near {
            return true;
        }
        if d > far {
            return false;
        }
        let span = p.range_far - p.range_near;
        let mut chance = if span != 0.0 { p.chance_near + (p.chance_far - p.chance_near) * (d - p.range_near) / span } else { p.chance_near };
        if dot(v.facing, unit(sub(v.hero, v.position))) < p.view_angle {
            chance *= 0.5;
        }
        (rng.range(1, 100) as f32) < chance
    }

    /// Behavior 8: the idle picker (base 0, or 97 while moving; variants 115 to 119, or 142
    /// while moving, after a hold of int[120, 600] ticks).
    fn idle(&mut self, moving: bool, v: &View, rng: &mut Rng) -> u32 {
        let base = if moving { 97 } else { 0 };
        if let Some(id) = self.idle_variant {
            if v.clip.action == id && !v.clip.ended() {
                return id;
            }
            self.idle_variant = None;
            self.idle_timer = rng.range(120, 600);
            return base;
        }
        if self.idle_timer > 0 {
            self.idle_timer -= 1;
            return base;
        }
        let id = if moving { 142 } else { 115 + rng.index(5) };
        if (v.has)(id) {
            self.idle_variant = Some(id);
            id
        } else {
            self.idle_timer = rng.range(120, 600);
            base
        }
    }

    /// Play an animation to its end: true once it has finished (at once when the table lacks it).
    fn through(&mut self, id: u32, v: &View, a: &mut Act) -> bool {
        if !(v.has)(id) {
            return true;
        }
        a.anim = Some(id);
        if self.through != Some(id) {
            self.through = Some(id);
            self.through_seen = false;
            return false;
        }
        if v.clip.action == id && !v.clip.ended() {
            self.through_seen = true;
        }
        v.clip.action == id && v.clip.ended() && self.through_seen
    }

    /// Gait n of the current set (gaits 5 and 6 are swapped in the lookup).
    fn gait(&self, n: usize) -> u32 {
        self.gaits[match n {
            5 => 6,
            6 => 5,
            n => n,
        }]
    }

    fn walk(&self, n: usize, to: Vec3, v: &View, a: &mut Act) {
        a.anim = Some(self.gait(n));
        a.go = Some(if matches!(n, 3 | 5 | 6) { Go::Facing(v.hero) } else { Go::Toward(to) });
    }

    fn hold_anim(&self, id: u32, w: f32, v: &View, a: &mut Act) {
        a.anim = Some(id);
        a.face = Some((v.hero, w));
    }

    /// The outer sub-state entry routine (Behavior 44).
    fn enter(&mut self, event: u32, param: u32) {
        match event {
            1 => {
                self.inner = 1;
                self.gaits = [21, 21, 21, 23, 55, 0, 0];
            }
            3 => {
                self.inner = 4;
                self.gaits = [24, 24, 24, 26, 55, 0, 0];
            }
            4 => {
                self.inner = 10;
                self.hold = 0;
                self.gaits = [28, 28, 28, 32, 55, 30, 29];
            }
            2 => {
                self.seq = param;
                self.hit = false;
                self.reentered = false;
            }
            10 => {
                self.react = param;
                self.react_phase = 0;
            }
            _ => {}
        }
        self.outer = event;
        self.through = None;
    }

    /// The take-hit hook (Behavior 37; no ghost absorb, no hit lock, no stun): returns the
    /// damage to subtract and the point to face (the attacker), and sets the force-start.
    pub fn take_hit(&mut self, damage: i32, attacker: Vec3, rng: &mut Rng) -> (i32, Vec3) {
        if self.dead {
            return (0, attacker);
        }
        let mut damage = damage;
        if self.state != 19 {
            damage *= 2;
            self.previous = self.state;
            self.state = 19;
        }
        if !(self.outer == 10 && self.react == 2 && self.stun_type == 1) {
            let kind = if self.p.can_fly != 0 {
                match self.p.flying_hit {
                    1 => Some(7),
                    0 | 2 => Some(6),
                    _ => None,
                }
            } else {
                Some(0)
            };
            if let Some(kind) = kind {
                if let (2, 14, Some(k)) = (self.outer, self.seq, self.selected) {
                    self.end_path(k, rng);
                    self.selected = None;
                }
                self.enter(10, kind);
            }
        }
        self.hit_recovery = 90;
        self.force = true;
        self.alert = true;
        if self.p.anger {
            self.anger_count += 1;
        }
        (damage, attacker)
    }

    /// Behavior 34 (no reanimation, no flyer grounding, no drops): base state 16 and the death
    /// disappear countdown.
    pub fn die(&mut self) {
        if !self.dead {
            self.dead = true;
            self.disappear = self.p.disappear;
        }
    }

    /// A strike left the hero at 0 HP: the victory pose (state 10).
    pub fn hero_down(&mut self, v: &View, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        self.set(10, v, rng, &mut a);
        a
    }

    /// The sequencer's end path: outer 0, the recovery countdown and the slot cooldown.
    fn end_path(&mut self, k: usize, rng: &mut Rng) {
        let s = self.p.slots[k];
        self.outer = 0;
        self.recovery = rng.range(self.p.attack_timer_min as i32, self.p.attack_timer_max as i32);
        self.cooldown[k] = rng.range(s.delay_min as i32, s.delay_max as i32);
        self.through = None;
    }

    /// Behavior 36: the dead state (death clip 58 once, the disappear countdown from progress
    /// 0.95, the sink of 1/180 per tick, release below 0).
    fn dead_tick(&mut self, v: &View) -> Act {
        let mut a = Act { anim: Some(58), ..Act::default() };
        let flying = self.p.can_fly == 2;
        if flying {
            a.anim = Some(153);
        }
        if v.clip.action == a.anim.unwrap_or(58) && v.clip.progress >= 0.95 {
            self.disappear -= 1;
            if !flying && self.disappear < self.p.disappear - 30 {
                a.sink = 1.0 / 180.0;
            }
        }
        if self.disappear < 0 {
            self.gone = true;
        }
        a
    }

    /// One think-loop tick.
    pub fn tick(&mut self, v: &View, rng: &mut Rng) -> Act {
        if self.dead {
            return self.dead_tick(v);
        }
        let mut a = Act { force: std::mem::take(&mut self.force), ..Act::default() };
        // Pre-step (Behavior 2): awareness countdown, alert flag, the state switch.
        if self.awareness < 0 {
            self.awareness = self.p.awareness_timer as i32;
        } else {
            self.awareness -= 1;
        }
        if std::mem::take(&mut self.alert) && ![19, 1, 4, 16].contains(&self.state) {
            let s = if self.p.hostile { 19 } else { 1 };
            self.set(s, v, rng, &mut a);
        }
        match self.state {
            0 => self.set(self.default_state as u32, v, rng, &mut a),
            8 => self.emerge_step(v, rng, &mut a),
            3 | 5..=7 | 9 => {
                if self.notice(v, rng) {
                    self.set(2, v, rng, &mut a);
                }
            }
            10 => {
                if v.clip.action == 73 && v.clip.ended() {
                    self.set(0, v, rng, &mut a);
                }
            }
            19 => self.agro(v, rng, &mut a),
            _ => {}
        }
        // State handlers.
        match self.state {
            1 => {
                a.anim = Some(20);
                a.go = Some(Go::Toward(along(v.position, self.approach_dir, 5.0)));
                self.approach_timer -= 1;
                if self.approach_timer < 1 {
                    self.set(0, v, rng, &mut a);
                }
            }
            2 | 21 => {
                if self.state == 21 {
                    a.anim = Some(self.idle(true, v, rng));
                    a.go = Some(Go::Toward(self.last_seen));
                } else {
                    a.anim = Some(self.idle(false, v, rng));
                    a.face = Some((self.last_seen, 0.1));
                }
                self.search -= 1;
                if self.search < 1 {
                    self.set(0, v, rng, &mut a);
                }
            }
            10 if a.anim.is_none() && (v.has)(73) => a.anim = Some(73),
            22 => {
                if self.through(75, v, &mut a) {
                    self.set(0, v, rng, &mut a);
                }
            }
            23 => {
                if self.through(81, v, &mut a) {
                    self.set(self.default_state as u32, v, rng, &mut a);
                }
            }
            8 => self.hide_handler(v, &mut a),
            0 | 3 | 5..=7 | 9 | 20 => a.anim = Some(self.idle(false, v, rng)),
            _ => {}
        }
        a
    }

    /// Behavior 7: the normal attack choice.
    fn choose(&mut self, v: &View, rng: &mut Rng) {
        if !v.hero_attackable {
            return;
        }
        for c in &mut self.cooldown {
            if *c > 0 {
                *c -= 1;
            }
        }
        if self.outer == 2 {
            return;
        }
        self.selected = None;
        if ![1, 3, 4, 11].contains(&self.outer) || self.recovery >= 1 {
            return;
        }
        let d = len(sub(v.hero, v.position));
        let cos = dot(v.facing, unit(sub(v.hero, v.position)));
        for k in (0..4).rev() {
            let s = self.p.slots[k];
            if s.kind == UNUSED || self.cooldown[k] >= 1 {
                continue;
            }
            if !(s.trigger_min < d && d < s.trigger_max) || cos <= s.arc {
                continue;
            }
            let mut threshold = s.chance as i32;
            if self.p.approach == 4 && dot(v.facing, v.hero_facing) <= 0.0 {
                threshold = (threshold as f32 * self.p.backbiter) as i32;
            }
            if s.flags & 0x40 != 0 {
                threshold = (self.anger_count as f32 * 0.05) as i32;
            }
            if rng.range(1, 100) >= threshold {
                continue;
            }
            // Summons are not spawned here: the live count is 0.
            if s.flags & 0x02 != 0 && s.spawn_cap == 0 {
                continue;
            }
            // Trigger special 1 (airborne only) never passes: flyers are not covered.
            if s.special as i32 == 1 {
                continue;
            }
            let ok = if s.charge == -1 { v.route || (s.kind == MELEE && d < 10.0) || s.kind == MISSILE } else { v.route };
            if !ok {
                continue;
            }
            self.selected = Some(k);
            self.cooldown[k] = rng.range(s.delay_min as i32, s.delay_max as i32);
            if s.flags & 0x40 != 0 {
                self.anger_count = 0;
            }
            let start = if s.loopable {
                13
            } else if s.charge == -1 {
                0
            } else {
                6
            };
            self.enter(2, start);
            self.draw_distances(rng);
            return;
        }
    }

    /// Behavior 44: one agro tick.
    fn agro(&mut self, v: &View, rng: &mut Rng, a: &mut Act) {
        if self.recovery > 0 {
            self.recovery -= 1;
        }
        // Lost-sight test (no grab-and-throw slots run here).
        if self.outer != 9 && !v.sight {
            self.outer = 9;
            self.last_seen = v.hero;
            self.counter = 0;
        }
        self.choose(v, rng);
        let d = len(sub(v.hero, v.position));
        let dir = unit(sub(v.hero, v.position));
        let near_waypoint = hdist(v.position, v.waypoint) <= 1.0;
        match self.outer {
            0 => {
                let ap = self.p.approach as u32;
                self.enter(ap, 0);
                self.route = true;
                a.anim = Some(self.idle(false, v, rng));
            }
            1 => {
                let (rmin, rmax) = self.p.rush;
                if !self.route {
                    self.inner = if near_waypoint { 1 } else { 0 };
                }
                match self.inner {
                    0 => {
                        let g = if self.route && d > 2.0 * rmax { 2 } else { 0 };
                        self.walk(g, v.hero, v, a);
                        if self.route && d < self.desired {
                            self.inner = 1;
                        }
                    }
                    1 => {
                        self.hold_anim(22, self.p.turn, v, a);
                        if self.route {
                            if d < rmin {
                                self.inner = 2;
                            } else if d > rmax {
                                self.inner = 0;
                            }
                        }
                    }
                    _ => {
                        self.walk(3, sub(v.hero, dir.map(|c| c * (rmax + 2.0))), v, a);
                        if d > self.desired {
                            self.inner = 1;
                        }
                    }
                }
            }
            3 => {
                let (dmin, dmax) = self.p.desired;
                let (tmin, tmax) = self.p.retreat;
                if !self.route {
                    self.inner = if near_waypoint { 4 } else { 3 };
                    if self.inner == 4 {
                        self.hold_anim(25, 0.25, v, a);
                    } else {
                        self.walk(0, v.hero, v, a);
                    }
                } else {
                    match self.inner {
                        4 => {
                            self.hold_anim(25, 0.25, v, a);
                            if d > dmax {
                                self.inner = 3;
                            } else if d < dmin {
                                self.inner = 5;
                            } else if d < tmin {
                                self.inner = 6;
                            }
                        }
                        3 => {
                            self.walk(0, v.hero, v, a);
                            if d < self.desired {
                                self.inner = 4;
                            }
                            if d < tmin {
                                self.inner = 6;
                            }
                        }
                        5 => {
                            self.walk(3, sub(v.hero, dir.map(|c| c * dmax)), v, a);
                            if d > self.desired {
                                self.inner = 4;
                            } else if d < tmin {
                                self.inner = 6;
                            }
                        }
                        _ => {
                            self.walk(2, sub(v.hero, dir.map(|c| c * (tmax + 2.0))), v, a);
                            if d >= self.retreat {
                                self.inner = 4;
                            }
                        }
                    }
                }
            }
            4 => self.strafe(d, dir, v, rng, a),
            2 => self.sequencer(d, v, rng, a),
            9 => {
                if !v.sight {
                    let mut pt = self.last_seen;
                    match self.p.approach {
                        3 => pt = along(pt, dir, self.p.desired.0),
                        4 => pt = along(pt, dir, self.p.strafe.0),
                        _ => {}
                    }
                    if hdist(v.position, pt) < 0.5 {
                        self.counter += 1;
                        a.anim = Some(self.idle(false, v, rng));
                    } else {
                        self.walk(2, pt, v, a);
                        if v.route {
                            self.counter = 0;
                        } else {
                            self.counter += 1;
                        }
                    }
                    if self.counter > 120 {
                        self.set(0, v, rng, a);
                    }
                } else {
                    self.outer = 0;
                    a.anim = Some(self.idle(false, v, rng));
                }
            }
            10 => self.reaction(v, a),
            _ => {}
        }
        // The agro tail (Behavior 23).
        if self.state == 19 {
            if [1, 3, 4].contains(&self.outer) {
                self.route = v.route;
            }
            if [1, 3, 4].contains(&self.outer) && !self.route {
                if hdist(v.waypoint, v.hero) >= 15.0 {
                    self.leash += 1;
                } else {
                    self.leash = 0;
                }
            } else {
                self.leash = 0;
            }
            if self.leash > 180 {
                self.leash = 0;
                self.set(0, v, rng, a);
            }
        }
    }

    fn strafe(&mut self, d: f32, dir: Vec3, v: &View, rng: &mut Rng, a: &mut Act) {
        let (smin, smax) = self.p.strafe;
        if self.lock > 0 {
            self.lock -= 1;
        }
        if !self.route {
            self.inner = if hdist(v.position, v.waypoint) <= 1.0 { 10 } else { 7 };
        }
        let tests = |e: &mut Self| {
            if d > smax {
                e.inner = 7;
            } else if d < smin && e.lock < 1 {
                e.inner = 11;
            }
        };
        match self.inner {
            7 => {
                self.walk(0, v.hero, v, a);
                if d < self.desired {
                    self.hold = 0;
                    self.inner = 10;
                }
            }
            8 | 9 => {
                let side = unit([-dir[2], 0.0, dir[0]]);
                let s = if self.inner == 8 { -2.0 } else { 2.0 };
                let g = if self.inner == 8 { 6 } else { 5 };
                self.walk(g, along(v.position, side, s), v, a);
                self.hold -= 1;
                if self.hold < 1 {
                    self.inner = 10;
                }
                if v.blocked {
                    self.inner = 10;
                    self.lock = 120;
                }
                tests(self);
            }
            10 => {
                self.hold_anim(31, self.p.turn, v, a);
                self.hold -= 1;
                if self.hold <= 0 {
                    let r = rng.range(1, 3);
                    self.inner = match r {
                        2 if self.lock < 1 => 8,
                        3 if self.lock < 1 => 9,
                        _ => 10,
                    };
                    self.hold = rng.range(60, 180);
                }
                tests(self);
            }
            _ => {
                self.walk(3, sub(v.hero, dir.map(|c| c * smax)), v, a);
                if d > self.desired || v.blocked {
                    if v.blocked {
                        self.lock = 120;
                    }
                    self.hold = 0;
                    self.inner = 10;
                }
            }
        }
    }

    /// Behavior 44 hit reactions (outer 10).
    fn reaction(&mut self, v: &View, a: &mut Act) {
        let (first, second) = match self.react {
            0 => (57, None),
            5 => (127, None),
            6 => (152, None),
            7 => (76, Some(74)),
            2 => (60, Some(61)),
            3 => (62, Some(64)),
            _ => {
                self.outer = 0;
                return;
            }
        };
        if self.react_phase == 0 {
            let done = match self.react {
                7 => {
                    a.anim = Some(76);
                    v.speed <= 2.0
                }
                2 => {
                    a.anim = Some(60);
                    self.stun <= 0
                }
                _ => self.through(first, v, a),
            };
            if !done {
                return;
            }
            self.react_phase = 1;
            self.through = None;
        }
        match second {
            Some(id) if (v.has)(id) => {
                if self.through(id, v, a) {
                    self.outer = 0;
                }
            }
            _ => self.outer = 0,
        }
    }

    /// Behavior 13: the attack sequencer (outer 2).
    fn sequencer(&mut self, d: f32, v: &View, rng: &mut Rng, a: &mut Act) {
        let Some(k) = self.selected else {
            self.outer = 0;
            return;
        };
        let s = self.p.slots[k];
        let [ka, kb, kc, kd, ke] = std::array::from_fn(|n| slot_anim(k, n as u32));
        if s.orient && (1..=3).contains(&self.seq) {
            a.face = Some((v.hero, 1.0));
        }
        let cos = dot(v.facing, unit(sub(v.hero, v.position)));
        let strike = |e: &mut Self, a: &mut Act| -> bool {
            if v.contact && !e.hit {
                e.hit = true;
                a.strike = Some(Strike {
                    slot: k,
                    damage: s.damage,
                    knockback: s.knockback,
                    unblockable: s.flags & 0x01 != 0,
                    force_blocked: s.flags & 0x20 != 0,
                });
                return true;
            }
            false
        };
        let t = len(sub(v.position, self.start));
        match self.seq {
            0 => {
                let mut face_test = |e: &mut Self, a: &mut Act| {
                    if cos >= 0.95 {
                        e.seq = 1;
                    } else {
                        a.face = Some((v.hero, 0.1));
                        a.anim = Some(e.idle(false, v, rng));
                    }
                };
                if s.kind == MELEE && d > s.range {
                    self.walk(0, v.hero, v, a);
                    if !v.hero_on_mesh {
                        face_test(self, a);
                    }
                } else if v.hero_on_mesh {
                    self.seq = 1;
                } else {
                    face_test(self, a);
                }
            }
            1 => {
                a.rate = Some(s.wind_speed);
                if self.through(ka, v, a) {
                    if (v.has)(kb) {
                        self.through(kb, v, a);
                        self.seq = 2;
                    } else {
                        self.seq = 5;
                    }
                }
            }
            2 => {
                a.rate = Some(s.strike_speed);
                let done = self.through(kb, v, a);
                if s.kind == MELEE && strike(self, a) {
                    self.seq = 3;
                } else if done {
                    self.seq = 5;
                }
            }
            3 => {
                if (v.has)(kc) {
                    if self.through(kc, v, a) {
                        self.seq = 5;
                    }
                } else {
                    let done = self.through(kb, v, a);
                    if !self.reentered && v.contact && !done {
                        self.reentered = true;
                        self.hit = false;
                        self.seq = 2;
                    } else if done {
                        self.seq = 5;
                    }
                }
            }
            4 => {
                if self.through(kd, v, a) {
                    self.seq = 5;
                }
            }
            5 => {
                a.rate = Some(s.recover_speed);
                if s.flags & 0x08 != 0 || self.through(ke, v, a) {
                    self.end_path(k, rng);
                }
            }
            6 => {
                self.start = v.position;
                a.rate = Some(s.wind_speed);
                if self.through(ka, v, a) {
                    if (v.has)(kb) {
                        self.through(kb, v, a);
                        self.seq = 7;
                        self.ticks = 0;
                    } else {
                        self.seq = 12;
                    }
                }
            }
            7 => {
                self.ticks += 1;
                self.through(kb, v, a);
                if s.charge != 2 && strike(self, a) {
                    self.seq = 8;
                } else if self.ticks > 100 && v.speed <= 0.001 {
                    self.seq = 12;
                } else {
                    let c = self.charge[k];
                    match s.charge {
                        2 if d <= s.range => self.seq = 8,
                        3 if t >= c || d <= s.range || v.speed <= 0.1 => self.seq = 12,
                        3 => {}
                        _ if t >= c => self.seq = 12,
                        _ => {}
                    }
                }
            }
            8 => {
                let done = self.through(kc, v, a);
                match s.charge {
                    0 | 1 => {
                        if t >= self.charge[k] || v.blocked {
                            self.seq = 12;
                        }
                    }
                    2 => {
                        strike(self, a);
                        if done {
                            self.seq = 12;
                        }
                    }
                    _ => {
                        if done {
                            self.seq = 12;
                        }
                    }
                }
            }
            12 | 15 => {
                if self.through(ke, v, a) {
                    self.end_path(k, rng);
                }
            }
            13 => {
                if d > s.range {
                    self.walk(0, v.hero, v, a);
                } else {
                    self.seq = 14;
                    self.loop_count = 60;
                }
            }
            14 => {
                if (v.has)(kb) {
                    a.anim = Some(kb);
                }
                if self.loop_count == 0 {
                    a.strike = Some(Strike { slot: k, damage: s.damage, knockback: s.knockback, unblockable: true, force_blocked: false });
                    self.loop_count = rng.range(s.loop_min, s.loop_max);
                } else {
                    self.loop_count -= 1;
                }
                if d > s.loop_hold {
                    self.seq = 15;
                }
            }
            _ => self.end_path(k, rng),
        }
        // Chase stop (Behavior 48.6).
        if self.outer == 2 && self.seq <= 3 && d < v.radii {
            a.stop = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton() -> Params {
        // SPEC-0038 test vectors: Dumb Skeleton, Awareness 65, band 3 to 5, chance 50 at 3 and 1
        // at 5; two melee slots with trigger band 0 to 10.
        let mut p = Params {
            approach: 1,
            hostile: true,
            display_default: true,
            awareness: 65,
            awareness_timer: 0.0,
            chance_near: 50.0,
            chance_far: 1.0,
            range_near: 3.0,
            range_far: 5.0,
            view_angle: 0.707,
            rush: (2.0, 6.0),
            attack_timer_min: 15.0,
            attack_timer_max: 30.0,
            disappear: 60,
            ..Params::default()
        };
        for s in &mut p.slots {
            s.kind = UNUSED;
        }
        p.slots[1] = Slot { kind: MELEE, trigger_max: 10.0, arc: 0.71, chance: 40.0, delay_min: 60.0, delay_max: 120.0, charge: -1, range: 3.0, damage: 6.0, ..Slot::default() };
        p.slots[0] = Slot { kind: MELEE, trigger_max: 10.0, arc: 0.92, chance: 60.0, delay_min: 30.0, delay_max: 240.0, charge: -1, range: 3.0, damage: 6.0, ..Slot::default() };
        p
    }

    fn view<'a>(hero: Vec3, has: &'a dyn Fn(u32) -> bool) -> View<'a> {
        View {
            position: [0.0; 3],
            facing: [1.0, 0.0, 0.0],
            hero,
            hero_facing: [-1.0, 0.0, 0.0],
            hero_attackable: true,
            bands: (1.0, 0.5),
            sight: true,
            route: true,
            hero_on_mesh: true,
            waypoint: [0.0; 3],
            blocked: false,
            speed: 0.0,
            contact: false,
            radii: 2.0,
            clip: Clip::default(),
            has,
        }
    }

    #[test]
    fn band_factor_lookup() {
        let f1 = [0.75, 1.5, 1.75, 0.5, 0.3, 0.5, 0.7];
        let f2 = [0.25, 0.75, 0.9, 0.3, 0.0, 0.0, 0.0];
        assert_eq!(band_factors(4, &f1, &f2), (1.75, 0.9));
        assert_eq!(band_factors(7, &f1, &f2), (1.0, 0.5));
        assert_eq!(band_factors(1, &f1, &f2), (0.7, 0.0));
    }

    #[test]
    fn notice_band_and_chance() {
        let has = |_| true;
        let mut rng = Rng::seeded(1);
        let mut e = Enemy::new(skeleton(), Spawn { default_state: 5, ..Spawn::default() }, &mut rng);
        e.awareness = 0;
        // Closer than the near edge: noticed whenever roll A passes (65 of 100).
        let close = (0..2000).filter(|_| e.notice(&view([2.0, 0.0, 0.0], &has), &mut rng)).count();
        assert!((1150..1450).contains(&close), "{close}");
        // Beyond the far edge: never; in hero action state 1 (F1 0.7) 4 units is beyond 3.5.
        assert!((0..500).all(|_| !e.notice(&view([5.5, 0.0, 0.0], &has), &mut rng)));
        let mut v = view([4.0, 0.0, 0.0], &has);
        v.bands = (0.7, 0.0);
        assert!((0..500).all(|_| !e.notice(&v, &mut rng)));
        // Only the countdown's zero tick can notice.
        e.awareness = 3;
        assert!((0..500).all(|_| !e.notice(&view([2.0, 0.0, 0.0], &has), &mut rng)));
    }

    #[test]
    fn hit_sends_to_agro_doubled_then_attacks() {
        let has = |id: u32| id != 22;
        let mut rng = Rng::seeded(7);
        let mut e = Enemy::new(skeleton(), Spawn { default_state: 5, ..Spawn::default() }, &mut rng);
        assert_eq!(e.state, 0);
        let (dmg, _) = e.take_hit(6, [2.5, 0.0, 0.0], &mut rng);
        assert_eq!((dmg, e.state, e.outer, e.react), (12, 19, 10, 0));
        let (dmg, _) = e.take_hit(6, [2.5, 0.0, 0.0], &mut rng);
        assert_eq!(dmg, 6, "the next hit in state 19 is not doubled");
        // Finish the reaction clip 57: outer 0, then rush set-up.
        let mut v = view([2.5, 0.0, 0.0], &has);
        v.clip = Clip { action: 57, progress: 0.5, finished: false };
        e.tick(&v, &mut rng);
        e.tick(&v, &mut rng);
        v.clip = Clip { action: 57, progress: 1.0, finished: true };
        e.tick(&v, &mut rng);
        assert_eq!(e.outer, 0);
        e.tick(&v, &mut rng);
        assert_eq!((e.outer, e.inner), (1, 1));
        // Within the band, facing the hero: a slot is eventually chosen and the sequencer runs.
        let mut chosen = false;
        for _ in 0..400 {
            e.tick(&v, &mut rng);
            if e.outer == 2 {
                chosen = true;
                break;
            }
        }
        assert!(chosen);
        assert_eq!(e.seq, 1, "within range on the mesh: wind-up at once");
    }

    #[test]
    fn strike_hits_once_then_recovers() {
        let has = |_| true;
        let mut rng = Rng::seeded(3);
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        e.state = 19;
        e.selected = Some(0);
        e.enter(2, 1);
        let mut v = view([2.0, 0.0, 0.0], &has);
        let (a, b) = (slot_anim(0, 0), slot_anim(0, 1));
        // Wind-up A plays to its end, then B starts.
        e.tick(&v, &mut rng);
        v.clip = Clip { action: a, progress: 0.5, finished: false };
        e.tick(&v, &mut rng);
        v.clip = Clip { action: a, progress: 1.0, finished: true };
        let act = e.tick(&v, &mut rng);
        assert_eq!((e.seq, act.anim), (2, Some(b)));
        v.clip = Clip { action: b, progress: 0.2, finished: false };
        v.contact = true;
        let act = e.tick(&v, &mut rng);
        assert_eq!(act.strike.map(|s| s.damage), Some(6.0));
        assert_eq!(e.seq, 3);
        // C plays to its end, then E, then the end path.
        let c = slot_anim(0, 2);
        e.tick(&v, &mut rng);
        v.clip = Clip { action: c, progress: 0.5, finished: false };
        assert!(e.tick(&v, &mut rng).strike.is_none());
        v.clip = Clip { action: c, progress: 1.0, finished: true };
        e.tick(&v, &mut rng);
        assert_eq!(e.seq, 5);
        let ee = slot_anim(0, 4);
        e.tick(&v, &mut rng);
        v.clip = Clip { action: ee, progress: 0.5, finished: false };
        e.tick(&v, &mut rng);
        v.clip = Clip { action: ee, progress: 1.0, finished: true };
        e.tick(&v, &mut rng);
        assert_eq!(e.outer, 0);
        assert!((15..=30).contains(&e.recovery));
    }

    #[test]
    fn strafe_hold_and_circle() {
        let has = |_| true;
        let mut rng = Rng::seeded(11);
        let mut p = skeleton();
        p.approach = 4;
        p.strafe = (4.0, 15.0);
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.enter(4, 0);
        e.desired = 9.5;
        let v = view([0.0, 0.0, 10.0], &has);
        // Strafe circle vector: enemy at the origin, hero at (0, 0, 10): inner 8 steers to
        // (2, 0, 0), inner 9 to (-2, 0, 0).
        e.inner = 8;
        e.hold = 50;
        let mut a = Act::default();
        e.strafe(10.0, [0.0, 0.0, 1.0], &v, &mut rng, &mut a);
        assert_eq!(a.go, Some(Go::Facing([0.0, 0.0, 10.0])));
        assert_eq!(a.anim, Some(30));
        e.inner = 10;
        e.hold = 1;
        e.lock = 50;
        e.strafe(10.0, [0.0, 0.0, 1.0], &v, &mut rng, &mut Act::default());
        assert_eq!(e.inner, 10, "locked: no circling");
        let mut vb = view([0.0, 0.0, 10.0], &has);
        vb.blocked = true;
        e.inner = 9;
        e.hold = 50;
        e.strafe(10.0, [0.0, 0.0, 1.0], &vb, &mut rng, &mut Act::default());
        assert_eq!((e.inner, e.lock), (10, 120));
    }

    #[test]
    fn emerge_from_hiding() {
        // Behavior 40 vector: emerge distance 10.0, hero in action state 1 (F1 0.7): 7.0 units
        // leaves it hidden; 6.99 starts sub 1; the next tick plays 10; finished: agro.
        let has = |_| true;
        let mut rng = Rng::seeded(2);
        let spawn = Spawn { default_state: 8, emerge_distance: 10.0, ..Spawn::default() };
        let mut e = Enemy::new(skeleton(), spawn, &mut rng);
        assert_eq!(e.state, 8);
        let mut v = view([7.0, 0.0, 0.0], &has);
        v.bands = (0.7, 0.0);
        assert_eq!(e.tick(&v, &mut rng).anim, Some(9), "hidden at home");
        assert_eq!(e.emerge, 0);
        v.hero = [6.99, 0.0, 0.0];
        e.tick(&v, &mut rng);
        assert_eq!(e.emerge, 1);
        assert_eq!(e.tick(&v, &mut rng).anim, Some(10));
        v.clip = Clip { action: 10, progress: 0.5, finished: false };
        e.tick(&v, &mut rng);
        v.clip = Clip { action: 10, progress: 1.0, finished: true };
        e.tick(&v, &mut rng);
        assert_eq!(e.state, 19);
    }

    #[test]
    fn corpse_countdown_and_sink() {
        let has = |_| true;
        let mut rng = Rng::seeded(5);
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        e.die();
        let mut v = view([9.0, 0.0, 0.0], &has);
        v.clip = Clip { action: 58, progress: 0.96, finished: false };
        let mut sunk = 0.0;
        let mut ticks = 0;
        while !e.gone {
            sunk += e.tick(&v, &mut rng).sink;
            ticks += 1;
        }
        // Timer 60: the countdown runs 59 down to -1 (61 ticks); 31 of them sink.
        assert_eq!(ticks, 61);
        assert!((sunk - 31.0 / 180.0).abs() < 1e-5, "{sunk}");
    }
}
