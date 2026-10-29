//! Character shadow volumes on screen (SPEC-0046 Character shadow volumes, steps 7 and 8). The
//! original counts each volume polygon in front of a surface as +8 (its outward side toward the
//! camera) or -8 in a buffer and darkens the frame by 8 where the count is positive. Here the
//! two signs are drawn straight onto the frame after the opaque scene, depth-tested and without
//! depth writes, as multiplies on the float (HDR) picture: a +8 polygon multiplies by f and a -8
//! polygon by 1/f, so a surface behind both walls of a volume is unchanged and one inside it is
//! darkened by f (for a camera outside the volumes, the usual case; overlapping volumes darken
//! more where the original's count would stay 1). The multiply keeps hues where the original's
//! fixed drop does not; f is the Display page's choice: Original matches the drop of 8 of 255
//! at mid grey, Strong is this build's.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::shader::ShaderRef;
use std::sync::OnceLock;

static SHADER: OnceLock<Handle<Shader>> = OnceLock::new();

/// The shadow factor of the volumes (linear colour): Original is a drop of 8 of 255 at mid grey
/// (128 to 120 in display values; SPEC-0046 step 8). Detailed (2) draws no volumes: the sun's
/// shadow map gives the characters' own shapes instead.
const FACTORS: [f32; 3] = [1.0, 0.87, 1.0];

pub struct ShadowPlugin;

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut App) {
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(Shader::from_wgsl(WGSL, "neopets/shadow_volume.wgsl"));
        let _ = SHADER.set(handle);
        app.add_plugins((MaterialPlugin::<ShadowSubtract>::default(), MaterialPlugin::<ShadowAdd>::default()));
        // The sun's shadow map: 4096 texels a cascade (Bevy's default 2048 left the characters'
        // shadows soft and faint).
        app.insert_resource(bevy::light::DirectionalLightShadowMap { size: 4096 });
        app.add_systems(Update, apply_quality);
        app.add_systems(PostUpdate, extend_cascades.after(bevy::light::SimulationLightSystems::UpdateDirectionalLightCascades).before(bevy::light::SimulationLightSystems::UpdateLightFrusta));
    }
}

/// The frame times the fragment's colour.
fn blend() -> BlendState {
    BlendState {
        color: BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::Src, operation: BlendOperation::Add },
        alpha: BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
    }
}

macro_rules! volume_material {
    ($name:ident) => {
        #[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
        pub struct $name {
            #[uniform(0)]
            pub amount: Vec4,
        }

        impl Material for $name {
            fn fragment_shader() -> ShaderRef {
                SHADER.get().map_or(ShaderRef::Default, |h| ShaderRef::Handle(h.clone()))
            }

            fn alpha_mode(&self) -> AlphaMode {
                AlphaMode::Blend
            }

            fn enable_prepass() -> bool {
                false
            }

            fn enable_shadows() -> bool {
                false
            }

            fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
                descriptor.primitive.cull_mode = None;
                if let Some(depth) = descriptor.depth_stencil.as_mut() {
                    depth.depth_write_enabled = Some(false);
                }
                if let Some(target) = descriptor.fragment.as_mut().and_then(|f| f.targets.get_mut(0)).and_then(|t| t.as_mut()) {
                    target.blend = Some(blend());
                }
                Ok(())
            }
        }
    };
}

volume_material!(ShadowSubtract);
volume_material!(ShadowAdd);

/// The two volume meshes (the +8 and the -8 polygons).
#[derive(Component)]
pub struct VolumeMesh(bool);

fn empty_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default()).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
}

/// Each frame: the hero's shadow volume rebuilt into the two meshes, at the Display page's
/// strength (hidden when Off or outside play).
#[allow(clippy::too_many_arguments)]
pub fn sync_shadows(
    mut commands: Commands,
    state: Option<Res<crate::PlayState>>,
    settings: Res<crate::display::DisplaySettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut subtract: ResMut<Assets<ShadowSubtract>>,
    mut add: ResMut<Assets<ShadowAdd>>,
    mut parts: Query<(&VolumeMesh, &Mesh3d, &mut Visibility)>,
    mut made: Local<Option<(Handle<ShadowSubtract>, Handle<ShadowAdd>)>>,
) {
    let _span = crate::perf::span("sync_shadows");
    let f = FACTORS[settings.char_shadows.min(FACTORS.len() - 1)];
    let (down, up) = (Vec4::splat(f), Vec4::splat(1.0 / f));
    let Some(state) = state else { return };
    if made.is_none() {
        let s = subtract.add(ShadowSubtract { amount: down });
        let a = add.add(ShadowAdd { amount: up });
        commands.spawn((Mesh3d(meshes.add(empty_mesh())), MeshMaterial3d(s.clone()), Transform::IDENTITY, Visibility::Hidden, VolumeMesh(true)));
        commands.spawn((Mesh3d(meshes.add(empty_mesh())), MeshMaterial3d(a.clone()), Transform::IDENTITY, Visibility::Hidden, VolumeMesh(false)));
        *made = Some((s, a));
        return;
    }
    if let Some((s, a)) = made.as_ref() {
        if subtract.get(s).is_some_and(|m| m.amount != down) {
            if let Some(mut m) = subtract.get_mut(s) {
                m.amount = down;
            }
        }
        if add.get(a).is_some_and(|m| m.amount != up) {
            if let Some(mut m) = add.get_mut(a) {
                m.amount = up;
            }
        }
    }
    let on = f < 1.0;
    let v = if on { state.play.shadow_volume() } else { Default::default() };
    for (part, mesh, mut vis) in &mut parts {
        let tris = if part.0 { &v.toward } else { &v.away };
        let show = on && !tris.is_empty();
        let want = if show { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        if !show {
            continue;
        }
        if let Some(mut m) = meshes.get_mut(&mesh.0) {
            let positions: Vec<[f32; 3]> = tris.iter().flatten().copied().collect();
            m.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        }
    }
}

/// The sun (this build's option): a directional light with shadow maps, on while World shadows
/// is on or Character shadows is Detailed, from the world's brightest directional light
/// (`Play::sun_direction`).
#[derive(Component)]
pub struct Sun;

pub fn sync_sun(mut commands: Commands, state: Option<Res<crate::PlayState>>, settings: Res<crate::display::DisplaySettings>, mut suns: Query<(&mut Transform, &mut Visibility), With<Sun>>, mut made: Local<bool>) {
    let _span = crate::perf::span("sync_sun");
    if !*made {
        *made = true;
        commands.spawn((
            // Smaller biases than Bevy's (0.02 depth, 1.8 normal texels) keep a character's
            // shadow at its feet with the finer map.
            DirectionalLight { illuminance: crate::graphics::sun_illuminance(), shadow_maps_enabled: true, shadow_depth_bias: 0.01, shadow_normal_bias: 0.9, ..default() },
            bevy::light::CascadeShadowConfigBuilder { num_cascades: 3, maximum_distance: 200.0, first_cascade_far_bound: 20.0, ..default() }.build(),
            Transform::default(),
            Visibility::Hidden,
            Sun,
        ));
        return;
    }
    let on = (settings.world_shadows || settings.char_shadows == 2) && state.is_some();
    let dir = state.as_ref().map_or([0.35, -1.0, 0.25], |s| s.play.sun_direction());
    let d = Vec3::from(dir).normalize_or(Vec3::NEG_Y);
    for (mut t, mut vis) in &mut suns {
        let want = if on { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        let target = Transform::default().looking_to(d, if d.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y });
        if t.rotation.dot(target.rotation).abs() < 0.99999 {
            t.rotation = target.rotation;
        }
    }
}

/// A mesh this module took out of the shadow casters (so the dome's own setting is left alone).
#[derive(Component)]
pub struct CasterOff;

/// How far each cascade's culling box reaches back toward the sun, in world units.
const CASTER_REACH: f32 = 500.0;

/// NEOPETS_SUN="size,cascades,distance,reach": the sun's shadow map size, cascade count,
/// shadow distance and caster reach, over the Shadow quality setting (diagnostic).
fn sun_override() -> Option<(f32, f32, f32, f32)> {
    static T: std::sync::OnceLock<Option<(f32, f32, f32, f32)>> = std::sync::OnceLock::new();
    *T.get_or_init(|| {
        let v = std::env::var("NEOPETS_SUN").ok()?;
        let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        match n[..] {
            [a, b, c, d] => Some((a, b, c, d)),
            _ => None,
        }
    })
}

/// The Shadow quality levels: each cascade's map size, the cascade count and the shadow
/// distance. High is the 4096 map that keeps the characters' shadows crisp; Medium halves the
/// map; Low quarters it and draws two cascades over a shorter distance.
fn quality(level: usize) -> (f32, f32, f32) {
    match level {
        0 => (1024.0, 2.0, 150.0),
        1 => (2048.0, 3.0, 200.0),
        _ => (4096.0, 3.0, 200.0),
    }
}

/// The Shadow quality setting, applied to the shadow map and the sun's cascades when it
/// changes (and once the sun exists).
fn apply_quality(settings: Res<crate::display::DisplaySettings>, mut map: ResMut<bevy::light::DirectionalLightShadowMap>, mut suns: Query<&mut bevy::light::CascadeShadowConfig, With<Sun>>, mut last: Local<Option<usize>>) {
    let level = settings.shadow_quality;
    if *last == Some(level) || suns.is_empty() {
        return;
    }
    *last = Some(level);
    let (size, cascades, distance) = sun_override().map_or(quality(level), |t| (t.0, t.1, t.2));
    if map.size != size as usize {
        map.size = size as usize;
    }
    for mut c in &mut suns {
        *c = bevy::light::CascadeShadowConfigBuilder { num_cascades: cascades as usize, maximum_distance: distance, first_cascade_far_bound: 20.0, ..default() }.build();
    }
}

/// Bevy fits each cascade's depth range to its slice of the camera's view, so a caster between
/// the sun and that range (a tall tree beside the path) was cut from the near cascade: the
/// ground close to the camera came out lit inside the tree's shadow, a wedge with straight edges
/// at the cascade's bounds. Each cascade's depth range is extended CASTER_REACH toward the sun:
/// the orthographic depth row is remade so the near side sits CASTER_REACH further out, and the
/// view-projection follows it (the culling box is built from it afterwards).
fn extend_cascades(mut lights: Query<&mut bevy::light::Cascades>) {
    for mut c in &mut lights {
        for cascades in c.cascades.values_mut() {
            for k in cascades.iter_mut() {
                let p = k.clip_from_cascade;
                let r = p.z_axis.z;
                if r <= 0.0 || p.w_axis.z != 1.0 {
                    continue;
                }
                let cascade_from_world = p.inverse() * k.clip_from_world;
                let reach = sun_override().map_or(CASTER_REACH, |t| t.3);
                let r2 = 1.0 / (1.0 / r + reach);
                let mut q = p;
                q.z_axis.z = r2;
                q.w_axis.z = 1.0 - r2 * reach;
                k.clip_from_cascade = q;
                k.clip_from_world = q * cascade_from_world;
            }
        }
    }
}

/// A blended mesh that its vertex colours make mostly see-through (every vertex alpha below
/// 0.5: fades, glows and dark decal quads) never casts: Bevy's shadow pass cuts out by the
/// texture's alpha only, so such a quad would cast a solid rectangle.
#[derive(Component)]
pub struct FaintCaster;

/// Marks the new faint blended meshes (`FaintCaster`, NotShadowCaster).
#[allow(clippy::type_complexity)]
pub fn mark_faint(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    added: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), (Or<(Added<Mesh3d>, Changed<MeshMaterial3d<StandardMaterial>>)>, Without<FaintCaster>)>,
) {
    let _span = crate::perf::span("mark_faint");
    for (e, mesh, material) in &added {
        let Some(m) = materials.get(&material.0) else { continue };
        if matches!(m.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_)) {
            continue;
        }
        let Some(mesh) = meshes.get(&mesh.0) else { continue };
        let faint = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(bevy::mesh::VertexAttributeValues::Float32x4(c)) => !c.is_empty() && c.iter().all(|v| v[3] * m.base_color.alpha() < 0.5),
            _ => m.base_color.alpha() < 0.5,
        };
        if faint {
            commands.entity(e).insert((FaintCaster, bevy::light::NotShadowCaster));
        }
    }
}

/// With Character shadows Detailed and World shadows off, only the characters (the hero, the
/// world's characters, their items and attachments) cast: every other mesh is marked
/// NotShadowCaster; with World shadows on the marks come off again.
#[allow(clippy::type_complexity)]
pub fn sync_casters(
    mut commands: Commands,
    settings: Res<crate::display::DisplaySettings>,
    meshes: Query<(Entity, Has<CasterOff>, Has<bevy::light::NotShadowCaster>), With<Mesh3d>>,
    added: Query<(), Added<Mesh3d>>,
    parents: Query<&ChildOf>,
    roots: Query<(), Or<(With<crate::HeroRoot>, With<crate::CharacterRoot>, With<crate::AttachmentRoot>, With<crate::HeroItemRoot>)>>,
    mut last: Local<Option<bool>>,
    faint: Query<(), With<FaintCaster>>,
) {
    let _span = crate::perf::span("sync_casters");
    let characters_only = !settings.world_shadows;
    if *last == Some(characters_only) && added.is_empty() {
        return;
    }
    *last = Some(characters_only);
    for (e, managed, off) in &meshes {
        let character = std::iter::successors(Some(e), |&x| parents.get(x).ok().map(|p| p.parent())).any(|x| roots.contains(x));
        let want_off = characters_only && !character;
        if want_off && !off {
            commands.entity(e).insert((bevy::light::NotShadowCaster, CasterOff));
        } else if !want_off && managed && !faint.contains(e) {
            commands.entity(e).remove::<(bevy::light::NotShadowCaster, CasterOff)>();
        }
    }
}

const WGSL: &str = r#"
#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> amount: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(amount.rgb, 1.0);
}
"#;
