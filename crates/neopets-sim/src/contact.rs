//! Box push-out of a point (SPEC-0018, Box convention and Point versus box with radius r): a box
//! is a low corner o, a vertical axis V and two horizontal axes A and B with signed full extents.
//! A point inside the box grown by r is pushed out through the nearest face. The corner rule and
//! the preferred-face rule (Medium-Low in the spec) are not implemented.

use crate::hero::Vec3;

/// SPEC-0018 Constants: the contact push radius for box primitives.
pub const CONTACT_RADIUS: f32 = 1.25;
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

#[cfg(test)]
mod tests {
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
}
