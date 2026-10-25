//! Navigation meshes (SPEC-0018 Navigation meshes, chunk type 21; SPEC-0056 Route table): per
//! mesh its kind (1 ground, 2 flying), triangles of three vertices with the neighbour index
//! across each edge (−1 boundary), and the route table (2 bits per ordered triangle pair: the
//! edge to leave by).
use crate::resource::Chunk;

pub const NAVMESH: u16 = 21;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavMeshData {
    pub kind: u32,
    pub triangles: Vec<[[f32; 3]; 3]>,
    pub neighbours: Vec<[i32; 3]>,
    /// The route-table bytes (the record's bit block).
    pub table: Vec<u8>,
}

impl NavMeshData {
    /// The edge of triangle a to leave by toward triangle b (SPEC-0056 Route table).
    pub fn route(&self, a: usize, b: usize) -> Option<u8> {
        let t = self.triangles.len();
        if a >= t || b >= t {
            return None;
        }
        let k = 2 * (a * t + b);
        Some((self.table.get(k / 8)? >> (k % 8)) & 3)
    }
}

/// All navigation meshes of a pack (its first type-21 chunk).
pub fn navmeshes(bytes: &[u8], top: &[Chunk]) -> Vec<NavMeshData> {
    let Some(c) = top.iter().find(|c| c.kind == NAVMESH) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let n = u32_at(p, 0).unwrap_or(0).min(1 << 12) as usize;
    let Some(table) = u32_at(p, 4).map(|t| t as usize) else { return Vec::new() };
    (0..n)
        .filter_map(|i| {
            let r = u32_at(p, table + 4 * i)? as usize;
            let kind = u32_at(p, r + 0x0C)?;
            let t = u32_at(p, r + 0x10)?.min(1 << 16) as usize;
            let tris = r + u32_at(p, r + 0x14)? as usize;
            let bits = r + u32_at(p, r + 0x18)? as usize;
            let mut triangles = Vec::with_capacity(t);
            let mut neighbours = Vec::with_capacity(t);
            for k in 0..t {
                let at = tris + 0x30 * k;
                let v = |j: usize| Some([f32_at(p, at + 16 * j)?, f32_at(p, at + 16 * j + 4)?, f32_at(p, at + 16 * j + 8)?]);
                triangles.push([v(0)?, v(1)?, v(2)?]);
                let nb = |j: usize| f32_at(p, at + 16 * j + 12).map(|f| f as i32);
                neighbours.push([nb(0)?, nb(1)?, nb(2)?]);
            }
            let table = p.get(bits..tris.max(bits))?.to_vec();
            Some(NavMeshData { kind, triangles, neighbours, table })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_bits_follow_the_spec_vector() {
        // SPEC-0056 Route table test vector (pack 722, mesh 2, T = 5): table rows "from a".
        let rows = [[0u8, 1, 1, 1, 1], [2, 0, 2, 0, 0], [0, 1, 0, 1, 1], [2, 2, 2, 0, 0], [1, 1, 1, 1, 0]];
        let mut table = vec![0u8; 7];
        for (a, row) in rows.iter().enumerate() {
            for (b, &v) in row.iter().enumerate() {
                let k = 2 * (a * 5 + b);
                table[k / 8] |= v << (k % 8);
            }
        }
        let mesh = NavMeshData { kind: 1, triangles: vec![[[0.0; 3]; 3]; 5], neighbours: vec![[-1; 3]; 5], table };
        assert_eq!(mesh.route(1, 3), Some(0));
        assert_eq!(mesh.route(0, 4), Some(1));
        assert_eq!(mesh.route(3, 1), Some(2));
        assert_eq!(mesh.route(5, 0), None);
    }
}
