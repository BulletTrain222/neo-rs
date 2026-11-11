//! The developer test stage's look (this build's own art, not from any disc): a wide flat
//! floor of muted stone tiles, the arena ring the Cloud Beast moves on (radius 40 by default, as
//! its stand-in arena), a mark at the centre, a stone wall round the arena (radius 50, at the
//! user's request; solid as a ring of push boxes, `wall_boxes`) and four storm-stone pillars
//! with cloud wisps at their feet (`ROCKS`; solid as stacked push spheres, and the Cloud Beast's
//! bolts bounce off them). The Cloud Beast's six points sit on a fixed uneven ring
//! (`arena_points`), each marked by a stone disc. The stage itself is `play::World::test_stage`.
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

/// The six arena points of the test stage (this build's layout, not the original's: no Cloud
/// Beast arena exists on the disc): a ring round the centre in order, each point's radius and
/// angle varied by a fixed amount (radius 0.78 to 1.0 of `ring()`, angle within 13 degrees) so
/// it reads as a natural arena while every move of 2 to 4 steps still crosses the middle. Point
/// 0 lies on +Z; each faces the centre.
pub fn arena_points() -> [([f32; 3], [f32; 3]); 6] {
    const SHAPE: [(f32, f32); 6] = [(1.0, 0.0), (0.82, 9.0), (0.95, -12.0), (0.78, 6.0), (0.92, -8.0), (0.86, 13.0)];
    std::array::from_fn(|k| {
        let (f, da) = SHAPE[k];
        let a = (k as f32 * 60.0 + da).to_radians();
        let d = [a.sin(), 0.0, a.cos()];
        ([d[0] * f * ring(), 0.0, d[2] * f * ring()], [-d[0], 0.0, -d[2]])
    })
}

/// The storm-stone pillars: base centre (x, z), radius and height (this build's own layout),
/// placed between the centre and the ring, clear of the hero's start at the centre.
pub const ROCKS: [([f32; 2], f32, f32); 4] = [([9.6, 10.7], 2.6, 6.5), ([13.0, -10.9], 2.0, 5.0), ([-10.6, -10.3], 3.0, 7.5), ([-16.4, 9.5], 2.2, 4.5)];

/// The pillars as push spheres (centre, radius), stacked every 2.5 up the height so neither
/// walking nor a jump passes through.
pub fn rock_spheres() -> Vec<([f32; 3], f32)> {
    ROCKS.iter().flat_map(|&([x, z], r, h)| (0..=((h / 2.5) as usize)).map(move |k| ([x, 2.5 * k as f32, z], r))).collect()
}

/// The first hit of the segment a to b on a pillar (a vertical cylinder of its radius and
/// height): the fraction along the segment and the outward normal there.
pub fn rock_hit(a: [f32; 3], b: [f32; 3]) -> Option<(f32, [f32; 3])> {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let qa = dx * dx + dz * dz;
    ROCKS
        .iter()
        .filter_map(|&([x, z], r, h)| {
            let (ox, oz) = (a[0] - x, a[2] - z);
            let c = ox * ox + oz * oz - r * r;
            let t = if c <= 0.0 {
                0.0
            } else {
                if qa < 1e-9 {
                    return None;
                }
                let qb = 2.0 * (ox * dx + oz * dz);
                let disc = qb * qb - 4.0 * qa * c;
                if disc < 0.0 {
                    return None;
                }
                (-qb - disc.sqrt()) / (2.0 * qa)
            };
            let y = a[1] + t * (b[1] - a[1]);
            if !(0.0..=1.0).contains(&t) || y < 0.0 || y > h {
                return None;
            }
            let (nx, nz) = (ox + t * dx, oz + t * dz);
            let l = nx.hypot(nz).max(1e-6);
            Some((t, [nx / l, 0.0, nz / l]))
        })
        .min_by(|p, q| p.0.total_cmp(&q.0))
}

/// A cloud wisp at a pillar's foot: its pillar, its phase and its size.
#[derive(Component)]
pub struct Wisp(usize, f32, f32);

/// The wisps drift slowly round their pillars and breathe.
pub fn drift_wisps(time: Res<Time>, mut wisps: Query<(&Wisp, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (w, mut tr) in &mut wisps {
        let ([x, z], r, _) = ROCKS[w.0];
        let a = w.1 + 0.12 * t;
        let d = r + 0.9 + 0.4 * (t * 0.7 + w.1).sin();
        tr.translation = Vec3::new(x + d * a.cos(), 0.5 + 0.25 * (t * 0.9 + 2.0 * w.1).sin(), z + d * a.sin());
        tr.scale = Vec3::new(w.2, 0.55 * w.2, w.2) * (1.0 + 0.08 * (t * 1.3 + w.1).sin());
    }
}

/// A faceted pillar: a subdivided sphere pushed in and out by a fixed hash, flattened at the
/// foot and stretched to the pillar's radius and height, with flat normals (the PS2 manner).
fn rock_mesh(seed: u32, r: f32, h: f32) -> Mesh {
    let mut m = Sphere::new(1.0).mesh().ico(2).unwrap_or_else(|_| Sphere::new(1.0).mesh().uv(12, 8));
    let hash = |p: [f32; 3]| {
        let k = ((p[0] * 73.0) as i32 as u32).wrapping_mul(0x9E37_79B1) ^ ((p[1] * 151.0) as i32 as u32).wrapping_mul(0x85EB_CA77) ^ ((p[2] * 197.0) as i32 as u32).wrapping_mul(0xC2B2_AE3D) ^ seed.wrapping_mul(0x2545_F491);
        ((k ^ (k >> 15)).wrapping_mul(0x2C1B_3C6D) >> 16) as f32 / 65535.0
    };
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(ps)) = m.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for p in ps.iter_mut() {
            let k = 0.8 + 0.35 * hash(*p);
            let y = (p[1] * k).max(-0.55);
            *p = [p[0] * k * r, (y + 0.55) / 1.55 * h, p[2] * k * r];
        }
    }
    m.duplicate_vertices();
    m.compute_flat_normals();
    m
}

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
/// The reed field's plain world: the same tiled floor and nothing else (no ring, marks, pillars
/// or wall).
pub fn spawn_floor(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>, part: impl Component + Copy) {
    let floor = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(tiles())),
        perceptual_roughness: 1.0,
        reflectance: 0.1,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.0 * HALF / (8.0 * TILE))),
        ..default()
    });
    commands.spawn((Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(HALF)).mesh())), MeshMaterial3d(floor), Transform::IDENTITY, bevy::light::NotShadowCaster, part));
}

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
    // A stone disc on each of the six arena points.
    for (p, _) in arena_points() {
        commands.spawn((Mesh3d(meshes.add(Annulus::new(1.6, 1.6 + LINE).mesh().resolution(32))), MeshMaterial3d(line.clone()), Transform::from_xyz(p[0], 0.02, p[2]).with_rotation(flat.rotation), bevy::light::NotShadowCaster, part));
    }
    // The storm-stone pillars and the cloud wisps at their feet.
    let rock = materials.add(StandardMaterial { base_color: Color::srgb(0.46, 0.47, 0.58), perceptual_roughness: 1.0, reflectance: 0.05, ..default() });
    let wisp = materials.add(StandardMaterial { base_color: Color::srgba(0.9, 0.9, 0.98, 0.45), unlit: true, alpha_mode: AlphaMode::Blend, ..default() });
    let puff = meshes.add(Sphere::new(1.0).mesh().uv(14, 9));
    for (i, &([x, z], r, h)) in ROCKS.iter().enumerate() {
        commands.spawn((Mesh3d(meshes.add(rock_mesh(i as u32 + 1, r, h))), MeshMaterial3d(rock.clone()), Transform::from_xyz(x, 0.0, z), part));
        for k in 0..5 {
            let phase = k as f32 * 1.2566 + i as f32;
            commands.spawn((Mesh3d(puff.clone()), MeshMaterial3d(wisp.clone()), Transform::from_xyz(x, 0.5, z), bevy::light::NotShadowCaster, Wisp(i, phase, 0.9 + 0.25 * (k % 3) as f32), part));
        }
    }
    commands.spawn((Mesh3d(meshes.add(Annulus::new(0.9, 0.9 + LINE).mesh().resolution(32))), MeshMaterial3d(line), flat, bevy::light::NotShadowCaster, part));
    // The wall: the annulus extruded upward (the extrusion runs along its local Z), in a warm
    // stone tone, lit and casting shadows.
    let stone = materials.add(StandardMaterial { base_color: Color::srgb(0.58, 0.55, 0.48), perceptual_roughness: 1.0, reflectance: 0.05, ..default() });
    let up = Transform::from_xyz(0.0, 0.5 * WALL_HEIGHT, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    commands.spawn((Mesh3d(meshes.add(Extrusion::new(Annulus::new(WALL, WALL + WALL_THICK), WALL_HEIGHT).mesh().resolution(2 * WALL_SEGMENTS as u32))), MeshMaterial3d(stone), up, part));
}
