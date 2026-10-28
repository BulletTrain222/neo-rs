//! Where node trees live inside a resource's chunk chain (SPEC-0004, "Where it lives on the
//! disc"): standalone type-13/12 chunks, the children of type-8 mesh containers, type-3 level
//! display trees, and the same pieces nested inside type-18 model-set wrappers.
use crate::resource::{self, Chunk};

pub const MESH_CONTAINER: u16 = 8;
pub const MODEL_SET: u16 = 18;
pub const LEVEL_TREE: u16 = 3;
pub const TREE_13: u16 = 13;
pub const TREE_12: u16 = 12;
/// Nesting on disc is shallow; this bounds hostile input.
const MAX_DEPTH: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRef {
    /// Byte range of the tree's payload inside the resource.
    pub payload: std::ops::Range<usize>,
    /// Chunk type that holds the tree (13, 12 or 3).
    pub kind: u16,
    /// Role id (header bytes 14–15) of a mesh-container child or standalone chunk.
    pub role_id: u16,
    /// Nesting depth: 0 for top-level chunks.
    pub depth: usize,
    /// Index of the chain (0 = top level, then each nested chain in walk order) whose shader
    /// list governs the tree's materials.
    pub chain: usize,
}

/// A chunk chain and where it sits in the resource; trees name the chain whose shader list
/// and texture chunks they use.
#[derive(Clone, Debug)]
pub struct ChainRef {
    pub base: usize,
    pub chunks: Vec<Chunk>,
    pub depth: usize,
}

/// Every node tree in a chunk chain, recursing into mesh containers and model-set wrappers.
pub fn find_trees(bytes: &[u8], top: &[Chunk]) -> (Vec<TreeRef>, Vec<ChainRef>) {
    let mut trees = Vec::new();
    let mut chains = vec![ChainRef {
        base: 0,
        chunks: top.to_vec(),
        depth: 0,
    }];
    visit(bytes, 0, top, 0, 0, &mut trees, &mut chains);
    (trees, chains)
}

fn visit(
    bytes: &[u8],
    base: usize,
    chain: &[Chunk],
    depth: usize,
    chain_index: usize,
    trees: &mut Vec<TreeRef>,
    chains: &mut Vec<ChainRef>,
) {
    for c in chain {
        let payload = base + c.payload.start..base + c.payload.end;
        match c.kind {
            TREE_13 | TREE_12 | LEVEL_TREE => trees.push(TreeRef {
                payload,
                kind: c.kind,
                role_id: c.role_id,
                depth,
                chain: chain_index,
            }),
            // Container children share the enclosing chain's shader list.
            MESH_CONTAINER if depth < MAX_DEPTH => {
                if let Ok(children) = resource::walk_chunks(&bytes[payload.clone()], None) {
                    visit(bytes, payload.start, &children.chunks, depth + 1, chain_index, trees, chains);
                }
            }
            // A model set is its own chain with its own shader list and textures.
            MODEL_SET if depth < MAX_DEPTH && bytes[payload.clone()].starts_with(b"IDM") => {
                if let Ok(children) = resource::walk_chunks(&bytes[payload.clone()], None) {
                    let index = chains.len();
                    chains.push(ChainRef {
                        base: payload.start,
                        chunks: children.chunks.clone(),
                        depth: depth + 1,
                    });
                    visit(bytes, payload.start, &children.chunks, depth + 1, index, trees, chains);
                }
            }
            _ => {}
        }
    }
}

use crate::gsmem::GsMemory;
use crate::material::{self, ShaderList, TextureRef};
use crate::mesh::{self, Layout};
use crate::texture::{Image, TextureChunk};
use std::collections::HashMap;

/// A decoded texture at its material's sampling size, with its wrap modes.
#[derive(Clone, Debug)]
pub struct ModelTexture {
    pub image: Image,
    pub repeat_u: bool,
    pub repeat_v: bool,
}

/// One mesh leaf as an indexed triangle list (pass 0 only).
#[derive(Clone, Debug)]
pub struct MeshPart {
    pub tree: usize,
    pub leaf_offset: usize,
    pub layout: Layout,
    pub group: Option<u16>,
    pub program: Option<u32>,
    /// The group's character colour slot (0 for no group).
    pub colour_slot: u8,
    /// The zone column that gates this part: its leaf's enclosing type-3 node, else the
    /// instance's or placed object's column (None draws always).
    pub zone: Option<u8>,
    /// Pass 0's UV scroll speeds (UV mode 2), when it scrolls.
    pub scroll: Option<[f32; 2]>,
    /// Pass 0's UV mode is 1, the environment map (SPEC-0003 UV modes): its UVs are generated
    /// from the view each frame, not the stored ones.
    pub env_map: bool,
    /// Pass 0's variant-1 flipbook (a variant-2 group always shows cell 0, its mesh UVs as
    /// stored; SPEC-0003 Sub-texture frame addressing).
    pub flipbook: Option<material::Flipbook>,
    pub positions: Vec<[f32; 3]>,
    /// Layout B has no normals; its parts get +Y placeholders (it is drawn unlit).
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Texture modulation per vertex, 1.0 = unchanged, in the GS's gamma space: the vertex
    /// program's output / 128. Lit parts are lit with the scene default lights in the space they
    /// are built in (world space for world scenes).
    pub colors: Vec<[f32; 4]>,
    /// The stored colour bytes, for relighting moving lit parts (skinned characters).
    pub raw_colors: Vec<[u8; 4]>,
    /// Drawn by a lit program (1, 2 or 6).
    pub lit: bool,
    pub indices: Vec<u32>,
    pub texture: Option<usize>,
    /// Layout A only: three skin weights and three skin bone indices (slot byte / 3) per vertex.
    pub weights: Vec<[f32; 3]>,
    pub bones: Vec<[u16; 3]>,
    /// The second render pass of a two-pass group (SPEC-0003 How a pass is drawn; SPEC-0004
    /// Two-pass batches): the same positions, colours and normals with its own texture and UVs.
    pub pass1: Option<Pass1>,
}

/// A part's render pass 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Pass1 {
    pub texture: usize,
    /// The pass-1 UVs (the second half of each batch's UV array).
    pub uvs: Vec<[f32; 2]>,
    /// UV mode: 0 none, 1 environment map, 2 scroll, 3 flipbook.
    pub uv_mode: u32,
    pub scroll: [f32; 2],
    /// The second register set's ALPHA (fields A, B, C, D in bits 0 to 7, FIX in 32 to 39).
    pub alpha: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct ModelData {
    pub parts: Vec<MeshPart>,
    pub textures: Vec<ModelTexture>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Leaves not drawn: other packet kinds (CPU-animated water) or decode errors.
    pub skipped_leaves: Vec<String>,
    /// The first tree's first distance-cull node fields (+0x50, +0x54, +0x58, +0x64).
    pub lod: Option<[f32; 4]>,
    /// Trees added (including ones whose leaves were all skipped).
    pub trees: usize,
    /// World placements whose references do not resolve (stale data on disc).
    pub unresolved_placements: usize,
    /// Built from a world pack's static scene (world space).
    pub world: bool,
    /// A character model set's attachment models by slot (children with role 10 + k; SPEC-0046
    /// Second walk and attachment slots), kept out of the body.
    pub attachments: Vec<(u8, ModelData)>,
}

impl ModelData {
    /// Add another model's parts and textures (for example the sky dome under a world).
    pub fn append(&mut self, mut other: ModelData) {
        let base = self.textures.len();
        for p in &mut other.parts {
            p.texture = p.texture.map(|t| t + base);
            p.tree += self.trees;
        }
        if !other.parts.is_empty() {
            for k in 0..3 {
                self.min[k] = self.min[k].min(other.min[k]);
                self.max[k] = self.max[k].max(other.max[k]);
            }
        }
        self.trees += other.trees;
        self.parts.extend(other.parts);
        self.textures.extend(other.textures);
        self.skipped_leaves.extend(other.skipped_leaves);
    }

    /// A box around where most of the geometry is: the median vertex position, extended by
    /// the distance that covers `share` of the vertices. Level meshes reach far beyond their
    /// playable area, so the full box would frame mostly empty space.
    pub fn dense_box(&self, share: f32) -> ([f32; 3], [f32; 3]) {
        let total: usize = self.parts.iter().map(|p| p.positions.len()).sum();
        if total == 0 {
            return (self.min, self.max);
        }
        let step = (total / 20_000).max(1);
        let sample: Vec<[f32; 3]> = self
            .parts
            .iter()
            .flat_map(|p| p.positions.iter().copied())
            .step_by(step)
            .collect();
        let median = |k: usize| {
            let mut v: Vec<f32> = sample.iter().map(|p| p[k]).collect();
            v.sort_by(f32::total_cmp);
            v[v.len() / 2]
        };
        let c = [median(0), median(1), median(2)];
        let mut d: Vec<f32> = sample
            .iter()
            .map(|p| ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt())
            .collect();
        d.sort_by(f32::total_cmp);
        let r = d[((d.len() - 1) as f32 * share.clamp(0.0, 1.0)) as usize];
        (c.map(|v| v - r), c.map(|v| v + r))
    }
}

/// The shader list and texture chunks that a chain's trees draw with.
struct ChainMaterials {
    list: ShaderList,
    chunks: Vec<TextureChunk>,
    memories: Vec<Option<GsMemory>>,
}

impl ChainMaterials {
    fn of(bytes: &[u8], chunks: &[Chunk]) -> Result<Option<Self>, String> {
        let lists = material::lists_in_chain(bytes, chunks)?;
        Ok(lists.into_iter().find_map(|l| {
            let list = l.list?;
            let n = l.chunks.len();
            Some(Self {
                list,
                chunks: l.chunks,
                memories: (0..n).map(|_| None).collect(),
            })
        }))
    }
}

/// GS output for a colour byte at a light level of 128 (lit programs: 16 means 1.0) or for an
/// unlit program (128 means 1.0); the GS then modulates by output / 128.
fn colour_factor(byte: u8, unit: f32) -> f32 {
    (byte as f32 * 128.0 / unit).min(255.0) / 128.0
}

/// Texture modulation for each vertex of a part: lit parts through the scene default lights on
/// the part's normals, unlit parts by their colour bytes.
pub fn vertex_colours(part: &MeshPart, unit: f32) -> Vec<[f32; 4]> {
    if part.lit {
        lit_colours(&part.raw_colors, &part.normals, &crate::light::Lights::scene_default())
    } else {
        part.raw_colors.iter().map(|c| c.map(|v| colour_factor(v, unit))).collect()
    }
}

/// Lit modulation (output / 128) for colour bytes and normals in the lights' space.
pub fn lit_colours(colours: &[[u8; 4]], normals: &[[f32; 3]], lights: &crate::light::Lights) -> Vec<[f32; 4]> {
    colours
        .iter()
        .zip(normals)
        .map(|(c, n)| lights.vertex(*n, *c).map(|v| v as f32 / 128.0))
        .collect()
}

/// Turns node trees into mesh parts, resolving each leaf's material against its chain's
/// shader list and decoding each texture once.
struct Builder {
    materials: Vec<Option<ChainMaterials>>,
    shared: Option<ChainMaterials>,
    cache: HashMap<(usize, TextureRef, u8, u8), usize>,
    out: ModelData,
}

impl Builder {
    fn new(materials: Vec<Option<ChainMaterials>>, shared: Option<ChainMaterials>) -> Self {
        Self {
            materials,
            shared,
            cache: HashMap::new(),
            out: ModelData {
                min: [f32::INFINITY; 3],
                max: [f32::NEG_INFINITY; 3],
                ..Default::default()
            },
        }
    }

    /// Add one tree, drawn with chain `chain`'s materials and placed by `matrix`.
    fn add_tree(
        &mut self,
        bytes: &[u8],
        payload: std::ops::Range<usize>,
        chain: usize,
        matrix: Option<&crate::world::Matrix>,
        layer_mask: Option<u32>,
        zone: Option<u8>,
    ) -> Result<(), String> {
        let ti = self.out.trees;
        self.out.trees += 1;
        let payload = &bytes[payload];
        let tree = mesh::walk_tree_layers(payload, layer_mask)?;
        if self.out.lod.is_none() {
            self.out.lod = tree.lod;
        }
        for leaf in &tree.leaves {
            let batches = match mesh::decode_packet(payload, leaf) {
                Ok(b) => b,
                Err(e) => {
                    self.out.skipped_leaves.push(format!("tree {ti} leaf {:#x}: {e}", leaf.offset));
                    continue;
                }
            };
            let Some(first) = batches.first() else { continue };
            // Trees in a chain without a shader list use the shared list, keyed apart.
            let (mats, key) = match self.materials.get_mut(chain).and_then(|m| m.as_mut()) {
                Some(m) => (Some(m), chain),
                None => (self.shared.as_mut(), usize::MAX),
            };
            let group = mats
                .as_ref()
                .and_then(|m| m.list.groups.get(leaf.material_group as usize).copied());
            let unit = group.map_or(128.0, |g| g.colour_unit());
            let mut texture = None;
            let mut texture1 = None;
            if let (Some(m), Some(g)) = (mats, group) {
                // Render pass 1's texture, cached apart from pass 0's (its wrap may differ).
                if g.two_pass() {
                    if let (Some(r), Some(t1)) = (material::resolve_pass1(&g, &m.chunks), g.tex0_pass1) {
                        let k = (key | 1 << 60, r, t1.tw, t1.th);
                        texture1 = match self.cache.get(&k) {
                            Some(&i) => Some(i),
                            None => {
                                let mem = m.memories[r.chunk].get_or_insert_with(|| m.chunks[r.chunk].gs_memory());
                                let (w, h) = (1u32 << t1.tw.min(10), 1u32 << t1.th.min(10));
                                match m.chunks[r.chunk].decode_region(mem, r.texture, r.palette, w, h) {
                                    Ok(image) => {
                                        let (repeat_u, repeat_v) = g.repeats_pass1();
                                        self.out.textures.push(ModelTexture { image, repeat_u, repeat_v });
                                        self.cache.insert(k, self.out.textures.len() - 1);
                                        Some(self.out.textures.len() - 1)
                                    }
                                    Err(_) => None,
                                }
                            }
                        };
                    }
                }
                if let (Some(r), Some(t0)) = (material::resolve(&g, &m.chunks), g.tex0) {
                    let k = (key, r, t0.tw, t0.th);
                    texture = match self.cache.get(&k) {
                        Some(&i) => Some(i),
                        None => {
                            let mem = m.memories[r.chunk]
                                .get_or_insert_with(|| m.chunks[r.chunk].gs_memory());
                            let (w, h) = (1u32 << t0.tw.min(10), 1u32 << t0.th.min(10));
                            let image =
                                m.chunks[r.chunk].decode_region(mem, r.texture, r.palette, w, h)?;
                            let (repeat_u, repeat_v) = g.repeats();
                            self.out.textures.push(ModelTexture { image, repeat_u, repeat_v });
                            self.cache.insert(k, self.out.textures.len() - 1);
                            Some(self.out.textures.len() - 1)
                        }
                    };
                }
            }
            let mut part = MeshPart {
                tree: ti,
                leaf_offset: leaf.offset,
                layout: first.layout,
                group: group.map(|_| leaf.material_group),
                program: group.map(|g| g.program),
                colour_slot: group.map_or(0, |g| g.colour_slot),
                zone: leaf.zone.or(zone),
                scroll: group.filter(|g| g.uv_mode == 2 && g.uv_speed() != [0.0; 2]).map(|g| g.uv_speed()),
                env_map: group.is_some_and(|g| g.uv_mode == 1),
                flipbook: group.filter(|g| g.animator == 1 && g.grid()[0] as i32 > 0).and_then(|g| {
                    let t = g.tex0?;
                    let [count, rows, cols, d] = g.grid().map(|f| f as i32);
                    Some(material::Flipbook { count, rows, cols, divisor: d, width: 1 << t.tw.min(10), height: 1 << t.th.min(10) })
                }),
                positions: Vec::new(),
                normals: Vec::new(),
                uvs: Vec::new(),
                colors: Vec::new(),
                indices: Vec::new(),
                texture,
                weights: Vec::new(),
                bones: Vec::new(),
                pass1: None,
                raw_colors: Vec::new(),
                lit: group.is_some_and(|g| matches!(g.program, 1 | 2 | 6)),
            };
            for b in &batches {
                let base = part.positions.len() as u32;
                part.indices.extend(b.triangles().iter().flat_map(|t| t.map(|i| i + base)));
                part.positions.extend_from_slice(&b.positions);
                part.uvs.extend_from_slice(&b.uvs);
                part.raw_colors.extend_from_slice(&b.colors);
                if !b.weights.is_empty() {
                    part.weights.extend_from_slice(&b.weights);
                    part.bones.extend(b.bones.iter().map(|s| [0, 1, 2].map(|k| s[k] as u16 / 3)));
                }
                if b.normals.is_empty() {
                    // Mixed layouts in one leaf do not occur; keep arrays aligned anyway.
                    part.normals.extend(std::iter::repeat_n([0.0, 1.0, 0.0], b.positions.len()));
                } else {
                    part.normals.extend_from_slice(&b.normals);
                }
            }
            // Pass 1 when every batch carries its half of the UV array.
            if let (Some(t), Some(g)) = (texture1, group) {
                if batches.iter().all(|b| b.uvs_pass1.len() == b.positions.len()) {
                    part.pass1 = Some(Pass1 {
                        texture: t,
                        uvs: batches.iter().flat_map(|b| b.uvs_pass1.iter().copied()).collect(),
                        uv_mode: g.uv_mode_pass1,
                        scroll: g.uv_speed_pass1(),
                        alpha: g.alpha_pass1,
                    });
                }
            }
            if let Some(m) = matrix {
                for p in &mut part.positions {
                    *p = crate::world::transform_point(m, *p);
                }
                for n in &mut part.normals {
                    *n = crate::world::transform_normal(m, *n);
                }
            }
            part.colors = vertex_colours(&part, unit);
            for p in &part.positions {
                for k in 0..3 {
                    self.out.min[k] = self.out.min[k].min(p[k]);
                    self.out.max[k] = self.out.max[k].max(p[k]);
                }
            }
            self.out.parts.push(part);
        }
        Ok(())
    }

    fn finish(mut self) -> ModelData {
        if self.out.parts.is_empty() {
            self.out.min = [0.0; 3];
            self.out.max = [0.0; 3];
        }
        self.out
    }
}

/// Build one node tree (a byte range of `bytes`) in its own frame, drawn with the chain's
/// shader list (a placed object's inline mesh).
pub fn build_tree(bytes: &[u8], top: &[Chunk], payload: std::ops::Range<usize>, layer_mask: u32) -> Result<ModelData, String> {
    let materials = vec![ChainMaterials::of(bytes, top)?];
    let mut b = Builder::new(materials, None);
    b.add_tree(bytes, payload, 0, None, Some(layer_mask), None)?;
    Ok(b.finish())
}

/// Build every node tree of a resource into mesh parts. `shared` supplies the shader list for
/// chains without one: resources 544–575 draw with resource 7's list (SPEC-0004).
pub fn build(
    bytes: &[u8],
    top: &[Chunk],
    shared: Option<(&[u8], &[Chunk])>,
) -> Result<ModelData, String> {
    build_roles(bytes, top, shared, None)
}

/// As `build`, keeping only trees whose role id is listed (None keeps all).
pub fn build_roles(
    bytes: &[u8],
    top: &[Chunk],
    shared: Option<(&[u8], &[Chunk])>,
    roles: Option<&[u16]>,
) -> Result<ModelData, String> {
    build_masked(bytes, top, shared, roles, None)
}

/// A region character's draw mask (SPEC-0004 Drawing a node tree, item 3a): bit 30 normally,
/// bit 31 while the region's colour override is on.
pub const REGION_MASK: u32 = 0x4000_0000;
pub const REGION_OVERRIDE_MASK: u32 = 0x8000_0000;

/// As `build_roles`, with a character draw mask: a type-4 node with nonzero layer flags keeps
/// its subtree only when the flags share a bit with the mask (None draws every part).
pub fn build_masked(
    bytes: &[u8],
    top: &[Chunk],
    shared: Option<(&[u8], &[Chunk])>,
    roles: Option<&[u16]>,
    layer_mask: Option<u32>,
) -> Result<ModelData, String> {
    let (trees, chains) = find_trees(bytes, top);
    let materials = chains
        .iter()
        .map(|c| ChainMaterials::of(&bytes[c.base..], &c.chunks))
        .collect::<Result<_, _>>()?;
    let shared = match shared {
        Some((b, chunks)) => ChainMaterials::of(b, chunks)?,
        None => None,
    };
    let mut b = Builder::new(materials, shared);
    for t in trees.iter().filter(|t| roles.is_none_or(|r| r.contains(&t.role_id))) {
        b.add_tree(bytes, t.payload.clone(), t.chain, None, layer_mask, None)?;
    }
    Ok(b.finish())
}

/// Build a world pack's static scene: the level display tree in world space, and placed
/// objects and instances transformed by their matrices, all with the pack's shader list.
/// Trees that do not walk (stale references) are skipped and counted. `layer_mask` is the world
/// layer mask of SPEC-0046 (0 for a new game: mask word 0 and no story layer visible).
pub fn build_world(bytes: &[u8], top: &[Chunk], layer_mask: u32) -> Result<ModelData, String> {
    build_world_parts(bytes, top, layer_mask, true)
}

/// As `build_world`, leaving out the placed objects (type 29) when `objects` is false (they are
/// drawn on their own with `build_tree`).
pub fn build_world_parts(bytes: &[u8], top: &[Chunk], layer_mask: u32, objects: bool) -> Result<ModelData, String> {
    let mut scene = crate::world::scene(bytes, top);
    if !objects {
        scene.trees.retain(|t| t.source != crate::world::PLACED_OBJECTS);
    }
    let materials = vec![ChainMaterials::of(bytes, top)?];
    let mut b = Builder::new(materials, None);
    for t in &scene.trees {
        if let Err(e) = b.add_tree(bytes, t.payload.clone(), 0, t.matrix.as_ref(), Some(layer_mask), t.zone) {
            b.out.skipped_leaves.push(format!("tree at {:#x}: {e}", t.payload.start));
        }
    }
    b.out.unresolved_placements = scene.unresolved;
    b.out.world = true;
    Ok(b.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: u16, role: u16, payload: &[u8]) -> Vec<u8> {
        let size = 16 + payload.len().next_multiple_of(4);
        let mut c = b"IDM\x01".to_vec();
        c.extend(kind.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend((size as u32).to_le_bytes());
        c.extend([0, 0]);
        c.extend(role.to_le_bytes());
        c.extend(payload);
        c.resize(size, 0);
        c
    }

    /// The 16-byte type-0 header that ends a chain.
    fn end() -> Vec<u8> {
        chunk(0, 0, &[])
    }

    #[test]
    fn colour_scale_follows_the_program() {
        // SPEC-0004: lit byte 16 -> 1.0, 24 -> 1.5; unlit 128 -> 1.0, 64 -> 0.5.
        assert_eq!(colour_factor(16, 16.0), 1.0);
        assert_eq!(colour_factor(24, 16.0), 1.5);
        assert_eq!(colour_factor(128, 128.0), 1.0);
        assert_eq!(colour_factor(64, 128.0), 0.5);
        assert_eq!(colour_factor(255, 16.0), 255.0 / 128.0, "lit output saturates at 255");
    }

    #[test]
    fn finds_trees_in_containers_and_model_sets() {
        let tree = [0xAAu8; 8];
        let mut container = chunk(TREE_13, 10, &tree);
        container.extend(chunk(TREE_12, 11, &tree));
        container.extend(end());
        let mut set = chunk(MESH_CONTAINER, 0, &{
            let mut c = chunk(TREE_13, 0, &tree);
            c.extend(end());
            c
        });
        set.extend(end());
        let mut file = chunk(2, 0, &[0; 4]);
        file.extend(chunk(MESH_CONTAINER, 0, &container));
        file.extend(chunk(MODEL_SET, 0, &set));
        file.extend(chunk(LEVEL_TREE, 0, &tree));
        file.extend(end());
        let top = resource::walk_chunks(&file, None).unwrap();
        let (trees, chains) = find_trees(&file, &top.chunks);
        let summary: Vec<_> = trees.iter().map(|t| (t.kind, t.role_id, t.depth, t.chain)).collect();
        assert_eq!(
            summary,
            [(13, 10, 1, 0), (12, 11, 1, 0), (13, 0, 2, 1), (3, 0, 0, 0)],
            "container children use the top chain; the model set is chain 1"
        );
        assert_eq!(chains.len(), 2);
        assert!(trees.iter().all(|t| file[t.payload.clone()].starts_with(&tree)));
    }
}
