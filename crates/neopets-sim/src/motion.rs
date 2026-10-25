//! Motion-record physics of a character (SPEC-0040 Behavior 14A items 5 to 7): the ground
//! constraint, gravity with the gain G, friction on the ground and the integrator. Velocity is in
//! units per 1/30 step; one call of each step is one game tick (SPEC-0007, 60 Hz).
//! Not covered: moving platforms (item 6b), the alternate friction branch, the clear-on-ground
//! and no-snap flags, and the floor-contact flag.

use crate::hero::Vec3;

/// The physics block's time scale k (1/30) and gravity g (32).
pub const TIME_SCALE: f32 = 1.0 / 30.0;
pub const GRAVITY: f32 = 32.0;
/// Character records: friction mu 0.6, penetration tolerance -0.1, grounded below 0.1.
const FRICTION: f32 = 0.6;
const TOLERANCE: f32 = -0.1;
const GROUNDED: f32 = 0.1;
/// SPEC-0040 Layout, body size: both heroes weigh 5.5.
pub const HERO_WEIGHT: f32 = 5.5;
/// SPEC-0018: the character ground step probes 2.5 above the record's height.
pub const PROBE_CEILING: f32 = 2.5;

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub position: Vec3,
    pub velocity: Vec3,
    force: Vec3,
    pub weight: f32,
    /// Gravity gain G (1.0 after reset).
    pub gain: f32,
    pub no_gravity: bool,
    /// Ground gap (+0x24) and ground normal (+0x10), measured at the start of the record tick.
    pub gap: f32,
    pub normal: Vec3,
}

impl Body {
    pub fn new(position: Vec3, weight: f32) -> Self {
        Self {
            position,
            velocity: [0.0; 3],
            force: [0.0; 3],
            weight,
            gain: 1.0,
            no_gravity: false,
            gap: 0.0,
            normal: [0.0, 1.0, 0.0],
        }
    }

    /// The ground step (SPEC-0040 Behavior 14 cadence, item 2): `probe(x, z, ceiling)` gives the
    /// ground height and normal; the gap is the height above it.
    pub fn measure(&mut self, probe: impl Fn(f32, f32, f32) -> (f32, Vec3)) {
        let p = self.position;
        let (h, n) = probe(p[0], p[2], p[1] + PROBE_CEILING);
        self.gap = p[1] - h;
        self.normal = n;
    }

    /// c = gap + (vy + (Fy / W) k) k: the gap predicted after this tick's integration.
    fn predicted_gap(&self) -> f32 {
        self.gap + (self.velocity[1] + self.force[1] / self.weight * TIME_SCALE) * TIME_SCALE
    }

    /// Item 5b, ground constraint: when the predicted gap is at or below the tolerance, the
    /// force and velocity lose their normal parts (all velocity if it still rises) and the
    /// record moves onto the ground plane.
    pub fn constrain(&mut self) {
        if self.predicted_gap() > TOLERANCE {
            return;
        }
        let n = self.normal;
        let (f, v) = (dot(self.force, n), dot(self.velocity, n));
        for k in 0..3 {
            self.force[k] -= f * n[k];
            self.velocity[k] -= v * n[k];
        }
        if self.velocity[1] > 0.0 {
            self.velocity = [0.0; 3];
        }
        let d = self.gap * n[1];
        for k in 0..3 {
            self.position[k] -= n[k] * d;
        }
        self.gap = 0.0;
    }

    /// Item 5, gravity a = -(W g G) up; on the ground it is projected onto the ground plane and
    /// friction (item 5c) acts. Then F += a.
    pub fn gravity(&mut self) {
        let mut a = [0.0, -self.weight * GRAVITY * self.gain, 0.0];
        if self.no_gravity {
            a[1] = 0.0;
        }
        if self.predicted_gap() <= GROUNDED {
            let n = self.normal;
            let an = dot(a, n);
            let at = [a[0] - an * n[0], a[1] - an * n[1], a[2] - an * n[2]];
            let k = TIME_SCALE / self.weight;
            let u: Vec3 = std::array::from_fn(|i| self.velocity[i] + (at[i] + self.force[i]) * k);
            let l = dot(u, u).sqrt();
            let mut f = FRICTION * an * k;
            if l < f.abs() {
                f = -l;
            }
            if l > 0.0 {
                for i in 0..3 {
                    self.velocity[i] += f * u[i] / l;
                }
            }
            a = at;
        }
        for k in 0..3 {
            self.force[k] += a[k];
        }
    }

    /// Item 6, integrator: v += (F / W) k, F = 0, p += v k.
    pub fn integrate(&mut self) {
        for k in 0..3 {
            self.velocity[k] += self.force[k] / self.weight * TIME_SCALE;
            self.force[k] = 0.0;
            self.position[k] += self.velocity[k] * TIME_SCALE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_: f32, _: f32, _: f32) -> (f32, Vec3) {
        (0.0, [0.0, 1.0, 0.0])
    }

    fn step(b: &mut Body) {
        b.measure(flat);
        b.constrain();
        b.gravity();
        b.integrate();
    }

    #[test]
    fn fall_law_matches_spec_drops() {
        // SPEC-0033 9A: from rest at gain 1.0 the drops are 0.0356, 0.1067, 0.2133.
        let mut b = Body::new([0.0, 10.0, 0.0], HERO_WEIGHT);
        for want in [0.0356, 0.1067, 0.2133] {
            step(&mut b);
            assert!((10.0 - b.position[1] - want).abs() < 1e-4, "{}", 10.0 - b.position[1]);
        }
    }

    #[test]
    fn standing_record_rests_on_flat_ground() {
        let mut b = Body::new([0.0; 3], HERO_WEIGHT);
        for _ in 0..10 {
            step(&mut b);
        }
        assert_eq!(b.position, [0.0; 3]);
        assert_eq!(b.velocity, [0.0; 3]);
    }

    #[test]
    fn landing_friction_and_snap() {
        // SPEC-0033 Behavior 50 flight: from y 0.2329 with vy -8.1567, the first grounded tick
        // gives vy -7.5167 and y -0.0177; the next snaps to 0 with vy 0.
        let mut b = Body::new([0.0, 0.2329, 0.0], HERO_WEIGHT);
        b.velocity[1] = -8.15667;
        step(&mut b);
        assert!((b.velocity[1] + 7.5167).abs() < 1e-3, "{}", b.velocity[1]);
        assert!((b.position[1] + 0.0177).abs() < 1e-3, "{}", b.position[1]);
        step(&mut b);
        assert!(b.position[1].abs() < 1e-6 && b.velocity[1] == 0.0, "{:?}", b);
    }
}
