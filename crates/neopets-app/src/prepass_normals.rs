//! Surface normals for ambient occlusion (this build's presentation; no spec). The disc's scene
//! parts store normals that point straight up, walls included (see scene_merge.rs `is_ground`),
//! and lit drawing keeps them. Bevy's ambient occlusion reads the normal prepass, and on a wall
//! with an upward normal it drew fine vertical stripes, seen where the wall is in the sun's
//! shadow (user report 130 in 934). Bevy's prepass fragment shader (its embedded
//! `bevy_pbr/render/pbr_prepass.wgsl`, MIT/Apache Bevy code, not game data) is patched when it
//! loads: where the stored normal lies more than 60 degrees from the surface's own facing (taken
//! from the screen-space derivatives of the position, turned toward the camera), the prepass
//! writes the surface's facing instead. Floors and characters keep their normals; lighting is not
//! changed. NEOPETS_STORED_AO_NORMALS keeps Bevy's shader (diagnostic).
use bevy::prelude::*;

const PATH: &str = "embedded://bevy_pbr/render/pbr_prepass.wgsl";
const MARKER: &str = "// neopets: surface normals for ambient occlusion";

const HEAD: &str = ") -> prepass_io::FragmentOutput {";
const HEAD_NEW: &str = ") -> prepass_io::FragmentOutput {
#ifndef MESHLET_MESH_MATERIAL_PASS
    // neopets: surface normals for ambient occlusion
    let neopets_face = normalize(cross(dpdy(in.world_position.xyz), dpdx(in.world_position.xyz)));
#endif";
const OUT: &str = "        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);";
const OUT_NEW: &str = "#ifndef MESHLET_MESH_MATERIAL_PASS
        var neopets_facing = neopets_face;
        if dot(neopets_facing, view.world_position.xyz - in.world_position.xyz) < 0.0 {
            neopets_facing = -neopets_facing;
        }
        if abs(dot(normalize(normal), neopets_facing)) < 0.5 {
            normal = neopets_facing;
        }
#endif
        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);";

pub struct PrepassNormalsPlugin;

impl Plugin for PrepassNormalsPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var_os("NEOPETS_STORED_AO_NORMALS").is_none() {
            app.add_systems(Update, replace_shader);
        }
    }
}

/// When Bevy's prepass shader is loaded (or reloaded), its source is patched.
fn replace_shader(asset_server: Res<AssetServer>, mut shaders: ResMut<Assets<Shader>>, mut events: MessageReader<AssetEvent<Shader>>, mut handle: Local<Option<Handle<Shader>>>) {
    let h = handle.get_or_insert_with(|| asset_server.load(PATH)).clone();
    let touched = events.read().any(|e| matches!(e, AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } if *id == h.id()));
    if !touched {
        return;
    }
    let Some(current) = shaders.get(&h) else { return };
    let bevy::shader::Source::Wgsl(src) = &current.source else { return };
    if src.contains(MARKER) {
        return;
    }
    if src.matches(HEAD).count() != 1 || src.matches(OUT).count() != 1 {
        warn!("prepass normals: Bevy's prepass shader is not the expected one; left as it is");
        return;
    }
    let patched = src.replace(HEAD, HEAD_NEW).replace(OUT, OUT_NEW);
    let path = current.path.clone();
    let _ = shaders.insert(&h, Shader::from_wgsl(patched, path));
    info!("prepass normals: Bevy's prepass shader patched");
}
