//! Hero ground locomotion (SPEC-0033 Behaviors 2 and 5; SPEC-0040 Behavior 14C): stick speed
//! and direction, the speed tiers that pick slow walk, walk or run and the play rate, the facing
//! turn, and travel by the root step of the playing clip turned by the facing basis.
//! Not covered: jumps, gravity and falling, wall contacts, status factors, strafe (state 32),
//! traversal states, and the displayed-height lag of Behavior 7.

pub type Vec3 = [f32; 3];

/// Hero action states used here (SPEC-0033 Behavior 3): 1 stance, 2 slow walk, 3 walk, 4 run.
pub const STANCE: u8 = 1;
pub const SLOW_WALK: u8 = 2;
pub const WALK: u8 = 3;
pub const RUN: u8 = 4;
/// Passes after a run ends before running may restart (0.5 s at 60 ticks per second).
const RUN_RESTART: u32 = 30;

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalise(v: Vec3) -> Vec3 {
    let l = dot(v, v).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

fn flatten(v: Vec3) -> Vec3 {
    normalise([v[0], 0.0, v[2]])
}

/// One pass of controller input: the left stick after the dead zone, `side` positive to the
/// left and `forward` positive up, each in [-1, 1]; `run` is R1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeroInput {
    pub side: f32,
    pub forward: f32,
    pub run: bool,
}

/// Speed value v: the stick length, capped at 1.0 (SPEC-0033 Behavior 2).
pub fn speed_value(i: &HeroInput) -> f32 {
    (i.side * i.side + i.forward * i.forward).sqrt().min(1.0)
}

/// The facing basis of SPEC-0040 Behavior 14C: B1 side, B2 up, B3 forward.
pub fn basis(facing: Vec3) -> [Vec3; 3] {
    let b3 = flatten(facing);
    let c = cross([0.0, 1.0, 0.0], b3);
    let b1 = if dot(c, c).sqrt() < 0.01 { [1.0, 0.0, 0.0] } else { normalise(c) };
    [b1, cross(b3, b1), b3]
}

/// Facing turn while moving (speed value above 0.01): c = max(d, 0.95) and the new facing is
/// c f + √|1 − c²| u, flattened and normalised: at most 18.19° per pass (SPEC-0033 Behavior 2).
pub fn turn_moving(f: Vec3, u: Vec3) -> Vec3 {
    let d = dot(f, u).min(1.0);
    let c = d.max(0.95);
    let s = (1.0 - c * c).abs().sqrt();
    flatten([c * f[0] + s * u[0], c * f[1] + s * u[1], c * f[2] + s * u[2]])
}

/// The tier step (SPEC-0033 5.1): the state to request and the factor on the play rate, or None
/// for no movement (v ≤ 0.1).
pub fn tier(v: f32, run_mode: bool) -> Option<(u8, f32)> {
    if v <= 0.1 {
        return None;
    }
    let v = v.max(0.3);
    Some(if run_mode {
        if v >= 0.66 {
            (RUN, v)
        } else if v >= 0.33 {
            (WALK, v)
        } else {
            (SLOW_WALK, (v / 0.33).min(1.0))
        }
    } else if v >= 0.5 {
        (WALK, v)
    } else {
        (SLOW_WALK, (2.0 * v).min(1.0))
    })
}

/// A hero's ground position, facing and locomotion state.
#[derive(Clone, Debug, PartialEq)]
pub struct Locomotion {
    pub position: Vec3,
    pub facing: Vec3,
    pub state: u8,
    /// Stick direction u (steering target): kept while the stick is centred.
    pub target: Vec3,
    /// Per-pass speed scale P, the hero's play rate R (base scale 1.0).
    pub rate: f32,
    run_counter: u32,
}

impl Locomotion {
    pub fn spawn(position: Vec3, facing: Vec3) -> Self {
        let f = flatten(facing);
        let f = if dot(f, f) == 0.0 { [1.0, 0.0, 0.0] } else { f };
        Self {
            position,
            facing: f,
            state: STANCE,
            target: f,
            rate: 1.0,
            run_counter: RUN_RESTART,
        }
    }

    /// The animation id the current state plays (SPEC-0033 5.1: states 2 to 4 play ids 2 to 4;
    /// the stance plays the base idle 0).
    pub fn action(&self) -> u32 {
        match self.state {
            SLOW_WALK | WALK | RUN => self.state as u32,
            _ => 0,
        }
    }

    /// The hero update of one pass: stick direction from the camera's ground forward and
    /// screen-left vectors, the facing turn, and the speed tiers.
    pub fn update(&mut self, input: &HeroInput, camera_forward: Vec3, camera_left: Vec3) {
        let v = speed_value(input);
        self.rate = 1.0;
        if v > 0.0 {
            let l = v.max(f32::MIN_POSITIVE);
            let (x, z) = (input.side / l, input.forward / l);
            let c = flatten(camera_forward);
            let left = flatten(camera_left);
            self.target = normalise([z * c[0] + x * left[0], 0.0, z * c[2] + x * left[2]]);
            self.facing = turn_moving(self.facing, self.target);
        }
        self.run_counter = if self.state == RUN { 0 } else { (self.run_counter + 1).min(RUN_RESTART) };
        let run_mode = input.run && (self.state == RUN || self.run_counter >= RUN_RESTART);
        match tier(v, run_mode) {
            Some((state, factor)) => {
                self.state = state;
                self.rate *= factor;
            }
            None => self.state = STANCE,
        }
    }

    /// The travel step of one pass: the root step written on the previous pass, discarded when
    /// the stick points behind the facing, turned by the facing basis and added to the position.
    /// The hero then sits on the ground found by `probe(x, z, ceiling)` from 2.5 above its
    /// height, unless the probe misses (below -1e8).
    pub fn travel(&mut self, root_step: Vec3, probe: impl Fn(f32, f32, f32) -> f32) {
        let step = if dot(self.target, self.facing) < 0.0 { [0.0; 3] } else { root_step };
        let [b1, b2, b3] = basis(self.facing);
        for k in 0..3 {
            self.position[k] += step[0] * b1[k] + step[1] * b2[k] + step[2] * b3[k];
        }
        let h = probe(self.position[0], self.position[2], self.position[1] + 2.5);
        if h > -1.0e8 {
            self.position[1] = h;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn angle(a: Vec3, b: Vec3) -> f32 {
        dot(a, b).clamp(-1.0, 1.0).acos().to_degrees()
    }

    #[test]
    fn speed_value_matches_spec_tests() {
        let v = |side, forward| speed_value(&HeroInput { side, forward, run: false });
        assert_eq!(v(0.0, 1.0), 1.0);
        assert_eq!(v(0.8, 0.6), 1.0);
        assert!((v(0.5, 0.25) - 0.559).abs() < 1e-3);
        assert_eq!(v(1.0, 1.0), 1.0, "capped");
    }

    #[test]
    fn basis_matches_spec_example() {
        assert_eq!(basis([0.0, 0.0, 1.0]), [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    #[test]
    fn moving_turn_follows_spec_sequence_from_90_degrees() {
        let u = [1.0, 0.0, 0.0];
        let mut f = [0.0, 0.0, 1.0];
        for want in [71.81, 55.99, 43.03, 32.78, 24.84] {
            f = turn_moving(f, u);
            assert!((angle(f, u) - want).abs() < 0.05, "{} vs {want}", angle(f, u));
        }
    }

    #[test]
    fn tiers_match_spec_speeds() {
        // Walk at R = 1 moves 0.2251 per pass; speed = 0.2251 × factor × 60.
        let speed = |v: f32, run: bool| tier(v, run).map(|(_, f)| 0.2251 * f * 60.0);
        assert_eq!(tier(0.1, false), None);
        assert_eq!(tier(0.5, false).unwrap().0, WALK);
        assert!((speed(0.5, false).unwrap() - 6.75).abs() < 0.01);
        assert_eq!(tier(0.49, false), Some((SLOW_WALK, 0.98)));
        assert_eq!(tier(0.66, true).unwrap().0, RUN);
        assert_eq!(tier(0.65, true), Some((WALK, 0.65)));
        assert_eq!(tier(0.2, true), Some((SLOW_WALK, (0.3f32 / 0.33).min(1.0))));
    }

    #[test]
    fn stick_behind_turns_in_place_before_moving() {
        // SPEC-0033 5.5: at 135° the first two passes move 0, the third moves a full step.
        let mut h = Locomotion::spawn([0.0; 3], [0.0, 0.0, 1.0]);
        let back_left = [(0.5f32).sqrt(), 0.0, -(0.5f32).sqrt()];
        // Camera forward +Z, screen-left +X: stick fully back and left points along back_left.
        let input = HeroInput { side: (0.5f32).sqrt(), forward: -(0.5f32).sqrt(), run: false };
        let mut moved = Vec::new();
        for _ in 0..3 {
            h.update(&input, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
            let before = h.position;
            h.travel([0.0, 0.0, 0.2251], |_, _, _| 0.0);
            moved.push(((h.position[0] - before[0]).powi(2) + (h.position[2] - before[2]).powi(2)).sqrt());
        }
        assert!((angle(h.target, back_left)).abs() < 1e-3);
        assert_eq!(&moved[..2], &[0.0, 0.0]);
        assert!((moved[2] - 0.2251).abs() < 1e-4);
    }

    #[test]
    fn travel_follows_the_ground_and_keeps_height_on_a_miss() {
        let mut h = Locomotion::spawn([0.0, 5.0, 0.0], [0.0, 0.0, 1.0]);
        h.travel([0.0, 0.0, 1.0], |_, z, c| if z > 0.5 { 4.0f32.min(c) } else { -1.0e9 });
        assert_eq!(h.position, [0.0, 4.0, 1.0]);
        h.travel([0.0, 0.0, 1.0], |_, _, _| -1.0e9);
        assert_eq!(h.position, [0.0, 4.0, 2.0]);
    }

    #[test]
    fn run_needs_r1_and_the_restart_wait() {
        let mut h = Locomotion::spawn([0.0; 3], [0.0, 0.0, 1.0]);
        let full = HeroInput { side: 0.0, forward: 1.0, run: true };
        h.update(&full, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!((h.state, h.action()), (RUN, 4));
        h.update(&HeroInput::default(), [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!(h.state, STANCE);
        h.update(&full, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert_eq!(h.state, WALK, "running cannot restart for 30 passes");
    }
}
