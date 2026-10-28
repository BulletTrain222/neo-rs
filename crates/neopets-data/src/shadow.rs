//! Shadow hulls: the type-33 per-model block (SPEC-0004 Optional per-model block): bone-attached
//! convex pieces, each a bounding sphere, an edge list (the two triangles sharing the edge and
//! its two hull points), a triangle list (unit normal and three hull point indices) and hull
//! points, plus the bone-matrix bank index. The lists are payload-relative on disc.

use crate::resource::Chunk;

/// The chunk type of the block.
pub const SHADOW: u16 = 33;

/// One shadow-casting piece.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Unit {
    pub centre: [f32; 3],
    pub radius: f32,
    /// The bone index across the render object's bone-matrix arrays (SPEC-0004 Bank index).
    pub bank: u8,
    /// (first triangle, second triangle, first point, second point).
    pub edges: Vec<[u8; 4]>,
    /// (unit normal, point indices).
    pub triangles: Vec<([f32; 3], [u8; 3])>,
    pub points: Vec<[f32; 3]>,
}

fn u32_at(p: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(p.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(p: &[u8], at: usize) -> Option<f32> {
    u32_at(p, at).map(f32::from_bits)
}

fn v3(p: &[u8], at: usize) -> Option<[f32; 3]> {
    Some([f32_at(p, at)?, f32_at(p, at + 4)?, f32_at(p, at + 8)?])
}

/// Parse a type-33 payload: n units at 0x10 + 0x20 k, each list read through its offset and
/// count. A list that runs past the payload is an error.
pub fn parse(p: &[u8]) -> Result<Vec<Unit>, String> {
    let n = u32_at(p, 0).ok_or("Shadow block too short")? as usize;
    if n > 255 {
        return Err(format!("Shadow block: {n} units"));
    }
    let bad = |what: &str, k: usize| format!("Shadow block unit {k}: {what} outside the payload");
    (0..n)
        .map(|k| {
            let u = 0x10 + 0x20 * k;
            let centre = v3(p, u).ok_or_else(|| bad("unit", k))?;
            let radius = f32_at(p, u + 0x0C).ok_or_else(|| bad("unit", k))?;
            let (eo, to, po) = (u32_at(p, u + 0x10).unwrap_or(0) as usize, u32_at(p, u + 0x14).unwrap_or(0) as usize, u32_at(p, u + 0x18).unwrap_or(0) as usize);
            let counts = p.get(u + 0x1C..u + 0x20).ok_or_else(|| bad("unit", k))?;
            let (ne, nt, bank, np) = (counts[0] as usize, counts[1] as usize, counts[2], counts[3] as usize);
            let edges = (0..ne).map(|i| p.get(eo + 4 * i..eo + 4 * i + 4).map(|b| [b[0], b[1], b[2], b[3]]).ok_or_else(|| bad("edge", k))).collect::<Result<Vec<_>, _>>()?;
            let triangles = (0..nt)
                .map(|i| {
                    let t = to + 32 * i;
                    let normal = v3(p, t).ok_or_else(|| bad("triangle", k))?;
                    let ix = p.get(t + 0x10..t + 0x13).ok_or_else(|| bad("triangle", k))?;
                    Ok((normal, [ix[0], ix[1], ix[2]]))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let points = (0..np).map(|i| v3(p, po + 16 * i).ok_or_else(|| bad("point", k))).collect::<Result<Vec<_>, _>>()?;
            Ok(Unit { centre, radius, bank, edges, triangles, points })
        })
        .collect()
}

/// The last type-33 chunk of a chain (the outfit chains end with theirs), parsed.
pub fn find(bytes: &[u8], chunks: &[Chunk]) -> Option<Vec<Unit>> {
    let c = chunks.iter().rev().find(|c| c.kind == SHADOW)?;
    parse(bytes.get(c.payload.clone())?).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_a_unit_with_its_lists() {
        // One unit: a two-triangle quad (points 0..3), one shared edge, bank 2.
        let mut p = vec![0u8; 0x30];
        p[0] = 1;
        let put = |p: &mut Vec<u8>, at: usize, b: &[u8]| {
            if p.len() < at + b.len() {
                p.resize(at + b.len(), 0);
            }
            p[at..at + b.len()].copy_from_slice(b);
        };
        put(&mut p, 0x10, &0.5f32.to_le_bytes());
        put(&mut p, 0x1C, &1.0f32.to_le_bytes());
        let (tris, pts, edges) = (0x30usize, 0x70usize, 0xB0usize);
        put(&mut p, 0x20, &(edges as u32).to_le_bytes());
        put(&mut p, 0x24, &(tris as u32).to_le_bytes());
        put(&mut p, 0x28, &(pts as u32).to_le_bytes());
        put(&mut p, 0x2C, &[1, 2, 2, 4]);
        for (i, ix) in [[0u8, 1, 2], [0, 2, 3]].iter().enumerate() {
            put(&mut p, tris + 32 * i + 8, &1.0f32.to_le_bytes());
            put(&mut p, tris + 32 * i + 0x10, ix);
        }
        for (i, q) in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]].iter().enumerate() {
            for (j, v) in q.iter().enumerate() {
                put(&mut p, pts + 16 * i + 4 * j, &v.to_le_bytes());
            }
        }
        put(&mut p, edges, &[0, 1, 0, 2]);
        let u = &super::parse(&p).unwrap()[0];
        assert_eq!((u.centre[0], u.radius, u.bank), (0.5, 1.0, 2));
        assert_eq!(u.edges, vec![[0, 1, 0, 2]]);
        assert_eq!(u.triangles[1], ([0.0, 0.0, 1.0], [0, 2, 3]));
        assert_eq!(u.points[2], [1.0, 1.0, 0.0]);
        // A list past the payload is refused.
        p.truncate(0xB2);
        assert!(super::parse(&p).is_err());
    }
}
