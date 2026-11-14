//! A test tool of this build (the original has only save points): F9 opens the save-state list.
//! The top row makes a new state under a name the player types; any other row loads its state
//! (Enter) or deletes it (Delete twice). There is no limit on the count: each state is its own
//! pair of files in the states folder (`crate::states_dir`). The game is paused while the list
//! is up, and the hero and the game menus get no input.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::dev_menu::DevMenu;

/// One saved state as the list shows it.
struct Entry {
    id: String,
    name: String,
    date: String,
    pack: String,
}

/// The list's state while it is up.
#[derive(Resource, Default)]
pub struct StatesMenu {
    cursor: usize,
    entries: Vec<Entry>,
    /// The name being typed for a new state.
    naming: Option<String>,
    /// Delete was pressed once on this row; a second press deletes.
    confirm: Option<usize>,
    /// The game speed to give back on close.
    speed: usize,
    /// A line under the list (the last thing that happened).
    line: String,
    /// NEOPETS_TEST_STATES opened the list once (diagnostic, for a screenshot).
    tested: bool,
    /// NEOPETS_LOAD_STATE has had its load.
    auto_loaded: bool,
}

#[derive(Component)]
pub struct StatesPanel;

/// The states in the folder, newest first.
fn read_entries() -> Vec<Entry> {
    let dir = crate::states_dir();
    let mut v: Vec<Entry> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension()? != "txt" {
                return None;
            }
            let id = path.file_stem()?.to_string_lossy().to_string();
            if !std::path::Path::new(&format!("{dir}/{id}.bin")).exists() {
                return None;
            }
            let text = std::fs::read_to_string(&path).ok()?;
            let mut lines = text.lines();
            let pack = lines.next().and_then(|l| l.split_whitespace().next()).unwrap_or("?").to_string();
            let name = lines.next().unwrap_or("").to_string();
            let date = lines.next().unwrap_or("").to_string();
            Some(Entry { id, name, date, pack })
        })
        .collect();
    // The ids are "s" and the save's Unix seconds (and "-k" for a second one in that second).
    let key = |id: &str| {
        let (s, k) = id.trim_start_matches('s').split_once('-').unwrap_or((id.trim_start_matches('s'), "1"));
        (s.parse::<u64>().unwrap_or(0), k.parse::<u32>().unwrap_or(1))
    };
    v.sort_by(|a, b| key(&b.id).cmp(&key(&a.id)));
    v
}

#[allow(clippy::too_many_arguments)]
pub fn states_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    gamepads: Query<&Gamepad>,
    mut menu: ResMut<StatesMenu>,
    mut dev: ResMut<DevMenu>,
    play: Option<ResMut<crate::PlayState>>,
    archive: Option<ResMut<crate::Archive>>,
    mut panels: Query<(Entity, &mut Text), With<StatesPanel>>,
    mut movies: Option<ResMut<crate::movie_view::Movies>>,
) {
    let typed: Vec<Key> = typed.read().filter(|k| k.state.is_pressed()).map(|k| k.logical_key.clone()).collect();
    let Some(mut play) = play else { return };
    // NEOPETS_LOAD_STATE=name (or id): that save state loaded once, as soon as play has run a
    // few ticks (over a new game's opening movies, which are dropped), as the list's Enter does
    // (a test tool, e.g. to host from a state without opening the list).
    let mut archive = archive;
    if !menu.auto_loaded && play.play.ticks > 2 {
        if let Ok(want) = std::env::var("NEOPETS_LOAD_STATE") {
            menu.auto_loaded = true;
            let want = want.trim().to_lowercase();
            let found = read_entries().into_iter().find(|e| e.id.to_lowercase() == want || e.name.trim().to_lowercase() == want);
            match (found, archive.as_deref_mut()) {
                (Some(e), Some(a)) => {
                    let crate::Archive { disc, index } = a;
                    match crate::load_state(disc, index, &play.play, &e.id) {
                        Some(Ok(new)) => {
                            play.play = new;
                            if let Some(m) = movies.as_deref_mut() {
                                m.drop_all();
                            }
                            match a.disc.reopen() {
                                Ok(second) => commands.insert_resource(crate::Streamer::start(second, a.index.clone(), play.play.boot.clone())),
                                Err(err) => error!("No loader thread, streaming is off: {err}"),
                            }
                            dev.toast = Some((format!("Loaded \"{}\"", e.name), 150));
                            info!("NEOPETS_LOAD_STATE: loaded \"{}\" ({})", e.name, e.id);
                        }
                        Some(Err(err)) => error!("NEOPETS_LOAD_STATE: could not load {}: {err}", e.id),
                        None => error!("NEOPETS_LOAD_STATE: {}'s files are gone", e.id),
                    }
                }
                (None, _) => error!("NEOPETS_LOAD_STATE: no save state named {want}"),
                (_, None) => error!("NEOPETS_LOAD_STATE: no disc"),
            }
        }
    }
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    if !dev.states_open {
        // F9 opens the list, unless the report note or the developer menu has the keys.
        let test = !menu.tested && std::env::var_os("NEOPETS_TEST_STATES").is_some() && play.play.ticks > 120;
        if (keys.just_pressed(KeyCode::F9) || test) && !dev.debug.note_open && !dev.open {
            menu.tested = true;
            menu.speed = dev.note_begin();
            dev.states_open = true;
            menu.entries = read_entries();
            menu.cursor = 0;
            menu.naming = None;
            menu.confirm = None;
            menu.line = String::new();
        } else {
            return;
        }
    } else if let Some(mut name) = menu.naming.take() {
        // Typing the new state's name: Enter saves, Esc goes back to the list.
        let mut done = None;
        for k in &typed {
            match k {
                Key::Enter => done = Some(true),
                Key::Escape => done = Some(false),
                Key::Backspace => {
                    name.pop();
                }
                Key::Space => name.push(' '),
                Key::Character(c) if name.chars().count() < 60 => name.push_str(c),
                _ => {}
            }
        }
        if pad(GamepadButton::East) {
            done = Some(false);
        }
        match done {
            Some(true) => {
                let label = if name.trim().is_empty() { "Unnamed".to_string() } else { name.trim().to_string() };
                match crate::save_state(&play.play, &label) {
                    Ok(_) => {
                        menu.entries = read_entries();
                        menu.cursor = 1;
                        menu.line = format!("Saved \"{label}\"");
                    }
                    Err(e) => menu.line = format!("Could not save: {e}"),
                }
            }
            Some(false) => menu.line = String::new(),
            None => menu.naming = Some(name),
        }
    } else {
        let rows = menu.entries.len() + 1;
        let close = keys.just_pressed(KeyCode::F9) || typed.contains(&Key::Escape) || pad(GamepadButton::East);
        if keys.just_pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown) {
            menu.cursor = (menu.cursor + 1) % rows;
            menu.confirm = None;
        }
        if keys.just_pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp) {
            menu.cursor = (menu.cursor + rows - 1) % rows;
            menu.confirm = None;
        }
        let enter = typed.contains(&Key::Enter) || pad(GamepadButton::South);
        if close {
            dev.states_open = false;
            dev.note_end(menu.speed);
        } else if enter && menu.cursor == 0 {
            if play.play.state_ready() {
                menu.naming = Some(String::new());
                menu.line = String::new();
            } else {
                menu.line = "Not during a scene: close the list and try again when it ends".into();
            }
        } else if enter {
            let k = menu.cursor - 1;
            let (id, name) = (menu.entries[k].id.clone(), menu.entries[k].name.clone());
            match archive {
                Some(mut archive) => {
                    let crate::Archive { disc, index } = &mut *archive;
                    match crate::load_state(disc, index, &play.play, &id) {
                        Some(Ok(new)) => {
                            play.play = new;
                            match archive.disc.reopen() {
                                Ok(second) => commands.insert_resource(crate::Streamer::start(second, archive.index.clone(), play.play.boot.clone())),
                                Err(e) => error!("No loader thread, streaming is off: {e}"),
                            }
                            dev.states_open = false;
                            dev.note_end(menu.speed);
                            dev.toast = Some((format!("Loaded \"{name}\""), 150));
                        }
                        Some(Err(e)) => menu.line = format!("Could not load: {e}"),
                        None => menu.line = "That state's files are gone".into(),
                    }
                }
                None => menu.line = "No disc to load from".into(),
            }
        } else if menu.cursor > 0 && (keys.just_pressed(KeyCode::Delete) || pad(GamepadButton::West)) {
            let k = menu.cursor - 1;
            if menu.confirm == Some(k) {
                let dir = crate::states_dir();
                let e = &menu.entries[k];
                let gone = std::fs::remove_file(format!("{dir}/{}.bin", e.id)).and_then(|_| std::fs::remove_file(format!("{dir}/{}.txt", e.id)));
                menu.line = match gone {
                    Ok(()) => format!("Deleted \"{}\"", e.name),
                    Err(err) => format!("Could not delete: {err}"),
                };
                menu.confirm = None;
                menu.entries = read_entries();
                menu.cursor = menu.cursor.min(menu.entries.len());
            } else {
                menu.confirm = Some(k);
                menu.line = format!("Press Delete again to delete \"{}\"", menu.entries[k].name);
            }
        }
    }
    if !dev.states_open {
        for (e, _) in &panels {
            commands.entity(e).despawn();
        }
        return;
    }
    // The panel's text.
    let mut body = String::from("SAVE STATES\n\n");
    if let Some(name) = &menu.naming {
        body.push_str(&format!("Name for the new state:\n\n  {name}_\n\nEnter saves, Esc goes back"));
    } else {
        let row = |k: usize, text: String| format!("{} {text}\n", if menu.cursor == k { ">" } else { " " });
        body.push_str(&row(0, "+ New save state".into()));
        // Up to 15 rows around the cursor, so a long list scrolls.
        let first = menu.cursor.saturating_sub(8).min(menu.entries.len().saturating_sub(15));
        if first > 0 {
            body.push_str("   ...\n");
        }
        for (k, e) in menu.entries.iter().enumerate().skip(first).take(15) {
            body.push_str(&row(k + 1, format!("{}   (world {}, {})", e.name, e.pack, e.date)));
        }
        if first + 15 < menu.entries.len() {
            body.push_str("   ...\n");
        }
        if menu.entries.is_empty() {
            body.push_str("\n  (no save states yet)\n");
        }
        body.push_str("\nUp/Down pick, Enter makes or loads, Delete deletes, F9 or Esc closes");
    }
    if !menu.line.is_empty() {
        body.push_str(&format!("\n\n{}", menu.line));
    }
    match panels.iter_mut().next() {
        Some((_, mut t)) => {
            if t.0 != body {
                t.0 = body;
            }
        }
        None => {
            commands.spawn((
                Text::new(body),
                TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                Node { position_type: PositionType::Absolute, left: percent(20), right: percent(20), top: percent(15), padding: UiRect::all(px(14)), ..default() },
                GlobalZIndex(5000),
                StatesPanel,
            ));
        }
    }
}
