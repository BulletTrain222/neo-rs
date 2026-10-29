//! The player's choice of button pictures (Display page, "Button pictures"; this build's option,
//! not in the original): the disc's PlayStation icons, or Xbox-style A, B, X and Y drawn here
//! in their place. The pictures are this build's own drawing (a dark disc, a coloured rim and
//! a coloured letter), not art from any disc.
use bevy::prelude::*;
use neopets_data::texture::Image as Rgba;
use std::collections::HashMap;

/// The front end's sheet assets of the four face-button icons (found by their pictures: the
/// dialogue's Square, Triangle and Circle are logical sprites 0x1B, 0x78 and 0x79, SPEC-0022),
/// with the Xbox button at the same place on the pad and its colour.
const FACE: [(u32, char, [u8; 3]); 4] = [
    (30, 'A', [96, 190, 70]),  // Cross: bottom
    (32, 'B', [225, 60, 55]),  // Circle: right
    (36, 'X', [60, 130, 230]), // Square: left
    (33, 'Y', [240, 195, 40]), // Triangle: top
];

/// The icon size drawn (the sheet's icons are 32 x 32; larger requests scale it).
const SIZE: u32 = 64;

/// The strokes of a letter as polylines in a unit box (x right, y down).
fn strokes(c: char) -> Vec<Vec<[f32; 2]>> {
    match c {
        'A' => vec![vec![[0.0, 1.0], [0.5, 0.0], [1.0, 1.0]], vec![[0.22, 0.62], [0.78, 0.62]]],
        'B' => vec![
            vec![[0.08, 0.0], [0.08, 1.0]],
            vec![[0.08, 0.0], [0.58, 0.0], [0.8, 0.1], [0.84, 0.26], [0.78, 0.4], [0.58, 0.48], [0.08, 0.48]],
            vec![[0.58, 0.48], [0.84, 0.58], [0.9, 0.74], [0.84, 0.9], [0.6, 1.0], [0.08, 1.0]],
        ],
        'X' => vec![vec![[0.0, 0.0], [1.0, 1.0]], vec![[1.0, 0.0], [0.0, 1.0]]],
        _ => vec![vec![[0.0, 0.0], [0.5, 0.5], [1.0, 0.0]], vec![[0.5, 0.5], [0.5, 1.0]]],
    }
}

fn segment_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ab, ap) = ([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]);
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-6)).clamp(0.0, 1.0);
    let (dx, dy) = (ap[0] - ab[0] * t, ap[1] - ab[1] * t);
    (dx * dx + dy * dy).sqrt()
}

/// One button icon: a dark disc with a coloured rim and the letter, anti-aliased by 4 x 4
/// samples a pixel.
pub fn icon(letter: char, colour: [u8; 3]) -> Rgba {
    let n = SIZE as f32;
    let (centre, radius, rim) = (n / 2.0, n / 2.0 - 1.0, n * 0.07);
    // The letter box: 40 percent of the icon wide, 50 percent high, centred.
    let (lw, lh) = (n * 0.4, n * 0.5);
    let (lx, ly) = (centre - lw / 2.0, centre - lh / 2.0);
    let pen = n * 0.085;
    let lines = strokes(letter);
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (mut disc, mut ring, mut ink) = (0.0f32, 0.0f32, 0.0f32);
            for sy in 0..4 {
                for sx in 0..4 {
                    let p = [x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0];
                    let d = ((p[0] - centre).powi(2) + (p[1] - centre).powi(2)).sqrt();
                    if d <= radius {
                        disc += 1.0;
                        if d >= radius - rim {
                            ring += 1.0;
                        }
                        let q = [(p[0] - lx) / lw, (p[1] - ly) / lh];
                        let near = lines.iter().flat_map(|l| l.windows(2).map(move |w| (w[0], w[1]))).map(|(a, b)| segment_distance([q[0] * lw, q[1] * lh], [a[0] * lw, a[1] * lh], [b[0] * lw, b[1] * lh])).fold(f32::MAX, f32::min);
                        if near <= pen / 2.0 {
                            ink += 1.0;
                        }
                    }
                }
            }
            let (disc, ring, ink) = (disc / 16.0, ring / 16.0, ink / 16.0);
            // A dark disc, a little lighter at the top; the rim and the letter in the colour.
            let shade = 34.0 + 22.0 * (1.0 - y as f32 / n);
            let paint = (ring + ink).min(1.0);
            let mix = |c: u8| ((shade * (1.0 - paint) + c as f32 * paint) / disc.max(1e-6) * disc).round().clamp(0.0, 255.0) as u8;
            rgba.extend_from_slice(&[mix(colour[0]), mix(colour[1]), mix(colour[2]), (disc * 255.0).round() as u8]);
        }
    }
    Rgba { width: SIZE, height: SIZE, rgba }
}

/// The overrides for a choice: none for the PlayStation pictures, the four Xbox icons else.
pub fn overrides(xbox: bool) -> HashMap<u32, Rgba> {
    if !xbox {
        return HashMap::new();
    }
    FACE.iter().map(|&(asset, letter, colour)| (asset, icon(letter, colour))).collect()
}

/// When the choice changes: the HUD art's overrides are replaced and its picture caches are
/// marked stale (the HUD, the dialogue box and the menus rebuild their icons).
pub fn apply(settings: Res<crate::display::DisplaySettings>, art: Option<ResMut<crate::hud_view::HudArt>>, mut last: Local<Option<bool>>) {
    let _span = crate::perf::span("apply");
    let Some(mut art) = art else { return };
    if *last == Some(settings.xbox_buttons) {
        return;
    }
    *last = Some(settings.xbox_buttons);
    art.set_overrides(overrides(settings.xbox_buttons));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_are_round_and_lettered() {
        let i = icon('A', [96, 190, 70]);
        let at = |x: u32, y: u32| &i.rgba[((y * SIZE + x) * 4) as usize..((y * SIZE + x) * 4 + 4) as usize];
        // The corners are clear, the middle of the disc is opaque.
        assert_eq!(at(0, 0)[3], 0);
        assert_eq!(at(SIZE / 2, SIZE / 2 + 12)[3], 255);
        // The A's apex is green.
        let apex = at(SIZE / 2, SIZE / 4 + 2);
        assert!(apex[1] > apex[0] && apex[1] > 120, "{apex:?}");
        assert!(overrides(false).is_empty() && overrides(true).len() == 4);
    }
}
