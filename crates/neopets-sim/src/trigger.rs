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
    /// The surface byte (+0x11), for a hero shot's contact cue.
    pub surface: u8,
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

    /// The hero shot pass's fire gate (SPEC-0028 section 13, Script trigger fire by flag bit):
    /// kinds 1 to 3 with bit 3, a static rectangle with bit 8, a moving one with bit 5, and the
    /// gate mask against the world's gate word.
    pub fn fires_on_shot(&self, world_mask: u16) -> bool {
        if self.mask != 0 && self.mask & world_mask == 0 {
            return false;
        }
        let bit = |n: u16| self.flags & (1 << n) != 0;
        match (self.kind, self.shape) {
            (1..=3, _) => bit(3),
            (4 | 7, Shape::Rectangle { extents, .. }) if extents[2] <= 0.0 => bit(8),
            (4 | 7, Shape::Rectangle { .. }) => bit(5),
            _ => false,
        }
    }

    /// A hero shot's world contact, first list (SPEC-0036 World contact): kinds 1 to 4 and 7
    /// whose gate passes, without the kind's trigger bit (a trigger counts as no contact; the
    /// shot pass fires it), with the solid class bit 0.
    pub fn solid_for_shot(&self, world_mask: u16) -> bool {
        if self.mask != 0 && self.mask & world_mask == 0 {
            return false;
        }
        let bit = |n: u16| self.flags & (1 << n) != 0;
        let trigger = match (self.kind, self.shape) {
            (1..=3, _) => bit(3),
            (4 | 7, Shape::Rectangle { extents, .. }) if extents[2] <= 0.0 => bit(8),
            (4 | 7, Shape::Rectangle { .. }) => bit(5),
            _ => return false,
        };
        !trigger && bit(0)
    }

    /// Whether a shot at point `p` with radius `r` touches the volume (the shot's test is not
    /// given in the spec; here the volume grown by r: a sphere and a cylinder axis by distance,
    /// a rectangle by its extents and, for a static one, within r of its plane).
    pub fn shot_hits(&self, p: [f32; 3], r: f32) -> bool {
        match self.shape {
            Shape::Sphere { centre, radius } => dot(sub(p, centre), sub(p, centre)).sqrt() <= radius + r,
            Shape::Cylinder { base, axis, length, radius } => {
                let end = [base[0] + axis[0] * length, base[1] + axis[1] * length, base[2] + axis[2] * length];
                segment_point(base, end, p) <= radius + r
            }
            Shape::Rectangle { origin, axes, extents } => {
                let d = sub(p, origin);
                let within = |k: usize, e: f32| {
                    let a = dot(d, axes[k]);
                    a >= e.min(0.0) - r && a <= e.max(0.0) + r
                };
                let across = if extents[2] > 0.0 { within(2, extents[2]) } else { dot(d, axes[2]).abs() <= r };
                within(0, extents[0]) && within(1, extents[1]) && across
            }
        }
    }

    /// The movable-block fire gate (SPEC-0048 Trigger volumes): a sphere or a cylinder with
    /// flag bit 3, a static rectangle (kind 4, switch value 0 or below) with bit 8, a moving one
    /// with bit 5, and the gate mask against the world's gate word. Kinds 2 and 7 never fire.
    pub fn fires_on_block(&self, world_mask: u16) -> bool {
        if self.mask != 0 && self.mask & world_mask == 0 {
            return false;
        }
        let bit = |n: u16| self.flags & (1 << n) != 0;
        match (self.kind, self.shape) {
            (1 | 3, _) => bit(3),
            (4, Shape::Rectangle { extents, .. }) if extents[2] <= 0.0 => bit(8),
            (4, Shape::Rectangle { .. }) => bit(5),
            _ => false,
        }
    }

    /// A moving block's test: an upright finite cylinder from the block position, radius S
    /// (0.6 S against a moving rectangle) and height H (SPEC-0048 Trigger volumes; the shapes
    /// are Medium there and the tests are the body tests with that cylinder).
    pub fn block_hits(&self, p: [f32; 3], size: f32, height: f32) -> bool {
        match self.shape {
            // The moving rectangle's five faces (SPEC-0048, The block's moving-rectangle test):
            // with a, b and u the position's coordinates from the corner along the +0x40, +0x50
            // and +0x60 axes, the net rule is a in [-r, eA + r], b in [-r, eB + r] and u > -H,
            // r = 0.6 S.
            Shape::Rectangle { origin, axes, extents } if extents[2] > 0.0 => {
                let r = 0.6 * size;
                let d = sub(p, origin);
                let (u, b, a) = (dot(d, axes[0]), dot(d, axes[1]), dot(d, axes[2]));
                (-r..=extents[2] + r).contains(&a) && (-r..=extents[1] + r).contains(&b) && u > -height
            }
            _ => self.touches_with(p, size, height),
        }
    }

    /// Whether the body standing at feet point `p` touches the volume.
    pub fn touches(&self, p: [f32; 3]) -> bool {
        self.touches_with(p, BODY_RADIUS, BODY_HEIGHT)
    }

    /// An upright cylinder of radius `body_radius` and height `body_height` from `p` against
    /// the volume.
    fn touches_with(&self, p: [f32; 3], body_radius: f32, body_height: f32) -> bool {
        let top = [p[0], p[1] + body_height, p[2]];
        match self.shape {
            Shape::Sphere { centre, radius } => segment_point(p, top, centre) <= radius + body_radius,
            Shape::Cylinder { base, axis, length, radius } if axis[1].abs() > 0.999 => {
                // Upright: the two circles meet and the height ranges overlap.
                let (lo, hi) = (base[1].min(base[1] + axis[1] * length), base[1].max(base[1] + axis[1] * length));
                let (dx, dz) = (p[0] - base[0], p[2] - base[2]);
                (dx * dx + dz * dz).sqrt() <= radius + body_radius && top[1] >= lo && p[1] <= hi
            }
            Shape::Cylinder { base, axis, length, radius } => {
                let end = [base[0] + axis[0] * length, base[1] + axis[1] * length, base[2] + axis[2] * length];
                segment_segment(p, top, base, end) <= radius + body_radius
            }
            Shape::Rectangle { origin, axes, extents } => {
                let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
                let along = |k: usize, v: [f32; 3]| v[0] * axes[k][0] + v[1] * axes[k][1] + v[2] * axes[k][2];
                // The body's extent along each axis is its axis segment plus the radius across.
                let span = |k: usize| {
                    let a = along(k, d);
                    let b = a + body_height * axes[k][1];
                    let r = body_radius * (1.0 - axes[k][1] * axes[k][1]).max(0.0).sqrt();
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

/// SPEC-0028 Blade recoil: the volume mode byte (+0x11) to the recoil material.
pub fn recoil_material(mode: u8) -> u8 {
    match mode {
        0 => 0,
        1 => 1,
        2 => 4,
        3 => 6,
        4 => 9,
        5 => 2,
        6 => 8,
        7 => 3,
        8..=16 => mode + 10,
        17 => 10,
        18 => 27,
        _ => 0,
    }
}

/// SPEC-0028 Blade recoil: a world-face record's material byte to the recoil material.
pub fn face_material(b: u8) -> u8 {
    match b {
        0..=3 => b + 1,
        4..=8 => b,
        9 | 10 => 9,
        11 => 8,
        12..=18 => b - 1,
        19 => 10,
        _ => 0,
    }
}

/// SPEC-0028 Blade recoil: the recoil cue of a material (SPEC-0036 plays it).
pub fn recoil_cue(material: u8) -> u16 {
    match material {
        1 | 6 | 20 => 11,
        2 | 23 | 26 => 20,
        3 => 18,
        4 => 13,
        5 | 22 | 27 => 15,
        7 => 17,
        8 => 19,
        9 | 21 | 24 => 12,
        10 => 29,
        11 => 25,
        12 => 21,
        13 => 22,
        14 | 18 | 25 => 23,
        15 => 26,
        16 => 24,
        17 | 19 => 28,
        _ => 11,
    }
}

/// The blade test cylinder's radius (SPEC-0028 Kind-3 blade test).
pub const BLADE_RADIUS: f32 = 0.5;

impl Volume {
    /// A solid volume for Tor's blade recoil (SPEC-0028 Blade recoil): flag bit 0 on a sphere,
    /// a cylinder or a rectangle, with the gate mask against the world's gate word (the hero's
    /// contact walk visits the primitives of the current collision mask).
    pub fn solid(&self, world_mask: u16) -> bool {
        (self.mask == 0 || self.mask & world_mask != 0) && self.flags & 1 != 0 && matches!(self.kind, 1 | 3 | 4 | 7)
    }

    /// The blade test (SPEC-0028 Blade recoil). A cylinder (kind 3) takes the upright test
    /// cylinder of radius 0.5 and length 0.25 x the hero's contact radius from the contact point
    /// `foot` (the static overlap: the volumes do not move). The other kinds take the blade
    /// triangle's three edges: a sphere by the closest distance (at most the stored radius), a
    /// static rectangle by the segment crossing its plane inside the extents, a moving one by
    /// the same test on its box's five faces (all but the top). No margins.
    pub fn blade_hits(&self, foot: [f32; 3], contact_radius: f32, tri: [[f32; 3]; 3]) -> bool {
        let edges = [(tri[0], tri[1]), (tri[0], tri[2]), (tri[1], tri[2])];
        match self.shape {
            Shape::Cylinder { .. } => self.touches_with(foot, BLADE_RADIUS, 0.25 * contact_radius),
            Shape::Sphere { centre, radius } => edges.iter().any(|&(a, b)| segment_point(a, b, centre) <= radius),
            Shape::Rectangle { origin, axes, extents } => {
                // Box coordinates of a point.
                let local = |p: [f32; 3]| {
                    let d = sub(p, origin);
                    [dot(d, axes[0]), dot(d, axes[1]), dot(d, axes[2])]
                };
                // The segment crossing the plane x[n] = c with the other two coordinates in range.
                let face = |a: [f32; 3], b: [f32; 3], n: usize, c: f32, ranges: [(usize, f32); 2]| {
                    let (da, db) = (a[n] - c, b[n] - c);
                    if (da > 0.0 && db > 0.0) || (da < 0.0 && db < 0.0) {
                        return false;
                    }
                    let t = if da == db { 0.0 } else { da / (da - db) };
                    ranges.iter().all(|&(k, e)| {
                        let x = a[k] + t * (b[k] - a[k]);
                        x >= e.min(0.0) && x <= e.max(0.0)
                    })
                };
                edges.iter().any(|&(p, q)| {
                    let (a, b) = (local(p), local(q));
                    if extents[2] <= 0.0 {
                        // Static: the plane through the corner along the normal (axis 2).
                        face(a, b, 2, 0.0, [(0, extents[0]), (1, extents[1])])
                    } else {
                        // Moving: U (axis 0, extent eU), B (axis 1, eB), A (axis 2, eA); the two
                        // A faces, the two B faces and the bottom (u = 0).
                        let (eu, eb, ea) = (extents[0], extents[1], extents[2]);
                        face(a, b, 2, 0.0, [(1, eb), (0, eu)])
                            || face(a, b, 2, ea, [(1, eb), (0, eu)])
                            || face(a, b, 1, 0.0, [(2, ea), (0, eu)])
                            || face(a, b, 1, eb, [(2, ea), (0, eu)])
                            || face(a, b, 0, 0.0, [(2, ea), (1, eb)])
                    }
                })
            }
        }
    }
}

/// The blade triangle (SPEC-0028 Blade triangle): Q the first weapon attachment frame's
/// translation, U its first basis vector, D the secondary direction, W the weapon reach;
/// r = 0.6 W, P0 = Q + 0.25 U, P1 = P0 + 0.25 r D, P2 = P0 + 0.25 r U.
pub fn blade_triangle(q: [f32; 3], u: [f32; 3], d: [f32; 3], reach: f32) -> [[f32; 3]; 3] {
    let r = 0.6 * reach;
    let p0 = [q[0] + 0.25 * u[0], q[1] + 0.25 * u[1], q[2] + 0.25 * u[2]];
    let at = |v: [f32; 3]| [p0[0] + 0.25 * r * v[0], p0[1] + 0.25 * r * v[1], p0[2] + 0.25 * r * v[2]];
    [p0, at(d), at(u)]
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
    /// +0x28, the last fire argument: -1 for hero fires, the movable block's number when a
    /// movable-block contact fired it (SPEC-0030 Layout section 3).
    pub block: i32,
}

impl Default for Record {
    fn default() -> Self {
        Self { state: 0, stamp: -1, object: false, block: -1 }
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
        self.block = last;
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
    fn blade_recoil_maps_and_tests() {
        // SPEC-0028 synthetic: mode 4 gives material 9 and cue 12; 17 gives 10 and 29; 0 and 19
        // give 0 and 11; face byte 11 gives material 8 and cue 19.
        assert_eq!((recoil_material(4), recoil_cue(9)), (9, 12));
        assert_eq!((recoil_material(17), recoil_cue(10)), (10, 29));
        assert_eq!((recoil_material(0), recoil_cue(0)), (0, 11));
        assert_eq!(recoil_material(19), 0);
        assert_eq!((face_material(11), recoil_cue(8)), (8, 19));
        assert_eq!(recoil_cue(4), 13);
        // Bit 0 marks the solid; 0x0008 alone does not.
        let sphere = Volume { kind: 1, flags: 0x0001, mask: 0, index: 0, surface: 0, shape: Shape::Sphere { centre: [0.0; 3], radius: 1.0 } };
        assert!(sphere.solid(0) && !Volume { flags: 0x0008, ..sphere }.solid(0));
        // A sphere of radius 1.0 at the origin: an edge from (1, 0, -2) to (1, 0, 2) hits (the
        // boundary counts); at x = 1.01 it misses.
        let tri = |x: f32| [[x, 0.0, -2.0], [x, 0.0, 2.0], [x, 0.0, 2.0]];
        assert!(sphere.blade_hits([9.0; 3], 1.0, tri(1.0)));
        assert!(!sphere.blade_hits([9.0; 3], 1.0, tri(1.01)));
        // A static rectangle: normal (0, 1, 0), axis (1, 0, 0) extent 10, axis (0, 0, 1) extent
        // 4. A segment crossing at (5, 0, 2) hits; one crossing at x = 12 does not.
        let rect = Volume { kind: 4, shape: Shape::Rectangle { origin: [0.0; 3], axes: [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]], extents: [10.0, 4.0, 0.0] }, ..sphere };
        let seg = |x: f32| [[x, 1.0, 2.0], [x, -1.0, 2.0], [x, -1.0, 2.0]];
        assert!(rect.blade_hits([0.0; 3], 1.0, seg(5.0)));
        assert!(!rect.blade_hits([0.0; 3], 1.0, seg(12.0)));
        // A moving rectangle (the 1324 plate's box): a segment through the far A face hits; one
        // above the box (only the top face, which is not built) does not.
        let plate = Volume { kind: 4, shape: Shape::Rectangle { origin: [0.0; 3], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [10.0, 5.0, 2.0] }, ..sphere };
        assert!(plate.blade_hits([0.0; 3], 1.0, [[2.0, 5.0, 1.0], [2.0, 5.0, 3.0], [2.0, 5.0, 3.0]]));
        assert!(!plate.blade_hits([0.0; 3], 1.0, [[2.0, 11.0, 1.0], [2.0, 9.0 + 2.5, 1.5], [2.0, 12.0, 1.0]]));
        // A cylinder: the test cylinder of radius 0.5 and length 0.25 x 4.0 = 1.0 from the foot.
        let cyl = Volume { kind: 3, shape: Shape::Cylinder { base: [0.0; 3], axis: [0.0, 1.0, 0.0], length: 3.5, radius: 1.0 }, ..sphere };
        assert!(cyl.blade_hits([1.4, 0.0, 0.0], 4.0, tri(0.0)));
        assert!(!cyl.blade_hits([1.6, 0.0, 0.0], 4.0, tri(0.0)));
        assert!(!cyl.blade_hits([0.0, -1.1, 0.0], 4.0, tri(0.0)));
        // The triangle with the default reach 1.5: r 0.9, each extension 0.225.
        let t = blade_triangle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.5);
        assert_eq!(t[0], [0.0, 0.0, 0.25]);
        assert!((t[1][0] - 0.225).abs() < 1e-6 && (t[2][2] - 0.475).abs() < 1e-6);
    }

    #[test]
    fn shot_gates_and_hits() {
        // The 0x0120 boxes of pack 1174 fire from shots only (bit 8 blocks the body's moving test).
        let b = Volume { kind: 4, flags: 0x0120, mask: 0, index: 36, surface: 0, shape: Shape::Rectangle { origin: [0.0; 3], axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], extents: [2.0, 2.0, 2.0] } };
        assert!(b.fires_on_shot(0) && !b.fires_on_contact(0));
        assert!(b.shot_hits([1.0, 1.0, 1.0], 0.5) && b.shot_hits([2.4, 1.0, 1.0], 0.5) && !b.shot_hits([3.0, 1.0, 1.0], 0.5));
        // A flags-0x0001 solid volume never fires.
        let s = Volume { kind: 1, flags: 0x0001, mask: 0, index: 0, surface: 0, shape: Shape::Sphere { centre: [0.0; 3], radius: 1.0 } };
        assert!(!s.fires_on_shot(0));
        let c = Volume { kind: 3, flags: 0x0008, mask: 0, index: 1, surface: 0, shape: Shape::Cylinder { base: [0.0; 3], axis: [0.0, 1.0, 0.0], length: 3.0, radius: 6.0 } };
        assert!(c.fires_on_shot(0) && c.shot_hits([6.2, 1.0, 0.0], 0.5) && !c.shot_hits([7.0, 1.0, 0.0], 0.5));
    }

    #[test]
    fn block_moving_rectangle() {
        // SPEC-0048's worked example: pack 1324's plate 2 and bell 0 (S 3.305, H 9.2) pushed
        // along -Z: nothing at z 3.608 (step 34 of the sixth push), a hit at z 3.565 (step 35);
        // with the bell on the ground at 0.0 nothing ever fires.
        let plate = Volume { kind: 4, flags: 0x0020, mask: 0, index: 2, surface: 0, shape: Shape::Rectangle { origin: [-20.221, 31.162, 1.266], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [10.0, 5.553, 0.340] } };
        assert!(plate.fires_on_block(0));
        assert!(!plate.block_hits([-17.429, 31.191, 3.608], 3.305, 9.2));
        assert!(plate.block_hits([-17.429, 31.191, 3.565], 3.305, 9.2));
        assert!(!plate.block_hits([-17.429, 0.0, 3.565], 3.305, 9.2));
    }

    #[test]
    fn block_gates_hits_and_number() {
        // SPEC-0048 Trigger volumes: a bit-3 sphere fires for a block, a kind-7 box never; the
        // block's cylinder has radius S and height H from its position.
        let s = Volume { kind: 1, flags: 0x0008, mask: 0, index: 2, surface: 0, shape: Shape::Sphere { centre: [0.0, 1.0, 0.0], radius: 1.0 } };
        assert!(s.fires_on_block(0) && s.block_hits([2.5, 0.0, 0.0], 2.0, 3.0) && !s.block_hits([3.5, 0.0, 0.0], 2.0, 3.0));
        let r = Volume { kind: 7, flags: 0x0100, mask: 0, index: 3, surface: 0, shape: Shape::Rectangle { origin: [0.0; 3], axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [2.0, 2.0, 0.0] } };
        assert!(!r.fires_on_block(0));
        // SPEC-0030 section 3: a block fire writes its number; a hero fire -1.
        let mut rec = Record::default();
        rec.fire(0, false, 3);
        assert_eq!((rec.state, rec.block), (1, 3));
        rec.fire(1, true, -1);
        assert_eq!(rec.block, -1);
    }

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
        let v = |kind, flags, shape| Volume { kind, flags, mask: 0, index: 0, surface: 0, shape };
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
        let pump = Volume { kind: 3, flags: 8, mask: 0, index: 5, surface: 0, shape: Shape::Cylinder { base: [-1949.41, 45.47, 1058.06], axis: [0.0, 1.0, 0.0], length: 4.49, radius: 2.66 } };
        assert!(pump.touches([-1949.41, 45.5, 1061.5]));
        assert!(!pump.touches([-1949.41, 45.5, 1062.0]), "3.94 away is past 2.66 + 1");
        assert!(!pump.touches([-1949.41, 40.0, 1058.06]), "5.5 below the base, the body's top is under it");
        let ball = Volume { kind: 1, flags: 8, mask: 0, index: 0, surface: 0, shape: Shape::Sphere { centre: [0.0, 5.0, 0.0], radius: 2.0 } };
        assert!(ball.touches([2.9, 2.0, 0.0]) && !ball.touches([3.1, 2.0, 0.0]));
        // The 909 hint box (index 7): x 0 to 5.1, y 0 to 5.85, z 0 to 4.82 from its corner.
        let o = [-1941.45, 43.85, 1073.44];
        let hint = Volume { kind: 4, flags: 0x20, mask: 0, index: 7, surface: 0, shape: Shape::Rectangle { origin: o, axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], extents: [5.85, 5.1, 4.82] } };
        assert!(hint.touches([o[0] + 2.0, o[1], o[2] + 2.0]));
        assert!(hint.touches([o[0] - 0.9, o[1], o[2] + 2.0]), "the body radius reaches in");
        assert!(!hint.touches([o[0] - 1.1, o[1], o[2] + 2.0]));
        assert!(!hint.touches([o[0] + 2.0, o[1] + 6.0, o[2] + 2.0]));
    }
}
