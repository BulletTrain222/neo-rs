//! Sound-block cue playback in the driver (SPEC-0031 section 7, Play rule; Driver volume, pan and
//! pitch; Cue instance limit): a playback instance per started cue runs the cue's command records
//! with their delays (types 1 and 9 voice start, 20 no-op, 21 to 23 loops, 24 hold, 25 and 39
//! variant groups, 26 random delay, 27 and 29 bend, 30 to 32 and 40 variables, 34 wait on a
//! variable, 35 and 36 labels, 41 release, 42 kill, 43 end) over a voice range of 24 synth voices,
//! rendered at 48 kHz.
//!
//! Deviations: the record delays count updates at the driver's 240 Hz (the spec leaves the tick
//! unit open, Open question 47); the random draws use this crate's generator, not the driver's
//! 250-entry table (Open question 49); types 4, 5 and 38 do nothing (their meaning is open).

use crate::synth::{Sample, Synth, VoiceDef};
use std::collections::HashMap;
use std::sync::Arc;

/// A cue entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CueDef {
    pub volume: u8,
    pub pan: u8,
    pub count: u8,
    pub limit: u8,
    pub flags: u16,
    pub first: usize,
}

/// A command record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cmd {
    pub operand: [u8; 3],
    pub kind: u8,
    pub delay: u16,
}

impl Cmd {
    fn value(&self) -> i32 {
        self.operand[0] as i32 | (self.operand[1] as i32) << 8 | (self.operand[2] as i32) << 16
    }
}

/// A loaded sound block: its number, cues, commands and voices (by voice-table offset) with
/// their samples.
#[derive(Clone, Debug, Default)]
pub struct BlockDef {
    pub number: u32,
    pub cues: Vec<CueDef>,
    pub commands: Vec<Cmd>,
    /// Voice by table offset: the voice, its signed level and its sample.
    pub voices: HashMap<u32, (VoiceDef, i8, Sample)>,
}

#[derive(Clone, Debug)]
struct Instance {
    id: usize,
    block: Arc<BlockDef>,
    cue: usize,
    pc: usize,
    end: usize,
    countdown: i32,
    done: bool,
    stopping: bool,
    volume: i32,
    cue_volume: i32,
    pan: i32,
    bend: i32,
    vars: [i32; 4],
    /// The running variant: (its end, the group's end).
    variant: Option<(usize, usize)>,
    start: u64,
}

/// The driver side of the sound blocks: instances over a 24-voice synth.
#[derive(Clone, Debug)]
pub struct Effects {
    pub synth: Synth,
    instances: Vec<Instance>,
    /// The type-25 last pick and the type-39 counter, kept per command record (byte 2).
    counters: HashMap<(u32, usize), u8>,
    rng: u32,
    frame: usize,
    clock: u64,
    /// Instances that ended since the last `take_finished`.
    finished: Vec<usize>,
}

impl Default for Effects {
    fn default() -> Self {
        Self { synth: Synth::new(), instances: Vec::new(), counters: HashMap::new(), rng: 0x1234_5678, frame: 0, clock: 0, finished: Vec::new() }
    }
}

impl Effects {
    fn random(&mut self) -> u32 {
        // xorshift32 (the driver's own generator is not described; Open question 49).
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    /// Play rule steps 1 to 4: start cue `cue` of `block` as instance `id` with request volume
    /// `volume` (1,024 = 1.0) and pan angle `pan` (None: the cue's default pan). Returns false
    /// when nothing plays (bad cue, no commands, or refused at the instance limit).
    pub fn play(&mut self, id: usize, block: Arc<BlockDef>, cue: i32, volume: i32, pan: Option<i32>) -> bool {
        let Some(def) = usize::try_from(cue).ok().and_then(|c| block.cues.get(c)).copied() else { return false };
        if def.count == 0 || volume < 1 {
            return false;
        }
        let cue = cue as usize;
        let same = |i: &Instance| i.block.number == block.number && i.cue == cue && !i.stopping;
        if def.flags & 0x2 != 0 {
            let ids: Vec<usize> = self.instances.iter().filter(|i| same(i)).map(|i| i.id).collect();
            for i in ids {
                self.stop(i, true);
            }
        }
        if def.flags & 0x8 != 0 {
            let running: Vec<(usize, i32, u64)> = self.instances.iter().filter(|i| same(i)).map(|i| (i.id, i.volume, i.start)).collect();
            if running.len() >= def.limit as usize {
                let quietest = running.iter().min_by_key(|r| r.1).copied();
                let oldest = running.iter().min_by_key(|r| r.2).copied();
                let victim = match (def.flags & 0x10 != 0, def.flags & 0x20 != 0) {
                    (true, _) if quietest.is_some_and(|q| q.1 < volume) => quietest,
                    (true, true) => quietest,
                    (false, true) => oldest,
                    _ => None,
                };
                match victim {
                    Some(v) => self.stop(v.0, true),
                    None => return false,
                }
            }
        }
        self.clock += 1;
        let first = def.first;
        let countdown = block.commands.get(first).map_or(0, |c| c.delay as i32);
        let inst = Instance {
            id,
            cue,
            pc: first,
            end: first + def.count as usize,
            countdown,
            done: false,
            stopping: false,
            volume,
            cue_volume: def.volume as i32,
            pan: pan.unwrap_or(def.pan as i32),
            bend: 0,
            vars: [0; 4],
            variant: None,
            start: self.clock,
            block,
        };
        self.instances.push(inst);
        let k = self.instances.len() - 1;
        if self.instances[k].countdown <= 0 {
            self.run(k);
        }
        true
    }

    /// Stop an instance: its voices are released (or killed), and it issues no more records.
    pub fn stop(&mut self, id: usize, kill: bool) {
        for i in self.instances.iter_mut().filter(|i| i.id == id) {
            i.done = true;
            i.stopping = true;
        }
        if kill {
            self.synth.kill(id);
        } else {
            self.synth.release_child(id);
        }
    }

    /// A new request volume and pan (the per-pass positional update).
    pub fn set(&mut self, id: usize, volume: i32, pan: Option<i32>) {
        for i in self.instances.iter_mut().filter(|i| i.id == id) {
            i.volume = volume;
            if let Some(p) = pan {
                i.pan = p;
            }
            self.synth.set_instance(id, volume, i.pan);
        }
    }

    pub fn take_finished(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.finished)
    }

    pub fn playing(&self, id: usize) -> bool {
        self.instances.iter().any(|i| i.id == id)
    }

    /// Run records of instance k while its countdown is 0 or less (Play rule step 5).
    fn run(&mut self, k: usize) {
        for _ in 0..10_000 {
            let i = &self.instances[k];
            if i.done || i.countdown > 0 {
                return;
            }
            if let Some((vend, gend)) = i.variant {
                if i.pc >= vend {
                    let inst = &mut self.instances[k];
                    inst.pc = gend;
                    inst.variant = None;
                    inst.countdown = inst.block.commands.get(gend).map_or(0, |c| c.delay as i32);
                    continue;
                }
            }
            if i.pc >= i.end {
                self.instances[k].done = true;
                return;
            }
            let pc = i.pc;
            let cmd = i.block.commands[pc];
            let (r, next) = self.handle(k, pc, cmd);
            let inst = &mut self.instances[k];
            if inst.done {
                return;
            }
            if r < 0 {
                inst.countdown = 0;
                return;
            }
            inst.pc = next;
            inst.countdown = inst.block.commands.get(next).filter(|_| next < inst.end).map_or(0, |c| c.delay as i32) + r;
        }
    }

    /// A record's handler: (result r, next record).
    fn handle(&mut self, k: usize, pc: usize, cmd: Cmd) -> (i32, usize) {
        let b = cmd.operand;
        let next = pc + 1;
        let var = |i: &Instance, sel: u8| i.vars.get(sel as usize).copied().unwrap_or(0);
        match cmd.kind {
            1 | 9 => {
                let i = &self.instances[k];
                if let Some((def, level, sample)) = i.block.voices.get(&(cmd.value() as u32)).cloned() {
                    let lv = if level >= 0 { level as i32 } else if level >= -4 { var(i, (-level - 1) as u8) } else { 0 };
                    let mut def = def;
                    if def.pan < 0 {
                        def.pan = (var(i, (-def.pan - 1) as u8) * 360 / 127) as i16;
                    }
                    let (id, v, vc, pan, bend) = (i.id, i.volume, i.cue_volume, i.pan, i.bend);
                    self.synth.start_block(id, def, sample, v, vc, lv, pan, bend);
                }
                (0, next)
            }
            22 => {
                // Loop back: just after the nearest preceding type-21 record.
                let i = &self.instances[k];
                let first = i.block.cues[i.cue].first;
                let back = (first..pc).rev().find(|&j| i.block.commands[j].kind == 21).map_or(next, |j| j + 1);
                (0, back)
            }
            23 => {
                let i = &self.instances[k];
                let past = (next..i.end).find(|&j| i.block.commands[j].kind == 22).map_or(i.end, |j| j + 1);
                (0, past)
            }
            24 => (-1, pc),
            25 | 39 => {
                let (n, m) = (b[0].max(1) as usize, b[1] as usize);
                let key = (self.instances[k].block.number, pc);
                let last = self.counters.get(&key).copied().unwrap_or(b[2]);
                let pick = if cmd.kind == 25 {
                    let mut p = (self.random() % n as u32) as usize;
                    if p == last as usize {
                        p = (p + 1) % n;
                    }
                    self.counters.insert(key, p as u8);
                    p
                } else {
                    let p = last as usize % n;
                    self.counters.insert(key, ((p + 1) % n) as u8);
                    p
                };
                let start = next + pick * m;
                let gend = next + n * m;
                self.instances[k].variant = Some((start + m, gend));
                (0, start)
            }
            26 => {
                let max = cmd.value().max(0) as u32;
                (if max == 0 { 0 } else { (self.random() % (max + 1)) as i32 }, next)
            }
            27 => {
                let r = (self.random() as u16) as i16 as i32;
                let bend = r * b[0] as i32 / 100;
                let id = self.instances[k].id;
                self.instances[k].bend = bend;
                self.synth.bend((id, 0), bend);
                (0, next)
            }
            29 => {
                self.instances[k].bend = b[0] as i32 * 32767 / 127;
                (0, next)
            }
            30 => {
                if let Some(v) = self.instances[k].vars.get_mut(b[0] as usize) {
                    *v = b[1] as i32;
                }
                (0, next)
            }
            31 => {
                let (lo, hi) = (b[1] as u32, b[2] as u32);
                let r = if hi >= lo { lo + self.random() % (hi - lo + 1) } else { lo };
                if let Some(v) = self.instances[k].vars.get_mut(b[0] as usize) {
                    *v = r as i32;
                }
                (0, next)
            }
            32 => {
                if let Some(v) = self.instances[k].vars.get_mut(b[0] as usize) {
                    *v = (*v + 1).min(127);
                }
                (0, next)
            }
            40 => {
                if let Some(v) = self.instances[k].vars.get_mut(b[1] as usize) {
                    *v = (*v + b[0] as i8 as i32).clamp(-128, 127);
                }
                (0, next)
            }
            34 => {
                let v = var(&self.instances[k], b[0]);
                let x = b[2] as i32;
                let hold = match b[1] {
                    0 => v < x,
                    1 => v == x,
                    2 => v > x,
                    _ => false,
                };
                if hold { (-1, pc) } else { (0, next) }
            }
            36 => {
                let i = &self.instances[k];
                let first = i.block.cues[i.cue].first;
                let to = (first..i.end).find(|&j| i.block.commands[j].kind == 35 && i.block.commands[j].operand[0] == b[0]).unwrap_or(next);
                (0, to)
            }
            41 => {
                self.synth.release_child(self.instances[k].id);
                (0, next)
            }
            42 => {
                self.synth.kill(self.instances[k].id);
                (0, next)
            }
            43 => {
                self.instances[k].done = true;
                (0, next)
            }
            _ => (0, next),
        }
    }

    /// One driver update: each instance's countdown drops by one and due records run; an
    /// instance that has issued its last record and has no voice left ends.
    pub fn update(&mut self) {
        for k in 0..self.instances.len() {
            let i = &mut self.instances[k];
            if !i.done {
                i.countdown -= 1;
                if i.countdown <= 0 {
                    self.run(k);
                }
            }
        }
        let synth = &self.synth;
        let mut ended = Vec::new();
        self.instances.retain(|i| {
            let keep = !(i.done && synth.sounding(i.id) == 0);
            if !keep {
                ended.push(i.id);
            }
            keep
        });
        self.finished.extend(ended);
    }

    /// Render stereo frames (interleaved, -1 to 1); one update every 200 frames.
    pub fn render(&mut self, out: &mut [f32]) {
        for pair in out.chunks_mut(2) {
            if self.frame == 0 {
                self.update();
            }
            self.frame = (self.frame + 1) % crate::sequencer::FRAMES_PER_TICK;
            let (l, r) = self.synth.frame();
            pair[0] = (l as f32 / 32768.0).clamp(-1.0, 1.0);
            if pair.len() > 1 {
                pair[1] = (r as f32 / 32768.0).clamp(-1.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(commands: Vec<Cmd>, cues: Vec<CueDef>) -> Arc<BlockDef> {
        let def = VoiceDef { volume: 0, centre: 0xBC, low: 0, high: 127, adsr1: 0x00FF, adsr2: 0x1FC0, ..Default::default() };
        let sample = Sample { pcm: Arc::new(vec![1000; 2000]), loop_start: None };
        let mut voices = HashMap::new();
        voices.insert(0, (def, 110, sample));
        Arc::new(BlockDef { number: 347, cues, commands, voices })
    }

    fn cmd(kind: u8, operand: [u8; 3], delay: u16) -> Cmd {
        Cmd { operand, kind, delay }
    }

    #[test]
    fn delays_voices_and_end() {
        // Voice start at once, a second 3 updates later, then end.
        let b = block(vec![cmd(1, [0; 3], 0), cmd(1, [0; 3], 3), cmd(43, [0; 3], 0)], vec![CueDef { volume: 120, count: 3, ..Default::default() }]);
        let mut e = Effects::default();
        assert!(e.play(7, b.clone(), 0, 1024, None));
        assert_eq!(e.synth.sounding(7), 1);
        e.update();
        e.update();
        assert_eq!(e.synth.sounding(7), 1);
        e.update();
        assert_eq!(e.synth.sounding(7), 2);
        // A bad cue, an empty cue and a volume below 1 play nothing.
        assert!(!e.play(8, b.clone(), 5, 1024, None));
        assert!(!e.play(8, b.clone(), -1, 1024, None));
        assert!(!e.play(8, b, 0, 0, None));
    }

    #[test]
    fn variants_loops_and_labels() {
        // Variant group of 2 variants of 1 record each: exactly one of the two voices starts,
        // then the group ends and the end record runs.
        let b = block(
            vec![cmd(25, [2, 1, 0], 0), cmd(1, [0; 3], 0), cmd(1, [0; 3], 0), cmd(43, [0; 3], 0)],
            vec![CueDef { volume: 120, count: 4, ..Default::default() }],
        );
        let mut e = Effects::default();
        e.play(1, b, 0, 1024, None);
        assert_eq!(e.synth.sounding(1), 1);
        // Round robin: picks 0, 1, 0 on successive plays.
        let rr = block(vec![cmd(39, [2, 1, 0], 0), cmd(30, [0, 5, 0], 0), cmd(30, [0, 9, 0], 0)], vec![CueDef { volume: 120, count: 3, ..Default::default() }]);
        let mut e = Effects::default();
        let mut picks = Vec::new();
        for id in 0..3 {
            e.play(id, rr.clone(), 0, 1024, None);
            picks.push(e.instances.iter().find(|i| i.id == id).map(|i| i.vars[0]));
        }
        assert_eq!(picks, vec![Some(5), Some(9), Some(5)]);
        // Label and goto: the goto skips a variable set.
        let g = block(vec![cmd(36, [1, 0, 0], 0), cmd(30, [0, 1, 0], 0), cmd(35, [1, 0, 0], 0), cmd(30, [1, 2, 0], 0), cmd(24, [0; 3], 0)], vec![CueDef { volume: 120, count: 5, ..Default::default() }]);
        let mut e = Effects::default();
        e.play(3, g, 0, 1024, None);
        let i = &e.instances[0];
        assert_eq!((i.vars[0], i.vars[1]), (0, 2));
        // A hold keeps the instance alive.
        e.update();
        assert!(e.playing(3));
    }

    #[test]
    fn instance_limits() {
        let one = |flags: u16| block(vec![cmd(24, [0; 3], 0)], vec![CueDef { volume: 120, count: 1, limit: 2, flags, ..Default::default() }]);
        // Limit 2 with flag 0x20: a third play stops the oldest.
        let b = one(0x8 | 0x20);
        let mut e = Effects::default();
        e.play(1, b.clone(), 0, 500, None);
        e.play(2, b.clone(), 0, 700, None);
        assert!(e.play(3, b.clone(), 0, 600, None));
        e.update();
        assert!(!e.playing(1) && e.playing(2) && e.playing(3));
        // Limit 2 with flag 0x10: 600 replaces the 500 one; 400 is refused.
        let b = one(0x8 | 0x10);
        let mut e = Effects::default();
        e.play(1, b.clone(), 0, 500, None);
        e.play(2, b.clone(), 0, 700, None);
        assert!(e.play(3, b.clone(), 0, 600, None));
        assert!(!e.play(4, b.clone(), 0, 400, None));
        // Flag 0x8 alone refuses at the limit.
        let b = one(0x8);
        let mut e = Effects::default();
        e.play(1, b.clone(), 0, 500, None);
        e.play(2, b.clone(), 0, 500, None);
        assert!(!e.play(3, b, 0, 500, None));
    }
}
