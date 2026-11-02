//! Tor's Mote power-attack objects (SPEC-0036 Charm attack hits, Object pairing, Ten-slot timed
//! lists, Hurt shapes): the object a power attack spawns for the Mote in word 2, its volume
//! against each target's body cylinder, its ten-slot hit list and the charm damage rule.
//!
//! Covered: Lava 161 (family B, a sphere orbiting the hero raised by 2.0 on world axes, its
//! angle continuing from the previous orbit), Wind 167 (a vertical cylinder on the ground under
//! the hero), Supernova 173 (the Nova cylinder at the hero raised by 1.5), and family A: Water
//! 163 and Nova 172 (vertical cylinders of height 4.0 whose radius follows their particle's
//! size line, ending with the particle's life) and Rock 165 (three stationary sub-shots started
//! 16 ticks apart). Not covered: Shadow 169 and Sun 171 (their radius is a live effect field),
//! the effect objects, the status requests other than Lava's fire, and the breakable walks.
//! Also Tor's Shadow 169 and Sun 171, and Roberta's objects (`spawn_roberta`).
//! The family A centres are the spawn point (Water and Nova's attach points are taken there).

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
    /// Roberta's orbit radius and life (0x2DCBAC, 0x2DCBA8), and the beams' life (0x2DCB98).
    pub r_orbit_radius: f32,
    pub r_orbit_seconds: f32,
    pub beam_seconds: f32,
    /// See `LEAF`.
    pub leaf: [f32; 5],
}

pub const ORBIT_RADIUS: u32 = 0x002D_CB68;
pub const ORBIT_SECONDS: u32 = 0x002D_CB64;
pub const NOVA_RADIUS: u32 = 0x002D_CBB0;
pub const WIND_RADIUS: u32 = 0x002D_CBB8;
pub const WIND_HEIGHT: u32 = 0x002D_CBBC;
pub const R_ORBIT_RADIUS: u32 = 0x002D_CBAC;
pub const R_ORBIT_SECONDS: u32 = 0x002D_CBA8;
pub const BEAM_SECONDS: u32 = 0x002D_CB98;
/// Roberta's Leaf ring: start, top, rate, life and height (0x2DCB70, 74, 78, 7C, 80).
pub const LEAF: [u32; 5] = [0x002D_CB70, 0x002D_CB74, 0x002D_CB78, 0x002D_CB7C, 0x002D_CB80];

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
    /// A vertical cylinder centred on the object point; `follow` the hero: 0 none, 1 the hero
    /// raised by 1.5 (Nova 173), 2 the ground under the hero (Wind 167).
    Cylinder { radius: f32, height: f32, follow: u8 },
    /// Family A: a vertical cylinder of height 4.0 whose radius is start + rate x age / life
    /// (age and life in seconds; SPEC-0036 Family A users).
    Grow { start: f32, rate: f32, life: f32 },
    /// Tor's Shadow: a sphere fixed at the spawn point.
    Sphere { radius: f32 },
    /// Tor's Sun: a sphere that tests each character once, on its first active tick.
    Burst { radius: f32 },
    /// Roberta's Leaf: a vertical cylinder whose radius grows at `rate` a second up to `top`.
    Ring { start: f32, top: f32, rate: f32, height: f32 },
    /// Family C: the segment from `centre` to `end`, of transverse half-width `half`.
    Beam { half: f32, end: Vec3 },
}

/// The family A height and the particle lines (start, rate, life in seconds).
const GROW_HEIGHT: f32 = 4.0;
const WATER: (f32, f32, f32) = (0.5, 14.5, 0.7);
const NOVA: (f32, f32, f32) = (0.0, 18.0, 1.2);
/// Roberta's Lava (definition 415: life 0.78 to 0.82 s, radius 0.5 rising to 25.0).
const R_LAVA: (f32, f32, f32) = (0.5, 24.5, 0.8);
const ROCK: (f32, f32, f32) = (0.0, 20.0, 0.6);
/// The Rock sub-shots' spacing (0x2DCB94, 0.25 s: 15 ticks of countdown, so 16 apart).
const ROCK_GAP: u32 = 16;
/// The anchor offsets of the orbit (2.0) and the Nova cylinder (1.5).
const ORBIT_LIFT: f32 = 2.0;
const NOVA_LIFT: f32 = 1.5;
/// Tor's Shadow and Sun: both radii 10.0; Shadow's life 30 ticks; the Sun's centre raised by 1.5. Sun lives
/// until the longest particle of definition 559 dies: 174 ticks on average, at most 180.
const SPHERE_RADIUS: f32 = 10.0;
const SHADOW_TICKS: u32 = 30;
const SUN_TICKS: u32 = 174;
const BEAM_LENGTH: f32 = 20.0;
const BEAM_HALF: f32 = 1.0;
/// A Leaf entry never expires.
const FOREVER: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    /// The object's own Mote id.
    pub id: u16,
    pub shape: Shape,
    pub centre: Vec3,
    /// The orbit angle (radians; the offset is radius x (cos, 0, -sin) on the world axes).
    pub angle: f32,
    /// Ticks left.
    pub life: u32,
    /// The stored outgoing scale A x D x B x G x E.
    pub scale: f32,
    list: [Option<(u64, u32)>; 10],
    /// Ticks before a Rock sub-shot starts (no volume meanwhile).
    delay: u32,
    /// Family A, the ring and the Sun: ticks since the start.
    age: u32,
    /// 0 Tor, 1 Roberta.
    pub hero: usize,
    /// The host's effect objects, once started.
    pub fx: Vec<usize>,
    pub fx_started: bool,
}

/// Tor's power attack objects for a word-2 Mote: none for the effect-only Motes (160, 162,
/// 164, 166, 168, 170) and the ones not covered here, three for Rock. `at` is the spawn point
/// (the hero position plus facing x reach), `angle` the orbit's continuing angle.
pub fn spawn_tor(id: u16, w: &Words, at: Vec3, hero: Vec3, angle: f32, scale: f32) -> Vec<Object> {
    let ticks = |s: f32| (mul(s, 60.0) as i64).max(0) as u32;
    let particle = |s: f32| (s * 60.0).round() as u32;
    let grow = |(start, rate, life): (f32, f32, f32)| Shape::Grow { start, rate, life };
    let one = |shape: Shape, centre: Vec3, life: u32, delay: u32| Object { id, shape, centre, angle, life, scale, list: [None; 10], delay, age: 0, hero: 0, fx: Vec::new(), fx_started: false };
    let mut v = match id {
        161 => vec![one(Shape::Orbit { radius: w.orbit_radius, sphere: ORBIT_SPHERE }, hero, ticks(w.orbit_seconds), 0)],
        167 => vec![one(Shape::Cylinder { radius: w.wind_radius, height: w.wind_height, follow: 2 }, at, ticks(CYLINDER_SECONDS), 0)],
        173 => vec![one(Shape::Cylinder { radius: w.nova_radius, height: NOVA_HEIGHT, follow: 1 }, at, ticks(CYLINDER_SECONDS), 0)],
        // The particle's life ends the object: age + 1/60 reaching the life, so 42 and 72 ticks
        // (rounded: 0.7 x 60 is just under 42 in single precision).
        163 => vec![one(grow(WATER), at, particle(WATER.2), 0)],
        172 => vec![one(grow(NOVA), at, particle(NOVA.2), 0)],
        // Three stationary sub-shots at the release point, started on the Rock object's ticks
        // 1, 17 and 33, each living 36 ticks with its own hit list.
        165 => (0..3).map(|k| one(grow(ROCK), at, particle(ROCK.2), k * ROCK_GAP)).collect(),
        169 => vec![one(Shape::Sphere { radius: SPHERE_RADIUS }, at, SHADOW_TICKS, 0)],
        171 => vec![one(Shape::Burst { radius: SPHERE_RADIUS }, [hero[0], add(hero[1], NOVA_LIFT), hero[2]], SUN_TICKS, 0)],
        _ => Vec::new(),
    };
    for o in &mut v {
        o.place(hero, None);
    }
    v
}

/// Roberta's full-release objects (SPEC-0036 Object pairing): Lava 161 (family A), the Leaf ring 164, the
/// Shadow and Sun beams 169 and 171 (family C) and the Supernova orbit 173. Not built: the Bubble beam 162
/// (R-0206), the Rock skipping shots 165, the Fog cloud 166 and the Supernova's bolts.
pub fn spawn_roberta(id: u16, w: &Words, at: Vec3, hero: Vec3, angle: f32, scale: f32) -> Vec<Object> {
    let ticks = |s: f32| (mul(s, 60.0) as i64).max(0) as u32;
    let (shape, centre, life) = match id {
        161 => (Shape::Grow { start: R_LAVA.0, rate: R_LAVA.1, life: R_LAVA.2 }, at, (R_LAVA.2 * 60.0).round() as u32),
        164 => {
            let [start, top, rate, life, height] = w.leaf;
            (Shape::Ring { start, top, rate, height }, at, ticks(life))
        }
        169 | 171 => (Shape::Beam { half: BEAM_HALF, end: at }, hero, ticks(w.beam_seconds)),
        173 => (Shape::Orbit { radius: w.r_orbit_radius, sphere: ORBIT_SPHERE }, hero, ticks(w.r_orbit_seconds)),
        _ => return Vec::new(),
    };
    let mut o = Object { id, shape, centre, angle, life, scale, list: [None; 10], delay: 0, age: 0, hero: 1, fx: Vec::new(), fx_started: false };
    o.place(hero, None);
    vec![o]
}

/// The effect definitions an object plays (SPEC-0036 Effect definitions; bank 3). Tor's Sun also holds 568,
/// the ring it rescales, which is not played here.
pub fn definitions(hero: usize, id: u16) -> &'static [i32] {
    match (hero, id) {
        (0, 161) => &[420],
        (0, 163) => &[416, 564],
        (0, 165) => &[553, 562],
        (0, 167) => &[555],
        (0, 169) => &[557],
        (0, 171) => &[559, 566],
        (0, 172) => &[560],
        (0, 173) => &[561],
        (1, 161) => &[415],
        (1, 164) => &[541],
        (1, 169) => &[546],
        (1, 171) => &[548],
        (1, 173) => &[550],
        _ => &[],
    }
}

impl Object {
    /// The centre from the hero: the orbit at the hero raised by 2.0 plus radius x (cos, 0,
    /// -sin); Nova at the hero raised by 1.5; Wind on the ground under the hero (`ground`, the
    /// previous height kept on a miss).
    fn place(&mut self, hero: Vec3, ground: Option<f32>) {
        match self.shape {
            Shape::Orbit { radius, .. } => {
                let (s, c) = self.angle.sin_cos();
                self.centre = [add(hero[0], mul(radius, c)), add(hero[1], ORBIT_LIFT), sub(hero[2], mul(radius, s))];
            }
            Shape::Cylinder { follow: 1, .. } => self.centre = [hero[0], add(hero[1], NOVA_LIFT), hero[2]],
            Shape::Cylinder { follow: 2, .. } => self.centre = [hero[0], ground.unwrap_or(self.centre[1]), hero[2]],
            _ => {}
        }
    }

    /// The family A radius now.
    pub fn radius(&self) -> f32 {
        match self.shape {
            Shape::Grow { start, rate, life } => add(start, mul(rate, self.age as f32 / 60.0 / life)),
            Shape::Orbit { sphere, .. } => sphere,
            Shape::Cylinder { radius, .. } => radius,
            Shape::Sphere { radius } | Shape::Burst { radius } => radius,
            Shape::Ring { start, top, rate, .. } => add(start, mul(rate, self.age as f32 / 60.0)).min(top),
            Shape::Beam { half, .. } => half,
        }
    }

    /// The beam rebuilt each live tick: from the hero's query point along the normalized aim.
    pub fn aim(&mut self, from: Vec3, direction: Vec3) {
        if let Shape::Beam { end, .. } = &mut self.shape {
            *end = [from[0] + BEAM_LENGTH * direction[0], from[1], from[2] + BEAM_LENGTH * direction[2]];
            self.centre = from;
        }
    }

    /// The Sun has only its first active tick: the tick after the spawn.
    fn active(&self) -> bool {
        self.delay == 0 && (!matches!(self.shape, Shape::Burst { .. }) || self.age == 2)
    }

    /// The volume as a segment and a radius, for the world breakables; none while it is not active.
    pub fn segment(&self) -> Option<(Vec3, Vec3, f32)> {
        if !self.active() {
            return None;
        }
        let c = self.centre;
        let up = |h: f32| ([c[0], c[1] - 0.5 * h, c[2]], [c[0], c[1] + 0.5 * h, c[2]], self.radius());
        Some(match self.shape {
            Shape::Cylinder { height, .. } | Shape::Ring { height, .. } => up(height),
            Shape::Grow { .. } => up(GROW_HEIGHT),
            Shape::Beam { half, end } => (c, end, half),
            _ => (c, c, self.radius()),
        })
    }

    /// The Mote kind of the breakable walk (SPEC-0036 Attack sweeps): fixed for Tor's Shadow and Sun, by the
    /// object otherwise; Roberta's Supernova has no filter (7 passes all). Roberta's Leaf has no walk.
    pub fn breakable_kind(&self) -> Option<i32> {
        match (self.hero, self.id) {
            (1, 164) => None,
            (1, 173) => Some(7),
            _ => Some(crate::mote::element(Some(self.id))),
        }
    }

    /// One tick: a waiting sub-shot counts its delay; the life and the hit-list counters count
    /// down (an expired entry frees its slot), and the orbit advances by 2 pi / 60 about the
    /// hero. Returns whether it is live.
    pub fn tick(&mut self, hero: Vec3, ground: Option<f32>) -> bool {
        if self.delay > 0 {
            self.delay -= 1;
            return true;
        }
        if matches!(self.shape, Shape::Grow { .. } | Shape::Ring { .. } | Shape::Burst { .. }) && self.life > 0 {
            self.age += 1;
        }
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
        }
        self.place(hero, ground);
        self.life > 0
    }

    /// The volume against a target's body cylinder (feet point, radius, height); a waiting
    /// sub-shot has none.
    pub fn touches(&self, feet: Vec3, radius: f32, height: f32) -> bool {
        if !self.active() {
            return false;
        }
        match self.shape {
            Shape::Sphere { radius: r } | Shape::Burst { radius: r } => sphere_meets_body(self.centre, r, feet, radius, height),
            Shape::Orbit { sphere, .. } => sphere_meets_body(self.centre, sphere, feet, radius, height),
            Shape::Cylinder { radius: r, height: h, .. } => cylinder_meets_body(self.centre, r, h, feet, radius, height),
            Shape::Grow { .. } => cylinder_meets_body(self.centre, self.radius(), GROW_HEIGHT, feet, radius, height),
            Shape::Ring { height: h, .. } => cylinder_meets_body(self.centre, self.radius(), h, feet, radius, height),
            Shape::Beam { half, end } => beam_meets_body(self.centre, end, half, feet, radius, height),
        }
    }

    /// List a target for 60 ticks: false when it is listed already or all ten slots are taken.
    pub fn list(&mut self, target: u64) -> bool {
        // The Sun keeps no list: it has one tick to test.
        if matches!(self.shape, Shape::Burst { .. }) {
            return true;
        }
        if self.list.iter().flatten().any(|&(t, _)| t == target) {
            return false;
        }
        match self.list.iter_mut().find(|e| e.is_none()) {
            Some(e) => {
                *e = Some((target, if matches!(self.shape, Shape::Ring { .. }) { FOREVER } else { LISTED }));
                true
            }
            None => false,
        }
    }
}

/// The horizontal beam against a body cylinder: the body axis within half + radius of the segment across, and
/// the beam's height over the body's span by the same half (R-0206 for the vertical extent).
pub fn beam_meets_body(a: Vec3, b: Vec3, half: f32, feet: Vec3, radius: f32, height: f32) -> bool {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let l = (dx * dx + dz * dz).max(1e-6);
    let t = (((feet[0] - a[0]) * dx + (feet[2] - a[2]) * dz) / l).clamp(0.0, 1.0);
    let (ex, ez) = (feet[0] - a[0] - t * dx, feet[2] - a[2] - t * dz);
    (ex * ex + ez * ez).sqrt() <= half + radius && a[1] + half >= feet[1] && a[1] - half <= feet[1] + height
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

    const W: Words = Words {
        orbit_radius: 2.5,
        orbit_seconds: 10.0,
        nova_radius: 4.0,
        wind_radius: 4.0,
        wind_height: 5.5,
        r_orbit_radius: 2.5,
        r_orbit_seconds: 10.0,
        beam_seconds: 5.0,
        leaf: [1.0, 5.0, 1.0, 10.0, 3.0],
    };

    /// SPEC-0036 item 5: Shadow ends after 30 ticks where it was cast; the Sun's vector (A at 4.0 hit once,
    /// B at 12.0 untouched).
    #[test]
    fn tor_shadow_and_sun() {
        let mut o = spawn_tor(169, &W, [1.0, 0.0, 0.0], [0.0; 3], 0.0, 1.0).remove(0);
        for _ in 0..29 {
            assert!(o.tick([50.0, 0.0, 0.0], None));
        }
        assert_eq!(o.centre, [1.0, 0.0, 0.0]);
        assert!(o.touches([10.0, 0.0, 0.0], 1.0, 2.0));
        assert!(!o.tick([50.0, 0.0, 0.0], None));
        let mut s = spawn_tor(171, &W, [0.0; 3], [0.0; 3], 0.0, 1.0).remove(0);
        let hits = |o: &Object| (o.touches([4.0, 0.0, 0.0], 0.5, 2.0), o.touches([12.0, 0.0, 0.0], 0.5, 2.0));
        assert_eq!(s.centre[1], 1.5);
        s.tick([0.0; 3], None);
        assert_eq!(hits(&s), (false, false));
        s.tick([0.0; 3], None);
        assert_eq!(hits(&s), (true, false));
        s.tick([0.0; 3], None);
        assert_eq!(hits(&s), (false, false));
    }

    /// SPEC-0036: a target in Roberta's beam is hit on beam ticks 1, 61, 121, 181 and 241; the box spans -1.0
    /// to +1.0; the Leaf ring's radius 2.5 s after the spawn is 3.5, 5.0 from 4 s, and its list never clears.
    #[test]
    fn roberta_beam_and_ring() {
        let mut o = spawn_roberta(169, &W, [0.0; 3], [0.0; 3], 0.0, 1.0).remove(0);
        assert_eq!(o.life, 300);
        let (mut hits, mut live) = (Vec::new(), 0);
        for t in 1..=400 {
            if !o.tick([0.0; 3], None) {
                break;
            }
            live += 1;
            o.aim([0.0, 1.5, 0.0], [0.0, 0.0, 1.0]);
            if o.touches([0.5, 0.0, 10.0], 0.5, 2.0) && o.list(7) {
                hits.push(t);
            }
        }
        assert_eq!(live, 299);
        assert_eq!(hits, [1, 61, 121, 181, 241]);
        assert!(o.touches([1.5, 0.0, 10.0], 0.5, 2.0) && !o.touches([1.6, 0.0, 10.0], 0.5, 2.0));
        assert!(!o.touches([0.0, 0.0, 22.0], 0.5, 2.0), "past its end");
        let mut l = spawn_roberta(164, &W, [0.0; 3], [0.0; 3], 0.0, 1.0).remove(0);
        assert!(l.list(7));
        for _ in 0..150 {
            l.tick([0.0; 3], None);
        }
        assert!((l.radius() - 3.5).abs() < 1e-4);
        assert!(!l.list(7));
        for _ in 150..300 {
            l.tick([0.0; 3], None);
        }
        assert_eq!(l.radius(), 5.0);
        assert!(!l.list(7));
    }

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
        // The offset is radius x (cos, 0, -sin) at the hero raised by 2.0: (2.5, 2, 0) at 0 and
        // (0, 2, -2.5) a quarter turn on.
        let mut o = spawn_tor(161, &W, [0.0; 3], [0.0; 3], 0.0, 4.0).remove(0);
        assert_eq!(o.life, 600);
        assert!((o.centre[0] - 2.5).abs() < 1e-4 && (o.centre[1] - 2.0).abs() < 1e-4 && o.centre[2].abs() < 1e-4, "{:?}", o.centre);
        for _ in 0..15 {
            o.tick([0.0; 3], None);
        }
        assert!(o.centre[0].abs() < 1e-3 && (o.centre[2] + 2.5).abs() < 1e-3, "{:?}", o.centre);
        assert!(o.list(7) && !o.list(7));
        let mut hits = 1;
        for _ in 0..120 {
            o.tick([0.0; 3], None);
            if o.list(7) {
                hits += 1;
            }
        }
        assert_eq!(hits, 3);
        // Ten slots: an eleventh target is refused.
        // Nova 173 sits at the hero raised by 1.5.
        let mut o = spawn_tor(173, &W, [1.0, 0.0, 0.0], [0.0; 3], 0.0, 4.0).remove(0);
        assert_eq!((o.centre, o.life), ([0.0, 1.5, 0.0], 600));
        assert!((0..10).all(|t| o.list(t)) && !o.list(10));
        assert!(spawn_tor(160, &W, [0.0; 3], [0.0; 3], 0.0, 4.0).is_empty());
        // Wind follows the ground under the hero and keeps the last height on a miss.
        let mut o = spawn_tor(167, &W, [1.0, 0.0, 0.0], [0.0; 3], 0.0, 4.0).remove(0);
        o.tick([3.0, 5.0, 1.0], Some(2.0));
        assert_eq!(o.centre, [3.0, 2.0, 1.0]);
        o.tick([4.0, 5.0, 1.0], None);
        assert_eq!(o.centre, [4.0, 2.0, 1.0]);
    }

    #[test]
    fn family_a_growth() {
        // SPEC-0036 Family A: Water at 0.35 s is 0.5 + 14.5 x 0.5 = 7.75 and lives 42 ticks;
        // Nova at 36 ticks is 9.0 and lives 72; Rock's sub-shots start 16 ticks apart and are
        // 10.0 after 18 ticks of their own.
        let mut o = spawn_tor(163, &W, [0.0; 3], [0.0; 3], 0.0, 1.0).remove(0);
        assert_eq!(o.life, 42);
        for _ in 0..21 {
            o.tick([9.0; 3], None);
        }
        assert!((o.radius() - 7.75).abs() < 1e-4 && o.centre == [0.0; 3], "{}", o.radius());
        let mut ticks = 21;
        while o.tick([9.0; 3], None) {
            ticks += 1;
        }
        assert_eq!(ticks + 1, 42);
        let mut o = spawn_tor(172, &W, [0.0; 3], [0.0; 3], 0.0, 1.0).remove(0);
        assert_eq!(o.life, 72);
        for _ in 0..36 {
            o.tick([0.0; 3], None);
        }
        assert!((o.radius() - 9.0).abs() < 1e-4);
        // The height interval is the centre +- 2.0.
        assert!(o.touches([8.0, 2.0, 0.0], 1.0, 2.0) && !o.touches([8.0, 2.01, 0.0], 1.0, 2.0));
        let mut v = spawn_tor(165, &W, [0.0; 3], [0.0; 3], 0.0, 1.0);
        assert_eq!(v.len(), 3);
        for _ in 0..34 {
            for o in v.iter_mut() {
                o.tick([0.0; 3], None);
            }
        }
        // After 34 ticks: the first sub-shot is 34 ticks old, the second 18, the third 2.
        assert!((v[1].radius() - 10.0).abs() < 1e-4, "{}", v[1].radius());
        assert!(v[0].radius() > v[1].radius() && v[2].radius() < 2.0);
        // A waiting sub-shot has no volume.
        let w = spawn_tor(165, &W, [0.0; 3], [0.0; 3], 0.0, 1.0);
        assert!(w[0].touches([0.0; 3], 1.0, 2.0) && !w[1].touches([0.0; 3], 1.0, 2.0));
    }

    #[test]
    fn charm_damage_rule() {
        // Scale 10 with C = 1 and P = 1.5: 15; a defence of 12 absorbs the hit.
        assert_eq!(damage(10.0, 1.0, 0.0, 1.5), (15, false));
        assert_eq!(damage(10.0, 1.0, 12.0, 1.5), (0, true));
        assert_eq!(scale(4.0, 1.0, 1.0, 1.0, 1.0), 4.0);
    }
}
