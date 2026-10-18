//! Orbit camera for the asset viewers (M3/M4/M6). Viewer tooling only: this is not the
//! original game's camera, which needs its own spec.
use bevy::prelude::*;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

const PITCH_LIMIT: f32 = 89.0_f32.to_radians();
const MIN_DISTANCE: f32 = 0.5;
const MAX_DISTANCE: f32 = 500.0;
/// Radians of rotation per pixel of mouse drag.
const DRAG_SPEED: f32 = 0.005;
/// Fraction of the distance moved per scroll line.
const ZOOM_STEP: f32 = 0.1;

impl Default for Orbit {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: 35.0_f32.to_radians(),
            pitch: 25.0_f32.to_radians(),
            distance: 12.0,
        }
    }
}

impl Orbit {
    /// Dragging right turns the camera around the target to the right; dragging up raises it.
    pub fn drag(&mut self, pixels: Vec2) {
        self.yaw = (self.yaw - pixels.x * DRAG_SPEED).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + pixels.y * DRAG_SPEED).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Positive lines zoom in.
    pub fn zoom(&mut self, lines: f32) {
        let factor = (1.0 - ZOOM_STEP).powf(lines);
        self.distance = (self.distance * factor).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + self.distance * Vec3::new(cp * sy, sp, cp * cy)
    }

    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.eye()).looking_at(self.target, Vec3::Y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_keeps_distance_from_target() {
        let mut o = Orbit {
            target: Vec3::new(1.0, 2.0, 3.0),
            ..default()
        };
        for drag in [Vec2::new(300.0, 0.0), Vec2::new(-50.0, 120.0), Vec2::new(0.0, -999.0)] {
            o.drag(drag);
            assert!((o.eye().distance(o.target) - o.distance).abs() < 1e-4);
        }
    }

    #[test]
    fn pitch_stays_short_of_the_poles() {
        let mut o = Orbit::default();
        o.drag(Vec2::new(0.0, 1e6));
        assert_eq!(o.pitch, PITCH_LIMIT);
        o.drag(Vec2::new(0.0, -1e6));
        assert_eq!(o.pitch, -PITCH_LIMIT);
        // looking_at stays well-defined because the eye is never straight above the target.
        assert!(o.transform().rotation.is_finite());
    }

    #[test]
    fn yaw_wraps_and_zoom_clamps() {
        let mut o = Orbit::default();
        o.drag(Vec2::new(-1e5, 0.0));
        assert!((0.0..std::f32::consts::TAU).contains(&o.yaw));
        o.zoom(1000.0);
        assert_eq!(o.distance, MIN_DISTANCE);
        o.zoom(-1000.0);
        assert_eq!(o.distance, MAX_DISTANCE);
        let before = o.distance;
        o.zoom(1.0);
        assert!(o.distance < before);
    }
}
