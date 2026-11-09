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
    /// The main family's translation scale selector (+0x8C; SPEC-0005 Per-bone translation
    /// scale): a 1-based set of the main definition's scale block, 0 for none.
    pub scale_set: i16,
    pub summon: bool,
    /// The character kind key (s16 +0xAC) that a spawn-by-key request matches (SPEC-0029
    /// Behavior 31).
    pub key: i16,
    /// The default state's parameter words read by the common enemies (SPEC-0038 Layout
    /// section 1, Behavior 40): the emerge distance (f32 +0xC0) and the shared value / selector
    /// word (+0xC8).
    pub emerge_distance: f32,
    pub selector: u32,
    /// The default state's words at +0xB8 to +0xD4 (SPEC-0006 kind 3: the patrol path, the
    /// wander side, the pause and stand ranges and percents; SPEC-0038 states 5 to 7).
    pub words: [u32; 8],
    /// The alarm group byte (+0xB7; 0xFF none; SPEC-0038 Behavior 3).
    pub alarm: u8,
    /// The drop placement ordinal (s16 +0xB0): a kind-8 record of the same region, -1 none.
    pub drop: i16,
    /// The character's colour bytes (+0x40; the talk partner's frame colour, SPEC-0022).
    pub colour: [u8; 3],
    /// The 15 colour words (+0x48 + 4 i; SPEC-0040 Behavior 26): bytes 0 to 2 are red, green
    /// and blue; a part draws with the word of its group's colour slot.
    pub colours: [u32; 15],
}

/// A kind-8 item point (SPEC-0006 Kind 8; SPEC-0021 placements).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemPoint {
    pub position: [f32; 3],
    pub facing: [f32; 3],
    /// Flags (+0x20), drop group (+0x22, -1 none), primary key (+0x24, -1 none), luck gate
    /// (+0x28), fixed count (+0x2A) and the treasure-map byte (+0x2C).
    pub flags: u8,
    pub group: i16,
    pub key: i16,
    pub luck: i16,
    pub count: i16,
    pub map: u8,
    /// Snap to ground (+0x21; SPEC-0006 Kind 8): spawned at the ground height + 0.5.
    pub snap: bool,
}

/// A kind-4 save point (SPEC-0006 Kind 4; SPEC-0025 Save points): position (+0x00), facing
/// (+0x10, not used by the save-point step) and the flag word (+0x20).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavePointRecord {
    pub position: [f32; 3],
    pub facing: [f32; 3],
    pub flag: u32,
}

/// A region keeps at most this many save points; further kind-4 records are dropped.
pub const SAVE_POINTS: usize = 8;

/// The record walk of a type-36 table: kind-3 placements and kind-2 animation tables (their
/// data: a u32 count N, then N 8-byte entries; ranges relative to the walked bytes).
#[derive(Clone, Debug, Default)]
pub struct Records {
    pub placements: Vec<Placement>,
    pub tables: Vec<std::ops::Range<usize>>,
    /// Kind-8 item points in walk order (their ordinals).
    pub items: Vec<ItemPoint>,
    /// Kind-4 save points in walk order (at most 8), and the number dropped past the eighth.
    pub save_points: Vec<SavePointRecord>,
    pub save_points_dropped: usize,
    /// Kind-16 effect lists in walk order (SPEC-0034 Layout section 1); a clip's effect index
    /// names one by position.
    pub effect_lists: Vec<Vec<EffectEntry>>,
}

/// An attach slot value: the low three bits name matrix slot 0 to 7; 0x4FFFFFFF is none.
pub fn attach_slot(raw: u32) -> Option<u8> {
    (raw != 0x4FFF_FFFF).then_some((raw & 7) as u8)
}

/// One effect-list entry (SPEC-0034 1.1, 0x50 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectEntry {
    /// Point offsets in attach slot A's and B's frames (w = 1).
    pub offset_a: [f32; 3],
    pub offset_b: [f32; 3],
    pub slot_a: Option<u8>,
    pub slot_b: Option<u8>,
    /// Start and end as fractions of the clip duration.
    pub start: f32,
    pub end: f32,
    /// Effect definition id (-1 none).
    pub id: i32,
    pub rumble_seconds: f32,
    pub intensity: f32,
    /// +0x3C: probe 8.0 below point 1; +0x3D: the owner's ground height.
    pub probe: bool,
    pub owner_ground: bool,
    /// Positional mode; 255 none.
    pub positional: u8,
    pub inner: f32,
    pub outer: f32,
    /// +0x48 = 0 starts a swing trail instead.
    pub trail: bool,
    /// +0x49: the effect object gets slot A's whole frame.
    pub full_frame: bool,
}

fn effect_list(p: &[u8], data: usize) -> Vec<EffectEntry> {
    let n = u32_at(p, data + 4).unwrap_or(0) as usize;
    (0..n)
        .filter_map(|i| {
            let e = data + 0x10 + 0x50 * i;
            let v = |o: usize| Some([f32_at(p, e + o)?, f32_at(p, e + o + 4)?, f32_at(p, e + o + 8)?]);
            Some(EffectEntry {
                offset_a: v(0)?,
                offset_b: v(0x10)?,
                slot_a: attach_slot(u32_at(p, e + 0x20)?),
                slot_b: attach_slot(u32_at(p, e + 0x24)?),
                start: f32_at(p, e + 0x28)?,
                end: f32_at(p, e + 0x2C)?,
                id: u32_at(p, e + 0x30)? as i32,
                rumble_seconds: f32_at(p, e + 0x34)?,
                intensity: f32_at(p, e + 0x38)?,
                probe: *p.get(e + 0x3C)? != 0,
                owner_ground: *p.get(e + 0x3D)? == 1,
                positional: *p.get(e + 0x3E)?,
                inner: f32_at(p, e + 0x40)?,
                outer: f32_at(p, e + 0x44)?,
                trail: *p.get(e + 0x48)? == 0,
                full_frame: *p.get(e + 0x49)? != 0,
            })
        })
        .collect()
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
                    scale_set: i16_at(p, data + 0x8C).unwrap_or(0),
                    variant: s(0xB2) as u16,
                    summon: p.get(data + 0xB6).is_some_and(|&b| (b as i8) > 0),
                    key: s(0xAC),
                    emerge_distance: f32_at(p, data + 0xC0).unwrap_or(0.0),
                    selector: u32_at(p, data + 0xC8).unwrap_or(0),
                    words: std::array::from_fn(|i| u32_at(p, data + 0xB8 + 4 * i).unwrap_or(0)),
                    alarm: p.get(data + 0xB7).copied().unwrap_or(0xFF),
                    drop: i16_at(p, data + 0xB0).unwrap_or(-1),
                    colour: [p.get(data + 0x40).copied().unwrap_or(0), p.get(data + 0x41).copied().unwrap_or(0), p.get(data + 0x42).copied().unwrap_or(0)],
                    colours: std::array::from_fn(|i| p.get(data + 0x48 + 4 * i..data + 0x4C + 4 * i).map_or(0x80FF_FFFF, |b| u32::from_le_bytes(b.try_into().unwrap()))),
                });
            }
            8 => {
                let v = |o: usize| [f32_at(p, data + o).unwrap_or(0.0), f32_at(p, data + o + 4).unwrap_or(0.0), f32_at(p, data + o + 8).unwrap_or(0.0)];
                let s = |o: usize| i16_at(p, data + o).unwrap_or(-1);
                out.items.push(ItemPoint {
                    position: v(0),
                    facing: v(0x10),
                    flags: p.get(data + 0x20).copied().unwrap_or(0),
                    group: s(0x22),
                    key: s(0x24),
                    luck: s(0x28),
                    count: s(0x2A),
                    map: p.get(data + 0x2C).copied().unwrap_or(0),
                    snap: p.get(data + 0x21).is_some_and(|&b| b != 0),
                });
            }
            4 => {
                let v = |o: usize| [f32_at(p, data + o).unwrap_or(0.0), f32_at(p, data + o + 4).unwrap_or(0.0), f32_at(p, data + o + 8).unwrap_or(0.0)];
                if out.save_points.len() < SAVE_POINTS {
                    out.save_points.push(SavePointRecord { position: v(0), facing: v(0x10), flag: u32_at(p, data + 0x20).unwrap_or(0) });
                } else {
                    out.save_points_dropped += 1;
                }
            }
            2 => out.tables.push(data..at + len),
            16 => out.effect_lists.push(effect_list(p, data)),
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

/// The records of a world pack's own character region (its type-32 chunk's type-36 table), or
/// none when the pack embeds no region.
pub fn embedded_records(bytes: &[u8], chain: &[Chunk]) -> Option<Records> {
    let c = chain.iter().find(|c| c.kind == EMBEDDED)?;
    let payload = bytes.get(c.payload.clone())?;
    let nested = crate::resource::walk_chunks(payload, None).ok()?;
    let t = nested.chunks.iter().find(|c| c.kind == RECORDS)?;
    Some(records(payload.get(t.payload.clone())?))
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

    /// SPEC-0034 test vector 10 and a synthetic one-entry list.
    #[test]
    fn effect_lists_and_attach_slots() {
        assert_eq!((attach_slot(0x15), attach_slot(0x4FFF_FFFF)), (Some(5), None));
        let mut p = vec![0u8; 0x10 + 0x10 + 0x50 + 0x10];
        p[0] = 16;
        p[0x14] = 1;
        let e = 0x20;
        p[e + 0x20..e + 0x24].copy_from_slice(&7u32.to_le_bytes());
        p[e + 0x24..e + 0x28].copy_from_slice(&0x4FFF_FFFFu32.to_le_bytes());
        p[e + 0x28..e + 0x2C].copy_from_slice(&0.3f32.to_le_bytes());
        p[e + 0x2C..e + 0x30].copy_from_slice(&1.0f32.to_le_bytes());
        p[e + 0x30..e + 0x34].copy_from_slice(&474i32.to_le_bytes());
        p[e + 0x3E] = 255;
        p[e + 0x48] = 255;
        p[0x70] = 0x7F;
        let r = records(&p);
        assert_eq!(r.effect_lists.len(), 1);
        let x = r.effect_lists[0][0];
        assert_eq!((x.slot_a, x.slot_b, x.start, x.end, x.id, x.trail, x.positional), (Some(7), None, 0.3, 1.0, 474, false, 255));
    }

    #[test]
    fn save_points_are_read_and_capped_at_eight() {
        let mut p = vec![0u8; 0x40 * 9 + 0x10];
        for k in 0..9 {
            let at = 0x40 * k;
            p[at] = 4;
            p[at + 0x10..at + 0x14].copy_from_slice(&(k as f32).to_le_bytes());
            p[at + 0x30] = k as u8;
        }
        p[0x40 * 9] = 0x7F;
        let r = records(&p);
        assert_eq!(r.save_points.len(), 8);
        assert_eq!(r.save_points_dropped, 1);
        assert_eq!(r.save_points[3].position, [3.0, 0.0, 0.0]);
        assert_eq!(r.save_points[5].flag, 5);
    }

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
