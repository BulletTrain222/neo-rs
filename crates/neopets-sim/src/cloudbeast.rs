//! The Cloud Beast (SPEC-0010): the class-9 boss controller, its gust column, the two
//! lightning beams and the bouncing balls.
//!
//! The boss is cut content: the disc has its controller and tuning words but no model, rig,
//! clip bank, arena or effects. What this build supplies itself (marked "stand-in" below and
//! recorded in docs/STATUS.md): the clip lengths and the bouncing event's place in its clip
//! (`clip_len`), the stand-in rig's bone 6 (`BONE6`), and, in the host, the arena ring and the
//! companion's pose point. Everything else follows the spec.

use crate::combat::{overlap, segment_distance, Cylinder};
use crate::rng::Rng;

pub type Vec3 = [f32; 3];

/// The tuning block (SPEC-0010 Layout): 82 words from 0x2DA4D0.
pub const BASE: u32 = 0x002D_A4D0;
pub const WORDS: usize = 82;

/// Stand-in rig: bone 6 (the gust origin), up and forward of the boss's ground point.
pub const BONE6: (f32, f32) = (3.4, 2.4);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    /// Hesitation (min, max) seconds: stage 1, then stages 2 and 3.
    pub hesitation: [(f32, f32); 2],
    /// Weights by stage: fidget, air-flow, rapid, lightning, bouncing.
    pub weights: [[i32; 5]; 3],
    /// Cooldowns (min, max) seconds by stage.
    pub lightning_timer: [(f32, f32); 3],
    pub air_timer: [(f32, f32); 3],
    pub bounce_timer: [(f32, f32); 3],
    pub shock_range: f32,
    pub gust_speed: [f32; 3],
    pub rapid: (i32, i32),
    pub gust_damage: i32,
    /// Wind-up seconds for stages 1 and 2 (stage 3 sets none).
    pub windup: [f32; 2],
    pub fire_time: [f32; 3],
    pub bolt_damage: f32,
    pub shock_radius: f32,
    pub shock_damage: i32,
    pub ball_range: f32,
    pub balls: [i32; 3],
    pub ball_speed: [f32; 3],
    pub ball_damage: f32,
    pub knock_speed: f32,
    pub gust_travel: f32,
    pub gust_radius: f32,
    pub growth: f32,
    pub guard_shift: f32,
    pub ball_vy: f32,
    pub hold: f32,
    pub reply_damage: i32,
    pub ball_gravity: f32,
    pub ball_radius: f32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |a: u32| b[((a - BASE) / 4) as usize];
        let i = |a: u32| w(a) as i32;
        let f = |a: u32| f32::from_bits(w(a));
        let pair = |a: u32| (f(a), f(a + 4));
        let pairs = |a: u32| [pair(a), pair(a + 8), pair(a + 16)];
        Some(Self {
            hesitation: [pair(0x2D_A4D0), pair(0x2D_A4D8)],
            weights: [0, 1, 2].map(|s| [0x2D_A4E8, 0x2D_A4F4, 0x2D_A500, 0x2D_A50C, 0x2D_A518].map(|a| i(a + 4 * s))),
            lightning_timer: pairs(0x2D_A524),
            air_timer: pairs(0x2D_A53C),
            bounce_timer: pairs(0x2D_A554),
            shock_range: f(0x2D_A56C),
            gust_speed: [0, 1, 2].map(|s| f(0x2D_A580 + 4 * s)),
            rapid: (i(0x2D_A590), i(0x2D_A594)),
            gust_damage: i(0x2D_A598),
            windup: [f(0x2D_A5A0), f(0x2D_A5A4)],
            fire_time: [0, 1, 2].map(|s| f(0x2D_A5A8 + 4 * s)),
            bolt_damage: f(0x2D_A5B4),
            shock_radius: f(0x2D_A5B8),
            shock_damage: i(0x2D_A5BC),
            ball_range: f(0x2D_A5C0),
            balls: [0, 1, 2].map(|s| i(0x2D_A5C4 + 4 * s)),
            ball_speed: [0, 1, 2].map(|s| f(0x2D_A5D0 + 4 * s)),
            ball_damage: f(0x2D_A5DC),
            knock_speed: f(0x2D_A5E0),
            gust_travel: f(0x2D_A5E8),
            gust_radius: f(0x2D_A5EC),
            growth: f(0x2D_A5F4),
            guard_shift: f(0x2D_A5FC),
            ball_vy: f(0x2D_A600),
            hold: f(0x2D_A604),
            reply_damage: i(0x2D_A608),
            ball_gravity: f(0x2D_A60C),
            ball_radius: f(0x2D_A610),
        })
    }
}

/// Stand-in clip lengths in ticks, and whether the clip loops (the disc has no clip bank).
pub fn clip_len(id: u32) -> (f32, bool) {
    match id {
        4 => (120.0, true),
        5 => (90.0, false),
        6 => (40.0, false),
        7 => (60.0, false),
        22 => (40.0, false),
        8 => (60.0, true),
        10 => (40.0, false),
        11 => (12.0, false),
        12 => (30.0, true),
        13 => (30.0, false),
        14 => (70.0, false),
        15 | 16 => (50.0, false),
        23 => (40.0, true),
        _ => (60.0, false),
    }
}

/// The bouncing clip's event 32 (the ball spawn), as clip progress (stand-in).
pub const BALL_EVENT: f32 = 0.5;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub id: u32,
    pub t: f32,
}

impl Clip {
    pub fn progress(&self) -> f32 {
        let (len, looped) = clip_len(self.id);
        if looped { (self.t / len).fract() } else { (self.t / len).min(1.0) }
    }
    pub fn finished(&self) -> bool {
        let (len, looped) = clip_len(self.id);
        !looped && self.t >= len
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
    #[default]
    Idle,
    Shock,
    Fidget,
    AirFlow,
    Rapid,
    Lightning,
    Bouncing,
    Reply,
    Reaction,
}

/// A beam object (SPEC-0010 Lightning, Beam objects).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Beam {
    pub active: bool,
    pub start: Vec3,
    pub dir: Vec3,
    pub len: f32,
    target: f32,
}

impl Beam {
    fn reset(&mut self) {
        *self = Beam { active: true, target: f32::MAX, ..Default::default() };
    }
    fn set(&mut self, a: Vec3, b: Vec3, growth: f32) {
        let d = sub(b, a);
        self.target = len(d);
        self.start = a;
        self.dir = unit(d);
        if self.len < self.target - growth {
            self.len += growth;
        } else {
            self.len = self.target;
        }
    }
    fn stop(&mut self) {
        self.active = false;
    }
    fn extended(&self) -> bool {
        self.target <= self.len
    }
    pub fn end(&self) -> Vec3 {
        add(self.start, scale(self.dir, self.len))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ball {
    pub position: Vec3,
    prev: Vec3,
    dir: Vec3,
    vy: f32,
    travel: f32,
}

/// What the boss sees this tick.
pub struct View<'a> {
    /// The boss's position (its ground point) and horizontal forward.
    pub position: Vec3,
    pub forward: Vec3,
    pub hp: i32,
    pub max_hp: i32,
    pub hero: Vec3,
    pub hero_forward: Vec3,
    pub hero_radius: f32,
    pub hero_height: f32,
    /// The hero variant: 0 Tor, 1 Roberta.
    pub variant: u8,
    /// Tor's in-attack test.
    pub in_attack: bool,
    /// The hero's guard query, and the Fog Mote (166) in the shield word.
    pub guard: bool,
    pub fog: bool,
    /// Roberta's armed shots: (position, travel per update).
    pub shots: &'a [(Vec3, Vec3)],
    /// The companion's pose point 1, when the companion exists.
    pub companion: Option<Vec3>,
    /// The six arena points (position, facing); None: the boss stays where it is.
    pub points: Option<[(Vec3, Vec3); 6]>,
    /// The ground probe (x, z, ceiling).
    pub ground: &'a dyn Fn(f32, f32, f32) -> f32,
    /// The beam's wall sweep from a start to an end: the nearest hit fraction and the face
    /// normal.
    pub wall: &'a dyn Fn(Vec3, Vec3) -> Option<(f32, Vec3)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    /// Damage to the hero (not blocked).
    Damage(i32),
    /// The blocked take-hit with amount 0.
    Blocked,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Act {
    /// The boss's new position and facing.
    pub position: Option<Vec3>,
    pub forward: Vec3,
    pub hits: Vec<Hit>,
    /// The gust's knock-back velocity for the hero (state 48).
    pub knock: Option<Vec3>,
    /// Damage the boss deals itself (the special reply).
    pub self_damage: i32,
    /// The reply's hit effect, requested this tick.
    pub hit_fx: bool,
    /// Ball bursts (touch or range end).
    pub bursts: Vec<Vec3>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cloud {
    pub state: State,
    pub stage: u8,
    sub: u8,
    hesitation: i32,
    air_cd: i32,
    lightning_cd: i32,
    bounce_cd: i32,
    sub_timer: i32,
    fire_total: i32,
    latch: bool,
    rapid: i32,
    hold: i32,
    reply_anim: u32,
    balls_spawned: bool,
    pub clip: Clip,
    pub speed: f32,
    /// The gust column (O, E) and its charge.
    pub gust: Option<(Vec3, Vec3)>,
    charge: f32,
    pub fade: i32,
    pub beams: [Beam; 2],
    pub balls: [Option<Ball>; 5],
    pub shock: bool,
    /// The arena ring indices.
    current: usize,
    next: Option<usize>,
    previous: usize,
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}
fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 1e-6 { scale(a, 1.0 / l) } else { [0.0; 3] }
}
fn flat(a: Vec3) -> Vec3 {
    unit([a[0], 0.0, a[2]])
}
fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

impl Cloud {
    /// Start-up (Test spawn walkthrough, tick 0): idle, stage 1, hesitation 60 to 120 ticks, the
    /// air-flow cooldown 180 ticks, point 0.
    pub fn new(w: &Words, rng: &mut Rng) -> Self {
        let (lo, hi) = w.hesitation[0];
        Self {
            state: State::Idle,
            stage: 1,
            sub: 0,
            hesitation: rng.range((60.0 * lo) as i32, (60.0 * hi) as i32),
            air_cd: 180,
            lightning_cd: 0,
            bounce_cd: 0,
            sub_timer: 0,
            fire_total: 0,
            latch: false,
            rapid: 0,
            hold: 0,
            reply_anim: 15,
            balls_spawned: false,
            clip: Clip { id: 4, t: 0.0 },
            speed: 1.0,
            gust: None,
            charge: 0.0,
            fade: 0,
            beams: [Beam::default(); 2],
            balls: [None; 5],
            shock: false,
            current: 0,
            next: None,
            previous: 0,
        }
    }

    /// The lightning's phase for drawing: Some(0) the wind-up, Some(1) the fire phase.
    pub fn lightning_phase(&self) -> Option<u8> {
        (self.state == State::Lightning && self.sub < 2).then_some(self.sub)
    }

    /// SPEC-0010 Vanishing: neither the boss nor the companion is drawn from the tick the gust
    /// clip ends through the travel (sub-step 2 of air-flow and of each rapid cycle).
    pub fn hidden(&self) -> bool {
        matches!(self.state, State::AirFlow | State::Rapid) && self.sub == 2
    }

    fn play(&mut self, id: u32) {
        if self.clip.id != id {
            self.clip = Clip { id, t: 0.0 };
        }
    }

    /// Advance the stand-in clip by the tick's animation speed (the host calls this after
    /// `tick`).
    pub fn advance_clip(&mut self) {
        self.clip.t += self.speed;
    }

    fn s(&self) -> usize {
        (self.stage.clamp(1, 3) - 1) as usize
    }

    fn cooldown(&self, (lo, hi): (f32, f32), rng: &mut Rng) -> i32 {
        (60.0 * rng.frange(lo, hi)) as i32
    }

    fn idle(&mut self) {
        self.state = State::Idle;
        self.sub = 0;
        self.gust = None;
        self.shock = false;
    }

    fn pick_next(&mut self, rng: &mut Rng) {
        self.next = Some((self.current + rng.range(1, 3) as usize + 1) % 6);
    }

    fn start(&mut self, s: State, w: &Words, rng: &mut Rng) {
        self.state = s;
        self.sub = 0;
        self.latch = false;
        self.balls_spawned = false;
        if matches!(s, State::AirFlow | State::Rapid) {
            self.pick_next(rng);
            self.charge = 0.0;
            if s == State::Rapid {
                self.rapid = rng.range(w.rapid.0, w.rapid.1);
            }
        }
    }

    /// The reply roll (lightning end, special reply, reaction).
    fn reply_roll(&mut self, w: &Words, rng: &mut Rng) {
        if rng.range(0, 100) >= 50 {
            self.idle();
        } else if rng.range(0, 100) < 50 {
            self.start(State::AirFlow, w, rng);
        } else {
            self.start(State::Rapid, w, rng);
        }
    }

    fn trigger(&self, v: &View) -> bool {
        let u = flat(sub(v.hero, v.position));
        match v.variant {
            0 => v.in_attack && hdist(v.position, v.hero) < 10.0 && dot(v.forward, u) > 0.5,
            1 => v.companion.is_some_and(|c| v.shots.iter().any(|&(s, vel)| segment_distance(s, add(s, scale(vel, 10.0)), c, c) <= 5.0)),
            _ => false,
        }
    }

    fn hero_body(v: &View) -> Cylinder {
        Cylinder { center: add(v.hero, [0.0, 0.5 * v.hero_height, 0.0]), axis: [0.0, 1.0, 0.0], radius: v.hero_radius, length: v.hero_height }
    }

    fn bone6(v: &View) -> Vec3 {
        add(add(v.position, [0.0, BONE6.0, 0.0]), scale(v.forward, BONE6.1))
    }

    /// One controller tick (SPEC-0010 Each tick, in order).
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act { forward: v.forward, ..Default::default() };
        // 1. Timers.
        for t in [&mut self.hesitation, &mut self.air_cd, &mut self.lightning_cd, &mut self.bounce_cd, &mut self.sub_timer] {
            if *t > 0 {
                *t -= 1;
            }
        }
        // 2. Fade and speed.
        self.fade = 0;
        self.speed = 1.0;
        // 3. Stage.
        let pct = (100.0 * v.hp as f32 / v.max_hp.max(1) as f32) as i32;
        if pct < 33 {
            self.stage = 3;
        } else if pct < 66 {
            self.stage = 2;
        }
        // 4. Reaction check.
        let trig = !matches!(self.state, State::AirFlow | State::Rapid) && self.trigger(v);
        let mut skip = false;
        if trig && self.state != State::Reaction {
            self.state = State::Reaction;
            self.hold = 0;
            self.shock = false;
            for b in &mut self.beams {
                b.stop();
            }
            skip = true;
        }
        // 5. Decide.
        if self.state == State::Idle {
            self.decide(w, rng);
            if self.state == State::Idle && hdist(v.position, v.hero) < w.shock_range {
                self.state = State::Shock;
                self.sub = 0;
                self.latch = false;
            }
        }
        // 6. The state.
        if !skip || self.state == State::Reaction {
            self.run(v, w, rng, trig, &mut a);
        }
        // 7. Arena movement, the balls.
        self.arena(v, &mut a);
        self.balls_step(v, w, rng, &mut a);
        a
    }

    fn decide(&mut self, w: &Words, rng: &mut Rng) {
        if self.hesitation > 0 {
            return;
        }
        let r = rng.range(0, 100);
        let [f, aw, rr, l, b] = w.weights[self.s()];
        let (ca, cl, cb) = (self.air_cd > 0, self.lightning_cd > 0, self.bounce_cd > 0);
        let choice = 'c: {
            if r < f {
                break 'c Some(State::Fidget);
            }
            let base = if r < f + aw {
                if !ca {
                    break 'c Some(State::AirFlow);
                }
                f
            } else {
                f + aw
            };
            if !ca && r < base + rr {
                break 'c Some(State::Rapid);
            }
            let mut u = base + rr + l;
            if r < u {
                if !cl {
                    break 'c Some(State::Lightning);
                }
                u -= l;
            }
            if r < u + b && !cb {
                break 'c Some(State::Bouncing);
            }
            None
        };
        // The hesitation is redrawn on every decision; stage 3 uses the stage-2 pair.
        let (lo, hi) = w.hesitation[if self.stage == 1 { 0 } else { 1 }];
        self.hesitation = rng.range((60.0 * lo) as i32, (60.0 * hi) as i32);
        if let Some(s) = choice {
            self.start(s, w, rng);
            if s == State::Lightning {
                match self.stage {
                    1 => self.sub_timer = (60.0 * w.windup[0]) as i32,
                    2 => self.sub_timer = (60.0 * w.windup[1]) as i32,
                    _ => {}
                }
            }
        }
    }

    fn run(&mut self, v: &View, w: &Words, rng: &mut Rng, trig: bool, a: &mut Act) {
        let hd = hdist(v.position, v.hero);
        match self.state {
            State::Idle => self.play(4),
            State::Fidget => {
                self.play(5);
                if self.clip.finished() {
                    self.idle();
                }
            }
            State::Shock => match self.sub {
                0 => {
                    self.play(11);
                    self.sub_timer = 60;
                    self.sub = 1;
                }
                1 => {
                    if self.clip.id != 11 || self.clip.finished() {
                        self.play(12);
                    }
                    if self.sub_timer > 0 {
                        self.shock = true;
                        if hd <= w.shock_radius && !self.latch {
                            a.hits.push(Hit::Damage(w.shock_damage));
                            self.latch = true;
                        }
                    } else {
                        self.shock = false;
                        self.play(13);
                        self.sub = 2;
                    }
                }
                _ => {
                    if self.clip.id == 13 && self.clip.finished() {
                        self.idle();
                    }
                }
            },
            State::AirFlow | State::Rapid => self.gust_step(v, w, rng, a),
            State::Lightning => self.lightning(v, w, rng, a),
            State::Bouncing => {
                self.play(14);
                if !self.balls_spawned && self.clip.progress() >= BALL_EVENT {
                    self.balls_spawned = true;
                    for _ in 0..w.balls[self.s()] {
                        self.spawn_ball(v, w, rng);
                    }
                }
                if self.clip.finished() {
                    self.idle();
                    self.bounce_cd = self.cooldown(w.bounce_timer[self.s()], rng);
                }
            }
            State::Reply => {
                if self.clip.id != self.reply_anim {
                    self.play(self.reply_anim);
                    a.hit_fx = true;
                } else if self.clip.finished() {
                    a.self_damage = w.reply_damage;
                    self.reply_roll(w, rng);
                }
            }
            State::Reaction => {
                if trig {
                    self.clip = Clip { id: 23, t: if self.clip.id == 23 { self.clip.t } else { 0.0 } };
                    self.hold = (60.0 * w.hold) as i32;
                } else if self.hold > 0 {
                    self.hold -= 1;
                } else {
                    self.reply_roll(w, rng);
                }
            }
        }
    }

    fn gust_test(&mut self, v: &View, w: &Words, a: &mut Act) {
        let Some((o, e)) = self.gust else { return };
        if self.latch {
            return;
        }
        let l = len(sub(e, o));
        if l <= 0.0 {
            return;
        }
        let g = Cylinder { center: scale(add(o, e), 0.5), axis: unit(sub(e, o)), radius: w.gust_radius, length: l };
        if overlap(&Self::hero_body(v), &g) {
            self.latch = true;
            a.hits.push(Hit::Damage(w.gust_damage));
            let fh = flat(v.hero_forward);
            a.knock = Some(scale(unit(add(scale(fh, -9.0), [0.0, 12.0, 0.0])), w.knock_speed));
        }
    }

    fn gust_step(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        let b6 = Self::bone6(v);
        let gs = w.gust_speed[self.s()];
        match self.sub {
            0 => {
                self.play(6);
                if self.clip.finished() {
                    self.charge = 0.0;
                    self.sub = 1;
                }
            }
            1 => {
                self.play(7);
                self.speed = gs;
                self.charge = (self.charge + 0.25).min(10.0);
                let o = b6;
                self.gust = Some((o, add(o, scale(v.forward, self.charge))));
                self.fade = ((1.0 - self.clip.progress()) * 128.0) as i32;
                self.gust_test(v, w, a);
                if self.clip.finished() {
                    if let Some(n) = self.next.take() {
                        self.previous = self.current;
                        self.current = n;
                    }
                    self.sub = 2;
                }
            }
            2 => {
                self.play(22);
                self.speed = 0.0;
                if let Some((o, e)) = self.gust {
                    let d = sub(b6, e);
                    if len(d) < w.gust_travel {
                        self.sub = 3;
                    } else {
                        let m = scale(unit(d), w.gust_travel);
                        self.gust = Some((add(o, m), add(e, m)));
                    }
                }
                self.gust_test(v, w, a);
            }
            3 => {
                self.play(22);
                self.speed = gs;
                self.fade = (self.clip.progress() * 128.0) as i32;
                self.charge = (self.charge - 0.25).max(0.0);
                let e = b6;
                self.gust = Some((add(e, scale(v.forward, self.charge)), e));
                self.gust_test(v, w, a);
                if self.clip.finished() {
                    self.sub = 4;
                }
            }
            _ => {
                self.gust = None;
                if self.state == State::Rapid && self.rapid > 0 {
                    self.rapid -= 1;
                    self.pick_next(rng);
                    self.clip = Clip { id: 7, t: 0.0 };
                    self.sub = 1;
                } else {
                    self.idle();
                    self.air_cd = self.cooldown(w.air_timer[self.s()], rng);
                }
            }
        }
    }

    fn lightning(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        match self.sub {
            0 => {
                self.play(8);
                if self.sub_timer == 0 {
                    if v.companion.is_none() {
                        // No companion: the fire phase is skipped (Edge cases).
                        self.sub = 2;
                    } else {
                        self.beams[0].reset();
                        self.fire_total = (60.0 * w.fire_time[self.s()]) as i32;
                        self.sub_timer = self.fire_total;
                        self.sub = 1;
                    }
                }
            }
            1 => {
                if self.sub_timer == 0 {
                    for b in &mut self.beams {
                        b.stop();
                    }
                    self.sub = 2;
                    self.play(10);
                    return;
                }
                self.fire(v, w, rng, a);
            }
            _ => {
                self.play(10);
                if self.clip.finished() {
                    self.lightning_cd = self.cooldown(w.lightning_timer[self.s()], rng);
                    self.reply_roll(w, rng);
                }
            }
        }
    }

    fn fire(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        let Some(s) = v.companion else { return };
        let remaining = self.sub_timer;
        if remaining % 60 == 0 {
            self.latch = false;
        }
        let p = v.hero;
        let fh = flat(v.hero_forward);
        let u = flat(sub(v.position, p));
        let dd = hdist(v.position, p);
        let mut aim = add(p, [0.0, 1.5, 0.0]);
        if v.guard && dot(u, fh) > 0.75 {
            aim = add(aim, scale(u, w.guard_shift));
        }
        self.play(8);
        self.beams[0].set(s, aim, w.growth);
        let l = self.beams[0].len;
        let d = unit(sub(aim, s));
        // (a) The wall bounce.
        if let Some((frac, n)) = (v.wall)(s, add(s, scale(d, l))) {
            let hit = add(s, scale(d, l * frac));
            self.beams[0].set(s, hit, w.growth);
            if !self.beams[1].active {
                self.beams[1].reset();
            } else {
                let vv = sub(v.hero, v.position);
                let r = sub(vv, scale(n, 2.0 * dot(vv, n)));
                self.beams[1].set(hit, add(hit, r), w.growth);
            }
            return;
        }
        if self.latch || self.fire_total - remaining < 6 {
            return;
        }
        let beam = Cylinder { center: add(s, scale(d, 0.5 * l)), axis: d, radius: 1.0, length: l };
        let over = overlap(&beam, &Self::hero_body(v));
        let mut blocked = false;
        if (over || self.beams[0].extended()) && v.guard && dot(d, fh) < -0.75 {
            if v.fog {
                // (b) The reflect candidate.
                let k = add(s, scale(d, l));
                if !self.beams[1].active {
                    self.beams[1].reset();
                    return;
                }
                let r = add(add(p, scale(fh, dd + 5.0)), [0.0, 4.0, 0.0]);
                self.beams[1].set(k, r, w.growth);
                if dot(unit(sub(r, k)), u) > 0.99 {
                    self.state = State::Reply;
                    self.reply_anim = if rng.range(0, 1) == 0 { 15 } else { 16 };
                    for b in &mut self.beams {
                        b.stop();
                    }
                }
                return;
            }
            a.hits.push(Hit::Blocked);
            self.latch = true;
            blocked = true;
        }
        self.beams[1].stop();
        if over && !blocked {
            a.hits.push(Hit::Damage(w.bolt_damage as i32));
            self.latch = true;
        }
    }

    fn arena(&mut self, v: &View, a: &mut Act) {
        let Some(pts) = v.points else { return };
        let (p, pf) = pts[self.current];
        let y = (v.ground)(p[0], p[2], p[1] + 8.0);
        let at = [p[0], y, p[2]];
        a.position = Some(at);
        let toward = |t: Vec3| flat(sub(t, at));
        let (dir, wgt) = match (self.state, self.sub) {
            (State::AirFlow, 0) => (self.next.map_or(flat(pf), |n| toward(pts[n].0)), 0.1),
            (State::AirFlow, 1) => (self.next.map_or(flat(pf), |n| toward(pts[n].0)), 1.0),
            (State::AirFlow, 2) => (toward(pts[self.previous].0), 1.0),
            _ => (flat(pf), 0.1),
        };
        let f = flat(add(scale(v.forward, 1.0 - wgt), scale(dir, wgt)));
        a.forward = if f == [0.0; 3] { v.forward } else { f };
    }

    fn spawn_ball(&mut self, v: &View, w: &Words, rng: &mut Rng) {
        let Some(slot) = self.balls.iter_mut().find(|b| b.is_none()) else { return };
        let f = flat(v.forward);
        let p = flat([f[2], 0.0, -f[0]]);
        let r = rng.frange(-0.5, 0.5);
        let at = add(v.position, scale(f, 2.0));
        *slot = Some(Ball { position: at, prev: at, dir: flat(add(scale(f, 1.0 - r), scale(p, r))), vy: w.ball_vy, travel: 0.0 });
    }

    fn balls_step(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        let speed = w.ball_speed[self.s()] / 60.0;
        for slot in &mut self.balls {
            let Some(b) = slot.as_mut() else { continue };
            b.position = add(b.position, scale(b.dir, speed));
            b.position[1] += b.vy;
            b.vy += w.ball_gravity;
            if b.vy < 0.0 {
                let g = (v.ground)(b.position[0], b.position[2], b.position[1] + 8.0);
                if b.position[1] <= g - 1.0 {
                    let r = rng.frange(-0.9, 0.9);
                    let perp = flat([b.dir[2], 0.0, -b.dir[0]]);
                    b.dir = flat(add(scale(b.dir, 1.0 - r.abs()), scale(perp, r)));
                    b.vy = w.ball_vy;
                }
                b.travel += hdist(b.prev, b.position);
                b.prev = b.position;
                if b.travel > w.ball_range {
                    a.bursts.push(b.position);
                    *slot = None;
                    continue;
                }
            }
            if crate::projectile::sphere_meets_body(b.position, w.ball_radius, v.hero, v.hero_radius, v.hero_height) {
                a.bursts.push(b.position);
                a.hits.push(Hit::Damage(w.ball_damage as i32));
                *slot = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        let mut b = vec![0u32; WORDS];
        let mut put = |a: u32, w: u32| b[((a - BASE) / 4) as usize] = w;
        let f = |x: f32| x.to_bits();
        for (a, x) in [(0x2D_A4D0, 1.0), (0x2D_A4D4, 2.0), (0x2D_A4D8, 0.5), (0x2D_A4DC, 1.0)] {
            put(a, f(x));
        }
        for (k, ws) in [[20, 10, 0], [40, 30, 25], [0, 5, 15], [40, 35, 25], [0, 20, 35]].iter().enumerate() {
            for (s, &x) in ws.iter().enumerate() {
                put(0x2D_A4E8 + 12 * k as u32 + 4 * s as u32, x as u32);
            }
        }
        for base in [0x2D_A524u32, 0x2D_A53C, 0x2D_A554] {
            for (k, x) in [5.0, 10.0, 2.0, 5.0, 1.0, 2.0].iter().enumerate() {
                put(base + 4 * k as u32, f(*x));
            }
        }
        for (a, x) in [(0x2D_A56C, 15.0), (0x2D_A580, 1.25), (0x2D_A584, 2.0), (0x2D_A588, 3.0), (0x2D_A5A0, 1.0), (0x2D_A5A4, 0.5), (0x2D_A5A8, 3.0), (0x2D_A5AC, 2.0), (0x2D_A5B0, 1.0), (0x2D_A5B4, 10.0), (0x2D_A5B8, 8.0), (0x2D_A5C0, 50.0), (0x2D_A5D0, 10.0), (0x2D_A5D4, 10.0), (0x2D_A5D8, 10.0), (0x2D_A5DC, 50.0), (0x2D_A5E0, 7.0), (0x2D_A5E8, 0.5), (0x2D_A5EC, 6.0), (0x2D_A5F4, 500.0), (0x2D_A5FC, 1.65), (0x2D_A600, 1.0), (0x2D_A604, 1.5), (0x2D_A60C, -0.1), (0x2D_A610, 2.0)] {
            put(a, f(x));
        }
        for (a, x) in [(0x2D_A590, 3), (0x2D_A594, 5), (0x2D_A598, 20), (0x2D_A5BC, 10), (0x2D_A5C4, 2), (0x2D_A5C8, 3), (0x2D_A5CC, 4), (0x2D_A608, 20)] {
            put(a, x);
        }
        Words::read(&b).unwrap()
    }

    fn view<'a>(hero: Vec3, ground: &'a dyn Fn(f32, f32, f32) -> f32, wall: &'a dyn Fn(Vec3, Vec3) -> Option<(f32, Vec3)>) -> View<'a> {
        View {
            position: [0.0; 3],
            forward: [0.0, 0.0, 1.0],
            hp: 240,
            max_hp: 240,
            hero,
            hero_forward: [0.0, 0.0, -1.0],
            hero_radius: 1.0,
            hero_height: 3.5,
            variant: 0,
            in_attack: false,
            guard: false,
            fog: false,
            shots: &[],
            companion: Some([0.0, 6.0, 0.0]),
            points: None,
            ground,
            wall,
        }
    }

    #[test]
    fn words_and_stage_rule() {
        let w = words();
        assert_eq!(w.weights[1], [10, 30, 5, 35, 20]);
        assert_eq!(w.air_timer[2], (1.0, 2.0));
        let mut rng = Rng::seeded(1);
        let mut c = Cloud::new(&w, &mut rng);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let mut v = view([0.0, 0.0, 30.0], &g, &nw);
        v.hp = 120;
        c.tick(&v, &w, &mut rng);
        assert_eq!(c.stage, 2);
        v.hp = 200;
        c.tick(&v, &w, &mut rng);
        assert_eq!(c.stage, 2, "never back to stage 1");
        c.stage = 3;
        c.tick(&v, &w, &mut rng);
        assert_eq!(c.stage, 3, "66 % or above stays stage 3");
        v.hp = 120;
        c.tick(&v, &w, &mut rng);
        assert_eq!(c.stage, 2);
        v.hp = 70;
        c.tick(&v, &w, &mut rng);
        assert_eq!(c.stage, 3);
    }

    #[test]
    fn decision_bands() {
        let w = words();
        let mut rng = Rng::seeded(4);
        // Stage 2, air-flow and lightning cooling: count the outcomes over many rolls.
        let mut seen = std::collections::HashMap::new();
        for _ in 0..3000 {
            let mut c = Cloud::new(&w, &mut rng);
            (c.stage, c.hesitation, c.air_cd, c.lightning_cd) = (2, 0, 100, 100);
            c.decide(&w, &mut rng);
            *seen.entry(c.state).or_insert(0) += 1;
        }
        assert!(!seen.contains_key(&State::Lightning) && !seen.contains_key(&State::AirFlow) && !seen.contains_key(&State::Rapid));
        assert!(seen[&State::Bouncing] > seen[&State::Fidget]);
    }

    #[test]
    fn shock_hits_once_inside_eight() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let mut c = Cloud::new(&w, &mut rng);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let v = view([0.0, 0.0, 7.0], &g, &nw);
        let mut damage = 0;
        for _ in 0..120 {
            let a = c.tick(&v, &w, &mut rng);
            c.advance_clip();
            damage += a.hits.iter().map(|h| if let Hit::Damage(d) = h { *d } else { 0 }).sum::<i32>();
            if c.state == State::Shock && c.sub == 2 {
                break;
            }
        }
        assert_eq!(damage, 10);
    }

    #[test]
    fn lightning_hits_with_the_latch_clock() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let mut c = Cloud::new(&w, &mut rng);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let v = view([0.0, 0.0, 20.0], &g, &nw);
        c.start(State::Lightning, &w, &mut rng);
        c.sub_timer = 60;
        c.hesitation = 10_000;
        let mut hits = 0;
        for _ in 0..300 {
            let a = c.tick(&v, &w, &mut rng);
            c.advance_clip();
            hits += a.hits.iter().filter(|h| matches!(h, Hit::Damage(10))).count();
            if c.state != State::Lightning {
                break;
            }
        }
        assert_eq!(hits, 3, "stage 1: clears at 120 and 60");
    }

    #[test]
    fn guarded_bolt_is_blocked_and_fog_reflects() {
        let w = words();
        let mut rng = Rng::seeded(5);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let mut v = view([0.0, 0.0, 20.0], &g, &nw);
        v.companion = Some([0.0, 1.5, 0.0]);
        v.guard = true;
        let mut c = Cloud::new(&w, &mut rng);
        c.start(State::Lightning, &w, &mut rng);
        c.hesitation = 10_000;
        let mut blocked = 0;
        for _ in 0..200 {
            let a = c.tick(&v, &w, &mut rng);
            c.advance_clip();
            assert!(!a.hits.iter().any(|h| matches!(h, Hit::Damage(_))));
            blocked += a.hits.iter().filter(|h| matches!(h, Hit::Blocked)).count();
        }
        assert!(blocked >= 1);
        // With the Fog Mote, from 20 units: the reflection sends it to the special reply.
        v.fog = true;
        let mut c = Cloud::new(&w, &mut rng);
        c.start(State::Lightning, &w, &mut rng);
        c.hesitation = 10_000;
        let mut replied = false;
        for _ in 0..200 {
            c.tick(&v, &w, &mut rng);
            c.advance_clip();
            replied |= c.state == State::Reply;
        }
        assert!(replied);
    }

    #[test]
    fn gust_knocks_back_once() {
        let w = words();
        let mut rng = Rng::seeded(6);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let mut c = Cloud::new(&w, &mut rng);
        let mut v = view([0.0, 0.0, 8.0], &g, &nw);
        v.hero_forward = [0.0, 0.0, 1.0];
        c.start(State::AirFlow, &w, &mut rng);
        c.hesitation = 10_000;
        let mut knocks = Vec::new();
        for _ in 0..400 {
            let a = c.tick(&v, &w, &mut rng);
            c.advance_clip();
            knocks.extend(a.knock);
            if c.state == State::Idle {
                break;
            }
        }
        assert_eq!(knocks.len(), 1);
        let k = knocks[0];
        assert!((k[1] - 5.6).abs() < 1e-3 && (k[2] + 4.2).abs() < 1e-3);
    }

    #[test]
    fn balls_bounce_and_expire() {
        let w = words();
        let mut rng = Rng::seeded(7);
        let g = |_: f32, _: f32, _: f32| 0.0;
        let nw = |_: Vec3, _: Vec3| None;
        let mut c = Cloud::new(&w, &mut rng);
        let v = view([500.0, 0.0, 500.0], &g, &nw);
        c.spawn_ball(&v, &w, &mut rng);
        let mut a = Act::default();
        let mut ticks = 0;
        while c.balls[0].is_some() && ticks < 1000 {
            c.balls_step(&v, &w, &mut rng, &mut a);
            ticks += 1;
        }
        assert!((290..=330).contains(&ticks), "{ticks}");
        assert_eq!(a.bursts.len(), 1);
    }
}
