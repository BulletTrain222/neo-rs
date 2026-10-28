//! Character shadow volumes (SPEC-0046 Character shadow volumes): the light direction from a
//! caster's light context (step 2a), the triangle facing (3), the extrusion by E = 16 (4), the
//! silhouette walls (5), the caps (6) and each polygon's count sign (7: +8 when the camera is on
//! its outward side, -8 otherwise). The resolve (step 8) leaves a surface darkened where the
//! signs of the polygons in front of it sum to a positive count; the host draws the two signs
//! as subtracting and adding passes against the scene depth, which gives that result for a
//! camera outside the volumes.
//!
//! Not built: the clip of the swept segment against the five culling planes (step 4: only a
//! unit whose sphere leaves the view is skipped there, and its near end is moved off the near
//! plane); here every unit is extruded from its own position (A = 0, B = 16 L).

pub type Vec3 = [f32; 3];

/// The extrusion length E (data word 0x2ECCC8), the light y limit (0x2EC9E4), the caster
/// range (50 units, squared 2500) and the caster count.
pub const EXTRUDE: f32 = 16.0;
pub const LIGHT_Y_LIMIT: f32 = -0.4;
pub const RANGE_SQ: f32 = 2500.0;
pub const CASTERS: usize = 3;

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn unit(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 0.0 { scale(a, 1.0 / l) } else { [0.0; 3] }
}

/// One light slot of a caster's context: its kind (2 directional, 3 point; others ignored),
/// weight, the entry direction (type 2) and the light position (type 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSlot {
    pub kind: i32,
    pub weight: f32,
    pub direction: Vec3,
    pub position: Vec3,
}

/// Step 2a: S = the sum of w D over the type-3 slots (D = normalize(caster - light)) plus w D of
/// the type-2 slot with the largest weight (the first on equal weights); L = normalize(S) with
/// its y held at or below -0.4 (then normalised again). A zero S gives (0, -1, 0); a missing
/// slot gives no direction (None); `straight_down` (data word 0x2ECCCC) gives (0, -1, 0).
pub fn light_direction(slots: &[Option<LightSlot>], caster: Vec3, straight_down: bool) -> Option<Vec3> {
    if straight_down {
        return Some([0.0, -1.0, 0.0]);
    }
    if slots.iter().any(|s| s.is_none()) {
        return None;
    }
    let slots: Vec<LightSlot> = slots.iter().flatten().copied().collect();
    let mut s = [0.0f32; 3];
    for l in slots.iter().filter(|l| l.kind == 3) {
        s = add(s, scale(unit(sub(caster, l.position)), l.weight));
    }
    let mut best: Option<LightSlot> = None;
    for l in slots.iter().filter(|l| l.kind == 2) {
        if best.is_none_or(|b| l.weight > b.weight) {
            best = Some(*l);
        }
    }
    if let Some(b) = best {
        s = add(s, scale(b.direction, b.weight));
    }
    let mut l = unit(s);
    if l == [0.0; 3] {
        return Some([0.0, -1.0, 0.0]);
    }
    if l[1] > LIGHT_Y_LIMIT {
        l[1] = LIGHT_Y_LIMIT;
        l = unit(l);
    }
    Some(l)
}

/// A hull in bone space (SPEC-0004 type 33).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hull {
    pub bank: u8,
    pub edges: Vec<[u8; 4]>,
    pub triangles: Vec<(Vec3, [u8; 3])>,
    pub points: Vec<Vec3>,
}

/// A bone frame: three axis columns and a translation (a point p goes to x p.x + y p.y + z p.z + t).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub cols: [Vec3; 3],
    pub t: Vec3,
}

impl Frame {
    fn point(&self, p: Vec3) -> Vec3 {
        add(add(add(scale(self.cols[0], p[0]), scale(self.cols[1], p[1])), scale(self.cols[2], p[2])), self.t)
    }
    fn normal(&self, n: Vec3) -> Vec3 {
        unit(add(add(scale(self.cols[0], n[0]), scale(self.cols[1], n[1])), scale(self.cols[2], n[2])))
    }
}

/// The volume polygons of one caster as triangles, split by count sign: `toward` (+8, the camera
/// on the outward side) and `away` (-8).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Volume {
    pub toward: Vec<[Vec3; 3]>,
    pub away: Vec<[Vec3; 3]>,
}

impl Volume {
    fn push(&mut self, tri: [Vec3; 3], outward: bool) {
        if outward { self.toward.push(tri) } else { self.away.push(tri) }
    }
}

/// Steps 2 to 7 for one caster: each hull moved by its bone frame (`frame(bank)`; a unit with
/// no frame or no geometry adds nothing), the facing flags (f = n . L >= 0 faces away), the
/// silhouette walls, the near and far caps and the count signs against the camera `eye`.
pub fn volume(hulls: &[Hull], frame: impl Fn(u8) -> Option<Frame>, l: Vec3, eye: Vec3, out: &mut Volume) {
    let b = scale(l, EXTRUDE);
    for h in hulls {
        if h.points.is_empty() || h.triangles.is_empty() {
            continue;
        }
        let Some(m) = frame(h.bank) else { continue };
        let near: Vec<Vec3> = h.points.iter().map(|&p| m.point(p)).collect();
        let far: Vec<Vec3> = near.iter().map(|&p| add(p, b)).collect();
        let normals: Vec<Vec3> = h.triangles.iter().map(|t| m.normal(t.0)).collect();
        let away: Vec<bool> = normals.iter().map(|&n| dot(n, l) >= 0.0).collect();
        let at = |list: &Vec<Vec3>, i: u8| list.get(i as usize).copied();
        // Caps: a triangle facing the light at its near copies, the others at their far copies.
        for (k, t) in h.triangles.iter().enumerate() {
            let src = if away[k] { &far } else { &near };
            let (Some(a), Some(b2), Some(c)) = (at(src, t.1[0]), at(src, t.1[1]), at(src, t.1[2])) else { continue };
            let p0 = at(&near, t.1[0]).unwrap_or(a);
            out.push([a, b2, c], dot(sub(eye, p0), normals[k]) >= 0.0);
        }
        // Walls on the silhouette edges.
        for e in &h.edges {
            let (t0, t1) = (e[0] as usize, e[1] as usize);
            let (Some(&f0), Some(&f1)) = (away.get(t0), away.get(t1)) else { continue };
            if f0 == f1 {
                continue;
            }
            let (Some(pa), Some(pb)) = (at(&near, e[2]), at(&near, e[3])) else { continue };
            let edge = if !f0 { sub(pa, pb) } else { sub(pb, pa) };
            let n = cross(edge, l);
            let outward = dot(sub(eye, pa), n) >= 0.0;
            let (qa, qb) = (add(pa, b), add(pb, b));
            out.push([pa, pb, qb], outward);
            out.push([pa, qb, qa], outward);
        }
    }
}

/// Step 1: the casters to take, by 3D squared distance from the view point: those below 2500,
/// the nearest three, a tie with the farthest held one not replacing it. Returns indices into
/// `distances_sq`, in holding order.
pub fn choose(distances_sq: &[f32]) -> Vec<usize> {
    let mut held: Vec<usize> = Vec::new();
    for (i, &d) in distances_sq.iter().enumerate() {
        if d >= RANGE_SQ {
            continue;
        }
        if held.len() < CASTERS {
            held.push(i);
            continue;
        }
        let far = (0..held.len()).filter(|&k| distances_sq[held[k]] > d).max_by(|&a, &b| distances_sq[held[a]].total_cmp(&distances_sq[held[b]]));
        if let Some(k) = far {
            held[k] = i;
        }
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit cube hull (points 0..7), with outward normals and its 18 edges between the 12
    /// triangles.
    fn cube() -> Hull {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]];
        let faces: [(Vec3, [u8; 4]); 6] = [
            ([0.0, 0.0, -1.0], [0, 3, 2, 1]),
            ([0.0, 0.0, 1.0], [4, 5, 6, 7]),
            ([0.0, -1.0, 0.0], [0, 1, 5, 4]),
            ([0.0, 1.0, 0.0], [3, 7, 6, 2]),
            ([-1.0, 0.0, 0.0], [0, 4, 7, 3]),
            ([1.0, 0.0, 0.0], [1, 2, 6, 5]),
        ];
        let mut triangles = Vec::new();
        for (n, q) in faces {
            triangles.push((n, [q[0], q[1], q[2]]));
            triangles.push((n, [q[0], q[2], q[3]]));
        }
        // Every pair of triangles sharing two points is an edge.
        let mut edges = Vec::new();
        for i in 0..triangles.len() {
            for j in i + 1..triangles.len() {
                let shared: Vec<u8> = triangles[i].1.iter().copied().filter(|p| triangles[j].1.contains(p)).collect();
                if shared.len() == 2 {
                    edges.push([i as u8, j as u8, shared[0], shared[1]]);
                }
            }
        }
        Hull { bank: 0, edges, triangles, points }
    }

    #[test]
    fn light_direction_rules() {
        let slot = |kind, weight, direction, position| Some(LightSlot { kind, weight, direction, position });
        // One directional slot pointing sideways: the y is held at -0.4.
        let l = light_direction(&[slot(2, 1.0, [1.0, 0.0, 0.0], [0.0; 3]), slot(1, 1.0, [0.0; 3], [0.0; 3]), slot(1, 0.0, [0.0; 3], [0.0; 3]), slot(1, 0.0, [0.0; 3], [0.0; 3])], [0.0; 3], false).unwrap();
        assert!((l[1] - (-0.4f32 / (1.0f32 + 0.16).sqrt())).abs() < 1e-5, "{l:?}");
        // A point light straight above: (0, -1, 0). No light at all: straight down too.
        let l = light_direction(&[slot(3, 1.0, [0.0; 3], [0.0, 10.0, 0.0]), slot(0, 1.0, [0.0; 3], [0.0; 3]), slot(0, 1.0, [0.0; 3], [0.0; 3]), slot(0, 1.0, [0.0; 3], [0.0; 3])], [0.0; 3], false).unwrap();
        assert_eq!(l, [0.0, -1.0, 0.0]);
        assert_eq!(light_direction(&[slot(0, 1.0, [0.0; 3], [0.0; 3])], [0.0; 3], false), Some([0.0, -1.0, 0.0]));
        // A missing slot: no direction.
        assert_eq!(light_direction(&[None, slot(2, 1.0, [1.0, 0.0, 0.0], [0.0; 3])], [0.0; 3], false), None);
    }

    #[test]
    fn cube_volume_straight_down() {
        let mut v = Volume::default();
        let frame = |_| Some(Frame { cols: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], t: [0.0, 5.0, 0.0] });
        // Light straight down, camera above and to the side.
        volume(&[cube()], frame, [0.0, -1.0, 0.0], [10.0, 20.0, 0.5], &mut v);
        let all: Vec<[Vec3; 3]> = v.toward.iter().chain(&v.away).copied().collect();
        // 12 cap triangles and 4 silhouette edges x 2 wall triangles: with the light straight down
        // the sides (n . L = 0) count as facing away, so the silhouette is the top face's rim.
        assert_eq!(all.len(), 12 + 4 * 2, "{} toward {} away", v.toward.len(), v.away.len());
        // The far cap sits 16 below the bottom face.
        assert!(all.iter().flatten().any(|p| (p[1] - (5.0 - 16.0)).abs() < 1e-5));
        // The top (near) cap faces the camera; the far cap (the bottom face extruded) does not.
        let top = v.toward.iter().any(|t| t.iter().all(|p| (p[1] - 6.0).abs() < 1e-5));
        assert!(top);
    }

    #[test]
    fn caster_choice() {
        // Within 50 units only; the nearest three; a tie with the farthest held one is dropped.
        // 100, 400 and 900 are held; 50 replaces the farthest (900); 3000 is out of range.
        assert_eq!(choose(&[100.0, 3000.0, 400.0, 900.0, 50.0]), vec![0, 2, 4]);
        assert_eq!(choose(&[100.0, 100.0, 100.0, 100.0]), vec![0, 1, 2]);
    }
}
