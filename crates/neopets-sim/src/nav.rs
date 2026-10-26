//! Navigation meshes at run time (SPEC-0018 Navmesh queries; SPEC-0056 Route step, Route table;
//! SPEC-0029 Behavior 2b nearest triangle): the inside test, the hinted and full locates, the
//! route table read, and the route step's steering point.

use crate::hero::Vec3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavMesh {
    pub kind: u32,
    pub triangles: Vec<[Vec3; 3]>,
    pub neighbours: Vec<[i32; 3]>,
    pub table: Vec<u8>,
}

/// GetClosestNavMeshId (SPEC-0030 row 122): among the meshes of the kind (2 for a flyer, else 1),
/// each holding the point's (x, z) in some triangle, the one whose first holding triangle's
/// vertex-0 height is nearest the point's height (a later mesh only when strictly nearer); -1
/// when none holds it.
pub fn closest(meshes: &[NavMesh], p: Vec3, kind: u32) -> i32 {
    let mut best: Option<(usize, f32)> = None;
    for (i, m) in meshes.iter().enumerate().filter(|(_, m)| m.kind == kind) {
        if let Some(t) = m.first_holding(p) {
            let d = (m.triangles[t][0][1] - p[1]).abs();
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((i, d));
            }
        }
    }
    best.map_or(-1, |(i, _)| i as i32)
}

fn cross2(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (b[0] - a[0]) * (p[2] - a[2]) - (b[2] - a[2]) * (p[0] - a[0])
}

impl NavMesh {
    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Inside test in the (x, z) plane: on the inner side of all three edges for the mesh's
    /// winding ((x1−x0)(z2−z0) − (z1−z0)(x2−x0) > 0); points on an edge count as inside.
    pub fn contains(&self, t: usize, p: Vec3) -> bool {
        let Some(&[a, b, c]) = self.triangles.get(t) else { return false };
        cross2(a, b, p) >= 0.0 && cross2(b, c, p) >= 0.0 && cross2(c, a, p) >= 0.0
    }

    /// Locate with a last triangle (hint), SPEC-0018: the hint, its neighbours in edge order,
    /// then outward from the hint (−1, +1, −2, +2, ...).
    pub fn locate(&self, hint: Option<usize>, p: Vec3) -> Option<usize> {
        let n = self.len();
        let Some(h) = hint.filter(|&h| h < n) else { return self.locate_from_middle(p) };
        if self.contains(h, p) {
            return Some(h);
        }
        for &nb in &self.neighbours[h] {
            if nb >= 0 && self.contains(nb as usize, p) {
                return Some(nb as usize);
            }
        }
        for k in 1..n {
            let below = h.checked_sub(k);
            let above = (h + k < n).then_some(h + k);
            if below.is_none() && above.is_none() {
                break;
            }
            if let Some(t) = below.filter(|&t| self.contains(t, p)) {
                return Some(t);
            }
            if let Some(t) = above.filter(|&t| self.contains(t, p)) {
                return Some(t);
            }
        }
        None
    }

    /// Locate with no hint: outward from floor(N/2), the middle itself never tested.
    fn locate_from_middle(&self, p: Vec3) -> Option<usize> {
        let n = self.len();
        let m = n / 2;
        for k in 1..=n {
            let below = m.checked_sub(k);
            let above = (m + k < n).then_some(m + k);
            if below.is_none() && above.is_none() {
                break;
            }
            if let Some(t) = below.filter(|&t| self.contains(t, p)) {
                return Some(t);
            }
            if let Some(t) = above.filter(|&t| self.contains(t, p)) {
                return Some(t);
            }
        }
        None
    }

    /// A full-scan locate: the first triangle containing p.
    pub fn locate_full(&self, p: Vec3) -> Option<usize> {
        (0..self.len()).find(|&t| self.contains(t, p))
    }

    /// The first triangle in array order holding (x, z), edges inside (GetClosestNavMeshId).
    pub fn first_holding(&self, p: Vec3) -> Option<usize> {
        self.locate_full(p)
    }

    /// The far centroid pick (SPEC-0056): up to T uniform triangle draws, the first centroid
    /// more than `d` horizontally from `p`, else the last one drawn; None for an empty mesh.
    pub fn far_centroid(&self, p: Vec3, d: f32, rng: &mut crate::rng::Rng) -> Option<Vec3> {
        let mut last = None;
        for _ in 0..self.len() {
            let c = self.centroid(rng.index(self.len() as u32) as usize);
            last = Some(c);
            if (c[0] - p[0]).hypot(c[2] - p[2]) > d {
                return Some(c);
            }
        }
        last
    }

    pub fn centroid(&self, t: usize) -> Vec3 {
        let [a, b, c] = self.triangles[t];
        std::array::from_fn(|k| (a[k] + b[k] + c[k]) / 3.0)
    }

    /// SPEC-0029 Behavior 2b nearest triangle: smallest 3-D distance from its centroid to q,
    /// earlier triangle on a tie. (The occupancy quirk, which passes or fails every triangle of
    /// one search alike, is not modelled: every triangle passes.)
    pub fn nearest(&self, q: Vec3) -> Option<usize> {
        let mut best = (1.0e9f32, None);
        for t in 0..self.len() {
            let c = self.centroid(t);
            let d = ((c[0] - q[0]).powi(2) + (c[1] - q[1]).powi(2) + (c[2] - q[2]).powi(2)).sqrt();
            if d < best.0 {
                best = (d, Some(t));
            }
        }
        best.1
    }

    /// The route table entry: the edge of a to leave by toward b.
    pub fn route(&self, a: usize, b: usize) -> Option<usize> {
        let t = self.len();
        if a >= t || b >= t {
            return None;
        }
        let k = 2 * (a * t + b);
        Some(((self.table.get(k / 8)? >> (k % 8)) & 3) as usize)
    }

    /// SPEC-0056 Route step: the steering point K on edge e of triangle c for a character at P
    /// with goal G and clearance m (motion-record radius + 0.25).
    pub fn route_point(&self, c: usize, e: usize, p: Vec3, g: Vec3, m: f32) -> Vec3 {
        let tri = self.triangles[c];
        let (a, b, cc) = (tri[e % 3], tri[(e + 1) % 3], tri[(e + 2) % 3]);
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let mid: Vec3 = std::array::from_fn(|k| (a[k] + b[k]) / 2.0);
        let dist3 = ((g[0] - p[0]).powi(2) + (g[1] - p[1]).powi(2) + (g[2] - p[2]).powi(2)).sqrt();
        let mut k = mid;
        if dist3 < 100.0 {
            let (dx, dz) = (g[0] - p[0], g[2] - p[2]);
            let s = if dx == 0.0 {
                (p[0] - a[0]) / d[0]
            } else {
                let q = dz / dx;
                ((p[2] - a[2]) + q * a[0] - q * p[0]) / (d[2] - q * d[0])
            };
            // Clearance clamp: distances of B from line CA and of A from line CB.
            let line_dist = |from: Vec3, to: Vec3, pt: Vec3| {
                let (lx, lz) = (to[0] - from[0], to[2] - from[2]);
                let len = (lx * lx + lz * lz).sqrt();
                if len == 0.0 { 0.0 } else { (lx * (pt[2] - from[2]) - lz * (pt[0] - from[0])).abs() / len }
            };
            let (k1, k2) = (line_dist(cc, a, b), line_dist(cc, b, a));
            if k1 > 0.0 && k2 > 0.0 && s.is_finite() {
                let lo = (m / k1).min(0.5);
                let hi = (1.0 - m / k2).max(0.5);
                let s = s.clamp(lo, hi);
                k = std::array::from_fn(|i| a[i] + s * d[i]);
            }
        }
        // Behind test: K behind the direction to G goes back to the midpoint.
        let unit = |v: [f32; 2]| {
            let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
            if l > 0.0 { [v[0] / l, v[1] / l] } else { v }
        };
        let (ug, uk) = (unit([g[0] - p[0], g[2] - p[2]]), unit([k[0] - p[0], k[2] - p[2]]));
        if ug[0] * uk[0] + ug[1] * uk[1] < 0.0 {
            k = mid;
        }
        k
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn closest_mesh() {
        // SPEC-0030's vector: meshes 0 and 1 (kind 1) both hold (0.2, 5.0, 0.2); heights 4.0 and
        // 6.5 give 0; at 6.0 the tie keeps 0; a flyer tests kind 2 only and gets -1.
        let p = [0.2, 5.0, 0.2];
        let wind = |y: f32, order: bool| {
            let t = if order { [[0.0, y, 0.0], [0.0, y, 1.0], [1.0, y, 0.0]] } else { [[0.0, y, 0.0], [1.0, y, 0.0], [0.0, y, 1.0]] };
            NavMesh { kind: 1, triangles: vec![t], neighbours: vec![[-1; 3]], table: Vec::new() }
        };
        let order = wind(4.0, true).contains(0, p);
        let mk = |y: f32| wind(y, order);
        assert!(mk(4.0).contains(0, p));
        assert_eq!(closest(&[mk(4.0), mk(6.5)], p, 1), 0);
        assert_eq!(closest(&[mk(4.0), mk(6.0)], p, 1), 0);
        assert_eq!(closest(&[mk(6.5), mk(4.0)], p, 1), 1);
        assert_eq!(closest(&[mk(4.0), mk(6.5)], p, 2), -1);
        assert_eq!(closest(&[mk(4.0)], [3.0, 5.0, 3.0], 1), -1);
    }

    use super::*;

    /// Two triangles forming the square x, z in [0, 10] (positive winding), sharing the diagonal.
    fn square() -> NavMesh {
        NavMesh {
            kind: 1,
            // Triangle 0 leaves by edge 2 (the diagonal) toward 1; triangle 1 by edge 0.
            triangles: vec![
                [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 0.0, 10.0]],
                [[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 0.0, 10.0]],
            ],
            neighbours: vec![[-1, -1, 1], [0, -1, -1]],
            table: vec![0b0000_1000],
        }
    }

    #[test]
    fn winding_and_locates() {
        let m = square();
        // Both triangles have the winding the spec requires.
        for [a, b, c] in &m.triangles {
            assert!((b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0]) > 0.0);
        }
        assert_eq!(m.locate(Some(0), [8.0, 0.0, 2.0]), Some(0));
        assert_eq!(m.locate(Some(0), [2.0, 0.0, 8.0]), Some(1), "neighbour across edge 2");
        assert_eq!((m.route(0, 1), m.route(1, 0)), (Some(2), Some(0)));
        assert_eq!(m.locate_full([20.0, 0.0, 2.0]), None);
        assert_eq!(m.nearest([9.0, 0.0, 1.0]), Some(0));
    }

    #[test]
    fn route_point_crosses_the_shared_edge_toward_the_goal() {
        let m = square();
        // From (8, 0, 2) in triangle 0 toward (2, 0, 8) in triangle 1, leaving by edge 2 (the
        // diagonal): the line from P to G crosses it at (5, 0, 5).
        let k = m.route_point(0, 2, [8.0, 0.0, 2.0], [2.0, 0.0, 8.0], 1.425);
        assert!((k[0] - 5.0).abs() < 1e-4 && (k[2] - 5.0).abs() < 1e-4, "{k:?}");
        // A goal 100 or more away steers at the midpoint.
        let k = m.route_point(0, 2, [8.0, 0.0, 2.0], [-200.0, 0.0, 300.0], 1.425);
        assert_eq!(k, [5.0, 0.0, 5.0]);
    }
}
