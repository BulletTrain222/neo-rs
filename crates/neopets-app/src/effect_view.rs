//! Effect sprites (SPEC-0034 4.7): each frame the live sprite particles are drawn as
//! camera-facing quads turned by their spin angle, one mesh per material group of the resource-9
//! shader list, with the group's texture and its blend (alpha or additive). The vertex colour is
//! the particle's colour bytes on the GS scale (128 is 1.0). Light emitters are not drawn; model
//! particles are `sync_effect_models` in main.
use crate::PlayState;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct EffectBatch(pub usize);

#[derive(Component)]
pub struct TrailBatch;

/// The material group of common object 297 (SPEC-0034 2.9 and 3.8): additive, the trails'.
const TRAIL_GROUP: usize = 11;

/// The swing trails (SPEC-0034 3.8 Draw): quads between consecutive live samples, from point 1
/// to point 2 of each, with the sample colours (GS scale) and texture u from the sample u, in
/// the additive material of group 11. One mesh for all trails.
#[allow(clippy::too_many_arguments)]
pub fn sync_trails(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut batch: Query<&mut Visibility, With<TrailBatch>>,
    mut made: Local<Option<(Handle<Mesh>, Option<Handle<StandardMaterial>>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _span = crate::perf::span("sync_trails");
    let Some(state) = state else { return };
    let play = &state.play;
    let (mut positions, mut uvs, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let k = |x: f32| (x / 128.0).clamp(0.0, 1.0);
    for t in play.trails.iter() {
        for (a, b) in t.segments() {
            let base = positions.len() as u32;
            for (s, v) in [(a, 0.0f32), (a, 1.0), (b, 1.0), (b, 0.0)] {
                positions.push(if v == 0.0 { s.p1 } else { s.p2 });
                uvs.push([s.u, v]);
                let c = s.colour;
                colours.push([k(c[0]).powf(2.2), k(c[1]).powf(2.2), k(c[2]).powf(2.2), k(c[3])]);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    let empty = indices.is_empty();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_indices(Indices::U32(indices));
    match made.as_ref() {
        Some((h, _)) => {
            let empty_again = empty && meshes.get(h).is_some_and(|m| m.count_vertices() == 0);
            if !empty_again {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = mesh;
                }
            }
        }
        None => {
            if empty {
                return;
            }
            let material = play.effect_images.get(TRAIL_GROUP).and_then(|g| {
                let image = g.image.as_ref()?;
                let mut texture = crate::rgba_image(image.width, image.height, image.rgba.clone());
                texture.sampler = bevy::image::ImageSampler::linear();
                Some(materials.add(StandardMaterial {
                    base_color_texture: Some(images.add(texture)),
                    unlit: true,
                    alpha_mode: if g.additive { AlphaMode::Add } else { AlphaMode::Blend },
                    cull_mode: None,
                    ..default()
                }))
            });
            let h = meshes.add(mesh);
            if let Some(m) = &material {
                commands.spawn((TrailBatch, Mesh3d(h.clone()), MeshMaterial3d(m.clone()), Transform::default(), Visibility::Visible, bevy::camera::visibility::NoFrustumCulling, bevy::light::NotShadowCaster));
            }
            *made = Some((h, material));
        }
    }
    for mut vis in &mut batch {
        *vis = if empty { Visibility::Hidden } else { Visibility::Visible };
    }
}

/// The materials made so far, by group, and each batch's mesh.
#[derive(Default)]
pub struct Batches {
    materials: HashMap<usize, Option<Handle<StandardMaterial>>>,
    meshes: HashMap<usize, Handle<Mesh>>,
}

#[allow(clippy::too_many_arguments)]
pub fn sync_effects(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut batches: Query<(&EffectBatch, &mut Visibility)>,
    mut made: Local<Batches>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _span = crate::perf::span("sync_effects");
    let Some(state) = state else { return };
    let Some(camera) = cameras.iter().next() else { return };
    let (right, up) = (camera.right().as_vec3(), camera.up().as_vec3());
    let play = &state.play;
    let mut by_group: HashMap<usize, Vec<(&neopets_sim::effect::Particle, [u8; 4])>> = HashMap::new();
    for (p, group) in play.effects.sprites(&play.effect_table) {
        by_group.entry(group).or_default().push((p, p.bytes()));
    }
    for (group, sprites) in &by_group {
        let material = made
            .materials
            .entry(*group)
            .or_insert_with(|| {
                let g = play.effect_images.get(*group)?;
                let image = g.image.as_ref()?;
                let mut texture = crate::rgba_image(image.width, image.height, image.rgba.clone());
                texture.sampler = bevy::image::ImageSampler::linear();
                Some(materials.add(StandardMaterial {
                    base_color_texture: Some(images.add(texture)),
                    unlit: true,
                    alpha_mode: if g.additive { AlphaMode::Add } else { AlphaMode::Blend },
                    cull_mode: None,
                    ..default()
                }))
            })
            .clone();
        let Some(material) = material else { continue };
        let mut positions = Vec::with_capacity(sprites.len() * 4);
        let mut uvs = Vec::with_capacity(sprites.len() * 4);
        let mut colours = Vec::with_capacity(sprites.len() * 4);
        let mut indices = Vec::with_capacity(sprites.len() * 6);
        for (p, b) in sprites {
            let (s, c) = (neopets_sim::effect::series_sin(p.spin), neopets_sim::effect::series_cos(p.spin));
            let centre = Vec3::from(p.position);
            let base = positions.len() as u32;
            for (u, v) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // SPEC-0034 16.4: lane z is the half-width; lane x reaches up to the top edge
                // and lane y down to the bottom edge.
                let (x, y) = (u * p.half[2], if v > 0.0 { p.half[0] } else { -p.half[1] });
                positions.push((centre + right * (x * c - y * s) + up * (x * s + y * c)).to_array());
                uvs.push([(u + 1.0) * 0.5, (1.0 - v) * 0.5]);
                // The GS scale: a byte of 128 is 1.0; Bevy blends vertex colour in linear space.
                let k = |x: u8| (x as f32 / 128.0).min(1.0);
                colours.push([k(b[0]).powf(2.2), k(b[1]).powf(2.2), k(b[2]).powf(2.2), k(b[3])]);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
        mesh.insert_indices(Indices::U32(indices));
        match made.meshes.get(group) {
            Some(h) => {
                let empty_again = mesh.count_vertices() == 0 && meshes.get(h).is_some_and(|m| m.count_vertices() == 0);
                if !empty_again {
                    if let Some(mut m) = meshes.get_mut(h) {
                        *m = mesh;
                    }
                }
            }
            None => {
                let h = meshes.add(mesh);
                made.meshes.insert(*group, h.clone());
                commands.spawn((EffectBatch(*group), Mesh3d(h), MeshMaterial3d(material), Transform::default(), Visibility::Visible, bevy::camera::visibility::NoFrustumCulling, bevy::light::NotShadowCaster));
            }
        }
    }
    for (b, mut vis) in &mut batches {
        *vis = if by_group.contains_key(&b.0) { Visibility::Visible } else { Visibility::Hidden };
    }
}
