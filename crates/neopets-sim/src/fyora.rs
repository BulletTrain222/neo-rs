//! The Queen Fyora duel (SPEC-0029 Behaviors 19 and 30; behaviour class 38): the four shared
//! typed slots (type 0 the Darkest Faerie's orb at Fyora, type 1 Fyora's orb at her, type 2 the
//! summon orb at the hero, type 3 a deflected orb), their per-tick steps, the deflect relaunch
//! (Behavior 11, rule 4) and Queen Fyora's controller (idle, deflect, cast). The Darkest Faerie's
//! side of the duel is loop B of `darkest`.
//!
//! Only Queen Fyora's controller advances the shared slots. Every value is read from the boot
//! data at the addresses of SPEC-0029 or is one of its fixed constants.

use crate::rng::Rng;

type Vec3 = [f32; 3];

/// The duel words: the slot step (0x2DAAA8) and the deflected orb's hit (0x2DAAAC).
pub const WORDS: u32 = 0x002D_AAA8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub step: f32,
    pub hit: i32,
}

impl Words {
    pub fn read(b: &[u32]) -> Option<Self> {
        Some(Self { step: f32::from_bits(*b.first()?), hit: *b.get(1)? as i32 })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub ty: u8,
    pub position: Vec3,
    /// Type 3: the launch direction and the 60-tick timer.
    pub dir: Vec3,
    pub timer: i32,
}

pub type Slots = [Option<Slot>; 4];

/// Open a slot of a type at a point (the first free slot; all four active: nothing).
pub fn open(slots: &mut Slots, ty: u8, at: Vec3) -> bool {
    match slots.iter_mut().find(|s| s.is_none()) {
        Some(s) => {
            *s = Some(Slot { ty, position: at, dir: [0.0; 3], timer: 0 });
            true
        }
        None => false,
    }
}

pub fn active(slots: &Slots, ty: u8) -> bool {
    slots.iter().flatten().any(|s| s.ty == ty)
}

fn hdist(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

/// A slot of this type within 1.5 units (horizontal) of a point.
pub fn near(slots: &Slots, ty: u8, at: Vec3) -> bool {
    slots.iter().flatten().any(|s| s.ty == ty && hdist(s.position, at) < 1.5)
}

/// The deflect relaunch (Behavior 11, rule 4): the first active slot of the type becomes type 3
/// with a 60-tick timer, launched along unit(|f| forward + (1 - |f|) side) with the side negated
/// when f is 0 or less, f = U[0, 1) - 0.5. Returns the slot's position for its effect.
pub fn deflect(slots: &mut Slots, ty: u8, forward: Vec3, rng: &mut Rng) -> Option<Vec3> {
    let s = slots.iter_mut().flatten().find(|s| s.ty == ty)?;
    let f = rng.frange(0.0, 1.0) - 0.5;
    let side = [forward[2], 0.0, -forward[0]];
    let side = if f <= 0.0 { side.map(|c| -c) } else { side };
    let a = f.abs();
    let v: Vec3 = std::array::from_fn(|k| a * forward[k] + (1.0 - a) * side[k]);
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    s.ty = 3;
    s.timer = 60;
    s.dir = v.map(|c| c / l);
    Some(s.position)
}

/// What the slots did this tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Events {
    /// The hero took a deflected orb's hit.
    pub hero_damage: Vec<i32>,
    /// A type-2 orb reached the hero: a summon by the Faerie's slot-0 key at the hero (when level
    /// state 2 is clear).
    pub summon: bool,
}

/// One advance of the four slots (ascending order): types 0 to 2 step toward their targets
/// horizontally (the height kept); a type-2 orb ends within 2.0 of the hero (measured before the
/// step); a type-3 orb flies its direction, ends on its timer or hits the hero within 1.5.
pub fn advance(slots: &mut Slots, w: &Words, fyora: Vec3, faerie: Vec3, hero: Vec3, level2: bool) -> Events {
    let mut e = Events::default();
    for slot in slots.iter_mut() {
        let Some(s) = slot.as_mut() else { continue };
        let step = |s: &mut Slot, t: Vec3| {
            let d = [t[0] - s.position[0], t[2] - s.position[2]];
            let l = d[0].hypot(d[1]);
            if l > 0.0 {
                let k = (w.step / l).min(1.0);
                s.position[0] += d[0] * k;
                s.position[2] += d[1] * k;
            }
        };
        match s.ty {
            0 => step(s, fyora),
            1 => step(s, faerie),
            2 => {
                if hdist(s.position, hero) < 2.0 {
                    *slot = None;
                    e.summon |= !level2;
                    continue;
                }
                step(s, hero);
            }
            _ => {
                for k in 0..3 {
                    s.position[k] += w.step * s.dir[k];
                }
                s.timer -= 1;
                if s.timer < 1 {
                    *slot = None;
                } else if hdist(s.position, hero) < 1.5 {
                    e.hero_damage.push(w.hit);
                    *slot = None;
                }
            }
        }
    }
    e
}

/// Queen Fyora's view.
pub struct View {
    pub position: Vec3,
    pub facing: Vec3,
    /// The class-37 object (none: she waits).
    pub faerie: Option<Vec3>,
    pub hero: Vec3,
    /// Her sample point (the cast point).
    pub sample: Vec3,
    pub clip: (u32, bool),
    pub frame_bits: u32,
    pub level2: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Act {
    pub anim: Option<u32>,
    /// Her facing, turned with blend 1.0 toward the class-37 object.
    pub face: Option<Vec3>,
    pub events: Events,
    pub effects: Vec<(i32, Vec3)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fyora {
    pub state: u8,
    pub cooldown: i32,
    flag: bool,
}

impl Fyora {
    /// One tick: the slots, the chooser, the turn, the state.
    pub fn tick(&mut self, v: &View, slots: &mut Slots, w: &Words, rng: &mut Rng) -> Act {
        let mut a = Act::default();
        let Some(faerie) = v.faerie else { return a };
        a.events = advance(slots, w, v.position, faerie, v.hero, v.level2);
        if near(slots, 0, v.position) {
            self.state = 2;
            self.flag = false;
        } else if self.state == 0 && !active(slots, 1) {
            if self.cooldown < 1 {
                self.state = 3;
                self.flag = false;
            } else {
                self.cooldown -= 1;
            }
        }
        let to = [faerie[0] - v.position[0], 0.0, faerie[2] - v.position[2]];
        let l = to[0].hypot(to[2]);
        if l > 0.0 {
            a.face = Some([to[0] / l, 0.0, to[2] / l]);
        }
        let forward = a.face.unwrap_or(v.facing);
        let current = |n: u32| v.clip.0 == n;
        match self.state {
            2 => {
                a.anim = Some(23);
                if current(23) && !v.clip.1 && !self.flag {
                    self.flag = true;
                    if let Some(p) = deflect(slots, 0, forward, rng) {
                        a.effects.push((4007, p));
                    }
                }
                if current(23) && v.clip.1 && self.flag {
                    self.state = 0;
                }
            }
            3 => {
                a.anim = Some(22);
                if current(22) && v.frame_bits != 0 && !self.flag {
                    self.flag = true;
                    open(slots, 1, v.sample);
                }
                if current(22) && v.clip.1 && self.flag {
                    self.state = 0;
                    self.cooldown = rng.range(30, 45);
                }
            }
            _ => a.anim = Some(26),
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        Words::read(&[0.5f32.to_bits(), 5]).unwrap()
    }

    #[test]
    fn slot_steps() {
        let w = words();
        // Type 0 at (0, 7, 0) toward (3, 0, 4): (0.3, 7, 0.4).
        let mut s: Slots = [None; 4];
        open(&mut s, 0, [0.0, 7.0, 0.0]);
        advance(&mut s, &w, [3.0, 0.0, 4.0], [50.0, 0.0, 0.0], [90.0, 0.0, 0.0], false);
        let p = s[0].unwrap().position;
        assert!((p[0] - 0.3).abs() < 1e-5 && p[1] == 7.0 && (p[2] - 0.4).abs() < 1e-5);
        // Type 3 along (1, 0, 0): 30 units after 60 ticks, then freed.
        let mut s: Slots = [Some(Slot { ty: 3, position: [0.0, 2.0, 0.0], dir: [1.0, 0.0, 0.0], timer: 60 }), None, None, None];
        for _ in 0..59 {
            advance(&mut s, &w, [0.0; 3], [0.0; 3], [0.0, 0.0, 90.0], false);
        }
        assert!((s[0].unwrap().position[0] - 29.5).abs() < 1e-3);
        advance(&mut s, &w, [0.0; 3], [0.0; 3], [0.0, 0.0, 90.0], false);
        assert!(s[0].is_none());
        // A deflected orb within 1.5 of the hero hits for 5.
        let mut s: Slots = [Some(Slot { ty: 3, position: [0.0, 2.0, 0.0], dir: [1.0, 0.0, 0.0], timer: 60 }), None, None, None];
        let e = advance(&mut s, &w, [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0], false);
        assert_eq!((e.hero_damage, s[0]), (vec![5], None));
        // A type-2 orb at 2.0 steps; below 2.0 it ends with a summon unless level state 2 is set.
        let mut s: Slots = [None; 4];
        open(&mut s, 2, [0.0; 3]);
        assert!(!advance(&mut s, &w, [0.0; 3], [0.0; 3], [2.0, 0.0, 0.0], false).summon);
        assert!(advance(&mut s, &w, [0.0; 3], [0.0; 3], [2.0, 0.0, 0.0], false).summon);
        let mut s: Slots = [None; 4];
        open(&mut s, 2, [0.0; 3]);
        assert!(!advance(&mut s, &w, [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0], true).summon && s[0].is_none());
    }

    #[test]
    fn fyora_states() {
        let w = words();
        let mut rng = Rng::seeded(1);
        let mut slots: Slots = [None; 4];
        let mut f = Fyora { cooldown: 32, ..Fyora::default() };
        let v = |clip: (u32, bool), bits: u32| View { position: [0.0, 5.0, 0.0], facing: [0.0, 0.0, 1.0], faerie: Some([3.0, 9.0, 4.0]), hero: [0.0, 0.0, 40.0], sample: [0.0, 7.0, 0.0], clip, frame_bits: bits, level2: false };
        // The aim (0.6, 0, 0.8); the cooldown counts 32 ticks, then the cast on tick 33.
        let a = f.tick(&v((26, false), 0), &mut slots, &w, &mut rng);
        let face = a.face.unwrap();
        assert!((face[0] - 0.6).abs() < 1e-5 && (face[2] - 0.8).abs() < 1e-5);
        for _ in 0..31 {
            f.tick(&v((26, false), 0), &mut slots, &w, &mut rng);
        }
        assert_eq!((f.state, f.cooldown), (0, 0));
        f.tick(&v((26, false), 0), &mut slots, &w, &mut rng);
        assert_eq!(f.state, 3);
        // The cast's event opens one type-1 orb; while it flies she does not cast again.
        f.tick(&v((22, false), 1), &mut slots, &w, &mut rng);
        f.tick(&v((22, false), 1), &mut slots, &w, &mut rng);
        assert_eq!(slots.iter().flatten().filter(|s| s.ty == 1).count(), 1);
        f.tick(&v((22, true), 0), &mut slots, &w, &mut rng);
        assert!(f.state == 0 && (30..=45).contains(&f.cooldown));
        // A type-0 orb at 1.49 (horizontal) of her: the deflect; at 1.5 it is not.
        let mut slots: Slots = [None; 4];
        open(&mut slots, 0, [1.5, 5.0, 0.0]);
        let mut g = Fyora { cooldown: 100, ..Fyora::default() };
        assert!(!near(&slots, 0, [0.0, 5.0, 0.0]));
        slots[0].as_mut().unwrap().position = [1.49, 5.0, 0.0];
        assert!(near(&slots, 0, [0.0, 5.0, 0.0]));
        let mut vv = v((23, false), 0);
        vv.faerie = Some([1.49, 5.0, 30.0]);
        let a = g.tick(&vv, &mut slots, &w, &mut rng);
        assert_eq!(g.state, 2);
        assert_eq!(a.effects.len(), 1);
        assert_eq!(slots[0].map(|s| (s.ty, s.timer)), Some((3, 60)));
        // The deflect leaves 45 to 90 degrees from her facing.
        let d = slots[0].unwrap().dir;
        let face = a.face.unwrap();
        let c = d[0] * face[0] + d[2] * face[2];
        assert!((-0.01..=0.7072).contains(&c));
    }
}
