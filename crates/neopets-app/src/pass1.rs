//! Render pass 1 of two-pass materials (SPEC-0003 How a pass is drawn, UV modes; SPEC-0004
//! Two-pass batches): the same positions, colours and normals drawn a second time with the
//! pass's own texture and UVs, blended by its ALPHA register. An environment-map pass (UV mode
//! 1) takes its UVs from the reflected view direction in world space (level geometry: M the
//! identity, E the camera position): D = P - E, R = D - 2 (D . N) N, u = 0.5 (1 - R^z),
//! v = 0.5 (1 + R^y), worked out per pixel here. A scroll pass (mode 2) adds frac(s x N) to its
//! UVs each frame. The blend follows the GS rule (A - B) x C + D: with B zero and D the
//! destination it adds the source times its alpha (C = As), the FIX value (C = FIX) or, in this
//! build, the source alpha again for C = Ad (the destination alpha is not kept); any other
//! setting blends by alpha.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError};
use bevy::shader::ShaderRef;
use std::sync::OnceLock;

static SHADER: OnceLock<Handle<Shader>> = OnceLock::new();

pub struct Pass1Plugin;

impl Plugin for Pass1Plugin {
    fn build(&self, app: &mut App) {
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(Shader::from_wgsl(WGSL.replace("#{SUN_SHARE}", &format!("{:?}", crate::graphics::SUN_SHARE)), "neopets/pass1.wgsl"));
        let _ = SHADER.set(handle);
        app.add_plugins(MaterialPlugin::<Pass1Material>::default()).add_systems(Update, scroll_pass1);
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct Pass1Params {
    /// x: 1 for an environment map; y: 0 add by source alpha, 1 add by FIX, 2 alpha blend;
    /// z: FIX / 128.
    pub mode: Vec4,
    /// The scroll offset (u, v).
    pub offset: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct Pass1Material {
    #[uniform(0)]
    pub params: Pass1Params,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
    /// Alpha blend instead of the additive blend.
    pub blend: bool,
    /// Drawn as an opaque pass 0 with the alpha mask at 0.5 (an environment-mapped pass 0).
    pub mask: bool,
}

impl Material for Pass1Material {
    fn fragment_shader() -> ShaderRef {
        SHADER.get().map_or(ShaderRef::Default, |h| ShaderRef::Handle(h.clone()))
    }

    fn alpha_mode(&self) -> AlphaMode {
        if self.mask {
            AlphaMode::Mask(0.5)
        } else if self.blend {
            AlphaMode::Blend
        } else {
            AlphaMode::Add
        }
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        // The disc's materials draw both faces.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// The material of an environment-mapped pass 0 (SPEC-0003 UV mode 1 on pass 0): the part's
/// own texture with view-generated UVs, drawn opaque with the alpha mask at 0.5 like the other
/// pass-0 materials.
pub fn env_base(texture: Handle<Image>) -> Pass1Material {
    Pass1Material { params: Pass1Params { mode: Vec4::new(1.0, 3.0, 1.0, 0.0), offset: Vec4::ZERO }, texture, blend: false, mask: true }
}

/// The blend kind and FIX scale of an ALPHA register value (fields A, B, C, D in bits 0 to 7,
/// FIX in bits 32 to 39): (kind, fix) with kind 0 add by source alpha, 1 add by FIX, 2 blend.
pub fn blend_of(alpha: Option<u64>) -> (u32, f32) {
    let Some(a) = alpha else { return (2, 1.0) };
    let (fa, fb, fc, fd) = (a & 3, (a >> 2) & 3, (a >> 4) & 3, (a >> 6) & 3);
    let fix = ((a >> 32) & 0xFF) as f32 / 128.0;
    if fa == 0 && fb == 2 && fd == 1 {
        match fc {
            2 => (1, fix),
            _ => (0, 1.0),
        }
    } else {
        (2, 1.0)
    }
}

/// A pass-1 part's scroll speeds per pass (UV mode 2).
#[derive(Component, Clone, Copy)]
pub struct Pass1Scroll(pub [f32; 2]);

/// Each frame: a scrolling pass-1 part's offset is frac(s x N) per axis, N the 16-bit pass
/// counter (the play tick, else the frame count).
fn scroll_pass1(state: Option<Res<crate::PlayState>>, mut frames: Local<u32>, parts: Query<(&Pass1Scroll, &MeshMaterial3d<Pass1Material>)>, mut materials: ResMut<Assets<Pass1Material>>) {
    *frames = frames.wrapping_add(1);
    let n = state.map_or(*frames as u16, |s| s.play.ticks as u16);
    // Performance: only a changed offset is written (a get_mut marks the material changed).
    for (s, m) in &parts {
        let want = Vec4::new(neopets_data::material::scroll_offset(s.0[0], n), neopets_data::material::scroll_offset(s.0[1], n), 0.0, 0.0);
        if materials.get(&m.0).is_some_and(|mat| mat.params.offset != want) {
            if let Some(mut mat) = materials.get_mut(&m.0) {
                mat.params.offset = want;
            }
        }
    }
}

const WGSL: &str = r#"
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::{view, lights}
#import bevy_pbr::mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT
#import bevy_pbr::shadows::fetch_directional_shadow

// The sun's share of a lit surface (graphics.rs SUN_SHARE): while the shadow sun shines the
// lit pass-0 surfaces take (1 - share) from the ambient light and share x N.L x shadow from the
// sun, and these layers follow them, so a shadow is not painted over by an unlit layer.
const SUN_SHARE: f32 = #{SUN_SHARE};

fn sun_factor(in: VertexOutput) -> f32 {
    if lights.n_directional_lights == 0u {
        return 1.0;
    }
    let light = &lights.directional_lights[0];
    let n = normalize(in.world_normal);
    let ndl = abs(dot(n, (*light).direction_to_light));
    var shadow = 1.0;
    if ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
        let view_z = dot(vec4<f32>(view.view_from_world[0].z, view.view_from_world[1].z, view.view_from_world[2].z, view.view_from_world[3].z), in.world_position);
        let toward = select(-n, n, dot(n, (*light).direction_to_light) >= 0.0);
        shadow = fetch_directional_shadow(0u, in.world_position, toward, view_z, in.position.xy);
    }
    return (1.0 - SUN_SHARE) + SUN_SHARE * ndl * shadow;
}

struct Pass1 {
    mode: vec4<f32>,
    offset: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> pass1: Pass1;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var pass1_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var pass1_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
#ifdef VERTEX_UVS_A
    var uv = in.uv + pass1.offset.xy;
#else
    var uv = pass1.offset.xy;
#endif
    if pass1.mode.x > 0.5 {
        let d = in.world_position.xyz - view.world_position;
        let n = normalize(in.world_normal);
        let r = normalize(d - 2.0 * dot(d, n) * n);
        uv = vec2<f32>(0.5 * (1.0 - r.z), 0.5 * (1.0 + r.y));
    }
    let t = textureSample(pass1_texture, pass1_sampler, uv);
    var c = t.rgb * sun_factor(in);
    var a = t.a;
#ifdef VERTEX_COLORS
    c = c * in.color.rgb;
    a = a * in.color.a;
#endif
    if pass1.mode.y < 0.5 {
        return vec4<f32>(c * a, 0.0);
    }
    if pass1.mode.y < 1.5 {
        return vec4<f32>(c * pass1.mode.z, 0.0);
    }
    if pass1.mode.y > 2.5 {
        if a < 0.5 {
            discard;
        }
        return vec4<f32>(c, 1.0);
    }
    return vec4<f32>(c, a);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_register_kinds() {
        // 0x48: Cs x As + Cd; 0x68 with FIX 64: Cs x 0.5 + Cd; 0x58: Cs x Ad + Cd (As here);
        // 0x44: (Cs - Cd) x As + Cd, the alpha blend.
        assert_eq!(blend_of(Some(0x48)), (0, 1.0));
        assert_eq!(blend_of(Some(0x68 | (64 << 32))), (1, 0.5));
        assert_eq!(blend_of(Some(0x58)), (0, 1.0));
        assert_eq!(blend_of(Some(0x44)), (2, 1.0));
        assert_eq!(blend_of(None), (2, 1.0));
    }
}
