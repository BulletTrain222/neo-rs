//! The debug tools of the developer menu: a tool of this build, not part of the original game, and following no spec.
//! Each is a row of the menu's DEBUG TOOLS section.
//!
//! - Save a bug report (also the End key): private/reports/NNN/ gets a screenshot, the state, the event log and the last
//!   minute of input.
//! - Inspector: a live panel on the hero and the character nearest the view's centre.
//! - Collision view: walking maps, solid boxes and spheres, doors, ladders, triggers and nearby walls as coloured lines.
//! - Event log: flags, items, worlds and pickups as they change.
//! - Hide HUD: for clean pictures.

use std::collections::{BTreeSet, HashMap, VecDeque};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::dev_menu::DevMenu;
use crate::PlayState;

/// The menu's switches.
#[derive(Clone, Copy, Debug, Default)]
pub struct Switches {
    pub inspector: bool,
    pub collision: bool,
    pub log: bool,
    pub hide_hud: bool,
    pub report: bool,
}

impl Switches {
    /// NEOPETS_DEBUG="inspector,collision,log,hud": the tools on from the start, for picture runs (diagnostic).
    pub fn from_env() -> Self {
        let v = std::env::var("NEOPETS_DEBUG").unwrap_or_default();
        Self { inspector: v.contains("inspector"), collision: v.contains("collision"), log: v.contains("log"), hide_hud: v.contains("hud"), report: false }
    }
}

/// The kinds of collision line, with their colours and names (the panel's legend).
pub const KINDS: [(&str, [f32; 3]); 7] = [
    ("walking maps", [0.1, 0.9, 0.9]),
    ("solid boxes", [1.0, 0.15, 0.15]),
    ("solid spheres", [1.0, 0.55, 0.1]),
    ("doors", [1.0, 0.2, 1.0]),
    ("ladders and climb panels", [0.2, 1.0, 0.2]),
    ("triggers", [1.0, 1.0, 0.2]),
    ("walls near you", [1.0, 1.0, 1.0]),
];

/// The event log's lines kept for a report.
const LOG_LINES: usize = 2000;

#[derive(Resource, Default)]
pub struct DebugState {
    log: VecDeque<String>,
    flags: BTreeSet<u16>,
    items: HashMap<u16, u32>,
    pack: u32,
    notes: [String; 4],
    started: bool,
}

#[derive(Component)]
pub struct DebugPanel;

#[derive(Component)]
pub struct DebugLines;

/// The log: what changed in the game state this frame, each line with its tick.
pub fn track_events(state: Option<Res<PlayState>>, mut d: ResMut<DebugState>) {
    let Some(state) = state else { return };
    let p = &state.play;
    let t = p.ticks;
    let mut lines = Vec::new();
    if d.started {
        for f in p.state.flags.difference(&d.flags) {
            lines.push(format!("{t}: flag {f} on"));
        }
        for f in d.flags.difference(&p.state.flags) {
            lines.push(format!("{t}: flag {f} off"));
        }
        for (id, n) in &p.state.items {
            let old = d.items.get(id).copied().unwrap_or(0);
            if old != *n {
                lines.push(format!("{t}: item {id} ({}) {old} -> {n}", p.item_label(*id)));
            }
        }
        for id in d.items.keys().filter(|id| !p.state.items.contains_key(id)) {
            lines.push(format!("{t}: item {id} ({}) gone", p.item_label(*id)));
        }
        if d.pack != p.world.pack {
            lines.push(format!("{t}: world {} -> {}", d.pack, p.world.pack));
        }
    }
    let notes = [&p.last_pickup, &p.last_door, &p.last_swap, &p.last_boss];
    for (k, n) in notes.into_iter().enumerate() {
        if d.notes[k] != *n {
            d.notes[k].clone_from(n);
            if !n.is_empty() {
                lines.push(format!("{t}: {n}"));
            }
        }
    }
    if p.state.flags != d.flags {
        d.flags.clone_from(&p.state.flags);
    }
    if p.state.items != d.items {
        d.items.clone_from(&p.state.items);
    }
    d.pack = p.world.pack;
    d.started = true;
    for l in lines {
        d.log.push_back(l);
        if d.log.len() > LOG_LINES {
            d.log.pop_front();
        }
    }
}

/// The End key, and the menu's row: a report in private/reports/NNN/.
pub fn save_report(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, time: Res<Time<bevy::time::Real>>, mut fired: Local<bool>, state: Option<Res<PlayState>>, d: Res<DebugState>, mut dev: ResMut<DevMenu>) {
    // NEOPETS_DEBUG holding "report": one report 9 s in, for checking this tool without a player.
    let elapsed = time.elapsed_secs();
    let auto = !*fired && elapsed > 9.0 && std::env::var("NEOPETS_DEBUG").is_ok_and(|v| v.contains("report"));
    *fired |= auto;
    if !(auto || keys.just_pressed(KeyCode::End) || std::mem::take(&mut dev.debug.report)) {
        return;
    }
    let Some(state) = state else { return };
    let p = &state.play;
    let root = std::path::Path::new("private/reports");
    let n = std::fs::read_dir(root).map_or(0, |r| r.count()) + 1;
    let dir = root.join(format!("{n:03}"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("Bug report: could not make {}: {e}", dir.display());
        return;
    }
    let h = p.loco.position();
    let (eye, forward, _, _) = p.view();
    let mut text = format!(
        "tick {}\nworld {} other {:?}\nhero {:.2},{:.2},{:.2} state {} phase {} hp {}/{} mp {}/{} hero index {}\neye {eye:?} forward {forward:?}\nflags {:?}\nneopoints {}\nitems {:?}\n\n",
        p.ticks,
        p.world.pack,
        p.other.as_ref().map(|o| o.pack),
        h[0],
        h[1],
        h[2],
        p.loco.state,
        p.loco.phase,
        p.hero_hp,
        p.hero_max,
        p.hero_mp,
        p.hero_mp_max,
        p.hero_index,
        p.state.flags,
        p.state.neopoints,
        p.state.items
    );
    for part in [p.debug_inspect(), p.scene_status(), p.status(), p.objects_near(), p.placed_summary()] {
        text.push_str(&part);
        text.push_str("\n\n");
    }
    let log: Vec<&str> = d.log.iter().map(String::as_str).collect();
    let (start, inputs) = p.recorder.export();
    let replay = format!("Replay in a trace (no window):\n  python scripts/replay.py {n}\nor by hand:\n  NEOPETS_REPLAY=private/reports/{n:03} target/release/neopets-app.exe --play-trace {} --pack {} --seed 1\n", p.recorder.len() + 30, p.recorder.start_pack(p.world.pack));
    let writes = [("state.txt", text), ("events.txt", log.join("\n")), ("start.txt", start), ("input.csv", inputs), ("replay.txt", replay)];
    for (name, body) in writes {
        if let Err(e) = std::fs::write(dir.join(name), body) {
            error!("Bug report: could not write {name}: {e}");
        }
    }
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};
    commands.spawn(Screenshot::primary_window()).observe(save_to_disk(dir.join("screen.png")));
    info!("Bug report {n} saved to {}", dir.display());
    dev.toast = Some((format!("Bug report {n} saved: tell Claude \"report {n}\""), 240));
}

/// The panel: the inspector, the collision legend and the recent events.
pub fn show_panel(mut commands: Commands, state: Option<Res<PlayState>>, d: Res<DebugState>, dev: Res<DevMenu>, mut panels: Query<(Entity, &mut Text), With<DebugPanel>>) {
    let s = dev.debug;
    let mut out = String::new();
    // The menu covers this part of the screen, so the panel waits until it closes.
    if let (true, false, Some(state)) = (s.inspector, dev.open, state.as_ref()) {
        out.push_str(&state.play.debug_inspect());
    }
    if s.collision && !dev.open {
        out.push_str("\nCOLLISION VIEW:");
        for (name, _) in KINDS {
            out.push_str(&format!(" {name},"));
        }
        out.pop();
        out.push_str(" (cyan, red, orange, pink, green, yellow, white)\n");
    }
    if s.log && !dev.open {
        out.push_str("\nEVENTS:\n");
        let n = d.log.len();
        for l in d.log.iter().skip(n.saturating_sub(10)) {
            out.push_str(l);
            out.push('\n');
        }
    }
    match (out.is_empty(), panels.iter_mut().next()) {
        (true, Some((e, _))) => commands.entity(e).despawn(),
        (false, Some((_, mut t))) => {
            if t.0 != out {
                t.0 = out;
            }
        }
        (false, None) => {
            commands.spawn((
                Text::new(out),
                TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
                TextColor(Color::srgb(0.9, 1.0, 0.8)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
                Node { position_type: PositionType::Absolute, right: px(8), top: px(190), width: px(470), padding: UiRect::all(px(6)), ..default() },
                GlobalZIndex(4900),
                DebugPanel,
            ));
        }
        (true, None) => {}
    }
}

/// The HUD off for clean pictures.
pub fn hide_hud(dev: Res<DevMenu>, mut canvas: Query<&mut Visibility, With<crate::hud_view::HudCanvas>>) {
    for mut v in &mut canvas {
        let want = if dev.debug.hide_hud { Visibility::Hidden } else { Visibility::Inherited };
        if *v != want {
            *v = want;
        }
    }
}

/// The collision view's line meshes, rebuilt about once a second while it is on.
pub fn show_lines(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    dev: Res<DevMenu>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<DebugLines>>,
    mut frames: Local<u32>,
) {
    if !dev.debug.collision {
        for e in &old {
            commands.entity(e).despawn();
        }
        *frames = 0;
        return;
    }
    *frames += 1;
    if *frames % 60 != 1 {
        return;
    }
    let Some(state) = state else { return };
    for e in &old {
        commands.entity(e).despawn();
    }
    let mut lists: Vec<Vec<[f32; 3]>> = vec![Vec::new(); KINDS.len()];
    for (k, a, b) in state.play.debug_lines() {
        if let Some(l) = lists.get_mut(k as usize) {
            l.push(a);
            l.push(b);
        }
    }
    for (k, positions) in lists.into_iter().enumerate() {
        if positions.is_empty() {
            continue;
        }
        let c = KINDS[k].1;
        let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        let material = materials.add(StandardMaterial { base_color: Color::srgb(c[0], c[1], c[2]), unlit: true, cull_mode: None, ..default() });
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material), Transform::IDENTITY, bevy::light::NotShadowCaster, bevy::light::NotShadowReceiver, DebugLines));
    }
}

/// A report's replay in a trace (NEOPETS_REPLAY): its start state, applied once, and its input rows in order.
pub struct Replay {
    start: Vec<(String, String)>,
    rows: VecDeque<neopets_sim::hero::HeroInput>,
    begun: bool,
}

impl Replay {
    pub fn from_env() -> Option<Self> {
        let dir = std::env::var("NEOPETS_REPLAY").ok()?;
        let dir = std::path::Path::new(&dir);
        let start = std::fs::read_to_string(dir.join("start.txt")).ok()?;
        let start = start.lines().filter_map(|l| l.split_once(' ')).map(|(a, b)| (a.to_string(), b.to_string())).collect();
        let rows = std::fs::read_to_string(dir.join("input.csv")).ok()?;
        let rows = rows
            .lines()
            .filter_map(|l| {
                let v: Vec<f32> = l.split(',').filter_map(|x| x.parse().ok()).collect();
                let b = |k: usize| v.get(k).is_some_and(|x| *x != 0.0);
                (v.len() >= 14).then(|| neopets_sim::hero::HeroInput {
                    side: v[1],
                    forward: v[2],
                    run: b(3),
                    jump: b(4),
                    attack_held: b(5),
                    attack: b(6),
                    square: b(7),
                    square_held: b(8),
                    guard: b(9),
                    r2_held: b(10),
                    circle_held: b(11),
                    cross_held: b(12),
                    l2: b(13),
                })
            })
            .collect();
        Some(Self { start, rows, begun: false })
    }

    pub fn apply(&mut self, p: &mut crate::play::Play, input: &mut neopets_sim::hero::HeroInput) {
        if !self.begun {
            self.begun = true;
            for (k, v) in &self.start {
                match k.as_str() {
                    "position" => {
                        let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                        if let [x, y, z] = c[..] {
                            p.place_at(x, z, y + 3.0);
                        }
                    }
                    "hp" => p.hero_hp = v.parse().unwrap_or(p.hero_hp),
                    "neopoints" => p.state.neopoints = v.parse().unwrap_or(0),
                    "flags" => {
                        for f in v.split(',').filter_map(|x| x.parse::<u16>().ok()) {
                            p.state.set(f, true);
                        }
                    }
                    "items" => {
                        for (id, n) in v.split(',').filter_map(|x| x.split_once(':')) {
                            if let (Ok(id), Ok(n)) = (id.parse::<u16>(), n.parse::<u32>()) {
                                p.state.items.insert(id, n);
                            }
                        }
                    }
                    _ => {}
                }
            }
            println!("REPLAY started: {} input rows", self.rows.len());
        }
        match self.rows.pop_front() {
            Some(i) => *input = i,
            None => *input = Default::default(),
        }
    }
}
