//! Sound-block cue playback in the driver (SPEC-0031 section 7, Play rule; Driver volume, pan and
//! pitch; Cue instance limit): a playback instance per started cue runs the cue's command records
//! with their delays (types 1 and 9 voice start, 20 no-op, 21 to 23 loops, 24 stop, 25 and 39
//! variant groups, 26 random delay, 27 and 29 bend, 30 to 32 and 40 variables, 34 a conditional
//! skip, 35 and 36 labels, 38 wait for voices, 41 release, 42 kill, 43 end) over a voice range of 24 synth voices,
//! rendered at 48 kHz.
//!
//! Deviations: the record delays count updates at the driver's 240 Hz (the spec leaves the tick
//! unit open, Open question 47); the random draws use this crate's generator, not the driver's
//! 250-entry table (Open question 49). Type 4 channels and type 5 child cues are built; noise voices are not synthesised.

use crate::synth::{Sample, Synth, VoiceDef};
#[cfg(test)]
use crate::synth::pan_pull;
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
    /// The raw type-4 and type-5 records by table offset.
    pub records: HashMap<u32, Vec<u8>>,
}

#[derive(Clone, Debug)]
struct Instance {
    id: usize,
    block: Arc<BlockDef>,
    cue: usize,
    channels: [Option<Channel>; 4],
    mods: Mods,
    /// The instance that started this one with a type-5 record.
    parent: Option<usize>,
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

/// At most this many channels run at once across the driver (SPEC-0031 type 4).
const MAX_CHANNELS: usize = 10;
/// A channel's cycle: 2048 phase units of 65,536 steps.
const CYCLE: u32 = 1 << 27;

/// A type-4 channel: a low-frequency oscillator moving one voice parameter of its instance.
#[derive(Clone, Copy, Debug)]
struct Channel {
    target: u8,
    other: usize,
    shape: u8,
    width: u32,
    depth: i32,
    depth_now: i32,
    invert: bool,
    phase: u32,
    step: u32,
    step_now: u32,
    held: i32,
    second_half: bool,
}

impl Channel {
    fn of(r: &[u8]) -> Option<(usize, Self)> {
        let word = |at: usize| Some(i16::from_le_bytes([*r.get(at)?, *r.get(at + 1)?]) as i32);
        let step = u32::from_le_bytes([*r.get(12)?, *r.get(13)?, *r.get(14)?, *r.get(15)?]);
        let depth = word(6)?;
        let c = Self {
            target: r[1],
            other: r[2] as usize,
            shape: r[3],
            width: word(4)? as u32,
            depth,
            depth_now: depth,
            invert: r[8] & 1 != 0,
            phase: (word(10)? as u32).wrapping_mul(65_536) % CYCLE,
            step,
            step_now: step,
            held: 0,
            second_half: false,
        };
        Some((r[0] as usize, c))
    }

    /// The wave at the current phase, about -32,767 to +32,767 (SPEC-0031 type 4 wave shapes).
    fn wave(&self) -> i32 {
        let x = self.phase as f64 / CYCLE as f64;
        let out = match self.shape {
            1 => (x * std::f64::consts::TAU).sin() * 32_767.0,
            2 => {
                if self.phase / 65_536 < self.width {
                    32_767.0
                } else {
                    -32_767.0
                }
            }
            3 => {
                if x < 0.25 {
                    x * 4.0 * 32_767.0
                } else if x < 0.75 {
                    (0.5 - x) * 4.0 * 32_767.0
                } else {
                    (x - 1.0) * 4.0 * 32_767.0
                }
            }
            4 => {
                if x < 0.5 {
                    x * 2.0 * 32_767.0
                } else {
                    (x - 1.0) * 2.0 * 32_767.0
                }
            }
            5 => self.held as f64,
            _ => 0.0,
        } as i32;
        if self.invert { -out } else { out }
    }
}

/// The driver's channel results of an instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Mods {
    level: i32,
    pan: i32,
    fine: i32,
    bend: i32,
}

/// The driver side of the sound blocks: instances over a 24-voice synth.
#[derive(Clone, Debug)]
pub struct Effects {
    pub synth: Synth,
    instances: Vec<Instance>,
    /// The type-25 last pick and the type-39 counter, kept per command record (byte 2).
    counters: HashMap<(u32, usize), u8>,
    rng: u32,
    /// The ids of child cues, far above the sound manager's.
    next_child: usize,
    frame: usize,
    clock: u64,
    /// Instances that ended since the last `take_finished`.
    finished: Vec<usize>,
}

impl Default for Effects {
    fn default() -> Self {
        Self { synth: Synth::new(), instances: Vec::new(), counters: HashMap::new(), rng: 0x1234_5678, next_child: usize::MAX / 2, frame: 0, clock: 0, finished: Vec::new() }
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
                self.stop(i, false);
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
                    Some(v) => self.stop(v.0, false),
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
            channels: [None; 4],
            mods: Mods::default(),
            parent: None,
        };
        self.instances.push(inst);
        let k = self.instances.len() - 1;
        if self.instances[k].countdown <= 0 {
            self.run(k);
        }
        true
    }

    /// Stop an instance (SPEC-0031 Stopping a cue and its stop records): its voices are released (or killed), then the records
    /// after the cue's first end record run at once with their delays ignored; voices those start are not released.
    pub fn stop(&mut self, id: usize, kill: bool) {
        // Child cues of the instance are stopped the same way.
        let children: Vec<usize> = self.instances.iter().filter(|i| i.parent == Some(id) && !i.stopping).map(|i| i.id).collect();
        for c in children {
            self.stop(c, kill);
        }
        if kill {
            self.synth.kill(id);
        } else {
            self.synth.release_child(id);
        }
        for k in 0..self.instances.len() {
            if self.instances[k].id != id || self.instances[k].stopping {
                continue;
            }
            self.instances[k].stopping = true;
            self.instances[k].done = false;
            let i = &self.instances[k];
            let first = i.block.cues[i.cue].first;
            let end = i.end;
            let mut pc = (first..end).find(|&j| i.block.commands[j].kind == 43).map_or(end, |j| j + 1);
            if !kill {
                for _ in 0..10_000 {
                    if pc >= end {
                        break;
                    }
                    let cmd = self.instances[k].block.commands[pc];
                    let (r, next) = self.handle(k, pc, cmd);
                    if self.instances[k].done || r < 0 {
                        break;
                    }
                    pc = next;
                }
            }
            self.instances[k].done = true;
        }
    }

    /// A new request volume and pan (the per-pass positional update).
    pub fn set(&mut self, id: usize, volume: i32, pan: Option<i32>) {
        for k in 0..self.instances.len() {
            if self.instances[k].id == id {
                self.instances[k].volume = volume;
                if let Some(p) = pan {
                    self.instances[k].pan = p;
                }
                self.apply_levels(k);
            }
        }
    }

    /// The instance's levels with its channel results, on all its own voices.
    fn apply_levels(&mut self, k: usize) {
        let i = &self.instances[k];
        let pull = i.block.cues[i.cue].flags & 0x4 != 0;
        let (id, v, c, pan) = (i.id, i.volume, (127 + i.mods.level).clamp(0, 127), (i.pan + i.mods.pan).rem_euclid(360));
        self.synth.set_instance(id, v, c, pan, pull);
    }

    /// One driver tick of an instance's channels (SPEC-0031 type 4): results that changed reach its voices.
    fn step_channels(&mut self, k: usize) {
        if self.instances[k].channels.iter().all(Option::is_none) {
            return;
        }
        let cue_volume = self.instances[k].block.cues[self.instances[k].cue].volume as i32;
        let before = self.instances[k].mods;
        for n in 0..4 {
            let Some(mut c) = self.instances[k].channels[n] else { continue };
            c.phase = (c.phase + c.step_now.wrapping_mul(2) % CYCLE) % CYCLE;
            let second = c.phase >= CYCLE / 2;
            if c.shape == 5 && second != c.second_half {
                let r = self.random();
                let span = (r >> 8 & 0xFFFF) as i32;
                c.held = if second {
                    span - 32_766
                } else if r & 1 == 0 {
                    0
                } else {
                    -(span / 2)
                };
            }
            c.second_half = second;
            let out = c.wave();
            let scale = |max: i64| (out as i64 * max / 32_767) as i32;
            let m = &mut self.instances[k].mods;
            match c.target {
                // Only ever down: 0 at the top of the wave, -S at the bottom.
                1 => {
                    let s = cue_volume * c.depth_now / 1024;
                    m.level = -(s as i64 * (32_767 - out) as i64 / 65_534) as i32;
                }
                2 => m.pan = scale(c.depth_now as i64 * 45 / 256),
                3 => m.fine = scale(c.depth_now as i64 * 381 / 64),
                4 => m.bend = scale(c.depth_now as i64 * 32_767 / 1024),
                5 | 6 => {
                    let f = c.depth_now as f64 / 1024.0 * out as f64 / 32_767.0;
                    if let Some(o) = self.instances[k].channels.get_mut(c.other).and_then(|o| o.as_mut()) {
                        if c.target == 5 {
                            o.step_now = (o.step as f64 * (1.0 + f)).max(0.0) as u32;
                        } else {
                            o.depth_now = (o.depth as f64 * (1.0 + f)) as i32;
                        }
                    }
                }
                _ => {}
            }
            // A channel can change itself through the other-channel words; keep those.
            let now = self.instances[k].channels[n].unwrap_or(c);
            self.instances[k].channels[n] = Some(Channel { phase: c.phase, held: c.held, second_half: c.second_half, ..now });
        }
        let after = self.instances[k].mods;
        if (after.level, after.pan) != (before.level, before.pan) {
            self.apply_levels(k);
        }
        if (after.fine, after.bend) != (before.fine, before.bend) {
            let i = &self.instances[k];
            self.synth.set_instance_pitch(i.id, after.fine, i.bend + after.bend);
        }
    }

    fn running_channels(&self) -> usize {
        self.instances.iter().map(|i| i.channels.iter().flatten().count()).sum()
    }

    pub fn take_finished(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.finished)
    }

    pub fn playing(&self, id: usize) -> bool {
        self.instances.iter().any(|i| i.id == id)
    }

    /// The running instances, for a trace: "block:cue #id" with its volume, its parent and
    /// whether it is stopping or done (this build's test tool).
    pub fn describe(&self) -> Vec<String> {
        self.instances
            .iter()
            .map(|i| {
                let parent = i.parent.map_or(String::new(), |p| format!(" child of #{p}"));
                let state = if i.stopping { " stopping" } else if i.done { " done" } else { "" };
                format!("{}:{} #{} vol {} ch {}{parent}{state}", i.block.number, i.cue, i.id, i.volume, i.channels.iter().flatten().count())
            })
            .collect()
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
                    let mods = i.mods;
                    self.synth.start_block(id, def, sample, v, vc, lv, pan, bend);
                    // The voice starts with the pan pull and the channel results as they stand.
                    self.apply_levels(k);
                    if mods.fine != 0 || mods.bend != 0 {
                        self.synth.set_instance_pitch(id, mods.fine, bend + mods.bend);
                    }
                }
                (0, next)
            }
            // SPEC-0031 type 4: a channel of this instance; target 0 clears it.
            4 => {
                let i = &self.instances[k];
                if let Some((n, c)) = i.block.records.get(&(cmd.value() as u32)).and_then(|r| Channel::of(r)) {
                    let free = i.channels.get(n).is_some_and(Option::is_some) || self.running_channels() < MAX_CHANNELS;
                    if let Some(slot) = self.instances[k].channels.get_mut(n) {
                        *slot = (c.target != 0 && free).then_some(c);
                    }
                }
                (0, next)
            }
            // SPEC-0031 type 5: another cue of this block, only while the cue volume is not 0. Its level and pan come from
            // the record; the parent's request volume also scales it, so a far emitter's child stays quiet (this build's
            // reading: the spec gives only the record's level).
            5 => {
                let i = &self.instances[k];
                if i.cue_volume != 0 {
                    if let Some(r) = i.block.records.get(&(cmd.value() as u32)).cloned() {
                        let word = |at: usize| r.get(at..at + 4).map_or(0, |b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                        let select = |x: i32| if x < 0 { var(i, (-x - 1) as u8) } else { x };
                        let level = select(word(0)).abs().min(127);
                        let pan = select(word(4));
                        let (block, cue, volume, parent) = (i.block.clone(), word(12), i.volume * level / 127, i.id);
                        self.next_child += 1;
                        let id = self.next_child;
                        if self.play(id, block, cue, volume, Some(pan)) {
                            if let Some(c) = self.instances.iter_mut().find(|c| c.id == id) {
                                c.parent = Some(parent);
                            }
                        }
                    }
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
            // SPEC-0031 type 24: when it runs, the cue is stopped.
            24 => {
                let id = self.instances[k].id;
                if self.instances[k].stopping {
                    return (-1, pc);
                }
                self.stop(id, false);
                (-1, pc)
            }
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
            // SPEC-0031 type 38: while the instance owns a voice this record is tried again after its delay plus 1.
            38 => {
                if self.synth.owns(self.instances[k].id) {
                    (1, pc)
                } else {
                    (0, next)
                }
            }
            34 => {
                let v = var(&self.instances[k], b[0]);
                let x = b[2] as i32;
                // SPEC-0031 type 34, a conditional skip: on a failed test the next record is skipped, delay and all.
                let pass = match b[1] {
                    0 => v < x,
                    1 => v == x,
                    2 => v > x,
                    _ => true,
                };
                (0, if pass { next } else { next + 1 })
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
            self.step_channels(k);
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
        // The instance disappears when its program has ended and its last voice and child cue are gone.
        let parents: Vec<usize> = self.instances.iter().filter_map(|i| i.parent).collect();
        self.instances.retain(|i| {
            let keep = !(i.done && synth.sounding(i.id) == 0 && !parents.contains(&i.id));
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
        Arc::new(BlockDef { number: 347, cues, commands, voices, records: HashMap::new() })
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
        let g = block(vec![cmd(36, [1, 0, 0], 0), cmd(30, [0, 1, 0], 0), cmd(35, [1, 0, 0], 0), cmd(30, [1, 2, 0], 0), cmd(20, [0; 3], 10_000)], vec![CueDef { volume: 120, count: 5, ..Default::default() }]);
        let mut e = Effects::default();
        e.play(3, g, 0, 1024, None);
        let i = &e.instances[0];
        assert_eq!((i.vars[0], i.vars[1]), (0, 2));
        // A long delay keeps the instance alive.
        e.update();
        assert!(e.playing(3));
    }

    /// SPEC-0031 type 4: a depth-1024 sine on a cue of volume 120 sweeps the level from 127 down to 7; a step of 1,118,481
    /// makes a 60-tick cycle; type 5 starts a child cue with the record's level.
    #[test]
    fn channels_and_children() {
        let mut r = vec![0u8; 16];
        r[1] = 1;
        r[3] = 1;
        r[4..6].copy_from_slice(&1024i16.to_le_bytes());
        r[6..8].copy_from_slice(&1024i16.to_le_bytes());
        r[12..16].copy_from_slice(&1_118_481u32.to_le_bytes());
        let mut child = vec![0u8; 32];
        child[0] = 127;
        child[12] = 1;
        let mut records = HashMap::new();
        records.insert(16, r);
        records.insert(32, child);
        let cue = |first, count| CueDef { volume: 120, count, first, ..Default::default() };
        let commands = vec![cmd(4, [16, 0, 0], 0), cmd(5, [32, 0, 0], 0), cmd(20, [0; 3], 10_000), cmd(20, [0; 3], 10_000)];
        let b = Arc::new(BlockDef { number: 1, cues: vec![cue(0, 3), cue(3, 1)], commands, voices: HashMap::new(), records });
        let mut e = Effects::default();
        e.play(1, b, 0, 1024, None);
        assert_eq!(e.instances.len(), 2);
        assert_eq!(e.instances[1].parent, Some(1));
        let (mut low, mut high) = (0, -200);
        for _ in 0..60 {
            e.update();
            let m = e.instances[0].mods.level;
            low = low.min(m);
            high = high.max(m);
        }
        assert!(low <= -118 && high >= -2, "{low} {high}");
        // Stopping the parent stops the child.
        e.stop(1, false);
        assert!(e.instances.iter().all(|i| i.done));
    }

    /// SPEC-0031 Pan pull's vectors.
    #[test]
    fn pan_pull_vectors() {
        for (v, a, want) in [(512, 90, 45), (512, 270, 315), (500, 100, 48), (853, 181, 182), (854, 181, 181), (170, 90, 0)] {
            assert_eq!(pan_pull(a, v), want, "{v} {a}");
        }
    }

    /// SPEC-0031: type 34 skips the next record on a failed test; type 24 stops the cue and runs the stop records.
    #[test]
    fn skip_and_stop_records() {
        let b = block(vec![cmd(34, [0, 1, 5], 0), cmd(30, [1, 9, 0], 0), cmd(30, [2, 7, 0], 0), cmd(20, [0; 3], 10_000)], vec![CueDef { volume: 120, count: 4, ..Default::default() }]);
        let mut e = Effects::default();
        e.play(1, b, 0, 1024, None);
        assert_eq!((e.instances[0].vars[1], e.instances[0].vars[2]), (0, 7));
        let s = block(vec![cmd(24, [0; 3], 0), cmd(43, [0; 3], 0), cmd(30, [3, 4, 0], 0)], vec![CueDef { volume: 120, count: 3, ..Default::default() }]);
        let mut e = Effects::default();
        e.play(2, s, 0, 1024, None);
        assert!(e.instances.first().is_none_or(|i| i.done && i.vars[3] == 4));
    }

    #[test]
    fn instance_limits() {
        let one = |flags: u16| block(vec![cmd(20, [0; 3], 10_000)], vec![CueDef { volume: 120, count: 1, limit: 2, flags, ..Default::default() }]);
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
