//! The enemy projectile pool (SPEC-0029 Behavior 9) and the missile spawners of the common
//! enemies (SPEC-0029 Behavior 13; SPEC-0038 Behavior 13): ten shared slots, first free slot,
//! the per-tick update (owner check, area mode, lifetime, motion with gravity and homing, the
//! hero hit test, the ground test), and the straight, homing, ballistic and lobbed launches.
//!
//! Not covered: the effects and trails (SPEC-0034), the hold word, the mote eater's drain (built by the host from the event), the
//! status roll (no statuses yet), the summon of a lobbed shot (Behavior 31), the owner's own block query after a reflection (enemies do not guard
//! here), the shield Mote's charge, and the world
//! test against static meshes, primitives and moving objects: the build tests the segment
//! against the cell's frame-record walls instead (R-0016).

use crate::hero::Vec3;
use crate::rng::Rng;

/// g' = 32 / 60^2 units per tick squared.
pub const GRAVITY: f32 = 32.0 / 3600.0;
pub const SLOTS: usize = 10;

/// The setup record a spawner submits (Behavior 9, Setup record and submit).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Setup {
    pub kind: i32,
    pub visual: i32,
    pub element: i32,
    pub owner: Option<u64>,
    pub start: Vec3,
    pub velocity: Vec3,
    pub lifetime: i32,
    pub damage: f32,
    pub radius: f32,
    pub spin: f32,
    pub area_radius: f32,
    pub area_duration: i32,
    /// Target id or spawn key.
    pub key: i32,
    pub ignore_world: bool,
    /// Status id (-1 none), chance and value.
    pub status: (i32, i32, i32),
    /// The six effect ids (flight 0 and 1, blocked, expiry, world impact, hit; -1 none).
    pub effects: [i32; 6],
    /// The trail model part (SPEC-0029 Visual update): the index into the owner's table of 11
    /// extra model parts (its model set's mesh children with codes 10 to 20, entry code - 10),
    /// drawn at the shot for visual kinds below 5; -1 for none.
    pub trail: i32,
}

/// SPEC-0029 Behavior 13, trail model part: the table entry a missile slot's shot requests.
/// Flight modes 0 and 2 request entries 1, 7, 8, 9 and 10 for shot counters 0 to 4 and entry 1
/// for other counters; mode 1 always requests entry 1.
pub fn trail_entry(flight: i32, counter: u32) -> i32 {
    match (flight, counter) {
        (0 | 2, 0..=4) => [1, 7, 8, 9, 10][counter as usize],
        _ => 1,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub s: Setup,
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: i32,
    pub area: bool,
    /// Hit selector 0: after a reflection the shot tests its owner instead of the hero.
    pub at_owner: bool,
    pulse: i32,
    area_left: i32,
}

/// What a pool tick does to the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The hero is hit by a shot of this damage and element, with the blocked flag. The caller
    /// computes D = max(0, F(damage - S)) (the shield block, armour and global factor of
    /// SPEC-0036) and the hero takes trunc(D), or max(1, 0.25 D) when blocked (`taken`).
    Hero { damage: f32, owner: Option<u64>, blocked: bool, element: i32 },
    /// A blocked shot was reflected: the shield's charge is updated (SPEC-0036).
    Reflected { element: i32 },
    /// After an unblocked hero hit: the status roll (status id, chance, value in seconds).
    Status { code: i32, chance: i32, seconds: i32 },
    /// A reflected shot hits its owner for trunc(damage).
    Owner { id: u64, damage: f32 },
    /// A spell (kinds 2 to 7) touched its target, stored in the key (SPEC-0038 Behavior 43).
    SpellHit { kind: i32, target: u64 },
    /// A kind-8 release: the keyed summon (not spawned yet).
    Summon { key: i32, at: Vec3 },
    /// One of the shot's effect ids plays at a point (ids 2 to 5; -1 is not sent).
    Effect { id: i32, at: Vec3 },
    /// After a leach hit on the hero (kind 15, blocked or not): the leach extra (SPEC-0011).
    Leach,
    /// After an unblocked Mote-eater hit (kind 14): the drain starts (SPEC-0011; SPEC-0034 3.13).
    MoteEater,
}

/// The damage a hero takes from a shot hit of reduced damage D: max(1, 0.25 D) when blocked
/// (not reflected), else D.
pub fn taken(d: f32, blocked: bool) -> f32 {
    if blocked { (0.25 * d).max(1.0) } else { d }
}

/// The world as the pool sees it this tick.
pub struct Env<'a> {
    pub hero: Vec3,
    /// The hero's motion-record radius and height, and its gap above the ground.
    pub hero_radius: f32,
    pub hero_height: f32,
    pub hero_gap: f32,
    /// The hero's forward axis, its guard query, and whether it holds a shield.
    pub hero_facing: Vec3,
    pub guard: bool,
    pub shield: bool,
    /// The hero holds the poison status (the poison cloud's gate).
    pub poisoned: bool,
    pub owner_exists: &'a dyn Fn(u64) -> bool,
    /// An owner's body: position, motion-record radius and height.
    pub owner_body: &'a dyn Fn(u64) -> Option<(Vec3, f32, f32)>,
    /// A spell target's body, as the owner's.
    pub target_body: &'a dyn Fn(u64) -> Option<(Vec3, f32, f32)>,
    /// Ground height under (x, z), probed from a ceiling.
    pub ground: &'a dyn Fn(f32, f32, f32) -> Option<f32>,
    /// A wall lies on the segment.
    pub wall: &'a dyn Fn(Vec3, Vec3) -> bool,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len(a: Vec3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 0.0 { a.map(|c| c / l) } else { a }
}

/// A sphere (centre c, radius r) against a vertical flat-ended body capsule from base P with
/// radius Rb and height Hb: the distance from c to the solid cylinder is below r (side reach
/// Rb + r, top reach on the axis Hb + r).
pub fn sphere_meets_body(c: Vec3, r: f32, base: Vec3, rb: f32, hb: f32) -> bool {
    let y = c[1] - base[1];
    let dy = if y < 0.0 {
        -y
    } else if y > hb {
        y - hb
    } else {
        0.0
    };
    let h = (c[0] - base[0]).hypot(c[2] - base[2]);
    let dh = (h - rb).max(0.0);
    if dy == 0.0 {
        return h < rb + r;
    }
    (dh * dh + dy * dy).sqrt() < r
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pool {
    pub slots: [Option<Shot>; SLOTS],
}

/// SPEC-0038 Behavior 43: the spell of attack sub-type `sub` (0 to 6) at target `key`: straight at
/// 1.0 unit per tick, lifetime trunc(distance) + 5, damage word 3.0 (unused), radius 1.0, visual
/// kind 23.
pub fn spell(subtype: u8, owner: u64, element: i32, launch: Vec3, aim: Vec3, key: u64) -> Setup {
    let (kind, first) = [(2, 0x848), (3, 0x849), (7, 0x84E), (4, 0x84A), (5, 0x84B), (5, 0x84C), (6, 0x84D)][subtype.min(6) as usize];
    let d = sub(aim, launch);
    Setup {
        kind,
        visual: 23,
        element,
        owner: Some(owner),
        start: launch,
        velocity: unit(d),
        lifetime: len(d) as i32 + 5,
        damage: 3.0,
        radius: 1.0,
        spin: 0.0,
        area_radius: 0.0,
        area_duration: 0,
        key: key as i32,
        ignore_world: false,
        status: (-1, 0, 0),
        effects: [first, -1, 412, -1, 412, 412],
        trail: -1,
    }
}

/// LaunchFireMote (SPEC-0044 1056 Shield lesson; SPEC-0030): behaviour kind 0, visual 23, fire
/// (element 1), no owner, damage 3.0, radius 2.0, the effects 528, -1, 991, -1, 991 and 991, the
/// direction toward the target at `speed` units per tick and a lifetime of trunc(distance /
/// speed) + 20 ticks.
pub fn fire_mote(start: Vec3, target: Vec3, speed: f32) -> Setup {
    let to = [target[0] - start[0], target[1] - start[1], target[2] - start[2]];
    let d = len(to);
    let dir = if d > 0.0 { unit(to) } else { [0.0; 3] };
    Setup {
        kind: 0,
        visual: 23,
        element: 1,
        owner: None,
        start,
        velocity: [dir[0] * speed, dir[1] * speed, dir[2] * speed],
        lifetime: if speed > 0.0 { (d / speed) as i32 + 20 } else { 20 },
        damage: 3.0,
        radius: 2.0,
        spin: 0.0,
        area_radius: 0.0,
        area_duration: 0,
        key: -1,
        ignore_world: false,
        status: (-1, 0, 0),
        effects: [528, -1, 991, -1, 991, 991],
        trail: -1,
    }
}

impl Pool {
    /// A live shot among the first ten slots with hit selector 0 (reflected; SPEC-0044 1056).
    pub fn deflected(&self) -> bool {
        self.slots.iter().take(10).flatten().any(|s| s.at_owner)
    }

    /// Claim the first free slot; a full pool creates nothing.
    pub fn submit(&mut self, s: Setup) -> bool {
        match self.slots.iter_mut().find(|x| x.is_none()) {
            Some(slot) => {
                *slot = Some(Shot {
                    s,
                    position: s.start,
                    velocity: s.velocity,
                    life: s.lifetime,
                    area: false,
                    at_owner: false,
                    pulse: 0,
                    area_left: s.area_duration,
                });
                true
            }
            None => false,
        }
    }

    /// All slots reset (a world start).
    pub fn reset(&mut self) {
        self.slots = [None; SLOTS];
    }

    pub fn live(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    /// One pool update, slots 0 to 9 in order.
    pub fn tick(&mut self, env: &Env) -> Vec<Event> {
        let mut out = Vec::new();
        for slot in &mut self.slots {
            let Some(p) = slot.as_mut() else { continue };
            let release = |p: &Shot, out: &mut Vec<Event>| {
                if p.s.kind == 8 {
                    out.push(Event::Summon { key: p.s.key, at: p.position });
                }
            };
            if p.s.owner.is_some_and(|o| !(env.owner_exists)(o)) {
                release(p, &mut out);
                *slot = None;
                continue;
            }
            if p.area {
                // Area mode (kind 1 after landing): the hero's pulse, then the countdowns. The
                // landed shot no longer moves (R-0016).
                let h = (env.hero[0] - p.position[0]).hypot(env.hero[2] - p.position[2]);
                if p.pulse == 0 && h < p.s.area_radius && env.hero_gap < 1.0 {
                    p.pulse = 60;
                    out.push(Event::Hero { damage: p.s.damage.max(0.0), owner: p.s.owner, blocked: false, element: p.s.element });
                }
                p.area_left -= 1;
                if p.pulse > 0 {
                    p.pulse -= 1;
                }
                if p.area_left < 1 {
                    *slot = None;
                }
                continue;
            }
            if p.life < 0 {
                effect(p, 3, p.position, &mut out);
                *slot = None;
                continue;
            }
            // Kind 9, the sinking poison cloud (SPEC-0016 section 10): more than 2.0 above the
            // ground under the hero it sinks 0.1, else it drifts 0.1 toward the hero
            // horizontally; within its radius of a hero without poison, the status. The launch
            // velocity is not used and the cloud makes no hit.
            if p.s.kind == 9 {
                let g = (env.ground)(env.hero[0], env.hero[2], env.hero[1] + 2.5).unwrap_or(env.hero[1]);
                if p.position[1] > g + 2.0 {
                    p.position[1] -= 0.1;
                } else {
                    let d = [env.hero[0] - p.position[0], env.hero[2] - p.position[2]];
                    let l = d[0].hypot(d[1]);
                    if l > 0.0 {
                        let k = (0.1 / l).min(1.0);
                        p.position[0] += d[0] * k;
                        p.position[2] += d[1] * k;
                    }
                }
                let h = (env.hero[0] - p.position[0]).hypot(env.hero[2] - p.position[2]);
                if h < p.s.area_radius && !env.poisoned && p.s.status.0 != -1 {
                    out.push(Event::Status { code: p.s.status.0, chance: p.s.status.1, seconds: p.s.status.2 });
                }
                p.life -= 1;
                continue;
            }
            // Motion.
            if matches!(p.s.kind, 1 | 8) {
                p.velocity[1] -= GRAVITY;
            }
            if p.s.kind == 17 {
                let u = unit(p.velocity);
                let w = unit(sub([env.hero[0], env.hero[1] + 2.0, env.hero[2]], p.position));
                p.velocity = std::array::from_fn(|k| 0.75 * u[k] + 0.25 * w[k]);
            }
            for k in 0..3 {
                p.position[k] += p.velocity[k];
            }
            // A spell touches only its target (SPEC-0038 Behavior 43).
            if matches!(p.s.kind, 2..=7) {
                if let Some((base, rb, hb)) = (env.target_body)(p.s.key as u64) {
                    if sphere_meets_body(p.position, p.s.radius, base, rb, hb) {
                        out.push(Event::SpellHit { kind: p.s.kind, target: p.s.key as u64 });
                        effect(p, 5, p.position, &mut out);
                        *slot = None;
                        continue;
                    }
                }
            }
            // Character hit test: the hero (hit selector 1) or, after a reflection, the owner;
            // spells and lobbed shots skip it.
            if !matches!(p.s.kind, 2..=8) {
                if p.at_owner {
                    let body = p.s.owner.and_then(|o| (env.owner_body)(o).map(|b| (o, b)));
                    if let Some((o, (base, rb, hb))) = body {
                        if sphere_meets_body(p.position, p.s.radius, base, rb, hb) {
                            out.push(Event::Owner { id: o, damage: p.s.damage.max(0.0) });
                            *slot = None;
                            continue;
                        }
                    }
                } else if sphere_meets_body(p.position, p.s.radius, env.hero, env.hero_radius, env.hero_height) {
                    let d = unit(p.velocity);
                    let f = env.hero_facing;
                    let blocked = env.guard && d[0] * f[0] + d[1] * f[1] + d[2] * f[2] < 0.0;
                    let visual_ok = matches!(p.s.visual, 0..=5 | 7..=14 | 16 | 23);
                    // Shields have element kind 0, so the element rule passes for element 0 only.
                    let reflectable = blocked && matches!(p.s.kind, 0 | 10) && visual_ok && env.shield && p.s.element == 0;
                    let owner = p.s.owner.and_then(|o| (env.owner_body)(o));
                    if let (true, Some((base, _, _))) = (reflectable, owner) {
                        let speed = len(p.velocity);
                        let aim = [base[0], base[1] + 2.0, base[2]];
                        p.velocity = unit(sub(aim, p.position)).map(|c| c * speed);
                        if p.s.kind == 10 {
                            p.s.damage = 0.0;
                        }
                        p.at_owner = true;
                        effect(p, 2, p.position, &mut out);
                        out.push(Event::Reflected { element: p.s.element });
                    } else {
                        let d = p.s.damage.max(0.0);
                        effect(p, if blocked { 2 } else { 5 }, p.position, &mut out);
                        if blocked {
                            out.push(Event::Hero { damage: d, owner: p.s.owner, blocked: true, element: p.s.element });
                        } else {
                            out.push(Event::Hero { damage: d, owner: p.s.owner, blocked: false, element: p.s.element });
                            if p.s.kind == 14 {
                                out.push(Event::MoteEater);
                            }
                            if p.s.status.0 != -1 {
                                out.push(Event::Status { code: p.s.status.0, chance: p.s.status.1, seconds: p.s.status.2 });
                            }
                        }
                        if p.s.kind == 15 {
                            out.push(Event::Leach);
                        }
                        *slot = None;
                        continue;
                    }
                }
            }
            // World collision, once more than 2.0 from the start point.
            if len(sub(p.position, p.s.start)) > 2.0 {
                let ground = (env.ground)(p.position[0], p.position[2], p.position[1] + 2.5);
                if let Some(h) = ground.filter(|&h| p.position[1] - p.s.radius < h) {
                    if !(p.s.kind == 9 || (p.s.ignore_world && !matches!(p.s.kind, 1..=8))) {
                        effect(p, 4, [p.position[0], h, p.position[2]], &mut out);
                    }
                    match p.s.kind {
                        1 => {
                            p.area = true;
                            p.pulse = 60;
                            p.velocity = [0.0; 3];
                        }
                        2..=8 => {
                            release(p, &mut out);
                            *slot = None;
                            continue;
                        }
                        9 => {}
                        _ if p.s.ignore_world => {}
                        _ => {
                            *slot = None;
                            continue;
                        }
                    }
                } else if !p.s.ignore_world {
                    let flown = (p.s.lifetime - p.life) as f32;
                    let back = flown.min(5.0);
                    let a = std::array::from_fn(|k| p.position[k] - p.velocity[k] * back);
                    if (env.wall)(a, p.position) {
                        release(p, &mut out);
                        *slot = None;
                        continue;
                    }
                }
            }
            p.life -= 1;
        }
        out
    }
}

/// Send effect id j of a shot when it is not -1.
fn effect(p: &Shot, j: usize, at: Vec3, out: &mut Vec<Event>) {
    if p.s.effects[j] != -1 {
        out.push(Event::Effect { id: p.s.effects[j], at });
    }
}

/// The missile values of one attack slot (SPEC-0038 Layout section 2, + 4k).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Missile {
    /// Flight mode (+0x2BC): 0 straight, 1 speed ramp with a ballistic arc, 2 homing.
    pub flight: i32,
    pub speed: f32,
    pub end_speed: f32,
    pub radius: f32,
    pub range: f32,
    pub inaccuracy: f32,
    pub spin: f32,
    pub damage: f32,
    /// The trigger band (bmin +0x18C, bmax +0x17C) the speed ramp reads.
    pub band: (f32, f32),
    pub flags: u32,
    pub visual: i32,
    pub area_duration: f32,
    pub area_radius: f32,
    pub key: u16,
    pub status: i32,
    pub status_chance: f32,
    pub status_range: (f32, f32),
    /// The six shot effect ids.
    pub effects: [i32; 6],
}

fn ballistic(launch: Vec3, aim: Vec3, s: f32) -> (Vec3, i32) {
    let off = [aim[0] - launch[0], 0.0, aim[2] - launch[2]];
    let d = len(off);
    let dy = aim[1] - launch[1];
    if s <= 0.0 || d <= 0.0 {
        return ([0.0, 0.0, 0.0], 1);
    }
    let t = d / s;
    let h = unit(off);
    let vy = (0.5 * GRAVITY * t * t + dy) / t;
    ([h[0] * s, vy, h[2] * s], (60.0 * t) as i32 + 1)
}

/// SPEC-0029 Behavior 13: build one shot for slot values m from the launch point toward the
/// hero (scatter e = U[0, inaccuracy] + 5.0 outside the camera cone + 3.0 while the attack
/// manager has the enemy waiting in line (SPEC-0038 Behavior 45), along a random horizontal unit
/// vector; drawn e, then x, then z). None for a flight mode other than 0, 1 or 2 without the
/// lobbed flag.
pub fn spawn(m: &Missile, element: i32, owner: u64, launch: Vec3, hero: Vec3, (out_of_cone, waiting): (bool, bool), rng: &mut Rng) -> Option<Setup> {
    let lobbed = m.flags & 0x02 != 0;
    if !lobbed && !(0..=2).contains(&m.flight) {
        return None;
    }
    let e = rng.frange(0.0, m.inaccuracy) + if out_of_cone { 5.0 } else { 0.0 } + if waiting { 3.0 } else { 0.0 };
    let wx = rng.frange(-1.0, 1.0);
    let wz = rng.frange(-1.0, 1.0);
    let w = unit([wx, 0.0, wz]);
    let straight = !lobbed && m.flight != 1;
    let target = if straight { [hero[0], hero[1] + 2.5, hero[2]] } else { hero };
    let aim = [target[0] + e * w[0], target[1], target[2] + e * w[2]];
    let v = m.speed;
    let status_value = if m.status != -1 { (rng.frange(m.status_range.0, m.status_range.1) / 60.0) as i32 } else { 0 };
    let mut s = Setup {
        kind: 0,
        visual: m.visual,
        element,
        owner: Some(owner),
        start: launch,
        velocity: [0.0; 3],
        lifetime: 0,
        damage: m.damage.max(0.0),
        radius: m.radius,
        spin: m.spin,
        area_radius: m.area_radius,
        area_duration: m.area_duration as i32,
        key: -1,
        ignore_world: m.flags & 0x10 != 0 && straight,
        status: (m.status, m.status_chance as i32, status_value),
        effects: m.effects,
        trail: -1,
    };
    let life = |v: f32| if v > 0.0 { (m.range / v) as i32 } else { 0 };
    if lobbed {
        let (vel, l) = ballistic(launch, aim, v);
        s = Setup { kind: 8, visual: 23, damage: 3.0, radius: 1.0, velocity: vel, lifetime: l, key: m.key as i32, ..s };
    } else {
        match m.flight {
            0 => {
                s.velocity = unit(sub(aim, launch)).map(|c| c * v);
                s.lifetime = life(v);
            }
            2 => {
                s.kind = 17;
                s.velocity = unit(sub(aim, launch)).map(|c| c * v);
                s.lifetime = 2 * life(v);
            }
            _ => {
                let (bmin, bmax) = m.band;
                let d = (aim[0] - launch[0]).hypot(aim[2] - launch[2]);
                let u = if bmax != bmin { 1.0 - (d - bmin) / (bmax - bmin) } else { 0.0 };
                let sp = v + u * u * (m.end_speed - v);
                let (vel, l) = ballistic(launch, aim, sp);
                s.kind = 1;
                s.velocity = vel;
                s.lifetime = l;
            }
        }
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_mote_vectors() {
        let s = fire_mote([0.0; 3], [3.0, 4.0, 0.0], 0.5);
        assert!((s.velocity[0] - 0.3).abs() < 1e-6 && (s.velocity[1] - 0.4).abs() < 1e-6);
        assert_eq!(s.lifetime, 30);
        assert_eq!(fire_mote([0.0; 3], [0.0, 0.0, 1.0], 0.9).lifetime, 21);
        // The courtyard's locators 101 and 102: 65 ticks.
        assert_eq!(fire_mote([-53.94, 37.14, 421.53], [-16.82, 20.21, 423.77], 0.9).lifetime, 65);
    }

    fn env<'a>(hero: Vec3, owner: &'a dyn Fn(u64) -> bool, ground: &'a dyn Fn(f32, f32, f32) -> Option<f32>, wall: &'a dyn Fn(Vec3, Vec3) -> bool) -> Env<'a> {
        Env {
            hero,
            hero_radius: 0.5,
            hero_height: 3.0,
            hero_gap: 0.0,
            hero_facing: [0.0, 0.0, -1.0],
            guard: false,
            shield: false,
            poisoned: false,
            owner_exists: owner,
            owner_body: &|_| Some(([0.0; 3], 0.5, 3.0)),
            target_body: &|_| None,
            ground,
            wall,
        }
    }

    #[test]
    fn poison_cloud() {
        // Kind 9: sinks while more than 2.0 above the ground under the hero, then drifts 0.1;
        // within 6.0 of an unpoisoned hero the status (2 for 15 s); at 6.0 nothing.
        let (o, g, w) = (|_| true, |_: f32, _: f32, _: f32| Some(0.0), |_, _| false);
        let mut pool = Pool::default();
        let cloud = Setup { kind: 9, visual: 15, element: 0, owner: None, start: [0.0, 3.0, 0.0], velocity: [0.0, 0.0, 2.0], lifetime: 300, damage: 0.0, radius: 1.0, spin: 0.0, area_radius: 6.0, area_duration: 0, key: 0, ignore_world: true, status: (2, 100, 15), effects: [414, -1, -1, -1, -1, -1], trail: -1 };
        pool.submit(cloud);
        let ev = pool.tick(&env([0.0, 0.0, 6.1], &o, &g, &w));
        let c = pool.slots[0].unwrap().position;
        assert!((c[1] - 2.9).abs() < 1e-5 && c[2] == 0.0 && ev.is_empty());
        let mut pool = Pool::default();
        pool.submit(Setup { start: [0.0, 1.0, 0.0], ..cloud });
        let ev = pool.tick(&env([0.0, 0.0, 6.0], &o, &g, &w));
        assert!((pool.slots[0].unwrap().position[2] - 0.1).abs() < 1e-5);
        assert_eq!(ev, vec![Event::Status { code: 2, chance: 100, seconds: 15 }]);
        let mut e = env([0.0, 0.0, 6.0], &o, &g, &w);
        e.poisoned = true;
        assert!(pool.tick(&e).is_empty());
    }

    fn archer() -> Missile {
        Missile { flight: 0, speed: 2.0, range: 70.0, radius: 0.05, damage: 10.0, visual: 23, status: 7, status_chance: 100.0, status_range: (600.0, 1800.0), effects: [-1; 6], ..Missile::default() }
    }

    #[test]
    fn archer_arrow_hits_on_tick_ten() {
        // SPEC-0029 test vector: launch (0, 2, 0), hero at (0, 0, 20), no scatter.
        let mut rng = Rng::seeded(1);
        let s = spawn(&archer(), 1, 7, [0.0, 2.0, 0.0], [0.0, 0.0, 20.0], (false, false), &mut rng).unwrap();
        assert_eq!(s.lifetime, 35);
        assert!((s.velocity[1] - 0.04998).abs() < 1e-4 && (s.velocity[2] - 1.99938).abs() < 1e-4);
        assert!((10..=30).contains(&s.status.2));
        let mut pool = Pool::default();
        assert!(pool.submit(s));
        let (o, g, w) = (|_| true, |_, _, _| None, |_, _| false);
        let e = env([0.0, 0.0, 20.0], &o, &g, &w);
        for tick in 1..=10 {
            let ev = pool.tick(&e);
            if tick < 10 {
                assert!(ev.is_empty(), "tick {tick}");
            } else {
                assert_eq!(ev[0], Event::Hero { damage: 10.0, owner: Some(7), blocked: false, element: 1 });
                assert!(matches!(ev[1], Event::Status { code: 7, chance: 100, seconds: 10..=30 }), "fire at chance 100");
            }
        }
        assert_eq!(pool.live(), 0);
    }

    /// Behavior 9's six effect ids: the unblocked hit plays id 5 at the shot, expiry id 3, and
    /// -1 plays nothing.
    #[test]
    fn shot_effect_ids() {
        let mut rng = Rng::seeded(1);
        let m = Missile { effects: [1400, 1400, 1399, -1, 1399, 1398], ..archer() };
        let s = spawn(&m, 1, 7, [0.0, 2.0, 0.0], [0.0, 0.0, 20.0], (false, false), &mut rng).unwrap();
        let mut pool = Pool::default();
        pool.submit(s);
        let (o, g, w) = (|_| true, |_, _, _| None, |_, _| false);
        let e = env([0.0, 0.0, 20.0], &o, &g, &w);
        let ev: Vec<Event> = (0..10).flat_map(|_| pool.tick(&e)).collect();
        assert!(matches!(ev[0], Event::Effect { id: 1398, at } if at[2] > 19.0));
        // Expiry with id 3 = -1: nothing.
        let s = spawn(&m, 1, 7, [0.0, 2.0, 0.0], [0.0, 0.0, 200.0], (false, false), &mut rng).unwrap();
        pool.submit(s);
        let far = env([0.0, 0.0, 500.0], &o, &g, &w);
        let ev: Vec<Event> = (0..60).flat_map(|_| pool.tick(&far)).collect();
        assert!(ev.is_empty() && pool.live() == 0);
    }

    #[test]
    fn arrow_expires_after_thirty_six_moves() {
        let mut rng = Rng::seeded(1);
        let s = spawn(&archer(), 1, 7, [0.0, 2.0, 0.0], [0.0, 0.0, 20.0], (false, false), &mut rng).unwrap();
        let mut pool = Pool::default();
        pool.submit(s);
        let (o, g, w) = (|_| true, |_, _, _| None, |_, _| false);
        let e = env([500.0, 0.0, 500.0], &o, &g, &w);
        for _ in 0..36 {
            pool.tick(&e);
        }
        let z = pool.slots[0].unwrap().position[2];
        assert!((z - 71.977).abs() < 0.01, "{z}");
        pool.tick(&e);
        assert_eq!(pool.live(), 0, "reset at the start of update 37");
    }

    #[test]
    fn homing_turns_toward_the_hero() {
        let mut pool = Pool::default();
        let mut s = spawn(&archer(), 0, 1, [0.0, 2.0, 0.0], [0.0, 0.0, 20.0], (false, false), &mut Rng::seeded(1)).unwrap();
        s.kind = 17;
        s.velocity = [0.0, 0.0, 1.0];
        s.lifetime = 200;
        pool.submit(s);
        let (o, g, w) = (|_| true, |_, _, _| None, |_, _| false);
        let e = env([10.0, 0.0, 0.0], &o, &g, &w);
        pool.tick(&e);
        let v = pool.slots[0].unwrap().velocity;
        assert!((v[0] - 0.25).abs() < 1e-5 && (v[2] - 0.75).abs() < 1e-5);
        pool.tick(&e);
        let v = pool.slots[0].unwrap().velocity;
        assert!((v[0] - 0.4864).abs() < 1e-3 && (v[2] - 0.6923).abs() < 1e-3, "{v:?}");
    }

    #[test]
    fn flower_seed_ballistic_launch() {
        // SPEC-0038 vector: v 0.5, v_end 0.1, band 3 to 60; launch (0, 1, 0), hero (20, 0, 0).
        let m = Missile { flight: 1, speed: 0.5, end_speed: 0.1, band: (3.0, 60.0), effects: [-1; 6], ..Missile::default() };
        let s = spawn(&m, 0, 1, [0.0, 1.0, 0.0], [20.0, 0.0, 0.0], (false, false), &mut Rng::seeded(1)).unwrap();
        assert_eq!((s.kind, s.lifetime), (1, 3961));
        assert!((s.velocity[0] - 0.30302).abs() < 1e-4, "{:?}", s.velocity);
        assert!((s.velocity[1] - 0.27820).abs() < 1e-4, "{:?}", s.velocity);
    }

    #[test]
    fn full_pool_and_area_pulse() {
        let mut pool = Pool::default();
        let s = Setup {
            kind: 1,
            visual: 0,
            element: 0,
            owner: None,
            start: [0.0, 5.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            lifetime: 100,
            damage: 4.0,
            radius: 0.2,
            spin: 0.0,
            area_radius: 3.0,
            area_duration: 200,
            key: -1,
            ignore_world: false,
            status: (-1, 0, 0),
            effects: [-1; 6],
            trail: -1,
        };
        for _ in 0..SLOTS {
            assert!(pool.submit(s));
        }
        assert!(!pool.submit(s), "a full pool creates nothing");
        pool.reset();
        pool.submit(s);
        let (o, g, w) = (|_| true, |_, _, _| Some(4.9), |_, _| false);
        let e = Env {
            hero: [3.5, 4.9, 0.0],
            hero_radius: 0.1,
            hero_height: 0.1,
            hero_gap: 0.0,
            hero_facing: [1.0, 0.0, 0.0],
            guard: false,
            shield: false,
            poisoned: false,
            owner_exists: &o,
            owner_body: &|_| None,
            target_body: &|_| None,
            ground: &g,
            wall: &w,
        };
        // It lands once past 2.0 from the start (tick 2); the first pulse comes 61 ticks later,
        // then one every 60 (the timer is set to 60 and counts down the same tick).
        let mut hits = Vec::new();
        for t in 1..=130 {
            if !pool.tick(&e).is_empty() {
                hits.push(t);
            }
        }
        assert!(pool.slots[0].is_some_and(|p| p.area) || pool.live() == 0);
        assert_eq!(hits, vec![63, 123]);
    }

    #[test]
    fn guard_blocks_and_reflects() {
        let mut rng = Rng::seeded(1);
        let s = spawn(&archer(), 0, 7, [0.0, 2.0, 0.0], [0.0, 0.0, 20.0], (false, false), &mut rng).unwrap();
        let (o, g, w) = (|_| true, |_, _, _| None, |_, _| false);
        let mut e = env([0.0, 0.0, 20.0], &o, &g, &w);
        e.guard = true;
        // Facing the shot without a shield: blocked, not reflected, max(1, 0.25 x 10).
        let mut pool = Pool::default();
        pool.submit(s);
        let ev: Vec<Event> = (0..10).flat_map(|_| pool.tick(&e)).collect();
        assert_eq!(ev, vec![Event::Hero { damage: 10.0, owner: Some(7), blocked: true, element: 0 }]);
        assert_eq!((taken(10.0, true), taken(2.0, true), taken(10.0, false)), (2.5, 1.0, 10.0));
        // With a shield, an element-0 straight shot is sent back at its owner (at the origin).
        e.shield = true;
        let mut pool = Pool::default();
        pool.submit(s);
        let ev: Vec<Event> = (0..30).flat_map(|_| pool.tick(&e)).collect();
        assert_eq!(ev, vec![Event::Reflected { element: 0 }, Event::Owner { id: 7, damage: 10.0 }]);
        // From behind (facing away from the shot) the guard does not block.
        e.hero_facing = [0.0, 0.0, 1.0];
        let mut pool = Pool::default();
        pool.submit(s);
        let ev: Vec<Event> = (0..10).flat_map(|_| pool.tick(&e)).collect();
        assert_eq!(ev[0], Event::Hero { damage: 10.0, owner: Some(7), blocked: false, element: 0 });
    }

    #[test]
    fn body_sphere_reach() {
        let base = [0.0; 3];
        assert!(sphere_meets_body([0.54, 1.0, 0.0], 0.05, base, 0.5, 3.0));
        assert!(!sphere_meets_body([0.56, 1.0, 0.0], 0.05, base, 0.5, 3.0));
        assert!(sphere_meets_body([0.0, 3.04, 0.0], 0.05, base, 0.5, 3.0), "top reach Hb + r on the axis");
        assert!(!sphere_meets_body([0.0, 3.06, 0.0], 0.05, base, 0.5, 3.0));
    }
}
