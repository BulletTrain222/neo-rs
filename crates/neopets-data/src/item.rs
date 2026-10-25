//! The item config table (SPEC-0021, resource 529): one 0x30-byte record per item id with its
//! type and stats A to H; and the per-hero weapon reach tables of the boot program.

pub const ITEM_CONFIG: u32 = 529;
const RECORDS_AT: usize = 0x10;
const RECORD: usize = 0x30;
/// SPEC-0021 Weapon reach: Tor's table for ids 98 to 108 (11 f32), Roberta's for 111 to 118.
pub const REACH_TOR: u32 = 0x002D_D5E0;
pub const REACH_ROBERTA: u32 = 0x002D_D670;
/// SPEC-0036 Start values: starting and maximum health of Tor (Roberta at + 4).
pub const HEALTH_TOR: u32 = 0x0031_9BC0;
/// SPEC-0021 Where it lives: the coin denominations and coin item ids (5 u32 each), and the
/// eight coin motion words (f32) from the launch speed centre to the horizontal speed centre.
pub const COIN_DENOMINATIONS: u32 = 0x0030_F6F0;
pub const COIN_IDS: u32 = 0x0030_F708;
pub const COIN_WORDS: u32 = 0x002E_B3E8;
/// The sub-tables and drop groups follow the item records at this offset of resource 529.
const DROP_TABLES_AT: usize = 0x2710;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item {
    pub id: u16,
    pub kind: i16,
    /// The pickup distance base (+0x08, 1.5 in every record).
    pub pickup_base: f32,
    /// Stats A to H (+0x20 to +0x2E).
    pub stats: [i16; 8],
}

impl Item {
    /// Weapon types (3 Tor, 6 Roberta): stat A is the melee damage factor A of SPEC-0036.
    pub fn is_weapon(&self) -> bool {
        matches!(self.kind, 3 | 6)
    }
}

/// A drop group (SPEC-0021 Layout): money chance, money range and (sub-table, weight) entries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DropGroup {
    pub chance: i16,
    pub min: i16,
    pub max: i16,
    pub entries: Vec<(i16, i16)>,
}

#[derive(Clone, Debug, Default)]
pub struct ItemTable {
    pub items: Vec<Item>,
    /// The drop sub-tables ((item id, weight) lists) and the drop groups.
    pub sub_tables: Vec<Vec<(u16, i16)>>,
    pub groups: Vec<DropGroup>,
}

impl ItemTable {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let n = u32_at(b, 0).ok_or("Truncated item table")? as usize;
        let items = (0..n.min(1024))
            .map(|i| {
                let r = RECORDS_AT + RECORD * i;
                let stats = std::array::from_fn(|k| i16_at(b, r + 0x20 + 2 * k).unwrap_or(0));
                let pickup_base = u32_at(b, r + 0x08).map(f32::from_bits)?;
                Some(Item { id: i16_at(b, r + 0x0C)? as u16, kind: i16_at(b, r + 0x0E)?, pickup_base, stats })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("Item record past the end")?;
        let (sub_tables, groups) = drop_tables(b).unwrap_or_default();
        Ok(Self { items, sub_tables, groups })
    }

    pub fn get(&self, id: u16) -> Option<&Item> {
        self.items.get(id as usize)
    }
}

fn align16(x: usize) -> usize {
    x.div_ceil(16) * 16
}

/// The sub-tables (count M at 0x2710, the first at the next 16-byte boundary, each a u32 count
/// and (u16 id, s16 weight) entries, unpadded) and the drop groups (count K at the next 16-byte
/// boundary after them, the first group at the next boundary after K; each u16, s16 chance, s16
/// min, s16 max, u32 count, then (s16 sub-table, s16 weight) entries, unpadded).
fn drop_tables(b: &[u8]) -> Option<(Vec<Vec<(u16, i16)>>, Vec<DropGroup>)> {
    let m = u32_at(b, DROP_TABLES_AT)? as usize;
    let mut at = align16(DROP_TABLES_AT + 4);
    let mut subs = Vec::new();
    for _ in 0..m.min(256) {
        let n = u32_at(b, at)? as usize;
        at += 4;
        let mut t = Vec::new();
        for _ in 0..n.min(256) {
            t.push((i16_at(b, at)? as u16, i16_at(b, at + 2)?));
            at += 4;
        }
        subs.push(t);
    }
    let k_at = align16(at);
    let k = u32_at(b, k_at)? as usize;
    let mut at = align16(k_at + 4);
    let mut groups = Vec::new();
    for _ in 0..k.min(256) {
        let n = u32_at(b, at + 8)? as usize;
        let entries = (0..n.min(256)).map(|e| Some((i16_at(b, at + 12 + 4 * e)?, i16_at(b, at + 14 + 4 * e)?))).collect::<Option<Vec<_>>>()?;
        groups.push(DropGroup { chance: i16_at(b, at + 2)?, min: i16_at(b, at + 4)?, max: i16_at(b, at + 6)?, entries });
        at += 12 + 4 * n;
    }
    Some((subs, groups))
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

    #[test]
    fn drop_tables_layout() {
        // M = 2 sub-tables: [(152, 100)] and [(172, 75), (173, 25)]; K = 1 group: chance 40,
        // money 300..800, entries (1, 15).
        let mut b = vec![0u8; DROP_TABLES_AT + 0x100];
        let put = |b: &mut Vec<u8>, at: usize, v: &[u8]| b[at..at + v.len()].copy_from_slice(v);
        put(&mut b, DROP_TABLES_AT, &2u32.to_le_bytes());
        let mut at = DROP_TABLES_AT + 0x10;
        for t in [vec![(152u16, 100i16)], vec![(172, 75), (173, 25)]] {
            put(&mut b, at, &(t.len() as u32).to_le_bytes());
            at += 4;
            for (id, w) in t {
                put(&mut b, at, &id.to_le_bytes());
                put(&mut b, at + 2, &w.to_le_bytes());
                at += 4;
            }
        }
        let k_at = align16(at);
        put(&mut b, k_at, &1u32.to_le_bytes());
        let g = align16(k_at + 4);
        put(&mut b, g, &0xFFFFu16.to_le_bytes());
        put(&mut b, g + 2, &40i16.to_le_bytes());
        put(&mut b, g + 4, &300i16.to_le_bytes());
        put(&mut b, g + 6, &800i16.to_le_bytes());
        put(&mut b, g + 8, &1u32.to_le_bytes());
        put(&mut b, g + 12, &1i16.to_le_bytes());
        put(&mut b, g + 14, &15i16.to_le_bytes());
        let (subs, groups) = drop_tables(&b).unwrap();
        assert_eq!(subs, vec![vec![(152, 100)], vec![(172, 75), (173, 25)]]);
        assert_eq!(groups, vec![DropGroup { chance: 40, min: 300, max: 800, entries: vec![(1, 15)] }]);
    }
}
