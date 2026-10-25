//! The lit draw (SPEC-0046, Light slots and the lit draw; Scene default lights): an ambient
//! colour plus three directional lights, applied per vertex by the lit programs 1, 2 and 6.
//! The default rig is a set of code constants of the original (SPEC-0046 says they are not
//! disc data). Character light sets from the area (type 30) are not applied yet.

/// A light context after its slots are filled: ambient A and, per directional slot, the unit
/// direction toward the light (the negated travel direction) and its weighted colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lights {
    pub ambient: [f32; 3],
    pub directional: [([f32; 3], [f32; 3]); 3],
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

impl Lights {
    /// SPEC-0046 Scene default lights: ambient (50, 50, 50) and three type-2 lights of strength
    /// 0.25 with the travel directions and colours of its table.
    pub fn scene_default() -> Self {
        let slot = |travel: [f32; 3], colour: [f32; 3]| {
            (normalise(travel.map(|c| -c)), colour.map(|c| c * 0.25))
        };
        Self {
            ambient: [50.0; 3],
            directional: [
                slot([0.666, -0.524, 0.531], [255.0, 255.0, 195.0]),
                slot([0.193, -0.598, -0.778], [205.0, 205.0, 255.0]),
                slot([-0.649, -0.665, 0.368], [205.0, 205.0, 255.0]),
            ],
        }
    }

    /// The lit colour L for a normal, per RGB channel in 0–255 units.
    pub fn lit(&self, n: [f32; 3]) -> [f32; 3] {
        let mut l = self.ambient;
        for (t, c) in &self.directional {
            let d = (n[0] * t[0] + n[1] * t[1] + n[2] * t[2]).clamp(0.0, 1.0);
            for k in 0..3 {
                l[k] += d * c[k];
            }
        }
        l
    }

    /// Lit vertex output (programs 1, 2 and 6, draw colour scale 1): RGB = min(255, L × byte
    /// / 16) and alpha = min(255, 128 × alpha byte / 16), truncated.
    pub fn vertex(&self, n: [f32; 3], colour: [u8; 4]) -> [u8; 4] {
        let l = self.lit(n);
        let ch = |k: usize| (l[k] * colour[k] as f32 / 16.0).min(255.0) as u8;
        [ch(0), ch(1), ch(2), (128.0 * colour[3] as f32 / 16.0).min(255.0) as u8]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_lights_match_spec_vectors() {
        let lights = Lights::scene_default();
        // SPEC-0004: normal up with bytes 16 gives (148, 148, 156, 128); normal down gives the
        // ambient alone.
        assert_eq!(lights.vertex([0.0, 1.0, 0.0], [16; 4]), [148, 148, 156, 128]);
        assert_eq!(lights.vertex([0.0, -1.0, 0.0], [16; 4])[..3], [50, 50, 50]);
        // SPEC-0046: the slot colours are strength × colour.
        assert_eq!(lights.directional[0].1, [63.75, 63.75, 48.75]);
    }

    #[test]
    fn output_saturates_at_255() {
        let lights = Lights::scene_default();
        assert_eq!(lights.vertex([0.0, 1.0, 0.0], [128, 128, 128, 128]), [255; 4]);
    }
}
