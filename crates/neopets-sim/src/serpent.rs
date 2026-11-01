//! The Plague Serpent (SPEC-0016; behaviour class 2): the health phases with the two-pass phase
//! change (the facing at 0.1 to each stored anchor's centre, animation 21, the level-state flag
//! of the slot at its event), the eight anchors (the nearest by its centre, the relocation and
//! the arrival at point A or C on the ground), the engagement (the bite above 115.0 within 20.0
//! on an unsuppressed anchor; else the belch on its cooldown), the poison belch, and the bite's
//! sub-states (decide, mesmerize with the stun from tick 52, the windup with the block count, the
//! spit, the lunge with its weapon-3 hit or block, the stun with its single strike, and the
//! recoveries). The tail slap and the wait state are never entered and are not built.
//!
//! The host measures what needs the world (the anchor points, the ground, the lunge's weapon
//! test, the strike) and applies the requests. Every tuning value is read from the boot data at
//! the addresses of SPEC-0016.

use crate::projectile::Setup;
use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning block: 40 words from 0x2DAFF8 (the 38 tunables and the two internal constants).
pub const BASE: u32 = 0x002D_AFF8;
pub const WORDS: usize = 40;

/// The anchor slots' level object ids A, B and C (SPEC-0016 Arena anchors).
pub const ANCHORS: [[usize; 3]; 8] = [[106, 105, 104], [103, 102, 101], [124, 123, 122], [121, 120, 119], [118, 117, 116], [115, 114, 113], [112, 111, 110], [109, 108, 107]];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub mesmerize: [i32; 3],
    pub drawback: [(f32, f32); 3],
    pub block_time: f32,
    pub lunge_damage: i32,
    pub stun: f32,
    pub poison_time: f32,
    pub speed: i32,
    pub radius: f32,
    pub life: f32,
    pub belch: (f32, f32),
    pub mesmerize_start: f32,
    pub mesmerize_delay: i32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        if b.len() < WORDS {
            return None;
        }
        let w = |at: u32| b[((at - BASE) / 4) as usize];
        let f = |at: u32| f32::from_bits(w(at));
        let i = |at: u32| w(at) as i32;
        Some(Self {
            mesmerize: [i(0x002D_B038), i(0x002D_B03C), i(0x002D_B040)],
            drawback: [(f(0x002D_B044), f(0x002D_B048)), (f(0x002D_B04C), f(0x002D_B050)), (f(0x002D_B054), f(0x002D_B058))],
            block_time: f(0x002D_B05C),
            lunge_damage: i(0x002D_B060),
            stun: f(0x002D_B064),
            poison_time: f(0x002D_B070),
            speed: i(0x002D_B074),
            radius: f(0x002D_B078),
            life: f(0x002D_B07C),
            belch: (f(0x002D_B088), f(0x002D_B08C)),
            mesmerize_start: f(0x002D_B090),
            mesmerize_delay: i(0x002D_B094),
        })
    }

    /// The mesmerize chance and the post-mesmerize drawback range of a phase (phases 1, 4 and
    /// above: 0 and the first range).
    fn by_phase(&self, phase: i32) -> (i32, (f32, f32)) {
        match phase {
            2 => (self.mesmerize[1], self.drawback[1]),
            3 => (self.mesmerize[2], self.drawback[2]),
            _ => (self.mesmerize[0], self.drawback[0]),
        }
    }
}

/// The phase of a health fraction (SPEC-0016 Behavior 2).
pub fn band(f: f32) -> i32 {
    if f > 0.75 {
        1
    } else if f > 0.5 {
        2
    } else if f > 0.25 {
        3
    } else {
        4
    }
}

/// The bite's sub-states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bite {
    Decide,
    Mesmerize { ticks: i32 },
    Windup { drawback: f32, blocks: i32 },
    Spit { fired: bool },
    Lunge { hit: bool },
    Landed,
    Blocked,
    Stun { left: i32 },
    Struck,
    Recover,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Idle,
    Move,
    Belch { fired: bool },
    Relocate,
    Arrive,
    Bite(Bite),
    /// The phase change: the pass (0 for S1, 1 for S2) and its step (0 turning, 1 the first
    /// tick of animation 21, 2 playing it).
    Phase { pass: u8, step: u8 },
}

pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero's action code, blocking (guard) and stunned flags.
    pub hero_code: u8,
    pub hero_blocking: bool,
    pub hero_stunned: bool,
    pub hp: i32,
    pub max_hp: i32,
    /// The anchor points A, B and C by slot.
    pub anchors: [[Option<Vec3>; 3]; 8],
    /// The ground under (x, z) probed from a height.
    pub ground: &'a dyn Fn(f32, f32, f32) -> Option<f32>,
    /// The lunge's weapon-3 test touches the hero this tick.
    pub lunge_contact: bool,
    /// A strike in the stun: the hero's damage for it, and whether it was a shot.
    pub strike: Option<(i32, bool)>,
    pub mouth: Vec3,
    pub clip: (u32, bool),
    pub frame_bits: u32,
    pub level: &'a dyn Fn(u16) -> bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// The clip's playback multiplier (the windup's drawback).
    pub rate: f32,
    /// Turn toward a direction at a weight.
    pub face: Option<(Vec3, f32)>,
    /// Placed at a point (the arrival).
    pub place: Option<Vec3>,
    /// The lunge's raw damage (with the hero's reaction state 0x36).
    pub lunge: Option<i32>,
    /// Statuses for the hero: (status id, seconds).
    pub statuses: Vec<(i32, i32)>,
    pub launch: Option<Setup>,
    pub set_level: Vec<u16>,
    pub hp_loss: i32,
    pub consume_shot: bool,
}

impl Default for Act {
    fn default() -> Self {
        Self { anim: None, rate: 1.0, face: None, place: None, lunge: None, statuses: Vec::new(), launch: None, set_level: Vec::new(), hp_loss: 0, consume_shot: false }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Serpent {
    pub state: State,
    /// The six neck angles (SPEC-0016 6A), stepped by the host in state 0, else 0.
    pub neck: [f32; 6],
    pub phase: i32,
    pub slot: usize,
    flag: u8,
    variant: u8,
    pub stored: [Option<usize>; 2],
    pub belch: i32,
    struck: bool,
}

impl Default for Serpent {
    fn default() -> Self {
        Self { state: State::Idle, neck: [0.0; 6], phase: 1, slot: 0, flag: 0, variant: 0, stored: [None, None], belch: 0, struck: false }
    }
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

/// The hero's in-the-air test: action codes 7 to 11 and 43 to 46.
fn airborne(code: u8) -> bool {
    matches!(code, 7..=11 | 43..=46)
}

impl Serpent {
    /// The stun window that takes the one strike (SPEC-0016 Hero actions: sub 7, not yet struck).
    pub fn strikeable(&self) -> bool {
        matches!(self.state, State::Bite(Bite::Stun { .. })) && !self.struck
    }

    fn free(&self) -> bool {
        matches!(self.state, State::Idle | State::Belch { .. })
    }

    /// The poison cloud: pool kind 9, visual 15, no damage, the poison status for the poison
    /// time within the poison radius, effect 414, launched at the hero at the projectile speed.
    fn cloud(&self, v: &View, w: &Words) -> Setup {
        let d = [v.hero[0] - v.mouth[0], v.hero[1] - v.mouth[1], v.hero[2] - v.mouth[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        Setup {
            kind: 9,
            visual: 15,
            element: 0,
            owner: None,
            start: v.mouth,
            velocity: d.map(|c| c / l * w.speed as f32),
            lifetime: ticks(w.life),
            damage: 0.0,
            radius: 1.0,
            spin: 0.0,
            area_radius: w.radius,
            area_duration: 0,
            key: 0,
            ignore_world: true,
            status: (2, 100, w.poison_time as i32),
            effects: [414, -1, -1, -1, -1, -1],
        }
    }

    /// One tick (SPEC-0016 Behavior 1).
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        if self.belch > 0 {
            self.belch -= 1;
        }
        // The health band (idle or belching only).
        if self.free() {
            let f = if v.max_hp > 0 { v.hp as f32 / v.max_hp as f32 } else { 0.0 };
            let b = band(f);
            if b != 1 && self.phase != b {
                // The phase-change slots from the current anchor: k and its partner.
                let partner = match self.slot {
                    1 => Some(2),
                    2 => Some(1),
                    4 => Some(5),
                    5 => Some(4),
                    6 => Some(7),
                    7 => Some(6),
                    _ => None,
                };
                if let Some(p) = partner {
                    self.stored = [Some(self.slot), Some(p)];
                }
                self.state = State::Phase { pass: if self.stored[0].is_some() { 0 } else { 1 }, step: 0 };
            }
        }
        // The engagement (idle or belching; nothing while biting).
        if self.free() {
            let d = hdist(v.position, v.hero);
            let suppressed = self.slot == 0 || (self.slot == 3 && self.phase != 4);
            if v.hero[1] > 115.0 && d < 20.0 && !suppressed {
                if v.hero_code != 30 {
                    self.state = State::Bite(Bite::Decide);
                }
            } else if self.belch == 0 && !matches!(self.state, State::Belch { .. }) {
                self.state = State::Belch { fired: false };
            }
        }
        // The anchors (idle, belch or arrive).
        if matches!(self.state, State::Idle | State::Belch { .. } | State::Arrive) {
            let mut best = (f32::MAX, self.slot);
            for (k, a) in v.anchors.iter().enumerate() {
                if let Some(b) = a[1] {
                    let d = hdist(v.hero, b);
                    if d < best.0 {
                        best = (d, k);
                    }
                }
            }
            if self.state == State::Arrive {
                let p = v.anchors[self.slot][if self.flag == 0 { 2 } else { 0 }];
                if let Some(p) = p {
                    let y = (v.ground)(p[0], p[2], p[1] + 8.0).unwrap_or(p[1]);
                    a.place = Some([p[0], y, p[2]]);
                }
                self.state = State::Move;
            } else if best.1 != self.slot {
                self.state = State::Relocate;
                self.variant = self.flag;
                self.slot = best.1;
                self.flag = if rng.range(0, 1) == 1 { 0 } else { 1 };
            }
        }
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        match self.state {
            State::Idle => a.anim = Some(5),
            State::Move => {
                a.anim = Some(3);
                let [ap, _, cp] = v.anchors[self.slot];
                if let (Some(ap), Some(cp)) = (ap, cp) {
                    let dir = if self.flag == 0 { [ap[0] - cp[0], 0.0, ap[2] - cp[2]] } else { [cp[0] - ap[0], 0.0, cp[2] - ap[2]] };
                    a.face = Some((dir, 0.25));
                }
                if ended(3) {
                    self.state = State::Idle;
                }
            }
            State::Belch { fired } => {
                a.anim = Some(10);
                if v.clip.0 == 10 && v.frame_bits != 0 && !fired {
                    a.launch = Some(self.cloud(v, w));
                    self.state = State::Belch { fired: true };
                }
                if ended(10) {
                    self.state = State::Idle;
                    self.belch = rng.range(ticks(w.belch.0), ticks(w.belch.1));
                }
            }
            State::Relocate => {
                let n = if self.variant == 0 { 4 } else { 6 };
                a.anim = Some(n);
                if ended(n) {
                    self.state = State::Arrive;
                }
            }
            State::Arrive => {}
            State::Phase { pass, step } => self.phase_tick(v, pass, step, &mut a),
            State::Bite(b) => self.bite(v, w, b, rng, &mut a),
        }
        a
    }

    fn phase_tick(&mut self, v: &View, pass: u8, step: u8, a: &mut Act) {
        let Some(k) = self.stored[pass as usize] else {
            // A pass with no slot: the second pass alone, or (both none) the soft-lock.
            if pass == 0 && self.stored[1].is_some() {
                self.state = State::Phase { pass: 1, step: 0 };
            }
            return;
        };
        let Some(b) = v.anchors[k][1] else { return };
        if step == 0 {
            let to = [b[0] - v.position[0], 0.0, b[2] - v.position[2]];
            let l = to[0].hypot(to[2]).max(1e-6);
            if (to[0] * v.facing[0] + to[2] * v.facing[2]) / l < 0.99 {
                a.anim = Some(5);
                a.face = Some((to, 0.1));
                return;
            }
        }
        a.anim = Some(21);
        if v.clip.0 == 21 && v.frame_bits & 0x20 != 0 && !(v.level)(k as u16) {
            a.set_level.push(k as u16);
        }
        self.state = State::Phase { pass, step: (step + 1).min(2) };
        if step == 2 && v.clip.0 == 21 && v.clip.1 {
            self.stored[pass as usize] = None;
            if pass == 0 {
                self.state = State::Phase { pass: 1, step: 0 };
            } else {
                self.state = State::Idle;
                self.phase += 1;
            }
        }
    }

    fn bite(&mut self, v: &View, w: &Words, b: Bite, rng: &mut Rng, a: &mut Act) {
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let (chance, range) = w.by_phase(self.phase);
        let next = match b {
            Bite::Decide => {
                if !airborne(v.hero_code) && v.hero_code != 30 && rng.range(0, 100) < chance {
                    Bite::Mesmerize { ticks: 0 }
                } else {
                    Bite::Windup { drawback: rng.frange(w.drawback[0].0, w.drawback[0].1), blocks: 0 }
                }
            }
            Bite::Mesmerize { ticks } => {
                a.anim = Some(20);
                let ticks = ticks + 1;
                let from = w.mesmerize_delay + ticks_from(w.mesmerize_start);
                if ticks >= from && !v.hero_stunned {
                    a.statuses.push((0, 5));
                }
                if ended(20) || airborne(v.hero_code) || v.hero_code == 30 {
                    Bite::Windup { drawback: rng.frange(range.0, range.1), blocks: 0 }
                } else {
                    Bite::Mesmerize { ticks }
                }
            }
            Bite::Windup { drawback, blocks } => {
                a.anim = Some(11);
                a.rate = drawback;
                let blocks = if v.hero_blocking { blocks + 1 } else { 0 };
                if blocks > ticks(w.block_time) {
                    Bite::Spit { fired: false }
                } else if ended(11) {
                    Bite::Lunge { hit: false }
                } else {
                    Bite::Windup { drawback, blocks }
                }
            }
            Bite::Spit { fired } => {
                a.anim = Some(12);
                let mut fired = fired;
                if v.clip.0 == 12 && v.frame_bits != 0 && !fired {
                    fired = true;
                    a.launch = Some(self.cloud(v, w));
                }
                if ended(12) {
                    self.state = State::Relocate;
                    return;
                }
                Bite::Spit { fired }
            }
            Bite::Lunge { hit } => {
                a.anim = Some(13);
                let u = {
                    let d = hdist(v.position, v.hero).max(1e-6);
                    [(v.hero[0] - v.position[0]) / d, 0.0, (v.hero[2] - v.position[2]) / d]
                };
                let faces = -(u[0] * v.hero_facing[0] + u[2] * v.hero_facing[2]) > 0.5;
                if v.clip.0 == 13 && v.lunge_contact && !hit {
                    if v.hero_blocking && faces {
                        Bite::Blocked
                    } else {
                        a.lunge = Some(w.lunge_damage);
                        Bite::Landed
                    }
                } else if ended(13) {
                    self.state = State::Relocate;
                    return;
                } else {
                    Bite::Lunge { hit }
                }
            }
            Bite::Landed => {
                a.anim = Some(14);
                if ended(14) {
                    self.state = State::Relocate;
                    return;
                }
                b
            }
            Bite::Blocked => {
                a.anim = Some(22);
                if ended(22) {
                    self.struck = false;
                    Bite::Stun { left: ticks(w.stun) }
                } else {
                    b
                }
            }
            Bite::Stun { left } => {
                a.anim = Some(23);
                if let (Some((d, shot)), false) = (v.strike, self.struck) {
                    self.struck = true;
                    a.hp_loss = d.min(v.hp).max(0);
                    a.consume_shot = shot;
                    Bite::Struck
                } else if left - 1 < 1 {
                    Bite::Recover
                } else {
                    Bite::Stun { left: left - 1 }
                }
            }
            Bite::Struck => {
                a.anim = Some(28);
                if ended(28) {
                    self.state = State::Arrive;
                    return;
                }
                b
            }
            Bite::Recover => {
                a.anim = Some(24);
                if ended(24) {
                    self.state = State::Relocate;
                    return;
                }
                b
            }
        };
        self.state = State::Bite(next);
    }
}

/// The mesmerize effect's start in ticks after its delay (0.7 s: 42).
fn ticks_from(s: f32) -> i32 {
    (s * 60.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        let mut b = vec![0u32; WORDS];
        let mut put = |at: u32, v: u32| b[((at - BASE) / 4) as usize] = v;
        let fl = |x: f32| x.to_bits();
        for (at, v) in [(0x002D_B038, 0), (0x002D_B03C, 40), (0x002D_B040, 60), (0x002D_B060, 20), (0x002D_B074, 2), (0x002D_B094, 10)] {
            put(at, v);
        }
        for (at, v) in [(0x002D_B044, 1.0f32), (0x002D_B048, 1.2), (0x002D_B04C, 0.9), (0x002D_B050, 1.1), (0x002D_B054, 0.8), (0x002D_B058, 1.1), (0x002D_B05C, 0.5), (0x002D_B064, 5.0), (0x002D_B070, 15.0), (0x002D_B078, 6.0), (0x002D_B07C, 5.0), (0x002D_B088, 5.0), (0x002D_B08C, 7.0), (0x002D_B090, 0.7)] {
            put(at, fl(v));
        }
        Words::read(&b).unwrap()
    }

    fn anchors() -> [[Option<Vec3>; 3]; 8] {
        std::array::from_fn(|k| {
            let x = 40.0 * k as f32;
            [Some([x - 5.0, 120.0, 0.0]), Some([x, 120.0, 0.0]), Some([x + 5.0, 120.0, 0.0])]
        })
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, ground: &'a dyn Fn(f32, f32, f32) -> Option<f32>, hero: Vec3) -> View<'a> {
        View {
            position: [80.0, 120.0, 0.0],
            facing: [0.0, 0.0, 1.0],
            hero,
            hero_facing: [0.0, 0.0, -1.0],
            hero_code: 0,
            hero_blocking: false,
            hero_stunned: false,
            hp: 100,
            max_hp: 100,
            anchors: anchors(),
            ground,
            lunge_contact: false,
            strike: None,
            mouth: [80.0, 125.0, 0.0],
            clip: (5, false),
            frame_bits: 0,
            level,
        }
    }

    #[test]
    fn bands_and_engagement() {
        assert_eq!([1.0, 0.75, 0.5, 0.25, 0.2].map(band), [1, 2, 3, 4, 4]);
        let w = words();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        let ground = |_: f32, _: f32, _: f32| Some(118.0);
        // Hero 19 away at height 120 on slot 2 with code 30: no bite and no belch.
        let mut s = Serpent { slot: 2, ..Serpent::default() };
        let mut v = view(&no, &ground, [80.0, 120.0, 19.0]);
        v.hero_code = 30;
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Idle);
        // Code 0: the bite starts (phase 1 never mesmerizes).
        v.hero_code = 0;
        s.tick(&v, &w, &mut rng);
        assert!(matches!(s.state, State::Bite(Bite::Windup { .. })), "{:?}", s.state);
        // Height 115.0 exactly or 20.0 away: not a candidate; the belch starts on cooldown 0.
        for hero in [[80.0, 115.0, 19.0], [80.0, 120.0, 20.0]] {
            let mut s = Serpent { slot: 2, ..Serpent::default() };
            s.tick(&view(&no, &ground, hero), &w, &mut rng);
            assert_eq!(s.state, State::Belch { fired: false });
        }
        // Slot 0 suppresses the bite.
        let mut s = Serpent::default();
        let mut v = view(&no, &ground, [0.0, 120.0, 10.0]);
        v.position = [0.0, 120.0, 0.0];
        s.tick(&v, &w, &mut rng);
        assert!(matches!(s.state, State::Belch { .. }));
    }

    #[test]
    fn relocation_and_arrival() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let no = |_: u16| false;
        let ground = |_: f32, _: f32, _: f32| Some(3.2);
        // At slot 0 idle with the hero nearest slot 5: relocate; the arrival places it on the
        // ground at A (flag 1) or C (flag 0), then the move.
        let mut s = Serpent { belch: 1000, ..Serpent::default() };
        let v = view(&no, &ground, [200.0, 0.0, 30.0]);
        s.tick(&v, &w, &mut rng);
        assert_eq!((s.state, s.slot, s.variant), (State::Relocate, 5, 0));
        let mut v = view(&no, &ground, [200.0, 0.0, 30.0]);
        v.clip = (4, true);
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Arrive);
        let a = s.tick(&view(&no, &ground, [200.0, 0.0, 30.0]), &w, &mut rng);
        let x = if s.flag == 0 { 205.0 } else { 195.0 };
        assert_eq!((a.place, s.state), (Some([x, 3.2, 0.0]), State::Move));
    }

    #[test]
    fn bite_lunge_block_stun_and_strike() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        let ground = |_: f32, _: f32, _: f32| Some(118.0);
        // Phase 2, a mesmerize: from tick 52 the hero is stunned for 5 s.
        let mut s = Serpent { slot: 2, phase: 2, belch: 1000, state: State::Bite(Bite::Mesmerize { ticks: 50 }), ..Serpent::default() };
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.hp = 70;
        v.clip = (20, false);
        assert!(s.tick(&v, &w, &mut rng).statuses.is_empty());
        assert_eq!(s.tick(&v, &w, &mut rng).statuses, vec![(0, 5)]);
        // The windup: 30 blocking ticks stay, 31 spit.
        let mut s = Serpent { slot: 2, belch: 1000, state: State::Bite(Bite::Windup { drawback: 1.1, blocks: 29 }), ..Serpent::default() };
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.hero_blocking = true;
        v.clip = (11, false);
        let a = s.tick(&v, &w, &mut rng);
        assert_eq!((a.rate, s.state), (1.1, State::Bite(Bite::Windup { drawback: 1.1, blocks: 30 })));
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Bite(Bite::Spit { fired: false }));
        // The lunge: a blocking hero facing it (dot 0.6 or more): blocked, then the 300-tick stun.
        let mut s = Serpent { slot: 2, belch: 1000, state: State::Bite(Bite::Lunge { hit: false }), ..Serpent::default() };
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.hero_blocking = true;
        v.lunge_contact = true;
        v.clip = (13, false);
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Bite(Bite::Blocked));
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.clip = (22, true);
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Bite(Bite::Stun { left: 300 }));
        // One strike for 17; a second does nothing.
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.strike = Some((17, false));
        assert_eq!(s.tick(&v, &w, &mut rng).hp_loss, 17);
        assert_eq!(s.state, State::Bite(Bite::Struck));
        // Unblocked: 20 raw.
        let mut s = Serpent { slot: 2, belch: 1000, state: State::Bite(Bite::Lunge { hit: false }), ..Serpent::default() };
        let mut v = view(&no, &ground, [80.0, 120.0, 10.0]);
        v.lunge_contact = true;
        v.clip = (13, false);
        assert_eq!(s.tick(&v, &w, &mut rng).lunge, Some(20));
    }

    #[test]
    fn phase_change_passes() {
        let w = words();
        let mut rng = Rng::seeded(4);
        let ground = |_: f32, _: f32, _: f32| Some(118.0);
        // Anchor 2 at 75 %: the slots 2 and 1; each pass faces, plays 21, sets its flag at 0x20.
        let mut s = Serpent { slot: 2, belch: 1000, ..Serpent::default() };
        let no = |_: u16| false;
        let mut v = view(&no, &ground, [80.0, 0.0, 300.0]);
        v.hp = 75;
        v.position = [80.0, 120.0, -10.0];
        v.facing = [0.0, 0.0, 1.0];
        s.tick(&v, &w, &mut rng);
        assert_eq!((s.state, s.stored), (State::Phase { pass: 0, step: 1 }, [Some(2), Some(1)]));
        v.clip = (21, false);
        v.frame_bits = 0x20;
        assert_eq!(s.tick(&v, &w, &mut rng).set_level, vec![2]);
        v.clip = (21, true);
        v.frame_bits = 0;
        s.tick(&v, &w, &mut rng);
        assert_eq!(s.state, State::Phase { pass: 1, step: 0 });
        // The second pass turns toward slot 1's centre (0.1) before its animation 21.
        let a = s.tick(&v, &w, &mut rng);
        assert_eq!((a.anim, a.face.map(|f| f.1)), (Some(5), Some(0.1)));
        let to = [40.0 - 80.0, 0.0, 10.0f32];
        let l = to[0].hypot(to[2]);
        v.facing = [to[0] / l, 0.0, to[2] / l];
        v.clip = (5, false);
        s.tick(&v, &w, &mut rng);
        v.clip = (21, false);
        s.tick(&v, &w, &mut rng);
        v.clip = (21, true);
        s.tick(&v, &w, &mut rng);
        assert_eq!((s.state, s.phase), (State::Idle, 2));
        // Both slots none: the change stays in state 11.
        let mut s = Serpent { slot: 0, belch: 1000, ..Serpent::default() };
        let mut v = view(&no, &ground, [0.0, 0.0, 300.0]);
        v.hp = 50;
        for _ in 0..5 {
            s.tick(&v, &w, &mut rng);
        }
        assert!(matches!(s.state, State::Phase { pass: 1, .. }));
    }
}
