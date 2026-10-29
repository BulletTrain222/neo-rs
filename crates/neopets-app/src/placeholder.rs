//! Stand-in models for characters the disc has no model for. These are this build's own
//! placeholder art (procedural ellipsoids with vertex colours), not recovered assets.
//!
//! The Cloud Beast (SPEC-0010, Placement and reachability: no model, skeleton or clip bank on
//! the disc): the Cloudbreaker, built after the user's own concept sheet (faceted lavender cloud
//! puffs, swirled shoulders, big fists, glowing eyes and cracks, a wisp tail), and its
//! companion (row 2), a small horned storm puff that rides on its crown.

use neopets_data::mesh::Layout;
use neopets_data::model::{MeshPart, ModelData};

/// How high the stand-in body floats above its ground point (the wisp tail hangs below it).
pub const CLOUD_HOVER: f32 = 2.0;
/// Where the companion rides, above the boss's ground point: on the Cloudbreaker's crown (the
/// rig contract's bone 11 is not recoverable; this is the stand-in's choice).
pub const COMPANION_RIDE: f32 = CLOUD_HOVER + 4.9;

fn part() -> MeshPart {
    MeshPart {
        tree: 0,
        leaf_offset: 0,
        layout: Layout::Plain,
        group: None,
        program: None,
        colour_slot: 0,
        zone: None,
        scroll: None,
        env_map: false,
        flipbook: None,
        positions: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        colors: Vec::new(),
        raw_colors: Vec::new(),
        lit: false,
        indices: Vec::new(),
        texture: None,
        weights: Vec::new(),
        bones: Vec::new(),
        pass1: None,
        cull: None,
    }
}

type V = [f32; 3];

fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: V, s: f32) -> V {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn unit(a: V) -> V {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-6);
    mul(a, 1.0 / l)
}

/// A small fixed hash for the facet colour jitter (the same model every build).
fn jitter(i: u32) -> f32 {
    let x = i.wrapping_mul(2_654_435_761).rotate_left(13) ^ 0x9E37_79B9;
    (x % 1000) as f32 / 1000.0 - 0.5
}

/// One flat-shaded triangle: its own three vertices, the face normal and one colour.
fn tri(p: &mut MeshPart, a: V, b: V, c: V, col: V) {
    let n = unit(cross(sub(b, a), sub(c, a)));
    let base = p.positions.len() as u32;
    for q in [a, b, c] {
        p.positions.push(q);
        p.normals.push(n);
        p.uvs.push([0.0, 0.0]);
        p.colors.push([col[0], col[1], col[2], 1.0]);
        p.raw_colors.push([128; 4]);
    }
    p.indices.extend([base, base + 1, base + 2]);
}

/// A faceted ellipsoid (the low-poly cloud puff): centre `c`, radii `r` along the basis `axes`
/// (x, y, z rows), coloured from `top` (facets facing up) to `bottom` with a slight jitter per
/// facet. Colours are in the GS's units (an untextured part draws at half), so 2.0 is white.
#[allow(clippy::too_many_arguments)]
fn puff(p: &mut MeshPart, c: V, r: V, axes: [V; 3], top: V, bottom: V, rings: u32, segs: u32) {
    let point = |i: u32, j: u32| {
        let th = i as f32 / rings as f32 * std::f32::consts::PI;
        let ph = j as f32 / segs as f32 * std::f32::consts::TAU;
        let n = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
        let l = [r[0] * n[0], r[1] * n[1], r[2] * n[2]];
        add(c, add(add(mul(axes[0], l[0]), mul(axes[1], l[1])), mul(axes[2], l[2])))
    };
    let mut k = p.positions.len() as u32;
    for i in 0..rings {
        for j in 0..segs {
            let (a, b, cc, d) = (point(i, j), point(i + 1, j), point(i + 1, j + 1), point(i, j + 1));
            for (x, y, z) in [(a, b, d), (d, b, cc)] {
                let n = unit(cross(sub(y, x), sub(z, x)));
                if n == [0.0; 3] || (sub(y, x) == [0.0; 3]) {
                    continue;
                }
                let up = 0.5 + 0.5 * n[1];
                // A fixed key light from above, front and left gives the facets their shade.
                let key = unit([-0.45, 0.75, 0.55]);
                let lit = 0.74 + 0.4 * (n[0] * key[0] + n[1] * key[1] + n[2] * key[2]).max(0.0);
                let j = (1.0 + 0.09 * jitter(k)) * lit;
                k += 1;
                let col = [0, 1, 2].map(|q| (bottom[q] + (top[q] - bottom[q]) * up) * j);
                tri(p, x, z, y, col);
            }
        }
    }
}

const IDENT: [V; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// A basis turned by `yaw` about y, then `roll` about the turned z.
fn basis(yaw: f32, roll: f32) -> [V; 3] {
    let (sy, cy) = yaw.sin_cos();
    let (sr, cr) = roll.sin_cos();
    let x = [cy, 0.0, -sy];
    let z = [sy, 0.0, cy];
    let y = [0.0, 1.0, 0.0];
    [add(mul(x, cr), mul(y, sr)), add(mul(y, cr), mul(x, -sr)), z]
}

/// A flat ribbon along `pts` of half width `w`, lying across `side` (the surface's outward
/// direction), drawn double-sided by the material: the glowing cracks and the swirl marks.
fn ribbon(p: &mut MeshPart, pts: &[V], side: V, w: f32, col: V) {
    for s in pts.windows(2) {
        let d = unit(sub(s[1], s[0]));
        let across = mul(unit(cross(d, side)), w);
        let (a, b, c, e) = (sub(s[0], across), add(s[0], across), add(s[1], across), sub(s[1], across));
        tri(p, a, b, c, col);
        tri(p, a, c, e, col);
    }
}

/// A spiral mark on a puff's surface: `turns` turns from radius `r0` out to `r1` around the
/// point `c`, in the plane across `out`.
fn swirl(p: &mut MeshPart, c: V, out: V, r1: f32, col: V) {
    let o = unit(out);
    let u = unit(cross(o, if o[1].abs() > 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] }));
    let v = cross(o, u);
    let pts: Vec<V> = (0..=28)
        .map(|k| {
            let t = k as f32 / 28.0;
            let a = t * std::f32::consts::TAU * 1.6;
            let r = 0.08 + r1 * t;
            add(c, add(mul(u, r * a.cos()), mul(v, r * a.sin())))
        })
        .collect();
    ribbon(p, &pts, o, 0.07 + 0.04 * r1, col);
}

/// A zigzag lightning crack from `a` to `b` on a surface facing `out`.
fn crack(p: &mut MeshPart, a: V, b: V, out: V, kink: f32) {
    let d = sub(b, a);
    let side = unit(cross(d, out));
    let pts: Vec<V> = (0..=5).map(|k| add(add(a, mul(d, k as f32 / 5.0)), mul(side, if k % 2 == 1 { kink } else if k == 0 || k == 5 { 0.0 } else { -kink }))).collect();
    ribbon(p, &pts, out, 0.07, GLOW);
}

/// A cone (a fang or a horn tip) from base centre `c` with radius `r` to `tip`.
fn cone(p: &mut MeshPart, c: V, r: f32, tip: V, col: V) {
    let d = unit(sub(tip, c));
    let u = unit(cross(d, if d[1].abs() > 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] }));
    let v = cross(d, u);
    let n = 6;
    for k in 0..n {
        let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let e0 = add(c, add(mul(u, r * a0.cos()), mul(v, r * a0.sin())));
        let e1 = add(c, add(mul(u, r * a1.cos()), mul(v, r * a1.sin())));
        tri(p, e0, e1, tip, col);
    }
}

/// The glowing blue of the eyes and cracks (bright, so bloom picks it up), the lavender of the
/// cloud body and its shade, the darker swirl marks, the mouth and the fangs.
const GLOW: V = [0.55, 1.75, 2.0];
const LAV_TOP: V = [1.62, 1.56, 2.0];
const LAV_LOW: V = [1.08, 1.02, 1.55];
const SWIRL: V = [0.62, 0.58, 1.0];
const BROW: V = [1.75, 1.7, 2.0];
const MOUTH: V = [0.18, 0.12, 0.3];
const FANG: V = [1.95, 1.92, 1.85];

fn model(p: MeshPart) -> ModelData {
    let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
    for q in &p.positions {
        for k in 0..3 {
            min[k] = min[k].min(q[k]);
            max[k] = max[k].max(q[k]);
        }
    }
    ModelData { parts: vec![p], min, max, trees: 1, ..Default::default() }
}

/// A fist: a knuckled puff with four knuckle bumps toward `fwd`.
fn fist(p: &mut MeshPart, c: V, s: f32) {
    puff(p, c, [1.05 * s, 0.95 * s, 1.0 * s], IDENT, LAV_TOP, LAV_LOW, 6, 9);
    for k in 0..4 {
        let x = (k as f32 - 1.5) * 0.42 * s;
        puff(p, add(c, [x, 0.25 * s, 0.85 * s]), [0.3 * s, 0.32 * s, 0.3 * s], IDENT, LAV_TOP, LAV_LOW, 4, 6);
    }
    // The thumb over the front.
    puff(p, add(c, [0.0, -0.35 * s, 0.8 * s]), [0.55 * s, 0.25 * s, 0.25 * s], IDENT, LAV_TOP, LAV_LOW, 4, 6);
}

/// A wisp tail: shrinking puffs along a curl from `start` downward.
fn tail(p: &mut MeshPart, start: V, len: f32, r0: f32, curl: f32) {
    let n = 9;
    for k in 0..n {
        let t = k as f32 / (n - 1) as f32;
        let a = t * std::f32::consts::PI * 1.3;
        let at = add(start, [curl * a.sin() * len * 0.35, -t * len, -curl * (1.0 - a.cos()) * len * 0.2]);
        let r = r0 * (1.0 - 0.85 * t);
        puff(p, at, [r, 0.8 * r + 0.05, r], IDENT, LAV_TOP, LAV_LOW, 5, 8);
    }
}

/// The Cloudbreaker, the Cloud Beast stand-in (after the user's concept art "Concept 07 —
/// Cloudbreaker", this build's own art): a hulking storm cloud of faceted lavender puffs with
/// swirled shoulders, huge knuckled fists, a horned brow over glowing cyan eyes, a fanged open
/// mouth, glowing blue cracks down its arms and a curling wisp tail in place of legs. It faces
/// +z and floats at CLOUD_HOVER, about the size of its row (radius 3.0, height 3.5).
pub fn cloud_beast() -> ModelData {
    custom_cloud_beast().unwrap_or_else(cloudbreaker)
}

/// The user's own Cloud Beast model, when present: a Wavefront OBJ at NEOPETS_CLOUD_MODEL, else
/// `assets/models/cloud_beast.obj` beside the executable or in the working folder (shipped with
/// the build; assets/README.md), else `private/models/cloud_beast.obj` (positions and faces;
/// normals, UVs and materials ignored).
/// It is scaled to stand from the ground to just under the companion's ride height, centred,
/// turned by NEOPETS_CLOUD_MODEL_YAW degrees (default 0: the file's +z is the front, where the
/// user's Tempest Golem holds its fists) and
/// coloured lavender with the stand-in's key-light shading from smooth vertex normals.
fn custom_cloud_beast() -> Option<ModelData> {
    let beside = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("assets/models/cloud_beast.obj")));
    let candidates: Vec<std::path::PathBuf> = match std::env::var("NEOPETS_CLOUD_MODEL") {
        Ok(p) => vec![p.into()],
        Err(_) => beside.into_iter().chain(["assets/models/cloud_beast.obj".into(), "private/models/cloud_beast.obj".into()]).collect(),
    };
    let text = candidates.iter().find_map(|p| std::fs::read_to_string(p).ok())?;
    let mut pos: Vec<V> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let v: Vec<f32> = it.take(3).filter_map(|t| t.parse().ok()).collect();
                if let [x, y, z] = v[..] {
                    pos.push([x, y, z]);
                }
            }
            Some("f") => {
                // Each corner is "v", "v/vt", "v//vn" or "v/vt/vn", 1-based or negative.
                let corners: Vec<u32> = it
                    .filter_map(|t| t.split('/').next()?.parse::<i64>().ok())
                    .map(|i| if i < 0 { pos.len() as i64 + i } else { i - 1 })
                    .filter(|&i| i >= 0)
                    .map(|i| i as u32)
                    .collect();
                for k in 1..corners.len().saturating_sub(1) {
                    idx.extend([corners[0], corners[k], corners[k + 1]]);
                }
            }
            _ => {}
        }
    }
    if pos.is_empty() || idx.is_empty() || idx.iter().any(|&i| i as usize >= pos.len()) {
        return None;
    }
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for q in &pos {
        for k in 0..3 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    let top = COMPANION_RIDE - 0.3;
    let s = top / (hi[1] - lo[1]).max(1e-3);
    let yaw = std::env::var("NEOPETS_CLOUD_MODEL_YAW").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(0.0).to_radians();
    let (sy, cy) = yaw.sin_cos();
    let mid = [(lo[0] + hi[0]) / 2.0, lo[1], (lo[2] + hi[2]) / 2.0];
    let place = |q: V| {
        let (x, y, z) = ((q[0] - mid[0]) * s, (q[1] - mid[1]) * s, (q[2] - mid[2]) * s);
        [x * cy + z * sy, y, -x * sy + z * cy]
    };
    let positions: Vec<V> = pos.iter().map(|&q| place(q)).collect();
    // Smooth normals: the area-weighted face normals summed at each vertex.
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for t in idx.chunks_exact(3) {
        let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
        let n = cross(sub(b, a), sub(c, a));
        for &i in t {
            normals[i as usize] = add(normals[i as usize], n);
        }
    }
    let normals: Vec<V> = normals.into_iter().map(unit).collect();
    let key = unit([-0.45, 0.75, 0.55]);
    let colors: Vec<[f32; 4]> = normals
        .iter()
        .map(|n| {
            let up = 0.5 + 0.5 * n[1];
            let lit = 0.74 + 0.4 * (n[0] * key[0] + n[1] * key[1] + n[2] * key[2]).max(0.0);
            let c = [0, 1, 2].map(|q| (LAV_LOW[q] + (LAV_TOP[q] - LAV_LOW[q]) * up) * lit);
            [c[0], c[1], c[2], 1.0]
        })
        .collect();
    let mut p = part();
    p.uvs = vec![[0.0, 0.0]; positions.len()];
    p.raw_colors = vec![[128; 4]; positions.len()];
    p.positions = positions;
    p.normals = normals;
    p.colors = colors;
    p.indices = idx;
    Some(model(p))
}

/// The procedural Cloudbreaker (used when no custom model file is present).
pub fn cloudbreaker() -> ModelData {
    let mut p = part();
    let h = CLOUD_HOVER;
    let y = |v: f32| h + v;
    // The chest and belly.
    puff(&mut p, [0.0, y(2.0), 0.0], [1.9, 1.6, 1.5], IDENT, LAV_TOP, LAV_LOW, 7, 10);
    puff(&mut p, [0.0, y(1.0), 0.25], [1.4, 1.0, 1.15], IDENT, LAV_TOP, LAV_LOW, 6, 9);
    // The back's humps.
    for (x, yy, z, r) in [(-0.9, 2.7, -0.9, 1.1), (0.9, 2.7, -0.9, 1.1), (0.0, 3.4, -0.6, 1.0)] {
        puff(&mut p, [x, y(yy), z], [r, 0.9 * r, r], IDENT, LAV_TOP, LAV_LOW, 6, 9);
    }
    // The head, sunk between the shoulders, and its crown puffs.
    puff(&mut p, [0.0, y(3.25), 0.75], [1.15, 1.0, 0.95], IDENT, LAV_TOP, LAV_LOW, 7, 10);
    for (x, yy, z, r) in [(-0.6, 4.05, 0.3, 0.6), (0.6, 4.05, 0.3, 0.6), (0.0, 4.3, 0.1, 0.65), (0.0, 3.9, -0.4, 0.7)] {
        puff(&mut p, [x, y(yy), z], [r, 0.85 * r, r], IDENT, LAV_TOP, LAV_LOW, 5, 8);
    }
    // The cheeks and jaw puffs around the mouth.
    for s in [-1.0, 1.0] {
        puff(&mut p, [0.75 * s, y(2.75), 1.25], [0.55, 0.5, 0.5], IDENT, LAV_TOP, LAV_LOW, 5, 8);
    }
    puff(&mut p, [0.0, y(2.35), 1.15], [0.85, 0.45, 0.55], IDENT, LAV_TOP, LAV_LOW, 5, 8);
    // The brow: two slanted tufts over the eyes, curling up and out like horns.
    for s in [-1.0f32, 1.0] {
        puff(&mut p, [0.45 * s, y(3.72), 1.45], [0.5, 0.17, 0.3], basis(0.0, 0.45 * s), BROW, LAV_TOP, 4, 8);
        puff(&mut p, [0.95 * s, y(3.95), 1.2], [0.32, 0.2, 0.25], basis(0.0, 0.9 * s), BROW, LAV_TOP, 4, 7);
        cone(&mut p, [1.15 * s, y(4.05), 1.1], 0.16, [1.45 * s, y(4.45), 1.0], BROW);
        // The eye: a glowing cyan almond with a dark slit pupil, slanted in a scowl.
        puff(&mut p, [0.45 * s, y(3.42), 1.62], [0.27, 0.15, 0.08], basis(0.0, 0.3 * s), GLOW, GLOW, 4, 8);
        puff(&mut p, [0.43 * s, y(3.42), 1.69], [0.07, 0.11, 0.03], IDENT, MOUTH, MOUTH, 3, 6);
    }
    // The snout bump between the eyes.
    puff(&mut p, [0.0, y(3.25), 1.72], [0.22, 0.15, 0.15], IDENT, LAV_TOP, LAV_LOW, 4, 6);
    // The open roaring mouth with its fangs.
    puff(&mut p, [0.0, y(2.72), 1.6], [0.62, 0.36, 0.12], IDENT, MOUTH, MOUTH, 5, 10);
    for x in [-0.42f32, -0.15, 0.15, 0.42] {
        cone(&mut p, [x, y(3.0), 1.68], 0.08, [x * 0.95, y(2.72), 1.72], FANG);
    }
    for x in [-0.3f32, 0.3] {
        cone(&mut p, [x, y(2.42), 1.68], 0.08, [x * 0.9, y(2.66), 1.72], FANG);
    }
    // The shoulders: big swirled puffs, then the arms down to the fists.
    for s in [-1.0f32, 1.0] {
        let sh = [2.35 * s, y(3.0), -0.1];
        puff(&mut p, sh, [1.35, 1.25, 1.25], IDENT, LAV_TOP, LAV_LOW, 7, 10);
        swirl(&mut p, add(sh, [0.95 * s, 0.15, 0.55]), [s, 0.15, 0.6], 0.55, SWIRL);
        let arm = [2.85 * s, y(1.75), 0.35];
        puff(&mut p, arm, [0.95, 1.05, 0.9], IDENT, LAV_TOP, LAV_LOW, 6, 9);
        puff(&mut p, [2.75 * s, y(1.0), 0.6], [0.8, 0.7, 0.8], IDENT, LAV_TOP, LAV_LOW, 5, 8);
        fist(&mut p, [2.7 * s, y(0.15), 1.0], 1.15);
        swirl(&mut p, [3.75 * s, y(0.25), 0.9], [s, 0.0, 0.25], 0.35, SWIRL);
        // The glowing cracks down the shoulder and the forearm.
        crack(&mut p, [1.75 * s, y(3.55), 1.0], [1.5 * s, y(2.7), 1.25], [0.2 * s, 0.0, 1.0], 0.12);
        crack(&mut p, [3.6 * s, y(2.3), 0.75], [3.4 * s, y(1.15), 1.2], [s, 0.0, 0.6], 0.14);
        crack(&mut p, [1.5 * s, y(1.4), 1.2], [1.0 * s, y(0.8), 1.25], [0.3 * s, -0.2, 1.0], 0.1);
    }
    // The wisp tail under the belly, curling back.
    tail(&mut p, [0.0, y(0.2), 0.1], 2.0, 0.9, 1.0);
    model(p)
}

/// The companion stand-in (after the same concept sheet): a small round storm puff with curled
/// horns, glowing cyan eyes, a fanged grin, two little fists, a glowing crack and a wisp tail.
/// Its origin is its tail tip's level (it rides at COMPANION_RIDE).
pub fn cloud_companion() -> ModelData {
    let mut p = part();
    let y = |v: f32| v;
    puff(&mut p, [0.0, y(1.0), 0.0], [0.62, 0.56, 0.55], IDENT, LAV_TOP, LAV_LOW, 6, 9);
    // The horns: chains of shrinking puffs curling up and out, a point at the end.
    for s in [-1.0f32, 1.0] {
        let mut at = [0.38 * s, y(1.4), 0.0];
        let mut r = 0.2;
        for k in 0..5 {
            let a = k as f32 * 0.55;
            at = add(at, [0.13 * s * a.cos(), 0.12 * (1.0 - 0.35 * k as f32).max(-0.4) + 0.02, -0.03]);
            puff(&mut p, at, [r, r, r], IDENT, LAV_TOP, LAV_LOW, 4, 6);
            r *= 0.84;
        }
        cone(&mut p, at, r, add(at, [0.18 * s, -0.05, 0.0]), LAV_TOP);
        // The brow tuft and the eye.
        puff(&mut p, [0.2 * s, y(1.28), 0.48], [0.2, 0.07, 0.1], basis(0.0, 0.45 * s), BROW, LAV_TOP, 3, 6);
        puff(&mut p, [0.2 * s, y(1.12), 0.5], [0.12, 0.08, 0.05], basis(0.0, 0.3 * s), GLOW, GLOW, 3, 6);
        puff(&mut p, [0.19 * s, y(1.12), 0.54], [0.035, 0.05, 0.02], IDENT, MOUTH, MOUTH, 3, 5);
        // The little fists, held up in front.
        puff(&mut p, [0.42 * s, y(0.72), 0.42], [0.24, 0.22, 0.22], IDENT, LAV_TOP, LAV_LOW, 4, 7);
    }
    // The grin with its fangs.
    puff(&mut p, [0.0, y(0.88), 0.5], [0.24, 0.08, 0.06], IDENT, MOUTH, MOUTH, 3, 8);
    for x in [-0.13f32, 0.13] {
        cone(&mut p, [x, y(0.94), 0.54], 0.035, [x, y(0.82), 0.56], FANG);
    }
    crack(&mut p, [0.5, y(1.2), 0.25], [0.45, y(0.85), 0.35], [1.0, 0.0, 0.5], 0.05);
    tail(&mut p, [0.0, y(0.55), -0.05], 0.55, 0.3, 1.0);
    model(p)
}

#[cfg(test)]
mod tests {
    #[test]
    fn stand_ins_have_geometry() {
        let m = super::cloudbreaker();
        let p = &m.parts[0];
        assert!(p.indices.len() % 3 == 0 && p.indices.iter().all(|&i| (i as usize) < p.positions.len()));
        assert_eq!(p.positions.len(), p.colors.len());
        assert!(m.max[1] > super::CLOUD_HOVER + 3.0 && m.min[1] >= 0.0);
        assert!(!super::cloud_companion().parts[0].positions.is_empty());
    }
}
