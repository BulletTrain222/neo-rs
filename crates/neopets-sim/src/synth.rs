//! The bank-music voices of the sound driver (SPEC-0031 Playing a bank sound; Driver volume, pan
//! and pitch; Voice allocator), rendered at 48 kHz: the note-on scan of a program's voices by key
//! window, the pitch register 4096 x 2^(((note - centre) x 128 + fine + centre fine) / 1536), the
//! bank form of the volume step with the equal-power pan table and the rear-half inversion, the
//! SPU2 ADSR envelope from the voice's two register words (the public SPU envelope rule), sample
//! loops, sustain, pitch bend and the voice allocator (24 voices per range). Sound-block voices
//! (section 7) use the same voices with the block form of the volume step, key 60 and an
//! instance's request volume and pan, which follow the emitter each frame.
//!
//! Deviations: samples are read with linear interpolation (the SPU2 uses a 4-point Gaussian
//! filter); the music volume scales the mix instead of each voice's level (the truncation order
//! of A differs); reverb is not rendered.

use std::sync::Arc;

pub const RATE: u32 = 48_000;
pub const VOICES: usize = 24;

/// A decoded sample and its loop start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sample {
    pub pcm: Arc<Vec<i16>>,
    pub loop_start: Option<usize>,
}

/// A voice record as the synth needs it (SPEC-0031 section 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VoiceDef {
    pub priority: u8,
    pub volume: u8,
    pub centre: u8,
    pub fine: u8,
    pub pan: i16,
    pub low: i8,
    pub high: i8,
    pub bend: (u8, u8),
    pub adsr1: u16,
    pub adsr2: u16,
    pub flags: u16,
    /// Index into `Instruments::samples`.
    pub sample: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupDef {
    pub volume: u8,
    pub pan: u16,
    pub voices: Vec<VoiceDef>,
}

/// A bank's instruments: the directory groups (programs) and the samples.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instruments {
    pub groups: Vec<GroupDef>,
    pub samples: Vec<Sample>,
}

/// The pitch register: 4096 x 2^(((note - centre) x 128 + note fine + centre fine) / 1536), with
/// the 44.1 kHz factor when the centre byte has no top bit; limited to the SPU2's 0x3FFF.
pub fn pitch(n: i32, centre_byte: u8, centre_fine: u8) -> u32 {
    let centre = (centre_byte as i8 as i32).abs();
    let note = n.div_euclid(128);
    let fine = n - 128 * note;
    let e = ((note - centre) * 128 + fine + centre_fine as i32) as f64 / 1536.0;
    let mut p = 4096.0 * 2f64.powf(e);
    if centre_byte < 128 {
        p *= 44_100.0 / 48_000.0;
    }
    (p as u32).min(0x3FFF)
}

/// The bend term of a note value: trunc(down x bend x 128 / 32768) below 0, trunc(up x bend x
/// 128 / 32767) at or above 0.
pub fn bend_term(bend: i32, range: (u8, u8)) -> i32 {
    if bend < 0 { range.0 as i32 * bend * 128 / 32768 } else { range.1 as i32 * bend * 128 / 32767 }
}

/// The channel bend from a 14-bit pitch-bend value: trunc(b x 65535 / 16383) - 32768.
pub fn bend_value(b: i32) -> i32 {
    b * 65535 / 16383 - 32768
}

/// The channel pan angle from a controller-10 value.
pub fn pan_angle(v: i32) -> i32 {
    if v < 64 { 270 + 90 * v / 64 } else { 90 * (v - 64) / 63 }
}

/// X of the block form: A = floor(V x 127 / 1024), B = trunc(Vc^2 / 127), D = trunc(Lv^2 / 127),
/// C the instance level (127).
pub fn block_x(v: i32, vc: i32, lv: i32, c: i32) -> i32 {
    let a = v.max(0) * 127 / 1024;
    let b = vc * vc / 127;
    let d = lv.max(0) * lv.max(0) / 127;
    (b * 258 * d / 127 * c / 127 * a / 127).min(32_766)
}

/// The volume and pan step (bank form): levels L and R from X and the angle theta, with the
/// rear half's weaker channel inverted (stereo with inversion, the default mode).
pub fn levels(x: i32, theta: i32) -> (i32, i32) {
    let table = |i: i32| {
        let a = (i as f64 / 2.0).to_radians();
        ((16383.0 * a.cos()) as i32, (16383.0 * a.sin()) as i32)
    };
    let theta = theta.rem_euclid(360);
    let q = if theta < 270 { theta + 90 } else { theta - 270 };
    if q < 180 {
        let t = table(q);
        (x * t.0 / 16383, x * t.1 / 16383)
    } else {
        let t = table(q - 180);
        let (l, r) = (x * t.1 / 16383, x * t.0 / 16383);
        if r < l { (l, -r) } else { (-l, r) }
    }
}

/// X of the bank form: B = trunc(b^2 / 127) with b = trunc(velocity x channel volume / 127),
/// C and D the group and voice volumes on the same squared curve, A the master volume.
pub fn level_x(velocity: i32, channel: i32, group: i32, voice: i32, master: i32) -> i32 {
    let sq = |v: i32| v.max(0) * v.max(0) / 127;
    let b = sq(velocity * channel / 127);
    let (c, d) = (sq(group), sq(voice));
    (b * 258 * d / 127 * c / 127 * master / 127).min(32_766)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

/// The SPU2 ADSR envelope (public register layout and step rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope {
    phase: Phase,
    pub level: i32,
    counter: u32,
    adsr1: u16,
    adsr2: u16,
}

impl Envelope {
    pub fn new(adsr1: u16, adsr2: u16) -> Self {
        Self { phase: Phase::Attack, level: 0, counter: 0, adsr1, adsr2 }
    }

    pub fn release(&mut self) {
        if self.phase != Phase::Off {
            self.phase = Phase::Release;
            self.counter = 0;
        }
    }

    pub fn off(&self) -> bool {
        self.phase == Phase::Off
    }

    /// One 48 kHz step.
    pub fn step(&mut self) {
        let (rate, exp, decrease, sl) = match self.phase {
            Phase::Attack => (((self.adsr1 >> 8) & 0x7F) as i32, self.adsr1 & 0x8000 != 0, false, 0),
            Phase::Decay => ((((self.adsr1 >> 4) & 0x0F) << 2) as i32, true, true, ((self.adsr1 & 0x0F) as i32 + 1) * 0x800),
            Phase::Sustain => (((self.adsr2 >> 6) & 0x7F) as i32, self.adsr2 & 0x8000 != 0, self.adsr2 & 0x4000 != 0, 0),
            Phase::Release => (((self.adsr2 & 0x1F) << 2) as i32, self.adsr2 & 0x20 != 0, true, 0),
            Phase::Off => return,
        };
        let shift = rate >> 2;
        let mut cycles = 1u32 << (shift - 11).max(0);
        let base = if decrease { -8 + (rate & 3) } else { 7 - (rate & 3) };
        let base = if matches!(self.phase, Phase::Decay | Phase::Release) { -8 } else { base };
        let mut step = base << (11 - shift).max(0);
        if exp && !decrease && self.level > 0x6000 {
            cycles *= 4;
        }
        if exp && decrease {
            step = step * self.level / 0x8000;
        }
        self.counter += 1;
        if self.counter < cycles {
            return;
        }
        self.counter = 0;
        self.level = (self.level + step).clamp(0, 0x7FFF);
        match self.phase {
            Phase::Attack if self.level >= 0x7FFF => self.phase = Phase::Decay,
            Phase::Decay if self.level <= sl => self.phase = Phase::Sustain,
            Phase::Release if self.level <= 0 => self.phase = Phase::Off,
            _ => {}
        }
    }
}

/// A sounding voice.
#[derive(Clone, Debug)]
struct Voice {
    owner: (usize, usize, u8),
    def: VoiceDef,
    sample: Sample,
    /// A block voice's cue volume, voice level and voice pan, for a new request volume or pan.
    block: Option<(i32, i32, i32)>,
    note_value_base: i32,
    pos: f64,
    step: f64,
    env: Envelope,
    left: i32,
    right: i32,
    held: bool,
    start: u64,
}

/// The 24 music voices.
#[derive(Clone, Debug, Default)]
pub struct Synth {
    voices: Vec<Option<Voice>>,
    clock: u64,
}

/// A channel's state as a note-on reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelState {
    pub program: usize,
    pub volume: i32,
    pub pan: i32,
    pub bend: i32,
}

impl Synth {
    pub fn new() -> Self {
        Self { voices: vec![None; VOICES], clock: 0 }
    }

    /// The voice allocator (range 1): the first free voice, else steal a candidate whose
    /// priority is at most p and below 126 (lower priority first, then the earlier start).
    fn allocate(&mut self, p: u8) -> Option<usize> {
        if let Some(k) = self.voices.iter().position(|v| v.as_ref().is_none_or(|v| v.env.off())) {
            return Some(k);
        }
        let mut cand = 0usize;
        for k in 1..VOICES {
            let (a, b) = (self.voices[k].as_ref()?, self.voices[cand].as_ref()?);
            if a.def.flags & 0x20 == 0 && (a.def.priority < b.def.priority || (a.def.priority == b.def.priority && a.start < b.start)) {
                cand = k;
            }
        }
        let c = self.voices[cand].as_ref()?;
        (c.def.flags & 0x20 == 0 && c.def.priority <= p && c.def.priority < 126).then_some(cand)
    }

    /// Note-on: every voice of the channel's program whose key window holds the note starts.
    pub fn note_on(&mut self, inst: &Instruments, owner: (usize, usize), ch: ChannelState, note: u8, velocity: u8) {
        let Some(group) = inst.groups.get(ch.program) else { return };
        for def in group.voices.iter().filter(|v| (v.low as i32..=v.high as i32).contains(&(note as i32))).take(32) {
            let Some(sample) = inst.samples.get(def.sample).cloned() else { continue };
            let Some(k) = self.allocate(def.priority) else { continue };
            let x = level_x(velocity as i32, ch.volume, group.volume as i32, def.volume as i32, 127);
            let (left, right) = levels(x, ch.pan + group.pan as i32 + def.pan as i32);
            let base = 128 * note as i32;
            let n = base + bend_term(ch.bend, def.bend);
            self.clock += 1;
            self.voices[k] = Some(Voice {
                owner: (owner.0, owner.1, note),
                def: *def,
                sample,
                block: None,
                note_value_base: base,
                pos: 0.0,
                step: pitch(n, def.centre, def.fine) as f64 / 4096.0,
                env: Envelope::new(def.adsr1, def.adsr2),
                left: left >> 1,
                right: right >> 1,
                held: false,
                start: self.clock,
            });
        }
    }

    /// Start a sound-block voice for instance `owner` at key 60: request volume `v` (1,024 =
    /// 1.0), cue volume `vc`, voice level `lv`, pan angle `pan` (the instance's, added to the
    /// voice's) and the instance bend. Returns whether a voice was taken.
    #[allow(clippy::too_many_arguments)]
    pub fn start_block(&mut self, owner: usize, def: VoiceDef, sample: Sample, v: i32, vc: i32, lv: i32, pan: i32, bend: i32) -> bool {
        let Some(k) = self.allocate(def.priority) else { return false };
        let (left, right) = levels(block_x(v, vc, lv, 127), pan + def.pan as i32);
        let base = 60 * 128;
        self.clock += 1;
        self.voices[k] = Some(Voice {
            owner: (owner, 0, 0),
            def,
            sample,
            block: Some((vc, lv, def.pan as i32)),
            note_value_base: base,
            pos: 0.0,
            step: pitch(base + bend_term(bend, def.bend), def.centre, def.fine) as f64 / 4096.0,
            env: Envelope::new(def.adsr1, def.adsr2),
            left: left >> 1,
            right: right >> 1,
            held: false,
            start: self.clock,
        });
        true
    }

    /// A new request volume and pan for an instance's block voices.
    pub fn set_instance(&mut self, owner: usize, v: i32, pan: i32) {
        for voice in self.voices.iter_mut().flatten().filter(|x| x.owner.0 == owner) {
            if let Some((vc, lv, pv)) = voice.block {
                let (l, r) = levels(block_x(v, vc, lv, 127), pan + pv);
                voice.left = l >> 1;
                voice.right = r >> 1;
            }
        }
    }

    /// Stop an instance's voices at once.
    /// Whether the instance still owns a voice.
    pub fn owns(&self, owner: usize) -> bool {
        self.voices.iter().flatten().any(|v| v.owner.0 == owner)
    }

    pub fn kill(&mut self, owner: usize) {
        for slot in &mut self.voices {
            if slot.as_ref().is_some_and(|v| v.owner.0 == owner) {
                *slot = None;
            }
        }
    }

    /// The voices an instance still has sounding.
    pub fn sounding(&self, owner: usize) -> usize {
        self.voices.iter().flatten().filter(|v| v.owner.0 == owner && !v.env.off()).count()
    }

    /// Note-off: release the channel's voices for that note (held while sustain is on).
    pub fn note_off(&mut self, owner: (usize, usize), note: u8, sustain: bool) {
        for v in self.voices.iter_mut().flatten().filter(|v| v.owner == (owner.0, owner.1, note) && !v.env.off()) {
            if sustain {
                v.held = true;
            } else {
                v.env.release();
            }
        }
    }

    /// Sustain released: the held voices of the channel are released.
    pub fn sustain_off(&mut self, owner: (usize, usize)) {
        for v in self.voices.iter_mut().flatten().filter(|v| (v.owner.0, v.owner.1) == owner && v.held) {
            v.held = false;
            v.env.release();
        }
    }

    /// A new bend re-pitches the channel's voices at once.
    pub fn bend(&mut self, owner: (usize, usize), bend: i32) {
        for v in self.voices.iter_mut().flatten().filter(|v| (v.owner.0, v.owner.1) == owner) {
            let n = v.note_value_base + bend_term(bend, v.def.bend);
            v.step = pitch(n, v.def.centre, v.def.fine) as f64 / 4096.0;
        }
    }

    /// Release every voice a child started (its sequence stopped).
    pub fn release_child(&mut self, child: usize) {
        for v in self.voices.iter_mut().flatten().filter(|v| v.owner.0 == child) {
            v.env.release();
        }
    }

    pub fn clear(&mut self) {
        self.voices = vec![None; VOICES];
    }

    pub fn active(&self) -> usize {
        self.voices.iter().flatten().filter(|v| !v.env.off()).count()
    }

    /// Mix one 48 kHz stereo frame (levels in 1/32768).
    pub fn frame(&mut self) -> (i32, i32) {
        let (mut l, mut r) = (0i64, 0i64);
        for slot in &mut self.voices {
            let Some(v) = slot.as_mut() else { continue };
            if v.env.off() {
                *slot = None;
                continue;
            }
            let sample = &v.sample;
            let pcm = &sample.pcm;
            let i = v.pos as usize;
            if i >= pcm.len() {
                *slot = None;
                continue;
            }
            let frac = v.pos - i as f64;
            let next = match pcm.get(i + 1) {
                Some(&s) => s as f64,
                None => sample.loop_start.and_then(|ls| pcm.get(ls)).map_or(0.0, |&s| s as f64),
            };
            let s = (pcm[i] as f64 * (1.0 - frac) + next * frac) as i64;
            let e = v.env.level as i64;
            l += s * e / 0x8000 * v.left as i64 / 0x4000;
            r += s * e / 0x8000 * v.right as i64 / 0x4000;
            v.env.step();
            v.pos += v.step;
            if v.pos >= v.sample.pcm.len() as f64 {
                match v.sample.loop_start {
                    Some(ls) if ls < v.sample.pcm.len() => v.pos -= (v.sample.pcm.len() - ls) as f64,
                    _ => *slot = None,
                }
            }
        }
        (l.clamp(-0x7FFF_FFFF, 0x7FFF_FFFF) as i32, r.clamp(-0x7FFF_FFFF, 0x7FFF_FFFF) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_vectors() {
        // Pitch: centre 80 with fine 124 at key 60 is 1,364; centre 60 at key 72 is 8,192; a
        // centre byte without the top bit (60) at key 60 gives 3,763.
        assert_eq!(pitch(60 * 128, 0xB0, 124), 1364);
        assert_eq!(pitch(72 * 128, 0xC4, 0), 8192);
        assert_eq!(pitch(60 * 128, 60, 0), 3763);
        // Bend: +32767 with up 2 gives note 62; -32768 with down 2 gives note 58; 16384 with
        // up 1 gives note 60 fine 64.
        assert_eq!(60 * 128 + bend_term(32767, (0, 2)), 62 * 128);
        assert_eq!(60 * 128 + bend_term(-32768, (2, 0)), 58 * 128);
        assert_eq!(60 * 128 + bend_term(16384, (0, 1)), 60 * 128 + 64);
        assert_eq!([0, 4096, 8192, 12288, 16383].map(bend_value), [-32768, -16384, 1, 16386, 32767]);
        assert_eq!([0, 32, 63, 64, 96, 127].map(pan_angle), [270, 315, 358, 0, 45, 90]);
        // Levels at full X: theta 0, 270, 90 and 180.
        assert_eq!(levels(32_766, 0), (23_168, 23_168));
        assert_eq!(levels(32_766, 270), (32_766, 0));
        assert_eq!(levels(32_766, 90), (0, 32_766));
        assert_eq!(levels(32_766, 180), (-23_168, 23_168));
        // The block form's example has X 21,808 (Vc 120, Lv 110, C 127, A 127).
        let x = (113 * 258 * 95 / 127 * 127 / 127 * 127 / 127).min(32_766);
        assert_eq!(x, 21_808);
        assert_eq!(levels(x, 0), (15_419, 15_419));
    }

    #[test]
    fn envelope_attacks_decays_and_releases() {
        // 0x80FF: exponential attack at rate 0 (fastest), decay rate 15, sustain level 15.
        let mut e = Envelope::new(0x80FF, 0x9FCC);
        let mut n = 0;
        while e.phase == Phase::Attack && n < 10_000 {
            e.step();
            n += 1;
        }
        assert!(n < 100, "rate 0 attacks within a few steps ({n})");
        e.release();
        let mut m = 0;
        while !e.off() && m < 1_000_000 {
            e.step();
            m += 1;
        }
        assert!(e.off());
    }

    #[test]
    fn notes_start_by_key_window_and_release() {
        let pcm = Arc::new(vec![1000i16; 1000]);
        let v = |low: i8, high: i8| VoiceDef { volume: 127, centre: 0xBC, low, high, adsr1: 0x00FF, adsr2: 0x1FC0, sample: 0, ..Default::default() };
        let inst = Instruments {
            groups: vec![GroupDef { volume: 127, pan: 0, voices: vec![v(0, 59), v(60, 127), v(0, 127)] }],
            samples: vec![Sample { pcm, loop_start: Some(0) }],
        };
        let mut s = Synth::new();
        let ch = ChannelState { program: 0, volume: 127, pan: 0, bend: 0 };
        s.note_on(&inst, (1, 0), ch, 60, 127);
        assert_eq!(s.active(), 2, "the 60-127 and 0-127 windows");
        for _ in 0..100 {
            s.frame();
        }
        let (l, r) = s.frame();
        assert!(l > 0 && l == r);
        s.note_off((1, 0), 60, false);
        for _ in 0..200_000 {
            s.frame();
        }
        assert_eq!(s.active(), 0);
        // A program past the directory plays nothing.
        s.note_on(&inst, (1, 0), ChannelState { program: 5, ..ch }, 60, 127);
        assert_eq!(s.active(), 0);
    }
}
