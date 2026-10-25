//! The item config table (SPEC-0021, resource 529): one 0x30-byte record per item id with its
//! type and stats A to H; and the per-hero weapon reach tables of the boot program.

pub const ITEM_CONFIG: u32 = 529;
const RECORDS_AT: usize = 0x10;
const RECORD: usize = 0x30;
/// SPEC-0021 Weapon reach: Tor's table for ids 98 to 108 (11 f32), Roberta's for 111 to 118.
pub const REACH_TOR: u32 = 0x002D_D5E0;
pub const REACH_ROBERTA: u32 = 0x002D_D670;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
    pub kind: i16,
    /// Stats A to H (+0x20 to +0x2E).
    pub stats: [i16; 8],
}

impl Item {
    /// Weapon types (3 Tor, 6 Roberta): stat A is the melee damage factor A of SPEC-0036.
    pub fn is_weapon(&self) -> bool {
        matches!(self.kind, 3 | 6)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ItemTable {
    pub items: Vec<Item>,
}

impl ItemTable {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let n = u32_at(b, 0).ok_or("Truncated item table")? as usize;
        let items = (0..n.min(1024))
            .map(|i| {
                let r = RECORDS_AT + RECORD * i;
                let stats = std::array::from_fn(|k| i16_at(b, r + 0x20 + 2 * k).unwrap_or(0));
                Some(Item { id: i16_at(b, r + 0x0C)? as u16, kind: i16_at(b, r + 0x0E)?, stats })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("Item record past the end")?;
        Ok(Self { items })
    }

    pub fn get(&self, id: u16) -> Option<&Item> {
        self.items.get(id as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_by_id() {
        let mut b = vec![0u8; RECORDS_AT + 2 * RECORD];
        b[0] = 2;
        let r1 = RECORDS_AT + RECORD;
        b[r1 + 0x0C..r1 + 0x0E].copy_from_slice(&1u16.to_le_bytes());
        b[r1 + 0x0E..r1 + 0x10].copy_from_slice(&3i16.to_le_bytes());
        b[r1 + 0x20..r1 + 0x22].copy_from_slice(&4i16.to_le_bytes());
        let t = ItemTable::parse(&b).unwrap();
        let it = t.get(1).unwrap();
        assert!(it.is_weapon() && it.stats[0] == 4);
        assert!(t.get(2).is_none());
    }
}
