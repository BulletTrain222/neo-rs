//! Script trigger volumes and script groups of a world pack.
//!
//! - The trigger volumes are the type-11 primitive records (SPEC-0028 sections 6 to 6.4 and 13):
//!   spheres (pointer +0x30, count +0x08), cylinders (+0x38, +0x0C), kind-4 rectangles (+0x40,
//!   +0x18) and kind-7 rectangles (+0x3C, +0x14), each a 0x20-byte header (kind +0x10, flags
//!   +0x12, gate mask +0x1A, number +0x1C, class +0x1E) and its shape. The trigger capacity is the
//!   first u32 of the payload (SPEC-0030 Triggers, Trigger capacity).
//! - The script groups are the type-40 list entries (SPEC-0028 section 11; SPEC-0048 Script
//!   groups): count at +0x594, list at +0x598, 0x44-byte entries with seven count-and-pointer
//!   lists of s32 indices.

use crate::ground::OBJECT_HEADER;
use crate::resource::Chunk;

/// The type-40 chunk.
pub const SCRIPT_GROUPS: u16 = 40;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    u32_at(b, at).map(|v| v as i32)
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn v3(b: &[u8], at: usize) -> Option<[f32; 3]> {
    Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?])
}

/// A volume's shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Kind 1: centre (+0x30) and radius (+0x3C).
    Sphere { centre: [f32; 3], radius: f32 },
    /// Kind 3: base point (+0x30), unit axis (+0x40), signed full length (+0x24), radius (+0x3C).
    Cylinder { base: [f32; 3], axis: [f32; 3], length: f32, radius: f32 },
    /// Kinds 4 and 7: reference point (+0x30); axes V (+0x60), A (+0x50) and the normal B
    /// (+0x40); signed extents along them (+0x24, +0x28, +0x2C, the last the switch value).
    Rectangle { origin: [f32; 3], axes: [[f32; 3]; 3], extents: [f32; 3] },
}

/// One type-11 primitive record with its header.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub kind: u8,
    pub flags: u16,
    pub mask: u16,
    /// The script trigger index (+0x1C); the door number on a door.
    pub index: u16,
    pub class: u16,
    pub shape: Shape,
}

/// The trigger capacity and the volumes of a pack, in list order: spheres, cylinders, kind-4
/// rectangles, kind-7 rectangles. A list stops at the first record that does not fit.
pub fn volumes(bytes: &[u8], top: &[Chunk]) -> (usize, Vec<Volume>) {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return (0, Vec::new()) };
    let Some(p) = bytes.get(c.payload.clone()) else { return (0, Vec::new()) };
    volumes_in(p)
}

/// A placed object's inline type-11 header (object +0x1D8, SPEC-0028 sections 6 and 7): its
/// trigger count (the first word), its primitives and its inline locator list (count +0x24,
/// 0x50-byte records at +0x4C, each with its point +0x30 and direction +0x40; SPEC-0030 Note on
/// animated locators). The primitives are in the object's rest frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectHeader {
    pub capacity: usize,
    pub volumes: Vec<Volume>,
    pub locators: Vec<([f32; 3], [f32; 3])>,
}

/// Read an object's inline type-11 header from its payload.
pub fn object_header(p: &[u8]) -> ObjectHeader {
    let (capacity, volumes) = volumes_in(p);
    let n = i32_at(p, 0x24).unwrap_or(0).clamp(0, 1 << 12) as usize;
    let list = u32_at(p, 0x4C).unwrap_or(0) as usize;
    let locators = (0..n).map_while(|i| Some((v3(p, list + 0x50 * i + 0x30)?, v3(p, list + 0x50 * i + 0x40)?))).collect();
    ObjectHeader { capacity, volumes, locators }
}

/// The trigger capacity and the volumes of a type-11 payload.
fn volumes_in(p: &[u8]) -> (usize, Vec<Volume>) {
    let capacity = u32_at(p, 0).unwrap_or(0).min(1 << 12) as usize;
    let mut out = Vec::new();
    let lists: [(u8, usize, usize, usize); 4] = [(1, 0x08, 0x30, 0x40), (3, 0x0C, 0x38, 0x50), (4, 0x18, 0x40, 0x70), (7, 0x14, 0x3C, 0x70)];
    for (kind, count_at, list_at, size) in lists {
        let n = i32_at(p, count_at).unwrap_or(0).clamp(0, 1 << 14) as usize;
        let Some(list) = u32_at(p, list_at).map(|v| v as usize) else { continue };
        for i in 0..n {
            let r = list + i * size;
            if r + size > p.len() {
                break;
            }
            let shape = match kind {
                1 => v3(p, r + 0x30).zip(f32_at(p, r + 0x3C)).map(|(centre, radius)| Shape::Sphere { centre, radius }),
                3 => (|| Some(Shape::Cylinder { base: v3(p, r + 0x30)?, axis: v3(p, r + 0x40)?, length: f32_at(p, r + 0x24)?, radius: f32_at(p, r + 0x3C)? }))(),
                _ => (|| Some(Shape::Rectangle { origin: v3(p, r + 0x30)?, axes: [v3(p, r + 0x60)?, v3(p, r + 0x50)?, v3(p, r + 0x40)?], extents: v3(p, r + 0x24)? }))(),
            };
            let (Some(shape), Some(flags), Some(mask), Some(index), Some(class)) = (shape, u16_at(p, r + 0x12), u16_at(p, r + 0x1A), u16_at(p, r + 0x1C), u16_at(p, r + 0x1E)) else { break };
            out.push(Volume { kind, flags, mask, index, class, shape });
        }
    }
    (capacity, out)
}

/// A movable-block source: an n1 entry of the type-11 chunk (SPEC-0028 section 6, 0x24-byte
/// entries, count at header +0x1C, table at +0x44; SPEC-0048 Movable block record +0x68) with its
/// box, a kind-4 rectangle record (entry +0x20) whose flags have bit 1 set.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockSource {
    /// Entry +0x10, the size S (the step length is half of it).
    pub size: f32,
    /// Entry +0x14, the height H.
    pub height: f32,
    /// Entry +0x18, the registry model id (-1 on disc).
    pub model: i32,
    /// Entry +0x1C, the inline mesh (byte range of the pack).
    pub mesh: Option<std::ops::Range<usize>>,
    /// The box: its reference corner (+0x30), first axis A (+0x40), second axis B (+0x50),
    /// vertical axis (+0x60) and extents (vertical +0x24, along B +0x28, along A +0x2C).
    pub corner: [f32; 3],
    pub axes: [[f32; 3]; 3],
    pub extents: [f32; 3],
    pub flags: u16,
}

/// Every n1 entry that builds a movable block, in entry order (SPEC-0048 Source). The table and
/// the two pointers are payload-relative offsets, as the header's other tables (SPEC-0028
/// section 6: rewritten at load).
pub fn movable_blocks(bytes: &[u8], top: &[Chunk]) -> Vec<BlockSource> {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let n = i32_at(p, 0x1C).unwrap_or(0).clamp(0, 1 << 10) as usize;
    let Some(table) = u32_at(p, 0x44).map(|v| v as usize) else { return Vec::new() };
    (0..n)
        .filter_map(|i| {
            let e = table + i * 0x24;
            let r = u32_at(p, e + 0x20)? as usize;
            if r == 0 || r + 0x70 > p.len() {
                return None;
            }
            let flags = u16_at(p, r + 0x12)?;
            if flags & 0x0002 == 0 {
                return None;
            }
            let mesh = u32_at(p, e + 0x1C).filter(|&o| o != 0).and_then(|o| crate::world::inline_payload(p, o as usize)).map(|m| c.payload.start + m.start..c.payload.start + m.end);
            Some(BlockSource {
                size: f32_at(p, e + 0x10)?,
                height: f32_at(p, e + 0x14)?,
                model: i32_at(p, e + 0x18)?,
                mesh,
                corner: v3(p, r + 0x30)?,
                axes: [v3(p, r + 0x40)?, v3(p, r + 0x50)?, v3(p, r + 0x60)?],
                extents: v3(p, r + 0x24)?,
                flags,
            })
        })
        .collect()
}

/// The type-10 chunk: ladders and climb panels.
pub const PANELS: u16 = 10;

/// A ladder or climb-panel record of a type-10 group (SPEC-0006 Type 10): its origin, the
/// face plane (unit normal n, offset in the fourth lane), the side axis U and up axis V (each
/// with its offset, so the plane value is the position along it), the lengths along U and V
/// and the type (-3 ladder, -2 climb panel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panel {
    pub origin: [f32; 3],
    pub normal: [f32; 4],
    pub u: [f32; 4],
    pub v: [f32; 4],
    pub lu: f32,
    pub lv: f32,
    pub kind: i8,
}

/// Every record of the first type-10 chunk, group by group. The chunk's offsets count from the
/// chunk's own first byte, 16 before the payload (SPEC-0006: chunk-relative).
pub fn panels(bytes: &[u8], top: &[Chunk]) -> Vec<Panel> {
    let Some(c) = top.iter().find(|c| c.kind == PANELS) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let v4 = |at: usize| Some([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?, f32_at(p, at + 12)?]);
    let groups = u32_at(p, 0).unwrap_or(0).min(1 << 10) as usize;
    let Some(table) = u32_at(p, 4).and_then(|v| (v as usize).checked_sub(16)) else { return Vec::new() };
    let mut out = Vec::new();
    for g in 0..groups {
        let at = table + 0x20 * g;
        let (Some(n), Some(list)) = (u32_at(p, at + 0x10), u32_at(p, at + 0x14)) else { break };
        for k in 0..(n as usize).min(1 << 10) {
            let Some(r) = (list as usize).checked_sub(16).map(|l| l + 0x60 * k) else { break };
            let rec = (|| {
                let o = v4(r)?;
                Some(Panel { origin: [o[0], o[1], o[2]], normal: v4(r + 0x10)?, u: v4(r + 0x20)?, v: v4(r + 0x30)?, lu: f32_at(p, r + 0x40)?, lv: f32_at(p, r + 0x44)?, kind: *p.get(r + 0x4C)? as i8 })
            })();
            let Some(rec) = rec else { break };
            out.push(rec);
        }
    }
    out
}

/// The type-10 groups' centres and squared radii (for the census check).
pub fn panel_groups(bytes: &[u8], top: &[Chunk]) -> Vec<([f32; 3], f32, usize)> {
    let Some(c) = top.iter().find(|c| c.kind == PANELS) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let groups = u32_at(p, 0).unwrap_or(0).min(1 << 10) as usize;
    let Some(table) = u32_at(p, 4).and_then(|v| (v as usize).checked_sub(16)) else { return Vec::new() };
    (0..groups)
        .map_while(|g| {
            let at = table + 0x20 * g;
            Some(([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?], f32_at(p, at + 0x0C)?, u32_at(p, at + 0x10)? as usize))
        })
        .collect()
}

/// A type-40 script group entry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScriptGroup {
    pub kind: u32,
    /// The collected-table row (SPEC-0051) the group marks when it completes.
    pub key: u32,
    pub locators: Vec<i32>,
    pub triggers: Vec<i32>,
    pub objects: Vec<i32>,
    pub cameras: Vec<i32>,
    pub breakables: Vec<i32>,
    pub effects: Vec<i32>,
    /// The drop pair: [0] a placement id to drop, [1] a character id to activate.
    pub drops: Vec<i32>,
}

/// The script groups of a pack (empty without a type-40 chunk).
pub fn script_groups(bytes: &[u8], top: &[Chunk]) -> Vec<ScriptGroup> {
    let Some(c) = top.iter().find(|c| c.kind == SCRIPT_GROUPS) else { return Vec::new() };
    let Some(p) = bytes.get(c.payload.clone()) else { return Vec::new() };
    let n = u32_at(p, 0x594).unwrap_or(0).min(1 << 12) as usize;
    let Some(list) = u32_at(p, 0x598).map(|v| v as usize) else { return Vec::new() };
    let ints = |r: usize, at: usize| -> Vec<i32> {
        let (Some(count), Some(ptr)) = (u32_at(p, r + at), u32_at(p, r + at + 4)) else { return Vec::new() };
        (0..count.min(1 << 10) as usize).map_while(|i| i32_at(p, ptr as usize + 4 * i)).collect()
    };
    (0..n)
        .map_while(|i| {
            let r = list + i * 0x44;
            (r + 0x44 <= p.len()).then(|| ScriptGroup {
                kind: u32_at(p, r).unwrap_or(0),
                key: u32_at(p, r + 0x08).unwrap_or(0),
                locators: ints(r, 0x0C),
                triggers: ints(r, 0x14),
                objects: ints(r, 0x1C),
                cameras: ints(r, 0x24),
                breakables: ints(r, 0x2C),
                effects: ints(r, 0x34),
                drops: ints(r, 0x3C),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(b: &mut Vec<u8>, at: usize, v: &[u8]) {
        if b.len() < at + v.len() {
            b.resize(at + v.len(), 0);
        }
        b[at..at + v.len()].copy_from_slice(v);
    }

    fn chunk(kind: u16, payload: Vec<u8>) -> (Vec<u8>, Vec<Chunk>) {
        let len = payload.len();
        (payload, vec![Chunk { offset: 0, tag: [0; 4], kind, extra: 0, size: len as u32, role_id: 0, payload: 0..len }])
    }

    #[test]
    fn reads_a_cylinder_and_a_rectangle_with_their_trigger_indices() {
        let mut p = vec![0u8; 0x60];
        put(&mut p, 0, &9u32.to_le_bytes());
        put(&mut p, 0x0C, &1i32.to_le_bytes());
        put(&mut p, 0x38, &0x60u32.to_le_bytes());
        put(&mut p, 0x18, &1i32.to_le_bytes());
        put(&mut p, 0x40, &0xB0u32.to_le_bytes());
        // Cylinder: flags 0x0008, index 5, base (1, 2, 3), axis up, length 4, radius 2.5.
        let r = 0x60;
        put(&mut p, r + 0x12, &0x0008u16.to_le_bytes());
        put(&mut p, r + 0x1C, &5u16.to_le_bytes());
        put(&mut p, r + 0x24, &4.0f32.to_le_bytes());
        for (k, v) in [1.0f32, 2.0, 3.0].iter().enumerate() {
            put(&mut p, r + 0x30 + 4 * k, &v.to_le_bytes());
        }
        put(&mut p, r + 0x3C, &2.5f32.to_le_bytes());
        put(&mut p, r + 0x44, &1.0f32.to_le_bytes());
        // Rectangle: flags 0x0020, index 7, extents 5.85, 5.1, 4.8.
        let r = 0xB0;
        put(&mut p, r + 0x12, &0x0020u16.to_le_bytes());
        put(&mut p, r + 0x1C, &7u16.to_le_bytes());
        for (k, v) in [5.85f32, 5.1, 4.8].iter().enumerate() {
            put(&mut p, r + 0x24 + 4 * k, &v.to_le_bytes());
        }
        put(&mut p, r + 0x70 - 1, &[0]);
        let (b, top) = chunk(OBJECT_HEADER, p);
        let (cap, v) = volumes(&b, &top);
        assert_eq!(cap, 9);
        assert_eq!(v.len(), 2);
        assert_eq!((v[0].kind, v[0].flags, v[0].index), (3, 8, 5));
        assert_eq!(v[0].shape, Shape::Cylinder { base: [1.0, 2.0, 3.0], axis: [0.0, 1.0, 0.0], length: 4.0, radius: 2.5 });
        assert_eq!((v[1].kind, v[1].flags, v[1].index), (4, 0x20, 7));
        assert!(matches!(v[1].shape, Shape::Rectangle { extents: [5.85, 5.1, 4.8], .. }));
    }

    #[test]
    fn reads_a_script_group_with_its_lists() {
        let mut p = vec![0u8; 0x59C + 0x44];
        put(&mut p, 0x594, &1u32.to_le_bytes());
        put(&mut p, 0x598, &0x59Cu32.to_le_bytes());
        let r = 0x59C;
        put(&mut p, r, &1u32.to_le_bytes());
        put(&mut p, r + 0x08, &12u32.to_le_bytes());
        // Triggers [4]; drops [3, -1].
        let at = p.len();
        put(&mut p, at, &4i32.to_le_bytes());
        put(&mut p, r + 0x14, &1u32.to_le_bytes());
        put(&mut p, r + 0x18, &(at as u32).to_le_bytes());
        let at = p.len();
        put(&mut p, at, &3i32.to_le_bytes());
        put(&mut p, at + 4, &(-1i32).to_le_bytes());
        put(&mut p, r + 0x3C, &2u32.to_le_bytes());
        put(&mut p, r + 0x40, &(at as u32).to_le_bytes());
        let (b, top) = chunk(SCRIPT_GROUPS, p);
        let g = script_groups(&b, &top);
        assert_eq!(g.len(), 1);
        assert_eq!((g[0].kind, g[0].key), (1, 12));
        assert_eq!(g[0].triggers, vec![4]);
        assert_eq!(g[0].drops, vec![3, -1]);
        assert!(g[0].locators.is_empty());
    }
}
