//! Chain bodies, modes 1 and 2 (SPEC-0040 Behavior 25): the pendulum chains of Tor's tail and
//! Roberta's two head chains. Each link hangs from a joint of the sub-part joint list; each tick,
//! after the pose is built, its point is sprung toward a rest direction, carried by its own
//! velocity and the owner's motion, measured back into a clamped yaw and pitch, and written as
//! the joint's local rotation with local translation (0, 0, -a).
use crate::hero::Vec3;

/// A link's fixed constants (SPEC-0040 Behavior 25, Constants).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkConst {
    /// The distance from the link joint to this joint.
    pub a: f32,
    /// The distance from this joint to its point.
    pub l: f32,
    /// Rest yaw and pitch.
    pub alpha: f32,
    pub beta: f32,
    /// Spring.
    pub k: f32,
    pub yaw: [f32; 2],
    pub pitch: [f32; 2],
    /// Velocity keep.
    pub b: f32,
    /// Drag along the owner's motion.
    pub c: f32,
}

/// A link: its joint and the joint it hangs from (sub-part joint indexes), its constants, and
/// its previous and current points (None until seeded).
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub joint: usize,
    pub hangs_from: usize,
    pub k: LinkConst,
    pub points: Option<(Vec3, Vec3)>,
}

/// One chain's links, in step order.
pub type Chain = Vec<Link>;

/// The world frame of a link joint: axes X, Y, Z and position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub p: Vec3,
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn unit(a: Vec3) -> Option<Vec3> {
    let n = dot(a, a).sqrt();
    (n > 1e-12).then(|| scale(a, 1.0 / n))
}

/// The unit vector of a displacement, or zero when there is none (the owner's û).
pub fn motion_unit(d: Vec3) -> Vec3 {
    unit(d).unwrap_or([0.0; 3])
}

/// dir(θ, φ): -Z turned by pitch φ about X, then by yaw θ about Y, in the frame's axes.
fn dir(f: &Frame, theta: f32, phi: f32) -> Vec3 {
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    add(add(scale(f.x, -st * cp), scale(f.y, sp)), scale(f.z, -ct * cp))
}

/// One link's step (SPEC-0040 Behavior 25, Per-tick step, items 2 to 8) from its link joint's
/// world frame and the owner's motion unit vector. Returns the clamped yaw and pitch that
/// become the joint's local rotation. An unseeded link starts with both points on its rest
/// target (the clean seed the spec allows).
pub fn step_link(link: &mut Link, f: &Frame, u: Vec3) -> (f32, f32) {
    let k = link.k;
    let t = sub(f.p, scale(f.z, k.a));
    let x1 = add(t, scale(dir(f, k.alpha, k.beta), k.l));
    let (b, c) = link.points.unwrap_or((x1, x1));
    let spring = scale(sub(c, x1), -k.k);
    let x3 = add(add(add(c, scale(sub(c, b), k.b)), scale(u, k.c)), spring);
    let (theta, phi) = match unit(sub(x3, t)) {
        Some(d) => match unit(sub(d, scale(f.y, dot(d, f.y)))) {
            Some(p) => {
                let theta = dot(p, scale(f.z, -1.0)).min(1.0).acos();
                let theta = if dot(p, f.x) > 0.0 { -theta } else { theta };
                let phi = dot(p, d).min(1.0).acos();
                (theta, if dot(d, f.y) < 0.0 { -phi } else { phi })
            }
            None => (0.0, 0.0),
        },
        None => (0.0, 0.0),
    };
    let theta = theta.clamp(k.yaw[0], k.yaw[1]);
    let phi = phi.clamp(k.pitch[0], k.pitch[1]);
    let n = add(t, scale(dir(f, theta, phi), k.l));
    link.points = Some((c, n));
    (theta, phi)
}

/// The joint's local rotation q = q_yaw(Y) ⊗ q_pitch(X), as (x, y, z, w).
pub fn link_rotation(theta: f32, phi: f32) -> [f32; 4] {
    let (sy, cy) = (theta * 0.5).sin_cos();
    let (sx, cx) = (phi * 0.5).sin_cos();
    [cy * sx, sy * cx, -sy * sx, cy * cx]
}

/// Tor's tail (mode 1): one chain on sub-part joints 8 to 12, link 0 hanging from joint 0 and
/// each later link from the joint before it.
pub fn tor() -> Vec<Chain> {
    let a = [0.188, 0.448, 0.466, 0.410, 0.340];
    let yaw = [[-3.5, 1.25], [-1.25, 1.25], [-1.57, 1.57], [-1.7, 1.7], [-2.0, 2.0]];
    let pitch = [[-10.6, 0.45], [-1.4, 2.0], [-1.4, 1.9], [-1.5, 2.3], [-1.6, 2.4]];
    let chain = (0..5)
        .map(|i| Link {
            joint: 8 + i,
            hangs_from: if i == 0 { 0 } else { 7 + i },
            k: LinkConst {
                a: a[i],
                l: a.get(i + 1).copied().unwrap_or(a[4]),
                alpha: 0.0,
                beta: if i == 0 { -0.6 } else { 0.4 },
                k: 0.05,
                yaw: yaw[i],
                pitch: pitch[i],
                b: if i == 0 { 0.85 } else { 0.7 },
                c: if i == 0 { -0.01 } else { -0.015 },
            },
            points: None,
        })
        .collect();
    vec![chain]
}

/// Roberta's two head chains (mode 2): joints 7 to 10 from joint 3, and 11 to 14 from joint 5.
pub fn roberta() -> Vec<Chain> {
    let a = [0.038, 0.383, 0.457, 0.470];
    let beta = [0.25, -3.5, -6.0, 0.2];
    let pitch = [[0.25, 10.25], [-3.5, 8.75], [-6.0, 9.25], [0.2, 10.2]];
    [(7, 3), (11, 5)]
        .into_iter()
        .map(|(first, root)| {
            (0..4)
                .map(|i| Link {
                    joint: first + i,
                    hangs_from: if i == 0 { root } else { first + i - 1 },
                    k: LinkConst {
                        a: a[i],
                        l: a.get(i + 1).copied().unwrap_or(a[3]),
                        alpha: 0.0,
                        beta: beta[i],
                        k: 0.15,
                        yaw: [-10.0, 10.0],
                        pitch: pitch[i],
                        b: 0.2,
                        c: if i == 0 { -0.01 } else { -0.015 },
                    },
                    points: None,
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: Frame = Frame { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], p: [0.0; 3] };

    fn close(a: Vec3, b: Vec3) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-4)
    }

    #[test]
    fn rotation_turns_minus_z_onto_dir() {
        let (theta, phi) = (0.7f32, -0.3f32);
        let q = link_rotation(theta, phi);
        let m = neopets_data_free_quat(q);
        let v = [-m[2][0], -m[2][1], -m[2][2]];
        assert!(close(v, dir(&IDENTITY, theta, phi)), "{v:?}");
    }

    /// The standard quaternion matrix (columns), as the pose uses.
    fn neopets_data_free_quat(q: [f32; 4]) -> [[f32; 3]; 3] {
        let [x, y, z, w] = q;
        [
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
            [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)],
            [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)],
        ]
    }

    #[test]
    fn a_still_owner_rests_on_the_rest_direction() {
        let mut chain = tor();
        let link = &mut chain[0][1];
        for _ in 0..3 {
            let (theta, phi) = step_link(link, &IDENTITY, [0.0; 3]);
            assert!(theta.abs() < 1e-3 && (phi - 0.4).abs() < 1e-3, "{theta} {phi}");
        }
    }

    #[test]
    fn motion_swings_the_link_back_and_the_spring_returns_it() {
        let mut chain = tor();
        let link = &mut chain[0][1];
        step_link(link, &IDENTITY, [0.0; 3]);
        // The owner moves along -X (to its right in the link frame): drag c < 0 pushes the
        // point along +X, a negative yaw.
        let (theta, _) = step_link(link, &IDENTITY, [-1.0, 0.0, 0.0]);
        assert!(theta < 0.0, "{theta}");
        let mut last = theta;
        for _ in 0..400 {
            last = step_link(link, &IDENTITY, [0.0; 3]).0;
        }
        assert!(last.abs() < 1e-3, "the spring settles it: {last}");
    }

    #[test]
    fn clamps_hold() {
        let mut chain = tor();
        let link = &mut chain[0][1];
        // A point far down and to the side: pitch and yaw both at their limits.
        link.points = Some(([5.0, -50.0, 0.0], [5.0, -50.0, 0.0]));
        let (theta, phi) = step_link(link, &IDENTITY, [0.0; 3]);
        assert_eq!(phi, -1.4);
        assert_eq!(theta, -1.25);
    }

    #[test]
    fn chain_tables() {
        let t = tor();
        assert_eq!(t[0].iter().map(|l| (l.joint, l.hangs_from)).collect::<Vec<_>>(), [(8, 0), (9, 8), (10, 9), (11, 10), (12, 11)]);
        assert_eq!(t[0].iter().map(|l| l.k.l).collect::<Vec<_>>(), [0.448, 0.466, 0.410, 0.340, 0.340]);
        let r = roberta();
        assert_eq!(r[1].iter().map(|l| (l.joint, l.hangs_from)).collect::<Vec<_>>(), [(11, 5), (12, 11), (13, 12), (14, 13)]);
        assert_eq!(r[0][3].k.pitch, [0.2, 10.2]);
    }
}
