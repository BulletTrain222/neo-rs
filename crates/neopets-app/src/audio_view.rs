//! Sound in the window (SPEC-0009, SPEC-0031): decoded PCM as a Bevy audio source; the music
//! controller's two stereo-stream channels played as looping voices whose volume follows the
//! channel volume (1/1024 units, linear); and the bank music, rendered by the sim's engine
//! (sequencer and synth) into an endless 48 kHz source. The sound blocks and the cues are not
//! played yet.

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
}

impl Pcm {
    pub fn from(p: neopets_data::audio::Pcm) -> Self {
        Self { samples: Arc::new(p.samples), channels: p.channels.max(1), rate: p.rate.max(1) }
    }
}

pub struct PcmDecoder {
    samples: Arc<Vec<i16>>,
    at: usize,
    channels: u16,
    rate: u32,
}

impl Iterator for PcmDecoder {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let s = *self.samples.get(self.at)?;
        self.at += 1;
        Some(s as f32 / 32768.0)
    }
}

impl Source for PcmDecoder {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.at)
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(self.channels).unwrap_or(ChannelCount::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(self.rate).unwrap_or(SampleRate::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(self.samples.len() as f64 / self.channels as f64 / self.rate as f64))
    }
}

impl Decodable for Pcm {
    type Decoder = PcmDecoder;

    fn decoder(&self) -> Self::Decoder {
        PcmDecoder { samples: self.samples.clone(), at: 0, channels: self.channels, rate: self.rate }
    }
}

pub fn register(app: &mut App) {
    app.add_audio_source::<Pcm>();
    app.add_audio_source::<MusicSource>();
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
    let Some(state) = state else { return };
    let p = &state.play;
    if !link.spawned {
        link.spawned = true;
        let h = sources.add(MusicSource { engine: link.engine.clone() });
        commands.spawn((AudioPlayer(h), PlaybackSettings::LOOP));
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
pub fn sync_music(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut pcms: ResMut<Assets<Pcm>>,
    mut voices: Query<(Entity, &StreamVoice, Option<&mut AudioSink>)>,
    mut cache: Local<HashMap<u32, Option<Handle<Pcm>>>>,
) {
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
        let handle = cache.entry(r).or_insert_with(|| {
            let crate::Archive { disc, index } = archive.as_deref_mut()?;
            let bytes = index.read(disc, r).ok()?;
            match neopets_data::audio::kpv(&bytes) {
                Ok(p) => Some(pcms.add(Pcm::from(p))),
                Err(e) => {
                    error!("Stream {r}: {e}");
                    None
                }
            }
        });
        if let Some(h) = handle.clone() {
            commands.spawn((AudioPlayer(h), PlaybackSettings::LOOP.with_volume(Volume::Linear(ch.volume as f32 / 1024.0)), StreamVoice(k, r)));
        }
    }
}
