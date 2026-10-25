//! The area table (SPEC-0006, Area numbering): 193 rows, one per area slot, giving the world
//! pack and the world id. Read from the boot program at run time, behind its fingerprint.
use crate::Disc;
use crate::archive::{boot_u32_table, verified_boot_program};

/// SPEC-0006: table A (world-pack resource numbers) and table B (world ids), 193 rows each.
const TABLE_A: u32 = 0x0031_EFA8;
const TABLE_B: u32 = 0x0031_F2B0;
pub const SLOTS: usize = 193;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaTable {
    pub packs: Vec<u32>,
    pub world_ids: Vec<u32>,
}

impl AreaTable {
    pub fn from_disc(disc: &mut Disc) -> Result<Self, String> {
        let boot = verified_boot_program(disc)?;
        Ok(Self {
            packs: boot_u32_table(&boot, TABLE_A, SLOTS)?,
            world_ids: boot_u32_table(&boot, TABLE_B, SLOTS)?,
        })
    }

    /// The first slot holding a pack (two packs occupy two slots; the lookup takes the first).
    pub fn slot_of(&self, pack: u32) -> Option<usize> {
        self.packs.iter().position(|&p| p == pack)
    }
}

/// Sky dome resource for an area slot (SPEC-0046 Sky domes): 2 for slots 151–156 and 186, 3 for
/// 157–185, 1 for every other slot (and for packs with no slot).
pub fn dome_for_slot(slot: Option<usize>) -> u32 {
    match slot {
        Some(151..=156) | Some(186) => 2,
        Some(157..=185) => 3,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dome_rule_matches_spec() {
        assert_eq!(dome_for_slot(Some(150)), 1);
        assert_eq!(dome_for_slot(Some(151)), 2);
        assert_eq!(dome_for_slot(Some(186)), 2);
        assert_eq!(dome_for_slot(Some(157)), 3);
        assert_eq!(dome_for_slot(Some(185)), 3);
        assert_eq!(dome_for_slot(Some(187)), 1);
        assert_eq!(dome_for_slot(None), 1);
    }

    #[test]
    fn slot_lookup_takes_the_first_row() {
        let t = AreaTable { packs: vec![678, 691, 1391, 1391], world_ids: vec![0, 11, 20, 21] };
        assert_eq!(t.slot_of(1391), Some(2));
        assert_eq!(t.slot_of(9999), None);
    }
}
