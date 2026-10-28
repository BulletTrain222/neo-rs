//! Stand-in models for characters the disc has no model for. These are this build's own
//! placeholder art (procedural ellipsoids with vertex colours), not recovered assets.
//!
//! The Cloud Beast (SPEC-0010, Placement and reachability: no model, skeleton or clip bank on
//! the disc): a hovering storm cloud sized to its row (radius 3.0, height 3.5), white puffs over
//! a grey underside, with two dark eyes on its forward (+z) side. Its companion (row 2), the
//! lightning source that rides on it: a small yellow-white orb with darker cloud puffs.

use neopets_data::mesh::Layout;
use neopets_data::model::{MeshPart, ModelData};

/// How high the stand-in body floats above its ground point.
pub const CLOUD_HOVER: f32 = 2.0;
/// Where the companion rides, above the boss's ground point (the rig contract's bone 11 is not
/// recoverable; this is the stand-in's choice).
pub const COMPANION_RIDE: f32 = CLOUD_HOVER + 3.4;

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

/// An ellipsoid at `c` with radii `r`, coloured from `top` (upward faces) to `bottom`. Colours
/// are in the GS's units (an untextured part draws at half), so 2.0 is full white.
fn blob(p: &mut MeshPart, c: [f32; 3], r: [f32; 3], top: [f32; 3], bottom: [f32; 3]) {
    let (rings, segs) = (10u32, 16u32);
    let base = p.positions.len() as u32;
    for i in 0..=rings {
        let v = i as f32 / rings as f32;
        let th = v * std::f32::consts::PI;
        for j in 0..=segs {
            let ph = j as f32 / segs as f32 * std::f32::consts::TAU;
            let n = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
            p.positions.push([c[0] + r[0] * n[0], c[1] + r[1] * n[1], c[2] + r[2] * n[2]]);
            p.normals.push(n);
            p.uvs.push([0.0, 0.0]);
            let t = 0.5 + 0.5 * n[1];
            let k = |a: f32, b: f32| b + (a - b) * t;
            p.colors.push([k(top[0], bottom[0]), k(top[1], bottom[1]), k(top[2], bottom[2]), 1.0]);
            p.raw_colors.push([128; 4]);
        }
    }
    let w = segs + 1;
    for i in 0..rings {
        for j in 0..segs {
            let a = base + i * w + j;
            let b = a + w;
            p.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
}

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

/// The Cloud Beast stand-in.
pub fn cloud_beast() -> ModelData {
    let mut p = part();
    let h = CLOUD_HOVER;
    let white = [1.95, 1.95, 2.0];
    let grey = [0.95, 1.0, 1.15];
    let storm = [0.55, 0.6, 0.75];
    // The core and the puffs around it.
    blob(&mut p, [0.0, h + 1.8, 0.0], [2.4, 1.5, 2.0], white, grey);
    for (x, y, z, r) in [
        (-2.2, 1.5, 0.3, 1.3),
        (2.2, 1.5, 0.3, 1.3),
        (-1.2, 2.7, -0.2, 1.2),
        (1.1, 2.8, 0.1, 1.25),
        (0.0, 3.1, -0.6, 1.1),
        (-1.4, 1.6, -1.4, 1.1),
        (1.5, 1.7, -1.3, 1.1),
        (0.0, 2.0, -1.9, 1.2),
        (-3.1, 1.2, -0.4, 0.8),
        (3.1, 1.3, -0.3, 0.8),
    ] {
        blob(&mut p, [x, h + y, z], [r, 0.85 * r, r], white, grey);
    }
    // The dark underside.
    for (x, z, r) in [(-1.3, 0.5, 1.1), (1.3, 0.5, 1.1), (0.0, -0.6, 1.3), (0.0, 1.0, 0.9)] {
        blob(&mut p, [x, h + 0.75, z], [r, 0.55 * r, r], grey, storm);
    }
    // Eyes and brows on the forward side.
    let eye = [0.12, 0.12, 0.3];
    for s in [-1.0, 1.0] {
        blob(&mut p, [0.75 * s, h + 2.1, 1.85], [0.32, 0.42, 0.2], eye, eye);
        blob(&mut p, [0.68 * s, h + 2.2, 2.0], [0.09, 0.11, 0.06], white, white);
        blob(&mut p, [0.8 * s, h + 2.62, 1.82], [0.42, 0.08, 0.12], storm, storm);
    }
    // A frowning mouth.
    blob(&mut p, [0.0, h + 1.35, 1.95], [0.55, 0.1, 0.12], eye, eye);
    model(p)
}

/// The companion stand-in: a small lightning orb in a ring of dark puffs.
pub fn cloud_companion() -> ModelData {
    let mut p = part();
    let gold = [2.0, 1.9, 0.9];
    let amber = [1.9, 1.35, 0.3];
    blob(&mut p, [0.0, 0.6, 0.0], [0.55, 0.55, 0.55], gold, amber);
    let dark = [0.7, 0.72, 0.85];
    let darker = [0.4, 0.42, 0.55];
    for k in 0..5 {
        let a = k as f32 / 5.0 * std::f32::consts::TAU;
        blob(&mut p, [0.75 * a.cos(), 0.35, 0.75 * a.sin()], [0.42, 0.3, 0.42], dark, darker);
    }
    // Spark points.
    for (x, y, z) in [(0.0, 1.35, 0.0), (0.55, 1.05, 0.2), (-0.5, 1.1, -0.15)] {
        blob(&mut p, [x, y, z], [0.12, 0.28, 0.12], gold, gold);
    }
    model(p)
}

#[cfg(test)]
mod tests {
    #[test]
    fn stand_ins_have_geometry() {
        let m = super::cloud_beast();
        let p = &m.parts[0];
        assert!(p.indices.len() % 3 == 0 && p.indices.iter().all(|&i| (i as usize) < p.positions.len()));
        assert_eq!(p.positions.len(), p.colors.len());
        assert!(m.max[1] > super::CLOUD_HOVER + 3.0 && m.min[1] >= 0.0);
        assert!(!super::cloud_companion().parts[0].positions.is_empty());
    }
}
