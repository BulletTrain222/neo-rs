//! Foliage patches (SPEC-0060 Behavior 1 to 3): the family and models a key selects, and the
//! placement of each plant (jitter, ground snap, yaw, scale) with the patch distances.
//!
//! The bush state machine (Behavior 6: cut, shrink, regrow, sparkle wait, the up axis's
//! spring-back and hero bend) and the drop rules (Behavior 7).
//!
//! The near/far distances and weights of section 3 (`Distances`). Not covered: the zone gate,
//! grass sway and fade, the debris pieces, tree trunks and the
//! once-per-area build gates.
//!
//! Reeds (Behavior 9): the bend away from the hero within 5 units, the cut, the three-state
//! fall of the cut piece and the drop roll.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Grass,
    Flowers,
    Bush,
    Tree,
    Reed,
    Bed,
}

/// What a key builds: the family, its near (main) model and far model registry ids, whether
/// the plants snap to the ground, and the lower bound of the random scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kind {
    pub family: Family,
    pub near: Option<u16>,
    pub far: Option<u16>,
    pub snap: bool,
    pub s0: f32,
}

/// The family table of Behavior 1 (keys 15 and 40 and up build nothing).
pub fn kind(key: u32) -> Option<Kind> {
    use Family::*;
    let k = |family, near, far, s0| Some(Kind { family, near, far, snap: true, s0 });
    match key {
        0 => k(Grass, None, Some(283), 1.0),
        12 => Some(Kind { snap: false, ..kind(0)? }),
        1 | 7 => k(Flowers, Some(194), None, 0.6),
        6 => k(Flowers, Some(193), None, 0.6),
        3 | 9 => k(Bush, Some(196), None, 0.6),
        2 | 29 => k(Bush, Some(195), None, 0.6),
        5 => k(Bush, Some(198), None, 0.6),
        16 => k(Bush, Some(222), None, 0.6),
        17 => k(Bush, Some(223), None, 0.6),
        18 => k(Bush, Some(224), None, 0.6),
        19 => k(Bush, Some(226), None, 0.6),
        20 => k(Bush, Some(227), None, 0.6),
        21 => k(Bush, Some(229), None, 0.6),
        22 => k(Bush, Some(231), None, 0.6),
        28 => k(Bush, Some(248), None, 0.6),
        30 => k(Bush, Some(253), None, 0.6),
        31 => k(Bush, Some(256), None, 0.6),
        32 => k(Bush, Some(259), None, 0.6),
        33 => k(Bush, Some(262), None, 0.6),
        34 => k(Bush, Some(265), None, 1.0),
        36 => k(Bush, Some(271), None, 0.6),
        37 => k(Bush, Some(274), None, 0.6),
        38 => k(Bush, Some(277), None, 0.6),
        39 => k(Bush, Some(280), None, 0.6),
        4 => k(Tree, Some(200), None, 0.85),
        25 => k(Tree, Some(200), Some(199), 0.85),
        13 => k(Tree, None, Some(220), 0.85),
        8 => k(Reed, Some(202), Some(203), 0.73),
        10 => k(Bed, Some(197), None, 0.85),
        11 => k(Bed, Some(204), None, 0.85),
        14 => k(Bed, Some(221), None, 0.85),
        23 => k(Bed, Some(233), None, 0.85),
        24 => k(Bed, Some(234), None, 0.85),
        26 => k(Bed, Some(235), None, 0.85),
        27 => k(Bed, Some(239), None, 0.85),
        35 => k(Bed, Some(268), None, 0.85),
        _ => None,
    }
}

/// A patch's draw distances (SPEC-0060 section 3): far F, switch S, out band O, fade band W.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distances {
    pub far: f32,
    pub switch: f32,
    pub out: f32,
    pub fade: f32,
}

impl Distances {
    /// With the overrides (A or D nonzero): F = D, S = B + A, O = B, W = C. Else from the near
    /// model's distance-cull node (`lod` = +0x50, +0x54, +0x58, +0x64): F the fade end, S the
    /// +0x54 float, W the fade length, O = S - the +0x50 float; without a far object O and W
    /// are both raised to max(O, W) and S lowered to F - O when larger. With no model F = 4000.
    pub fn of(overrides: [f32; 4], lod: Option<[f32; 4]>, has_far: bool) -> Self {
        let [a, b, c, d] = overrides;
        if a != 0.0 || d != 0.0 {
            return Self { far: d, switch: b + a, out: b, fade: c };
        }
        let Some([p50, p54, fade, end]) = lod else { return Self { far: 4000.0, switch: 4000.0, out: 0.0, fade: 0.0 } };
        let mut x = Self { far: end, switch: p54, out: p54 - p50, fade };
        if !has_far {
            let m = x.out.max(x.fade);
            x.out = m;
            x.fade = m;
            if x.far - m < x.switch {
                x.switch = x.far - m;
            }
        }
        x
    }

    /// The near and far weights (a, b) of a plant at distance d (SPEC-0060 section 3).
    pub fn weights(&self, d: f32, near: bool, far: bool) -> (f32, f32) {
        match (near, far) {
            (true, false) => {
                let start = self.far - self.fade;
                let a = if d <= start { 1.0 } else if self.fade > 0.0 { (1.0 - (d - start) / self.fade).max(0.0) } else { 0.0 };
                (a, 0.0)
            }
            (false, true) => (0.0, 1.0),
            (true, true) => {
                let (s, o, w) = (self.switch, self.out, self.fade);
                if d < s - o {
                    (1.0, 0.0)
                } else if d < s {
                    (1.0, if o > 0.0 { (d - (s - o)) / o } else { 1.0 })
                } else if d <= s + w {
                    (if w > 0.0 { (s + w - d) / w } else { 0.0 }, 1.0)
                } else {
                    (0.0, 1.0)
                }
            }
            (false, false) => (1.0, 1.0),
        }
    }
}

/// A bush's cut (second) model by main model (Behavior 1 bush table).
pub fn cut_model(main: u16) -> Option<u16> {
    Some(match main {
        196 => 192,
        195 => 244,
        198 => 201,
        222 => 251,
        223 => 246,
        226 => 242,
        227 => 228,
        229 => 230,
        231 => 232,
        248 => 249,
        253 => 254,
        256 => 257,
        259 => 260,
        262 => 263,
        265 => 266,
        271 => 272,
        274 => 275,
        277 => 278,
        280 => 281,
        _ => return None,
    })
}

/// A bush (Behavior 6): state 2 grown, 0 cut, 1 regrowing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bush {
    pub state: u8,
    pub counter: u32,
    pub countdown: u32,
    pub cut: bool,
    pub wait: u32,
    /// The base scale K.
    pub k: f32,
    /// The up axis the bush is drawn along (tilted by the hero bend, Behavior 6).
    pub up: [f32; 3],
}

/// The hero as a bush bend sees it: horizontal distance to the bush, whether the body test
/// hits, the hero's speed (units per second) and facing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BendHero {
    pub distance: f32,
    pub contact: bool,
    pub speed: f32,
    pub facing: [f32; 3],
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if n > 0.0 { [v[0] / n, v[1] / n, v[2] / n] } else { [0.0, 1.0, 0.0] }
}

const UP: [f32; 3] = [0.0, 1.0, 0.0];

/// The sparkle wait trunc((u + 0.5) x 60) with u in [-0.2, 0.2]: 18 to 42 ticks.
pub fn sparkle_wait(r: &mut crate::rng::Rng) -> u32 {
    ps2_float::fpu::mul(r.frange(-0.2, 0.2) + 0.5, 60.0) as u32
}

/// What one bush step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BushStep {
    /// Cuts made this step (each rolls a drop).
    pub cuts: u32,
    /// Effect 139 starts at the bush.
    pub sparkle: bool,
}

impl Bush {
    pub fn new(k: f32, r: &mut crate::rng::Rng) -> Self {
        Self { state: 2, counter: 0, countdown: 0, cut: false, wait: sparkle_wait(r), k, up: UP }
    }

    /// Whether the helpers (and so the hit tests) run this step: not while regrowing with the
    /// counter at most 450, and not while cut.
    pub fn hittable(&self) -> bool {
        !self.cut && !(self.state == 1 && self.counter <= 450)
    }

    /// Spring-back: up = normalize(0.925 up + 0.075 (0, 1, 0)).
    fn spring_back(&mut self) {
        let u = self.up;
        self.up = normalize(std::array::from_fn(|i| 0.925 * u[i] + 0.075 * UP[i]));
    }

    /// The hero bend within 5 units: with body contact and speed above 0.1 the target axis is
    /// 0.76 U + 0.24 facing, else the current up; when its y is below 0.994 the up axis becomes
    /// normalize(0.02 U + 0.98 T).
    fn bend(&mut self, h: &BendHero) {
        let t = if h.contact && h.speed > 0.1 { std::array::from_fn(|i| 0.76 * UP[i] + 0.24 * h.facing[i]) } else { self.up };
        if t[1] < 0.994 {
            self.up = normalize(std::array::from_fn(|i| 0.02 * UP[i] + 0.98 * t[i]));
        }
    }

    /// The up axis step of the helpers (Behavior 6 step 1). Key 5 (`contact_only`): nothing
    /// beyond 5 units (the tilt is kept), the bend within. Other keys: the bend while the hero
    /// holds the bush (body contact, speed above 0.1), else spring-back (Medium: the bend latch
    /// reading of "not held by the hero bend"). A cut bush only springs back.
    pub fn bend_step(&mut self, contact_only: bool, hero: &BendHero) {
        if self.state == 1 && self.counter <= 450 {
            return;
        }
        if self.cut {
            self.spring_back();
            return;
        }
        let near = hero.distance < 5.0;
        if contact_only {
            if near {
                self.bend(hero);
            }
            return;
        }
        // The bend latch: a bush the hero is bending is held, and springs back only once free.
        if near && hero.contact && hero.speed > 0.1 {
            self.bend(hero);
        } else {
            self.spring_back();
        }
    }

    /// One logic step: `hits` triggers fired this step (weapon first, then each shot in range;
    /// each cuts again), and whether the bush may sparkle. Returns the cuts and the sparkle.
    pub fn step(&mut self, hits: u32, may_sparkle: bool, r: &mut crate::rng::Rng) -> BushStep {
        let mut out = BushStep::default();
        let helpers = !(self.state == 1 && self.counter <= 450);
        if helpers {
            if self.cut {
                self.countdown = self.countdown.saturating_sub(1);
                if self.countdown == 0 {
                    self.cut = false;
                    self.state = 1;
                    self.counter = 0;
                }
            } else {
                for _ in 0..hits {
                    self.counter = 0;
                    self.countdown = 3800;
                    self.cut = true;
                    self.state = 0;
                    out.cuts += 1;
                }
            }
        }
        if !self.cut && self.state == 1 {
            if self.counter < 600 {
                self.counter += 1;
            }
            if self.counter >= 600 {
                self.state = 2;
            }
        }
        if self.wait > 0 {
            self.wait -= 1;
        } else if self.state == 2 && may_sparkle {
            out.sparkle = true;
            self.wait = sparkle_wait(r);
        }
        out
    }

    /// The draw scale v (Behavior 6 step 2), in single precision.
    pub fn scale(&self) -> f32 {
        use ps2_float::fpu::mul;
        if self.cut {
            mul(mul(self.k, self.countdown as f32), 1.0 / 3800.0)
        } else if self.state == 1 {
            mul(mul(self.k, self.counter as f32), 1.0 / 600.0)
        } else {
            self.k
        }
    }
}

/// A bush drop (Behavior 7) for main model `main` and roll r in [0, 100]: the items to spawn
/// (the coin bush's four coins with `luck`), or none. `negg_open` is whether the Negg's
/// once-per-area bit is clear.
pub fn bush_drop(main: u16, r: i32, luck: i32, negg_open: bool) -> Vec<u16> {
    let pick = |rules: &[(i32, u16)]| rules.iter().find(|&&(top, _)| r <= top).map(|&(_, id)| vec![id]).unwrap_or_default();
    match main {
        223 | 227 | 265 => pick(&[(9, 149), (24, 150)]),
        229 => pick(&[(19, 146)]),
        231 => pick(&[(14, 156)]),
        248 => pick(&[(19, 157)]),
        253 => pick(&[(49, 147)]),
        256 | 274 => pick(&[(0, 152), (6, 155), (24, 153)]),
        259 => pick(&[(1, 138), (11, 136), (39, 135)]),
        262 => {
            let v = r / 2 + 3 * luck;
            let id = if v <= 30 {
                135
            } else if v <= 60 {
                136
            } else {
                138
            };
            vec![id; 4]
        }
        271 if negg_open => vec![174],
        277 if negg_open => vec![151],
        280 if negg_open => vec![154],
        271 | 277 | 280 => Vec::new(),
        _ => pick(&[(0, 138), (5, 136), (19, 135)]),
    }
}

/// The Negg item of a Negg bush model (174 Red, 151 Golden, 154 Starry).
pub fn negg_of(main: u16) -> Option<u16> {
    match main {
        271 => Some(174),
        277 => Some(151),
        280 => Some(154),
        _ => None,
    }
}

/// The upper bound of the random scale: 3.5 for the bushes of model 265, else 1.0.
pub fn scale_max(kind: &Kind) -> f32 {
    if kind.near == Some(265) { 3.5 } else { 1.0 }
}

/// The jitter half-range J (Behavior 2 step 4): 0.9 for grass, 0 for reeds; with the jitter flag
/// 0.4 x depth / rows for bushes, beds and trees and 0.27 x depth / rows for flowers.
pub fn jitter(family: Family, flag: bool, depth: f32, rows: usize) -> f32 {
    let per_row = if rows == 0 { 0.0 } else { depth / rows as f32 };
    match family {
        Family::Grass => 0.9,
        Family::Reed => 0.0,
        _ if !flag => 0.0,
        Family::Flowers => 0.27 * per_row,
        _ => 0.4 * per_row,
    }
}

/// The ceiling the ground probe starts above the patch height: 8 for the bushes of models 198
/// and 223 (SPEC-0018 Ceilings supplied by callers); None for the default column probe.
pub fn probe_ceiling(kind: &Kind) -> Option<f32> {
    matches!(kind.near, Some(198 | 223)).then_some(8.0)
}

/// The far distance F of a patch (Behavior 3): the override D when A or D is nonzero, else
/// 4000 (the LOD node's value when the model has one is not read).
pub fn far_distance(overrides: [f32; 4]) -> f32 {
    if overrides[0] != 0.0 || overrides[3] != 0.0 { overrides[3] } else { 4000.0 }
}

/// The patch radius (Behavior 2 step 2): 0.7072 x the larger box side.
pub fn radius(area: [f32; 4]) -> f32 {
    0.7072 * (area[1] - area[0]).max(area[3] - area[2])
}

/// A placed plant: position, yaw (radians), scale K and colour bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub position: [f32; 3],
    pub yaw: f32,
    pub scale: f32,
    pub colour: [u8; 4],
}

/// Place plant `offset` of a row at `row` (Behavior 2 steps 3 to 8): jitter x and z in [-J, J],
/// snap to the ground (`ground` returns None when there is no ground: the plant is not placed),
/// a random yaw and a scale in [s0, max].
#[allow(clippy::too_many_arguments)]
pub fn place(kind: &Kind, area: [f32; 4], height: f32, row: f32, offset: f32, j: f32, colour: [u8; 4], ground: &dyn Fn(f32, f32) -> Option<f32>, r: &mut crate::rng::Rng) -> Option<Placed> {
    let mut x = area[0] + offset;
    let mut z = area[2] + row;
    if j > 0.0 {
        x += r.frange(-j, j);
        z += r.frange(-j, j);
    }
    let y = if kind.snap { ground(x, z).filter(|&h| h > -1.0e8)? } else { height };
    let yaw = r.frange(-std::f32::consts::PI, std::f32::consts::PI);
    let scale = r.frange(kind.s0, scale_max(kind));
    Some(Placed { position: [x, y, z], yaw, scale, colour })
}

/// A reed (Behavior 9): state 0 standing, 1 cut and falling, 2 gone (not drawn, never regrown
/// until the patch is built again).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reed {
    pub state: u8,
    /// The up axis it is drawn along.
    pub up: [f32; 3],
    /// The cut piece's displacement from its anchor (the reed's base).
    pub offset: [f32; 3],
    velocity: [f32; 3],
    fall: u8,
}

impl Default for Reed {
    fn default() -> Self {
        Self { state: 0, up: UP, offset: [0.0; 3], velocity: [0.0; 3], fall: 0 }
    }
}

impl Reed {
    /// The bend (state 0): within 5 units the reed tilts away from the hero, taken as the bush
    /// bend's target 0.76 U + 0.24 `away` (the horizontal direction from the hero to the reed)
    /// and its 0.98 step; farther, or with no direction, it springs back as a bush does.
    pub fn bend_step(&mut self, distance: f32, away: [f32; 3]) {
        if self.state != 0 {
            return;
        }
        if distance < 5.0 && away != [0.0; 3] {
            let t: [f32; 3] = std::array::from_fn(|i| 0.76 * UP[i] + 0.24 * away[i]);
            if t[1] < 0.994 {
                self.up = normalize(std::array::from_fn(|i| 0.02 * UP[i] + 0.98 * t[i]));
            }
        } else {
            let u = self.up;
            self.up = normalize(std::array::from_fn(|i| 0.925 * u[i] + 0.075 * UP[i]));
        }
    }

    /// The cut by a weapon hit: state 1 and the fall's sub-state 0, the piece launched along
    /// `velocity` (the blade direction). False when the reed is not standing.
    pub fn cut(&mut self, velocity: [f32; 3]) -> bool {
        if self.state != 0 {
            return false;
        }
        self.state = 1;
        self.fall = 0;
        self.velocity = velocity;
        true
    }

    /// One fall tick (state 1). `ground` is the ground probe result under the piece with ceiling
    /// y + 8.0 and `base_y` the anchor's height. Sub-state 0: offset += 0.05 v until the piece is
    /// more than 0.5 from its anchor; 1: y -= 0.07 until below the ground less 1.25; 2: up =
    /// normalize(0.965 up + 0.035 v) until |up.y| is 0.001 or less, then state 2.
    pub fn fall_step(&mut self, base_y: f32, ground: Option<f32>) {
        if self.state != 1 {
            return;
        }
        match self.fall {
            0 => {
                for k in 0..3 {
                    self.offset[k] += 0.05 * self.velocity[k];
                }
                if self.offset.iter().map(|c| c * c).sum::<f32>().sqrt() > 0.5 {
                    self.fall = 1;
                }
            }
            1 => {
                self.offset[1] -= 0.07;
                if ground.is_none_or(|g| base_y + self.offset[1] < g - 1.25) {
                    self.fall = 2;
                }
            }
            _ => {
                let (u, v) = (self.up, self.velocity);
                self.up = normalize(std::array::from_fn(|i| 0.965 * u[i] + 0.035 * v[i]));
                if self.up[1].abs() <= 0.001 || v == [0.0; 3] {
                    self.state = 2;
                }
            }
        }
    }
}

/// The reed drop (Behavior 9): r uniform in 0 to 100; 0 gives item 138 (25 NP), 1 to 5 item 136
/// (5 NP), 6 to 19 item 135 (1 NP), else nothing. Luck plays no part.
pub fn reed_drop(r: i32) -> Option<u16> {
    match r {
        0 => Some(138),
        1..=5 => Some(136),
        6..=19 => Some(135),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn foliage_distances_and_weights() {
        use super::Distances;
        // SPEC-0060 section 3 vector: S 80, +0x50 30, fade length 12, fade end 150, no far
        // object: O 50, W 50, F 150, S 80.
        let x = Distances::of([0.0; 4], Some([30.0, 80.0, 12.0, 150.0]), false);
        assert_eq!((x.out, x.fade, x.far, x.switch), (50.0, 50.0, 150.0, 80.0));
        // Overrides (200, 32, 50, 150): F 150, S 232, O 32, W 50.
        let o = Distances::of([200.0, 32.0, 50.0, 150.0], None, true);
        assert_eq!((o.far, o.switch, o.out, o.fade), (150.0, 232.0, 32.0, 50.0));
        // Near only: full to F - W = 100, half at 125, none at 150.
        assert_eq!(x.weights(90.0, true, false), (1.0, 0.0));
        assert_eq!(x.weights(125.0, true, false), (0.5, 0.0));
        assert_eq!(x.weights(160.0, true, false), (0.0, 0.0));
        // Both models with S 80, O 50, W 50: near only below 30, the far fading in to 80, the
        // near fading out to 130.
        let both = Distances { far: 150.0, switch: 80.0, out: 50.0, fade: 50.0 };
        assert_eq!(both.weights(20.0, true, true), (1.0, 0.0));
        assert_eq!(both.weights(55.0, true, true), (1.0, 0.5));
        assert_eq!(both.weights(105.0, true, true), (0.5, 1.0));
        assert_eq!(both.weights(140.0, true, true), (0.0, 1.0));
        assert_eq!(both.weights(10.0, false, true), (0.0, 1.0));
    }
    #[test]
    fn reed_bend_cut_fall_drop() {
        use super::*;
        let mut r = Reed::default();
        // Within 5 units it tilts away from the hero (+x); farther it springs back.
        r.bend_step(3.0, [1.0, 0.0, 0.0]);
        assert!(r.up[0] > 0.2 && r.up[1] < 0.99);
        let tilted = r.up;
        r.bend_step(6.0, [1.0, 0.0, 0.0]);
        assert!(r.up[0] < tilted[0]);
        // A cut launches the piece; it moves 0.05 v a tick until 0.5 out (11 ticks at |v| 1).
        assert!(r.cut([1.0, 0.0, 0.0]));
        assert!(!r.cut([1.0, 0.0, 0.0]));
        for _ in 0..11 {
            r.fall_step(0.0, Some(0.0));
        }
        assert_eq!(r.fall, 1);
        // It sinks 0.07 a tick until below the ground less 1.25, then turns over and is gone.
        for _ in 0..19 {
            r.fall_step(0.0, Some(0.0));
        }
        assert_eq!(r.fall, 2);
        for _ in 0..400 {
            r.fall_step(0.0, Some(0.0));
        }
        assert_eq!(r.state, 2);
        // The drop roll.
        assert_eq!((reed_drop(0), reed_drop(5), reed_drop(19), reed_drop(20), reed_drop(100)), (Some(138), Some(136), Some(135), None, None));
    }

    use super::*;

    #[test]
    fn hero_bend_and_spring_back() {
        let mut r = crate::rng::Rng::seeded(1);
        let mut b = Bush::new(1.0, &mut r);
        // SPEC-0060 vector: speed 0.05 does not lean; speed 2.0 facing +X gives about
        // (0.2940, 0.9558, 0) from upright.
        let slow = BendHero { distance: 3.0, contact: true, speed: 0.05, facing: [1.0, 0.0, 0.0] };
        b.bend_step(true, &slow);
        assert_eq!(b.up, [0.0, 1.0, 0.0]);
        let walk = BendHero { speed: 2.0, ..slow };
        b.bend_step(true, &walk);
        assert!((b.up[0] - 0.2940).abs() < 1e-3 && (b.up[1] - 0.9558).abs() < 1e-3, "{:?}", b.up);
        // Key 5 beyond 5 units keeps its tilt.
        let far = BendHero { distance: 6.0, ..walk };
        let tilted = b.up;
        b.bend_step(true, &far);
        assert_eq!(b.up, tilted);
        // Other keys spring back once the hero has gone.
        for _ in 0..200 {
            b.bend_step(false, &far);
        }
        assert!(b.up[1] > 0.9999, "{:?}", b.up);
    }

    #[test]
    fn families_and_rules() {
        assert_eq!(kind(0).unwrap().far, Some(283));
        assert!(!kind(12).unwrap().snap);
        assert_eq!(kind(33).unwrap().near, Some(262));
        assert_eq!((kind(15), kind(40)), (None, None));
        assert_eq!(probe_ceiling(&kind(5).unwrap()), Some(8.0));
        assert_eq!(jitter(Family::Grass, false, 10.0, 2), 0.9);
        assert_eq!(jitter(Family::Bush, false, 10.0, 2), 0.0);
        assert!((jitter(Family::Bush, true, 10.0, 2) - 2.0).abs() < 1e-6);
        assert!((jitter(Family::Flowers, true, 10.0, 2) - 1.35).abs() < 1e-6);
        assert_eq!((far_distance([0.0; 4]), far_distance([200.0, 32.0, 50.0, 150.0])), (4000.0, 150.0));
        // SPEC-0006 check: pack 678 record 0's plant 1 of row 0 lies at x = -1015.50 + 1.2959.
        let k = kind(9).unwrap();
        let mut r = crate::rng::Rng::seeded(1);
        let p = place(&k, [-1015.5, -1012.81, 1320.54, 1325.01], 74.1661, 2.073, 1.2959, 0.0, [64, 64, 64, 128], &|_, _| Some(74.0), &mut r).unwrap();
        assert!((p.position[0] + 1014.2041).abs() < 1e-3 && (p.position[2] - 1322.613).abs() < 1e-3);
        assert!((0.6..=1.0).contains(&p.scale));
        assert!(place(&k, [0.0; 4], 0.0, 0.0, 0.0, 0.0, [0; 4], &|_, _| None, &mut r).is_none());
    }

    /// Behavior 6: a cut shrinks the bush over 3,800 ticks, then it regrows over 600 and is
    /// grown 4,400 ticks after the cut; while regrowing with counter 450 it cannot be cut, with
    /// 451 it can; two triggers in one step cut twice.
    #[test]
    fn bush_cut_and_regrow() {
        let mut r = crate::rng::Rng::seeded(3);
        let mut b = Bush::new(0.8, &mut r);
        assert!((18..=42).contains(&b.wait));
        assert_eq!(b.step(2, false, &mut r).cuts, 2);
        assert_eq!((b.state, b.countdown, b.cut), (0, 3800, true));
        let mut ticks = 1;
        while b.state != 2 {
            b.step(0, false, &mut r);
            ticks += 1;
            if b.state == 1 && b.counter == 450 {
                assert!(!b.hittable());
                assert_eq!(b.step(1, false, &mut r).cuts, 0);
                ticks += 1;
                assert!(b.hittable());
            }
        }
        assert_eq!(ticks, 4400);
        assert_eq!(b.scale(), 0.8);
        // The sparkle on the first grown tick when the wait is 0.
        b.wait = 0;
        assert!(b.step(0, true, &mut r).sparkle);
    }

    #[test]
    fn bush_drops() {
        assert_eq!(bush_drop(196, 0, 0, true), vec![138]);
        assert_eq!(bush_drop(196, 5, 0, true), vec![136]);
        assert_eq!(bush_drop(196, 19, 0, true), vec![135]);
        assert!(bush_drop(196, 20, 0, true).is_empty());
        assert_eq!(bush_drop(262, 100, 0, true), vec![136; 4]);
        assert_eq!(bush_drop(262, 60, 24, true), vec![138; 4]);
        assert_eq!(bush_drop(277, 50, 0, true), vec![151]);
        assert!(bush_drop(277, 50, 0, false).is_empty());
        assert_eq!(bush_drop(256, 0, 0, true), vec![152]);
        assert_eq!(cut_model(262), Some(263));
    }
}
