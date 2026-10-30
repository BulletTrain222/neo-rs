//! The developer menu: cheats and a free camera for testing. It is a tool of this build, not
//! part of the original game, and follows no spec.
//!
//! F1 (or the controller chord set on the Gamepad bindings tab, LB + L3 + A by default) opens
//! and closes the menu; F4 pauses or resumes
//! the game and F5 steps one tick while it is paused. In the menu: Up and Down (the D-pad) pick an
//! entry, Left and Right change its value (Shift, or LB / RB, for steps of 10), Enter, Space or
//! the south button (Cross / A) applies it, and Escape or the east button (Circle / B) closes the
//! menu. The mouse works too: hovering picks a row, clicking applies it, and the < and > buttons
//! change its value. F2 toggles the free camera: WASD or the arrows
//! move, E and Q rise and sink, J/L and I/K or a right-button drag look around, Shift is fast.
//! On a controller the menu's Free camera row turns it on (and closes the menu) and the east
//! button (Circle / B) ends it; the left stick moves, the right stick looks, the right and left
//! triggers (R2, L2) rise and sink, and holding the south button (Cross / A) is fast.
//! While the menu is open or the free camera is on, the hero and the game menus get no input.
//! Warp rebuilds the game in another world pack (the save's world-key list), keeping the
//! session (flags, items, Neopoints, maps, the heroes' pools and equipment, cheats, options).

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::orbit::Orbit;
use crate::PlayState;

#[derive(Resource)]
pub struct DevMenu {
    pub open: bool,
    cursor: usize,
    amount: usize,
    item: u16,
    flag: u16,
    /// The warp list's cursor, and a warp the host runs after this frame's ticks.
    warp_at: usize,
    pub warp: Option<u32>,
    /// The boss list's cursor, and a boss fight the host builds after this frame's ticks.
    boss_at: usize,
    pub boss: Option<usize>,
    /// The game speed (an index into SPEEDS; 0 is paused) and a one-tick step asked for.
    speed: usize,
    pub step: bool,
    start: usize,
    pub free: Option<FreeCam>,
    note: String,
    /// A short message shown at the top of the screen for its frame count (F8 view saves).
    pub toast: Option<(String, u32)>,
    /// The weapon list's cursor.
    weapon_at: usize,
    /// A mouse click this frame: apply the cursor's entry, or step its value.
    clicked: bool,
    click_step: i32,
    /// The input that closed the menu is kept from the game for this frame.
    swallow: bool,
    /// The warp list's area names by world pack (the pack's map area named by design table 5,
    /// "" when the pack has none), read from the disc by the host as the cursor reaches them.
    pub warp_names: std::collections::HashMap<u32, String>,
    /// The quest rows (design table 1: title, start flag, done flag), read once when the menu
    /// first opens, to name the story flags.
    quests: Option<Vec<neopets_sim::mappage::Quest>>,
}

/// The game speeds: paused, a quarter, a half, normal, double, four times.
pub const SPEEDS: [f64; 6] = [0.0, 0.25, 0.5, 1.0, 2.0, 4.0];
const NORMAL: usize = 3;

impl Default for DevMenu {
    fn default() -> Self {
        Self { open: false, cursor: 0, amount: 0, item: 0, flag: 0, warp_at: 0, warp: None, boss_at: 0, boss: None, speed: NORMAL, step: false, start: 0, free: None, note: String::new(), toast: None, weapon_at: 0, clicked: false, click_step: 0, swallow: false, warp_names: Default::default(), quests: None }
    }
}

impl DevMenu {
    /// The factor applied to the frame's elapsed time (0 while paused).
    pub fn speed(&self) -> f64 {
        SPEEDS[self.speed]
    }

    /// The hero and the game menus get no input this frame.
    pub fn blocks_input(&self) -> bool {
        self.open || self.free.is_some() || self.swallow
    }
}

/// The free camera's place and look angles (radians; yaw about +Y from +Z, pitch up).
#[derive(Clone, Copy, Debug)]
pub struct FreeCam {
    pos: Vec3,
    yaw: f32,
    pitch: f32,
}

impl FreeCam {
    fn from_transform(t: &Transform) -> Self {
        let f = t.forward().as_vec3();
        Self { pos: t.translation, yaw: f.x.atan2(f.z), pitch: f.y.clamp(-1.0, 1.0).asin() }
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), self.yaw.cos() * self.pitch.cos())
    }
}

#[derive(Component)]
pub struct DevPanel;

#[derive(Component)]
pub struct DevText;

const AMOUNTS: [i32; 4] = [100, 1_000, 10_000, 100_000];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    God,
    Magic,
    Charge,
    FreeCam,
    Neopoints,
    Heal,
    Motes,
    Item,
    Hero,
    Flag,
    Warp,
    Speed,
    Kill,
    Drop,
    Start,
    ToCamera,
    Maps,
    Weapon,
    Noclip,
    CloudBeast,
    BossFight,
}

/// The menu's sections and their entries, in display order.
const SECTIONS: [(&str, &[Entry]); 6] = [
    ("CHEATS", &[Entry::God, Entry::Noclip, Entry::Magic, Entry::Charge]),
    ("TIME AND CAMERA", &[Entry::Speed, Entry::FreeCam, Entry::ToCamera]),
    ("TRAVEL", &[Entry::Warp, Entry::Start]),
    ("HERO", &[Entry::Heal, Entry::Hero, Entry::Kill, Entry::BossFight, Entry::CloudBeast]),
    ("ITEMS AND MONEY", &[Entry::Weapon, Entry::Item, Entry::Drop, Entry::Motes, Entry::Maps, Entry::Neopoints]),
    ("STORY", &[Entry::Flag]),
];

const ENTRIES: [Entry; 21] = [
    Entry::God,
    Entry::Noclip,
    Entry::Magic,
    Entry::Charge,
    Entry::Speed,
    Entry::FreeCam,
    Entry::ToCamera,
    Entry::Warp,
    Entry::Start,
    Entry::Heal,
    Entry::Hero,
    Entry::Kill,
    Entry::BossFight,
    Entry::CloudBeast,
    Entry::Weapon,
    Entry::Item,
    Entry::Drop,
    Entry::Motes,
    Entry::Maps,
    Entry::Neopoints,
    Entry::Flag,
];

/// How a row behaves: an on/off switch, a value changed with < and >, or a one-shot action.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Toggle,
    Choice,
    Action,
}

/// The number of rows in the left column (the first three sections).
const LEFT: usize = {
    let mut n = 0;
    let mut k = 0;
    while k < 3 {
        n += SECTIONS[k].1.len();
        k += 1;
    }
    n
};

/// The rows of the column holding row `i`, as (first, count).
fn column_of(i: usize) -> (usize, usize) {
    if i < LEFT { (0, LEFT) } else { (LEFT, ENTRIES.len() - LEFT) }
}

/// Row `i` moved `d` rows within its column, wrapping.
fn column_step(i: usize, d: i32) -> usize {
    let (first, n) = column_of(i);
    first + (i as i32 - first as i32 + d).rem_euclid(n as i32) as usize
}

/// The row at the same height in the other column (its last row when that column is shorter).
fn other_column(i: usize) -> usize {
    let (first, _) = column_of(i);
    let (to, n) = column_of(if first == 0 { LEFT } else { 0 });
    to + (i - first).min(n - 1)
}

fn kind(e: Entry) -> Kind {
    match e {
        Entry::God | Entry::Noclip | Entry::Magic | Entry::Charge | Entry::FreeCam => Kind::Toggle,
        Entry::Speed | Entry::Warp | Entry::Start | Entry::Weapon | Entry::Item | Entry::Neopoints | Entry::Flag | Entry::BossFight => Kind::Choice,
        _ => Kind::Action,
    }
}

/// The weapons of the item table (types 3, Tor's, and 6, Roberta's), by id.
fn weapons(p: &crate::play::Play) -> Vec<u16> {
    p.items.items.iter().filter(|i| i.is_weapon()).map(|i| i.id).collect()
}

/// The warp list: the save's world keys (nonzero, sorted, each once).
/// The world packs whose area names the open menu shows (the warp cursor's and the current
/// world's) and that the host has not read yet.
pub fn warp_names_wanted(dev: &DevMenu, p: &crate::play::Play) -> Vec<u32> {
    if !dev.open {
        return Vec::new();
    }
    [worlds(p).get(dev.warp_at).copied(), Some(p.world.pack)].into_iter().flatten().filter(|k| !dev.warp_names.contains_key(k)).collect()
}

/// The quest steps whose start flag (true) or done flag (false) is n: (start, title, objective).
fn flag_quests(dev: &DevMenu, n: u16) -> Vec<(bool, String, String)> {
    let text = |b: &[u8]| String::from_utf8_lossy(b).split_whitespace().collect::<Vec<_>>().join(" ");
    let mut v = Vec::new();
    for q in dev.quests.iter().flatten() {
        if q.start == n as i32 {
            v.push((true, text(&q.title), text(&q.objective)));
        }
        if q.done == n as i32 {
            v.push((false, text(&q.title), text(&q.objective)));
        }
    }
    v
}

/// The story flag row's detail: the flag's state and its quest steps.
fn detail(dev: &DevMenu, p: &crate::play::Play) -> String {
    {
        {
            let n = dev.flag;
            let state = if p.state.flag(n) { "set" } else { "clear" };
            let q = flag_quests(dev, n);
            let starts: Vec<&(bool, String, String)> = q.iter().filter(|e| e.0).collect();
            // A step that ends here without a following step of the same quest ends that quest.
            let ends: Vec<&(bool, String, String)> = q.iter().filter(|e| !e.0 && !starts.iter().any(|s| s.1 == e.1)).collect();
            let mut parts: Vec<String> = starts.iter().map(|(_, t, o)| if o.is_empty() { format!("{t} begins") } else { format!("{t}: {o}") }).collect();
            parts.extend(ends.iter().map(|(_, t, _)| format!("{t} is finished")));
            let s = if parts.is_empty() { format!("Flag {n} is {state}; no quest step uses it") } else { format!("Flag {n} is {state}. {}", parts.join(" | ")) };
            if s.chars().count() > 190 { format!("{}...", s.chars().take(187).collect::<String>()) } else { s }
        }
    }
}

fn area(dev: &DevMenu, pack: u32) -> Option<&str> {
    dev.warp_names.get(&pack).map(String::as_str).filter(|n| !n.is_empty())
}

fn worlds(p: &crate::play::Play) -> Vec<u32> {
    let mut v: Vec<u32> = p.world_keys.0.iter().copied().filter(|&k| k > 0).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// The menu window (its rows are buttons) and the small status box shown while the menu is
/// closed (the game speed and the free camera).
#[derive(Component)]
pub struct DevWindow;

/// A row of the menu by its entry index; its label, value and < > buttons.
#[derive(Component)]
pub struct DevRow(usize);
#[derive(Component)]
pub struct DevLabel(usize);
#[derive(Component)]
pub struct DevValue(usize);
#[derive(Component)]
pub struct DevStep(usize, i32);
#[derive(Component)]
pub struct DevNote;
/// The story flag's detail under its row (its state and quest steps).
#[derive(Component)]
pub struct DevDetail;
/// A section title; the travel title also says where the hero is.
#[derive(Component)]
pub struct DevHeader(usize);

const PANEL: Color = Color::srgba(0.05, 0.06, 0.10, 0.96);
const ROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
const ROW_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.28);
const ARROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);
const ARROW_HOVER: Color = Color::srgba(0.95, 0.75, 0.25, 0.55);
const GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.96);
const DIM: Color = Color::srgb(0.60, 0.63, 0.70);
const ON: Color = Color::srgb(0.45, 0.90, 0.50);
const OFF: Color = Color::srgb(0.85, 0.45, 0.45);

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_panel(mut commands: Commands) {
    // The status box (menu closed).
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: px(12), top: px(12), max_width: percent(60), padding: UiRect::all(px(10)), display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            GlobalZIndex(5000),
            DevPanel,
        ))
        .with_children(|ui| {
            ui.spawn((Text::default(), font(15.0), TextColor(Color::srgb(0.9, 0.95, 0.75)), DevText));
        });
    // The menu window.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: px(16),
                width: px(860),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(14)),
                row_gap: px(3),
                display: Display::None,
                ..default()
            },
            crate::ui_fit::FitToWindow::Left,
            BackgroundColor(PANEL),
            GlobalZIndex(5002),
            DevWindow,
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("Developer menu"), font(22.0), TextColor(GOLD)));
            ui.spawn((Text::new("A test tool of this build, not part of the game"), font(13.0), TextColor(DIM)));
            ui.spawn((
                Text::new("Keys: arrows pick and change (Shift x10), Tab other column, Enter apply, Esc/F1 close; F2 free camera, F4 pause, F5 step
Pad: LB+L3+A (rebindable) open/close, D-pad pick and change (LB/RB x10), LT/RT other column, A apply, B close | Mouse: click rows and < >"),
                font(12.0),
                TextColor(DIM),
                Node { margin: UiRect { top: px(4), bottom: px(4), ..default() }, ..default() },
            ));
            // Two columns: the first three sections on the left, the rest on the right.
            let mut k = 0;
            ui.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(16), ..default() }).with_children(|cols| {
              for half in [&SECTIONS[..3], &SECTIONS[3..]] {
               cols.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3), width: px(408), ..default() }).with_children(|ui| {
                for (title, entries) in half {
                let entries: &[Entry] = entries;
                let section = SECTIONS.iter().position(|s| s.0 == *title).unwrap_or(0);
                ui.spawn((Text::new(*title), font(12.0), TextColor(GOLD), Node { margin: UiRect { top: px(4), bottom: px(1), ..default() }, ..default() }, DevHeader(section)));
                for &e in entries {
                    debug_assert!(ENTRIES[k] == e);
                    let choice = kind(e) == Kind::Choice;
                    ui.spawn((
                        Button,
                        Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(px(10), px(4)), min_height: px(24), ..default() },
                        BackgroundColor(ROW),
                        DevRow(k),
                    ))
                    .with_children(|row| {
                        row.spawn((Text::default(), font(14.0), TextColor(TEXT), Node { max_width: px(270), ..default() }, TextLayout::no_wrap(), DevLabel(k)));
                        row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|right| {
                            let arrow = |right: &mut ChildSpawnerCommands, d: i32| {
                                right
                                    .spawn((Button, Node { width: px(22), height: px(20), justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: if choice { Display::Flex } else { Display::None }, ..default() }, BackgroundColor(ARROW), DevStep(k, d)))
                                    .with_children(|b| {
                                        b.spawn((Text::new(if d < 0 { "<" } else { ">" }), font(14.0), TextColor(TEXT)));
                                    });
                            };
                            arrow(right, -1);
                            right.spawn((Text::default(), font(14.0), TextColor(GOLD), Node { min_width: px(44), justify_content: JustifyContent::Center, ..default() }, TextLayout::justify(Justify::Center).with_no_wrap(), DevValue(k)));
                            arrow(right, 1);
                        });
                    });
                    k += 1;
                }
                // Under the story flag row: the flag's quest steps (design table 1).
                if entries.contains(&Entry::Flag) {
                    ui.spawn((Text::default(), font(13.0), TextColor(TEXT), Node { max_width: px(400), margin: UiRect::left(px(4)), ..default() }, DevDetail));
                }
                }
               });
              }
            });
            ui.spawn((Text::default(), font(14.0), TextColor(ON), Node { margin: UiRect::top(px(8)), ..default() }, DevNote));
        });
}

/// The mouse on the menu: hovering a row moves the cursor to it; a click applies it, and a
/// click on < or > steps its value. Runs before `dev_input`, which acts on the click.
pub fn dev_mouse(mut dev: ResMut<DevMenu>, rows: Query<(&Interaction, &DevRow), Changed<Interaction>>, steps: Query<(&Interaction, &DevStep), Changed<Interaction>>) {
    let _span = crate::perf::span("dev_mouse");
    if !dev.open {
        return;
    }
    for (i, r) in &rows {
        match i {
            Interaction::Hovered => dev.cursor = r.0,
            Interaction::Pressed => {
                dev.cursor = r.0;
                dev.clicked = true;
            }
            Interaction::None => {}
        }
    }
    for (i, s) in &steps {
        if *i == Interaction::Pressed {
            dev.cursor = s.0;
            dev.click_step = s.1;
        }
    }
}

/// The menu's keys and actions, before the game reads its input.
pub fn dev_input(keys: Res<ButtonInput<KeyCode>>, gamepads: Query<&Gamepad>, bindings: Res<crate::bindings::Bindings>, page: Res<crate::controls_page::ControlsPage>, mut dev: ResMut<DevMenu>, play: Option<ResMut<PlayState>>, cameras: Query<&Transform, With<Orbit>>, mut perf: ResMut<crate::perf::PerfOverlay>, mut frames: Local<u32>) {
    let _span = crate::perf::span("dev_input");
    let Some(mut play) = play else { return };
    if dev.open && dev.quests.is_none() {
        dev.quests = Some(play.play.map_tables().0);
        // NEOPETS_DEV containing "quests": list each quest's start and done flags (diagnostic).
        if std::env::var("NEOPETS_DEV").is_ok_and(|v| v.contains("quests")) {
            for q in dev.quests.iter().flatten() {
                println!("quest start {} done {} {}", q.start, q.done, String::from_utf8_lossy(&q.title));
            }
        }
    }
    let toggle_free = |dev: &mut DevMenu| {
        dev.free = match dev.free {
            Some(_) => None,
            None => cameras.iter().next().map(FreeCam::from_transform),
        };
    };
    // NEOPETS_DEV="menu,free,perf,warp=N,speed=K,drop,view=NAME": open the menu, start the free camera, show
    // the performance overlay, warp to world pack N and/or set speed step K (0 paused, 3 normal) once play has run a few frames
    // (diagnostic, for screenshots); with "free" the camera also backs up and rises.
    *frames += 1;
    if *frames == 30 {
        let v = std::env::var("NEOPETS_DEV").unwrap_or_default();
        dev.open |= v.contains("menu");
        perf.on |= v.contains("perf");
        dev.warp = v.split(',').find_map(|t| t.trim().strip_prefix("warp=")?.parse().ok());
        // "row=K" and "flag=N": the menu cursor on row K and the story flag row at N (screenshots).
        if let Some(k) = v.split(',').find_map(|t| t.trim().strip_prefix("row=")?.parse::<usize>().ok()) {
            dev.cursor = k.min(ENTRIES.len() - 1);
        }
        if let Some(n) = v.split(',').find_map(|t| t.trim().strip_prefix("flag=")?.parse::<u16>().ok()) {
            dev.flag = n;
        }
        if let Some(k) = v.split(',').find_map(|t| t.trim().strip_prefix("speed=")?.parse::<usize>().ok()) {
            dev.speed = k.min(SPEEDS.len() - 1);
        }
        // "view=NAME" (or "view=NAME:D"): the free camera D units (default 4) in front of the
        // named character, looking at its chest.
        if let Some(arg) = v.split(',').find_map(|t| t.trim().strip_prefix("view=")) {
            let (name, d) = arg.split_once(':').map_or((arg, 4.0), |(n, d)| (n, d.parse().unwrap_or(4.0)));
            if let Some(c) = play.play.characters.iter().find(|c| c.name == name) {
                let p = Vec3::from(c.position);
                let f = Vec3::new(c.facing[0], 0.0, c.facing[2]).normalize_or(Vec3::Z);
                let pos = p + f * d + Vec3::Y * 2.2;
                let to = p + Vec3::Y * 1.8 - pos;
                dev.free = Some(FreeCam { pos, yaw: to.x.atan2(to.z), pitch: (to.y / to.length()).asin() });
            }
        }
        if v.contains("free") && dev.free.is_none() {
            toggle_free(&mut dev);
            if let Some(fc) = dev.free.as_mut() {
                let f = fc.forward();
                // "freehigh" backs much further away and looks down more, for overviews.
                let (back, up, tilt) = if v.contains("freehigh") { (90.0, 70.0, 0.6) } else { (25.0, 15.0, 0.35) };
                fc.pos += -f * back + Vec3::Y * up;
                fc.pitch -= tilt;
            }
        }
    }
    // NEOPETS_DEV_BOSS=n: the boss fight request for boss n, once, a second in (diagnostic).
    if *frames == 60 {
        if let Some(n) = std::env::var("NEOPETS_DEV_BOSS").ok().and_then(|v| v.parse::<usize>().ok()).filter(|&n| n < crate::play::BOSSES.len()) {
            dev.boss = Some(n);
        }
    }
    // NEOPETS_DEV_QUEST: every quest item (category 7) given, two seconds in (diagnostic, for
    // the item screen's Quest-items tab).
    if *frames == 120 && std::env::var_os("NEOPETS_DEV_QUEST").is_some() {
        let ids = play.play.items_of_category(7);
        for &id in &ids {
            play.play.give(id);
        }
        info!("Developer quest items: {ids:?}");
    }
    // NEOPETS_DEV="drop": the coin drop (with item 160) a second after the options apply.
    if *frames == 90 && std::env::var("NEOPETS_DEV").is_ok_and(|v| v.contains("drop")) {
        info!("Developer drop: {} world items", play.play.dev_drop(160));
    }
    dev.swallow = false;
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    let pad_held = |b: GamepadButton| gamepads.iter().any(|g| g.pressed(b));
    // The chord: every one of its buttons held, the last pressed this frame (not while the
    // bindings page is up, where the chord is being set).
    let chord_fired = !page.open && gamepads.iter().any(|g| crate::bindings::chord_fired(&bindings.dev_chord, |b| g.pressed(b), |b| g.just_pressed(b)));
    // F8: the view to private/views.txt (the world, the hero's position and the drawn camera),
    // one numbered line per press, for reproducing what the player sees (a test tool of this
    // build). NEOPETS_VIEW=n (or "last") starts there again.
    if keys.just_pressed(KeyCode::F8) {
        if let Some(t) = cameras.iter().next() {
            let fc = dev.free.unwrap_or_else(|| FreeCam::from_transform(t));
            let h = play.play.loco.position();
            let path = "private/views.txt";
            let n = std::fs::read_to_string(path).map_or(0, |s| s.lines().filter(|l| l.starts_with("view ")).count()) + 1;
            let line = format!(
                "view {n}: pack {} hero {:.2},{:.2},{:.2} cam {:.3},{:.3},{:.3},{:.5},{:.5} tick {}
",
                play.play.world.pack, h[0], h[2], h[1], fc.pos.x, fc.pos.y, fc.pos.z, fc.yaw, fc.pitch, play.play.ticks
            );
            let _ = std::fs::create_dir_all("private");
            match std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes())) {
                Ok(()) => {
                    info!("Saved {}", line.trim());
                    dev.toast = Some((format!("View {n} saved (F8)"), 150));
                }
                Err(e) => error!("F8: could not write {path}: {e}"),
            }
        }
    }
    // The Cloud Beast fight's one-time hint (this build's addition), on the same toast.
    if let Some(h) = play.play.cloud_hint.take() {
        dev.toast = Some((h, 480));
    }
    if let Some((_, f)) = dev.toast.as_mut() {
        *f = f.saturating_sub(1);
        if *f == 0 {
            dev.toast = None;
        }
    }
    // NEOPETS_VIEW=n or "last": the camera of saved view n, as the free camera, once play has
    // run a second (the world and the hero position come from the launch: --pack and --start).
    if *frames == 60 {
        if let Some(v) = std::env::var("NEOPETS_VIEW").ok() {
            let text = std::fs::read_to_string("private/views.txt").unwrap_or_default();
            let lines: Vec<&str> = text.lines().filter(|l| l.starts_with("view ")).collect();
            let line = if v == "last" { lines.last().copied() } else { lines.iter().copied().find(|l| l.starts_with(&format!("view {v}:"))) };
            if let Some(cam) = line.and_then(|l| l.split(" cam ").nth(1)).and_then(|r| r.split_whitespace().next()) {
                let n: Vec<f32> = cam.split(',').filter_map(|x| x.parse().ok()).collect();
                if n.len() == 5 {
                    dev.free = Some(FreeCam { pos: Vec3::new(n[0], n[1], n[2]), yaw: n[3], pitch: n[4] });
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::F1) || chord_fired {
        dev.open = !dev.open;
        dev.swallow = true;
        if chord_fired {
            return;
        }
    }
    if dev.open && (keys.just_pressed(KeyCode::Escape) || pad(GamepadButton::East)) {
        dev.open = false;
        dev.swallow = true;
        return;
    }
    if keys.just_pressed(KeyCode::F2) {
        toggle_free(&mut dev);
    }
    // B ends the free camera while the menu is closed.
    if !dev.open && dev.free.is_some() && pad(GamepadButton::East) {
        dev.free = None;
        dev.swallow = true;
        return;
    }
    // T or the gamepad's Y while the free camera is on: the hero to the ground under the camera,
    // facing the way it looks, and the free camera ends (a test tool of this build).
    if !dev.open && (keys.just_pressed(KeyCode::KeyT) || pad(GamepadButton::North)) {
        if let Some(fc) = dev.free {
            let f = fc.forward();
            if play.play.dev_place([fc.pos.x, fc.pos.y, fc.pos.z], [f.x, 0.0, f.z]) {
                dev.free = None;
                dev.swallow = true;
                dev.toast = Some(("Tor moved here".into(), 90));
                return;
            }
            dev.toast = Some(("No ground under the camera".into(), 90));
        }
    }
    if keys.just_pressed(KeyCode::F4) {
        dev.speed = if dev.speed == 0 { NORMAL } else { 0 };
    }
    if keys.just_pressed(KeyCode::F5) && dev.speed == 0 {
        dev.step = true;
    }
    if !dev.open {
        return;
    }
    // Up and down move within the cursor's column (wrapping); LT or RT (Tab) jump to the other
    // column at the same height, and so do Left and Right on a row with no value to change.
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) || pad(GamepadButton::DPadDown) {
        dev.cursor = column_step(dev.cursor, 1);
    }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) || pad(GamepadButton::DPadUp) {
        dev.cursor = column_step(dev.cursor, -1);
    }
    let sideways = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) || keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft);
    if keys.just_pressed(KeyCode::Tab) || pad(GamepadButton::LeftTrigger2) || pad(GamepadButton::RightTrigger2) || (sideways && kind(ENTRIES[dev.cursor]) != Kind::Choice) {
        dev.cursor = other_column(dev.cursor);
        return;
    }
    let tens = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) || pad_held(GamepadButton::LeftTrigger) || pad_held(GamepadButton::RightTrigger);
    let step: i32 = if tens { 10 } else { 1 };
    let delta = if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) {
        step
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft) {
        -step
    } else {
        std::mem::take(&mut dev.click_step) * step
    };
    let entry = ENTRIES[dev.cursor];
    if delta != 0 {
        match entry {
            Entry::Neopoints => dev.amount = (dev.amount as i32 + delta.signum()).clamp(0, AMOUNTS.len() as i32 - 1) as usize,
            Entry::Item => dev.item = (dev.item as i32 + delta).clamp(0, play.play.items.items.len().saturating_sub(1) as i32) as u16,
            Entry::Weapon => {
                let n = weapons(&play.play).len().max(1) as i32;
                dev.weapon_at = (dev.weapon_at as i32 + delta).rem_euclid(n) as usize;
            }
            Entry::Flag => dev.flag = (dev.flag as i32 + delta).clamp(0, u16::MAX as i32) as u16,
            Entry::Warp => {
                let n = worlds(&play.play).len().max(1) as i32;
                dev.warp_at = (dev.warp_at as i32 + delta).rem_euclid(n) as usize;
            }
            Entry::BossFight => dev.boss_at = (dev.boss_at as i32 + delta.signum()).rem_euclid(crate::play::BOSSES.len() as i32) as usize,
            Entry::Speed => dev.speed = (dev.speed as i32 + delta.signum()).clamp(0, SPEEDS.len() as i32 - 1) as usize,
            Entry::Start => {
                let n = play.play.dev_start_points().max(1) as i32;
                dev.start = (dev.start as i32 + delta).rem_euclid(n) as usize;
            }
            _ => {}
        }
    }
    let clicked = std::mem::take(&mut dev.clicked);
    if !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) || keys.just_pressed(KeyCode::Space) || pad(GamepadButton::South) || clicked) {
        return;
    }
    let p = &mut play.play;
    let on = |b: bool| if b { "on" } else { "off" };
    dev.note = match entry {
        Entry::God => {
            p.cheats.god = !p.cheats.god;
            format!("God mode {}", on(p.cheats.god))
        }
        Entry::Magic => {
            p.cheats.magic = !p.cheats.magic;
            format!("Infinite magic {}", on(p.cheats.magic))
        }
        Entry::Charge => {
            p.cheats.charge = !p.cheats.charge;
            format!("Infinite Mote charge {}", on(p.cheats.charge))
        }
        Entry::FreeCam => {
            toggle_free(&mut dev);
            // Turned on from the menu: the menu closes so the camera can move at once (it is
            // held still while the menu is open).
            if dev.free.is_some() {
                dev.open = false;
                dev.swallow = true;
            }
            format!("Free camera {}", on(dev.free.is_some()))
        }
        Entry::Neopoints => {
            let a = AMOUNTS[dev.amount];
            p.state.neopoints = p.state.neopoints.saturating_add(a);
            format!("+{a} NP (now {})", p.state.neopoints)
        }
        Entry::Heal => {
            p.dev_heal();
            "Healed both heroes".into()
        }
        Entry::Motes => {
            for id in 160..=173 {
                p.give(id);
            }
            "Gave every Mote (charge 100 each)".into()
        }
        Entry::Noclip => {
            p.cheats.noclip = !p.cheats.noclip;
            format!("No clip {}", on(p.cheats.noclip))
        }
        Entry::Weapon => match weapons(p).get(dev.weapon_at).copied() {
            Some(id) => {
                p.give(id);
                format!("Gave weapon {id} ({}); a higher id already held stays equipped", p.item_label(id))
            }
            None => "No weapons in the item table".into(),
        },
        Entry::Item => {
            p.give(dev.item);
            format!("Gave item {} ({})", dev.item, p.item_label(dev.item))
        }
        Entry::Hero => {
            let before = p.hero_index;
            p.trace_select_hero(1 - before);
            if p.hero_index == before { "No other hero to switch to here".into() } else { format!("Now playing hero {}", p.hero_index) }
        }
        Entry::Flag => {
            let f = dev.flag;
            let v = !p.state.flag(f);
            p.state.set(f, v);
            format!("Story flag {f} {}", if v { "set" } else { "cleared" })
        }
        Entry::Speed => {
            dev.speed = NORMAL;
            "Normal speed".into()
        }
        Entry::Kill => format!("Defeated {} enemies here", p.dev_kill_enemies()),
        Entry::CloudBeast => p.dev_cloud_beast(),
        Entry::BossFight => {
            let b = crate::play::BOSSES[dev.boss_at % crate::play::BOSSES.len()];
            dev.boss = Some(dev.boss_at % crate::play::BOSSES.len());
            format!("Starting the {} fight (world pack {})", b.name, b.pack)
        }
        Entry::Drop => format!("Dropped {} world items", p.dev_drop(dev.item)),
        Entry::Start => {
            if p.dev_go_start(dev.start) { format!("To start point {}", dev.start) } else { "No start points here".into() }
        }
        Entry::ToCamera => match dev.free {
            Some(fc) => {
                p.place_at(fc.pos.x, fc.pos.z, fc.pos.y);
                "Hero moved under the free camera".into()
            }
            None => "Turn the free camera on first".into(),
        },
        Entry::Maps => {
            for n in 1..=46 {
                p.map_held(n);
            }
            "Every treasure map held".into()
        }
        Entry::Warp => match worlds(p).get(dev.warp_at).copied() {
            Some(pack) => {
                dev.warp = Some(pack);
                match area(&dev, pack) {
                    Some(n) => format!("Warping to {n} (world pack {pack})"),
                    None => format!("Warping to world pack {pack}"),
                }
            }
            None => "No world list".into(),
        },
    };
}

/// The free camera: moves and looks, then replaces the play camera's pose (after the play's
/// frame has set it).
pub fn free_camera(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, gamepads: Query<&Gamepad>, mut dev: ResMut<DevMenu>, mut cameras: Query<&mut Transform, With<Orbit>>) {
    let _span = crate::perf::span("free_camera");
    let open = dev.open;
    let Some(fc) = dev.free.as_mut() else { return };
    let dt = time.delta_secs().min(0.1);
    if !open {
        let key = |k: &[KeyCode]| if keys.any_pressed(k.iter().copied()) { 1.0 } else { 0.0 };
        let turn = 1.6 * dt;
        fc.yaw += (key(&[KeyCode::KeyJ]) - key(&[KeyCode::KeyL])) * turn;
        fc.pitch += (key(&[KeyCode::KeyI]) - key(&[KeyCode::KeyK])) * turn;
        if mouse.pressed(MouseButton::Right) {
            fc.yaw -= motion.delta.x * 0.004;
            fc.pitch -= motion.delta.y * 0.004;
        }
        // The controller: sticks past a small dead zone, triggers as rise and sink.
        let dead = |v: Vec2| if v.length() > 0.15 { v } else { Vec2::ZERO };
        let (mut stick, mut look, mut lift, mut fast) = (Vec2::ZERO, Vec2::ZERO, 0.0, false);
        for g in &gamepads {
            stick += dead(g.left_stick());
            look += dead(g.right_stick());
            lift += g.get(GamepadButton::RightTrigger2).unwrap_or(0.0) - g.get(GamepadButton::LeftTrigger2).unwrap_or(0.0);
            fast |= g.pressed(GamepadButton::South);
        }
        fc.yaw -= look.x * turn;
        fc.pitch += look.y * turn;
        fc.pitch = fc.pitch.clamp(-1.5, 1.5);
        let fast = fast || keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let speed = if fast { 120.0 } else { 30.0 } * dt;
        let f = fc.forward();
        let right = Vec3::new(-fc.yaw.cos(), 0.0, fc.yaw.sin());
        let ahead = (key(&[KeyCode::KeyW, KeyCode::ArrowUp]) - key(&[KeyCode::KeyS, KeyCode::ArrowDown]) + stick.y).clamp(-1.0, 1.0);
        let side = (key(&[KeyCode::KeyD, KeyCode::ArrowRight]) - key(&[KeyCode::KeyA, KeyCode::ArrowLeft]) + stick.x).clamp(-1.0, 1.0);
        let up = (key(&[KeyCode::KeyE]) - key(&[KeyCode::KeyQ]) + lift).clamp(-1.0, 1.0);
        fc.pos += (f * ahead + right * side + Vec3::Y * up) * speed;
    }
    let pose = Transform::from_translation(fc.pos).looking_to(fc.forward(), Vec3::Y);
    for mut t in &mut cameras {
        *t = pose;
    }
}

/// The label and value of a row.
fn row_text(e: Entry, dev: &DevMenu, p: &crate::play::Play) -> (String, String) {
    let on = |b: bool| if b { "ON".to_string() } else { "OFF".to_string() };
    match e {
        Entry::God => ("God mode".into(), on(p.cheats.god)),
        Entry::Noclip => ("No clip (walk through walls)".into(), on(p.cheats.noclip)),
        Entry::Magic => ("Infinite magic".into(), on(p.cheats.magic)),
        Entry::Charge => ("Infinite Mote charge".into(), on(p.cheats.charge)),
        Entry::FreeCam => ("Free camera (F2)".into(), on(dev.free.is_some())),
        Entry::Speed => ("Game speed (apply: 1x)".into(), speed_label(dev.speed)),
        Entry::ToCamera => ("Hero to the free camera".into(), String::new()),
        Entry::Warp => {
            let w = worlds(p);
            let target = w.get(dev.warp_at).copied();
            let label = match target.and_then(|k| area(dev, k)) {
                Some(n) => format!("Warp to {n}"),
                None => "Warp".to_string(),
            };
            (label, target.map_or("-".to_string(), |k| k.to_string()))
        }
        Entry::Start => (format!("Start point ({} here)", p.dev_start_points()), dev.start.to_string()),
        Entry::Heal => ("Full heal".into(), format!("{}/{} HP", p.hero_hp, p.hero_max)),
        Entry::Hero => (format!("Switch hero  (now {})", if p.hero_index == 0 { "Tor" } else { "Roberta" }), String::new()),
        Entry::Kill => ("Defeat every enemy here".into(), String::new()),
        Entry::CloudBeast => ("Spawn the Cloud Beast here (cut boss)".into(), String::new()),
        Entry::BossFight => {
            let n = crate::play::BOSSES.len();
            (format!("Boss fight: {}", crate::play::BOSSES[dev.boss_at % n].name), format!("{} of {n}", dev.boss_at % n + 1))
        }
        Entry::Weapon => {
            let w = weapons(p);
            match w.get(dev.weapon_at) {
                Some(&id) => (format!("Weapon: {}", p.item_label(id)), format!("{} {}", if p.items.get(id).is_some_and(|i| i.kind == 3) { "Tor" } else { "Rob" }, dev.weapon_at + 1)),
                None => ("Give weapon".into(), "-".into()),
            }
        }
        Entry::Item => (format!("Item: {}", p.item_label(dev.item)), dev.item.to_string()),
        Entry::Drop => ("Drop coins + that item here".into(), String::new()),
        Entry::Motes => ("Give every Mote".into(), String::new()),
        Entry::Maps => ("Give every treasure map".into(), String::new()),
        Entry::Neopoints => (format!("Add Neopoints ({} now)", p.state.neopoints), format!("+{}", AMOUNTS[dev.amount])),
        Entry::Flag => {
            let q = flag_quests(dev, dev.flag);
            let label = match q.iter().find(|e| e.0).or(q.first()) {
                Some((_, t, _)) => format!("Flag: {t}"),
                None => "Story flag".to_string(),
            };
            (label, format!("{} {}", dev.flag, if p.state.flag(dev.flag) { "set" } else { "clear" }))
        }
    }
}

/// A label's visible width in characters.
const FIT: usize = 28;

/// A label that fits one row (28 characters) as it is; a longer one keeps its kind ("Item: ",
/// "Boss fight: " and the like, up to the first ": ") in place and runs the rest as a marquee
/// at `t` seconds in the width left: its start held for 2 s, then scrolling left at 3
/// characters a second and wrapping round after a gap.
fn fit(s: String, t: f64) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= FIT {
        return s;
    }
    let kind = s.find(": ").map(|b| s[..b + 2].chars().count()).filter(|&k| k + 8 <= FIT).unwrap_or(0);
    let (head, rest) = chars.split_at(kind);
    let width = FIT - kind;
    const GAP: usize = 5;
    const RATE: f64 = 3.0;
    const HOLD: f64 = 2.0;
    let n = rest.len() + GAP;
    let phase = t.rem_euclid(HOLD + n as f64 / RATE);
    let start = (((phase - HOLD).max(0.0) * RATE) as usize).min(n - 1);
    head.iter().copied().chain((0..width).map(|k| rest.get((start + k) % n).copied().unwrap_or(' '))).collect()
}

/// The menu window's rows (labels, values, cursor and hover colours), the last result, and the
/// status box while the menu is closed.
#[allow(clippy::type_complexity)]
/// The F8 toast: one text node at the top middle while `DevMenu::toast` is set.
#[derive(Component)]
pub struct ViewToast;

pub fn show_toast(mut commands: Commands, dev: Res<DevMenu>, mut toasts: Query<(Entity, &mut Text), With<ViewToast>>) {
    match (&dev.toast, toasts.iter_mut().next()) {
        (Some((msg, _)), Some((_, mut t))) => {
            if t.0 != *msg {
                t.0 = msg.clone();
            }
        }
        (Some((msg, _)), None) => {
            commands.spawn((
                Text::new(msg.clone()),
                TextFont { font_size: bevy::text::FontSize::Px(26.0), ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.4)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                Node { position_type: PositionType::Absolute, top: px(16), left: percent(42), padding: UiRect::all(px(8)), ..default() },
                GlobalZIndex(1000),
                ViewToast,
            ));
        }
        (None, Some((e, _))) => commands.entity(e).despawn(),
        (None, None) => {}
    }
}

pub fn show_panel(
    time: Res<Time<Real>>,
    dev: Res<DevMenu>,
    play: Option<Res<PlayState>>,
    mut boxes: Query<&mut Node, (With<DevPanel>, Without<DevWindow>)>,
    mut windows: Query<&mut Node, (With<DevWindow>, Without<DevPanel>)>,
    mut texts: Query<&mut Text, (With<DevText>, Without<DevLabel>, Without<DevValue>, Without<DevNote>)>,
    mut labels: Query<(&mut Text, &DevLabel), (Without<DevValue>, Without<DevNote>)>,
    mut values: Query<(&mut Text, &mut TextColor, &DevValue), (Without<DevLabel>, Without<DevNote>)>,
    mut notes: Query<&mut Text, (With<DevNote>, Without<DevLabel>, Without<DevValue>)>,
    mut details: Query<&mut Text, (With<DevDetail>, Without<DevNote>, Without<DevLabel>, Without<DevValue>, Without<DevText>)>,
    mut headers: Query<(&mut Text, &DevHeader), (Without<DevDetail>, Without<DevNote>, Without<DevLabel>, Without<DevValue>, Without<DevText>)>,
    mut rows: Query<(&Interaction, &DevRow, &mut BackgroundColor), Without<DevStep>>,
    mut arrows: Query<(&Interaction, &mut BackgroundColor), (With<DevStep>, Without<DevRow>)>,
    // Each row's label and when it was first shown: a changed label scrolls from its start.
    mut shown: Local<std::collections::HashMap<usize, (String, f64)>>,
) {
    let _span = crate::perf::span("show_panel");
    for mut n in &mut windows {
        n.display = if dev.open { Display::Flex } else { Display::None };
    }
    let status = !dev.open && (dev.free.is_some() || dev.speed != NORMAL);
    for mut n in &mut boxes {
        n.display = if status { Display::Flex } else { Display::None };
    }
    let Some(play) = play else { return };
    let p = &play.play;
    if dev.open {
        for (mut t, l) in &mut labels {
            let label = row_text(ENTRIES[l.0], &dev, p).0;
            let now = time.elapsed_secs_f64();
            let since = shown.entry(l.0).or_insert_with(|| (label.clone(), now));
            if since.0 != label {
                *since = (label.clone(), now);
            }
            let s = fit(label, now - since.1);
            if t.0 != s {
                t.0 = s;
            }
        }
        for (mut t, mut c, v) in &mut values {
            let s = row_text(ENTRIES[v.0], &dev, p).1;
            c.0 = match s.as_str() {
                "ON" => ON,
                "OFF" => OFF,
                _ => GOLD,
            };
            if t.0 != s {
                t.0 = s;
            }
        }
        for (i, r, mut bg) in &mut rows {
            bg.0 = if r.0 == dev.cursor {
                ROW_CURSOR
            } else if *i == Interaction::Hovered {
                ROW_HOVER
            } else {
                ROW
            };
        }
        for (i, mut bg) in &mut arrows {
            bg.0 = if *i == Interaction::None { ARROW } else { ARROW_HOVER };
        }
        for mut t in &mut notes {
            if t.0 != dev.note {
                t.0 = dev.note.clone();
            }
        }
        let d = detail(&dev, p);
        for mut t in &mut details {
            if t.0 != d {
                t.0 = d.clone();
            }
        }
        for (mut t, h) in &mut headers {
            let title = SECTIONS[h.0].0;
            let s = if title == "TRAVEL" {
                match area(&dev, p.world.pack) {
                    Some(n) => format!("TRAVEL  (now in {n}, {})", p.world.pack),
                    None => format!("TRAVEL  (now in world pack {})", p.world.pack),
                }
            } else {
                title.to_string()
            };
            if t.0 != s {
                t.0 = s;
            }
        }
    }
    if status {
        let mut s = String::new();
        if dev.speed != NORMAL {
            s.push_str(&format!("GAME SPEED {} (F4 pause/resume, F5 step)\n", speed_label(dev.speed)));
        }
        if let Some(fc) = dev.free {
            s.push_str(&format!(
                "FREE CAMERA (F2 or B ends; T or Y moves Tor here) at ({:.1}, {:.1}, {:.1}): WASD or left stick move, E/Q or RT/LT up/down, J/L I/K, right stick or right-drag look, Shift or A fast",
                fc.pos.x, fc.pos.y, fc.pos.z
            ));
        }
        for mut t in &mut texts {
            if t.0 != s {
                t.0 = s.clone();
            }
        }
    }
}

fn speed_label(k: usize) -> String {
    if k == 0 { "paused".into() } else { format!("{}x", SPEEDS[k]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns() {
        // Down wraps within the left column; the other column keeps the height.
        assert_eq!(column_step(LEFT - 1, 1), 0);
        assert_eq!(column_step(LEFT, -1), ENTRIES.len() - 1);
        assert_eq!(other_column(2), LEFT + 2);
        assert_eq!(other_column(ENTRIES.len() - 1), LEFT - 1);
        assert_eq!(other_column(other_column(3)), 3);
    }

    #[test]
    fn long_labels_scroll() {
        // A short label is kept; a long one holds its start, then scrolls and wraps.
        assert_eq!(super::fit("Full heal".into(), 9.0), "Full heal");
        let long = "Boss fight: The Darkest Faerie (final form)".to_string();
        let a = super::fit(long.clone(), 0.5);
        assert_eq!(a.chars().count(), super::FIT);
        assert!(long.starts_with(&a));
        // After the hold, 1 s scrolls the name 3 characters; "Boss fight: " stays put.
        let b = super::fit(long.clone(), 2.0 + 1.0);
        assert_eq!(b, format!("Boss fight: {}", long.chars().skip(12 + 3).take(super::FIT - 12).collect::<String>()));
    }
}

