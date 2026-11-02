//! Roberta's attack step and shots (SPEC-0036 Attack input (Roberta), Charge completion, Combo
//! windows, Attack router, Attack animation choice, Hero shot slots, Roberta shot projectile):
//! the hold with its 80-tick threshold, the quick-shot chain and the rapid shot, the full release
//! (and sub-state 14 for ids 162, 169 and 171), the spins and the leap start, the requests of
//! each sub-state, the launch classes and the shot rows, the six shot slots with their launch,
//! homing, life and movement, and the damage scale.
//!
//! Not covered: the near-target swat start (no locked target), the leap's own steps (sub-states
//! 8 and 13 request their clips and end with them), the spin replays, the fan and the random
//! target finder of homing modes 2 to 5 (the launch aims at the auto-target or straight ahead),
//! the bounce and deflect on guarding targets, hostile shots, world contact other than the
//! ground, the shot aim override and the right-stick aim.

use crate::hero::Vec3;
use crate::rng::Rng;

const THRESHOLD: u32 = 80;

/// Requests (main animation ids).
pub const HOLD_START: u32 = 17;
pub const HOLD_LOOP: u32 = 25;
pub const SHOT_1: u32 = 12;
pub const SHOT_2: u32 = 13;
pub const SHOT_3: u32 = 14;
pub const SWAT: u32 = 16;
pub const RAPID: u32 = 15;
pub const SPIN_CW: u32 = 22;
pub const SPIN_CCW: u32 = 21;
pub const SPIN_CW_END: u32 = 148;
pub const SPIN_CCW_END: u32 = 147;
pub const LEAP: u32 = 23;
pub const OBJECT_HOLD: u32 = 63;

/// Attack animation choice, Roberta: the full-release animation for the Mote in word 2.
pub fn release_anim(mote: Option<u16>) -> u32 {
    match mote {
        Some(161 | 164) => 0x1E,
        Some(162 | 169 | 171) => 0x1D,
        Some(167 | 173) => 0x1F,
        Some(168 | 170 | 172) => 0x1C,
        _ => 0x1B,
    }
}

/// The playing clip as the attack step reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub action: u32,
    pub progress: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RobertaAttack {
    /// Sub-state (+0xF90); 0 ends the attack.
    pub sub: u8,
    /// Sub-state tick counter F.
    pub f: u32,
    repeats: u8,
    closed: bool,
    /// The shots of this sub-state have been fired (cleared on every transition).
    pub fired: bool,
    restart: bool,
    /// The hit list is cleared on every transition.
    pub cleared: bool,
    /// The held ticks at the release (the stored power is F / 120).
    pub held: u32,
    /// A Mote object is live (the host's word; sub-state 14 waits for it).
    pub object_live: bool,
    /// Sub-state 14's Mote animation has ended: request 63.
    pub waiting: bool,
}

impl RobertaAttack {
    /// The attack start with the gesture mode word: 0 the hold, 1 the leap (8), 2 the clockwise
    /// spin (6), 3 the counter-clockwise spin (7).
    pub fn start_with(mode: u8) -> Self {
        let sub = match mode {
            1 => 8,
            2 => 6,
            3 => 7,
            _ => 1,
        };
        Self { sub, cleared: true, ..Default::default() }
    }

    fn go(&mut self, sub: u8) {
        self.sub = sub;
        self.f = 0;
        self.cleared = true;
        self.fired = false;
    }

    /// One tick: `held` the attack button held, `pressed` its press edge, `mote` the Mote in
    /// word 2. Returns the request, or None when the attack has ended.
    pub fn tick(&mut self, held: bool, pressed: bool, mote: Option<u16>, clip: Clip) -> Option<(u32, bool)> {
        let done = |s: &Self, id: u32| s.f != 0 && clip.action == id && clip.progress >= 0.99;
        match self.sub {
            1 => {
                if held {
                    self.f += 1;
                } else if self.f >= THRESHOLD {
                    self.held = self.f;
                    if matches!(mote, Some(162 | 169 | 171)) {
                        self.go(14);
                        self.fired = true;
                    } else {
                        self.go(9);
                    }
                } else {
                    self.held = self.f;
                    self.go(2);
                }
            }
            2 | 3 => {
                let (next, id) = if self.sub == 2 { (3, SHOT_1) } else { (4, SHOT_2) };
                if pressed && !self.closed {
                    if self.f < 20 {
                        self.go(10);
                        self.restart = true;
                    } else if self.f <= 40 {
                        self.go(next);
                    } else {
                        self.closed = true;
                    }
                } else if done(self, id) {
                    self.go(0);
                }
            }
            4 => {
                if done(self, SHOT_3) {
                    self.go(0);
                }
            }
            10 => {
                if pressed && self.repeats <= 4 && self.f > 10 && self.f < 50 {
                    self.repeats += 1;
                    self.go(10);
                    self.restart = true;
                } else if done(self, RAPID) {
                    self.go(0);
                }
            }
            9 => {
                if done(self, release_anim(mote)) {
                    self.go(0);
                }
            }
            14 => {
                // The Mote animation, then 63 (loop) until no Mote object is live (no object: done).
                if self.waiting || done(self, release_anim(mote)) {
                    self.waiting = true;
                    if !self.object_live {
                        self.waiting = false;
                        self.go(0);
                    }
                }
            }
            6 | 7 => {
                let end = if self.sub == 6 { SPIN_CW_END } else { SPIN_CCW_END };
                if done(self, end) {
                    self.go(0);
                }
            }
            8 => {
                if done(self, LEAP) {
                    self.go(0);
                }
            }
            _ => self.go(0),
        }
        if self.sub != 0 && self.sub != 1 {
            self.f += 1;
        }
        let action = match self.sub {
            0 => return None,
            1 => {
                if clip.action == HOLD_START && clip.progress >= 0.99 || clip.action == HOLD_LOOP {
                    HOLD_LOOP
                } else {
                    HOLD_START
                }
            }
            2 => SHOT_1,
            3 => SHOT_2,
            4 => SHOT_3,
            5 => SWAT,
            6 | 7 => {
                let (spin, end) = if self.sub == 6 { (SPIN_CW, SPIN_CW_END) } else { (SPIN_CCW, SPIN_CCW_END) };
                if clip.action == end || (clip.action == spin && clip.progress >= 0.99) { end } else { spin }
            }
            8 => LEAP,
            14 if self.waiting => 63,
            9 | 14 => release_anim(mote),
            _ => RAPID,
        };
        Some((action, std::mem::take(&mut self.restart)))
    }

    /// The launch class of the sub-state: 0 for 5, 6, 7 and 10; 1 for 2, 3, 8, 11 and 12; 2 for
    /// 4; 3 for 9 and 14; 4 otherwise.
    pub fn class(&self) -> u8 {
        match self.sub {
            5..=7 | 10 => 0,
            2 | 3 | 8 | 11 | 12 => 1,
            4 => 2,
            9 | 14 => 3,
            _ => 4,
        }
    }

    /// Roberta's step factor D by sub-state (SPEC-0036 Damage).
    pub fn step_factor(&self) -> f32 {
        match self.sub {
            4 => 2.0,
            5 | 12 => 0.5,
            9 => 1.5,
            10 => 0.33,
            _ => 1.0,
        }
    }
}

/// A hero shot's world-contact cue, first collider list, by the collider's surface byte
/// (SPEC-0036 World contact): 2 gives 8; 4, 8, 11, 16, 18 give 7; 5 gives 15; 6 gives 14; 7
/// gives 13; 9, 14 give 25; 12, 13 give 20; 15 gives 19; 17 gives 26; others give 6.
pub fn collider_cue(surface: u8) -> i32 {
    match surface {
        2 => 8,
        4 | 8 | 11 | 16 | 18 => 7,
        5 => 15,
        6 => 14,
        7 => 13,
        9 | 14 => 25,
        12 | 13 => 20,
        15 => 19,
        17 => 26,
        _ => 6,
    }
}

/// A hero shot's world-contact cue by a surface record's byte +0x4D (second list), and the
/// weapon-on-world cue by a collision face's material code (SPEC-0031 Weapon hits on the
/// world; used here for the ground end too): 1, 13, 14, 15, 17 give 15; 2 gives 13; 3, 4 give
/// 8; 5 gives 10; 7 gives 12; 8, 11 give 14; 9, 18, 19 give 7; 10 gives 11; others give 6.
pub fn surface_cue(code: u32) -> i32 {
    match code {
        1 | 13 | 14 | 15 | 17 => 15,
        2 => 13,
        3 | 4 => 8,
        5 => 10,
        7 => 12,
        8 | 11 => 14,
        9 | 18 | 19 => 7,
        10 => 11,
        _ => 6,
    }
}

/// One shot row: speed (units per second), radius, life (ticks), homing mode, reaction status.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Row {
    pub speed: f32,
    pub radius: f32,
    pub life: i32,
    pub homing: i32,
    pub reaction: i32,
}

/// The eleven rows at 0x2DD6D0 + 0x14 r.
pub const ROWS_AT: u32 = 0x002D_D6D0;

/// The shot rows read from the boot data (life converted from seconds to ticks).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rows(pub Vec<Row>);

impl Rows {
    /// From the 55 words of the eleven rows.
    pub fn from_words(w: &[u32]) -> Self {
        Self(
            w.chunks(5)
                .map(|r| Row { speed: f32::from_bits(r[0]), radius: f32::from_bits(r[1]), life: r[2] as i32 * 60, homing: r[3] as i32, reaction: r[4] as i32 })
                .collect(),
        )
    }

    fn row(&self, i: usize) -> Row {
        self.0.get(i).copied().unwrap_or_default()
    }

    /// The default row: the second row's speed, radius 3.0, 120 ticks, straight, no reaction.
    fn default_row(&self) -> Row {
        Row { speed: self.row(1).speed, radius: 3.0, life: 120, homing: 0, reaction: -1 }
    }

    /// The row and shot count of a launch: quick classes by sub-state (table A), full release by
    /// the Mote in word 2 (table B).
    pub fn launch(&self, class: u8, sub: u8, mote: Option<u16>) -> (Row, u32) {
        match class {
            0 if sub == 10 => (self.row(0), 1),
            0 | 1 => (self.row(1), 1),
            2 => (self.row(2), 1),
            3 => match mote {
                None => (self.row(2), 1),
                Some(163) => (self.row(4), 1),
                Some(161) => (self.row(5), 1),
                Some(160) => (self.row(6), 1),
                Some(167) => (self.row(7), 2),
                Some(170) => (self.row(8), 3),
                Some(168) => (self.row(9), 3),
                Some(172) => (self.row(10), 6),
                Some(162 | 169 | 171) => (self.default_row(), 0),
                Some(_) => (self.default_row(), 1),
            },
            _ => (self.default_row(), 1),
        }
    }
}

/// A shot in one of the six slots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub position: Vec3,
    pub dir: Vec3,
    pub speed: f32,
    pub radius: f32,
    pub life: i32,
    pub homing: i32,
    /// The target character id (homing), chosen at launch.
    pub target: Option<u64>,
    pub element: i32,
    /// The damage scale S and the per-shot scale T.
    pub scale: f32,
    pub t: f32,
    /// The launch class (0 to 2 quick; 3 full release).
    pub class: u8,
    /// The status request (code, duration, amount, period), or None.
    pub status: Option<[i32; 4]>,
    /// The host's id for the shot's visual record (SPEC-0034 Roberta's shot visuals).
    pub id: u32,
    /// The arming counter (+0x7C): 0 at launch, 15 after a bounce; while above 0 after its
    /// decrement the shot ignores guards and does not steer (SPEC-0036 Arming counter).
    pub arming: i32,
    /// Deflected by a guarding target with the deflect flag: it now tests the hero and, once
    /// armed, steers toward the current hero (SPEC-0036 Quick shot meets a guarding target).
    pub hostile: bool,
}

/// Roberta's shot-visual tables (SPEC-0034 Roberta's shot visuals), read from the boot data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualTables {
    /// The flight tables by kind 0 to 2 (8 entries by shot step).
    pub flight: [[i32; 8]; 3],
    /// The full-release flight table by Mote (14 entries, m - 160).
    pub full: [i32; 14],
    /// The Mote aura table (the second flight layer), by Mote.
    pub aura: [i32; 14],
    /// The impact table by Mote, and the impact tables by kind 0 to 2 (by shot step).
    pub impact_mote: [i32; 14],
    pub impact: [[i32; 8]; 3],
}

impl Default for VisualTables {
    fn default() -> Self {
        Self { flight: [[-1; 8]; 3], full: [-1; 14], aura: [-1; 14], impact_mote: [-1; 14], impact: [[-1; 8]; 3] }
    }
}

/// The flight table addresses by kind, the full-release table, the impact tables.
pub const FLIGHT_AT: [u32; 3] = [0x0031_54C8, 0x0031_54E8, 0x0031_5508];
pub const FULL_AT: u32 = 0x0031_5440;
pub const IMPACT_MOTE_AT: u32 = 0x0031_5478;
pub const IMPACT_AT: [u32; 3] = [0x0031_5528, 0x0031_5548, 0x0031_5568];

impl VisualTables {
    /// A shot's effect ids (A flight, B second flight layer, C impact; -1 none) for its kind (the
    /// launch class: 0 to 2 quick, 3 full release), shot step (0 to 7) and Mote. None when the
    /// Mote's power-attack object supplies the visuals instead (a full release with 161, 162,
    /// 164, 165, 166, 169, 171 or 173).
    pub fn ids(&self, kind: u8, step: usize, mote: Option<u16>) -> Option<[i32; 3]> {
        let m = mote.and_then(|id| usize::try_from(id as i32 - 160).ok()).filter(|&k| k < 14);
        let step = step.min(7);
        if kind == 3 && matches!(mote, Some(161 | 162 | 164 | 165 | 166 | 169 | 171 | 173)) {
            return None;
        }
        let a = match kind {
            0..=2 => self.flight[kind as usize][step],
            3 => m.map_or(self.flight[2][step], |k| self.full[k]),
            _ => 402,
        };
        let b = if kind == 3 { -1 } else { m.map_or(-1, |k| self.aura[k]) };
        let c = match (m, kind) {
            (Some(k), _) => self.impact_mote[k],
            (None, 0..=2) => self.impact[kind as usize][step],
            _ => 403,
        };
        Some([a, b, c])
    }
}

fn unit(v: Vec3) -> Vec3 {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l == 0.0 { v } else { [v[0] / l, v[1] / l, v[2] / l] }
}

impl Shot {
    /// One tick before the hit tests: the life counts down (0 ends the shot), the homing steers
    /// toward the target's aim point (modes 1 to 5) and the shot moves by d x speed / 60.
    /// Returns false when the shot has expired.
    pub fn step(&mut self, target_aim: Option<Vec3>) -> bool {
        self.life -= 1;
        if self.life <= 0 {
            return false;
        }
        if self.arming > 0 {
            self.arming -= 1;
        }
        if self.homing != 0 && self.arming == 0 {
            if let Some(a) = target_aim {
                let w = unit([a[0] - self.position[0], a[1] - self.position[1], a[2] - self.position[2]]);
                self.dir = unit(std::array::from_fn(|k| 0.75 * self.dir[k] + 0.25 * w[k]));
            }
        }
        for k in 0..3 {
            self.position[k] += self.dir[k] * self.speed / 60.0;
        }
        true
    }
}

impl Shot {
    /// Guard handling is allowed (the arming counter is 0 after this tick's decrement).
    pub fn armed(&self) -> bool {
        self.arming == 0
    }

    /// A quick shot meeting a guarding target with no deflect (SPEC-0036 Quick shot meets a
    /// guarding target): d and the side axis (d x up) are negated; r1 = U(0, 0.5) with a fair
    /// sign, d1 = r1 side + (1 - |r1|) d; r2 drawn the same way, d2 = r2 up + (1 - |r2|) d1, not
    /// renormalised (the speed falls to 61 to 100 percent); the arming counter becomes 15. The
    /// rebuilt-frame quirk of a second bounce is not reproduced.
    pub fn bounce(&mut self, rng: &mut crate::rng::Rng) {
        let mut draw = || {
            let r = rng.frange(0.0, 0.5);
            if rng.range(0, 1) == 1 { -r } else { r }
        };
        let (r1, r2) = (draw(), draw());
        self.bounce_with(r1, r2);
    }

    /// The deflect: d = unit(the current hero's aim point - the shot's position), the arming
    /// counter 15, and the shot turns hostile.
    pub fn deflect(&mut self, hero_aim: Vec3) {
        self.dir = unit([hero_aim[0] - self.position[0], hero_aim[1] - self.position[1], hero_aim[2] - self.position[2]]);
        self.arming = 15;
        self.hostile = true;
    }

    pub fn bounce_with(&mut self, r1: f32, r2: f32) {
        let d = self.dir;
        let side = [-d[2], 0.0, d[0]];
        let (d, side) = (d.map(|c| -c), side.map(|c| -c));
        let d1: Vec3 = std::array::from_fn(|k| r1 * side[k] + (1.0 - r1.abs()) * d[k]);
        let up = [0.0, 1.0, 0.0];
        self.dir = std::array::from_fn(|k| r2 * up[k] + (1.0 - r2.abs()) * d1[k]);
        self.arming = 15;
    }
}

/// The launch direction from the body anchor to the aim point.
pub fn launch_dir(anchor: Vec3, aim: Vec3) -> Vec3 {
    unit([aim[0] - anchor[0], aim[1] - anchor[1], aim[2] - anchor[2]])
}

/// The body anchor: the hero position + (0, 1.5, 0) + 0.5 x facing (the test vector's anchor
/// for a hero at the origin facing +Z; Medium).
pub fn anchor(position: Vec3, facing: Vec3) -> Vec3 {
    [position[0] + 0.5 * facing[0], position[1] + 1.5, position[2] + 0.5 * facing[2]]
}

/// The default aim point: hero position + 20 x facing + (0, 2, 0).
pub fn default_aim(position: Vec3, facing: Vec3) -> Vec3 {
    [position[0] + 20.0 * facing[0], position[1] + 2.0, position[2] + 20.0 * facing[2]]
}

/// The full release's charge rule: x = Q x (S x 0.01) rounded half away from zero, at least 1
/// when Q is at least 1.
pub fn charge_full(q: i32, s: f32) -> i32 {
    let x = ps2_float::fpu::mul(q as f32, ps2_float::fpu::mul(s, 0.01));
    let n = if x < 0.0 { (x - 0.5) as i32 } else { (x + 0.5) as i32 };
    if n < 1 && q >= 1 { 1 } else { n }
}

/// A uniform draw in [-1, 1] (the fan; kept for the modes that use it).
pub fn fan_draw(rng: &mut Rng) -> f32 {
    rng.frange(-1.0, 1.0)
}

/// The fan of homing modes 2 to 5 (SPEC-0036 Roberta shot projectile, Launch step 4): with
/// s = unit(d x up), d1 = r1 s + (1 - |r1|) d and d2 = r2 up + (1 - |r2|) d1, not renormalised
/// (so a fanned shot that is not steered flies slower than its row speed).
pub fn fan(d: Vec3, r1: f32, r2: f32) -> Vec3 {
    let s = unit([-d[2], 0.0, d[0]]);
    let d1: Vec3 = std::array::from_fn(|k| r1 * s[k] + (1.0 - r1.abs()) * d[k]);
    let up = [0.0, 1.0, 0.0];
    std::array::from_fn(|k| r2 * up[k] + (1.0 - r2.abs()) * d1[k])
}

/// The attack-family driver's two timed displays of Roberta (SPEC-0036 Roberta charge-up hold
/// effect and Timed glow): each accumulates the tick length by a truncating add while its
/// gate holds and is reset to 0 when it fails.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Displays {
    hold: f32,
    glow: f32,
}

/// Effect 239 (the charge-up hold, while not yet charged) and 0xF1 (the leap glow).
pub const HOLD_EFFECT: i32 = 239;
pub const GLOW_EFFECT: i32 = 0xF1;

impl Displays {
    /// One driver pass: returns (hold shown, glow shown). The hold shows from the pass on which
    /// the accumulator of sub-state 1 with F at or below 80 (not yet charged) reaches 0.8 s,
    /// and stops (the accumulator to 0) once F is above 80 or the sub-state ends; the glow while
    /// the accumulator of sub-state 8 is in [0.5, 0.8).
    pub fn step(&mut self, sub: u8, f: u32) -> (bool, bool) {
        let dt = 1.0f32 / 60.0;
        if sub == 1 && f <= 80 {
            self.hold = ps2_float::fpu::add(self.hold, dt);
        } else {
            self.hold = 0.0;
        }
        if sub == 8 {
            self.glow = ps2_float::fpu::add(self.glow, dt);
        } else {
            self.glow = 0.0;
        }
        (self.hold >= 0.8, self.glow >= 0.5 && self.glow < 0.8)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn shot_bounce() {
        // SPEC-0036 Bounce test vectors: d = (1, 0, 0) (side (0, 0, 1)), r1 = r2 = 0.3 gives
        // (-0.49, 0.3, -0.21); both -0.5 gives (-0.25, -0.5, 0.25); r = 0 gives (-1, 0, 0).
        let shot = || Shot { position: [0.0; 3], dir: [1.0, 0.0, 0.0], speed: 60.0, radius: 1.0, life: 30, homing: 1, target: Some(1), element: 0, scale: 1.0, t: 1.0, class: 0, status: None, id: 0, arming: 0, hostile: false };
        let near = |a: Vec3, b: Vec3| a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-5);
        for ((r1, r2), want) in [((0.3, 0.3), [-0.49, 0.3, -0.21]), ((-0.5, -0.5), [-0.25, -0.5, 0.25]), ((0.0, 0.0), [-1.0, 0.0, 0.0])] {
            let mut s = shot();
            s.bounce_with(r1, r2);
            assert!(near(s.dir, want), "{:?}", s.dir);
            assert_eq!(s.arming, 15);
        }
        // Armed again after 15 updates; no steer before.
        let mut s = shot();
        s.bounce_with(0.0, 0.0);
        for _ in 0..14 {
            s.step(Some([0.0, 0.0, 10.0]));
            assert!(!s.armed() && s.dir == [-1.0, 0.0, 0.0]);
        }
        s.step(Some([0.0, 0.0, 10.0]));
        assert!(s.armed() && s.dir != [-1.0, 0.0, 0.0]);
        // The deflect: aimed at the hero's aim point, armed 15, hostile.
        let mut s = shot();
        s.position = [10.0, 1.0, 0.0];
        s.deflect([0.0, 1.0, 0.0]);
        assert!(near(s.dir, [-1.0, 0.0, 0.0]) && s.arming == 15 && s.hostile);
    }

    use super::*;

    #[test]
    fn shot_fan_vectors() {
        // SPEC-0036 launch test vectors: mode 2 with no target, d = (0, 0.02563, 0.99967).
        let d = [0.0, 0.02563, 0.99967];
        let close = |a: Vec3, b: Vec3| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);
        let a = fan(d, 0.5, 0.5);
        assert!(close(a, [-0.25, 0.50641, 0.24992]), "{a:?}");
        assert!((a.iter().map(|c| c * c).sum::<f32>().sqrt() - 0.6176).abs() < 1e-3);
        assert!(close(fan(d, -0.5, -0.5), [0.25, -0.49359, 0.24992]));
        assert!(close(fan(d, 0.0, 0.0), d));
    }

    #[test]
    fn shot_visual_ids() {
        // SPEC-0034 Roberta's shot visuals, synthetic tables.
        let mut t = VisualTables::default();
        t.flight = [[10, 11, 12, 13, 14, 15, 16, 17], [20; 8], [30, 31, 32, 33, 34, 35, 36, 37]];
        t.impact = [[50; 8], [60; 8], [70, 71, 72, 73, 74, 75, 76, 77]];
        t.full[3] = 90;
        t.aura[3] = 91;
        t.impact_mote[3] = 92;
        // Quick shots: A by kind and step, B the aura (none without a Mote), C by kind and step.
        assert_eq!(t.ids(0, 2, None), Some([12, -1, 50]));
        assert_eq!(t.ids(1, 0, Some(163)), Some([20, 91, 92]));
        // Full release: A by Mote, or the kind-2 table at the step without one; no B.
        assert_eq!(t.ids(3, 0, Some(163)), Some([90, -1, 92]));
        assert_eq!(t.ids(3, 4, None), Some([34, -1, 403]));
        // The Mote objects that supply their own visuals; another kind gives 402 and 403.
        assert_eq!(t.ids(3, 0, Some(161)), None);
        assert_eq!(t.ids(4, 0, None), Some([402, -1, 403]));
    }

    #[test]
    fn shot_world_cues() {
        // SPEC-0036 World contact vectors: collider surface 2 plays 8; a record with +0x4D 5
        // plays 10.
        assert_eq!(collider_cue(2), 8);
        assert_eq!(collider_cue(17), 26);
        assert_eq!(collider_cue(3), 6);
        assert_eq!(surface_cue(5), 10);
        assert_eq!(surface_cue(19), 7);
        assert_eq!(surface_cue(0), 6);
    }

    fn rows() -> Rows {
        let mut w = Vec::new();
        for (sp, r, life, h, re) in [(30.0f32, 1.0f32, 2u32, 1u32, u32::MAX), (60.0, 1.0, 3, 0, u32::MAX), (90.0, 1.0, 3, 0, u32::MAX), (0.0, 0.0, 0, 0, u32::MAX), (20.0, 1.0, 8, 1, 0), (75.0, 1.0, 3, 0, u32::MAX), (75.0, 1.0, 3, 0, 7), (20.0, 1.0, 3, 2, 3), (20.0, 1.0, 3, 4, 6), (20.0, 1.0, 3, 3, 1), (20.0, 1.0, 3, 5, u32::MAX)] {
            w.extend([sp.to_bits(), r.to_bits(), life, h, re]);
        }
        Rows::from_words(&w)
    }

    #[test]
    fn quick_chain_and_rapid() {
        // A release at F 30 goes to sub-state 2; a press 25 ticks later chains to 3; a press 12
        // ticks into 3 enters the rapid shot 10.
        let mut a = RobertaAttack::start_with(0);
        let mut clip = Clip::default();
        for _ in 0..30 {
            a.tick(true, false, None, clip);
        }
        a.tick(false, false, None, clip);
        assert_eq!(a.sub, 2);
        for _ in 0..24 {
            a.tick(false, false, None, clip);
        }
        a.tick(false, true, None, clip);
        assert_eq!(a.sub, 3);
        for _ in 0..11 {
            a.tick(false, false, None, clip);
        }
        let r = a.tick(false, true, None, clip);
        assert_eq!((a.sub, r), (10, Some((RAPID, true))));
        clip = Clip { action: RAPID, progress: 0.99 };
        assert!(a.tick(false, false, None, clip).is_none());
    }

    #[test]
    fn full_release_and_classes() {
        // Held 96 ticks with no Mote: sub-state 9 (class 3) and the 0x1B animation; with Mote
        // 169: sub-state 14 with the fired flag set.
        let mut a = RobertaAttack::start_with(0);
        for _ in 0..96 {
            a.tick(true, false, None, Clip::default());
        }
        let r = a.tick(false, false, None, Clip::default());
        assert_eq!((a.sub, a.class(), r.map(|r| r.0), a.held), (9, 3, Some(0x1B), 96));
        let mut a = RobertaAttack::start_with(0);
        for _ in 0..80 {
            a.tick(true, false, Some(169), Clip::default());
        }
        a.tick(false, false, Some(169), Clip::default());
        assert!(a.sub == 14 && a.fired);
        assert_eq!((RobertaAttack::start_with(2).sub, RobertaAttack::start_with(1).sub), (6, 8));
        assert_eq!((release_anim(Some(161)), release_anim(Some(167)), release_anim(Some(172)), release_anim(None)), (0x1E, 0x1F, 0x1C, 0x1B));
    }

    #[test]
    fn shot_rows_and_counts() {
        let r = rows();
        assert_eq!(r.launch(0, 10, None).0.homing, 1);
        assert_eq!(r.launch(1, 2, None).0.life, 180);
        assert_eq!(r.launch(2, 4, None).0.speed, 90.0);
        assert_eq!((r.launch(3, 9, Some(163)).0.life, r.launch(3, 9, Some(172)).1, r.launch(3, 9, Some(168)).1, r.launch(3, 9, Some(167)).1), (480, 6, 3, 2));
        assert_eq!((r.launch(3, 9, Some(164)).0.radius, r.launch(3, 9, Some(164)).0.life), (3.0, 120));
        assert_eq!(r.launch(3, 14, Some(169)).1, 0);
        assert_eq!(r.launch(3, 9, None).0.speed, 90.0);
    }

    #[test]
    fn launch_and_homing_vectors() {
        // Hero at the origin facing +Z, anchor (0, 1.5, 0.5); no target: aim (0, 2, 20).
        let a = anchor([0.0; 3], [0.0, 0.0, 1.0]);
        assert_eq!(a, [0.0, 1.5, 0.5]);
        let d = launch_dir(a, default_aim([0.0; 3], [0.0, 0.0, 1.0]));
        assert!((d[1] - 0.02563).abs() < 1e-4 && (d[2] - 0.99967).abs() < 1e-4);
        // Homing: shot at the origin, d (1, 0, 0), speed 60, target (0, 0, 10).
        let mut s = Shot { position: [0.0; 3], dir: [1.0, 0.0, 0.0], speed: 60.0, radius: 1.0, life: 10, homing: 1, target: Some(1), element: 0, scale: 1.0, t: 1.0, class: 0, status: None, id: 0, arming: 0, hostile: false };
        s.step(Some([0.0, 0.0, 10.0]));
        assert!((s.dir[0] - 0.948683).abs() < 1e-5 && (s.position[2] - 0.316228).abs() < 1e-5);
        s.step(Some([0.0, 0.0, 10.0]));
        assert!((s.dir[0] - 0.816441).abs() < 1e-4);
        // The full release's charge: lesser 25 and greater 40 with S = 100.
        assert_eq!((charge_full(25, 100.0), charge_full(40, 100.0)), (25, 40));
    }

    /// SPEC-0036 test vectors: with F rising 1 a pass from 1, the hold starts on pass 49 (F 49),
    /// plays through F = 80 and stops (the accumulator 0) when F becomes 81, not restarting
    /// while she keeps holding; leaving sub-state 1 stops it; the glow shows from tick 31
    /// through 48 of the leap and resets on another sub-state.
    #[test]
    fn hold_and_glow_timers() {
        let mut d = Displays::default();
        let shown: Vec<bool> = (1..=80).map(|f| d.step(1, f).0).collect();
        assert!(!shown[47] && shown[48] && shown[79]);
        assert!(!d.step(1, 81).0);
        assert_eq!(d.hold, 0.0);
        assert!(!d.step(1, 82).0);
        let mut d = Displays::default();
        for f in 1..=60 {
            d.step(1, f);
        }
        assert!(!d.step(13, 60).0);
        let mut d = Displays::default();
        let glow: Vec<bool> = (1..=49).map(|_| d.step(8, 0).1).collect();
        assert!(!glow[29] && glow[30] && glow[47] && !glow[48]);
        let mut d = Displays::default();
        for _ in 0..40 {
            d.step(8, 0);
        }
        assert_eq!(d.step(13, 0), (false, false));
        assert_eq!(d.glow, 0.0);
    }
}
