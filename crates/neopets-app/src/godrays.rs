//! Light shafts and haze (an experiment of this build, not in the original game; user request).
//! Bevy's volumetric fog: the 3D camera raymarches a box of fog that follows it, lit by the sun
//! (shadow_view.rs `Sun`, tagged as a volumetric light), so where the sun's shadow map has a
//! caster (trees and buildings with World shadows on) the light comes through in shafts, and
//! the fog adds a haze with distance. Off unless NEOPETS_GODRAYS is set:
//! NEOPETS_GODRAYS="density,light,ambient,absorption" (each optional).
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
}

/// The experiment's settings, if on.
pub fn settings() -> Option<Settings> {
    static S: std::sync::OnceLock<Option<Settings>> = std::sync::OnceLock::new();
    *S.get_or_init(|| {
        let v = std::env::var("NEOPETS_GODRAYS").ok()?;
        let n: Vec<f32> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
        Some(Settings { density: n.first().copied().unwrap_or(0.006), light: n.get(1).copied().unwrap_or(15.0), ambient: n.get(2).copied().unwrap_or(0.05), absorption: n.get(3).copied().unwrap_or(0.01) })
    })
}

pub struct GodRaysPlugin;

impl Plugin for GodRaysPlugin {
    fn build(&self, app: &mut App) {
        if settings().is_some() {
            app.add_systems(PostUpdate, sync_godrays.before(bevy::transform::TransformSystems::Propagate));
        }
    }
}

#[derive(Component)]
struct FogBox;

#[allow(clippy::type_complexity)]
fn sync_godrays(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera, &Transform, Has<VolumetricFog>), (With<Camera3d>, Without<FogBox>)>,
    suns: Query<(Entity, Has<VolumetricLight>), With<crate::shadow_view::Sun>>,
    mut boxes: Query<&mut Transform, (With<FogBox>, Without<Camera3d>)>,
) {
    let Some(s) = settings() else { return };
    // The main view's camera (order 0; the book's camera draws over it).
    let Some((cam, _, at, has_fog)) = cameras.iter().find(|(_, c, _, _)| c.order == 0) else { return };
    if !has_fog {
        commands.entity(cam).insert(VolumetricFog { ambient_intensity: s.ambient, step_count: 64, ..default() });
    }
    for (sun, has) in &suns {
        if !has {
            commands.entity(sun).insert(VolumetricLight);
        }
    }
    let want = Transform::from_translation(at.translation).with_scale(Vec3::new(BOX.0, BOX.1, BOX.0));
    match boxes.single_mut() {
        Ok(mut t) => {
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
