//! The music controller's stream step (SPEC-0009 Music selection): the run-time flag (progress
//! bit 216 and the pack's selector test), the row's bank, stream and block resources with the
//! flag's offsets, and the two stream channels with their start at 51/1024, fade-in by 1/1024
//! per tick up to trunc(Cap x Sound-effects gain), fade-out by 1/1024 per tick and the lowered
//! ceiling applied at once.
//!
//! Not covered: the bank music (the sequencer of SPEC-0031 section 8), its volume ramps and
//! intensity inputs, reverb, the sound block, overrides and the wait for a bank change to settle.

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
}
