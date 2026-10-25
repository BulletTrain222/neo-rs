//! The options block and the apply rule (SPEC-0035 Options menu and option block, Behavior 3;
//! SPEC-0025 Options block): the three toggles and the unlabelled cycler and toggle, the three
//! volume sliders, and the apply rule that sets the Music, Sound-effects and Voice gains, the
//! knob positions and the pad inversion flags. The gains are 1.0 until the first apply.

/// The 32-byte block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub invert_lr: u32,
    pub invert_ud: u32,
    pub vibration: u32,
    pub cycler: u32,
    pub toggle: u32,
    pub music: f32,
    pub effects: f32,
    pub voice: f32,
}

impl Default for Block {
    /// The defaults written at start-up.
    fn default() -> Self {
        Self { invert_lr: 0, invert_ud: 0, vibration: 1, cycler: 1, toggle: 0, music: 75.0, effects: 75.0, voice: 75.0 }
    }
}

impl Block {
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut b = [0u8; 32];
        for (k, v) in [self.invert_lr, self.invert_ud, self.vibration, self.cycler, self.toggle].into_iter().enumerate() {
            b[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
        }
        for (k, v) in [self.music, self.effects, self.voice].into_iter().enumerate() {
            b[0x14 + 4 * k..0x18 + 4 * k].copy_from_slice(&v.to_le_bytes());
        }
        b
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        let w = |k: usize| u32::from_le_bytes(b[k..k + 4].try_into().unwrap());
        Self { invert_lr: w(0), invert_ud: w(4), vibration: w(8), cycler: w(0x0C), toggle: w(0x10), music: f32::from_bits(w(0x14)), effects: f32::from_bits(w(0x18)), voice: f32::from_bits(w(0x1C)) }
    }
}

/// A toggle press: new = (old != 1).
pub fn toggle(v: u32) -> u32 {
    (v != 1) as u32
}

/// The cycler: Left sets 2 below 1, else value - 1; Right sets (value + 1) mod 3; Left first.
pub fn cycle(mut v: u32, left: bool, right: bool) -> u32 {
    if left {
        v = if v < 1 { 2 } else { v - 1 };
    }
    if right {
        v = (v + 1) % 3;
    }
    v
}

/// A slider pass: held Right adds 1.0, else held Left subtracts 1.0; clamped to 0 to 100.
pub fn slide(v: f32, left: bool, right: bool) -> f32 {
    let v = if right {
        v + 1.0
    } else if left {
        v - 1.0
    } else {
        v
    };
    v.clamp(0.0, 100.0)
}

/// The knob x-position of a slider value: trunc(value x 1.29 + 2.0).
pub fn knob(v: f32) -> i32 {
    (v * 1.29 + 2.0) as i32
}

/// The live options: the block, the gains and knob positions the last apply set (None before
/// the first apply), and the pad inversion flags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub block: Block,
    pub gains: [f32; 3],
    pub knobs: Option<[i32; 3]>,
    pub invert: (bool, bool),
}

impl Default for Options {
    fn default() -> Self {
        Self { block: Block::default(), gains: [1.0; 3], knobs: None, invert: (false, false) }
    }
}

impl Options {
    /// The apply rule: gains = value x 0.01 clamped to 0 to 1; knobs; inversion flags.
    pub fn apply(&mut self) {
        let b = self.block;
        self.gains = [b.music, b.effects, b.voice].map(|v| (v * 0.01).clamp(0.0, 1.0));
        self.knobs = Some([b.music, b.effects, b.voice].map(knob));
        self.invert = (b.invert_lr != 0, b.invert_ud != 0);
    }

    pub fn music(&self) -> f32 {
        self.gains[0]
    }

    pub fn effects(&self) -> f32 {
        self.gains[1]
    }

    pub fn voice(&self) -> f32 {
        self.gains[2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_apply() {
        assert_eq!((toggle(1), toggle(0), toggle(7)), (0, 1, 1));
        // The cycler: 0 Left gives 2; 2 Right gives 0; 1 Right gives 2.
        assert_eq!((cycle(0, true, false), cycle(2, false, true), cycle(1, false, true)), (2, 0, 2));
        assert_eq!((slide(99.5, false, true), slide(0.5, true, false), slide(50.0, true, true)), (100.0, 0.0, 51.0));
        // Gains are 1.0 until the first apply, which sets 0.75 with the defaults.
        let mut o = Options::default();
        assert_eq!(o.gains, [1.0; 3]);
        o.apply();
        assert_eq!(o.gains, [0.75; 3]);
        assert_eq!(o.knobs, Some([knob(75.0); 3]));
        assert_eq!(knob(75.0), 98);
        assert_eq!(knob(0.0), 2);
        // A slot's block (volumes 75, 40, 100) gives 0.75, 0.40 and 1.0.
        o.block = Block { music: 75.0, effects: 40.0, voice: 100.0, ..Block::default() };
        o.apply();
        assert!((o.gains[0] - 0.75).abs() < 1e-6 && (o.gains[1] - 0.4).abs() < 1e-6 && o.gains[2] == 1.0, "{:?}", o.gains);
        // Round trip of the 32 bytes.
        let b = Block { invert_lr: 1, music: 12.0, ..Block::default() };
        assert_eq!(Block::from_bytes(&b.to_bytes()), b);
    }
}
