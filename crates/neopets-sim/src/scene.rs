//! In-engine scene staging (SPEC-0052; SPEC-0027 Scripted cameras and Cutscene camera tracks):
//! camera track sampling, the camera play with its run state and end modes, and the screen
//! effect's fade. Times are accumulated in single precision as the spec's counts require.

/// One track key: quaternion (x, y, z, w), position and focal length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub q: [f32; 4],
    pub p: [f32; 3],
    pub focal: f32,
}

/// A camera track: the head (used with no keys), the key interval and the keys.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub head: Key,
    pub dt: f32,
    pub per_key_focal: bool,
    pub keys: Vec<Key>,
}

/// A sampled camera pose: the rows right, up and forward (the camera looks along forward),
/// the eye position and the focal length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub rows: [[f32; 3]; 3],
    pub eye: [f32; 3],
    pub focal: f32,
}

fn matrix(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [x * x + w * w - y * y - z * z, 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
        [2.0 * (x * y + w * z), y * y + w * w - x * x - z * z, 2.0 * (y * z - w * x)],
        [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), z * z + w * w - x * x - y * y],
    ]
}

/// SPEC-0052 Sampling a track at track time t: the head with no keys; None past the last pair.
pub fn sample(track: &Track, t: f32) -> Option<Pose> {
    if track.keys.is_empty() {
        return Some(Pose { rows: matrix(track.head.q), eye: track.head.p, focal: track.head.focal });
    }
    let i = (t / track.dt) as usize;
    if i + 1 >= track.keys.len() {
        return None;
    }
    let f = (t - i as f32 * track.dt) / track.dt;
    let (a, mut b) = (track.keys[i], track.keys[i + 1]);
    let dot: f32 = (0..4).map(|k| a.q[k] * b.q[k]).sum();
    if dot < 0.0 {
        b.q = b.q.map(|v| -v);
    }
    let q = [0, 1, 2, 3].map(|k| (1.0 - f) * a.q[k] + f * b.q[k]);
    let eye = [0, 1, 2].map(|k| (1.0 - f) * a.p[k] + f * b.p[k]);
    let focal = if track.per_key_focal { (1.0 - f) * a.focal + f * b.focal } else { track.head.focal };
    Some(Pose { rows: matrix(q), eye, focal })
}

/// What an update of the camera play did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayUpdate {
    /// The camera is posed from the track.
    Posed(Pose),
    /// The play ended this update; `release` is whether the end mode frees the hero (modes 0
    /// and 2; mode 0 only from a non-scripted state, which the caller judges).
    Ended { idle: bool, release: bool },
    /// Ended earlier and held (end mode 1), or idle.
    Held,
}

/// A running camera play (SPEC-0027 Scripted cameras): the track id, its time, run state
/// (1 playing, 2 ended, 0 idle), speed factor and end mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPlay {
    pub track: usize,
    pub t: f32,
    pub state: u8,
    pub speed: f32,
    pub end_mode: u8,
    /// The last pose sampled (held after an end-mode-1 end).
    pub last: Option<Pose>,
}

impl CameraPlay {
    pub fn new(track: usize, end_mode: u8, speed: f32) -> Self {
        Self { track, t: 0.0, state: 1, speed, end_mode, last: None }
    }

    /// One camera update: sample at t; a valid sample poses the camera and t advances by
    /// (1/60) x speed; no sample ends the play and its end mode acts.
    pub fn update(&mut self, tracks: &[Track]) -> PlayUpdate {
        if self.state != 1 {
            return PlayUpdate::Held;
        }
        let Some(track) = tracks.get(self.track) else {
            self.state = 0;
            return PlayUpdate::Ended { idle: true, release: false };
        };
        match sample(track, self.t) {
            Some(pose) => {
                self.t += (1.0 / 60.0) * self.speed;
                self.last = Some(pose);
                PlayUpdate::Posed(pose)
            }
            None => {
                self.state = 2;
                match self.end_mode {
                    1 => PlayUpdate::Ended { idle: false, release: false },
                    _ => {
                        self.state = 0;
                        PlayUpdate::Ended { idle: true, release: true }
                    }
                }
            }
        }
    }

    /// The WAIT test of the play calls: the caller waits while the play is neither ended nor
    /// idle (and only on a track with keys).
    pub fn waiting(&self) -> bool {
        self.state == 1
    }
}

/// Frames for a script time in seconds: max(1, trunc(s x 60)), the product one console
/// single-precision multiply (SPEC-0008 Seconds to ticks: 0.7 s gives 41).
pub fn frames(seconds: f32) -> u32 {
    (ps2_float::fpu::mul(seconds, 60.0) as i64).max(1) as u32
}

/// The screen effect's fade (SPEC-0052 Screen effect): colour, alpha 0 to 128, target, step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fade {
    pub colour: [u8; 3],
    pub alpha: f32,
    pub target: f32,
    pub step: f32,
}

impl Fade {
    /// Fade out over n frames: alpha 0, target 128, step 128 / n.
    pub fn out(&mut self, colour: [u8; 3], n: u32) {
        *self = Fade { colour, alpha: 0.0, target: 128.0, step: 128.0 / n as f32 };
    }

    /// Fade in over n frames: alpha 128, target 0, step -128 / n.
    pub fn fade_in(&mut self, colour: [u8; 3], n: u32) {
        *self = Fade { colour, alpha: 128.0, target: 0.0, step: -128.0 / n as f32 };
    }

    /// The per-pass update (step 3): add the step, clamp at the target past it, then to 0 to
    /// 128; at 0 the colour returns to black.
    pub fn update(&mut self) {
        if self.step != 0.0 {
            self.alpha += self.step;
            if (self.step > 0.0 && self.alpha > self.target) || (self.step < 0.0 && self.alpha < self.target) {
                self.alpha = self.target;
                self.step = 0.0;
            }
        }
        self.alpha = self.alpha.clamp(0.0, 128.0);
        if self.alpha == 0.0 {
            self.colour = [0; 3];
        }
    }

    /// The fade WAIT: alpha differs from its target.
    pub fn moving(&self) -> bool {
        self.alpha != self.target
    }

    /// The drawn alpha, 0 to 1 (round(alpha) / 128).
    pub fn opacity(&self) -> f32 {
        self.alpha.round() / 128.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(p: [f32; 3], q: [f32; 4]) -> Key {
        Key { q, p, focal: 791.448 }
    }

    #[test]
    fn synthetic_sampling() {
        // SPEC-0027: 2 keys at 1.0 s, positions 0 and (10, 0, 0), quaternions (0,0,0,1) and
        // (0,0,0,-1): at t 0.25 (2.5, 0, 0) and the identity; at t 1.0 no sample.
        let t = Track { head: key([0.0; 3], [0.0, 0.0, 0.0, 1.0]), dt: 1.0, per_key_focal: false, keys: vec![key([0.0; 3], [0.0, 0.0, 0.0, 1.0]), key([10.0, 0.0, 0.0], [0.0, 0.0, 0.0, -1.0])] };
        let s = sample(&t, 0.25).unwrap();
        assert_eq!(s.eye, [2.5, 0.0, 0.0]);
        assert_eq!(s.rows, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        assert_eq!(sample(&t, 1.0), None);
        // N = 1: never a sample; N = 0: the head at any time.
        let one = Track { keys: vec![key([0.0; 3], [0.0, 0.0, 0.0, 1.0])], ..t.clone() };
        assert_eq!(sample(&one, 0.0), None);
        let none = Track { keys: Vec::new(), head: key([1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0]), ..t };
        assert_eq!(sample(&none, 99.0).unwrap().eye, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn play_counts_in_single_precision() {
        // SPEC-0027 Counts: N 30, dt 1/15 (bit pattern 0x3D888889): at speed 1.0, 117 posed
        // updates and the end on update 118; at 0.5, 233 and 234; at 2.0, 59 and 60.
        let dt = f32::from_bits(0x3D88_8889);
        let keys: Vec<Key> = (0..30).map(|i| key([i as f32, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])).collect();
        let tracks = vec![Track { head: keys[0], dt, per_key_focal: false, keys }];
        for (speed, posed, ends) in [(1.0, 117, 118), (0.5, 233, 234), (2.0, 59, 60)] {
            let mut play = CameraPlay::new(0, 1, speed);
            let mut n = 0;
            let end = loop {
                n += 1;
                match play.update(&tracks) {
                    PlayUpdate::Posed(_) => {}
                    PlayUpdate::Ended { idle, .. } => {
                        assert!(!idle, "end mode 1 holds");
                        break n;
                    }
                    PlayUpdate::Held => unreachable!(),
                }
            };
            assert_eq!((end - 1, end), (posed, ends), "speed {speed}");
            assert_eq!(play.update(&tracks), PlayUpdate::Held);
            assert!(play.last.is_some());
        }
        // End mode 2 goes idle and releases the hero.
        let mut play = CameraPlay::new(0, 2, 2.0);
        let end = (0..100).find_map(|_| match play.update(&tracks) {
            PlayUpdate::Ended { idle, release } => Some((idle, release)),
            _ => None,
        });
        assert_eq!(end, Some((true, true)));
        assert_eq!(play.state, 0);
    }

    #[test]
    fn fade_vectors() {
        // SPEC-0052: frames 0.3 s 18, 0.5 s 30, 0.7 s 41, 0 s 1.
        assert_eq!([0.3, 0.5, 0.7, 0.0].map(frames), [18, 30, 41, 1]);
        // FadeScreenOut(0.7 s): alpha below 128 after update 41, clamped on update 42.
        let mut f = Fade::default();
        f.out([0; 3], frames(0.7));
        let n = (1..100).find(|_| {
            f.update();
            !f.moving()
        });
        assert_eq!(n, Some(42));
        // Fade in from white over 8 frames: 112, 96, ... then 0 and black on update 8.
        let mut f = Fade::default();
        f.fade_in([255; 3], 8);
        let seen: Vec<f32> = (0..8)
            .map(|_| {
                f.update();
                f.alpha
            })
            .collect();
        assert_eq!(seen, vec![112.0, 96.0, 80.0, 64.0, 48.0, 32.0, 16.0, 0.0]);
        assert_eq!(f.colour, [0; 3]);
        // FadeScreenIn(1.0 s) from 128: the clamp to 0 comes on update 61.
        let mut f = Fade::default();
        f.fade_in([0; 3], frames(1.0));
        let n = (1..100).find(|_| {
            f.update();
            !f.moving()
        });
        assert_eq!(n, Some(61));
    }
}
