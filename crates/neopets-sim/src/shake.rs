//! Camera shake (SPEC-0027 Camera shake, Camera shake record): one timed record started by
//! StartCameraShake or StartCameraShakeMode and stepped once per frame with dt = 1/60 s. Every
//! start clears the elapsed time, the key index, the phase, the finished flag and the whole key
//! list, so each shake plays in full from its own beginning and replaces a running one.

const DT: f32 = 1.0 / 60.0;

/// The shake record: six envelope keys (time, amplitude, rate), the start amplitude, the
/// current amplitude and rate, the aim factor, the two angle terms, the finished flag, the
/// elapsed time, the key index and the wave phase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shake {
    pub keys: [(f32, f32, f32); 6],
    pub duration: f32,
    pub start_amplitude: f32,
    pub amplitude: f32,
    pub rate: f32,
    pub aim: f32,
    pub side: f32,
    pub up: f32,
    pub finished: bool,
    pub elapsed: f32,
    pub key: usize,
    pub phase: f32,
}

impl Default for Shake {
    /// The record as created on first use: zero keys and finished.
    fn default() -> Self {
        Self { keys: [(0.0, 0.0, 0.0); 6], duration: 0.0, start_amplitude: 0.0, amplitude: 0.0, rate: 0.0, aim: 0.0, side: 0.0, up: 0.0, finished: true, elapsed: 0.0, key: 0, phase: 0.0 }
    }
}

/// One frame's movement: the amounts along the frame's level right axis and the world up, and
/// the aim distance when the shaken camera keeps looking at a point ahead (Z above 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub side: f32,
    pub up: f32,
    pub aim: Option<f32>,
}

/// StartCameraShakeMode's presets: (S, p4, R, phi, Z) for a type and duration; None for a type
/// outside 0 to 5.
pub fn preset(kind: i32, duration: f32) -> Option<(f32, f32, f32, f32, f32)> {
    let half = std::f32::consts::FRAC_PI_2;
    Some(match kind {
        0 => (12.0, 0.3, 0.5, half, 0.2),
        1 => (12.0, 0.0, 0.0, half, 0.2),
        2 => (12.0, duration, 0.0, half, 0.2),
        3 => (12.0, 0.3, 0.5, 0.0, 0.2),
        4 => (12.0, 0.3, 0.5, half, 0.8),
        5 => (12.0, 0.3, 0.5, 0.0, half),
        _ => return None,
    })
}

impl Shake {
    /// StartCameraShake(T, A, S, p4, R, phi, Z): the record cleared (elapsed time, key index,
    /// phase, finished flag and all six keys), then keys (0, 0), (p4, A), (T - R, A), (T, 0) at
    /// rate S (keys 4 and 5 stay zero) and the current amplitude 0.
    #[allow(clippy::too_many_arguments)]
    pub fn start(&mut self, t: f32, a: f32, s: f32, p4: f32, r: f32, phi: f32, z: f32) {
        self.keys = [(0.0, 0.0, 0.0); 6];
        self.elapsed = 0.0;
        self.key = 0;
        self.keys[0] = (0.0, 0.0, s);
        self.keys[1] = (p4, a, s);
        self.keys[2] = (t - r, a, s);
        self.keys[3] = (t, 0.0, s);
        self.duration = t;
        self.start_amplitude = a;
        self.amplitude = 0.0;
        self.rate = s;
        self.aim = z;
        self.side = phi.abs().cos();
        self.up = (phi - std::f32::consts::FRAC_PI_2).abs().cos();
        self.finished = false;
        self.phase = 0.0;
    }

    /// StartCameraShakeMode(type, duration, strength): amplitude 0.3 x strength and the preset.
    pub fn start_mode(&mut self, kind: i32, duration: f32, strength: f32) {
        if let Some((s, p4, r, phi, z)) = preset(kind, duration) {
            self.start(duration, 0.3 * strength, s, p4, r, phi, z);
        }
    }

    /// One frame: the elapsed time, the key search, the envelope blend, the end test (that frame
    /// still moves), the phase and the wave. None once finished.
    pub fn step(&mut self) -> Option<Offset> {
        if self.finished {
            return None;
        }
        self.elapsed += DT;
        let e = self.elapsed;
        while self.key + 1 < 6 && self.keys[self.key + 1].0 <= e {
            self.key += 1;
        }
        let j = self.key;
        if j + 1 < 6 {
            let (t0, a0, r0) = self.keys[j];
            let (t1, a1, r1) = self.keys[j + 1];
            if e - t0 > 0.0 && t1 - t0 > 0.0 {
                let f = (e - t0) / (t1 - t0);
                self.amplitude = a0 + (a1 - a0) * f;
                self.rate = r0 + (r1 - r0) * f;
            }
        }
        if e <= 0.0 || e >= self.duration {
            self.finished = true;
        }
        self.phase += DT * self.rate;
        let w = (2.0 * std::f32::consts::PI * self.phase.fract()).sin();
        let a = self.amplitude * w;
        Some(Offset { side: a * self.side, up: a * self.up, aim: (self.aim > 0.0).then(|| 0.5 * self.start_amplitude + 100.0 * (1.0 - self.aim)) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_and_wave() {
        // StartCameraShakeMode(0, 1.0, 1.0): keys (0, 0), (0.3, 0.3), (0.5, 0.3), (1.0, 0).
        let mut s = Shake::default();
        s.start_mode(0, 1.0, 1.0);
        assert_eq!(&s.keys[..4], &[(0.0, 0.0, 12.0), (0.3, 0.3, 12.0), (0.5, 0.3, 12.0), (1.0, 0.0, 12.0)]);
        let mut amps = Vec::new();
        let mut frames = 0;
        while let Some(o) = s.step() {
            frames += 1;
            amps.push(s.amplitude);
            // Preset 0 shakes up and down only (phi = pi/2), and aims 80 units ahead.
            assert!(o.side.abs() < 1e-6);
            assert!((o.aim.unwrap() - 80.15).abs() < 1e-3);
        }
        // The single-precision elapsed sum is just under 1.0 after 60 frames, so the end test
        // passes on frame 61.
        assert_eq!(frames, 61);
        assert!((amps[29] - 0.3).abs() < 1e-3 && amps[59] < 0.01);
        // With S = 12 one wave cycle takes 5 frames (61 frames: 12.2 cycles).
        assert!((s.phase - 12.2).abs() < 1e-3);
    }

    #[test]
    fn short_shake_drops_and_every_shake_plays_in_full() {
        // StartCameraShakeMode(0, 0.5, 1.0): the third key (0.0) is earlier than the second, so
        // at 0.3 s the amplitude drops to 0.3 x (1 - 0.3 / 0.5) = 0.12.
        let mut s = Shake::default();
        s.start_mode(0, 0.5, 1.0);
        let mut a = Vec::new();
        while s.step().is_some() {
            a.push(s.amplitude);
        }
        // Frame 18 (e = 0.3 s) is the first after the drop.
        assert!(a[16] > 0.25 && (a[17] - 0.12).abs() < 1e-4, "{} {}", a[16], a[17]);
        // SPEC-0027: a second shake plays exactly like the first (every start clears the record).
        let first = a.clone();
        s.start_mode(0, 0.5, 1.0);
        assert_eq!((s.key, s.elapsed), (0, 0.0));
        let mut b = Vec::new();
        while s.step().is_some() {
            b.push(s.amplitude);
        }
        assert_eq!(b, first);
        // A start while one runs replaces it at once from elapsed time 0.
        s.start_mode(1, 5.0, 1.0);
        for _ in 0..10 {
            s.step();
        }
        s.start_mode(0, 0.3, 1.0);
        assert_eq!((s.key, s.elapsed), (0, 0.0));
        // Types outside 0 to 5 start nothing.
        assert!(preset(6, 1.0).is_none());
    }
}
