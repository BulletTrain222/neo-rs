//! Ledges (SPEC-0033 Behaviors 32, 37, 49 and 51; R-0080, R-0085): the edge acceptance test
//! that makes a frame record the hero's edge contact, the in-air ledge chooser and its classes,
//! the ledge-drop edge test, and the hero states 30 (hang from a ledge), 35 (climb up from a
//! fall), 37 (drop off an edge into a hang) and 47 (climb up while rising).
//!
//! Not built: the ladder check of state 30, the touched-flag branch of state 47 (the wall contact is
//! off in these states), the obstacle probe of sub-modes 1 and 2 of states 35 and 47 (taken as
//! passing), state 38 (the drop onto a climb panel), the busy gate of the drop test, and the edge
//! offer's replace rule beyond the facing test.

use crate::hero::Vec3;
use crate::motion::HERO_HEIGHT;

/// A frame record as the ledge rules read it (SPEC-0033 Behavior 37): the face plane (unit
/// normal toward the side the hero stands on, offset), the run plane, the top plane, the length
/// L, the height value H and the type byte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// The start point (+0x00): on the face and run planes, at the top edge.
    pub start: Vec3,
    pub face: [f32; 4],
    pub run: [f32; 4],
    pub top: [f32; 4],
    pub length: f32,
    pub height: f32,
    pub kind: i8,
}

fn plane(q: [f32; 4], p: Vec3) -> f32 {
    q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Edge {
    pub fn normal(&self) -> Vec3 {
        [self.face[0], self.face[1], self.face[2]]
    }

    /// h(p): the height at p's horizontal position where the top plane reaches H, less 0.25.
    pub fn h(&self, p: Vec3) -> f32 {
        let m = self.top;
        (self.height - m[0] * p[0] - m[2] * p[2] - m[3]) / m[1] - 0.25
    }

    /// The run-plane value s at p.
    pub fn along(&self, p: Vec3) -> f32 {
        plane(self.run, p)
    }

    /// An approach target (Behavior 51): the horizontal point P - n(n.P - (d - inset)) for the
    /// face plane n.x = d, at t + dy with t = h at that point.
    pub fn target(&self, p: Vec3, inset: f32, dy: f32) -> Vec3 {
        let n = self.normal();
        let k = n[0] * p[0] + n[2] * p[2] + self.face[3] + inset;
        let mut q = [p[0] - n[0] * k, 0.0, p[2] - n[2] * k];
        q[1] = self.h(q) + dy;
        q
    }

    /// The ledge-drop edge test (Behavior 49), gates 2 to 8, for a character at `pos` with
    /// velocity `v`, forward row `forward`, radius `r` and action state `state`. `Some(true)` is
    /// an accept (a negative type), `Some(false)` passes on to the drop rule, None rejects.
    pub fn drop_gates(&self, pos: Vec3, v: Vec3, forward: Vec3, r: f32, state: u8) -> Option<bool> {
        // 2. Reference point.
        let p: Vec3 = if self.kind < 0 {
            match state {
                1 => std::array::from_fn(|k| pos[k] + 0.5 * v[k]),
                2..=4 => std::array::from_fn(|k| pos[k] + 0.5 * forward[k]),
                _ => std::array::from_fn(|k| pos[k] - 0.5 * forward[k]),
            }
        } else {
            pos
        };
        // 3. Normal.
        let n = self.normal();
        if n[1].abs() > 0.2 || self.top[1] == 0.0 {
            return None;
        }
        // 4. Face side.
        let (a, b) = (plane(self.face, p), plane(self.face, pos));
        if !((a < 0.0 || b < 0.0) && (a.abs() < 0.3 || b.abs() < 0.3)) {
            return None;
        }
        // 5. Vertical window.
        let h = self.h(p);
        if !(h - 0.2 < pos[1] && pos[1] < h + 3.0) {
            return None;
        }
        // 6. Along the edge.
        let s = self.along(p);
        if !(s > 0.0 && s < self.length) {
            return None;
        }
        // 7. Look-ahead.
        let q: Vec3 = std::array::from_fn(|k| p[k] + 5.0 * v[k] / 30.0);
        let c = plane(self.face, q);
        if c.abs() >= 0.5 * r && c <= 0.0 {
            return None;
        }
        // 8. A negative type is accepted.
        Some(self.kind < 0)
    }

    /// The drop rule's probe point: the start point + 0.1n + (L / 2) times the run direction.
    pub fn drop_probe(&self) -> Vec3 {
        let n = self.normal();
        std::array::from_fn(|k| self.start[k] + 0.1 * n[k] + 0.5 * self.length * self.run[k])
    }

    /// The edge acceptance test (Behavior 37) with no bypass, for a hero at `pos` with its
    /// forward row (also taken as the locomotion direction), velocity `v`, radius `r` and
    /// action state `state`. The carried motion and the force are taken as zero.
    pub fn accepts(&self, pos: Vec3, forward: Vec3, v: Vec3, r: f32, state: u8) -> bool {
        // 1. Type.
        if self.kind == 3 || self.kind == 6 {
            return false;
        }
        // 2. Reference point.
        let p = if self.kind < 0 {
            let s = if (1..=4).contains(&state) { 0.5 } else { -0.5 };
            std::array::from_fn(|k| pos[k] + s * forward[k])
        } else {
            pos
        };
        // 3. Face tilt.
        let n = self.normal();
        if n[1].abs() > 0.2 || self.top[1] == 0.0 {
            return false;
        }
        // 4. Face distance.
        let a = plane(self.face, p);
        if a <= -0.15 || a.abs() >= 1.25 * r {
            return false;
        }
        // 5. Height.
        let d = self.h(pos) - pos[1];
        if !(d > 0.0 && d < 4.0) {
            return false;
        }
        // 6. Along the edge.
        let s = self.along(p);
        if !(s > 0.0 && s < self.length) {
            return false;
        }
        // 7. Facing.
        if dot(forward, n) >= -0.5 {
            return false;
        }
        // 8. Look-ahead: five ticks of the velocity.
        let q: Vec3 = std::array::from_fn(|k| p[k] + 5.0 * v[k] / 30.0);
        let c = plane(self.face, q);
        if c.abs() >= 0.5 * r && (c <= 0.0 || c >= a) {
            return false;
        }
        // 9. Type 0 or more.
        self.kind >= 0
    }
}

/// The ledge classes of the in-air chooser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Result 0: state 30.
    Hang,
    /// Result 1: state 47 while rising, 35 while falling.
    Climb,
}

/// The ledge-offer gate (Behavior 32) and the chooser (Behavior 51): t = h(p), y the feet
/// height, g the ground probe below the hero, `normal_y` the ground normal's vertical part.
pub fn choose(t: f32, y: f32, g: f32, normal_y: f32, jumping: bool) -> Option<Class> {
    let hh = HERO_HEIGHT;
    if normal_y < 0.85 || t <= y + if jumping { 0.0 } else { 0.75 * hh } {
        return None;
    }
    if t - g > hh + 0.75 {
        return (t <= y + hh).then_some(Class::Hang);
    }
    (t - g > 1.75 && t - 0.5 <= y + 0.85 * hh).then_some(Class::Climb)
}

pub const HANG: u8 = 30;
pub const CLIMB_FALL: u8 = 35;
pub const CLIMB_RISE: u8 = 47;
pub const DROP: u8 = 37;

/// The ledge-drop test's gate 9 (Behavior 49): g0 the character's ground, `g1` the probe past
/// the edge (height and ground kind; None for no ground).
pub fn drop_rule(g0: f32, g1: Option<(f32, Option<u32>)>) -> bool {
    match g1 {
        Some((_, Some(2))) => true,
        None => false,
        Some((g, _)) if g <= -1_000_000.0 || g >= g0 => false,
        Some((g, _)) => g0 - g > 1.75,
    }
}

/// The clip as the ledge states read it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub action: u32,
    pub progress: f32,
}

/// What a ledge state asks of the hero this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    /// The request (None keeps the current one).
    pub request: Option<u32>,
    /// The facing to steer toward at rate 0.5 (the negated face normal).
    pub facing: Option<Vec3>,
    pub place: Option<Vec3>,
    /// The state to leave to (1 or 4).
    pub exit: Option<u8>,
    /// The steering rate (0.5 when None).
    pub rate: Option<f32>,
    /// State 37's end: go to state 30 with this edge.
    pub hang: bool,
    /// The hang moved onto its next (true, link 1) or previous (false, link 3) edge.
    pub took: Option<bool>,
    /// The pull-up's hero voice cue 9 (quick exert) at the hero, do-not-restart (R-0193).
    pub voice: bool,
    /// State 35's push of 0.01 along the facing.
    pub push: bool,
    /// Phase 3 of state 37: with the weapon drawn, both item-manager flags and the refusal counter clear.
    pub sheathe: bool,
    /// Phase 0's landing exit arms the drop test's refusal counter (120 ticks).
    pub refuse: bool,
}

/// One ledge state with its held edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ledge {
    pub state: u8,
    /// The event bits of the hero clip's current frame (set by the host each tick).
    pub frame_bits: u32,
    pub edge: Edge,
    pub sub: u8,
    /// State 37: the walk state it was entered from.
    pub from: u8,
    /// The held edge's linked records (SPEC-0018 Climb and hang data), set by the caller each
    /// tick: link 1 (the next record, from its end) and link 3 (the previous one).
    pub next: Option<Edge>,
    pub prev: Option<Edge>,
}

/// The hang point's distance in front of the face (SPEC-0033 Behavior 51A item 6).
pub const HANG_FRONT: f32 = 0.475;
/// States 35 and 47 approach a point r − 0.5 = 0.675 in front of the face (Behavior 51).
pub const CLIMB_FRONT: f32 = 0.675;
/// A shimmy's along limit at an end that does not continue (Behavior 51A item 7).
pub const SHIMMY_LIMIT: f32 = 1.5;

/// A joint continues when the two normals have a dot product above 0.4.
/// The edge offer of the hero's wall contact (SPEC-0033 Behavior 51A item 2, Behavior 49): the
/// record touches the hero at `pos` (one-sided except type 6: in front within reach r, which
/// after the contact's push is a = r; −r ≤ s ≤ L + r; the wall span t − depth − H < y < t − 0.3)
/// and is offered when its type is not 3 or 6 and the hero is in its middle part (r < s < L − r)
/// or, outside it, its link 3 continues (`prev_continues`). The offer is ranked by
/// [`Edge::offer_score`]; the air chooser of the next tick reads the best one.
pub fn offered(e: &Edge, pos: Vec3, r: f32, prev_continues: bool) -> bool {
    if matches!(e.kind, 3 | 6) || e.normal()[1].abs() > 0.2 || e.top[1] == 0.0 || e.length <= 0.0 {
        return false;
    }
    let a = plane(e.face, pos);
    if !(a >= 0.0 && a <= r + 1e-3) {
        return false;
    }
    let s = e.along(pos);
    if !(-r..=e.length + r).contains(&s) {
        return false;
    }
    let t = e.h(pos);
    if !(pos[1] < t - 0.3 && pos[1] > t - e.height - HERO_HEIGHT) {
        return false;
    }
    (s > r && s < e.length - r) || prev_continues
}

/// The air chooser's class gates before the class test (Behavior 51A item 3): the hero faces into
/// the face within 60 degrees (forward · −n > 0.5) and his centre is in front of it (a > 0.25).
pub fn faces_into(e: &Edge, pos: Vec3, forward: Vec3) -> bool {
    -dot(forward, e.normal()) > 0.5 && plane(e.face, pos) > 0.25
}

pub fn continues(a: &Edge, b: Option<&Edge>) -> bool {
    b.is_some_and(|b| dot(a.normal(), b.normal()) > 0.4)
}

fn approach(hero: Vec3, target: Vec3, len: f32, snap: f32) -> (Vec3, bool) {
    let d: Vec3 = std::array::from_fn(|k| target[k] - hero[k]);
    let dist = dot(d, d).sqrt();
    if dist > snap {
        (std::array::from_fn(|k| hero[k] + d[k] * len / dist), false)
    } else {
        (target, true)
    }
}

impl Ledge {
    pub fn new(state: u8, edge: Edge) -> Self {
        Self { state, edge, sub: 0, from: 0, next: None, prev: None, frame_bits: 0 }
    }

    /// State 37 entered from walk state `from`.
    pub fn drop_from(edge: Edge, from: u8) -> Self {
        Self { state: DROP, edge, sub: 0, from, next: None, prev: None, frame_bits: 0 }
    }

    /// State 37 (Behavior 51), phases 0 to 3. `facing` is the hero's facing and `deep` the
    /// depth check's result (the ground 2.0 and 2.5 ahead more than 4.5 below, or none).
    /// `touched` is the motion record's touched flag and `action` the current clip's action. The
    /// touched flag ends phases 0 to 2 in state 1 (phase 0 arms the refusal counter). Phase 0: with
    /// action 2 to 6 a shallow edge returns to the state of that number (R-0193); else the hero
    /// turns out over the edge at rate 0.1. Phase 1: walk request 2 until 0.15 past the face.
    /// Phase 2: request 73 (action 1 instead, with no turn, while the ground gap `gap` is 0 or
    /// below), the facing blended 0.25 toward the inward face direction each tick (the shared
    /// steering blend), done when the updated facing is within 0.99 of it. Phase 3: state 30
    /// (`Step::hang`). Phase 1 faces out at rate 1.0; past 0.15 in front of the face it goes on
    /// to phase 2 when the gap is above 0.25, else to the slow walk (state 2). Phase 2 (Behavior
    /// 51, R-0134 update): with d = y − (t − H − 0.1), while d > −0.01 the hero is lowered 0.2 a
    /// tick (state 1 when the gap is then 0 or below); the turn to the wall blends 0.25 a tick and
    /// the phase ends only on a tick with no lowering and the facing within 0.99 of −n. (Setting
    /// the height exactly would keep d at 0 and never end the phase.)
    pub fn drop_tick(&mut self, hero: Vec3, facing: Vec3, deep: bool, gap: f32, touched: bool, action: u32) -> Step {
        let n = self.edge.normal();
        let (out, inward) = ([n[0], 0.0, n[2]], [-n[0], 0.0, -n[2]]);
        let mut step = Step { rate: Some(0.1), ..Step::default() };
        if touched && self.sub <= 2 {
            step.refuse = self.sub == 0;
            step.exit = Some(1);
            return step;
        }
        match self.sub {
            0 => {
                if (2..=6).contains(&action) && !deep {
                    step.exit = Some(action as u8);
                } else {
                    step.facing = Some(out);
                    step.request = Some(0);
                    self.sub = 1;
                }
            }
            1 => {
                step.facing = Some(out);
                step.rate = Some(1.0);
                step.request = Some(2);
                let a = plane(self.edge.face, hero);
                if a > 0.15 {
                    if gap > 0.25 {
                        // The neighbouring-edge walk: the face distance past this edge's length
                        // hands the hold to link 1, with no dot product test (R-0193).
                        if self.edge.length < a {
                            match self.next {
                                Some(e) => {
                                    self.edge = e;
                                    step.took = Some(true);
                                }
                                None => {
                                    step.exit = Some(1);
                                    return step;
                                }
                            }
                        }
                        self.sub = 2;
                    } else {
                        step.exit = Some(2);
                    }
                }
            }
            2 => {
                step.request = Some(73);
                let d = hero[1] - (self.edge.h(hero) - HERO_HEIGHT - 0.1);
                let lowered = d > -0.01;
                if lowered {
                    step.place = Some([hero[0], hero[1] - 0.2, hero[2]]);
                    if gap - 0.2 <= 0.0 {
                        step.exit = Some(1);
                        return step;
                    }
                }
                step.facing = Some(inward);
                step.rate = Some(0.25);
                if !lowered && dot(crate::hero::steer(facing, inward, 0.25), inward) > 0.99 {
                    self.sub = 3;
                }
            }
            _ => {
                step.hang = true;
                step.sheathe = true;
            }
        }
        step
    }

    /// One tick. `side` is the stick's side value (negative to the hero's right), `forward`
    /// whether the stick is pushed forward and dominant, `gap` the ground gap.
    pub fn tick(&mut self, hero: Vec3, side: f32, forward: bool, gap: f32, clip: Clip) -> Step {
        let n = self.edge.normal();
        let mut step = Step { facing: Some([-n[0], 0.0, -n[2]]), ..Step::default() };
        match self.state {
            HANG => self.hang(&mut step, hero, side, forward, gap, clip),
            CLIMB_FALL => {
                let target = self.edge.target(hero, -CLIMB_FRONT, -0.75);
                self.climb(&mut step, hero, target, 0.25, 0.25, 75, forward, clip);
            }
            _ => {
                let target = self.edge.target(hero, -CLIMB_FRONT, -1.1);
                self.climb(&mut step, hero, target, 0.2, 0.2, 134, forward, clip);
                // A non-positive gap ends the approach.
                if self.sub < 2 && gap <= 0.0 {
                    self.sub = 2;
                    step.request = Some(134);
                }
            }
        }
        step
    }

    /// States 35 and 47: 0 the entry (state 47 approaches in it), 1 the approach (`len` a tick, snapping within `snap`)
    /// while the clip plays, 2 the clip to 0.95.
    #[allow(clippy::too_many_arguments)]
    fn climb(&mut self, step: &mut Step, hero: Vec3, target: Vec3, len: f32, snap: f32, anim: u32, forward: bool, clip: Clip) {
        match self.sub {
            // State 35's entry; state 47 approaches from its entry pass.
            0 if self.state == CLIMB_FALL => {
                self.sub = 1;
                step.request = Some(anim);
            }
            0 | 1 => {
                step.request = Some(anim);
                if clip.action != anim && self.state == CLIMB_FALL {
                    return;
                }
                let (p, snapped) = approach(hero, target, len, snap);
                if !snapped && clip.progress < 0.99 {
                    step.place = Some(p);
                } else {
                    step.place = Some(target);
                    self.sub = 2;
                }
            }
            // Sub-mode 2 (R-0193): the clip's action test, 0.95 ends in state 1; before that, state 35
            // pushes 0.01 along the facing from progress 0.75, and event bit 0x40 with the stick
            // forward runs out to state 4.
            _ => {
                if clip.action != anim {
                    step.request = Some(anim);
                } else if clip.progress >= 0.95 {
                    step.exit = Some(1);
                } else {
                    if self.state == CLIMB_FALL && clip.progress >= 0.75 {
                        step.push = true;
                    }
                    if forward && self.frame_bits & 0x40 != 0 {
                        step.exit = Some(4);
                    }
                }
            }
        }
    }

    /// State 30 (Behavior 51A): 0 the grab (72), 1 the move to the hang point (0.25 a tick
    /// toward 0.475 in front of the face at t − H − 0.1, set there within 0.25 or at progress
    /// 0.99; then 71), 2 (73), 3 the hang: the hold (0.475 in front, feet t − H + 0.1) and,
    /// between steps, the stick: toward the wall the pull-up (not on type 5), toward +run (side
    /// negative) a step of 61, toward −run (side positive) a step of 60. A step reaching 1.5
    /// from an end goes on round a continuing joint (sub-mode 6 toward +run, 5 toward −run, no
    /// hold) and is cut at an end that does not continue; in 5 and 6 the held edge becomes the
    /// linked one on the first between-step tick past the joint. 4 the pull-up to progress 0.95.
    /// Every sub-mode faces the wall. The ground under the hanging hero reaching his feet ends
    /// the hang (state 1). Not built:
    /// the obstacle probe, the pull-up's early run-out from frame 37.
    fn hang(&mut self, step: &mut Step, hero: Vec3, side: f32, forward: bool, gap: f32, clip: Clip) {
        if gap <= 0.0 && matches!(self.sub, 3 | 5 | 6) {
            step.exit = Some(1);
            return;
        }
        // Every sub-mode faces the wall (the held edge's −n).
        let n = self.edge.normal();
        step.facing = Some([-n[0], 0.0, -n[2]]);
        let s = self.edge.along(hero);
        let l = self.edge.length;
        let pull = forward && self.edge.kind != 5;
        let stepping = matches!(clip.action, 60 | 61) && clip.progress < 0.95;
        let (on_next, on_prev) = (continues(&self.edge, self.next.as_ref()), continues(&self.edge, self.prev.as_ref()));
        match self.sub {
            0 => {
                self.sub = 1;
                step.request = Some(72);
            }
            1 => {
                step.request = Some(72);
                if clip.action != 72 {
                    return;
                }
                let target = self.edge.target(hero, -HANG_FRONT, -0.1 - HERO_HEIGHT);
                let (p, snapped) = approach(hero, target, 0.25, 0.25);
                if !snapped && clip.progress < 0.99 {
                    step.place = Some(p);
                } else {
                    step.place = Some(target);
                    self.sub = 2;
                    step.request = Some(71);
                }
            }
            2 => {
                self.sub = 3;
                step.request = Some(73);
            }
            3 => {
                step.place = Some(self.edge.target(hero, -HANG_FRONT, 0.1 - HERO_HEIGHT));
                if stepping {
                    if clip.action == 61 && s >= l - SHIMMY_LIMIT {
                        if on_next {
                            self.sub = 6;
                        } else {
                            step.request = Some(73);
                        }
                    } else if clip.action == 60 && s <= SHIMMY_LIMIT {
                        if on_prev {
                            self.sub = 5;
                        } else {
                            step.request = Some(73);
                        }
                    }
                } else if pull {
                    self.sub = 4;
                    step.request = Some(59);
                    step.voice = clip.action != 59;
                } else if side < 0.0 && (s < l - SHIMMY_LIMIT || on_next) {
                    step.request = Some(61);
                } else if side > 0.0 && (s > SHIMMY_LIMIT || on_prev) {
                    step.request = Some(60);
                } else {
                    step.request = Some(73);
                }
            }
            // The pull-up: while the clip runs, event bit 0x40 with the stick forward runs out to
            // state 4; at 0.95 it ends in state 1 (SPEC-0033 sub-mode 4).
            4 => {
                step.request = Some(59);
                step.voice = clip.action != 59;
                if clip.action == 59 && clip.progress < 0.95 && forward && self.frame_bits & 0x40 != 0 {
                    step.exit = Some(4);
                } else if clip.action == 59 && clip.progress >= 0.95 {
                    step.exit = Some(1);
                }
            }
            sub => {
                // 6 goes on toward +run onto link 1, 5 toward −run onto link 3.
                let ahead = if sub == 6 { self.next } else { self.prev };
                if stepping {
                    return;
                }
                if let Some(n) = ahead {
                    let past = if sub == 6 { n.along(hero) > 0.0 } else { n.along(hero) < n.length };
                    if past {
                        self.edge = n;
                        self.sub = 3;
                        step.took = Some(sub == 6);
                        step.request = Some(73);
                        return;
                    }
                }
                if pull {
                    self.sub = 4;
                    step.request = Some(59);
                } else if (sub == 6 && side < 0.0) || (sub == 5 && side > 0.0) {
                    step.request = Some(if sub == 6 { 61 } else { 60 });
                } else if side != 0.0 {
                    // Back the other way on the same edge.
                    self.sub = 3;
                    step.request = Some(if side < 0.0 { 61 } else { 60 });
                } else {
                    step.request = Some(73);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A face plane x = 0 facing +x, running along z, flat top at height 10 (top plane y with
    /// H = 10).
    fn edge() -> Edge {
        Edge { start: [0.0, 10.0, 0.0], face: [1.0, 0.0, 0.0, 0.0], run: [0.0, 0.0, 1.0, 0.0], top: [0.0, 1.0, 0.0, 0.0], length: 4.0, height: 10.0, kind: 0 }
    }

    #[test]
    fn chooser_boundaries() {
        // Vector 181: H 3.5, y 1.0, g 0.0, not jumping.
        assert_eq!(choose(3.625, 1.0, 0.0, 1.0, false), None);
        assert_eq!(choose(3.626, 1.0, 0.0, 1.0, false), Some(Class::Climb));
        assert_eq!(choose(4.25, 1.0, 0.0, 1.0, false), Some(Class::Climb));
        assert_eq!(choose(4.2501, 1.0, 0.0, 1.0, false), Some(Class::Hang));
        assert_eq!(choose(4.5001, 1.0, 0.0, 1.0, false), None);
        // The 0.85 ground-normal cutoff; jumping drops the 0.75H term.
        assert_eq!(choose(4.25, 1.0, 0.0, 0.84, false), None);
        assert_eq!(choose(2.0, 1.0, 0.0, 1.0, true), Some(Class::Climb));
    }

    #[test]
    fn edge_relative_targets() {
        // Vector 182.
        let e = edge();
        let p = [1.5, 0.0, 0.0];
        assert!((e.h(p) - 9.75).abs() < 1e-6);
        // The hang hold 0.475 in front of the face at t − H + 0.1; states 35 and 47 0.675 in
        // front at t − 0.75 and t − 1.1 (Behaviors 51 and 51A).
        let t30 = e.target(p, -HANG_FRONT, 0.1 - HERO_HEIGHT);
        assert!((t30[0] - 0.475).abs() < 1e-6 && (t30[1] - 6.35).abs() < 1e-5);
        let t35 = e.target(p, -CLIMB_FRONT, -0.75);
        assert!((t35[0] - 0.675).abs() < 1e-6 && (t35[1] - 9.0).abs() < 1e-5);
        let t47 = e.target(p, -CLIMB_FRONT, -1.1);
        assert!((t47[0] - 0.675).abs() < 1e-6 && (t47[1] - 8.65).abs() < 1e-5);
    }

    #[test]
    fn ledge_drop_test() {
        // The top of the face x = 0 at 9.75 (h); a hero walking out (+x) at x -0.1 on the top.
        let e = edge();
        let out = [1.0, 0.0, 0.0];
        assert_eq!(e.drop_gates([-0.1, 9.8, 2.0], [3.0, 0.0, 0.0], out, 1.175, 3), Some(false));
        // Too far behind the face, below the window, past the run's end: rejected.
        assert_eq!(e.drop_gates([-0.5, 9.8, 2.0], [3.0, 0.0, 0.0], out, 1.175, 3), None);
        assert_eq!(e.drop_gates([-0.1, 9.5, 2.0], [3.0, 0.0, 0.0], out, 1.175, 3), None);
        assert_eq!(e.drop_gates([-0.1, 9.8, 4.5], [3.0, 0.0, 0.0], out, 1.175, 3), None);
        // A negative type is accepted at once.
        assert_eq!(Edge { kind: -1, ..e }.drop_gates([-0.1, 9.8, 2.0], [3.0, 0.0, 0.0], out, 1.175, 3), Some(true));
        // The probe point and the drop rule.
        assert_eq!(e.drop_probe(), [0.1, 10.0, 2.0]);
        assert!(drop_rule(9.8, Some((5.0, None))));
        assert!(!drop_rule(9.8, Some((8.1, None))));
        assert!(!drop_rule(9.8, Some((9.9, None))));
        assert!(!drop_rule(9.8, None));
        assert!(drop_rule(9.8, Some((9.9, Some(2)))));
        // State 37 from a slow walk: a shallow edge returns to the walk; a deep one goes out past
        // the face, turns back and hangs.
        let mut d = Ledge::drop_from(e, 2);
        assert_eq!(d.drop_tick([-0.1, 9.8, 2.0], out, false, 5.0, false, 2).exit, Some(2));
        let mut d = Ledge::drop_from(e, 2);
        d.drop_tick([-0.1, 9.8, 2.0], out, true, 5.0, false, 2);
        assert_eq!(d.drop_tick([0.1, 9.8, 2.0], out, true, 5.0, false, 2).request, Some(2));
        d.drop_tick([0.2, 9.8, 2.0], out, true, 5.0, false, 2);
        assert_eq!(d.sub, 2);
        // Phase 2 on the ground: the lowering puts the feet on it, state 1.
        let s0 = d.clone().drop_tick([0.2, 9.8, 2.0], out, true, 0.0, false, 2);
        assert_eq!(s0.exit, Some(1));
        // Phase 2 in the air: 0.2 a tick down until the feet are 0.01 to 0.21 below
        // t − H − 0.1 = 6.15, turning meanwhile; it ends only on a tick with no lowering.
        let (mut hero, mut facing) = ([0.2f32, 9.8, 2.0], out);
        let mut ticks = 0;
        loop {
            let s = d.drop_tick(hero, facing, true, 5.0, false, 2);
            if let Some(p) = s.place {
                hero = p;
            }
            if let Some(f) = s.facing {
                facing = crate::hero::steer(facing, f, s.rate.unwrap_or(0.5));
            }
            ticks += 1;
            if s.hang || ticks > 60 {
                break;
            }
        }
        assert!(ticks < 60, "phase 2 never ended");
        assert!(hero[1] < 6.15 - 0.01 && hero[1] >= 6.15 - 0.21, "{}", hero[1]);
    }

    #[test]
    fn shimmy_follows_continuing_links_and_stops_at_corners() {
        // Edge along +z from z 0 to 4; the next edge carries on from z 4 (a 0.9 dot turn), the
        // previous one meets it at a right angle (dot 0).
        let e = edge();
        let next = Edge { start: [0.0, 10.0, 4.0], face: [0.9, 0.0, -0.436, 1.744], run: [0.436, 0.0, 0.9, -3.6], ..e };
        let prev = Edge { start: [0.0, 10.0, 0.0], face: [0.0, 0.0, -1.0, 0.0], run: [1.0, 0.0, 0.0, 0.0], ..e };
        let mut l = Ledge { sub: 3, next: Some(next), prev: Some(prev), ..Ledge::new(HANG, e) };
        let hang = |z: f32| [0.475, 6.35, z];
        // Toward +run (side negative) from the middle: a step of 61.
        assert_eq!(l.tick(hang(2.0), -1.0, false, 5.0, Clip { action: 73, progress: 0.0 }).request, Some(61));
        // A step reaching 1.5 from the end goes on round the continuing joint (sub-mode 6).
        l.tick(hang(2.6), -1.0, false, 5.0, Clip { action: 61, progress: 0.5 });
        assert_eq!(l.sub, 6);
        // Past the joint, between steps: the held edge is the next one.
        let s = l.tick([0.6, 6.35, 4.3], -1.0, false, 5.0, Clip { action: 61, progress: 0.96 });
        assert_eq!((s.took, l.sub, l.edge), (Some(true), 3, next));
        // Toward −run near the start, the right-angle corner cuts the step and starts none.
        let mut l = Ledge { sub: 3, next: Some(next), prev: Some(prev), ..Ledge::new(HANG, e) };
        assert_eq!(l.tick(hang(1.2), 1.0, false, 5.0, Clip { action: 60, progress: 0.4 }).request, Some(73));
        assert_eq!(l.tick(hang(1.2), 1.0, false, 5.0, Clip { action: 73, progress: 0.0 }).request, Some(73));
        assert_eq!(l.sub, 3);
    }

    #[test]
    fn acceptance_gates() {
        let e = edge();
        // Facing the wall from 1.0 out, the top 9.75 - 7 = 2.75 above the feet.
        let into = [-1.0, 0.0, 0.0];
        // Moving into it (the look-ahead c 1.0 - 0.5 is inside 0.5r); at rest c = a rejects.
        assert!(e.accepts([1.0, 7.0, 2.0], into, [-3.0, 0.0, 0.0], 1.25, 7));
        assert!(!e.accepts([1.0, 7.0, 2.0], into, [0.0; 3], 1.25, 7));
        // Facing along it, past its end, too far below, a type-3 edge: rejected.
        assert!(!e.accepts([1.0, 7.0, 2.0], [0.0, 0.0, 1.0], [0.0; 3], 1.25, 7));
        assert!(!e.accepts([1.0, 7.0, 5.0], into, [0.0; 3], 1.25, 7));
        assert!(!e.accepts([1.0, 5.0, 2.0], into, [0.0; 3], 1.25, 7));
        assert!(!Edge { kind: 3, ..e }.accepts([1.0, 7.0, 2.0], into, [0.0; 3], 1.25, 7));
    }

    #[test]
    fn hang_reaches_its_target_and_pulls_up() {
        let mut l = Ledge::new(HANG, edge());
        let mut hero = [1.5, 6.0, 2.0];
        let s = l.tick(hero, 0.0, false, 5.0, Clip::default());
        assert_eq!((s.request, l.sub), (Some(72), 1));
        for _ in 0..40 {
            let s = l.tick(hero, 0.0, false, 5.0, Clip { action: 72, progress: 0.5 });
            hero = s.place.unwrap_or(hero);
            if l.sub == 2 {
                break;
            }
        }
        assert_eq!(l.sub, 2);
        // The approach point: 0.475 in front of the face at t − H − 0.1 = 6.15.
        assert!((hero[0] - 0.475).abs() < 1e-5 && (hero[1] - 6.15).abs() < 1e-5);
        l.tick(hero, 0.0, false, 5.0, Clip { action: 71, progress: 0.0 });
        let s = l.tick(hero, 0.0, true, 5.0, Clip { action: 73, progress: 0.0 });
        assert_eq!((s.request, l.sub), (Some(59), 4));
        let s = l.tick(hero, 0.0, false, 5.0, Clip { action: 59, progress: 0.96 });
        assert_eq!(s.exit, Some(1));
    }
}
