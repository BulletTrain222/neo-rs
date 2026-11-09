//! Animated placed objects (SPEC-0028 section 7.1; SPEC-0030 rows 1 to 4, 7 to 9, 11 and 13):
//! the recorded single-transform motion of a type-29 object, its script states and the per-tick
//! update, sampled with the absolute rule (the per-tick translation path needs a linked locator
//! record, which only the MoveObjToLocator link forms set; they are not built).
//!
//! Not covered: the script offset and extra rotation (mode byte), locator links, the reference
//! centre, the riders' ground attach and the moving walls of the object's parts.

/// Keys per second of a motion stream.
pub const KEY_RATE: f32 = 15.0;

/// A motion stream (ten channels: rotation x, y, z, w; translation x, y, z; scale x, y, z). A
/// constant channel holds one value; a sampled channel holds F values (s16 / 1024). Translation
/// samples are per-key changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Motion {
    pub keys: usize,
    pub channels: [Vec<f32>; 10],
}

impl Motion {
    /// Parse a stream of F keys from its bytes: per channel an s16 flag, then one s16 (flag 0)
    /// or F s16 samples.
    pub fn parse(b: &[u8], keys: usize) -> Option<Self> {
        let mut at = 0;
        let mut read = || -> Option<i16> {
            let v = i16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?);
            at += 2;
            Some(v)
        };
        let mut channels: [Vec<f32>; 10] = Default::default();
        for ch in channels.iter_mut() {
            let n = if read()? == 0 { 1 } else { keys };
            for _ in 0..n {
                ch.push(read()? as f32 / 1024.0);
            }
        }
        Some(Self { keys, channels })
    }

    /// The duration D = (F - 1) / 15 s.
    pub fn duration(&self) -> f32 {
        self.keys.saturating_sub(1) as f32 / KEY_RATE
    }

    fn value(&self, ch: usize, k: usize) -> f32 {
        let c = &self.channels[ch];
        if c.len() == 1 { c[0] } else { c.get(k.min(c.len().saturating_sub(1))).copied().unwrap_or(0.0) }
    }

    /// The absolute sample at `time` (step 4): rotation (not normalised; the second key negated
    /// when the dot product is negative), the root offset (sum of the translation changes to
    /// key k plus f of key k + 1) and the scale.
    pub fn sample(&self, time: f32) -> ([f32; 4], [f32; 3], [f32; 3]) {
        let last = self.keys.saturating_sub(1);
        let k = ((time * 14.999999) as usize).min(last);
        let f = (time - k as f32 * 0.06666667) * 14.999999;
        let k1 = (k + 1).min(last);
        let a: [f32; 4] = std::array::from_fn(|i| self.value(i, k));
        let mut b: [f32; 4] = std::array::from_fn(|i| self.value(i, k1));
        if a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3] < 0.0 {
            b = b.map(|v| -v);
        }
        let q = std::array::from_fn(|i| a[i] + (b[i] - a[i]) * f);
        let root = std::array::from_fn(|i| {
            let ch = 4 + i;
            let sum: f32 = (0..=k).map(|j| self.value(ch, j)).sum();
            if k < last { sum + f * self.value(ch, k + 1) } else { sum }
        });
        let scale = std::array::from_fn(|i| {
            let ch = 7 + i;
            let (s0, s1) = (self.value(ch, k), self.value(ch, k1));
            s0 + (s1 - s0) * f
        });
        (q, root, scale)
    }
}

/// An object's script state (section 7.1 Script states).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjAnim {
    /// 3 untouched (disc), 9 stopped at its start, 8 finished, 2 play-once, 0 loop, 10 stopped,
    /// 4 deactivated.
    pub state: i32,
    pub time: f32,
    pub speed: f32,
    /// The drawn byte (+0x1B6 as InitObjOn and DeactivateObj write it).
    pub drawn: bool,
    /// A script call has touched the object (this build draws untouched objects at rest).
    pub scripted: bool,
    /// The 3 animated-object sound slots (SPEC-0031 Animated-object sounds): cue, start and end
    /// time in seconds; all empty at world setup.
    pub sounds: [Option<(i32, f32, f32)>; 3],
    /// The slots' cue to start and cue to stop from the last advance or set time, for the host
    /// to play on the area slot at the object.
    pub sound_start: Option<i32>,
    pub sound_stop: Option<i32>,
}

impl Default for ObjAnim {
    fn default() -> Self {
        Self { state: 3, time: 0.0, speed: 0.0, drawn: false, scripted: false, sounds: [None; 3], sound_start: None, sound_stop: None }
    }
}

impl ObjAnim {
    /// InitObjOn and ActivateObj (SPEC-0028 state table): drawn, at time 0 in state 10 on a clip
    /// of two or more keys (state 8 on a one-key clip), or at the end in state 8.
    pub fn init_on(&mut self, m: &Motion, end: bool) {
        self.scripted = true;
        self.drawn = true;
        if end {
            self.time = m.duration();
            self.state = 8;
        } else {
            self.time = 0.0;
            self.state = if m.duration() > 0.0 { 10 } else { 8 };
        }
    }

    /// PlayAnim: play once at `speed`; a negative speed first moves to the end.
    pub fn play(&mut self, m: &Motion, speed: f32) {
        self.scripted = true;
        self.state = 2;
        self.speed = speed;
        if speed < 0.0 {
            self.time = m.duration();
        }
    }

    /// LoopAnim: loop at `speed`.
    pub fn looped(&mut self, speed: f32) {
        self.scripted = true;
        self.state = 0;
        self.speed = speed;
    }

    /// SetObjToTime: the time clamped to 0 .. duration; at or past the end the object is
    /// finished (8, time = duration), below it stopped (10) at that time. The sound slots are
    /// checked between the old and the new time.
    pub fn set_time(&mut self, m: &Motion, t: f32) {
        self.scripted = true;
        let d = m.duration();
        let before = self.time;
        if t >= d {
            self.time = d;
            self.state = 8;
        } else {
            self.time = t.max(0.0);
            self.state = 10;
        }
        self.sound_check(before, self.time);
    }

    /// AddAnimObjSound (SPEC-0031 Animated-object sounds): the first empty slot takes the cue,
    /// start max(a, 0.00001) x D and end min(b, 0.99999) x D; with all 3 in use nothing
    /// happens.
    pub fn add_sound(&mut self, m: &Motion, cue: i32, a: f32, b: f32) {
        let d = m.duration();
        if let Some(slot) = self.sounds.iter_mut().find(|s| s.is_none()) {
            *slot = Some((cue, a.max(0.00001) * d, b.min(0.99999) * d));
        }
    }

    /// The slot check between times p and q: the first slot with lo <= start < hi starts its
    /// cue, the first with lo <= end <= hi stops its cue (at most one of each).
    fn sound_check(&mut self, p: f32, q: f32) {
        let (lo, hi) = (p.min(q), p.max(q));
        self.sound_start = self.sounds.iter().flatten().find(|s| lo <= s.1 && s.1 < hi).map(|s| s.0);
        self.sound_stop = self.sounds.iter().flatten().find(|s| lo <= s.2 && s.2 <= hi).map(|s| s.0);
    }

    /// StopAnim: state 10 unless finished (8).
    pub fn stop(&mut self) {
        self.scripted = true;
        if self.state != 8 {
            self.state = 10;
        }
    }

    /// DeactivateObj: state 4, not drawn.
    pub fn deactivate(&mut self) {
        self.scripted = true;
        self.state = 4;
        self.drawn = false;
    }

    /// The per-tick update (steps 1 to 3) for states 0, 2, 5 and 6.
    pub fn tick(&mut self, m: &Motion) {
        self.sound_start = None;
        self.sound_stop = None;
        if !matches!(self.state, 0 | 2 | 5 | 6) {
            return;
        }
        let d = m.duration();
        let before = self.time;
        self.time += self.speed / 60.0;
        let mut end = false;
        // SPEC-0028: forward play takes floor(4 (F - 1) / speed) + 1 advances: reaching the end exactly is not yet finished.
        // Backward play ends on the plain round-up. The margin stands in for the original's truncating float sum.
        if self.time < 1.0e-5 {
            self.time = 0.0;
            end = self.speed < 0.0;
        } else if self.time > d + 1.0e-5 {
            self.time = d;
            end = self.speed > 0.0;
        }
        if end {
            match self.state {
                6 => self.state = 4,
                2 => self.state = 8,
                _ if d > 0.0 => {
                    let mut n = (self.time / d) as i32;
                    if self.speed < 0.0 {
                        n -= 1;
                    }
                    self.time -= n as f32 * d;
                }
                _ => {}
            }
        }
        // After the end clamp and the loop wrap (SPEC-0031 Animated-object sounds).
        self.sound_check(before, self.time);
    }

    /// IsObjAnimating (play-once or loop) and IsObjAnimFinished.
    pub fn animating(&self) -> bool {
        matches!(self.state, 0 | 2 | 5 | 6)
    }

    /// IsObjAnimFinished: true in states 4 (after DeactivateObj or a play-and-hide) and 8.
    pub fn finished(&self) -> bool {
        matches!(self.state, 4 | 8)
    }

    /// The draw gate (section 10): with the gate byte 1, drawn while the state is not 3 or 4.
    pub fn visible(&self) -> bool {
        !matches!(self.state, 3 | 4)
    }
}

/// The rotation matrix of a quaternion (x, y, z, w) as columns (local to world), unnormalised
/// as the original uses it.
pub fn rotation(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
        [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)],
        [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_object_sound_vectors() {
        // SPEC-0031 vectors: F = 31 keys (D = 2.0 s); slot (cue 5, 0.25, 1.0): start 0.5, end
        // 1.99998.
        let m = Motion { keys: 31, channels: Default::default() };
        let mut o = ObjAnim::default();
        o.add_sound(&m, 5, 0.25, 1.0);
        let (cue, start, end) = o.sounds[0].unwrap();
        assert_eq!(cue, 5);
        assert!((start - 0.5).abs() < 1e-6 && (end - 1.99998).abs() < 1e-5);
        o.sound_check(0.49, 0.51);
        assert_eq!((o.sound_start, o.sound_stop), (Some(5), None));
        o.sound_check(1.99, 2.0);
        assert_eq!((o.sound_start, o.sound_stop), (None, Some(5)));
        // The loop wrap: lo 0.0067, hi 1.99 holds the start again.
        o.sound_check(1.99, 0.0067);
        assert_eq!(o.sound_start, Some(5));
        // a = 0 starts on the first tick from 0; a = 1.0 never plays.
        let mut z = ObjAnim::default();
        z.add_sound(&m, 7, 0.0, 1.0);
        z.sound_check(0.0, 0.01667);
        assert_eq!(z.sound_start, Some(7));
        let mut never = ObjAnim::default();
        never.add_sound(&m, 0, 1.0, 1.0);
        never.sound_check(1.9, 2.0);
        assert_eq!(never.sound_start, None);
    }

    /// A 16-key lift: rotation constant (0, 0, 0, 1), y rising 0.5 per key after key 0, scale 1.
    fn lift() -> Motion {
        let mut b = Vec::new();
        let mut put = |v: i16| b.extend(v.to_le_bytes());
        for (i, v) in [0i16, 0, 0, 1024].iter().enumerate() {
            let _ = i;
            put(0);
            put(*v);
        }
        put(0);
        put(0);
        put(1);
        put(0);
        for _ in 1..16 {
            put(512);
        }
        put(0);
        put(0);
        for _ in 0..3 {
            put(0);
            put(1024);
        }
        Motion::parse(&b, 16).unwrap()
    }

    #[test]
    fn stream_and_absolute_sample() {
        let m = lift();
        assert_eq!(m.duration(), 1.0);
        let (q, root, scale) = m.sample(0.0);
        assert_eq!((q, root, scale), ([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0; 3]));
        // At the end the root is the sum of the 15 changes, 7.5.
        assert!((m.sample(1.0).1[1] - 7.5).abs() < 1e-4);
        // Half a key past key 2: 1.0 + 0.5 x 0.5.
        let t = 2.5 / 15.0;
        assert!((m.sample(t).1[1] - 1.25).abs() < 1e-3);
    }

    #[test]
    fn script_states_and_ticks() {
        let m = lift();
        let mut o = ObjAnim::default();
        assert!(!o.visible());
        o.init_on(&m, false);
        // State 10 on a clip of two or more keys, not finished (SPEC-0028 state table).
        assert_eq!((o.state, o.time, o.visible(), o.finished()), (10, 0.0, true, false));
        let mut gone = o;
        gone.deactivate();
        assert!(gone.finished(), "IsObjAnimFinished is true after DeactivateObj");
        // One second of play at speed 1 ends on tick 60 or 61 (single-precision sums).
        o.play(&m, 1.0);
        let mut n = 0;
        while o.animating() {
            o.tick(&m);
            n += 1;
        }
        assert!((60..=61).contains(&n) && o.finished() && o.time == 1.0, "{n}");
        // Backwards from the end.
        o.play(&m, -1.0);
        assert_eq!(o.time, 1.0);
        let mut n = 0;
        while o.animating() {
            o.tick(&m);
            n += 1;
        }
        assert!((60..=61).contains(&n) && o.finished() && o.time == 0.0, "{n}");
        // A loop wraps to the start.
        o.looped(2.0);
        for _ in 0..31 {
            o.tick(&m);
        }
        assert_eq!(o.state, 0);
        assert!(o.time < 0.05, "{}", o.time);
        o.stop();
        assert_eq!(o.state, 10);
        o.deactivate();
        assert!(!o.visible() && !o.drawn);
    }
}
