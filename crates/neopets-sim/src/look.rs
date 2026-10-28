//! The hero's head look (SPEC-0040 Behavior 23A, mode 0): the stored yaw and pitch, stepped
//! toward the target by the rate cap 0.10 a call and clamped to the limits, and the look point
//! they give.

pub type Vec3 = [f32; 3];

/// The rate cap per call and the yaw and pitch limits (boot data, SPEC-0040 Where it lives).
pub const RATE: f32 = 0.10;
pub const YAW_LIMIT: f32 = 1.57;
pub const PITCH_LIMIT: f32 = 0.35;
/// The reference vector R (the up vector).
const UP: Vec3 = [0.0, 1.0, 0.0];

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn unit(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 1e-6 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0; 3] }
}
fn flat(a: Vec3) -> Vec3 {
    unit([a[0], 0.0, a[2]])
}

/// The neck joint list of mode 1 (six s32 words, joints 6, 7, 8, 11, 14 and 17; SPEC-0016 6A).
pub const NECK_JOINTS_AT: u32 = 0x0032_0F48;
/// The neck's step per tick.
pub const NECK_STEP: f32 = 0.01;

/// Mode 1, the Plague Serpent's neck (SPEC-0040 Behavior 23A; SPEC-0016 6A): A = acos(h_t . h_b)
/// with no clamp (a dot above 1 gives no number, and nothing moves), s = unit(h_b x up) . h_t,
/// tau = -A/6 when s > 0, else +A/6; each angle snaps to tau within 0.01, otherwise it may only
/// increase (s > 0) or only decrease (s <= 0) by 0.01 toward tau (the quirk: the allowed
/// direction points away from tau, so from 0 the neck moves only by the snap).
pub fn neck_step(angles: &mut [f32; 6], j6: Vec3, j17: Vec3, target: Vec3) {
    let ht = flat([target[0] - j6[0], 0.0, target[2] - j6[2]]);
    let hb = flat([j17[0] - j6[0], 0.0, j17[2] - j6[2]]);
    let a = dot(ht, hb).acos();
    let c = [hb[1] * UP[2] - hb[2] * UP[1], hb[2] * UP[0] - hb[0] * UP[2], hb[0] * UP[1] - hb[1] * UP[0]];
    let s = dot(unit(c), ht);
    let tau = if s > 0.0 { -a / 6.0 } else { a / 6.0 };
    for x in angles.iter_mut() {
        if (tau - *x).abs() <= NECK_STEP {
            *x = tau;
        } else if s > 0.0 && tau > *x {
            *x += NECK_STEP;
        } else if s <= 0.0 && tau < *x {
            *x -= NECK_STEP;
        }
    }
}

/// The hero's stored angles (player +0xF30 and +0xF34), kept between looks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

impl Look {
    /// One call (steps 1 to 7): `o` the head position, `t` the target, `f` and `x` the forward
    /// and side axes of the root frame (or of joint 0 with `reset`). Returns the look point
    /// O + 2.0 U.
    pub fn step(&mut self, o: Vec3, t: Vec3, f: Vec3, x: Vec3, reset: bool) -> Vec3 {
        let (fh, xh) = (flat(f), flat(x));
        let d = [t[0] - o[0], t[1] - o[1], t[2] - o[2]];
        let (dh, du) = (flat(d), unit(d));
        let mut yaw_t = dot(fh, dh).min(1.0).acos();
        if dot(xh, dh) < 0.0 {
            yaw_t = -yaw_t;
        }
        // The pitch target is negated when the target is above the head (the quirk).
        let mut pitch_t = dot(dh, du).min(1.0).acos();
        if dot(du, UP) > 0.0 {
            pitch_t = -pitch_t;
        }
        if reset {
            (self.yaw, self.pitch) = (0.0, 0.0);
        } else {
            let toward = |a: f32, t: f32| if (t - a).abs() <= RATE { t } else { a + RATE * (t - a).signum() };
            self.yaw = toward(self.yaw, yaw_t);
            self.pitch = toward(self.pitch, pitch_t);
        }
        self.yaw = self.yaw.clamp(-YAW_LIMIT, YAW_LIMIT);
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let (sy, cy) = self.yaw.sin_cos();
        let h = unit([fh[0] * cy + xh[0] * sy, fh[1] * cy + xh[1] * sy, fh[2] * cy + xh[2] * sy]);
        let (sp, cp) = self.pitch.sin_cos();
        let u = unit([h[0] * cp + UP[0] * sp, h[1] * cp + UP[1] * sp, h[2] * cp + UP[2] * sp]);
        [o[0] + 2.0 * u[0], o[1] + 2.0 * u[1], o[2] + 2.0 * u[2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: Vec3 = [0.0, 0.0, 1.0];
    const X: Vec3 = [1.0, 0.0, 0.0];

    #[test]
    fn steps_by_the_rate_cap_and_clamps() {
        let mut l = Look::default();
        // A target straight to the side (+X): the yaw climbs 0.10 a call to the limit 1.57.
        l.step([0.0; 3], [10.0, 0.0, 0.0], F, X, false);
        assert!((l.yaw - 0.10).abs() < 1e-6);
        for _ in 0..40 {
            l.step([0.0; 3], [-10.0, 0.0, -0.01], F, X, false);
        }
        assert!((l.yaw + YAW_LIMIT).abs() < 1e-6, "held at the limit: {}", l.yaw);
        // From one limit to the other takes at least 32 calls.
        let mut n = 0;
        while (l.yaw - YAW_LIMIT).abs() > 1e-6 {
            l.step([0.0; 3], [10.0, 0.0, -0.01], F, X, false);
            n += 1;
        }
        assert!(n >= 32);
    }

    #[test]
    fn neck_snaps_or_freezes() {
        // The hero nearly straight along the neck: A small, tau within 0.01, the snap.
        let mut n = [0.0; 6];
        neck_step(&mut n, [0.0; 3], [0.0, 0.0, 5.0], [0.3, 0.0, 10.0]);
        assert!(n.iter().all(|&x| x != 0.0 && x.abs() <= NECK_STEP));
        // Far to the side: tau more than 0.01 away and the allowed direction away from it.
        let mut n = [0.0; 6];
        neck_step(&mut n, [0.0; 3], [0.0, 0.0, 5.0], [10.0, 0.0, 0.0]);
        assert_eq!(n, [0.0; 6]);
    }

    #[test]
    fn pitch_quirk_and_reset() {
        let mut l = Look::default();
        // A target above: the pitch goes negative, so the look point drops below the head.
        for _ in 0..10 {
            l.step([0.0; 3], [0.0, 5.0, 5.0], F, X, false);
        }
        assert!((l.pitch + PITCH_LIMIT).abs() < 1e-6);
        let p = l.step([0.0; 3], [0.0, 5.0, 5.0], F, X, false);
        assert!(p[1] < 0.0 && p[2] > 0.0);
        // Reset: both angles 0, the point straight ahead.
        let p = l.step([0.0; 3], [9.0, 9.0, 9.0], F, X, true);
        assert_eq!((l.yaw, l.pitch), (0.0, 0.0));
        assert!((p[2] - 2.0).abs() < 1e-6);
    }
}
