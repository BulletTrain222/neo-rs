//! The performance overlay (a test tool of this build; no spec): F3 shows the frame rate and
//! frame times, the simulation's ticks, the entity and character counts, the world loads in
//! flight and the current worlds.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{PlayState, SimDriver, Streamer};

#[derive(Resource, Default)]
pub struct PerfOverlay {
    pub on: bool,
}

#[derive(Component)]
pub struct PerfPanel;

#[derive(Component)]
pub struct PerfText;

/// The last two seconds of frames: (frame seconds, sim ticks run).
#[derive(Default)]
pub struct Samples(VecDeque<(f32, u32)>);

pub fn spawn_panel(mut commands: Commands) {
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, right: px(12), top: px(12), padding: UiRect::all(px(10)), display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GlobalZIndex(5001),
            PerfPanel,
        ))
        .with_children(|ui| {
            ui.spawn((Text::default(), TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() }, TextColor(Color::srgb(0.75, 0.95, 0.8)), PerfText));
        });
}

#[allow(clippy::too_many_arguments)]
pub fn perf_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    driver: Res<SimDriver>,
    play: Option<Res<PlayState>>,
    streamer: Option<Res<Streamer>>,
    windows: Query<&Window>,
    entities: Query<Entity>,
    mut overlay: ResMut<PerfOverlay>,
    mut samples: Local<Samples>,
    mut panels: Query<&mut Node, With<PerfPanel>>,
    mut texts: Query<&mut Text, With<PerfText>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        overlay.on = !overlay.on;
    }
    let dt = time.delta_secs();
    let s = &mut samples.0;
    s.push_back((dt, driver.last_ticks));
    while s.iter().map(|x| x.0).sum::<f32>() > 2.0 && s.len() > 1 {
        s.pop_front();
    }
    for mut n in &mut panels {
        n.display = if overlay.on { Display::Flex } else { Display::None };
    }
    if !overlay.on {
        return;
    }
    let span: f32 = s.iter().map(|x| x.0).sum::<f32>().max(1e-6);
    let fps = s.len() as f32 / span;
    let mut times: Vec<f32> = s.iter().map(|x| x.0 * 1000.0).collect();
    times.sort_by(f32::total_cmp);
    let worst = times.last().copied().unwrap_or(0.0);
    // The 1% low: the frame rate of the slowest 1% of frames.
    let k = (times.len() / 100).max(1);
    let low = 1000.0 / (times[times.len().saturating_sub(k)..].iter().sum::<f32>() / k as f32).max(1e-3);
    let tps = s.iter().map(|x| x.1).sum::<u32>() as f32 / span;
    let mut text = format!(
        "PERFORMANCE (F3)\nFPS {fps:.0}  (1% low {low:.0})\nFrame {:.2} ms avg, {worst:.2} ms worst\nSim {tps:.0} ticks/s, {} this frame\nEntities {}",
        1000.0 * span / s.len() as f32,
        driver.last_ticks,
        entities.iter().count()
    );
    if let Some(w) = windows.iter().next() {
        text.push_str(&format!("\nWindow {}x{}", w.physical_width(), w.physical_height()));
    }
    if let Some(p) = play.as_deref() {
        let p = &p.play;
        let here = p.characters.iter().filter(|c| c.pack == p.world.pack).count();
        text.push_str(&format!(
            "\nWorld {} (other {})\nCharacters {} ({} here)\nRegion loads in flight {}\nPlay tick {}",
            p.world.pack,
            p.other.as_ref().map_or("none".to_string(), |o| o.pack.to_string()),
            p.characters.len(),
            here,
            streamer.as_deref().map_or(0, |s| s.regions_out),
            p.ticks
        ));
    }
    for mut t in &mut texts {
        *t = Text::new(text.clone());
    }
}
