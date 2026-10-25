//! Sound in the window (SPEC-0009): decoded PCM as a Bevy audio source, and the music
//! controller's two stereo-stream channels played as looping voices whose volume follows the
//! channel volume (1/1024 units, linear). The bank music, the sound blocks and the cues are not
//! played yet.

use crate::PlayState;
use bevy::audio::{AddAudioSource, AudioPlayer, AudioSink, AudioSinkPlayback, ChannelCount, Decodable, PlaybackSettings, SampleRate, Source, Volume};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
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
