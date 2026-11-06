//! Motion-record physics of a character (SPEC-0040 Behavior 14A items 5 to 7): the ground
//! constraint, gravity with the gain G, friction on the ground and the integrator. Velocity is in
//! units per 1/30 step; one call of each step is one game tick (SPEC-0007, 60 Hz).
//! Not covered: moving platforms (item 6b), the alternate friction branch, the clear-on-ground
//! and no-snap flags. The floor-contact flag is set by the caller's contacts (`floor_contact`).

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
/// SPEC-0040 Layout, body size: both heroes are 3.5 high (the camera's focus height).
pub const HERO_HEIGHT: f32 = 3.5;
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
    /// The no-snap flag (+0x148, SPEC-0040 Behavior 14A item 5b): the ground constraint clears
    /// the motion along the normal but does not move the record onto the ground.
    pub no_snap: bool,
    /// Ground gap (+0x24) and ground normal (+0x10), measured at the start of the record tick.
    pub gap: f32,
    pub normal: Vec3,
    /// +0x150, the floor-contact flag (SPEC-0033 Floor-contact flag): cleared at the start of
    /// each record tick, set by a contact against a surface with |n.y| above 0.9; the gap reads
    /// 0 while it is set.
    pub floor: bool,
    /// SPEC-0040 Behavior 14A item 6b: the vertical displacement this tick of the moving placed
    /// object the record stands on (its platform pointer is set), or None on ordinary ground.
    pub platform_dy: Option<f32>,
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
            no_snap: false,
            gap: 0.0,
            normal: [0.0, 1.0, 0.0],
            floor: false,
            platform_dy: None,
        }
    }

    /// A contact resolved against a floor (|n.y| > 0.9, pushing up): the flag is set, the gap
    /// reads 0, and the downward parts of the velocity and the force are removed (the contact
    /// response's into-face parts).
    pub fn floor_contact(&mut self) {
        self.floor = true;
        self.gap = 0.0;
        self.velocity[1] = self.velocity[1].max(0.0);
        self.force[1] = self.force[1].max(0.0);
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
        if let Some(d) = self.platform_dy {
            return self.constrain_platform(d);
        }
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
        if self.no_snap {
            return;
        }
        let d = self.gap * n[1];
        for k in 0..3 {
            self.position[k] -= n[k] * d;
        }
        self.gap = 0.0;
    }

    /// Item 6b, the moving platform: with the platform's vertical displacement d this tick,
    /// c' = gap + d + (vy + (Fy / W) k - d / k) k. At 0.1 or more nothing changes; between 0 and
    /// 0.1, vy = 1.05 d / k; at 0 or less the velocity becomes (0, 0.95 d / k, 0) (1.05 when d is
    /// below 0), F loses its normal part and the record is put on the platform top.
    fn constrain_platform(&mut self, d: f32) {
        let k = TIME_SCALE;
        let c = self.gap + d + (self.velocity[1] + self.force[1] / self.weight * k - d / k) * k;
        if c >= 0.1 {
            return;
        }
        if c > 0.0 {
            self.velocity[1] = 1.05 * d / k;
            return;
        }
        let f = if d < 0.0 { 1.05 } else { 0.95 };
        self.velocity = [0.0, f * d / k, 0.0];
        let n = self.normal;
        let fn_ = dot(self.force, n);
        for i in 0..3 {
            self.force[i] -= fn_ * n[i];
        }
        if !self.no_snap {
            let g = self.gap * n[1];
            for i in 0..3 {
                self.position[i] -= n[i] * g;
            }
            self.gap = 0.0;
        }
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

    #[test]
    fn sinking_platform_vector() {
        // SPEC-0040 item 6b test vector: pack 1382 object 27 sinking 0.2217 a tick, the hero on it
        // with gap 0, vy 0 and no force: c' = 0, so vy = 1.05 x -6.65 = -6.98 and he sinks with it.
        let mut b = Body::new([0.0; 3], HERO_WEIGHT);
        b.platform_dy = Some(-0.2217);
        b.constrain();
        assert!((b.velocity[1] - 1.05 * -0.2217 * 30.0).abs() < 1e-3, "{:?}", b.velocity);
        // Rising 0.1 a tick from rest: c' = 0.1 + 0 - 0.1 = 0, the velocity becomes 0.95 x 3.0.
        let mut b = Body::new([0.0; 3], HERO_WEIGHT);
        b.platform_dy = Some(0.1);
        b.constrain();
        assert!((b.velocity[1] - 0.95 * 3.0).abs() < 1e-3);
        // Well above it: nothing changes.
        let mut b = Body::new([0.0; 3], HERO_WEIGHT);
        b.gap = 1.0;
        b.platform_dy = Some(-0.1);
        b.constrain();
        assert_eq!(b.velocity, [0.0; 3]);
    }

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
