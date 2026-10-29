//! Graphics options (a feature of this build; the original draws none of this): anti-aliasing,
//! sharper textures at a distance (mipmaps with anisotropic filtering), soft bloom, colour
//! grading and ambient occlusion, chosen on the Options menu's Display page.
//!
//! The game's surfaces are unlit (their lighting is baked into vertex colours). Ambient
//! occlusion needs lit surfaces, so while it is on the solid surfaces turn lit under an even
//! ambient light set so the lit colour equals the unlit one: with roughness 1, no reflectance
//! and no metal, Bevy's ambient term is base colour x 0.4524 - 0.0024 (its F_AB at roughness 1 is
//! independent of the view angle), times the light and the camera exposure. The occlusion
//! then darkens creases and corners only.
//!
//! Colour grading runs where the 3D picture is drawn into the window (`SceneGrade`), so the
//! original colours are untouched when it is off.

use bevy::anti_alias::fxaa::Fxaa;
use bevy::anti_alias::smaa::Smaa;
use bevy::asset::AssetEvent;
use bevy::camera::Hdr;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::AmbientLight;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::view::Msaa;
use bevy::shader::ShaderRef;
use bevy::ui_render::prelude::{UiMaterial, UiMaterialPlugin};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::display::DisplaySettings;

pub const AA: [&str; 4] = ["Off", "MSAA 4x", "SMAA", "FXAA"];
pub const BLOOM: [&str; 3] = ["Off", "Soft", "Strong"];
pub const COLOUR: [&str; 3] = ["Original", "Vivid", "Rich"];

/// Whether new world textures sample their mipmaps with anisotropic filtering (else only the
/// full-size level, as the original).
static SHARP: AtomicBool = AtomicBool::new(true);
static GRADE_SHADER: OnceLock<Handle<Shader>> = OnceLock::new();

/// The camera exposure Bevy gives a 3D camera (EV100 9.7) and the ambient term's scale at
/// roughness 1: the ambient brightness that makes a lit surface match its unlit colour.
fn matched_ambient() -> f32 {
    1.0 / (0.4524 * exposure())
}

fn exposure() -> f32 {
    1.0 / (1.2 * 2f32.powf(9.7))
}

/// World shadows (this build's option): the share of a lit surface's brightness that comes from
/// the sun, and so the most a shadow takes away; the ambient light keeps the rest.
pub const SUN_SHARE: f32 = 0.45;

/// The sun's illuminance that adds SUN_SHARE of a surface's matched colour where it faces the
/// sun straight on (a rough diffuse surface: albedo / pi x illuminance x exposure).
pub fn sun_illuminance() -> f32 {
    SUN_SHARE * std::f32::consts::PI / exposure()
}

/// The ambient brightness while lit: the full matched one, less the sun's share with world
/// shadows on.
fn ambient(settings: &DisplaySettings) -> f32 {
    matched_ambient() * if sun_on(settings) { 1.0 - SUN_SHARE } else { 1.0 }
}

/// Whether the shadow sun shines: World shadows, or Character shadows Detailed.
pub fn sun_on(settings: &DisplaySettings) -> bool {
    settings.world_shadows || settings.char_shadows == 2
}

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(Shader::from_wgsl(GRADE_WGSL, "neopets/scene_grade.wgsl"));
        let _ = GRADE_SHADER.set(handle);
        app.add_plugins(UiMaterialPlugin::<SceneGrade>::default()).add_systems(Update, (apply_camera, apply_materials, apply_textures, apply_grade));
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct GradeUniform {
    /// Saturation, contrast, warmth, vibrance.
    pub a: Vec4,
    /// Vignette, -, -, on (above 0.5).
    pub b: Vec4,
}

/// The 3D picture drawn into the window, with the colour grade.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SceneGrade {
    #[uniform(0)]
    pub grade: GradeUniform,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl UiMaterial for SceneGrade {
    fn fragment_shader() -> ShaderRef {
        GRADE_SHADER.get().map_or(ShaderRef::Default, |h| ShaderRef::Handle(h.clone()))
    }
}

pub fn grade_uniform(colour: usize) -> GradeUniform {
    match colour {
        1 => GradeUniform { a: Vec4::new(1.10, 1.06, 0.0, 0.25), b: Vec4::new(0.10, 0.0, 0.0, 1.0) },
        2 => GradeUniform { a: Vec4::new(1.18, 1.10, 0.015, 0.30), b: Vec4::new(0.18, 0.0, 0.0, 1.0) },
        _ => GradeUniform::default(),
    }
}

const GRADE_WGSL: &str = r#"
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct Grade {
    a: vec4<f32>,
    b: vec4<f32>,
}

@group(1) @binding(0) var<uniform> grade: Grade;
@group(1) @binding(1) var scene: texture_2d<f32>;
@group(1) @binding(2) var scene_sampler: sampler;

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let s = textureSample(scene, scene_sampler, in.uv);
    if grade.b.w < 0.5 {
        return vec4<f32>(s.rgb, 1.0);
    }
    var c = to_srgb(s.rgb);
    let luma = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    // Saturation, lifted more for dull colours (vibrance).
    let sat = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
    c = mix(vec3<f32>(luma), c, grade.a.x * (1.0 + grade.a.w * (1.0 - sat)) - grade.a.w * (1.0 - sat) * 0.5);
    // Contrast about mid grey, then warmth.
    c = (c - vec3<f32>(0.5)) * grade.a.y + vec3<f32>(0.5);
    c = c + vec3<f32>(grade.a.z, 0.0, -grade.a.z);
    // A soft vignette.
    let d = distance(in.uv, vec2<f32>(0.5));
    c = c * (1.0 - grade.b.x * smoothstep(0.4, 0.85, d));
    return vec4<f32>(to_linear(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
"#;

/// The camera's anti-aliasing, bloom and ambient occlusion from the settings.
fn apply_camera(mut commands: Commands, settings: Res<DisplaySettings>, cameras: Query<Entity, With<Camera3d>>, mut last: Local<Option<(usize, usize, bool, usize, bool, bool)>>) {
    let key = (settings.aa, settings.bloom, settings.ao, cameras.iter().count(), settings.char_shadows == 1, sun_on(&settings));
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    for e in &cameras {
        let mut c = commands.entity(e);
        // Ambient occlusion reads a single-sampled depth and normal prepass, so MSAA gives way
        // to SMAA while it is on.
        let aa = if settings.ao && settings.aa == 1 { 2 } else { settings.aa };
        c.insert(if aa == 1 { Msaa::Sample4 } else { Msaa::Off });
        c.remove::<(Smaa, Fxaa)>();
        match aa {
            2 => {
                c.insert(Smaa::default());
            }
            3 => {
                c.insert(Fxaa::default());
            }
            _ => {}
        }
        if settings.bloom > 0 {
            let (intensity, threshold) = if settings.bloom == 2 { (0.22, 0.55) } else { (0.10, 0.7) };
            c.insert((Hdr, Bloom { intensity, prefilter: BloomPrefilter { threshold, threshold_softness: 0.4 }, composite_mode: BloomCompositeMode::Additive, ..Bloom::NATURAL }));
        } else if settings.char_shadows == 1 {
            // The shadow volumes multiply the picture up and down again (shadow_view.rs), which
            // needs the float picture.
            c.remove::<Bloom>();
            c.insert(Hdr);
        } else {
            c.remove::<(Bloom, Hdr)>();
        }
        // NEOPETS_GFX_LIT_ONLY: the lit surfaces without the occlusion pass (diagnostic, to
        // check the matched colours against the unlit ones).
        if settings.ao && std::env::var_os("NEOPETS_GFX_LIT_ONLY").is_some() {
            c.remove::<(ScreenSpaceAmbientOcclusion, DepthPrepass, NormalPrepass)>();
            c.insert(AmbientLight { color: Color::WHITE, brightness: ambient(&settings), affects_lightmapped_meshes: true });
        } else if settings.ao {
            c.insert((ScreenSpaceAmbientOcclusion::default(), AmbientLight { color: Color::WHITE, brightness: ambient(&settings), affects_lightmapped_meshes: true }));
        } else if sun_on(&settings) {
            c.remove::<(ScreenSpaceAmbientOcclusion, NormalPrepass)>();
            c.insert(AmbientLight { color: Color::WHITE, brightness: ambient(&settings), affects_lightmapped_meshes: true });
            // Performance: a depth prepass, so the lit main pass shades each pixel about once
            // instead of every overlapping surface (NEOPETS_NO_PREPASS: none, for comparison).
            if std::env::var_os("NEOPETS_NO_PREPASS").is_some() {
                c.remove::<DepthPrepass>();
            } else {
                c.insert(DepthPrepass);
            }
        } else {
            c.remove::<(ScreenSpaceAmbientOcclusion, DepthPrepass, NormalPrepass, AmbientLight)>();
            // NEOPETS_PREPASS_ALWAYS: a depth prepass without the sun too (diagnostic).
            if std::env::var_os("NEOPETS_PREPASS_ALWAYS").is_some() {
                c.insert(DepthPrepass);
            }
        }
        // NEOPETS_OCCLUSION: GPU occlusion culling, with the depth prepass it needs (diagnostic).
        if std::env::var_os("NEOPETS_OCCLUSION").is_some() {
            c.insert((DepthPrepass, bevy::render::occlusion_culling::OcclusionCulling));
        }
    }
}

/// While ambient occlusion or world shadows are on, the solid unlit surfaces turn lit under the
/// matched ambient light (and back when both go off); new ones as they load.
fn apply_materials(settings: Res<DisplaySettings>, mut materials: ResMut<Assets<StandardMaterial>>, mut events: MessageReader<AssetEvent<StandardMaterial>>, mut lit: Local<HashSet<AssetId<StandardMaterial>>>, mut last: Local<Option<bool>>) {
    let eligible = |m: &StandardMaterial| m.unlit && matches!(m.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_));
    let light = |m: &mut StandardMaterial| {
        m.unlit = false;
        m.perceptual_roughness = 1.0;
        m.reflectance = 0.0;
        m.metallic = 0.0;
        // The disc's normals already face out; the strips' winding varies, so a two-sided
        // material's back-face flip would turn half the ground's normals down.
        m.double_sided = false;
    };
    let added: Vec<AssetId<StandardMaterial>> = events.read().filter_map(|e| if let AssetEvent::Added { id } = e { Some(*id) } else { None }).collect();
    let on = settings.ao || sun_on(&settings);
    if *last != Some(on) {
        *last = Some(on);
        if on {
            let ids: Vec<_> = materials.iter().filter(|(_, m)| eligible(m)).map(|(id, _)| id).collect();
            for id in ids {
                if let Some(mut m) = materials.get_mut(id) {
                    light(&mut m);
                    lit.insert(id);
                }
            }
        } else {
            for id in lit.drain() {
                if let Some(mut m) = materials.get_mut(id) {
                    m.unlit = true;
                }
            }
        }
        return;
    }
    if on {
        for id in added {
            if materials.get(id).is_some_and(|m| eligible(m)) {
                if let Some(mut m) = materials.get_mut(id) {
                    light(&mut m);
                    lit.insert(id);
                }
            }
        }
    }
}

/// The sampler of a mipmapped world texture: anisotropic over the mipmaps, or the full-size
/// level only.
pub fn world_sampler(base: ImageSamplerDescriptor) -> ImageSampler {
    let sharp = SHARP.load(Ordering::Relaxed);
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: if sharp { ImageFilterMode::Linear } else { ImageFilterMode::Nearest },
        anisotropy_clamp: if sharp { 16 } else { 1 },
        lod_max_clamp: if sharp { 32.0 } else { 0.0 },
        ..base
    })
}

/// A world texture with its mipmaps: each level the 2 x 2 average of the one above, colours
/// weighted by alpha so cut-out edges do not darken.
pub fn with_mipmaps(mut image: Image) -> Image {
    let (mut w, mut h) = (image.width(), image.height());
    let Some(base) = image.data.clone() else { return image };
    let mut data = base.clone();
    let mut level = base;
    let mut levels = 1;
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                let mut sum = [0f32; 4];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                    let i = ((sy * w + sx) * 4) as usize;
                    let a = level[i + 3] as f32;
                    for c in 0..3 {
                        sum[c] += level[i + c] as f32 * a;
                    }
                    sum[3] += a;
                }
                let o = ((y * nw + x) * 4) as usize;
                for c in 0..3 {
                    next[o + c] = if sum[3] > 0.0 { (sum[c] / sum[3]).round() as u8 } else { 0 };
                }
                next[o + 3] = (sum[3] / 4.0).round() as u8;
            }
        }
        data.extend_from_slice(&next);
        level = next;
        (w, h) = (nw, nh);
        levels += 1;
    }
    image.data = Some(data);
    image.texture_descriptor.mip_level_count = levels;
    image
}

/// The texture setting: new textures take it, and the loaded mipmapped ones change sampler.
fn apply_textures(settings: Res<DisplaySettings>, mut images: ResMut<Assets<Image>>, mut last: Local<Option<bool>>) {
    if *last == Some(settings.sharp_textures) {
        return;
    }
    let first = last.is_none();
    *last = Some(settings.sharp_textures);
    SHARP.store(settings.sharp_textures, Ordering::Relaxed);
    if first {
        return;
    }
    let ids: Vec<_> = images.iter().filter(|(_, i)| i.texture_descriptor.mip_level_count > 1).map(|(id, _)| id).collect();
    for id in ids {
        if let Some(mut i) = images.get_mut(id) {
            if let ImageSampler::Descriptor(d) = i.sampler.clone() {
                i.sampler = world_sampler(d);
            }
        }
    }
}

/// The colour grade from the settings.
fn apply_grade(settings: Res<DisplaySettings>, mut grades: ResMut<Assets<SceneGrade>>, mut last: Local<Option<usize>>) {
    if *last == Some(settings.colour) && !grades.is_changed() {
        return;
    }
    let want = grade_uniform(settings.colour);
    let ids: Vec<_> = grades.iter().filter(|(_, g)| g.grade.a != want.a || g.grade.b != want.b).map(|(id, _)| id).collect();
    for id in ids {
        if let Some(mut g) = grades.get_mut(id) {
            g.grade = want;
        }
    }
    *last = Some(settings.colour);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mipmaps_halve_down_to_one_texel() {
        let mut img = crate::rgba_image(4, 2, vec![255; 4 * 2 * 4]);
        img = with_mipmaps(img);
        assert_eq!(img.texture_descriptor.mip_level_count, 3);
        assert_eq!(img.data.as_ref().unwrap().len(), (8 + 2 + 1) * 4);
    }

    #[test]
    fn transparent_texels_do_not_darken_the_average() {
        // One opaque red texel beside three clear black ones.
        let mut px = vec![0u8; 16];
        px[0..4].copy_from_slice(&[200, 0, 0, 255]);
        let img = with_mipmaps(crate::rgba_image(2, 2, px));
        let d = img.data.unwrap();
        assert_eq!(&d[16..20], &[200, 0, 0, 64]);
    }
}
