//! Character regions (SPEC-0006 Embedded character region and Character regions; SPEC-0005):
//! a chunk chain holding a type-36 record table, model sets (type 18) and skeleton and
//! animation chunks (types 22 to 27). A world pack embeds one as its type-32 chunk; region slot
//! j of pack P is resource P + 1 + j.
use crate::resource::Chunk;

pub const EMBEDDED: u16 = 32;
pub const RECORDS: u16 = 36;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i16_at(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn r16(x: usize) -> usize {
    x.div_ceil(16) * 16
}

/// A kind-3 character placement (SPEC-0006 table; offsets from the record's data).
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    /// Index among the region's kind-3 records (the ordinal entity ids name).
    pub ordinal: usize,
    pub position: [f32; 3],
    pub facing: [f32; 3],
    pub name: String,
    /// Main, sub and face skeleton indices (−1 none; −5, −6, −7 global main sets).
    pub skeletons: [i16; 3],
    /// Main, sub and face animation-table indices (kind-2 records of the region, in order).
    pub tables: [i16; 3],
    pub model_set: i16,
    pub monster_type: u16,
    pub variant: u16,
    /// Behavior code (+0x84), the AI mode the townsfolk routine starts from (SPEC-0056).
    pub ai_code: i16,
    pub summon: bool,
    /// The default state's parameter words read by the common enemies (SPEC-0038 Layout
    /// section 1, Behavior 40): the emerge distance (f32 +0xC0) and the shared value / selector
    /// word (+0xC8).
    pub emerge_distance: f32,
    pub selector: u32,
}

/// The record walk of a type-36 table: kind-3 placements and kind-2 animation tables (their
/// data: a u32 count N, then N 8-byte entries; ranges relative to the walked bytes).
#[derive(Clone, Debug, Default)]
pub struct Records {
    pub placements: Vec<Placement>,
    pub tables: Vec<std::ops::Range<usize>>,
}

/// Walk a type-36 payload; any other kind ends the walk (SPEC-0006).
pub fn records(p: &[u8]) -> Records {
    let mut out = Records::default();
    let mut at = 0usize;
    while let Some(kind) = u32_at(p, at) {
        let data = at + 0x10;
        let len = match kind {
            3 => 0xF0,
            2 => match u32_at(p, data) {
                Some(n) => 0x10 + r16(8 * n as usize + 4),
                None => break,
            },
            4 | 8 => 0x40,
            16 => match u32_at(p, data + 4) {
                Some(n) => 0x10 + r16(0x50 * n as usize + 0x10),
                None => break,
            },
            17 => match u32_at(p, at + 8) {
                Some(n) => 0x10 + n as usize,
                None => break,
            },
            _ => break,
        };
        if at + len > p.len() {
            break;
        }
        match kind {
            3 => {
                let v = |o: usize| [f32_at(p, data + o).unwrap_or(0.0), f32_at(p, data + o + 4).unwrap_or(0.0), f32_at(p, data + o + 8).unwrap_or(0.0)];
                let name_bytes = &p[data + 0x20..data + 0x48];
                let name: String = name_bytes.iter().take_while(|&&c| c != 0).map(|&c| c as char).collect();
                let s = |o: usize| i16_at(p, data + o).unwrap_or(-1);
                out.placements.push(Placement {
                    ordinal: out.placements.len(),
                    position: v(0),
                    facing: v(0x10),
                    name,
                    skeletons: [s(0x86), s(0x88), s(0x8A)],
                    tables: [s(0x9E), s(0xA0), s(0xA2)],
                    model_set: s(0xA4),
                    monster_type: s(0xB4) as u16,
                    ai_code: s(0x84),
                    variant: s(0xB2) as u16,
                    summon: p.get(data + 0xB6).is_some_and(|&b| b != 0),
                    emerge_distance: f32_at(p, data + 0xC0).unwrap_or(0.0),
                    selector: u32_at(p, data + 0xC8).unwrap_or(0),
                });
            }
            2 => out.tables.push(data..at + len),
            _ => {}
        }
        at += len;
    }
    out
}

/// A region's chunks by role: the record table, the model sets (type-18 chunks that are not
/// flat bone tables), and the k-th skeleton (22, 23, 24) and clip set (25, 26, 27) of each
/// family, numbered in parallel.
#[derive(Clone, Debug, Default)]
pub struct Region {
    pub records: Records,
    pub model_sets: Vec<Chunk>,
    pub skeletons: [Vec<Chunk>; 3],
    pub clip_sets: [Vec<Chunk>; 3],
    /// The chain the chunks belong to (offsets are within the bytes it was read from).
    pub chain: Vec<Chunk>,
}

impl Region {
    /// A region chain (a region resource's top chain, or the nested chain of a type-32 chunk).
    pub fn of(bytes: &[u8], chain: &[Chunk]) -> Self {
        let mut r = Region { chain: chain.to_vec(), ..Default::default() };
        for c in chain {
            let p = &bytes[c.payload.clone()];
            match c.kind {
                RECORDS => {
                    // Table ranges are rebased from the payload to the region's bytes.
                    r.records = records(p);
                    let base = c.payload.start;
                    for t in &mut r.records.tables {
                        *t = t.start + base..t.end + base;
                    }
                }
                18 if p.starts_with(b"IDM") => r.model_sets.push(c.clone()),
                22..=24 => r.skeletons[(c.kind - 22) as usize].push(c.clone()),
                25..=27 => r.clip_sets[(c.kind - 25) as usize].push(c.clone()),
                _ => {}
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_walk_reads_placements_and_tables() {
        // A kind-2 table with N = 1 (length 0x10 + r16(12) = 0x20), then a kind-3 placement,
        // then an unknown kind that ends the walk.
        let mut p = vec![0u8; 0x20 + 0xF0 + 0x10];
        p[0] = 2;
        p[0x10] = 1;
        let k3 = 0x20;
        p[k3] = 3;
        let d = k3 + 0x10;
        p[d..d + 4].copy_from_slice(&10.0f32.to_le_bytes());
        p[d + 0x20..d + 0x23].copy_from_slice(b"Bob");
        p[d + 0xA4..d + 0xA6].copy_from_slice(&2i16.to_le_bytes());
        p[d + 0x86..d + 0x88].copy_from_slice(&(-7i16).to_le_bytes());
        p[k3 + 0xF0] = 99;
        let r = records(&p);
        assert_eq!(r.tables, vec![0x10..0x20]);
        assert_eq!(r.placements.len(), 1);
        let pl = &r.placements[0];
        assert_eq!((pl.name.as_str(), pl.position[0], pl.model_set, pl.skeletons[0]), ("Bob", 10.0, 2, -7));
    }
}
