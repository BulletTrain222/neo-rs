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
    pub positions: Vec<[f32; 3]>,
    /// Layout B has no normals; its parts get +Y placeholders (it is drawn unlit).
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Texture modulation per vertex, 1.0 = unchanged, in the GS's gamma space. Lit programs
    /// assume a light level of 1.0 until lighting (SPEC-0046) is implemented.
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub texture: Option<usize>,
    /// Layout A only: three skin weights and three skin bone indices (slot byte / 3) per vertex.
    pub weights: Vec<[f32; 3]>,
    pub bones: Vec<[u16; 3]>,
}

#[derive(Clone, Debug, Default)]
pub struct ModelData {
    pub parts: Vec<MeshPart>,
    pub textures: Vec<ModelTexture>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Leaves not drawn: other packet kinds (CPU-animated water) or decode errors.
    pub skipped_leaves: Vec<String>,
    /// Trees added (including ones whose leaves were all skipped).
    pub trees: usize,
    /// World placements whose references do not resolve (stale data on disc).
    pub unresolved_placements: usize,
    /// Built from a world pack's static scene (world space).
    pub world: bool,
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

/// GS output for a colour byte: lit programs scale 16 to 1.0 times the light level (taken
/// as 128 here), unlit programs use the byte as is; the GS then modulates by output / 128.
fn colour_factor(byte: u8, unit: f32) -> f32 {
    (byte as f32 * 128.0 / unit).min(255.0) / 128.0
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
    ) -> Result<(), String> {
        let ti = self.out.trees;
        self.out.trees += 1;
        let payload = &bytes[payload];
        let tree = mesh::walk_tree_layers(payload, layer_mask)?;
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
            if let (Some(m), Some(g)) = (mats, group) {
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
                positions: Vec::new(),
                normals: Vec::new(),
                uvs: Vec::new(),
                colors: Vec::new(),
                indices: Vec::new(),
                texture,
                weights: Vec::new(),
                bones: Vec::new(),
            };
            for b in &batches {
                let base = part.positions.len() as u32;
                part.indices.extend(b.triangles().iter().flat_map(|t| t.map(|i| i + base)));
                part.positions.extend_from_slice(&b.positions);
                part.uvs.extend_from_slice(&b.uvs);
                part.colors.extend(b.colors.iter().map(|c| c.map(|v| colour_factor(v, unit))));
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
            if let Some(m) = matrix {
                for p in &mut part.positions {
                    *p = crate::world::transform_point(m, *p);
                }
                for n in &mut part.normals {
                    *n = crate::world::transform_normal(m, *n);
                }
            }
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
        b.add_tree(bytes, t.payload.clone(), t.chain, None, None)?;
    }
    Ok(b.finish())
}

/// Build a world pack's static scene: the level display tree in world space, and placed
/// objects and instances transformed by their matrices, all with the pack's shader list.
/// Trees that do not walk (stale references) are skipped and counted. `layer_mask` is the world
/// layer mask of SPEC-0046 (0 for a new game: mask word 0 and no story layer visible).
pub fn build_world(bytes: &[u8], top: &[Chunk], layer_mask: u32) -> Result<ModelData, String> {
    let scene = crate::world::scene(bytes, top);
    let materials = vec![ChainMaterials::of(bytes, top)?];
    let mut b = Builder::new(materials, None);
    for t in &scene.trees {
        if let Err(e) = b.add_tree(bytes, t.payload.clone(), 0, t.matrix.as_ref(), Some(layer_mask)) {
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
