//! Sound banks (SPEC-0031 Layout sections 1 to 6 and 8): the file header, the "SBv2" section with
//! its object record, directory groups and voice records, the sample pool with its extents and
//! loop classes (decoded to PCM with the PS-ADPCM rules of SPEC-0009), and the MMID music
//! container's "MID " children (tempo, PPQ and the raw event stream).

use crate::audio::{Adpcm, Frame};
use std::collections::HashMap;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

/// A voice record (section 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voice {
    pub priority: u8,
    pub volume: u8,
    /// The centre byte (+0x02) and the centre fine (+0x03).
    pub centre: u8,
    pub fine: u8,
    pub pan: i16,
    pub low: i8,
    pub high: i8,
    /// Bend ranges down (+0x08) and up (+0x09), in semitones.
    pub bend: (u8, u8),
    pub adsr1: u16,
    pub adsr2: u16,
    pub flags: u16,
    /// The sample's pool offset (relative to the pool start), and its sample id.
    pub offset: u32,
    pub sample_id: u32,
}

impl Voice {
    fn parse(r: &[u8]) -> Option<Self> {
        Some(Self {
            priority: r[0],
            volume: r[1],
            centre: r[2],
            fine: r[3],
            pan: u16_at(r, 4)? as i16,
            low: r[6] as i8,
            high: r[7] as i8,
            bend: (r[8], r[9]),
            adsr1: u16_at(r, 0x0A)?,
            adsr2: u16_at(r, 0x0C)?,
            flags: u16_at(r, 0x0E)?,
            offset: u32_at(r, 0x10)?,
            sample_id: u32_at(r, 0x14)?,
        })
    }
}

/// A directory group (section 3): its volume, pan angle and voices (empty when its count is 0).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Group {
    pub volume: u8,
    pub pan: u16,
    pub voices: Vec<Voice>,
}

/// A decoded sample: PCM at the sample's own rate (set by the pitch register), the loop start
/// (the first bit-2 frame) when its end frame repeats, and its class.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sample {
    pub pcm: Vec<i16>,
    pub loop_start: Option<usize>,
}

/// A "MID " child of the MMID container (section 8a): initial tempo (microseconds per quarter
/// note), PPQ and the event bytes from the first event.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sequence {
    pub tempo: u32,
    pub ppq: u32,
    pub events: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bank {
    pub name: String,
    /// The object record's default volume (+0x10).
    pub master: u16,
    pub groups: Vec<Group>,
    /// Samples by pool offset.
    pub samples: HashMap<u32, Sample>,
    pub children: Vec<Sequence>,
}

/// Decode one sample's pool bytes: up to and including the first end-flag frame; a repeat bit
/// on that frame makes it a loop from the first loop-start frame (section 6).
pub fn decode_sample(bytes: &[u8]) -> Sample {
    let mut dec = Adpcm::default();
    let mut pcm = Vec::with_capacity(bytes.len() / 16 * 28);
    let mut loop_mark = None;
    let mut loop_start = None;
    for f in bytes.chunks_exact(16) {
        if f[1] & 4 != 0 && loop_mark.is_none() {
            loop_mark = Some(pcm.len());
        }
        match dec.frame(f, &mut pcm) {
            Frame::Invalid => break,
            _ if f[1] & 1 != 0 => {
                if f[1] & 2 != 0 {
                    loop_start = Some(loop_mark.unwrap_or(0));
                }
                break;
            }
            _ => {}
        }
    }
    Sample { pcm, loop_start }
}

impl Bank {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        let w = |at: usize| u32_at(b, at).ok_or_else(|| "Truncated bank header".to_string());
        if (w(0)?, w(4)?, w(8)?) != (1, 3, 0x20) {
            return Err("Not a sound bank".into());
        }
        let (len, pool, pool_len, tail) = (w(0x0C)? as usize, w(0x10)? as usize, w(0x14)? as usize, w(0x18)? as usize);
        let s = b.get(0x20..0x20 + len).ok_or("Truncated bank section")?;
        if s.get(..4) != Some(b"SBv2") {
            return Err("Bank section without SBv2".into());
        }
        let name: String = s[0x0C..0x10].iter().map(|&c| c as char).collect();
        let n2 = u16_at(s, 0x16).ok_or("Truncated section")? as usize;
        let w8 = u32_at(s, 0x20).ok_or("Truncated section")? as usize;
        let master = u16_at(s, 0x34 + 0x10).unwrap_or(127);
        let mut groups = Vec::with_capacity(n2);
        for g in 0..n2 {
            let r = s.get(w8 + 8 * g..w8 + 8 * g + 8).ok_or("Truncated directory")?;
            let count = r[0] as usize;
            let first = u32_at(r, 4).unwrap_or(0) as usize;
            let voices = (0..count.min(32)).filter_map(|i| s.get(first + 24 * i..first + 24 * i + 24).and_then(Voice::parse)).collect();
            groups.push(Group { volume: r[1], pan: u16_at(r, 2).unwrap_or(0), voices });
        }
        // Sample extents: distinct pool offsets in order; each ends where the next starts.
        let pool_bytes = b.get(pool..pool + pool_len).ok_or("Truncated sample pool")?;
        let mut offsets: Vec<u32> = groups.iter().flat_map(|g| g.voices.iter().map(|v| v.offset)).collect();
        offsets.sort_unstable();
        offsets.dedup();
        let mut samples = HashMap::new();
        for (i, &o) in offsets.iter().enumerate() {
            let end = offsets.get(i + 1).map_or(pool_len, |&n| n as usize);
            if let Some(bytes) = pool_bytes.get(o as usize..end.min(pool_len)) {
                samples.insert(o, decode_sample(bytes));
            }
        }
        let mut children = Vec::new();
        let m = b.get(tail + 0x10..).ok_or("Truncated bank tail")?;
        if m.get(..4) == Some(b"MMID") {
            let count = m[7] as usize;
            for k in 0..count {
                let Some(off) = u32_at(m, 0x10 + 4 * k) else { break };
                let c = m.get(off as usize..).unwrap_or(&[]);
                if c.get(..4) != Some(b"MID ") {
                    continue;
                }
                let first = u32_at(c, 0x18).unwrap_or(0xE8) as usize;
                // A child's events run to the next child (or the end of the container).
                let next = (k + 1..count).filter_map(|j| u32_at(m, 0x10 + 4 * j)).map(|o| o as usize).find(|&o| o > off as usize);
                let end = next.map_or(m.len(), |n| n.min(m.len())) - off as usize;
                children.push(Sequence { tempo: u32_at(c, 0x20).unwrap_or(500_000), ppq: u32_at(c, 0x24).unwrap_or(480), events: c.get(first..end).unwrap_or(&[]).to_vec() });
            }
        }
        Ok(Self { name, master, groups, samples, children })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_classes() {
        // Frame flags: one-shot 0, 1 (end), 7; loop 0, 4 (loop start), 2, 3 (end + repeat).
        let frame = |flags: u8| {
            let mut f = [0u8; 16];
            f[0] = 0x0C;
            f[1] = flags;
            f[2] = 0x01;
            f
        };
        let one: Vec<u8> = [0u8, 1, 7].iter().flat_map(|&f| frame(f)).collect();
        let s = decode_sample(&one);
        assert_eq!((s.pcm.len(), s.loop_start), (56, None), "stops at the 0x01 frame");
        let lp: Vec<u8> = [0u8, 4, 2, 3].iter().flat_map(|&f| frame(f)).collect();
        let s = decode_sample(&lp);
        assert_eq!((s.pcm.len(), s.loop_start), (112, Some(28)));
        // The "other" class (SPEC-0031, 312 samples): no end frame before the final one, which
        // carries all three bits, so it loops its last frame.
        let other: Vec<u8> = [0u8, 0, 7].iter().flat_map(|&f| frame(f)).collect();
        let s = decode_sample(&other);
        assert_eq!((s.pcm.len(), s.loop_start), (84, Some(56)));
    }

    #[test]
    fn bank_layout() {
        // A synthetic bank: one group of one voice, one two-frame sample, one MID child.
        let mut s = vec![0u8; 0x50 + 8 + 24];
        s[..4].copy_from_slice(b"SBv2");
        s[0x0C..0x10].copy_from_slice(b"TEST");
        s[0x16..0x18].copy_from_slice(&1u16.to_le_bytes());
        s[0x18..0x1A].copy_from_slice(&1u16.to_le_bytes());
        s[0x20..0x24].copy_from_slice(&0x50u32.to_le_bytes());
        s[0x24..0x28].copy_from_slice(&0x58u32.to_le_bytes());
        s[0x34 + 0x10..0x34 + 0x12].copy_from_slice(&127u16.to_le_bytes());
        s[0x50] = 1;
        s[0x51] = 100;
        s[0x54..0x58].copy_from_slice(&0x58u32.to_le_bytes());
        let v = 0x58;
        s[v + 1] = 100;
        s[v + 2] = 0xBC;
        s[v + 7] = 127;
        s[v + 0x0A..v + 0x0C].copy_from_slice(&0x80FFu16.to_le_bytes());
        let pool_at = 0x20 + s.len();
        let mut pool = vec![0u8; 32];
        pool[0] = 0x0C;
        pool[2] = 0x07;
        pool[16] = 0x0C;
        pool[17] = 1;
        let tail = pool_at + pool.len();
        let mut b = vec![0u8; 0x20];
        for (k, x) in [1u32, 3, 0x20, s.len() as u32, pool_at as u32, 32, tail as u32, 0].iter().enumerate() {
            b[4 * k..4 * k + 4].copy_from_slice(&x.to_le_bytes());
        }
        b.extend(&s);
        b.extend(&pool);
        b.extend([2u32, 1, 0x10, 0].iter().flat_map(|x| x.to_le_bytes()));
        let mut m = vec![0u8; 0x14];
        m[..4].copy_from_slice(b"MMID");
        m[7] = 1;
        m[0x10..0x14].copy_from_slice(&0x14u32.to_le_bytes());
        let mut c = vec![0u8; 0xE8];
        c[..4].copy_from_slice(b"MID ");
        c[0x18..0x1C].copy_from_slice(&0xE8u32.to_le_bytes());
        c[0x20..0x24].copy_from_slice(&500_000u32.to_le_bytes());
        c[0x24..0x28].copy_from_slice(&480u32.to_le_bytes());
        c.extend([0x00, 0xFF, 0x2F, 0x00]);
        m.extend(c);
        b.extend(m);
        let bank = Bank::parse(&b).unwrap();
        assert_eq!((bank.name.as_str(), bank.master, bank.groups.len()), ("TEST", 127, 1));
        let g = &bank.groups[0];
        assert_eq!((g.volume, g.voices.len(), g.voices[0].centre, g.voices[0].adsr1), (100, 1, 0xBC, 0x80FF));
        assert_eq!(bank.samples[&0].pcm.len(), 56);
        assert_eq!(bank.samples[&0].pcm[0], 7);
        assert_eq!(bank.children.len(), 1);
        assert_eq!((bank.children[0].tempo, bank.children[0].ppq, &bank.children[0].events[..]), (500_000, 480, &[0x00, 0xFF, 0x2F, 0x00][..]));
    }
}
