//! The music controller's stream step (SPEC-0009 Music selection): the run-time flag (progress
//! bit 216 and the pack's selector test), the row's bank, stream and block resources with the
//! flag's offsets, and the two stream channels with their start at 51/1024, fade-in by 1/1024
//! per tick up to trunc(Cap x Sound-effects gain), fade-out by 1/1024 per tick and the lowered
//! ceiling applied at once.
//!
//! The bank music (Volume and ramps; Intensity inputs): the playing bank and its volume (up by
//! at most 15 and down by 20 per tick on a change, the stop rows' timed fade), the hero's excite
//! level from the agro count, the adventure counter C, register 2 and the global excite G; and a
//! render engine that runs the sequencer at 240 ticks per second over the synth at 48 kHz.
//!
//! Not covered: reverb, the sound block, overrides, stinger ducking, the adventure zones and the
//! wait for a bank change to settle before a stream starts.

/// Selector tests (Run-time flag): (bit that must be set, bit that must be clear).
fn selector_passes(selector: u32, flag: &dyn Fn(u16) -> bool) -> bool {
    let rule: Option<(u16, Option<u16>)> = match selector {
        1 => Some((83, Some(5))),
        2 => Some((10, Some(322))),
        3 => Some((64, Some(283))),
        4 => Some((57, Some(58))),
        5 => Some((74, Some(263))),
        6 => Some((85, Some(298))),
        7 => Some((92, Some(94))),
        8 => Some((248, None)),
        9 => Some((332, Some(100))),
        10 => Some((332, Some(102))),
        11 => Some((332, Some(104))),
        12 => Some((332, Some(106))),
        13 => return true,
        _ => None,
    };
    rule.is_some_and(|(set, clear)| flag(set) && clear.is_none_or(|c| !flag(c)))
}

/// The run-time flag: progress bit 216 and the selector test.
pub fn runtime_flag(selector: u32, flag: &dyn Fn(u16) -> bool) -> bool {
    flag(216) && selector_passes(selector, flag)
}

/// What a row selects with the flag (None keeps the current one): the bank, stream and block
/// resources; `stop` for a stop row (bank offset below -1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub bank: Option<u32>,
    pub stop: bool,
    pub stream: Option<u32>,
    pub block: Option<u32>,
}

/// Row offsets to resources (Track row): bank + 10 (+ 67 with the flag), stream + 146 (+ 55),
/// block + 395 (the flag does not change it). The flag is added to negative offsets too.
pub fn select(bank: i32, stream: i32, block: i32, flag: bool) -> Selection {
    let bank = bank + if flag { 67 } else { 0 };
    let stream = stream + if flag { 55 } else { 0 };
    Selection {
        bank: (bank >= 0).then(|| (bank + 10) as u32),
        stop: bank < -1,
        stream: (stream != -1 && stream >= 0).then(|| (stream + 146) as u32),
        block: (block >= 0).then(|| (block + 395) as u32),
    }
}

/// One stream channel: its stream resource and volume in 1/1024 units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub stream: Option<u32>,
    pub volume: i32,
}

/// The two stream channels (Streams).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Streams {
    pub channels: [Channel; 2],
}

impl Streams {
    /// One tick of the stream step toward `target` (None keeps the playing stream) with the
    /// ceiling trunc(cap x gain). A stop row fades both channels out.
    pub fn tick(&mut self, target: Option<u32>, stop: bool, cap: i32, gain: f32) {
        let ceiling = (cap as f32 * gain) as i32;
        let target = if stop { None } else { target.or_else(|| self.channels.iter().find(|c| c.volume > 0).and_then(|c| c.stream)) };
        let on = target.and_then(|t| self.channels.iter().position(|c| c.stream == Some(t)));
        let on = match (on, target) {
            (Some(k), _) => Some(k),
            (None, Some(t)) => match self.channels.iter().position(|c| c.stream.is_none()) {
                Some(k) => {
                    self.channels[k] = Channel { stream: Some(t), volume: 51 };
                    Some(k)
                }
                None => None,
            },
            (None, None) => None,
        };
        for (k, c) in self.channels.iter_mut().enumerate() {
            if Some(k) == on {
                if c.volume > ceiling {
                    c.volume = ceiling;
                } else if c.volume < ceiling {
                    c.volume += 1;
                }
            } else if c.stream.is_some() {
                c.volume -= 1;
                if c.volume <= 0 {
                    *c = Channel::default();
                }
            }
        }
    }
}

/// The bank music's game side: the playing bank resource, its volume V units (1/1024 of the
/// bank sound's volume 127), a staged bank waiting for the fade-out, and a restart request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BankMusic {
    pub playing: Option<u32>,
    pub volume: i32,
    staged: Option<u32>,
    /// Set on the tick a bank starts (from child 0).
    pub started: bool,
}

impl BankMusic {
    /// One controller tick for a row's bank offset (already with the flag): -1 keeps, a stop
    /// row (-3600 or -6000 ms) fades out, 0 or more names the bank (offset + 10).
    pub fn tick(&mut self, bank_offset: i32, level: i32, gain: f32) {
        self.started = false;
        let v = (level as f32 * gain) as i32;
        if bank_offset < -1 {
            let fall = (v as f32 / (bank_offset.unsigned_abs() as f32 * 0.06)) as i32;
            self.volume = (self.volume - fall).max(0);
            if self.volume <= 0 {
                self.playing = None;
            }
            return;
        }
        if bank_offset >= 0 {
            let target = (bank_offset + 10) as u32;
            if self.playing != Some(target) {
                self.staged = Some(target);
            }
        }
        if let Some(t) = self.staged {
            if self.playing.is_some() && self.volume > 1 {
                self.volume = (self.volume - 20).max(0);
                return;
            }
            self.playing = Some(t);
            self.staged = None;
            self.volume = 0;
            self.started = true;
        }
        if self.playing.is_some() {
            self.volume += (v - self.volume).clamp(-15, 15);
        }
    }

    /// The volume sent to the driver: min(127, trunc(V x 127 / 1024)).
    pub fn sent(&self) -> i32 {
        (self.volume * 127 / 1024).min(127)
    }
}

/// The intensity inputs (Intensity inputs): the hero's excite float X and level E, the
/// adventure counter C, and the register 2 and G values they give.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Intensity {
    pub x: f32,
    pub e: i32,
    pub c: i32,
    pub register2: u8,
    pub excite: u8,
}

impl Intensity {
    /// The hero's excite update from the agro count k and the hero's health, then the
    /// controller's register 2 and G.
    pub fn tick(&mut self, k: usize, hp: i32, max: i32) {
        let target = match k {
            0 => {
                let h = hp as f32 / (0.15 * max.max(1) as f32);
                if h < 1.0 && 101.0 - h >= 101.0 { 101.0 - h } else { 0.0 }
            }
            1..=3 => 100.0,
            _ => 101.0 - 0.25 * k as f32,
        };
        if target <= 0.0 {
            self.x = 0.0;
        } else if self.x < target {
            self.x = (self.x + 1.15).min(target);
        } else if self.x > target {
            self.x = (self.x - 1.3125).max(target);
        }
        // SetMidiExciteLevel stores n only when 0 <= n < 100.
        let n = self.x as i32;
        if (0..100).contains(&n) {
            self.e = n;
        }
        if self.e >= 1 {
            self.c = 100;
            self.register2 = (self.e / 50 + 1).min(127) as u8;
            self.excite = 100;
        } else {
            self.register2 = 0;
            self.excite = self.c.clamp(0, 100) as u8;
            if self.c > 0 {
                self.c -= 1;
            }
        }
    }

    /// SetMidiAdventureLevel(f): C = trunc(49 + 51 f).
    pub fn adventure(&mut self, f: f32) {
        self.c = (49.0 + 51.0 * f) as i32;
    }
}

/// The music render engine: a bank's instruments and sequencer over the synth, rendered as
/// 48 kHz stereo; one sequencer tick every 200 frames.
pub struct Engine {
    pub instruments: std::sync::Arc<crate::synth::Instruments>,
    pub sequencer: crate::sequencer::Sequencer,
    pub synth: crate::synth::Synth,
    /// The volume sent to the driver (0 to 127).
    pub volume: i32,
    frame: usize,
}

impl Engine {
    pub fn new(instruments: std::sync::Arc<crate::synth::Instruments>, tracks: Vec<crate::sequencer::Track>) -> Self {
        Self { instruments, sequencer: crate::sequencer::Sequencer::new(tracks), synth: crate::synth::Synth::new(), volume: 0, frame: 0 }
    }

    /// Render stereo frames into `out` (interleaved, -1 to 1).
    pub fn render(&mut self, out: &mut [f32]) {
        for pair in out.chunks_mut(2) {
            if self.frame == 0 {
                self.sequencer.tick(&self.instruments, &mut self.synth);
            }
            self.frame = (self.frame + 1) % crate::sequencer::FRAMES_PER_TICK;
            let (l, r) = self.synth.frame(&self.instruments);
            let g = self.volume as f32 / 127.0 / 32768.0;
            pair[0] = (l as f32 * g).clamp(-1.0, 1.0);
            if pair.len() > 1 {
                pair[1] = (r as f32 * g).clamp(-1.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_and_selection() {
        // Pack 678: selector 1 needs bit 83 set and bit 5 clear, plus bit 216.
        let bits = |set: &'static [u16]| move |n: u16| set.contains(&n);
        assert!(runtime_flag(1, &bits(&[216, 83])));
        assert!(!runtime_flag(1, &bits(&[216, 83, 5])));
        assert!(!runtime_flag(1, &bits(&[83])));
        assert!(!runtime_flag(0, &bits(&[216, 83])));
        assert!(runtime_flag(8, &bits(&[216, 248])));
        // Row 5: bank 18, stream 14, block 16.
        assert_eq!(select(18, 14, 16, false), Selection { bank: Some(28), stop: false, stream: Some(160), block: Some(411) });
        assert_eq!(select(18, 14, 16, true), Selection { bank: Some(95), stop: false, stream: Some(215), block: Some(411) });
        assert_eq!(select(3, -1, 1, false).bank, Some(13));
        assert_eq!(select(3, -1, 1, true).bank, Some(80));
        let stop = select(-3600, 5, -3600, false);
        assert!(stop.stop && stop.bank.is_none() && stop.block.is_none());
        assert_eq!(select(-1, -1, -1, false), Selection { bank: None, stop: false, stream: None, block: None });
    }

    #[test]
    fn streams_fade_and_crossfade() {
        let mut s = Streams::default();
        // Cap 700 at gain 1.0: from 51 the ceiling is reached after 649 ticks.
        s.tick(Some(160), false, 700, 1.0);
        assert_eq!(s.channels[0], Channel { stream: Some(160), volume: 52 });
        for _ in 0..648 {
            s.tick(Some(160), false, 700, 1.0);
        }
        assert_eq!(s.channels[0].volume, 700);
        // A lower gain drops to the new ceiling at once (cap 700 x 0.75 = 525).
        s.tick(None, false, 700, 0.75);
        assert_eq!(s.channels[0].volume, 525);
        // A new stream starts on the free channel while the old one fades out.
        s.tick(Some(191), false, 700, 0.75);
        assert_eq!(s.channels[1], Channel { stream: Some(191), volume: 52 });
        assert_eq!(s.channels[0].volume, 524);
        for _ in 0..524 {
            s.tick(Some(191), false, 700, 0.75);
        }
        assert_eq!(s.channels[0], Channel::default());
        // From 51 at gain 0.75 the fade-in to 525 takes 474 ticks.
        let mut t = Streams::default();
        let n = (1..2000).find(|_| {
            t.tick(Some(160), false, 700, 0.75);
            t.channels[0].volume == 525
        });
        assert_eq!(n, Some(474));
        // A stop row fades everything out.
        t.tick(None, true, 700, 0.75);
        assert_eq!(t.channels[0].volume, 524);
    }

    #[test]
    fn bank_volume_ramps() {
        let mut b = BankMusic::default();
        // Row bank offset 18 (bank 28), level 750 at gain 1.0: start at 0, up 15 per tick.
        b.tick(18, 750, 1.0);
        assert_eq!((b.playing, b.started, b.volume), (Some(28), true, 15));
        for _ in 0..60 {
            b.tick(18, 750, 1.0);
        }
        assert_eq!(b.volume, 750);
        // Level 700 at gain 0.75: V = 525, sent as 65.
        let mut c = BankMusic { playing: Some(28), volume: 525, ..Default::default() };
        c.tick(18, 700, 0.75);
        assert_eq!(c.sent(), 65);
        // A bank change: 100 falls 80, 60, 40, 20, 0, then the new bank starts.
        let mut d = BankMusic { playing: Some(28), volume: 100, ..Default::default() };
        let mut seen = Vec::new();
        for _ in 0..6 {
            d.tick(56, 750, 1.0);
            seen.push((d.playing, d.volume));
        }
        assert_eq!(seen[..5].iter().map(|s| s.1).collect::<Vec<_>>(), vec![80, 60, 40, 20, 0]);
        assert_eq!(seen[5], (Some(66), 15));
        // A stop row: -3600 at V 562 falls 2 per tick.
        let mut e = BankMusic { playing: Some(28), volume: 562, ..Default::default() };
        e.tick(-3600, 562, 1.0);
        assert_eq!(e.volume, 560);
    }

    #[test]
    fn intensity_inputs() {
        // Two agro enemies: X climbs 1.15 per tick; E 1 at tick 1, register 2 = 2 from tick 44.
        let mut i = Intensity::default();
        i.tick(2, 150, 150);
        assert_eq!((i.e, i.register2, i.excite), (1, 1, 100));
        let mut t = 1;
        while i.register2 != 2 {
            i.tick(2, 150, 150);
            t += 1;
        }
        assert_eq!(t, 44);
        // The last agro enemy leaves: E is 0 at once; G stays 100 and then counts down.
        i.tick(0, 150, 150);
        assert_eq!((i.e, i.register2, i.excite), (0, 0, 100));
        i.tick(0, 150, 150);
        assert_eq!(i.excite, 99);
        // E stops at 98 while X passes 99 (100 is refused).
        let mut j = Intensity::default();
        for _ in 0..200 {
            j.tick(1, 150, 150);
        }
        assert_eq!(j.e, 98, "X goes 98.9 then 100, which is refused");
        // Adventure levels.
        let mut a = Intensity::default();
        for (f, c) in [(1.0, 100), (0.5, 74), (0.0, 49)] {
            a.adventure(f);
            assert_eq!(a.c, c);
        }
    }
}
