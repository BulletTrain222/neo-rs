//! The monster table (SPEC-0029 section 4): resource 530 (rows of 0x504 bytes) and resource 531
//! (companion rows of 0x60 bytes) share a header of u32 N = 112 types and N u32 first-row
//! indices; rows start at 0x1D0. A placement's row is first[type] + variant.

const HEADER_TYPES: usize = 112;
const ROWS_AT: usize = 0x1D0;
const ROW: usize = 0x504;
const COMPANION: usize = 0x60;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// The row fields used so far (offsets in SPEC-0029 section 4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub radius: f32,
    pub height: f32,
    pub hit_points: u16,
    /// Default behavior class: the row's (+0x36) or, when none, the companion row's (+0x10).
    pub class: Option<i16>,
    pub hostile: bool,
    pub weight: f32,
    /// Can fly (+0x94): 0 never, 1 while grounded countdown ≤ 0, 2 always.
    pub can_fly: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MonsterTable {
    rows: Vec<u8>,
    companions: Vec<u8>,
    first: Vec<u32>,
}

impl MonsterTable {
    /// From the bytes of resources 530 and 531.
    pub fn parse(rows: &[u8], companions: &[u8]) -> Result<Self, String> {
        let n = u32_at(rows, 0).ok_or("Truncated monster table")? as usize;
        if n != HEADER_TYPES {
            return Err(format!("Monster table has {n} types, expected {HEADER_TYPES}"));
        }
        let first = (0..n).map(|t| u32_at(rows, 4 + 4 * t).ok_or("Truncated header")).collect::<Result<_, _>>()?;
        Ok(Self { rows: rows.to_vec(), companions: companions.to_vec(), first })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len().saturating_sub(ROWS_AT) / ROW
    }

    /// The row index of (type, variant), when the type is in range.
    pub fn index(&self, monster_type: u16, variant: u16) -> Option<usize> {
        Some(*self.first.get(monster_type as usize)? as usize + variant as usize)
    }

    pub fn row(&self, monster_type: u16, variant: u16) -> Option<Row> {
        let i = self.index(monster_type, variant)?;
        let r = ROWS_AT + ROW * i;
        let c = ROWS_AT + COMPANION * i;
        if r + ROW > self.rows.len() {
            return None;
        }
        let own = i16_at(&self.rows, r + 0x36)?;
        let class = if own >= 0 { Some(own) } else { i16_at(&self.companions, c + 0x10).filter(|&k| k >= 0) };
        Some(Row {
            radius: f32_at(&self.rows, r + 0x28)?,
            height: f32_at(&self.rows, r + 0x2C)?,
            hit_points: u16::from_le_bytes(self.rows.get(r + 0x30..r + 0x32)?.try_into().ok()?),
            class,
            hostile: i16_at(&self.rows, r + 0x3E)? != 0,
            weight: f32_at(&self.rows, r + 0x118)?,
            can_fly: u32_at(&self.rows, r + 0x94)? as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_lookup_and_companion_class() {
        // Two types: type 0 starts at row 0, type 1 at row 1; row 1 has no class of its own and
        // its companion row gives 6.
        let mut rows = vec![0u8; ROWS_AT + 2 * ROW];
        rows[0..4].copy_from_slice(&112u32.to_le_bytes());
        rows[8..12].copy_from_slice(&1u32.to_le_bytes());
        let r1 = ROWS_AT + ROW;
        rows[r1 + 0x30..r1 + 0x32].copy_from_slice(&50u16.to_le_bytes());
        rows[r1 + 0x36..r1 + 0x38].copy_from_slice(&(-1i16).to_le_bytes());
        rows[r1 + 0x28..r1 + 0x2C].copy_from_slice(&1.175f32.to_le_bytes());
        let mut comp = vec![0u8; ROWS_AT + 2 * COMPANION];
        comp[ROWS_AT + COMPANION + 0x10..ROWS_AT + COMPANION + 0x12].copy_from_slice(&6i16.to_le_bytes());
        let t = MonsterTable::parse(&rows, &comp).unwrap();
        assert_eq!(t.row_count(), 2);
        let row = t.row(1, 0).unwrap();
        assert_eq!((row.hit_points, row.class, row.radius), (50, Some(6), 1.175));
        assert!(t.row(1, 1).is_none(), "past the end");
    }
}
