//! Light shafts and haze (this build's option, the Display page's Light shafts; not in the
//! original game). Bevy's volumetric fog: the 3D camera raymarches a box of fog that follows it,
//! lit by the sun (shadow_view.rs `Sun`, tagged as a volumetric light), so where the sun's shadow
//! map has a caster (trees and buildings with World shadows on) the light comes through in
//! shafts, and the fog adds a haze toward the sun. The game's sun is faint (its exposure is matched
//! to the unlit colours), so the fog takes a light gain of about 15; at 30 or more the view washes
//! out. NEOPETS_GODRAYS="density,light,ambient,absorption,steps" (each optional) turns it on with
//! other values for tuning, whatever the setting.
use bevy::light::{FogVolume, VolumetricFog, VolumetricLight};
use bevy::prelude::*;

/// The fog box's size around the camera, in world units (across, height).
const BOX: (f32, f32) = (400.0, 160.0);

#[derive(Clone, Copy)]
pub struct Settings {
    pub density: f32,
    pub light: f32,
    pub ambient: f32,
    pub absorption: f32,
    pub steps: u32,
}

/// The tuning: NEOPETS_GODRAYS's values, else the defaults; and whether that variable is set.
pub fn tuning() -> (Settings, bool) {
    static S: std::sync::OnceLock<(Settings, bool)> = std::sync::OnceLock::new();
    *S.get_or_init(|| {
        let v = std::env::var("NEOPETS_GODRAYS").ok();
        let n: Vec<f32> = v.as_deref().unwrap_or("").split(',').filter_map(|t| t.trim().parse().ok()).collect();
        let at = |i: usize, d: f32| n.get(i).copied().unwrap_or(d);
        (Settings { density: at(0, 0.006), light: at(1, 15.0), ambient: at(2, 0.05), absorption: at(3, 0.01), steps: at(4, 16.0) as u32 }, v.is_some())
    })
}

pub struct GodRaysPlugin;

impl Plugin for GodRaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_godrays.before(bevy::transform::TransformSystems::Propagate));
    }
}

#[derive(Component)]
struct FogBox;

#[allow(clippy::type_complexity)]
fn sync_godrays(
    mut commands: Commands,
    display: Res<crate::display::DisplaySettings>,
    cameras: Query<(Entity, &Camera, &Transform, Has<VolumetricFog>), (With<Camera3d>, Without<FogBox>, Without<crate::water_reflect::MirrorCam>)>,
    suns: Query<(Entity, Has<VolumetricLight>), With<crate::shadow_view::Sun>>,
    mut boxes: Query<(Entity, &mut Transform), (With<FogBox>, Without<Camera3d>)>,
) {
    let (s, forced) = tuning();
    let on = display.light_shafts || forced;
    // The main view's camera (order 0; the book's camera draws over it).
    let Some((cam, _, at, has_fog)) = cameras.iter().find(|(_, c, _, _)| c.order == 0) else { return };
    if !on {
        if has_fog {
            commands.entity(cam).remove::<VolumetricFog>();
        }
        for (sun, has) in &suns {
            if has {
                commands.entity(sun).remove::<VolumetricLight>();
            }
        }
        for (e, _) in &boxes {
            commands.entity(e).despawn();
        }
        return;
    }
    if !has_fog {
        commands.entity(cam).insert(VolumetricFog { ambient_intensity: s.ambient, step_count: s.steps, ..default() });
    }
    for (sun, has) in &suns {
        if !has {
            commands.entity(sun).insert(VolumetricLight);
        }
    }
    let want = Transform::from_translation(at.translation).with_scale(Vec3::new(BOX.0, BOX.1, BOX.0));
    match boxes.single_mut() {
        Ok((_, mut t)) => {
            if *t != want {
                *t = want;
            }
        }
        Err(_) => {
            commands.spawn((
                FogVolume { density_factor: s.density, absorption: s.absorption, scattering: 0.6, fog_color: Color::srgb(1.0, 0.95, 0.85), light_tint: Color::srgb(1.0, 0.92, 0.75), scattering_asymmetry: 0.75, light_intensity: s.light, ..default() },
                want,
                FogBox,
            ));
        }
    }
}
