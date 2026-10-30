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

/// One outline record of the candidate list test (SPEC-0018 Ground probe step 3): the z-span
/// gate (origin z, scalar), then exactly one of Q . R > 0 (Q = (x, 0, z, 1); R given as its x,
/// z and w lanes) and R.x > 0.
fn outline_record_counts(x: f32, z: f32, oz: f32, scalar: f32, r: [f32; 3]) -> bool {
    let dz = z - oz;
    let span = if oz < z { dz < scalar } else { dz >= scalar };
    span && ((x * r[0] + z * r[1] + r[2] > 0.0) != (r[0] > 0.0))
}

/// The height patches of a type-5 model-set payload (count +0x04, entry table +0x08).
fn read_patches(set: &[u8]) -> Option<Vec<Option<Patch>>> {
    let count = (u32_at(set, 4)? as usize).min(MAX_ENTRIES);
    let table = u32_at(set, 8)? as usize;
    Some(
        (0..count)
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
            .collect(),
    )
}

/// A placed object's inline model set, registered under its patch id (the first word of its
/// payload; SPEC-0018 Patch-id table), with the object it belongs to.
struct OwnedSet {
    id: i32,
    object: usize,
    set: Vec<u8>,
    patches: Vec<Option<Patch>>,
}

/// An owner object's two transform blocks for the probe's owner placement (SPEC-0018 Ground
/// probe step 4), column-major: its current block (+0x100) and its rest block (+0xC0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnerPose {
    pub current: [f32; 16],
    pub rest: [f32; 16],
}

impl OwnerPose {
    /// The sampling point p' in the rest frame and the height raise O.y - T.y.
    pub fn map(&self, x: f32, z: f32) -> (f32, f32, f32) {
        let (c, r) = (&self.current, &self.rest);
        let (ox, oz) = (c[12], c[14]);
        let d1 = c[0] * (x - ox) + c[2] * (z - oz);
        let d2 = c[8] * (x - ox) + c[10] * (z - oz);
        (r[12] + d1 * r[0] + d2 * r[8], r[14] + d1 * r[2] + d2 * r[10], c[13] - r[13])
    }
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

/// A world pack's ground: its grid, the patches of its own model set (patch id 0) and those of
/// the placed objects' inline model sets (registered with `add_object_set`).
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
    owned: Vec<OwnedSet>,
    /// The world collision mask (+0x56B4, written by SetWorldCollisMask; 0 at load) that the
    /// model-set entry gate (+0x42) is tested against (SPEC-0018).
    pub mask: u16,
}

/// SPEC-0018 model-set entry gate: the entry's u16 at +0x42 passes when it is 0 or shares a bit
/// with the world mask.
fn gate_passes(set: &[u8], entry: usize, mask: u16) -> bool {
    let gate = set.get(entry + 0x42..entry + 0x44).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
    gate == 0 || gate & mask != 0
}

impl Ground {
    /// The pack's first type-19 grid and first top-level type-5 model set (copied).
    pub fn of(bytes: &[u8], top: &[Chunk], order: CellOrder) -> Option<Self> {
        let grid = &bytes[top.iter().find(|c| c.kind == GRID)?.payload.clone()];
        let set = &bytes[top.iter().find(|c| c.kind == MODEL_SET)?.payload.clone()];
        let patches = read_patches(set)?;
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
            owned: Vec::new(),
            mask: 0,
        })
    }
}

/// Height and normal of one patch of the set payload `set` at (x, z).
fn sample_in(set: &[u8], p: &Patch, x: f32, z: f32) -> Option<(f32, [f32; 3])> {
    {
        if x < p.x0 || z < p.z0 {
            return None;
        }
        let cx = ((x - p.x0) / p.cell).floor() as i64;
        let cz = ((z - p.z0) / p.cell).floor() as i64;
        if cx >= p.a as i64 || cz >= p.b as i64 {
            return None;
        }
        let cell_at = u32_at(set, p.at + p.cells + 4 * (cx + cz * p.a as i64) as usize)? as usize;
        if cell_at == 0 {
            return None;
        }
        let cell = p.at + cell_at;
        let heights = cell + u32_at(set, cell)? as usize;
        let lx = x - p.x0 - cx as f32 * p.cell;
        let lz = z - p.z0 - cz as f32 * p.cell;
        let ix = (lx / p.spacing).trunc() as usize;
        let iz = (lz / p.spacing).trunc() as usize;
        let fx = lx / p.spacing - ix as f32;
        let fz = lz / p.spacing - iz as f32;
        let h = |i: usize, j: usize| f32_at(set, heights + 4 * (i + j * p.stride));
        let (h00, h10, h01, h11) = (h(ix, iz)?, h(ix + 1, iz)?, h(ix, iz + 1)?, h(ix + 1, iz + 1)?);
        let top = h00 * (1.0 - fx) + h10 * fx;
        let bottom = h01 * (1.0 - fx) + h11 * fx;
        let r = [
            (-((h10 - h00) + (h11 - h01)) / 2.0).trunc(),
            p.spacing.trunc(),
            (-((h01 - h00) + (h11 - h10)) / 2.0).trunc(),
        ];
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        let n = if l > 0.0 { r.map(|c| c / l) } else { [0.0; 3] };
        Some((top * (1.0 - fz) + bottom * fz, n))
    }
}

impl Ground {
    /// Ground probe with ceiling `c`: the highest sampled height above the best so far and at
    /// most `c`; if none qualifies, the last rejected height; MISS when nothing was sampled; 0.0
    /// when the grid has no cell or entries here (SPEC-0018, Ground probe). Placed objects'
    /// patches are skipped here (see `probe_owned`); the entry gate is applied against `mask`, the
    /// count gate is not.
    pub fn probe(&self, x: f32, z: f32, c: f32) -> f32 {
        self.probe_normal(x, z, c).0
    }

    /// The probe with its surface normal: the chosen candidate's (or the last rejected one's);
    /// straight up when the grid gives no cell or no candidate is sampled.
    pub fn probe_normal(&self, x: f32, z: f32, c: f32) -> (f32, [f32; 3]) {
        let hit = self.probe_hit(x, z, c);
        (hit.height, hit.normal)
    }

    /// The grid's extent: x min, z min, x max, z max.
    pub fn bounds(&self) -> [f32; 4] {
        [self.x_min, self.z_min, self.x_min + self.cell * self.nx as f32, self.z_min + self.cell * self.nz as f32]
    }

    /// The grid cell record containing (x, z), if the cell is not empty.
    fn cell_at(&self, x: f32, z: f32) -> Option<usize> {
        let cx = ((x - self.x_min) / self.cell).floor() as i64;
        let cz = ((z - self.z_min) / self.cell).floor() as i64;
        if cx < 0 || cz < 0 || cx >= self.nx as i64 || cz >= self.nz as i64 {
            return None;
        }
        let index = match self.order {
            CellOrder::XMajor => cx + cz * self.nx as i64,
            CellOrder::ZMajor => cz + cx * self.nz as i64,
        } as usize;
        u32_at(&self.grid, self.table + 4 * index).filter(|&r| r != 0).map(|r| r as usize)
    }

    /// The deadly-ground entries of the model set (kind 2 at +0x38): their respawn entry index
    /// (+0x0D) and surface word (+0x40) (SPEC-0055 Layout).
    pub fn deadly_entries(&self) -> Vec<(u8, u16)> {
        self.patches
            .iter()
            .flatten()
            .filter(|p| u32_at(&self.set, p.at + 0x38) == Some(2))
            .map(|p| (self.set.get(p.at + 0x0D).copied().unwrap_or(0xFF), self.set.get(p.at + 0x40..p.at + 0x42).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]))))
            .collect()
    }

    /// The probe with the chosen entry's ground kind (the u32 at entry +0x38, which selects
    /// the camera profile in SPEC-0027).
    pub fn probe_hit(&self, x: f32, z: f32, c: f32) -> ProbeHit {
        self.probe_owned(x, z, c, &|_| None)
    }

    /// Register a placed object's inline model set (its type-5 payload) under its patch id.
    pub fn add_object_set(&mut self, object: usize, set: &[u8]) {
        let (Some(id), Some(patches)) = (i32_at(set, 0), read_patches(set)) else { return };
        if id != 0 {
            self.owned.push(OwnedSet { id, object, set: set.to_vec(), patches });
        }
    }

    /// The registered patch ids and their objects.
    pub fn object_sets(&self) -> Vec<(i32, usize)> {
        self.owned.iter().map(|o| (o.id, o.object)).collect()
    }

    /// The probe with placed-object patches (SPEC-0018 Ground probe steps 2 and 4): an entry
    /// whose patch id names a placed object's inline model set is tried only while `owner`
    /// gives that object's pose (the object drawn, its draw-test byte set); the point is mapped
    /// into the object's rest frame and the height raised by the frames' height difference.
    pub fn probe_owned(&self, x: f32, z: f32, c: f32, owner: &dyn Fn(usize) -> Option<OwnerPose>) -> ProbeHit {
        const UP: [f32; 3] = [0.0, 1.0, 0.0];
        let none = |height| ProbeHit { height, normal: UP, kind: None, found: false, respawn: None, surface: None, zone: None };
        let Some(rec) = self.cell_at(x, z) else {
            return none(0.0);
        };
        let count = i32_at(&self.grid, rec + 0x08).unwrap_or(0).max(0) as usize;
        let entries = rec + u32_at(&self.grid, rec + 0x0C).unwrap_or(0) as usize;
        if count == 0 {
            return none(0.0);
        }
        let (mut best, mut rejected, mut sampled) = (none(MISS), None, false);
        for e in 0..count.min(MAX_ENTRIES) {
            let at = entries + CELL_ENTRY * e;
            let (Some(id), Some(index)) = (i32_at(&self.grid, at), i32_at(&self.grid, at + 4)) else {
                continue;
            };
            // The patch id: 0 the pack's own set, else a placed object's set, tried only while
            // the object is drawn, at the point mapped into its rest frame.
            let (set, patches, pose) = if id == 0 {
                (&self.set[..], &self.patches, None)
            } else {
                let Some(o) = self.owned.iter().find(|o| o.id == id) else { continue };
                let Some(pose) = owner(o.object) else { continue };
                (&o.set[..], &o.patches, Some(pose))
            };
            let Some(Some(p)) = patches.get(index as usize) else { continue };
            // Ground probe step 2: a height patch is a candidate only when its entry's gate
            // passes.
            if !gate_passes(set, p.at, self.mask) {
                continue;
            }
            let (sx, sz, raise) = pose.map_or((x, z, 0.0), |q| q.map(x, z));
            // SPEC-0018 Ground probe step 3: with two or more entries in the cell, the patch must
            // pass its outline list test.
            if count >= 2 && !list_test_in(set, p, sx, sz) {
                continue;
            }
            let Some((h, n)) = sample_in(set, p, sx, sz) else { continue };
            let h = h + raise;
            sampled = true;
            let hit = ProbeHit {
                height: h,
                normal: n,
                kind: u32_at(set, p.at + 0x38),
                found: true,
                respawn: set.get(p.at + 0x0D).copied(),
                surface: set.get(p.at + 0x40..p.at + 0x42).map(|b| u16::from_le_bytes([b[0], b[1]])),
                zone: set.get(p.at + 0x0C).copied(),
            };
            if h > best.height && h <= c {
                best = hit;
            } else {
                rejected = Some(hit);
            }
        }
        match (sampled, best.height > MISS) {
            (false, _) => none(MISS),
            (true, true) => best,
            (true, false) => rejected.unwrap_or(none(MISS)),
        }
    }
}

/// The candidate list test over the set payload `set`.
fn list_test_in(set: &[u8], p: &Patch, x: f32, z: f32) -> bool {
    {
        let n = i32_at(set, p.at + 0x50).unwrap_or(0);
        if n <= 0 {
            return true;
        }
        let base = p.at + u32_at(set, p.at + 0x54).unwrap_or(0) as usize;
        let f = |o: usize| u32_at(set, o).map(f32::from_bits);
        let mut hits = 0;
        for k in 0..(n as usize).min(4096) {
            let r = base + 0x70 * k;
            let (Some(oz), Some(scalar), Some(rx), Some(ry), Some(rz), Some(rw)) = (f(r + 0x08), f(r + 0x68), f(r + 0x10), f(r + 0x14), f(r + 0x18), f(r + 0x1C)) else { break };
            let _ = ry;
            if outline_record_counts(x, z, oz, scalar, [rx, rz, rw]) {
                hits += 1;
            }
        }
        hits % 2 == 1
    }
}

impl Ground {
    /// The exit value of the cell under (x, z): the low 16 bits of the word at cell record
    /// +0x18 (SPEC-0006 Pending step), or -1 when there is no cell.
    pub fn cell_value(&self, x: f32, z: f32) -> i32 {
        self.cell_at(x, z).and_then(|rec| u32_at(&self.grid, rec + 0x18)).map_or(-1, |w| (w & 0xFFFF) as i32)
    }

    /// SPEC-0018 Segment query: blocked when the endpoints differ in height by more than 50, or
    /// when the segment crosses a frame record of either endpoint's cell (type 6 skipped while
    /// `restrict` is set). Not applied: the +1.5 horizontal endpoint shift, whose coordinate the
    /// spec leaves open. Returns true when clear.
    pub fn segment_clear(&self, p0: [f32; 3], p1: [f32; 3], restrict: bool) -> bool {
        if (p0[1] - p1[1]).abs() > 50.0 {
            return false;
        }
        let (c0, c1) = (self.cell_at(p0[0], p0[2]), self.cell_at(p1[0], p1[2]));
        let mut frames = self.frames_at(p0[0], p0[2]);
        if c1 != c0 {
            frames.extend(self.frames_at(p1[0], p1[2]));
        }
        !frames.iter().any(|f| !(restrict && f.kind == 6) && f.crossed_by(p0, p1))
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
    use super::OwnerPose;

    #[test]
    fn aim_target_entries() {
        // A type-11 payload: n2 = 2 at +0x20, the table at 0x60, records at 0x80 and 0xD0.
        let mut p = vec![0u8; 0x120];
        p[0x20..0x24].copy_from_slice(&2i32.to_le_bytes());
        p[0x48..0x4C].copy_from_slice(&0x60u32.to_le_bytes());
        for (i, (model, rec, pt)) in [(-1i32, 0x80u32, [1.0f32, 2.0, 3.0]), (7, 0xD0, [-4.0, 5.5, 6.0])].into_iter().enumerate() {
            let e = 0x60 + 0x0C * i;
            p[e..e + 4].copy_from_slice(&model.to_le_bytes());
            p[e + 8..e + 12].copy_from_slice(&rec.to_le_bytes());
            for (k, v) in pt.iter().enumerate() {
                let at = rec as usize + 0x30 + 4 * k;
                p[at..at + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        let top = [crate::resource::Chunk { offset: 0, tag: *b"IDM", kind: super::OBJECT_HEADER, extra: 0, size: 0, role_id: 0, payload: 0..p.len() }];
        let t = super::aim_targets(&p, &top);
        assert_eq!(t.len(), 2);
        assert_eq!((t[0].model, t[0].mesh, t[0].point), (-1, 0, [1.0, 2.0, 3.0]));
        assert_eq!((t[1].model, t[1].point), (7, [-4.0, 5.5, 6.0]));
        // A record past the payload ends the list.
        p[0x6C + 8..0x6C + 12].copy_from_slice(&0x200u32.to_le_bytes());
        assert_eq!(super::aim_targets(&p, &top).len(), 1);
    }

    #[test]
    fn owner_mapping() {
        // SPEC-0018 step 4's vector: O = (10, 5, 20) with identity axes, p = (12, 23), T =
        // (100, 2, 200), A = (0, 0, 1), B = (-1, 0, 0): p' = (97, 202), raised by 3.
        let mut current = [0.0f32; 16];
        current[0] = 1.0;
        current[5] = 1.0;
        current[10] = 1.0;
        current[15] = 1.0;
        current[12..15].copy_from_slice(&[10.0, 5.0, 20.0]);
        let mut rest = [0.0f32; 16];
        rest[2] = 1.0;
        rest[5] = 1.0;
        rest[8] = -1.0;
        rest[15] = 1.0;
        rest[12..15].copy_from_slice(&[100.0, 2.0, 200.0]);
        assert_eq!(OwnerPose { current, rest }.map(12.0, 23.0), (97.0, 202.0, 3.0));
    }

    #[test]
    fn outline_record_rule() {
        // R = (1, 0, -5, w 0) here as (x 1, z 0, w -5): Q . R = x - 5. A point at x 3 gives -2
        // (not positive) with R.x positive: exactly one holds, so it counts; at x 7 both hold.
        assert!(outline_record_counts(3.0, 0.0, -1.0, 10.0, [1.0, 0.0, -5.0]));
        assert!(!outline_record_counts(7.0, 0.0, -1.0, 10.0, [1.0, 0.0, -5.0]));
        // The z-span gate: origin z below z needs z - oz under the scalar; else at least it.
        assert!(!outline_record_counts(3.0, 20.0, -1.0, 10.0, [1.0, 0.0, -5.0]));
        assert!(outline_record_counts(3.0, 0.0, 5.0, -10.0, [1.0, 0.0, -5.0]));
        assert!(!outline_record_counts(3.0, 0.0, 5.0, 0.0, [1.0, 0.0, -5.0]));
    }

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

/// A teleport plane (SPEC-0006 Entry records and teleport planes): a kind-7 rectangle whose
/// kind mask has bit 1 and whose class is not 500 or 501. Its position in the list is its plane
/// index, which is also the door number. Landing fields: reference point P (+0x30), normal F
/// (+0x40), width axis D (+0x50) with extent w (+0x28), height axis Q (+0x60) with extent h
/// (+0x24).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeleportPlane {
    pub p: [f32; 3],
    pub f: [f32; 3],
    pub d: [f32; 3],
    pub q: [f32; 3],
    pub w: f32,
    pub h: f32,
}

/// The teleport plane list of a pack, in disc order.
pub fn teleport_planes(bytes: &[u8], top: &[Chunk]) -> Vec<TeleportPlane> {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let n = i32_at(p, 0x14).unwrap_or(0).clamp(0, 1 << 14) as usize;
    let Some(list) = u32_at(p, 0x3C).map(|v| v as usize) else { return Vec::new() };
    let u16_at = |at: usize| p.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    (0..n)
        .filter_map(|i| {
            let r = list + i * 0x70;
            let (mask, class) = (u16_at(r + 0x12)?, u16_at(r + 0x1E)?);
            if mask & 0x0002 == 0 || class == 500 || class == 501 {
                return None;
            }
            let v = |at: usize| Some([f32_at(p, r + at)?, f32_at(p, r + at + 4)?, f32_at(p, r + at + 8)?]);
            Some(TeleportPlane { p: v(0x30)?, f: v(0x40)?, d: v(0x50)?, q: v(0x60)?, w: f32_at(p, r + 0x28)?, h: f32_at(p, r + 0x24)? })
        })
        .collect()
}

/// Entry record i of the type-11 chunk (SPEC-0006): position and facing, or None past the end.
pub fn entry_record(bytes: &[u8], top: &[Chunk], i: usize) -> Option<([f32; 3], [f32; 3])> {
    let p = &bytes[top.iter().find(|c| c.kind == OBJECT_HEADER)?.payload.clone()];
    if i >= i32_at(p, 0x24)?.max(0) as usize {
        return None;
    }
    let e = u32_at(p, 0x4C)? as usize + 0x50 * i;
    let v = |at: usize| Some([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?]);
    Some((v(e + 0x30)?, v(e + 0x40)?))
}

/// A rope-gun aim target (SPEC-0028 section 6, n2 entries; SPEC-0030 row 126): the registry
/// model id, the inline marker mesh offset (0 for none) and the target point (the locator-like
/// record's +0x30).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimTarget {
    pub model: i32,
    pub mesh: u32,
    pub point: [f32; 3],
}

/// The pack's aim-target list: the n2 entries (count +0x20, 0x0C-byte entries at the table
/// +0x48). The entry's +0x08 is read as a payload-relative offset like the header's tables; an
/// entry whose record falls outside the payload ends the list.
pub fn aim_targets(bytes: &[u8], top: &[Chunk]) -> Vec<AimTarget> {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let n = i32_at(p, 0x20).unwrap_or(0).clamp(0, 1 << 12) as usize;
    let Some(table) = u32_at(p, 0x48).map(|v| v as usize) else { return Vec::new() };
    let mut out = Vec::new();
    for i in 0..n {
        let e = table + 0x0C * i;
        let (Some(model), Some(mesh), Some(rec)) = (i32_at(p, e), u32_at(p, e + 4), u32_at(p, e + 8)) else { break };
        let at = rec as usize + 0x30;
        let (Some(x), Some(y), Some(z)) = (f32_at(p, at), f32_at(p, at + 4), f32_at(p, at + 8)) else { break };
        out.push(AimTarget { model, mesh, point: [x, y, z] });
    }
    out
}

/// The locator tag of entry record i (u16 at +0x1E; 0 for none; SPEC-0006 Locator lookup).
pub fn entry_tag(bytes: &[u8], top: &[Chunk], i: usize) -> Option<u16> {
    let p = &bytes[top.iter().find(|c| c.kind == OBJECT_HEADER)?.payload.clone()];
    if i >= i32_at(p, 0x24)?.max(0) as usize {
        return None;
    }
    let e = u32_at(p, 0x4C)? as usize + 0x50 * i + 0x1E;
    Some(u16::from_le_bytes(p.get(e..e + 2)?.try_into().ok()?))
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

/// A kind-8 fourth-list record (SPEC-0028 section 6.5; SPEC-0031 Placed sound emitters): an
/// ambient loop (sub-kind 0) or an adventure zone (sub-kind not 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ambient {
    pub centre: [f32; 3],
    pub radius: f32,
    pub inner: f32,
    pub factor: f32,
    pub cue: i32,
    pub positioned: bool,
    pub subkind: u8,
}

/// A kind-1 sphere emitter: a sphere record whose header flags have bit 0x10, its cue (header
/// +0x1E), the radius at +0x28 and the centre and radius at +0x30 and +0x3C.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereEmitter {
    pub centre: [f32; 3],
    pub radius: f32,
    pub hearing: f32,
    pub cue: i32,
}

/// The pack's placed sound emitters: the fourth list (pointer +0x54, count +0x2C) and the
/// sphere list (pointer +0x30, count +0x08) records with flag bit 0x10.
pub fn sound_emitters(bytes: &[u8], top: &[Chunk]) -> (Vec<Ambient>, Vec<SphereEmitter>) {
    let Some(c) = top.iter().find(|c| c.kind == OBJECT_HEADER) else { return (Vec::new(), Vec::new()) };
    let p = &bytes[c.payload.clone()];
    let v = |at: usize| Some([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?]);
    let mut ambients = Vec::new();
    let n4 = i32_at(p, 0x2C).unwrap_or(0).clamp(0, 1 << 14) as usize;
    if let Some(list) = u32_at(p, 0x54).map(|x| x as usize) {
        for i in 0..n4 {
            let r = list + i * 0x50;
            let (Some(centre), Some(radius), Some(inner), Some(factor), Some(cue)) = (v(r + 0x30), f32_at(p, r + 0x3C), f32_at(p, r + 0x40), f32_at(p, r + 0x44), i32_at(p, r + 0x48)) else { break };
            let byte = |o: usize| p.get(r + o).copied().unwrap_or(0);
            ambients.push(Ambient { centre, radius, inner, factor, cue, positioned: byte(0x4D) != 0, subkind: byte(0x4E) });
        }
    }
    let mut spheres = Vec::new();
    let n1 = i32_at(p, 0x08).unwrap_or(0).clamp(0, 1 << 14) as usize;
    if let Some(list) = u32_at(p, 0x30).map(|x| x as usize) {
        for i in 0..n1 {
            let r = list + i * 0x40;
            let Some(flags) = p.get(r + 0x12..r + 0x14).map(|b| u16::from_le_bytes([b[0], b[1]])) else { break };
            if flags & 0x10 == 0 {
                continue;
            }
            let (Some(centre), Some(radius), Some(hearing), Some(cue)) = (v(r + 0x30), f32_at(p, r + 0x3C), f32_at(p, r + 0x28), p.get(r + 0x1E..r + 0x20).map(|b| i16::from_le_bytes([b[0], b[1]]) as i32)) else { break };
            spheres.push(SphereEmitter { centre, radius, hearing, cue });
        }
    }
    (ambients, spheres)
}

/// A probe result: height, surface normal and the chosen patch's ground kind (+0x38).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbeHit {
    pub height: f32,
    pub normal: [f32; 3],
    pub kind: Option<u32>,
    /// A candidate was sampled: the world has ground here (SPEC-0006 world swap).
    pub found: bool,
    /// The entry's respawn entry index (+0x0D, 0xFF none) and surface word (+0x40), read by
    /// the deadly-ground rules (SPEC-0055 Layout).
    pub respawn: Option<u8>,
    pub surface: Option<u16>,
    /// The entry's zone number z: the low byte of its header word +0x0C (SPEC-0006 Zones).
    pub zone: Option<u8>,
}

/// A frame record (0x70 bytes): a wall edge named by a grid cell's entry items (SPEC-0018,
/// Items of an entry). Fields as used by the edge tests of SPEC-0033 Behavior 37: the start
/// point (+0x00), the face plane (+0x10, unit normal and offset), the run plane (+0x20, unit
/// direction along the edge and offset), the top plane (+0x30), the length L (+0x40), the
/// height value H (+0x44) and the type byte (+0x4C). The value of a plane at p is n·p + w.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameRecord {
    pub kind: i8,
    /// The model-set entry's gate (+0x42) the record belongs to.
    pub gate: u16,
    pub start: [f32; 3],
    pub face: [f32; 4],
    pub run: [f32; 4],
    pub top: [f32; 4],
    pub length: f32,
    pub height: f32,
    /// The surface byte (+0x4D) a hero shot's world contact reads for its cue (SPEC-0036 World
    /// contact, second list).
    pub surface: u8,
}

impl FrameRecord {
    fn plane(q: [f32; 4], p: [f32; 3]) -> f32 {
        q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
    }

    /// The top-plane value at p: 0 at the wall's bottom, H at its top edge (the start point).
    /// On disc the start point lies on the face and run planes and has top-plane value H, so
    /// the record is a wall of depth H hanging below its top edge (`--ground` census).
    pub fn depth_at(&self, p: [f32; 3]) -> f32 {
        Self::plane(self.top, p)
    }

    /// SPEC-0018 Segment versus a rectangle, read onto a frame record: the face plane is the
    /// plane, the run plane gives s in [−0.25, L + 0.25] and the top plane u in [−0.25,
    /// H − 0.25]. Both faces block. The axis reading is this build's (Medium-Low in the spec).
    pub fn crossed_by(&self, p0: [f32; 3], p1: [f32; 3]) -> bool {
        let (d0, d1) = (Self::plane(self.face, p0), Self::plane(self.face, p1));
        if d0 == d1 || self.top[1] == 0.0 {
            return false;
        }
        let t = d0 / (d0 - d1);
        if !(-0.02..=1.02).contains(&t) {
            return false;
        }
        let q: [f32; 3] = std::array::from_fn(|k| p0[k] + t * (p1[k] - p0[k]));
        let s = Self::plane(self.run, q);
        let u = Self::plane(self.top, q);
        (-0.25..=self.length + 0.25).contains(&s) && (-0.25..=self.height - 0.25).contains(&u)
    }
}

impl Ground {
    /// The frame records named by the items of the cell containing (x, z), for the pack's own
    /// model set (patch id 0).
    pub fn frames_at(&self, x: f32, z: f32) -> Vec<FrameRecord> {
        self.cell_frames(x, z, &|id| (id == 0).then_some(&self.set[..])).into_iter().flat_map(|(_, f)| f).collect()
    }

    /// The frame records of the placed objects' inline model sets named in the cell containing
    /// (x, z), for each object `owner` gives a pose (the object drawn), in the object's rest
    /// frame (SPEC-0028 Character versus moving placed objects; SPEC-0018 Who walks what, item 4).
    pub fn object_frames_at(&self, x: f32, z: f32, owner: &dyn Fn(usize) -> Option<OwnerPose>) -> Vec<(OwnerPose, Vec<FrameRecord>)> {
        let mut out: Vec<(usize, OwnerPose, Vec<FrameRecord>)> = Vec::new();
        for (id, frames) in self.cell_frames(x, z, &|id| (id != 0).then(|| self.owned.iter().find(|o| o.id == id).map(|o| &o.set[..])).flatten()) {
            let Some(o) = self.owned.iter().find(|o| o.id == id) else { continue };
            let Some(pose) = owner(o.object) else { continue };
            match out.iter_mut().find(|e| e.0 == o.object) {
                Some(e) => e.2.extend(frames),
                None => out.push((o.object, pose, frames)),
            }
        }
        out.into_iter().map(|(_, p, f)| (p, f)).collect()
    }

    /// The frame records of each cell entry whose patch id `set_of` maps to a model set, by
    /// patch id; an entry whose gate fails gives none (SPEC-0018).
    fn cell_frames<'a>(&'a self, x: f32, z: f32, set_of: &dyn Fn(i32) -> Option<&'a [u8]>) -> Vec<(i32, Vec<FrameRecord>)> {
        let Some(rec) = self.cell_at(x, z) else {
            return Vec::new();
        };
        let count = i32_at(&self.grid, rec + 0x08).unwrap_or(0).clamp(0, 1 << 12) as usize;
        let entries = rec + u32_at(&self.grid, rec + 0x0C).unwrap_or(0) as usize;
        let mut out = Vec::new();
        for e in 0..count {
            let at = entries + CELL_ENTRY * e;
            let (Some(id), Some(index)) = (i32_at(&self.grid, at), i32_at(&self.grid, at + 4)) else {
                continue;
            };
            let Some(set) = set_of(id) else { continue };
            let table = u32_at(set, 8).unwrap_or(0) as usize;
            let Some(entry) = u32_at(set, table + 4 * index as usize).map(|v| v as usize) else { continue };
            if !gate_passes(set, entry, self.mask) {
                continue;
            }
            let gate = set.get(entry + 0x42..entry + 0x44).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
            let n = i32_at(&self.grid, at + 0x0C).unwrap_or(0).clamp(0, 1 << 12) as usize;
            let items = entries + u32_at(&self.grid, at + 0x10).unwrap_or(0) as usize;
            let mut frames = Vec::new();
            for k in 0..n {
                let Some(item) = u32_at(&self.grid, items + 4 * k) else { break };
                let r = entry + item as usize;
                let v = |o: usize| Some([f32_at(set, r + o)?, f32_at(set, r + o + 4)?, f32_at(set, r + o + 8)?, f32_at(set, r + o + 12)?]);
                if let (Some(start), Some(face), Some(run), Some(top), Some(length), Some(height), Some(kind)) =
                    (v(0x00), v(0x10), v(0x20), v(0x30), f32_at(set, r + 0x40), f32_at(set, r + 0x44), set.get(r + 0x4C).copied())
                {
                    let surface = set.get(r + 0x4D).copied().unwrap_or(0);
                    frames.push(FrameRecord { kind: kind as i8, gate, start: [start[0], start[1], start[2]], face, run, top, length, height, surface });
                }
            }
            out.push((id, frames));
        }
        out
    }
}

#[cfg(test)]
mod frame_tests {
    use super::FrameRecord;

    /// A wall in the plane x = 0, running along +z for 10 from its top corner (0, 5, 0), 5 deep:
    /// the top-plane value is y and H = 5.
    fn wall(kind: i8) -> FrameRecord {
        FrameRecord {
            kind,
            gate: 0,
            start: [0.0, 5.0, 0.0],
            face: [1.0, 0.0, 0.0, 0.0],
            run: [0.0, 0.0, 1.0, 0.0],
            top: [0.0, 1.0, 0.0, 0.0],
            length: 10.0,
            height: 5.0,
            surface: 0,
        }
    }

    #[test]
    fn segment_crosses_inside_the_margins() {
        let w = wall(0);
        assert!(w.crossed_by([-1.0, 2.0, 5.0], [1.0, 2.0, 5.0]));
        assert!(w.crossed_by([-1.0, 2.0, 10.2], [1.0, 2.0, 10.2]), "run margin +0.25");
        assert!(!w.crossed_by([-1.0, 2.0, 10.3], [1.0, 2.0, 10.3]));
        assert!(!w.crossed_by([-1.0, 4.8, 5.0], [1.0, 4.8, 5.0]), "top margin: u above H - 0.25");
        assert!(!w.crossed_by([-1.0, 2.0, 5.0], [-0.5, 2.0, 5.0]), "t beyond 1.02");
        assert!(!w.crossed_by([0.0, 1.0, 5.0], [0.0, 3.0, 5.0]), "parallel to the plane");
        assert_eq!(w.depth_at(w.start), w.height);
    }
}
