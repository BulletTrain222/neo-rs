//! Swing trails (SPEC-0034 3.8 and section 18): ribbons between two edge points sampled once per
//! pass, with a colour gradient per record. The manager holds 12 trails, each a ring of 18
//! samples. A sample takes colour A, moves to B over c updates and to C by the cap n, and is
//! retired at age n + 1; its alpha is also multiplied each update by its fade factor F, set
//! from how far the next sample's points moved (0 at or below d0, 1 at or above d1). A trail
//! ends when none of its samples is live.
//!
//! Not covered: the u wrap of the draw (+0x48, 0 in every record) and the override colour
//! (never set). The newest sample's F, before a later sample sets it, is taken as 1.

use crate::hero::Vec3;
use std::collections::VecDeque;

pub const TRAILS: usize = 12;
pub const RING: usize = 18;

/// A colour-gradient record (section 18 layout): colours A, B and C (R, G, B, alpha in GS units),
/// count c, cap n, fade distances d0 and d1, steps per update without an append, u step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub c: [f32; 4],
    pub count: i32,
    pub cap: i32,
    pub d0: f32,
    pub d1: f32,
    pub steps: i32,
    pub u_step: f32,
}

const fn grey(a: f32, b: f32, c: f32, steps: i32, u_step: f32, cap: i32) -> Record {
    Record { a: [128.0, 128.0, 128.0, a], b: [128.0, 128.0, 128.0, b], c: [128.0, 128.0, 128.0, c], count: 2, cap, d0: 0.02, d1: 0.3, steps, u_step }
}

/// The held-weapon record (0x392790; SPEC-0034 3.8 Records).
pub const HELD: Record = grey(32.0, 10.0, 0.0, 1, 2.8, 14);

const fn element(rgb: [f32; 3]) -> Record {
    Record { a: [rgb[0], rgb[1], rgb[2], 48.0], b: [rgb[0], rgb[1], rgb[2], 40.0], c: [rgb[0], rgb[1], rgb[2], 2.0], count: 2, cap: 14, d0: 0.02, d1: 0.3, steps: 1, u_step: 2.8 }
}

/// The eight element records of section 18 (records 0 to 7, by record number).
pub const ELEMENT: [Record; 8] = [
    element([128.0, 128.0, 128.0]),
    element([255.0, 0.0, 0.0]),
    element([0.0, 128.0, 0.0]),
    element([0.0, 0.0, 128.0]),
    element([0.0, 128.0, 128.0]),
    element([128.0, 0.0, 128.0]),
    element([128.0, 128.0, 0.0]),
    Record { a: [128.0, 0.0, 0.0, 48.0], b: [0.0, 128.0, 0.0, 40.0], c: [0.0, 0.0, 128.0, 2.0], count: 8, cap: 14, d0: 0.02, d1: 0.3, steps: 1, u_step: 2.8 },
];

/// The record of a trail kind (the attacker's element; 3.8 step 2): kinds 0 to 7 take records
/// 0, 1, 2, 6, 3, 4, 5 and 7; any other kind record 0.
pub fn record_for(kind: i32) -> Record {
    const MAP: [usize; 8] = [0, 1, 2, 6, 3, 4, 5, 7];
    ELEMENT[usize::try_from(kind).ok().and_then(|k| MAP.get(k).copied()).unwrap_or(0)]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub p1: Vec3,
    pub p2: Vec3,
    pub width: f32,
    pub colour: [f32; 4],
    /// The age in updates; -1 once retired.
    pub age: i32,
    pub u: f32,
    pub fade: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trail {
    pub record: Record,
    pub samples: VecDeque<Sample>,
    appended: bool,
}

fn dist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl Trail {
    pub fn new(record: Record, p1: Vec3, p2: Vec3) -> Self {
        let mut t = Self { record, samples: VecDeque::with_capacity(RING), appended: false };
        t.append(p1, p2);
        t
    }

    /// One sample: width |p1 - p2|, colour A, age 0, u the previous u + D times the u step; the
    /// previous sample takes its fade from D.
    pub fn append(&mut self, p1: Vec3, p2: Vec3) {
        let r = self.record;
        let mut u = 0.0;
        if let Some(prev) = self.samples.back_mut() {
            let d = dist(p1, prev.p1).max(dist(p2, prev.p2));
            prev.fade = if d <= r.d0 {
                0.0
            } else if d >= r.d1 {
                1.0
            } else {
                (d - r.d0) / (r.d1 - r.d0)
            };
            u = prev.u + d * r.u_step;
        }
        if self.samples.len() == RING {
            self.samples.pop_front();
        }
        self.samples.push_back(Sample { p1, p2, width: dist(p1, p2), colour: r.a, age: 0, u, fade: 1.0 });
        self.appended = true;
    }

    /// The age-and-colour step of one update: 1 step after an append, else the record's steps.
    pub fn age(&mut self) {
        let r = self.record;
        let steps = if std::mem::take(&mut self.appended) { 1 } else { r.steps };
        for _ in 0..steps {
            for s in self.samples.iter_mut().filter(|s| s.age >= 0) {
                s.age += 1;
                if s.age > r.cap {
                    s.age = -1;
                    s.colour = r.c;
                    continue;
                }
                for k in 0..4 {
                    let d = if s.age <= r.count { (r.b[k] - r.a[k]) / r.count as f32 } else { (r.c[k] - r.b[k]) / (r.cap - r.count) as f32 };
                    s.colour[k] += d;
                }
                s.colour[3] *= s.fade;
                for v in &mut s.colour {
                    *v = v.clamp(0.0, 255.0);
                }
            }
        }
    }

    pub fn live(&self) -> bool {
        self.samples.iter().any(|s| s.age >= 0)
    }

    /// The quads to draw: consecutive pairs of live samples.
    pub fn segments(&self) -> impl Iterator<Item = (&Sample, &Sample)> {
        self.samples.iter().zip(self.samples.iter().skip(1)).filter(|(a, b)| a.age >= 0 && b.age >= 0)
    }
}

/// The trail manager: 12 slots.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trails {
    slots: Vec<Option<Trail>>,
}

impl Trails {
    /// Take a free trail and start it; None when all 12 are in use.
    pub fn start(&mut self, record: Record, p1: Vec3, p2: Vec3) -> Option<usize> {
        if self.slots.len() < TRAILS {
            self.slots.resize(TRAILS, None);
        }
        let k = self.slots.iter().position(Option::is_none)?;
        self.slots[k] = Some(Trail::new(record, p1, p2));
        Some(k)
    }

    /// Append to trail `h` when it is still going; false when it has ended.
    pub fn append(&mut self, h: usize, p1: Vec3, p2: Vec3) -> bool {
        match self.slots.get_mut(h).and_then(Option::as_mut) {
            Some(t) => {
                t.append(p1, p2);
                true
            }
            None => false,
        }
    }

    /// Keep a held trail going (3.8 Held-weapon trails): append while it lives, else start a
    /// new one. Returns the handle.
    pub fn hold(&mut self, held: Option<usize>, record: Record, p1: Vec3, p2: Vec3) -> Option<usize> {
        match held {
            Some(h) if self.append(h, p1, p2) => Some(h),
            _ => self.start(record, p1, p2),
        }
    }

    /// One update: every trail ages; a trail with no live sample ends and its slot is freed.
    pub fn tick(&mut self) {
        for slot in &mut self.slots {
            if let Some(t) = slot {
                t.age();
                if !t.live() {
                    *slot = None;
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Trail> {
        self.slots.iter().flatten()
    }

    /// Free every trail (a world change).
    pub fn clear(&mut self) {
        self.slots.clear();
    }
}

/// The held-weapon edge points (3.8): point 1 = B + 0.25 D, point 2 = point 1 + L D.
pub fn held_points(base: Vec3, d: Vec3, l: f32) -> (Vec3, Vec3) {
    let p1: Vec3 = std::array::from_fn(|k| base[k] + 0.25 * d[k]);
    (p1, std::array::from_fn(|k| p1[k] + l * d[k]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_points() {
        // Kind 3: record 6 (yellow); kind 4: record 3 (blue); kind 9 and kind -1: record 0.
        assert_eq!(record_for(3).a, [128.0, 128.0, 0.0, 48.0]);
        assert_eq!(record_for(4).a, [0.0, 0.0, 128.0, 48.0]);
        assert_eq!(record_for(9), ELEMENT[0]);
        assert_eq!(record_for(-1), ELEMENT[0]);
        // Tor with item 98 (W 1.7, L 1.02); no weapon (L 0.9); an enemy (L 1.5).
        let (b, d) = ([10.0, 0.0, 5.0], [0.0, 0.0, 1.0]);
        let close = |a: Vec3, e: Vec3| (0..3).all(|k| (a[k] - e[k]).abs() < 1e-5);
        assert!(close(held_points(b, d, 0.6 * 1.7).1, [10.0, 0.0, 6.27]));
        assert!(close(held_points(b, d, 0.9).1, [10.0, 0.0, 6.15]));
        assert!(close(held_points(b, d, 1.5).1, [10.0, 0.0, 6.75]));
        assert!(close(held_points(b, d, 1.5).0, [10.0, 0.0, 5.25]));
    }

    #[test]
    fn colour_and_retirement() {
        // Record 0, F = 1, one step per update: alpha 44, 40, then 21 at age 8, 2 at age 14;
        // retired at 15 with alpha 2, and the trail ends.
        let mut t = Trail::new(ELEMENT[0], [0.0; 3], [0.0, 1.0, 0.0]);
        let alpha = |t: &Trail| t.samples[0].colour[3];
        t.age();
        assert!((alpha(&t) - 44.0).abs() < 1e-4);
        t.age();
        assert!((alpha(&t) - 40.0).abs() < 1e-4);
        for _ in 3..=8 {
            t.age();
        }
        assert!((alpha(&t) - 21.0).abs() < 1e-3);
        for _ in 9..=14 {
            t.age();
        }
        assert!((alpha(&t) - 2.0).abs() < 1e-3 && t.live());
        t.age();
        assert_eq!(t.samples[0].age, -1);
        assert_eq!(alpha(&t), 2.0);
        assert!(!t.live());
    }

    #[test]
    fn fade_compounds() {
        // D = 0.16 between samples gives F = 0.5: alpha 48 becomes 22 and then 9.
        let mut t = Trail::new(ELEMENT[0], [0.0; 3], [0.0, 1.0, 0.0]);
        t.append([0.16, 0.0, 0.0], [0.16, 1.0, 0.0]);
        assert!((t.samples[0].fade - 0.5).abs() < 1e-5);
        assert!((t.samples[1].u - 0.16 * 2.8).abs() < 1e-5);
        t.age();
        assert!((t.samples[0].colour[3] - 22.0).abs() < 1e-4);
        t.age();
        assert!((t.samples[0].colour[3] - 9.0).abs() < 1e-4);
        // D at or below 0.02: F 0; at or above 0.3: F 1.
        let mut t = Trail::new(ELEMENT[0], [0.0; 3], [0.0, 1.0, 0.0]);
        t.append([0.02, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(t.samples[0].fade, 0.0);
        t.append([0.5, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(t.samples[1].fade, 1.0);
    }

    #[test]
    fn manager_slots_and_hold() {
        let mut m = Trails::default();
        let hs: Vec<_> = (0..12).map(|_| m.start(HELD, [0.0; 3], [0.0, 1.0, 0.0])).collect();
        assert!(hs.iter().all(Option::is_some));
        assert_eq!(m.start(HELD, [0.0; 3], [0.0, 1.0, 0.0]), None);
        // A held trail appends while it lives; once ended, hold starts a new one.
        let mut m = Trails::default();
        let h = m.hold(None, HELD, [0.0; 3], [0.0, 1.0, 0.0]);
        assert_eq!(m.hold(h, HELD, [0.1, 0.0, 0.0], [0.1, 1.0, 0.0]), h);
        assert_eq!(m.iter().next().unwrap().samples.len(), 2);
        for _ in 0..20 {
            m.tick();
        }
        assert_eq!(m.iter().count(), 0);
        assert!(m.hold(h, HELD, [0.0; 3], [0.0, 1.0, 0.0]).is_some());
    }
}
