//! Common enemies, class 17 (SPEC-0038): the think loop's pre-step (awareness countdown, alert
//! flag, per-state switch), the notice check (Behavior 3) with the band factors (Behavior 18),
//! the state setter (Behavior 5), the idle picker (Behavior 8), the agro machine (Behavior 44:
//! set-up, rush, keep distance, strafe, search after lost sight, hit reactions), the normal
//! attack choice (Behavior 7), the distance draws (Behavior 4), the attack sequencer (Behavior 13:
//! melee, charge and loop states), the agro tail with the leash (Behavior 23), the chase stop
//! (Behavior 48.6), the take-hit hook (Behavior 37), the dead state (Behaviors 34 and 36) and the
//! melee and missile defences (outers 5 to 8, with the defence hold and exit).
//!
//! Not covered: the alarm groups and the group alert (Behaviors 3 and 10), socialize, the global
//! AI pause (state 20), the retarget countdown and flyer re-launch (state 22 is played but never
//! entered), the map search (Behaviors 9, 17, 18, 39, 42, 43), the named-target state 9 (it
//! stands and notices), the alarm-group emerge of state 8, wandering in
//! states 5 to 7 (they stand and notice), retreat home (state 14) and the distance-hide gate,
//! the counter-attack choice (Behavior 14), the
//! confused outer 11, the attack manager and the wait in line (outer 12), projectiles (missile
//! slots play their strike with no shot), the grab-and-throw, theft and Mote drain,
//! ghosts, reanimation (state 16 of rows 224, 226, 227) and drops and coins. The alerted search
//! (state 2) only aims at the last-seen point until its countdown ends (R-0015).
//!
//! Flyers (SPEC-0038 "Flyer", Behaviors 2 step 4, 34 step 2, 37 step 7, 41, 44 and 48; SPEC-0040
//! Behavior 14D item 7 and Behavior 21): the airborne test, the retarget countdown with the
//! re-launch (state 22), grounding on death and on hit kind 0, the flying animation ids, the
//! flying row speed along the movement class axis and the goal height the host's keeper follows.
//!
//! Statuses (SPEC-0029 Behavior 27): the apply with its element immunity and defaults, the
//! stunned state 4, poison and fire hits, the slow and speed scales and the outgoing curse and
//! strength scales. Not covered: confusion's outer 11 (the timer runs but the enemy is not
//! confused) and the status display.

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
    /// The blocked-charge kind (+0x2AC), the stun range (max +0x27C, min +0x28C) and type
    /// (+0x29C) of a blocked charge.
    pub blocked_charge: i32,
    pub stun_max: f32,
    pub stun_min: f32,
    pub stun_type: i32,
    /// The attack's damage alignment (its element; SPEC-0029 Layout 4).
    pub alignment: i32,
    /// Spawn count (+0x33C) and the missile values of the spawners.
    pub spawn_count: i32,
    pub missile: crate::projectile::Missile,
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
    /// The flyer words: turn (0x88), idle and agro velocity (0x8C, 0x90), ground timer max and
    /// min (0x98, 0x9C) and Max Fly Height (0xB4).
    pub fly_turn: f32,
    pub fly_idle: f32,
    pub fly_agro: f32,
    pub ground_max: f32,
    pub ground_min: f32,
    pub max_fly: f32,
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
    /// Alignment (+0x3C): the element of every projectile the enemy fires.
    pub element: i32,
    pub slots: [Slot; 4],
    pub defence: Defence,
}

/// The defence words of the monster row (SPEC-0038 Layout section 2, 0xE0 to 0x108).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Defence {
    /// Missile defend time (0xE0): the defence hold set when a defence starts, and the missile
    /// look-ahead in ticks.
    pub hold: i32,
    /// Missile dodge and block chances (0xE4, 0xE8).
    pub missile: (f32, f32),
    /// Melee block and dodge chances for quick, normal and heavy swings (0xEC to 0xF4, 0xF8 to
    /// 0x100).
    pub block: [f32; 3],
    pub dodge: [f32; 3],
    /// Defence timer min and max (0x104, 0x108), drawn when a defence ends.
    pub exit: (f32, f32),
    /// Blocking Angle (0x10C; SPEC-0036 Hit detection step 4).
    pub angle: f32,
}

/// The clip entry a locomotion mode plays from the seven values of the shared slot setter
/// (SPEC-0029 Behavior 2; R-0067): modes 0 to 4 read values 1 to 5, mode 6 the 6th and mode 5
/// the 7th; -1 (or any other mode) plays nothing.
pub fn slot_entry(slots: &[i32; 7], mode: u8) -> Option<u32> {
    let v = match mode {
        0..=4 => slots[mode as usize],
        5 => slots[6],
        6 => slots[5],
        _ => -1,
    };
    u32::try_from(v).ok()
}

/// The hero's reach and facing arc for the melee defence (hero +0x14B8 and +0x14B4, set at
/// spawn: 8.0 units and cosine 0.5 for both heroes).
pub const DEFENCE_REACH: f32 = 8.0;
pub const DEFENCE_ARC: f32 = 0.5;

/// The swing class of a hero attack sub-state (R-0017): 0 quick, 1 normal, 2 heavy, None for a
/// sub-state that never draws a block or dodge. `hero` 0 is Tor, 1 Roberta.
pub fn swing_class(hero: usize, sub: u8) -> Option<u8> {
    match (hero, sub) {
        (0, 5 | 6) => Some(0),
        (0, 2 | 3 | 9) => Some(1),
        (0, 4 | 7 | 8 | 12) => Some(2),
        (1, 5 | 6 | 7 | 10) => Some(0),
        (1, 2 | 3 | 8 | 11 | 12) => Some(1),
        (1, 4) => Some(2),
        _ => None,
    }
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
    /// The hero guards and faces this enemy (dot of the facings at most -0.5).
    pub hero_blocks: bool,
    /// The hero's melee swing class while it swings (swing_class), None otherwise.
    pub hero_swing: Option<u8>,
    /// A live hero shot's path over the look-ahead meets this enemy's body capsule, and whether
    /// any hero shot is live.
    pub missile_threat: bool,
    pub hero_shot_live: bool,
    /// Event bits on the current frame: 0x20 is the release event (the missile slot's spawn
    /// event, SPEC-0005 table B), and 0x1 to 0x10 on that frame pick emit points 0 to 4.
    pub spawn_bits: u32,
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

/// A missile slot's spawn event fired: spawner for slot k from emit point n, with the shot
/// counter before this shot (Behavior 13, state 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotRequest {
    pub slot: usize,
    pub emit: usize,
    pub counter: u32,
}

/// A strike on the hero by slot k (Behavior 13, strike test or loop damage).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub slot: usize,
    pub damage: f32,
    pub knockback: f32,
    pub unblockable: bool,
    pub force_blocked: bool,
    /// The strike was blocked (the hero guards facing the enemy, or flag 0x20).
    pub blocked: bool,
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
    pub shot: Option<ShotRequest>,
    /// Discard this tick's root step (chase stop).
    pub stop: bool,
    /// Sink the corpse by this much along world up.
    pub sink: f32,
    /// This tick's poison and fire hits (Behavior 27.2 steps 3 and 8), each a take-hit.
    pub status_hits: [Option<i32>; 2],
    /// A flyer's row-speed step F in the character's own basis (x side B1, z forward B3), per
    /// tick (SPEC-0040 Behavior 14D item 7).
    pub fly: Option<Vec3>,
}

/// SPEC-0040 Behavior 21, the flyer goal height. `ambush` is the ambush word (+0xBC0); while it
/// is 0 the home height is used. Can Fly 1: hero y + min(H, 0.2 x the horizontal distance), at
/// least the ground + 2.0, then the home height while ambush is 0. Can Fly 2: hero y (or the home
/// height), at most ground + H. `ground` is the ground under the goal (None: no clamp).
pub fn fly_goal(can_fly: i32, max_fly: f32, hero: Vec3, at: Vec3, home: Vec3, ambush: i32, ground: Option<f32>) -> f32 {
    match can_fly {
        1 => {
            let mut y = hero[1] + max_fly.min(0.2 * hdist(at, hero));
            if let Some(g) = ground {
                y = y.max(g + 2.0);
            }
            if ambush == 0 {
                y = home[1];
            }
            y
        }
        2 => {
            let y = if ambush == 0 { home[1] } else { hero[1] };
            match ground {
                Some(g) if y - g > max_fly => g + max_fly,
                _ => y,
            }
        }
        _ => at[1],
    }
}

/// The flying form of an animation id (SPEC-0038 Behaviors 8, 44 and 48; SPEC-0040 Behavior 6
/// mode entries and the idle pick): locomotion (2 wander 98, 4 patrol 99, 11 state 9 100, 20 state
/// 1 154, rush 21/23 as 101/102, keep distance 24/26 as 107/108, strafe 28/32/29/30 as 103 to 106
/// for movement classes 7 to 10), the holds (22 155, 25 156, 31 132), the defences (55 140, 56
/// 150, 126 151, 53 138, 54 139), the reactions (60 158, 61 159, 62 160, 64 161), victory 73 163,
/// the base idle 0 as 97 and the fidgets 115 to 119 as 142. The flying strafe order (29 as 105,
/// 30 as 106) follows the class numbers of the gait swap (Medium-Low).
pub fn fly_anim(id: u32) -> u32 {
    match id {
        0 => 97,
        2 => 98,
        4 => 99,
        11 => 100,
        20 => 154,
        21 => 101,
        23 => 102,
        24 => 107,
        26 => 108,
        28 => 103,
        32 => 104,
        29 => 105,
        30 => 106,
        22 => 155,
        25 => 156,
        31 => 132,
        55 => 140,
        56 => 150,
        126 => 151,
        53 => 138,
        54 => 139,
        60 => 158,
        61 => 159,
        62 => 160,
        64 => 161,
        73 => 163,
        115..=119 => 142,
        n => n,
    }
}

/// The movement-class axis of a flying locomotion id in the character's basis: forward (+B3)
/// for 98 to 101, 103, 107 and 154; backward (-B3) for 102, 104 and 108; +B1 for 105 and -B1 for
/// 106 (SPEC-0040 Behavior 14D item 7).
fn fly_axis(id: u32) -> Option<Vec3> {
    match id {
        98..=101 | 103 | 107 | 154 => Some([0.0, 0.0, 1.0]),
        102 | 104 | 108 => Some([0.0, 0.0, -1.0]),
        105 => Some([1.0, 0.0, 0.0]),
        106 => Some([-1.0, 0.0, 0.0]),
        _ => None,
    }
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
    /// The shot counter and the spawn-pending flag of a missile slot.
    shots: u32,
    pending: bool,
    idle_timer: i32,
    idle_variant: Option<u32>,
    through: Option<u32>,
    through_seen: bool,
    force: bool,
    /// The 14 status timers in ticks (0x808 to 0x83C) and the poison and fire (amount, period)
    /// words (0xCB4 to 0xCC0; -1 the defaults).
    timers: [i32; 14],
    poison: (i32, i32),
    fire: (i32, i32),
    /// The defence hold (0xAB8) and the melee-defence swing latch (0xB88).
    defence_hold: i32,
    swing_latch: bool,
    /// Base actor state 20 (blocking): set by the melee block, read by the hero's swing.
    pub blocking: bool,
    /// The pending block-hit flag: the melee block plays 126 once.
    pub block_hit: bool,
    /// The retarget (grounded) countdown (0xBF4): -1 idle; a grounding sets it positive.
    pub retarget: i32,
    /// The ambush word (+0xBC0; SPEC-0040 Behavior 21): 0 keeps a flyer at its home height.
    pub ambush: i32,
    /// A navmesh re-pick is pending (passed out through the next act).
    repick: bool,
    /// DisableInvisibility (SPEC-0030 row 121; +0xE60): an invisible kind (Display in Default
    /// 0) loses its hit and lock veto, and SetAI_Agro enters state 2, not 21.
    pub visible: bool,
}

impl Enemy {
    /// Creation: the distance draws (Behavior 4), the idle timer, and the think loop's start
    /// (Behavior 1, step 1: the setter with 0; state 8 when the default-state word is 8).
    pub fn new(p: Params, spawn: Spawn, rng: &mut Rng) -> Self {
        let default_state = spawn.default_state;
        let mut e = Self {
            visible: false,
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
            timers: [0; 14],
            poison: (-1, -1),
            fire: (-1, -1),
            defence_hold: 0,
            swing_latch: false,
            blocking: false,
            block_hit: false,
            ticks: 0,
            loop_count: 0,
            reentered: false,
            hit: false,
            shots: 0,
            pending: false,
            idle_timer: 0,
            idle_variant: None,
            through: None,
            through_seen: false,
            force: false,
            retarget: -1,
            ambush: 0,
            repick: false,
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
                self.shots = 0;
                self.pending = false;
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
        // Behavior 37 step 7: no hit reaction while stunned (status 0).
        if !self.status(0) && !(self.outer == 10 && self.react == 2 && self.stun_type == 1) {
            let kind = if self.p.can_fly != 0 && self.retarget < 1 {
                match self.p.flying_hit {
                    1 => Some(7),
                    0 | 2 => Some(6),
                    _ => None,
                }
            } else {
                Some(0)
            };
            // Behavior 44 kind 0: a flyer's grounded countdown starts (the navmesh pick follows
            // through the host on the next tick).
            if kind == Some(0) && self.p.can_fly != 0 {
                self.ground(rng);
            }
            if let Some(kind) = kind {
                if let (2, 14, Some(k)) = (self.outer, self.seq, self.selected) {
                    self.end_path(k, rng);
                    self.selected = None;
                }
                self.enter(10, kind);
            }
        }
        self.hit_recovery = 90;
        // Step 13: the force-start only when not stunned.
        self.force = !self.status(0);
        self.alert = true;
        if self.p.anger {
            self.anger_count += 1;
        }
        (damage, attacker)
    }

    /// Behavior 34 (no reanimation, no flyer grounding, no drops): base state 16 and the death
    /// disappear countdown.
    /// SetAI_Agro (SPEC-0030 row 115): a living enemy goes to state 2, the alerted search (21
    /// when the row's Display in Default word is 0), with the search countdown drawn from the
    /// row's search range and the last-seen point at the hero. Returns the animation it
    /// requests (18).
    pub fn agro_on(&mut self, hero: Vec3, rng: &mut Rng) -> Option<u32> {
        if self.dead {
            return None;
        }
        self.previous = self.state;
        self.state = if self.p.display_default || self.visible { 2 } else { 21 };
        self.search = rng.frange(self.p.search_min, self.p.search_max) as i32;
        self.last_seen = hero;
        Some(18)
    }

    /// A summoned enemy's start (SPEC-0029 Behavior 31): the agro state 19, or for the minion
    /// keys (and the arrival flag) animation 82 and state 21, the search with the hero as the
    /// last-seen point. Returns the animation it requests.
    pub fn summoned(&mut self, arrive: bool, hero: Vec3, rng: &mut Rng) -> Option<u32> {
        if self.dead {
            return None;
        }
        self.previous = self.state;
        self.through = None;
        if arrive {
            self.state = 21;
            self.search = rng.frange(self.p.search_min, self.p.search_max) as i32;
            self.last_seen = hero;
            Some(82)
        } else {
            self.state = 19;
            self.outer = 0;
            self.recovery = 0;
            self.cooldown = [0; 4];
            None
        }
    }

    /// Behavior 34 step 2 with the death countdown: a can-fly 1 enemy that dies airborne is
    /// grounded (the countdown from the ground timer, max not reached).
    pub fn die(&mut self, rng: &mut Rng) {
        if !self.dead {
            if self.p.can_fly == 1 && self.retarget <= 0 {
                self.ground(rng);
            }
            self.dead = true;
            self.disappear = self.p.disappear;
        }
    }

    /// The grounded countdown: trunc(float[ground min, ground max)).
    fn ground(&mut self, rng: &mut Rng) {
        self.retarget = rng.frange(self.p.ground_min, self.p.ground_max) as i32;
        self.repick = true;
    }

    /// A navmesh re-pick requested since the last call (grounding, retarget).
    pub fn take_repick(&mut self) -> bool {
        std::mem::take(&mut self.repick)
    }

    /// SPEC-0040 Behavior 20 item 1, the class-state eligibility veto for the hero's candidate
    /// list, the branch of rows whose Display in Default word is 0: vetoed in the configured
    /// default state, or in state 21 unless animation 82 plays (`clip` the current action). The
    /// active-hero actor-table exception and the Display-in-Default branch (+0xE3C, the ambush
    /// word, +0xC4, +0xC8, state 14) are not built.
    pub fn lock_vetoed(&self, clip: u32) -> bool {
        !self.p.display_default && !self.visible && (self.state == self.default_state as u32 || (self.state == 21 && clip != 82))
    }

    /// The flyer test: can-fly 1 with the retarget countdown at most 0, or can-fly 2.
    pub fn airborne(&self) -> bool {
        (self.p.can_fly == 1 && self.retarget <= 0) || self.p.can_fly == 2
    }

    /// A strike left the hero at 0 HP: the victory pose (state 10).
    pub fn hero_down(&mut self, v: &View, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        self.set(10, v, rng, &mut a);
        a
    }

    /// SPEC-0029 Behavior 27.1, the enemy's status apply: code, duration in whole seconds (-1:
    /// the default) and the poison or fire amount and period (-1: the default). Nothing on a
    /// dead enemy or one immune by its target type; stun enters state 4 at once. Returns whether
    /// a timer was set.
    pub fn apply_status(&mut self, code: i32, seconds: i32, amount: i32, period: i32) -> bool {
        if self.dead {
            return false;
        }
        let t = self.p.element;
        let default = match code {
            0 if t != 4 => 5,
            1 if t != 6 => 60,
            2 if t != 2 => 120,
            3 if t != 5 => 60,
            6 if t != 3 => 30,
            7 if t != 1 => 5,
            4 | 5 => 60,
            11 => 30,
            _ => return false,
        };
        let seconds = if seconds == -1 { default } else { seconds };
        if seconds < 0 {
            return false;
        }
        match code {
            0 => {
                self.previous = self.state;
                self.state = 4;
            }
            2 => self.poison = (amount, period),
            7 => self.fire = (amount, period),
            _ => {}
        }
        self.timers[code as usize] = seconds * 60;
        true
    }

    /// Whether status `code` is active (its timer is 1 or more).
    pub fn status(&self, code: usize) -> bool {
        self.timers.get(code).is_some_and(|&t| t >= 1)
    }

    /// The outgoing damage scale of the enemy's projectiles (Behavior 13): x 0.5 under curse, x
    /// 1.5 under strength.
    pub fn shot_scale(&self) -> f32 {
        let mut g = 1.0;
        if self.status(1) {
            g *= 0.5;
        }
        if self.status(4) {
            g *= 1.5;
        }
        g
    }

    /// The attacker's status scale G of a strike on a hero (SPEC-0036 Enemy attack damage to a
    /// hero): 0.75 under curse, 1.5 under strength, 1.125 with both.
    pub fn strike_scale(&self) -> f32 {
        let mut g = 1.0;
        if self.status(1) {
            g *= 0.75;
        }
        if self.status(4) {
            g *= 1.5;
        }
        g
    }

    /// Behavior 27.2 steps 1 to 4 and 8 (the timers, stun, poison and fire): returns the poison
    /// and fire hits of this tick. The scales of steps 4 and 6 are applied to the act after the
    /// think tick (status_scale).
    fn status_tick(&mut self) -> [Option<i32>; 2] {
        for t in self.timers.iter_mut() {
            if *t >= 1 {
                *t -= 1;
            }
        }
        if self.status(0) {
            self.state = 4;
        }
        let dot = |timer: i32, (amount, period): (i32, i32), (a0, p0): (i32, i32)| {
            let p = if period < 0 { p0 } else { period };
            let a = if amount < 0 { a0 } else { amount };
            (timer >= 1 && p > 0 && timer % (p * 60) == 0).then_some(a)
        };
        [dot(self.timers[2], self.poison, (2, 4)), dot(self.timers[7], self.fire, (2, 2))]
    }

    /// Steps 4 and 6: slow halves the scale when moving or attacking; speed doubles it when
    /// moving (tests A and B read as moving and attacking; Medium); stun sets it to 0.
    fn status_scale(&self, a: &Act) -> f32 {
        if self.status(0) {
            return 0.0;
        }
        let moving = a.go.is_some();
        let mut s = 1.0;
        if self.status(3) && (moving || self.outer == 2) {
            s *= 0.5;
        }
        if self.status(5) && moving {
            s *= 2.0;
        }
        s
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
        let flying = self.airborne();
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
        let dots = self.status_tick();
        let mut a = self.think(v, rng);
        a.status_hits = dots;
        if self.airborne() {
            self.fly_step(&mut a);
        }
        let s = self.status_scale(&a);
        if s != 1.0 {
            a.rate = Some(a.rate.unwrap_or(1.0) * s);
        }
        a
    }

    /// The flying forms and the row speed (SPEC-0040 Behavior 14D item 7): the animation's flying
    /// id (141 for the agro idle), then F = v / 30 along the movement class axis while a flying
    /// locomotion id plays (v the agro velocity in state 19, else the idle velocity); the
    /// sequencer's states 7, 8, 9 and 12 push forward whatever the animation; the dodges 53, 54,
    /// 138 and 139 stop it.
    fn fly_step(&self, a: &mut Act) {
        if let Some(id) = a.anim {
            a.anim = Some(if self.state == 19 && matches!(id, 0 | 1) { 141 } else { fly_anim(id) });
        }
        let v = if self.state == 19 { self.p.fly_agro } else { self.p.fly_idle } / 30.0;
        let axis = if self.state == 19 && self.outer == 2 && matches!(self.seq, 7..=9 | 12) {
            Some([0.0, 0.0, 1.0])
        } else {
            a.anim.and_then(fly_axis)
        };
        a.fly = match a.anim {
            Some(53 | 54 | 138 | 139) => None,
            _ => axis.map(|d| d.map(|c| c * v)),
        };
    }

    /// The goal height for the host's height keeper (SPEC-0040 Behavior 14D item 7): can-fly 1
    /// in sequencer states 7 to 9 and 12 aims at the hero's height + 2.0, otherwise the goal of
    /// Behavior 21 (the move goal is taken as the enemy's own position).
    pub fn fly_target(&self, v: &View, ground: Option<f32>) -> f32 {
        if self.p.can_fly == 1 && self.state == 19 && self.outer == 2 && matches!(self.seq, 7..=9 | 12) {
            return v.hero[1] + 2.0;
        }
        fly_goal(self.p.can_fly, self.p.max_fly, v.hero, v.position, self.spawn.home, self.ambush, ground)
    }

    fn think(&mut self, v: &View, rng: &mut Rng) -> Act {
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
        // Step 4, the retarget countdown: at 0, unless the Action-19 gate blocks (state 19 with
        // the sequencer in states 0 to 3), the navmesh is picked again and the state becomes 22
        // directly; then the countdown goes down (at -1 nothing happens).
        if self.retarget == 0 && !(self.state == 19 && self.outer == 2 && self.seq <= 3) {
            self.repick = true;
            self.previous = self.state;
            self.state = 22;
            self.through = None;
        }
        if self.retarget >= 0 {
            self.retarget -= 1;
        }
        match self.state {
            0 => self.set(self.default_state as u32, v, rng, &mut a),
            // State 4 (stunned): back to agro once status 0 is no longer active.
            4 if !self.status(0) => self.set(19, v, rng, &mut a),
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
            // Trigger special: 1 only while airborne, 2 only while not.
            match s.special as i32 {
                1 if !self.airborne() => continue,
                2 if self.airborne() => continue,
                _ => {}
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

    /// Behavior 44 missile and melee defence (before the other choices). Each starts outer 6 or
    /// 8 (missile) or 5 or 7 (melee) and sets the defence hold.
    fn defend(&mut self, v: &View, rng: &mut Rng) {
        let d = self.p.defence;
        // The melee latch follows the hero's swing every tick, so a swing is judged once.
        let swinging = v.hero_swing.is_some();
        let edge = swinging && !self.swing_latch;
        self.swing_latch = swinging;
        if self.outer != 10 && !(self.outer == 2 && self.seq == 5) && self.defence_hold < 1 && v.missile_threat {
            let mut to = None;
            if rng.range(1, 100) < d.missile.0 as i32 {
                to = Some(6);
            }
            if rng.range(1, 100) < d.missile.1 as i32 {
                to = Some(8);
            }
            if let Some(o) = to {
                self.start_defence(o, rng);
                return;
            }
        }
        let gate = self.outer != 10 && !(self.outer == 2 && ![0, 1, 5].contains(&self.seq)) && self.defence_hold < 1;
        if let (true, true, Some(class)) = (edge, gate, v.hero_swing) {
            let off = sub(v.position, v.hero);
            let reach = len(off) < DEFENCE_REACH && dot(v.hero_facing, unit(off)) > DEFENCE_ARC;
            if reach {
                let (r1, r2) = (rng.range(1, 100), rng.range(1, 100));
                let c = class.min(2) as usize;
                if r1 <= d.block[c] as i32 {
                    self.start_defence(7, rng);
                } else if r2 < d.dodge[c] as i32 {
                    self.start_defence(5, rng);
                }
            }
        }
    }

    fn start_defence(&mut self, outer: u32, rng: &mut Rng) {
        self.outer = outer;
        // Outer 6 picks its dodge, 53 or 54, by int[0, 1].
        self.inner = if outer == 6 { 53 + rng.range(0, 1) as u32 } else { 0 };
        self.through = None;
        self.block_hit = false;
        self.defence_hold = self.p.defence.hold;
    }

    /// The defence exit: outer 0, base state 1, defence hold = int[int(0x104), int(0x108)].
    fn defence_exit(&mut self, rng: &mut Rng) {
        self.outer = 0;
        self.blocking = false;
        let (lo, hi) = (self.p.defence.exit.0 as i32, self.p.defence.exit.1 as i32);
        self.defence_hold = rng.range(lo.min(hi), lo.max(hi));
    }

    /// Outers 5 to 8.
    fn defence(&mut self, v: &View, rng: &mut Rng, a: &mut Act) {
        match self.outer {
            5 | 6 => {
                let id = if self.outer == 5 { 55 } else { self.inner };
                if self.through(id, v, a) {
                    self.defence_exit(rng);
                }
            }
            7 => {
                if v.hero_swing.is_some() {
                    self.blocking = true;
                    if self.block_hit && (v.has)(126) {
                        if self.through(126, v, a) {
                            self.block_hit = false;
                        }
                        a.face = Some((v.hero, self.p.turn));
                    } else {
                        self.block_hit = false;
                        self.hold_anim(56, self.p.turn, v, a);
                    }
                } else {
                    self.defence_exit(rng);
                }
            }
            _ => {
                if v.hero_shot_live {
                    a.anim = Some(56);
                } else {
                    self.defence_exit(rng);
                }
            }
        }
    }

    /// Behavior 44: one agro tick.
    fn agro(&mut self, v: &View, rng: &mut Rng, a: &mut Act) {
        if self.recovery > 0 {
            self.recovery -= 1;
        }
        // Base state 20 holds only through the blocks (outers 7 and 8).
        if self.blocking && !matches!(self.outer, 7 | 8) {
            self.blocking = false;
        }
        if self.defence_hold > 0 {
            self.defence_hold -= 1;
        }
        self.defend(v, rng);
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
            5..=8 => self.defence(v, rng, a),
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

    /// State 2 of a missile slot: the spawn-pending flag and the spawn loop. True when the shot
    /// counter has reached the spawn count (go to state 3).
    fn missile_step(&mut self, k: usize, v: &View, a: &mut Act) -> bool {
        let active = v.spawn_bits & 0x20 != 0;
        if self.pending {
            if !active {
                self.pending = false;
            }
            return false;
        }
        if !active {
            return false;
        }
        let points = v.spawn_bits & 0x1F;
        let emit = if points == 0 { 0 } else { points.trailing_zeros() as usize };
        a.shot = Some(ShotRequest { slot: k, emit, counter: self.shots });
        self.shots += 1;
        if (self.shots as i32) < self.p.slots[k].spawn_count {
            self.pending = true;
            return false;
        }
        true
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
        let strike = |e: &mut Self, a: &mut Act| -> Option<bool> {
            if v.contact && !e.hit {
                e.hit = true;
                let unblockable = s.flags & 0x01 != 0;
                let force_blocked = s.flags & 0x20 != 0;
                let blocked = force_blocked || (!unblockable && v.hero_blocks);
                a.strike = Some(Strike { slot: k, damage: s.damage, knockback: s.knockback, unblockable, force_blocked, blocked });
                return Some(blocked);
            }
            None
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
                let struck = if s.kind == MELEE { strike(self, a) } else { None };
                if let Some(blocked) = struck {
                    self.seq = if blocked { 4 } else { 3 };
                } else if s.kind == MISSILE && self.missile_step(k, v, a) {
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
                let struck = if s.charge != 2 { strike(self, a) } else { None };
                if let Some(blocked) = struck {
                    self.seq = if blocked { 9 } else { 8 };
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
            9 => {
                // The blocked charge, by the slot's blocked-charge kind.
                match s.blocked_charge {
                    0 => {
                        self.through(kd, v, a);
                        if t >= self.charge[k] {
                            self.seq = 12;
                        }
                    }
                    1 => {
                        if self.through(kd, v, a) {
                            self.seq = 12;
                        }
                    }
                    2 => {
                        self.stun = rng.frange(s.stun_min, s.stun_max) as i32;
                        self.stun_type = s.stun_type;
                        self.enter(10, 2);
                    }
                    _ => self.enter(10, 3),
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
                    a.strike = Some(Strike { slot: k, damage: s.damage, knockback: s.knockback, unblockable: true, force_blocked: false, blocked: false });
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
            hero_blocks: false,
            hero_swing: None,
            missile_threat: false,
            hero_shot_live: false,
            spawn_bits: 0,
            radii: 2.0,
            clip: Clip::default(),
            has,
        }
    }

    #[test]
    fn melee_defence_blocks_or_dodges_once_per_swing() {
        let has = |_| true;
        let mut rng = Rng::seeded(3);
        let mut p = skeleton();
        p.defence = Defence { hold: 30, block: [100.0, 100.0, 100.0], dodge: [0.0; 3], exit: (5.0, 5.0), ..Defence::default() };
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.outer = 1;
        // The hero 4 units away, facing the enemy (dot 1 with the negated offset).
        let mut v = view([4.0, 0.0, 0.0], &has);
        v.hero_swing = Some(1);
        e.tick(&v, &mut rng);
        assert_eq!(e.outer, 7, "a 100 percent block chance blocks");
        assert!(e.blocking);
        v.hero_swing = None;
        e.tick(&v, &mut rng);
        assert_eq!(e.outer, 0, "the swing over: the defence exit");
        assert!(!e.blocking);
        // Out of the arc (the hero facing away) or beyond 8.0 units: nothing.
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.outer = 1;
        let mut v = view([4.0, 0.0, 0.0], &has);
        v.hero_facing = [1.0, 0.0, 0.0];
        v.hero_swing = Some(0);
        e.tick(&v, &mut rng);
        assert_ne!(e.outer, 7);
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.outer = 1;
        let mut v = view([8.0, 0.0, 0.0], &has);
        v.hero_swing = Some(0);
        e.tick(&v, &mut rng);
        assert_ne!(e.outer, 7, "8.0 is not within the reach");
    }

    #[test]
    fn slot_entries() {
        // SPEC-0029 Behavior 2: the setter's 6th value is mode 6's clip, the 7th mode 5's.
        let knight = [3, 3, 3, -1, -1, 5, 4];
        assert_eq!([0, 3, 5, 6].map(|m| slot_entry(&knight, m)), [Some(3), None, Some(4), Some(5)]);
        let ixi = [9, 10, 30, 14, 15, 17, 16];
        assert_eq!([1, 5, 6, 9].map(|m| slot_entry(&ixi, m)), [Some(10), Some(16), Some(17), None]);
    }

    #[test]
    fn swing_classes() {
        assert_eq!(swing_class(0, 5), Some(0));
        assert_eq!(swing_class(0, 9), Some(1));
        assert_eq!(swing_class(0, 12), Some(2));
        assert_eq!(swing_class(0, 10), None, "Tor's power step never draws a defence");
        assert_eq!(swing_class(1, 10), Some(0));
        assert_eq!(swing_class(1, 4), Some(2));
        assert_eq!(swing_class(1, 9), None);
    }

    #[test]
    fn status_vectors() {
        // SPEC-0029 Behavior 27 test vectors.
        let has = |_| true;
        let mut rng = Rng::seeded(1);
        let mut run = |t: i32, code: i32, s: i32, amt: i32, per: i32| {
            let mut p = skeleton();
            p.element = t;
            let mut e = Enemy::new(p, Spawn::default(), &mut rng);
            let applied = e.apply_status(code, s, amt, per);
            let mut total = 0;
            for _ in 0..if s == -1 { 7300 } else { s.max(0) * 60 + 5 } {
                let a = e.tick(&view([50.0, 0.0, 0.0], &has), &mut rng);
                total += a.status_hits.iter().flatten().sum::<i32>();
            }
            (applied, total)
        };
        // Tor's Leaf: 34 s, 5 HP every 4 s: 8 hits, 40 HP; nothing on T = 2.
        assert_eq!(run(0, 2, 34, 5, 4), (true, 40));
        assert_eq!(run(2, 2, 34, 5, 4), (false, 0));
        // Fire 12 s, 5 HP every 2 s: 25 HP. The default poison: 2 HP every 4 s for 120 s.
        assert_eq!(run(0, 7, 12, 5, 2), (true, 25));
        assert_eq!(run(0, 2, -1, -1, -1), (true, 2 * 29));
        // Duration 0: the timer is set to 0 and nothing is active.
        assert_eq!(run(0, 7, 0, 5, 2), (true, 0));
        // Stun: nothing on T = 4; otherwise state 4 at once, timer 300, then back to agro.
        let mut p = skeleton();
        p.element = 4;
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        assert!(!e.apply_status(0, -1, -1, -1));
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        assert!(e.apply_status(0, -1, -1, -1));
        assert_eq!((e.state, e.timers[0]), (4, 300));
        let a = e.tick(&view([50.0, 0.0, 0.0], &has), &mut rng);
        assert_eq!((e.state, a.rate), (4, Some(0.0)));
        for _ in 0..300 {
            e.tick(&view([50.0, 0.0, 0.0], &has), &mut rng);
        }
        assert_eq!(e.state, 19);
        // Outgoing scales.
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        e.apply_status(1, -1, -1, -1);
        e.apply_status(4, -1, -1, -1);
        assert_eq!((e.shot_scale(), e.strike_scale()), (0.75, 1.125));
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
    fn a_guarding_hero_blocks_the_strike() {
        let has = |_| true;
        let mut rng = Rng::seeded(3);
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        e.state = 19;
        e.selected = Some(0);
        e.enter(2, 2);
        let mut v = view([2.0, 0.0, 0.0], &has);
        v.clip = Clip { action: slot_anim(0, 1), progress: 0.2, finished: false };
        v.contact = true;
        v.hero_blocks = true;
        let act = e.tick(&v, &mut rng);
        assert_eq!(act.strike.map(|s| s.blocked), Some(true));
        assert_eq!(e.seq, 4, "a blocked strike plays D");
        // An unblockable slot (flag 0x01) is not blocked.
        let mut p = skeleton();
        p.slots[0].flags = 0x01;
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.selected = Some(0);
        e.enter(2, 2);
        assert_eq!(e.tick(&v, &mut rng).strike.map(|s| s.blocked), Some(false));
        assert_eq!(e.seq, 3);
    }

    #[test]
    fn spawn_count_loop() {
        // SPEC-0038 vector: count 3; spawn event on ticks 1, 2, off 3, on 4, 5, off 6, on 7.
        let has = |_| true;
        let mut rng = Rng::seeded(4);
        let mut p = skeleton();
        p.slots[0].kind = MISSILE;
        p.slots[0].spawn_count = 3;
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        e.state = 19;
        e.selected = Some(0);
        e.enter(2, 2);
        let mut v = view([2.5, 0.0, 0.0], &has);
        v.clip = Clip { action: slot_anim(0, 1), progress: 0.2, finished: false };
        let mut fired = Vec::new();
        for (tick, on) in [true, true, false, true, true, false, true].into_iter().enumerate() {
            v.spawn_bits = if on { 0x20 } else { 0 };
            if let Some(s) = e.tick(&v, &mut rng).shot {
                fired.push((tick + 1, s.counter));
            }
        }
        assert_eq!(fired, vec![(1, 0), (4, 1), (7, 2)]);
        assert_eq!(e.seq, 3);
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
    fn flyer_goal_heights() {
        // SPEC-0040 vector 37 (Can Fly 1, H 10, ground 3, ambush 1).
        let h = |hero: Vec3, ambush| fly_goal(1, 10.0, hero, [0.0, 40.0, 0.0], [0.0, 1.0, 0.0], ambush, Some(3.0));
        assert_eq!(h([30.0, 0.0, 0.0], 1), 6.0);
        assert_eq!(h([10.0, 0.0, 0.0], 1), 5.0);
        assert_eq!(h([30.0, 0.0, 0.0], 0), 1.0);
        // Vector 156 (Can Fly 2, H 4, ground 10, ambush 1).
        assert_eq!(fly_goal(2, 4.0, [0.0, 20.0, 0.0], [0.0; 3], [0.0; 3], 1, Some(10.0)), 14.0);
        assert_eq!(fly_goal(2, 4.0, [0.0, 8.0, 0.0], [0.0; 3], [0.0; 3], 1, Some(10.0)), 8.0);
    }

    #[test]
    fn flyer_grounding_and_relaunch() {
        let has = |_| true;
        let mut rng = Rng::seeded(5);
        let mut p = skeleton();
        (p.can_fly, p.ground_min, p.ground_max, p.fly_idle, p.flying_hit) = (1, 60.0, 120.0, 6.0, 1);
        let mut e = Enemy::new(p, Spawn::default(), &mut rng);
        assert!(e.airborne());
        // Airborne idle: the flying base idle 97; wander id 2 flies as 98 at 6 / 30 per tick.
        let v = view([50.0, 0.0, 0.0], &has);
        let a = e.tick(&v, &mut rng);
        assert_eq!(a.anim, Some(97));
        let mut a = Act { anim: Some(2), ..Act::default() };
        e.fly_step(&mut a);
        assert_eq!((a.anim, a.fly), (Some(98), Some([0.0, 0.0, 0.2])));
        // A hit while airborne: flying hit 1 gives kind 7 and no grounding.
        e.take_hit(1, [0.0; 3], &mut rng);
        assert_eq!((e.outer, e.react, e.retarget), (10, 7, -1));
        // Grounded (as by a death or kind 0): not airborne, and on reaching 0 the re-launch.
        e.ground(&mut rng);
        assert!((60..120).contains(&e.retarget) && !e.airborne());
        assert!(e.take_repick());
        e.state = 0;
        e.retarget = 1;
        e.tick(&v, &mut rng);
        e.tick(&v, &mut rng);
        assert_eq!((e.state, e.retarget, e.take_repick()), (22, -1, true));
        // Death while airborne grounds a can-fly 1 enemy.
        let mut f = Enemy::new(p, Spawn::default(), &mut rng);
        f.die(&mut rng);
        assert!(f.retarget >= 60 && !f.airborne());
    }

    #[test]
    fn corpse_countdown_and_sink() {
        let has = |_| true;
        let mut rng = Rng::seeded(5);
        let mut e = Enemy::new(skeleton(), Spawn::default(), &mut rng);
        e.die(&mut rng);
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
