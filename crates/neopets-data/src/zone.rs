//! Binary grids (chunk type 35, SPEC-0006 Binary grids): a world pack's zone-visibility
//! matrices. Block 0 is the pack's own zones by its own zones; each further block is keyed by a
//! neighbour's world id, with this pack's zones as rows and the neighbour's as columns.

/// The binary-grid chunk of a world pack.
pub const ZONE_CHUNK: u16 = 35;

/// One block: its key (-1 for block 0 in the file), width W, height H and W × H cell bytes,
/// plus the byte just past the cells (read by the query's off-by-one index N; 0 past the
/// payload end).
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneBlock {
    pub key: i32,
    pub width: i32,
    pub height: i32,
    pub cells: Vec<u8>,
    pub after: u8,
}

/// The blocks of a walked world pack chain's type-35 chunk, or None when it has none: a u32
/// count at +0 (1 to 8) and the offset of a table of count block offsets at +4; a block's
/// cells start at its own +0x10 offset.
pub fn zone_grid(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Option<Vec<ZoneBlock>> {
    let c = chain.iter().find(|c| c.kind == ZONE_CHUNK)?;
    zone_grid_of(bytes.get(c.payload.clone())?)
}

/// [`zone_grid`] on a type-35 payload.
pub fn zone_grid_of(p: &[u8]) -> Option<Vec<ZoneBlock>> {
    let word = |at: usize| p.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let int = |at: usize| word(at).map(|w| w as i32);
    let n = word(0)?.min(64) as usize;
    let table = word(4)? as usize;
    (0..n)
        .map(|i| {
            let at = word(table + 4 * i)? as usize;
            let (key, width, height) = (int(at)?, int(at + 4)?, int(at + 8)?);
            let count = (width.max(0) as usize).checked_mul(height.max(0) as usize)?;
            let start = at + word(at + 0x10)? as usize;
            let cells = p.get(start..start + count)?.to_vec();
            Some(ZoneBlock { key, width, height, cells, after: p.get(start + count).copied().unwrap_or(0) })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_read_through_the_table() {
        // Two blocks: block 0 (key -1, 2 x 2) at 0x10 and block 1 (key 602, 3 x 2) at 0x30.
        let mut p = vec![0u8; 0x50];
        p[0..4].copy_from_slice(&2u32.to_le_bytes());
        p[4..8].copy_from_slice(&8u32.to_le_bytes());
        p[8..12].copy_from_slice(&0x10u32.to_le_bytes());
        p[12..16].copy_from_slice(&0x30u32.to_le_bytes());
        let block = |p: &mut Vec<u8>, at: usize, key: i32, w: i32, h: i32, cells: &[u8]| {
            p[at..at + 4].copy_from_slice(&key.to_le_bytes());
            p[at + 4..at + 8].copy_from_slice(&w.to_le_bytes());
            p[at + 8..at + 12].copy_from_slice(&h.to_le_bytes());
            p[at + 12..at + 16].copy_from_slice(&(w * h).to_le_bytes());
            p[at + 16..at + 20].copy_from_slice(&0x14u32.to_le_bytes());
            p[at + 0x14..at + 0x14 + cells.len()].copy_from_slice(cells);
        };
        block(&mut p, 0x10, -1, 2, 2, &[1, 0, 0, 1]);
        p[0x10 + 0x14 + 4] = 7;
        block(&mut p, 0x30, 602, 3, 2, &[1, 1, 0, 0, 0, 1]);
        let g = zone_grid_of(&p).unwrap();
        assert_eq!(g.len(), 2);
        assert_eq!((g[0].key, g[0].width, g[0].cells.clone(), g[0].after), (-1, 2, vec![1, 0, 0, 1], 7));
        assert_eq!((g[1].key, g[1].height, g[1].cells[5]), (602, 2, 1));
        // A block that runs past the payload makes the grid unreadable.
        assert_eq!(zone_grid_of(&p[..0x40]), None);
    }
}
