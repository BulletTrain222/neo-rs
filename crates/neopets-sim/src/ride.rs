//! Riding a moving placed object (SPEC-0018 Behavior, the ground attach's carry; SPEC-0028
//! Riding, carrying and collision with moving placed objects): a character standing on a patch of
//! a placed object's inline height-patch set is moved each tick by the object's translation change
//! plus a rotation term, with x and z clamped to 1.5, and its facing is turned by the relative
//! platform orientation.

use crate::hero::Vec3;

/// A quaternion with its lanes in the order x, y, z, w.
pub type Quat = [f32; 4];

/// rotate(q, v) = q v q⁻¹.
pub fn rotate(q: Quat, v: Vec3) -> Vec3 {
    let [x, y, z, w] = q;
    let u = [x, y, z];
    let cross = |a: Vec3, b: Vec3| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let t = cross(u, v).map(|c| 2.0 * c);
    let c = cross(u, t);
    std::array::from_fn(|k| v[k] + w * t[k] + c[k])
}

pub fn conjugate(q: Quat) -> Quat {
    [-q[0], -q[1], -q[2], q[3]]
}

/// The rotation quaternion of a column-major 4 x 4 transform (its columns normalised to remove
/// the scale).
pub fn rotation_of(m: &[f32; 16]) -> Quat {
    let col = |k: usize| {
        let c = [m[4 * k], m[4 * k + 1], m[4 * k + 2]];
        let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-12);
        c.map(|v| v / l)
    };
    let (c0, c1, c2) = (col(0), col(1), col(2));
    // r[row][col]
    let r = |i: usize, j: usize| [c0, c1, c2][j][i];
    let tr = r(0, 0) + r(1, 1) + r(2, 2);
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [(r(2, 1) - r(1, 2)) / s, (r(0, 2) - r(2, 0)) / s, (r(1, 0) - r(0, 1)) / s, 0.25 * s]
    } else if r(0, 0) > r(1, 1) && r(0, 0) > r(2, 2) {
        let s = (1.0 + r(0, 0) - r(1, 1) - r(2, 2)).sqrt() * 2.0;
        [0.25 * s, (r(0, 1) + r(1, 0)) / s, (r(0, 2) + r(2, 0)) / s, (r(2, 1) - r(1, 2)) / s]
    } else if r(1, 1) > r(2, 2) {
        let s = (1.0 + r(1, 1) - r(0, 0) - r(2, 2)).sqrt() * 2.0;
        [(r(0, 1) + r(1, 0)) / s, 0.25 * s, (r(1, 2) + r(2, 1)) / s, (r(0, 2) - r(2, 0)) / s]
    } else {
        let s = (1.0 + r(2, 2) - r(0, 0) - r(1, 1)).sqrt() * 2.0;
        [(r(0, 2) + r(2, 0)) / s, (r(1, 2) + r(2, 1)) / s, 0.25 * s, (r(1, 0) - r(0, 1)) / s]
    };
    let l = q.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-12);
    q.map(|v| v / l)
}

/// The carry of one tick: the displacement to add to the character (x and z clamped to 1.5; y
/// only when `vertical`), and the turned facing. `m` is the object's current transform,
/// `q_prev` the rotation saved on the previous tick, `t_prev` its previous translation.
pub fn carry(p: Vec3, facing: Vec3, m: &[f32; 16], q_prev: Quat, t_prev: Vec3, vertical: bool) -> (Vec3, Vec3) {
    let q = rotation_of(m);
    let t = [m[12], m[13], m[14]];
    let d = [p[0] - t[0], p[1] - t[1], p[2] - t[2]];
    let c = |k: usize| [m[4 * k], m[4 * k + 1], m[4 * k + 2]];
    let (c0, c1, c2) = (c(0), c(1), c(2));
    let l: Vec3 = std::array::from_fn(|k| c0[k] * d[0] + c1[k] * d[1] + c2[k] * d[2]);
    let (a, b) = (rotate(q_prev, l), rotate(q, l));
    let mut out: Vec3 = std::array::from_fn(|k| a[k] - b[k] + (t[k] - t_prev[k]));
    out[0] = out[0].clamp(-1.5, 1.5);
    out[2] = out[2].clamp(-1.5, 1.5);
    if !vertical {
        out[1] = 0.0;
    }
    let f = rotate(q_prev, rotate(conjugate(q), facing));
    (out, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaw(a: f32) -> [f32; 16] {
        // A +Y yaw: (x, z) -> (x cos a + z sin a, -x sin a + z cos a), columns as the transform.
        let (s, c) = a.sin_cos();
        [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0]
    }

    #[test]
    fn translation_carries_and_clamps() {
        let ident = [0.0, 0.0, 0.0, 1.0];
        let mut m = yaw(0.0);
        m[12] = 0.5;
        let (d, f) = carry([3.0, 0.0, 1.0], [1.0, 0.0, 0.0], &m, ident, [0.0; 3], false);
        assert!((d[0] - 0.5).abs() < 1e-6 && d[1] == 0.0 && d[2].abs() < 1e-6, "{d:?}");
        assert!((f[0] - 1.0).abs() < 1e-6);
        m[12] = 2.0;
        m[13] = 0.5;
        let (d, _) = carry([3.0, 0.0, 1.0], [1.0, 0.0, 0.0], &m, ident, [0.0; 3], false);
        assert!((d[0] - 1.5).abs() < 1e-6 && d[1] == 0.0);
        let (d, _) = carry([3.0, 0.0, 1.0], [1.0, 0.0, 0.0], &m, ident, [0.0; 3], true);
        assert!((d[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn facing_turns_opposite_to_the_platform() {
        // qprev identity, q a +90-degree yaw: a +X facing becomes +Z (SPEC-0018 facing test).
        let (_, f) = carry([0.0; 3], [1.0, 0.0, 0.0], &yaw(std::f32::consts::FRAC_PI_2), [0.0, 0.0, 0.0, 1.0], [0.0; 3], false);
        assert!(f[0].abs() < 1e-5 && (f[2] - 1.0).abs() < 1e-5, "{f:?}");
    }

    #[test]
    fn yaw_only_rotation_term() {
        // D = R(a0 + a1) d - R(2 a1) d for a yaw-only platform, angles a0 then a1 (spec formula).
        let (a0, a1) = (0.3f32, 0.5f32);
        let q0 = rotation_of(&yaw(a0));
        let d = [2.0, 0.0, 1.0];
        let (got, _) = carry(d, [1.0, 0.0, 0.0], &yaw(a1), q0, [0.0; 3], false);
        let r = |a: f32, v: [f32; 3]| [v[0] * a.cos() + v[2] * a.sin(), 0.0, -v[0] * a.sin() + v[2] * a.cos()];
        let (x, y) = (r(a0 + a1, d), r(2.0 * a1, d));
        assert!((got[0] - (x[0] - y[0])).abs() < 1e-4 && (got[2] - (x[2] - y[2])).abs() < 1e-4, "{got:?}");
        // Equal angles: no carry.
        let (z, _) = carry(d, [1.0, 0.0, 0.0], &yaw(a1), rotation_of(&yaw(a1)), [0.0; 3], false);
        assert!(z.iter().all(|v| v.abs() < 1e-5));
    }
}
