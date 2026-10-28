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

/// The shadow factor of each strength (linear colour): Original is a drop of 8 of 255 at mid
/// grey (128 to 120 in display values; SPEC-0046 step 8); Strong is this build's.
const FACTORS: [f32; 3] = [1.0, 0.87, 0.6];

pub struct ShadowPlugin;

impl Plugin for ShadowPlugin {
    fn build(&self, app: &mut App) {
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(Shader::from_wgsl(WGSL, "neopets/shadow_volume.wgsl"));
        let _ = SHADER.set(handle);
        app.add_plugins((MaterialPlugin::<ShadowSubtract>::default(), MaterialPlugin::<ShadowAdd>::default()));
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
        if let Some(mut m) = subtract.get_mut(s) {
            if m.amount != down {
                m.amount = down;
            }
        }
        if let Some(mut m) = add.get_mut(a) {
            if m.amount != up {
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

/// The world-shadow sun (this build's option): a directional light with shadow maps, on while
/// World shadows is on. Its direction is the strongest directional light of the hero's light
/// context (the area's own light, SPEC-0046 Character lights), held at least 0.4 downward,
/// else a default slant.
#[derive(Component)]
pub struct Sun;

pub fn sync_sun(mut commands: Commands, state: Option<Res<crate::PlayState>>, settings: Res<crate::display::DisplaySettings>, mut suns: Query<(&mut Transform, &mut Visibility), With<Sun>>, mut made: Local<bool>) {
    if !*made {
        *made = true;
        commands.spawn((
            DirectionalLight { illuminance: crate::graphics::sun_illuminance(), shadow_maps_enabled: true, ..default() },
            bevy::light::CascadeShadowConfigBuilder { maximum_distance: 160.0, first_cascade_far_bound: 25.0, ..default() }.build(),
            Transform::default(),
            Visibility::Hidden,
            Sun,
        ));
        return;
    }
    let on = settings.world_shadows && state.is_some();
    let dir = state.as_ref().and_then(|s| s.play.sun_direction()).unwrap_or([0.35, -1.0, 0.25]);
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

const WGSL: &str = r#"
#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> amount: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(amount.rgb, 1.0);
}
"#;
