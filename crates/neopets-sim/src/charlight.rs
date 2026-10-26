//! Character lights (SPEC-0046 Character lights, Character light slots, Light slots and the lit
//! draw, Scene default lights): the four-slot light state a character draws with, rebuilt once
//! per main-loop pass from the scene default lights, the area's light set (SPEC-0028 section 2)
//! and the pass's dynamic point lights, and filled with the 5 percent ease of a persistent
//! context.
//!
//! The zone gates are the caller's: `zone` answers the zone query for an entry's zone key.

/// A light entry (the SPEC-0028 0x50-byte layout): type 1 ambient, 2 directional, 3 point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub position: [f32; 3],
    /// The travel direction (type 2 only).
    pub direction: [f32; 3],
    pub colour: [f32; 4],
    pub kind: i32,
    pub strength: f32,
    pub flag: i32,
    pub radius: f32,
    pub radius_sq: f32,
    pub zone: u8,
}

/// A light-set record: the gate word and its type 2, type 3 and type 1 arrays.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Record {
    pub gate: bool,
    pub a: Vec<Entry>,
    pub b: Vec<Entry>,
    pub c: Vec<Entry>,
}

/// The step of a kind-0 slot's colour fill (data word 0x2EC9D4).
pub const EASE: f32 = 0.05;

fn default_entry(kind: i32, flag: i32, travel: [f32; 3], colour: [f32; 3], strength: f32) -> Entry {
    Entry {
        position: [0.0; 3],
        direction: travel,
        colour: [colour[0], colour[1], colour[2], 0.0],
        kind,
        strength,
        flag,
        radius: -1.0,
        radius_sq: -1.0,
        zone: 0,
    }
}

/// The four scene default lights (SPEC-0046 Scene default lights), slots 0 to 3.
pub fn defaults() -> [Entry; 4] {
    [
        default_entry(1, 0, [0.0; 3], [50.0; 3], 1.0),
        default_entry(2, 1, [0.666, -0.524, 0.531], [255.0, 255.0, 195.0], 0.25),
        default_entry(2, 0, [0.193, -0.598, -0.778], [205.0, 205.0, 255.0], 0.25),
        default_entry(2, 0, [-0.649, -0.665, 0.368], [205.0, 205.0, 255.0], 0.25),
    ]
}

/// One light slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub entry: Option<Entry>,
    pub weight: f32,
    pub locked: bool,
    /// Taken by a dynamic light in this walk (its next fill replaces).
    pub dynamic: bool,
    /// The slot kind: 1 until the first fill, then 0 (which eases).
    pub kind: u8,
    pub stored: [f32; 4],
}

impl Default for Slot {
    fn default() -> Self {
        Self { entry: None, weight: 0.0, locked: false, dynamic: false, kind: 1, stored: [0.0; 4] }
    }
}

/// A character's light state (persistent across passes).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub slots: [Slot; 4],
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]) * (a[k] - b[k])).sum()
}

/// The type-3 weight for strength s, radius r and distance d: s to 0.5 r, linear to 0 at 0.9 r,
/// and -1 beyond.
pub fn falloff(s: f32, r: f32, d: f32) -> f32 {
    if d <= 0.5 * r {
        s
    } else if d <= 0.9 * r {
        s * (1.0 - (d - 0.5 * r) / (0.4 * r))
    } else {
        -1.0
    }
}

impl State {
    fn is_type2(&self, s: usize) -> bool {
        self.slots[s].entry.is_some_and(|e| e.kind == 2)
    }

    /// The displacement candidate of slots 2 and 3 (both occupied).
    fn candidate(&self) -> Option<usize> {
        let (w2, w3) = (self.slots[2].weight, self.slots[3].weight);
        let f = if w2 > w3 { 3 } else { 2 };
        let (l2, l3) = (self.slots[2].locked, self.slots[3].locked);
        match (self.is_type2(2), self.is_type2(3)) {
            (true, true) => Some(match (l2, l3) {
                (false, true) => 2,
                (true, false) => 3,
                _ => f,
            }),
            (t2, t3) => {
                if t2 && !l2 {
                    Some(2)
                } else if t3 && !l3 {
                    Some(3)
                } else {
                    None
                }
            }
        }
    }

    fn accept(&mut self, s: usize, e: Entry, w: f32, dynamic: bool) {
        let slot = &mut self.slots[s];
        slot.entry = Some(e);
        slot.weight = w;
        slot.dynamic = dynamic;
    }

    fn free(&self) -> Option<usize> {
        [2, 3].into_iter().find(|&s| self.slots[s].entry.is_none())
    }

    /// The slot chooser for one entry (Character light slots steps 2 to 7): stores the entry in
    /// a slot, cross-fades a candidate, or refuses it.
    pub fn offer(&mut self, e: &Entry, at: [f32; 3], dynamic: bool, zone: &impl Fn(u8) -> bool) {
        if !zone(e.zone) {
            return;
        }
        let d2 = dist_sq(e.position, at);
        if matches!(e.kind, 1 | 3) && e.radius > 0.0 && d2 >= e.radius_sq {
            return;
        }
        let d = d2.sqrt();
        match e.kind {
            1 => self.accept(0, *e, e.strength, dynamic),
            2 | 3 if e.flag != 0 => {
                let w = if e.kind == 3 { falloff(e.strength, e.radius, d) } else { e.strength };
                self.accept(1, *e, w, dynamic);
            }
            2 => {
                if let Some(s) = self.free() {
                    return self.accept(s, *e, e.strength, dynamic);
                }
                let Some(c) = self.candidate() else { return };
                if e.strength >= self.slots[c].weight {
                    self.accept(c, *e, e.strength, dynamic);
                }
            }
            3 => {
                // The one type-3 entry with radius 0 on disc is treated as absent (SPEC-0046
                // Edge cases): the original divides by zero in the cross-fade.
                if e.radius <= 0.0 {
                    return;
                }
                let w = falloff(e.strength, e.radius, d);
                if let Some(s) = self.free() {
                    return self.accept(s, *e, w, dynamic);
                }
                match self.candidate() {
                    None => {
                        if dynamic && w > 0.0 {
                            let s = if self.slots[2].weight < self.slots[3].weight { 2 } else { 3 };
                            self.accept(s, *e, w, true);
                        }
                    }
                    Some(c) => {
                        if self.slots[c].locked {
                            return;
                        }
                        let x = d - 0.9 * e.radius;
                        if x > 0.0 {
                            let strength = self.slots[c].entry.map_or(0.0, |o| o.strength);
                            self.slots[c].weight = x / (0.1 * e.radius) * strength;
                            self.slots[c].locked = true;
                        } else if w >= 0.0 {
                            self.accept(c, *e, w, dynamic);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// One walk (Character lights, order of the walk): reset, the default lights as own entries,
    /// pass 1 (type 2 entries, then the first type 1 entry, of each ungated record), pass 2
    /// (type 3 entries), then the dynamic point lights.
    pub fn walk(&mut self, records: &[Record], dynamic: &[Entry], at: [f32; 3], zone: &impl Fn(u8) -> bool) {
        for s in &mut self.slots {
            s.entry = None;
            s.weight = 0.0;
            s.locked = false;
            s.dynamic = false;
        }
        for (s, e) in defaults().into_iter().enumerate() {
            self.accept(s, e, e.strength, false);
        }
        for r in records.iter().filter(|r| !r.gate) {
            for e in r.a.iter().chain(r.c.first()) {
                self.offer(e, at, false, zone);
            }
        }
        for r in records.iter().filter(|r| !r.gate) {
            for e in &r.b {
                self.offer(e, at, false, zone);
            }
        }
        for e in dynamic {
            self.offer(e, at, true, zone);
        }
    }

    /// The colour fill of a lit draw (Light slots and the lit draw step 1): new = colour x
    /// weight; a kind-0 slot not taken by a dynamic light eases 5 percent toward it, any other
    /// replaces and becomes kind 0.
    pub fn fill(&mut self) {
        for s in &mut self.slots {
            let Some(e) = s.entry else { continue };
            let new = e.colour.map(|c| c * s.weight);
            if s.kind == 0 && !s.dynamic {
                for k in 0..4 {
                    s.stored[k] += EASE * (new[k] - s.stored[k]);
                }
            } else {
                s.stored = new;
                s.kind = 0;
            }
        }
    }

    /// The light block terms for a model at `at`: the ambient A and, for slots 1 to 3, the travel
    /// direction D_k (type 2: the entry direction; type 3: normalize(at - light position); any
    /// other type: zero colour) and the stored colour C_k.
    pub fn block(&self, at: [f32; 3]) -> ([f32; 3], [([f32; 3], [f32; 3]); 3]) {
        let rgb = |s: &Slot| [s.stored[0], s.stored[1], s.stored[2]];
        let ambient = rgb(&self.slots[0]);
        let mut out = [([0.0; 3], [0.0; 3]); 3];
        for k in 0..3 {
            let s = &self.slots[k + 1];
            out[k] = match s.entry {
                Some(e) if e.kind == 2 => (e.direction, rgb(s)),
                Some(e) if e.kind == 3 => {
                    let v = [at[0] - e.position[0], at[1] - e.position[1], at[2] - e.position[2]];
                    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                    (if l > 0.0 { v.map(|c| c / l) } else { v }, rgb(s))
                }
                _ => ([0.0; 3], [0.0; 3]),
            };
        }
        (ambient, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: fn(u8) -> bool = |_| true;

    fn point(at: [f32; 3], s: f32, r: f32) -> Entry {
        Entry { position: at, direction: [0.0; 3], colour: [255.0, 0.0, 0.0, 255.0], kind: 3, strength: s, flag: 0, radius: r, radius_sq: r * r, zone: 0 }
    }

    fn walked() -> State {
        let mut s = State::default();
        s.walk(&[], &[], [0.0; 3], &ALL);
        s
    }

    #[test]
    fn point_light_weights() {
        assert_eq!(falloff(0.5, 10.0, 0.0), 0.5);
        assert_eq!(falloff(0.5, 10.0, 6.0), 0.375);
        assert!((falloff(0.5, 10.0, 8.0) - 0.125).abs() < 1e-6);
        assert_eq!(falloff(0.5, 10.0, 9.5), -1.0);
    }

    #[test]
    fn cross_fade_of_a_default_occupant() {
        for (d, w) in [(9.5, 0.125), (9.9, 0.225)] {
            let mut s = walked();
            s.offer(&point([d, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
            assert!((s.slots[2].weight - w).abs() < 1e-5, "{d}");
            assert!(s.slots[2].locked && s.slots[2].entry.unwrap().kind == 2);
        }
        // d = r fails the radius gate (100 is not below 100).
        let mut s = walked();
        s.offer(&point([10.0, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
        assert_eq!(s, walked());
    }

    #[test]
    fn slot_ease() {
        let mut s = State::default();
        s.slots[0].entry = Some(Entry { colour: [100.0; 4], ..defaults()[0] });
        s.slots[0].weight = 1.0;
        s.slots[0].kind = 0;
        let got: Vec<f32> = (0..3)
            .map(|_| {
                s.fill();
                s.slots[0].stored[0]
            })
            .collect();
        assert!((got[0] - 5.0).abs() < 1e-4 && (got[1] - 9.75).abs() < 1e-4 && (got[2] - 14.2625).abs() < 1e-3, "{got:?}");
        // A kind-1 slot replaces on its first fill.
        let mut t = State::default();
        t.slots[0] = Slot { entry: s.slots[0].entry, weight: 1.0, ..Slot::default() };
        t.fill();
        assert_eq!((t.slots[0].stored[0], t.slots[0].kind), (100.0, 0));
    }

    #[test]
    fn walk_takes_the_first_two_point_lights() {
        let record = Record { gate: false, b: vec![point([1.0, 0.0, 0.0], 0.5, 10.0), point([0.0, 1.0, 0.0], 0.3, 10.0)], ..Record::default() };
        let later = Record { gate: false, b: vec![point([0.0, 0.0, 1.0], 0.9, 10.0)], ..Record::default() };
        let mut s = State::default();
        s.walk(&[record.clone(), later], &[], [0.0; 3], &ALL);
        assert_eq!(s.slots[2].entry.unwrap().strength, 0.5);
        assert_eq!(s.slots[3].entry.unwrap().strength, 0.3);
        // A dynamic light with no candidate replaces the weaker of slots 2 and 3.
        s.walk(&[record], &[point([0.0, 0.0, 2.0], 0.4, 10.0)], [0.0; 3], &ALL);
        assert_eq!(s.slots[3].entry.unwrap().strength, 0.4);
        assert!(s.slots[3].dynamic && !s.slots[2].dynamic);
    }

    #[test]
    fn gates_and_type_rules() {
        // A gated record never reaches the walk.
        let ambient = Entry { colour: [128.0, 110.0, 83.0, 255.0], strength: 0.2, radius: 10.0, radius_sq: 100.0, ..defaults()[0] };
        let mut s = State::default();
        s.walk(&[Record { gate: true, c: vec![ambient], ..Record::default() }], &[], [0.0; 3], &ALL);
        assert_eq!(s.slots[0].entry.unwrap().strength, 1.0);
        // Radius gate of a type 1 entry: refused at 10.0 units, taken at 9.9.
        let mut s = walked();
        s.offer(&ambient, [10.0, 0.0, 0.0], false, &ALL);
        assert_eq!(s.slots[0].entry.unwrap().strength, 1.0);
        s.offer(&ambient, [9.9, 0.0, 0.0], false, &ALL);
        assert_eq!(s.slots[0].entry.unwrap().strength, 0.2);
        // Type 0 is refused; a failed zone query refuses.
        let mut s = walked();
        s.offer(&Entry { kind: 0, ..ambient }, [0.0; 3], false, &ALL);
        s.offer(&ambient, [0.0; 3], false, &|_| false);
        assert_eq!(s, walked());
        // First free slot: slot 2, then slot 3.
        let mut s = State::default();
        s.offer(&point([1.0, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
        assert!(s.slots[2].entry.is_some() && s.slots[3].entry.is_none());
        s.offer(&point([1.0, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
        assert!(s.slots[3].entry.is_some());
    }

    #[test]
    fn displacement_and_synthetic_cross_fade() {
        let mut s = walked();
        s.slots[2].weight = 3.0;
        s.slots[3].weight = 2.0;
        s.slots[3].entry = Some(Entry { strength: 0.6, ..defaults()[3] });
        assert_eq!(s.candidate(), Some(3));
        let mut fade = s;
        fade.offer(&point([9.5, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
        assert!((fade.slots[3].weight - 0.3).abs() < 1e-5 && fade.slots[3].locked);
        assert_eq!(fade.slots[3].entry.unwrap().kind, 2);
        s.offer(&point([8.5, 0.0, 0.0], 0.5, 10.0), [0.0; 3], false, &ALL);
        assert_eq!(s.slots[3].entry.unwrap().kind, 3);
    }

    #[test]
    fn default_block_matches_the_scene_rig() {
        let mut s = walked();
        s.fill();
        let (a, d) = s.block([0.0; 3]);
        assert_eq!(a, [50.0; 3]);
        assert_eq!(d[0].1, [63.75, 63.75, 48.75]);
        assert_eq!(d[1].1, [51.25, 51.25, 63.75]);
        // A point light's direction is normalize(model - light).
        let mut p = State::default();
        p.slots[1] = Slot { entry: Some(point([3.0, 0.0, 0.0], 1.0, 10.0)), weight: 1.0, ..Slot::default() };
        assert_eq!(p.block([0.0, 0.0, 4.0]).1[0].0, [-0.6, 0.0, 0.8]);
    }
}
