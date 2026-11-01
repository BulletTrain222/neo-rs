//! The player's choice of button pictures (Display page, "Button pictures"; this build's option,
//! not in the original): the disc's PlayStation icons, or Xbox-style A, B, X and Y drawn here
//! in their place. The pictures are this build's own drawing (a dark glossy disc and a coloured
//! letter, in the PlayStation icons' style), not art from any disc. Each is fitted to the circle
//! of the PlayStation icon it replaces, measured from that icon's own pixels, so both choices
//! draw at the same size.
use bevy::prelude::*;
use neopets_data::texture::Image as Rgba;
use std::collections::HashMap;

/// The front end's sheet assets of the four face-button icons (found by their pictures: the
/// dialogue's Square, Triangle and Circle are logical sprites 0x1B, 0x78 and 0x79, SPEC-0022),
/// with the Xbox button at the same place on the pad and its colour.
const FACE: [(u32, char, [u8; 3]); 4] = [
    (30, 'A', [110, 200, 70]), // Cross: bottom
    (32, 'B', [235, 70, 60]),  // Circle: right
    (36, 'X', [70, 145, 240]), // Square: left
    (33, 'Y', [250, 200, 50]), // Triangle: top
];

/// The Cross (Xbox A) icon's sheet asset.
pub const CROSS_ASSET: u32 = FACE[0].0;

/// The icon size drawn (the sheet's icons are 32 x 32; larger requests scale it).
const SIZE: u32 = 64;

/// Where the disc sits in the icon: centre and radius as fractions of the icon's size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
}

/// Used when a PlayStation icon cannot be measured: the circle the disc's icons fill (about 72
/// percent of their width, measured on screen).
const DEFAULT_FIT: Fit = Fit { cx: 0.5, cy: 0.5, r: 0.36 };

/// The circle of a PlayStation icon: the box of its texels with alpha above a quarter, as a
/// centre and a radius (half the box's larger side).
pub fn measure(icon: &Rgba) -> Option<Fit> {
    let (w, h) = (icon.width, icon.height);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if icon.rgba.get(((y * w + x) * 4 + 3) as usize).is_some_and(|&a| a > 64) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    if x0 > x1 || w == 0 || h == 0 {
        return None;
    }
    let (bw, bh) = ((x1 - x0 + 1) as f32 / w as f32, (y1 - y0 + 1) as f32 / h as f32);
    Some(Fit { cx: (x0 + x1 + 1) as f32 / 2.0 / w as f32, cy: (y0 + y1 + 1) as f32 / 2.0 / h as f32, r: bw.max(bh) / 2.0 })
}

/// The strokes of a letter as polylines in a unit box (x right, y down).
fn strokes(c: char) -> Vec<Vec<[f32; 2]>> {
    match c {
        'A' => vec![vec![[0.04, 1.0], [0.5, 0.0], [0.96, 1.0]], vec![[0.25, 0.64], [0.75, 0.64]]],
        'B' => vec![
            vec![[0.12, 0.0], [0.12, 1.0]],
            vec![[0.12, 0.0], [0.56, 0.0], [0.76, 0.07], [0.83, 0.24], [0.76, 0.41], [0.56, 0.48], [0.12, 0.48]],
            vec![[0.56, 0.48], [0.8, 0.55], [0.89, 0.74], [0.8, 0.93], [0.58, 1.0], [0.12, 1.0]],
        ],
        'X' => vec![vec![[0.06, 0.0], [0.94, 1.0]], vec![[0.94, 0.0], [0.06, 1.0]]],
        _ => vec![vec![[0.04, 0.0], [0.5, 0.52], [0.96, 0.0]], vec![[0.5, 0.52], [0.5, 1.0]]],
    }
}

fn segment_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ab, ap) = ([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]);
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-6)).clamp(0.0, 1.0);
    let (dx, dy) = (ap[0] - ab[0] * t, ap[1] - ab[1] * t);
    (dx * dx + dy * dy).sqrt()
}

/// One button icon in the circle `fit`: a dark disc, lighter at the top with a soft gloss and a
/// thin darker outline, and the letter in the button's colour (lighter at its top, with a dark
/// edge), anti-aliased by 4 x 4 samples a pixel.
pub fn icon(letter: char, colour: [u8; 3], fit: Fit) -> Rgba {
    let n = SIZE as f32;
    let (cx, cy, radius) = (fit.cx * n, fit.cy * n, fit.r * n);
    // The letter box: 52 percent of the disc's diameter wide, 60 percent high, centred.
    let (lw, lh) = (radius * 2.0 * 0.5, radius * 2.0 * 0.58);
    let (lx, ly) = (cx - lw / 2.0, cy - lh / 2.0);
    let pen = radius * 0.24;
    let edge = radius * 0.07;
    let lines = strokes(letter);
    let near = |p: [f32; 2]| {
        let q = [p[0] - lx, p[1] - ly];
        lines.iter().flat_map(|l| l.windows(2).map(|w| (w[0], w[1]))).map(|(a, b)| segment_distance(q, [a[0] * lw, a[1] * lh], [b[0] * lw, b[1] * lh])).fold(f32::MAX, f32::min)
    };
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (mut cover, mut acc) = (0.0f32, [0.0f32; 3]);
            for sy in 0..4 {
                for sx in 0..4 {
                    let p = [x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0];
                    let d = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
                    if d > radius {
                        continue;
                    }
                    cover += 1.0;
                    // The disc: charcoal, lighter toward the top; a gloss band over its upper half;
                    // a thin dark outline.
                    let t = ((p[1] - (cy - radius)) / (2.0 * radius)).clamp(0.0, 1.0);
                    let mut c = [52.0 - 26.0 * t, 54.0 - 26.0 * t, 60.0 - 28.0 * t];
                    let g = ((p[0] - cx) / (radius * 0.8)).powi(2) + ((p[1] - (cy - radius * 0.45)) / (radius * 0.5)).powi(2);
                    if g < 1.0 {
                        let k = 34.0 * (1.0 - g);
                        c = [c[0] + k, c[1] + k, c[2] + k];
                    }
                    if d > radius - edge {
                        c = [14.0, 14.0, 16.0];
                    }
                    // The letter: a dark edge, then the colour, lighter toward its top.
                    let l = near(p);
                    if l <= pen / 2.0 + edge * 0.8 {
                        c = [12.0, 12.0, 14.0];
                    }
                    if l <= pen / 2.0 {
                        let lt = ((p[1] - ly) / lh).clamp(0.0, 1.0);
                        let lift = 0.35 * (1.0 - lt);
                        c = std::array::from_fn(|i| colour[i] as f32 + (255.0 - colour[i] as f32) * lift);
                    }
                    for i in 0..3 {
                        acc[i] += c[i];
                    }
                }
            }
            let alpha = cover / 16.0;
            let px: [u8; 3] = std::array::from_fn(|i| if cover > 0.0 { (acc[i] / cover).round().clamp(0.0, 255.0) as u8 } else { 0 });
            rgba.extend_from_slice(&[px[0], px[1], px[2], (alpha * 255.0).round() as u8]);
        }
    }
    Rgba { width: SIZE, height: SIZE, rgba }
}

/// The overrides for a choice: none for the PlayStation pictures, the four Xbox icons else, each
/// in the circle `fit_of` gives for its sheet asset.
pub fn overrides(xbox: bool, fit_of: impl Fn(u32) -> Option<Fit>) -> HashMap<u32, Rgba> {
    if !xbox {
        return HashMap::new();
    }
    FACE.iter().map(|&(asset, letter, colour)| (asset, icon(letter, colour, fit_of(asset).unwrap_or(DEFAULT_FIT)))).collect()
}

/// When the choice changes: the HUD art's overrides are replaced and its picture caches are
/// marked stale (the HUD, the dialogue box and the menus rebuild their icons). The PlayStation
/// icons are measured first, with no overrides in place.
pub fn apply(settings: Res<crate::display::DisplaySettings>, art: Option<ResMut<crate::hud_view::HudArt>>, mut last: Local<Option<bool>>) {
    let _span = crate::perf::span("apply");
    let Some(mut art) = art else { return };
    if *last == Some(settings.xbox_buttons) {
        return;
    }
    *last = Some(settings.xbox_buttons);
    art.set_overrides(HashMap::new());
    let fits: HashMap<u32, Fit> = {
        let parts = art.parts();
        let f = parts.f;
        FACE.iter()
            .filter_map(|&(asset, ..)| {
                let node = (0..f.nodes.len()).find(|&n| f.sprite(n).is_some_and(|s| s.asset == asset))?;
                let img = f.sprite_image(parts.mem, node, 32, 32)?;
                Some((asset, measure(&img)?))
            })
            .collect()
    };
    art.set_overrides(overrides(settings.xbox_buttons, |a| fits.get(&a).copied()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_fit_the_circle_and_are_lettered() {
        let fit = Fit { cx: 0.5, cy: 0.5, r: 0.36 };
        let i = icon('A', [110, 200, 70], fit);
        let at = |x: u32, y: u32| &i.rgba[((y * SIZE + x) * 4) as usize..((y * SIZE + x) * 4 + 4) as usize];
        // Outside the circle is clear (radius 0.36 of 64 = 23); the middle is opaque.
        assert_eq!(at(4, 32)[3], 0);
        assert_eq!(at(32, 32 + 10)[3], 255);
        // The A's apex is green.
        let apex = at(32, 32 - 11);
        assert!(apex[1] > apex[0] && apex[1] > 120, "{apex:?}");
        assert!(overrides(false, |_| None).is_empty() && overrides(true, |_| None).len() == 4);
    }

    #[test]
    fn measure_finds_the_circle() {
        // A 32 x 32 icon with an opaque 20 x 20 square at (6, 8).
        let mut rgba = vec![0u8; 32 * 32 * 4];
        for y in 8..28 {
            for x in 6..26 {
                rgba[(y * 32 + x) * 4 + 3] = 255;
            }
        }
        let f = measure(&Rgba { width: 32, height: 32, rgba }).unwrap();
        assert_eq!((f.cx, f.cy, f.r), (0.5, 0.5625, 0.3125));
    }
}
