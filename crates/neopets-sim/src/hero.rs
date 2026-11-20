//! Hero ground locomotion (SPEC-0033 Behaviors 2 and 5; SPEC-0040 Behavior 14C): stick speed
//! and direction, the speed tiers that pick slow walk, walk or run and the play rate, the facing
//! turn, and travel by the root step of the playing clip turned by the facing basis.
//! The motion record (SPEC-0040 Behavior 14A, `motion::Body`) carries the hero: gravity, the
//! walk-off fall (SPEC-0033 9A) and the jumps, air phases, steering and landings of Behavior 50.
//! Strafe is state 32 with its jumps 43 to 46 (Behaviors 4 and 11). Not covered here: reverse walk
//! (state 6) and the backward jump 11, the plunge's effect within 12.0 of the ground, the landing
//! event windows (Cross chain, L1 guard, the hard landing's early exit), the floor-contact flag,
//! the in-air ledge and block-edge choosers, and the displayed-height lag of Behavior 7.

use crate::motion::{Body, HERO_WEIGHT};

pub type Vec3 = [f32; 3];

/// Hero action states used here (SPEC-0033 Behavior 3): 1 stance, 2 slow walk, 3 walk, 4 run.
pub const STANCE: u8 = 1;
pub const SLOW_WALK: u8 = 2;
pub const WALK: u8 = 3;
pub const RUN: u8 = 4;
/// The attack step (SPEC-0036; driven by `attack::TorAttack`).
pub const ATTACK: u8 = 13;
/// SPEC-0036 combat states: 16 death, 17 hurt; SPEC-0033 Behavior 9B: 48 the long fall (the
/// knockback of a common-enemy hit enters it).
pub const DEAD: u8 = 16;
pub const HURT: u8 = 17;
pub const LONG_FALL: u8 = 48;
/// Strafe: walk and turn while L2 is held (SPEC-0033 Behavior 4).
pub const STRAFE: u8 = 32;
/// The get-up after the Plague Serpent's landed lunge (SPEC-0016, R-0189).
pub const GET_UP: u8 = 54;
/// SPEC-0055 death-end states: 59 arrival or return (clip 175), 60 respawn after deadly ground
/// (clip 137).
pub const ARRIVAL: u8 = 59;
/// The scripted one-shot animation (PlayHeroAnim; SPEC-0040 Behavior 15.3 route 51) and the
/// script-control state it can end in (0x21).
pub const ONE_SHOT: u8 = 51;
/// LoopHeroAnim's state (SPEC-0030 row 168).
pub const HERO_LOOP: u8 = 55;
pub const SCRIPTED: u8 = 33;
/// Held by an enemy's grab (SPEC-0040 state 53; the host's `held_step` runs it).
pub const HELD: u8 = 53;
/// SPEC-0047 Feeding a petpet: state 49 with clip 138 (0x8A); at progress 0.99 (word
/// 0x2DD854) the hero returns to state 1.
pub const FEED: u8 = 49;
pub const ANIM_FEED: u32 = 138;
/// SPEC-0033 Behavior 4A.3: the shield guard.
pub const GUARD: u8 = 20;
/// Carry a Miamouse (SPEC-0033 Behavior 18).
pub const CARRY: u8 = 57;
/// Push and pull a block (SPEC-0033 Behavior 4A.2; the block state's tick is `block::Hold`).
pub const BLOCK: u8 = 18;
/// Hang from a block's top edge and climb onto a block (SPEC-0033 Behavior 51, states 19 and
/// 58; their ticks are `block::Hang` and `block::Climb`).
pub const HANG: u8 = 19;
pub const CLIMB: u8 = 58;
/// The ledge states (SPEC-0033 Behavior 51): hang from a ledge, climb up from a fall, climb up
/// while rising.
pub const LEDGE: u8 = crate::ledge::HANG;
pub const LEDGE_UP: u8 = crate::ledge::CLIMB_FALL;
pub const LEDGE_RISE: u8 = crate::ledge::CLIMB_RISE;
pub const LEDGE_DROP: u8 = crate::ledge::DROP;
/// State 28, the hang from a grab line (SPEC-0033 Behavior 9).
pub const LINE: u8 = crate::grab::STATE;
/// State 27, the Rope Gun (SPEC-0033 State 27; its tick is `rope::RopeHero`).
pub const ROPE: u8 = crate::rope::STATE;
/// The ladder (SPEC-0033 Behavior 13; its tick is `ladder::Ladder`).
pub const LADDER: u8 = 41;
/// The climb on a climb panel (SPEC-0033 Behavior 51; its tick is `climb::Climb`).
pub const PANEL: u8 = 21;
/// Sneak idle, sneak reverse walk (39) and sneak walk (SPEC-0033 Behavior 4A.1 and 5.1).
pub const SNEAK: u8 = 22;
pub const SNEAK_WALK: u8 = 40;
pub const SNEAK_REVERSE: u8 = 39;
const ANIM_PICK_UP: u32 = 178;
const ANIM_CARRY_STAND: u32 = 179;
const ANIM_CARRY_WALK: u32 = 180;
const ANIM_PUT_DOWN: u32 = 181;
/// Guard clips: 63 shield up, 64 the block reaction, 119 the guard turn.
const ANIM_GUARD: u32 = 63;
const ANIM_BLOCK: u32 = 64;
const ANIM_GUARD_TURN: u32 = 119;
pub const RESPAWN: u8 = 60;
/// Jump states (Behavior 50): 7 standing, 8 slow, 9 walking, 10 running.
pub const JUMP: u8 = 7;
pub const JUMP_RUN: u8 = 10;

/// SPEC-0033 busy predicate, the state list: 7 to 19, 21, 23 to 30, 35, 37, 38, 41, 43 to 49,
/// 53, 54 and 56 to 60 are busy. The two sub-predicates and the global input lock are not
/// covered.
pub fn busy_state(s: u8) -> bool {
    matches!(s, 7..=19 | 21 | 23..=30 | 35 | 37 | 38 | 41 | 43..=49 | 53 | 54 | 56..=60)
}
/// Behavior 50 constants: launch vertical velocity, the gains and the phase thresholds.
const LAUNCH: f32 = 8.75;
const GAIN_RISE: f32 = 0.35;
const GAIN_FALL: f32 = 0.3;
const GAIN_DESCENT: f32 = 0.35;
const LAND_GAP: f32 = 1.0;
const HARD_FALL: f32 = 3.0;
/// Air steering: drag per tick, stick push per tick and the speed cap for the push.
const AIR_DRAG: f32 = 0.5;
const AIR_PUSH: f32 = 0.6;
const AIR_CAP: f32 = 9.0;
/// Hero animation ids (SPEC-0033 Behaviors 9A and 50).
const ANIM_JUMP_STILL: u32 = 70;
const ANIM_JUMP_MOVE: u32 = 76;
const ANIM_FALL_STILL: u32 = 67;
const ANIM_FALL_MOVE: u32 = 78;
const ANIM_SOFT_LAND: u32 = 68;
const ANIM_HARD_LAND: u32 = 118;
/// SPEC-0036 state 17 (32 armed, 0xC0 unarmed), SPEC-0033 4A.6 death on ordinary ground (34),
/// Behavior 9B long fall in the air (136) and its landing (137).
const ANIM_HURT: u32 = 32;
const ANIM_HURT_UNARMED: u32 = 0xC0;
const ANIM_DEATH: u32 = 34;
const ANIM_LONG_FALL: u32 = 136;
const ANIM_LONG_LAND: u32 = 137;
const GAIN_RISE_SWITCH: f32 = 12.0;
const DESCENT_VELOCITY: f32 = -7.0;

/// The jump speed S of states 7 to 10 (Behavior 50 table).
fn jump_speed(state: u8) -> f32 {
    match state {
        8 => 2.0,
        9 => 5.0,
        10 => 9.0,
        _ => 0.0,
    }
}
/// Passes after a run ends before running may restart (0.5 s at 60 ticks per second).
const RUN_RESTART: u32 = 30;

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalise(v: Vec3) -> Vec3 {
    let l = dot(v, v).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

fn flatten(v: Vec3) -> Vec3 {
    normalise([v[0], 0.0, v[2]])
}

/// One pass of controller input: the left stick after the dead zone, `side` positive to the
/// left and `forward` positive up, each in [-1, 1]; `run` is R1.
/// Hero Haste's fastest clip rate: past it the legs would blur.
const HASTE_LEGS: f32 = 2.5;

/// A full stick push over the stick scale 87.5 (SPEC-0036 Tor's sub-states 7 and 8).
const STICK_FULL: f32 = 127.5 / 87.5;

/// What the stick does in state 13 this tick; the attack driver of the host sets it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum StickStep {
    #[default]
    Off,
    /// Tor: the velocity cleared in all three parts and set.
    Clear,
    /// Roberta's spins: only x and z.
    Horizontal,
    /// Roberta's sub-state 14: a facing turn.
    Turn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeroInput {
    pub side: f32,
    pub forward: f32,
    pub run: bool,
    /// Cross pressed this pass (a press edge).
    pub jump: bool,
    /// The attack button (R2 or Circle): held, and its press edge this pass.
    pub attack_held: bool,
    pub attack: bool,
    /// Square pressed this pass (a press edge): confirms an interaction prompt.
    pub square: bool,
    /// Square held (the sneak lasts while it is held).
    pub square_held: bool,
    /// L1 held (the shield guard).
    pub guard: bool,
    /// R2 and Circle held apart (the aim mode's retained choice and the Rope Gun's entry).
    pub r2_held: bool,
    pub circle_held: bool,
    /// Cross held (the dialogue box's faster reveal).
    pub cross_held: bool,
    /// L2 held (strafe, Behavior 4).
    pub l2: bool,
}

/// The hero's playing clip as the animation pass left it on the previous tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipState {
    pub action: u32,
    pub progress: f32,
    pub finished: bool,
}

/// Speed value v: the stick length, capped at 1.0 (SPEC-0033 Behavior 2).
pub fn speed_value(i: &HeroInput) -> f32 {
    (i.side * i.side + i.forward * i.forward).sqrt().min(1.0)
}

/// The facing basis of SPEC-0040 Behavior 14C: B1 side, B2 up, B3 forward.
pub fn basis(facing: Vec3) -> [Vec3; 3] {
    let b3 = flatten(facing);
    let c = cross([0.0, 1.0, 0.0], b3);
    let b1 = if dot(c, c).sqrt() < 0.01 { [1.0, 0.0, 0.0] } else { normalise(c) };
    [b1, cross(b3, b1), b3]
}

/// Facing turn while moving (speed value above 0.01): c = max(d, 0.95) and the new facing is
/// c f + √|1 − c²| u, flattened and normalised: at most 18.19° per pass (SPEC-0033 Behavior 2).
pub fn turn_moving(f: Vec3, u: Vec3) -> Vec3 {
    let d = dot(f, u).min(1.0);
    let c = d.max(0.95);
    let s = (1.0 - c * c).abs().sqrt();
    flatten([c * f[0] + s * u[0], c * f[1] + s * u[1], c * f[2] + s * u[2]])
}

/// Steering (SPEC-0056 Steering and turn rate): the new facing is v / |v| with
/// v = w t + (1 − w) c for current facing c, target direction t and weight w; when c · t is
/// below −0.99, 0.1 times the side axis is added so the turn picks a side. Horizontal only.
pub fn steer(c: Vec3, t: Vec3, w: f32) -> Vec3 {
    let (c, t) = (flatten(c), flatten(t));
    if dot(t, t) == 0.0 {
        return c;
    }
    let mut v: Vec3 = std::array::from_fn(|k| w * t[k] + (1.0 - w) * c[k]);
    if dot(c, t) < -0.99 {
        let side = basis(c)[0];
        for k in 0..3 {
            v[k] += 0.1 * side[k];
        }
    }
    let f = flatten(v);
    if dot(f, f) == 0.0 { c } else { f }
}

/// The tier step (SPEC-0033 5.1): the state to request and the factor on the play rate, or None
/// for no movement (v ≤ 0.1).
pub fn tier(v: f32, run_mode: bool) -> Option<(u8, f32)> {
    if v <= 0.1 {
        return None;
    }
    let v = v.max(0.3);
    Some(if run_mode {
        if v >= 0.66 {
            (RUN, v)
        } else if v >= 0.33 {
            (WALK, v)
        } else {
            (SLOW_WALK, (v / 0.33).min(1.0))
        }
    } else if v >= 0.5 {
        (WALK, v)
    } else {
        (SLOW_WALK, (2.0 * v).min(1.0))
    })
}

/// A hero's motion record, facing and locomotion state.
#[derive(Clone, Debug, PartialEq)]
pub struct Locomotion {
    pub body: Body,
    pub facing: Vec3,
    pub state: u8,
    /// Stick direction u (steering target): kept while the stick is centred.
    pub target: Vec3,
    /// Per-pass speed scale P, the hero's play rate R (base scale 1.0).
    pub rate: f32,
    /// Air phase word of Behavior 50 (-1 launch pending).
    pub phase: i8,
    /// The weapon-out flag P1 (the host sets it each pass).
    pub p1: bool,
    /// The ground patch under the hero has kind word 1 or 6 (SPEC-0033 Behavior 4A.1): the slow walk.
    pub slow_patch: bool,
    /// The host's Tor with no Tattered Cloak: air attacks may start.
    pub air_ok: bool,
    /// The stick step of state 13 this tick.
    pub stick_step: StickStep,
    /// The developer menu's Hero Haste: how many times faster the hero covers ground (1.0 off). Not in the game.
    pub haste: f32,
    /// L2 last tick, and whether this hero is Roberta (the camera-locked state test).
    pub l2_held: bool,
    pub roberta: bool,
    /// State 17 was entered from state 32.
    hurt_from_strafe: bool,
    /// The air attack's sub-state word: 11 the slash, 12 the plunge, 0 none.
    pub air_sub: u8,
    /// Vertical speed kept through the next jump take-off (the class 20 launch's one-tick hold).
    pub launch_bonus: f32,
    air_used: bool,
    air_armed: bool,
    air_hold: u32,
    root_y: f32,
    /// State 32's requested clip and stick (side, forward), and a strafe jump's direction.
    strafe_id: u32,
    strafe_stick: Option<(f32, f32)>,
    jump_dir: Vec3,
    /// Take-off height stored by the jump chooser or the walk-off.
    start_height: f32,
    /// Position change of the last record tick (+0x160).
    last_motion: Vec3,
    /// A world-space displacement added to the next record tick's motion after the travel
    /// step (the Rope Gun's pulls, SPEC-0033 State 27), then cleared.
    pub extra: Vec3,
    request: u32,
    run_counter: u32,
    /// The hero holds a usable shield (the guard's entry gate).
    pub shield: bool,
    /// The speed scale P after the status routine (SPEC-0036 Status effects), the base of the
    /// play rate before the speed tiers.
    pub status_scale: f32,
    /// State 57's sub-state: 0 pick up, 1 stand, 2 walk, 3 put down.
    pub carry: u8,
    /// PlayHeroAnim's CTL: the one-shot ends in state 1 (set) or 33.
    pub one_shot_control: bool,
    /// LoopHeroAnim's counter.
    pub loop_ticks: i32,
    /// The displayed height and its follow (SPEC-0033 Behavior 7).
    pub follow: HeightFollow,
}

/// SPEC-0033 Behavior 7, the hero height follow: the drawn copy's height, which the camera
/// follows, catches up with the world height (quadratic in the gap, gain 0.04 at 0x2DA4BC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightFollow {
    pub shown: f32,
    catching: bool,
    counter: u32,
    v: f32,
}

impl HeightFollow {
    pub fn at(y: f32) -> Self {
        Self { shown: y, catching: false, counter: 0, v: 0.0 }
    }

    /// One tick with the world height and the hero state (the movement predicate passes only
    /// in the jump states 7 to 11 and 43 to 46).
    pub fn tick(&mut self, world: f32, state: u8) {
        let g = world - self.shown;
        if !self.catching {
            let jumping = matches!(state, 7..=11 | 43..=46);
            if g.abs() > 3.5 || !jumping {
                self.catching = true;
            } else if g.abs() > 0.1 {
                self.counter += 1;
                if self.counter > 120 || g < -0.5 {
                    self.catching = true;
                }
            } else {
                self.counter = 0;
            }
            self.v = 0.0;
            if !self.catching {
                return;
            }
        }
        if g.abs() < 0.05 {
            self.catching = false;
            self.counter = 0;
            return;
        }
        let q = 0.05 * g * g;
        if q < self.v {
            self.v = q;
        } else {
            self.v += 0.04 * q;
        }
        // A safety clamp (no spec): the step never passes the world height. It only acts for
        // gaps above about 500 units (0.002 g² > |g|), where the formula diverges, as after a
        // fall far below the world.
        self.shown += g.signum() * self.v.min(g.abs());
    }
}

impl Locomotion {
    pub fn spawn(position: Vec3, facing: Vec3) -> Self {
        let f = flatten(facing);
        let f = if dot(f, f) == 0.0 { [1.0, 0.0, 0.0] } else { f };
        Self {
            body: Body::new(position, HERO_WEIGHT),
            facing: f,
            state: STANCE,
            target: f,
            rate: 1.0,
            phase: 0,
            p1: false,
            air_ok: false,
            stick_step: StickStep::Off,
            haste: 1.0,
            l2_held: false,
            roberta: false,
            hurt_from_strafe: false,
            air_sub: 0,
            launch_bonus: 0.0,
            air_used: false,
            air_armed: false,
            air_hold: 0,
            root_y: 0.0,
            strafe_id: 0,
            strafe_stick: None,
            jump_dir: [0.0; 3],
            start_height: position[1],
            last_motion: [0.0; 3],
            extra: [0.0; 3],
            request: 0,
            run_counter: RUN_RESTART,
            shield: false,
            status_scale: 1.0,
            slow_patch: false,
            carry: 0,
            one_shot_control: true,
            loop_ticks: 0,
            follow: HeightFollow::at(position[1]),
        }
    }

    /// Square with no target (the chooser's last fallback): sneak (state 22) when not busy and
    /// not in states 20, 32, 43 to 46, 22, 39 or 40 (SPEC-0033 Behavior 1A).
    pub fn sneak(&mut self) {
        if !busy_state(self.state) && !matches!(self.state, 20 | 32 | 43..=46 | 22 | 39 | 40) {
            self.state = SNEAK;
        }
    }

    /// The catch: state 57, sub-state 0 (SPEC-0033 Behavior 18).
    pub fn start_carry(&mut self) {
        self.state = CARRY;
        self.carry = 0;
        self.request = ANIM_PICK_UP;
    }

    /// The drop prompt's Square press: sub-state 3 from 1 or 2. Returns whether it was taken.
    pub fn drop_carry(&mut self) -> bool {
        if self.state == CARRY && matches!(self.carry, 1 | 2) {
            self.carry = 3;
            self.request = ANIM_PUT_DOWN;
            return true;
        }
        false
    }

    /// State 57 per tick: sub-state 0 requests 178 and goes to 1 at 0.99; 1 stands with 179 and
    /// goes to 2 on a stick push (turning); 2 walks with 180, turning at 0.25, back to 1 when the
    /// stick centres; 3 puts down with 181 and returns to state 1 at 0.99.
    fn carry_step(&mut self, moving: bool, clip: ClipState) {
        match self.carry {
            0 => {
                self.request = ANIM_PICK_UP;
                if clip.action == ANIM_PICK_UP && clip.progress >= 0.99 {
                    self.carry = 1;
                }
            }
            3 => {
                self.request = ANIM_PUT_DOWN;
                if clip.action == ANIM_PUT_DOWN && clip.progress >= 0.99 {
                    self.state = STANCE;
                    self.carry = 1;
                }
            }
            2 => {
                self.request = ANIM_CARRY_WALK;
                self.facing = steer(self.facing, self.target, 0.25);
                if !moving {
                    self.carry = 1;
                }
            }
            _ => {
                if moving {
                    self.facing = turn_moving(self.facing, self.target);
                    self.carry = 2;
                }
                self.request = ANIM_CARRY_STAND;
            }
        }
    }

    /// Move the hero at once to a landing point and facing (SPEC-0006 teleport landing and the
    /// interior arrivals): velocity cleared, action state reset to the stance.
    pub fn teleport(&mut self, position: Vec3, facing: Vec3) {
        let f = flatten(facing);
        self.body.position = position;
        self.body.velocity = [0.0; 3];
        self.body.gain = 1.0;
        // A teleport out of a traversal state (a ladder, a hang) leaves its no-gravity and
        // no-snap flags behind.
        self.body.no_gravity = false;
        self.body.no_snap = false;
        if dot(f, f) > 0.0 {
            self.facing = f;
            self.target = f;
        }
        self.state = STANCE;
        self.phase = 0;
        self.last_motion = [0.0; 3];
        self.follow = HeightFollow::at(position[1]);
    }

    pub fn position(&self) -> Vec3 {
        self.body.position
    }

    /// PlayHeroAnim (SPEC-0030 row 157): the one-shot state 51 with the animation and CTL.
    pub fn feed(&mut self) {
        self.state = FEED;
        self.request = ANIM_FEED;
    }

    /// LoopHeroAnim (SPEC-0030 row 168): the clip for this many ticks; then state 1 with control,
    /// otherwise the scripted state.
    pub fn play_loop(&mut self, anim: u32, ticks: i32, control: bool) {
        self.state = HERO_LOOP;
        self.request = anim;
        self.one_shot_control = control;
        self.loop_ticks = ticks;
    }

    pub fn play_one_shot(&mut self, anim: u32, control: bool) {
        self.state = ONE_SHOT;
        self.request = anim;
        self.one_shot_control = control;
    }

    /// Enter or leave the attack step, and set the clip it requests.
    pub fn set_attack(&mut self, request: Option<u32>) {
        match request {
            Some(id) => {
                self.state = ATTACK;
                self.request = id;
            }
            None if self.state == ATTACK => self.state = STANCE,
            None => {}
        }
    }

    /// SPEC-0036 the stick step (Tor's sub-states 7, 8 and 10 in the attack window; Roberta's spins), with
    /// the stick off centre: the pair's length L uncapped (a full push is 127.5 / 87.5), the velocity
    /// the stick direction x L x 0.25 x 30. Tor's clears all three parts, Roberta's only x and z. Her
    /// sub-state 14 turns the facing toward the stick at rate 0.35 and writes no velocity.
    fn attack_step(&mut self, input: &HeroInput) {
        let l = (input.side * input.side + input.forward * input.forward).sqrt() * STICK_FULL;
        let d = self.target.map(|t| t * l * 0.25 * 30.0);
        match self.stick_step {
            StickStep::Off => {}
            StickStep::Clear => self.body.velocity = d,
            StickStep::Horizontal => self.set_horizontal(d),
            StickStep::Turn => {
                let f = self.facing;
                self.facing = normalise(std::array::from_fn(|k| f[k] + 0.35 * (self.target[k] - f[k])));
            }
        }
    }

    /// SPEC-0036 Taking a hit, step 5: the hit-reaction state 17 with its clip.
    pub fn hurt(&mut self, armed: bool) {
        // The previous state word: a repeat hit in state 17 keeps what it held.
        if self.state != HURT {
            self.hurt_from_strafe = self.state == STRAFE;
        }
        self.state = HURT;
        self.request = if armed { ANIM_HURT } else { ANIM_HURT_UNARMED };
    }

    /// SPEC-0036 Health and death, step 3: the death state 16 (request 34 on ordinary ground).
    pub fn die(&mut self) {
        self.state = DEAD;
        self.request = ANIM_DEATH;
    }

    /// SPEC-0033 Behavior 4A.3, the shield guard: L1 released goes to state 1; otherwise 63
    /// (not while the block reaction 64 plays below progress 0.99), and a pushed stick turns the
    /// hero to the stick direction at rate 1.0, requesting 119 when that direction is more than
    /// about 18 degrees from the facing (dot below 0.95; R-0017). No travel step.
    fn guard(&mut self, input: &HeroInput, stick: bool, clip: ClipState) {
        // L1 released: state 1 and the idle request, and the routine still runs its stick part
        // for this tick (it may request 119; SPEC-0033 4A.3, corrected from code).
        if !input.guard {
            self.state = STANCE;
            self.request = 0;
        } else {
            let blocking = clip.action == ANIM_BLOCK && clip.progress < 0.99;
            if !blocking {
                self.request = ANIM_GUARD;
            }
        }
        if stick {
            let u = self.target;
            if dot(u, self.facing) < 0.95 && clip.action != ANIM_BLOCK {
                self.request = ANIM_GUARD_TURN;
            }
            self.facing = steer(self.facing, u, 1.0);
        }
    }

    /// SPEC-0036 Taking a hit, step 5, blocked flag 1: the block reaction (request 64).
    pub fn block(&mut self) {
        if self.state == GUARD {
            self.request = ANIM_BLOCK;
        }
    }

    /// The death-state handler's clip (SPEC-0055 section 8A).
    pub fn set_death_clip(&mut self, id: u32) {
        if self.state == DEAD {
            self.request = id;
        }
    }

    /// A death-end state (59 or 60), written by the respawn at the start pose.
    pub fn set_end_state(&mut self, state: u8) {
        self.state = state;
    }

    /// StartHeroLongFall (SPEC-0030 row 280): the hero enters the long fall (state 48) with its
    /// velocity kept; no health change.
    pub fn long_fall_start(&mut self) {
        self.state = LONG_FALL;
        self.request = ANIM_LONG_FALL;
    }

    /// SPEC-0036 Knockback on the hero: the velocity is replaced and the hero enters the long
    /// fall (state 48, SPEC-0033 Behavior 9B).
    /// A hold that switched gravity off (a ledge, ladder, panel, line or rope) ends with the
    /// knock: without this the long fall kept the hold's no-gravity flag and a hero knocked off a
    /// ledge rose for ever at the knock's speed.
    pub fn knock(&mut self, velocity: Vec3) {
        self.body.velocity = velocity;
        self.body.gain = 1.0;
        self.body.no_gravity = false;
        self.body.no_snap = false;
        self.state = LONG_FALL;
        self.request = ANIM_LONG_FALL;
    }

    /// The camera's recentring gate T (SPEC-0027): the hero moved in its last motion step and
    /// is not in state 1.
    /// Turn the facing by the steering rule (the talking hero uses weight 0.1).
    /// State 18's entry (the grab-block press) and its clip request each tick.
    pub fn enter_block(&mut self) {
        self.state = BLOCK;
        self.request = 40;
    }

    /// The grab line's let-go (SPEC-0033 Behavior 9): state 7 in air phase 5 with request 67,
    /// leaving with `velocity`.
    pub fn line_let_go(&mut self, velocity: Vec3) {
        self.drop_off();
        self.body.velocity = velocity;
    }

    /// The ladder's jump off (Behavior 13): a jump state falling with the given velocity.
    pub fn jump_off(&mut self, velocity: Vec3) {
        self.enter_air(JUMP);
        self.phase = 2;
        self.request = ANIM_JUMP_STILL;
        self.body.velocity = velocity;
        self.body.no_gravity = false;
        self.body.gain = GAIN_FALL;
    }

    /// SPEC-0047 Class 20 state 1: the creature launches the hero: 8.0 added to the vertical
    /// velocity and state 7, with a one-tick hold so the next pass skips its velocity reset.
    /// This build reads the hold as keeping the 8.0 through state 7's own take-off, which then
    /// adds its 8.75 (R-0245, this build's choice): with the 8.0 alone he rose 3.2, below the
    /// shell's own pop-out of 6.7 (user's report 34). The take-off speed is then raised so the
    /// throw goes 10% higher (height grows with the square of the speed), at the user's request
    /// after playing it; also this build's choice.
    pub fn launch_up(&mut self, dv: f32) {
        self.enter_air(JUMP);
        let speed = (LAUNCH + self.body.velocity[1].max(0.0) + dv) * 1.1f32.sqrt();
        self.launch_bonus = speed - LAUNCH;
        self.body.no_gravity = false;
    }

    /// The climb's Stop Climbing (SPEC-0033 Behavior 51, state 21 exits): state 7 in air phase
    /// 5 with request 67, falling from rest.
    pub fn drop_off(&mut self) {
        self.enter_air(JUMP);
        self.phase = 5;
        self.request = 67;
        self.body.velocity = [0.0; 3];
        self.body.no_gravity = false;
        self.body.gain = GAIN_FALL;
    }

    /// Enter a block state (19 or 58) with its first request.
    pub fn enter_state(&mut self, state: u8, request: u32) {
        self.state = state;
        self.request = request;
    }

    pub fn block_request(&mut self, id: u32) {
        self.request = id;
    }

    pub fn steer_toward(&mut self, t: Vec3, w: f32) {
        self.facing = steer(self.facing, t, w);
        self.target = self.facing;
    }

    pub fn last_step_length(&self) -> f32 {
        dot(self.last_motion, self.last_motion).sqrt()
    }

    pub fn moved(&self) -> bool {
        self.state != STANCE && self.last_motion != [0.0; 3]
    }

    pub fn airborne(&self) -> bool {
        (JUMP..=JUMP_RUN).contains(&self.state) || (43..=46).contains(&self.state)
    }

    /// SPEC-0033 Behavior 4, the state-32 handler: the release, the jumps, the heading after the camera
    /// and the walk-and-turn codes; the stick re-aims the root step (`travel`).
    fn strafe(&mut self, input: &HeroInput, v: f32, camera_forward: Vec3) {
        self.strafe_stick = None;
        if !input.l2 {
            self.state = STANCE;
            self.strafe_id = 0;
            return;
        }
        let (f, s) = (input.forward, input.side);
        let p1 = self.p1;
        // Item 9: h = normalise(0.75 h + 0.25 C), with a tie-break when they are nearly opposite.
        let c = flatten(camera_forward);
        let mut h: Vec3 = std::array::from_fn(|k| 0.75 * self.facing[k] + 0.25 * c[k]);
        if dot(self.facing, c) < -0.99 {
            let right = cross(self.facing, [0.0, 1.0, 0.0]);
            h = std::array::from_fn(|k| h[k] + 0.1 * right[k]);
        }
        self.facing = normalise(h);
        self.strafe_id = if v == 0.0 {
            0
        } else if f.abs() * 0.25 < s.abs() {
            match (s < 0.0, f > 0.0, p1) {
                (true, true, false) => 151,
                (true, false, false) => 98,
                (true, true, true) => 153,
                (true, false, true) => 99,
                (false, true, false) => 154,
                (false, false, false) => 152,
                (false, true, true) => 101,
                (false, false, true) => 38,
            }
        } else if f > 0.0 {
            match (input.run, p1) {
                (true, false) => 4,
                (true, true) => 9,
                (false, false) => 3,
                (false, true) => 8,
            }
        } else if p1 {
            11
        } else {
            6
        };
        if v > 0.0 {
            self.strafe_stick = Some((s, f));
        }
        // Behavior 50, in strafe mode: pushed, the strafe jump by the current action; centred, state 7.
        if input.jump && self.body.normal[1] >= 0.82 {
            self.strafe_stick = None;
            if v > 0.0 {
                // SPEC-0033 Behavior 11: the stick in the hero's own frame, not the camera's.
                let right = cross(self.facing, [0.0, 1.0, 0.0]);
                let d: Vec3 = std::array::from_fn(|k| input.forward * self.facing[k] + input.side * right[k]);
                self.jump_dir = normalise(d);
                let state = match self.strafe_id {
                    6 | 11 => 45,
                    36 | 38 | 100 | 101 | 152 | 154 => 44,
                    37 | 39 | 98 | 99 | 151 | 153 => 43,
                    _ => 46,
                };
                self.enter_air(state);
            } else {
                self.enter_air(JUMP);
            }
        }
    }

    /// The animation id requested this pass. Ground states 2 to 4 play ids 2 to 4 and the
    /// stance the base idle 0 (SPEC-0033 5.1); the air states request their own.
    pub fn action(&self) -> u32 {
        match self.state {
            // Ids 7, 8 and 9 with the weapon out (P1; SPEC-0033 5.1).
            SLOW_WALK | WALK | RUN if self.slow_patch => 106,
            SLOW_WALK | WALK | RUN => self.state as u32 + if self.p1 { 5 } else { 0 },
            STRAFE => self.strafe_id,
            43..=46 => self.request,
            // The get-up state 54 plays its clip 137 too (report 159: left out, the hero stood in
            // idle 0 for good, waiting for a clip that was never asked for).
            JUMP..=JUMP_RUN | ATTACK | DEAD | HURT | LONG_FALL | GUARD | CARRY | ONE_SHOT | HERO_LOOP | FEED | BLOCK | HANG | CLIMB | LADDER | PANEL | LEDGE | LEDGE_UP | LEDGE_RISE | LEDGE_DROP | LINE | ROPE | GET_UP => self.request,
            ARRIVAL => 175,
            HELD => 143,
            SNEAK if self.slow_patch => 0,
            SNEAK => 87,
            SNEAK_WALK if self.slow_patch => 106,
            SNEAK_WALK => 85,
            SNEAK_REVERSE => 82,
            RESPAWN => ANIM_LONG_LAND,
            _ => 0,
        }
    }

    /// The hero update of one pass: stick direction from the camera's ground forward and
    /// screen-left vectors, then the state routine: on the ground the walk-off check, the jump
    /// chooser, the facing turn and the speed tiers; in the air the air routine.
    /// SPEC-0027 Follow step 9, the camera-locked hero states: 32, 43 to 46, 17 and 20 from 32 (the guard is
    /// not tracked here), and Roberta's state 13 with L2 held.
    pub fn camera_locked(&self) -> bool {
        self.state == STRAFE || (43..=46).contains(&self.state) || (self.state == HURT && self.hurt_from_strafe) || (self.state == ATTACK && self.roberta && self.l2_held)
    }

    /// The air-attack-used flag: Roberta's air shot takes it through the host.
    pub fn air_free(&self) -> bool {
        !self.air_used
    }

    pub fn use_air(&mut self) {
        self.air_used = true;
    }

    pub fn update(&mut self, input: &HeroInput, camera_forward: Vec3, camera_left: Vec3, clip: ClipState) {
        self.l2_held = input.l2;
        let v = speed_value(input);
        self.rate = self.status_scale;
        if v > 0.0 {
            let l = v.max(f32::MIN_POSITIVE);
            let (x, z) = (input.side / l, input.forward / l);
            let c = flatten(camera_forward);
            let left = flatten(camera_left);
            self.target = normalise([z * c[0] + x * left[0], 0.0, z * c[2] + x * left[2]]);
        }
        match self.state {
            // The attack step owns the hero: no stick movement or turning, but for the host's stick step.
            ATTACK => {
                if v > 0.0 {
                    self.attack_step(input);
                }
                return;
            }
            DEAD => return,
            HURT => {
                // State 17: at progress 0.99 request 0 and go to state 1, or to state 32 when L2 is
                // held and the previous state was 32.
                if clip.action == self.request && clip.progress >= 0.99 {
                    self.state = if input.l2 && self.hurt_from_strafe { STRAFE } else { STANCE };
                }
                return;
            }
            LONG_FALL => return self.long_fall(clip),
            // SPEC-0016 Hero get-up state 54: only clip 137 to 0.99, then state 1. No movement of its
            // own, no invulnerability.
            GET_UP => {
                self.request = ANIM_LONG_LAND;
                if clip.action == ANIM_LONG_LAND && clip.progress >= 0.99 {
                    self.state = STANCE;
                }
                return;
            }
            // Route 51: at progress 0.99 of the clip, state 1 with the control flag set, else 33.
            ONE_SHOT => {
                if clip.action == self.request && clip.progress >= 0.99 {
                    self.state = if self.one_shot_control { STANCE } else { SCRIPTED };
                }
                return;
            }
            HERO_LOOP => {
                if self.loop_ticks > 0 {
                    self.loop_ticks -= 1;
                } else {
                    self.state = if self.one_shot_control { STANCE } else { SCRIPTED };
                }
                return;
            }
            SCRIPTED | HELD => return,
            FEED => {
                if clip.action == self.request && clip.progress >= 0.99 {
                    self.state = STANCE;
                }
                return;
            }
            CARRY => return self.carry_step(v > 0.0, clip),
            // State 18: the block hold decides the clip and the facing (after this update).
            BLOCK | HANG | CLIMB | LADDER | PANEL | LEDGE | LEDGE_UP | LEDGE_RISE | LEDGE_DROP | LINE | ROPE => return,
            GUARD => return self.guard(input, v > 0.0, clip),
            ARRIVAL | RESPAWN => {
                // At progress 0.99 of the state's clip: state 1.
                if clip.action == self.action() && clip.progress >= 0.99 {
                    self.state = STANCE;
                }
                return;
            }
            _ => {}
        }
        if !self.airborne() {
            // Behavior 9A: the walk-off check runs first in states 1 to 6.
            if self.body.gap > 3.0 && self.body.velocity[1] < -3.0 {
                self.enter_air(JUMP);
                // The vertical lane of the motion is zeroed first, so the test is horizontal
                // (SPEC-0033, corrected from code).
                let mut m = self.last_motion;
                m[1] = 0.0;
                let still = dot(m, self.facing) < 0.0 || dot(m, m).sqrt() < 0.01;
                (self.phase, self.request) = if still { (2, ANIM_JUMP_STILL) } else { (4, ANIM_JUMP_MOVE) };
                return;
            }
            // Behavior 4A.3 entry from the button chain: L1 with a usable shield.
            if input.guard && self.shield {
                self.state = GUARD;
                return self.guard(input, v > 0.0, clip);
            }
            // Behavior 4: L2 held in idle enters state 32; the state-32 handler ends it.
            if self.state == STRAFE || (self.state == STANCE && input.l2) {
                self.state = STRAFE;
                self.strafe(input, v, camera_forward);
                return;
            }
            // Behavior 50 jump chooser: Cross on walkable ground (normal y at least 0.82).
            if !(input.jump && self.body.normal[1] >= 0.82) {
                if v > 0.0 {
                    self.facing = turn_moving(self.facing, self.target);
                }
                // Sneak: while Square is held the tiers give 22 (stopped), 40 (forward) or 39
                // (the stick pulled back, here: pointing more than 90 degrees from the facing;
                // the sign rule is Medium), with no factor; released, the hero is back in state 1.
                if matches!(self.state, SNEAK | SNEAK_REVERSE | SNEAK_WALK) {
                    if input.square_held {
                        self.state = if v <= 0.1 {
                            SNEAK
                        } else if dot(self.target, self.facing) < 0.0 {
                            SNEAK_REVERSE
                        } else {
                            SNEAK_WALK
                        };
                        return;
                    }
                    self.state = STANCE;
                }
                self.run_counter = if self.state == RUN { 0 } else { (self.run_counter + 1).min(RUN_RESTART) };
                let run_mode = input.run && (self.state == RUN || self.run_counter >= RUN_RESTART);
                match tier(v, run_mode) {
                    Some((state, factor)) => {
                        self.state = state;
                        self.rate *= factor;
                    }
                    None => self.state = STANCE,
                }
                return;
            }
            let m = (input.side * input.side + input.forward * input.forward).sqrt();
            self.enter_air(match m {
                m if m == 0.0 => 7,
                m if m < 0.3 => 8,
                m if m < 0.6 => 9,
                _ => 10,
            });
        }
        self.air_attack(input);
        self.air(v > 0.0, clip);
    }

    /// SPEC-0033 Behavior 9B, steps 1, 2 and 4 (no moving object; the mash speed-up is not
    /// covered): gains by vertical velocity in the air, the landing at a gap of 1.0 or less with
    /// the vertical velocity at most 0, then 137 to progress 0.99.
    fn long_fall(&mut self, clip: ClipState) {
        let vy = self.body.velocity[1];
        if clip.action == ANIM_LONG_LAND || (self.body.gap <= LAND_GAP && vy <= 0.0) {
            self.body.gain = 1.0;
            self.request = ANIM_LONG_LAND;
            if clip.action == ANIM_LONG_LAND && clip.progress >= 0.99 {
                self.state = STANCE;
            }
            return;
        }
        self.request = ANIM_LONG_FALL;
        self.body.gain = if vy > GAIN_RISE_SWITCH {
            GAIN_RISE
        } else if vy > DESCENT_VELOCITY {
            GAIN_FALL
        } else {
            GAIN_DESCENT
        };
    }

    /// The ledge-drop hand-off's running jump (SPEC-0033 Behavior 49): state 10 from the hero's
    /// position and facing.
    pub fn edge_jump(&mut self) {
        self.enter_air(JUMP_RUN);
    }

    /// The state setter's entry to the jump states: launch pending.
    /// SPEC-0033 Behavior 50 Air attacks, Tor (`air_ok`: the host's Tor with no Tattered Cloak). States 7
    /// to 11: the input gives the plunge when rising or 1.5 or more up. Strafe jumps, phases 2 to 5:
    /// the first tick arms, more than 10 further ticks give the plunge, a release gives the slash.
    fn air_attack(&mut self, input: &HeroInput) {
        let pressed = input.attack || input.circle_held;
        if self.air_used || !self.air_ok {
            return;
        }
        if (43..=46).contains(&self.state) {
            if !(2..=5).contains(&self.phase) {
                return;
            }
            if pressed {
                if self.air_armed {
                    self.air_hold += 1;
                    if self.air_hold > 10 {
                        self.plunge_start();
                    }
                } else {
                    (self.air_armed, self.air_hold) = (true, 0);
                }
            } else if self.air_armed {
                self.air_armed = false;
                self.air_used = true;
                self.air_sub = 11;
                self.request = 25;
            }
        } else if pressed && !matches!(self.phase, 7..=9) && (self.body.velocity[1] > 0.0 || self.body.gap >= 1.5) {
            self.plunge_start();
        }
    }

    fn plunge_start(&mut self) {
        (self.phase, self.air_used, self.air_armed, self.air_sub) = (8, true, false, 12);
        self.set_horizontal([0.0; 3]);
    }

    /// The plunge (phases 8 and 9): clip 23's root motion while it rises, then a fall at -8.0 at gain 0.3;
    /// within 1.0 of the ground phase 9 with clip 24, ending in state 1.
    fn plunge(&mut self, clip: ClipState) {
        if self.phase == 8 {
            self.request = 23;
            if clip.action != 23 || self.root_y > 0.0 {
                self.body.no_gravity = true;
                self.body.velocity = [0.0; 3];
                return;
            }
            self.body.no_gravity = false;
            self.body.gain = 0.3;
            if self.body.velocity[1] > -8.0 {
                self.body.velocity[1] = -8.0;
            }
            if self.body.gap <= 1.0 {
                self.phase = 9;
            }
            return;
        }
        self.request = 24;
        self.body.velocity = [0.0; 3];
        if clip.action == 24 && clip.progress >= 0.99 {
            self.state = STANCE;
            self.body.gain = 1.0;
            self.air_sub = 0;
            self.air_used = false;
        }
    }

    fn enter_air(&mut self, state: u8) {
        (self.air_used, self.air_armed, self.air_hold, self.air_sub) = (false, false, 0, 0);
        self.state = state;
        self.phase = -1;
        self.start_height = self.body.position[1];
    }

    fn horizontal(&self) -> Vec3 {
        [self.body.velocity[0], 0.0, self.body.velocity[2]]
    }

    fn set_horizontal(&mut self, h: Vec3) {
        self.body.velocity[0] = h[0];
        self.body.velocity[2] = h[2];
    }

    /// The air routine of Behavior 50 for one pass.
    fn air(&mut self, stick: bool, clip: ClipState) {
        let playing = |id: u32| clip.action == id;
        if matches!(self.phase, 8 | 9) {
            return self.plunge(clip);
        }
        // The slash ends at progress 0.99 of its clip; the hero goes on falling.
        if self.air_sub == 11 && playing(25) && clip.progress >= 0.99 {
            self.air_sub = 0;
            self.request = ANIM_FALL_MOVE;
        }
        if self.phase == -1 {
            // Launch: gain 1.0, velocity cleared, vertical 8.75, horizontal facing times S.
            self.body.gain = 1.0;
            // Behavior 11: a strafe jump flies at 9.0 along the stored direction.
            if (43..=46).contains(&self.state) {
                let d = self.jump_dir;
                self.body.velocity = [d[0] * 9.0, LAUNCH, d[2] * 9.0];
                self.request = match self.state {
                    43 => 102,
                    44 => 104,
                    45 => 65,
                    _ => 70,
                };
                self.phase = 1;
                return;
            }
            let s = jump_speed(self.state);
            let f = self.facing;
            self.body.velocity = [f[0] * s, LAUNCH + std::mem::take(&mut self.launch_bonus), f[2] * s];
            (self.phase, self.request) = if s > 0.01 { (1, ANIM_JUMP_MOVE) } else { (0, ANIM_JUMP_STILL) };
            return;
        }
        // Air steering in phases 0 to 5: drag, then the stick push below the speed cap. Strafe jumps
        // run none.
        if (0..=5).contains(&self.phase) && !(43..=46).contains(&self.state) {
            let h = self.horizontal();
            let len = dot(h, h).sqrt();
            let mut h2 = h;
            if len > 0.0 {
                let d = len.min(AIR_DRAG) / len;
                h2 = h.map(|c| c - c * d);
            }
            if stick && len < AIR_CAP {
                for k in [0, 2] {
                    h2[k] += AIR_PUSH * self.target[k];
                }
            }
            self.set_horizontal(h2);
        }
        let vy = self.body.velocity[1];
        let gap = self.body.gap;
        match self.phase {
            0 | 1 => {
                if vy >= 12.0 {
                    self.body.gain = GAIN_RISE;
                } else {
                    self.body.gain = GAIN_FALL;
                    let h = self.horizontal();
                    let still = dot(h, self.facing) < 0.0 || dot(h, h).sqrt() < 0.01;
                    self.phase = if still { 2 } else { 4 };
                }
            }
            2..=4 => {
                self.body.gain = GAIN_FALL;
                // Or the floor-contact flag (a block top the probe does not report).
                if vy < -7.0 || (vy < 0.0 && gap <= LAND_GAP) || self.body.floor {
                    self.phase = 5;
                    self.body.gain = GAIN_DESCENT;
                }
            }
            5 => {
                if gap <= LAND_GAP || self.body.floor {
                    let fall = self.start_height - self.body.position[1];
                    self.phase = if fall > HARD_FALL {
                        10
                    } else if !stick {
                        6
                    } else {
                        7
                    };
                    self.body.gain = 1.0;
                } else {
                    self.body.gain = GAIN_DESCENT;
                }
            }
            6 => {
                self.request = ANIM_SOFT_LAND;
                if gap <= 0.1 {
                    self.set_horizontal([0.0; 3]);
                }
                if playing(ANIM_SOFT_LAND) && clip.progress >= 0.99 {
                    self.state = STANCE;
                }
            }
            7 => {
                // Moving landing: the state field becomes 4 directly with request 4 (a strafe jump
                // writes state 32).
                self.set_horizontal([0.0; 3]);
                self.state = if (43..=46).contains(&self.state) { STRAFE } else { RUN };
            }
            10 => {
                self.request = ANIM_HARD_LAND;
                if gap <= 0.1 {
                    self.set_horizontal([0.0; 3]);
                    if self.body.velocity[1] > 0.0 {
                        self.body.velocity[1] = 0.0;
                    }
                }
                self.body.gain = 1.0;
                if playing(ANIM_HARD_LAND) && clip.progress >= 0.99 {
                    self.state = STANCE;
                    self.body.no_gravity = false;
                }
            }
            _ => {}
        }
        // Clip tails: 76 goes to 78 when it ends, 70 to 67.
        if (0..=5).contains(&self.phase) && clip.finished {
            if playing(ANIM_JUMP_MOVE) {
                self.request = ANIM_FALL_MOVE;
            } else if playing(ANIM_JUMP_STILL) {
                self.request = ANIM_FALL_STILL;
            }
        }
    }

    /// The motion-record tick (SPEC-0040 Behavior 14, cadence): the ground step, the root
    /// travel, the ground constraint, gravity, the contacts (`contacts` moves the position) and
    /// the integrator.
    pub fn record_tick(
        &mut self,
        root_step: Vec3,
        probe: impl Fn(f32, f32, f32) -> (f32, Vec3),
        contacts: impl FnOnce(&mut Vec3, &mut Vec3) -> bool,
    ) {
        let start = self.body.position;
        self.body.measure(probe);
        // SPEC-0033 Floor-contact flag: a record more than 0.05 inside a floor resolves a
        // contact against it, which sets the flag (here for the probed ground as well as the
        // contact surfaces; Medium).
        let inside = self.body.gap < -0.05 && self.body.normal[1] > 0.9;
        self.travel(root_step);
        for k in 0..3 {
            self.body.position[k] += self.extra[k];
        }
        self.extra = [0.0; 3];
        self.body.constrain();
        self.body.gravity();
        // The driver clears the floor-contact flag, then a floor contact sets it.
        self.body.floor = inside;
        if contacts(&mut self.body.position, &mut self.body.velocity) {
            self.body.floor_contact();
        }
        self.body.integrate();
        self.last_motion = std::array::from_fn(|k| self.body.position[k] - start[k]);
        self.follow.tick(self.body.position[1], self.state);
    }

    /// The travel step (SPEC-0040 Behavior 14C): the root step written on the previous pass,
    /// discarded when the stick points behind the facing, turned by the facing basis, projected
    /// onto the ground plane while the gap is below 1.0, and added to the position.
    pub fn travel(&mut self, root_step: Vec3) {
        // The step is kept with the stick behind the facing in states 21, 23 and 41 (SPEC-0033
        // Behavior 14C).
        // SPEC-0033 Behavior 4 item 9: in state 32 the stick re-aims the step, keeping its length.
        let root_step = match self.strafe_stick.filter(|_| self.state == STRAFE) {
            Some((s, f)) => {
                let l = dot(root_step, root_step).sqrt();
                let d = (s * s + f * f).sqrt().max(1e-6);
                [s / d * l, 0.0, f / d * l]
            }
            None => root_step,
        };
        self.root_y = root_step[1];
        let root_step = root_step.map(|v| v * self.haste_travel());
        let step = if dot(self.target, self.facing) < 0.0 && !matches!(self.state, PANEL | 23 | LADDER | STRAFE) { [0.0; 3] } else { root_step };
        let [b1, b2, b3] = basis(self.facing);
        let mut d: Vec3 = std::array::from_fn(|k| step[0] * b1[k] + step[1] * b2[k] + step[2] * b3[k]);
        // In the edge states the clips' root motion lifts the hero (the pull-up 59 rises 3.7 and
        // the climb 134 1.2), so it is not flattened onto the ground there.
        if self.body.gap < 1.0 && !matches!(self.state, HANG | CLIMB | LADDER | PANEL | LEDGE | LEDGE_UP | LEDGE_RISE | LINE) {
            let n = self.body.normal;
            let dn = dot(d, n);
            d = std::array::from_fn(|k| d[k] - dn * n[k]);
        }
        for k in 0..3 {
            self.body.position[k] += d[k];
        }
        // Hero Haste in a jump: the flight's speed, kept as the same velocity so the air rules are unchanged.
        if self.haste > 1.0 && self.airborne() {
            let v = self.body.velocity;
            let extra = (self.haste - 1.0) / 30.0;
            self.body.position[0] += v[0] * extra;
            self.body.position[2] += v[2] * extra;
        }
    }

    /// Hero Haste splits between the legs and the ground covered: the clip plays up to 2.5 times as fast (its root
    /// step with it), and the root step takes the rest.
    pub fn haste_rate(&self) -> f32 {
        if self.haste_moving() { self.haste.min(HASTE_LEGS) } else { 1.0 }
    }

    fn haste_travel(&self) -> f32 {
        if self.haste_moving() { self.haste / self.haste_rate() } else { 1.0 }
    }

    fn haste_moving(&self) -> bool {
        matches!(self.state, SLOW_WALK | WALK | RUN | STRAFE)
    }
}

/// Hero Haste's way back: with no real floor under him, the hero has left the world once he stands on the off-grid
/// height (`gap` near 0) or has dropped this far below where he last stood. A jump across a gap does neither.
pub fn haste_left_world(floor: bool, gap: f32, dropped: f32) -> bool {
    !floor && (gap < 1.0 || dropped > 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_haste_left_world() {
        assert!(!haste_left_world(true, 500.0, 500.0), "a long fall to a real floor");
        assert!(!haste_left_world(false, 1.0e9, 5.0), "a jump across a gap");
        assert!(haste_left_world(false, 1.0e9, 21.0), "falling with nothing below");
        assert!(haste_left_world(false, 0.0, 0.0), "standing on the off-grid height");
    }

    /// Hero Haste: the clip rate times the root step's factor is the setting; other states and off are unchanged.
    #[test]
    fn hero_haste() {
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        h.state = RUN;
        h.target = FWD;
        h.haste = 10.0;
        assert_eq!(h.haste_rate(), 2.5);
        h.travel([0.0, 0.0, 1.0]);
        assert!((h.position()[2] - 4.0).abs() < 1e-5, "{:?}", h.position());
        h.haste = 2.0;
        assert_eq!(h.haste_rate(), 2.0);
        h.state = HURT;
        assert_eq!(h.haste_rate(), 1.0);
        let z = h.position()[2];
        h.travel([0.0, 0.0, 1.0]);
        assert!((h.position()[2] - z - 1.0).abs() < 1e-5);
    }

    /// SPEC-0016 Hero get-up state 54: the clip 137 is asked for, and at its progress 0.99 the
    /// hero stands (state 1). Report 159: the state asked for idle 0 and never ended.
    #[test]
    fn get_up_plays_its_clip_and_ends() {
        let (c, left) = ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        h.state = GET_UP;
        h.update(&HeroInput::default(), c, left, ClipState { action: 0, progress: 0.99, finished: false });
        assert_eq!((h.state, h.action()), (GET_UP, ANIM_LONG_LAND));
        h.update(&HeroInput::default(), c, left, ClipState { action: ANIM_LONG_LAND, progress: 0.5, finished: false });
        assert_eq!(h.state, GET_UP);
        h.update(&HeroInput::default(), c, left, ClipState { action: ANIM_LONG_LAND, progress: 0.99, finished: false });
        assert_eq!(h.state, STANCE);
    }

    /// SPEC-0036 State 17: back to state 32 only with L2 held and state 32 before the hit.
    #[test]
    fn hurt_returns_to_strafe() {
        let (c, left) = ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let end = |id| ClipState { action: id, progress: 0.99, finished: false };
        for (from, l2, to) in [(STRAFE, true, STRAFE), (STRAFE, false, STANCE), (STANCE, true, STANCE)] {
            let mut h = Locomotion::spawn([0.0; 3], FWD);
            h.state = from;
            h.hurt(true);
            h.hurt(true);
            let held = HeroInput { l2, ..HeroInput::default() };
            h.update(&held, c, left, end(h.request));
            assert_eq!(h.state, to);
        }
    }

    /// SPEC-0036's vector: pair length 1.0 with the camera along +z gives (0, 0, 7.5). A centred stick
    /// leaves the velocity alone; Roberta's keeps the vertical part.
    #[test]
    fn attack_stick_step() {
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        h.state = ATTACK;
        h.stick_step = StickStep::Clear;
        h.body.velocity = [1.0, -3.0, 1.0];
        let (c, left) = ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        h.update(&HeroInput::default(), c, left, ClipState::default());
        assert_eq!(h.body.velocity, [1.0, -3.0, 1.0]);
        let one = HeroInput { forward: 1.0 / STICK_FULL, ..HeroInput::default() };
        h.update(&one, c, left, ClipState::default());
        assert!(h.body.velocity[0].abs() < 1e-5 && h.body.velocity[1] == 0.0 && (h.body.velocity[2] - 7.5).abs() < 1e-4, "{:?}", h.body.velocity);
        h.stick_step = StickStep::Horizontal;
        h.body.velocity[1] = -3.0;
        h.update(&one, c, left, ClipState::default());
        assert_eq!(h.body.velocity[1], -3.0);
    }

    /// SPEC-0033 Behavior 50 Air attacks: the plunge from a jump; the slash on a strafe jump's release.
    #[test]
    fn plunge_and_air_slash() {
        let mut h = Locomotion::spawn([0.0, 10.0, 0.0], FWD);
        h.air_ok = true;
        h.state = JUMP;
        h.phase = 2;
        h.body.gap = 5.0;
        let press = HeroInput { attack: true, ..HeroInput::default() };
        let c = [0.0, 0.0, 1.0];
        let left = [1.0, 0.0, 0.0];
        h.update(&press, c, left, ClipState::default());
        h.update(&HeroInput::default(), c, left, ClipState::default());
        assert_eq!((h.phase, h.air_sub, h.request), (8, 12, 23));
        // Clip 23 playing, its root no longer rising: the fall at -8.0, then phase 9 within 1.0.
        let clip = ClipState { action: 23, progress: 0.5, finished: false };
        h.update(&HeroInput::default(), c, left, clip);
        assert_eq!(h.body.velocity[1], -8.0);
        h.body.gap = 0.9;
        h.update(&HeroInput::default(), c, left, clip);
        assert_eq!(h.phase, 9);
        h.update(&HeroInput::default(), c, left, ClipState { action: 24, progress: 1.0, finished: true });
        assert_eq!((h.state, h.air_sub), (STANCE, 0));
        // A strafe jump: armed by the press, the slash on release.
        let mut h = Locomotion::spawn([0.0, 10.0, 0.0], FWD);
        h.air_ok = true;
        h.state = 46;
        h.phase = 3;
        h.body.gap = 5.0;
        h.update(&press, c, left, ClipState::default());
        assert_eq!(h.air_sub, 0);
        h.update(&HeroInput::default(), c, left, ClipState::default());
        assert_eq!((h.air_sub, h.request), (11, 25));
        // Held: the 12th tick gives the plunge.
        let mut h = Locomotion::spawn([0.0, 10.0, 0.0], FWD);
        h.air_ok = true;
        h.state = 46;
        h.phase = 3;
        h.body.gap = 50.0;
        let held = HeroInput { circle_held: true, ..HeroInput::default() };
        for _ in 0..11 {
            h.update(&held, c, left, ClipState::default());
            assert_ne!(h.phase, 8);
        }
        h.update(&held, c, left, ClipState::default());
        assert_eq!(h.phase, 8);
    }

    /// SPEC-0033 Behavior 4 and its test vector, and Behavior 11's jump.
    #[test]
    fn strafe_codes_heading_and_jump() {
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        h.body.normal = [0.0, 1.0, 0.0];
        let clip = ClipState::default();
        let c = [1.0, 0.0, 0.0];
        let left = [0.0, 0.0, 1.0];
        let input = HeroInput { l2: true, forward: 0.25, side: 0.5, ..HeroInput::default() };
        h.update(&input, c, left, clip);
        assert_eq!((h.state, h.action()), (STRAFE, 154));
        // The heading follows the camera, not the stick.
        for _ in 0..40 {
            h.update(&input, c, left, clip);
        }
        assert!(dot(h.facing, c) > 0.99);
        // The stick re-aims the step: a sideways push moves it sideways at the clip's length.
        h.facing = FWD;
        h.travel([0.0, 0.0, 0.2252]);
        let moved = h.body.position;
        assert!((dot(moved, moved).sqrt() - 0.2252).abs() < 1e-3);
        // A pushed Cross takes the strafe jump chosen by the action: 9.0 along the stick, no steering.
        let jump = HeroInput { jump: true, ..input };
        h.update(&jump, c, left, clip);
        assert_eq!(h.state, 44);
        h.update(&HeroInput { l2: true, ..HeroInput::default() }, c, left, clip);
        let v = h.body.velocity;
        assert!((v[0].hypot(v[2]) - 9.0).abs() < 1e-3 && v[1] > 8.0);
        assert_eq!(h.action(), 104);
        // Released: back to idle.
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        h.body.normal = [0.0, 1.0, 0.0];
        h.update(&HeroInput { l2: true, ..HeroInput::default() }, c, left, clip);
        assert_eq!((h.state, h.action()), (STRAFE, 0));
        h.update(&HeroInput::default(), c, left, clip);
        assert_eq!(h.state, STANCE);
    }

    const FWD: Vec3 = [0.0, 0.0, 1.0];
    const LEFT: Vec3 = [1.0, 0.0, 0.0];

    fn flat(_: f32, _: f32, _: f32) -> (f32, Vec3) {
        (0.0, [0.0, 1.0, 0.0])
    }

    /// One tick on flat ground with no root motion: hero update, then the record tick.
    fn tick(h: &mut Locomotion, input: &HeroInput, probe: impl Fn(f32, f32, f32) -> (f32, Vec3)) {
        h.update(input, FWD, LEFT, ClipState::default());
        h.record_tick([0.0; 3], probe, |_, _| false);
    }

    fn angle(a: Vec3, b: Vec3) -> f32 {
        dot(a, b).clamp(-1.0, 1.0).acos().to_degrees()
    }

    #[test]
    fn height_follow() {
        // Walking: the follow catches up; a 0.1 step closes smoothly over many ticks and stops
        // within 0.05.
        let mut f = HeightFollow::at(0.0);
        let mut seen = Vec::new();
        for _ in 0..400 {
            f.tick(0.1, WALK);
            seen.push(f.shown);
        }
        assert!(seen[0] > 0.0 && seen[0] < 0.01, "{}", seen[0]);
        assert!(seen.windows(2).all(|w| w[1] >= w[0]));
        assert!((0.1 - f.shown) < 0.05 + 1e-6);
        // Jumping: a rise up to 3.5 waits (120 ticks); a drop over 0.5 starts at once.
        let mut f = HeightFollow::at(0.0);
        f.tick(2.0, 8);
        assert_eq!(f.shown, 0.0);
        f.tick(-1.0, 8);
        assert!(f.shown < 0.0);
    }

    #[test]
    fn a_knock_from_a_no_gravity_hold_falls() {
        // A ledge hold switches gravity off; the knock must switch it back on so the long fall
        // comes down (it used to keep rising at the knock's speed).
        let mut h = Locomotion::spawn([0.0, 10.0, 0.0], FWD);
        h.body.no_gravity = true;
        h.body.no_snap = true;
        h.knock([0.0, 4.0, 1.0]);
        assert_eq!(h.state, LONG_FALL);
        assert!(!h.body.no_gravity && !h.body.no_snap);
        let first = h.body.velocity[1];
        for _ in 0..30 {
            tick(&mut h, &HeroInput::default(), |_, _, _| (-100.0, [0.0, 1.0, 0.0]));
        }
        assert!(h.body.velocity[1] < first, "{} then {}", first, h.body.velocity[1]);
    }

    #[test]
    fn steering_matches_spec_turn_sequence() {
        // SPEC-0056: a 90-degree turn with w = 0.3 leaves 66.80, 48.18, 34.23 degrees after
        // ticks 1 to 3; with w = 0.1 it first drops below 10 degrees (9.77) on tick 23.
        let t = [1.0, 0.0, 0.0];
        let mut f = [0.0, 0.0, 1.0];
        for want in [66.80, 48.18, 34.23, 24.13, 16.96, 11.89, 8.33] {
            f = steer(f, t, 0.3);
            assert!((angle(f, t) - want).abs() < 0.02, "{} vs {want}", angle(f, t));
        }
        let mut f = [0.0, 0.0, 1.0];
        for _ in 0..23 {
            f = steer(f, t, 0.1);
        }
        assert!((angle(f, t) - 9.77).abs() < 0.02, "{}", angle(f, t));
    }

    #[test]
    fn speed_value_matches_spec_tests() {
        let v = |side, forward| speed_value(&HeroInput { side, forward, ..Default::default() });
        assert_eq!(v(0.0, 1.0), 1.0);
        assert_eq!(v(0.8, 0.6), 1.0);
        assert!((v(0.5, 0.25) - 0.559).abs() < 1e-3);
        assert_eq!(v(1.0, 1.0), 1.0, "capped");
    }

    #[test]
    fn basis_matches_spec_example() {
        assert_eq!(basis([0.0, 0.0, 1.0]), [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    #[test]
    fn moving_turn_follows_spec_sequence_from_90_degrees() {
        let u = [1.0, 0.0, 0.0];
        let mut f = [0.0, 0.0, 1.0];
        for want in [71.81, 55.99, 43.03, 32.78, 24.84] {
            f = turn_moving(f, u);
            assert!((angle(f, u) - want).abs() < 0.05, "{} vs {want}", angle(f, u));
        }
    }

    #[test]
    fn tiers_match_spec_speeds() {
        // Walk at R = 1 moves 0.2251 per pass; speed = 0.2251 × factor × 60.
        let speed = |v: f32, run: bool| tier(v, run).map(|(_, f)| 0.2251 * f * 60.0);
        assert_eq!(tier(0.1, false), None);
        assert_eq!(tier(0.5, false).unwrap().0, WALK);
        assert!((speed(0.5, false).unwrap() - 6.75).abs() < 0.01);
        assert_eq!(tier(0.49, false), Some((SLOW_WALK, 0.98)));
        assert_eq!(tier(0.66, true).unwrap().0, RUN);
        assert_eq!(tier(0.65, true), Some((WALK, 0.65)));
        assert_eq!(tier(0.2, true), Some((SLOW_WALK, (0.3f32 / 0.33).min(1.0))));
    }

    #[test]
    fn stick_behind_turns_in_place_before_moving() {
        // SPEC-0033 5.5: at 135° the first two passes move 0, the third moves a full step.
        let mut h = Locomotion::spawn([0.0; 3], [0.0, 0.0, 1.0]);
        let back_left = [(0.5f32).sqrt(), 0.0, -(0.5f32).sqrt()];
        // Camera forward +Z, screen-left +X: stick fully back and left points along back_left.
        let input = HeroInput { side: (0.5f32).sqrt(), forward: -(0.5f32).sqrt(), ..Default::default() };
        let mut moved = Vec::new();
        for _ in 0..3 {
            h.update(&input, FWD, LEFT, ClipState::default());
            let before = h.position();
            h.travel([0.0, 0.0, 0.2251]);
            let p = h.position();
            moved.push(((p[0] - before[0]).powi(2) + (p[2] - before[2]).powi(2)).sqrt());
        }
        assert!((angle(h.target, back_left)).abs() < 1e-3);
        assert_eq!(&moved[..2], &[0.0, 0.0]);
        assert!((moved[2] - 0.2251).abs() < 1e-4);
    }

    #[test]
    fn travel_projects_onto_the_ground_plane_near_the_ground() {
        // A 1.0 step along +z on a plane tilted toward +z (normal (0, s, -s)) loses its normal part.
        let mut h = Locomotion::spawn([0.0; 3], [0.0, 0.0, 1.0]);
        let r = (0.5f32).sqrt();
        h.body.normal = [0.0, r, -r];
        h.travel([0.0, 0.0, 1.0]);
        let p = h.position();
        assert!((p[1] - 0.5).abs() < 1e-6 && (p[2] - 0.5).abs() < 1e-6, "{p:?}");
    }

    #[test]
    fn run_needs_r1_and_the_restart_wait() {
        let mut h = Locomotion::spawn([0.0; 3], [0.0, 0.0, 1.0]);
        let full = HeroInput { forward: 1.0, run: true, ..Default::default() };
        h.update(&full, FWD, LEFT, ClipState::default());
        assert_eq!((h.state, h.action()), (RUN, 4));
        h.update(&HeroInput::default(), FWD, LEFT, ClipState::default());
        assert_eq!(h.state, STANCE);
        h.update(&full, FWD, LEFT, ClipState::default());
        assert_eq!(h.state, WALK, "running cannot restart for 30 passes");
    }

    #[test]
    fn standing_jump_follows_the_spec_flight_curve() {
        // SPEC-0033 Behavior 50: heights at ticks 0, 5, ..., 47, then -0.0177 and 0.0.
        let mut h = Locomotion::spawn([0.0; 3], FWD);
        let want = [
            (0, 0.2561), (5, 1.3767), (10, 2.2306), (15, 2.8178), (20, 3.1383), (24, 3.2028),
            (30, 2.9794), (35, 2.5000), (40, 1.7539), (45, 0.7411), (46, 0.5048), (47, 0.2329),
            (48, -0.0177), (49, 0.0),
        ];
        let mut heights = Vec::new();
        for t in 0..50 {
            let jump = HeroInput { jump: t == 0, ..Default::default() };
            tick(&mut h, &jump, flat);
            heights.push(h.position()[1]);
            if t == 46 {
                assert_eq!(h.phase, 5, "phase 5 at tick 46");
            }
            if t == 47 {
                assert_eq!(h.phase, 6, "soft landing decided at tick 47");
            }
        }
        for (t, y) in want {
            assert!((heights[t] - y).abs() < 2e-3, "tick {t}: {} vs {y}", heights[t]);
        }
        assert_eq!(h.body.velocity[1], 0.0);
    }

    #[test]
    fn running_jump_drifts_by_the_spec_distances() {
        // Behavior 50 air steering, state 10: stick centred after launch 2.85 at the landing
        // decision (tick 47); stick held fully forward 14.0.
        for (held, want) in [(false, 2.85), (true, 14.0)] {
            let mut h = Locomotion::spawn([0.0; 3], FWD);
            for t in 0..48 {
                let input = HeroInput { forward: if t == 0 || held { 1.0 } else { 0.0 }, jump: t == 0, ..Default::default() };
                tick(&mut h, &input, flat);
                if t == 0 {
                    assert_eq!(h.state, JUMP_RUN);
                }
            }
            assert!((h.position()[2] - want).abs() < 0.02, "held {held}: {}", h.position()[2]);
        }
    }

    #[test]
    fn walk_off_from_ten_units_lands_hard() {
        // Behavior 9A: from 10 units the gate passes, phase 5 comes 10 ticks later and the
        // hard-landing decision 20 ticks after that, at height 0.0409.
        let mut h = Locomotion::spawn([0.0, 10.0, 0.0], FWD);
        let mut gate = None;
        let mut five = None;
        let mut decision = None;
        for t in 0..80 {
            let before = h.phase;
            let air = h.airborne();
            tick(&mut h, &HeroInput::default(), flat);
            if !air && h.airborne() {
                gate = Some(t);
            }
            if before != 5 && h.phase == 5 {
                five = Some(t);
            }
            if before == 5 && h.phase == 10 {
                decision = Some((t, h.position()[1]));
            }
        }
        let (g, f, (d, y)) = (gate.unwrap(), five.unwrap(), decision.unwrap());
        assert_eq!((f - g, d - f), (10, 20));
        assert!((y - 0.0409).abs() < 2e-3, "{y}");
    }

    #[test]
    fn height_follow_clamps_a_huge_gap() {
        // A 2,700-unit jump in one tick: the displayed height reaches the world height and stays
        // finite; a 10-unit gap closes as before (0.2 the first tick).
        let mut f = HeightFollow::at(-1175.0);
        for _ in 0..10 {
            f.tick(1536.0, STANCE);
        }
        assert!(f.shown.is_finite() && (f.shown - 1536.0).abs() < 0.05, "{}", f.shown);
        let mut f = HeightFollow::at(0.0);
        f.tick(10.0, STANCE);
        assert!((f.shown - 0.2).abs() < 1e-5, "{}", f.shown);
    }
}
