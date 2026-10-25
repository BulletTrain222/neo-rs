//! The game design tables of resource 533 (SPEC-0024 Container, Pointers and string
//! relocation): the k-th type-42 chunk is table k, an IDMX block of fixed-size records with a
//! string pool behind it; pointer fields are offsets from the block's start. Only tables 10
//! (hero name colours) and 13 (action labels) have accessors so far.

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

#[derive(Clone, Debug, Default)]
pub struct DesignTables {
    pub tables: Vec<Table>,
}

impl DesignTables {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let chain = resource::walk_chunks(bytes, None)?;
        let mut tables = Vec::new();
        for c in chain.chunks.iter().filter(|c| c.kind == TABLE) {
            let block = bytes[c.payload.clone()].to_vec();
            if block.get(..4) != Some(b"IDMX") {
                return Err("Design table without an IDMX block".into());
            }
            let count = u16::from_le_bytes([block[0x0C], block[0x0D]]) as usize;
            let size = u16::from_le_bytes([block[0x0E], block[0x0F]]) as usize;
            tables.push(Table { block, count, size });
        }
        Ok(Self { tables })
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
