//! Texture chunks (chunk type 4) and indexed-texture decoding (SPEC-0003). A chunk holds one
//! PSMCT32 pixel block plus a table of sub-textures and palettes addressed in GS memory; decoding
//! uploads the block into an emulated GS memory and reads each sub-texture back in its format.
use crate::gsmem::GsMemory;

pub const CHUNK_TYPE: u16 = 4;
const SEGMENT_SIZE: usize = 0x30;
const ENTRY_SIZE: usize = 0x30;
const ENTRIES_AT: usize = 0x40;
/// Sub-entry counts on disc are 2..=291; anything far beyond is malformed.
const MAX_ENTRIES: u32 = 4096;

pub const PSMCT32: u8 = 0x00;
pub const PSMT8: u8 = 0x13;
pub const PSMT4: u8 = 0x14;

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// Fields of a GS TEX0 register value (public GS documentation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tex0 {
    pub tbp: u32,
    pub tbw: u32,
    pub psm: u8,
    pub tw: u8,
    pub th: u8,
    pub tcc: bool,
    pub tfx: u8,
    pub cbp: u32,
    pub cpsm: u8,
    pub csm: u8,
}

impl Tex0 {
    pub fn decode(v: u64) -> Self {
        let bits = |lo: u32, n: u32| (v >> lo) & ((1u64 << n) - 1);
        Self {
            tbp: bits(0, 14) as u32,
            tbw: bits(14, 6) as u32,
            psm: bits(20, 6) as u8,
            tw: bits(26, 4) as u8,
            th: bits(30, 4) as u8,
            tcc: bits(34, 1) == 1,
            tfx: bits(35, 2) as u8,
            cbp: bits(37, 14) as u32,
            cpsm: bits(51, 4) as u8,
            csm: bits(55, 1) as u8,
        }
    }
}

/// The PSMCT32 rectangle uploaded from the chunk's pixel block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub tbp: u32,
    /// Destination buffer width in 64-pixel units: the segment width / 64 (SPEC-0003, Upload).
    pub bw: u32,
    /// Bits 14–19 of the segment's first word as stored. SPEC-0003 calls this width / 64, but
    /// on the disc it differs (e.g. 16 for a 256-wide segment); see request R-0008.
    pub stored_bw_field: u32,
    pub width: u32,
    pub height: u32,
}

impl Segment {
    fn parse(b: &[u8]) -> Result<Self, String> {
        let w0 = u32_at(b, 0).ok_or("Truncated upload segment")?;
        let w1 = u32_at(b, 4).ok_or("Truncated upload segment")?;
        if w1 >> 24 != PSMCT32 as u32 {
            return Err(format!("Upload segment format {:#x} is not PSMCT32", w1 >> 24));
        }
        let width = w1 & 0xfff;
        if width == 0 || width % 64 != 0 {
            return Err(format!("Upload segment width {width} is not a multiple of 64"));
        }
        Ok(Self {
            tbp: w0 & 0x3fff,
            bw: width / 64,
            stored_bw_field: (w0 >> 14) & 0x3f,
            width,
            height: (w1 >> 12) & 0xfff,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubTexture {
    pub tbp: u32,
    pub tbw: u32,
    /// Real size; the GS samples the power-of-two size in `tex0`.
    pub width: u32,
    pub height: u32,
    pub psm: u8,
    pub tex0: Tex0,
}

impl SubTexture {
    fn parse(b: &[u8]) -> Result<Self, String> {
        let bad = || "Truncated sub-texture entry".to_string();
        let size = u16_at(b, 4).ok_or_else(bad)? as u32;
        Ok(Self {
            tbp: (u16_at(b, 0).ok_or_else(bad)? & 0x3fff) as u32,
            tbw: u16_at(b, 2).ok_or_else(bad)? as u32,
            width: size & 0xfff,
            height: *b.get(6).ok_or_else(bad)? as u32 * 16 + (size >> 12),
            psm: *b.get(7).ok_or_else(bad)?,
            tex0: Tex0::decode(u64_at(b, 8).ok_or_else(bad)?),
        })
    }

    pub fn is_palette(&self) -> bool {
        self.psm == PSMCT32
    }

    /// Palette entries this indexed texture needs (256 for PSMT8, 16 for PSMT4).
    pub fn palette_len(&self) -> Option<usize> {
        match self.psm {
            PSMT8 => Some(256),
            PSMT4 => Some(16),
            _ => None,
        }
    }
}

/// A parsed texture chunk with its own copy of the pixel block.
#[derive(Clone, Debug)]
pub struct TextureChunk {
    pub segment: Segment,
    pub entries: Vec<SubTexture>,
    pixels: Vec<u8>,
}

/// Decoded straight RGBA8 pixels, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// GS alpha runs 0..=128 for 0..=1 (SPEC-0003: A8 = min(255, A × 2)).
pub fn alpha_to_8bit(a: u8) -> u8 {
    (a as u16 * 2).min(255) as u8
}

/// CSM1 storage for 256-entry palettes: index i reads entry i with bits 3 and 4 exchanged.
pub fn csm1(i: usize) -> usize {
    (i & !0x18) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)
}

impl TextureChunk {
    /// Parse a type-4 chunk payload (chunk start + 16 + extra header bytes).
    pub fn parse(payload: &[u8]) -> Result<Self, String> {
        let bad = || "Truncated texture payload header".to_string();
        let table_at = u32_at(payload, 0).ok_or_else(bad)? as usize;
        let pixels_at = u32_at(payload, 4).ok_or_else(bad)? as usize;
        let table = payload.get(table_at..).ok_or("Texture table outside the chunk")?;
        let segments = u32_at(table, 0).ok_or("Truncated texture table")?;
        if segments != 1 {
            return Err(format!("{segments} upload segments; only 1 is supported"));
        }
        let count = u32_at(table, 8).ok_or("Truncated texture table")?;
        if count > MAX_ENTRIES {
            return Err(format!("{count} sub-texture entries is implausible"));
        }
        let segment = Segment::parse(table.get(0x10..0x10 + SEGMENT_SIZE).ok_or_else(bad)?)?;
        let entries = (0..count as usize)
            .map(|i| {
                let at = ENTRIES_AT + i * ENTRY_SIZE;
                SubTexture::parse(table.get(at..at + ENTRY_SIZE).ok_or("Sub-texture entry outside the chunk")?)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let len = (segment.width * segment.height * 4) as usize;
        let pixels = payload
            .get(pixels_at..pixels_at + len)
            .ok_or("Pixel block runs past the chunk")?
            .to_vec();
        Ok(Self {
            segment,
            entries,
            pixels,
        })
    }

    /// A chunk with the given table and no pixels, for tests of table lookups.
    #[cfg(test)]
    pub(crate) fn with_entries(entries: Vec<SubTexture>) -> Self {
        Self {
            segment: Segment {
                tbp: 0,
                bw: 1,
                stored_bw_field: 1,
                width: 64,
                height: 0,
            },
            entries,
            pixels: Vec::new(),
        }
    }

    /// Emulated GS memory holding this chunk's upload, at the segment's own base so absolute
    /// (front-end) and chunk-relative addresses both resolve as stored.
    pub fn gs_memory(&self) -> GsMemory {
        let mut mem = GsMemory::default();
        let s = self.segment;
        mem.upload_ct32(s.tbp, s.bw, s.width, s.height, &self.pixels);
        mem
    }

    /// Indexes of indexed textures (not palettes), in table order.
    pub fn textures(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.is_palette())
            .map(|(i, _)| i)
    }

    /// Every palette entry with the right number of colours for this texture, in table order.
    pub fn palettes_for(&self, texture: usize) -> Vec<usize> {
        let Some(need) = self.entries.get(texture).and_then(|e| e.palette_len()) else {
            return Vec::new();
        };
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_palette() && (e.width * e.height) as usize == need)
            .map(|(i, _)| i)
            .collect()
    }

    /// The first palette entry with the right number of colours. The table does not pair
    /// textures with palettes (materials do), so this is a browsing aid, not the game's rule.
    pub fn first_palette_for(&self, texture: usize) -> Option<usize> {
        let need = self.entries.get(texture)?.palette_len()?;
        self.entries
            .iter()
            .position(|e| e.is_palette() && (e.width * e.height) as usize == need)
    }

    /// Palette colours in index order, RGBA8.
    pub fn palette(&self, mem: &GsMemory, entry: usize) -> Result<Vec<[u8; 4]>, String> {
        let e = self.entries.get(entry).ok_or("No such palette entry")?;
        if !e.is_palette() {
            return Err("Entry is not a palette".into());
        }
        let colors = (e.width * e.height) as usize;
        if colors != 256 && colors != 16 {
            return Err(format!("{}×{} is not a 16- or 256-colour palette", e.width, e.height));
        }
        let linear: Vec<[u8; 4]> = (0..e.height)
            .flat_map(|y| (0..e.width).map(move |x| (x, y)))
            .map(|(x, y)| {
                let [r, g, b, a] = mem.ct32(e.tbp, 1, x, y);
                [r, g, b, alpha_to_8bit(a)]
            })
            .collect();
        Ok(if colors == 256 {
            (0..256).map(|i| linear[csm1(i)]).collect()
        } else {
            linear
        })
    }

    /// Decode an indexed texture at its real size through the given palette entry.
    pub fn decode(&self, mem: &GsMemory, texture: usize, palette: usize) -> Result<Image, String> {
        let t = *self.entries.get(texture).ok_or("No such texture entry")?;
        self.decode_region(mem, texture, palette, t.width, t.height)
    }

    /// Decode a `width × height` region from the texture's origin. Materials sample the
    /// power-of-two size of their TEX0 (TW × TH), which may exceed the real size; UVs are in
    /// those units, so model textures decode at that size.
    pub fn decode_region(
        &self,
        mem: &GsMemory,
        texture: usize,
        palette: usize,
        width: u32,
        height: u32,
    ) -> Result<Image, String> {
        let t = *self.entries.get(texture).ok_or("No such texture entry")?;
        let need = t.palette_len().ok_or("Entry is not an indexed texture")?;
        let pal = self.palette(mem, palette)?;
        if pal.len() != need {
            return Err(format!("Texture needs {need} colours; palette has {}", pal.len()));
        }
        if width == 0 || height == 0 || width > 1024 || height > 1024 {
            return Err(format!("Texture size {width}×{height} is outside 1..=1024"));
        }
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let index = match t.psm {
                    PSMT8 => mem.t8(t.tbp, t.tbw, x, y),
                    _ => mem.t4(t.tbp, t.tbw, x, y),
                };
                rgba.extend_from_slice(&pal[index as usize]);
            }
        }
        Ok(Image {
            width,
            height,
            rgba,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_tex0_test_vector() {
        let t = Tex0::decode(0x0000_0705_5531_002C);
        assert_eq!((t.tbp, t.tbw, t.psm, t.tw, t.th), (0x2C, 4, PSMT8, 5, 5));
        assert!(t.tcc);
        assert_eq!((t.tfx, t.cbp, t.cpsm, t.csm), (0, 0x38, 0, 0));
    }

    #[test]
    fn spec_palette_and_alpha_vectors() {
        assert_eq!([8, 16, 7, 40, 255].map(csm1), [16, 8, 7, 48, 255]);
        assert_eq!([0x80, 0x40, 0x00].map(alpha_to_8bit), [255, 128, 0]);
    }

    #[test]
    fn parses_segment_words_and_entry_heights() {
        let seg = |w0: u32| {
            let mut b = [0u8; 8];
            b[..4].copy_from_slice(&w0.to_le_bytes());
            b[4..].copy_from_slice(&(256u32 | 64 << 12).to_le_bytes());
            Segment::parse(&b).unwrap()
        };
        assert_eq!((seg(0x0001_0000).tbp, seg(0x0001_0000).bw), (0, 4));
        assert_eq!((seg(0x0001_2A00).tbp, seg(0x0001_2A00).bw), (0x2A00, 4));
        // The buffer width comes from the segment width, whatever the stored field says.
        let odd = seg(0x0004_2A00);
        assert_eq!((odd.bw, odd.stored_bw_field), (4, 16));
        // SPEC-0003: TBP 0x3C, TBW 4, 32 × 17, PSMT8.
        let mut e = [0u8; ENTRY_SIZE];
        e[0..2].copy_from_slice(&0x3Cu16.to_le_bytes());
        e[2..4].copy_from_slice(&4u16.to_le_bytes());
        e[4..6].copy_from_slice(&(32u16 | 1 << 12).to_le_bytes());
        e[6] = 1;
        e[7] = PSMT8;
        let s = SubTexture::parse(&e).unwrap();
        assert_eq!((s.tbp, s.tbw, s.width, s.height, s.psm), (0x3C, 4, 32, 17, PSMT8));
        assert_eq!(s.palette_len(), Some(256));
    }

    /// Build a chunk payload whose pixel block, uploaded as PSMCT32, holds a PSMT8 and a PSMT4
    /// texture written through GS addressing, plus a 256- and a 16-colour palette.
    fn synthetic_chunk(seg_tbp: u32) -> Vec<u8> {
        // Four PSMCT32 pages (0x80 blocks) cover the textures and palettes placed below.
        let (sw, sh) = (64u32, 128u32);
        let mut mem = GsMemory::default();
        for y in 0..20 {
            for x in 0..24 {
                mem.set_t8(seg_tbp + 0x00, 2, x, y, ((x + y * 3) % 256) as u8);
                mem.set_t4(seg_tbp + 0x40, 2, x, y, ((x ^ y) % 16) as u8);
            }
        }
        // Linear palettes; the 256-colour one is stored in CSM1 order, so swap on the way in.
        for i in 0..256u32 {
            let colour = [csm1(i as usize) as u8, 0, 0, 0x80];
            mem.set_ct32(seg_tbp + 0x60, 1, i % 16, i / 16, colour);
        }
        for i in 0..16u32 {
            mem.set_ct32(seg_tbp + 0x64, 1, i % 8, i / 8, [0, i as u8 * 16, 0, 0x40]);
        }
        let mut pixels = Vec::new();
        for y in 0..sh {
            for x in 0..sw {
                pixels.extend_from_slice(&mem.ct32(seg_tbp, 1, x, y));
            }
        }
        let entry = |tbp: u32, tbw: u16, w: u16, h: u16, psm: u8| {
            let mut e = vec![0u8; ENTRY_SIZE];
            e[0..2].copy_from_slice(&(tbp as u16).to_le_bytes());
            e[2..4].copy_from_slice(&tbw.to_le_bytes());
            e[4..6].copy_from_slice(&(w | (h % 16) << 12).to_le_bytes());
            e[6] = (h / 16) as u8;
            e[7] = psm;
            e
        };
        let mut table = vec![0u8; ENTRIES_AT];
        table[0..4].copy_from_slice(&1u32.to_le_bytes());
        table[8..12].copy_from_slice(&4u32.to_le_bytes());
        table[0x10..0x14].copy_from_slice(&(seg_tbp | 1 << 14).to_le_bytes());
        table[0x14..0x18].copy_from_slice(&(sw | sh << 12).to_le_bytes());
        table.extend(entry(seg_tbp, 2, 24, 20, PSMT8));
        table.extend(entry(seg_tbp + 0x40, 2, 24, 20, PSMT4));
        table.extend(entry(seg_tbp + 0x60, 1, 16, 16, PSMCT32));
        table.extend(entry(seg_tbp + 0x64, 1, 8, 2, PSMCT32));
        let mut payload = vec![0u8; 16];
        payload[0..4].copy_from_slice(&16u32.to_le_bytes());
        let pixels_at = 16 + table.len() as u32;
        payload[4..8].copy_from_slice(&pixels_at.to_le_bytes());
        payload.extend(table);
        payload.extend(pixels);
        payload
    }

    #[test]
    fn decodes_synthetic_chunk_at_relative_and_absolute_bases() {
        for base in [0, 0x2A00] {
            let chunk = TextureChunk::parse(&synthetic_chunk(base)).unwrap();
            assert_eq!(chunk.textures().collect::<Vec<_>>(), [0, 1]);
            assert_eq!(chunk.first_palette_for(0), Some(2));
            assert_eq!(chunk.first_palette_for(1), Some(3));
            assert_eq!((chunk.palettes_for(0), chunk.palettes_for(2)), (vec![2], vec![]));
            let mem = chunk.gs_memory();
            let t8 = chunk.decode(&mem, 0, 2).unwrap();
            assert_eq!((t8.width, t8.height), (24, 20));
            for (x, y) in [(0, 0), (5, 3), (23, 19)] {
                let px = &t8.rgba[((y * 24 + x) * 4) as usize..][..4];
                assert_eq!(px, [((x + y * 3) % 256) as u8, 0, 0, 255], "t8 at {x},{y}");
            }
            let t4 = chunk.decode(&mem, 1, 3).unwrap();
            for (x, y) in [(0, 0), (7, 2), (23, 19)] {
                let px = &t4.rgba[((y * 24 + x) * 4) as usize..][..4];
                assert_eq!(px, [0, ((x ^ y) % 16) as u8 * 16, 0, 128], "t4 at {x},{y}");
            }
            assert!(chunk.decode(&mem, 0, 3).is_err(), "wrong palette size");
            assert!(chunk.decode(&mem, 2, 2).is_err(), "palette is not a texture");
        }
    }

    #[test]
    fn rejects_malformed_chunks() {
        let good = synthetic_chunk(0);
        assert!(TextureChunk::parse(&good[..20]).is_err());
        let mut two_segments = good.clone();
        two_segments[16] = 2;
        assert!(TextureChunk::parse(&two_segments).is_err());
        let mut truncated = good.clone();
        truncated.truncate(good.len() - 4);
        assert!(TextureChunk::parse(&truncated).is_err());
        let mut many = good.clone();
        many[24..28].copy_from_slice(&100_000u32.to_le_bytes());
        assert!(TextureChunk::parse(&many).is_err());
    }
}
