//! The game side of the sound blocks (SPEC-0031 Cue playback, Sound slots and their blocks,
//! Positional volume and pan, keep-alive and loop voices): the 25 bank slots and the block each
//! wants, the cue call with its checks, the 48-voice pool, the same-voice test, the hearing radii,
//! the positional volume and the pan angle (the original's sum, kept as written), the per-pass
//! voice update with the keep-alive counter, and the slot block rules for the hero's weapon,
//! shield and power. Requests for the driver are queued for the audio side.

/// Slot numbers used here (the original's slot numbers are run-time choices; these are fixed).
pub const MAIN: usize = 0;
pub const VOICE: usize = 1;
pub const WEAPON: usize = 2;
pub const SHIELD: usize = 3;
pub const POWER: usize = 4;
pub const AREA: usize = 5;
pub const SLOTS: usize = 25;
pub const POOL: usize = 48;
/// The manager's hearing radius (set at sound-system start).
pub const RADIUS: f32 = 200.0;

/// A request for the driver side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Start cue `cue` of block `block` as instance `id` at volume and pan (None: cue default).
    Play { id: usize, block: u32, cue: i32, volume: i32, pan: Option<i32> },
    Set { id: usize, volume: i32, pan: Option<i32> },
    Stop { id: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    OneShot,
    Loop,
    KeepAlive,
}

/// The listener: the camera position plus 5 x its forward axis, and the camera's rows (right,
/// up, forward).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Listener {
    pub point: [f32; 3],
    pub rows: [[f32; 3]; 3],
}

impl Listener {
    pub fn from_camera(position: [f32; 3], rows: [[f32; 3]; 3]) -> Self {
        let f = rows[2];
        Self { point: [position[0] + 5.0 * f[0], position[1] + 5.0 * f[1], position[2] + 5.0 * f[2]], rows }
    }

    /// The positional volume of base volume `v` with radii `r` and `r0`: None out of range.
    pub fn volume(&self, emitter: [f32; 3], v: i32, r: f32, r0: f32) -> Option<i32> {
        let d = (0..3).map(|k| (emitter[k] - self.point[k]).powi(2)).sum::<f32>().sqrt();
        if d >= r {
            return None;
        }
        let e = d - r0;
        let out = if e < 0.0 {
            v
        } else {
            let s = if r0 < r { r - r0 } else { r };
            (v as f32 * (1.0 - e / s)) as i32
        };
        Some(out)
    }

    /// The pan angle: u = v.x row0 + v.y row1 + v.z row2 (as the original sums it), phi from u,
    /// pan = trunc((1 - (phi / pi + 0.5) / 2) x 359).
    pub fn pan(&self, emitter: [f32; 3]) -> i32 {
        let v = [emitter[0] - self.point[0], emitter[1] - self.point[1], emitter[2] - self.point[2]];
        let r = &self.rows;
        let u: [f32; 3] = std::array::from_fn(|k| v[0] * r[0][k] + v[1] * r[1][k] + v[2] * r[2][k]);
        let pi = std::f32::consts::PI;
        let phi = if u[0].abs() < 1e-6 {
            if u[2] >= 0.0 { pi / 2.0 } else { -pi / 2.0 }
        } else {
            let a = (u[2] / u[0]).atan();
            if u[0] < 0.0 { a + pi } else { a }
        };
        ((1.0 - (phi / pi + 0.5) / 2.0) * 359.0) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Voice {
    id: usize,
    slot: usize,
    block: u32,
    cue: i32,
    emitter: Option<[f32; 3]>,
    base: i32,
    r: f32,
    r0: f32,
    mode: Mode,
    counter: i32,
}

/// How a cue is played.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Play {
    pub emitter: Option<[f32; 3]>,
    pub do_not_restart: bool,
    pub keep_alive: bool,
    pub looped: bool,
    /// Volume (1,024 = 1.0); None for the default.
    pub volume: Option<i32>,
    /// Hearing and full-volume radii (R, r0); None for the manager radius and 0.
    pub radius: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct SoundManager {
    /// The block each slot wants, and the blocks the audio side has loaded.
    pub slots: [Option<u32>; SLOTS],
    pub loaded: std::collections::HashSet<u32>,
    voices: Vec<Voice>,
    next: usize,
    /// The default cue volume (trunc(row Scale x Sound-effects gain); 1,024 at set-up).
    pub default_volume: i32,
    pub requests: Vec<Request>,
    pub listener: Listener,
}

impl Default for SoundManager {
    fn default() -> Self {
        let mut slots = [None; SLOTS];
        slots[MAIN] = Some(347);
        Self { slots, loaded: Default::default(), voices: Vec::new(), next: 1, default_volume: 1024, requests: Vec::new(), listener: Listener::default() }
    }
}

/// The weapon slot's block: Tor 461 + (sword - 98), Roberta 469 + (wand - 111); 466 when the
/// index is negative or there is no weapon.
pub fn weapon_block(roberta: bool, weapon: Option<u16>) -> u32 {
    let base = if roberta { (469, 111) } else { (461, 98) };
    match weapon.map(|w| w as i32 - base.1) {
        Some(k) if k >= 0 => (base.0 + k) as u32,
        _ => 466,
    }
}

/// The shield slot's block: Tor 455 + (shield - 105), 456 with none; Roberta always 459.
pub fn shield_block(roberta: bool, shield: Option<u16>) -> u32 {
    if roberta {
        return 459;
    }
    match shield {
        Some(s) => (455 + s as i32 - 105) as u32,
        None => 456,
    }
}

/// The power slot's block: 477 + (weapon Mote - 160), 485 with none.
pub fn power_block(mote: Option<u16>) -> u32 {
    match mote.map(|m| m as i32 - 160) {
        Some(k) if k >= 0 => (477 + k) as u32,
        _ => 485,
    }
}

impl SoundManager {
    /// The cue call (Cue playback rules 2 to 10). Returns whether a voice started.
    pub fn play(&mut self, slot: usize, cue: i32, how: Play) -> bool {
        let Some(block) = self.slots.get(slot).copied().flatten() else { return false };
        if !self.loaded.contains(&block) || cue < 0 {
            return false;
        }
        let base = how.volume.unwrap_or(self.default_volume);
        if base < 1 {
            return false;
        }
        let matches = |v: &Voice| v.block == block && v.cue == cue && (how.emitter.is_none() || v.emitter == how.emitter);
        if how.do_not_restart && self.voices.iter().any(matches) {
            return false;
        }
        let mode = if how.keep_alive {
            Mode::KeepAlive
        } else if how.looped {
            Mode::Loop
        } else {
            Mode::OneShot
        };
        if self.voices.len() >= POOL {
            return false;
        }
        let (r, r0) = how.radius.filter(|x| x.0 > 0.0).unwrap_or((RADIUS, 0.0));
        let (volume, pan) = match how.emitter {
            Some(e) => match self.listener.volume(e, base, r, r0) {
                Some(v) if v > 0 => (v, Some(self.listener.pan(e))),
                _ => return false,
            },
            None => (base, None),
        };
        let id = self.next;
        self.next += 1;
        self.voices.push(Voice { id, slot, block, cue, emitter: how.emitter, base, r, r0, mode, counter: 4 });
        self.requests.push(Request::Play { id, block, cue, volume, pan });
        true
    }

    /// The same-voice test for a slot, cue and emitter.
    pub fn has(&self, slot: usize, cue: i32, emitter: Option<[f32; 3]>) -> bool {
        let block = self.slots.get(slot).copied().flatten();
        self.voices.iter().any(|v| Some(v.block) == block && v.cue == cue && (emitter.is_none() || v.emitter == emitter))
    }

    /// A keep-alive refresh: the matching voice's counter goes up by 1; with none, a start.
    pub fn keep_alive(&mut self, slot: usize, cue: i32, emitter: Option<[f32; 3]>) {
        let block = self.slots.get(slot).copied().flatten();
        match self.voices.iter_mut().find(|v| Some(v.block) == block && v.cue == cue && (emitter.is_none() || v.emitter == emitter)) {
            Some(v) => v.counter += 1,
            None => {
                self.play(slot, cue, Play { emitter, do_not_restart: true, keep_alive: true, ..Default::default() });
            }
        }
    }

    /// VisitLoopSound: refresh the matching voice's keep-alive counter; starts nothing.
    pub fn refresh(&mut self, slot: usize, cue: i32, emitter: Option<[f32; 3]>) {
        let block = self.slots.get(slot).copied().flatten();
        if let Some(v) = self.voices.iter_mut().find(|v| Some(v.block) == block && v.cue == cue && (emitter.is_none() || v.emitter == emitter)) {
            v.counter += 1;
        }
    }

    /// Stop the first matching voice of a slot (KillSound; the same-voice test).
    pub fn stop(&mut self, slot: usize, cue: i32, emitter: Option<[f32; 3]>) {
        let block = self.slots.get(slot).copied().flatten();
        if let Some(k) = self.voices.iter().position(|v| Some(v.block) == block && v.cue == cue && (emitter.is_none() || v.emitter == emitter)) {
            let id = self.voices.remove(k).id;
            self.requests.push(Request::Stop { id });
        }
    }

    /// The per-pass voice update: drop voices the driver finished, count keep-alive voices down
    /// (stopped below 1), and resend each positioned voice's volume and pan (stopped out of
    /// range unless it loops).
    pub fn update(&mut self, finished: &[usize]) {
        self.voices.retain(|v| !finished.contains(&v.id) || v.mode == Mode::Loop);
        let listener = self.listener;
        let mut stops = Vec::new();
        let mut sets = Vec::new();
        for v in &mut self.voices {
            if v.mode == Mode::KeepAlive {
                v.counter -= 1;
                if v.counter < 1 {
                    stops.push(v.id);
                    continue;
                }
            }
            if let Some(e) = v.emitter {
                match listener.volume(e, v.base, v.r, v.r0) {
                    Some(vol) if vol > 0 => sets.push((v.id, vol, Some(listener.pan(e)))),
                    _ if v.mode == Mode::Loop => {}
                    _ => stops.push(v.id),
                }
            }
        }
        self.voices.retain(|v| !stops.contains(&v.id));
        self.requests.extend(stops.into_iter().map(|id| Request::Stop { id }));
        self.requests.extend(sets.into_iter().map(|(id, volume, pan)| Request::Set { id, volume, pan }));
    }

    pub fn voices(&self) -> usize {
        self.voices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level() -> Listener {
        // A camera at the origin facing +z (rows right, up, forward) puts the listener 5 ahead.
        Listener::from_camera([0.0, 0.0, -5.0], [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    #[test]
    fn positional_volume_and_pan() {
        let l = level();
        let at = |d: f32| [d, 0.0, 0.0];
        assert_eq!(l.volume(at(0.0), 1024, 200.0, 0.0), Some(1024));
        assert_eq!(l.volume(at(50.0), 1024, 200.0, 0.0), Some(768));
        assert_eq!(l.volume(at(100.0), 1024, 200.0, 0.0), Some(512));
        assert_eq!(l.volume(at(199.9), 1024, 200.0, 0.0), Some(0));
        assert_eq!(l.volume(at(200.0), 1024, 200.0, 0.0), None);
        assert_eq!(l.volume(at(50.0), 768, 200.0, 0.0), Some(576));
        assert_eq!(l.volume(at(5.0), 768, 30.0, 10.0), Some(768));
        assert_eq!(l.volume(at(20.0), 768, 30.0, 10.0), Some(384));
        assert_eq!(l.volume(at(30.0), 768, 30.0, 10.0), None);
        // Pan: u = (1, 0, 0) gives 269; (-1, 0, 0) 89; (0, 0, 1) 179; (0, 0, -1) 359.
        for (u, p) in [([1.0, 0.0, 0.0], 269), ([-1.0, 0.0, 0.0], 89), ([0.0, 0.0, 1.0], 179), ([0.0, 0.0, -1.0], 359)] {
            assert_eq!(l.pan(u), p);
        }
    }

    #[test]
    fn slot_blocks() {
        assert_eq!(weapon_block(false, Some(100)), 463);
        assert_eq!(weapon_block(false, None), 466);
        assert_eq!(weapon_block(true, Some(111)), 469);
        assert_eq!(weapon_block(true, Some(118)), 476);
        assert_eq!(weapon_block(true, None), 466);
        assert_eq!(shield_block(false, Some(106)), 456);
        assert_eq!(shield_block(false, Some(110)), 460);
        assert_eq!(shield_block(false, None), 456);
        assert_eq!(shield_block(true, None), 459);
        assert_eq!((power_block(Some(160)), power_block(Some(173)), power_block(None)), (477, 490, 485));
    }

    #[test]
    fn cue_call_rules_and_keep_alive() {
        let mut m = SoundManager { listener: level(), ..Default::default() };
        // Not loaded, a negative cue, an empty slot: nothing plays.
        assert!(!m.play(MAIN, 5, Play::default()));
        m.loaded.insert(347);
        assert!(!m.play(MAIN, -1, Play::default()));
        assert!(!m.play(WEAPON, 0, Play::default()));
        assert!(m.play(MAIN, 5, Play::default()));
        // Do not restart while a matching voice plays.
        assert!(!m.play(MAIN, 5, Play { do_not_restart: true, ..Default::default() }));
        // Out of range does not start.
        assert!(!m.play(MAIN, 6, Play { emitter: Some([300.0, 0.0, 0.0]), ..Default::default() }));
        // A volume of 0 is refused.
        assert!(!m.play(MAIN, 6, Play { volume: Some(0), ..Default::default() }));
        // Keep-alive: four updates without a refresh stop it on the fourth.
        m.requests.clear();
        m.keep_alive(MAIN, 168, None);
        for _ in 0..3 {
            m.update(&[]);
        }
        assert_eq!(m.voices(), 2);
        m.update(&[]);
        assert_eq!(m.voices(), 1);
        // VisitLoopSound refreshes without starting; KillSound stops the matching voice.
        m.refresh(MAIN, 200, None);
        assert_eq!(m.voices(), 1);
        m.loaded.insert(347);
        m.stop(MAIN, 5, Some([1.0, 2.0, 3.0]));
        assert_eq!(m.voices(), 1, "a different emitter does not match");
        m.stop(MAIN, 5, None);
        assert_eq!(m.voices(), 0);
        // A refresh each pass keeps it.
        m.keep_alive(MAIN, 168, None);
        for _ in 0..10 {
            m.keep_alive(MAIN, 168, None);
            m.update(&[]);
        }
        assert_eq!(m.voices(), 1);
    }
}
