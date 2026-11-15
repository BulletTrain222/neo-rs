//! Player colours for multiplayer (this build's option, not in the original game): Tor's yellow
//! fur and Roberta's blue skin recoloured per player so the players can be told apart. Only the
//! fur's yellow texels and the skin's blue texels change: each keeps its shading (its brightness) and takes the chosen hue (or,
//! for White, loses its colour and lightens). The model is recoloured as a copy when it is drawn;
//! the game's data is unchanged.

/// The choices of a Player colour row; 0 is the hero's own colours.
pub const COLOURS: [&str; 7] = ["Original", "Red", "Blue", "Green", "Purple", "Pink", "White"];

/// The hue (degrees) each choice turns yellow into; None for Original and White.
fn target_hue(choice: u8) -> Option<f32> {
    match choice {
        1 => Some(0.0),
        2 => Some(215.0),
        3 => Some(120.0),
        4 => Some(275.0),
        5 => Some(325.0),
        _ => None,
    }
}

fn to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= 1e-6 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, if max <= 1e-6 { 0.0 } else { d / max }, max)
}

fn from_hsv(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h.rem_euclid(360.0) / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

/// Whether a texel is part of the yellow fur: a yellow to orange hue with some colour in it.
fn yellow(h: f32, s: f32, v: f32) -> bool {
    (25.0..=75.0).contains(&h) && s > 0.3 && v > 0.12
}

/// One RGBA image's yellow texels recoloured to `choice`; returns how many changed.
pub fn recolour(rgba: &mut [u8], choice: u8) -> usize {
    if choice == 0 || choice as usize >= COLOURS.len() {
        return 0;
    }
    let hue = target_hue(choice);
    let mut n = 0;
    for px in rgba.chunks_exact_mut(4) {
        let (r, g, b) = (px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        let (h, s, v) = to_hsv(r, g, b);
        if !yellow(h, s, v) {
            continue;
        }
        let (r, g, b) = match hue {
            Some(t) => from_hsv(t, s.min(0.85), v),
            // White: the colour goes and the shade lightens.
            None => {
                let l = (0.35 + 0.65 * v).min(1.0);
                from_hsv(0.0, 0.0, l)
            }
        };
        (px[0], px[1], px[2]) = ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8);
        n += 1;
    }
    n
}

/// Whether a texel is Roberta's blue skin: a strong mid blue (her skin texture is one colour,
/// RGB 0 99 167, in each of her outfits).
fn skin_blue(h: f32, s: f32, v: f32) -> bool {
    (190.0..=220.0).contains(&h) && s > 0.75 && v > 0.45 && v < 0.9
}

/// Whether a texture is Roberta's skin: nearly all of it her skin's blue. Her dresses' blues
/// (navy, a light blue, a grey blue) are darker or paler and keep theirs, as do her eyes and the
/// pink inside her ears.
pub fn skin_texture(rgba: &[u8]) -> bool {
    let (mut n, mut blue) = (0usize, 0usize);
    for px in rgba.chunks_exact(4).filter(|px| px[3] > 0) {
        n += 1;
        let (h, s, v) = to_hsv(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        blue += usize::from(skin_blue(h, s, v));
    }
    n > 0 && blue * 10 >= n * 9
}

/// One RGBA image's skin-blue texels recoloured to `choice` (as the fur's: the shade kept, the
/// hue changed, or White); returns how many changed.
pub fn recolour_skin(rgba: &mut [u8], choice: u8) -> usize {
    if choice == 0 || choice as usize >= COLOURS.len() {
        return 0;
    }
    let mut n = 0;
    for px in rgba.chunks_exact_mut(4) {
        let (h, s, v) = to_hsv(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        if !skin_blue(h, s, v) {
            continue;
        }
        let (r, g, b) = match target_hue(choice) {
            Some(t) => from_hsv(t, s.min(0.85), v),
            None => from_hsv(0.0, 0.0, (0.35 + 0.65 * v).min(1.0)),
        };
        (px[0], px[1], px[2]) = ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8);
        n += 1;
    }
    n
}

/// Roberta's skin as a vertex colour (her head, its tails, her face: untextured parts tinted a
/// light blue, about RGB 135 183 215 in shape), recoloured to `choice`; None when not that blue.
/// A little more colour than her pale blue, so a red or a green reads as one.
fn recolour_rgb_skin(c: [f32; 3], choice: u8) -> Option<[f32; 3]> {
    let k = c[0].max(c[1]).max(c[2]).max(1e-6);
    let (h, s, v) = to_hsv(c[0] / k, c[1] / k, c[2] / k);
    if !((185.0..=230.0).contains(&h) && s > 0.15) {
        return None;
    }
    let (r, g, b) = match target_hue(choice) {
        Some(t) => from_hsv(t, s.clamp(0.45, 0.85), v),
        None => from_hsv(0.0, 0.0, (0.35 + 0.65 * v).min(1.0)),
    };
    Some([r * k, g * k, b * k])
}

/// One colour (any scale: a vertex modulation of 1.0 is unchanged) recoloured as a texel is;
/// None when it is not yellow.
fn recolour_rgb(c: [f32; 3], choice: u8) -> Option<[f32; 3]> {
    let k = c[0].max(c[1]).max(c[2]).max(1e-6);
    let (h, s, v) = to_hsv(c[0] / k, c[1] / k, c[2] / k);
    // Only a strong yellow tint is fur (the ears, about 50 degrees); the brown straps and belts
    // of the clothes (25 to 35 degrees, duller) keep theirs.
    if !((40.0..=70.0).contains(&h) && s > 0.5) {
        return None;
    }
    let (r, g, b) = match target_hue(choice) {
        Some(t) => from_hsv(t, s.min(0.85), v),
        None => from_hsv(0.0, 0.0, (0.35 + 0.65 * v).min(1.0)),
    };
    Some([r * k, g * k, b * k])
}

/// Whether a texture is fur: mostly bright yellow, with no strong other colour (the tunic's
/// red and blue around its gold crowns), no black (the eye's pupil) and not dark (the leather).
/// Only fur textures are recoloured, so the armour's gold trim, the crowns and the eyes keep
/// their colours (the user's choice: only his fur).
pub fn fur_texture(rgba: &[u8]) -> bool {
    let (mut n, mut bright, mut black, mut other, mut white, mut pink) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for px in rgba.chunks_exact(4).filter(|px| px[3] > 0) {
        n += 1;
        let (h, s, v) = to_hsv(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        if yellow(h, s, v) {
            bright += usize::from(v > 0.55);
        } else if v > 0.85 && s < 0.3 {
            white += 1;
        } else if (h >= 330.0 || h <= 15.0) && s > 0.2 && s < 0.7 && v > 0.6 {
            pink += 1;
        } else if v < 0.12 {
            black += 1;
        } else if s > 0.45 && v > 0.25 && !((8.0..45.0).contains(&h) && v < 0.65) {
            // (A brown shade where the fur fades out is not another colour.)
            other += 1;
        }
    }
    // Fur has the white tips of the tail (about 37% of its texture) or the pink pads of the paws
    // (about 19%); the yellow cloth of the outfits (the rags' stripes, sashes, an emblem, a gem)
    // has neither.
    n > 0 && bright * 10 >= n * 3 && black * 100 < n * 8 && other * 100 < n * 8 && (white * 100 >= n * 15 || pink * 100 >= n * 8)
}

/// A copy of a hero model with its fur recoloured to `choice`: the fur textures' yellow texels
/// and the yellow vertex tints of the untextured or fur-textured parts (Tor's ears are a yellow
/// tint with no texture), the stored colour bytes too (the lit draw relights from them).
pub fn recoloured(model: &neopets_data::model::ModelData, choice: u8) -> neopets_data::model::ModelData {
    let mut m = model.clone();
    if choice == 0 || choice as usize >= COLOURS.len() {
        return m;
    }
    let fur: Vec<bool> = m.textures.iter().map(|t| fur_texture(&t.image.rgba)).collect();
    // Roberta's models are the ones with her skin texture: their untextured parts' light-blue
    // tints are her skin (her head and its tails), recoloured too.
    let roberta = m.textures.iter().any(|t| skin_texture(&t.image.rgba));
    if roberta {
        for part in m.parts.iter_mut().filter(|p| p.texture.is_none()) {
            for c in &mut part.colors {
                if let Some(n) = recolour_rgb_skin([c[0], c[1], c[2]], choice) {
                    (c[0], c[1], c[2]) = (n[0], n[1], n[2]);
                }
            }
            for c in &mut part.raw_colors {
                if let Some(n) = recolour_rgb_skin([c[0] as f32, c[1] as f32, c[2] as f32], choice) {
                    (c[0], c[1], c[2]) = (n[0].round().clamp(0.0, 255.0) as u8, n[1].round().clamp(0.0, 255.0) as u8, n[2].round().clamp(0.0, 255.0) as u8);
                }
            }
        }
    }
    for (t, &is_fur) in m.textures.iter_mut().zip(&fur) {
        if is_fur {
            recolour(&mut t.image.rgba, choice);
        } else if skin_texture(&t.image.rgba) {
            // Roberta's skin.
            recolour_skin(&mut t.image.rgba, choice);
        }
    }
    for part in m.parts.iter_mut().filter(|p| p.texture.is_none_or(|k| fur.get(k).copied().unwrap_or(false))) {
        for c in &mut part.colors {
            if let Some(n) = recolour_rgb([c[0], c[1], c[2]], choice) {
                (c[0], c[1], c[2]) = (n[0], n[1], n[2]);
            }
        }
        for c in &mut part.raw_colors {
            if let Some(n) = recolour_rgb([c[0] as f32, c[1] as f32, c[2] as f32], choice) {
                (c[0], c[1], c[2]) = (n[0].round().clamp(0.0, 255.0) as u8, n[1].round().clamp(0.0, 255.0) as u8, n[2].round().clamp(0.0, 255.0) as u8);
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yellow_takes_the_hue_and_keeps_its_shade() {
        // A bright yellow, a dark yellow-orange, a grey, a red and a blue texel.
        let mut px = vec![240, 200, 40, 255, 120, 80, 10, 255, 128, 128, 128, 255, 200, 30, 30, 255, 40, 60, 200, 255];
        let n = recolour(&mut px, 2);
        assert_eq!(n, 2, "only the two yellow texels change");
        let (h0, _, v0) = to_hsv(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        assert!((h0 - 215.0).abs() < 3.0 && (v0 - 240.0 / 255.0).abs() < 0.01, "blue at the same brightness");
        let (h1, _, v1) = to_hsv(px[4] as f32 / 255.0, px[5] as f32 / 255.0, px[6] as f32 / 255.0);
        assert!((h1 - 215.0).abs() < 4.0 && (v1 - 120.0 / 255.0).abs() < 0.01, "the shaded texel stays darker");
        assert_eq!(&px[8..20], &[128, 128, 128, 255, 200, 30, 30, 255, 40, 60, 200, 255], "grey, red and blue unchanged");
        // Alpha is never touched; Original changes nothing.
        let mut a = vec![240, 200, 40, 7];
        recolour(&mut a, 1);
        assert_eq!(a[3], 7);
        let mut b = vec![240, 200, 40, 255];
        assert_eq!(recolour(&mut b, 0), 0);
        assert_eq!(b, [240, 200, 40, 255]);
        // White: no colour left, lighter than it was dark.
        let mut w = vec![120, 80, 10, 255];
        recolour(&mut w, 6);
        assert!(w[0] == w[1] && w[1] == w[2] && w[0] > 120);
    }

    #[test]
    fn roberta_skin_only() {
        let tex = |pixels: &[[u8; 3]]| pixels.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect::<Vec<u8>>();
        let (skin, navy, light, grey, pink) = ([0, 99, 167], [30, 45, 85], [130, 180, 210], [80, 110, 150], [220, 120, 170]);
        assert!(skin_texture(&tex(&[skin; 8])));
        // Her dresses' blues, a pink and a mix are not skin.
        for t in [[navy; 8], [light; 8], [grey; 8], [pink; 8]] {
            assert!(!skin_texture(&tex(&t)));
        }
        assert!(!skin_texture(&tex(&[skin, skin, navy, navy, light, skin, skin, skin])));
        // Red skin keeps its shade; Original changes nothing.
        let mut px = tex(&[skin]);
        assert_eq!(recolour_skin(&mut px, 1), 1);
        let (h, _, v) = to_hsv(px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        assert!((h.min(360.0 - h)) < 3.0 && (v - 167.0 / 255.0).abs() < 0.01);
        let mut same = tex(&[skin]);
        assert_eq!(recolour_skin(&mut same, 0), 0);
        // Her head's light-blue tint turns; grey, the red mouth and the purple lids do not.
        let red = recolour_rgb_skin([16.0, 22.0, 26.0], 1).unwrap();
        assert!(red[0] > red[1] && red[0] > red[2] && (red[0] - 26.0).abs() < 0.5);
        assert!(recolour_rgb_skin([16.0, 16.0, 16.0], 1).is_none() && recolour_rgb_skin([10.0, 0.0, 0.0], 1).is_none() && recolour_rgb_skin([15.0, 10.0, 19.0], 1).is_none());
        // Tor's fur is not skin, and her skin is not fur.
        assert!(!skin_texture(&tex(&[[240, 200, 40]; 8])) && !fur_texture(&tex(&[skin; 8])));
    }

    #[test]
    fn only_fur_textures() {
        let tex = |pixels: &[[u8; 3]]| pixels.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect::<Vec<u8>>();
        let (yel, white, red, blue, black, olive) = ([240, 200, 40], [250, 250, 240], [200, 30, 30], [40, 60, 200], [5, 5, 5], [100, 90, 30]);
        // Fur: yellow fading to white tips; the paws, yellow round pink pads.
        assert!(fur_texture(&tex(&[yel, yel, yel, white, yel, white])));
        let pad = [235, 150, 160];
        assert!(fur_texture(&tex(&[yel, yel, yel, pad, yel, yel, yel, yel])));
        // Plain yellow cloth (no white tips, no pads).
        assert!(!fur_texture(&tex(&[yel, yel, yel, yel, yel, yel])));
        // The tunic: gold crowns on red and blue.
        assert!(!fur_texture(&tex(&[yel, red, red, blue, blue, red])));
        // The eye: yellow round a black pupil.
        assert!(!fur_texture(&tex(&[yel, yel, black, yel, yel, black])));
        // Dark leather of a yellow hue.
        assert!(!fur_texture(&tex(&[olive, olive, olive, olive])));

    }

    #[test]
    fn yellow_vertex_tints_turn_too() {
        // A yellow modulation above 1.0 keeps its scale; a neutral grey one is left alone.
        let n = recolour_rgb([1.6, 1.3, 0.2], 1).expect("yellow");
        assert!((n[0] - 1.6).abs() < 1e-3 && n[1] < 0.5 && n[2] < 0.5, "red at the same strength: {n:?}");
        assert!(recolour_rgb([1.0, 1.0, 1.0], 1).is_none());
        // A brown strap's tint (about 26 degrees) is left alone.
        assert!(recolour_rgb([15.0, 11.0, 8.0], 1).is_none());
    }
}
