//! Sound in the window (SPEC-0009, SPEC-0031): decoded PCM as a Bevy audio source; the music
//! controller's two stereo-stream channels played as looping voices whose volume follows the
//! channel volume (1/1024 units, linear); and the bank music, rendered by the sim's engine
//! (sequencer and synth) into an endless 48 kHz source; the sound blocks' cues through the sim's
//! cue engine; the stingers and emote clips.
//!
//! The volumes follow the Options menu's Sound page (display.rs `SoundLevels`, this build's
//! feature): `sync_levels` hands them to the game each frame, and sets the movie voice's volume.

use crate::PlayState;
use bevy::audio::{AddAudioSource, AudioPlayer, AudioSink, AudioSinkPlayback, ChannelCount, Decodable, PlaybackSettings, SampleRate, Source, Volume};
use bevy::prelude::*;
use neopets_sim::music::Engine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Decoded 16-bit PCM as an asset.
#[derive(Asset, TypePath, Clone)]
pub struct Pcm {
    samples: Arc<Vec<i16>>,
    channels: u16,
    rate: u32,
    /// Plays from the start again at its end (a looping disc stream). Looping here, in place of
    /// a looping player, keeps one copy of the stream: a looping player keeps a second copy, as
    /// floats (about 40 MB for a three-minute stream).
    looping: bool,
}

impl Pcm {
    pub fn from(p: neopets_data::audio::Pcm) -> Self {
        Self { samples: Arc::new(p.samples), channels: p.channels.max(1), rate: p.rate.max(1), looping: false }
    }

    /// The same, playing from the start again at its end (play it once: it never ends).
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }
}

pub struct PcmDecoder {
    samples: Arc<Vec<i16>>,
    at: usize,
    channels: u16,
    rate: u32,
    looping: bool,
}

impl Iterator for PcmDecoder {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.looping && self.at >= self.samples.len() && !self.samples.is_empty() {
            self.at = 0;
        }
        let s = *self.samples.get(self.at)?;
        self.at += 1;
        Some(s as f32 / 32768.0)
    }
}

impl Source for PcmDecoder {
    fn current_span_len(&self) -> Option<usize> {
        (!self.looping).then(|| self.samples.len() - self.at)
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(self.channels).unwrap_or(ChannelCount::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(self.rate).unwrap_or(SampleRate::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        (!self.looping).then(|| Duration::from_secs_f64(self.samples.len() as f64 / self.channels as f64 / self.rate as f64))
    }
}

impl Decodable for Pcm {
    type Decoder = PcmDecoder;

    fn decoder(&self) -> Self::Decoder {
        PcmDecoder { samples: self.samples.clone(), at: 0, channels: self.channels, rate: self.rate, looping: self.looping }
    }
}

pub fn register(app: &mut App) {
    app.add_audio_source::<Pcm>();
    app.add_audio_source::<MusicSource>();
    app.add_audio_source::<EffectsSource>();
    app.add_audio_source::<crate::movie_view::MovieSound>();
}

/// A sound block for the sim's cue engine: cues, commands and the voices the commands name,
/// each with its decoded sample. Noise voices (flag 0x8) are left out: noise is not synthesised.
pub fn block_def(number: u32, b: &neopets_data::block::Block) -> neopets_sim::cues::BlockDef {
    use neopets_sim::cues::{BlockDef, Cmd, CueDef};
    use neopets_sim::synth::{Sample, VoiceDef};
    let cues = b.cues.iter().map(|c| CueDef { volume: c.volume, pan: c.pan, count: c.count, limit: c.limit, flags: c.flags, first: c.first }).collect();
    let commands = b.commands.iter().map(|c| Cmd { operand: c.operand, kind: c.kind, delay: c.delay }).collect();
    let mut samples: HashMap<(u32, u32), Sample> = HashMap::new();
    let voices = b
        .voices
        .iter()
        .filter(|(_, v)| v.flags & 0x8 == 0)
        .filter_map(|(&off, v)| {
            let s = b.samples.get(&(v.offset, v.length))?;
            let sample = samples.entry((v.offset, v.length)).or_insert_with(|| Sample { pcm: Arc::new(s.pcm.clone()), loop_start: s.loop_start }).clone();
            let def = VoiceDef {
                priority: v.priority,
                volume: 0,
                centre: v.centre,
                fine: v.fine,
                pan: v.pan,
                low: 0,
                high: 127,
                bend: v.bend,
                adsr1: v.adsr1,
                adsr2: v.adsr2,
                flags: v.flags,
                sample: usize::MAX,
            };
            Some((off, (def, v.level, sample)))
        })
        .collect();
    BlockDef { number, cues, commands, voices, records: b.records.clone() }
}

/// The sound blocks' driver side as an endless audio source.
#[derive(Asset, TypePath, Clone)]
pub struct EffectsSource {
    engine: Arc<Mutex<neopets_sim::cues::Effects>>,
}

pub struct EffectsDecoder {
    engine: Arc<Mutex<neopets_sim::cues::Effects>>,
    buffer: Vec<f32>,
    at: usize,
}

impl Iterator for EffectsDecoder {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at >= self.buffer.len() {
            if let Ok(mut e) = self.engine.lock() {
                e.render(&mut self.buffer);
            }
            self.at = 0;
        }
        let s = self.buffer[self.at];
        self.at += 1;
        Some(s)
    }
}

impl Source for EffectsDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(2).unwrap_or(ChannelCount::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(neopets_sim::synth::RATE).unwrap_or(SampleRate::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for EffectsSource {
    type Decoder = EffectsDecoder;

    fn decoder(&self) -> Self::Decoder {
        EffectsDecoder { engine: self.engine.clone(), buffer: vec![0.0; 2 * neopets_sim::sequencer::FRAMES_PER_TICK], at: usize::MAX }
    }
}

/// The sound blocks' link: the shared engine and the loaded blocks.
#[derive(Default)]
pub struct EffectsLink {
    engine: Arc<Mutex<neopets_sim::cues::Effects>>,
    blocks: HashMap<u32, Option<Arc<neopets_sim::cues::BlockDef>>>,
    spawned: bool,
}

/// Load the blocks the slots want, pass the game's cue requests to the engine, and report the
/// instances the engine finished back to the game.
pub fn sync_effects(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut link: Local<EffectsLink>,
    mut sources: ResMut<Assets<EffectsSource>>,
) {
    let _span = crate::perf::span("sync_effects");
    let Some(mut state) = state else { return };
    if !link.spawned {
        link.spawned = true;
        let h = sources.add(EffectsSource { engine: link.engine.clone() });
        // Once, not LOOP: the source never ends, and a looping player keeps a copy of every
        // sample to start over from (it grew about 21 MB a minute for good).
        commands.spawn((AudioPlayer(h), PlaybackSettings::ONCE));
    }
    let p = &mut state.play;
    let mut wanted: Vec<u32> = p.sound.slots.iter().flatten().copied().collect();
    for r in &p.sound.requests {
        if let neopets_sim::sound::Request::Play { block, .. } = r {
            if !wanted.contains(block) {
                wanted.push(*block);
            }
        }
    }
    for b in wanted {
        if !link.blocks.contains_key(&b) {
            let def = archive.as_deref_mut().and_then(|a| {
                let crate::Archive { disc, index } = a;
                let bytes = index.read(disc, b).ok()?;
                match neopets_data::block::Block::parse(&bytes) {
                    Ok(blk) => Some(Arc::new(block_def(b, &blk))),
                    Err(e) => {
                        error!("Block {b}: {e}");
                        None
                    }
                }
            });
            link.blocks.insert(b, def);
        }
        if link.blocks.get(&b).is_some_and(Option::is_some) {
            p.sound.loaded.insert(b);
        }
    }
    let requests = std::mem::take(&mut p.sound.requests);
    let mut not_started = Vec::new();
    if let Ok(mut e) = link.engine.lock() {
        for r in requests {
            match r {
                neopets_sim::sound::Request::Play { id, block, cue, volume, pan } => {
                    let started = match link.blocks.get(&block) {
                        Some(Some(def)) => e.play(id, def.clone(), cue, volume, pan),
                        _ => false,
                    };
                    // A voice the engine never started is forgotten by the sim too.
                    if !started {
                        not_started.push(id);
                    }
                }
                neopets_sim::sound::Request::Set { id, volume, pan } => e.set(id, volume, pan),
                neopets_sim::sound::Request::Stop { id } => e.stop(id, false),
            }
        }
        p.sound_finished.extend(e.take_finished());
    }
    p.sound.not_started(&not_started);
}

/// The Sound page's volumes, handed to the game each frame (play.rs `sound_levels`), and the
/// movie voice's volume (the Movies level).
pub fn sync_levels(
    settings: Res<crate::display::DisplaySettings>,
    state: Option<ResMut<PlayState>>,
    mut movie: Query<&mut AudioSink, With<crate::movie_view::MovieVoice>>,
) {
    if let Some(mut state) = state {
        if state.play.sound_levels != settings.sound {
            state.play.sound_levels = settings.sound;
        }
    }
    let v = settings.sound.movies();
    for mut sink in &mut movie {
        if (sink.volume().to_linear() - v).abs() > 1e-4 {
            sink.set_volume(Volume::Linear(v));
        }
    }
}

/// A bank's instruments and child tracks for the sim's engine: the directory groups with their
/// voices, each voice's sample by pool offset, and the MMID children (SPEC-0031 sections 3 to 8).
pub fn instruments(bank: &neopets_data::bank::Bank) -> (neopets_sim::synth::Instruments, Vec<neopets_sim::sequencer::Track>) {
    use neopets_sim::synth::{GroupDef, Instruments, Sample, VoiceDef};
    let mut offsets: Vec<u32> = bank.samples.keys().copied().collect();
    offsets.sort_unstable();
    let index: HashMap<u32, usize> = offsets.iter().enumerate().map(|(i, &o)| (o, i)).collect();
    let samples = offsets.iter().map(|o| Sample { pcm: Arc::new(bank.samples[o].pcm.clone()), loop_start: bank.samples[o].loop_start }).collect();
    let groups = bank
        .groups
        .iter()
        .map(|g| GroupDef {
            volume: g.volume,
            pan: g.pan,
            voices: g
                .voices
                .iter()
                .map(|v| VoiceDef {
                    priority: v.priority,
                    volume: v.volume,
                    centre: v.centre,
                    fine: v.fine,
                    pan: v.pan,
                    low: v.low,
                    high: v.high,
                    bend: v.bend,
                    adsr1: v.adsr1,
                    adsr2: v.adsr2,
                    flags: v.flags,
                    sample: index.get(&v.offset).copied().unwrap_or(usize::MAX),
                })
                .collect(),
        })
        .collect();
    let tracks = bank.children.iter().map(|c| neopets_sim::sequencer::Track { tempo: c.tempo, ppq: c.ppq, events: c.events.clone() }).collect();
    (Instruments { groups, samples }, tracks)
}

/// Read and convert a bank resource.
pub fn load_bank(archive: &mut crate::Archive, resource: u32) -> Result<Engine, String> {
    let crate::Archive { disc, index } = archive;
    let bank = neopets_data::bank::Bank::parse(&index.read(disc, resource)?)?;
    let (inst, tracks) = instruments(&bank);
    Ok(Engine::new(Arc::new(inst), tracks))
}

/// The bank music as an endless audio source: the decoder renders from the shared engine.
#[derive(Asset, TypePath, Clone)]
pub struct MusicSource {
    engine: Arc<Mutex<Option<Engine>>>,
}

pub struct MusicDecoder {
    engine: Arc<Mutex<Option<Engine>>>,
    buffer: Vec<f32>,
    at: usize,
}

impl Iterator for MusicDecoder {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at >= self.buffer.len() {
            self.buffer.iter_mut().for_each(|s| *s = 0.0);
            if let Ok(mut e) = self.engine.lock() {
                if let Some(e) = e.as_mut() {
                    e.render(&mut self.buffer);
                }
            }
            self.at = 0;
        }
        let s = self.buffer[self.at];
        self.at += 1;
        Some(s)
    }
}

impl Source for MusicDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(2).unwrap_or(ChannelCount::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(neopets_sim::synth::RATE).unwrap_or(SampleRate::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for MusicSource {
    type Decoder = MusicDecoder;

    fn decoder(&self) -> Self::Decoder {
        MusicDecoder { engine: self.engine.clone(), buffer: vec![0.0; 2 * neopets_sim::sequencer::FRAMES_PER_TICK], at: usize::MAX }
    }
}

/// The bank music's link to the audio thread: the shared engine, the bank it holds and the
/// register 2 and G values last sent (reset when a bank starts).
#[derive(Resource, Default)]
pub struct BankLink {
    engine: Arc<Mutex<Option<Engine>>>,
    bank: Option<u32>,
    sent: Option<(u8, u8)>,
    spawned: bool,
}

/// Follow the game's bank music: start a bank from child 0 when the controller starts it, stop it
/// when it stops, set the volume each frame, and send register 2 and G when they change.
pub fn sync_bank(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut link: Local<BankLink>,
    mut sources: ResMut<Assets<MusicSource>>,
) {
    let _span = crate::perf::span("sync_bank");
    let Some(state) = state else { return };
    let p = &state.play;
    if !link.spawned {
        link.spawned = true;
        let h = sources.add(MusicSource { engine: link.engine.clone() });
        // Once: an endless source (a looping player would keep every sample for good).
        commands.spawn((AudioPlayer(h), PlaybackSettings::ONCE));
    }
    let bm = p.bank_music;
    let want = bm.playing;
    if want != link.bank || bm.started {
        let engine = match (want, archive.as_deref_mut()) {
            (Some(r), Some(a)) => match load_bank(a, r) {
                Ok(e) => Some(e),
                Err(e) => {
                    error!("Bank {r}: {e}");
                    None
                }
            },
            _ => None,
        };
        if let Ok(mut slot) = link.engine.lock() {
            *slot = engine;
        }
        link.bank = want;
        link.sent = None;
    }
    let values = (p.intensity.register2, p.intensity.excite);
    let send = link.sent != Some(values);
    if let Ok(mut slot) = link.engine.lock() {
        if let Some(e) = slot.as_mut() {
            e.volume = bm.sent();
            if send {
                e.sequencer.registers[2] = values.0.min(127);
                e.sequencer.excite = values.1.min(127);
            }
        }
    }
    if send {
        link.sent = Some(values);
    }
}

/// A playing stream voice: the stream channel (0 or 1) and its resource.
#[derive(Component)]
pub struct StreamVoice(usize, u32);

/// Mirror the two stream channels: start a looping voice for a channel's stream, set its volume
/// each frame, and stop it when the channel stops or changes stream.
/// A music stream being decoded on its own thread, or its decoded sound.
pub enum MusicLoad {
    Decoding(std::sync::mpsc::Receiver<Result<neopets_data::audio::Pcm, String>>),
    Ready(Option<Handle<Pcm>>),
}

pub fn sync_music(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut pcms: ResMut<Assets<Pcm>>,
    mut voices: Query<(Entity, &StreamVoice, Option<&mut AudioSink>)>,
    mut cache: Local<HashMap<u32, MusicLoad>>,
) {
    let _span = crate::perf::span("sync_music");
    let Some(state) = state else { return };
    let channels = state.play.music.channels;
    for (e, v, sink) in &mut voices {
        let ch = channels[v.0];
        if ch.stream != Some(v.1) {
            commands.entity(e).despawn();
        } else if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(ch.volume as f32 / 1024.0));
        }
    }
    for (k, ch) in channels.iter().enumerate() {
        let Some(r) = ch.stream else { continue };
        if voices.iter().any(|(_, v, _)| v.0 == k && v.1 == r) {
            continue;
        }
        // Performance: a stream is read from the disc here but decoded on its own thread (a
        // whole track took 15 to 19 ms of one frame, at a world's start and at every change
        // of music); the voice starts on the first frame after the decode ends, a frame or
        // two later.
        let load = cache.entry(r).or_insert_with(|| {
            let Some(crate::Archive { disc, index }) = archive.as_deref_mut() else { return MusicLoad::Ready(None) };
            match index.read(disc, r) {
                Ok(bytes) => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(neopets_data::audio::kpv(&bytes));
                    });
                    MusicLoad::Decoding(rx)
                }
                Err(e) => {
                    error!("Stream {r}: {e}");
                    MusicLoad::Ready(None)
                }
            }
        });
        if let MusicLoad::Decoding(rx) = load {
            let done = match rx.try_recv() {
                Ok(Ok(p)) => Some(Some(pcms.add(Pcm::from(p).looping()))),
                Ok(Err(e)) => {
                    error!("Stream {r}: {e}");
                    Some(None)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
            };
            match done {
                Some(h) => *load = MusicLoad::Ready(h),
                None => continue,
            }
        }
        if let MusicLoad::Ready(Some(h)) = load {
            // Once: the stream loops in its own decoder (one copy kept, not two).
            commands.spawn((AudioPlayer(h.clone()), PlaybackSettings::ONCE.with_volume(Volume::Linear(ch.volume as f32 / 1024.0)), StreamVoice(k, r)));
        }
    }
}

/// A playing stinger voice (one at a time; SPEC-0009 Stinger and emote-clip playback).
#[derive(Component)]
pub struct StingerVoice;

/// Play requested stingers: clip k is VAG resource 256 + k at volume trunc(1024 x gain) (the
/// Sound page's Jingles level), as a one-shot; the game learns whether one is playing.
pub fn sync_stingers(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut pcms: ResMut<Assets<Pcm>>,
    voices: Query<(Entity, Option<&AudioSink>), With<StingerVoice>>,
    mut cache: Local<HashMap<i32, Option<Handle<Pcm>>>>,
    mut banks: Local<HashMap<u32, Option<Vec<u8>>>>,
) {
    let _span = crate::perf::span("sync_stingers");
    let Some(mut state) = state else { return };
    // A voice whose sink has emptied has finished.
    let mut playing = false;
    for (e, sink) in &voices {
        if sink.is_some_and(|s| s.empty()) {
            commands.entity(e).despawn();
        } else {
            playing = true;
        }
    }
    let p = &mut state.play;
    for k in std::mem::take(&mut p.stinger_requests) {
        // Stingers 0 to 18 only (SPEC-0009; online, the host's request is checked here too).
        if playing || !(0..19).contains(&k) {
            continue;
        }
        let handle = cache.entry(k).or_insert_with(|| {
            let crate::Archive { disc, index } = archive.as_deref_mut()?;
            let bytes = index.read(disc, 256 + k as u32).ok()?;
            neopets_data::audio::vag(&bytes).ok().map(|(pcm, _)| pcms.add(Pcm::from(pcm)))
        });
        if let Some(h) = handle.clone() {
            let v = p.stinger_volume() as f32 / 1024.0;
            commands.spawn((AudioPlayer(h), PlaybackSettings::ONCE.with_volume(Volume::Linear(v)), StingerVoice));
            playing = true;
        }
    }
    // Emote clips share the handle: a VAG-headed clip at its directory position (2,048-byte
    // units) of bank 144 or 145, at volume trunc(1024 x gain) (the Sound page's Voices level).
    for (bank, start, len) in std::mem::take(&mut p.emote_requests) {
        // Banks 144 and 145 only (online, a host's request for any other file of the disc, which
        // would be read whole and kept, is passed over).
        if playing || !(144..=145).contains(&bank) || start.checked_add(len).is_none_or(|e| e > 1 << 20) {
            continue;
        }
        let bytes = banks.entry(bank).or_insert_with(|| {
            let crate::Archive { disc, index } = archive.as_deref_mut()?;
            index.read(disc, bank).ok()
        });
        let clip = bytes.as_ref().and_then(|b| b.get(start as usize * 2048..((start + len) as usize * 2048).min(b.len())));
        if let Some(Ok((pcm, _))) = clip.map(neopets_data::audio::vag) {
            let v = p.emote_volume() as f32 / 1024.0;
            commands.spawn((AudioPlayer(pcms.add(Pcm::from(pcm))), PlaybackSettings::ONCE.with_volume(Volume::Linear(v)), StingerVoice));
            playing = true;
        }
    }
    p.stinger_playing = playing;
}
