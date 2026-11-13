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

/// A scene material drawn outside the depth prepass (see `Overlays`).
#[derive(Asset, TypePath, bevy::render::render_resource::AsBindGroup, Debug, Clone, Default)]
pub struct NoPrepass {}

impl bevy::pbr::MaterialExtension for NoPrepass {
    fn enable_prepass() -> bool {
        false
    }
}

pub type OverlayMaterial = bevy::pbr::ExtendedMaterial<StandardMaterial, NoPrepass>;

/// For each scene material drawn by a part laid over other ground, its copy outside the depth
/// prepass; `sync_overlays` keeps each copy equal to its source (graphics.rs lights and unlights
/// the scene materials in place).
#[derive(Resource, Default)]
pub struct Overlays(HashMap<AssetId<StandardMaterial>, Handle<OverlayMaterial>>);

impl Overlays {
    fn of(&mut self, source: &Handle<StandardMaterial>, materials: &Assets<StandardMaterial>, ext: &mut Assets<OverlayMaterial>) -> Handle<OverlayMaterial> {
        self.0
            .entry(source.id())
            .or_insert_with(|| ext.add(OverlayMaterial { base: materials.get(source).cloned().unwrap_or_default(), extension: NoPrepass {} }))
            .clone()
    }
}

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OverlayMaterial>::default()).init_resource::<Overlays>().add_systems(PostUpdate, sync_overlays);
    }
}

/// Each copy follows its source material; a copy whose source is gone is dropped.
fn sync_overlays(mut events: MessageReader<AssetEvent<StandardMaterial>>, mut overlays: ResMut<Overlays>, materials: Res<Assets<StandardMaterial>>, mut ext: ResMut<Assets<OverlayMaterial>>) {
    for e in events.read() {
        match *e {
            AssetEvent::Modified { id } => {
                if let (Some(h), Some(m)) = (overlays.0.get(&id), materials.get(id)) {
                    if let Some(mut x) = ext.get_mut(h) {
                        x.base = m.clone();
                    }
                }
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                overlays.0.remove(&id);
            }
            _ => {}
        }
    }
}

/// The area cell a merged group covers, in world units.
const CELL: f32 = 64.0;

/// A part lying along the ground: its triangles' area faces mostly up. A soft or fading one is a layer over the ground (a
/// path's torn grass edge), which almost touches the ground below it. It casts no sun shadow, which would draw as a dark band
/// along each path (user's F8 view 17), and it is drawn only blended, outside the depth prepass: ambient occlusion read
/// the prepass's small step at the layer's sides as a raised edge, with a hard line along each (user's F8 view 20).
fn lies_flat(p: &neopets_data::model::MeshPart) -> bool {
    let (mut up, mut all) = (0.0, 0.0);
    for t in p.indices.chunks_exact(3) {
        let [a, b, c] = [t[0], t[1], t[2]].map(|v| Vec3::from(p.positions[v as usize]));
        let n = (b - a).cross(c - a);
        up += n.y.abs();
        all += n.length();
    }
    all > 0.0 && up / all > 0.8
}

/// For each triangle of a part, whether solid ground lies just under it: its centre, then every
/// point of a grid of about one per unit over it, has a triangle of another part (not scrolling,
/// not a `skip`ped triangle) from 1.5 below it to `above` above it.
fn ground_under(m: &neopets_data::model::ModelData, tris: &TriGrid, i: usize, skip: &dyn Fn(usize, usize) -> bool, above: f32) -> Vec<bool> {
    let p = &m.parts[i];
    p.indices
        .chunks_exact(3)
        .map(|t| {
            let [a, b, c] = [t[0], t[1], t[2]].map(|v| Vec3::from(p.positions[v as usize]));
            tris.under(m, i, skip, above, (a + b + c) / 3.0) && tri_grid(a, b, c).all(|pt| tris.under(m, i, skip, above, pt))
        })
        .collect()
}

/// About one point per two units over a triangle (at most 36).
fn tri_grid(a: Vec3, b: Vec3, c: Vec3) -> impl Iterator<Item = Vec3> {
    let n = ((a.distance(b).max(b.distance(c)).max(c.distance(a)) / 2.0).ceil() as usize).clamp(1, 8);
    (0..n).flat_map(move |i| (0..n - i).map(move |j| a + (b - a) * ((i as f32 + 1.0 / 3.0) / n as f32) + (c - a) * ((j as f32 + 1.0 / 3.0) / n as f32)))
}

/// A scene's flat-ish triangles (not scrolling) by 4-unit cell across, for `ground_under`; the
/// few wider than 64 units (backdrops) by 128-unit cell.
struct TriGrid(HashMap<(i32, i32), Vec<(u32, u32)>>, HashMap<(i32, i32), Vec<(u32, u32)>>);

impl TriGrid {
    const CELL: f32 = 4.0;
    const WIDE: f32 = 128.0;

    fn new(m: &neopets_data::model::ModelData) -> Self {
        let mut map: HashMap<(i32, i32), Vec<(u32, u32)>> = HashMap::new();
        let mut wide: HashMap<(i32, i32), Vec<(u32, u32)>> = HashMap::new();
        for (j, q) in m.parts.iter().enumerate() {
            if q.scroll.is_some() {
                continue;
            }
            for (t, tri) in q.indices.chunks_exact(3).enumerate() {
                let [a, b, c] = [tri[0], tri[1], tri[2]].map(|k| Vec3::from(q.positions[k as usize]));
                // Steep triangles (walls) are never the ground under a point.
                if (b - a).cross(c - a).normalize_or_zero().y.abs() < 0.2 {
                    continue;
                }
                let (lo, hi) = (a.min(b).min(c), a.max(b).max(c));
                if hi.x - lo.x > 64.0 || hi.z - lo.z > 64.0 {
                    for x in (lo.x / Self::WIDE).floor() as i32..=(hi.x / Self::WIDE).floor() as i32 {
                        for z in (lo.z / Self::WIDE).floor() as i32..=(hi.z / Self::WIDE).floor() as i32 {
                            wide.entry((x, z)).or_default().push((j as u32, t as u32));
                        }
                    }
                    continue;
                }
                let (lo, hi) = (lo / Self::CELL, hi / Self::CELL);
                for x in lo.x.floor() as i32..=hi.x.floor() as i32 {
                    for z in lo.z.floor() as i32..=hi.z.floor() as i32 {
                        map.entry((x, z)).or_default().push((j as u32, t as u32));
                    }
                }
            }
        }
        Self(map, wide)
    }

    /// Whether a triangle of another part than `i` (not `skip`ped) lies from 1.5 below the point
    /// to `above` above it.
    fn under(&self, m: &neopets_data::model::ModelData, i: usize, skip: &dyn Fn(usize, usize) -> bool, above: f32, pt: Vec3) -> bool {
        let key = ((pt.x / Self::CELL).floor() as i32, (pt.z / Self::CELL).floor() as i32);
        let wide = ((pt.x / Self::WIDE).floor() as i32, (pt.z / Self::WIDE).floor() as i32);
        self.0.get(&key).into_iter().flatten().chain(self.1.get(&wide).into_iter().flatten()).any(|&(j, t)| {
                let j = j as usize;
                if j == i || skip(j, t as usize) {
                    return false;
                }
                let q = &m.parts[j];
                let t = t as usize * 3;
                let [a, b, c] = [q.indices[t], q.indices[t + 1], q.indices[t + 2]].map(|k| Vec3::from(q.positions[k as usize]));
                let d = (b.x - a.x) * (c.z - a.z) - (c.x - a.x) * (b.z - a.z);
                if d.abs() < 1e-9 {
                    return false;
                }
                let u = ((pt.x - a.x) * (c.z - a.z) - (c.x - a.x) * (pt.z - a.z)) / d;
                let w = ((b.x - a.x) * (pt.z - a.z) - (pt.x - a.x) * (b.z - a.z)) / d;
                u >= 0.0 && w >= 0.0 && u + w <= 1.0 && (pt.y - 1.5..=pt.y + above).contains(&(a.y + u * (b.y - a.y) + w * (c.y - a.y)))
        })
    }
}

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

/// A part of the ground, by its shape: mostly facing up, or a low bank that is shallow for its width. The stored normals
/// cannot tell: they all point straight up in the worlds measured, walls included. The ground casts no sun shadow in this
/// build: with the sun low, grass a little above a road laid straight-edged shadows along every path (user's F8 views 19
/// and 20). Walls, cliffs and steep roofs still cast (user's report 5).
fn is_ground(p: &neopets_data::model::MeshPart) -> bool {
    let (mut up, mut all) = (0.0, 0.0);
    for t in p.indices.chunks_exact(3) {
        let [a, b, c] = [t[0], t[1], t[2]].map(|v| Vec3::from(p.positions[v as usize]));
        let n = (b - a).cross(c - a);
        up += n.y.abs();
        all += n.length();
    }
    if all <= 0.0 {
        return false;
    }
    let share = up / all;
    let lo = p.positions.iter().fold([f32::MAX; 3], |a, q| std::array::from_fn(|k| a[k].min(q[k])));
    let hi = p.positions.iter().fold([f32::MIN; 3], |a, q| std::array::from_fn(|k| a[k].max(q[k])));
    let width = (hi[0] - lo[0]).max(hi[2] - lo[2]);
    share >= 0.8 || (share >= 0.3 && hi[1] - lo[1] < 0.2 * width)
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
    overlays: &mut Overlays,
    ext_materials: &mut Assets<OverlayMaterial>,
) -> usize {
    let m = &*b.model;
    let no_pass1 = std::env::var_os("NEOPETS_NO_PASS1").is_some();
    // NEOPETS_NO_MERGE: every part keeps its own entity, as before (diagnostic).
    let no_merge = std::env::var_os("NEOPETS_NO_MERGE").is_some();
    let mut env_textures: HashMap<usize, Handle<Image>> = HashMap::new();
    // Each group's parts, whole or (Some) only some of their triangles.
    #[allow(clippy::type_complexity)]
    let mut groups: HashMap<(AssetId<StandardMaterial>, Option<u8>, i32, i32, (bool, bool)), (Handle<StandardMaterial>, Vec<(usize, Option<Vec<u32>>)>)> = HashMap::new();
    // Parts that are not ground for `ground_under`: soft or fading ones (their blended draws write
    // no depth).
    let soft_or_fading: Vec<bool> = m.parts.iter().map(|q| q.texture.is_some_and(|t| b.soft[t]) || crate::vertex_fades(q)).collect();
    let not_ground = |j: usize| soft_or_fading[j];
    // NEOPETS_NO_OVERLAY: flat parts over other ground stay in the depth prepass (diagnostic).
    let no_overlay = std::env::var_os("NEOPETS_NO_OVERLAY");
    // Each part's triangles already drawn outside the prepass (empty: none).
    let mut overlaid: Vec<Vec<bool>> = vec![Vec::new(); m.parts.len()];
    let built_at = std::time::Instant::now();
    let tris = TriGrid::new(m);
    if std::env::var_os("NEOPETS_TRACE_LAYER").is_some() {
        println!("TRIGRID {pack} {:.1} ms, {} cells, {} entries, {} wide", built_at.elapsed().as_secs_f64() * 1000.0, tris.0.len(), tris.0.values().map(Vec::len).sum::<usize>(), tris.1.values().map(Vec::len).sum::<usize>());
    }
    let mut count = 0;
    let ranks = crate::draw_order::ranks(m);
    let rank = |i: usize, second: u32| crate::draw_order::DrawRank { pack: Some(pack), rank: ranks[i] + second };
    // NEOPETS_TRACE_FLAT: each part's mean normal height, flat share of its area and bounds (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_FLAT").is_some() {
        for (i, p) in m.parts.iter().enumerate() {
            let n = p.normals.len().max(1) as f32;
            let mean = p.normals.iter().map(|q| q[1]).sum::<f32>() / n;
            let (mut up, mut all) = (0.0, 0.0);
            for t in p.indices.chunks_exact(3) {
                let [a, b, c] = [t[0], t[1], t[2]].map(|v| Vec3::from(p.positions[v as usize]));
                let cr = (b - a).cross(c - a);
                up += cr.y.abs();
                all += cr.length();
            }
            let lo = p.positions.iter().fold([f32::MAX; 3], |a, q| std::array::from_fn(|k| a[k].min(q[k])));
            let hi = p.positions.iter().fold([f32::MIN; 3], |a, q| std::array::from_fn(|k| a[k].max(q[k])));
            println!("FLAT {pack} part {i} tex {:?} mean {mean:.3} up {:.3} area {:.0} lo {:.0},{:.0},{:.0} hi {:.0},{:.0},{:.0}", p.texture, up / f32::max(all, 1e-6), all / 2.0, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }
    // NEOPETS_TRACE_COVER="x,z;x,z;...": for each point, the parts with a triangle over it and
    // the height there (diagnostic, for holes in the level mesh).
    if let Ok(v) = std::env::var("NEOPETS_TRACE_COVER") {
        for pt in v.split(';').filter_map(|e| e.split_once(',')).filter_map(|(a, b)| Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?))) {
            let mut hits = Vec::new();
            for (i, p) in m.parts.iter().enumerate() {
                for t in p.indices.chunks_exact(3) {
                    let [a, b, c] = [t[0], t[1], t[2]].map(|k| p.positions[k as usize]);
                    let d = (b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2]);
                    if d.abs() < 1e-9 {
                        continue;
                    }
                    let u = ((pt.0 - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (pt.1 - a[2])) / d;
                    let w = ((b[0] - a[0]) * (pt.1 - a[2]) - (pt.0 - a[0]) * (b[2] - a[2])) / d;
                    if u >= 0.0 && w >= 0.0 && u + w <= 1.0 {
                        let y = a[1] + u * (b[1] - a[1]) + w * (c[1] - a[1]);
                        hits.push(format!("part {i} tex {:?} y {y:.2}", p.texture));
                        break;
                    }
                }
            }
            println!("COVER {pack} ({:.1}, {:.1}): {}", pt.0, pt.1, hits.join("; "));
        }
    }
    // NEOPETS_TRACE_GAP="pack:i": the parts with a vertex nearest that part's vertices, and how
    // near (diagnostic, for pieces that look set apart).
    if let Some((pk, pi)) = std::env::var("NEOPETS_TRACE_GAP").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<usize>().ok()?)))) {
        if let (true, Some(p)) = (pk == pack, m.parts.get(pi)) {
            let mut near: Vec<(f32, usize)> = m
                .parts
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != pi)
                .map(|(j, q)| (p.positions.iter().flat_map(|a| q.positions.iter().map(move |b| Vec3::from(*a).distance(Vec3::from(*b)))).fold(f32::MAX, f32::min), j))
                .collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            let list: Vec<String> = near.iter().take(8).map(|(d, j)| format!("{j} (tex {:?}) {d:.2}", m.parts[*j].texture)).collect();
            println!("GAP {pack}:{pi} nearest: {}", list.join(", "));
            let pts: Vec<String> = p.positions.iter().take(12).map(|v| format!("({:.1},{:.1},{:.1})", v[0], v[1], v[2])).collect();
            println!("GAP {pack}:{pi} vertices: {}", pts.join(" "));
        }
    }
    // NEOPETS_TRACE_PART="pack:i": one part's material facts (diagnostic).
    if let Some((pk, pi)) = std::env::var("NEOPETS_TRACE_PART").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<usize>().ok()?)))) {
        if pk == pack {
            if let Some(p) = m.parts.get(pi) {
                let ta = p.texture.and_then(|t| m.textures.get(t)).map(|t| {
                    let a: Vec<u8> = t.image.rgba.chunks_exact(4).map(|q| q[3]).collect();
                    (a.iter().copied().min(), a.iter().copied().max(), a.iter().map(|&x| x as u32).sum::<u32>() / a.len().max(1) as u32)
                });
                let ca: Vec<[u8; 4]> = p.raw_colors.iter().take(6).copied().collect();
                println!("PART {pack}:{pi} tex {:?} alpha reg {:?} scroll {:?} pass {} pass1 {} env {} lit {} texture alpha min/max/mean {ta:?} vertex colours {ca:?}", p.texture, p.alpha.map(|a| format!("{a:#x}")), p.scroll, p.pass, p.pass1.is_some(), p.env_map, p.lit);
            }
        }
    }
    // NEOPETS_HIDE_PART="pack:i,i,...": those parts are not spawned (diagnostic).
    let hidden: Vec<usize> = std::env::var("NEOPETS_HIDE_PART").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| (a.trim().parse::<u32>().ok()? == pack).then(|| b.split(',').filter_map(|x| x.trim().parse().ok()).collect()))).unwrap_or_default();
    for (i, p) in m.parts.iter().enumerate() {
        if hidden.contains(&i) {
            continue;
        }
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
            // A flat part laid over other ground a little below it (a path over the grass): drawn
            // outside the depth prepass, so ambient occlusion sees the ground under it and draws
            // no crease along its edge (user report 124: the 909 path 215 lies 0.07 over the grass
            // 211, and the soft strip over that edge showed the crease as a line).
            // Done by triangle: a path can run past the ground under it (the 691 path 59 of user
            // report 127), and only its triangles with ground under them leave the prepass. Ground
            // level with it counts (within 0.02), but not a triangle already drawn this way, so two
            // level triangles cannot both leave the prepass with nothing in it under them.
            let started = std::time::Instant::now();
            let over = if no_overlay.is_none() && lies_flat(p) { ground_under(m, &tris, i, &|j, t| not_ground(j) || overlaid[j].get(t).copied().unwrap_or(false), 0.02) } else { Vec::new() };
            let count_over = over.iter().filter(|&&o| o).count();
            overlaid[i] = over.clone();
            if std::env::var_os("NEOPETS_TRACE_LAYER").is_some() && (count_over > 0 || started.elapsed().as_secs_f64() > 0.005) {
                println!("OVERLAY {pack} part {i} tex {:?} {count_over} of {} triangles, {:.2} ms", p.texture, p.indices.len() / 3, started.elapsed().as_secs_f64() * 1000.0);
            }
            let key = |overlay: bool| (material.id(), p.zone, (c[0] / CELL).floor() as i32, (c[1] / CELL).floor() as i32, (is_ground(p), overlay));
            if count_over == 0 || count_over == over.len() {
                groups.entry(key(count_over > 0)).or_insert_with(|| (material.clone(), Vec::new())).1.push((i, None));
            } else {
                for side in [true, false] {
                    let indices: Vec<u32> = p.indices.chunks_exact(3).zip(&over).filter(|(_, o)| **o == side).flat_map(|(t, _)| t.iter().copied()).collect();
                    groups.entry(key(side)).or_insert_with(|| (material.clone(), Vec::new())).1.push((i, Some(indices)));
                }
            }
            continue;
        }
        // SPEC-0003 TEST: a pixel at or above 120 of 128 writes depth, a softer one only blends. A fading part's triangles
        // solid at all three corners draw as solid, so that what lies behind them stays hidden (user's F8 view 16); the rest
        // blend. Done by triangle, as Bevy's prepass does not see vertex alpha (see `vertex_fades`).
        // A flat fading strip (a path edge) is drawn only blended, with no solid core, so ambient
        // occlusion draws no crease along it (user's F8 view 20). A flat part that is soft only
        // through its texture keeps the cut core of SPEC-0003: without it the 758 street wrote no
        // depth and the water drawn after it showed through (user report 115).
        // NEOPETS_SOFT_LAYER brings back the layer for those too (diagnostic).
        let layer = lies_flat(p) && (fades || (soft && std::env::var_os("NEOPETS_SOFT_LAYER").is_some()));
        // A flat part soft only through its texture keeps its cut core (it hides what lies under it:
        // the 758 water, report 115; the hairline gaps between the 934 grass and path, report 128),
        // drawn outside the depth prepass so that ambient occlusion draws no crease along it (user
        // reports 124 and 125). NEOPETS_CORE_PREPASS: such cores in the prepass, as before
        // (diagnostic).
        let core_out = soft && !layer && lies_flat(p) && std::env::var_os("NEOPETS_CORE_PREPASS").is_none();
        // NEOPETS_TRACE_LAYER: each flat part drawn as a layer or with its core outside the
        // prepass (diagnostic).
        if (layer || core_out) && std::env::var_os("NEOPETS_TRACE_LAYER").is_some() {
            println!("LAYER {pack} part {i} tex {:?} fades {fades} soft {soft} core outside {core_out}", p.texture);
        }
        let split = (fades && !layer && !scrolling && env.is_none()).then(|| split_fading(p)).filter(|(solid, _)| !solid.is_empty());
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
                // Flat ground casts no sun shadow (see `is_ground`), its blended layer included: since
                // a flat part soft only through its texture keeps its cut core (report 115), the
                // torn grass strips along paths were no layer and their blended draw cast a dark line
                // along every path edge with World shadows on (user report 123).
                if layer || is_ground(p) {
                    e.insert(bevy::light::NotShadowCaster);
                }
                count += 1;
                if let Some(z) = p.zone {
                    e.insert(ZoneGate(pack, z));
                }
                if let Some((at, d1, _)) = p.cull {
                    e.insert(DrawCull(Vec3::from(at), d1));
                }
                // A ground layer has no cut core: the core would enter the depth prepass.
                if layer {
                    continue;
                }
            }
        }
        // SPEC-0003 TEST for a scrolling part (a waterfall, a water surface): its texels at or
        // above 120 of 128 write depth in the original, so what is drawn after it and lies behind
        // it stays hidden; the blended draw below writes none. A cut core that scrolls with it
        // writes that depth (user report 105: the 1335 waterfalls showed the plants and lattices
        // behind them in front).
        if let (true, None, Some(t), Some(speed)) = (scrolling, env, p.texture, p.scroll) {
            if std::env::var_os("NEOPETS_NO_SCROLL_CORE").is_none() && m.textures[t].image.rgba.chunks_exact(4).any(|q| q[3] >= 240) {
                let core = materials.add(StandardMaterial { base_color_texture: Some(images.add(model_texture(&m.textures[t]))), unlit: true, alpha_mode: AlphaMode::Mask(120.0 / 128.0), cull_mode: None, ..default() });
                let mut c = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(core), Transform::IDENTITY, part, b.sigs[i], UvScroll(speed), bevy::light::NotShadowCaster));
                count += 1;
                if let Some(z) = p.zone {
                    c.insert(ZoneGate(pack, z));
                }
                if let Some((at, d1, _)) = p.cull {
                    c.insert(DrawCull(Vec3::from(at), d1));
                }
            }
        }
        let mut e = match env {
            // An environment-mapped pass 0 is drawn by the pass-1 shader with view-generated UVs
            // (pass1.rs), its texture shared per model texture.
            Some(t) => {
                let image = env_textures.entry(t).or_insert_with(|| images.add(model_texture(&m.textures[t]))).clone();
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(pass1_materials.add(pass1::env_base_for(image, p.alpha))), Transform::IDENTITY, part))
            }
            None if scrolling => {
                let own = materials.add(StandardMaterial { base_color_texture: Some(images.add(model_texture(&m.textures[p.texture.unwrap_or(0)]))), unlit: true, alpha_mode: AlphaMode::Blend, cull_mode: None, ..default() });
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(own), Transform::IDENTITY, part))
            }
            None if core_out => commands.spawn((Mesh3d(mesh), MeshMaterial3d(overlays.of(&material, materials, ext_materials)), Transform::IDENTITY, part)),
            None => commands.spawn((Mesh3d(mesh), MeshMaterial3d(material.clone()), Transform::IDENTITY, part)),
        };
        count += 1;
        if let Some(z) = p.zone {
            e.insert(ZoneGate(pack, z));
        }
        e.insert((b.sigs[i], rank(i, 0)));
        if layer || is_ground(p) {
            e.insert(bevy::light::NotShadowCaster);
        }
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
    for ((_, zone, _, _, (flat, overlay)), (material, list)) in groups {
        let pieces: Vec<std::borrow::Cow<neopets_data::model::MeshPart>> = list
            .into_iter()
            .map(|(i, indices)| match indices {
                Some(indices) => std::borrow::Cow::Owned(neopets_data::model::MeshPart { indices, ..m.parts[i].clone() }),
                None => std::borrow::Cow::Borrowed(&m.parts[i]),
            })
            .collect();
        let parts: Vec<&neopets_data::model::MeshPart> = pieces.iter().map(|c| c.as_ref()).collect();
        let mesh = Mesh3d(meshes.add(merged_mesh(&parts)));
        let mut e = if overlay {
            let ext = overlays.of(&material, materials, ext_materials);
            commands.spawn((mesh, MeshMaterial3d(ext), Transform::IDENTITY, part))
        } else {
            commands.spawn((mesh, MeshMaterial3d(material), Transform::IDENTITY, part))
        };
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
