//! The movie player shell (SPEC-0026 Movies, Movie skip; SPEC-0008 Movie files): the queue in
//! the order the level scripts add to it, one movie at a time at its own frame rate, the skip
//! check once per frame, and the game loop suspended while a movie plays.
//!
//! Not covered: the picture and sound. No spec describes the Bink stream yet (R-0112), so each
//! movie is a black screen with its name for its real play time. Also not covered: the reload
//! world hand-off (Playback steps 2, 3, 6 and 7: the world stays loaded and the hero keeps
//! their place), stopping the game's sound, and the attract pair (movies 0 and 86).
use bevy::prelude::*;
use neopets_data::movie::{self, Header};
use std::collections::VecDeque;

#[derive(Resource, Default)]
pub struct Movies {
    /// By movie index: the table name and the file's header (None when it could not be read).
    table: Vec<Option<(String, Header)>>,
    queue: VecDeque<u16>,
    current: Option<Current>,
}

struct Current {
    index: u16,
    frames: u32,
    period_ns: u64,
    shown: u32,
    carry_ns: u64,
    skip: neopets_sim::movie::Skip,
}

impl Movies {
    pub fn load(disc: &mut neopets_data::Disc) -> Result<Self, String> {
        let boot = neopets_data::archive::verified_boot_program(disc)?;
        let names = movie::names(&boot)?;
        let table = names.into_iter().map(|n| Header::read(disc, &n).ok().map(|h| (n, h))).collect();
        Ok(Self { table, ..Default::default() })
    }

    /// Add the movies a level script queued this frame, in order.
    pub fn take(&mut self, queued: &mut Vec<u16>) {
        self.queue.extend(queued.drain(..));
    }

    /// Advance by `elapsed_ns` with Cross or Start `held`. Returns true while a movie plays (the
    /// game loop is suspended for the whole frame).
    pub fn step(&mut self, elapsed_ns: u64, held: bool) -> bool {
        if self.current.is_none() {
            let Some(index) = self.queue.pop_front() else { return false };
            let Some((name, h)) = self.table.get(index as usize).cloned().flatten() else {
                eprintln!("movie {index}: not in the table or unreadable; skipped");
                return !self.queue.is_empty();
            };
            println!("movie {index} ({name}): {} frames at {:.3} fps, {:.1} s (picture not decoded)", h.frames, h.fps(), h.seconds());
            self.current = Some(Current {
                index,
                frames: h.frames,
                period_ns: (1e9 / h.fps()) as u64,
                shown: 0,
                carry_ns: 0,
                skip: Default::default(),
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
                break;
            }
        }
        true
    }

    fn caption(&self) -> Option<String> {
        let c = self.current.as_ref()?;
        let (name, _) = self.table.get(c.index as usize)?.as_ref()?;
        let left = c.frames.saturating_sub(c.shown) as f64 * c.period_ns as f64 / 1e9;
        Some(format!(
            "Movie {}: {}   {:.0} s\n\nMovie pictures and sound are not built yet (the Bink decoder waits on a spec).\nHold Cross or Start to skip.",
            c.index,
            name.trim_end_matches(".BIK").replace('\\', "/"),
            left.ceil()
        ))
    }
}

#[derive(Component)]
pub struct MovieScreen;

#[derive(Component)]
pub struct MovieCaption;

pub fn spawn_screen(mut commands: Commands) {
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
                MovieCaption,
                Text::new(""),
                TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
                TextLayout::justify(Justify::Center),
            ));
        });
}

pub fn sync_screen(movies: Option<Res<Movies>>, mut screen: Query<&mut Visibility, With<MovieScreen>>, mut caption: Query<&mut Text, With<MovieCaption>>) {
    let text = movies.as_deref().and_then(Movies::caption);
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
