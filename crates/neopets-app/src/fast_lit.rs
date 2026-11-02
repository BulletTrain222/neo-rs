//! A faster lit shading for the world's surfaces under the sun (a performance feature of this
//! build). With World shadows or Detailed character shadows on, the game's unlit surfaces turn
//! lit under a matched ambient light and the shadow sun (graphics.rs): a surface facing the sun
//! keeps its unlit colour, and shade takes away up to SUN_SHARE of it. Bevy's full lighting
//! does that through its whole physically based path (clusters, specular, environment terms),
//! which costs about 4 ms a frame at 2560 x 1600 for terms that are zero here (no reflectance,
//! no metal, no point lights). This replaces Bevy's standard fragment shader (its embedded
//! `bevy_pbr/render/pbr.wgsl`, which is MIT/Apache Bevy code, not game data) with one that
//! computes the same result directly: base x ((1 - S) + S x Burley diffuse x N.L x shadow),
//! with the sun's shadow from Bevy's own cascade lookup. Unlit surfaces, the prepass, and every
//! case outside the matched setup (ambient occlusion on, no sun) keep Bevy's own path.

use bevy::asset::AssetEvent;
use bevy::prelude::*;

/// The embedded path of Bevy's standard material fragment shader.
const PBR_PATH: &str = "embedded://bevy_pbr/render/pbr.wgsl";
/// A marker in the replacement source, to tell it from Bevy's.
const MARKER: &str = "neopets fast lit";

pub struct FastLitPlugin;

impl Plugin for FastLitPlugin {
    fn build(&self, app: &mut App) {
        // NEOPETS_FULL_PBR: keep Bevy's full lighting (diagnostic; a screenshot pair at 934 differs
        // by 0.16 of 255 on average, 0.01 percent of pixels by more than 12).
        if std::env::var_os("NEOPETS_FULL_PBR").is_none() {
            app.add_systems(Update, replace_shader);
        }
    }
}

/// When Bevy's standard fragment shader is loaded (or reloaded), its source is replaced.
fn replace_shader(asset_server: Res<AssetServer>, mut shaders: ResMut<Assets<Shader>>, mut events: MessageReader<AssetEvent<Shader>>, mut handle: Local<Option<Handle<Shader>>>) {
    let h = handle.get_or_insert_with(|| asset_server.load(PBR_PATH)).clone();
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
    let _ = shaders.insert(&h, Shader::from_wgsl(source(), path));
    info!("fast lit: Bevy's standard fragment shader replaced");
}

fn source() -> String {
    SOURCE.replace("SUN_SHARE", &format!("{:.6}", crate::graphics::SUN_SHARE)).replace("MARKER", MARKER)
}

/// Bevy 0.19's pbr.wgsl with one branch added in the forward path (marked below).
const SOURCE: &str = r#"// MARKER
#import bevy_pbr::{
    pbr_types,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decals,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    mesh_view_bindings as view_bindings,
    mesh_view_types,
    shadows,
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
}
#endif

#ifdef VISIBILITY_RANGE_DITHER
#import bevy_pbr::pbr_functions::visibility_range_dither;
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
#import bevy_pbr::decal::forward::get_forward_decal_info
#endif

#ifndef PREPASS_PIPELINE
// The matched sun lighting: (1 - S) of the colour from the ambient light, and up to S from the
// sun through Bevy's Burley diffuse at roughness 1, its N.L and its cascade shadow.
fn fast_lit(pbr_input: pbr_types::PbrInput) -> vec4<f32> {
    let base = pbr_input.material.base_color;
    let light = &view_bindings::lights.directional_lights[0];
    let N = pbr_input.N;
    let V = pbr_input.V;
    let L = (*light).direction_to_light;
    let NdotL = saturate(dot(N, L));
    var lit = 1.0 - SUN_SHARE;
    if NdotL > 0.0 {
        let NdotV = max(dot(N, V), 0.0001);
        let H = normalize(L + V);
        let LdotH = saturate(dot(L, H));
        let roughness = pbr_input.material.perceptual_roughness * pbr_input.material.perceptual_roughness;
        let f90 = 0.5 + 2.0 * roughness * LdotH * LdotH;
        let burley = (1.0 + (f90 - 1.0) * pow(1.0 - NdotL, 5.0)) * (1.0 + (f90 - 1.0) * pow(1.0 - NdotV, 5.0));
        var shadow = 1.0;
        if (pbr_input.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u
                && ((*light).flags & mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            let view_z = dot(vec4<f32>(
                view_bindings::view.view_from_world[0].z,
                view_bindings::view.view_from_world[1].z,
                view_bindings::view.view_from_world[2].z,
                view_bindings::view.view_from_world[3].z
            ), pbr_input.world_position);
            shadow = shadows::fetch_directional_shadow(0u, pbr_input.world_position, pbr_input.world_normal, view_z, pbr_input.frag_coord.xy);
        }
        // Near grazing the map cannot resolve the surface against itself (stripes and blocks on steep
        // cliffs at any bias), and the sun adds little there: the shadow fades out.
        lit += SUN_SHARE * burley * NdotL * mix(1.0, shadow, smoothstep(0.0, 0.6, NdotL));
    }
    return vec4<f32>(base.rgb * lit, base.a);
}
#endif

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    let vertex_output = resolve_vertex_output(frag_coord);
    let is_front = true;
#endif

    var in = vertex_output;

#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(in);
    in.world_position = forward_decal_info.world_position;
    in.uv = forward_decal_info.uv;
#endif

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    apply_decals(&pbr_input);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        // Added: the matched sun lighting while the sun shines and no occlusion pass is on;
        // Bevy's full lighting otherwise.
#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
        out.color = apply_pbr_lighting(pbr_input);
#else
        if view_bindings::lights.n_directional_lights > 0u {
            out.color = fast_lit(pbr_input);
        } else {
            out.color = apply_pbr_lighting(pbr_input);
        }
#endif
    } else {
        out.color = pbr_input.material.base_color;
    }

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

#ifdef OIT_ENABLED
    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        oit_draw(in.position, out.color);
        discard;
    }
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
        out.color.a = min(forward_decal_info.alpha, out.color.a);
#endif

        return out;
}
"#;
