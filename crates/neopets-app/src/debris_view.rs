//! Breakable debris on screen (SPEC-0048 Piece flight; R-0187): a model piece draws its debris model
//! with its own rows, a sprite piece a camera-facing quad in a material group of its world's shader
//! list. The models and textures are read from the pack the first time a piece needs them.
use crate::PlayState;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Component)]
pub struct DebrisRoot(u64);

#[derive(Component)]
pub struct DebrisBatch((u32, i32));

type Parts = Vec<(Handle<Mesh>, Handle<StandardMaterial>)>;

/// What has been built: each piece record's model, each (pack, group)'s material and batch mesh.
#[derive(Default)]
pub struct Made {
    models: HashMap<(u32, usize, usize), Parts>,
    materials: HashMap<(u32, i32), Option<Handle<StandardMaterial>>>,
    meshes: HashMap<(u32, i32), Handle<Mesh>>,
    bytes: HashMap<u32, Vec<u8>>,
    groups: HashMap<u32, Vec<neopets_data::effect::GroupImage>>,
}

#[allow(clippy::too_many_arguments)]
pub fn sync_debris(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut roots: Query<(Entity, &DebrisRoot, &mut Transform, &mut Visibility)>,
    mut batches: Query<(&DebrisBatch, &mut Visibility), Without<DebrisRoot>>,
    mut made: Local<Made>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _span = crate::perf::span("sync_debris");
    let Some(state) = state else { return };
    let play = &state.play;
    let Some(camera) = cameras.iter().next() else { return };
    let (right, up) = (camera.right().as_vec3(), camera.up().as_vec3());
    // The pack's bytes, once, on the first piece of a pack.
    made.bytes.retain(|p, _| *p == play.world.pack || play.other.as_ref().is_some_and(|o| o.pack == *p));
    if let Some(a) = archive.as_deref_mut() {
        for q in &play.debris.pieces {
            if !made.bytes.contains_key(&q.pack) {
                let crate::Archive { disc, index } = a;
                if let Ok(b) = index.read(disc, q.pack) {
                    made.bytes.insert(q.pack, b);
                }
            }
        }
    }
    let live: HashSet<u64> = play.debris.pieces.iter().filter(|q| q.model).map(|q| q.id).collect();
    let mut have = HashSet::new();
    for (entity, root, mut t, mut vis) in &mut roots {
        match play.debris.pieces.iter().find(|q| q.id == root.0).filter(|_| live.contains(&root.0)) {
            Some(q) => {
                have.insert(root.0);
                let r = q.rows();
                *t = Transform::from_matrix(Mat4::from_cols(Vec3::from(r[0]).extend(0.0), Vec3::from(r[1]).extend(0.0), Vec3::from(r[2]).extend(0.0), Vec3::from(q.position).extend(1.0)));
                *vis = if q.shown() { Visibility::Visible } else { Visibility::Hidden };
            }
            None => commands.entity(entity).despawn(),
        }
    }
    let world_of = |pack: u32| [&play.world].into_iter().chain(play.other.as_ref()).find(|w| w.pack == pack);
    for q in play.debris.pieces.iter().filter(|q| q.model && !have.contains(&q.id)) {
        let key = (q.pack, q.breakable, q.piece);
        if !made.models.contains_key(&key) {
            let mut parts = Parts::new();
            let range = world_of(q.pack).and_then(|w| w.breakables.get(q.breakable)).and_then(|b| b.debris.get(q.piece)).and_then(|d| d.model.clone());
            if let (Some(range), Some(bytes)) = (range, made.bytes.get(&q.pack)) {
                let chain = neopets_data::resource::walk_chunks(bytes, None);
                if let Some(m) = chain.ok().and_then(|c| neopets_data::model::build_tree(bytes, &c.chunks, range, u32::MAX).ok()) {
                    parts = crate::part_assets(&m, &mut meshes, &mut materials, &mut images).into_iter().map(|(_, mesh, mat)| (mesh, mat)).collect();
                }
            }
            made.models.insert(key, parts);
        }
        let parts = &made.models[&key];
        commands.spawn((DebrisRoot(q.id), Transform::from_translation(Vec3::from(q.position)), Visibility::Hidden)).with_children(|c| {
            for (mesh, mat) in parts {
                c.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::IDENTITY, bevy::light::NotShadowCaster));
            }
        });
    }
    // The sprites, one mesh per (pack, material group).
    let mut by_group: HashMap<(u32, i32), Vec<&neopets_sim::debris::Piece>> = HashMap::new();
    for q in play.debris.pieces.iter().filter(|q| !q.model && q.shown()) {
        let group = world_of(q.pack).and_then(|w| w.breakables.get(q.breakable)).and_then(|b| b.debris.get(q.piece)).map_or(-1, |d| d.material);
        by_group.entry((q.pack, group)).or_default().push(q);
    }
    for (&key, sprites) in &by_group {
        if !made.groups.contains_key(&key.0) {
            if let Some(bytes) = made.bytes.get(&key.0) {
                let list = neopets_data::resource::walk_chunks(bytes, None).ok().and_then(|c| neopets_data::effect::group_images(bytes, &c.chunks).ok()).unwrap_or_default();
                made.groups.insert(key.0, list);
            }
        }
        if !made.materials.contains_key(&key) {
            let g = usize::try_from(key.1).ok().and_then(|g| made.groups.get(&key.0)?.get(g));
            let material = g.and_then(|g| {
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
            made.materials.insert(key, material);
        }
        let Some(material) = made.materials[&key].clone() else { continue };
        let (mut positions, mut uvs, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for q in sprites {
            let (s, c) = q.spin.sin_cos();
            let (centre, half) = (Vec3::from(q.position), q.half());
            let base = positions.len() as u32;
            for (u, v) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // The lanes as effect sprites use them: z the half-width, x up and y down.
                let (x, y) = (u * half[2], if v > 0.0 { half[0] } else { -half[1] });
                positions.push((centre + right * (x * c - y * s) + up * (x * s + y * c)).to_array());
                uvs.push([(u + 1.0) * 0.5, (1.0 - v) * 0.5]);
                // Colour (0.5, 0.5, 0.5) is 128 on the GS scale, which is 1.0.
                colours.push([1.0, 1.0, 1.0, q.alpha]);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
        mesh.insert_indices(Indices::U32(indices));
        match made.meshes.get(&key) {
            Some(h) => {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = mesh;
                }
            }
            None => {
                let h = meshes.add(mesh);
                commands.spawn((DebrisBatch(key), Mesh3d(h.clone()), MeshMaterial3d(material), Transform::default(), Visibility::Visible, bevy::light::NotShadowCaster));
                made.meshes.insert(key, h);
            }
        }
    }
    for (batch, mut vis) in &mut batches {
        *vis = if by_group.contains_key(&batch.0) { Visibility::Visible } else { Visibility::Hidden };
    }
}
