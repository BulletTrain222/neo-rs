//! PS2 Graphics Synthesizer pixel formats converted to straight RGBA8, from public hardware
//! documentation only. This is generic hardware behavior, not the game's texture format:
//! which formats the game uses, how its textures and palettes are laid out, and whether they
//! are stored swizzled in GS memory order all await the texture spec (R-0003). Input here is
//! assumed to be linear, row-major pixels.

/// The GS caps texture width and height at 1024 (2^10).
pub const MAX_DIMENSION: u32 = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// PSMCT32: R, G, B, A bytes; alpha 0x80 is fully opaque.
    Ct32,
    /// PSMCT16: little-endian 16-bit, 5 bits each of R, G, B (low to high), then 1 alpha bit.
    Ct16,
    /// PSMT8: one byte per pixel, an index into a 256-entry palette.
    T8,
    /// PSMT4: two pixels per byte, low nibble first, indexing a 16-entry palette.
    T4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClutFormat {
    Ct32,
    Ct16,
}

/// A palette (CLUT) as stored.
#[derive(Clone, Copy, Debug)]
pub struct Clut<'a> {
    pub format: ClutFormat,
    /// Storage mode CSM1 interleaves 256-entry palettes in groups of eight; CSM2 is linear.
    pub csm1: bool,
    pub data: &'a [u8],
}

/// The TEXA register's effect on 16-bit color: alpha for a clear or set A bit, and whether
/// pure black with a clear A bit becomes fully transparent (AEM).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Texa {
    pub ta0: u8,
    pub ta1: u8,
    pub aem: bool,
}

impl Default for Texa {
    fn default() -> Self {
        Self {
            ta0: 0,
            ta1: 0x80,
            aem: false,
        }
    }
}

/// GS alpha runs 0..=0x80 for 0..=1; larger values saturate.
pub fn alpha_to_rgba8(a: u8) -> u8 {
    ((a.min(0x80) as u16 * 255 + 64) / 128) as u8
}

/// CSM1 stores 256-entry palettes with entries 8..15 and 16..23 of every 32 swapped.
pub fn csm1_index(i: u8) -> u8 {
    (i & !0x18) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)
}

fn ct32(p: &[u8]) -> [u8; 4] {
    [p[0], p[1], p[2], alpha_to_rgba8(p[3])]
}

fn ct16(p: &[u8], texa: Texa) -> [u8; 4] {
    let v = u16::from_le_bytes([p[0], p[1]]);
    // The GS widens 5-bit channels by shifting, without replicating the high bits.
    let c = |shift: u16| (((v >> shift) & 0x1f) as u8) << 3;
    let rgb = v & 0x7fff;
    let a = if v & 0x8000 != 0 {
        texa.ta1
    } else if texa.aem && rgb == 0 {
        0
    } else {
        texa.ta0
    };
    [c(0), c(5), c(10), alpha_to_rgba8(a)]
}

fn palette(clut: &Clut, entries: usize, texa: Texa) -> Result<Vec<[u8; 4]>, String> {
    let size = match clut.format {
        ClutFormat::Ct32 => 4,
        ClutFormat::Ct16 => 2,
    };
    let bytes = clut
        .data
        .get(..entries * size)
        .ok_or_else(|| format!("Palette needs {entries} entries of {size} bytes"))?;
    let colors: Vec<[u8; 4]> = bytes
        .chunks_exact(size)
        .map(|p| match clut.format {
            ClutFormat::Ct32 => ct32(p),
            ClutFormat::Ct16 => ct16(p, texa),
        })
        .collect();
    Ok(if clut.csm1 && entries == 256 {
        (0..=255u8).map(|i| colors[csm1_index(i) as usize]).collect()
    } else {
        colors
    })
}

/// Decode `width × height` linear pixels to straight RGBA8, four bytes per pixel.
pub fn decode(
    format: PixelFormat,
    width: u32,
    height: u32,
    pixels: &[u8],
    clut: Option<&Clut>,
    texa: Texa,
) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(format!("Texture size {width}×{height} is outside 1..=1024"));
    }
    let count = (width * height) as usize;
    let needed = match format {
        PixelFormat::Ct32 => count * 4,
        PixelFormat::Ct16 => count * 2,
        PixelFormat::T8 => count,
        PixelFormat::T4 => count.div_ceil(2),
    };
    let pixels = pixels
        .get(..needed)
        .ok_or_else(|| format!("{format:?} {width}×{height} needs {needed} bytes"))?;
    let indexed = |entries: usize| {
        let clut = clut.ok_or_else(|| format!("{format:?} needs a palette"))?;
        palette(clut, entries, texa)
    };
    let out: Vec<[u8; 4]> = match format {
        PixelFormat::Ct32 => pixels.chunks_exact(4).map(ct32).collect(),
        PixelFormat::Ct16 => pixels.chunks_exact(2).map(|p| ct16(p, texa)).collect(),
        PixelFormat::T8 => {
            let pal = indexed(256)?;
            pixels.iter().map(|&i| pal[i as usize]).collect()
        }
        PixelFormat::T4 => {
            let pal = indexed(16)?;
            pixels
                .iter()
                .flat_map(|&b| [b & 0x0f, b >> 4])
                .take(count)
                .map(|i| pal[i as usize])
                .collect()
        }
    };
    Ok(out.into_flattened())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_scale_and_saturation() {
        assert_eq!(alpha_to_rgba8(0), 0);
        assert_eq!(alpha_to_rgba8(0x40), 128);
        assert_eq!(alpha_to_rgba8(0x80), 255);
        assert_eq!(alpha_to_rgba8(0xff), 255);
    }

    #[test]
    fn csm1_swaps_middle_groups_of_eight() {
        let map: Vec<u8> = (0..32).map(csm1_index).collect();
        let expected: Vec<u8> = (0..8).chain(16..24).chain(8..16).chain(24..32).collect();
        assert_eq!(map, expected);
        assert_eq!(csm1_index(0x28), 0x30);
        assert!((0..=255u8).all(|i| csm1_index(csm1_index(i)) == i));
    }

    #[test]
    fn decodes_direct_color() {
        let rgba = decode(PixelFormat::Ct32, 1, 1, &[1, 2, 3, 0x80], None, Texa::default());
        assert_eq!(rgba.unwrap(), [1, 2, 3, 255]);
        // R=31, G=0, B=1 with A set, then pure black with A clear.
        let px = [(31 | 1 << 10 | 1 << 15) as u16, 0u16];
        let bytes: Vec<u8> = px.iter().flat_map(|p| p.to_le_bytes()).collect();
        let plain = decode(PixelFormat::Ct16, 2, 1, &bytes, None, Texa::default()).unwrap();
        assert_eq!(plain, [248, 0, 8, 255, 0, 0, 0, 0]);
        let texa = Texa {
            ta0: 0x40,
            ta1: 0x80,
            aem: true,
        };
        let aem = decode(PixelFormat::Ct16, 2, 1, &bytes, None, texa).unwrap();
        assert_eq!(&aem[4..], [0, 0, 0, 0]);
        let no_aem = decode(PixelFormat::Ct16, 2, 1, &bytes, None, Texa { aem: false, ..texa });
        assert_eq!(&no_aem.unwrap()[4..], [0, 0, 0, 128]);
    }

    #[test]
    fn decodes_indexed_with_linear_and_csm1_palettes() {
        let pal: Vec<u8> = (0..=255u8).flat_map(|i| [i, 0, 0, 0x80]).collect();
        let linear = Clut {
            format: ClutFormat::Ct32,
            csm1: false,
            data: &pal,
        };
        let csm1 = Clut { csm1: true, ..linear };
        let t8 = |clut| decode(PixelFormat::T8, 2, 1, &[9, 17], Some(clut), Texa::default());
        assert_eq!(t8(&linear).unwrap(), [9, 0, 0, 255, 17, 0, 0, 255]);
        assert_eq!(t8(&csm1).unwrap(), [17, 0, 0, 255, 9, 0, 0, 255]);
        // T4: low nibble first; an odd pixel count leaves the last high nibble unused.
        let t4 = decode(PixelFormat::T4, 3, 1, &[0x21, 0xf3], Some(&csm1), Texa::default());
        let reds: Vec<u8> = t4.unwrap().chunks(4).map(|p| p[0]).collect();
        assert_eq!(reds, [1, 2, 3], "16-entry palettes are never CSM1-swizzled");
    }

    #[test]
    fn rejects_bad_sizes_and_missing_data() {
        let t = Texa::default();
        assert!(decode(PixelFormat::Ct32, 0, 1, &[], None, t).is_err());
        assert!(decode(PixelFormat::Ct32, 1025, 1, &[0; 4100], None, t).is_err());
        assert!(decode(PixelFormat::Ct32, 2, 1, &[0; 7], None, t).is_err());
        assert!(decode(PixelFormat::T8, 1, 1, &[0], None, t).is_err());
        let short = Clut {
            format: ClutFormat::Ct16,
            csm1: false,
            data: &[0; 30],
        };
        assert!(decode(PixelFormat::T4, 1, 1, &[0], Some(&short), t).is_err());
        assert!(decode(PixelFormat::T4, 1, 1, &[0], Some(&Clut { data: &[0; 32], ..short }), t).is_ok());
    }
}
