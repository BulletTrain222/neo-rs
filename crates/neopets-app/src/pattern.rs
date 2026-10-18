//! A synthetic indexed texture, generated in code and decoded through `neopets_data::gs`, to
//! exercise the decode -> GPU path before any real texture spec exists. Not game content.
use neopets_data::gs::{Clut, ClutFormat, PixelFormat, Texa, decode};

pub const SIZE: u32 = 64;

/// 64×64 PSMT4 pixels over a 16-entry 32-bit palette: a checkerboard of two 8-step ramps,
/// with palette entry 0 transparent so alpha handling shows on screen.
pub fn rgba() -> Result<Vec<u8>, String> {
    let palette: Vec<u8> = (0..16u8)
        .flat_map(|i| {
            let step = (i % 8) * 32;
            let alpha = if i == 0 { 0 } else { 0x80 };
            if i < 8 {
                [step, 96, 255 - step, alpha]
            } else {
                [255, step, 64, alpha]
            }
        })
        .collect();
    let index = |x: u32, y: u32| -> u8 {
        let checker = ((x / 8 + y / 8) % 2) as u8;
        checker * 8 + (x * 8 / SIZE) as u8
    };
    let pixels: Vec<u8> = (0..SIZE * SIZE / 2)
        .map(|n| {
            let (x, y) = ((n * 2) % SIZE, (n * 2) / SIZE);
            index(x, y) | index(x + 1, y) << 4
        })
        .collect();
    let clut = Clut {
        format: ClutFormat::Ct32,
        csm1: false,
        data: &palette,
    };
    decode(PixelFormat::T4, SIZE, SIZE, &pixels, Some(&clut), Texa::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_decodes_with_transparent_corner() {
        let rgba = rgba().unwrap();
        assert_eq!(rgba.len(), (SIZE * SIZE * 4) as usize);
        // Pixel (0, 0) uses palette entry 0; pixel (8, 0) is in the other checker colour.
        assert_eq!(rgba[3], 0);
        assert_eq!(&rgba[8 * 4..8 * 4 + 4], [255, 32, 64, 255]);
    }
}
