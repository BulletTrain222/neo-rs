//! Foliage patches drawn (SPEC-0060 Behavior 1 to 3, 6): the plants of the play's built patches
//! with their family model (the near model, else the far model) from the world pack's type-8
//! children by registry id (283 to 320 from resource 9), each with its yaw, scale and colour.
//! Bushes draw their main model scaled by the bush's draw scale, and their cut model in state 0.
//! No fades or far batches yet (see `neopets_sim::foliage`).
use crate::play::Play;
use bevy::prelude::*;
use neopets_data::Disc;
use neopets_data::archive::ArchiveIndex;
use std::collections::HashMap;

/// A bush plant's part: (pack, patch, plant, whether it is the cut model).
#[derive(Component, Clone, Copy)]
pub struct BushPart {
    pack: u32,
    patch: usize,
    plant: usize,
    cut: bool,
}

/// Build the models one world's patches draw: each patch model and, for bushes, the cut model.
pub fn models(disc: &mut Disc, index: &ArchiveIndex, play: &Play, pack: u32) -> Result<HashMap<u16, neopets_data::model::ModelData>, String> {
    let Some(patches) = play.foliage.get(&pack) else { return Ok(HashMap::new()) };
    let mut ids: Vec<u16> = patches.iter().map(|p| p.model).collect();
    ids.extend(patches.iter().filter(|p| !p.bushes.is_empty()).filter_map(|p| neopets_sim::foliage::cut_model(p.model)));
    ids.sort_unstable();
    ids.dedup();
    let bytes = index.read(disc, pack)?;
    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
    let mut shared: Option<(Vec<u8>, Vec<neopets_data::resource::Chunk>)> = None;
    let mut out = HashMap::new();
    for id in ids {
        let model = if (283..=320).contains(&id) {
            if shared.is_none() {
                let b = index.read(disc, neopets_data::effect::RESOURCE)?;
                let c = neopets_data::resource::walk_chunks(&b, None)?.chunks;
                shared = Some((b, c));
            }
            let (b, c) = shared.as_ref().unwrap();
            neopets_data::model::build_roles(b, c, None, Some(&[id - 283]))?
        } else {
            neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[id]))?
        };
        if !model.parts.is_empty() {
            out.insert(id, model);
        }
    }
    Ok(out)
}

/// Spawn one world's plants: each model's parts once as meshes, a material per (part material,
/// plant colour), one entity per plant and part; bush parts carry `BushPart`.
#[allow(clippy::too_many_arguments)]
pub fn spawn<C: Component + Copy>(
    commands: &mut Commands,
    play: &Play,
    pack: u32,
    models: &HashMap<u16, neopets_data::model::ModelData>,
    tag: C,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> usize {
    let Some(patches) = play.foliage.get(&pack) else { return 0 };
    let mut assets: HashMap<u16, Vec<(usize, Handle<Mesh>, Handle<StandardMaterial>)>> = HashMap::new();
    let mut tinted: HashMap<(AssetId<StandardMaterial>, [u8; 4]), Handle<StandardMaterial>> = HashMap::new();
    let mut count = 0;
    for (pi, p) in patches.iter().enumerate() {
        let bush = !p.bushes.is_empty();
        let mut drawn: Vec<(u16, bool)> = vec![(p.model, false)];
        if let Some(c) = neopets_sim::foliage::cut_model(p.model).filter(|_| bush) {
            drawn.push((c, true));
        }
        for (id, cut) in drawn {
            let Some(model) = models.get(&id) else { continue };
            let parts = assets.entry(id).or_insert_with(|| crate::part_assets(model, meshes, materials, images)).clone();
            for (k, q) in p.plants.iter().enumerate() {
                let transform = Transform::from_translation(Vec3::from(q.position)).with_rotation(Quat::from_rotation_y(q.yaw)).with_scale(Vec3::splat(q.scale));
                let visibility = if cut { Visibility::Hidden } else { Visibility::Visible };
                for (_, mesh, material) in &parts {
                    let m = tinted
                        .entry((material.id(), q.colour))
                        .or_insert_with(|| {
                            let mut base = materials.get(material).cloned().unwrap_or_default();
                            let f = |b: u8| (b as f32 / 128.0).min(1.0);
                            base.base_color = Color::srgba(f(q.colour[0]), f(q.colour[1]), f(q.colour[2]), f(q.colour[3]));
                            materials.add(base)
                        })
                        .clone();
                    let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(m), transform, visibility, tag, PatchPart { pack, patch: pi }));
                    if bush {
                        e.insert(BushPart { pack, patch: pi, plant: k, cut });
                    }
                }
                count += usize::from(!cut);
            }
        }
    }
    count
}

/// The patch a plant entity belongs to (for the zone gate).
#[derive(Component, Clone, Copy)]
pub struct PatchPart {
    pub pack: u32,
    pub patch: usize,
}

/// Each frame: a plant shows only while its patch passed the zone gate; a bush's main model
/// shows unless it is cut (state 0), its cut model only then, both at the bush's draw scale.
pub fn sync_bushes(state: Option<Res<crate::PlayState>>, mut parts: Query<(&PatchPart, Option<&BushPart>, &mut Transform, &mut Visibility)>) {
    let Some(state) = state else { return };
    for (p, b, mut t, mut vis) in &mut parts {
        let on = state.play.patch_on(p.pack, p.patch);
        let shown = match b {
            None => on,
            Some(b) => {
                let Some(bush) = state.play.foliage.get(&b.pack).and_then(|p| p.get(b.patch)).and_then(|p| p.bushes.get(b.plant)) else { continue };
                t.scale = Vec3::splat(bush.scale().max(0.0));
                on && (bush.state == 0) == b.cut
            }
        };
        let want = if shown { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
}
