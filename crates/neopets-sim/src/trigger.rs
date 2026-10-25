//! Script trigger records and the hero's body-contact fire (SPEC-0030 Layout section 3 and
//! Behavior section 3; SPEC-0028 section 13, Script trigger fire by flag bit and Trigger pass).
//!
//! A volume carries a trigger index; the selected hero's body fires it with object 1 when its
//! flags pass the gate of its kind and the body touches it. The record keeps the state word
//! (0 idle, 1 first fire, 2 repeated fire, 3 decaying), the frame stamp and the object word.
//! The tested shapes are Medium-Low in the spec: the body is the cylinder of radius 1.0 from the
//! feet to 3.5 above, tested here by circle and height overlap (upright cylinders), the distance
//! between axis segments (other cylinders), the distance from a
//! segment (spheres) and by its extent along each rectangle axis (rectangles).

/// A volume's shape (the type-11 records of SPEC-0028 sections 6.1 to 6.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Sphere { centre: [f32; 3], radius: f32 },
    Cylinder { base: [f32; 3], axis: [f32; 3], length: f32, radius: f32 },
    /// Axes V, A and the normal B; signed extents along them from the reference point, the last
    /// the switch value (zero or below: a static rectangle; above zero: a moving one).
    Rectangle { origin: [f32; 3], axes: [[f32; 3]; 3], extents: [f32; 3] },
}

/// A trigger volume: its kind (1 sphere, 2 cone, 3 cylinder, 4 and 7 rectangles, 8 fourth list),
/// flags, gate mask and trigger index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub kind: u8,
    pub flags: u16,
    pub mask: u16,
    pub index: u16,
    pub shape: Shape,
}

/// The body cylinder of a character's trigger step (SPEC-0028 Trigger pass, Tested shape).
pub const BODY_RADIUS: f32 = 1.0;
pub const BODY_HEIGHT: f32 = 3.5;

impl Volume {
    /// The body-contact fire gate of the volume's kind (SPEC-0028 section 13 table, cell list A
    /// walk) and the gate mask against the world's gate word.
    pub fn fires_on_contact(&self, world_mask: u16) -> bool {
        if self.mask != 0 && self.mask & world_mask == 0 {
            return false;
        }
        let f = self.flags;
        let bit = |n: u16| f & (1 << n) != 0;
        match (self.kind, self.shape) {
            (1 | 2, _) => !bit(0) && bit(3),
            (3, _) => bit(3) && !bit(1) && !bit(2),
            (4 | 7, Shape::Rectangle { extents, .. }) if extents[2] <= 0.0 => bit(8) && !bit(0) && !bit(1) && !bit(4) && !bit(6),
            (4 | 7, Shape::Rectangle { .. }) => bit(5) && !bit(1) && !bit(7) && !bit(8),
            _ => false,
        }
    }

    /// Whether the body standing at feet point `p` touches the volume.
    pub fn touches(&self, p: [f32; 3]) -> bool {
        let top = [p[0], p[1] + BODY_HEIGHT, p[2]];
        match self.shape {
            Shape::Sphere { centre, radius } => segment_point(p, top, centre) <= radius + BODY_RADIUS,
            Shape::Cylinder { base, axis, length, radius } if axis[1].abs() > 0.999 => {
                // Upright: the two circles meet and the height ranges overlap.
                let (lo, hi) = (base[1].min(base[1] + axis[1] * length), base[1].max(base[1] + axis[1] * length));
                let (dx, dz) = (p[0] - base[0], p[2] - base[2]);
                (dx * dx + dz * dz).sqrt() <= radius + BODY_RADIUS && top[1] >= lo && p[1] <= hi
            }
            Shape::Cylinder { base, axis, length, radius } => {
                let end = [base[0] + axis[0] * length, base[1] + axis[1] * length, base[2] + axis[2] * length];
                segment_segment(p, top, base, end) <= radius + BODY_RADIUS
            }
            Shape::Rectangle { origin, axes, extents } => {
                let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
                let along = |k: usize, v: [f32; 3]| v[0] * axes[k][0] + v[1] * axes[k][1] + v[2] * axes[k][2];
                // The body's extent along each axis is its axis segment plus the radius across.
                let span = |k: usize| {
                    let a = along(k, d);
                    let b = a + BODY_HEIGHT * axes[k][1];
                    let r = BODY_RADIUS * (1.0 - axes[k][1] * axes[k][1]).max(0.0).sqrt();
                    (a.min(b) - r, a.max(b) + r)
                };
                let inside = |k: usize, e: f32| {
                    let (lo, hi) = span(k);
                    hi > e.min(0.0) && lo < e.max(0.0)
                };
                // A static rectangle (switch zero or below) is a plane: the body must reach it.
                let (lo, hi) = span(2);
                let across = if extents[2] > 0.0 { inside(2, extents[2]) } else { lo <= 0.0 && hi >= 0.0 };
                inside(0, extents[0]) && inside(1, extents[1]) && across
            }
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn segment_point(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = sub(b, a);
    let t = (dot(sub(c, a), ab) / dot(ab, ab).max(1e-12)).clamp(0.0, 1.0);
    let q = [a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]];
    dot(sub(c, q), sub(c, q)).sqrt()
}

/// The closest distance between segments p0p1 and q0q1.
fn segment_segment(p0: [f32; 3], p1: [f32; 3], q0: [f32; 3], q1: [f32; 3]) -> f32 {
    let (d1, d2, r) = (sub(p1, p0), sub(q1, q0), sub(p0, q0));
    let (a, e, f) = (dot(d1, d1), dot(d2, d2), dot(d2, r));
    let (s, t) = if a <= 1e-12 && e <= 1e-12 {
        (0.0, 0.0)
    } else if a <= 1e-12 {
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = dot(d1, r);
        if e <= 1e-12 {
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b = dot(d1, d2);
            let den = a * e - b * b;
            let mut s = if den > 1e-12 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
            let mut t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
            (s, t)
        }
    };
    let cp = [p0[0] + d1[0] * s, p0[1] + d1[1] * s, p0[2] + d1[2] * s];
    let cq = [q0[0] + d2[0] * t, q0[1] + d2[1] * t, q0[2] + d2[2] * t];
    dot(sub(cp, cq), sub(cp, cq)).sqrt()
}

/// A trigger record's collision block (SPEC-0030 Layout section 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub state: u8,
    pub stamp: i64,
    /// The object word: true only when the selected hero's body fired it.
    pub object: bool,
}

impl Default for Record {
    fn default() -> Self {
        Self { state: 0, stamp: -1, object: false }
    }
}

impl Record {
    /// The fire step's write on `frame` (the update rule): written when the object is set, the
    /// record is idle, or its object is clear and `last` is 0 or above.
    pub fn fire(&mut self, frame: i64, object: bool, last: i32) {
        if !(object || self.state == 0 || (!self.object && last >= 0)) {
            return;
        }
        if object && !self.object && self.state != 0 {
            self.state = 1;
        } else if self.state == 0 {
            self.state = 1;
        } else if self.state == 1 && self.stamp != frame {
            self.state = 2;
        }
        self.stamp = frame;
        self.object = object;
    }

    /// The decay in the script step of `frame`: a record last written before frame - 1 toggles
    /// to 3, then to 0.
    pub fn decay(&mut self, frame: i64) {
        if self.state != 0 && self.stamp >= 0 && self.stamp < frame - 1 {
            self.state = if self.state == 3 { 0 } else { 3 };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_contact_states_and_decay() {
        // SPEC-0030: entering gives 1, staying 2, and after the last contact frame s the record
        // reads 3 from the script step of s + 2 and 0 from s + 3.
        let mut r = Record::default();
        let mut seen = Vec::new();
        for f in 0..8 {
            if f < 3 {
                r.fire(f, true, -1);
            }
            r.decay(f);
            seen.push(r.state);
        }
        assert_eq!(seen, vec![1, 2, 2, 2, 3, 0, 0, 0]);
    }

    #[test]
    fn object_zero_firer_cycles() {
        // SPEC-0030: an object-0 firer that stays inside a record it started cycles 1, 1, 3, 0, 1.
        let mut r = Record::default();
        let states: Vec<u8> = (0..5)
            .map(|f| {
                r.fire(f, false, -1);
                r.decay(f);
                r.state
            })
            .collect();
        assert_eq!(states, vec![1, 1, 3, 0, 1]);
    }

    #[test]
    fn a_hero_write_over_an_object_zero_record_restarts_at_1() {
        let mut r = Record::default();
        r.fire(0, false, -1);
        r.fire(1, false, -1);
        assert_eq!((r.state, r.object), (1, false));
        r.fire(2, false, 4);
        assert_eq!(r.state, 2, "a movable block's write repeats");
        r.fire(3, true, -1);
        assert_eq!((r.state, r.object), (1, true));
    }

    #[test]
    fn fire_gates_by_kind_and_flag_bit() {
        let cyl = Shape::Cylinder { base: [0.0; 3], axis: [0.0, 1.0, 0.0], length: 4.0, radius: 2.0 };
        let v = |kind, flags, shape| Volume { kind, flags, mask: 0, index: 0, shape };
        assert!(v(3, 0x0008, cyl).fires_on_contact(0));
        assert!(!v(3, 0x0001, cyl).fires_on_contact(0), "a solid volume never fires");
        assert!(!v(3, 0x000A, cyl).fires_on_contact(0));
        let moving = Shape::Rectangle { origin: [0.0; 3], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [5.0, 5.0, 4.0] };
        let fixed = Shape::Rectangle { origin: [0.0; 3], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [5.0, 5.0, 0.0] };
        assert!(v(4, 0x0020, moving).fires_on_contact(0));
        assert!(!v(4, 0x0020, fixed).fires_on_contact(0));
        assert!(v(7, 0x0100, fixed).fires_on_contact(0));
        assert!(!v(4, 0x0120, moving).fires_on_contact(0), "0x0120 boxes fire only from shots and blocks");
        assert!(!v(8, 0x0008, cyl).fires_on_contact(0));
        let mut gated = v(3, 0x0008, cyl);
        gated.mask = 2;
        assert!(!gated.fires_on_contact(0) && gated.fires_on_contact(3));
    }

    #[test]
    fn body_contact_shapes() {
        // The 909 pump cylinder: base (-1949.41, 45.47, 1058.06), up, length 4.49, radius 2.66.
        let pump = Volume { kind: 3, flags: 8, mask: 0, index: 5, shape: Shape::Cylinder { base: [-1949.41, 45.47, 1058.06], axis: [0.0, 1.0, 0.0], length: 4.49, radius: 2.66 } };
        assert!(pump.touches([-1949.41, 45.5, 1061.5]));
        assert!(!pump.touches([-1949.41, 45.5, 1062.0]), "3.94 away is past 2.66 + 1");
        assert!(!pump.touches([-1949.41, 40.0, 1058.06]), "5.5 below the base, the body's top is under it");
        let ball = Volume { kind: 1, flags: 8, mask: 0, index: 0, shape: Shape::Sphere { centre: [0.0, 5.0, 0.0], radius: 2.0 } };
        assert!(ball.touches([2.9, 2.0, 0.0]) && !ball.touches([3.1, 2.0, 0.0]));
        // The 909 hint box (index 7): x 0 to 5.1, y 0 to 5.85, z 0 to 4.82 from its corner.
        let o = [-1941.45, 43.85, 1073.44];
        let hint = Volume { kind: 4, flags: 0x20, mask: 0, index: 7, shape: Shape::Rectangle { origin: o, axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [5.85, 5.1, 4.82] } };
        assert!(hint.touches([o[0] + 2.0, o[1], o[2] + 2.0]));
        assert!(hint.touches([o[0] - 0.9, o[1], o[2] + 2.0]), "the body radius reaches in");
        assert!(!hint.touches([o[0] - 1.1, o[1], o[2] + 2.0]));
        assert!(!hint.touches([o[0] + 2.0, o[1] + 6.0, o[2] + 2.0]));
    }
}
