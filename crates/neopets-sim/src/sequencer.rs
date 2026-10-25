//! The sound driver's music sequencer (SPEC-0031 section 8b to 8d; SPEC-0009 Interactive
//! segments): one playing instance of a bank's music, its children (each with 16 channels), the
//! 16 registers and the global excite G, run at 240 ticks per second with the driver's delta
//! timing; channel events go to the synth, command messages start, jump and stop children, set
//! and test registers, define and apply channel groups and macros, with the skip rules.
//!
//! When a child stops, the voices it started are released (the spec does not say what happens to
//! them; R-0021).

use crate::synth::{ChannelState, Instruments, Synth};
use std::collections::HashMap;
use std::sync::Arc;

/// T = 4,170 microseconds per sequencer tick.
pub const T: i64 = 4170;
/// Sequencer ticks per second, and 48 kHz frames per tick.
pub const TICKS: u32 = 240;
pub const FRAMES_PER_TICK: usize = 200;

/// A child sequence: initial tempo, PPQ and its event bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Track {
    pub tempo: u32,
    pub ppq: u32,
    pub events: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Channel {
    muted: bool,
    volume: i32,
    pan: i32,
    program: usize,
    sustain: bool,
    bend: i32,
}

impl Default for Channel {
    fn default() -> Self {
        Self { muted: false, volume: 127, pan: 0, program: 0, sustain: false, bend: 0 }
    }
}

#[derive(Clone, Debug, Default)]
struct Child {
    pos: usize,
    status: u8,
    wait: i64,
    carry: i64,
    tempo: i64,
    rate: i64,
    channels: [Channel; 16],
    ended_this_tick: bool,
    /// Started in this tick: its wait is not counted down until the next tick.
    fresh: bool,
}

/// The driver's rate: trunc(1000 x T / trunc(tempo / trunc(PPQ / 10))).
pub fn rate(tempo: i64, ppq: i64) -> i64 {
    let div = (tempo / (ppq / 10).max(1)).max(1);
    1000 * T / div
}

fn vlq(b: &[u8], at: &mut usize) -> Option<i64> {
    let mut v = 0i64;
    for _ in 0..4 {
        let c = *b.get(*at)?;
        *at += 1;
        v = (v << 7) | (c & 0x7F) as i64;
        if c & 0x80 == 0 {
            return Some(v);
        }
    }
    Some(v)
}

/// Operand bytes of a command opcode (definitions 0x0B and 0x0F are handled apart).
fn operands(op: u8) -> usize {
    match op {
        0x00..=0x03 | 0x0C..=0x0E | 0x10..=0x12 | 0x14 | 0x15 => 1,
        0x06 | 0x07 | 0x13 | 0x16 => 2,
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Skip {
    Normal,
    One,
    Block,
}

/// One playing instance (bank sound 0 of a bank).
#[derive(Clone, Debug, Default)]
pub struct Sequencer {
    tracks: Vec<Arc<Track>>,
    children: Vec<Option<Child>>,
    /// The driver's active list, in order.
    active: Vec<usize>,
    pub registers: [u8; 16],
    pub excite: u8,
    macros: HashMap<u8, Vec<u8>>,
    groups: HashMap<u8, (u8, Vec<(u8, u8, u8)>)>,
}

impl Sequencer {
    /// Start a bank's music: child 0 with start argument 0.
    pub fn new(tracks: Vec<Track>) -> Self {
        let n = tracks.len();
        let mut s = Self { tracks: tracks.into_iter().map(Arc::new).collect(), children: vec![None; n], ..Default::default() };
        s.start(0);
        s
    }

    pub fn playing(&self) -> Vec<usize> {
        self.active.clone()
    }

    /// Start child k (no effect while it plays): the start state, then its first delta.
    fn start(&mut self, k: usize) {
        if k >= self.tracks.len() || self.children[k].is_some() {
            return;
        }
        let t = self.tracks[k].clone();
        let tempo = t.tempo as i64;
        let mut c = Child { tempo, rate: rate(tempo, t.ppq as i64), fresh: true, ..Default::default() };
        Self::schedule(&t, &mut c);
        self.children[k] = Some(c);
        self.active.push(k);
    }

    fn stop(&mut self, k: usize, synth: &mut Synth) {
        if let Some(slot) = self.children.get_mut(k) {
            if slot.take().is_some() {
                synth.release_child(k);
            }
        }
        self.active.retain(|&c| c != k);
    }

    /// Read the next delta and set the wait (section 8d).
    fn schedule(track: &Track, c: &mut Child) {
        let Some(d) = vlq(&track.events, &mut c.pos) else {
            c.wait = 0;
            return;
        };
        let a = 100 * d + c.carry;
        if a < 0 || a < c.rate / 2 {
            c.wait = 0;
            c.carry = a;
        } else {
            let ppq = track.ppq.max(1) as i64;
            let u = c.tempo * (a / 100) / ppq;
            let w = (u - 1 + T) / T;
            c.wait = w;
            c.carry = a - c.rate * w;
        }
    }

    #[cfg(test)]
    fn wait(&self, k: usize) -> Option<i64> {
        self.children[k].as_ref().map(|c| c.wait)
    }

    /// One sequencer tick: the active list in order, including children started during the
    /// tick (reached later in it).
    pub fn tick(&mut self, inst: &Instruments, synth: &mut Synth) {
        for c in self.children.iter_mut().flatten() {
            c.ended_this_tick = false;
            c.fresh = false;
        }
        let mut done: Vec<usize> = Vec::new();
        while let Some(k) = self.active.iter().copied().find(|k| !done.contains(k)) {
            done.push(k);
            let Some(c) = self.children[k].as_mut() else { continue };
            if c.wait > 0 {
                if c.fresh {
                    continue;
                }
                c.wait -= 1;
                if c.wait > 0 {
                    continue;
                }
            }
            // Run events while the wait is 0.
            for _ in 0..100_000 {
                if self.event(k, inst, synth) != Some(true) {
                    break;
                }
                let track = self.tracks[k].clone();
                let Some(c) = self.children[k].as_mut() else { break };
                Self::schedule(&track, c);
                if c.wait > 0 {
                    break;
                }
            }
        }
    }

    /// Run the event at child k's position. Some(true): the child goes on.
    fn event(&mut self, k: usize, inst: &Instruments, synth: &mut Synth) -> Option<bool> {
        let track = self.tracks[k].clone();
        let ev = &track.events;
        let c = self.children[k].as_mut()?;
        let mut at = c.pos;
        let first = *ev.get(at)?;
        let status = if first & 0x80 != 0 {
            at += 1;
            first
        } else {
            c.status
        };
        c.status = status;
        match status & 0xF0 {
            0x80 | 0x90 | 0xD0 => {
                let n = if status & 0xF0 == 0xD0 { 1 } else { 2 };
                let d = ev.get(at..at + n)?;
                c.pos = at + n;
                let ch = (status & 0x0F) as usize;
                let (note, vel) = (d[0], if n == 2 { d[1] } else { 0 });
                let chan = c.channels[ch];
                if status & 0xF0 == 0x90 && vel > 0 {
                    if !chan.muted {
                        synth.note_on(inst, (k, ch), ChannelState { program: chan.program, volume: chan.volume, pan: chan.pan, bend: chan.bend }, note, vel);
                    }
                } else {
                    synth.note_off((k, ch), note, chan.sustain);
                }
                Some(true)
            }
            0xB0 => {
                let d = ev.get(at..at + 2)?;
                c.pos = at + 2;
                let ch = (status & 0x0F) as usize;
                let chan = &mut c.channels[ch];
                match d[0] {
                    7 => chan.volume = d[1] as i32,
                    10 => chan.pan = crate::synth::pan_angle(d[1] as i32),
                    64 => {
                        let on = d[1] >= 64;
                        if chan.sustain && !on {
                            synth.sustain_off((k, ch));
                        }
                        chan.sustain = on;
                    }
                    _ => {}
                }
                Some(true)
            }
            0xC0 => {
                let d = ev.get(at..at + 1)?;
                c.pos = at + 1;
                c.channels[(status & 0x0F) as usize].program = d[0] as usize;
                Some(true)
            }
            0xE0 => {
                let d = ev.get(at..at + 2)?;
                c.pos = at + 2;
                let ch = (status & 0x0F) as usize;
                let b = crate::synth::bend_value(d[0] as i32 + 128 * d[1] as i32);
                c.channels[ch].bend = b;
                synth.bend((k, ch), b);
                Some(true)
            }
            0xF0 if status == 0xFF => {
                let ty = *ev.get(at)?;
                at += 1;
                let len = vlq(ev, &mut at)? as usize;
                let body = ev.get(at..at + len)?;
                c.pos = at + len;
                match ty {
                    0x2F => self.end_of_track(k, synth),
                    0x51 if body.len() == 3 => {
                        c.tempo = (body[0] as i64) << 16 | (body[1] as i64) << 8 | body[2] as i64;
                        c.rate = rate(c.tempo, track.ppq as i64);
                        Some(true)
                    }
                    _ => {
                        self.stop(k, synth);
                        None
                    }
                }
            }
            0xF0 => {
                if ev.get(at) == Some(&0x75) {
                    // A command message: commands up to the 0xF7 after a command.
                    let (stopped, used) = self.command(k, &ev[at + 1..], synth);
                    if let Some(c) = self.children[k].as_mut() {
                        c.pos = at + 1 + used;
                    }
                    return Some(!stopped);
                }
                // Any other message: skipped up to and including the first 0xF7.
                let end = ev[at..].iter().position(|&b| b == 0xF7)? + at;
                c.pos = end + 1;
                Some(true)
            }
            _ => {
                self.stop(k, synth);
                None
            }
        }
    }

    /// End of track: the repeat count is 0, so the child restarts at its first event in the same
    /// tick; a second end of track in one tick stops it.
    fn end_of_track(&mut self, k: usize, synth: &mut Synth) -> Option<bool> {
        let c = self.children[k].as_mut()?;
        if c.ended_this_tick {
            self.stop(k, synth);
            return None;
        }
        c.ended_this_tick = true;
        c.pos = 0;
        c.carry = 0;
        Some(true)
    }

    fn select(&self, sel: u8) -> u8 {
        if sel == 0 { self.excite } else { self.registers.get(sel as usize - 1).copied().unwrap_or(0) }
    }

    fn reg(&self, r: u8) -> u8 {
        self.registers.get(r as usize).copied().unwrap_or(0)
    }

    /// Run a command message for the running child k (up to the 0xF7 after a command, or the
    /// end of a macro body); returns whether k stopped and the bytes used.
    fn command(&mut self, k: usize, m: &[u8], synth: &mut Synth) -> (bool, usize) {
        let mut skip = Skip::Normal;
        let mut i = 0;
        let arg = |i: usize| m.get(i).copied().unwrap_or(0);
        while i < m.len() {
            let op = m[i];
            i += 1;
            if op == 0xF7 {
                return (false, i);
            }
            if op == 0x0B {
                // Macro definition: m, then bytes up to the macro's own 0xF7 (always defined).
                let id = arg(i);
                let body: Vec<u8> = m[(i + 1).min(m.len())..].iter().take_while(|&&b| b != 0xF7).copied().collect();
                i += 1 + body.len() + 1;
                self.macros.insert(id, body);
                continue;
            }
            if op == 0x0F {
                // Group definition: g, s, then (c, lo, hi) triples up to 0xF7.
                let (g, s) = (arg(i), arg(i + 1));
                i += 2;
                let mut list = Vec::new();
                while i < m.len() && m[i] != 0xF7 {
                    list.push((arg(i), arg(i + 1), arg(i + 2)));
                    i += 3;
                }
                i += 1;
                match skip {
                    Skip::Normal => {
                        self.groups.insert(g, (s, list));
                    }
                    Skip::One => skip = Skip::Normal,
                    Skip::Block => {}
                }
                continue;
            }
            let (a, b) = (arg(i), arg(i + 1));
            i += operands(op);
            match skip {
                Skip::Block => {
                    if op == 0x05 {
                        skip = Skip::Normal;
                    }
                    continue;
                }
                Skip::One => {
                    match op {
                        0x04 => skip = Skip::Block,
                        0x05 | 0x08..=0x0A => {}
                        _ => skip = Skip::Normal,
                    }
                    continue;
                }
                Skip::Normal => {}
            }
            let test = |ok: bool| if ok { Skip::Normal } else { Skip::One };
            match op {
                0x00 => skip = test(self.excite > a),
                0x01 => skip = test(self.excite as u16 == a as u16 + 1),
                0x02 => skip = test(self.excite < a),
                0x16 => skip = test(self.reg(a) == b),
                0x06 => skip = test(self.reg(a) < b),
                0x07 => skip = test(self.reg(a) > b),
                0x13 => {
                    if let Some(r) = self.registers.get_mut(a as usize) {
                        *r = b.min(127);
                    }
                }
                0x14 => {
                    if let Some(r) = self.registers.get_mut(a as usize) {
                        *r = (*r + 1).min(127);
                    }
                }
                0x15 => {
                    if let Some(r) = self.registers.get_mut(a as usize) {
                        *r = r.saturating_sub(1);
                    }
                }
                0x12 => self.start(a as usize),
                0x0E => self.start((self.reg(a) as usize).wrapping_sub(1)),
                0x11 | 0x0D => {
                    let s = if op == 0x11 { a as usize } else { (self.reg(a) as usize).wrapping_sub(1) };
                    self.start(s);
                    self.stop(k, synth);
                    return (true, Self::rest(m, i));
                }
                0x03 => {
                    self.stop(a as usize, synth);
                    if a as usize == k {
                        return (true, Self::rest(m, i));
                    }
                }
                0x10 => {
                    if let Some((sel, list)) = self.groups.get(&a).cloned() {
                        let v = self.select(sel);
                        if let Some(c) = self.children[k].as_mut() {
                            for (ch, lo, hi) in list {
                                if let Some(chan) = c.channels.get_mut(ch as usize) {
                                    chan.muted = !(lo..=hi).contains(&v);
                                }
                            }
                        }
                    }
                }
                0x0C => {
                    if let Some(body) = self.macros.get(&a).cloned() {
                        if self.command(k, &body, synth).0 {
                            return (true, Self::rest(m, i));
                        }
                    }
                }
                _ => {}
            }
        }
        (false, i)
    }

    /// The end of a message from `i` when the rest is ignored: commands are stepped over
    /// (operands and definitions) to the terminating 0xF7.
    fn rest(m: &[u8], mut i: usize) -> usize {
        while i < m.len() {
            let op = m[i];
            i += 1;
            match op {
                0xF7 => return i,
                0x0B => {
                    i += 1;
                    while i < m.len() && m[i] != 0xF7 {
                        i += 1;
                    }
                    i += 1;
                }
                0x0F => {
                    i += 2;
                    while i < m.len() && m[i] != 0xF7 {
                        i += 3;
                    }
                    i += 1;
                }
                _ => i += operands(op),
            }
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{GroupDef, Sample, VoiceDef};

    fn inst() -> Instruments {
        Instruments {
            groups: vec![GroupDef { volume: 127, pan: 0, voices: vec![VoiceDef { volume: 127, centre: 0xBC, low: 0, high: 127, adsr1: 0x00FF, adsr2: 0x1FC0, ..Default::default() }] }],
            samples: vec![Sample { pcm: Arc::new(vec![500; 4800]), loop_start: Some(0) }],
        }
    }

    #[test]
    fn rate_and_timing() {
        // 500,000 us per quarter at PPQ 480: 4 MIDI ticks per sequencer tick.
        assert_eq!(rate(500_000, 480), 400);
        // A note at 0, its 0xDn note-off 480 MIDI ticks later, then the end of track.
        let events = vec![0x00, 0x90, 60, 100, 0x83, 0x60, 0xD0, 60, 0x00, 0xFF, 0x2F, 0x00];
        let mut s = Sequencer::new(vec![Track { tempo: 500_000, ppq: 480, events }]);
        let mut synth = Synth::new();
        let i = inst();
        s.tick(&i, &mut synth);
        assert_eq!(synth.active(), 1, "the note starts on the first tick");
        // a = 48,000, u = 500,000 us, W = trunc((u - 1 + T) / T) = 120, carry 0.
        assert_eq!(s.wait(0), Some(120));
        for _ in 0..120 {
            s.tick(&i, &mut synth);
        }
        // Tick 121 ran the note-off, the end of track and the restart's note-on.
        assert_eq!(s.playing(), vec![0]);
        assert_eq!(s.wait(0), Some(120));
    }

    #[test]
    fn commands_switch_children() {
        // Child 0: setup: clear register 2, start child 2, jump to 1.
        let setup = vec![0x00, 0xF0, 0x75, 0x13, 2, 0, 0x12, 2, 0x11, 1, 0xF7, 0x00, 0xFF, 0x2F, 0x00];
        // Child 1: each beat, if register 2 = 1 jump to child 3.
        let one = vec![0x00, 0xF0, 0x75, 0x16, 2, 1, 0x11, 3, 0xF7, 0x83, 0x60, 0xFF, 0x2F, 0x00];
        let plain = vec![0x83, 0x60, 0xFF, 0x2F, 0x00];
        let t = |e: Vec<u8>| Track { tempo: 500_000, ppq: 480, events: e };
        let mut s = Sequencer::new(vec![t(setup), t(one), t(plain.clone()), t(plain)]);
        let (i, mut synth) = (inst(), Synth::new());
        s.tick(&i, &mut synth);
        assert_eq!(s.playing(), vec![2, 1], "child 0 ended itself; 2 then 1 started");
        for _ in 0..200 {
            s.tick(&i, &mut synth);
        }
        assert_eq!(s.playing(), vec![2, 1]);
        s.registers[2] = 1;
        for _ in 0..130 {
            s.tick(&i, &mut synth);
        }
        assert_eq!(s.playing(), vec![2, 3], "child 1 jumped to child 3 on its beat");
    }

    #[test]
    fn groups_mute_by_excite_and_skips() {
        // Define group 0 on G (channel 0 open for 0..49, channel 1 for 50..100), apply it, then
        // a note on each channel.
        let msg = vec![0x00, 0xF0, 0x75, 0x0F, 0, 0, 0, 0, 49, 1, 50, 100, 0xF7, 0x10, 0, 0xF7, 0x00, 0x90, 60, 100, 0x00, 0x91, 60, 100, 0x83, 0x60, 0xFF, 0x2F, 0x00];
        let mut s = Sequencer::new(vec![Track { tempo: 500_000, ppq: 480, events: msg }]);
        let (i, mut synth) = (inst(), Synth::new());
        s.excite = 10;
        s.tick(&i, &mut synth);
        assert_eq!(synth.active(), 1, "only channel 0 sounds at G 10");
        // A failed test skips the next command: G > 50 fails, so register 0 is not set.
        let msg = vec![0x00, 0xF0, 0x75, 0x00, 50, 0x13, 0, 9, 0x13, 1, 7, 0xF7, 0x83, 0x60, 0xFF, 0x2F, 0x00];
        let mut s = Sequencer::new(vec![Track { tempo: 500_000, ppq: 480, events: msg }]);
        s.excite = 10;
        s.tick(&i, &mut synth);
        assert_eq!((s.registers[0], s.registers[1]), (0, 7));
    }
}
