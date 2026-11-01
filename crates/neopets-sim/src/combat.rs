//! Combat geometry (SPEC-0036 Hit detection, Combat reach test; SPEC-0040 Behavior 12A): flat-
//! ended finite cylinders (bone hurt capsules, weapon volumes), their static overlap, and the
//! swept test of a moving weapon against a capsule.
//! The static cylinder-cylinder overlap and the segment-versus-cylinder test are named "the
//! finite-cylinder test of SPEC-0018" but not described there (R-0014); this build uses the
//! closest distance between the two axis segments (and between a segment and the axis segment)
//! against the radii, which treats the ends as rounded.

use crate::hero::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylinder {
    pub center: Vec3,
    /// Unit axis.
    pub axis: Vec3,
    pub radius: f32,
    /// Full length.
    pub length: f32,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Vec3, s: f32) -> Vec3 {
    a.map(|c| c * s)
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn len(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}

fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}

impl Cylinder {
    /// A capsule posed from a bone: from P along Y for the length (SPEC-0036 Hurt shapes).
    pub fn from_base(base: Vec3, axis: Vec3, radius: f32, length: f32) -> Self {
        Self { center: add(base, scale(axis, 0.5 * length)), axis, radius, length }
    }

    fn ends(&self) -> (Vec3, Vec3) {
        let h = scale(self.axis, 0.5 * self.length);
        (sub(self.center, h), add(self.center, h))
    }
}

/// Closest distance between segments p0–p1 and q0–q1.
pub fn segment_distance(p0: Vec3, p1: Vec3, q0: Vec3, q1: Vec3) -> f32 {
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
            let denom = a * e - b * b;
            let mut s = if denom > 1e-12 { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
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
    len(sub(add(p0, scale(d1, s)), add(q0, scale(d2, t))))
}

/// Static overlap of two finite cylinders (approximation; R-0014).
pub fn overlap(a: &Cylinder, b: &Cylinder) -> bool {
    let (a0, a1) = a.ends();
    let (b0, b1) = b.ends();
    segment_distance(a0, a1, b0, b1) <= a.radius + b.radius
}

/// The line against cylinder routine of SPEC-0018 (Finite cylinder tests): the parameters at
/// which the line p + t D (t along D normalised in the cylinder's frame) meets the flat-ended
/// cylinder c, at most two, with the axis used as stored.
fn line_cylinder(p: Vec3, d: Vec3, c: &Cylinder) -> Vec<f32> {
    let a = c.axis;
    let u = if a[1].abs() <= a[0].abs() {
        let k = (a[0] * a[0] + a[2] * a[2]).sqrt();
        if k == 0.0 {
            return Vec::new();
        }
        [-a[2] / k, 0.0, a[0] / k]
    } else {
        let k = (a[1] * a[1] + a[2] * a[2]).sqrt();
        if k == 0.0 {
            return Vec::new();
        }
        [0.0, a[2] / k, -a[1] / k]
    };
    let v = cross(a, u);
    let q = sub(p, c.center);
    let (x0, y0, z0) = (dot(u, q), dot(v, q), dot(a, q));
    let dc = [dot(u, d), dot(v, d), dot(a, d)];
    let dl = len(dc);
    if dl == 0.0 {
        return Vec::new();
    }
    let (dx, dy, dz) = (dc[0] / dl, dc[1] / dl, dc[2] / dl);
    let (r, h2) = (c.radius, 0.5 * c.length);
    let circle = || -> Vec<f32> {
        let a2 = dx * dx + dy * dy;
        let b = x0 * dx + y0 * dy;
        let cc = x0 * x0 + y0 * y0 - r * r;
        let disc = b * b - a2 * cc;
        if a2 == 0.0 || disc < 0.0 {
            Vec::new()
        } else if disc == 0.0 {
            vec![-b / a2]
        } else {
            let s = disc.sqrt();
            vec![(-b - s) / a2, (-b + s) / a2]
        }
    };
    if dz.abs() >= 0.99999 {
        if x0 * x0 + y0 * y0 > r * r {
            return Vec::new();
        }
        return vec![(h2 - z0) / dz, (-h2 - z0) / dz];
    }
    if dz.abs() <= 0.00001 {
        return if z0.abs() > h2 { Vec::new() } else { circle() };
    }
    let caps = [(h2 - z0) / dz, (-h2 - z0) / dz];
    let inside = |t: f32| (x0 + t * dx).powi(2) + (y0 + t * dy).powi(2) <= r * r;
    let kept: Vec<f32> = caps.iter().copied().filter(|&t| inside(t)).collect();
    if kept.len() == 2 {
        return kept;
    }
    let walls = circle();
    if walls.is_empty() {
        return Vec::new();
    }
    let (lo, hi) = (caps[0].min(caps[1]), caps[0].max(caps[1]));
    let mut out = kept;
    out.extend(walls.into_iter().filter(|&t| t >= lo && t <= hi));
    out.truncate(2);
    out
}

/// The static overlap routine of SPEC-0018 (Finite cylinder tests): shape A a flat-ended
/// cylinder; when the axes are nearly parallel the radii and axial intervals decide; otherwise B
/// is the card through its axis that faces A's centre, and the crossings of its two end edges
/// and its far long edge decide. Axes are used as stored. In the monster weapon strike A is the
/// weapon and B each hurt capsule of the target.
pub fn finite_overlap(a: &Cylinder, b: &Cylinder) -> bool {
    let d = sub(a.center, b.center);
    let reach = |c: &Cylinder| (c.radius * c.radius + 0.25 * c.length * c.length).sqrt();
    if len(d) > reach(a) + reach(b) {
        return false;
    }
    let ab = b.axis;
    if len(ab) == 0.0 {
        return false;
    }
    let bu = unit(ab);
    if dot(a.axis, ab).abs() > 0.99 {
        let along = dot(d, ab);
        let lateral = len(sub(d, scale(bu, dot(d, bu))));
        return along.abs() <= 0.5 * (a.length + b.length) && lateral <= a.radius + b.radius;
    }
    let perp = sub(d, scale(bu, dot(d, bu)));
    if len(perp) == 0.0 {
        return false;
    }
    let n = unit(perp);
    let half = scale(ab, 0.5 * b.length);
    // T and U: the end edges at +h/2 and -h/2, from the -n corner toward +n; S: the long edge on
    // the +n side from the -h/2 end toward +h/2.
    let t_start = sub(add(b.center, half), scale(n, b.radius));
    let u_start = sub(sub(b.center, half), scale(n, b.radius));
    let s_start = add(sub(b.center, half), scale(n, b.radius));
    let crosses = |start: Vec3, dir: Vec3, length: f32| -> (bool, Vec<f32>) {
        let roots = line_cylinder(start, dir, a);
        (roots.iter().any(|&t| t > 0.0 && t < length), roots)
    };
    let (tc, _) = crosses(t_start, n, 2.0 * b.radius);
    let (uc, _) = crosses(u_start, n, 2.0 * b.radius);
    let (sc, s_roots) = crosses(s_start, ab, b.length);
    if tc && uc {
        return true;
    }
    if sc && (tc || uc) {
        return true;
    }
    if tc || uc {
        // S starts inside A: two roots, one negative and one positive.
        return s_roots.len() == 2 && s_roots.iter().any(|&t| t < 0.0) && s_roots.iter().any(|&t| t > 0.0);
    }
    if sc {
        if !(s_roots.len() == 2 && s_roots.iter().all(|&t| t > 0.0 && t < b.length)) {
            return false;
        }
        let au = unit(a.axis);
        let far = |t: f32| dot(sub(add(s_start, scale(bu, t)), a.center), au).abs() > 0.99 * 0.5 * a.length;
        return !(far(s_roots[0]) && far(s_roots[1]));
    }
    // No edge crosses: one shape wholly inside the other (strictly smaller in both sizes), each
    // of four rays from the inner centre meeting the outer at a positive parameter.
    let rays_meet = |from: Vec3, axis: Vec3, side: Vec3, outer: &Cylinder| {
        [axis, scale(axis, -1.0), side, scale(side, -1.0)].iter().all(|&dir| line_cylinder(from, dir, outer).iter().any(|&t| t > 0.0))
    };
    if b.radius < a.radius && b.length < a.length {
        return rays_meet(b.center, ab, n, a);
    }
    if a.radius < b.radius && a.length < b.length {
        let dm = sub(b.center, a.center);
        let au = unit(a.axis);
        let mp = sub(dm, scale(au, dot(dm, au)));
        if len(mp) == 0.0 {
            return false;
        }
        return rays_meet(a.center, a.axis, unit(mp), b);
    }
    false
}

/// A segment meets a finite cylinder (approximation; R-0014).
pub fn segment_meets(s0: Vec3, s1: Vec3, c: &Cylinder) -> bool {
    let (c0, c1) = c.ends();
    segment_distance(s0, s1, c0, c1) <= c.radius
}

/// SPEC-0036 Combat reach test: the swept triangle A, B, C (weapon base, tip now, tip last
/// tick) against the capsule (centre p, axis b, radius r, length h), with the plane slab, the
/// shifted point quirk, the inside test, the edge gates and the edge cylinder test.
pub fn swept(a: Vec3, b_tip: Vec3, c_tip: Vec3, cap: &Cylinder) -> bool {
    let (p, b, r, h) = (cap.center, cap.axis, cap.radius, cap.length);
    // The normal's sign matters only for the c = 0 shift; SPEC-0036's vectors fix it as
    // (C − A) × (B − A) in this build's right-handed cross product.
    let n = unit(cross(sub(c_tip, a), sub(b_tip, a)));
    if len(n) == 0.0 {
        return false;
    }
    let c = dot(n, b).abs();
    let s = (1.0 - c * c).abs().sqrt();
    let dist = dot(n, sub(p, a));
    let slab = r * s + 0.5 * h * c;
    if dist.abs() > slab {
        return false;
    }
    let d = 0.4 * (r * c + h * s);
    let q = if c == 0.0 && dist > 0.0 { add(p, scale(b, dist / slab)) } else { sub(p, scale(b, dist.abs() / slab)) };
    let tri = [a, b_tip, c_tip];
    // Inside test: q projected on the plane lies inside the triangle.
    let qp = sub(q, scale(n, dot(n, sub(q, a))));
    let side = |i: usize| dot(n, cross(sub(tri[(i + 1) % 3], tri[i]), sub(qp, tri[i])));
    let signs = [side(0), side(1), side(2)];
    if signs.iter().all(|&v| v >= 0.0) || signs.iter().all(|&v| v <= 0.0) {
        return true;
    }
    // Edge gates with in-plane outward normals.
    let centroid = scale(add(add(a, b_tip), c_tip), 1.0 / 3.0);
    for i in 0..3 {
        let (v0, v1) = (tri[i], tri[(i + 1) % 3]);
        let e = unit(sub(v1, v0));
        let l = len(sub(v1, v0));
        let mut m = unit(cross(e, n));
        if dot(m, sub(centroid, v0)) > 0.0 {
            m = scale(m, -1.0);
        }
        if dot(m, sub(q, v0)) > d {
            return false;
        }
        let along = dot(e, sub(p, v0));
        if !(-d < along && along < l + d) {
            return false;
        }
    }
    (0..3).any(|i| segment_meets(tri[i], tri[(i + 1) % 3], cap))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cyl(r: f32, h: f32, c: Vec3, a: Vec3) -> Cylinder {
        Cylinder { center: c, axis: a, radius: r, length: h }
    }

    #[test]
    fn finite_cylinder_vectors() {
        // SPEC-0018 Finite cylinder tests, the synthetic vectors (A first, then B).
        let y = [0.0, 1.0, 0.0];
        let b = cyl(0.5, 2.0, [0.0, 1.0, 0.0], y);
        assert!(finite_overlap(&cyl(1.0, 3.5, [1.2, 1.0, 0.0], y), &b), "parallel, lateral 1.2");
        assert!(!finite_overlap(&cyl(1.0, 3.5, [1.6, 1.0, 0.0], y), &b));
        assert!(!finite_overlap(&cyl(1.0, 3.5, [0.0, 4.0, 0.0], y), &b), "axial 3 > 2.75");
        assert!(finite_overlap(&cyl(1.0, 3.5, [0.0, 2.7, 0.0], y), &b));
        let b = cyl(0.5, 1.0, [0.0, 0.5, 0.0], y);
        let x = [1.0, 0.0, 0.0];
        assert!(finite_overlap(&cyl(2.0, 1.0, [0.9, 0.5, 0.0], x), &b), "both end edges cross");
        assert!(!finite_overlap(&cyl(2.0, 1.0, [1.1, 0.5, 0.0], x), &b));
        assert!(!finite_overlap(&cyl(2.0, 2.0, [0.9, 2.0, 0.5], [0.0, 0.0, 1.0]), &b), "solid overlap that still misses");
        assert!(finite_overlap(&cyl(3.0, 1.0, [1.2, 0.0, 0.0], [0.0, 0.0, 1.0]), &cyl(0.5, 0.9, [0.0, 0.5, 0.0], y)), "wholly inside");
        assert!(!finite_overlap(&cyl(3.0, 1.0, [1.2, 0.0, 0.0], [0.0, 0.0, 1.0]), &cyl(0.5, 1.0, [0.0, 0.5, 0.0], y)));
        assert!(!finite_overlap(&cyl(2.0, 1.0, [1.02, 0.5, 0.0], x), &b), "axis length 1.0");
        assert!(finite_overlap(&cyl(2.0, 1.0, [1.02, 0.5, 0.0], [0.95, 0.0, 0.0]), &b), "axis length 0.95");
        // Flat ends: centres 5 apart on a shared axis, half-heights summing to 4.
        assert!(!finite_overlap(&cyl(1.0, 4.0, [0.0, 2.0, 0.0], y), &cyl(1.0, 4.0, [0.0, 7.0, 0.0], y)));
    }

    const A: Vec3 = [0.0, 0.0, 0.0];
    const B: Vec3 = [2.0, 0.0, 0.0];
    const C: Vec3 = [0.0, 0.0, 2.0];

    fn cap(p: Vec3, b: Vec3, r: f32, h: f32) -> Cylinder {
        Cylinder { center: p, axis: b, radius: r, length: h }
    }

    #[test]
    fn swept_test_vectors() {
        // SPEC-0036 Combat reach test vectors.
        let y = [0.0, 1.0, 0.0];
        assert!(swept(A, B, C, &cap([0.5, 0.49, 0.5], y, 0.3, 1.0)));
        assert!(!swept(A, B, C, &cap([0.5, 0.51, 0.5], y, 0.3, 1.0)));
        assert!(swept(A, B, C, &cap([-0.1, 0.0, 1.0], y, 0.3, 1.0)));
        assert!(!swept(A, B, C, &cap([-0.2, 0.0, 1.0], y, 0.3, 1.0)), "the edge gate fails");
        let x = [1.0, 0.0, 0.0];
        assert!(swept(A, B, C, &cap([0.5, 0.29, 0.5], x, 0.3, 1.0)));
        assert!(!swept(A, B, C, &cap([0.5, 0.31, 0.5], x, 0.3, 1.0)));
        let t = [0.6, 0.8, 0.0];
        assert!(swept(A, B, C, &cap([0.5, 0.35, 0.5], t, 0.5, 1.0)));
        assert!(!swept(A, B, C, &cap([-0.3, 0.35, 1.0], t, 0.5, 1.0)));
        assert!(!swept(A, B, C, &cap([-0.3, -0.35, 1.0], t, 0.5, 1.0)));
        assert!(swept(A, B, C, &cap([2.3, 0.35, 0.2], t, 0.5, 1.0)));
    }

    #[test]
    fn capsule_from_a_bone_base() {
        // SPEC-0005: base (1, 2, 3), length 0.8 along Y: from (1, 2, 3) to (1, 2.8, 3).
        let c = Cylinder::from_base([1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 0.3, 0.8);
        assert!((c.center[1] - 2.4).abs() < 1e-6);
        let other = Cylinder { center: [1.5, 2.4, 3.0], axis: [0.0, 1.0, 0.0], radius: 0.25, length: 1.0 };
        assert!(overlap(&c, &other));
        let far = Cylinder { center: [1.6, 2.4, 3.0], ..other };
        assert!(!overlap(&c, &far));
    }
}
