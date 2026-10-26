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
    /// A type-20 instance's zone column (+0x40; SPEC-0004 Instanced meshes).
    pub zone: Option<u8>,
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

/// The payload of an inline chunk at `at` of a payload (crate use).
pub(crate) fn inline_payload(b: &[u8], at: usize) -> Option<Range<usize>> {
    inline_chunk(b, at)
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
                // SPEC-0028 section 7: the object's zone column (0x1B8), read as stored.
                zone: p.get(obj + 0x1B8).copied(),
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
                    zone: p.get(rec + at as usize + i * INSTANCE_SIZE + 0x40).copied(),
                }),
                None => scene.unresolved += 1,
            }
        }
    }
}

/// A type-29 placed object (SPEC-0028 section 7 and 7.1), by its index in the world's object
/// table: its inline mesh tree (absolute byte range), the rest block (+0xC0), the scale
/// (+0x140), the disc state (+0x1A0), and the key count F (+0x1E4) with its motion stream bytes
/// (offset +0x1E8, payload-relative; up to the chunk end).
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub tree: Option<Range<usize>>,
    pub rest: Matrix,
    pub scale: [f32; 3],
    pub state: i32,
    pub keys: u16,
    pub stream: Option<Range<usize>>,
    /// The zone column (0x1B8): drawn only while the zone query passes (SPEC-0028 section 7).
    pub zone: u8,
    /// The inline model set (0x1D4, a type-5 chunk): its payload's byte range in the pack.
    pub model_set: Option<Range<usize>>,
}

/// Every placed object of the first type-29 chunk of a world pack's top chain, in table order
/// (the order the script calls index). A table entry that is not an object gives None.
pub fn objects(bytes: &[u8], top: &[Chunk]) -> Vec<Option<Object>> {
    let Some(c) = top.iter().find(|c| c.kind == PLACED_OBJECTS) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let base = c.payload.start;
    let count = u32_at(p, 0).unwrap_or(0) as usize;
    let table = u32_at(p, 4).unwrap_or(0) as usize;
    (0..count.min(MAX_OBJECTS))
        .map(|k| {
            let obj = u32_at(p, table + 4 * k)? as usize;
            if obj == 0 || obj + 0x1EC > p.len() {
                return None;
            }
            let rest = matrix_at(p, obj + OBJECT_MATRIX)?;
            let f = |at: usize| u32_at(p, obj + at).map(f32::from_bits).unwrap_or(1.0);
            let tree = u32_at(p, obj + OBJECT_TREE).filter(|&t| t != 0).and_then(|t| inline_chunk(p, t as usize)).map(|r| base + r.start..base + r.end);
            let keys = u16::from_le_bytes([p[obj + 0x1E4], p[obj + 0x1E5]]);
            let stream = u32_at(p, obj + 0x1E8).map(|o| o as usize).filter(|&o| o != 0 && o < p.len()).map(|o| base + o..base + p.len());
            let model_set = u32_at(p, obj + 0x1D4).filter(|&t| t != 0).and_then(|t| inline_chunk(p, t as usize)).map(|r| base + r.start..base + r.end);
            Some(Object { tree, rest, scale: [f(0x140), f(0x144), f(0x148)], state: u32_at(p, obj + 0x1A0).unwrap_or(3) as i32, keys, stream, zone: p.get(obj + 0x1B8).copied().unwrap_or(0), model_set })
        })
        .collect()
}

/// A breakable's type-11 volume record (SPEC-0048 +0x60; SPEC-0028 section 6): its kind byte
/// (+0x10; 1 sphere, 3 cylinder, 4 and 7 rectangles), and the body fields the attack contact
/// tests read (SPEC-0036 Hit detection, step 6): the centre or origin (+0x30), the radius in the
/// fourth lane (+0x3C), the cylinder height (+0x24) and axis (+0x40), and for a rectangle its
/// three axes (+0x60, +0x50, +0x40) and extents (+0x24).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub kind: u8,
    pub centre: [f32; 3],
    pub radius: f32,
    pub height: f32,
    pub axis: [f32; 3],
    pub axes: [[f32; 3]; 3],
    pub extents: [f32; 3],
}

/// A world breakable, an entry of the type-29 second list (SPEC-0048 Breakable entry).
#[derive(Clone, Debug, PartialEq)]
pub struct Breakable {
    /// The draw matrix (the entry's first 64 bytes, column-major); its translation is the break
    /// position.
    pub matrix: Matrix,
    pub state: i32,
    /// The mesh it draws (byte range of the pack): the inline type-12 chunk at +0x44, or the
    /// near mesh of record +0x48 of the first type-20 chunk.
    pub mesh: Option<Range<usize>>,
    pub pieces: u32,
    pub spawn_key: i16,
    pub drop: i16,
    pub script: i16,
    pub break_float: i16,
    pub material: i16,
    pub element: u8,
    pub start_mode: u8,
    pub volume: Option<Volume>,
}

impl Breakable {
    pub fn position(&self) -> [f32; 3] {
        [self.matrix[12], self.matrix[13], self.matrix[14]]
    }
}

/// The near mesh (the record's tree chunk) of record k of the first type-20 chunk.
fn instance_tree(bytes: &[u8], top: &[Chunk], k: usize) -> Option<Range<usize>> {
    let c = top.iter().find(|c| c.kind == INSTANCES)?;
    let p = &bytes[c.payload.clone()];
    if k >= u32_at(p, 0)? as usize {
        return None;
    }
    let rec = u32_at(p, u32_at(p, 4)? as usize + 4 * k)? as usize;
    inline_chunk(p, rec + RECORD_TREE_CHUNK).map(|r| c.payload.start + r.start..c.payload.start + r.end)
}

/// Every breakable of the first type-29 chunk: count at +0x08, offset table at +0x0C, each entry
/// at its offset measured from the start of the table.
pub fn breakables(bytes: &[u8], top: &[Chunk]) -> Vec<Breakable> {
    let Some(c) = top.iter().find(|c| c.kind == PLACED_OBJECTS) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let base = c.payload.start;
    let count = u32_at(p, 8).unwrap_or(0) as usize;
    let table = u32_at(p, 0x0C).unwrap_or(0) as usize;
    (0..count.min(MAX_OBJECTS))
        .filter_map(|k| {
            let e = table + u32_at(p, table + 4 * k)? as usize;
            let i16_at = |o: usize| p.get(e + o..e + o + 2).map(|b| i16::from_le_bytes([b[0], b[1]]));
            let f = |o: usize| u32_at(p, o).map(f32::from_bits);
            let v3 = |o: usize| Some([f(o)?, f(o + 4)?, f(o + 8)?]);
            let matrix = matrix_at(p, e)?;
            let instance = u32_at(p, e + 0x48)? as i32;
            let inline = u32_at(p, e + 0x44).filter(|&o| o != 0).and_then(|o| inline_chunk(p, e + o as usize)).map(|r| base + r.start..base + r.end);
            let mesh = usize::try_from(instance).ok().and_then(|i| instance_tree(bytes, top, i)).or(inline);
            let volume = u32_at(p, e + 0x60).filter(|&o| o != 0).and_then(|o| {
                let r = e + o as usize;
                Some(Volume {
                    kind: *p.get(r + 0x10)?,
                    centre: v3(r + 0x30)?,
                    radius: f(r + 0x3C)?,
                    height: f(r + 0x24)?,
                    axis: v3(r + 0x40)?,
                    axes: [v3(r + 0x60)?, v3(r + 0x50)?, v3(r + 0x40)?],
                    extents: v3(r + 0x24)?,
                })
            });
            Some(Breakable {
                matrix,
                state: u32_at(p, e + 0x40)? as i32,
                mesh,
                pieces: u32_at(p, e + 0x4C)?,
                spawn_key: i16_at(0x54)?,
                drop: i16_at(0x56)?,
                script: i16_at(0x58)?,
                break_float: i16_at(0x5A)?,
                material: i16_at(0x5C)?,
                element: *p.get(e + 0x5E)?,
                start_mode: *p.get(e + 0x5F)?,
                volume,
            })
        })
        .collect()
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
                zone: None,
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
    fn breakable_entries() {
        // A type-29 payload: no objects; one second-list entry at table + 0x10, with a sphere
        // volume at entry + 0x80.
        let mut p = vec![0u8; 0x200];
        p[8..12].copy_from_slice(&1u32.to_le_bytes());
        p[0x0C..0x10].copy_from_slice(&0x20u32.to_le_bytes());
        p[0x20..0x24].copy_from_slice(&0x10u32.to_le_bytes());
        let e = 0x30;
        for (i, v) in [1.0f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 6.0, 7.0, 1.0].iter().enumerate() {
            p[e + 4 * i..e + 4 * i + 4].copy_from_slice(&v.to_le_bytes());
        }
        p[e + 0x40..e + 0x44].copy_from_slice(&1u32.to_le_bytes());
        p[e + 0x48..e + 0x4C].copy_from_slice(&(-1i32).to_le_bytes());
        p[e + 0x56..e + 0x58].copy_from_slice(&3i16.to_le_bytes());
        p[e + 0x58..e + 0x5A].copy_from_slice(&9i16.to_le_bytes());
        p[e + 0x5C..e + 0x5E].copy_from_slice(&4i16.to_le_bytes());
        p[e + 0x60..e + 0x64].copy_from_slice(&0x80u32.to_le_bytes());
        let r = e + 0x80;
        p[r + 0x10] = 1;
        for (i, v) in [1.0f32, 2.0, 3.0, 0.75].iter().enumerate() {
            p[r + 0x30 + 4 * i..r + 0x34 + 4 * i].copy_from_slice(&v.to_le_bytes());
        }
        let mut file = idm(PLACED_OBJECTS, &p);
        file.extend(idm(0, &[]));
        let chain = crate::resource::walk_chunks(&file, None).unwrap();
        let b = breakables(&file, &chain.chunks);
        assert_eq!(b.len(), 1);
        assert_eq!((b[0].position(), b[0].state, b[0].drop, b[0].script, b[0].material, b[0].mesh.clone()), ([5.0, 6.0, 7.0], 1, 3, 9, 4, None));
        let v = b[0].volume.unwrap();
        assert_eq!((v.kind, v.centre, v.radius), (1, [1.0, 2.0, 3.0], 0.75));
    }

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
