//! Static batching of a world scene (this build's presentation; no spec): the scene's plain
//! parts are merged into one mesh per material, zone gate and 64-unit area cell, so the main
//! view and every shadow cascade cull and draw a few hundred entities instead of thousands.
//! A part keeps its own entity when it needs per-part handling: an environment-mapped pass 0,
//! a flipbook, a scrolling texture, an instance draw distance, or a twin in another drawn scene
//! (the duplicate rule of `sync_zones` hides such parts one by one). Pass 1 parts are never
//! merged.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::{model_texture, part_signature, pass1, pass1_assets, DrawCull, PartSig, UvFlip, UvScroll, WorldPart, ZoneGate};

/// The area cell a merged group covers, in world units.
const CELL: f32 = 64.0;

/// A part's triangles split into those solid at all three corners (vertex alpha at or above 120 of 128) and the others.
fn split_fading(p: &neopets_data::model::MeshPart) -> (Vec<u32>, Vec<u32>) {
    let (mut solid, mut faded) = (Vec::new(), Vec::new());
    for t in p.indices.chunks_exact(3) {
        let whole = t.iter().all(|&v| p.colors.get(v as usize).is_some_and(|c| c[3] >= 120.0 / 128.0));
        (if whole { &mut solid } else { &mut faded }).extend_from_slice(t);
    }
    (solid, faded)
}

/// A built scene kept for a respawn: its model, its pass-0 part signatures, its materials and
/// which parts were spawned as twins of another drawn scene.
pub struct Built {
    pub model: Arc<neopets_data::model::ModelData>,
    pub sigs: Vec<PartSig>,
    pub shared: Vec<bool>,
    textures: Vec<Handle<StandardMaterial>>,
    /// Each texture's opaque twin (made when a part needs it; see `mask_never_cuts`).
    opaque: HashMap<usize, Handle<StandardMaterial>>,
    /// Each texture's blended twin (made when a part's vertex alpha fades; see `vertex_fades`).
    blend: HashMap<usize, Handle<StandardMaterial>>,
    /// Textures with soft alpha (a texel between 16 and 239 of 255) and their cut twins at the
    /// alpha test's reference (SPEC-0003 TEST: ATE GEQUAL 120 of 128, AFAIL framebuffer only:
    /// texels at or above 120/128 write depth, softer ones still blend without writing it). A
    /// soft part is drawn twice: the cut twin, then the blended twin over it.
    soft: Vec<bool>,
    cut: HashMap<usize, Handle<StandardMaterial>>,
    untextured: Handle<StandardMaterial>,
}

impl Built {
    pub fn new(model: neopets_data::model::ModelData, pack: u32, materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>) -> Self {
        let textures: Vec<Handle<StandardMaterial>> = model
            .textures
            .iter()
            .map(|t| materials.add(StandardMaterial { base_color_texture: Some(images.add(model_texture(t))), unlit: true, alpha_mode: AlphaMode::Mask(0.5), cull_mode: None, ..default() }))
            .collect();
        let untextured = materials.add(StandardMaterial { unlit: true, cull_mode: None, ..default() });
        let sigs = model.parts.iter().map(|p| part_signature(pack, p, false)).collect();
        let shared = vec![false; model.parts.len()];
        let mut opaque = HashMap::new();
        let mut blend = HashMap::new();
        let soft: Vec<bool> = model.textures.iter().map(|t| std::env::var_os("NEOPETS_NO_SOFT").is_none() && t.image.rgba.chunks_exact(4).any(|q| (16..240).contains(&q[3]))).collect();
        let mut cut = HashMap::new();
        for p in &model.parts {
            if let Some(t) = p.texture.filter(|&t| p.flipbook.is_none() && !p.env_map && p.scroll.is_none() && soft[t] && !crate::vertex_fades(p)) {
                cut.entry(t).or_insert_with(|| crate::cut_twin(materials, &textures[t]));
                blend.entry(t).or_insert_with(|| crate::blend_twin(materials, &textures[t]));
                continue;
            }
            if let Some(t) = p.texture.filter(|_| p.flipbook.is_none() && crate::vertex_fades(p)) {
                blend.entry(t).or_insert_with(|| crate::blend_twin(materials, &textures[t]));
                continue;
            }
            if let Some(t) = p.texture.filter(|&t| p.flipbook.is_none() && crate::mask_never_cuts(&model.textures[t], p)) {
                opaque.entry(t).or_insert_with(|| crate::opaque_twin(materials, &textures[t]));
            }
        }
        Self { model: Arc::new(model), sigs, shared, textures, opaque, blend, soft, cut, untextured }
    }

    /// The parts that repeat a part of one of `others` (the other drawn scenes' signatures).
    pub fn twins(&self, others: &[&[PartSig]]) -> Vec<bool> {
        // NEOPETS_TRACE_TWINS: each large part's closest part in the other scenes (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_TWINS").is_some() {
            for s in self.sigs.iter().filter(|s| s.size() > 200.0) {
                let best = others.iter().flat_map(|o| o.iter()).map(|t| (t.diff(s), t.count, t.pack, t.same(s), s.same(t))).min_by(|a, b| a.0.total_cmp(&b.0));
                println!("twin check {} part ({} verts, size {:.0}, lo {:?}): closest {best:?}", s.pack, s.count, s.size(), s.lo);
            }
        }
        // A twin repeats a part of another scene, or its box is covered by 90 percent by the
        // other scene's boxes together (`sync_zones` hides it while the other world is current).
        let out: Vec<bool> = self.sigs.iter().map(|s| others.iter().any(|o| o.iter().any(|t| t.same(s)) || (crate::dup_level() >= 4 && o.iter().map(|t| t.contains(s)).sum::<f32>() >= 0.9))).collect();
        if std::env::var_os("NEOPETS_TRACE_TWINS").is_some() {
            println!("twins of {:?}: {} of {} against {} other scenes", self.sigs.first().map(|s| s.pack), out.iter().filter(|&&t| t).count(), out.len(), others.len());
            for (s, t) in self.sigs.iter().zip(&out).filter(|(s, _)| s.size() > 1000.0) {
                println!("  big part {} verts lo {:?}: twin {t}", s.count, s.lo);
            }
        }
        out
    }
}

/// A part that lies flat: its mean normal points up (above 0.9).
fn is_flat(p: &neopets_data::model::MeshPart) -> bool {
    let n = p.normals.len().max(1) as f32;
    p.normals.iter().map(|q| q[1]).sum::<f32>() / n > 0.9
}

/// One part's mesh, as `part_assets` builds it.
fn part_mesh(part: &neopets_data::model::MeshPart) -> Mesh {
    let g = crate::gs_unit(part);
    let colors: Vec<[f32; 4]> = part.colors.iter().map(|c| [(c[0] * g).powf(2.2), (c[1] * g).powf(2.2), (c[2] * g).powf(2.2), c[3].min(1.0)]).collect();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, part.uvs.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(if part.scroll.is_some() { part.indices.clone() } else { crate::drawn_indices(part) }));
    mesh
}

/// The merged mesh of several parts (their attributes appended, the indices offset).
fn merged_mesh(parts: &[&neopets_data::model::MeshPart]) -> Mesh {
    let (mut pos, mut nrm, mut uv, mut col, mut idx) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for part in parts {
        let base = pos.len() as u32;
        let g = crate::gs_unit(part);
        pos.extend_from_slice(&part.positions);
        nrm.extend_from_slice(&part.normals);
        uv.extend_from_slice(&part.uvs);
        col.extend(part.colors.iter().map(|c| [(c[0] * g).powf(2.2), (c[1] * g).powf(2.2), (c[2] * g).powf(2.2), c[3].min(1.0)]));
        idx.extend(crate::drawn_indices(part).iter().map(|i| i + base));
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    mesh.insert_indices(Indices::U32(idx));
    mesh
}

/// Spawn a scene: the merged groups, the parts kept apart and the pass-1 parts. Returns the
/// number of entities spawned.
#[allow(clippy::too_many_arguments)]
pub fn spawn(
    commands: &mut Commands,
    part: WorldPart,
    pack: u32,
    b: &Built,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pass1_materials: &mut Assets<pass1::Pass1Material>,
    images: &mut Assets<Image>,
) -> usize {
    let m = &*b.model;
    let no_pass1 = std::env::var_os("NEOPETS_NO_PASS1").is_some();
    // NEOPETS_NO_MERGE: every part keeps its own entity, as before (diagnostic).
    let no_merge = std::env::var_os("NEOPETS_NO_MERGE").is_some();
    let mut env_textures: HashMap<usize, Handle<Image>> = HashMap::new();
    let mut groups: HashMap<(AssetId<StandardMaterial>, Option<u8>, i32, i32, bool), (Handle<StandardMaterial>, Vec<usize>)> = HashMap::new();
    // NEOPETS_FLAT_NOCAST: flat ground parts cast no sun shadow (diagnostic, to measure).
    let flat_ground = std::env::var_os("NEOPETS_FLAT_NOCAST").is_some();
    let mut count = 0;
    let ranks = crate::draw_order::ranks(m);
    let rank = |i: usize, second: u32| crate::draw_order::DrawRank { pack: Some(pack), rank: ranks[i] + second };
    for (i, p) in m.parts.iter().enumerate() {
        let env = p.texture.filter(|_| p.env_map && !no_pass1);
        let scrolling = p.scroll.is_some() && p.texture.is_some();
        let fades = p.flipbook.is_none() && crate::vertex_fades(p);
        let soft = p.texture.is_some_and(|t| b.soft[t]) && p.flipbook.is_none() && !p.env_map && p.scroll.is_none() && !fades;
        let material = match p.texture {
            None => b.untextured.clone(),
            Some(t) if soft => b.cut.get(&t).cloned().unwrap_or_else(|| b.textures[t].clone()),
            Some(t) if fades => b.blend.get(&t).cloned().unwrap_or_else(|| b.textures[t].clone()),
            Some(t) if p.flipbook.is_none() && crate::mask_never_cuts(&m.textures[t], p) => b.opaque.get(&t).cloned().unwrap_or_else(|| b.textures[t].clone()),
            Some(t) => b.textures[t].clone(),
        };
        // A blended part is kept apart so it sorts with the other transparent draws.
        let apart = no_merge || env.is_some() || scrolling || fades || soft || p.flipbook.is_some() || p.cull.is_some() || b.shared[i];
        if !apart {
            let n = p.positions.len().max(1) as f32;
            let c = p.positions.iter().fold([0.0f32; 2], |a, q| [a[0] + q[0] / n, a[1] + q[2] / n]);
            let key = (material.id(), p.zone, (c[0] / CELL).floor() as i32, (c[1] / CELL).floor() as i32, flat_ground && is_flat(p));
            groups.entry(key).or_insert_with(|| (material, Vec::new())).1.push(i);
            continue;
        }
        // SPEC-0003 TEST: a pixel at or above 120 of 128 writes depth, a softer one only blends. A fading part's triangles
        // solid at all three corners draw as solid, so that what lies behind them stays hidden (user's F8 view 16); the rest
        // blend. Done by triangle, as Bevy's prepass does not see vertex alpha (see `vertex_fades`).
        let split = (fades && !scrolling && env.is_none()).then(|| split_fading(p)).filter(|(solid, _)| !solid.is_empty());
        let mesh = match &split {
            Some((solid, faded)) => {
                let t = p.texture.unwrap_or(0);
                let mut e = commands.spawn((Mesh3d(meshes.add(part_mesh(&neopets_data::model::MeshPart { indices: solid.clone(), ..p.clone() }))), MeshMaterial3d(b.textures[t].clone()), Transform::IDENTITY, part, b.sigs[i]));
                count += 1;
                if let Some(z) = p.zone {
                    e.insert(ZoneGate(pack, z));
                }
                if let Some((at, d1, _)) = p.cull {
                    e.insert(DrawCull(Vec3::from(at), d1));
                }
                meshes.add(part_mesh(&neopets_data::model::MeshPart { indices: faded.clone(), ..p.clone() }))
            }
            None => meshes.add(part_mesh(p)),
        };
        // A soft part's blended layer over its cut core.
        if let (true, Some(t)) = (soft, p.texture) {
            if let Some(m) = b.blend.get(&t) {
                let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(m.clone()), Transform::IDENTITY, part, b.sigs[i], rank(i, 0)));
                count += 1;
                if let Some(z) = p.zone {
                    e.insert(ZoneGate(pack, z));
                }
                if let Some((at, d1, _)) = p.cull {
                    e.insert(DrawCull(Vec3::from(at), d1));
                }
            }
        }
        let mut e = match env {
            // An environment-mapped pass 0 is drawn by the pass-1 shader with view-generated UVs
            // (pass1.rs), its texture shared per model texture.
            Some(t) => {
                let image = env_textures.entry(t).or_insert_with(|| images.add(model_texture(&m.textures[t]))).clone();
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(pass1_materials.add(pass1::env_base(image))), Transform::IDENTITY, part))
            }
            None if scrolling => {
                let own = materials.add(StandardMaterial { base_color_texture: Some(images.add(model_texture(&m.textures[p.texture.unwrap_or(0)]))), unlit: true, alpha_mode: AlphaMode::Blend, cull_mode: None, ..default() });
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(own), Transform::IDENTITY, part))
            }
            None => commands.spawn((Mesh3d(mesh), MeshMaterial3d(material.clone()), Transform::IDENTITY, part)),
        };
        count += 1;
        if let Some(z) = p.zone {
            e.insert(ZoneGate(pack, z));
        }
        e.insert((b.sigs[i], rank(i, 0)));
        if let Some((at, d1, _)) = p.cull {
            e.insert(DrawCull(Vec3::from(at), d1));
        }
        if let Some(speed) = p.scroll.filter(|_| p.texture.is_some()) {
            e.insert(UvScroll(speed));
        }
        // A variant-1 flipbook steps its own copy of the material through the cells.
        if let (Some(fb), None) = (p.flipbook, env) {
            if let Some(own) = materials.get(&material).cloned() {
                e.insert((MeshMaterial3d(materials.add(own)), UvFlip(fb)));
            }
        }
    }
    for ((_, zone, _, _, flat), (material, list)) in groups {
        let parts: Vec<&neopets_data::model::MeshPart> = list.iter().map(|&i| &m.parts[i]).collect();
        let mut e = commands.spawn((Mesh3d(meshes.add(merged_mesh(&parts))), MeshMaterial3d(material), Transform::IDENTITY, part));
        if flat {
            e.insert(bevy::light::NotShadowCaster);
        }
        count += 1;
        if let Some(z) = zone {
            e.insert(ZoneGate(pack, z));
        }
    }
    // Render pass 1 of the two-pass parts (pass1.rs), gated like their pass 0; its signature is
    // kept apart so the duplicate test never hides it as pass 0's twin.
    for (i, mesh, material, scroll) in pass1_assets(m, meshes, pass1_materials, images) {
        let mut e = commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, part, rank(i, 1)));
        count += 1;
        if let Some(z) = m.parts[i].zone {
            e.insert(ZoneGate(pack, z));
        }
        e.insert(part_signature(pack, &m.parts[i], true));
        if let Some(speed) = scroll {
            e.insert(pass1::Pass1Scroll(speed));
        }
    }
    count
}
