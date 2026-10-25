//! Foliage patches (SPEC-0060 Layout; SPEC-0006 Box records): the type-16 chunks of a world
//! pack. Every stored offset is relative to the chunk header, 16 bytes before the payload.
pub const CHUNK_TYPE: u16 = 16;
/// Patches per chunk and rows per patch stay far below these on disc.
const MAX_PATCHES: usize = 4096;
const MAX_ROWS: usize = 256;
const MAX_PLANTS: usize = 4096;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

/// A plant: its x offset from the box's x0 and its colour bytes (R, G, B, A; 128 = full).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plant {
    pub offset: f32,
    pub colour: [u8; 4],
}

/// A row: its z offset from the box's z0 and its plants.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub offset: f32,
    pub plants: Vec<Plant>,
}

/// A patch record (0x40 bytes).
#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    /// x0, x1, z0, z1.
    pub area: [f32; 4],
    pub key: u32,
    /// The base height y.
    pub height: f32,
    pub zone: u32,
    pub jitter: bool,
    /// Draw-distance overrides A, B, C, D (used when A or D is nonzero).
    pub overrides: [f32; 4],
    pub rows: Vec<Row>,
}

/// Read one type-16 chunk: `header` is the chunk header's offset in `bytes`, `payload` its
/// payload range. A record or row that runs out of the chunk ends the read.
pub fn chunk(bytes: &[u8], header: usize, payload: std::ops::Range<usize>) -> Vec<Patch> {
    let Some(p) = bytes.get(payload.clone()) else { return Vec::new() };
    let end = payload.end;
    let n = u32_at(p, 0).unwrap_or(0) as usize;
    let table = u32_at(p, 4).unwrap_or(0x20) as usize;
    let mut out = Vec::new();
    for k in 0..n.min(MAX_PATCHES) {
        let r = header + table + 0x40 * k;
        if r + 0x40 > end {
            break;
        }
        let f = |o: usize| f32_at(bytes, r + o).unwrap_or(0.0);
        let u = |o: usize| u32_at(bytes, r + o).unwrap_or(0);
        let mut rows = Vec::new();
        let (count, first) = (u(0x10) as usize, u(0x14) as usize);
        for j in 0..count.min(MAX_ROWS) {
            let e = header + first + 0x14 * j;
            if e + 0x14 > end {
                break;
            }
            let plants_at = header + u32_at(bytes, e + 0x10).unwrap_or(0) as usize;
            let m = (u32_at(bytes, e).unwrap_or(0) as usize).min(MAX_PLANTS);
            if plants_at + 16 * m > end {
                break;
            }
            let plants = (0..m)
                .map(|i| {
                    let a = plants_at + 16 * i;
                    Plant { offset: f32_at(bytes, a).unwrap_or(0.0), colour: bytes[a + 8..a + 12].try_into().unwrap() }
                })
                .collect();
            rows.push(Row { offset: f32_at(bytes, e + 4).unwrap_or(0.0), plants });
        }
        out.push(Patch {
            area: [f(0), f(4), f(8), f(0xC)],
            key: u(0x18),
            height: f(0x1C),
            zone: u(0x20),
            jitter: u(0x24) != 0,
            overrides: [f(0x28), f(0x2C), f(0x30), f(0x34)],
            rows,
        });
    }
    out
}

/// Every patch of every type-16 chunk in a world pack's top chain.
pub fn patches(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Vec<Patch> {
    chain.iter().filter(|c| c.kind == CHUNK_TYPE).flat_map(|c| chunk(bytes, c.offset, c.payload.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One patch with two rows (the layout of SPEC-0006 Box records, synthetic values).
    #[test]
    fn reads_patch_rows_and_plants() {
        let mut b = vec![0u8; 0x200];
        let put = |b: &mut Vec<u8>, at: usize, v: &[u8]| b[at..at + v.len()].copy_from_slice(v);
        // Chunk header at 0, payload from 0x10.
        put(&mut b, 0x10, &1u32.to_le_bytes());
        put(&mut b, 0x14, &0x20u32.to_le_bytes());
        let r = 0x20;
        for (k, v) in [-10.0f32, -5.0, 20.0, 24.0].iter().enumerate() {
            put(&mut b, r + 4 * k, &v.to_le_bytes());
        }
        put(&mut b, r + 0x10, &2u32.to_le_bytes());
        put(&mut b, r + 0x14, &0x60u32.to_le_bytes());
        put(&mut b, r + 0x18, &9u32.to_le_bytes());
        put(&mut b, r + 0x1C, &74.0f32.to_le_bytes());
        // Rows at 0x60: (2 plants, z 1.5, at 0x90), (1 plant, z 3.0, at 0xB0).
        for (j, (m, z, at)) in [(2u32, 1.5f32, 0x90u32), (1, 3.0, 0xB0)].iter().enumerate() {
            let e = 0x60 + 0x14 * j;
            put(&mut b, e, &m.to_le_bytes());
            put(&mut b, e + 4, &z.to_le_bytes());
            put(&mut b, e + 0x10, &at.to_le_bytes());
        }
        put(&mut b, 0x90, &1.25f32.to_le_bytes());
        put(&mut b, 0x98, &0x8040_4040u32.to_le_bytes());
        put(&mut b, 0xA0, &3.0f32.to_le_bytes());
        put(&mut b, 0xB0, &4.5f32.to_le_bytes());
        let ps = chunk(&b, 0, 0x10..0x200);
        assert_eq!(ps.len(), 1);
        let p = &ps[0];
        assert_eq!((p.key, p.height, p.area), (9, 74.0, [-10.0, -5.0, 20.0, 24.0]));
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.rows[0].plants[0], Plant { offset: 1.25, colour: [64, 64, 64, 128] });
        assert_eq!((p.rows[1].offset, p.rows[1].plants[0].offset), (3.0, 4.5));
    }
}
