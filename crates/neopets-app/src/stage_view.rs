//! The developer test stage's look (this build's own art, not from any disc): a wide flat
//! floor of muted stone tiles, the arena ring the Cloud Beast moves on (radius 40 by default, as
//! its stand-in arena), a mark at the centre and a stone wall round the arena (radius 50, at the
//! user's request; solid as a ring of push boxes, `wall_boxes`). The stage itself is
//! `play::World::test_stage`.
use bevy::prelude::*;

/// The floor's half width and one tile's size, in world units.
const HALF: f32 = 600.0;
const TILE: f32 = 4.0;
/// The arena ring's radius (the Cloud Beast's stand-in ring: its six points sit on it) and the
/// drawn line's width. 40 by default at the user's request (neighbouring points 40 apart,
/// opposite ones 80; the two disc levels with points 101 to 106 spread theirs over about 30 to 35
/// units); NEOPETS_CLOUD_RING
/// sets another radius (diagnostic).
pub fn ring() -> f32 {
    static R: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *R.get_or_init(|| std::env::var("NEOPETS_CLOUD_RING").ok().and_then(|v| v.trim().parse::<f32>().ok()).filter(|r| *r > 2.0).unwrap_or(40.0))
}
const LINE: f32 = 0.35;

/// The arena wall: its inner radius, thickness and height, and the push boxes that make it
/// solid (this build's own stage; WALL_SEGMENTS boxes, each a little longer than its share of
/// the circle so neighbours overlap and leave no gap).
pub const WALL: f32 = 50.0;
const WALL_THICK: f32 = 2.0;
const WALL_HEIGHT: f32 = 10.0;
const WALL_SEGMENTS: usize = 64;

pub fn wall_boxes() -> Vec<neopets_sim::contact::BoxShape> {
    let len = std::f32::consts::TAU * (WALL + WALL_THICK) / WALL_SEGMENTS as f32 + 0.6;
    (0..WALL_SEGMENTS)
        .map(|k| {
            let a = k as f32 * std::f32::consts::TAU / WALL_SEGMENTS as f32;
            let (out, along) = ([a.cos(), 0.0, a.sin()], [-a.sin(), 0.0, a.cos()]);
            neopets_sim::contact::BoxShape {
                origin: [WALL * out[0] - 0.5 * len * along[0], 0.0, WALL * out[2] - 0.5 * len * along[2]],
                axes: [[0.0, 1.0, 0.0], along, out],
                extents: [WALL_HEIGHT, len, WALL_THICK],
            }
        })
        .collect()
}

/// The tile texture: 8 x 8 tiles of 8 pixels in two stone tones, each tile with a darker
/// edge, so the floor reads as ground at speed.
fn tiles() -> Image {
    const N: u32 = 64;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let (tx, ty) = (x / 8, y / 8);
            let base: [f32; 3] = if (tx + ty) % 2 == 0 { [0.47, 0.50, 0.44] } else { [0.41, 0.44, 0.39] };
            let edge = x % 8 == 0 || y % 8 == 0;
            let k = if edge { 0.82 } else { 1.0 };
            rgba.extend(base.iter().map(|c| (c * k * 255.0) as u8));
            rgba.push(255);
        }
    }
    let mut image = crate::graphics::with_mipmaps(crate::rgba_image(N, N, rgba));
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
    image.sampler = crate::graphics::world_sampler(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

/// Spawn the stage's floor, ring and centre mark, each tagged with `part`.
pub fn spawn(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>, part: impl Component + Copy) {
    let floor = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(tiles())),
        perceptual_roughness: 1.0,
        reflectance: 0.1,
        // One texture repeat covers 8 tiles.
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.0 * HALF / (8.0 * TILE))),
        ..default()
    });
    commands.spawn((Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(HALF)).mesh())), MeshMaterial3d(floor), Transform::IDENTITY, bevy::light::NotShadowCaster, part));
    let line = materials.add(StandardMaterial { base_color: Color::srgb(0.85, 0.82, 0.62), unlit: true, ..default() });
    // The flat shapes are built in the XY plane; laid down on the floor, a hair above it.
    let flat = Transform::from_xyz(0.0, 0.02, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((Mesh3d(meshes.add(Annulus::new(ring() - LINE / 2.0, ring() + LINE / 2.0).mesh().resolution(96))), MeshMaterial3d(line.clone()), flat, bevy::light::NotShadowCaster, part));
    commands.spawn((Mesh3d(meshes.add(Annulus::new(0.9, 0.9 + LINE).mesh().resolution(32))), MeshMaterial3d(line), flat, bevy::light::NotShadowCaster, part));
    // The wall: the annulus extruded upward (the extrusion runs along its local Z), in a warm
    // stone tone, lit and casting shadows.
    let stone = materials.add(StandardMaterial { base_color: Color::srgb(0.58, 0.55, 0.48), perceptual_roughness: 1.0, reflectance: 0.05, ..default() });
    let up = Transform::from_xyz(0.0, 0.5 * WALL_HEIGHT, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((Mesh3d(meshes.add(Extrusion::new(Annulus::new(WALL, WALL + WALL_THICK), WALL_HEIGHT).mesh().resolution(2 * WALL_SEGMENTS as u32))), MeshMaterial3d(stone), up, part));
}
