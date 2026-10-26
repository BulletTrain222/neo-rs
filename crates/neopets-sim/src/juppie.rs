//! The Juppie boss (SPEC-0015; behaviour class 18): the stages and their HP thresholds, the
//! hub and stage-start states, the attack cadence, the head slam with its stage events, stun
//! loop and counted hits, the set-backs and the death sequence, and the finale holds. The level
//! flags it reads and posts are shared with the level script (SPEC-0045 pack 1217).
//!
//! Not covered: the spit and the dazed loop (both need the spit flag, the Act 3 marker 83),
//! the stun box blocker (the head cylinder blocker is returned) and the hit helper's deflection
//! effects. Every tuning value is read from the boot data at the addresses of SPEC-0015.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The tuning words (SPEC-0015 Layout, Tuning values and fixed values read at run time).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub slam_min: f32,
    pub slam_max: f32,
    pub slam_chance: i32,
    pub slam_damage: i32,
    pub stun_min: f32,
    pub stun_max: f32,
    pub counts: [i32; 3],
    pub spit_distance: f32,
    pub spit_chance: i32,
    pub cooldown: [(f32, f32); 3],
    pub impact: f32,
    pub push: f32,
    pub cone: f32,
    pub locator_distance: f32,
    pub stun_table: [i32; 3],
}

/// The boot addresses: (slam min, max, chance, damage, stun min, max, counts, spit distance,
/// chance, cooldowns, impact radius, push, locator cone, locator distance, stun table).
pub const SLAM_MIN: u32 = 0x002D_AF50;
pub const SLAM_MAX: u32 = 0x002D_AF54;
pub const SLAM_PUSH: u32 = 0x002D_AF5C;
pub const STUN_MIN: u32 = 0x002D_AF60;
pub const STUN_MAX: u32 = 0x002D_AF64;
pub const SLAM_CHANCE: u32 = 0x002D_AF68;
pub const SLAM_DAMAGE: u32 = 0x002D_AF6C;
pub const SPIT_DISTANCE: u32 = 0x002D_AF70;
pub const SPIT_CHANCE: u32 = 0x002D_AF78;
pub const COOLDOWNS: u32 = 0x002D_AF88;
pub const COUNTS: u32 = 0x002D_AFA0;
pub const LOCATOR_DISTANCE: u32 = 0x002D_AFB8;
pub const LOCATOR_CONE: u32 = 0x002D_AFBC;
pub const IMPACT: u32 = 0x002D_AFC8;
pub const STUN_TABLE: u32 = 0x002D_AFE8;

/// The head bone (15) and the stage locators by stage with their level flags.
pub const HEAD_BONE: usize = 15;
const LOCATORS: [&[(usize, u16)]; 3] = [&[(101, 2)], &[(102, 6), (103, 7)], &[(104, 11), (105, 12), (106, 13)]];
/// The stage locators (tag, level flag) of a stage 1 to 3.
pub fn stage_locators(stage: u8) -> &'static [(usize, u16)] {
    LOCATORS.get((stage.max(1) - 1) as usize).copied().unwrap_or(&[])
}

/// The stage thresholds (5/6 in single precision, 1/2, 0).
const THRESHOLDS: [f32; 3] = [0.833_333_4, 0.5, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Top {
    /// Waiting for the script's stage flags (animation 6 once, then 7).
    Hub { played: bool },
    /// The stage start (animation 5, facing the hero), then free.
    StageStart,
    /// The free decision state (animation 7).
    Free,
    /// The head slam at a sub-step.
    Slam(u8),
    /// The finale hold (top-level state 6, animation 7) and the idle hold (animation 0).
    Finale,
    Idle,
}

/// What the boss sees this tick.
pub struct View<'a> {
    pub position: Vec3,
    pub facing: Vec3,
    /// The head bone's world position.
    pub head: Vec3,
    pub hero: Vec3,
    /// The playing clip: action, progress, ended.
    pub clip: (u32, f32, bool),
    /// The boss-side action-frame query (event bits 0x1 to 0x10 on its frame).
    pub can_act: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub level: &'a dyn Fn(u16) -> bool,
    pub locator: &'a dyn Fn(usize) -> Option<Vec3>,
    /// The hero's melee damage when its swing hit the boss this tick.
    pub hero_hit: Option<i32>,
}

/// What it asks for this tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Turn toward the hero at this rate.
    pub face: Option<f32>,
    /// A hit on the hero (raw damage) with the slam push speed.
    pub hero_damage: Option<(i32, f32)>,
    pub hp_loss: i32,
    pub set_level: Vec<u16>,
    pub clear_level: Vec<u16>,
    /// The death sequence: every other living non-townsfolk character takes 10000 and the boss's
    /// HP becomes 1.
    pub death: bool,
    /// The head cylinder blocker (centre, radius 4.0, length 10.0) of the stun.
    pub blocker: Option<Vec3>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Juppie {
    pub stage: u8,
    pub marker: Option<u8>,
    pub top: Top,
    cooldown: i32,
    stun: i32,
    counted: i32,
    slam_latch: bool,
    once: bool,
    pub uses: u32,
    /// The controller's start request of animation 7 has been made.
    started: bool,
}

impl Default for Juppie {
    fn default() -> Self {
        Self { stage: 0, marker: None, top: Top::Hub { played: false }, cooldown: 0, stun: 0, counted: 0, slam_latch: false, once: false, uses: 0, started: false }
    }
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn ticks(s: f32) -> i32 {
    (s * 60.0) as i32
}

impl Juppie {
    fn fraction(v: &View) -> f32 {
        if v.max_hp <= 0 { 0.0 } else { v.hp as f32 / v.max_hp as f32 }
    }

    fn threshold(&self) -> f32 {
        THRESHOLDS[(self.stage.max(1) - 1) as usize]
    }

    /// Play an animation: requested each tick; returns whether it is current and has ended.
    fn play(a: &mut Act, v: &View, id: u32) -> bool {
        a.anim = Some(id);
        v.clip.0 == id && v.clip.2
    }

    /// One tick.
    pub fn tick(&mut self, v: &View, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        match self.top {
            Top::Hub { .. } => {
                // The controller's start requests animation 7 once (R-0036 4); after that the
                // hub's rule: nothing while 7 is current, 6 from any other animation, and 7 once
                // 6 reaches the 0.99 gate.
                if !self.started {
                    self.started = true;
                    a.anim = Some(7);
                } else if v.clip.0 == 7 {
                } else if v.clip.0 != 6 {
                    a.anim = Some(6);
                } else if v.clip.2 {
                    a.anim = Some(7);
                }
                let next = match self.stage {
                    0 if (v.level)(1) => Some(1),
                    1 if (v.level)(5) => Some(2),
                    2 if (v.level)(10) => Some(3),
                    _ => None,
                };
                if let Some(s) = next {
                    self.stage = s;
                    self.marker = None;
                    self.top = Top::StageStart;
                }
            }
            Top::StageStart => {
                a.face = Some(1.0);
                if Self::play(&mut a, v, 5) {
                    self.top = Top::Free;
                }
            }
            Top::Free => self.free(v, w, rng, &mut a),
            Top::Slam(s) => self.slam(s, v, w, rng, &mut a),
            Top::Finale => {
                a.anim = Some(7);
                if (v.level)(18) {
                    self.top = Top::Idle;
                }
            }
            Top::Idle => a.anim = Some(0),
        }
        a
    }

    /// The free state: the stage rules when the stage is complete, else the cadence.
    fn free(&mut self, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        a.anim = Some(7);
        let f = Self::fraction(v);
        if let Some(stage) = self.marker {
            self.marker = None;
            match stage {
                1 if f <= THRESHOLDS[0] => a.set_level.push(3),
                1 => {
                    self.stage = 0;
                    a.set_level.push(0xF);
                    a.clear_level.push(1);
                }
                2 if f <= THRESHOLDS[1] => a.set_level.push(8),
                2 => {
                    self.stage = 1;
                    a.set_level.push(0x10);
                    a.clear_level.push(5);
                }
                _ if f <= 0.0 => {
                    self.death(a);
                    return;
                }
                _ => {
                    self.stage = 2;
                    a.set_level.push(0x11);
                    a.clear_level.push(10);
                }
            }
            self.top = Top::Hub { played: false };
            return;
        }
        if self.stage == 0 {
            return;
        }
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return;
        }
        let (lo, hi) = w.cooldown[(self.stage.max(1) - 1) as usize];
        self.cooldown = rng.range(ticks(lo), ticks(hi));
        let d = hdist(v.position, v.hero);
        if d > w.slam_min && d < w.slam_max && rng.range(0, 100) < w.slam_chance {
            self.top = Top::Slam(0);
        }
        // The spit needs the spit flag (Act 3), which this build does not run.
    }

    fn death(&mut self, a: &mut Act) {
        a.set_level.push(0xE);
        a.death = true;
        self.top = Top::Finale;
    }

    /// The stage events of a slam: the first usable locator of the stage whose flag is clear.
    fn stage_events(&mut self, v: &View, w: &Words, a: &mut Act) -> bool {
        let Some(list) = LOCATORS.get((self.stage.max(1) - 1) as usize) else { return false };
        for &(id, flag) in *list {
            if (v.level)(flag) {
                continue;
            }
            let Some(p) = (v.locator)(id) else { continue };
            let (dx, dz) = (p[0] - v.position[0], p[2] - v.position[2]);
            let l = (dx * dx + dz * dz).sqrt();
            if l <= 0.0 {
                continue;
            }
            let dot = (dx * v.facing[0] + dz * v.facing[2]) / l;
            let head = ((v.head[0] - p[0]).powi(2) + (v.head[1] - p[1]).powi(2) + (v.head[2] - p[2]).powi(2)).sqrt();
            if dot > w.cone && head < w.locator_distance {
                a.set_level.push(flag);
                self.uses += 1;
                return true;
            }
        }
        false
    }

    fn slam(&mut self, s: u8, v: &View, w: &Words, rng: &mut Rng, a: &mut Act) {
        match s {
            0 => {
                a.face = Some(1.0);
                if Self::play(a, v, 0x15) {
                    self.slam_latch = false;
                    self.once = false;
                    self.counted = 0;
                    self.top = Top::Slam(1);
                }
            }
            1 => {
                let ended = Self::play(a, v, 0x12);
                if !ended && v.clip.0 == 0x12 && !self.slam_latch && v.can_act {
                    // The stage events, tried on each window tick before the impact until one
                    // locator is used (at most one use per slam).
                    if !self.once && self.stage_events(v, w, a) {
                        self.once = true;
                    }
                    if hdist(v.head, v.hero) < w.impact {
                        self.slam_latch = true;
                        a.hero_damage = Some((w.slam_damage, w.push));
                    }
                }
                if ended {
                    if self.once {
                        self.stun = rng.range(ticks(w.stun_min), ticks(w.stun_max));
                        self.top = Top::Slam(2);
                    } else {
                        self.top = Top::Slam(4);
                    }
                }
            }
            2 => {
                a.anim = Some(0x10);
                a.blocker = Some(v.head);
                if let Some(d) = v.hero_hit {
                    self.counted += 1;
                    a.hp_loss = w.stun_table[0] * d;
                    let f = (v.hp - a.hp_loss) as f32 / v.max_hp.max(1) as f32;
                    if f <= 0.0 {
                        self.death(a);
                        return;
                    }
                    if f > self.threshold() {
                        self.top = Top::Slam(3);
                    } else {
                        self.marker = Some(self.stage);
                        self.top = Top::Slam(4);
                    }
                    return;
                }
                self.stun -= 1;
                if self.stun <= 0 {
                    self.top = Top::Slam(4);
                }
            }
            3 => {
                if Self::play(a, v, 9) {
                    let count = w.counts[(self.stage.max(1) - 1) as usize];
                    self.top = if self.counted < count { Top::Slam(2) } else { Top::Slam(4) };
                }
            }
            _ => {
                if Self::play(a, v, 0x11) {
                    self.top = Top::Free;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        Words {
            slam_min: 10.0,
            slam_max: 50.0,
            slam_chance: 50,
            slam_damage: 25,
            stun_min: 2.0,
            stun_max: 5.0,
            counts: [3; 3],
            spit_distance: 100.0,
            spit_chance: 50,
            cooldown: [(1.5, 3.5), (0.75, 2.0), (0.25, 1.0)],
            impact: 7.0,
            push: 5.0,
            cone: 0.9,
            locator_distance: 10.0,
            stun_table: [4, 2, 0],
        }
    }

    fn view<'a>(level: &'a dyn Fn(u16) -> bool, locator: &'a dyn Fn(usize) -> Option<Vec3>, clip: (u32, f32, bool)) -> View<'a> {
        View { position: [0.0; 3], facing: [0.0, 0.0, 1.0], head: [0.0, 0.0, 3.0], hero: [0.0, 0.0, 20.0], clip, can_act: true, hp: 190, max_hp: 190, level, locator, hero_hit: None }
    }

    #[test]
    fn the_fight_starts_on_animation_7() {
        // SPEC-0015 test vector (R-0036 4).
        let w = words();
        let mut rng = Rng::seeded(1);
        let no = |_: u16| false;
        let none = |_: usize| None;
        let mut j = Juppie::default();
        assert_eq!(j.tick(&view(&no, &none, (0, 0.0, false)), &w, &mut rng).anim, Some(7), "the start requests 7");
        assert_eq!(j.tick(&view(&no, &none, (7, 0.3, false)), &w, &mut rng).anim, None, "7 current: nothing");
        assert_eq!(j.tick(&view(&no, &none, (17, 1.0, true)), &w, &mut rng).anim, Some(6), "after a set-back: 6");
        assert_eq!(j.tick(&view(&no, &none, (6, 0.5, false)), &w, &mut rng).anim, None);
        assert_eq!(j.tick(&view(&no, &none, (6, 0.99, true)), &w, &mut rng).anim, Some(7));
    }

    #[test]
    fn hub_stage_start_and_cadence() {
        let mut rng = Rng::seeded(1);
        let w = words();
        let none = |_: usize| None;
        let flag1 = |f: u16| f == 1;
        let mut j = Juppie::default();
        // The hub plays 6 once, then waits; level flag 1 starts stage 1 (animation 5).
        j.tick(&view(&flag1, &none, (6, 1.0, true)), &w, &mut rng);
        assert_eq!((j.stage, j.top), (1, Top::StageStart));
        j.tick(&view(&flag1, &none, (5, 1.0, true)), &w, &mut rng);
        assert_eq!(j.top, Top::Free);
        // Cooldown draws in stage 1: 90 to 210 ticks.
        let mut seen = Vec::new();
        for _ in 0..200 {
            let mut k = Juppie { stage: 1, top: Top::Free, ..Juppie::default() };
            k.tick(&view(&flag1, &none, (7, 0.5, false)), &w, &mut rng);
            seen.push(k.cooldown);
        }
        assert!(seen.iter().all(|&c| (90..=210).contains(&c)), "{seen:?}");
        // Distance 10.0 is not eligible; 10.5 is (with a roll below 50 about half the time).
        let mut slams = 0;
        for _ in 0..400 {
            let mut k = Juppie { stage: 1, top: Top::Free, ..Juppie::default() };
            let mut v = view(&flag1, &none, (7, 0.5, false));
            v.hero = [0.0, 0.0, 10.0];
            k.tick(&v, &w, &mut rng);
            assert_eq!(k.top, Top::Free);
            v.hero = [0.0, 0.0, 10.5];
            let mut k = Juppie { stage: 1, top: Top::Free, ..Juppie::default() };
            k.tick(&v, &w, &mut rng);
            slams += (k.top == Top::Slam(0)) as i32;
        }
        assert!((150..250).contains(&slams), "{slams}");
    }

    #[test]
    fn slam_impact_events_and_stun() {
        let mut rng = Rng::seeded(2);
        let w = words();
        let loc = |id: usize| (id == 101).then_some([0.0, 0.0, 5.0]);
        let flags = |_: u16| false;
        let mut j = Juppie { stage: 1, top: Top::Slam(1), ..Juppie::default() };
        // Locator 101 ahead (dot 1.0), the head 2.0 from it: used; the hero 6.9 from the head:
        // hit for 25 with the push 5.0.
        let mut v = view(&flags, &loc, (0x12, 0.4, false));
        v.hero = [0.0, 0.0, 9.9];
        let a = j.tick(&v, &w, &mut rng);
        assert_eq!((a.set_level.clone(), a.hero_damage, j.uses), (vec![2], Some((25, 5.0)), 1));
        // At the clip end: the stun wait (120 to 300) and sub-step 2.
        j.tick(&view(&flags, &loc, (0x12, 1.0, true)), &w, &mut rng);
        assert_eq!(j.top, Top::Slam(2));
        assert!((120..=300).contains(&j.stun));
        // A counted hit of 12 costs 48; above the threshold: the count check (animation 9).
        let mut v = view(&flags, &loc, (0x10, 0.5, false));
        (v.hp, v.max_hp) = (400, 400);
        v.hero_hit = Some(12);
        let a = j.tick(&v, &w, &mut rng);
        assert_eq!((a.hp_loss, j.top), (48, Top::Slam(3)));
        let stun = j.stun;
        j.tick(&view(&flags, &loc, (9, 1.0, true)), &w, &mut rng);
        assert_eq!((j.top, j.stun), (Top::Slam(2), stun), "no new stun draw");
        // A hit that takes the fraction to the threshold marks the stage complete.
        let mut v = view(&flags, &loc, (0x10, 0.5, false));
        v.hp = 160;
        v.hero_hit = Some(1);
        j.tick(&v, &w, &mut rng);
        assert_eq!((j.marker, j.top), (Some(1), Top::Slam(4)));
        // Recover, then free with the marker: f <= 5/6 sets level flag 3, back to the hub.
        j.tick(&view(&flags, &loc, (0x11, 1.0, true)), &w, &mut rng);
        let mut v = view(&flags, &loc, (7, 0.1, false));
        v.hp = 156;
        let a = j.tick(&v, &w, &mut rng);
        assert_eq!((a.set_level, j.top), (vec![3], Top::Hub { played: false }));
        // A slam 7.0 from the hero misses.
        let mut j = Juppie { stage: 1, top: Top::Slam(1), once: true, ..Juppie::default() };
        let mut v = view(&flags, &loc, (0x12, 0.4, false));
        v.hero = [0.0, 0.0, 10.0];
        assert_eq!(j.tick(&v, &w, &mut rng).hero_damage, None);
    }

    #[test]
    fn set_backs_and_death() {
        let mut rng = Rng::seeded(3);
        let w = words();
        let none = |_: usize| None;
        let flags = |_: u16| false;
        // Stage 2 at 0.51: back to 1, level flag 0x10 set and 5 cleared.
        let mut j = Juppie { stage: 2, marker: Some(2), top: Top::Free, ..Juppie::default() };
        let mut v = view(&flags, &none, (7, 0.1, false));
        v.hp = 97;
        let a = j.tick(&v, &w, &mut rng);
        assert_eq!((j.stage, a.set_level, a.clear_level), (1, vec![0x10], vec![5]));
        // Stage 3 at 0: the death sequence (flag 0xE, finale); level flag 18 then the idle hold.
        let mut j = Juppie { stage: 3, marker: Some(3), top: Top::Free, ..Juppie::default() };
        let mut v = view(&flags, &none, (7, 0.1, false));
        v.hp = 0;
        let a = j.tick(&v, &w, &mut rng);
        assert!(a.death && a.set_level == vec![0xE] && j.top == Top::Finale);
        let f18 = |f: u16| f == 18;
        j.tick(&view(&f18, &none, (7, 0.1, false)), &w, &mut rng);
        assert_eq!(j.top, Top::Idle);
    }
}
