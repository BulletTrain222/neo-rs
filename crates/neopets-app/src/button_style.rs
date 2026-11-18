//! The player's choice of button pictures (Display page, "Button pictures"; this build's option,
//! not in the original): the disc's PlayStation icons, Xbox-style A, B, X and Y drawn here in
//! their place, keyboard keys (a key cap with the bound key's name in the game's own prompt
//! font), or Auto (the Xbox icons after a gamepad press, the keys after a key or a click). The pictures are this build's own drawing (a dark glossy disc and a coloured
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

/// Which face button a sheet asset pictures: 0 Cross, 1 Circle, 2 Square, 3 Triangle (the menus'
/// mouse support finds the button prompts by their icons).
pub fn face_index(asset: u32) -> Option<usize> {
    FACE.iter().position(|f| f.0 == asset)
}

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

/// The bindings' actions of the four face icons, in `FACE` order.
const FACE_ACTIONS: [usize; 4] = [crate::bindings::CROSS, crate::bindings::CIRCLE, crate::bindings::SQUARE, crate::bindings::TRIANGLE];

/// A key's name short enough for a key cap.
fn short(name: &str) -> String {
    match name {
        "Backspace" => "Bksp".into(),
        "L Shift" | "R Shift" => "Shift".into(),
        "L Ctrl" | "R Ctrl" => "Ctrl".into(),
        "L Alt" | "R Alt" => "Alt".into(),
        "Caps Lock" => "Caps".into(),
        "Num Enter" => "Enter".into(),
        "Left Mouse" => "LMB".into(),
        "Right Mouse" => "RMB".into(),
        "Middle Mouse" => "MMB".into(),
        n => n.strip_prefix("Num ").map_or_else(|| n.to_string(), |d| d.to_string()),
    }
}

/// The key a face icon names: the first key bound to its action (a mouse button only when no key
/// is). On a menu page Triangle is Esc (it goes back there; menu_view.rs `menu_mouse`) and Cross
/// is Enter when Enter is bound to it.
pub fn key_label(bindings: &crate::bindings::Bindings, action: usize, menu: bool) -> String {
    use crate::bindings::Input;
    let bound: Vec<Input> = bindings.keys.get(action).map(|k| k.iter().flatten().copied().collect()).unwrap_or_default();
    if menu && action == crate::bindings::TRIANGLE {
        return "Esc".into();
    }
    if menu && action == crate::bindings::CROSS && bound.contains(&Input::Key(KeyCode::Enter)) {
        return "Enter".into();
    }
    bound.iter().find(|i| matches!(i, Input::Key(_))).or(bound.first()).map_or_else(|| "-".into(), |i| short(&i.name()))
}

/// A rounded rectangle's signed distance (negative inside): centre c, half sizes h, corner
/// radius r.
fn rounded(p: [f32; 2], c: [f32; 2], h: [f32; 2], r: f32) -> f32 {
    let (qx, qy) = ((p[0] - c[0]).abs() - (h[0] - r), (p[1] - c[1]).abs() - (h[1] - r));
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r
}

/// One keyboard key icon in the circle `fit` (the square around it): a dark key cap like the Xbox
/// discs (charcoal, lighter toward the top, a raised top face with a soft gloss, a thin dark
/// outline) with `label` (the key's name drawn in the game's font) filling its face, brightened;
/// a long name ("Enter", "Space") widens the cap up to the icon's width. Anti-aliased by 4 x 4
/// samples a pixel.
pub fn key_icon(label: Option<&Rgba>, fit: Fit) -> Rgba {
    let n = SIZE as f32;
    let (c, half) = ([fit.cx * n, fit.cy * n], (fit.r * n).min(n / 2.0 - 0.5));
    // The label's height on the face, and the cap's half width that holds it.
    let text_h = half * 1.2;
    let text_w = label.filter(|l| l.height > 0).map_or(0.0, |l| l.width as f32 * text_h / l.height as f32);
    let half_w = (text_w / 2.0 / 0.8).clamp(half, n / 2.0 - 0.5);
    let hs = [half_w, half];
    let (corner, edge) = (half * 0.3, half * 0.07);
    // The top face: inset, raised a little toward the top.
    let (face_c, face_hs) = ([c[0], c[1] - half * 0.08], [half_w - half * 0.14, half * 0.8]);
    let face_h = face_hs[1];
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (mut cover, mut acc) = (0.0f32, [0.0f32; 3]);
            for sy in 0..4 {
                for sx in 0..4 {
                    let p = [x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0];
                    let d = rounded(p, c, hs, corner);
                    if d > 0.0 {
                        continue;
                    }
                    cover += 1.0;
                    let t = ((p[1] - (c[1] - half)) / (2.0 * half)).clamp(0.0, 1.0);
                    // The cap's sides, darker; the top face lighter, with a gloss over its upper half.
                    let mut col = [40.0 - 20.0 * t, 42.0 - 20.0 * t, 47.0 - 22.0 * t];
                    if rounded(p, face_c, face_hs, corner * 0.8) <= 0.0 {
                        let ft = ((p[1] - (face_c[1] - face_h)) / (2.0 * face_h)).clamp(0.0, 1.0);
                        col = [64.0 - 22.0 * ft, 66.0 - 22.0 * ft, 73.0 - 24.0 * ft];
                        let g = ((p[0] - face_c[0]) / (face_hs[0] * 0.95)).powi(2) + ((p[1] - (face_c[1] - face_h * 0.55)) / (face_h * 0.45)).powi(2);
                        if g < 1.0 {
                            let k = 26.0 * (1.0 - g);
                            col = [col[0] + k, col[1] + k, col[2] + k];
                        }
                    }
                    if d > -edge {
                        col = [14.0, 14.0, 16.0];
                    }
                    for i in 0..3 {
                        acc[i] += col[i];
                    }
                }
            }
            let px: [u8; 3] = std::array::from_fn(|i| if cover > 0.0 { (acc[i] / cover).round().clamp(0.0, 255.0) as u8 } else { 0 });
            rgba.extend_from_slice(&[px[0], px[1], px[2], (cover / 16.0 * 255.0).round() as u8]);
        }
    }
    // The label, scaled to fill the top face, drawn over it by its alpha with each output pixel
    // the mean of the label texels it covers, its colours lifted toward white.
    if let Some(l) = label.filter(|l| l.width > 0 && l.height > 0) {
        let fw = face_hs[0] * 2.0 * 0.96;
        let k = (fw / l.width as f32).min(text_h / l.height as f32);
        let (w, h) = (l.width as f32 * k, l.height as f32 * k);
        let (x0, y0) = (face_c[0] - w / 2.0, face_c[1] - h / 2.0);
        for y in (y0.floor().max(0.0) as u32)..((y0 + h).ceil().min(n) as u32) {
            for x in (x0.floor().max(0.0) as u32)..((x0 + w).ceil().min(n) as u32) {
                let mut acc = [0.0f32; 4];
                let mut count = 0.0;
                for sy in 0..4 {
                    for sx in 0..4 {
                        let (px, py) = (x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0);
                        let (u, v) = ((px - x0) / k, (py - y0) / k);
                        count += 1.0;
                        if u < 0.0 || v < 0.0 || u >= l.width as f32 || v >= l.height as f32 {
                            continue;
                        }
                        let i = ((v as u32 * l.width + u as u32) * 4) as usize;
                        let a = l.rgba[i + 3] as f32 / 255.0;
                        for ch in 0..3 {
                            acc[ch] += l.rgba[i + ch] as f32 * a;
                        }
                        acc[3] += a;
                    }
                }
                let a = acc[3] / count;
                if a <= 0.0 {
                    continue;
                }
                let o = ((y * SIZE + x) * 4) as usize;
                for ch in 0..3 {
                    let src = (acc[ch] / acc[3] * 1.5).min(255.0);
                    rgba[o + ch] = (rgba[o + ch] as f32 * (1.0 - a) + src * a).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    Rgba { width: SIZE, height: SIZE, rgba }
}

/// The last input device used, for Auto: a gamepad button or stick, or a key or mouse button.
#[derive(Resource, Default)]
pub struct LastDevice {
    pub pad: bool,
}

/// Keep `LastDevice`: a gamepad button press or a stick pushed past half marks the pad; a key or
/// mouse button press marks the keyboard (mouse motion alone does not).
pub fn track_device(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, pads: Query<&Gamepad>, mut last: ResMut<LastDevice>) {
    let pad = pads.iter().any(|g| g.get_just_pressed().next().is_some() || g.left_stick().length() > 0.5 || g.right_stick().length() > 0.5);
    let board = keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some();
    if pad && !last.pad {
        last.pad = true;
    } else if board && !pad && last.pad {
        last.pad = false;
    }
}

/// When the pictures shown change (the choice; for Auto the last device; for the keys their
/// bindings and whether a menu page is up): the HUD art's overrides are replaced and its picture
/// caches are marked stale (the HUD, the dialogue box and the menus rebuild their icons). The
/// PlayStation icons are measured first, with no overrides in place.
pub fn apply(
    settings: Res<crate::display::DisplaySettings>,
    art: Option<ResMut<crate::hud_view::HudArt>>,
    bindings: Res<crate::bindings::Bindings>,
    device: Res<LastDevice>,
    menu: Option<Res<crate::menu_view::Menu>>,
    mut last: Local<Option<(usize, Vec<String>)>>,
) {
    let _span = crate::perf::span("apply");
    let Some(mut art) = art else { return };
    // 0 PlayStation, 1 Xbox, 2 keys.
    let shown = match settings.buttons {
        3 if device.pad => 1,
        3 => 2,
        b => b.min(2),
    };
    let on_menu = menu.as_ref().is_some_and(|m| !matches!(m.flow.current, crate::menus::PLAY | crate::menus::BOX_HOST));
    let labels: Vec<String> = if shown == 2 { FACE_ACTIONS.iter().map(|&a| key_label(&bindings, a, on_menu)).collect() } else { Vec::new() };
    let want = (shown, labels);
    if last.as_ref() == Some(&want) {
        return;
    }
    *last = Some(want.clone());
    let (shown, labels) = want;
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
    if shown != 2 {
        art.set_overrides(overrides(shown == 1, |a| fits.get(&a).copied()));
        return;
    }
    // The keys: each label in the game's dialogue font (font 0, bolder than the prompts' font 1).
    let keys: HashMap<u32, Rgba> = {
        let parts = art.parts();
        FACE.iter()
            .zip(&labels)
            .map(|(&(asset, ..), label)| {
                let text = parts.f.text_image(parts.mem, 0, label.as_bytes(), 0, [128, 128, 128, 128]).map(|(img, _)| img);
                (asset, key_icon(text.as_ref(), fits.get(&asset).copied().unwrap_or(DEFAULT_FIT)))
            })
            .collect()
    };
    art.set_overrides(keys);
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
    fn keys_are_capped_and_labelled() {
        use crate::bindings::{Bindings, CIRCLE, CROSS, SQUARE, TRIANGLE};
        let b = Bindings::default();
        // Play: the first key bound; a menu page: Esc for Triangle, Enter for Cross.
        let play: Vec<String> = [CROSS, CIRCLE, SQUARE, TRIANGLE].iter().map(|&a| key_label(&b, a, false)).collect();
        assert_eq!(play, ["Space", "F", "E", "T"]);
        assert_eq!((key_label(&b, CROSS, true), key_label(&b, TRIANGLE, true)), ("Enter".to_string(), "Esc".to_string()));
        assert_eq!((short("Backspace"), short("Num 8"), short("Left Mouse")), ("Bksp".to_string(), "8".to_string(), "LMB".to_string()));
        // A 4 x 2 white label: the cap is opaque in the middle and clear in its corners; the
        // label's white is at the face's centre.
        let label = Rgba { width: 4, height: 2, rgba: [255u8; 32].to_vec() };
        let i = key_icon(Some(&label), Fit { cx: 0.5, cy: 0.5, r: 0.36 });
        let at = |x: u32, y: u32| i.rgba[((y * SIZE + x) * 4) as usize..((y * SIZE + x) * 4 + 4) as usize].to_vec();
        assert_eq!(at(2, 2)[3], 0);
        assert_eq!(at(32, 50)[3], 255);
        let mid = at(32, 30);
        assert!(mid[0] > 240 && mid[3] == 255, "{mid:?}");
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
