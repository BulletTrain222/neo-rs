//! A stand-in of this build (user report 162, the user's choice, 2026-11-20): the Plague
//! Serpent's two mesmerize beams. SPEC-0016 (bite sub 1) says two beam effects run from the
//! serpent's hand joints to the hero's chest while the mesmerize runs, but no spec names their
//! effect definitions yet (request R-0286). Until it does, each beam is drawn as a column of
//! thin blue rings travelling from the hand to the hero and widening as they go, after the
//! user's picture of the original. Replace this with the disc's effects once R-0286 is answered.

use bevy::prelude::*;

use crate::PlayState;

/// Rings per beam.
const RINGS: usize = 16;
/// Ticks for a ring to travel the whole beam.
const TRAVEL_TICKS: f32 = 150.0;
/// Ring radius at the hand and at the hero.
const RADIUS: (f32, f32) = (0.35, 1.35);

/// A beam ring: its beam (0 or 1) and its place in the column.
#[derive(Component)]
pub struct BeamRing(usize, usize);

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(Torus { minor_radius: 0.022, major_radius: 1.0 });
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.26, 1.0, 0.8),
        emissive: LinearRgba::rgb(0.25, 0.3, 2.2),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        ..default()
    });
    for beam in 0..2 {
        for k in 0..RINGS {
            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden, BeamRing(beam, k), bevy::light::NotShadowCaster));
        }
    }
}

/// Place the rings along the two beams each frame (hidden while no mesmerize beam runs). The
/// rings move with the game's ticks, so a paused game holds them still.
pub fn sync(state: Option<Res<PlayState>>, mut rings: Query<(&BeamRing, &mut Transform, &mut Visibility)>) {
    let beams = state.as_ref().and_then(|s| s.play.mesmerize_beams());
    for (ring, mut t, mut v) in &mut rings {
        let placed = beams.and_then(|(a, b, chest, ticks)| {
            let from = Vec3::from(if ring.0 == 0 { a } else { b });
            let d = Vec3::from(chest) - from;
            let len = d.length();
            (len > 0.01).then(|| {
                // The second beam's rings sit half a step behind the first's, so the two columns
                // do not pulse together.
                let phase = (ticks as f32 / TRAVEL_TICKS * RINGS as f32 + 0.5 * ring.0 as f32).fract();
                let s = (ring.1 as f32 + phase) / RINGS as f32;
                let radius = RADIUS.0 + (RADIUS.1 - RADIUS.0) * s;
                Transform { translation: from + d * s, rotation: Quat::from_rotation_arc(Vec3::Y, d / len), scale: Vec3::splat(radius) }
            })
        });
        match placed {
            Some(p) => {
                *t = p;
                if *v != Visibility::Visible {
                    *v = Visibility::Visible;
                }
            }
            None if *v != Visibility::Hidden => *v = Visibility::Hidden,
            None => {}
        }
    }
}
