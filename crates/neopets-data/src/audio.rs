//! Audio clips (SPEC-0009 Layout): the public PS-ADPCM frame decoder, VAG mono clips
//! (resources 256 to 274, and the clips of the emote banks 144 and 145), and the " KPV" stereo
//! streams (resources 146 to 255), decoded to 16-bit PCM.

/// The standard PS-ADPCM filter pairs, in 1/64 units.
const F0: [i32; 5] = [0, 60, 115, 98, 122];
const F1: [i32; 5] = [0, 0, -52, -55, -60];

/// One channel's decoder state: the two history samples.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Adpcm {
    h1: i32,
    h2: i32,
}

/// What a frame was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    /// 28 samples were decoded; `end` is the loop-end / end-of-data flag (bit 0 of byte 1).
    Audio { end: bool },
    /// A silent padding frame (header byte 0xC0 in the streams): 28 zero samples.
    Silent,
    /// A filter index above 4: decoding stops here.
    Invalid,
}

impl Adpcm {
    /// Decode one 16-byte frame, appending 28 samples (none for an invalid frame).
    pub fn frame(&mut self, f: &[u8], out: &mut Vec<i16>) -> Frame {
        if f.len() < 16 {
            return Frame::Invalid;
        }
        if f[0] == 0xC0 && f[1..16].iter().all(|&b| b == 0) {
            out.extend([0i16; 28]);
            *self = Self::default();
            return Frame::Silent;
        }
        let filter = (f[0] >> 4) as usize;
        let shift = (f[0] & 0x0F) as i32;
        if filter > 4 {
            return Frame::Invalid;
        }
        for i in 0..28 {
            let byte = f[2 + i / 2];
            let nibble = if i % 2 == 0 { byte & 0x0F } else { byte >> 4 };
            // The 4-bit sample, sign-extended, at the top of a 16-bit range, then shifted.
            let s = (((nibble as i32) << 28) >> 16) >> shift.min(12);
            let v = s + ((self.h1 * F0[filter] + self.h2 * F1[filter] + 32) >> 6);
            let v = v.clamp(i16::MIN as i32, i16::MAX as i32);
            self.h2 = self.h1;
            self.h1 = v;
            out.push(v as i16);
        }
        Frame::Audio { end: f[1] & 1 != 0 }
    }
}

/// Decoded PCM: interleaved samples, the channel count and the sample rate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pcm {
    pub samples: Vec<i16>,
    pub channels: u16,
    pub rate: u32,
}

impl Pcm {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / self.channels.max(1) as f32 / self.rate.max(1) as f32
    }
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// A VAG clip (big-endian header: "VAGp", data size at 0x0C, rate at 0x10, name at 0x20,
/// frames from 0x30), decoded until its data size or an invalid frame. The end flag does not
/// stop it: the data size does.
pub fn vag(b: &[u8]) -> Result<(Pcm, String), String> {
    if b.get(..4) != Some(b"VAGp") {
        return Err("Not a VAG clip".into());
    }
    let size = be32(b, 0x0C).ok_or("Truncated VAG header")? as usize;
    let rate = be32(b, 0x10).ok_or("Truncated VAG header")?;
    let name: String = b.get(0x20..0x30).unwrap_or(&[]).iter().take_while(|&&c| c != 0).map(|&c| c as char).collect();
    let data = b.get(0x30..(0x30 + size).min(b.len())).ok_or("Truncated VAG data")?;
    let mut dec = Adpcm::default();
    let mut samples = Vec::with_capacity(size / 16 * 28);
    for f in data.chunks_exact(16) {
        if dec.frame(f, &mut samples) == Frame::Invalid {
            break;
        }
    }
    Ok((Pcm { samples, channels: 1, rate }, name))
}

/// The " KPV" stream header (little-endian): bytes per channel, interleave, rate, channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KpvHeader {
    pub per_channel: u32,
    pub interleave: u32,
    pub rate: u32,
    pub channels: u32,
}

pub fn kpv_header(b: &[u8]) -> Result<KpvHeader, String> {
    if b.get(..4) != Some(b" KPV") {
        return Err("Not a KPV stream".into());
    }
    let w = |at| le32(b, at).ok_or_else(|| "Truncated KPV header".to_string());
    Ok(KpvHeader { per_channel: w(4)?, interleave: w(8)?, rate: w(0x10)?, channels: w(0x14)? })
}

/// A stereo stream decoded to interleaved PCM: data from 0x800 in interleave blocks, left then
/// right, every block to the end of the resource (silent frames give silence), then the trailing
/// silence trimmed.
pub fn kpv(b: &[u8]) -> Result<Pcm, String> {
    let h = kpv_header(b)?;
    let (n, block) = (h.channels.max(1) as usize, h.interleave as usize);
    if block == 0 || block % 16 != 0 || n > 8 {
        return Err("Bad KPV interleave".into());
    }
    let mut decs = vec![Adpcm::default(); n];
    let mut chans: Vec<Vec<i16>> = vec![Vec::new(); n];
    for (k, blk) in b.get(0x800..).unwrap_or(&[]).chunks(block).enumerate() {
        let c = k % n;
        for f in blk.chunks_exact(16) {
            if decs[c].frame(f, &mut chans[c]) == Frame::Invalid {
                // A frame with a bad filter: silence for it (none occur on disc).
                chans[c].extend([0i16; 28]);
            }
        }
    }
    let len = chans.iter().map(Vec::len).min().unwrap_or(0);
    let mut samples = Vec::with_capacity(len * n);
    for i in 0..len {
        for ch in &chans {
            samples.push(ch[i]);
        }
    }
    // Trim the trailing silence (every stream ends in a silent block).
    let last = samples.iter().rposition(|&s| s != 0).map_or(0, |i| i / n + 1);
    samples.truncate(last * n);
    Ok(Pcm { samples, channels: n as u16, rate: h.rate })
}

/// A music row of design table 4 (SPEC-0024; SPEC-0009 Track row): stream, bank and block
/// offsets, the stream cap, the bank level and the cue scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicRow {
    pub stream: i32,
    pub bank: i32,
    pub block: i32,
    pub cap: i32,
    pub level: i32,
    pub scale: i32,
}

impl MusicRow {
    /// The defaults for row ids 193 and above.
    pub const DEFAULT: MusicRow = MusicRow { stream: -1, bank: 1, block: 1, cap: 0, level: 1024, scale: 1024 };

    pub fn from_record(r: &[u8]) -> Option<Self> {
        let w = |at: usize| le32(r, at).map(|v| v as i32);
        Some(Self { stream: w(0)?, bank: w(4)?, block: w(8)?, cap: w(0x0C)?, level: w(0x10)?, scale: w(0x14)? })
    }
}

/// A world pack's type-1001 record (SPEC-0009 Area to music row): the key (record +0, low 16
/// bits) and the selector (record +8), after the 16-byte child header.
pub fn pack_record(bytes: &[u8], chain: &[crate::resource::Chunk]) -> Option<(i32, u32)> {
    let c = chain.iter().find(|c| c.kind == 1001)?;
    let p = bytes.get(c.payload.clone())?;
    Some((le32(p, 0x10)? as u16 as i32, le32(p, 0x18)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_decode_with_their_shift_and_filter() {
        // Filter 0, shift 0, every nibble 1: every sample is 4096; shift 12 gives 1.
        let mut f = [0x11u8; 16];
        f[0] = 0x00;
        f[1] = 0x00;
        let mut out = Vec::new();
        assert_eq!(Adpcm::default().frame(&f, &mut out), Frame::Audio { end: false });
        assert!(out.len() == 28 && out.iter().all(|&s| s == 4096));
        f[0] = 0x0C;
        f[1] = 0x01;
        out.clear();
        assert_eq!(Adpcm::default().frame(&f, &mut out), Frame::Audio { end: true });
        assert!(out.iter().all(|&s| s == 1));
        // Nibble 8 is -8: shift 12 gives -8.
        let mut g = [0x88u8; 16];
        g[0] = 0x0C;
        g[1] = 0;
        out.clear();
        Adpcm::default().frame(&g, &mut out);
        assert!(out.iter().all(|&s| s == -8));
        // Filter 1 carries the history: 4096, then 4096 + (4096 x 60 + 32) >> 6 = 7936, ...
        let mut h = [0u8; 16];
        h[0] = 0x10;
        h[2] = 0x01;
        out.clear();
        Adpcm::default().frame(&h, &mut out);
        assert_eq!(&out[..3], &[4096, 3840, 3600]);
        // A silent padding frame, and a bad filter.
        let mut silent = [0u8; 16];
        silent[0] = 0xC0;
        out.clear();
        assert_eq!(Adpcm::default().frame(&silent, &mut out), Frame::Silent);
        assert_eq!(out.len(), 28);
        let mut bad = [0u8; 16];
        bad[0] = 0x50;
        assert_eq!(Adpcm::default().frame(&bad, &mut out), Frame::Invalid);
    }

    #[test]
    fn vag_and_kpv_containers() {
        // A VAG of two frames at 22,050 Hz named "Revive".
        let mut v = vec![0u8; 0x30 + 32];
        v[..4].copy_from_slice(b"VAGp");
        v[0x0C..0x10].copy_from_slice(&32u32.to_be_bytes());
        v[0x10..0x14].copy_from_slice(&22_050u32.to_be_bytes());
        v[0x20..0x26].copy_from_slice(b"Revive");
        v[0x30] = 0x0C;
        v[0x32] = 0x11;
        let (pcm, name) = vag(&v).unwrap();
        assert_eq!((pcm.samples.len(), pcm.rate, pcm.channels, name.as_str()), (56, 22_050, 1, "Revive"));
        assert_eq!(&pcm.samples[..3], &[1, 1, 0]);
        // Header arithmetic: 0x13540 bytes are 4,948 frames, 138,544 samples, 6.28 s.
        assert_eq!(0x13540 / 16 * 28, 138_544);
        // A KPV with 32-byte blocks: left, right, left, right; then a silent block trimmed.
        let mut k = vec![0u8; 0x800 + 32 * 6];
        k[..4].copy_from_slice(b" KPV");
        k[4..8].copy_from_slice(&64u32.to_le_bytes());
        k[8..12].copy_from_slice(&32u32.to_le_bytes());
        k[0x10..0x14].copy_from_slice(&22_050u32.to_le_bytes());
        k[0x14..0x18].copy_from_slice(&2u32.to_le_bytes());
        for blk in 0..4 {
            for fr in 0..2 {
                let at = 0x800 + 32 * blk + 16 * fr;
                k[at] = 0x0C;
                k[at + 2] = if blk % 2 == 0 { 0x01 } else { 0x02 };
            }
        }
        for blk in 4..6 {
            k[0x800 + 32 * blk] = 0xC0;
            k[0x800 + 32 * blk + 16] = 0xC0;
        }
        let pcm = kpv(&k).unwrap();
        assert_eq!((pcm.channels, pcm.rate), (2, 22_050));
        assert_eq!(&pcm.samples[..2], &[1, 2], "left then right");
        // Left samples (1 then 27 zeros per frame) end at the last nonzero pair.
        assert_eq!(pcm.samples.len() % 2, 0);
        assert!(pcm.samples.len() <= 2 * 28 * 4);
        // The stream length rule: S = 1,114,144 at 22,050 Hz is about 88.4 s.
        assert!(((1_114_144.0 / 16.0 * 28.0 / 22_050.0) - 88.4f32).abs() < 0.05);
    }

    #[test]
    fn music_rows() {
        let mut r = vec![0u8; 40];
        for (k, v) in [14i32, 18, 16, 700, 650, 1024].iter().enumerate() {
            r[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
        }
        let row = MusicRow::from_record(&r).unwrap();
        assert_eq!((row.stream + 146, row.bank + 10, row.block + 395), (160, 28, 411));
        assert_eq!(MusicRow::DEFAULT.bank + 10, 11);
    }
}
