//! Character skinning and lighting on the GPU (a performance feature of this build).
//!
//! The CPU path (`character::skin_part`, `relight`, `tinted` in main.rs) rebuilt every drawn
//! character's meshes on each game tick, positions, normals and lit colours, and Bevy then freed,
//! reallocated and uploaded each one (about 21 meshes a frame in the user's views). Here a
//! character's skinned parts keep their rest vertices and Bevy's own skinning draws them: each
//! character has joint entities whose world matrices are written from the game's skin matrices
//! (`GlobalTransform` set directly, so a matrix keeps any shear), and Bevy uploads only those
//! matrices. The rules stay the CPU path's (SPEC-0004 skinning, SPEC-0046 lit draw):
//! - position: the weighted sum of the three bones' transforms of the rest position;
//! - normal: turned by the first bone only, normalised;
//! - colour: lit parts from the light block (ambient plus three directional lights, RGB =
//!   min(255, L x byte / 16) and alpha = min(255, 128 x alpha byte / 16), truncated, over 128),
//!   unlit parts from their stored colour; then raised to 2.2 and times the part's tint
//!   ((status tint x slot colour x GS unit) raised to 2.2), and the alpha times a ghost's level.
//!
//! The light block, tint, slot colours, ghost level and the lighting rotation travel as extra
//! "data joints" after the bones (12 numbers each), and a skinned part's second UV set holds the
//! index of its first data joint and its flags (colour slot, lit, textured). Bevy's mesh vertex
//! shader (its embedded `bevy_pbr/render/mesh.wgsl`, MIT/Apache Bevy code, not game data) is
//! replaced by a copy that applies those rules when a mesh is skinned; nothing else in the game
//! uses Bevy's skinning. NEOPETS_CPU_SKIN keeps the CPU path (diagnostic, for comparison).

use bevy::asset::AssetEvent;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;

/// The embedded path of Bevy's mesh vertex shader.
const MESH_PATH: &str = "embedded://bevy_pbr/render/mesh.wgsl";
/// A marker in the replacement source, to tell it from Bevy's.
const MARKER: &str = "neopets gpu skin";
/// Data joints after the bones: D0 lighting rotation and ghost level, D1 and D2 the light
/// block and the status tint, D3 to D6 the fifteen slot colours. One identity joint follows,
/// for a bone index outside the character's table (the CPU path's identity fallback).
pub const DATA_JOINTS: usize = 7;
/// Bevy's joint limit per skin.
const MAX_JOINTS: usize = 256;

/// Whether characters are skinned on the GPU (NEOPETS_CPU_SKIN: the CPU path).
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("NEOPETS_CPU_SKIN").is_none())
}

/// Whether a character with this many bones fits Bevy's joint limit with its data joints.
pub fn fits(bones: usize) -> bool {
    bones + DATA_JOINTS + 1 <= MAX_JOINTS
}

/// A joint entity of a GPU-skinned character (a child of its root, with a world matrix only).
#[derive(Component)]
pub struct SkinJoint;

/// The inverse bind poses every character shares: identities (the joint matrices are the final
/// skin matrices).
#[derive(Resource)]
pub struct SkinAssets {
    pub bindposes: Handle<SkinnedMeshInverseBindposes>,
}

pub struct GpuSkinPlugin;

impl Plugin for GpuSkinPlugin {
    fn build(&self, app: &mut App) {
        let bindposes = app.world_mut().resource_mut::<Assets<SkinnedMeshInverseBindposes>>().add(SkinnedMeshInverseBindposes::from(vec![Mat4::IDENTITY; MAX_JOINTS]));
        app.insert_resource(SkinAssets { bindposes });
        if enabled() {
            app.add_systems(Update, replace_shader);
        }
    }
}

/// When Bevy's mesh vertex shader is loaded (or reloaded), its source is replaced.
fn replace_shader(asset_server: Res<AssetServer>, mut shaders: ResMut<Assets<Shader>>, mut events: MessageReader<AssetEvent<Shader>>, mut handle: Local<Option<Handle<Shader>>>) {
    let h = handle.get_or_insert_with(|| asset_server.load(MESH_PATH)).clone();
    let ours = |s: &Shader| matches!(&s.source, bevy::shader::Source::Wgsl(src) if src.contains(MARKER));
    let touched = events.read().any(|e| matches!(e, AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } if *id == h.id()));
    if !touched {
        return;
    }
    let Some(current) = shaders.get(&h) else { return };
    if ours(current) {
        return;
    }
    let path = current.path.clone();
    let _ = shaders.insert(&h, Shader::from_wgsl(SOURCE.replace("MARKER", MARKER), path));
    info!("gpu skin: Bevy's mesh vertex shader replaced");
}

/// The flags of a part in its second UV set: colour slot (bits 0 to 3), lit (bit 4),
/// textured (bit 5).
fn flags(slot: u8, lit: bool, textured: bool) -> f32 {
    (slot.min(14) as u32 | (lit as u32) << 4 | (textured as u32) << 5) as f32
}

/// The joint indices of a part's vertices (three bones and an unused fourth); a bone outside
/// the table points at the identity joint.
fn joint_indices(bones_of: &[[u16; 3]], bones: usize) -> Vec<[u16; 4]> {
    let identity = (bones + DATA_JOINTS) as u16;
    let index = |b: u16| if (b as usize) < bones { b } else { identity };
    bones_of.iter().map(|b| [index(b[0]), index(b[1]), index(b[2]), 0]).collect()
}

/// Gives a skinned part's mesh what the GPU path reads: its rest positions and normals, the
/// joint indices and weights (a bone outside the table points at the identity joint), the data
/// joints' index and the part's flags, and its colour bytes (lit) or stored colours (unlit).
pub fn prepare_mesh(mesh: &mut Mesh, part: &neopets_data::model::MeshPart, bones: usize) {
    let joints = joint_indices(&part.bones, bones);
    let weights: Vec<[f32; 4]> = part.weights.iter().map(|w| [w[0], w[1], w[2], 0.0]).collect();
    let data = [bones as f32, flags(part.colour_slot, part.lit, part.texture.is_some())];
    let colours: Vec<[f32; 4]> = if part.lit { part.raw_colors.iter().map(|c| c.map(|v| v as f32)).collect() } else { part.colors.clone() };
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, bevy::mesh::VertexAttributeValues::Uint16x4(joints));
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![data; part.positions.len()]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
}

/// What the data joints carry for one character.
pub struct Look<'a> {
    /// The rotation the CPU path lights the normals with (the facing).
    pub light_rotation: Quat,
    pub lights: &'a neopets_data::light::Lights,
    pub tint: [f32; 3],
    pub slots: &'a [[f32; 3]; 15],
    pub ghost: f32,
}

fn joint(c: [[f32; 3]; 4]) -> GlobalTransform {
    GlobalTransform::from(bevy::math::Affine3A::from_cols(Vec3::from(c[0]).into(), Vec3::from(c[1]).into(), Vec3::from(c[2]).into(), Vec3::from(c[3]).into()))
}

/// The world matrices of a character's joints: each bone's skin matrix after the root's world
/// matrix, then the data joints, then the identity joint.
pub fn joint_matrices(root: &bevy::math::Affine3A, matrices: &[neopets_data::anim::Affine], look: &Look) -> Vec<GlobalTransform> {
    let mut out: Vec<GlobalTransform> = matrices
        .iter()
        .map(|m| {
            let local = bevy::math::Affine3A::from_cols(Vec3::from(m.cols[0]).into(), Vec3::from(m.cols[1]).into(), Vec3::from(m.cols[2]).into(), Vec3::from(m.t).into());
            GlobalTransform::from(*root * local)
        })
        .collect();
    // The lighting normal is the light rotation times the bone-turned normal; the shader has
    // the root's linear part already applied, so D0 undoes it first.
    let q = Mat3::from_quat(look.light_rotation) * Mat3::from(root.matrix3).inverse();
    let l = look.lights;
    let [d0, d1, d2] = [l.directional[0], l.directional[1], l.directional[2]];
    out.push(joint([q.x_axis.to_array(), q.y_axis.to_array(), q.z_axis.to_array(), [look.ghost, 0.0, 0.0]]));
    out.push(joint([l.ambient, d0.0, d0.1, d1.0]));
    out.push(joint([d1.1, d2.0, d2.1, look.tint]));
    for k in 0..4 {
        let s = |i: usize| look.slots.get(4 * k + i).copied().unwrap_or([1.0; 3]);
        out.push(joint([s(0), s(1), s(2), s(3)]));
    }
    out.push(GlobalTransform::IDENTITY);
    out
}

/// Bevy 0.19's mesh.wgsl with the skinned branch changed (marked below).
const SOURCE: &str = r#"// MARKER
#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = bevy_pbr::morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph_normal(vertex_index, i, instance_index);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, instance_index), 0.0);
#endif
    }
    return vertex;
}
#endif

#ifdef SKINNED
// neopets: one joint matrix of this mesh's skin.
fn skin_joint(i: u32, instance_index: u32) -> mat4x4<f32> {
#ifdef SKINS_USE_UNIFORM_BUFFERS
    return skinning::joint_matrices.data[i];
#else
    return skinning::joint_matrices[mesh[instance_index].current_skin_index + i];
#endif
}

fn gamma(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(1e-20)), vec3<f32>(2.2));
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    var world_from_local = mesh_world_from_local;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    // neopets: the normal turns with the first bone only (SPEC-0004).
    let bone0 = skin_joint(vertex.joint_indices.x, vertex_no_morph.instance_index);
    let turned = (bone0 * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.world_normal = normalize(turned);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#ifdef SKINNED
#ifdef VERTEX_UVS_B
#ifdef VERTEX_NORMALS
    // neopets: the lit draw and the part's tint (SPEC-0046), from the data joints.
    {
        let ii = vertex_no_morph.instance_index;
        let base = u32(vertex.uv_b.x + 0.5);
        let flags = u32(vertex.uv_b.y + 0.5);
        let d0 = skin_joint(base, ii);
        let d1 = skin_joint(base + 1u, ii);
        let d2 = skin_joint(base + 2u, ii);
        let slot = flags & 15u;
        let sj = skin_joint(base + 3u + slot / 4u, ii);
        let s = sj[slot % 4u].xyz;
        let g = select(128.0 / 255.0, 1.0, (flags & 32u) != 0u);
        let t = gamma(max(d2[3].xyz * s * g, vec3<f32>(0.0)));
        var rgb: vec3<f32>;
        var a: f32;
        if (flags & 16u) != 0u {
            let n = normalize(mat3x3<f32>(d0[0].xyz, d0[1].xyz, d0[2].xyz) * turned);
            let l = d1[0].xyz
                + clamp(dot(n, d1[1].xyz), 0.0, 1.0) * d1[2].xyz
                + clamp(dot(n, d1[3].xyz), 0.0, 1.0) * d2[0].xyz
                + clamp(dot(n, d2[1].xyz), 0.0, 1.0) * d2[2].xyz;
            rgb = gamma(min(vec3<f32>(255.0), floor(l * vertex.color.rgb / 16.0)) / 128.0);
            a = min(min(255.0, floor(128.0 * vertex.color.a / 16.0)) / 128.0, 1.0);
        } else {
            rgb = gamma(vertex.color.rgb);
            a = min(vertex.color.a, 1.0);
        }
        out.color = vec4<f32>(rgb * t, a * d0[3].x);
    }
#endif
#endif
#endif
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
#ifdef VERTEX_COLORS
    return mesh.color;
#else
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
#endif
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_joints_pack_lights_tint_slots_and_ghost() {
        let lights = neopets_data::light::Lights { ambient: [1.0, 2.0, 3.0], directional: [([0.0, 1.0, 0.0], [4.0, 5.0, 6.0]), ([1.0, 0.0, 0.0], [7.0, 8.0, 9.0]), ([0.0, 0.0, 1.0], [10.0, 11.0, 12.0])] };
        let mut slots = [[1.0; 3]; 15];
        slots[5] = [0.5, 0.25, 0.125];
        let look = Look { light_rotation: Quat::IDENTITY, lights: &lights, tint: [0.9, 0.8, 0.7], slots: &slots, ghost: 0.5 };
        let bone = neopets_data::anim::Affine { cols: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], t: [0.0, 1.0, 0.0] };
        let root = bevy::math::Affine3A::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let j = joint_matrices(&root, &[bone, bone], &look);
        assert_eq!(j.len(), 2 + DATA_JOINTS + 1);
        // A bone joint is the root after the skin matrix.
        assert_eq!(j[0].translation(), Vec3::new(10.0, 1.0, 0.0));
        let m = |k: usize| j[k].affine();
        // D0: identity rotation (no turn beyond the root's), the ghost level in the translation.
        assert_eq!(m(2).translation.x, 0.5);
        assert!((Mat3::from(m(2).matrix3) - Mat3::IDENTITY).abs_diff_eq(Mat3::ZERO, 1e-6));
        // D1: ambient, light 0 direction and colour, light 1 direction.
        assert_eq!(Vec3::from(m(3).matrix3.x_axis), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Vec3::from(m(3).matrix3.z_axis), Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(Vec3::from(m(3).translation), Vec3::new(1.0, 0.0, 0.0));
        // D2: light 1 colour, light 2, and the tint.
        assert_eq!(Vec3::from(m(4).matrix3.x_axis), Vec3::new(7.0, 8.0, 9.0));
        assert_eq!(Vec3::from(m(4).translation), Vec3::new(0.9, 0.8, 0.7));
        // Slot 5 is the second column of the second slot joint (D4).
        assert_eq!(Vec3::from(m(6).matrix3.y_axis), Vec3::new(0.5, 0.25, 0.125));
        // The last joint is the identity.
        assert_eq!(j[j.len() - 1], GlobalTransform::IDENTITY);
    }

    #[test]
    fn bones_outside_the_table_point_at_the_identity_joint() {
        assert_eq!(joint_indices(&[[1, 0, 2], [9, 3, 0]], 4), vec![[1, 0, 2, 0], [(4 + DATA_JOINTS) as u16, 3, 0, 0]]);
    }

    #[test]
    fn part_flags_hold_slot_lit_and_textured() {
        assert_eq!(flags(3, true, false), (3 | 16) as f32);
        assert_eq!(flags(20, false, true), (14 | 32) as f32);
    }
}
