//! Ground height (SPEC-0018) and the hero's arrival point (SPEC-0006): the level grid (type 19)
//! names, per cell, the type-5 height patches that cover it; the probe samples them and keeps
//! the highest height at or below a caller's ceiling.
use crate::resource::Chunk;

pub const GRID: u16 = 19;
pub const MODEL_SET: u16 = 5;
pub const OBJECT_HEADER: u16 = 11;
/// The probe's miss value: nothing was sampled.
pub const MISS: f32 = -1.0e9;
const CELL_ENTRY: usize = 0x14;
/// Grid sizes on disc stay far below these; they bound hostile input.
const MAX_CELLS: usize = 1 << 22;
const MAX_ENTRIES: usize = 1 << 16;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    u32_at(b, at).map(|v| v as i32)
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

/// Order of the level grid's cell table. SPEC-0006 leaves it open (x- or z-major); the
/// inspector's `--ground` check measures which one puts the probe on the navmesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellOrder {
    XMajor,
    ZMajor,
}

/// One type-5 height patch.
#[derive(Clone, Copy, Debug)]
struct Patch {
    /// Offset of the entry within the model-set payload.
    at: usize,
    cell: f32,
    spacing: f32,
    stride: usize,
    a: i32,
    b: i32,
    x0: f32,
    z0: f32,
    cells: usize,
}

/// A world pack's ground: its grid and the patches of its own model set (patch id 0). Patches
/// of placed objects' inline model sets (moving platforms and the like) are not read yet.
pub struct Ground {
    grid: Vec<u8>,
    set: Vec<u8>,
    x_min: f32,
    z_min: f32,
    cell: f32,
    nx: i32,
    nz: i32,
    table: usize,
    pub order: CellOrder,
    patches: Vec<Option<Patch>>,
}

impl Ground {
    /// The pack's first type-19 grid and first top-level type-5 model set (copied).
    pub fn of(bytes: &[u8], top: &[Chunk], order: CellOrder) -> Option<Self> {
        let grid = &bytes[top.iter().find(|c| c.kind == GRID)?.payload.clone()];
        let set = &bytes[top.iter().find(|c| c.kind == MODEL_SET)?.payload.clone()];
        let count = (u32_at(set, 4)? as usize).min(MAX_ENTRIES);
        let table = u32_at(set, 8)? as usize;
        let patches = (0..count)
            .map(|i| {
                let at = u32_at(set, table + 4 * i)? as usize;
                let p = Patch {
                    at,
                    cell: f32_at(set, at + 0x10)?,
                    spacing: f32_at(set, at + 0x14)?,
                    stride: i32_at(set, at + 0x1C)?.max(0) as usize,
                    a: i32_at(set, at + 0x20)?,
                    b: i32_at(set, at + 0x24)?,
                    x0: f32_at(set, at + 0x28)?,
                    z0: f32_at(set, at + 0x2C)?,
                    cells: u32_at(set, at + 0x4C)? as usize,
                };
                (p.a > 0 && p.b > 0 && p.cells != 0 && p.cell > 0.0 && p.spacing > 0.0).then_some(p)
            })
            .collect();
        let nx = i32_at(grid, 0x14)?;
        let nz = i32_at(grid, 0x18)?;
        if nx <= 0 || nz <= 0 || nx as usize * nz as usize > MAX_CELLS {
            return None;
        }
        Some(Self {
            grid: grid.to_vec(),
            set: set.to_vec(),
            x_min: f32_at(grid, 0)?,
            z_min: f32_at(grid, 8)?,
            cell: f32_at(grid, 0x10)?,
            nx,
            nz,
            table: u32_at(grid, 0x1C)? as usize,
            order,
            patches,
        })
    }

    /// Height of one patch at (x, z): bilinear over the four samples of the sample square, or
    /// None outside the patch or on an empty cell (SPEC-0018, Height at a point).
    fn sample(&self, p: &Patch, x: f32, z: f32) -> Option<f32> {
        if x < p.x0 || z < p.z0 {
            return None;
        }
        let cx = ((x - p.x0) / p.cell).floor() as i64;
        let cz = ((z - p.z0) / p.cell).floor() as i64;
        if cx >= p.a as i64 || cz >= p.b as i64 {
            return None;
        }
        let cell_at = u32_at(&self.set, p.at + p.cells + 4 * (cx + cz * p.a as i64) as usize)? as usize;
        if cell_at == 0 {
            return None;
        }
        let cell = p.at + cell_at;
        let heights = cell + u32_at(&self.set, cell)? as usize;
        let lx = x - p.x0 - cx as f32 * p.cell;
        let lz = z - p.z0 - cz as f32 * p.cell;
        let ix = (lx / p.spacing).trunc() as usize;
        let iz = (lz / p.spacing).trunc() as usize;
        let fx = lx / p.spacing - ix as f32;
        let fz = lz / p.spacing - iz as f32;
        let h = |i: usize, j: usize| f32_at(&self.set, heights + 4 * (i + j * p.stride));
        let top = h(ix, iz)? * (1.0 - fx) + h(ix + 1, iz)? * fx;
        let bottom = h(ix, iz + 1)? * (1.0 - fx) + h(ix + 1, iz + 1)? * fx;
        Some(top * (1.0 - fz) + bottom * fz)
    }

    /// Ground probe with ceiling `c`: the highest sampled height above the best so far and at
    /// most `c`; if none qualifies, the last rejected height; MISS when nothing was sampled; 0.0
    /// when the grid has no cell or entries here (SPEC-0018, Ground probe). Not implemented:
    /// runtime gates (owner, mask, count), the candidate outline test, owner placement.
    pub fn probe(&self, x: f32, z: f32, c: f32) -> f32 {
        let cx = ((x - self.x_min) / self.cell).floor() as i64;
        let cz = ((z - self.z_min) / self.cell).floor() as i64;
        if cx < 0 || cz < 0 || cx >= self.nx as i64 || cz >= self.nz as i64 {
            return 0.0;
        }
        let index = match self.order {
            CellOrder::XMajor => cx + cz * self.nx as i64,
            CellOrder::ZMajor => cz + cx * self.nz as i64,
        } as usize;
        let Some(rec) = u32_at(&self.grid, self.table + 4 * index).filter(|&r| r != 0) else {
            return 0.0;
        };
        let rec = rec as usize;
        let count = i32_at(&self.grid, rec + 0x08).unwrap_or(0).max(0) as usize;
        let entries = rec + u32_at(&self.grid, rec + 0x0C).unwrap_or(0) as usize;
        if count == 0 {
            return 0.0;
        }
        let (mut best, mut rejected, mut sampled) = (MISS, None, false);
        for e in 0..count.min(MAX_ENTRIES) {
            let at = entries + CELL_ENTRY * e;
            let (Some(0), Some(index)) = (i32_at(&self.grid, at), i32_at(&self.grid, at + 4)) else {
                continue;
            };
            let Some(Some(p)) = self.patches.get(index as usize) else { continue };
            let Some(h) = self.sample(p, x, z) else { continue };
            sampled = true;
            if h > best && h <= c {
                best = h;
            } else {
                rejected = Some(h);
            }
        }
        match (sampled, best > MISS) {
            (false, _) => MISS,
            (true, true) => best,
            (true, false) => rejected.unwrap_or(MISS),
        }
    }
}

/// Entry record 0 of the type-11 chunk: the world spawn position and facing (SPEC-0006).
pub fn spawn(bytes: &[u8], top: &[Chunk]) -> Option<([f32; 3], [f32; 3])> {
    let p = &bytes[top.iter().find(|c| c.kind == OBJECT_HEADER)?.payload.clone()];
    if i32_at(p, 0x24)? <= 0 {
        return None;
    }
    let e = u32_at(p, 0x4C)? as usize;
    let v = |at: usize| Some([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?]);
    Some((v(e + 0x30)?, v(e + 0x40)?))
}

/// The spawn height rule: probe from y + 8; keep the record's y when the result is below -1e-7.
pub fn spawn_height(ground: &Ground, s: [f32; 3]) -> f32 {
    let h = ground.probe(s[0], s[2], s[1] + 8.0);
    if h < -1e-7 { s[1] } else { h }
}

/// Navigation-mesh vertices (type 21) of a pack, for checking the ground against them.
pub fn navmesh_vertices(bytes: &[u8], top: &[Chunk]) -> Vec<[f32; 3]> {
    let Some(c) = top.iter().find(|c| c.kind == 21) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let n = i32_at(p, 0).unwrap_or(0).clamp(0, 4096) as usize;
    let table = u32_at(p, 4).unwrap_or(0) as usize;
    let mut out = Vec::new();
    for i in 0..n {
        let Some(m) = u32_at(p, table + 4 * i).map(|v| v as usize) else { continue };
        let t = u32_at(p, m + 0x10).unwrap_or(0).min(1 << 16) as usize;
        let tris = m + u32_at(p, m + 0x14).unwrap_or(0) as usize;
        for k in 0..t * 3 {
            let v = tris + (k / 3) * 0x30 + (k % 3) * 0x10;
            if let (Some(x), Some(y), Some(z)) = (f32_at(p, v), f32_at(p, v + 4), f32_at(p, v + 8)) {
                out.push([x, y, z]);
            }
        }
    }
    out
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

    /// A model set with two one-cell patches over (0..10, 0..10), per = 1 (2 × 2 samples):
    /// patch 0 a slope from height 0 at x = 0 to 10 at x = 10; patch 1 flat at height 20.
    fn model_set() -> Vec<u8> {
        let mut s = vec![0u8; 0x10];
        put(&mut s, 4, &2u32.to_le_bytes());
        put(&mut s, 8, &0x10u32.to_le_bytes());
        for (k, heights) in [[0.0f32, 10.0, 0.0, 10.0], [20.0; 4]].iter().enumerate() {
            let at = 0x20 + k * 0x100;
            put(&mut s, 0x10 + 4 * k, &(at as u32).to_le_bytes());
            put(&mut s, at + 0x10, &10f32.to_le_bytes());
            put(&mut s, at + 0x14, &10f32.to_le_bytes());
            put(&mut s, at + 0x18, &1i32.to_le_bytes());
            put(&mut s, at + 0x1C, &2i32.to_le_bytes());
            put(&mut s, at + 0x20, &1i32.to_le_bytes());
            put(&mut s, at + 0x24, &1i32.to_le_bytes());
            put(&mut s, at + 0x4C, &0x60u32.to_le_bytes()); // cell table
            put(&mut s, at + 0x60, &0x70u32.to_le_bytes()); // cell 0 at entry + 0x70
            put(&mut s, at + 0x70, &0x10u32.to_le_bytes()); // heights at cell + 0x10
            for (i, h) in heights.iter().enumerate() {
                put(&mut s, at + 0x80 + 4 * i, &h.to_le_bytes());
            }
        }
        s
    }

    /// A 2 × 1 grid of 10-unit cells: cell 0 lists both patches, cell 1 none.
    fn grid() -> Vec<u8> {
        let mut g = vec![0u8; 0x30];
        for (i, v) in [0f32, 20.0, 0.0, 10.0, 10.0].iter().enumerate() {
            put(&mut g, 4 * i, &v.to_le_bytes());
        }
        put(&mut g, 0x14, &2i32.to_le_bytes());
        put(&mut g, 0x18, &1i32.to_le_bytes());
        put(&mut g, 0x1C, &0x30u32.to_le_bytes());
        put(&mut g, 0x30, &0x40u32.to_le_bytes());
        put(&mut g, 0x40 + 0x08, &2i32.to_le_bytes());
        put(&mut g, 0x40 + 0x0C, &0x18u32.to_le_bytes());
        for (k, patch) in [0i32, 1].iter().enumerate() {
            put(&mut g, 0x58 + CELL_ENTRY * k + 4, &patch.to_le_bytes());
        }
        put(&mut g, 0x80, &[0]);
        g
    }

    fn ground() -> Ground {
        let mut file = Vec::new();
        let mut top = Vec::new();
        for (kind, payload) in [(GRID, grid()), (MODEL_SET, model_set())] {
            let start = file.len();
            file.extend(payload);
            top.push(Chunk {
                offset: start,
                tag: *b"IDM\x01",
                kind,
                extra: 0,
                size: 0,
                role_id: 0,
                payload: start..file.len(),
            });
        }
        Ground::of(&file, &top, CellOrder::XMajor).unwrap()
    }

    #[test]
    fn samples_bilinearly_and_keeps_the_highest_height_under_the_ceiling() {
        let g = ground();
        // The slope gives x / 10 * 10 = x; the flat patch is 20.
        assert!((g.probe(2.5, 5.0, 10.0) - 2.5).abs() < 1e-5, "the flat 20 is above the ceiling");
        assert_eq!(g.probe(2.5, 5.0, 30.0), 20.0, "both qualify; the higher wins");
        // Nothing at or below the ceiling: the last rejected candidate's height.
        assert_eq!(g.probe(5.0, 5.0, -1.0), 20.0);
        // No cell record: 0.0; outside the grid: 0.0.
        assert_eq!(g.probe(15.0, 5.0, 100.0), 0.0);
        assert_eq!(g.probe(-5.0, 5.0, 100.0), 0.0);
    }

    #[test]
    fn spawn_height_rule() {
        let g = ground();
        // Probe from y + 8: ground at 2.5 under a record at y 3.
        assert!((spawn_height(&g, [2.5, 3.0, 5.0]) - 2.5).abs() < 1e-5);
        // A probe result below -1e-7 keeps the record's y.
        assert_eq!(spawn_height(&g, [15.0, 3.0, 5.0]), 0.0, "0.0 is not below -1e-7");
    }
}

/// A rectangle primitive of the type-11 chunk (SPEC-0028 section 6.1): kind 4 or 7, its header
/// flags and gate mask, low corner, unit axes V (+0x60), A (+0x50) and B (+0x40), and signed
/// extents along them (+0x24, +0x28, +0x2C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    pub kind: u8,
    pub flags: u16,
    pub mask: u16,
    pub origin: [f32; 3],
    pub axes: [[f32; 3]; 3],
    pub extents: [f32; 3],
}

impl Rectangle {
    /// The box-push gate of SPEC-0018: class 3 or class 0 (flag bits 3 or 0), the mask gate
    /// against the world mask, and a positive B extent (zero or less is a contact plane).
    pub fn pushes(&self, world_mask: u16) -> bool {
        self.flags & 0b1001 != 0
            && (self.mask == 0 || self.mask & world_mask != 0)
            && self.extents[2] > 0.0
    }
}

/// The kind-4 (+0x40, count +0x18) and kind-7 (+0x3C, count +0x14) rectangle lists of a pack.
pub fn rectangles(bytes: &[u8], top: &[Chunk]) -> Vec<Rectangle> {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let mut out = Vec::new();
    for (kind, count_at, list_at) in [(4u8, 0x18, 0x40), (7, 0x14, 0x3C)] {
        let n = i32_at(p, count_at).unwrap_or(0).clamp(0, 1 << 14) as usize;
        let Some(list) = u32_at(p, list_at).map(|v| v as usize) else { continue };
        for i in 0..n {
            let r = list + i * 0x70;
            let v = |at: usize| Some([f32_at(p, r + at)?, f32_at(p, r + at + 4)?, f32_at(p, r + at + 8)?]);
            let (Some(origin), Some(av), Some(aa), Some(ab), Some(e), Some(flags), Some(mask)) = (
                v(0x30),
                v(0x60),
                v(0x50),
                v(0x40),
                v(0x24),
                p.get(r + 0x12..r + 0x14).map(|b| u16::from_le_bytes([b[0], b[1]])),
                p.get(r + 0x1A..r + 0x1C).map(|b| u16::from_le_bytes([b[0], b[1]])),
            ) else {
                break;
            };
            out.push(Rectangle { kind, flags, mask, origin, axes: [av, aa, ab], extents: e });
        }
    }
    out
}

/// A frame record (0x70 bytes): a wall edge named by a grid cell's entry items (SPEC-0018,
/// Items of an entry). Fields as used by the edge tests of SPEC-0033 Behavior 37: the start
/// point (+0x00), the face plane (+0x10, unit normal and offset), the run plane (+0x20, unit
/// direction along the edge and offset), the top plane (+0x30), the length L (+0x40), the
/// height value H (+0x44) and the type byte (+0x4C). The value of a plane at p is n·p + w.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameRecord {
    pub kind: i8,
    pub start: [f32; 3],
    pub face: [f32; 4],
    pub run: [f32; 4],
    pub top: [f32; 4],
    pub length: f32,
    pub height: f32,
}

impl Ground {
    /// The frame records named by the items of the cell containing (x, z), for the pack's own
    /// model set (patch id 0).
    pub fn frames_at(&self, x: f32, z: f32) -> Vec<FrameRecord> {
        let cx = ((x - self.x_min) / self.cell).floor() as i64;
        let cz = ((z - self.z_min) / self.cell).floor() as i64;
        if cx < 0 || cz < 0 || cx >= self.nx as i64 || cz >= self.nz as i64 {
            return Vec::new();
        }
        let index = (cx + cz * self.nx as i64) as usize;
        let Some(rec) = u32_at(&self.grid, self.table + 4 * index).filter(|&r| r != 0) else {
            return Vec::new();
        };
        let rec = rec as usize;
        let count = i32_at(&self.grid, rec + 0x08).unwrap_or(0).clamp(0, 1 << 12) as usize;
        let entries = rec + u32_at(&self.grid, rec + 0x0C).unwrap_or(0) as usize;
        let table = u32_at(&self.set, 8).unwrap_or(0) as usize;
        let mut out = Vec::new();
        for e in 0..count {
            let at = entries + CELL_ENTRY * e;
            let (Some(0), Some(index)) = (i32_at(&self.grid, at), i32_at(&self.grid, at + 4)) else {
                continue;
            };
            let Some(entry) = u32_at(&self.set, table + 4 * index as usize).map(|v| v as usize) else { continue };
            let n = i32_at(&self.grid, at + 0x0C).unwrap_or(0).clamp(0, 1 << 12) as usize;
            let items = entries + u32_at(&self.grid, at + 0x10).unwrap_or(0) as usize;
            for k in 0..n {
                let Some(item) = u32_at(&self.grid, items + 4 * k) else { break };
                let r = entry + item as usize;
                let v = |o: usize| {
                    Some([f32_at(&self.set, r + o)?, f32_at(&self.set, r + o + 4)?, f32_at(&self.set, r + o + 8)?, f32_at(&self.set, r + o + 12)?])
                };
                if let (Some(start), Some(face), Some(run), Some(top), Some(length), Some(height), Some(kind)) = (
                    v(0x00),
                    v(0x10),
                    v(0x20),
                    v(0x30),
                    f32_at(&self.set, r + 0x40),
                    f32_at(&self.set, r + 0x44),
                    self.set.get(r + 0x4C).copied(),
                ) {
                    out.push(FrameRecord { kind: kind as i8, start: [start[0], start[1], start[2]], face, run, top, length, height });
                }
            }
        }
        out
    }
}
