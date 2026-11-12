//! Water reflections (this build's option, the Display page's Water reflections; not in the
//! original game).
//! A planar mirror: a second 3D camera renders the scene from the main camera's position mirrored
//! under the nearest flat water surface into a texture, with Bevy's oblique near plane clipping
//! everything below the water (so the river bed does not block the view). Each flat water
//! surface (a scrolling, blended scene part whose normals point up: SPEC-0046 level water) gets
//! a twin drawn over it in its own render layer, which blends that texture in by a Fresnel term
//! (stronger at grazing angles, as on real water), with a slight ripple. Bevy's screen-space
//! reflections need its deferred renderer, which the game's forward materials do not use.
//! NEOPETS_WATER_REFLECT="strength,divisor,every" turns it on whatever the setting, for tuning:
//! the strength (default 1.2; 9 or more paints the twins magenta), the mirror's size as the
//! scene's divided by `divisor` (default 2), and a new mirror picture every `every` frames
//! (default 1).
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureFormat};
use bevy::shader::ShaderRef;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};

/// The render layer of the reflection twins: seen by the main camera, not by the mirror.
pub const LAYER: usize = 6;

#[derive(Clone, Copy)]
pub struct Tuning {
    pub strength: f32,
    pub divisor: u32,
    pub every: u32,
    /// How far the mirror sees, in world units: what lies farther is culled from it.
    pub far: f32,
}

/// The tuning: NEOPETS_WATER_REFLECT's values, else the defaults; and whether that variable is set.
pub fn tuning() -> (Tuning, bool) {
    static S: std::sync::OnceLock<(Tuning, bool)> = std::sync::OnceLock::new();
    *S.get_or_init(|| {
        let v = std::env::var("NEOPETS_WATER_REFLECT").ok();
        let n: Vec<f32> = v.as_deref().unwrap_or("").split(',').filter_map(|t| t.trim().parse().ok()).collect();
        let at = |i: usize, d: f32| n.get(i).copied().unwrap_or(d);
        (Tuning { strength: at(0, 1.2), divisor: (at(1, 2.0) as u32).max(1), every: (at(2, 1.0) as u32).max(1), far: at(3, 0.0) }, v.is_some())
    })
}

fn on(display: &crate::display::DisplaySettings) -> bool {
    display.water_reflections || tuning().1
}

/// How near the water's edge the camera must be for the mirror to draw, in world units.
const NEAR: f32 = 150.0;

/// A reflection twin.
#[derive(Component)]
struct Twin;

/// The mirror camera.
#[derive(Component)]
pub struct MirrorCam;

/// A scrolling part already looked at.
#[derive(Component)]
struct Checked;

/// A flat water surface: its height and its extent across (centre x, z and radius).
#[derive(Component, Clone, Copy)]
struct Surface {
    height: f32,
    centre: Vec2,
    radius: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ReflectMaterial {
    /// x: strength.
    #[uniform(0)]
    params: Vec4,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>,
}

static SHADER: std::sync::OnceLock<Handle<Shader>> = std::sync::OnceLock::new();

impl Material for ReflectMaterial {
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
        Ok(())
    }
}

#[derive(Resource, Default)]
struct Mirror {
    camera: Option<Entity>,
    image: Option<Handle<Image>>,
    size: UVec2,
    material: Option<Handle<ReflectMaterial>>,
}

pub struct WaterReflectPlugin;

impl Plugin for WaterReflectPlugin {
    fn build(&self, app: &mut App) {
        let handle = app.world_mut().resource_mut::<Assets<Shader>>().add(Shader::from_wgsl(WGSL, "neopets/water_reflect.wgsl"));
        let _ = SHADER.set(handle);
        app.add_plugins(MaterialPlugin::<ReflectMaterial>::default())
            .init_resource::<Mirror>()
            .add_systems(Update, find_water)
            .add_systems(PostUpdate, place_mirror.before(bevy::transform::TransformSystems::Propagate));
    }
}

/// New scrolling parts: a flat, blended one is water and gets its reflection twin.
#[allow(clippy::type_complexity)]
fn find_water(
    mut commands: Commands,
    mut mirror: ResMut<Mirror>,
    mut images: ResMut<Assets<Image>>,
    mut reflect: ResMut<Assets<ReflectMaterial>>,
    display: Res<crate::display::DisplaySettings>,
    parts: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, &GlobalTransform, Option<&crate::draw_order::DrawRank>), (With<crate::UvScroll>, Without<Checked>)>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    if !on(&display) {
        return;
    }
    for (e, mesh, material, at, rank) in &parts {
        let Some(m) = meshes.get(&mesh.0) else { continue };
        commands.entity(e).insert(Checked);
        if !materials.get(&material.0).is_some_and(|x| matches!(x.alpha_mode, AlphaMode::Blend)) {
            continue;
        }
        let (Some(bevy::mesh::VertexAttributeValues::Float32x3(p)), Some(bevy::mesh::VertexAttributeValues::Float32x3(n))) = (m.attribute(Mesh::ATTRIBUTE_POSITION), m.attribute(Mesh::ATTRIBUTE_NORMAL)) else { continue };
        if p.is_empty() || n.iter().map(|v| v[1]).sum::<f32>() / (n.len() as f32) < 0.9 {
            continue;
        }
        let world: Vec<Vec3> = p.iter().map(|q| at.transform_point(Vec3::from(*q))).collect();
        let (lo, hi) = world.iter().fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(a, b), q| (a.min(*q), b.max(*q)));
        if hi.y - lo.y > 2.0 {
            continue;
        }
        let image = mirror.image.get_or_insert_with(|| images.add(Image::new_target_texture(16, 16, TextureFormat::Rgba8UnormSrgb, None))).clone();
        let mat = mirror.material.get_or_insert_with(|| reflect.add(ReflectMaterial { params: Vec4::new(tuning().0.strength, 0.0, 0.0, 0.0), image })).clone();
        let centre = (lo + hi) / 2.0;
        // NEOPETS_TRACE_WATER: each water surface found (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_WATER").is_some() {
            println!("water surface at ({:.0}, {:.1}, {:.0}), {:.0} x {:.0}", centre.x, centre.y, centre.z, hi.x - lo.x, hi.z - lo.z);
        }
        commands.entity(e).insert(Surface { height: (lo.y + hi.y) / 2.0, centre: Vec2::new(centre.x, centre.z), radius: (Vec2::new(hi.x - lo.x, hi.z - lo.z) / 2.0).length() });
        // The twin draws straight after its water: the see-through parts keep the original's
        // order (draw_order.rs), whose even ranks leave the next odd one for a part's second pass.
        let mut twin = commands.spawn((Mesh3d(mesh.0.clone()), MeshMaterial3d(mat), Transform::IDENTITY, RenderLayers::layer(LAYER), bevy::light::NotShadowCaster, ChildOf(e), Twin));
        if let Some(r) = rank {
            twin.insert(crate::draw_order::DrawRank { pack: r.pack, rank: r.rank + 1 });
        }
    }
}

/// Each frame: the mirror under the nearest drawn water surface, or off.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn place_mirror(
    mut commands: Commands,
    mut mirror: ResMut<Mirror>,
    mut images: ResMut<Assets<Image>>,
    mut reflect: ResMut<Assets<ReflectMaterial>>,
    mains: Query<(Entity, &Camera, &Transform, &Projection, Option<&AmbientLight>, Option<&RenderLayers>), (With<Camera3d>, Without<MirrorCam>)>,
    mut mirrors: Query<(&mut Camera, &mut Transform, &mut Projection), With<MirrorCam>>,
    surfaces: Query<(&Surface, &ViewVisibility)>,
    display: Res<crate::display::DisplaySettings>,
    mut twins: Query<&mut Visibility, With<Twin>>,
    mut frame: Local<u32>,
    mut near_water: Local<bool>,
) {
    let enabled = on(&display);
    // The twins show only while the mirror draws (a stale picture would show otherwise).
    let mut show_twins = |shown: bool| {
        let tw = if shown { Visibility::Inherited } else { Visibility::Hidden };
        for mut v in &mut twins {
            if *v != tw {
                *v = tw;
            }
        }
    };
    if !enabled {
        show_twins(false);
    }
    let Some((main, cam, at, projection, ambient, layers)) = mains.iter().find(|(_, c, ..)| c.order == 0) else { return };
    if layers.is_none() {
        commands.entity(main).insert(RenderLayers::from_layers(&[0, LAYER]));
    }
    let Some(image) = mirror.image.clone() else { return };
    // Half the scene's size, in its shape.
    let size = cam.physical_target_size().map_or(UVec2::new(16, 16), |s| (s / tuning().0.divisor).max(UVec2::splat(16)));
    let resized = mirror.size != size;
    if resized {
        mirror.size = size;
        let handle = images.add(Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None));
        images.remove(&image);
        mirror.image = Some(handle.clone());
        if let Some(mut m) = mirror.material.as_ref().and_then(|m| reflect.get_mut(m)) {
            m.image = handle;
        }
    }
    let Some(target) = mirror.image.clone() else { return };
    let Some(camera) = mirror.camera else {
        let e = commands
            .spawn((
                Camera3d::default(),
                Camera { order: -1, is_active: false, ..default() },
                RenderTarget::from(target),
                bevy::core_pipeline::tonemapping::Tonemapping::None,
                Msaa::Off,
                bevy::light::cluster::ClusterConfig::Single,
                Projection::Perspective(PerspectiveProjection::default()),
                Transform::default(),
                RenderLayers::layer(0),
                MirrorCam,
            ))
            .id();
        mirror.camera = Some(e);
        return;
    };
    if resized {
        commands.entity(camera).insert(RenderTarget::from(target));
    }
    if let Some(a) = ambient {
        commands.entity(camera).insert(a.clone());
    }
    let Ok((mut mc, mut mt, mut mp)) = mirrors.get_mut(camera) else { return };
    // Off, or between the frames that draw a new mirror picture: the mirror does not render.
    *frame = frame.wrapping_add(1);
    if !enabled || *frame % tuning().0.every != 0 {
        if mc.is_active {
            mc.is_active = false;
        }
        return;
    }
    // The nearest drawn surface below the camera.
    let eye = at.translation;
    let near = surfaces
        .iter()
        .filter(|(s, v)| v.get() && s.height < eye.y)
        .min_by(|(a, _), (b, _)| (Vec2::new(eye.x, eye.z).distance(a.centre) - a.radius).total_cmp(&(Vec2::new(eye.x, eye.z).distance(b.centre) - b.radius)));
    // The mirror draws only within NEAR of the water's edge (off past NEAR + 20, so it does not
    // flicker at the edge): its cost is a second view of the scene, about 1.1 ms a frame whatever
    // its size, and water farther off is a sliver whose reflection cannot be told apart.
    let gap = near.map(|(s, _)| Vec2::new(eye.x, eye.z).distance(s.centre) - s.radius);
    *near_water = gap.is_some_and(|g| g < if *near_water { NEAR + 20.0 } else { NEAR });
    let (Some((s, _)), true) = (near, *near_water) else {
        if mc.is_active {
            mc.is_active = false;
        }
        show_twins(false);
        return;
    };
    if !mc.is_active {
        mc.is_active = true;
    }
    show_twins(true);
    let h = s.height;
    // NEOPETS_TRACE_WATER: the mirror's surface and size about once a second (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_WATER").is_some() && (eye.x * 7.0 + eye.z).fract().abs() < 1.0 {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        if N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 60 == 0 {
            println!("mirror: water at {h:.1} centre {:?}, eye {eye:?}, target {:?}, active {}", s.centre, mirror.size, mc.is_active);
        }
    }
    let f = at.forward().as_vec3();
    let u = at.up().as_vec3();
    let flip = |v: Vec3| Vec3::new(v.x, -v.y, v.z);
    let t = Transform::from_translation(Vec3::new(eye.x, 2.0 * h - eye.y, eye.z)).looking_to(flip(f), flip(u));
    // Bevy's oblique near plane: the water plane in the mirror's view space, its normal up (away
    // from the mirror, toward what it keeps), a little above the surface so the water itself is
    // left out of its own reflection.
    let view = t.to_matrix().inverse();
    let n = (t.rotation.inverse() * Vec3::Y).normalize();
    let q = view.transform_point3(Vec3::new(eye.x, h + 0.1, eye.z));
    let plane = n.extend(-n.dot(q));
    let mut p = match projection {
        Projection::Perspective(p) => p.clone(),
        _ => PerspectiveProjection::default(),
    };
    p.near_clip_plane = plane;
    if tuning().0.far > 0.0 {
        p.far = tuning().0.far;
    }
    *mt = t;
    *mp = Projection::Perspective(p);
}

const WGSL: &str = r#"
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::{view, globals}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var mirror: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var mirror_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // A strength of 9 or more paints the twins magenta (diagnostic).
    if params.x >= 9.0 {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }
    // The mirror was drawn with a plain (not mirrored) camera, so its picture is the reflection
    // flipped left to right.
    var uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    uv.x = 1.0 - uv.x;
    // A slight moving ripple.
    let w = in.world_position.xz;
    let t = globals.time;
    uv += vec2<f32>(sin(w.x * 0.9 + t * 1.6) + sin(w.y * 1.3 - t * 1.1), cos(w.y * 0.8 + t * 1.4) + cos(w.x * 1.1 + t * 0.9)) * 0.0025;
    let r = textureSample(mirror, mirror_sampler, clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999))).rgb;
    // Fresnel (Schlick): more reflection at grazing angles.
    let v = normalize(view.world_position - in.world_position.xyz);
    let c = clamp(abs(v.y), 0.0, 1.0);
    let fres = 0.15 + 0.85 * pow(1.0 - c, 4.0);
    return vec4<f32>(r, clamp(params.x * fres, 0.0, 1.0));
}
"#;
