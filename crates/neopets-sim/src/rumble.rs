//! The rumble manager (SPEC-0033 Behavior 1C): presets of four breakpoint times and levels, a
//! request built into up to three linked segments in six envelope entries (merged when they
//! continue each other), and the per-pass step that picks the strongest active entry and sends
//! its level (0 to 255) to pad 0's large motor with a countdown. The presets are read from the
//! boot program at run time (0x2DD270, five records of eight floats).

/// The preset table's address and Tor's leap-landing request words (SPEC-0036 sub-state 13).
pub const PRESETS: u32 = 0x002D_D270;
pub const LANDING_PRESET: u32 = 0x002D_D7AC;
pub const LANDING_SECONDS: u32 = 0x002D_D7B0;
pub const LANDING_MAGNITUDE: u32 = 0x002D_D7B4;

/// A preset: breakpoint times t0 to t3 and levels L0 to L3.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Preset {
    pub t: [f32; 4],
    pub l: [f32; 4],
}

impl Preset {
    /// Five presets from 40 boot words.
    pub fn table(words: &[u32]) -> Vec<Preset> {
        words
            .chunks_exact(8)
            .take(5)
            .map(|w| {
                let f = |k: usize| f32::from_bits(w[k]);
                Preset { t: [f(0), f(1), f(2), f(3)], l: [f(4), f(5), f(6), f(7)] }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Entry {
    intensity: f32,
    step: f32,
    count: i32,
    active: bool,
    next: Option<usize>,
}

/// The manager: six entries and the last level and count sent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rumble {
    entries: [Option<Entry>; 6],
    last: (u8, i32),
}

/// A motor send: the level and the countdown in packets (ticks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Send {
    pub level: u8,
    pub count: i32,
}

impl Rumble {
    /// A request: preset p, duration `seconds` and magnitude `m`. Presets 5 and above do
    /// nothing.
    pub fn request(&mut self, presets: &[Preset], p: usize, seconds: f32, m: f32) {
        let Some(pr) = presets.get(p).filter(|_| p < 5) else { return };
        let mut prev: Option<usize> = None;
        let mut first = true;
        for i in 1..4 {
            // The running difference of t_i x T in 32-bit floats, times 60.
            let n = (60.0 * (pr.t[i] * seconds - pr.t[i - 1] * seconds)) as i32;
            if n <= 0 {
                continue;
            }
            let start = pr.l[i - 1] * m;
            let step = (pr.l[i] - pr.l[i - 1]) * m / n as f32;
            if let Some(k) = prev {
                let e = self.entries[k].as_mut().expect("kept segment");
                let end = e.intensity + e.step * e.count as f32;
                if (end - start).abs() < 0.001 && (e.step - step).abs() < 0.001 {
                    e.count += n;
                    e.step = step;
                    continue;
                }
            }
            let Some(slot) = self.entries.iter().position(|e| e.is_none()) else { continue };
            self.entries[slot] = Some(Entry { intensity: start, step, count: n, active: first, next: None });
            if let Some(k) = prev {
                if let Some(e) = self.entries[k].as_mut() {
                    e.next = Some(slot);
                }
            }
            prev = Some(slot);
            first = false;
        }
    }

    /// One pass (not while the game status is 9: `frozen`). Returns the send, if any; entries
    /// run down whether or not vibration is on (the caller drops the send when it is off).
    pub fn tick(&mut self, frozen: bool) -> Option<Send> {
        if frozen {
            return None;
        }
        let mut choice: Option<usize> = None;
        for k in 0..6 {
            let Some(mut e) = self.entries[k].filter(|e| e.active) else { continue };
            e.count -= 1;
            if e.count < 2 {
                if let Some(n) = e.next {
                    if let Some(x) = self.entries[n].as_mut() {
                        x.active = true;
                    }
                    choice = Some(n);
                }
                self.entries[k] = None;
                continue;
            }
            e.intensity += e.step;
            self.entries[k] = Some(e);
            if choice.is_none_or(|c| self.entries[c].is_none_or(|x| e.intensity > x.intensity)) {
                choice = Some(k);
            }
        }
        let Some(e) = choice.and_then(|c| self.entries[c]) else {
            self.last = (0, 0);
            return None;
        };
        let level = (e.intensity.clamp(0.0, 1.0) * 255.0) as u8;
        let skip = level == self.last.0 && e.count == self.last.1 - 1;
        self.last = (level, e.count);
        (!skip).then_some(Send { level, count: e.count })
    }
}

/// The intensity of an animation-event rumble (SPEC-0033 Behavior 1C source 2): full within
/// the inner radius, falling off to the outer radius, 0 beyond; `d2` the squared distance from
/// the camera to the character.
pub fn falloff(intensity: f32, inner: f32, outer: f32, d2: f32) -> f32 {
    let (ri, ro) = (inner * inner, outer * outer);
    if d2 <= ri {
        intensity
    } else if d2 <= ro {
        intensity * (1.0 - (d2 - ri) / (ri + ro))
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disc's shapes as the spec gives them (synthetic values, not read from the disc).
    fn presets() -> Vec<Preset> {
        let thirds = [0.0, 0.3333, 0.6666, 1.0];
        vec![
            Preset { t: thirds, l: [1.0, 1.0, 1.0, 1.0] },
            Preset { t: thirds, l: [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0] },
            Preset { t: thirds, l: [1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0] },
            Preset { t: [0.0, 0.5, 0.5, 1.0], l: [0.0, 1.0, 1.0, 0.0] },
            Preset { t: thirds, l: [0.0, 1.0, 1.0, 0.0] },
        ]
    }

    #[test]
    fn constant_request_sends_once() {
        // Preset 0, 1.0 s, 0.7: one 58-tick entry; level 178 sent once with countdown 57.
        let mut r = Rumble::default();
        r.request(&presets(), 0, 1.0, 0.7);
        assert_eq!(r.tick(false), Some(Send { level: 178, count: 57 }));
        let sends: Vec<Send> = (0..70).filter_map(|_| r.tick(false)).collect();
        assert!(sends.is_empty(), "{sends:?}");
        // Preset 0, 0.5 s, 0.6923: level 176.
        let mut r = Rumble::default();
        r.request(&presets(), 0, 0.5, 0.6923);
        assert_eq!(r.tick(false).map(|s| s.level), Some(176));
    }

    #[test]
    fn landing_ramp_merges() {
        // Preset 2, 0.8 s, 0.6: one 46-tick ramp from 0.6, step -0.0125: 149, 146, 143, ...
        let mut r = Rumble::default();
        r.request(&presets(), 2, 0.8, 0.6);
        let levels: Vec<u8> = (0..60).filter_map(|_| r.tick(false)).map(|s| s.level).collect();
        assert_eq!(&levels[..3], &[149, 146, 143]);
        assert_eq!(levels.len(), 44);
        assert_eq!(*levels.last().unwrap(), 12);
    }

    #[test]
    fn strongest_wins_then_the_other() {
        let mut r = Rumble::default();
        r.request(&presets(), 0, 1.0, 0.5);
        r.request(&presets(), 0, 0.5, 0.7);
        assert_eq!(r.tick(false).map(|s| s.level), Some(178));
        let next = (0..40).filter_map(|_| r.tick(false)).next();
        assert_eq!(next.map(|s| s.level), Some(127));
        // A preset of 5 does nothing; a frozen pass advances nothing.
        let mut r = Rumble::default();
        r.request(&presets(), 5, 1.0, 1.0);
        assert_eq!(r.tick(false), None);
        r.request(&presets(), 0, 1.0, 0.7);
        assert_eq!(r.tick(true), None);
        assert!(r.tick(false).is_some());
    }

    #[test]
    fn effect_falloff() {
        assert_eq!(falloff(0.8, 10.0, 50.0, 50.0), 0.8);
        let mid = falloff(1.0, 10.0, 50.0, 30.0 * 30.0);
        assert!((mid - (1.0 - 800.0 / 2600.0)).abs() < 1e-6);
        assert_eq!(falloff(1.0, 10.0, 50.0, 60.0 * 60.0), 0.0);
    }
}
