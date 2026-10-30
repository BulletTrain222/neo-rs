//! Overworld travel data (SPEC-0006, World buffers and transitions; SPEC-0030 section 1): the
//! boot-program tables of the pending step and the neighbour-world test, a pack's kind from its
//! level module's Init export, and the neighbour list (tagged table 0x0B). All read at run time.
use crate::archive::{boot_u32_table, verified_boot_program};
use crate::area::AreaTable;
use crate::elf;
use crate::resource::Chunk;
use crate::Disc;

/// SPEC-0006: the area-pair table (25 pairs), the no-swap list (16 packs) and the block-pair
/// list (4 pairs) of the neighbour-world test.
const AREA_PAIRS: u32 = 0x0031_EEE0;
const NO_SWAP: u32 = 0x0031_EEA0;
const BLOCK_PAIRS: u32 = 0x0031_EE80;
/// SPEC-0030: the host name table (406 records of 12 bytes); records 397 to 405 are the nine
/// family entry names.
const HOST_TABLE: u32 = 0x0030_30A0;
const FAMILY_NAMES: std::ops::Range<usize> = 397..406;
const MODULE: u16 = 37;
const TAGGED: u16 = 15;

/// SPEC-0006 World kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Transition,
    Exterior,
    Interior,
}

/// SPEC-0006: the 15 transition packs that carry no level module (members of the game's
/// transition rule as the spec states them).
const MODULELESS_TRANSITIONS: [u32; 15] = [683, 684, 703, 704, 705, 713, 717, 718, 719, 720, 721, 1121, 1370, 1395, 1396];

/// SPEC-0006 Neighbour-world test, step 2: the swap set of area slots, a decision rule of the
/// game's code whose members the spec states.
pub fn in_swap_set(slot: usize) -> bool {
    matches!(
        slot,
        20 | 22 | 29..=32 | 34 | 41 | 43 | 47 | 50 | 52 | 60 | 62..=65 | 68..=70 | 74 | 75 | 77..=81 | 85 | 88 | 90
            | 92..=94 | 96 | 98 | 99 | 101 | 103..=106 | 108 | 109 | 114 | 119 | 122 | 124 | 126..=130 | 133..=141
            | 143 | 147..=151 | 153..=155 | 157..=161 | 163..=166 | 168..=170 | 173 | 174 | 178..=180 | 187..=191
    )
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn c_string(b: &[u8], at: usize) -> Option<&[u8]> {
    let tail = b.get(at..)?;
    Some(&tail[..tail.iter().position(|&c| c == 0)?])
}

/// The boot-program tables travel needs.
#[derive(Clone, Debug)]
pub struct BootTravel {
    pub areas: AreaTable,
    pub area_pairs: Vec<(u32, u32)>,
    pub no_swap: Vec<u32>,
    pub block_pairs: Vec<(u32, u32)>,
    /// The nine family entry names, in host-table order (records 397 to 405).
    pub family_names: Vec<Vec<u8>>,
}

impl BootTravel {
    pub fn from_disc(disc: &mut Disc) -> Result<Self, String> {
        let boot = verified_boot_program(disc)?;
        let pairs = |at: u32, n: usize| -> Result<Vec<(u32, u32)>, String> {
            Ok(boot_u32_table(&boot, at, 2 * n)?.chunks(2).map(|p| (p[0], p[1])).collect())
        };
        let host = boot_u32_table(&boot, HOST_TABLE, 3 * FAMILY_NAMES.end)?;
        let family_names = FAMILY_NAMES
            .map(|r| {
                let at = elf::file_offset(&boot, host[3 * r], 1)? as usize;
                c_string(&boot, at).map(<[u8]>::to_vec).ok_or_else(|| format!("Host record {r} has no name"))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            areas: AreaTable::from_disc(disc)?,
            area_pairs: pairs(AREA_PAIRS, 25)?,
            no_swap: boot_u32_table(&boot, NO_SWAP, 16)?,
            block_pairs: pairs(BLOCK_PAIRS, 4)?,
            family_names,
        })
    }

    /// SPEC-0006 Area-pair test.
    pub fn paired(&self, current: u32, other: u32) -> bool {
        self.area_pairs.iter().any(|&(a, b)| (a, b) == (current, other) || (b, a) == (current, other))
    }

    /// SPEC-0006 Neighbour-world test for pack q, with the current pack c of kind `kind`.
    pub fn neighbour_ok(&self, current: u32, current_kind: Kind, q: u32) -> bool {
        if self.no_swap.contains(&current) {
            return false;
        }
        let slot = self.areas.slot_of(current);
        if current_kind != Kind::Transition && !slot.is_some_and(in_swap_set) {
            return false;
        }
        !self.block_pairs.iter().any(|&(a, b)| (a, b) == (current, q) || (b, a) == (current, q))
    }

    /// Whether a pack's module imports the host function `name` (import names are the function
    /// name, two underscores and the encoded parameter list; SPEC-0030 section 2).
    pub fn imports(&self, bytes: &[u8], top: &[Chunk], name: &str) -> bool {
        let Some(c) = top.iter().find(|c| c.kind == MODULE) else { return false };
        let m = &bytes[c.payload.clone()];
        if m.get(..4) != Some(b"SNR2") {
            return false;
        }
        let (Some(table), Some(count)) = (u32_at(m, 0x0C), u32_at(m, 0x10)) else { return false };
        let prefix = [name.as_bytes(), b"__"].concat();
        (1..count.min(1 << 16) as usize).any(|i| {
            let r = table as usize + 12 * i;
            let kind = u32_at(m, r + 8).map(|k| (k >> 16) & 0xFF);
            let n = u32_at(m, r).and_then(|at| c_string(m, at as usize));
            kind != Some(2) && n.is_some_and(|n| n.starts_with(&prefix))
        })
    }

    /// The host-table record (397 to 405) that a pack's module exports as one of the family
    /// entries, for each export that matches one, in symbol order.
    pub fn module_entries(&self, bytes: &[u8], top: &[Chunk]) -> Vec<usize> {
        let Some(c) = top.iter().find(|c| c.kind == MODULE) else { return Vec::new() };
        let m = &bytes[c.payload.clone()];
        if m.get(..4) != Some(b"SNR2") {
            return Vec::new();
        }
        let (Some(table), Some(count)) = (u32_at(m, 0x0C), u32_at(m, 0x10)) else { return Vec::new() };
        (1..count.min(1 << 16) as usize)
            .filter_map(|i| {
                let r = table as usize + 12 * i;
                let kind = (u32_at(m, r + 8)? >> 16) & 0xFF;
                let name = c_string(m, u32_at(m, r)? as usize)?;
                (kind == 2).then(|| self.family_names.iter().position(|n| n == name).map(|k| FAMILY_NAMES.start + k))?
            })
            .collect()
    }

    /// A pack's kind (SPEC-0006 World kinds): the family whose Init entry its module exports.
    /// The three Init names are host records 397, 398 and 399 for the Transition, Exterior and
    /// Interior families (the `--areas` census: 691 exports 397, 909 398, 1008 and 1429 399,
    /// giving SPEC-0006's 36, 87 and 50). A pack with no module is a transition when it is on
    /// the spec's list of 15, otherwise an exterior (879, 1298 and 1323 are not transitions).
    pub fn kind(&self, pack: u32, bytes: &[u8], top: &[Chunk]) -> Kind {
        let entries = self.module_entries(bytes, top);
        if entries.contains(&397) {
            Kind::Transition
        } else if entries.contains(&399) {
            Kind::Interior
        } else if entries.contains(&398) || !MODULELESS_TRANSITIONS.contains(&pack) {
            Kind::Exterior
        } else {
            Kind::Transition
        }
    }

    /// The 51 packs of the game's transition test (36 modules and 15 without).
    pub fn is_transition(&self, pack: u32, bytes: &[u8], top: &[Chunk]) -> bool {
        self.kind(pack, bytes, top) == Kind::Transition
    }
}

/// The neighbour list (tag 0x0B) of a pack: lookup of entry k gives a world id or -1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Neighbours {
    values: Vec<u16>,
    count: usize,
}

impl Neighbours {
    pub fn of(bytes: &[u8], top: &[Chunk]) -> Self {
        tagged_lookup(bytes, top, 0x0B).unwrap_or_default()
    }

    /// The arrival-door list (tag 0x0E): entry n gives the door at which the hero arrives in
    /// the destination when it leaves by door n.
    pub fn arrivals(bytes: &[u8], top: &[Chunk]) -> Self {
        tagged_lookup(bytes, top, 0x0E).unwrap_or_default()
    }

    /// Entry k: the value at index k + 3, or -1 when k is at or above N (or negative).
    pub fn get(&self, k: i32) -> i32 {
        if k < 0 || k as usize >= self.count {
            return -1;
        }
        self.values.get(k as usize + 3).map_or(-1, |&v| v as i16 as i32)
    }
}

/// The entity-id table (tag 0x12, SPEC-0006 Tagged tables): entry e gives the region byte
/// (0xFF for the pack's embedded region, else the slot-load number j of region P + 1 + j) and
/// the ordinal of a kind-3 record in that region.
/// The map area id of a world pack: the s32 at payload + 0x14 of its type-1001 chunk (SPEC-0006
/// Area names); -1 or no chunk means none.
pub fn map_area(bytes: &[u8], chain: &[Chunk]) -> Option<i32> {
    let c = chain.iter().find(|c| c.kind == 1001)?;
    let p = bytes.get(c.payload.clone())?;
    let m = i32::from_le_bytes(p.get(0x14..0x18)?.try_into().ok()?);
    (m >= 0).then_some(m)
}

pub fn entity_table(bytes: &[u8], top: &[Chunk]) -> Vec<(u8, u8)> {
    tagged_pairs(bytes, top, 0x12)
}

/// The placement id table (SPEC-0021 Placement id table; type-15 tag 0x13): per placement id,
/// the region byte (0xFF the embedded region, else a slot load's j) and the kind-8 ordinal.
pub fn placement_table(bytes: &[u8], top: &[Chunk]) -> Vec<(u8, u8)> {
    tagged_pairs(bytes, top, 0x13)
}

/// The first two bytes of each 16-byte entry of a tag-0x12 or tag-0x13 record.
fn tagged_pairs(bytes: &[u8], top: &[Chunk], tag: u32) -> Vec<(u8, u8)> {
    let Some(c) = top.iter().find(|c| c.kind == TAGGED) else { return Vec::new() };
    let p = &bytes[c.payload.clone()];
    let mut at = 0usize;
    while at + 0x10 <= p.len() {
        let Some(t) = u32_at(p, at) else { break };
        let len = match t {
            0x0B | 0x0D | 0x0E => {
                let n = p.get(at + 0x10..at + 0x12).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as usize);
                (0x10 + (n + 3) * 2).div_ceil(16) * 16
            }
            0x0A => {
                let n = p.get(at + 0x10..at + 0x12).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as usize);
                (0x10 + 4 * n + 2).div_ceil(16) * 16
            }
            0x12 | 0x13 => {
                let size = u32_at(p, at + 8).unwrap_or(0) as usize & !0xF;
                if t == tag {
                    return (0..size / 16).filter_map(|i| Some((*p.get(at + 0x10 + 16 * i)?, *p.get(at + 0x11 + 16 * i)?))).collect();
                }
                0x10 + size
            }
            _ => break,
        };
        at += len.max(0x10);
    }
    Vec::new()
}

/// Walk the type-15 tagged records and read the lookup table with the given tag.
/// The area's path list (SPEC-0006 Tagged tables, tag 0x0A; R-0127): at +0x10 a u16 count N,
/// then the u16 arrays A[N] and B[N]; path k is the pack's entry records A[k] to B[k], both
/// included. Empty when the pack has no such table.
pub fn path_list(bytes: &[u8], top: &[Chunk]) -> Vec<(u16, u16)> {
    let Some(c) = top.iter().find(|c| c.kind == TAGGED) else { return Vec::new() };
    paths_in(&bytes[c.payload.clone()]).unwrap_or_default()
}

fn paths_in(p: &[u8]) -> Option<Vec<(u16, u16)>> {
    let u16_at = |at: usize| p.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let mut at = 0usize;
    while at + 0x10 <= p.len() {
        let t = u32_at(p, at)?;
        let len = match t {
            0x0B | 0x0D | 0x0E => (0x10 + (u16_at(at + 0x10)? as usize + 3) * 2).div_ceil(16) * 16,
            0x0A => {
                let n = u16_at(at + 0x10)? as usize;
                return (0..n).map(|k| Some((u16_at(at + 0x12 + 2 * k)?, u16_at(at + 0x12 + 2 * (n + k))?))).collect();
            }
            0x12 | 0x13 => 0x10 + (u32_at(p, at + 8)? as usize & !0xF),
            _ => return None,
        };
        at += len.max(0x10);
    }
    Some(Vec::new())
}

fn tagged_lookup(bytes: &[u8], top: &[Chunk], tag: u32) -> Option<Neighbours> {
    let c = top.iter().find(|c| c.kind == TAGGED)?;
    let p = &bytes[c.payload.clone()];
    let mut at = 0usize;
    while at + 0x10 <= p.len() {
        let t = u32_at(p, at)?;
        let len = match t {
            0x0B | 0x0D | 0x0E => {
                let n = u16::from_le_bytes(p.get(at + 0x10..at + 0x12)?.try_into().ok()?) as usize;
                if t == tag {
                    let values = (0..n + 3)
                        .map(|i| p.get(at + 0x10 + 2 * i..at + 0x12 + 2 * i).map(|b| u16::from_le_bytes([b[0], b[1]])))
                        .collect::<Option<Vec<_>>>()?;
                    return Some(Neighbours { values, count: n });
                }
                (0x10 + (n + 3) * 2).div_ceil(16) * 16
            }
            0x0A => {
                let n = u16::from_le_bytes(p.get(at + 0x10..at + 0x12)?.try_into().ok()?) as usize;
                (0x10 + 4 * n + 2).div_ceil(16) * 16
            }
            0x12 | 0x13 => 0x10 + (u32_at(p, at + 8)? as usize & !0xF),
            _ => return None,
        };
        at += len.max(0x10);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_set_has_92_slots() {
        assert_eq!((0..193).filter(|&s| in_swap_set(s)).count(), 92);
        assert!(in_swap_set(68) && !in_swap_set(0) && in_swap_set(191));
    }

    #[test]
    fn path_list_ranges() {
        // SPEC-0006 test vector: N = 3, A = {1, 4, 7}, B = {3, 4, 9}.
        let mut p = vec![0u8; 0x20];
        p[0] = 0x0A;
        for (i, v) in [3u16, 1, 4, 7, 3, 4, 9].iter().enumerate() {
            p[0x10 + 2 * i..0x12 + 2 * i].copy_from_slice(&v.to_le_bytes());
        }
        assert_eq!(paths_in(&p), Some(vec![(1, 3), (4, 4), (7, 9)]));
    }

    #[test]
    fn neighbour_lookup_is_offset_by_three() {
        // N = 3, values: [N, pad, pad, v0, v1, v2]: lookup 1 reads index 4.
        let n = Neighbours { values: vec![3, 0, 0, 0xFFFF, 602, 11], count: 3 };
        assert_eq!((n.get(0), n.get(1), n.get(2), n.get(3), n.get(-1)), (-1, 602, 11, -1, -1));
    }
}
