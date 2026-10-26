//! The zone manager and query (SPEC-0006 Binary grids, Zone manager, Query; Edge cases, the
//! off-by-one quirk).

/// One binary-grid block: key (block 0 holds the pack's own world id once loaded), width,
/// height, the W × H cells and the byte just past them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Block {
    pub key: i32,
    pub width: i32,
    pub height: i32,
    pub cells: Vec<u8>,
    pub after: u8,
}

/// A world's binary grid (its blocks in chunk order).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grid {
    pub blocks: Vec<Block>,
}

/// The zone manager: row R, key K and the active block (an index into the grid's blocks).
/// The grid G itself is the caller's (the camera world's grid).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zones {
    pub row: i32,
    pub key: i32,
    pub active: Option<usize>,
}

impl Default for Zones {
    fn default() -> Self {
        Self { row: 0, key: -1, active: Some(0) }
    }
}

impl Zones {
    /// The camera's mode update: K = the camera world's id, and R = z (the zone of the camera's
    /// ground entry, or None without one) stored only when the active block exists, z is below
    /// its height and below 255.
    pub fn update(&mut self, grid: Option<&Grid>, key: i32, z: Option<u8>) {
        self.key = key;
        let Some(z) = z else { return };
        let Some(b) = grid.and_then(|g| g.blocks.get(self.active.unwrap_or(0))) else { return };
        if (z as i32) < b.height && z < 255 {
            self.row = z as i32;
        }
    }

    /// The query with column c and key k: 1 without a grid; block 0 for k = K, the active block
    /// for k = -1 (block 0 without one), else the block keyed k (1 when there is none). With
    /// i = R × W + c and N = W × H, i < 0 or i > N passes; i = N reads the byte just past the
    /// cells (original quirk); otherwise the cell decides.
    pub fn query(&self, grid: Option<&Grid>, c: i32, k: i32) -> bool {
        let Some(g) = grid else { return true };
        let block = if k == self.key {
            g.blocks.first()
        } else if k == -1 {
            g.blocks.get(self.active.unwrap_or(0))
        } else {
            g.blocks.iter().find(|b| b.key == k)
        };
        let Some(b) = block else { return true };
        let n = b.width as i64 * b.height as i64;
        let i = self.row as i64 * b.width as i64 + c as i64;
        if i < 0 || i > n {
            return true;
        }
        let byte = if i == n { b.after } else { b.cells.get(i as usize).copied().unwrap_or(0) };
        byte != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Grid {
        Grid {
            blocks: vec![
                Block { key: 602, width: 3, height: 3, cells: vec![1, 0, 1, 0, 1, 0, 0, 0, 1], after: 0 },
                Block { key: 11, width: 2, height: 3, cells: vec![0, 1, 1, 0, 0, 0], after: 9 },
            ],
        }
    }

    #[test]
    fn query_picks_the_block_by_key() {
        let g = grid();
        let mut z = Zones::default();
        z.update(Some(&g), 602, Some(1));
        assert_eq!(z.row, 1);
        assert!(z.query(Some(&g), 1, 602) && !z.query(Some(&g), 0, 602));
        // k = -1 uses the active block (block 0 here).
        assert!(z.query(Some(&g), 1, -1) && !z.query(Some(&g), 2, -1));
        // The neighbour block keyed 11: row 1 is (1, 0).
        assert!(z.query(Some(&g), 0, 11) && !z.query(Some(&g), 1, 11));
        // A key with no block, and no grid at all, pass.
        assert!(z.query(Some(&g), 0, 77) && z.query(None, 0, 602));
    }

    #[test]
    fn row_store_rule_and_off_by_one() {
        let g = grid();
        let mut z = Zones::default();
        z.update(Some(&g), 602, Some(2));
        // z = 3 is not below block 0's height: R keeps 2; no ground keeps it too.
        z.update(Some(&g), 602, Some(3));
        z.update(Some(&g), 602, None);
        assert_eq!(z.row, 2);
        // Row 2 of block 1: index 4 + c; c = 2 gives i = N = 6, the byte past the cells (9).
        assert!(z.query(Some(&g), 2, 11));
        // i > N passes; i < 0 passes.
        assert!(z.query(Some(&g), 3, 11) && z.query(Some(&g), -9, 11));
        // c = 1 at row 2 of block 0 is cell 7 = 0.
        assert!(!z.query(Some(&g), 1, 602));
    }
}
