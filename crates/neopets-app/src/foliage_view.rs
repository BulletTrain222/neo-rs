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
    // The far sprite models (SPEC-0060 Behavior 4 far batch).
    ids.extend(patches.iter().filter_map(|p| p.kind.far));
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
        if std::env::var_os("NEOPETS_TRACE_FOLIAGE_MODELS").is_some() {
            println!("foliage model {id}: {} parts, bounds {:?} to {:?}, first part {} vertices", model.parts.len(), model.min, model.max, model.parts.first().map_or(0, |p| p.positions.len()));
        }
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
        let bush = !p.bushes.is_empty() || !p.reeds.is_empty();
        let mut drawn: Vec<(u16, bool)> = vec![(p.model, false)];
        if let Some(c) = neopets_sim::foliage::cut_model(p.model).filter(|_| !p.bushes.is_empty()) {
            drawn.push((c, true));
        }
        // A family with no near model draws its far sprite (grass).
        let far_only = p.kind.near.is_none();
        // SPEC-0060 section 3: the patch distances, from its overrides or the near model's
        // distance-cull node.
        let lod_of = |id: Option<u16>| id.and_then(|i| models.get(&i)).and_then(|m| m.lod);
        let distances = neopets_sim::foliage::Distances::of(p.overrides, lod_of(p.kind.near), p.kind.far.is_some());
        if std::env::var_os("NEOPETS_TRACE_FOLIAGE_MODELS").is_some() {
            println!("foliage patch {pi}: models {:?}/{:?}, overrides {:?}, near lod {:?}, distances {:?}", p.kind.near, p.kind.far, p.overrides, lod_of(p.kind.near), distances);
        }
        let lod = |role: Role, plane_x: bool| FoliageLod { distances, near: p.kind.near.is_some(), far: p.kind.far.is_some(), role, plane_x };
        // A family with both models also has its far sprites, shown only by the original's
        // distance rule (the Display page's "Distant foliage").
        if let (Some(_), Some(f)) = (p.kind.near, p.kind.far) {
            if let Some(model) = models.get(&f) {
                let plane_x = quad_in_x(model);
                let parts = assets.entry(f).or_insert_with(|| crate::part_assets(model, meshes, materials, images)).clone();
                for q in &p.plants {
                    let transform = Transform::from_translation(Vec3::from(q.position)).with_scale(Vec3::splat(q.scale));
                    for (_, mesh, material) in &parts {
                        let m = tinted
                            .entry((material.id(), q.colour))
                            .or_insert_with(|| {
                                let mut base = materials.get(material).cloned().unwrap_or_default();
                                let k = q.colour.map(|b| (b as f32 / 128.0).min(2.0));
                                base.base_color = Color::srgba(k[0].min(1.0), k[1].min(1.0), k[2].min(1.0), k[3].min(1.0));
                                base.alpha_mode = AlphaMode::Blend;
                                materials.add(base)
                            })
                            .clone();
                        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(m), transform, Visibility::Hidden, tag, PatchPart { pack, patch: pi }, lod(Role::Far, plane_x)));
                    }
                }
            }
        }
        for (id, cut) in drawn {
            let Some(model) = models.get(&id) else { continue };
            let parts = assets.entry(id).or_insert_with(|| crate::part_assets(model, meshes, materials, images)).clone();
            for (k, q) in p.plants.iter().enumerate() {
                let transform = Transform::from_translation(Vec3::from(q.position)).with_rotation(Quat::from_rotation_y(q.yaw)).with_scale(Vec3::splat(q.scale));
                let visibility = if cut { Visibility::Hidden } else { Visibility::Visible };
                for (_, mesh, material) in &parts {
                    let m = tinted
                        .entry((material.id(), if far_only { q.colour } else { [0; 4] }))
                        .or_insert_with(|| {
                            let mut base = materials.get(material).cloned().unwrap_or_default();
                            // SPEC-0060 Plant colour in the draw (R-0072): a plant drawn by its near
                            // model ignores the colour word (the fixed default colour, opaque); one
                            // drawn only by its far sprite takes it as the GS vertex colour with
                            // MODULATE: each channel byte / 128 (64 grey draws at half brightness)
                            // and the alpha byte / 128 as its opacity.
                            if far_only {
                                let k = q.colour.map(|b| (b as f32 / 128.0).min(2.0));
                                base.base_color = Color::srgba(k[0].min(1.0), k[1].min(1.0), k[2].min(1.0), k[3].min(1.0));
                                base.alpha_mode = AlphaMode::Blend;
                            }
                            materials.add(base)
                        })
                        .clone();
                    let role = if far_only { Role::Far } else { Role::Near };
                    let plane_x = far_only && quad_in_x(model);
                    let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(m), transform, visibility, tag, PatchPart { pack, patch: pi }, lod(role, plane_x)));
                    if bush {
                        e.insert(BushPart { pack, patch: pi, plant: k, cut });
                    }
                    // World shadows (this build's option): only trees cast; grass,
                    // flowers and bushes bend round the hero and spring back, and their moving
                    // shadows showed as a dark patch fading behind him.
                    if p.kind.family != neopets_sim::foliage::Family::Tree {
                        e.insert(bevy::light::NotShadowCaster);
                    }
                }
                count += usize::from(!cut);
            }
        }
    }
    count
}

/// Whether a far sprite's quad lies in the model's YZ plane (its face along X), else in XY (face
/// along Z): the far models are single upright quads (199 in XY; 203 and 283 in YZ).
fn quad_in_x(m: &neopets_data::model::ModelData) -> bool {
    (m.max[0] - m.min[0]).abs() < (m.max[2] - m.min[2]).abs()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Near,
    Far,
}

/// A plant entity's distances and role (SPEC-0060 sections 3 and 4).
#[derive(Component, Clone, Copy)]
pub struct FoliageLod {
    distances: neopets_sim::foliage::Distances,
    near: bool,
    far: bool,
    role: Role,
    plane_x: bool,
}

/// The patch a plant entity belongs to (for the zone gate).
#[derive(Component, Clone, Copy)]
pub struct PatchPart {
    pub pack: u32,
    pub patch: usize,
}

/// Each frame: a plant shows only while its patch passed the zone gate; a bush's main model
/// shows unless it is cut (state 0), its cut model only then, both at the bush's draw scale.
/// Each frame, also: a far sprite turns upright to face the camera (the camera-facing sprite of
/// SPEC-0060 Behavior 4, kept upright here), and with the Display page's original distant
/// foliage, a plant shows only while its weight for its role is above 0 and its patch lies
/// within the far distance F (the weights are not used as opacity here).
pub fn sync_bushes(
    state: Option<Res<crate::PlayState>>,
    settings: Res<crate::display::DisplaySettings>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut parts: Query<(&PatchPart, Option<&BushPart>, Option<&FoliageLod>, &mut Transform, &mut Visibility), Without<Camera3d>>,
) {
    let Some(state) = state else { return };
    let eye = cameras.iter().next().map(|c| c.translation());
    let original = settings.original_foliage;
    // NEOPETS_TRACE_FOLIAGE_LOD: the shown near and far plant parts every 60 ticks (diagnostic).
    let trace = std::env::var_os("NEOPETS_TRACE_FOLIAGE_LOD").is_some() && state.play.ticks % 60 == 0;
    let (mut near_shown, mut far_shown) = (0, 0);
    for (p, b, lod, mut t, mut vis) in &mut parts {
        let mut on = state.play.patch_on(p.pack, p.patch);
        if let (Some(l), Some(e)) = (lod, eye) {
            let to = e - t.translation;
            if l.role == Role::Far {
                let yaw = to.x.atan2(to.z);
                t.rotation = Quat::from_rotation_y(if l.plane_x { yaw - std::f32::consts::FRAC_PI_2 } else { yaw });
            }
            let centre = state.play.foliage.get(&p.pack).and_then(|v| v.get(p.patch)).map(|q| Vec3::from(q.centre));
            if original {
                let gate = centre.is_some_and(|c| c.distance_squared(e) < l.distances.far * l.distances.far);
                let (a, w) = l.distances.weights(to.length(), l.near, l.far);
                on &= gate && if l.role == Role::Near { a > 0.0 } else { w > 0.0 };
            } else if l.role == Role::Far && l.near {
                // Full detail: a family with a near model draws only its near model.
                on = false;
            }
        }
        let shown = match b {
            None => on,
            Some(b) => {
                let Some(patch) = state.play.foliage.get(&b.pack).and_then(|p| p.get(b.patch)) else { continue };
                // A reed (SPEC-0060 Behavior 9): drawn along its up axis, its cut piece at the
                // fall's offset, hidden once gone.
                if let (Some(r), Some(q)) = (patch.reeds.get(b.plant), patch.plants.get(b.plant)) {
                    t.translation = Vec3::from(q.position) + Vec3::from(r.offset);
                    t.rotation = Quat::from_rotation_arc(Vec3::Y, Vec3::from(r.up)) * Quat::from_rotation_y(q.yaw);
                    let want = if on && r.state != 2 { Visibility::Visible } else { Visibility::Hidden };
                    if *vis != want {
                        *vis = want;
                    }
                    continue;
                }
                let (Some(bush), Some(q)) = (patch.bushes.get(b.plant), patch.plants.get(b.plant)) else { continue };
                t.scale = Vec3::splat(bush.scale().max(0.0));
                // The bush is drawn along its up axis (the hero bend and spring-back).
                t.rotation = Quat::from_rotation_arc(Vec3::Y, Vec3::from(bush.up)) * Quat::from_rotation_y(q.yaw);
                on && (bush.state == 0) == b.cut
            }
        };
        let want = if shown { Visibility::Visible } else { Visibility::Hidden };
        if trace && shown {
            match lod.map(|l| l.role) {
                Some(Role::Far) => far_shown += 1,
                _ => near_shown += 1,
            }
        }
        if *vis != want {
            *vis = want;
        }
    }
    if trace {
        println!("foliage on tick {}: {near_shown} near parts, {far_shown} far parts shown (original {original})", state.play.ticks);
    }
}
