//! The game design tables of resource 533 (SPEC-0024 Container, Pointers and string
//! relocation): the k-th type-42 chunk is table k, an IDMX block of fixed-size records with a
//! string pool behind it; pointer fields are offsets from the block's start. Accessors cover
//! table 0 (the item text rows: category and carry limit, SPEC-0021), table 5 (location names,
//! SPEC-0051 Map area names), table 9 (system message boxes), table 10 (hero name colours) and
//! table 13 (action labels).

use crate::resource;

pub const RESOURCE: u32 = 533;
const TABLE: u16 = 42;

#[derive(Clone, Debug, Default)]
pub struct Table {
    /// The IDMX block and its string pool (the chunk payload).
    block: Vec<u8>,
    pub count: usize,
    pub size: usize,
}

impl Table {
    pub fn record(&self, i: usize) -> Option<&[u8]> {
        (i < self.count).then(|| self.block.get(0x10 + self.size * i..0x10 + self.size * (i + 1))).flatten()
    }

    /// The NUL-terminated string a pointer field names (0: none).
    pub fn string(&self, ptr: u32) -> Option<&[u8]> {
        if ptr == 0 {
            return None;
        }
        let s = self.block.get(ptr as usize..)?;
        Some(&s[..s.iter().position(|&b| b == 0).unwrap_or(s.len())])
    }
}

/// A system message box (table 9).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Message {
    pub text: Vec<u8>,
    /// The first (Circle) and second (Triangle) button labels.
    pub labels: [Option<Vec<u8>>; 2],
    pub colour: [u8; 3],
}

#[derive(Clone, Debug, Default)]
pub struct DesignTables {
    pub tables: Vec<Table>,
    /// The path records (type-43 chunks, in order: the character emote bank 144, the hero
    /// emote bank 145, the books 576): each entry's (start, length) in 2,048-byte units.
    pub paths: Vec<Vec<(u32, u32)>>,
}

/// An emote row of table 7 or 8: clip index (255 none), chance in percent, variance byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmoteRow {
    pub clip: u8,
    pub chance: u8,
    pub variance: u8,
}

impl DesignTables {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let chain = resource::walk_chunks(bytes, None)?;
        let mut tables = Vec::new();
        let mut paths = Vec::new();
        for c in chain.chunks.iter().filter(|c| c.kind == 43) {
            let p = &bytes[c.payload.clone()];
            let n = p.get(0x100..0x104).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap())) as usize;
            let entries = (0..n.min(1 << 14))
                .map_while(|i| {
                    let at = 0x104 + 8 * i;
                    let w = |o: usize| p.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
                    Some((w(at)?, w(at + 4)?))
                })
                .collect();
            paths.push(entries);
        }
        for c in chain.chunks.iter().filter(|c| c.kind == TABLE) {
            let block = bytes[c.payload.clone()].to_vec();
            if block.get(..4) != Some(b"IDMX") {
                return Err("Design table without an IDMX block".into());
            }
            let count = u16::from_le_bytes([block[0x0C], block[0x0D]]) as usize;
            let size = u16::from_le_bytes([block[0x0E], block[0x0F]]) as usize;
            tables.push(Table { block, count, size });
        }
        Ok(Self { tables, paths })
    }

    /// Table 11 (SPEC-0024; SPEC-0049 Stage table rows): the progress flags stage `stage` sets,
    /// in table order (rows with flag -1 skipped).
    pub fn stage_flags(&self, stage: u16) -> Vec<u16> {
        let Some(t) = self.tables.get(11) else { return Vec::new() };
        (0..t.count)
            .filter_map(|i| t.record(i))
            .filter(|r| r.len() >= 4 && u16::from_le_bytes([r[0], r[1]]) == stage)
            .filter_map(|r| u16::try_from(i16::from_le_bytes([r[2], r[3]])).ok())
            .collect()
    }

    /// Table 12 (SPEC-0024): the start-point item grants of stage `stage` as (item, count), in
    /// table order (rows with item -1 or a count below 1 skipped).
    pub fn stage_items(&self, stage: u16) -> Vec<(u16, u32)> {
        let Some(t) = self.tables.get(12) else { return Vec::new() };
        (0..t.count)
            .filter_map(|i| t.record(i))
            .filter(|r| r.len() >= 6 && u16::from_le_bytes([r[0], r[1]]) == stage)
            .filter_map(|r| {
                let item = u16::try_from(i16::from_le_bytes([r[2], r[3]])).ok()?;
                let n = i16::from_le_bytes([r[4], r[5]]);
                (n > 0).then_some((item, n as u32))
            })
            .collect()
    }

    fn word(&self, table: usize, i: usize, at: usize) -> Option<u32> {
        let r = self.tables.get(table)?.record(i)?;
        Some(u32::from_le_bytes(r.get(at..at + 4)?.try_into().ok()?))
    }

    /// Table 0, row id: the item's text category (+0x04).
    pub fn item_category(&self, id: u16) -> Option<u32> {
        self.word(0, id as usize, 4)
    }

    /// Table 0, row id: the item name (pointer at +0x00; SPEC-0021 Item names).
    pub fn item_name(&self, id: u16) -> Option<Vec<u8>> {
        let ptr = self.word(0, id as usize, 0)?;
        self.tables.first()?.string(ptr).map(<[u8]>::to_vec).filter(|n| !n.is_empty())
    }

    /// Table 0, row id: the item description (pointer at +0x20).
    pub fn item_description(&self, id: u16) -> Option<Vec<u8>> {
        let ptr = self.word(0, id as usize, 0x20)?;
        self.tables.first()?.string(ptr).map(<[u8]>::to_vec).filter(|n| !n.is_empty())
    }

    /// Table 0, row id: the showcase text of a pickup (SPEC-0021 Item text database; SPEC-0033
    /// Behavior 42): the selector's pointer, +0x24 for a first pickup and +0x28 for a later one
    /// (this build's reading of "one text for a first pickup and one for later pickups"); when
    /// it is absent and the progress flag id at +0x1C is not negative, +0x2C while that flag is
    /// clear and +0x30 while it is set.
    pub fn item_showcase_text(&self, id: u16, first: bool, flag: impl Fn(u16) -> bool) -> Option<Vec<u8>> {
        let table = self.tables.first()?;
        let text = |at: usize| self.word(0, id as usize, at).filter(|&p| p != 0).and_then(|p| table.string(p)).map(<[u8]>::to_vec).filter(|t| !t.is_empty());
        text(if first { 0x24 } else { 0x28 }).or_else(|| {
            let progress = self.word(0, id as usize, 0x1C)? as u16 as i16;
            if progress < 0 {
                return None;
            }
            text(if flag(progress as u16) { 0x30 } else { 0x2C })
        })
    }

    /// Table 0, row id: the important flag (u8 +0x1F; SPEC-0021): quest items and equipment.
    pub fn item_important(&self, id: u16) -> bool {
        self.word(0, id as usize, 0x1C).is_some_and(|w| w >> 24 == 1)
    }

    /// Table 0, row id: the carry limit Q (+0x08; 0 = no limit).
    pub fn carry_limit(&self, id: u16) -> Option<u32> {
        self.word(0, id as usize, 8)
    }

    /// Table 16, record i: the attack power table's f32 (SPEC-0036 Layout, Attack power table).
    pub fn attack_power(&self, i: usize) -> Option<f32> {
        self.word(16, i, 0).map(f32::from_bits)
    }

    /// Table 5, row m: the name of map area m.
    pub fn location_name(&self, m: usize) -> Option<Vec<u8>> {
        let t = self.tables.get(5)?;
        t.string(self.word(5, m, 0)?).map(<[u8]>::to_vec)
    }

    /// Table 4: music row `id` (SPEC-0009 Track row); ids 193 and above give the defaults.
    pub fn music_row(&self, id: usize) -> crate::audio::MusicRow {
        if id >= 193 {
            return crate::audio::MusicRow::DEFAULT;
        }
        self.tables.get(4).and_then(|t| t.record(id)).and_then(crate::audio::MusicRow::from_record).unwrap_or(crate::audio::MusicRow::DEFAULT)
    }

    /// Table 2, row n: the journal kind and the stinger value v of progress flag n (v - 1 is the
    /// stinger played when the flag rises; 0 none).
    pub fn flag_cue(&self, n: u16) -> Option<(i32, i32)> {
        Some((self.word(2, n as usize, 0)? as i32, self.word(2, n as usize, 4)? as i32))
    }

    /// Table 9, record i: a system message box (text, first and second button labels, colour
    /// bytes R, G, B; a null colour word gives 128s).
    pub fn message(&self, i: usize) -> Option<Message> {
        let t = self.tables.get(9)?;
        let r = t.record(i)?;
        let word = |at: usize| u32::from_le_bytes(r[at..at + 4].try_into().unwrap());
        let text = |ptr: u32| t.string(ptr).map(<[u8]>::to_vec);
        let c = word(12);
        Some(Message {
            text: text(word(0)).unwrap_or_default(),
            labels: [text(word(4)), text(word(8))],
            colour: if c == 0 { [128; 3] } else { [c as u8, (c >> 8) as u8, (c >> 16) as u8] },
        })
    }

    /// Table 7 (character emotes, `hero` false) or 8 (hero emotes): row `id`.
    pub fn emote(&self, hero: bool, id: i32) -> Option<EmoteRow> {
        let r = self.tables.get(if hero { 8 } else { 7 })?.record(usize::try_from(id).ok()?)?;
        Some(EmoteRow { clip: r[0], chance: r[1], variance: r[2] })
    }

    /// Table 15: the credits rows: (text or None for a spacer, style index, colour set).
    pub fn credits(&self) -> Vec<(Option<Vec<u8>>, u16, u16)> {
        let Some(t) = self.tables.get(15) else { return Vec::new() };
        (0..t.count)
            .filter_map(|i| {
                let r = t.record(i)?;
                let ptr = u32::from_le_bytes(r[0..4].try_into().ok()?);
                Some((t.string(ptr).map(<[u8]>::to_vec), u16::from_le_bytes([r[4], r[5]]), u16::from_le_bytes([r[6], r[7]])))
            })
            .collect()
    }

    /// Table 10: the name colour of hero variant 0 (Tor) or 1 (Roberta), bytes R, G, B.
    pub fn hero_colour(&self, variant: usize) -> Option<[u8; 3]> {
        let r = self.tables.get(10)?.record(variant)?;
        Some([r[0], r[1], r[2]])
    }

    /// Table 13: action label i.
    pub fn label(&self, i: usize) -> Option<Vec<u8>> {
        let t = self.tables.get(13)?;
        let r = t.record(i)?;
        t.string(u32::from_le_bytes(r.try_into().ok()?)).map(<[u8]>::to_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_pointers() {
        // One IDMX block of two 4-byte records: a pointer to "Next" in the pool, and 0.
        let mut block = b"IDMX".to_vec();
        block.extend([76, 0, 0, 0]);
        block.extend(24u32.to_le_bytes());
        block.extend(2u16.to_le_bytes());
        block.extend(4u16.to_le_bytes());
        block.extend(24u32.to_le_bytes());
        block.extend(0u32.to_le_bytes());
        block.extend(b"Next\0");
        let t = Table { count: 2, size: 4, block };
        let p = u32::from_le_bytes(t.record(0).unwrap().try_into().unwrap());
        assert_eq!(t.string(p), Some(&b"Next"[..]));
        assert_eq!(t.string(0), None);
        assert!(t.record(2).is_none());
    }
}
