//! The movie player (SPEC-0026 Movies, Movie skip; SPEC-0008 Movie files and Movie decoder build
//! guide): the queue in the order the level scripts add to it, one movie at a time at its own
//! frame rate, the skip check once per frame, and the game loop suspended while a movie plays.
//! Each presented frame is decoded from the disc file by `neopets_data::bink` (the file is read
//! whole when the movie starts) and shown at 4:3; its audio is streamed to a voice of its own.
//! A movie that cannot be read or decoded shows a black screen with its name for its play time.
//!
//! Of the hand-off only step 3's
//! camera and screen-effect reset is built (`ended`; `Play::after_movies`); the reload of world
//! W (Playback steps 2, 6 and 7) is not (the world stays loaded and the hero keeps their place). The original stops the game's sound before the queue plays and the reload
//! restarts it; without the reload, the sound is paused here and resumes after the queue.
//!
//! The skip prompt in the window's bottom right corner is this build's addition, not in the
//! original: the words in the disc's prompt font round the Cross icon, or the chosen Button
//! pictures' A.
use bevy::audio::{AudioPlayer, ChannelCount, Decodable, PlaybackSettings, SampleRate, Source};
use bevy::prelude::*;
use neopets_data::bink;
use neopets_data::movie::{self, Header};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Resource, Default)]
pub struct Movies {
    /// By movie index: the table name and the file's header (None when it could not be read).
    table: Vec<Option<(String, Header)>>,
    queue: VecDeque<u16>,
    current: Option<Current>,
    /// The movie that started this frame, for its played word.
    pub started: Option<u16>,
    /// The queue ran empty this frame after playing (or passing over) movies: the hand-off's
    /// camera and screen-effect reset is due (SPEC-0026 Playback and handoff, step 3).
    pub ended: bool,
    /// The Display page's "Movie cutscenes" is Off: each queued movie is passed over at once,
    /// as a skip on its first frame would (its played word is still set).
    pub off: bool,
    /// The decoder's fixed tables (read from the boot program at load).
    tables: Option<bink::Tables>,
    /// The latest decoded picture as RGBA, waiting to be shown (taken by `sync_screen`).
    pub picture: Option<Vec<u8>>,
    /// The playing movie's sound: the queue its voice reads, its rate and channels; a new voice
    /// is due when `voice` is set.
    pub sound: Arc<Mutex<VecDeque<i16>>>,
    /// The playing movie's 48 kHz mixer stage.
    mixer: Option<bink::Mixer>,
    pub voice: Option<(u32, u16)>,
}

struct Current {
    index: u16,
    frames: u32,
    period_ns: u64,
    shown: u32,
    carry_ns: u64,
    skip: neopets_sim::movie::Skip,
    /// The file and its decoder, and the frames decoded so far (None: black screen only).
    stream: Option<(Vec<u8>, bink::Movie)>,
    decoded: u32,
}

impl Movies {
    pub fn load(disc: &mut neopets_data::Disc) -> Result<Self, String> {
        let boot = neopets_data::archive::verified_boot_program(disc)?;
        let names = movie::names(&boot)?;
        let table = names.into_iter().map(|n| Header::read(disc, &n).ok().map(|h| (n, h))).collect();
        let tables = bink::Tables::read(&boot).map_err(|e| error!("Movie decoder tables: {e}")).ok();
        Ok(Self { table, tables, ..Default::default() })
    }

    /// Decode the current movie up to the frame being shown: its picture (the last one) and
    /// every frame's sound. A decode error stops the picture (the movie keeps its play time).
    fn catch_up(&mut self) {
        let Some(c) = self.current.as_mut() else { return };
        let (Some((file, mv)), Some(t)) = (c.stream.as_mut(), self.tables.as_ref()) else { return };
        let mut fresh = false;
        while c.decoded <= c.shown && (c.decoded as usize) < mv.frames() {
            match mv.decode(file, c.decoded as usize, t) {
                Ok(f) => {
                    // SPEC-0008 Movie audio internals: the 48 kHz mixer stage.
                    let pcm = match self.mixer.as_mut() {
                        Some(m) => m.convert(&f.samples),
                        None => f.samples,
                    };
                    if let Ok(mut q) = self.sound.lock() {
                        q.extend(pcm);
                    }
                    fresh = true;
                }
                Err(e) => {
                    error!("movie {}: frame {}: {e}", c.index, c.decoded);
                    c.stream = None;
                    return;
                }
            }
            c.decoded += 1;
        }
        if fresh {
            self.picture = Some(mv.picture.rgba());
        }
    }


    /// Advance by `elapsed_ns` with Cross or Start `held`. Returns true while a movie plays (the
    /// game loop is suspended for the whole frame).
    pub fn step(&mut self, elapsed_ns: u64, held: bool, disc: Option<&mut neopets_data::Disc>) -> bool {
        let busy = self.current.is_some() || !self.queue.is_empty();
        let playing = self.step_queue(elapsed_ns, held, disc);
        self.catch_up();
        if busy && self.current.is_none() && self.queue.is_empty() {
            self.ended = true;
        }
        playing
    }

    fn step_queue(&mut self, elapsed_ns: u64, held: bool, disc: Option<&mut neopets_data::Disc>) -> bool {
        if self.current.is_none() {
            let Some(index) = self.queue.pop_front() else { return false };
            if self.off {
                println!("movie {index}: passed over (Movie cutscenes off)");
                self.started = Some(index);
                return false;
            }
            let Some((name, h)) = self.table.get(index as usize).cloned().flatten() else {
                eprintln!("movie {index}: not in the table or unreadable; skipped");
                return !self.queue.is_empty();
            };
            // The whole file, and its decoder; without either the movie is a black screen.
            let stream = disc.and_then(|d| d.read(&movie::disc_path(&name)).map_err(|e| error!("movie {index}: {e}")).ok()).and_then(|file| match bink::Movie::open(&file) {
                Ok(m) => Some((file, m)),
                Err(e) => {
                    error!("movie {index}: {e}");
                    None
                }
            });
            println!("movie {index} ({name}): {} frames at {:.3} fps, {:.1} s{}", h.frames, h.fps(), h.seconds(), if stream.is_some() { "" } else { " (not decoded)" });
            if let Ok(mut q) = self.sound.lock() {
                q.clear();
            }
            let format = stream.as_ref().and_then(|(_, m)| m.audio_format());
            self.mixer = format.map(|(rate, channels)| bink::Mixer::new(rate, channels));
            self.voice = format.map(|(_, channels)| (bink::MIX_RATE, channels));
            self.picture = None;
            self.started = Some(index);
            self.current = Some(Current {
                index,
                frames: h.frames,
                period_ns: (1e9 / h.fps()) as u64,
                shown: 0,
                carry_ns: 0,
                skip: Default::default(),
                stream,
                decoded: 0,
            });
            return true;
        }
        let c = self.current.as_mut().unwrap();
        c.carry_ns += elapsed_ns;
        while c.carry_ns >= c.period_ns {
            c.carry_ns -= c.period_ns;
            c.shown += 1;
            // The skip check runs once per presented frame; the last frame ends the movie.
            if c.skip.frame(held) || c.shown >= c.frames {
                self.current = None;
                if let Ok(mut q) = self.sound.lock() {
                    q.clear();
                }
                break;
            }
        }
        true
    }

    fn caption(&self) -> Option<String> {
        let Some(c) = self.current.as_ref() else {
            // Queued this frame: the screen goes black at once; the movie starts next frame.
            return (!self.queue.is_empty()).then(String::new);
        };
        // A decoded movie shows its picture alone.
        if c.stream.is_some() {
            return Some(String::new());
        }
        let (name, _) = self.table.get(c.index as usize)?.as_ref()?;
        let left = c.frames.saturating_sub(c.shown) as f64 * c.period_ns as f64 / 1e9;
        Some(format!(
            "Movie {}: {}   {:.0} s\n\nThis movie could not be decoded.\nHold Cross or Start to skip.",
            c.index,
            name.trim_end_matches(".BIK").replace('\\', "/"),
            left.ceil()
        ))
    }
}

/// After the frame's ticks: the movies the level scripts and the attract queued, in order, so
/// the frame that queued them already shows the movie screen (SPEC-0026: the movie step runs
/// at the end of the pass).
pub fn collect(mut movies: Option<ResMut<Movies>>, mut play: Option<ResMut<crate::PlayState>>, mut menu: Option<ResMut<crate::menu_view::Menu>>, settings: Res<crate::display::DisplaySettings>) {
    let _span = crate::perf::span("collect");
    let Some(mv) = movies.as_deref_mut() else { return };
    mv.off = !settings.movies;
    if let Some(m) = menu.as_deref_mut() {
        mv.queue.extend(m.flow.movies.drain(..));
    }
    if let Some(p) = play.as_deref_mut() {
        mv.queue.extend(p.play.state.movie_queue.drain(..));
    }
}

#[derive(Component)]
pub struct MovieScreen;

#[derive(Component)]
pub struct MovieCaption;

/// One part of the skip prompt: 0 and 2 the words, 1 the button icon.
#[derive(Component)]
pub struct MovieSkipPart(usize);

/// The skip prompt's words either side of the icon.
const SKIP_WORDS: [&[u8]; 2] = [b"Press", b"to Skip"];
/// The prompt's font and spacing, as the dialogue box's prompt row (SPEC-0022).
const SKIP_FONT: usize = 1;
const SKIP_SPACING: i32 = -1;
/// The prompt's colour (128 is full) and its margins from the window's right and bottom edges to
/// its right end and middle, in canvas units.
const SKIP_COLOUR: [u8; 4] = [128, 128, 128, 128];
const SKIP_MARGIN: (f32, f32) = (5.0, 18.0);

/// The picture: an image node sized to 4:3 inside the black screen.
#[derive(Component)]
pub struct MoviePicture;

/// The movie's voice.
#[derive(Component)]
pub struct MovieVoice;

/// The playing movie's sound as an endless source: the samples the decoder queued, silence when
/// it has none yet.
#[derive(Asset, TypePath, Clone)]
pub struct MovieSound {
    queue: Arc<Mutex<VecDeque<i16>>>,
    rate: u32,
    channels: u16,
}

pub struct MovieSoundDecoder {
    sound: MovieSound,
    buffer: Vec<i16>,
    at: usize,
}

impl Iterator for MovieSoundDecoder {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at >= self.buffer.len() {
            self.buffer.clear();
            self.at = 0;
            if let Ok(mut q) = self.sound.queue.lock() {
                let n = q.len().min(1024) / self.sound.channels as usize * self.sound.channels as usize;
                self.buffer.extend(q.drain(..n));
            }
            if self.buffer.is_empty() {
                self.buffer.resize(self.sound.channels as usize * 64, 0);
            }
        }
        let s = self.buffer[self.at];
        self.at += 1;
        Some(s as f32 / 32768.0)
    }
}

impl Source for MovieSoundDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(self.sound.channels).unwrap_or(ChannelCount::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(self.sound.rate).unwrap_or(SampleRate::MIN)
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Decodable for MovieSound {
    type Decoder = MovieSoundDecoder;

    fn decoder(&self) -> Self::Decoder {
        MovieSoundDecoder { sound: self.clone(), buffer: Vec::new(), at: 0 }
    }
}

/// Start the movie's voice when a movie with sound starts, and end it when the queue is done.
pub fn sync_sound(mut commands: Commands, movies: Option<ResMut<Movies>>, mut sources: ResMut<Assets<MovieSound>>, voices: Query<Entity, With<MovieVoice>>) {
    let Some(mut mv) = movies else { return };
    if let Some((rate, channels)) = mv.voice.take() {
        for e in &voices {
            commands.entity(e).despawn();
        }
        let h = sources.add(MovieSound { queue: mv.sound.clone(), rate, channels: channels.max(1) });
        commands.spawn((AudioPlayer(h), PlaybackSettings::LOOP, MovieVoice));
    } else if mv.current.is_none() && mv.queue.is_empty() {
        for e in &voices {
            commands.entity(e).despawn();
        }
    }
}

pub fn spawn_screen(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Smooth filtering: the 512 × 448 picture is scaled up to the window.
    let mut image = crate::rgba_image(512, 448, vec![0; 512 * 448 * 4]);
    image.sampler = bevy::image::ImageSampler::linear();
    let picture = images.add(image);
    commands
        .spawn((
            MovieScreen,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(4000),
            Visibility::Hidden,
        ))
        .with_children(|c| {
            c.spawn((
                MoviePicture,
                ImageNode::new(picture),
                // 4:3 at the window's height (the 512 × 448 picture was shown stretched to the TV
                // frame; SPEC-0008).
                Node { position_type: PositionType::Absolute, height: Val::Vh(100.0), width: Val::Vh(133.333), ..default() },
                Visibility::Hidden,
            ));
            c.spawn((
                MovieCaption,
                Text::new(""),
                TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
                TextLayout::justify(Justify::Center),
            ));
            for k in 0..3 {
                c.spawn((MovieSkipPart(k), ImageNode::default(), Node { position_type: PositionType::Absolute, ..default() }, Visibility::Hidden));
            }
        });
}

pub fn sync_screen(
    mut movies: Option<ResMut<Movies>>,
    mut screen: Query<&mut Visibility, (With<MovieScreen>, Without<MoviePicture>)>,
    mut pictures: Query<(&ImageNode, &mut Visibility), With<MoviePicture>>,
    mut images: ResMut<Assets<Image>>,
    mut caption: Query<&mut Text, With<MovieCaption>>,
) {
    let _span = crate::perf::span("sync_screen");
    let text = movies.as_deref().and_then(Movies::caption);
    // The decoded picture, shown while its movie plays.
    let fresh = movies.as_deref_mut().and_then(|m| m.picture.take());
    let decoding = movies.as_deref().is_some_and(|m| m.current.as_ref().is_some_and(|c| c.stream.is_some() && c.decoded > 0));
    for (node, mut v) in &mut pictures {
        if let Some(rgba) = fresh.as_ref() {
            if let Some(mut img) = images.get_mut(&node.image) {
                img.data = Some(rgba.clone());
            }
        }
        let want = if decoding { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
    for mut v in &mut screen {
        let want = if text.is_some() { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
    if let Some(t) = text {
        for mut c in &mut caption {
            if c.0 != t {
                c.0 = t.clone();
            }
        }
    }
}

/// The skip prompt, while a movie plays. Its images are made once, and again when the button
/// pictures change.
pub fn sync_skip(
    movies: Option<Res<Movies>>,
    art: Option<Res<crate::hud_view::HudArt>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut parts: Query<(&MovieSkipPart, &mut Node, &mut ImageNode, &mut Visibility)>,
    mut made: Local<Option<(u32, Vec<(Handle<Image>, f32, f32, f32)>)>>,
) {
    let _span = crate::perf::span("sync_skip");
    let playing = movies.is_some_and(|m| m.current.is_some());
    let (Some(art), Some(window), true) = (art.as_deref(), windows.iter().next(), playing) else {
        for (.., mut v) in &mut parts {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
        return;
    };
    if made.as_ref().is_none_or(|(style, _)| *style != art.style) {
        let crate::hud_view::HudArtParts { f, mem } = art.parts();
        // (image, width, height, top above the centre line); the words are centred on their
        // capitals' height, as the icon on its middle.
        let cap = f.fonts.get(SKIP_FONT).and_then(|ft| ft.glyph(b'P')).map_or(0, |g| g.height as i32);
        let word = |t: &[u8]| f.text_image(mem, SKIP_FONT, t, SKIP_SPACING, SKIP_COLOUR).map(|(i, base)| (i, (base - cap / 2) as f32));
        let icon = (0..f.nodes.len()).find(|&n| f.sprite(n).is_some_and(|s| s.asset == crate::button_style::CROSS_ASSET)).and_then(|n| f.sprite_image(mem, n, 32, 32)).map(|i| (i, 16.0));
        let list: Vec<_> = [word(SKIP_WORDS[0]), icon, word(SKIP_WORDS[1])]
            .into_iter()
            .flatten()
            .map(|(i, top)| {
                let (w, h) = (i.width as f32, i.height as f32);
                (images.add(crate::rgba_image(i.width, i.height, i.rgba)), w, h, top)
            })
            .collect();
        *made = Some((art.style, list));
    }
    let Some((_, list)) = made.as_ref() else { return };
    if list.len() != 3 {
        return;
    }
    let (ww, wh) = (window.width(), window.height());
    let s = (ww / 512.0).min(wh / 448.0);
    let total: f32 = list.iter().map(|p| p.1).sum();
    let centre = wh - SKIP_MARGIN.1 * s;
    let mut x = ww - (total + SKIP_MARGIN.0) * s;
    let mut at = [0.0; 3];
    for (k, p) in list.iter().enumerate() {
        at[k] = x;
        x += p.1 * s;
    }
    for (part, mut node, mut image, mut v) in &mut parts {
        let (handle, w, h, top) = &list[part.0];
        if image.image != *handle {
            image.image = handle.clone();
        }
        node.left = Val::Px(at[part.0]);
        node.top = Val::Px(centre - top * s);
        node.width = Val::Px(w * s);
        node.height = Val::Px(h * s);
        if *v != Visibility::Inherited {
            *v = Visibility::Inherited;
        }
    }
}

/// While a movie plays every game sound is paused; when the queue is done they resume.
pub fn hush(movies: Option<Res<Movies>>, mut sinks: Query<&mut bevy::audio::AudioSink, Without<MovieVoice>>, mut was: Local<bool>) {
    let _span = crate::perf::span("hush");
    use bevy::audio::AudioSinkPlayback;
    let now = movies.is_some_and(|m| m.current.is_some() || !m.queue.is_empty());
    if now {
        for s in &mut sinks {
            if !s.is_paused() {
                s.pause();
            }
        }
    } else if *was {
        for s in &mut sinks {
            s.play();
        }
    }
    *was = now;
}
