//! Sound blocks (SPEC-0031 Layout section 7, resources 347 to 490): the file header, the "SBlk"
//! section with its 12-byte cue entries, 8-byte command records and 24-byte voice records, and the
//! block's own PS-ADPCM pool; a voice's sample is decoded from its pool offset and length.

use crate::bank::{decode_sample, Sample};
use std::collections::HashMap;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// A cue entry (12 bytes from section +0x40; entry k is cue id k).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cue {
    pub volume: u8,
    pub group: u8,
    pub pan: u8,
    pub count: u8,
    pub limit: u8,
    pub flags: u16,
    /// Index of the first command record (the byte offset / 8).
    pub first: usize,
}

/// A command record: the 24-bit operand (bytes 0 to 2), the type and the delay in ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Command {
    pub operand: [u8; 3],
    pub kind: u8,
    pub delay: u16,
}

impl Command {
    pub fn value(&self) -> u32 {
        self.operand[0] as u32 | (self.operand[1] as u32) << 8 | (self.operand[2] as u32) << 16
    }
}

/// A block voice record (24 bytes, selected by a type-1 or type-9 operand).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockVoice {
    pub priority: u8,
    pub level: i8,
    pub centre: u8,
    pub fine: u8,
    pub pan: i16,
    pub bend: (u8, u8),
    pub adsr1: u16,
    pub adsr2: u16,
    pub flags: u16,
    /// Sample offset in the block's pool and length in bytes.
    pub offset: u32,
    pub length: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    pub cues: Vec<Cue>,
    pub commands: Vec<Command>,
    /// Voice records by their byte offset in the voice table (the type-1 operand).
    pub voices: HashMap<u32, BlockVoice>,
    /// The raw records of the type-4 (16 bytes) and type-5 (32 bytes) commands, by voice-table offset.
    pub records: HashMap<u32, Vec<u8>>,
    /// Decoded samples by (pool offset, length).
    pub samples: HashMap<(u32, u32), Sample>,
}

impl Block {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let w = |at: usize| u32_at(b, at).ok_or_else(|| "Truncated block header".to_string());
        if (w(0)?, w(4)?, w(8)?) != (3, 2, 0x18) {
            return Err("Not a sound block".into());
        }
        let (len, pool, pool_len) = (w(0x0C)? as usize, w(0x10)? as usize, w(0x14)? as usize);
        let s = b.get(0x18..0x18 + len).ok_or("Truncated block section")?;
        if s.get(..4) != Some(b"SBlk") {
            return Err("Block section without SBlk".into());
        }
        let n2 = u16_at(s, 0x16).ok_or("Truncated section")? as usize;
        let n1 = u16_at(s, 0x18).ok_or("Truncated section")? as usize;
        let w8 = u32_at(s, 0x20).ok_or("Truncated section")? as usize;
        let w13 = u32_at(s, 0x34).ok_or("Truncated section")? as usize;
        let cues = (0..n2)
            .filter_map(|k| {
                let e = s.get(0x40 + 12 * k..0x40 + 12 * k + 12)?;
                Some(Cue { volume: e[0], group: e[1], pan: e[2], count: e[4], limit: e[5], flags: u16_at(e, 6)?, first: u16_at(e, 8)? as usize / 8 })
            })
            .collect();
        let commands = (0..n1)
            .filter_map(|k| {
                let r = s.get(w8 + 8 * k..w8 + 8 * k + 8)?;
                Some(Command { operand: [r[0], r[1], r[2]], kind: r[3], delay: u16_at(r, 4)? })
            })
            .collect::<Vec<_>>();
        let pool_bytes = b.get(pool..pool + pool_len).ok_or("Truncated block pool")?;
        let mut voices = HashMap::new();
        let mut samples = HashMap::new();
        for c in commands.iter().filter(|c| matches!(c.kind, 1 | 9)) {
            let off = c.value();
            let Some(r) = s.get(w13 + off as usize..w13 + off as usize + 24) else { continue };
            let v = BlockVoice {
                priority: r[0],
                level: r[1] as i8,
                centre: r[2],
                fine: r[3],
                pan: u16_at(r, 4).unwrap_or(0) as i16,
                bend: (r[8], r[9]),
                adsr1: u16_at(r, 0x0A).unwrap_or(0),
                adsr2: u16_at(r, 0x0C).unwrap_or(0),
                flags: u16_at(r, 0x0E).unwrap_or(0),
                offset: u32_at(r, 0x10).unwrap_or(0),
                length: u32_at(r, 0x14).unwrap_or(0),
            };
            let key = (v.offset, v.length);
            if !samples.contains_key(&key) {
                if let Some(bytes) = pool_bytes.get(v.offset as usize..(v.offset + v.length) as usize) {
                    samples.insert(key, decode_sample(bytes));
                }
            }
            voices.insert(off, v);
        }
        let mut records = HashMap::new();
        for c in commands.iter().filter(|c| matches!(c.kind, 4 | 5)) {
            let (off, len) = (c.value(), if c.kind == 4 { 16 } else { 32 });
            if let Some(r) = s.get(w13 + off as usize..w13 + off as usize + len) {
                records.insert(off, r.to_vec());
            }
        }
        Ok(Self { cues, commands, voices, records, samples })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_layout() {
        // One cue of two commands (voice start at voice-table offset 0, then end), one voice of
        // one frame.
        let mut s = vec![0u8; 0x40 + 12 + 16 + 24];
        s[..4].copy_from_slice(b"SBlk");
        s[0x16..0x18].copy_from_slice(&1u16.to_le_bytes());
        s[0x18..0x1A].copy_from_slice(&2u16.to_le_bytes());
        s[0x20..0x24].copy_from_slice(&(0x40u32 + 12).to_le_bytes());
        s[0x34..0x38].copy_from_slice(&(0x40u32 + 12 + 16).to_le_bytes());
        s[0x40] = 100;
        s[0x44] = 2;
        let c = 0x40 + 12;
        s[c + 3] = 1;
        s[c + 8 + 3] = 43;
        s[c + 8 + 4] = 5;
        let v = c + 16;
        s[v + 1] = 110;
        s[v + 2] = 0xBC;
        s[v + 0x14..v + 0x18].copy_from_slice(&16u32.to_le_bytes());
        let mut b = vec![0u8; 0x18];
        for (k, x) in [3u32, 2, 0x18, s.len() as u32, 0x18 + s.len() as u32, 16].iter().enumerate() {
            b[4 * k..4 * k + 4].copy_from_slice(&x.to_le_bytes());
        }
        b.extend(&s);
        let mut frame = [0u8; 16];
        frame[0] = 0x0C;
        frame[1] = 1;
        frame[2] = 0x03;
        b.extend(frame);
        let blk = Block::parse(&b).unwrap();
        assert_eq!(blk.cues[0], Cue { volume: 100, count: 2, ..Default::default() });
        assert_eq!((blk.commands[0].kind, blk.commands[1].kind, blk.commands[1].delay), (1, 43, 5));
        assert_eq!((blk.voices[&0].level, blk.voices[&0].length), (110, 16));
        assert_eq!(blk.samples[&(0, 16)].pcm[0], 3);
    }
}
