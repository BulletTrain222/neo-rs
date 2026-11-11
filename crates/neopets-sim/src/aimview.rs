//! The aim-view marker object (SPEC-0033 section 23; SPEC-0036 Right-stick aim mode).
//!
//! One object stepped once per tick by the hero's aim update with a mode from 0 to 4 (0 off,
//! 1 aim mode, 2 Tor in aim mode, 3 a character aimed at, 4 a rope anchor aimed at). It holds a
//! spin angle, a model scale M and a four-lane colour, each gliding toward a target over a
//! remaining time. Placement: 10 units in front of the camera (S, never written), no side or up
//! offset (V1, V2, never written). The marker's model (registry 312) is not found on the disc
//! (R-0280), so what the app draws with this state is a stand-in.

/// The item that allows modes 2 and 4: the Rope Gun.
pub const ROPE_GUN: u16 = 36;
/// The placement distance in front of the camera (S, initial 10.0, never written).
pub const DISTANCE: f32 = 10.0;
/// The resting model scale (data word 0x2DD254).
pub const SCALE: f32 = 0.3;
/// The scale a change into mode 3 or 4 glides to (0.08 s from 2.0).
pub const LOCK_SCALE: f32 = 0.2;

/// One glide channel: the value moves to `target` over `left` seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glide {
    pub value: f32,
    pub target: f32,
    pub rate: f32,
    pub left: f32,
}

impl Glide {
    fn new(value: f32) -> Self {
        Self { value, target: value, rate: 0.0, left: 0.0 }
    }

    /// Start a glide from `from` to `to` over `secs`.
    fn start(&mut self, from: f32, to: f32, secs: f32) {
        *self = Self { value: from, target: to, rate: 0.0, left: secs };
    }

    /// Set the value at once, dropping any glide.
    fn set(&mut self, v: f32) {
        *self = Self::new(v);
    }

    /// One tick: the rate is taken before the count-down, the time is reduced, the value moves;
    /// at or past the end the value is the target exactly. A channel with no glide keeps its rate
    /// (the spin).
    fn step(&mut self, dt: f32) {
        if self.left > 0.0 {
            self.rate = (self.target - self.value) / self.left;
            self.left -= dt;
            self.value += self.rate * dt;
            if self.left <= 0.0 {
                self.value = self.target;
                self.rate = 0.0;
                self.left = 0.0;
            }
        } else {
            self.value += self.rate * dt;
        }
    }
}

/// The marker object's state.
#[derive(Clone, Debug, PartialEq)]
pub struct AimView {
    pub mode: u8,
    pub active: bool,
    /// Seconds since the last accepted mode change.
    pub mode_time: f32,
    pub time: f32,
    /// The spin angle A1 (radians) and its rate.
    pub spin: Glide,
    /// The model scale M.
    pub scale: Glide,
    /// The colour vector's four lanes.
    pub colour: [Glide; 4],
    /// The scale a change into mode 3 or 4 glides to: 0.2 in the spec; a host may hold the
    /// marker larger while it is on a target (this build's choice).
    pub lock_scale: f32,
}

impl Default for AimView {
    fn default() -> Self {
        Self { mode: 0, active: false, mode_time: 0.0, time: 0.0, spin: Glide::new(0.0), scale: Glide::new(SCALE), colour: [Glide::new(0.0); 4], lock_scale: LOCK_SCALE }
    }
}

impl AimView {
    /// One tick with the aim update's mode; `rope_gun` whether the party owns item 36.
    pub fn step(&mut self, dt: f32, mode: u8, rope_gun: bool) {
        self.time += dt;
        self.mode_time += dt;
        // A change into 2 or 4 is refused without the Rope Gun.
        if mode != self.mode && !(matches!(mode, 2 | 4) && !rope_gun) {
            self.mode = mode;
            self.mode_time = 0.0;
            if matches!(mode, 3 | 4) {
                self.scale.start(2.0, self.lock_scale, 0.08);
                let c = if mode == 3 { [0.4, 0.0, 0.0] } else { [0.0, 0.4, 0.0] };
                for (k, lane) in self.colour.iter_mut().enumerate() {
                    let v = if k < 3 { c[k] } else { 0.1 };
                    lane.start(v, if k < 3 { v } else { 1.0 }, 0.08);
                }
            }
        }
        self.active = self.mode != 0;
        match self.mode {
            0..=2 => {
                self.scale.set(SCALE);
                self.spin.rate = 0.8;
                if self.mode != 0 {
                    for (lane, v) in self.colour.iter_mut().zip([0.33, 0.0, 0.0, 1.0]) {
                        lane.set(v);
                    }
                }
            }
            _ => self.spin.rate = 4.0,
        }
        self.spin.step(dt);
        let turn = std::f32::consts::TAU;
        if self.spin.value.abs() > turn {
            self.spin.value -= (self.spin.value / turn).trunc() * turn;
        }
        self.scale.step(dt);
        let m = self.scale.value;
        self.scale.value = if m > 20.0 { 20.0 } else { m.max(0.1) };
        for lane in &mut self.colour {
            lane.step(dt);
        }
    }

    /// The colour as (r, g, b, a).
    pub fn rgba(&self) -> [f32; 4] {
        self.colour.map(|g| g.value)
    }
}

/// The distance-adjusted score a character must pass to count as aimed at (scan result 1):
/// 0.999 at 100 or more, else 1 - 0.001 x ((100 - d) x 0.04 + 1).
pub fn character_threshold(d: f32) -> f32 {
    if d >= 100.0 { 0.999 } else { 1.0 - 0.001 * ((100.0 - d) * 0.04 + 1.0) }
}

/// The aim view mode the hero's aim update passes (SPEC-0036): 4 when the scan found an
/// anchor, else 2 for Tor in aim mode, else 3 when it found a character, else 1 in aim mode,
/// else 0.
pub fn mode(aiming: bool, tor: bool, anchor: bool, character: bool) -> u8 {
    match (aiming, anchor, tor, character) {
        (false, ..) => 0,
        (true, true, ..) => 4,
        (true, false, true, _) => 2,
        (true, false, false, true) => 3,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn lock_on_scale_glide() {
        // SPEC-0033 section 23 test vector: into mode 3, M is 1.625, 1.25, 0.875, 0.5, 0.2.
        let mut v = AimView::default();
        v.step(DT, 1, false);
        let mut got = Vec::new();
        for _ in 0..6 {
            v.step(DT, 3, false);
            got.push(v.scale.value);
        }
        for (g, w) in got.iter().zip([1.625, 1.25, 0.875, 0.5, 0.2, 0.2]) {
            assert!((g - w).abs() < 1e-4, "{got:?}");
        }
        assert_eq!(v.rgba()[..3], [0.4, 0.0, 0.0]);
        // A host's larger lock scale is held after the glide.
        let mut w = AimView { lock_scale: 0.6, ..AimView::default() };
        for _ in 0..8 {
            w.step(DT, 3, false);
        }
        assert!((w.scale.value - 0.6).abs() < 1e-6);
        assert!((v.rgba()[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rope_gun_gate_and_resting_look() {
        let mut v = AimView::default();
        v.step(DT, 2, false);
        assert_eq!((v.mode, v.active), (0, false));
        v.step(DT, 4, false);
        assert_eq!(v.mode, 0);
        v.step(DT, 2, true);
        assert_eq!((v.mode, v.active), (2, true));
        assert_eq!(v.rgba(), [0.33, 0.0, 0.0, 1.0]);
        assert_eq!(v.scale.value, SCALE);
        v.step(DT, 4, true);
        assert_eq!(v.rgba()[1], 0.4);
        // Spin: 4.0 a second in modes 3 and 4.
        let a = v.spin.value;
        v.step(DT, 4, true);
        assert!((v.spin.value - a - 4.0 * DT).abs() < 1e-5);
    }

    #[test]
    fn modes_and_threshold() {
        assert_eq!(mode(false, false, true, true), 0);
        assert_eq!(mode(true, true, true, false), 4);
        assert_eq!(mode(true, true, false, true), 2);
        assert_eq!(mode(true, false, false, true), 3);
        assert_eq!(mode(true, false, false, false), 1);
        assert_eq!(character_threshold(150.0), 0.999);
        assert!((character_threshold(0.0) - 0.995).abs() < 1e-6);
        assert!((character_threshold(50.0) - 0.997).abs() < 1e-6);
    }
}
