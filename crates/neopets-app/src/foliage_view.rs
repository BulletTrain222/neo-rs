//! Foliage patches drawn (SPEC-0060 Behavior 1 to 3): each patch's family model (the near
//! model, else the far model) from the world pack's type-8 children by registry id (ids 283 to
//! 320 from resource 9), placed at every plant with its yaw and scale and tinted by its colour
//! bytes. No fades, far batches or family logic yet (see `neopets_sim::foliage`).
use crate::play::Play;
use bevy::prelude::*;
use neopets_data::Disc;
use neopets_data::archive::ArchiveIndex;
use std::collections::HashMap;

/// A model's plants: (registry id, placed plants).
pub type Plants = Vec<(u16, Vec<neopets_sim::foliage::Placed>)>;

/// Place every plant of a resident world's patches whose model is present; returns the models
/// by registry id and the plants of each.
pub fn build(disc: &mut Disc, index: &ArchiveIndex, play: &Play, pack: u32) -> Result<(HashMap<u16, neopets_data::model::ModelData>, Plants), String> {
    let bytes = index.read(disc, pack)?;
    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
    let patches = neopets_data::foliage::patches(&bytes, &chain.chunks);
    let (trees, _) = neopets_data::model::find_trees(&bytes, &chain.chunks);
    let children: Vec<u16> = trees.iter().filter(|t| t.depth > 0).map(|t| t.role_id).collect();
    let world = [&play.world].into_iter().chain(play.other.as_ref()).find(|w| w.pack == pack);
    let mut models: HashMap<u16, neopets_data::model::ModelData> = HashMap::new();
    let mut shared: Option<(Vec<u8>, Vec<neopets_data::resource::Chunk>)> = None;
    let mut plants: HashMap<u16, Vec<neopets_sim::foliage::Placed>> = HashMap::new();
    let mut rng = neopets_sim::rng::Rng::seeded(pack);
    for p in &patches {
        let Some(kind) = neopets_sim::foliage::kind(p.key) else { continue };
        let Some(id) = kind.near.or(kind.far) else { continue };
        if !models.contains_key(&id) {
            let model = if (283..=320).contains(&id) {
                if shared.is_none() {
                    let b = index.read(disc, neopets_data::effect::RESOURCE)?;
                    let c = neopets_data::resource::walk_chunks(&b, None)?.chunks;
                    shared = Some((b, c));
                }
                let (b, c) = shared.as_ref().unwrap();
                neopets_data::model::build_roles(b, c, None, Some(&[id - 283]))?
            } else if children.contains(&id) {
                neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[id]))?
            } else {
                continue;
            };
            if model.parts.is_empty() {
                continue;
            }
            models.insert(id, model);
        }
        let depth = p.area[3] - p.area[2];
        let j = neopets_sim::foliage::jitter(kind.family, p.jitter, depth, p.rows.len());
        let ceiling = neopets_sim::foliage::probe_ceiling(&kind).unwrap_or(neopets_sim::motion::PROBE_CEILING);
        let ground = |x: f32, z: f32| world.and_then(|w| w.ground_at(x, z, p.height + ceiling));
        let list = plants.entry(id).or_default();
        for row in &p.rows {
            for plant in &row.plants {
                if let Some(q) = neopets_sim::foliage::place(&kind, p.area, p.height, row.offset, plant.offset, j, plant.colour, &ground, &mut rng) {
                    list.push(q);
                }
            }
        }
    }
    Ok((models, plants.into_iter().collect()))
}

/// Spawn the plants: each model's parts once as meshes, a material per (part material, plant
/// colour), and one entity per plant and part.
pub fn spawn<C: Component + Copy>(
    commands: &mut Commands,
    models: &HashMap<u16, neopets_data::model::ModelData>,
    plants: &Plants,
    tag: C,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> usize {
    let mut count = 0;
    for (id, list) in plants {
        let Some(model) = models.get(id) else { continue };
        let parts = crate::part_assets(model, meshes, materials, images);
        let mut tinted: HashMap<(AssetId<StandardMaterial>, [u8; 4]), Handle<StandardMaterial>> = HashMap::new();
        for q in list {
            let transform = Transform::from_translation(Vec3::from(q.position)).with_rotation(Quat::from_rotation_y(q.yaw)).with_scale(Vec3::splat(q.scale));
            for (_, mesh, material) in &parts {
                let m = tinted
                    .entry((material.id(), q.colour))
                    .or_insert_with(|| {
                        let mut base = materials.get(material).cloned().unwrap_or_default();
                        let k = |b: u8| (b as f32 / 128.0).min(1.0);
                        base.base_color = Color::srgba(k(q.colour[0]), k(q.colour[1]), k(q.colour[2]), k(q.colour[3]));
                        materials.add(base)
                    })
                    .clone();
                commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(m), transform, tag));
            }
            count += 1;
        }
    }
    count
}
