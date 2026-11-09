//! Animation playback records (SPEC-0005, Playback and Layer step): a clip cursor in frames that
//! advances by a fixed speed each tick, the end rule for one-shot and looping clips, the key
//! pair and fraction to sample, and the fade-in that crossfades from the pose shown last.
//! Sampling the clip data itself is the data crate's job; this only decides what to sample.

/// What playback needs to know about a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipInfo {
    pub frames: u32,
    pub looping: bool,
    /// Speed byte S (entry byte 0x2A).
    pub speed_byte: u8,
    /// Fade-in length in ticks (entry byte 0x24).
    pub fade_in: u8,
}

/// The keys to sample this tick, and how to combine the sample into the pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub key: u32,
    pub next_key: u32,
    pub fraction: f32,
    /// Blend factor φ for a sample mixed into the pose shown last.
    pub blend: f32,
    /// Weight exactly 1.0: the sample replaces the pose.
    pub replace: bool,
    /// The loop wrapped this tick (cursor at or past F − 1 on a looping clip).
    pub wrapped: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaybackRecord {
    clip: Option<ClipInfo>,
    cursor: f32,
    fraction: f32,
    speed: f32,
    finished: bool,
    weight: f32,
    fade_step: f32,
    fade_count: u8,
    fading: bool,
}

impl Default for PlaybackRecord {
    /// Idle: no clip, finished, weight 0, speed 0.5.
    fn default() -> Self {
        Self {
            clip: None,
            cursor: 0.0,
            fraction: 0.0,
            speed: 0.5,
            finished: true,
            weight: 0.0,
            fade_step: 1.0,
            fade_count: 0,
            fading: false,
        }
    }
}

/// The console's float add truncates toward zero (SPEC-0008).
fn add_truncated(a: f32, b: f32) -> f32 {
    let exact = a as f64 + b as f64;
    let near = exact as f32;
    if (near as f64).abs() > exact.abs() { f32::from_bits(near.to_bits() - 1) } else { near }
}

/// Frames per tick at play rate `rate`: R × 0.5 ÷ S, or R × 0.5 when S is 0.
pub fn clip_speed(speed_byte: u8, rate: f32) -> f32 {
    if speed_byte == 0 {
        rate * 0.5
    } else {
        // SPEC-0005 Single-precision caveat: the console's division truncates, so 0.5 / 3 lies just below 1/6 and a clip with
        // S = 3 reaches its last key one advance late.
        let exact = rate as f64 * 0.5 / speed_byte as f64;
        let near = exact as f32;
        if near as f64 > exact { f32::from_bits(near.to_bits() - 1) } else { near }
    }
}

impl PlaybackRecord {
    /// Start a clip: cursor 0, weight 0 fading in over the clip's fade-in length.
    pub fn start(&mut self, clip: ClipInfo, rate: f32) {
        let fade = clip.fade_in.max(1);
        *self = Self {
            clip: Some(clip),
            cursor: 0.0,
            fraction: 0.0,
            speed: clip_speed(clip.speed_byte, rate),
            finished: false,
            weight: 0.0,
            fade_step: 1.0 / fade as f32,
            fade_count: fade,
            fading: true,
        };
    }

    /// Recompute the speed from a new play rate without restarting (heroes do this every pass;
    /// SPEC-0005 Playback 1a).
    pub fn set_rate(&mut self, rate: f32) {
        if let Some(c) = self.clip {
            self.speed = clip_speed(c.speed_byte, rate);
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn clip(&self) -> Option<ClipInfo> {
        self.clip
    }

    pub fn cursor(&self) -> f32 {
        self.cursor
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// Progress (c + t) / F, 1.0 when finished or idle (SPEC-0005, Clip progress).
    pub fn progress(&self) -> f32 {
        match self.clip {
            Some(c) if !self.finished && c.frames > 0 => (self.cursor + self.fraction) / c.frames as f32,
            _ => 1.0,
        }
    }

    /// One tick: end rule, key pair and fraction, fade step, then cursor advance. Returns the
    /// sample to take, or None when nothing is sampled (no clip, speed 0, or weight 0).
    pub fn tick(&mut self) -> Option<Sample> {
        let clip = self.clip?;
        if self.speed == 0.0 || clip.frames == 0 {
            return None;
        }
        let f = clip.frames as f32;
        let mut wrapped = false;
        // A looping main layer wraps only past F - 1 (a cursor of exactly F - 1 is left alone and
        // sets no wrap flag); the equality path is the one-shot's (SPEC-0005 Layer step,
        // confirmed 2026-11-09).
        if self.cursor > f - 1.0 || (self.cursor == f - 1.0 && !clip.looping) {
            if clip.looping {
                wrapped = true;
                self.cursor -= f;
                if self.cursor < 0.0 {
                    self.cursor += f;
                }
            } else {
                self.cursor = f - 1.0;
                self.finished = true;
            }
        }
        if self.cursor < 0.0 {
            if clip.looping {
                self.cursor += f;
            } else {
                self.cursor = 0.0;
                self.finished = true;
            }
        }
        let key = self.cursor.trunc() as u32;
        self.fraction = self.cursor - self.cursor.trunc();
        let next_key = if self.speed > 0.0 {
            (key + 1) % clip.frames
        } else {
            (key.wrapping_sub(1)) % clip.frames
        };
        let blend = if self.fading {
            let phi = self.weight + self.fade_step * (1.0 - self.weight);
            self.weight += self.fade_step;
            self.fade_count -= 1;
            if self.fade_count == 0 {
                self.fading = false;
                self.weight = 1.0;
            }
            phi
        } else {
            self.weight
        };
        let sample = (self.weight != 0.0).then_some(Sample {
            key,
            next_key,
            fraction: self.fraction,
            blend,
            replace: self.weight == 1.0,
            wrapped,
        });
        if !self.finished {
            self.cursor = add_truncated(self.cursor, self.speed);
        }
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(frames: u32, looping: bool, s: u8, fade: u8) -> ClipInfo {
        ClipInfo { frames, looping, speed_byte: s, fade_in: fade }
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn speed_rule_matches_spec_vectors() {
        assert!(clip_speed(3, 1.0) < 1.0 / 6.0 && (clip_speed(3, 1.0) - 1.0 / 6.0).abs() < 1e-6);
        // SPEC-0005: at R = 1 a one-shot S = 3 clip takes 6 (F - 1) + 1 advances.
        for (keys, advances) in [(1.0f32, 7), (10.0, 61)] {
            let (mut c, mut n) = (0.0f32, 0);
            while c < keys {
                c = add_truncated(c, clip_speed(3, 1.0));
                n += 1;
            }
            assert_eq!(n, advances, "{keys}");
        }
        assert_eq!(clip_speed(2, 1.0), 0.25);
        assert_eq!(clip_speed(1, 1.0), 0.5);
        assert_eq!(clip_speed(0, 1.0), 0.5);
        assert_eq!(clip_speed(4, 1.0), 0.125);
        assert!((clip_speed(3, 0.5) - 1.0 / 12.0).abs() < 1e-6);
        assert!(close(clip_speed(2, 0.6), 0.15));
    }

    #[test]
    fn cursor_reaches_one_after_six_ticks_at_s3() {
        let mut r = PlaybackRecord::default();
        r.start(clip(20, true, 3, 1), 1.0);
        for _ in 0..6 {
            r.tick();
        }
        assert!(close(r.cursor(), 1.0));
    }

    fn at(cursor: f32, looping: bool) -> (PlaybackRecord, Option<Sample>) {
        let mut r = PlaybackRecord::default();
        r.start(clip(19, looping, 2, 1), 1.0);
        r.cursor = cursor;
        let s = r.tick();
        (r, s)
    }

    #[test]
    fn looping_end_rule_keeps_the_overshoot() {
        let (_, s) = at(18.5, true);
        let s = s.unwrap();
        assert_eq!((s.key, s.next_key, s.fraction, s.wrapped), (18, 0, 0.5, true));
        let s = at(19.25, true).1.unwrap();
        assert_eq!((s.key, s.next_key, s.fraction), (0, 1, 0.25));
        let s = at(19.0, true).1.unwrap();
        assert_eq!((s.key, s.fraction), (0, 0.0));
        // Exactly F - 1 on a looping clip: left alone, no wrap flag.
        let s = at(18.0, true).1.unwrap();
        assert_eq!((s.key, s.next_key, s.wrapped), (18, 0, false));
    }

    #[test]
    fn one_shot_holds_its_last_key() {
        let (r, s) = at(18.5, false);
        let s = s.unwrap();
        assert_eq!((s.key, s.fraction), (18, 0.0));
        assert!(r.finished());
        assert_eq!(r.cursor(), 18.0, "a finished clip does not advance");
        assert_eq!(r.progress(), 1.0);
    }

    #[test]
    fn fade_in_four_matches_spec() {
        let mut r = PlaybackRecord::default();
        r.start(clip(20, true, 2, 4), 1.0);
        let phis: Vec<f32> = (0..5).map(|_| r.tick().unwrap().blend).collect();
        for (got, want) in phis.iter().zip([0.25, 0.4375, 0.625, 0.8125, 1.0]) {
            assert!(close(*got, want), "{phis:?}");
        }
        assert_eq!(r.weight(), 1.0);
        // From the fifth tick the weight is exactly 1.0 and the sample overwrites.
        assert!(r.tick().unwrap().replace);
    }

    #[test]
    fn fade_in_sixteen_matches_tor_walk_to_run() {
        let mut r = PlaybackRecord::default();
        r.start(clip(14, true, 1, 16), 1.0);
        let phis: Vec<f32> = (0..16).map(|_| r.tick().unwrap().blend).collect();
        assert!(close(phis[1], 0.1211) && close(phis[15], 0.9414), "{phis:?}");
        assert!(r.tick().unwrap().replace);
    }

    #[test]
    fn speed_zero_samples_nothing_and_holds_the_cursor() {
        let mut r = PlaybackRecord::default();
        r.start(clip(20, true, 2, 1), 0.0);
        assert_eq!(r.tick(), None);
        assert_eq!(r.cursor(), 0.0);
        assert_eq!(PlaybackRecord::default().tick(), None);
    }
}
