//! The static scene of a world pack: the level display tree (type 3, world space), placed
//! objects (type 29) and instanced meshes (type 20), each with its placement matrix
//! (SPEC-0004 layouts; SPEC-0006 loader table).
use crate::resource::Chunk;
use std::ops::Range;

pub const LEVEL_TREE: u16 = 3;
pub const INSTANCES: u16 = 20;
pub const PLACED_OBJECTS: u16 = 29;
const OBJECT_MATRIX: usize = 0xC0;
const OBJECT_TREE: usize = 0x1D0;
const OBJECT_MIN_LEN: usize = 0x1E4;
const RECORD_INSTANCES: usize = 0x28;
const RECORD_TREE_CHUNK: usize = 0x30;
const INSTANCE_SIZE: usize = 0x50;
/// Counts on disc: at most 241 objects per chunk, 8 records per chunk, and a few hundred
/// instances per record. These bound hostile input.
const MAX_OBJECTS: usize = 4096;
const MAX_INSTANCES: usize = 16384;

/// A column-major 4 × 4 matrix: columns 0–2 are the axes, elements 12–14 the translation.
pub type Matrix = [f32; 16];

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn matrix_at(b: &[u8], at: usize) -> Option<Matrix> {
    let m: Vec<f32> = (0..16)
        .map(|i| u32_at(b, at + i * 4).map(f32::from_bits))
        .collect::<Option<_>>()?;
    m.try_into().ok()
}

pub fn transform_point(m: &Matrix, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|r| m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r])
}

/// The axes applied to a direction, renormalised (matrices may scale slightly).
pub fn transform_normal(m: &Matrix, n: [f32; 3]) -> [f32; 3] {
    let v: [f32; 3] = std::array::from_fn(|r| m[r] * n[0] + m[4 + r] * n[1] + m[8 + r] * n[2]);
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 { v.map(|c| c / len) } else { n }
}

/// A node tree to draw, as a byte range of the resource, with its placement.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub payload: Range<usize>,
    /// None for world-space trees.
    pub matrix: Option<Matrix>,
    pub source: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub trees: Vec<Placed>,
    /// Object or record references that do not resolve to a chunk (stale data on disc).
    pub unresolved: usize,
}

/// The payload of an inline IDM chunk at `at`, or None when there is no chunk there.
fn inline_chunk(b: &[u8], at: usize) -> Option<Range<usize>> {
    if b.get(at..at + 3)? != b"IDM" {
        return None;
    }
    let extra = i16::from_le_bytes(b.get(at + 6..at + 8)?.try_into().ok()?);
    let size = u32_at(b, at + 8)? as usize;
    let start = at + 16 + usize::try_from(extra).ok()?;
    let end = at.checked_add(size)?;
    (start <= end && end <= b.len()).then_some(start..end)
}

fn placed_objects(p: &[u8], base: usize, scene: &mut Scene) {
    let count = u32_at(p, 0).unwrap_or(0) as usize;
    let table = u32_at(p, 4).unwrap_or(0) as usize;
    for k in 0..count.min(MAX_OBJECTS) {
        let Some(obj) = u32_at(p, table + 4 * k).map(|o| o as usize) else {
            scene.unresolved += 1;
            continue;
        };
        let tree = (obj + OBJECT_MIN_LEN <= p.len())
            .then(|| u32_at(p, obj + OBJECT_TREE))
            .flatten()
            .filter(|&t| t != 0)
            .and_then(|t| inline_chunk(p, t as usize));
        match (tree, matrix_at(p, obj + OBJECT_MATRIX)) {
            (Some(r), Some(m)) => scene.trees.push(Placed {
                payload: base + r.start..base + r.end,
                matrix: Some(m),
                source: PLACED_OBJECTS,
            }),
            _ => scene.unresolved += 1,
        }
    }
}

fn instances(p: &[u8], base: usize, scene: &mut Scene) {
    let count = u32_at(p, 0).unwrap_or(0) as usize;
    let table = u32_at(p, 4).unwrap_or(0) as usize;
    for k in 0..count.min(MAX_OBJECTS) {
        let Some(rec) = u32_at(p, table + 4 * k).map(|o| o as usize) else {
            scene.unresolved += 1;
            continue;
        };
        let (Some(tree), Some(n), Some(at)) = (
            inline_chunk(p, rec + RECORD_TREE_CHUNK),
            u32_at(p, rec + RECORD_INSTANCES),
            u32_at(p, rec + RECORD_INSTANCES + 4),
        ) else {
            scene.unresolved += 1;
            continue;
        };
        for i in 0..(n as usize).min(MAX_INSTANCES) {
            match matrix_at(p, rec + at as usize + i * INSTANCE_SIZE) {
                Some(m) => scene.trees.push(Placed {
                    payload: base + tree.start..base + tree.end,
                    matrix: Some(m),
                    source: INSTANCES,
                }),
                None => scene.unresolved += 1,
            }
        }
    }
}

/// Collect the drawable static scene of a world pack's top-level chain.
pub fn scene(bytes: &[u8], top: &[Chunk]) -> Scene {
    let mut s = Scene::default();
    for c in top {
        let p = &bytes[c.payload.clone()];
        match c.kind {
            LEVEL_TREE => s.trees.push(Placed {
                payload: c.payload.clone(),
                matrix: None,
                source: LEVEL_TREE,
            }),
            PLACED_OBJECTS => placed_objects(p, c.payload.start, &mut s),
            INSTANCES => instances(p, c.payload.start, &mut s),
            _ => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_placement_vector() {
        // SPEC-0004: X = (0, 0, -1), Y = (0, 1, 0), Z = (1, 0, 0), translation (10, 0, 5).
        let m: Matrix = [0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 10., 0., 5., 1.];
        assert_eq!(transform_point(&m, [1.0, 0.0, 0.0]), [10.0, 0.0, 4.0]);
        assert_eq!(transform_normal(&m, [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]);
    }

    fn idm(kind: u16, payload: &[u8]) -> Vec<u8> {
        let mut c = b"IDM\x01".to_vec();
        c.extend(kind.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend((16 + payload.len() as u32).to_le_bytes());
        c.extend([0; 4]);
        c.extend(payload);
        c
    }

    #[test]
    fn finds_objects_and_instances_and_counts_stale_references() {
        let tree = [7u8; 8];
        // Type 29: two objects; the second has no tree.
        let mut objects = vec![0u8; 0x18];
        objects[0..4].copy_from_slice(&2u32.to_le_bytes());
        objects[4..8].copy_from_slice(&0x10u32.to_le_bytes());
        for k in 0..2u32 {
            let obj = objects.len() as u32;
            objects[0x10 + 4 * k as usize..0x14 + 4 * k as usize].copy_from_slice(&obj.to_le_bytes());
            let mut o = vec![0u8; OBJECT_MIN_LEN];
            o[OBJECT_MATRIX + 60..OBJECT_MATRIX + 64].copy_from_slice(&1f32.to_le_bytes());
            if k == 0 {
                let at = obj + OBJECT_MIN_LEN as u32;
                o[OBJECT_TREE..OBJECT_TREE + 4].copy_from_slice(&at.to_le_bytes());
                o.extend(idm(12, &tree));
            }
            objects.extend(o);
        }
        // Type 20: one record with three instances.
        let mut inst = vec![0u8; 0x14];
        inst[0..4].copy_from_slice(&1u32.to_le_bytes());
        inst[4..8].copy_from_slice(&0x10u32.to_le_bytes());
        inst[0x10..0x14].copy_from_slice(&0x14u32.to_le_bytes());
        let mut rec = vec![0u8; RECORD_TREE_CHUNK];
        rec[RECORD_INSTANCES..RECORD_INSTANCES + 4].copy_from_slice(&3u32.to_le_bytes());
        let inline = idm(12, &tree);
        let at = (RECORD_TREE_CHUNK + inline.len()) as u32;
        rec[RECORD_INSTANCES + 4..RECORD_INSTANCES + 8].copy_from_slice(&at.to_le_bytes());
        rec.extend(inline);
        for i in 0..3 {
            let mut m = vec![0u8; INSTANCE_SIZE];
            m[48..52].copy_from_slice(&(i as f32).to_le_bytes());
            rec.extend(m);
        }
        inst.extend(rec);
        let mut file = idm(LEVEL_TREE, &tree);
        file.extend(idm(PLACED_OBJECTS, &objects));
        file.extend(idm(INSTANCES, &inst));
        file.extend(idm(0, &[]));
        let top = crate::resource::walk_chunks(&file, None).unwrap();
        let s = scene(&file, &top.chunks);
        let sources: Vec<u16> = s.trees.iter().map(|t| t.source).collect();
        assert_eq!(sources, [3, 29, 20, 20, 20]);
        assert_eq!(s.unresolved, 1);
        assert!(s.trees.iter().all(|t| file[t.payload.clone()] == tree));
        assert_eq!(s.trees[3].matrix.unwrap()[12], 1.0, "instance 1 translates by x = 1");
    }
}
