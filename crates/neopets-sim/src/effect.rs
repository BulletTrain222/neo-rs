//! Effects (SPEC-0034 Behavior sections 3 and 4): effect objects copied from the resource-9
//! definitions, their spawners (one per emitter node) with the cycle clock of 3.2, and the
//! particles with birth (4.1), motion (4.2) and the colour and size curves (4.6). The clocks and
//! curve sums use the console's truncating single-precision adds (SPEC-0008).
//!
//! Light emitters publish their point lights (4.5) through `Effects::lights`.
//!
//! Not covered: the swing trail, the pulse kind, presets (3.9), the per-character and
//! per-world pools (only the shared pools of group 3). Birth modes 2 and 3 place particles along
//! the segment from the spawner to its target (4.1 step 2); `set_target` writes the target.
use ps2_float::fpu::{add, div, mul, sub};

const DEF_SIZE: usize = 0x90;
const NODE_SIZE: usize = 0x70;
const A_SIZE: usize = 0x90;
const B_SIZE: usize = 0x90;
const C_SIZE: usize = 0x50;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    u32_at(b, at).map(|v| v as i32)
}

fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    u32_at(b, at).map(f32::from_bits)
}

fn vec4_at(b: &[u8], at: usize) -> Option<[f32; 4]> {
    Some([f32_at(b, at)?, f32_at(b, at + 4)?, f32_at(b, at + 8)?, f32_at(b, at + 12)?])
}

/// An 8.8 fixed-point field (value = s16 / 256; 2.6).
fn fixed_at(b: &[u8], at: usize) -> Option<f32> {
    Some(i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?) as f32 / 256.0)
}

/// A definition (2.2).
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub id: i32,
    pub active: u32,
    /// 0: stop after the first emitter that starts; else start every matching one.
    pub multi: u32,
    /// Squared near/far distance.
    pub threshold: f32,
    /// Node indices of the first n emitters.
    pub nodes: Vec<usize>,
    /// Motion scale S.
    pub scale: f32,
    /// Squared cull distance; -1 never.
    pub cull: f32,
    pub cone: bool,
}

/// An emitter node (2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Local transform: rows X, Y, Z and T.
    pub transform: [[f32; 4]; 4],
    pub link: i32,
    pub c: usize,
    pub a: usize,
    pub b: usize,
    /// Material group of the resource-9 shader list; None for a model particle.
    pub material: Option<usize>,
    pub child: i32,
    pub preset: u16,
    pub ring_fill: u8,
    pub child_gate: u8,
    pub ring: f32,
    /// Sprite half-extent factors per lane (x, y, z), read from +0x6C, +0x6E and +0x6A.
    pub factors: [f32; 3],
}

/// Emitter record A, timing and gates (2.4).
#[derive(Clone, Debug, PartialEq)]
pub struct RecordA {
    pub size: [f32; 4],
    pub size_rates: [[f32; 4]; 3],
    pub thresholds: [[f32; 4]; 2],
    pub end_gate: i32,
    pub per_burst: i32,
    pub period: f32,
    pub period_jitter: f32,
    pub cycles: i32,
    pub life: f32,
    pub life_jitter: f32,
    pub birth_mode: i32,
    pub gate: u8,
    pub follow: bool,
    pub looped: bool,
    pub light: bool,
}

/// Emitter record B, the colour curve (2.5).
#[derive(Clone, Debug, PartialEq)]
pub struct RecordB {
    /// Row 0 is the start value and the rate for key index 0; rows 1 to 5 are rates.
    pub rows: [[f32; 4]; 6],
    pub rgb_keys: [f32; 6],
    pub alpha_keys: [f32; 6],
}

/// Emitter record C, motion (2.6): (base, spread) pairs in 8.8 fixed point.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordC {
    pub extra: [f32; 4],
    pub accel: f32,
    pub drag: f32,
    pub cone_upper: f32,
    pub cone_lower: f32,
    pub speed: (f32, f32),
    pub clock: (f32, f32),
    pub spin: (f32, f32),
    pub kick: (f32, f32),
    pub kick_interval: (f32, f32),
    pub orient: u8,
    pub kick_zero: bool,
    pub spin_axis: u8,
    pub flag: u8,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectTable {
    pub definitions: Vec<Definition>,
    pub nodes: Vec<Node>,
    pub a: Vec<RecordA>,
    pub b: Vec<RecordB>,
    pub c: Vec<RecordC>,
    pub children: usize,
}

fn align16(v: usize) -> usize {
    (v + 15) & !15
}

impl EffectTable {
    /// Parse a type-34 chunk payload (2.1): six counts, then A, B and C, the child offset
    /// table, the definition offset table (aligned to 16) and the node array (aligned to 16).
    pub fn parse(p: &[u8]) -> Result<Self, String> {
        let count = |at: usize| -> Result<usize, String> {
            let v = i32_at(p, at).ok_or("Truncated effect table header")?;
            usize::try_from(v).ok().filter(|&n| n < 100_000).ok_or(format!("Effect table count {v} at {at:#x} is implausible"))
        };
        let (children, n_nodes, n_defs, n_a, n_b, n_c) = (count(0)?, count(4)?, count(8)?, count(0xC)?, count(0x10)?, count(0x14)?);
        let bad = |what: &str| format!("Effect table: {what} runs past the chunk");
        let a_at = 0x20;
        let b_at = a_at + n_a * A_SIZE;
        let c_at = b_at + n_b * B_SIZE;
        let child_at = c_at + n_c * C_SIZE;
        let def_table = align16(child_at + children * 4);
        let node_at = align16(def_table + n_defs * 4);
        let a = (0..n_a)
            .map(|i| {
                let r = p.get(a_at + i * A_SIZE..a_at + (i + 1) * A_SIZE).ok_or_else(|| bad("record A"))?;
                let v = |at| vec4_at(r, at).unwrap_or_default();
                Ok(RecordA {
                    size: v(0),
                    size_rates: [v(0x10), v(0x20), v(0x30)],
                    thresholds: [v(0x40), v(0x50)],
                    end_gate: i32_at(r, 0x60).unwrap_or(-1),
                    per_burst: i32_at(r, 0x64).unwrap_or(0),
                    period: f32_at(r, 0x6C).unwrap_or(-1.0),
                    period_jitter: f32_at(r, 0x70).unwrap_or(0.0),
                    cycles: i32_at(r, 0x74).unwrap_or(0),
                    life: f32_at(r, 0x78).unwrap_or(0.0),
                    life_jitter: f32_at(r, 0x7C).unwrap_or(0.0),
                    birth_mode: i32_at(r, 0x80).unwrap_or(0),
                    gate: r[0x84],
                    follow: r[0x85] != 0,
                    looped: r[0x86] != 0,
                    light: r[0x87] != 0,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let b = (0..n_b)
            .map(|i| {
                let r = p.get(b_at + i * B_SIZE..b_at + (i + 1) * B_SIZE).ok_or_else(|| bad("record B"))?;
                let keys = |at: usize| std::array::from_fn(|k| f32_at(r, at + 4 * k).unwrap_or(0.0));
                Ok(RecordB { rows: std::array::from_fn(|k| vec4_at(r, 0x10 * k).unwrap_or_default()), rgb_keys: keys(0x60), alpha_keys: keys(0x78) })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let c = (0..n_c)
            .map(|i| {
                let r = p.get(c_at + i * C_SIZE..c_at + (i + 1) * C_SIZE).ok_or_else(|| bad("record C"))?;
                let f = |at| fixed_at(r, at).unwrap_or(0.0);
                Ok(RecordC {
                    extra: vec4_at(r, 0x10).unwrap_or_default(),
                    accel: f(0x20),
                    drag: f(0x22),
                    cone_upper: f(0x24),
                    cone_lower: f(0x26),
                    speed: (f(0x28), f(0x2A)),
                    clock: (f(0x2C), f(0x2E)),
                    spin: (f(0x30), f(0x32)),
                    kick: (f(0x34), f(0x36)),
                    kick_interval: (f(0x38), f(0x3A)),
                    orient: r[0x3C],
                    kick_zero: r[0x3D] != 0,
                    spin_axis: r[0x3F],
                    flag: r[0x40],
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let nodes = (0..n_nodes)
            .map(|i| {
                let r = p.get(node_at + i * NODE_SIZE..node_at + (i + 1) * NODE_SIZE).ok_or_else(|| bad("emitter node"))?;
                let index = |at| i32_at(r, at).unwrap_or(-1);
                Ok(Node {
                    transform: std::array::from_fn(|k| vec4_at(r, 0x10 * k).unwrap_or_default()),
                    link: index(0x4C),
                    c: index(0x50).max(0) as usize,
                    a: index(0x54).max(0) as usize,
                    b: index(0x58).max(0) as usize,
                    material: usize::try_from(index(0x5C)).ok(),
                    child: index(0x60),
                    preset: u16::from_le_bytes([r[0x64], r[0x65]]),
                    ring_fill: r[0x66],
                    child_gate: r[0x67],
                    ring: fixed_at(r, 0x68).unwrap_or(0.0),
                    // SPEC-0034 2.3: lane x takes +0x6C, lane y +0x6E and lane z +0x6A.
                    factors: [0x6C, 0x6E, 0x6A].map(|at| fixed_at(r, at).unwrap_or(0.0)),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let definitions = (0..n_defs)
            .map(|i| {
                let entry = def_table + 4 * i;
                let rel = i32_at(p, entry).ok_or_else(|| bad("definition offset table"))?;
                let at = usize::try_from(entry as i64 + rel as i64).map_err(|_| format!("Definition {i} offset is negative"))?;
                let r = p.get(at..at + DEF_SIZE).ok_or_else(|| bad("definition"))?;
                let n = i32_at(r, 0x54).unwrap_or(0).clamp(0, 5) as usize;
                Ok(Definition {
                    id: i32_at(r, 0x40).unwrap_or(-1),
                    active: u32_at(r, 0x48).unwrap_or(0),
                    multi: u32_at(r, 0x4C).unwrap_or(0),
                    threshold: f32_at(r, 0x50).unwrap_or(0.0),
                    nodes: (0..n).filter_map(|k| usize::try_from(i32_at(r, 0x6C + 4 * k)?).ok().filter(|&x| x < n_nodes)).collect(),
                    scale: f32_at(r, 0x80).unwrap_or(1.0),
                    cull: f32_at(r, 0x84).unwrap_or(-1.0),
                    cone: r[0x88] != 0,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        if nodes.iter().any(|n| n.a >= a.len() || n.b >= b.len() || n.c >= c.len()) {
            return Err("Effect table: a node names a record out of range".into());
        }
        Ok(Self { definitions, nodes, a, b, c, children })
    }

    /// The first definition with this id (2.7).
    pub fn find(&self, id: i32) -> Option<usize> {
        self.definitions.iter().position(|d| d.id == id)
    }
}


/// The effect step, 1/60 s (Where it lives: the step word at 0x2ECCC4).
pub const DT: f32 = 1.0 / 60.0;
/// The shared group's pools (Where it lives, Pools).
pub const SPAWNERS: usize = 300;
pub const PARTICLES: usize = 800;
/// The cone limit, squared (0x2ECCC0).
pub const CONE_LIMIT: f32 = 225.0;

/// The uniform draw U(lo, hi) of 3.3.
pub trait Uniform {
    fn uniform(&mut self, lo: f32, hi: f32) -> f32;
}

impl Uniform for crate::rng::Rng {
    fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        if lo == hi {
            return lo;
        }
        lo + (hi - lo) * (self.next_u32() as f32 / 4_294_967_296.0)
    }
}

/// Every draw at the midpoint of its range (the spec's per-tick test vectors 70 to 72).
pub struct Midpoint;

impl Uniform for Midpoint {
    fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * 0.5
    }
}

/// The folded power series of section 12: P(fold(t)), sin t to about 5e-8 at every angle. The
/// fold takes the fraction of m = |t - pi/2| / 2 pi by flooring (the original's add-and-subtract
/// rounding trick floors on the console's truncating adder) and x = |f - 0.5| - 0.25, always in
/// -0.25 to 0.25 (the earlier round-to-nearest reading was withdrawn).
pub fn series_sin(t: f32) -> f32 {
    let m = (t - std::f32::consts::FRAC_PI_2).abs() / std::f32::consts::TAU;
    let f = m - m.floor();
    let x = (f - 0.5).abs() - 0.25;
    let x2 = x * x;
    let (c1, c3, c5, c7, c9) = (6.283185f32, 41.34167f32, 81.60223f32, 76.57496f32, 39.71066f32);
    x * (c1 - x2 * (c3 - x2 * (c5 - x2 * (c7 - x2 * c9))))
}

/// The cosine through the same series (sin of t + pi / 2).
pub fn series_cos(t: f32) -> f32 {
    series_sin(t + std::f32::consts::FRAC_PI_2)
}

type V3 = [f32; 3];

fn v_add(a: V3, b: V3) -> V3 {
    [add(a[0], b[0]), add(a[1], b[1]), add(a[2], b[2])]
}

fn v_scale(a: V3, s: f32) -> V3 {
    [mul(a[0], s), mul(a[1], s), mul(a[2], s)]
}

fn unit(a: V3) -> V3 {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if l == 0.0 { a } else { [a[0] / l, a[1] / l, a[2] / l] }
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// A frame: rows X, Y and Z (the axes) of an owner's matrix slot (5.1).
pub type Frame = [V3; 3];
pub const IDENTITY: Frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn apply(f: &Frame, v: V3) -> V3 {
    let mut o = [0.0; 3];
    for k in 0..3 {
        o[k] = f[0][k] * v[0] + f[1][k] * v[1] + f[2][k] * v[2];
    }
    o
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnerState {
    Running,
    Draining,
    Dead,
}

#[derive(Clone, Debug)]
pub struct Spawner {
    pub node: usize,
    pub state: SpawnerState,
    elapsed: f32,
    phase: f32,
    period: f32,
    pub cycles: i32,
    /// Bursts launched so far.
    pub bursts: u32,
    pub live: usize,
    pub position: V3,
    pub frame: Frame,
    previous: V3,
    target: V3,
    delta: V3,
    velocity: V3,
    scale: f32,
    last_born: Option<usize>,
    /// The effect object that keeps this spawner in a slot (3.4 Handle), if any.
    pub owner: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub spawner: usize,
    pub node: usize,
    pub position: V3,
    pub velocity: V3,
    accel: V3,
    drag: f32,
    pub life: f32,
    pub elapsed: f32,
    pub colour: [f32; 4],
    pub size: [f32; 4],
    rgb_key: usize,
    alpha_key: usize,
    kick_interval: f32,
    spin_rate: f32,
    pub spin: f32,
    pub half: V3,
    scale: f32,
    /// A model particle's stored frame (the spawner's at birth) and its random spin axis (spin
    /// axis code 0; SPEC-0034 4.2 step 3).
    frame: [V3; 3],
    axis: V3,
}

impl Particle {
    /// SPEC-0034 4.2 step 3: a model particle's draw frame. With orientation mode 0 the stored
    /// frame is turned by the spin angle about the axis of `code` (0 random, 1 X, 2 up, 3 Z, 4 to 6
    /// the frame's rows); each row is scaled by the size times the owner scale.
    fn draw_frame(&self, orient: u8, code: u8) -> [V3; 3] {
        let mut rows = self.frame;
        if orient == 0 && self.spin != 0.0 {
            let k = match code {
                1 => [1.0, 0.0, 0.0],
                2 => [0.0, 1.0, 0.0],
                3 => [0.0, 0.0, 1.0],
                4..=6 => unit(self.frame[code as usize - 4]),
                _ => self.axis,
            };
            let (s, c) = self.spin.sin_cos();
            rows = rows.map(|v| {
                let d = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
                let x = cross(k, v);
                std::array::from_fn(|i| v[i] * c + x[i] * s + k[i] * d * (1.0 - c))
            });
        }
        std::array::from_fn(|i| v_scale(rows[i], self.size[i] * self.scale))
    }

    /// The sprite colour bytes trunc(channel x 127) (4.1 step 7; 128 is 1.0).
    pub fn bytes(&self) -> [u8; 4] {
        self.colour.map(|c| mul(c, 127.0).max(0.0) as u8)
    }
}

/// An effect object (3.16): a copy of a definition with its position, frame, state and five
/// live spawner slots.
#[derive(Clone, Debug)]
pub struct EffectObject {
    pub def: Option<usize>,
    pub position: V3,
    pub frame: Frame,
    /// 0 near, 1 far.
    pub state: u8,
    pub slots: [Option<usize>; 5],
    /// A placed record's own cull (squared), used instead of the definition's.
    pub cull: Option<f32>,
    /// A placed record's own motion scale S (+0x80), used instead of the definition's
    /// (SPEC-0034 section 6, R-0267).
    pub scale: Option<f32>,
}

impl EffectObject {
    pub fn new(table: &EffectTable, id: i32, position: V3, frame: Frame) -> Self {
        Self { def: table.find(id), position, frame, state: 0, slots: [None; 5], cull: None, scale: None }
    }

    pub fn live(&self) -> bool {
        self.slots.iter().any(|s| s.is_some())
    }
}

/// The shared pools of group 3 and the effect objects that hold spawners.
#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub spawners: Vec<Option<Spawner>>,
    pub particles: Vec<Option<Particle>>,
    pub objects: Vec<Option<EffectObject>>,
}

impl Effects {
    pub fn new() -> Self {
        Self { spawners: vec![None; SPAWNERS], particles: vec![None; PARTICLES], objects: Vec::new() }
    }

    /// Add an effect object; returns its handle.
    pub fn add_object(&mut self, o: EffectObject) -> usize {
        match self.objects.iter().position(|s| s.is_none()) {
            Some(i) => {
                self.objects[i] = Some(o);
                i
            }
            None => {
                self.objects.push(Some(o));
                self.objects.len() - 1
            }
        }
    }

    /// The one-shot start of 3.16 (set up, place, start, detach with mode 2): the object's
    /// emitters start once with no listener and the object is released; its spawners run on.
    pub fn one_shot(&mut self, table: &EffectTable, id: i32, position: V3, frame: Frame, r: &mut impl Uniform) -> bool {
        let mut o = EffectObject::new(table, id, position, frame);
        if o.def.is_none() {
            return false;
        }
        self.start_emitters(table, &mut o, None, r);
        for s in o.slots.iter().flatten() {
            if let Some(sp) = self.spawners[*s].as_mut() {
                sp.owner = None;
            }
        }
        true
    }

    /// The object update of 3.16 with an optional listener (position, forward).
    pub fn update(&mut self, table: &EffectTable, handle: usize, listener: Option<(V3, V3)>, r: &mut impl Uniform) {
        let Some(mut o) = self.objects.get_mut(handle).and_then(|o| o.take()) else { return };
        if let Some(d) = o.def.map(|d| &table.definitions[d]) {
            if !d.nodes.is_empty() {
                let mut culled = false;
                let mut state = 0;
                if let Some((at, forward)) = listener {
                    let v = [o.position[0] - at[0], o.position[1] - at[1], o.position[2] - at[2]];
                    let d2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                    let u = unit(v);
                    let dot = forward[0] * u[0] + forward[1] * u[1] + forward[2] * u[2];
                    let cull = o.cull.unwrap_or(d.cull);
                    culled = (cull > 0.0 && d2 > cull) || (d.cone && dot < 0.5 && d2 > CONE_LIMIT);
                    state = u8::from(d2 > d.threshold);
                }
                if culled {
                    self.stop_object(table, &mut o);
                } else if !o.live() {
                    o.state = state;
                    self.start_emitters(table, &mut o, Some(handle), r);
                } else if state != o.state {
                    self.stop_object(table, &mut o);
                    o.state = state;
                    self.start_emitters(table, &mut o, Some(handle), r);
                }
            }
        }
        self.objects[handle] = Some(o);
    }

    /// Stop an object: drain every live spawner and clear the slots (3.16 Stop).
    pub fn stop(&mut self, table: &EffectTable, handle: usize) {
        if let Some(mut o) = self.objects.get_mut(handle).and_then(|o| o.take()) {
            self.stop_object(table, &mut o);
            self.objects[handle] = Some(o);
        }
    }

    /// Release an object (its spawners run on detached).
    pub fn release(&mut self, handle: usize) {
        if let Some(Some(o)) = self.objects.get_mut(handle).map(|o| o.take()) {
            for s in o.slots.iter().flatten() {
                if let Some(sp) = self.spawners[*s].as_mut() {
                    sp.owner = None;
                }
            }
        }
    }

    fn stop_object(&mut self, table: &EffectTable, o: &mut EffectObject) {
        for s in o.slots.iter_mut() {
            if let Some(i) = s.take() {
                self.drain(table, i);
            }
        }
    }

    /// Set an object's spawners' target point (the attack-effect driver's write, 3.17; read by
    /// birth modes 2 and 3).
    pub fn set_target(&mut self, handle: usize, t: V3) {
        let Some(Some(o)) = self.objects.get(handle) else { return };
        for s in o.slots.iter().flatten() {
            if let Some(sp) = self.spawners[*s].as_mut() {
                sp.target = t;
            }
        }
    }

    /// Set an object's position: copied to every live spawner (3.16).
    pub fn set_position(&mut self, handle: usize, p: V3) {
        let Some(Some(o)) = self.objects.get_mut(handle) else { return };
        o.position = p;
        for s in o.slots.iter().flatten() {
            if let Some(sp) = self.spawners[*s].as_mut() {
                sp.position = p;
            }
        }
    }

    /// 3.4: try the emitter slots below n whose A gate equals the state; stop after the first
    /// start when the multi-emitter flag is 0. A spawner is kept in a slot only for a base period
    /// of zero or more.
    fn start_emitters(&mut self, table: &EffectTable, o: &mut EffectObject, handle: Option<usize>, r: &mut impl Uniform) {
        let Some(d) = o.def.map(|d| &table.definitions[d]) else { return };
        for (k, &n) in d.nodes.iter().enumerate() {
            let node = &table.nodes[n];
            let a = &table.a[node.a];
            if a.gate != o.state {
                continue;
            }
            let Some(slot) = self.spawners.iter().position(|s| s.is_none()) else { continue };
            let position = v_add(o.position, apply(&o.frame, [node.transform[3][0], node.transform[3][1], node.transform[3][2]]));
            let frame: Frame = std::array::from_fn(|row| apply(&o.frame, [node.transform[row][0], node.transform[row][1], node.transform[row][2]]));
            let period = if a.period >= 0.0 { add(a.period, r.uniform(-a.period_jitter, a.period_jitter)) } else { -1.0 };
            let keep = a.period >= 0.0;
            self.spawners[slot] = Some(Spawner {
                node: n,
                state: SpawnerState::Running,
                elapsed: 0.0,
                phase: 0.0,
                period,
                cycles: 0,
                bursts: 0,
                live: 0,
                position,
                frame,
                previous: position,
                target: position,
                delta: [0.0; 3],
                velocity: [0.0; 3],
                scale: o.scale.unwrap_or(d.scale),
                last_born: None,
                owner: if keep { handle } else { None },
            });
            if keep {
                o.slots[k] = Some(slot);
            }
            if d.multi == 0 {
                break;
            }
        }
    }

    /// Drain (3.2): dead at once with no live particle, else state 5 with the delta zeroed; a
    /// looping emitter's last-born particle ends on the next tick.
    fn drain(&mut self, table: &EffectTable, i: usize) {
        let Some(s) = self.spawners[i].as_mut() else { return };
        let looped = table.a[table.nodes[s.node].a].looped;
        if s.live == 0 {
            self.spawners[i] = None;
            return;
        }
        s.state = SpawnerState::Draining;
        s.delta = [0.0; 3];
        s.owner = None;
        let last = s.last_born.filter(|_| looped);
        if let Some(p) = last.and_then(|p| self.particles[p].as_mut()) {
            p.elapsed = p.life;
        }
    }

    /// One effect tick: every spawner steps (3.2), then every live particle runs its motion and
    /// curves (4.6).
    pub fn tick(&mut self, table: &EffectTable, r: &mut impl Uniform) {
        for i in 0..self.spawners.len() {
            self.step_spawner(table, i, r);
        }
        for i in 0..self.particles.len() {
            self.step_particle(table, i, r);
        }
    }

    fn step_spawner(&mut self, table: &EffectTable, i: usize, r: &mut impl Uniform) {
        let Some(s) = self.spawners[i].as_mut() else { return };
        match s.state {
            SpawnerState::Dead => {
                self.spawners[i] = None;
                return;
            }
            SpawnerState::Draining => {
                if s.live == 0 {
                    self.end_spawner(i);
                }
                return;
            }
            SpawnerState::Running => {}
        }
        let a = &table.a[table.nodes[s.node].a];
        let mut burst = false;
        let mut drain = false;
        if s.period < 0.0 {
            burst = true;
            drain = !a.looped;
        } else {
            let first = s.elapsed == 0.0;
            s.elapsed = add(s.elapsed, DT);
            s.phase = add(s.phase, div(DT, s.period));
            if s.phase >= 1.0 || first {
                let next = add(a.period, r.uniform(-a.period_jitter, a.period_jitter)).max(0.0);
                let mut phase = if next > 0.0 { div(mul(s.phase, s.period), next) } else { 1.0 };
                if phase < 1.0 {
                    phase = 1.0;
                }
                s.cycles += phase.trunc() as i32;
                s.phase = phase.fract();
                s.period = next;
                burst = !a.looped || s.live == 0;
            }
            if !(s.cycles < a.cycles || a.end_gate < 0) && !a.looped {
                drain = true;
            }
        }
        let s = self.spawners[i].as_mut().unwrap();
        s.delta = [s.position[0] - s.previous[0], s.position[1] - s.previous[1], s.position[2] - s.previous[2]];
        s.previous = s.position;
        if burst {
            s.bursts += 1;
            for _ in 0..a.per_burst.max(0) {
                if !self.birth(table, i, r) {
                    break;
                }
            }
            if a.birth_mode == 3 {
                let s = self.spawners[i].as_mut().unwrap();
                s.target = s.position;
            }
        }
        if drain {
            self.drain(table, i);
        }
    }

    fn end_spawner(&mut self, i: usize) {
        if let Some(owner) = self.spawners[i].as_ref().and_then(|s| s.owner) {
            if let Some(Some(o)) = self.objects.get_mut(owner) {
                for slot in o.slots.iter_mut() {
                    if *slot == Some(i) {
                        *slot = None;
                    }
                }
            }
        }
        self.spawners[i] = None;
    }

    /// Birth (4.1). Returns false when the particle pool is empty.
    fn birth(&mut self, table: &EffectTable, si: usize, r: &mut impl Uniform) -> bool {
        let Some(pi) = self.particles.iter().position(|p| p.is_none()) else { return false };
        let s = self.spawners[si].as_ref().unwrap();
        let node: &Node = &table.nodes[s.node];
        let (a, b) = (&table.a[node.a], &table.b[node.b]);
        let c = &table.c[node.c];
        let mut p = s.position;
        if node.ring > 0.0 {
            let alpha = r.uniform(0.0, std::f32::consts::TAU);
            let radius = if node.ring_fill != 0 { node.ring } else { r.uniform(0.0, node.ring) };
            p[0] = add(p[0], mul(radius, series_cos(alpha)));
            p[2] = add(p[2], mul(radius, series_sin(alpha)));
        }
        // Launch modes 2 and 3: P is rebuilt from S alone, one draw along the segment to T.
        if matches!(a.birth_mode, 2 | 3) {
            p = along_segment(s.position, s.target, r);
        }
        let life = add(a.life, r.uniform(-a.life_jitter, a.life_jitter));
        let clock = add(c.clock.0, r.uniform(-c.clock.1, c.clock.1));
        let kick_interval = if c.kick_interval.0 >= 0.0 { add(c.kick_interval.0, r.uniform(-c.kick_interval.1, c.kick_interval.1)) } else { -1.0 };
        // Direction: the frame's row 1 (direction weight 0), a polar angle in the cone and an
        // azimuth; the two perpendiculars take a random sign each.
        let axis = unit(s.frame[1]);
        let theta = r.uniform(c.cone_lower, c.cone_upper);
        let phi = r.uniform(-std::f32::consts::PI, std::f32::consts::PI);
        let pick = if axis[2].abs() < 0.99 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
        let mut e2 = unit(cross(axis, pick));
        let mut e1 = unit(cross(e2, axis));
        if r.uniform(0.0, 1.0) < 0.5 {
            e1 = e1.map(|x| -x);
        }
        if r.uniform(0.0, 1.0) < 0.5 {
            e2 = e2.map(|x| -x);
        }
        let (st, ct) = (series_sin(theta), series_cos(theta));
        let (sp, cp) = (series_sin(phi), series_cos(phi).abs());
        let side = v_add(v_scale(e1, sp), v_scale(e2, cp));
        let d = unit(v_add(v_scale(axis, ct), v_scale(side, st)));
        let speed = add(c.speed.0, r.uniform(-c.speed.1, c.speed.1));
        let mut velocity = v_scale(d, speed);
        if !a.follow {
            velocity = v_add(velocity, s.velocity);
        }
        let spin_rate = add(c.spin.0, r.uniform(-c.spin.1, c.spin.1));
        let accel = if clock != 0.0 {
            let t = clock;
            let up = mul(mul(c.accel, t), 1.0);
            [div(c.extra[0], t), div(add(up, c.extra[1]), t), div(c.extra[2], t)].map(|x| mul(x, DT))
        } else {
            [0.0; 3]
        };
        let size = a.size;
        let half = std::array::from_fn(|k| (node.factors[k] * size[k] * s.scale).max(0.0));
        let scale = s.scale;
        self.particles[pi] = Some(Particle {
            spawner: si,
            node: s.node,
            position: p,
            velocity,
            accel,
            drag: mul(c.drag, DT),
            life,
            elapsed: 0.0,
            colour: b.rows[0],
            size,
            rgb_key: 0,
            alpha_key: 0,
            kick_interval,
            spin_rate,
            spin: 0.0,
            half,
            scale,
            frame: s.frame,
            axis: unit([r.uniform(-1.0, 1.0), r.uniform(-1.0, 1.0), r.uniform(-1.0, 1.0)]),
        });
        let s = self.spawners[si].as_mut().unwrap();
        s.live += 1;
        s.last_born = Some(pi);
        true
    }

    /// Motion (4.2), then age, expiry and the curves (4.6 steps 4 to 6).
    fn step_particle(&mut self, table: &EffectTable, i: usize, r: &mut impl Uniform) {
        let Some(mut p) = self.particles[i].take() else { return };
        let node = &table.nodes[p.node];
        let (a, b, c) = (&table.a[node.a], &table.b[node.b], &table.c[node.c]);
        let spawner = self.spawners[p.spawner].as_ref();
        let (spawner_at, delta, draining) = spawner.map_or(([0.0; 3], [0.0; 3], true), |s| (s.position, s.delta, s.state != SpawnerState::Running));
        // 1. v = (v + a) x (1 - k).
        p.velocity = v_add(p.velocity, p.accel);
        p.velocity = std::array::from_fn(|k| sub(p.velocity[k], mul(p.velocity[k], p.drag)));
        // 2. The kick.
        let e = p.elapsed;
        let k = p.kick_interval;
        if k == 0.0 || (k > 0.0 && (sub(e, DT) / k).trunc() < (e / k).trunc()) {
            p.kick_interval = if c.kick_interval.0 >= 0.0 { add(c.kick_interval.0, r.uniform(-c.kick_interval.1, c.kick_interval.1)) } else { -1.0 };
            if c.kick_zero {
                p.velocity = [0.0; 3];
            }
            let strength = add(c.kick.0, r.uniform(-c.kick.1, c.kick.1));
            let kick = [r.uniform(-strength, strength), strength, r.uniform(-strength, strength)];
            p.velocity = v_add(p.velocity, kick);
        }
        // 3. Spin and move.
        if p.spin_rate != 0.0 {
            p.spin = add(p.spin, mul(p.spin_rate, DT));
        }
        // A velocity-following model particle (C +0x3C nonzero): row 1 takes a 10 percent step
        // toward the velocity; rows 0 and 2 are rebuilt from it.
        if node.material.is_none() && c.orient != 0 {
            let speed = (p.velocity[0].powi(2) + p.velocity[1].powi(2) + p.velocity[2].powi(2)).sqrt();
            if speed != 0.0 {
                let d = v_scale(p.velocity, 1.0 / speed);
                let row1 = unit(v_add(v_scale(p.frame[1], 0.9), v_scale(d, 0.1)));
                let up = if d[1].abs() > 0.99 { [0.0, 0.0, 1.0] } else { [0.0, 1.0, 0.0] };
                let row0 = unit(cross(row1, up));
                p.frame = [row0, row1, unit(cross(row0, row1))];
            }
        }
        p.position = v_add(p.position, v_scale(p.velocity, mul(DT, p.scale)));
        if a.follow {
            p.position = v_add(p.position, delta);
        }
        // 4. Age and end.
        p.elapsed = add(p.elapsed, DT);
        let f = div(p.elapsed, p.life);
        if p.elapsed > p.life {
            if a.looped && !draining {
                p.position = spawner_at;
                p.elapsed = 0.0;
                p.colour = b.rows[0];
                p.size = a.size;
                p.rgb_key = 0;
                p.alpha_key = 0;
            } else {
                if let Some(s) = self.spawners[p.spawner].as_mut() {
                    s.live = s.live.saturating_sub(1);
                    if s.last_born == Some(i) {
                        s.last_born = None;
                    }
                }
                return;
            }
        }
        // 5. Colour.
        if p.rgb_key < 5 && f > b.rgb_keys[p.rgb_key] {
            p.rgb_key += 1;
        }
        if p.alpha_key < 5 && f > b.alpha_keys[p.alpha_key] {
            p.alpha_key += 1;
        }
        let q = div(DT, p.life);
        let row = b.rows[p.rgb_key];
        let rate = [row[0], row[1], row[2], b.rows[p.alpha_key][3]];
        for k in 0..4 {
            p.colour[k] = add(p.colour[k], mul(rate[k], q)).clamp(0.0, 2.0);
        }
        // 6. Size.
        let rows = &a.size_rates;
        let rate: [f32; 4] = std::array::from_fn(|k| {
            if f <= a.thresholds[0][k] {
                rows[0][k]
            } else if f <= a.thresholds[1][k] {
                rows[1][k]
            } else {
                rows[2][k]
            }
        });
        if rate.iter().any(|&x| x != 0.0) {
            for k in 0..4 {
                p.size[k] = add(p.size[k], mul(rate[k], q));
            }
            p.half = std::array::from_fn(|k| mul(mul(node.factors[k], p.size[k]), p.scale).max(0.0));
        }
        self.particles[i] = Some(p);
    }

    /// The drawable sprites: live particles of a node with a material, not of a light emitter
    /// (4.7): (particle, material group).
    pub fn sprites<'a>(&'a self, table: &'a EffectTable) -> impl Iterator<Item = (&'a Particle, usize)> + 'a {
        self.particles.iter().flatten().filter_map(move |p| {
            let node = &table.nodes[p.node];
            (!table.a[node.a].light).then_some(())?;
            Some((p, node.material?))
        })
    }

    /// The drawable model particles (4.7): live particles of a node with no material, not of a
    /// light emitter, whose child is a resource-9 model: (model index, position, the draw frame's
    /// rows, colour channel 1). The child rule of 2.3: a byte +0x67 below 0x80 names common object
    /// byte + 283 when it is 0x27 or less; else +0x60 from 0 to 49.
    pub fn models<'a>(&'a self, table: &'a EffectTable) -> impl Iterator<Item = (u16, V3, [V3; 3], [f32; 4])> + 'a {
        self.particles.iter().flatten().filter_map(move |p| {
            let node = &table.nodes[p.node];
            if table.a[node.a].light || node.material.is_some() {
                return None;
            }
            let model = match node.child_gate {
                g @ 0..=0x27 => g as u16,
                0x28..=0x7F => return None,
                _ => u16::try_from(node.child).ok().filter(|&c| c <= 49)?,
            };
            let c = &table.c[node.c];
            Some((model, p.position, p.draw_frame(c.orient, c.spin_axis), p.colour))
        })
    }

    /// The pass's dynamic point lights (4.5; SPEC-0046 Character lights): each live particle of a
    /// light emitter whose node has a material writes its spawner's light entry (type 3 at the
    /// particle position, colour channel 1 x 255 with alpha 0, strength channel 1's alpha, radius
    /// channel 2's x and its square; flag and zone 0) and lists it, so a spawner with k live
    /// particles is listed k times with its last particle's values (original quirk). At most
    /// 200 entries.
    pub fn lights(&self, table: &EffectTable) -> Vec<crate::charlight::Entry> {
        let mut last: std::collections::HashMap<usize, crate::charlight::Entry> = std::collections::HashMap::new();
        let mut order = Vec::new();
        for p in self.particles.iter().flatten() {
            let node = &table.nodes[p.node];
            if !table.a[node.a].light || node.material.is_none() {
                continue;
            }
            let r = p.size[0];
            last.insert(
                p.spawner,
                crate::charlight::Entry {
                    position: p.position,
                    direction: [0.0; 3],
                    colour: [p.colour[0] * 255.0, p.colour[1] * 255.0, p.colour[2] * 255.0, 0.0],
                    kind: 3,
                    strength: p.colour[3],
                    flag: 0,
                    radius: r,
                    radius_sq: r * r,
                    zone: 0,
                },
            );
            order.push(p.spawner);
        }
        order.into_iter().take(200).map(|s| last[&s]).collect()
    }

    pub fn live_particles(&self) -> usize {
        self.particles.iter().flatten().count()
    }

    pub fn live_spawners(&self) -> usize {
        self.spawners.iter().flatten().count()
    }
}

/// 4.1 step 2, modes 2 and 3: S + u t with u = (T - S) / L and one draw t = U(0, L); with L = 0
/// the point is S (the draw still runs).
fn along_segment(s: V3, t: V3, r: &mut impl Uniform) -> V3 {
    let d = [t[0] - s[0], t[1] - s[1], t[2] - s[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let k = r.uniform(0.0, l);
    if l == 0.0 {
        return s;
    }
    std::array::from_fn(|i| s[i] + d[i] / l * k)
}

#[cfg(test)]
mod tests {
    #[test]
    fn model_particle_draw_frame() {
        let mut p = Particle { spawner: 0, node: 0, position: [0.0; 3], velocity: [0.0; 3], accel: [0.0; 3], drag: 0.0, life: 1.0, elapsed: 0.0, colour: [1.0; 4], size: [2.0, 3.0, 4.0, 1.0], rgb_key: 0, alpha_key: 0, kick_interval: -1.0, spin_rate: 0.0, spin: 0.0, half: [0.0; 3], scale: 0.5, frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], axis: [0.0, 1.0, 0.0] };
        // No spin: the stored frame scaled by size x S.
        assert_eq!(p.draw_frame(0, 2), [[1.0, 0.0, 0.0], [0.0, 1.5, 0.0], [0.0, 0.0, 2.0]]);
        // A quarter turn about world up: row 0 goes to -Z, row 2 to +X; row 1 stays.
        p.spin = std::f32::consts::FRAC_PI_2;
        let f = p.draw_frame(0, 2);
        assert!((f[0][2] + 1.0).abs() < 1e-5 && f[0][0].abs() < 1e-5);
        assert!((f[2][0] - 2.0).abs() < 1e-5);
        assert!((f[1][1] - 1.5).abs() < 1e-5);
        // Orientation mode nonzero: the spin angle does not apply.
        assert_eq!(p.draw_frame(1, 2)[0], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn birth_along_the_target_segment() {
        // SPEC-0034 4.1 test vector: S (0, 1, 0), T (3, 1, 4): t = 2 gives (1.2, 1, 1.6).
        struct Fixed(f32);
        impl Uniform for Fixed {
            fn uniform(&mut self, _lo: f32, _hi: f32) -> f32 {
                self.0
            }
        }
        let p = along_segment([0.0, 1.0, 0.0], [3.0, 1.0, 4.0], &mut Fixed(2.0));
        assert!((p[0] - 1.2).abs() < 1e-5 && (p[1] - 1.0).abs() < 1e-5 && (p[2] - 1.6).abs() < 1e-5, "{p:?}");
        assert_eq!(along_segment([1.0; 3], [1.0; 3], &mut Fixed(0.0)), [1.0; 3]);
    }


    use super::*;

    fn record_a(period: f32, per_burst: i32, life: f32, looped: bool) -> RecordA {
        RecordA {
            size: [1.0, 1.0, 1.0, 1.0],
            size_rates: [[0.0; 4]; 3],
            thresholds: [[2.0; 4]; 2],
            end_gate: -1,
            per_burst,
            period,
            period_jitter: 0.0,
            cycles: 0,
            life,
            life_jitter: 0.0,
            birth_mode: 0,
            gate: 0,
            follow: false,
            looped,
            light: false,
        }
    }

    fn table(a: RecordA, row0: [f32; 4]) -> EffectTable {
        let mut rows = [[0.0; 4]; 6];
        rows[0] = row0;
        EffectTable {
            definitions: vec![Definition { id: 500, active: 1, multi: 1, threshold: 10000.0, nodes: vec![0], scale: 1.0, cull: -1.0, cone: true }],
            nodes: vec![Node {
                transform: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
                link: -1,
                c: 0,
                a: 0,
                b: 0,
                material: Some(18),
                child: -1,
                preset: 0,
                ring_fill: 0,
                child_gate: 0xFF,
                ring: 0.0,
                factors: [0.5; 3],
            }],
            a: vec![a],
            b: vec![RecordB { rows, rgb_keys: [1.0; 6], alpha_keys: [1.0; 6] }],
            c: vec![RecordC {
                extra: [0.0; 4],
                accel: 0.0,
                drag: 0.0,
                cone_upper: 0.0,
                cone_lower: 0.0,
                speed: (0.0, 0.0),
                clock: (1.0, 0.0),
                spin: (0.0, 0.0),
                kick: (0.0, 0.0),
                kick_interval: (-1.0, 0.0),
                orient: 0,
                kick_zero: false,
                spin_axis: 0,
                flag: 1,
            }],
            children: 0,
        }
    }

    /// Edge cases: a 0.1 s period bursts on ticks 1, 8, 14, 20 in single precision.
    #[test]
    fn period_clock_in_single_precision() {
        let t = table(record_a(0.1, 1, 0.05, false), [1.0; 4]);
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&t, 500, [0.0; 3], IDENTITY));
        fx.update(&t, h, None, &mut Midpoint);
        let mut bursts = Vec::new();
        let count = |fx: &Effects| fx.spawners.iter().flatten().next().unwrap().bursts;
        for tick in 1..=20 {
            let before = count(&fx);
            fx.tick(&t, &mut Midpoint);
            if count(&fx) > before {
                bursts.push(tick);
            }
        }
        assert_eq!(bursts, vec![1, 8, 14, 20]);
    }

    /// Test vector 8: life 0.5 s, row 0 x = 0.8; with truncating adds the accumulator after 15
    /// ticks is just under 0.4. Test vector 9: a rate of 3.0 clamps at 2.0 on tick 21.
    #[test]
    fn colour_curve_and_clamp() {
        let t = table(record_a(-1.0, 1, 0.5, false), [3.0, 0.0, 0.0, 0.0]);
        let mut fx = Effects::new();
        assert!(fx.one_shot(&t, 500, [0.0; 3], IDENTITY, &mut Midpoint));
        let x = |fx: &Effects| fx.particles.iter().flatten().next().unwrap().colour[0];
        // The start value is row 0 (3.0, clamped on its first update).
        fx.tick(&t, &mut Midpoint);
        assert_eq!(x(&fx), 2.0);
        // A fire-and-forget spawner drains with its first burst and dies after its particle.
        assert_eq!(fx.spawners.iter().flatten().next().unwrap().state, SpawnerState::Draining);
        for _ in 0..40 {
            fx.tick(&t, &mut Midpoint);
        }
        assert_eq!((fx.live_particles(), fx.live_spawners()), (0, 0));
    }

    #[test]
    fn light_emitters_list_their_spawner_per_particle() {
        let mut a = record_a(0.1, 2, 1.0, false);
        a.light = true;
        a.size = [3.0, 1.0, 1.0, 1.0];
        let t = table(a, [0.0; 4]);
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&t, 500, [1.0, 2.0, 3.0], IDENTITY));
        fx.update(&t, h, None, &mut Midpoint);
        fx.tick(&t, &mut Midpoint);
        let lights = fx.lights(&t);
        // Two live particles of one spawner: the spawner's entry is listed twice.
        assert_eq!(fx.live_particles(), 2);
        assert_eq!(lights.len(), 2);
        assert_eq!(lights[0], lights[1]);
        let e = lights[0];
        assert_eq!((e.kind, e.radius, e.radius_sq, e.colour[3]), (3, 3.0, 9.0, 0.0));
        // Light-emitter particles are never sprites.
        assert_eq!(fx.sprites(&t).count(), 0);
    }

    #[test]
    fn curve_accumulates_rate_over_life() {
        // Row 0 is both the start value and the rate for key index 0.
        let t = table(record_a(-1.0, 1, 0.5, false), [0.8, 0.0, 0.0, 0.0]);
        let mut fx = Effects::new();
        fx.one_shot(&t, 500, [0.0; 3], IDENTITY, &mut Midpoint);
        // Start 0.8 plus 15 x 0.8/30.
        for _ in 0..15 {
            fx.tick(&t, &mut Midpoint);
        }
        let p = fx.particles.iter().flatten().next().unwrap();
        assert!((p.colour[0] - 1.2).abs() < 1e-5);
        assert!(p.elapsed < 0.25);
    }

    /// 3.16: a looping emitter of base period 0 keeps one particle that restarts at its life
    /// end, and no new birth comes while it lives.
    #[test]
    fn loop_restart_keeps_one_particle() {
        let t = table(record_a(0.5, 1, 1.0, true), [0.5, 0.5, 0.5, 0.5]);
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&t, 500, [0.0; 3], IDENTITY));
        fx.update(&t, h, None, &mut Midpoint);
        for _ in 0..200 {
            fx.tick(&t, &mut Midpoint);
            assert!(fx.live_particles() <= 1);
        }
        assert_eq!(fx.live_particles(), 1);
        // Stop drains: the looping particle ends at once on the next tick.
        fx.stop(&t, h);
        fx.tick(&t, &mut Midpoint);
        fx.tick(&t, &mut Midpoint);
        assert_eq!((fx.live_particles(), fx.live_spawners()), (0, 0));
    }

    /// 5.2 and 3.16: the cone and the near/far state.
    #[test]
    fn cone_cull_and_state() {
        let t = table(record_a(0.5, 1, 1.0, false), [1.0; 4]);
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&t, 500, [0.0; 3], IDENTITY));
        // Behind the listener at d^2 = 400: culled, nothing starts.
        fx.update(&t, h, Some(([0.0, 0.0, 20.0], [0.0, 0.0, 1.0])), &mut Midpoint);
        assert_eq!(fx.live_spawners(), 0);
        // In front at d^2 = 10100: far (state 1), and the gate-0 emitter does not start.
        fx.update(&t, h, Some(([0.0, 0.0, -100.5], [0.0, 0.0, 1.0])), &mut Midpoint);
        assert_eq!((fx.objects[h].as_ref().unwrap().state, fx.live_spawners()), (1, 0));
        fx.update(&t, h, Some(([0.0, 0.0, -10.0], [0.0, 0.0, 1.0])), &mut Midpoint);
        assert_eq!((fx.objects[h].as_ref().unwrap().state, fx.live_spawners()), (0, 1));
    }

    #[test]
    fn series_matches_sine_near_zero() {
        assert!((series_sin(0.5) - 0.479426).abs() < 1e-5);
        assert!((series_cos(0.5) - 0.877583).abs() < 1e-5);
        assert!(series_sin(0.0).abs() < 1e-6 && (series_cos(0.0) - 1.0).abs() < 1e-6);
        // Every angle, the phase 3 pi / 2 included: the true sine to about 5e-8 (plus float
        // rounding of the angle).
        for k in -400..=400 {
            let t = k as f32 * 0.0471;
            assert!((series_sin(t) - t.sin()).abs() < 2e-6, "t {t}");
        }
        assert!((series_sin(3.0 * std::f32::consts::FRAC_PI_2) + 1.0).abs() < 1e-5);
    }

    fn put(b: &mut [u8], at: usize, v: &[u8]) {
        b[at..at + v.len()].copy_from_slice(v);
    }

    /// One child, two nodes, two definitions, one A, one B and one C record.
    #[test]
    fn parses_synthetic_table() {
        let (a_at, b_at, c_at) = (0x20, 0x20 + A_SIZE, 0x20 + A_SIZE + B_SIZE);
        let child_at = c_at + C_SIZE;
        let def_table = align16(child_at + 4);
        let node_at = align16(def_table + 8);
        let defs_at = node_at + 2 * NODE_SIZE;
        let mut p = vec![0u8; defs_at + 2 * DEF_SIZE];
        for (k, v) in [1i32, 2, 2, 1, 1, 1].iter().enumerate() {
            put(&mut p, 4 * k, &v.to_le_bytes());
        }
        put(&mut p, a_at + 0x64, &3i32.to_le_bytes());
        put(&mut p, a_at + 0x6C, &(-1.0f32).to_le_bytes());
        put(&mut p, a_at + 0x78, &2.0f32.to_le_bytes());
        p[a_at + 0x86] = 1;
        put(&mut p, b_at + 0x0C, &1.0f32.to_le_bytes());
        put(&mut p, b_at + 0x60, &0.5f32.to_le_bytes());
        // C: acceleration -2.5 (0xFD80) and speed 8.0.
        put(&mut p, c_at + 0x20, &(-640i16).to_le_bytes());
        put(&mut p, c_at + 0x28, &2048i16.to_le_bytes());
        for n in 0..2 {
            let at = node_at + n * NODE_SIZE;
            put(&mut p, at + 0x4C, &(-1i32).to_le_bytes());
            put(&mut p, at + 0x5C, &(if n == 0 { 18i32 } else { -1 }).to_le_bytes());
            // SPEC-0034 4.1 step 7 example: 144, 289 and 0 at +0x6A, +0x6C and +0x6E.
            put(&mut p, at + 0x6A, &144i16.to_le_bytes());
            put(&mut p, at + 0x6C, &289i16.to_le_bytes());
        }
        for d in 0..2 {
            let at = defs_at + d * DEF_SIZE;
            put(&mut p, def_table + 4 * d, &((at - (def_table + 4 * d)) as i32).to_le_bytes());
            put(&mut p, at + 0x40, &(if d == 0 { 137i32 } else { 101 }).to_le_bytes());
            put(&mut p, at + 0x54, &2i32.to_le_bytes());
            put(&mut p, at + 0x6C, &0i32.to_le_bytes());
            put(&mut p, at + 0x70, &1i32.to_le_bytes());
            put(&mut p, at + 0x80, &1.0f32.to_le_bytes());
        }
        let t = EffectTable::parse(&p).unwrap();
        assert_eq!((t.definitions.len(), t.nodes.len(), t.children), (2, 2, 1));
        assert_eq!(t.find(101), Some(1));
        assert_eq!(t.find(162), None);
        assert_eq!(t.definitions[0].nodes, vec![0, 1]);
        assert_eq!((t.nodes[0].material, t.nodes[1].material), (Some(18), None));
        assert_eq!(t.nodes[0].factors, [289.0 / 256.0, 0.0, 144.0 / 256.0]);
        let a = &t.a[0];
        assert_eq!((a.per_burst, a.period, a.life, a.looped), (3, -1.0, 2.0, true));
        assert_eq!((t.b[0].rows[0][3], t.b[0].rgb_keys[0]), (1.0, 0.5));
        assert_eq!((t.c[0].accel, t.c[0].speed.0), (-2.5, 8.0));
    }
}
