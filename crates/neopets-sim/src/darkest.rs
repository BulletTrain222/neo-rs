//! The Darkest Faerie (SPEC-0054; behaviour class 37): the routing (game state 364: the main
//! controller; level state 18: loop A; else loop B), the main controller's eleven member codes
//! (idle, burst, the shove, the orb volley, the aimed bolt, the stun and its count, the absorb
//! window, the timeout descent, the fall and the landed wait), its chooser (the level-10 gate,
//! the hold flag, the near shove, the 1-in-11 bolt), its four projectile slots (the elemental
//! orb with the block-to-reflect rule, the aimed bolt's detonation, the reflected orb's stun),
//! and loop A's knockback. She is invulnerable: no hero hit lowers her hit points.
//!
//! The host measures what needs the world (the ground under her, the hero's weapon reach and
//! attack form, the hero shots on her body) and applies the requests. Every tuning value is read
//! from the boot data at the addresses of SPEC-0054.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning words: 14 from 0x2DA618 and 3 from 0x2DAAB0.
pub const BASE: u32 = 0x002D_A618;
pub const WORDS: usize = 14;
pub const BASE_B: u32 = 0x002D_AAB0;
pub const WORDS_B: usize = 3;

/// The summon keys of the aimed bolt and the burst are the row's spawn types (+0x3CC, +0x3CE).
/// The effect ids of the orb kinds 0 to 6: flight, impact.
pub const KIND_EFFECTS: [(i32, i32); 7] = [(2127, 412), (1603, 991), (1605, 993), (1608, 988), (1604, 992), (1606, 989), (1607, 987)];
/// The cast glow of kinds 1 to 6.
pub const GLOWS: [i32; 6] = [1617, 1619, 1622, 1618, 1620, 1621];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub cooldown: (f32, f32),
    pub window: f32,
    pub hold: f32,
    pub dive_speed: f32,
    pub orb_damage: i32,
    pub speed: f32,
    pub stun: f32,
    pub hover: [f32; 3],
    pub descent: f32,
    pub b_half_span: f32,
    pub b_range: f32,
    pub b_speed: f32,
}

impl Words {
    pub fn read(a: &[u32], b: &[u32]) -> Option<Self> {
        if a.len() < WORDS || b.len() < WORDS_B {
            return None;
        }
        let w = |at: u32| a[((at - BASE) / 4) as usize];
        let f = |at: u32| f32::from_bits(w(at));
        Some(Self {
            cooldown: (f(0x002D_A618), f(0x002D_A61C)),
            window: f(0x002D_A628),
            hold: f(0x002D_A62C),
            dive_speed: f(0x002D_A630),
            orb_damage: w(0x002D_A634) as i32,
            speed: f(0x002D_A638),
            stun: f(0x002D_A63C),
            hover: [f(0x002D_A640), f(0x002D_A644), f(0x002D_A648)],
            descent: f(0x002D_A64C),
            b_half_span: f32::from_bits(b[0]),
            b_range: f32::from_bits(b[1]),
            b_speed: f32::from_bits(b[2]),
        })
    }
}

/// The hero's slot-1 attack kind for the block test (SPEC-0054 Hero attack-kind table).
pub fn attack_kind(id: Option<u16>) -> u8 {
    match id {
        Some(160 | 161) => 1,
        Some(162 | 163) => 4,
        Some(164 | 165) => 2,
        Some(166 | 167) => 5,
        Some(168 | 169) => 6,
        Some(170 | 171) => 3,
        Some(172 | 173) => 7,
        _ => 0,
    }
}

/// One projectile slot: type 0 elemental orb, 1 aimed bolt, 2 reflected orb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub ty: u8,
    pub kind: u8,
    pub position: Vec3,
}

/// What she sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    pub hero: Vec3,
    pub hero_facing: Vec3,
    /// The hero's guard query; its slot-1 attack kind.
    pub hero_guard: bool,
    pub hero_kind: u8,
    /// The hero's in-attack test, attack form 2, and its weapon reaching her body.
    pub hero_attacking: bool,
    pub form2: bool,
    pub weapon_reaches: bool,
    /// Her radius and height; the hero's motion radius.
    pub radius: f32,
    pub height: f32,
    pub hero_radius: f32,
    /// The ground under her (the probe 8.0 above).
    pub ground: Option<f32>,
    /// The armed hero shots (point and radius).
    pub shots: &'a [(Vec3, f32)],
    /// The launch point (her mouth).
    pub mouth: Vec3,
    pub clip: (u32, bool),
    pub frame_bits: u32,
    /// Game state 364 (the routing and the gate) and the level states.
    pub progress: bool,
    pub level: &'a dyn Fn(u16) -> bool,
}

/// A push of the hero: the position offset (the push-out), the velocity, and the hero state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push {
    pub offset: Vec3,
    pub velocity: Vec3,
    pub state: u8,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Act {
    pub anim: Option<u32>,
    /// Turn toward the hero at this weight.
    pub face: Option<f32>,
    /// Her new height.
    pub height: Option<f32>,
    pub push: Option<Push>,
    pub hero_damage: Vec<i32>,
    pub levels: Vec<(u16, bool)>,
    /// The hero shots (indices into the view's list) absorbed this tick.
    pub absorbed: Vec<usize>,
    /// Summons by the row's key: true for the burst (+0x3CE), false for the bolt (+0x3CC).
    pub summons: Vec<(bool, Vec3)>,
    /// One-shot impact effects.
    pub impacts: Vec<(i32, Vec3)>,
    /// The cast glow started (kind) or stopped.
    pub glow: Option<Option<u8>>,
    /// Turn toward a point at a weight (loop B's turn toward Queen Fyora).
    pub turn_to: Option<(Vec3, f32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Faerie {
    pub code: u8,
    first: bool,
    step: u8,
    pub timer: i32,
    pub cooldown: i32,
    pub kind: u8,
    pub absorbs: u8,
    pub hold: i32,
    pub stuns: u8,
    hold_flag: bool,
    flinch: bool,
    flinching: bool,
    fired: bool,
    pub slots: [Option<Slot>; 4],
    countdown: i32,
    /// Loop B: the state, the cooldown and the once flag.
    pub b_state: u8,
    b_cool: i32,
    b_flag: bool,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len(a: Vec3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn toward(p: Vec3, t: Vec3, step: f32) -> Vec3 {
    let d = sub(t, p);
    let l = len(d);
    if l <= step || l == 0.0 {
        t
    } else {
        std::array::from_fn(|k| p[k] + d[k] * step / l)
    }
}

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

/// A shove of the hero (SPEC-0054 lunge; loop A's knockback): the velocity zeroed, 12.0 up, the
/// push-out by 5 times the shortfall inside the two radii plus 0.1, 9 along the direction to the
/// hero, then the velocity scaled to the speed; hero state 0x30.
pub fn shove(from: Vec3, hero: Vec3, radii: f32, speed: f32) -> Push {
    let h = (hero[0] - from[0]).hypot(hero[2] - from[2]);
    let d = if h > 1e-6 { [(hero[0] - from[0]) / h, 0.0, (hero[2] - from[2]) / h] } else { [1.0, 0.0, 0.0] };
    let short = radii + 0.1 - h;
    let offset = if short > 0.0 { d.map(|c| c * 5.0 * short) } else { [0.0; 3] };
    let v = [9.0 * d[0], 12.0, 9.0 * d[2]];
    let l = len(v);
    Push { offset, velocity: v.map(|c| c / l * speed), state: 0x30 }
}

impl Default for Faerie {
    fn default() -> Self {
        Self { code: 0, first: true, step: 0, timer: 0, cooldown: 0, kind: 1, absorbs: 0, hold: 0, stuns: 0, hold_flag: false, flinch: false, flinching: false, fired: false, slots: [None; 4], countdown: 0, b_state: 0, b_cool: 0, b_flag: false }
    }
}

impl Faerie {
    fn enter(&mut self, code: u8) {
        if self.code != code {
            self.code = code;
            self.step = 0;
            self.fired = false;
        }
        self.first = true;
    }

    fn launch(&mut self, ty: u8, kind: u8, at: Vec3) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.is_none()) {
            *s = Some(Slot { ty, kind, position: at });
        }
    }

    /// One tick of the class-37 routine.
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        if v.progress {
            self.main(v, w, rng)
        } else if (v.level)(18) {
            self.loop_a(v, w)
        } else {
            // Loop B (the duel with Queen Fyora) is not built: she holds animation 0.
            Act { anim: Some(0), ..Act::default() }
        }
    }

    /// Loop B (game state 364 and level state 18 clear: the duel with Queen Fyora, SPEC-0029
    /// Behavior 7) once a class-38 partner exists: the chooser (the cooldown; a type-1 orb within
    /// 1.5 of her: the deflect; the hero within the knockback range: the knockback; from states 0
    /// and 2 with no type-0 orb and the cooldown done: the summon cast when the summon test passes
    /// (level state 2 clear, one in four, none of her summons alive), else the cast at Fyora), the
    /// turn toward Fyora with blend 1.0 (toward the hero at 0.25 in state 5), and the states: idle
    /// (6), the deflect (10; the first type-1 orb becomes type 3 with effect 4008), the cast (7; a
    /// type-0 orb at the event; cooldown 30 to 45), the knockback (9; loop A's lanes without the
    /// push-out at the loop B speed) and the summon cast (7; a type-2 orb; cooldown 120 to 240).
    #[allow(clippy::too_many_arguments)]
    pub fn loop_b(&mut self, v: &View, w: &Words, partner: Option<Vec3>, slots: &mut crate::fyora::Slots, summons_alive: bool, rng: &mut Rng) -> Act {
        use crate::fyora;
        let mut a = Act::default();
        let Some(fy) = partner else { return a };
        if self.b_cool > 0 {
            self.b_cool -= 1;
        }
        let d = (v.hero[0] - v.position[0]).hypot(v.hero[2] - v.position[2]);
        if fyora::near(slots, 1, v.position) {
            self.b_state = 2;
            self.b_flag = false;
        } else if d < w.b_range {
            self.b_state = 4;
            self.b_flag = false;
        } else if matches!(self.b_state, 0 | 2) && !fyora::active(slots, 0) && self.b_cool < 1 {
            let summon = !(v.level)(2) && rng.index(4) == 0 && !summons_alive;
            self.b_state = if summon { 5 } else { 3 };
            self.b_flag = false;
        }
        if self.b_state == 5 {
            a.face = Some(0.25);
        } else {
            a.turn_to = Some((fy, 1.0));
        }
        let current = |n: u32| v.clip.0 == n;
        match self.b_state {
            2 => {
                a.anim = Some(10);
                if current(10) && !v.clip.1 && !self.b_flag {
                    self.b_flag = true;
                    if let Some(p) = fyora::deflect(slots, 1, v.facing, rng) {
                        a.impacts.push((4008, p));
                    }
                }
                if current(10) && v.clip.1 && self.b_flag {
                    self.b_state = 0;
                }
            }
            3 | 5 => {
                a.anim = Some(7);
                if current(7) && v.frame_bits != 0 && !self.b_flag {
                    self.b_flag = true;
                    fyora::open(slots, if self.b_state == 3 { 0 } else { 2 }, v.mouth);
                }
                if current(7) && v.clip.1 && self.b_flag {
                    self.b_cool = if self.b_state == 3 { rng.range(30, 45) } else { rng.range(120, 240) };
                    self.b_state = 0;
                }
            }
            4 => {
                a.anim = Some(9);
                if current(9) && !self.b_flag {
                    self.b_flag = true;
                    let mut p = shove(v.position, v.hero, 0.0, w.b_speed);
                    p.offset = [0.0; 3];
                    p.state = 48;
                    a.push = Some(p);
                }
                if current(9) && v.clip.1 {
                    self.b_state = 0;
                }
            }
            _ => a.anim = Some(6),
        }
        a
    }

    fn loop_a(&mut self, v: &View, w: &Words) -> Act {
        let mut a = Act { anim: Some(0), ..Act::default() };
        if self.countdown > 0 {
            self.countdown -= 1;
        }
        let d = (v.hero[0] - v.position[0]).hypot(v.hero[2] - v.position[2]);
        if d < 10.0 && self.countdown < 1 {
            self.countdown = 180;
            let mut p = shove(v.position, v.hero, v.radius + v.hero_radius, 1.5 * w.dive_speed);
            p.state = 48;
            a.push = Some(p);
        }
        a
    }

    fn main(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let d = (v.hero[0] - v.position[0]).hypot(v.hero[2] - v.position[2]);
        self.slots_tick(v, w, &mut a);
        // The gate, then the choice.
        if (v.level)(10) || !v.progress {
            self.enter(1);
        }
        if !self.hold_flag {
            if self.cooldown > 0 {
                self.cooldown -= 1;
            }
            if d < 10.0 && !matches!(self.code, 5 | 6) {
                if self.code != 2 {
                    self.enter(2);
                }
            } else if matches!(self.code, 0 | 7 | 8 | 11..) && self.cooldown == 0 {
                if rng.range(0, 10) == 0 || v.hero_guard {
                    self.enter(4);
                } else {
                    self.kind = rng.range(1, 6) as u8;
                    self.enter(3);
                }
            }
        }
        let ended = |n: u32| v.clip.0 == n && v.clip.1;
        let event = |n: u32| v.clip.0 == n && v.frame_bits != 0;
        let first = std::mem::replace(&mut self.first, false);
        let ground = v.ground.unwrap_or(v.position[1]);
        match self.code {
            0 => {
                a.anim = Some(6);
                a.face = Some(0.25);
            }
            1 => {
                if (v.level)(15) {
                    a.levels.extend([(15, false), (10, false)]);
                    self.stuns = 0;
                    self.hold_flag = false;
                    self.enter(0);
                    a.anim = Some(6);
                    return a;
                }
                match self.step {
                    0 => {
                        a.anim = Some(6);
                        if (v.level)(14) {
                            self.step = 1;
                        }
                    }
                    1 => {
                        a.anim = Some(14);
                        if ended(14) {
                            self.step = 2;
                        }
                    }
                    _ => {
                        a.anim = Some(10);
                        if ended(10) {
                            let p = v.position;
                            a.summons.extend([[4.0, 4.0], [-4.0, 4.0], [4.0, -4.0], [-4.0, -4.0]].map(|[x, z]| (true, [p[0] + x, p[1], p[2] + z])));
                            a.levels.extend([(14, false), (10, false)]);
                            self.enter(7);
                            self.timer = ticks(w.window);
                            self.absorbs = 0;
                            self.hold = 0;
                        }
                    }
                }
            }
            // The shove (R-0069): animation 9 not current: request it and nothing else; current
            // before its gate: the push on the first such tick; at the gate: code 0 and animation
            // 6 in the same tick. A cycle is K + 1 ticks (153 for her 39-key clip).
            2 => {
                if v.clip.0 != 9 {
                    a.anim = Some(9);
                } else if !v.clip.1 {
                    a.anim = Some(9);
                    if !self.fired {
                        self.fired = true;
                        a.push = Some(shove(v.position, v.hero, v.radius + v.hero_radius, w.dive_speed));
                    }
                } else {
                    self.enter(0);
                    a.anim = Some(6);
                }
            }
            3 => {
                a.anim = Some(7);
                a.face = Some(0.25);
                if first {
                    a.glow = Some(Some(self.kind));
                }
                if event(7) && !self.fired {
                    self.fired = true;
                    self.launch(0, self.kind, v.mouth);
                    a.glow = Some(None);
                    self.cooldown = rng.range(ticks(w.cooldown.0), ticks(w.cooldown.1));
                }
                if ended(7) && !first {
                    self.enter(0);
                }
            }
            4 => {
                a.anim = Some(8);
                a.face = Some(0.25);
                if event(8) && !self.fired {
                    self.fired = true;
                    self.launch(1, 0, v.mouth);
                    self.cooldown = rng.range(ticks(w.cooldown.0), ticks(w.cooldown.1));
                }
                if ended(8) && !first {
                    self.enter(0);
                }
            }
            5 => {
                self.timer -= 1;
                if !v.hero_attacking {
                    self.flinch = false;
                }
                if v.hero_attacking && v.weapon_reaches {
                    if v.form2 {
                        self.enter(6);
                        a.anim = Some(13);
                        return a;
                    }
                    if !self.flinch {
                        self.flinch = true;
                        self.flinching = true;
                    }
                }
                if self.flinching && ended(13) {
                    self.flinching = false;
                }
                a.anim = Some(if self.flinching { 13 } else { 14 });
                if self.timer <= 0 {
                    self.flinching = false;
                    self.enter(0);
                }
            }
            6 => {
                a.anim = Some(13);
                if ended(13) && !first {
                    // While the finished animation is held the count keeps rising (message 13).
                    self.stuns = self.stuns.saturating_add(1);
                    if self.stuns < 3 {
                        self.enter(0);
                    } else {
                        a.levels.extend([(12, true), (10, true)]);
                        self.hold_flag = true;
                    }
                }
            }
            7 => {
                a.anim = Some(20);
                self.timer -= 1;
                let target = ground + w.hover[self.absorbs.min(2) as usize];
                a.height = Some(toward([0.0, v.position[1], 0.0], [0.0, target, 0.0], 0.1)[1]);
                for (k, &(p, r)) in v.shots.iter().enumerate() {
                    if crate::projectile::sphere_meets_body(p, r, v.position, v.radius, v.height) {
                        a.absorbed.push(k);
                        self.absorbs += 1;
                        self.hold = ticks(w.hold);
                    }
                }
                if self.hold > 0 {
                    self.hold -= 1;
                    if self.hold == 0 && matches!(self.absorbs, 1 | 2) {
                        self.absorbs -= 1;
                    }
                }
                if self.absorbs >= 3 {
                    self.enter(9);
                } else if self.timer <= 0 {
                    self.enter(8);
                }
            }
            8 | 9 => {
                a.anim = Some(20);
                let step = if self.code == 8 { w.descent } else { 0.1 };
                let y = (v.position[1] - step).max(ground);
                a.height = Some(y);
                if y <= ground {
                    if self.code == 8 {
                        a.levels.extend([(13, true), (10, true)]);
                        self.enter(1);
                    } else {
                        a.levels.push((16, true));
                        self.enter(10);
                    }
                }
            }
            _ => {
                a.anim = Some(20);
                if (v.level)(13) {
                    a.levels.push((10, true));
                    self.enter(1);
                }
            }
        }
        a
    }

    /// The four projectile slots: each moves 0.75 per tick toward its target and acts on
    /// arrival.
    fn slots_tick(&mut self, v: &View, w: &Words, a: &mut Act) {
        let aim = [v.hero[0], v.hero[1] + 1.75, v.hero[2]];
        let body = [v.position[0], v.position[1] + 0.5 * v.height, v.position[2]];
        let mut stun = false;
        for slot in &mut self.slots {
            let Some(s) = slot.as_mut() else { continue };
            let target = if s.ty == 2 { body } else { aim };
            // The gates use the separation before the step, strictly below (SPEC-0029 Behavior 19).
            let gap = len(sub(target, s.position));
            s.position = toward(s.position, target, w.speed);
            let impact = KIND_EFFECTS[s.kind.min(6) as usize].1;
            match s.ty {
                0 if gap < 1.5 => {
                    let to = sub(s.position, v.hero);
                    let h = (to[0] * to[0] + to[2] * to[2]).sqrt().max(1e-6);
                    let faces = (v.hero_facing[0] * to[0] + v.hero_facing[2] * to[2]) / h > 0.5;
                    if faces && (v.hero_kind == s.kind || v.hero_kind == 7) {
                        s.ty = 2;
                    } else {
                        a.hero_damage.push(w.orb_damage);
                        a.impacts.push((impact, s.position));
                        *slot = None;
                    }
                }
                1 if gap < 2.0 => {
                    a.summons.push((false, s.position));
                    a.impacts.push((impact, s.position));
                    *slot = None;
                }
                2 if gap < 1.5 => {
                    a.impacts.push((impact, s.position));
                    *slot = None;
                    stun = true;
                }
                _ => {}
            }
        }
        if stun {
            self.enter(5);
            self.timer = ticks(w.stun);
            self.flinch = false;
            self.flinching = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        let mut a = vec![0u32; WORDS];
        let mut put = |at: u32, v: u32| a[((at - BASE) / 4) as usize] = v;
        for (at, x) in [(0x002D_A618, 1.0f32), (0x002D_A61C, 4.0), (0x002D_A628, 50.0), (0x002D_A62C, 3.0), (0x002D_A630, 10.0), (0x002D_A638, 0.75), (0x002D_A63C, 3.0), (0x002D_A640, 15.0), (0x002D_A644, 10.0), (0x002D_A648, 5.0), (0x002D_A64C, 0.05)] {
            put(at, x.to_bits());
        }
        put(0x002D_A634, 10);
        Words::read(&a, &[2.4f32.to_bits(), 15.0f32.to_bits(), 10.0f32.to_bits()]).unwrap()
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, shots: &'a [(Vec3, f32)], d: f32) -> View<'a> {
        View {
            position: [0.0, 10.0, 0.0],
            facing: [0.0, 0.0, 1.0],
            hero: [0.0, 0.0, d],
            hero_facing: [0.0, 0.0, -1.0],
            hero_guard: false,
            hero_kind: 0,
            hero_attacking: false,
            form2: false,
            weapon_reaches: false,
            radius: 1.175,
            height: 3.5,
            hero_radius: 1.0,
            ground: Some(0.0),
            shots,
            mouth: [0.0, 12.0, 0.5],
            clip: (6, false),
            frame_bits: 0,
            progress: true,
            level,
        }
    }

    #[test]
    fn shove_lanes() {
        // K 10 along +x: (6, 8, 0); inside the radii the hero is pushed out by 5 x the shortfall.
        let p = shove([0.0; 3], [1.0, 0.0, 0.0], 2.175, 10.0);
        assert!((p.velocity[0] - 6.0).abs() < 1e-5 && (p.velocity[1] - 8.0).abs() < 1e-5);
        assert!((p.offset[0] - 5.0 * 1.275).abs() < 1e-4 && p.state == 0x30);
        assert_eq!(shove([0.0; 3], [20.0, 0.0, 0.0], 2.175, 10.0).offset, [0.0; 3]);
    }

    #[test]
    fn chooser_gate_and_shove() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        // Level 10 sends her to the burst; the near hero then turns it into the shove.
        let ten = |f: u16| f == 10;
        let mut f = Faerie::default();
        f.tick(&view(&ten, &[], 30.0), &w, &mut rng);
        assert_eq!(f.code, 1);
        let mut f = Faerie::default();
        f.tick(&view(&ten, &[], 5.0), &w, &mut rng);
        assert_eq!(f.code, 2);
        // A far hero with the cooldown at 0: an orb (kind 1 to 6) or the bolt; a guarding hero
        // always gets the bolt.
        let mut f = Faerie::default();
        f.tick(&view(&no, &[], 30.0), &w, &mut rng);
        assert!(matches!(f.code, 3 | 4) && (1..=6).contains(&f.kind));
        let mut f = Faerie::default();
        let mut v = view(&no, &[], 30.0);
        v.hero_guard = true;
        f.tick(&v, &w, &mut rng);
        assert_eq!(f.code, 4);
        // The shove (SPEC-0054 test vector, R-0069): tick 0 requests animation 9 with no push;
        // tick 1 (9 current) pushes; later ticks do not; the gate tick sets code 0 with
        // animation 6; the next tick chooses the shove again and requests 9.
        let mut f = Faerie::default();
        let a = f.tick(&view(&no, &[], 5.0), &w, &mut rng);
        assert_eq!((f.code, a.anim, a.push.is_some()), (2, Some(9), false));
        let mut v9 = view(&no, &[], 5.0);
        v9.clip = (9, false);
        assert!(f.tick(&v9, &w, &mut rng).push.is_some());
        assert!(f.tick(&v9, &w, &mut rng).push.is_none());
        v9.clip = (9, true);
        let a = f.tick(&v9, &w, &mut rng);
        assert_eq!((f.code, a.anim), (0, Some(6)));
        let a = f.tick(&view(&no, &[], 5.0), &w, &mut rng);
        assert_eq!((f.code, a.anim, a.push.is_some()), (2, Some(9), false));
        // The stun code is not cut short by a near hero.
        let mut f = Faerie::default();
        f.code = 5;
        f.timer = 100;
        f.tick(&view(&no, &[], 5.0), &w, &mut rng);
        assert_eq!(f.code, 5);
    }

    #[test]
    fn orbs_block_reflect_and_stun() {
        let w = words();
        let mut rng = Rng::seeded(2);
        let no = |_: u16| false;
        // An unblocked orb hits once for 10 at 1.5.
        let mut f = Faerie::default();
        f.cooldown = 1000;
        f.slots[0] = Some(Slot { ty: 0, kind: 3, position: [0.0, 1.75, 30.0 - 1.4] });
        let a = f.tick(&view(&no, &[], 30.0), &w, &mut rng);
        assert_eq!((a.hero_damage.clone(), f.slots[0]), (vec![10], None));
        // A hero facing it with the matching kind (or 7) reflects it; its arrival stuns for 180.
        let mut f = Faerie::default();
        f.cooldown = 1000;
        f.slots[0] = Some(Slot { ty: 0, kind: 3, position: [0.0, 1.75, 30.0 - 1.4] });
        let mut v = view(&no, &[], 30.0);
        v.hero_kind = 7;
        f.tick(&v, &w, &mut rng);
        assert_eq!(f.slots[0].map(|s| s.ty), Some(2));
        for _ in 0..60 {
            f.tick(&v, &w, &mut rng);
        }
        assert_eq!((f.code, f.slots[0]), (5, None));
        assert!(f.timer > 100 && f.timer <= 180);
        // The aimed bolt detonates at 2.0 into a summon.
        let mut f = Faerie::default();
        f.cooldown = 1000;
        // Pre-step separation 2.0 fails the strict gate, moves to 1.25, then passes.
        f.slots[1] = Some(Slot { ty: 1, kind: 0, position: [0.0, 1.75, 30.0 - 2.0] });
        assert!(f.tick(&view(&no, &[], 30.0), &w, &mut rng).summons.is_empty());
        let a = f.tick(&view(&no, &[], 30.0), &w, &mut rng);
        assert_eq!(a.summons.len(), 1);
        assert!(!a.summons[0].0);
        assert_eq!(attack_kind(Some(170)), 3);
    }

    #[test]
    fn stun_count_absorb_and_falls() {
        let w = words();
        let mut rng = Rng::seeded(3);
        let no = |_: u16| false;
        // A form-2 hit in the stun moves to the count; the third count sets 12 and 10.
        let mut f = Faerie::default();
        f.stuns = 2;
        f.code = 5;
        f.timer = 100;
        let mut v = view(&no, &[], 30.0);
        f.cooldown = 1000;
        v.hero_attacking = true;
        v.weapon_reaches = true;
        v.form2 = true;
        f.tick(&v, &w, &mut rng);
        assert_eq!(f.code, 6);
        let mut v = view(&no, &[], 30.0);
        v.clip = (13, false);
        f.tick(&v, &w, &mut rng);
        v.clip = (13, true);
        let a = f.tick(&v, &w, &mut rng);
        assert_eq!((a.levels.clone(), f.stuns), (vec![(12, true), (10, true)], 3));
        // The absorb window: a shot on her body is absorbed; three in the hold fall her.
        let mut f = Faerie::default();
        f.code = 7;
        f.timer = 3000;
        f.cooldown = 1000;
        let shots = [([0.0, 11.0, 0.0], 0.5)];
        let v = view(&no, &shots, 30.0);
        let a = f.tick(&v, &w, &mut rng);
        assert_eq!((a.absorbed.clone(), f.absorbs, f.hold), (vec![0], 1, 179));
        // The hover moves toward 15 above the ground (count 0 when the tick began).
        assert!((a.height.unwrap() - 10.1).abs() < 1e-5);
        f.tick(&v, &w, &mut rng);
        f.tick(&v, &w, &mut rng);
        assert_eq!(f.code, 9);
        // The fall: 0.1 per tick to the ground, then level 16 and code 10; code 10 waits for 13.
        let mut f = Faerie::default();
        f.code = 9;
        f.cooldown = 1000;
        let mut v = view(&no, &[], 30.0);
        v.position[1] = 0.05;
        let a = f.tick(&v, &w, &mut rng);
        assert_eq!((a.levels.clone(), f.code), (vec![(16, true)], 10));
        let thirteen = |f: u16| f == 13;
        let a = f.tick(&view(&thirteen, &[], 30.0), &w, &mut rng);
        assert_eq!((a.levels.clone(), f.code), (vec![(10, true)], 1));
        // The window's timeout descends at 0.05.
        let mut f = Faerie::default();
        f.code = 7;
        f.timer = 1;
        f.cooldown = 1000;
        f.tick(&view(&no, &[], 30.0), &w, &mut rng);
        assert_eq!(f.code, 8);
        let a = f.tick(&view(&no, &[], 30.0), &w, &mut rng);
        assert!((a.height.unwrap() - 9.95).abs() < 1e-5);
    }

    #[test]
    fn loop_b_duel() {
        let w = words();
        let mut rng = Rng::seeded(5);
        let no = |_: u16| false;
        let mut slots: crate::fyora::Slots = [None; 4];
        let mut f = Faerie::default();
        let mut v = view(&no, &[], 40.0);
        v.progress = false;
        // Waiting for the class-38 partner: nothing.
        assert_eq!(f.loop_b(&v, &w, None, &mut slots, false, &mut rng), Act::default());
        // A cast (3) or a summon cast (5) at once; she turns toward Fyora unless summoning.
        let fy = [0.0, 10.0, 20.0];
        let a = f.loop_b(&v, &w, Some(fy), &mut slots, false, &mut rng);
        assert!(matches!(f.b_state, 3 | 5));
        assert_eq!(a.turn_to.is_some(), f.b_state == 3);
        // With a summon alive she always casts at Fyora; its event opens a type-0 orb.
        let mut f = Faerie::default();
        f.loop_b(&v, &w, Some(fy), &mut slots, true, &mut rng);
        assert_eq!(f.b_state, 3);
        v.clip = (7, false);
        v.frame_bits = 0x20;
        f.loop_b(&v, &w, Some(fy), &mut slots, true, &mut rng);
        assert!(crate::fyora::active(&slots, 0));
        v.clip = (7, true);
        v.frame_bits = 0;
        f.loop_b(&v, &w, Some(fy), &mut slots, true, &mut rng);
        assert_eq!(f.b_state, 0);
        // A type-1 orb within 1.5: the deflect makes it type 3.
        let mut slots: crate::fyora::Slots = [None; 4];
        crate::fyora::open(&mut slots, 1, [1.0, 10.0, 0.0]);
        let mut f = Faerie::default();
        v.clip = (10, false);
        let a = f.loop_b(&v, &w, Some(fy), &mut slots, true, &mut rng);
        assert_eq!((f.b_state, slots[0].map(|s| s.ty), a.impacts.len()), (2, Some(3), 1));
        // The hero within 15: the knockback at 10.0, state 48, no push-out.
        let mut f = Faerie::default();
        let mut v = view(&no, &[], 5.0);
        v.progress = false;
        v.clip = (9, false);
        let p = f.loop_b(&v, &w, Some(fy), &mut [None; 4], false, &mut rng).push.unwrap();
        assert!((len(p.velocity) - 10.0).abs() < 1e-4 && p.state == 48 && p.offset == [0.0; 3]);
    }

    #[test]
    fn burst_and_loop_a() {
        let w = words();
        let mut rng = Rng::seeded(4);
        // The burst waits for 14, plays 14 then 10, summons four at +-4 and opens the window.
        let mut f = Faerie::default();
        f.code = 1;
        f.cooldown = 1000;
        let fourteen = |f: u16| f == 14;
        let mut v = view(&fourteen, &[], 30.0);
        f.tick(&v, &w, &mut rng);
        v.clip = (14, true);
        f.tick(&v, &w, &mut rng);
        v.clip = (10, true);
        let a = f.tick(&v, &w, &mut rng);
        assert_eq!((a.summons.len(), f.code, f.timer), (4, 7, 3000));
        assert!(a.levels.contains(&(14, false)) && a.levels.contains(&(10, false)));
        // Level 15 in the burst: 15 and 10 cleared, the stun count reset, code 0.
        let mut f = Faerie::default();
        f.code = 1;
        f.stuns = 3;
        let fifteen = |f: u16| f == 15;
        let a = f.tick(&view(&fifteen, &[], 30.0), &w, &mut rng);
        assert_eq!((f.code, f.stuns), (0, 0));
        assert!(a.levels.contains(&(15, false)));
        // Loop A: a knockback at 15.0 under 10, then a 180-tick countdown.
        let eighteen = |f: u16| f == 18;
        let mut f = Faerie::default();
        let mut v = view(&eighteen, &[], 5.0);
        v.progress = false;
        let a = f.tick(&v, &w, &mut rng);
        let p = a.push.unwrap();
        assert!((len(p.velocity) - 15.0).abs() < 1e-4 && p.state == 48);
        assert!(f.tick(&v, &w, &mut rng).push.is_none());
    }
}
