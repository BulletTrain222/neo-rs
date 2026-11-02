//! Breakable debris (SPEC-0048 Piece flight; R-0187): a break starts one emitter per piece record.
//! Its particles fly by gravity and drag from the breakable's position, with no collision and no
//! fade, and are removed at the end of their life. A model piece keeps the entry's rows and turns
//! about a random axis; a sprite piece turns its quad.

use crate::rng::Rng;

pub type Vec3 = [f32; 3];

/// The world's spawner pool holds 40 elements; a record with none free is skipped.
const SPAWNERS: usize = 40;

/// One type word's emitter preset (SPEC-0034 3.9, the table of SPEC-0048 Piece flight).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Preset {
    life: f32,
    speed: (f32, f32),
    cone: f32,
    gravity: f32,
    drag: f32,
    spin: (f32, f32),
    size: (f32, f32),
}

fn preset(kind: u16) -> Option<Preset> {
    Some(match kind {
        1 => Preset { life: 5.0, speed: (9.0, 1.5), cone: 0.6289, gravity: -18.199, drag: 0.0117, spin: (2.8008, 0.8008), size: (1.0, 0.0) },
        2 => Preset { life: 1.2, speed: (15.5, 4.398), cone: 0.8477, gravity: -25.801, drag: 0.0508, spin: (0.0, 17.0), size: (1.0, 0.0) },
        3 => Preset { life: 0.8, speed: (3.3984, 0.8008), cone: 1.2578, gravity: 0.1484, drag: 0.0117, spin: (0.0, 0.0), size: (0.3, 0.95) },
        5 => Preset { life: 2.2, speed: (0.1016, 0.1016), cone: 1.5703, gravity: 0.733, drag: 3.699, spin: (0.0, 0.0), size: (0.8, 0.1) },
        _ => return None,
    })
}

/// One piece record, as the host hands it over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
    pub kind: u16,
    /// The record's index in its breakable, which names the model or the material to draw.
    pub piece: usize,
    pub model: bool,
    /// The total count of its timing record (types 2 and 3; type 1 throws one, type 5 four).
    pub count: u32,
    /// Half-extent factors of the sprite lanes.
    pub half: Vec3,
}

/// One particle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    /// Unique, for the host's drawn roots.
    pub id: u64,
    pub pack: u32,
    pub breakable: usize,
    pub piece: usize,
    pub model: bool,
    pub position: Vec3,
    velocity: Vec3,
    /// Ticks before a later burst's particle starts.
    delay: i32,
    age: f32,
    p: Preset,
    /// The entry's rows at birth, a model piece's stored frame.
    rows: [Vec3; 3],
    axis: Vec3,
    spin_rate: f32,
    pub spin: f32,
    half: Vec3,
    pub alpha: f32,
    emitter: u64,
}

impl Piece {
    /// Whether it has started flying (a later burst's particles wait).
    pub fn shown(&self) -> bool {
        self.delay <= 0
    }

    /// A sprite's half extents: the factors times the size row.
    pub fn half(&self) -> Vec3 {
        let s = self.p.size.0 + self.p.size.1 * (self.age / self.p.life).min(1.0);
        self.half.map(|f| f * s)
    }

    /// A model piece's draw rows: the stored frame turned by the spin angle about its axis.
    pub fn rows(&self) -> [Vec3; 3] {
        let (s, c) = self.spin.sin_cos();
        let a = self.axis;
        self.rows.map(|v| {
            let d = dot(a, v) * (1.0 - c);
            let x = cross(a, v);
            std::array::from_fn(|k| v[k] * c + x[k] * s + a[k] * d)
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Pool {
    pub pieces: Vec<Piece>,
    next: u64,
    emitters: u64,
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 1e-6 { a.map(|v| v / l) } else { [0.0, 1.0, 0.0] }
}

/// The launch axis: normalize((1 - p) x own axis + p x d).
pub fn launch_axis(own: Vec3, push: Option<Vec3>) -> Vec3 {
    match push {
        Some(d) => unit(std::array::from_fn(|k| 0.75 * own[k] + 0.25 * d[k])),
        None => unit(own),
    }
}

impl Pool {
    /// A direction inside the cone about `axis`: the polar angle uniform in 0 to the cone, the
    /// azimuth uniform around it.
    fn spread(axis: Vec3, cone: f32, rng: &mut Rng) -> Vec3 {
        let (polar, az) = (rng.frange(0.0, cone.max(1e-6)), rng.frange(0.0, std::f32::consts::TAU));
        let side = unit(cross(axis, if axis[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] }));
        let other = cross(axis, side);
        let (sp, cp) = polar.sin_cos();
        let (sa, ca) = az.sin_cos();
        std::array::from_fn(|k| axis[k] * cp + (side[k] * ca + other[k] * sa) * sp)
    }

    /// One break: one emitter per record, in order, from the entry's position and rows. `push` is the
    /// unit direction from the attacker to the entry for an attack contact.
    pub fn start(&mut self, pack: u32, breakable: usize, at: Vec3, rows: [Vec3; 3], push: Option<Vec3>, records: &[Record], rng: &mut Rng) {
        let axis = launch_axis(rows[1], push);
        for r in records {
            let Some(p) = preset(r.kind) else { continue };
            let mut live: Vec<u64> = self.pieces.iter().map(|x| x.emitter).collect();
            live.sort_unstable();
            live.dedup();
            if live.len() >= SPAWNERS {
                continue;
            }
            self.emitters += 1;
            let emitter = self.emitters;
            // Type 3's cap of 6 a burst gives three whole bursts about 0.12 s apart; type 5 one of 4.
            let (n, burst) = match r.kind {
                1 => (1, 1),
                2 => (r.count.max(1), r.count.max(1)),
                3 => ((r.count / 6) * 6, 6),
                _ => (4, 4),
            };
            for k in 0..n {
                self.next += 1;
                let dir = Self::spread(axis, p.cone, rng);
                let speed = p.speed.0 + rng.frange(-p.speed.1, p.speed.1.max(-p.speed.1 + 1e-6));
                let spin_rate = p.spin.0 + if p.spin.1 > 0.0 { rng.frange(-p.spin.1, p.spin.1) } else { 0.0 };
                let spin_axis = unit([rng.frange(-1.0, 1.0), rng.frange(-1.0, 1.0), rng.frange(-1.0, 1.0)]);
                self.pieces.push(Piece {
                    id: self.next,
                    pack,
                    breakable,
                    piece: r.piece,
                    model: r.model,
                    position: at,
                    velocity: dir.map(|v| v * speed),
                    delay: (k / burst) as i32 * 7,
                    age: 0.0,
                    p,
                    rows,
                    axis: spin_axis,
                    spin_rate,
                    spin: 0.0,
                    half: r.half,
                    alpha: if r.kind == 3 { 0.8 } else { 1.0 },
                    emitter,
                });
            }
        }
    }

    /// One tick (dt = 1/60 s): v = (v + (0, g dt, 0)) x (1 - k dt), position + v dt. Removed once the
    /// age passes the life, abruptly.
    pub fn step(&mut self) {
        let dt = 1.0 / 60.0;
        self.pieces.retain_mut(|q| {
            if q.delay > 0 {
                q.delay -= 1;
                return true;
            }
            q.velocity[1] += q.p.gravity * dt;
            let keep = 1.0 - q.p.drag * dt;
            q.velocity = q.velocity.map(|v| v * keep);
            for k in 0..3 {
                q.position[k] += q.velocity[k] * dt;
            }
            q.spin += q.spin_rate * dt;
            q.age += dt;
            q.age <= q.p.life
        });
    }

    /// Only the resident worlds keep their pieces.
    pub fn retain_packs(&mut self, keep: impl Fn(u32) -> bool) {
        self.pieces.retain(|q| keep(q.pack));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: [Vec3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    fn record(kind: u16, count: u32) -> Record {
        Record { kind, piece: 0, model: kind == 1, count, half: [0.1; 3] }
    }

    /// SPEC-0048's worked example: pack 732, entry 0.
    #[test]
    fn flight_of_the_worked_example() {
        let axis = launch_axis([0.0, 1.0, 0.0], Some([1.0, 0.0, 0.0]));
        assert!((axis[0] - 0.316).abs() < 0.001 && (axis[1] - 0.949).abs() < 0.001);
        let mut rng = Rng::seeded(1);
        let mut pool = Pool::default();
        pool.start(732, 0, [3834.924, 6.345, 343.157], ROWS, None, &[record(1, 1)], &mut rng);
        pool.pieces[0].velocity = [2.846, 8.538, 0.0];
        pool.pieces[0].spin_rate = 0.0;
        pool.step();
        let q = pool.pieces[0];
        assert!((q.position[0] - 3834.971).abs() < 0.001 && (q.position[1] - 6.482).abs() < 0.001, "{:?}", q.position);
        assert!((q.velocity[0] - 2.845).abs() < 0.001 && (q.velocity[1] - 8.233).abs() < 0.001);
        for _ in 1..299 {
            pool.step();
        }
        // It passes through the floor (no collision) and goes at the end of its 5.0 s.
        assert!((pool.pieces[0].position[1] + 176.0).abs() < 3.0, "{:?}", pool.pieces[0].position);
        for _ in 0..3 {
            pool.step();
        }
        assert!(pool.pieces.is_empty());
    }

    #[test]
    fn counts_by_type_and_the_spawner_limit() {
        let mut rng = Rng::seeded(2);
        let mut pool = Pool::default();
        pool.start(1, 0, [0.0; 3], ROWS, None, &[record(1, 1), record(2, 9), record(3, 20), record(5, 4)], &mut rng);
        assert_eq!(pool.pieces.len(), 1 + 9 + 18 + 4);
        // Type 3's bursts: 6 start at once, the others wait.
        assert_eq!(pool.pieces.iter().filter(|q| !q.shown()).count(), 12);
        let mut pool = Pool::default();
        let many = vec![record(1, 1); 50];
        pool.start(1, 0, [0.0; 3], ROWS, None, &many, &mut rng);
        assert_eq!(pool.pieces.len(), SPAWNERS);
    }
}
