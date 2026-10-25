//! Tor's Mote power-attack objects (SPEC-0036 Charm attack hits, Object pairing, Ten-slot timed
//! lists, Hurt shapes): the object a power attack spawns for the Mote in word 2, its volume
//! against each target's body cylinder, its ten-slot hit list and the charm damage rule.
//!
//! Covered: Lava 161 (family B, a sphere orbiting the hero), Wind 167 (a vertical cylinder) and
//! Supernova 173 (the Nova cylinder). The centre of the Wind and Nova cylinders (object +0x80)
//! is not specified; it is held at the spawn point here (unverified, R-0033). Not covered: the
//! family A objects (163, 165, 172), Shadow 169 and Sun 171 (their sizes are open), the effect
//! objects, the status requests and the breakable walks.

use ps2_float::fpu::{add, mul, sub};

type Vec3 = [f32; 3];

/// The boot words the objects read (f32).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    /// Tor's orbit radius (0x2DCB68) and life in seconds (0x2DCB64).
    pub orbit_radius: f32,
    pub orbit_seconds: f32,
    /// The Nova cylinder radius (0x2DCBB0).
    pub nova_radius: f32,
    /// The Wind cylinder radius (0x2DCBB8) and height (0x2DCBBC).
    pub wind_radius: f32,
    pub wind_height: f32,
}

pub const ORBIT_RADIUS: u32 = 0x002D_CB68;
pub const ORBIT_SECONDS: u32 = 0x002D_CB64;
pub const NOVA_RADIUS: u32 = 0x002D_CBB0;
pub const WIND_RADIUS: u32 = 0x002D_CBB8;
pub const WIND_HEIGHT: u32 = 0x002D_CBBC;

/// The family B hit sphere radius (+0x34, a program constant), the Nova height (a code
/// constant), the 10 s life of the Wind and Nova objects and the 60-tick listing.
const ORBIT_SPHERE: f32 = 2.5;
const NOVA_HEIGHT: f32 = 2.0;
const CYLINDER_SECONDS: f32 = 10.0;
const LISTED: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A sphere of `sphere` radius circling the hero at `radius`, one turn per second.
    Orbit { radius: f32, sphere: f32 },
    /// A vertical cylinder centred on the object point.
    Cylinder { radius: f32, height: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    /// The object's own Mote id.
    pub id: u16,
    pub shape: Shape,
    pub centre: Vec3,
    /// The orbit angle (radians, about the up axis, 0 along +z).
    angle: f32,
    /// Ticks left.
    pub life: u32,
    /// The stored outgoing scale A x D x B x G x E.
    pub scale: f32,
    list: [Option<(u64, u32)>; 10],
}

/// Tor's power attack object for a word-2 Mote, or None for the effect-only Motes (160, 162,
/// 164, 166, 168, 170) and the ones not covered here. `at` is the spawn point (the hero
/// position plus facing x reach) and `facing` the hero's facing.
pub fn spawn_tor(id: u16, w: &Words, at: Vec3, hero: Vec3, facing: Vec3, scale: f32) -> Option<Object> {
    let ticks = |s: f32| (mul(s, 60.0) as i64).max(0) as u32;
    let (shape, centre, life) = match id {
        161 => (Shape::Orbit { radius: w.orbit_radius, sphere: ORBIT_SPHERE }, hero, ticks(w.orbit_seconds)),
        167 => (Shape::Cylinder { radius: w.wind_radius, height: w.wind_height }, at, ticks(CYLINDER_SECONDS)),
        173 => (Shape::Cylinder { radius: w.nova_radius, height: NOVA_HEIGHT }, at, ticks(CYLINDER_SECONDS)),
        _ => return None,
    };
    let mut o = Object { id, shape, centre, angle: facing[0].atan2(facing[2]), life, scale, list: [None; 10] };
    o.place(hero);
    Some(o)
}

impl Object {
    fn place(&mut self, hero: Vec3) {
        if let Shape::Orbit { radius, .. } = self.shape {
            let (s, c) = self.angle.sin_cos();
            self.centre = [add(hero[0], mul(radius, s)), hero[1], add(hero[2], mul(radius, c))];
        }
    }

    /// One tick: the life and the hit-list counters count down (an expired entry frees its
    /// slot), and the orbit advances by 2 pi / 60 about the hero. Returns whether it is live.
    pub fn tick(&mut self, hero: Vec3) -> bool {
        self.life = self.life.saturating_sub(1);
        for e in self.list.iter_mut() {
            if let Some((_, n)) = e {
                *n -= 1;
                if *n == 0 {
                    *e = None;
                }
            }
        }
        if matches!(self.shape, Shape::Orbit { .. }) {
            self.angle = add(self.angle, mul(std::f32::consts::TAU, 1.0 / 60.0));
            self.place(hero);
        }
        self.life > 0
    }

    /// The volume against a target's body cylinder (feet point, radius, height).
    pub fn touches(&self, feet: Vec3, radius: f32, height: f32) -> bool {
        match self.shape {
            Shape::Orbit { sphere, .. } => sphere_meets_body(self.centre, sphere, feet, radius, height),
            Shape::Cylinder { radius: r, height: h } => cylinder_meets_body(self.centre, r, h, feet, radius, height),
        }
    }

    /// List a target for 60 ticks: false when it is listed already or all ten slots are taken.
    pub fn list(&mut self, target: u64) -> bool {
        if self.list.iter().flatten().any(|&(t, _)| t == target) {
            return false;
        }
        match self.list.iter_mut().find(|e| e.is_none()) {
            Some(e) => {
                *e = Some((target, LISTED));
                true
            }
            None => false,
        }
    }
}

/// A sphere against a vertical body cylinder (feet up to feet + height); contact counts.
pub fn sphere_meets_body(c: Vec3, r: f32, feet: Vec3, radius: f32, height: f32) -> bool {
    let y = c[1].clamp(feet[1], feet[1] + height);
    let (dx, dz) = (c[0] - feet[0], c[2] - feet[2]);
    let h = ((dx * dx + dz * dz).sqrt() - radius).max(0.0);
    let dy = c[1] - y;
    h * h + dy * dy <= r * r
}

/// Two vertical cylinders: horizontal centre distance at most the sum of the radii and the
/// vertical intervals touching ([c - h/2, c + h/2] against [feet, feet + height]).
pub fn cylinder_meets_body(c: Vec3, r: f32, h: f32, feet: Vec3, radius: f32, height: f32) -> bool {
    let (dx, dz) = (c[0] - feet[0], c[2] - feet[2]);
    let half = 0.5 * h;
    (dx * dx + dz * dz).sqrt() <= r + radius && c[1] - half <= feet[1] + height && c[1] + half >= feet[1]
}

/// The stored scale at spawn: A x D x B x G x E (Tor's release sub-state 10, so D = 1).
pub fn scale(a: f32, d: f32, b: f32, g: f32, e: f32) -> f32 {
    mul(mul(mul(mul(a, d), b), g), e)
}

/// A charm hit: x = scale x C, z = x - defence (absorbed when x > 0 and z <= 0), floored at 0,
/// then damage = trunc(z x P), with no minimum.
pub fn damage(scale: f32, c: f32, defence: f32, p: f32) -> (i32, bool) {
    let x = mul(scale, c);
    let z = sub(x, defence);
    let absorbed = x > 0.0 && z <= 0.0;
    (mul(z.max(0.0), p) as i32, absorbed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Words = Words { orbit_radius: 2.5, orbit_seconds: 10.0, nova_radius: 4.0, wind_radius: 4.0, wind_height: 5.5 };

    #[test]
    fn cylinder_overlap_vectors() {
        // SPEC-0036 Family A synthetic vectors: radius 2 at the origin, height 4; a target of
        // radius 1, height 2: tangent at x = 3 hits, 3.01 misses; coincident centres with feet
        // at y = 2 hit (cap tangency), 2.01 misses.
        let c = [0.0, 0.0, 0.0];
        assert!(cylinder_meets_body(c, 2.0, 4.0, [3.0, 0.0, 0.0], 1.0, 2.0));
        assert!(!cylinder_meets_body(c, 2.0, 4.0, [3.01, 0.0, 0.0], 1.0, 2.0));
        assert!(cylinder_meets_body(c, 2.0, 4.0, [0.0, 2.0, 0.0], 1.0, 2.0));
        assert!(!cylinder_meets_body(c, 2.0, 4.0, [0.0, 2.01, 0.0], 1.0, 2.0));
        assert!(sphere_meets_body([3.0, 1.0, 0.0], 2.5, [0.0, 0.0, 0.0], 0.5, 2.0));
        assert!(!sphere_meets_body([3.5, 1.0, 0.0], 2.5, [0.0, 0.0, 0.0], 0.5, 2.0));
    }

    #[test]
    fn orbit_turns_and_lists() {
        // Tor's Lava orbit: 600 ticks of life, one turn per second at radius 2.5, a target
        // listed for 60 ticks so a target in contact is hit once per second.
        let mut o = spawn_tor(161, &W, [0.0; 3], [0.0; 3], [0.0, 0.0, 1.0], 4.0).unwrap();
        assert_eq!(o.life, 600);
        assert!((o.centre[2] - 2.5).abs() < 1e-4);
        for _ in 0..15 {
            o.tick([0.0; 3]);
        }
        assert!((o.centre[0] - 2.5).abs() < 1e-3 && o.centre[2].abs() < 1e-3, "{:?}", o.centre);
        assert!(o.list(7) && !o.list(7));
        let mut hits = 1;
        for _ in 0..120 {
            o.tick([0.0; 3]);
            if o.list(7) {
                hits += 1;
            }
        }
        assert_eq!(hits, 3);
        // Ten slots: an eleventh target is refused.
        let mut o = spawn_tor(173, &W, [1.0, 0.0, 0.0], [0.0; 3], [0.0, 0.0, 1.0], 4.0).unwrap();
        assert_eq!((o.centre, o.life), ([1.0, 0.0, 0.0], 600));
        assert!((0..10).all(|t| o.list(t)) && !o.list(10));
        assert!(spawn_tor(160, &W, [0.0; 3], [0.0; 3], [0.0, 0.0, 1.0], 4.0).is_none());
    }

    #[test]
    fn charm_damage_rule() {
        // Scale 10 with C = 1 and P = 1.5: 15; a defence of 12 absorbs the hit.
        assert_eq!(damage(10.0, 1.0, 0.0, 1.5), (15, false));
        assert_eq!(damage(10.0, 1.0, 12.0, 1.5), (0, true));
        assert_eq!(scale(4.0, 1.0, 1.0, 1.0, 1.0), 4.0);
    }
}
