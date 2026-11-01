//! Box push-out of a point (SPEC-0018, Box convention and Point versus box with radius r): a box
//! is a low corner o, a vertical axis V and two horizontal axes A and B with signed full extents.
//! A point inside the box grown by r is pushed out through the nearest face. The corner rule and
//! the preferred-face rule (Medium-Low in the spec) are not implemented.

use crate::hero::Vec3;

/// SPEC-0018 Constants: the 1.25 push radius. SPEC-0040 assigns it to the follow camera's eye
/// pass; characters push with their motion-record radius instead.
pub const CONTACT_RADIUS: f32 = 1.25;
/// SPEC-0040 Layout, body size: both heroes have radius 1.175 and height 3.5.
pub const HERO_RADIUS: f32 = 1.175;
const TOP_MARGIN: f32 = 0.1;
const BOTTOM_BIAS: f32 = 0.75;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShape {
    pub origin: Vec3,
    /// Vertical, A and B axes (unit, perpendicular).
    pub axes: [Vec3; 3],
    /// Extents along V, A and B (signed; used as stored).
    pub extents: [f32; 3],
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The push out of a point: the face (1 top, 2 bottom, 3 low B, 4 high B, 5 high A, 6 low A),
/// its outward normal and the push distance. None when the point is outside the grown box.
pub fn push(p: Vec3, b: &BoxShape, r: f32) -> Option<(u8, Vec3, f32)> {
    let d = [p[0] - b.origin[0], p[1] - b.origin[1], p[2] - b.origin[2]];
    let [vax, aax, bax] = b.axes;
    let (v, a, bb) = (dot(d, vax), dot(d, aax), dot(d, bax));
    let [ev, ea, eb] = b.extents;
    let inside = -r < a && a < ea + r && -r < bb && bb < eb + r && -r < v && v < ev - TOP_MARGIN;
    if !inside {
        return None;
    }
    let neg = |x: Vec3| x.map(|c| -c);
    let faces = [
        (1u8, vax, ev - TOP_MARGIN - v),
        (2, neg(vax), v + BOTTOM_BIAS),
        (3, neg(bax), bb),
        (4, bax, eb - bb),
        (5, aax, ea - a),
        (6, neg(aax), a),
    ];
    // Smallest depth wins; ties go to the lower face number.
    let mut best = faces[0];
    for f in &faces[1..] {
        if f.2 < best.2 {
            best = *f;
        }
    }
    // Top gate: a shallow top with all side depths above -0.5 r takes the top face.
    if faces[0].2 < 0.25 && faces[2..].iter().all(|f| f.2 > -0.5 * r) {
        best = faces[0];
    }
    let dist = match best.0 {
        1 => best.2,
        2 => (v + BOTTOM_BIAS + 0.5 * r).abs(),
        _ => best.2 + r,
    };
    Some((best.0, best.1, dist))
}

/// The sphere contact radius (SPEC-0018 Sphere contact; boot word 0x2DB400).
pub const SPHERE_CONTACT: f32 = 1.25;

/// A collision sphere pushes characters out when its id word has class 0 or class 1 and not
/// class 2 (SPEC-0018 Primitive class bits; Sphere contact).
pub fn sphere_pushes(id: u16) -> bool {
    id & 3 != 0 && id & 4 == 0
}

/// Sphere contact: a point closer than r + 1.25 to a sphere's centre moves along the unit vector
/// from the centre to it, onto the sphere of radius r + 1.25. Returns whether any pushed.
pub fn resolve_spheres(p: &mut Vec3, spheres: &[(Vec3, f32)]) -> bool {
    let mut any = false;
    for &(c, r) in spheres {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let reach = r + SPHERE_CONTACT;
        if len < reach && len > 0.0 {
            *p = std::array::from_fn(|k| c[k] + d[k] / len * reach);
            any = true;
        }
    }
    any
}

/// Push a point out of every box it is inside, in order, and report whether any pushed.
pub fn resolve(p: &mut Vec3, boxes: &[BoxShape], r: f32) -> bool {
    let mut hit = false;
    for b in boxes {
        if let Some((_, n, dist)) = push(*p, b, r) {
            for k in 0..3 {
                p[k] += n[k] * dist;
            }
            hit = true;
        }
    }
    hit
}

/// The finite cylinder test's parallel branch and the slide response's move for an upright owner
/// body (feet `p`, radius `owner_r`, height `owner_h`) against an upright cylinder (base centre,
/// length, radius), SPEC-0018 Breakable cylinders against characters and Tree trunks against the
/// hero: contact when the horizontal distance L is at most r + owner_r and the two centres are
/// within half the summed heights. The move puts the owner on the circle of r + owner_r. When L
/// is below owner_r and the axial overlap is at most 0.4 the end-cap rule uses the vertical
/// axis instead, moving the owner toward the volume by the overlap. Returns the move and the
/// unit normal from the owner toward the volume.
pub fn upright_cylinder_push(p: Vec3, owner_r: f32, owner_h: f32, base: Vec3, length: f32, radius: f32) -> Option<(Vec3, Vec3)> {
    let (dx, dz) = (base[0] - p[0], base[2] - p[2]);
    let l = dx.hypot(dz);
    let reach = radius + owner_r;
    let offset = (p[1] + 0.5 * owner_h) - (base[1] + 0.5 * length);
    let overlap = 0.5 * (length + owner_h) - offset.abs();
    if l > reach || overlap < 0.0 {
        return None;
    }
    if l < owner_r && overlap <= 0.4 {
        let down = if offset > 0.0 { -1.0 } else { 1.0 };
        return Some(([0.0, down * overlap, 0.0], [0.0, down, 0.0]));
    }
    let n = if l > 1e-4 { [dx / l, 0.0, dz / l] } else { [1.0, 0.0, 0.0] };
    let depth = l - reach;
    Some(([n[0] * depth, 0.0, n[2] * depth], n))
}

/// A wall edge (a frame record, SPEC-0018 and SPEC-0033 Behavior 37): the face plane (unit
/// normal, offset), the run plane, the top plane, the length L and the height value H.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallEdge {
    pub face: [f32; 4],
    pub run: [f32; 4],
    pub top: [f32; 4],
    pub length: f32,
    pub height: f32,
    /// The record's type byte (+0x4C): type 6 touches from both sides (SPEC-0033 Behavior 51A).
    pub kind: i8,
}

fn plane(q: [f32; 4], p: Vec3) -> f32 {
    q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
}

impl WallEdge {
    /// SPEC-0033 Behavior 37: the height where the top plane reaches H, lowered by 0.25.
    pub fn edge_height(&self, p: Vec3) -> f32 {
        let m = self.top;
        (self.height - m[0] * p[0] - m[2] * p[2] - m[3]) / m[1] - 0.25
    }

    /// Two-sided wall correction of a body of radius r at p. The ordinary-wall rule of SPEC-0018
    /// (Petpet frame-record wall pass): the correction is r − |a| along the face normal, toward
    /// the side the point is on, where a is the face-plane value. Gates taken from the edge
    /// tests of SPEC-0033 Behavior 37: a near-vertical face (|n_y| ≤ 0.2) and a run-plane value
    /// inside (0, L). The height gate (the top more than 0.25 above the feet, so a low step is not
    /// a wall) is this implementation's choice: the hero's own wall routine is not specified in
    /// that detail (Medium-Low; R-0011).
    pub fn push(&self, p: Vec3, r: f32) -> Option<Vec3> {
        let n = [self.face[0], self.face[1], self.face[2]];
        if n[1].abs() > 0.2 || self.length <= 0.0 {
            return None;
        }
        let a = plane(self.face, p);
        let s = plane(self.run, p);
        if a.abs() >= r || s <= 0.0 || s >= self.length || self.top[1] == 0.0 {
            return None;
        }
        // The wall hangs H below its top edge: the top-plane value is 0 at its bottom. A body
        // whose feet are 3.5 (the hero height) or more below the bottom is not touched.
        if self.edge_height(p) <= p[1] + 0.25 || plane(self.top, p) <= -3.5 {
            return None;
        }
        let side = if a < 0.0 { -1.0 } else { 1.0 };
        let d = (r - a.abs()) * side;
        Some([n[0] * d, 0.0, n[2] * d])
    }
}

impl WallEdge {
    /// The hero's per-record wall contact (SPEC-0033 Behavior 51A item 2; SPEC-0018): a record
    /// touches when the body is within r in front of its face (one-sided; type 6 from either
    /// side), within −r ≤ s ≤ L + r along it, and its wall span holds the feet (the top, less
    /// 0.25, more than 0.3 above them; the feet less than the hero height below its bottom). The
    /// push puts the body r in front of the face. A body that crossed the face from the front
    /// during this step (`from` in front, `p` less than r behind) is put back in front (this
    /// build's guard against stepping through a wall in one tick).
    pub fn hero_push(&self, p: Vec3, r: f32, from: Vec3) -> Option<Vec3> {
        let n = [self.face[0], self.face[1], self.face[2]];
        if n[1].abs() > 0.2 || self.length <= 0.0 || self.top[1] == 0.0 {
            return None;
        }
        let a = plane(self.face, p);
        let s = plane(self.run, p);
        if !(-r..=self.length + r).contains(&s) || a >= r || a <= -r {
            return None;
        }
        let t = self.edge_height(p);
        if p[1] >= t - 0.3 || p[1] <= t - self.height - crate::climb::HEIGHT {
            return None;
        }
        let side = if self.kind == 6 {
            if a < 0.0 { -1.0 } else { 1.0 }
        } else if a >= 0.0 || plane(self.face, from) >= 0.0 {
            1.0
        } else {
            return None;
        };
        let d = side * r - a;
        Some([n[0] * d, 0.0, n[2] * d])
    }
}

/// The hero's wall contact over a list of records (`WallEdge::hero_push`), in order, with the
/// slide response's velocity clip (SPEC-0018 Slide response step 3): a velocity `v` into a
/// pushing face loses its component along the face normal. Reports whether any record pushed.
pub fn resolve_hero_walls(p: &mut Vec3, v: &mut Vec3, walls: &[WallEdge], r: f32, from: Vec3) -> bool {
    let mut hit = false;
    for w in walls {
        if let Some(d) = w.hero_push(*p, r, from) {
            p[0] += d[0];
            p[2] += d[2];
            let len = d[0].hypot(d[2]);
            if len > 1e-6 {
                let n = [d[0] / len, 0.0, d[2] / len];
                let vn = n[0] * v[0] + n[2] * v[2];
                if vn < 0.0 {
                    v[0] -= vn * n[0];
                    v[2] -= vn * n[2];
                }
            }
            hit = true;
        }
    }
    hit
}

/// As `resolve_walls`, but each wall pushes toward the side of its face plane that `from` (the
/// position at the start of the step) was on, so a step that ends just past a wall's plane is
/// put back on the side it came from instead of out through the far side (this build's guard;
/// a hero stepping 1.2 past a wall at the Black Knight's dais went through it).
pub fn resolve_walls_from(p: &mut Vec3, walls: &[WallEdge], r: f32, from: Vec3) -> bool {
    let mut hit = false;
    for w in walls {
        if let Some(d) = w.push(*p, r) {
            let (a, a0) = (plane(w.face, *p), plane(w.face, from));
            let flip = a0.abs() > 1e-3 && (a < 0.0) != (a0 < 0.0);
            let d = if flip {
                // Back to the starting side: the full reach r from the plane.
                let side = if a0 < 0.0 { -1.0 } else { 1.0 };
                let n = [w.face[0], w.face[1], w.face[2]];
                let m = side * r - a;
                [n[0] * m, 0.0, n[2] * m]
            } else {
                d
            };
            p[0] += d[0];
            p[2] += d[2];
            hit = true;
        }
    }
    hit
}

/// Push a point out of each wall it touches, in order. Reports whether any wall pushed.
pub fn resolve_walls(p: &mut Vec3, walls: &[WallEdge], r: f32) -> bool {
    let mut hit = false;
    for w in walls {
        if let Some(d) = w.push(*p, r) {
            p[0] += d[0];
            p[2] += d[2];
            hit = true;
        }
    }
    hit
}

#[cfg(test)]
mod tests {

    #[test]
    fn sphere_contact() {
        // Class 0 or 1 push; class 2 or only class 3 or 4 does not.
        assert!(sphere_pushes(1) && sphere_pushes(2) && sphere_pushes(3));
        assert!(!sphere_pushes(5) && !sphere_pushes(8) && !sphere_pushes(16) && !sphere_pushes(0));
        // A sphere of radius 2 at the origin: a point 1.0 away on +x ends 3.25 away; one at 4.0
        // stays.
        let mut p = [1.0, 0.0, 0.0];
        assert!(resolve_spheres(&mut p, &[([0.0; 3], 2.0)]));
        assert!((p[0] - 3.25).abs() < 1e-6 && p[1] == 0.0);
        let mut q = [4.0, 0.0, 0.0];
        assert!(!resolve_spheres(&mut q, &[([0.0; 3], 2.0)]));
    }

    use super::*;

    /// SPEC-0028's synthetic box: reference (10, 0, -5), extents V 6, A 4, B 2, axes Y, X, Z.
    fn sample() -> BoxShape {
        BoxShape {
            origin: [10.0, 0.0, -5.0],
            axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            extents: [6.0, 4.0, 2.0],
        }
    }

    #[test]
    fn outside_the_grown_box_is_no_hit() {
        let b = sample();
        assert_eq!(push([5.0, 1.0, -4.0], &b, 1.25), None);
        // At v = eV - 0.1 or above there is no hit (strict upper bound, no growth).
        assert_eq!(push([12.0, 5.9, -4.0], &b, 1.25), None);
    }

    #[test]
    fn chest_cylinder_vectors() {
        // SPEC-0018 Breakable cylinders against characters, pack 951's breakable 28 as numbers.
        let (base, len, r) = ([-14.9674, 11.9709, -24.7527], 2.0594, 2.5929);
        let push = |x, y, z| upright_cylinder_push([x, y, z], 1.0, 3.5, base, len, r);
        let (d, n) = push(-14.967, 11.971, -21.5).unwrap();
        assert!((-21.5 + d[2] - -21.1598).abs() < 1e-3 && (n[2] - -1.0).abs() < 1e-3, "{d:?} {n:?}");
        let (d, _) = push(-13.0, 11.971, -22.5).unwrap();
        assert!((-13.0 + d[0] - -12.604).abs() < 1e-3 && (-22.5 + d[2] - -22.0465).abs() < 1e-3, "{d:?}");
        assert!(push(-14.967, 11.971, -20.7).is_none(), "L = 4.0527");
        let (d, _) = push(-12.967, 14.0, -24.753).unwrap();
        assert!((-12.967 + d[0] - -11.3744).abs() < 1e-3, "{d:?}");
        let (d, n) = push(-14.4674, 13.9, -24.7527).unwrap();
        assert!((d[1] - -0.1303).abs() < 1e-3 && n[1] == -1.0, "the end cap: {d:?}");
        // The feet range ends at b - 3.5 and b + H.
        assert!(push(-13.0, 8.46, -24.7).is_none() && push(-13.0, 14.04, -24.7).is_none());
        assert!(push(-13.0, 8.48, -24.7).is_some() && push(-13.0, 14.02, -24.7).is_some());
    }

    #[test]
    fn low_a_side_pushes_to_minus_r() {
        let b = sample();
        // a = -0.5, b = 1, v = 1: nearest is the low A side (depth -0.5); push = depth + r.
        let (face, n, dist) = push([9.5, 1.0, -4.0], &b, 1.25).unwrap();
        assert_eq!((face, n), (6, [-1.0, 0.0, 0.0]));
        let mut p = [9.5, 1.0, -4.0];
        resolve(&mut p, &[b], 1.25);
        assert!((p[0] - (10.0 - 1.25)).abs() < 1e-6, "a becomes -r: {p:?} {dist}");
    }

    #[test]
    fn shallow_top_lifts_onto_the_box() {
        let b = sample();
        // v = 5.8: top depth 0.1 < 0.25 and sides deep inside: top face, push 0.1 up.
        let (face, n, dist) = push([12.0, 5.8, -4.0], &b, 1.25).unwrap();
        assert_eq!((face, n), (1, [0.0, 1.0, 0.0]));
        assert!((dist - 0.1).abs() < 1e-5);
    }

    #[test]
    fn negative_extent_narrows_the_interval() {
        // SPEC-0028: extent -2 with radius 1.25 admits coordinate -1; with radius 1.0, none.
        let mut b = sample();
        b.extents = [6.0, -2.0, 2.0];
        assert!(push([9.0, 1.0, -4.0], &b, 1.25).is_some());
        assert!(push([9.0, 1.0, -4.0], &b, 1.0).is_none());
    }

    /// A wall at x = 0 facing +x, running along z from z = 0 for 10, top plane y with H = 5.
    fn wall() -> WallEdge {
        WallEdge { face: [1.0, 0.0, 0.0, 0.0], run: [0.0, 0.0, 1.0, 0.0], top: [0.0, 1.0, 0.0, 0.0], length: 10.0, height: 5.0, kind: 0 }
    }

    #[test]
    fn hero_wall_contact_is_one_sided() {
        // In front within r: out to r. Behind the face (a parallel record the hero stands
        // behind): no push, unless the step crossed the face from the front. Type 6 both sides.
        let w = wall();
        let r = 1.175;
        let p = w.hero_push([0.5, 0.0, 5.0], r, [0.5, 0.0, 5.0]).unwrap();
        assert!((p[0] - 0.675).abs() < 1e-6);
        assert_eq!(w.hero_push([-0.5, 0.0, 5.0], r, [-0.5, 0.0, 5.0]), None);
        let back = w.hero_push([-0.5, 0.0, 5.0], r, [0.3, 0.0, 5.0]).unwrap();
        assert!((back[0] - 1.675).abs() < 1e-6, "crossed this step: back in front");
        let six = WallEdge { kind: 6, ..w }.hero_push([-0.5, 0.0, 5.0], r, [-0.5, 0.0, 5.0]).unwrap();
        assert!((six[0] + 0.675).abs() < 1e-6);
        // Lateral reach −r to L + r; the top must be 0.55 above the feet; the feet less than H
        // below the bottom.
        assert!(w.hero_push([0.5, 0.0, 11.0], r, [0.5, 0.0, 11.0]).is_some());
        assert!(w.hero_push([0.5, 0.0, 11.2], r, [0.5, 0.0, 11.2]).is_none());
        assert!(w.hero_push([0.5, 4.2, 5.0], r, [0.5, 4.2, 5.0]).is_some());
        assert!(w.hero_push([0.5, 4.5, 5.0], r, [0.5, 4.5, 5.0]).is_none());
        assert!(w.hero_push([0.5, -3.0, 5.0], r, [0.5, -3.0, 5.0]).is_some());
        assert!(w.hero_push([0.5, -9.0, 5.0], r, [0.5, -9.0, 5.0]).is_none());
        // The velocity into the face is clipped; along it is kept.
        let (mut p, mut v) = ([0.5, 0.0, 5.0], [-3.0, 1.0, 2.0]);
        let from = p;
        assert!(resolve_hero_walls(&mut p, &mut v, &[w], r, from));
        assert_eq!(v, [0.0, 1.0, 2.0]);
    }

    #[test]
    fn wall_push_is_two_sided_to_the_radius() {
        // SPEC-0018 Petpet vector: radius 1.0, x = 0.6 moves to 1.0; x = -0.6 moves to -1.0.
        let mut p = [0.6, 0.0, 5.0];
        assert!(resolve_walls(&mut p, &[wall()], 1.0));
        assert!((p[0] - 1.0).abs() < 1e-6);
        let mut p = [-0.6, 0.0, 5.0];
        resolve_walls(&mut p, &[wall()], 1.0);
        assert!((p[0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn wall_gates() {
        let w = wall();
        assert_eq!(w.push([1.2, 0.0, 5.0], 1.175), None, "beyond the radius");
        assert_eq!(w.push([0.5, 0.0, 10.5], 1.175), None, "past the run end");
        assert_eq!(w.push([0.5, 4.6, 5.0], 1.175), None, "top 4.75 is within 0.25 of the feet");
        assert_eq!(w.push([0.5, -3.6, 5.0], 1.175), None, "feet 3.5 or more below the bottom");
        assert!(w.push([0.5, -3.4, 5.0], 1.175).is_some());
        let mut flat = w;
        flat.face = [0.0, 1.0, 0.0, 0.0];
        assert_eq!(flat.push([0.0, 0.5, 5.0], 1.175), None, "near-horizontal face");
        assert!((w.edge_height([0.0, 0.0, 0.0]) - 4.75).abs() < 1e-6);
    }
}
