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
}

impl Default for ObjAnim {
    fn default() -> Self {
        Self { state: 3, time: 0.0, speed: 0.0, drawn: false, scripted: false }
    }
}

impl ObjAnim {
    /// InitObjOn and ActivateObj: drawn, at time 0 in state 9, or at the end in state 8.
    pub fn init_on(&mut self, m: &Motion, end: bool) {
        self.scripted = true;
        self.drawn = true;
        if end {
            self.time = m.duration();
            self.state = 8;
        } else {
            self.time = 0.0;
            self.state = 9;
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
    /// finished (8, time = duration), below it stopped (10) at that time.
    pub fn set_time(&mut self, m: &Motion, t: f32) {
        self.scripted = true;
        let d = m.duration();
        if t >= d {
            self.time = d;
            self.state = 8;
        } else {
            self.time = t.max(0.0);
            self.state = 10;
        }
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
        if !matches!(self.state, 0 | 2 | 5 | 6) {
            return;
        }
        let d = m.duration();
        self.time += self.speed / 60.0;
        let mut end = false;
        if self.time < 0.0 {
            self.time = 0.0;
            end = self.speed < 0.0;
        } else if self.time >= d {
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
    }

    /// IsObjAnimating (play-once or loop) and IsObjAnimFinished.
    pub fn animating(&self) -> bool {
        matches!(self.state, 0 | 2 | 5 | 6)
    }

    pub fn finished(&self) -> bool {
        self.state == 8
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
        assert_eq!((o.state, o.time, o.visible()), (9, 0.0, true));
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
        for _ in 0..30 {
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
