//! Movable blocks (SPEC-0048 Movable blocks; SPEC-0033 Behaviors 4A.2, 15 and 15A): the block
//! record and its placement, the push start with the second clearance test, the per-tick move,
//! the character contact against the moving box, the current-block scan and the face finder;
//! and the hero's block state 18 (walk to the grab spot, then push, pull or move the block
//! sideways by the stick).
//!
//! Not covered: the footprint step (no block has a footprint word on disc), the predicted-position
//! second contact test, the block-slide cue and the side-touch latch, the trigger-volume fires of a
//! moving block, the attached effects and the sound index, the climb onto a block (state 58) and
//! the block-edge hang (state 19).

use crate::contact::{self, BoxShape};
use crate::hero::Vec3;

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Vec3, s: f32) -> Vec3 {
    a.map(|c| c * s)
}

/// The ceiling offset of the placement's ground probe.
pub const PLACE_CEILING: f32 = 3.0;
/// The face finder's gap limit (word 0x2DD568) and dominant-share ratio (0x2DD570).
pub const FACE_GAP: f32 = 2.0;
pub const FACE_RATIO: f32 = 0.5;
/// The grab spot's latch distance (0x2DD56C) and the approach turn rate (0x2DD58C).
pub const LATCH: f32 = 0.25;
pub const APPROACH_TURN: f32 = 0.25;
/// The block contact's character radius and the scan radius.
pub const CONTACT_RADIUS: f32 = 0.75;
pub const SCAN_RADIUS: f32 = 10.0;

/// A movable block record (SPEC-0048 Movable block record) with its box.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// +0x00 home, +0x10 axis A, +0x20 vertical, +0x30 axis B, +0x40 position.
    pub home: Vec3,
    pub a: Vec3,
    pub up: Vec3,
    pub b: Vec3,
    pub position: Vec3,
    /// The entry's size S and height H, and the step length L = 0.5 S.
    pub size: f32,
    pub height: f32,
    pub step: f32,
    /// +0x74 active word, +0x88 moving flag, +0x90 tick counter i, +0x91 tick count n, +0x94
    /// direction D (-1 after a placement).
    pub active: i32,
    pub moving: bool,
    pub i: u8,
    pub n: i8,
    pub dir: i32,
    /// The box record: reference corner, first axis (+0x40), second axis (+0x50), vertical
    /// axis (+0x60), extents (vertical +0x24, along the second +0x28, along the first +0x2C).
    pub corner: Vec3,
    pub box_axes: [Vec3; 3],
    pub extents: [f32; 3],
    /// The box top as loaded (the climb test reads it; a push does not change it).
    pub loaded_top: f32,
    /// +0x60, the volume the block last fired (its trigger index), and whether this tick's step
    /// tests the volumes (while moving and on the tick the flag clears).
    pub last_fired: Option<u16>,
    pub testing: bool,
}

impl Block {
    /// The record built at world load, then placed (SPEC-0048 Source and Placement).
    pub fn new(size: f32, height: f32, corner: Vec3, box_axes: [Vec3; 3], extents: [f32; 3], probe: impl Fn(f32, f32, f32) -> Option<f32>) -> Self {
        let mut b = Self {
            home: [0.0; 3],
            a: box_axes[0],
            up: box_axes[2],
            b: box_axes[1],
            position: [0.0; 3],
            size,
            height,
            step: 0.5 * size,
            active: 0,
            moving: false,
            i: 0,
            n: 0,
            dir: -1,
            corner,
            box_axes,
            extents,
            loaded_top: corner[1] + extents[0],
            last_fired: None,
            testing: false,
        };
        b.place(probe);
        b
    }

    /// Placement: the corner's height from the ground probe (ceiling corner + 3.0), the axes
    /// copied, the position and the home at corner + 0.5 S A + 0.5 S B, all four directions
    /// allowed, not moving, both counters 0, the direction -1, active 1 and L = 0.5 S. A probe
    /// that finds nothing keeps the corner's height.
    pub fn place(&mut self, probe: impl Fn(f32, f32, f32) -> Option<f32>) {
        if let Some(y) = probe(self.corner[0], self.corner[2], self.corner[1] + PLACE_CEILING) {
            self.corner[1] = y;
        }
        self.a = self.box_axes[0];
        self.b = self.box_axes[1];
        self.up = self.box_axes[2];
        let half = 0.5 * self.size;
        self.position = add(add(self.corner, scale(self.a, half)), scale(self.b, half));
        self.home = self.position;
        self.moving = false;
        self.i = 0;
        self.n = 0;
        self.dir = -1;
        self.active = 1;
        self.step = half;
    }

    /// Move to locator: the corner becomes X - 0.5 S (second axis) - 0.5 S (first axis), then
    /// the block is placed again.
    pub fn move_to(&mut self, x: Vec3, probe: impl Fn(f32, f32, f32) -> Option<f32>) {
        let half = 0.5 * self.size;
        self.corner = sub(sub(x, scale(self.box_axes[1], half)), scale(self.box_axes[0], half));
        self.place(probe);
    }

    /// Reacquire height patch: move to the block's own position.
    pub fn reacquire(&mut self, probe: impl Fn(f32, f32, f32) -> Option<f32>) {
        self.move_to(self.position, probe);
    }

    /// The unit vector of push direction d: 0 +A, 1 +B, 2 -A, 3 -B.
    pub fn direction(&self, d: i32) -> Option<Vec3> {
        match d {
            0 => Some(self.a),
            1 => Some(self.b),
            2 => Some(scale(self.a, -1.0)),
            3 => Some(scale(self.b, -1.0)),
            _ => None,
        }
    }

    /// The solid box: the placement corner plus the displacement from home, with the record's
    /// vertical, second and first axes and their extents.
    pub fn shape(&self) -> BoxShape {
        BoxShape { origin: add(self.corner, sub(self.position, self.home)), axes: [self.box_axes[2], self.box_axes[1], self.box_axes[0]], extents: self.extents }
    }

    /// The block top: the position height plus H.
    pub fn top(&self) -> f32 {
        self.position[1] + self.height
    }

    /// Per tick (active blocks): while moving and i < n, one step of L / n along D; while
    /// moving and i >= n, the moving flag clears with no motion. Returns whether it moved.
    pub fn tick(&mut self) -> bool {
        self.testing = self.active != 0 && self.moving;
        if self.active == 0 || !self.moving {
            return false;
        }
        if (self.i as i32) < self.n as i32 {
            if let Some(d) = self.direction(self.dir) {
                self.position = add(self.position, scale(d, self.step / self.n as f32));
            }
            self.i = self.i.wrapping_add(1);
            true
        } else {
            self.moving = false;
            false
        }
    }
}

/// The four relations of the second clearance test, on world x and z (A taken as +Z and B as
/// +X); r is the size of the first block.
fn forward(p: &Block, q: &Block, w: bool, x_axis: bool) -> bool {
    let r = p.size;
    let (along, across) = if x_axis { (q.position[0] - p.position[0], q.position[2] - p.position[2]) } else { (q.position[2] - p.position[2], q.position[0] - p.position[0]) };
    let reach = if w { 1.5 * (r + 0.5) } else { r + 0.5 };
    along >= 0.0 && across.abs() < r - 0.05 && along <= reach
}

fn side(p: &Block, q: &Block, w: bool, x_axis: bool) -> bool {
    let r = p.size;
    let (along, across) = if x_axis { (q.position[0] - p.position[0], q.position[2] - p.position[2]) } else { (q.position[2] - p.position[2], q.position[0] - p.position[0]) };
    let (lo, hi) = if w { (across - r, across) } else { (across, across + r) };
    let c = r - 0.05;
    along >= 0.0 && along <= r + 0.5 && lo < c && hi > -c
}

/// Whether block q is in the way of pushing p toward d from face f (the SPEC-0048 table).
fn in_way(p: &Block, q: &Block, d: i32, f: i32) -> bool {
    let w = f >= 2;
    // The pull pairs of D 0 and 1 use w = 1 for faces 2 and 3; D 2 and 3 exchange the roles
    // and use w = 1 for faces 0 and 1.
    match (d, f) {
        (0, 0 | 2) => forward(p, q, w, false),
        (0, 1 | 3) => side(p, q, w, false),
        (1, 0 | 2) => side(p, q, w, true),
        (1, 1 | 3) => forward(p, q, w, true),
        (2, 0 | 2) => forward(q, p, !w, false),
        (2, 1 | 3) => side(q, p, !w, false),
        (3, 0 | 2) => side(q, p, !w, true),
        (3, 1 | 3) => forward(q, p, !w, true),
        _ => false,
    }
}

/// The push start (SPEC-0048): block k toward d from face f over n ticks. Accepted when the
/// block is not moving, d is a direction (all four are always allowed) and no other block is in
/// the way; then moving, i = 0, D = d and n stored as a signed byte.
pub fn push_start(blocks: &mut [Block], k: usize, d: i32, f: i32, n: i32) -> bool {
    let Some(p) = blocks.get(k) else { return false };
    if p.moving || !(0..4).contains(&d) {
        return false;
    }
    if blocks.iter().enumerate().any(|(j, q)| j != k && in_way(p, q, d, f)) {
        return false;
    }
    let p = &mut blocks[k];
    p.moving = true;
    p.i = 0;
    p.dir = d;
    p.n = n as i8;
    true
}

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

/// The current block of a character (SPEC-0018 via SPEC-0048 Solid for characters, step 1):
/// the nearest active block within 10 horizontally whose top is more than 0.1 above the feet,
/// with its horizontal distance.
pub fn current(blocks: &[Block], feet: Vec3) -> Option<(usize, f32)> {
    blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.active != 0)
        .map(|(k, b)| (k, horizontal_distance(b.position, feet), b.top()))
        .filter(|&(_, d, top)| d * d < SCAN_RADIUS * SCAN_RADIUS && top > feet[1] + 0.1)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, d, _)| (k, d))
}

/// What the block contact pass found for one character.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contact {
    pub hit: bool,
    /// A top face (|n.y| > 0.9): the floor-contact flag.
    pub floor: bool,
    /// A side face (|n.y| < 0.1) of a block whose top is more than 1.0 and less than 5.0 above
    /// the character: the hero's side-touch latch.
    pub side_touch: bool,
}

/// The block contact pass for one character (SPEC-0048 Solid for characters, steps 1 to 3):
/// the active blocks within 10, nearest first, each pushing the reference point out of its box
/// grown by 0.75. A top-face push leaves the point 0.01 inside the face's test bound, so a
/// character standing on a block keeps touching it every tick (not from the spec: the box test
/// is strict at the bound).
pub fn contact(blocks: &[Block], p: &mut Vec3) -> Contact {
    let mut near: Vec<(f32, &Block)> = blocks.iter().filter(|b| b.active != 0).map(|b| (horizontal_distance(b.position, *p), b)).filter(|(d, _)| d * d < SCAN_RADIUS * SCAN_RADIUS).collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Contact::default();
    for (_, b) in near.into_iter().take(100) {
        let Some((_, n, dist)) = contact::push(*p, &b.shape(), CONTACT_RADIUS) else { continue };
        for k in 0..3 {
            p[k] += n[k] * dist;
        }
        out.hit = true;
        if n[1].abs() > 0.9 {
            out.floor = true;
            if n[1] > 0.0 {
                p[1] -= 0.01;
            }
        } else if n[1].abs() < 0.1 {
            let above = b.top() - p[1];
            out.side_touch |= above > 1.0 && above < 5.0;
        }
    }
    out
}

/// The climb-onto-block fit test (SPEC-0033 Behavior 15), gates 3 to 5 (the caller checks the
/// block and the moving state): the loaded top 0.2 to 3.5 above the feet (strict), the facing
/// within the face (dot above 0.5), and the stick pushed into it (`a` along the facing above 0
/// and above |`b`|, the side part).
pub fn fits(b: &Block, m: u8, feet: Vec3, facing: Vec3, a: f32, side: f32) -> bool {
    let h = b.loaded_top - feet[1];
    let Some(d) = b.direction(m as i32) else { return false };
    h > 0.2 && h < 3.5 && dot(facing, d) > 0.5 && a.abs() > side.abs() && a > 0.0
}

/// The push-into-face counter's limit (word 0x2DD574).
pub const FACE_COUNT: u32 = 15;

/// What states 19 and 58 ask of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeStep {
    pub request: u32,
    /// The facing to keep (the held face's direction).
    pub facing: Option<Vec3>,
    /// A position to move the hero to this tick (the approach's ease; not from the spec).
    pub place: Option<Vec3>,
    /// The state to leave to (1 or 4), with a move along the face direction.
    pub exit: Option<(u8, f32)>,
}

/// A clip as the edge states read it: id and progress.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeClip {
    pub action: u32,
    pub progress: f32,
}

/// The ease of an approach: half the remaining way each tick (the spec's "rate 0.5"; how the
/// hero gets to the edge point is not given, R-0079).
fn ease(hero: Vec3, target: Vec3) -> Vec3 {
    std::array::from_fn(|k| hero[k] + 0.5 * (target[k] - hero[k]))
}

/// State 19, hanging from a block's top edge (SPEC-0033 Behavior 51).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hang {
    pub block: usize,
    pub face: u8,
    pub sub: u8,
}

impl Hang {
    pub fn new(block: usize, face: u8) -> Self {
        Self { block, face, sub: 0 }
    }

    /// One tick. `side` is the stick's side value (positive to the left), `forward` whether the
    /// stick is pushed forward, `gap` the ground gap.
    pub fn tick(&mut self, blocks: &[Block], hero: Vec3, side: f32, forward: bool, gap: f32, clip: EdgeClip) -> EdgeStep {
        let Some(b) = blocks.get(self.block) else { return EdgeStep { exit: Some((1, 0.0)), ..EdgeStep::default() } };
        let Some(d) = b.direction(self.face as i32) else { return EdgeStep { exit: Some((1, 0.0)), ..EdgeStep::default() } };
        let mut step = EdgeStep { facing: Some(d), ..EdgeStep::default() };
        match self.sub {
            0 => {
                self.sub = 1;
                step.request = 72;
            }
            1 => {
                step.request = 72;
                if clip.action != 72 {
                    return step;
                }
                // The edge point: the block position less D (S / 2 + 0.35), at the top less 3.7
                // (the spec's box-record terms read as the block top; R-0079).
                let half = 0.5 * b.size + 0.35;
                let target = [b.position[0] - d[0] * half, b.top() - 3.7, b.position[2] - d[2] * half];
                let along = dot([target[0] - hero[0], 0.0, target[2] - hero[2]], d);
                if along < 0.0 && gap <= 0.01 {
                    step.request = 0;
                    step.exit = Some((1, 0.0));
                } else if along > 0.25 {
                    step.place = Some(ease(hero, target));
                } else {
                    self.sub = 2;
                    step.request = 71;
                    step.place = Some(target);
                }
            }
            2 => {
                step.request = 71;
                if forward {
                    self.sub = 4;
                    step.request = 59;
                } else if clip.action == 71 && clip.progress >= 0.95 {
                    self.sub = 3;
                    step.request = 73;
                }
            }
            3 => {
                step.request = 73;
                if forward {
                    self.sub = 4;
                    step.request = 59;
                } else {
                    // The along position on the edge, 0 to S: shimmy 61 lowers it (gate above
                    // 1.0) and 60 raises it (gate below S - 1.0). The edge's direction is the
                    // hero's side axis (its sign is not given; R-0079).
                    let e = [-d[2], 0.0, d[0]];
                    let along = 0.5 * b.size + dot(sub(hero, b.position), e);
                    if side > 0.0 && along > 1.0 {
                        step.request = 61;
                    } else if side < 0.0 && along < b.size - 1.0 {
                        step.request = 60;
                    }
                }
            }
            _ => {
                step.request = 59;
                if clip.action == 59 && clip.progress >= 0.95 {
                    step.exit = Some(if forward { (4, 0.0) } else { (1, 0.1) });
                }
            }
        }
        step
    }
}

/// State 58, climbing onto a block (SPEC-0033 Behavior 51).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climb {
    pub block: usize,
    pub face: u8,
    pub sub: u8,
}

impl Climb {
    pub fn new(block: usize, face: u8) -> Self {
        Self { block, face, sub: 0 }
    }

    pub fn tick(&mut self, blocks: &[Block], hero: Vec3, forward: bool, clip: EdgeClip) -> EdgeStep {
        let Some(b) = blocks.get(self.block) else { return EdgeStep { exit: Some((1, 0.0)), ..EdgeStep::default() } };
        let Some(d) = b.direction(self.face as i32) else { return EdgeStep { exit: Some((1, 0.0)), ..EdgeStep::default() } };
        let mut step = EdgeStep { facing: Some(d), ..EdgeStep::default() };
        if self.sub == 0 {
            // The target: the block position less D (S / 2 + 0.75), at the top less 1.2.
            step.request = 70;
            let half = 0.5 * b.size + 0.75;
            let target = [b.position[0] - d[0] * half, b.top() - 1.2, b.position[2] - d[2] * half];
            let far = horizontal_distance(target, hero) > LATCH || (target[1] - hero[1]).abs() > LATCH;
            if far {
                step.place = Some(ease(hero, target));
            } else {
                self.sub = 2;
                step.request = 134;
                step.place = Some(target);
            }
        } else {
            step.request = 134;
            if clip.action == 134 && clip.progress >= 0.95 {
                step.exit = Some(if forward { (4, 0.0) } else { (1, 0.0) });
            }
        }
        step
    }
}

/// The block face finder (SPEC-0033 Behavior 15A, no footprint step): the face index 0 to 3
/// the hero holds, or None.
pub fn face(b: &Block, distance: f32, hero: Vec3, facing: Vec3) -> Option<u8> {
    let gap = distance - 0.5 * b.size;
    if !(gap > 0.0 && gap < FACE_GAP) || b.position[1] - hero[1] > 10.0 {
        return None;
    }
    let o = [b.position[0] - hero[0], 0.0, b.position[2] - hero[2]];
    let (da, db) = (dot(o, b.a), dot(o, b.b));
    let len = dot(o, o).sqrt();
    let (m, d) = if db.abs() < da.abs() { (if da > 0.0 { 0 } else { 2 }, da) } else { (if db > 0.0 { 1 } else { 3 }, db) };
    if len <= 0.0 || d.abs() / len <= FACE_RATIO {
        return None;
    }
    (dot(facing, b.direction(m)?) > 0.0).then_some(m as u8)
}

/// What state 18 asks of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HoldStep {
    /// The clip to request.
    pub request: u32,
    /// The facing to steer toward and the rate (None: keep the facing).
    pub steer: Option<(Vec3, f32)>,
    /// The approach's walk: the hero steps forward at rate 1.0 (root motion).
    pub walking: bool,
    /// A push start was accepted (cue 11 and the strain voice play).
    pub pushed: bool,
    /// The held block's horizontal motion since the last tick, which the hero follows while it
    /// holds the block (not from the spec: the push clips carry no root motion, R-0077).
    pub follow: Vec3,
}

/// The hero's block state (SPEC-0033 Behavior 4A.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hold {
    pub block: usize,
    /// The direction mode: the face m the hero holds.
    pub face: u8,
    /// The grab latch (the approach ends within 0.25 of the grab spot).
    pub latched: bool,
    /// The block position on the last tick of the grab.
    pub at: Option<Vec3>,
}

/// The block clips: still, pull, push, and the two side moves.
pub const ANIM_STILL: u32 = 40;
pub const ANIM_PULL: u32 = 41;
pub const ANIM_PUSH: u32 = 42;
pub const ANIM_SIDE_POS: u32 = 144;
pub const ANIM_SIDE_NEG: u32 = 145;

/// The hero's playing clip for state 18: its id, phase, whole key of the cursor and key count.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HoldClip {
    pub action: u32,
    pub progress: f32,
    pub cursor: f32,
    pub keys: u32,
}

impl Hold {
    /// One tick of state 18. `stick` is the camera-relative stick direction scaled by its
    /// length (zero when centred).
    pub fn tick(&mut self, blocks: &mut [Block], hero: Vec3, facing: Vec3, stick: Vec3, clip: HoldClip) -> HoldStep {
        let Some(b) = blocks.get(self.block) else { return HoldStep::default() };
        let m = self.face as i32;
        let Some(normal) = b.direction(m) else { return HoldStep::default() };
        if !self.latched {
            // Approach: the grab spot at the centre minus the normal times (S / 2 + 1.5).
            let spot = sub(b.position, scale(normal, 0.5 * b.size + 1.5));
            let to = [spot[0] - hero[0], 0.0, spot[2] - hero[2]];
            let d = dot(to, to).sqrt();
            if d <= LATCH {
                self.latched = true;
            } else {
                let dir = scale(to, 1.0 / d);
                if dot(facing, dir) < 0.99 {
                    // The walk-turn request's clip is not given: the hero turns on the spot.
                    return HoldStep { request: 0, steer: Some((dir, APPROACH_TURN)), ..HoldStep::default() };
                }
                return HoldStep { request: 2, steer: Some((dir, 1.0)), walking: true, ..HoldStep::default() };
            }
        }
        // Grab: face the block at rate 1.0; the stick on the face normal and the side axis.
        let follow = self.at.map_or([0.0; 3], |at| [b.position[0] - at[0], 0.0, b.position[2] - at[2]]);
        self.at = Some(b.position);
        let mut step = HoldStep { steer: Some((normal, 1.0)), request: ANIM_STILL, follow, ..HoldStep::default() };
        let side_axis = b.direction((m + 1) % 4).unwrap_or([0.0; 3]);
        let (pp, sv) = (dot(stick, normal), dot(stick, side_axis));
        let stick_on = dot(stick, stick) > 0.0;
        let (request, d) = if !stick_on {
            (ANIM_STILL, -1)
        } else if pp.abs() >= sv.abs() {
            if pp < 0.0 || clip.action == ANIM_PULL { (ANIM_PULL, (m + 2) % 4) } else { (ANIM_PUSH, m) }
        } else if sv > 0.0 {
            (ANIM_SIDE_POS, (m + 1) % 4)
        } else {
            (ANIM_SIDE_NEG, (m + 3) % 4)
        };
        // 41 and 42 return to 40 at progress 0.95, so the next cycle starts the clip again.
        step.request = if matches!(clip.action, ANIM_PULL | ANIM_PUSH) && clip.progress >= 0.95 { ANIM_STILL } else { request };
        // The move: once per clip cycle, while the playing clip is the move's and its phase is
        // strictly between 0.05 and 0.15; n = keys - whole cursor - 2 (doubled for the push).
        if d >= 0 && clip.action == request && clip.progress > 0.05 && clip.progress < 0.15 {
            let mut n = clip.keys as i32 - clip.cursor.floor() as i32 - 2;
            if request == ANIM_PUSH {
                n = (n as f32 / 0.5) as i32;
            }
            step.pushed = push_start(blocks, self.block, d, m, n);
        }
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AXES: [Vec3; 3] = [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

    fn block(x: f32, z: f32, s: f32) -> Block {
        Block::new(s, s, [x, 0.0, z], AXES, [s, s, s], |_, _, _| Some(0.0))
    }

    #[test]
    fn placement_and_push() {
        // Pack 1394's cube (S 5.452): L 2.726; the spec's push of 38 ticks moves 0.0717 a tick.
        let mut v = vec![block(0.0, 0.0, 5.452)];
        assert_eq!(v[0].position, [2.726, 0.0, 2.726]);
        assert!(push_start(&mut v, 0, 0, 0, 38));
        assert!(!push_start(&mut v, 0, 0, 0, 38), "a moving block refuses");
        let start = v[0].position;
        for _ in 0..38 {
            assert!(v[0].tick());
        }
        assert!((v[0].position[2] - start[2] - 2.726).abs() < 1e-4);
        assert!((2.726f32 / 38.0 - 0.0717).abs() < 1e-4);
        assert!(!v[0].tick());
        assert!(!v[0].moving, "the flag clears on the tick after the n-th step");
        // The box follows: its origin is the corner plus the displacement.
        assert!((v[0].shape().origin[2] - 2.726).abs() < 1e-4);
    }

    #[test]
    fn clearance() {
        // A block of size 2 at z 2.4 ahead (+A = +Z) blocks a push of the first (2.4 <= 2.5).
        let mut v = vec![block(0.0, 0.0, 2.0), block(0.0, 2.4, 2.0)];
        assert!(!push_start(&mut v, 0, 0, 0, 10));
        // At 2.6 the push passes, but a pull from face 2 looks 1.5 x 2.5 = 3.75 ahead.
        v[1] = block(0.0, 2.6, 2.0);
        assert!(push_start(&mut v, 0, 0, 0, 10));
        v[0] = block(0.0, 0.0, 2.0);
        assert!(!push_start(&mut v, 0, 0, 2, 10));
        // A block behind does not stop a push forward, and n of 0 never moves.
        let mut w = vec![block(0.0, 0.0, 2.0), block(0.0, -2.4, 2.0)];
        assert!(push_start(&mut w, 0, 0, 0, 0));
        assert!(!w[0].tick());
        assert!(!w[0].moving);
    }

    #[test]
    fn face_finder() {
        // The spec's vector: size 2.0, the hero 2.9 from the centre along A: face 0; 3.1 fails.
        let b = block(-1.0, -1.0, 2.0);
        assert_eq!(b.position, [0.0, 0.0, 0.0]);
        assert_eq!(face(&b, 2.9, [0.0, 0.0, -2.9], [0.0, 0.0, 1.0]), Some(0));
        assert_eq!(face(&b, 3.1, [0.0, 0.0, -3.1], [0.0, 0.0, 1.0]), None);
        // Offset (1.0, 0, 1.1) from the centre: B dominant (1.1 / 1.487 > 0.5), face 1.
        let hero = [-1.1, 0.0, -1.0];
        let d = (1.1f32).hypot(1.0);
        assert_eq!(face(&b, d, hero, [1.0, 0.0, 0.0]), Some(1));
        // Facing away fails.
        assert_eq!(face(&b, d, hero, [-1.0, 0.0, 0.0]), None);
    }

    #[test]
    fn hold_pushes_in_the_window() {
        let mut v = vec![block(-1.0, -1.0, 2.0)];
        let mut h = Hold { block: 0, face: 0, latched: false, at: None };
        // From afar the hero walks to the grab spot (z = -2.5).
        let s = h.tick(&mut v, [0.0, 0.0, -6.0], [0.0, 0.0, 1.0], [0.0; 3], HoldClip::default());
        assert!(s.walking && s.request == 2);
        let s = h.tick(&mut v, [0.0, 0.0, -2.6], [0.0, 0.0, 1.0], [0.0; 3], HoldClip::default());
        assert!(h.latched && s.request == ANIM_STILL);
        // The stick forward asks for the push; inside the window (key count 22, cursor 1.5) the
        // push starts with n = (22 - 1 - 2) / 0.5 = 38.
        let clip = HoldClip { action: ANIM_PUSH, progress: 1.5 / 22.0, cursor: 1.5, keys: 22 };
        let s = h.tick(&mut v, [0.0, 0.0, -2.5], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], clip);
        assert!(s.pushed && s.request == ANIM_PUSH);
        assert_eq!((v[0].n, v[0].dir), (38, 0));
        // The pull (key count 26, cursor 1.5) is not doubled: n = 23, toward -A.
        let mut v = vec![block(-1.0, -1.0, 2.0)];
        let clip = HoldClip { action: ANIM_PULL, progress: 1.5 / 26.0, cursor: 1.5, keys: 26 };
        let s = h.tick(&mut v, [0.0, 0.0, -2.5], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], clip);
        assert!(s.pushed && s.request == ANIM_PULL);
        assert_eq!((v[0].n, v[0].dir), (23, 2));
        // Outside the window nothing starts.
        let mut v = vec![block(-1.0, -1.0, 2.0)];
        let clip = HoldClip { action: ANIM_PUSH, progress: 3.5 / 22.0, cursor: 3.5, keys: 22 };
        assert!(!h.tick(&mut v, [0.0, 0.0, -2.5], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], clip).pushed);
    }

    #[test]
    fn contact_pushes_out() {
        let v = vec![block(0.0, 0.0, 2.0)];
        let mut p = [1.0, 0.5, -0.5];
        let c = contact(&v, &mut p);
        assert!(c.hit && !c.floor && c.side_touch);
        assert!(p[2] <= -0.75 + 1e-4, "pushed out of the near face: {p:?}");
        // Standing on the top (2.0): pushed up to just under the bound, the floor flag set, and
        // again on the next tick.
        let mut p = [1.0, 1.85, 1.0];
        let c = contact(&v, &mut p);
        assert!(c.floor && (p[1] - 1.89).abs() < 1e-4, "{p:?}");
        assert!(contact(&v, &mut p).floor);
    }

    #[test]
    fn fit_and_edge_states() {
        // The spec's fit vector: pack 909's block (size 2.759, height 2.741), the hero on the
        // floor at its base, face 0, stick forward 0.8 and side 0.1: passes; -0.8 or side 0.9
        // fail. A 5.452 cube is too tall.
        let b = Block::new(2.759, 2.741, [0.0; 3], AXES, [2.741, 2.759, 2.759], |_, _, _| Some(0.0));
        assert!(fits(&b, 0, [1.38, 0.0, -1.0], [0.0, 0.0, 1.0], 0.8, 0.1));
        assert!(!fits(&b, 0, [1.38, 0.0, -1.0], [0.0, 0.0, 1.0], -0.8, 0.1));
        assert!(!fits(&b, 0, [1.38, 0.0, -1.0], [0.0, 0.0, 1.0], 0.8, 0.9));
        let cube = block(0.0, 0.0, 5.452);
        assert!(!fits(&cube, 0, [2.7, 0.0, -1.0], [0.0, 0.0, 1.0], 0.8, 0.1));
        // The hang: grab (72), the approach to the edge point, 71, the hang idle 73, the pull-up
        // 59 and the exit to state 1 with 0.1.
        let v = vec![block(-1.0, -1.0, 2.0)];
        let mut h = Hang::new(0, 0);
        assert_eq!(h.tick(&v, [0.0, 0.0, -3.0], 0.0, false, 0.0, EdgeClip::default()).request, 72);
        let s = h.tick(&v, [0.0, 0.0, -1.35], 0.0, false, 0.0, EdgeClip { action: 72, progress: 0.5 });
        assert_eq!((h.sub, s.request, s.place), (2, 71, Some([0.0, -1.7, -1.35])));
        h.tick(&v, [0.0, -1.7, -1.35], 0.0, false, 0.0, EdgeClip { action: 71, progress: 0.96 });
        assert_eq!(h.sub, 3);
        assert_eq!(h.tick(&v, [0.0, -1.7, -1.35], 0.0, true, 0.0, EdgeClip { action: 73, progress: 0.5 }).request, 59);
        let s = h.tick(&v, [0.0, 2.0, -0.4], 0.0, false, 0.0, EdgeClip { action: 59, progress: 0.96 });
        assert_eq!(s.exit, Some((1, 0.1)));
        // The climb: 70 to the target at the top less 1.2, then 134 and the exit.
        let mut c = Climb::new(0, 0);
        let s = c.tick(&v, [0.0, 0.8, -1.75], false, EdgeClip::default());
        assert_eq!((c.sub, s.request), (2, 134));
        assert_eq!(c.tick(&v, [0.0, 2.0, 0.5], true, EdgeClip { action: 134, progress: 0.96 }).exit, Some((4, 0.0)));
    }
}
