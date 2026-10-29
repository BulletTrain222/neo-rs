//! The Rope Gun (item 36): the right-stick aim mode (SPEC-0036 Right-stick aim mode), the aim
//! scan over the world's aim targets (SPEC-0030 row 126; SPEC-0033 State 27), the rope manager
//! and hero state 27's phases 0 to 4 (SPEC-0033 State 27).
//!
//! Stand-ins (the spec leaves these open): the rope's outbound travel moves the endpoint
//! `OUT_STEP` a tick straight toward its target and attaches on arrival (the spec's scalar
//! update and its W-lane attach test are not established in world units); phases 3 and 4 hold
//! the hero without gravity (the spec adds the pulls to the hero's motion and names no gravity
//! rule for them). Not built: the reticle and the aim view object (whose marker model is absent
//! from the disc), the behaviour-record release gate, cue 185's span gate, the character aim
//! candidates of the scan (the shot aim point is kept by the existing shot finder).

pub type Vec3 = [f32; 3];

/// The scan's range (strictly below), best-score threshold (strictly above) and the sight
/// sweep's drop below the camera eye.
pub const RANGE: f32 = 110.0;
pub const SCORE: f32 = 0.998;
pub const SIGHT_DROP: f32 = 1.5;
/// The rope's stored target: the anchor pulled back toward the muzzle by 0.935.
pub const TARGET_BACK: f32 = 0.935;
/// The reel pull per tick, the arrival distance |Q - M|, phase 4's aim drop, its vertical gate
/// and its pull.
pub const REEL: f32 = 0.8;
pub const ARRIVE: f32 = 3.0;
pub const HANG_DROP: f32 = 2.5;
pub const HANG_GATE: f32 = 0.35;
pub const HANG_PULL: f32 = 0.15;
/// This build's outbound step a tick (stand-in, see the module note).
pub const OUT_STEP: f32 = 4.0;
/// The hero state and the item.
pub const STATE: u8 = 27;
pub const ITEM: u16 = 36;
/// The clips of the phases and the cues (main bank).
pub const FIRE_CLIP: u32 = 167;
pub const ATTACHED_CLIP: u32 = 168;
pub const HANG_CLIP: u32 = 171;
pub const REEL_CLIP: u32 = 172;
pub const CUE_FIRE: i32 = 183;
pub const CUE_ATTACH: i32 = 184;

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}
fn unit(a: Vec3) -> Vec3 {
    let l = len(a);
    if l > 1e-6 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0; 3] }
}

/// One aim target of the world (built from a type-11 n2 entry; the active word starts at 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub point: Vec3,
    pub active: bool,
}

/// The anchor pass of the scan: among the active targets within `RANGE` of the camera eye,
/// the best score dot(unit(A - C), F) above `SCORE` whose sight line `clear(i)` passes.
pub fn scan(targets: &[Target], eye: Vec3, forward: Vec3, clear: impl Fn(usize) -> bool) -> Option<usize> {
    let f = unit(forward);
    let mut best: Option<(usize, f32)> = None;
    for (i, t) in targets.iter().enumerate() {
        if !t.active {
            continue;
        }
        let d = sub(t.point, eye);
        if len(d) >= RANGE {
            continue;
        }
        let s = dot(unit(d), f);
        if s > SCORE && best.is_none_or(|(_, b)| s > b) && clear(i) {
            best = Some((i, s));
        }
    }
    best.map(|(i, _)| i)
}

/// The sight sweep's sample distances along the line (SPEC-0030 row 126; R-0121): 0, 5, 10, 15
/// and every further 5 units, stopping when the offset reaches `length`.
pub fn sight_samples(length: f32) -> Vec<f32> {
    let (mut d, mut out) = (0.0f32, Vec::new());
    while d < length {
        out.push(d);
        d += 5.0;
    }
    out
}

/// The hero's aim mode word, its counter and the retained R2/Circle choice.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AimMode {
    pub on: bool,
    pub counter: u32,
    pub choice: bool,
}

impl AimMode {
    /// One tick: `state` the hero state, `attack_sub` the attack step's sub-state in state 13,
    /// `y` the right stick's converted Y (0 inside the dead zone). Returns whether the scan runs
    /// this tick (aim mode on in state 1 or 13).
    pub fn step(&mut self, state: u8, attack_sub: Option<u8>, y: f32, r2: bool, circle: bool) -> bool {
        if state == 13 && (r2 || circle) {
            self.choice = r2;
        }
        if !self.on {
            let attack_ok = state == 13 && matches!(attack_sub, Some(1 | 9 | 14)) && self.choice;
            if y != 0.0 && (state == 1 || attack_ok) {
                self.on = true;
                self.counter = 0;
            }
            return false;
        }
        self.counter += 1;
        if state != 1 && state != 13 {
            self.on = false;
            return false;
        }
        true
    }
}

/// The rope manager's states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RopeState {
    #[default]
    Idle,
    Out,
    Attached,
}

/// The world's single rope manager: the selected anchor, its point A, the stored target G and
/// the endpoint Q.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rope {
    pub state: RopeState,
    pub anchor: Option<usize>,
    pub a: Vec3,
    pub g: Vec3,
    pub q: Vec3,
}

impl Rope {
    /// The scan's anchor pass selects into an empty (idle) manager.
    pub fn select(&mut self, anchor: Option<(usize, Vec3)>) {
        if self.state == RopeState::Idle {
            self.anchor = anchor.map(|a| a.0);
            if let Some((_, a)) = anchor {
                self.a = a;
            }
        }
    }

    /// Fire from the muzzle M: G = A - 0.935 unit(A - M), the endpoint at M.
    pub fn fire(&mut self, m: Vec3) {
        let d = unit(sub(self.a, m));
        self.g = sub(self.a, [TARGET_BACK * d[0], TARGET_BACK * d[1], TARGET_BACK * d[2]]);
        self.q = m;
        self.state = RopeState::Out;
    }

    /// One update with the current muzzle M: idle follows M; outbound moves toward G (stand-in
    /// step) and attaches there.
    pub fn update(&mut self, m: Vec3) {
        match self.state {
            RopeState::Idle => self.q = m,
            RopeState::Out => {
                let d = sub(self.g, self.q);
                if len(d) <= OUT_STEP {
                    self.q = self.g;
                    self.state = RopeState::Attached;
                } else {
                    let u = unit(d);
                    self.q = [self.q[0] + OUT_STEP * u[0], self.q[1] + OUT_STEP * u[1], self.q[2] + OUT_STEP * u[2]];
                }
            }
            RopeState::Attached => {}
        }
    }

    /// Arrival: |Q - M| at most 3.0.
    pub fn arrived(&self, m: Vec3) -> bool {
        len(sub(self.q, m)) <= ARRIVE
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// What state 27 reads each tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct View {
    pub p: Vec3,
    /// The muzzle M = T + 0.37F - S of weapon attach frame 0.
    pub m: Vec3,
    pub clip: (u32, f32),
    pub frame_bits: u32,
    /// The touched flag (+0x238) of the last motion step.
    pub touched: bool,
    pub square: bool,
}

/// What state 27 asks of the host this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    pub request: Option<u32>,
    pub face: Option<(Vec3, f32)>,
    pub push: Vec3,
    pub cue: Option<i32>,
    pub holster: bool,
    pub no_gravity: bool,
    /// Let go: state 1 and the manager reset.
    pub exit: bool,
}

/// Hero state 27: its phase and whether the rope has been fired.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RopeHero {
    pub phase: u8,
    pub fired: bool,
}

impl RopeHero {
    pub fn tick(&mut self, v: View, rope: &mut Rope) -> Step {
        let mut s = Step::default();
        let toward = unit(sub(rope.a, v.p));
        let flat = [toward[0], 0.0, toward[2]];
        match self.phase {
            0 => {
                s.holster = true;
                s.request = Some(FIRE_CLIP);
                self.phase = 1;
            }
            1 => {
                s.face = Some((flat, 0.25));
                s.request = Some(FIRE_CLIP);
                if !self.fired && v.clip.0 == FIRE_CLIP && v.frame_bits & 0x20 != 0 {
                    self.fired = true;
                    rope.fire(v.m);
                    s.cue = Some(CUE_FIRE);
                }
                if rope.state == RopeState::Attached {
                    s.cue = Some(CUE_ATTACH);
                    self.phase = 2;
                }
                // A clip that ends without the fire event leaves the state (this build's guard).
                if !self.fired && v.clip.0 == FIRE_CLIP && v.clip.1 >= 0.99 {
                    s.exit = true;
                }
            }
            2 => {
                s.face = Some((flat, 0.25));
                s.request = Some(ATTACHED_CLIP);
                if v.clip.0 == ATTACHED_CLIP && v.clip.1 >= 0.99 {
                    self.phase = 3;
                }
            }
            3 => {
                s.face = Some((flat, 1.0));
                s.request = Some(REEL_CLIP);
                s.no_gravity = true;
                let u = unit(sub(rope.g, v.p));
                s.push = [REEL * u[0], REEL * u[1], REEL * u[2]];
                if rope.arrived(v.m) {
                    self.phase = 4;
                    s.push = [0.0; 3];
                } else if v.touched {
                    s.exit = true;
                }
            }
            _ => {
                s.request = Some(HANG_CLIP);
                s.no_gravity = true;
                let aim = unit(sub([rope.a[0], rope.a[1] - HANG_DROP, rope.a[2]], v.p));
                if aim[1] > HANG_GATE {
                    s.push = [HANG_PULL * aim[0], HANG_PULL * aim[1], HANG_PULL * aim[2]];
                }
                if v.square {
                    s.exit = true;
                }
            }
        }
        if s.exit {
            rope.reset();
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_range_and_score() {
        let t = |p: Vec3| Target { point: p, active: true };
        let eye = [0.0; 3];
        let f = [0.0, 0.0, 1.0];
        // In line at 100: selected; at 110: out of range (strictly below); off-axis: refused.
        assert_eq!(scan(&[t([0.0, 0.0, 100.0])], eye, f, |_| true), Some(0));
        assert_eq!(scan(&[t([0.0, 0.0, 110.0])], eye, f, |_| true), None);
        assert_eq!(scan(&[t([10.0, 0.0, 100.0])], eye, f, |_| true), None);
        // The vertical part counts toward the range.
        assert_eq!(scan(&[t([0.0, 60.0, 95.0])], eye, [0.0, 60.0, 95.0], |_| true), None);
        // Inactive and blocked targets are skipped; the best score wins.
        let list = [Target { point: [0.0, 0.0, 50.0], active: false }, t([0.5, 0.0, 50.0]), t([0.1, 0.0, 50.0])];
        assert_eq!(scan(&list, eye, f, |_| true), Some(2));
        assert_eq!(scan(&list, eye, f, |i| i != 2), Some(1));
        assert_eq!(sight_samples(22.0), vec![0.0, 5.0, 10.0, 15.0, 20.0]);
        assert_eq!(sight_samples(20.0), vec![0.0, 5.0, 10.0, 15.0]);
    }

    #[test]
    fn aim_mode_start_and_end() {
        let mut a = AimMode::default();
        // The stick X alone never starts it (y 0); Y in state 1 does.
        assert!(!a.step(1, None, 0.0, false, false));
        assert!(!a.step(1, None, 87.5, false, false) && a.on);
        assert!(a.step(1, None, 0.0, false, false));
        // A jump (state 7) ends it.
        assert!(!a.step(7, None, 0.0, false, false) && !a.on);
        // In state 13 the choice: R2 1, Circle 0, both 1, neither keeps it.
        a.step(13, Some(2), 0.0, false, true);
        assert!(!a.choice);
        a.step(13, Some(2), 0.0, true, true);
        assert!(a.choice);
        a.step(13, Some(2), 0.0, false, false);
        assert!(a.choice);
    }

    #[test]
    fn fire_reel_hang_and_let_go() {
        let mut rope = Rope::default();
        rope.select(Some((0, [0.0, 20.0, 20.0])));
        let mut h = RopeHero::default();
        let mut v = View { p: [0.0; 3], m: [0.0, 1.0, 0.0], clip: (FIRE_CLIP, 0.3), ..View::default() };
        assert!(h.tick(v, &mut rope).holster);
        // The fire event: G is the anchor pulled 0.935 back toward M.
        v.frame_bits = 0x20;
        assert_eq!(h.tick(v, &mut rope).cue, Some(CUE_FIRE));
        let back = len(sub(rope.a, rope.g));
        assert!((back - TARGET_BACK).abs() < 1e-5);
        while rope.state != RopeState::Attached {
            rope.update(v.m);
        }
        assert_eq!(h.tick(v, &mut rope).cue, Some(CUE_ATTACH));
        v.clip = (ATTACHED_CLIP, 0.99);
        h.tick(v, &mut rope);
        assert_eq!(h.phase, 3);
        // Reeling pulls 0.8 toward G; arrival within 3 of the muzzle hangs.
        let s = h.tick(v, &mut rope);
        assert!((len(s.push) - REEL).abs() < 1e-5 && s.no_gravity);
        v.m = sub(rope.g, [0.0, 2.0, 0.0]);
        v.p = sub(v.m, [0.0, 1.0, 0.0]);
        h.tick(v, &mut rope);
        assert_eq!(h.phase, 4);
        // Below the anchor's drop point by enough: the pull up; Square lets go and resets.
        v.p = [0.0, 10.0, 19.0];
        assert!(h.tick(v, &mut rope).push[1] > 0.0);
        v.square = true;
        assert!(h.tick(v, &mut rope).exit);
        assert_eq!(rope, Rope::default());
    }

    #[test]
    fn reel_stops_on_contact() {
        let mut rope = Rope { state: RopeState::Attached, anchor: Some(0), a: [0.0, 20.0, 20.0], g: [0.0, 19.4, 19.4], q: [0.0, 19.4, 19.4] };
        let mut h = RopeHero { phase: 3, fired: true };
        let v = View { p: [0.0; 3], m: [0.0, 1.0, 0.0], touched: true, ..View::default() };
        assert!(h.tick(v, &mut rope).exit);
        assert_eq!(rope.state, RopeState::Idle);
    }
}
