//! The debug tools of the developer menu: a tool of this build, not part of the original game, and following no spec.
//! Each is a row of the menu's DEBUG TOOLS section.
//!
//! - Save a bug report (also the End key): private/reports/NNN/ gets a screenshot, the state, the event log and the last
//!   minute of input.
//! - Inspector: a live panel on the hero and the character nearest the view's centre.
//! - Collision view: walking maps, solid boxes and spheres, doors, ladders, triggers and nearby walls as coloured lines.
//! - Wireframe: every triangle's edges over the picture, or the edges alone (which also shows what walls hide).
//! - Event log: flags, items, worlds and pickups as they change.
//! - Hide HUD: for clean pictures.

use std::collections::{BTreeSet, HashMap, VecDeque};

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::schedule::{Core3d, Core3dSystems};
use bevy::mesh::PrimitiveTopology;
use bevy::pbr::wireframe::{wireframe_3d, WireframeColor, WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{LoadOp, Operations, RenderPassDescriptor, StoreOp, WgpuFeatures};
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::view::{ViewDepthTexture, ViewTarget};
use bevy::render::{Extract, ExtractSchedule, RenderApp};

use crate::dev_menu::DevMenu;
use crate::PlayState;

/// The menu's switches.
#[derive(Clone, Copy, Debug, Default)]
pub struct Switches {
    pub inspector: bool,
    pub collision: bool,
    /// The collision view on black: nothing else drawn, so objects cannot hide its lines.
    pub lines_only: bool,
    /// The item gallery is open (item_gallery.rs).
    pub gallery: bool,
    /// A bug report's note is being typed.
    pub note_open: bool,
    /// Index into LAYERS.
    pub collision_layer: usize,
    /// 0 off, 1 over the picture, 2 alone.
    pub wireframe: u8,
    pub log: bool,
    pub hide_hud: bool,
    /// The stand-in aim marker is not drawn (R-0280).
    pub no_aim_marker: bool,
    pub report: bool,
}

impl Switches {
    /// NEOPETS_DEBUG="inspector,collision,wireframe,wireonly,log,hud,noaim": the tools on from the start, for picture runs (diagnostic).
    pub fn from_env() -> Self {
        let v = std::env::var("NEOPETS_DEBUG").unwrap_or_default();
        Self { inspector: v.contains("inspector"), collision: v.contains("collision") || v.contains("lineonly"), lines_only: v.contains("lineonly"), gallery: v.contains("gallery"), note_open: false, collision_layer: if v.contains("layersolid") { 1 } else { 0 }, wireframe: if v.contains("wireonly") { 2 } else if v.contains("wireframe") { 1 } else { 0 }, log: v.contains("log"), hide_hud: v.contains("hud"), no_aim_marker: v.contains("noaim"), report: false }
    }
}

/// The wireframe row's value.
pub fn wireframe_name(mode: u8) -> &'static str {
    match mode {
        0 => "OFF",
        1 => "Over picture",
        _ => "Lines only",
    }
}

/// The wireframe tool: Bevy's own wireframe pass, which runs after the main pass, and a wipe of the picture between the two for
/// "lines only".
pub struct WireframeTool;

impl Plugin for WireframeTool {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig { global: false, default_color: Color::srgb(0.25, 1.0, 0.45), ..default() });
        app.add_systems(Update, apply_wireframe);
        let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
        r.init_resource::<WipeScene>();
        r.add_systems(ExtractSchedule, extract_wipe);
        r.add_systems(Core3d, wipe_scene.after(Core3dSystems::MainPass).before(wireframe_3d));
    }
}

fn apply_wireframe(dev: Res<DevMenu>, mut config: ResMut<WireframeConfig>) {
    let on = dev.debug.wireframe != 0;
    if config.global != on {
        config.global = on;
    }
}

#[derive(Resource, Default)]
struct WipeScene(bool);

fn extract_wipe(dev: Extract<Res<DevMenu>>, mut wipe: ResMut<WipeScene>) {
    wipe.0 = dev.debug.wireframe == 2 || dev.debug.lines_only;
}

/// Lines only: clears the colour and depth the main pass drew, so the wireframe pass draws on black and nothing hides it but
/// other lines. A graphics card that can't draw lines keeps the picture, since the screen would otherwise be blank.
fn wipe_scene(view: ViewQuery<(&ViewTarget, &ViewDepthTexture)>, wipe: Res<WipeScene>, device: Res<RenderDevice>, mut ctx: RenderContext) {
    if !wipe.0 || !device.features().contains(WgpuFeatures::POLYGON_MODE_LINE | WgpuFeatures::IMMEDIATES) {
        return;
    }
    let (target, depth) = view.into_inner();
    let mut colour = target.get_color_attachment();
    colour.ops.load = LoadOp::Clear(LinearRgba::BLACK.into());
    let mut depth = depth.get_attachment(StoreOp::Store);
    depth.depth_ops = Some(Operations { load: LoadOp::Clear(0.0), store: StoreOp::Store });
    let _pass = ctx.begin_tracked_render_pass(RenderPassDescriptor { label: Some("wireframe_wipe"), color_attachments: &[Some(colour)], depth_stencil_attachment: Some(depth), timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
}

/// The kinds of collision line, with their colours and names (the panel's legend).
/// How far from the hero the collision view reaches.
const RANGE: f32 = 50.0;

/// What the collision view shows, by KINDS index.
pub const LAYERS: [(&str, &[u8]); 5] = [
    ("Everything", &[0, 1, 2, 3, 4, 5, 6, 7, 8]),
    ("Solids and walls", &[1, 2, 6, 7, 8]),
    ("Walking maps", &[0, 7]),
    ("Triggers", &[5, 7]),
    ("Doors and ladders", &[3, 4, 7]),
];

pub const KINDS: [(&str, [f32; 3]); 9] = [
    ("walking maps", [0.1, 0.9, 0.9]),
    ("solid boxes", [1.0, 0.15, 0.15]),
    ("solid spheres", [1.0, 0.55, 0.1]),
    ("doors", [1.0, 0.2, 1.0]),
    ("ladders and climb panels", [0.2, 1.0, 0.2]),
    ("triggers", [1.0, 1.0, 0.2]),
    ("walls near you", [1.0, 1.0, 1.0]),
    ("you", [0.3, 0.5, 1.0]),
    ("solid cylinders", [1.0, 0.45, 0.7]),
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
    note: Note,
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn save_report(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<bevy::time::Real>>,
    mut fired: Local<bool>,
    state: Option<Res<PlayState>>,
    mut d: ResMut<DebugState>,
    mut dev: ResMut<DevMenu>,
    settings: Res<crate::display::DisplaySettings>,
    meshes: Query<(&InheritedVisibility, Has<bevy::light::NotShadowCaster>, Has<crate::shadow_view::FaintCaster>, Has<crate::shadow_view::CasterOff>), With<Mesh3d>>,
    suns: Query<(&InheritedVisibility, &DirectionalLight, &bevy::light::Cascades), With<crate::shadow_view::Sun>>,
) {
    // NEOPETS_DEBUG holding "report": one report 9 s in, for checking this tool without a player.
    let elapsed = time.elapsed_secs();
    let auto = !*fired && elapsed > 9.0 && std::env::var("NEOPETS_DEBUG").is_ok_and(|v| v.contains("report"));
    *fired |= auto;
    if dev.debug.note_open || !(auto || keys.just_pressed(KeyCode::End) || std::mem::take(&mut dev.debug.report)) {
        return;
    }
    let Some(state) = state else { return };
    let p = &state.play;
    let root = std::path::Path::new("private/reports");
    // One past the highest numbered folder, so a removed folder never makes a new report land on
    // an old one.
    let n = std::fs::read_dir(root).map_or(0, |r| r.flatten().filter_map(|e| e.file_name().to_str()?.parse::<usize>().ok()).max().unwrap_or(0)) + 1;
    let dir = root.join(format!("{n:03}"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("Bug report: could not make {}: {e}", dir.display());
        return;
    }
    let h = p.player.loco.position();
    let (eye, forward, _, _) = p.view();
    let mut text = format!(
        "tick {}\nworld {} other {:?}\nhero {:.2},{:.2},{:.2} state {} phase {} hp {}/{} mp {}/{} hero index {}\neye {eye:?} forward {forward:?}\nflags {:?}\nneopoints {}\nitems {:?}\n\n",
        p.ticks,
        p.world.pack,
        p.other.as_ref().map(|o| o.pack),
        h[0],
        h[1],
        h[2],
        p.player.loco.state,
        p.player.loco.phase,
        p.player.hero_hp,
        p.player.hero_max,
        p.player.hero_mp,
        p.player.hero_mp_max,
        p.hero_index,
        p.state.flags,
        p.state.neopoints,
        p.state.items
    );
    for part in [p.debug_inspect(), p.scene_status(), p.status(), p.objects_near(), p.placed_summary()] {
        text.push_str(&part);
        text.push_str("\n\n");
    }
    text.push_str(&shadow_summary(p, &settings, &meshes, &suns));
    text.push_str(&p.debug_nearby(100.0));
    let log: Vec<&str> = d.log.iter().map(String::as_str).collect();
    let (start, inputs) = p.recorder.export();
    let replay = format!("Replay in a trace (no window):\n  python scripts/replay.py {n}\nor by hand:\n  NEOPETS_REPLAY=private/reports/{n:03} target/release/neopets-app.exe --play-trace {} --pack {} --seed 1\n", p.recorder.len() + 30, p.recorder.start_pack(p.world.pack));
    let writes = [("state.txt", text), ("events.txt", log.join("\n")), ("start.txt", start), ("input.csv", inputs), ("replay.txt", replay), ("history.csv", p.recorder.history_csv())];
    for (name, body) in writes {
        if let Err(e) = std::fs::write(dir.join(name), body) {
            error!("Bug report: could not write {name}: {e}");
        }
    }
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};
    commands.spawn(Screenshot::primary_window()).observe(save_to_disk(dir.join("screen.png")));
    info!("Bug report {n} saved to {}", dir.display());
    // The note's prompt comes up after the picture has been taken. A trial run with no player has no prompt, unless
    // NEOPETS_TEST_NOTE gives it the words.
    if auto && std::env::var_os("NEOPETS_TEST_NOTE").is_none() {
        dev.toast = Some((format!("Report {n} saved"), 240));
    } else {
        d.note = Note { number: n, dir, text: String::new(), frames: 0, speed: dev.note_begin() };
        dev.debug.note_open = true;
    }
}

/// The note of the report just saved: what was typed goes into its note.txt.
#[derive(Default)]
pub struct Note {
    number: usize,
    dir: std::path::PathBuf,
    text: String,
    frames: u32,
    speed: usize,
}

#[derive(Component)]
pub struct NotePanel;

/// Typing a bug report's note: Enter saves it, Esc (or the pad's B) leaves the report with none.
#[allow(clippy::too_many_arguments)]
pub fn note_input(
    mut commands: Commands,
    mut d: ResMut<DebugState>,
    mut dev: ResMut<DevMenu>,
    mut typed: MessageReader<bevy::input::keyboard::KeyboardInput>,
    gamepads: Query<&Gamepad>,
    mut panels: Query<(Entity, &mut Text), With<NotePanel>>,
) {
    use bevy::input::keyboard::Key;
    if !dev.debug.note_open {
        typed.clear();
        return;
    }
    let mut done = None;
    for k in typed.read().filter(|k| k.state.is_pressed()) {
        match &k.logical_key {
            Key::Enter => done = Some(true),
            Key::Escape => done = Some(false),
            Key::Backspace => {
                d.note.text.pop();
            }
            Key::Space => d.note.text.push(' '),
            Key::Character(c) if d.note.text.len() < 600 => d.note.text.push_str(c),
            _ => {}
        }
    }
    if gamepads.iter().any(|g| g.just_pressed(GamepadButton::East)) {
        done = Some(false);
    }
    d.note.frames += 1;
    // NEOPETS_TEST_NOTE: those words, entered as if typed (diagnostic).
    if let Ok(words) = std::env::var("NEOPETS_TEST_NOTE") {
        d.note.text = words;
        if d.note.frames == 900 {
            done = Some(true);
        }
    }
    if let Some(save) = done {
        let n = d.note.number;
        let saved = save && !d.note.text.trim().is_empty();
        if saved {
            if let Err(e) = std::fs::write(d.note.dir.join("note.txt"), d.note.text.trim()) {
                error!("Bug report: could not write its note: {e}");
            }
        }
        dev.debug.note_open = false;
        dev.note_end(d.note.speed);
        dev.toast = Some((format!("Report {n} saved{}", if saved { " with its note" } else { "" }), 240));
        for (e, _) in &panels {
            commands.entity(e).despawn();
        }
        return;
    }
    if d.note.frames < 4 {
        return;
    }
    let body = format!("Bug report {} saved. Type a note if you like:\n\n{}_\n\nEnter saves it, Esc skips", d.note.number, d.note.text);
    match panels.iter_mut().next() {
        Some((_, mut t)) => {
            if t.0 != body {
                t.0 = body;
            }
        }
        None => {
            commands.spawn((
                Text::new(body),
                TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                Node { position_type: PositionType::Absolute, left: percent(20), right: percent(20), top: percent(30), padding: UiRect::all(px(12)), ..default() },
                GlobalZIndex(5000),
                NotePanel,
            ));
        }
    }
}

/// The panel: the inspector, the collision legend and the recent events.
#[allow(clippy::too_many_arguments)]
pub fn show_panel(mut commands: Commands, state: Option<Res<PlayState>>, d: Res<DebugState>, dev: Res<DevMenu>, mut panels: Query<(Entity, &mut Text), With<DebugPanel>>) {
    let s = dev.debug;
    let mut out = String::new();
    // The menu covers this part of the screen, so the panel waits until it closes.
    if let (true, false, Some(state)) = (s.inspector, dev.open, state.as_ref()) {
        out.push_str(&state.play.debug_inspect());
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
    mut was: Local<(bool, usize)>,
) {
    // A change of mode rebuilds at once; lines alone are redrawn more often, as the hero's marker is all there is to follow.
    let mode = (dev.debug.lines_only, dev.debug.collision_layer);
    if *was != mode {
        *was = mode;
        *frames = 0;
    }
    if !dev.debug.collision {
        for e in &old {
            commands.entity(e).despawn();
        }
        *frames = 0;
        return;
    }
    *frames += 1;
    if *frames % if dev.debug.lines_only { 10 } else { 60 } != 1 {
        return;
    }
    let Some(state) = state else { return };
    for e in &old {
        commands.entity(e).despawn();
    }
    let mut lists: Vec<Vec<[f32; 3]>> = vec![Vec::new(); KINDS.len()];
    let kinds = LAYERS[dev.debug.collision_layer.min(LAYERS.len() - 1)].1;
    for (k, a, b) in state.play.debug_lines(RANGE, kinds).into_iter().chain(state.play.debug_hero_cross()) {
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
        let mut e = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material), Transform::IDENTITY, bevy::light::NotShadowCaster, bevy::light::NotShadowReceiver, WireframeColor { color: Color::srgb(c[0], c[1], c[2]) }, DebugLines));
        // The wipe clears the main pass, so lines alone are drawn by the wireframe pass.
        if dev.debug.lines_only {
            e.insert(bevy::pbr::wireframe::Wireframe);
        }
    }
}

/// A report's replay in a trace (NEOPETS_REPLAY): its start state, applied once, and its input rows in order.
pub struct Replay {
    start: Vec<(String, String)>,
    rows: VecDeque<neopets_sim::hero::HeroInput>,
    begun: bool,
}

/// What decides the world's shadows at this moment: the settings, the sun, the meshes that cast, and the plant patches the
/// zone gate has off near the hero (a hidden plant casts nothing).
#[allow(clippy::type_complexity)]
fn shadow_summary(
    p: &crate::play::Play,
    settings: &crate::display::DisplaySettings,
    meshes: &Query<(&InheritedVisibility, Has<bevy::light::NotShadowCaster>, Has<crate::shadow_view::FaintCaster>, Has<crate::shadow_view::CasterOff>), With<Mesh3d>>,
    suns: &Query<(&InheritedVisibility, &DirectionalLight, &bevy::light::Cascades), With<crate::shadow_view::Sun>>,
) -> String {
    let mut out = format!("SHADOWS: world {} characters {} quality {} distance {}; sun direction {:?}
", settings.world_shadows, settings.char_shadows, settings.shadow_quality, settings.shadow_distance, p.sun_direction());
    for (shown, light, cascades) in suns {
        let counts: Vec<usize> = cascades.cascades.values().map(Vec::len).collect();
        out.push_str(&format!("sun shown {} shadow maps {} cascades {counts:?}
", shown.get(), light.shadow_maps_enabled));
    }
    let (mut shown, mut casting, mut faint, mut off) = (0, 0, 0, 0);
    for (v, no_cast, is_faint, is_off) in meshes {
        if v.get() {
            shown += 1;
            casting += usize::from(!no_cast);
            faint += usize::from(is_faint);
            off += usize::from(is_off);
        }
    }
    out.push_str(&format!("meshes shown {shown}, casting {casting}, marked faint {faint}, marked off {off}
"));
    let h = p.player.loco.position();
    for (pack, patches) in &p.foliage {
        let on = patches.iter().filter(|q| q.on).count();
        let mut hidden: Vec<(f32, usize, u16)> = patches
            .iter()
            .enumerate()
            .filter(|(_, q)| !q.on)
            .map(|(i, q)| ((q.centre[0] - h[0]).hypot(q.centre[2] - h[2]), i, q.model))
            .collect();
        hidden.sort_by(|a, b| a.0.total_cmp(&b.0));
        hidden.truncate(8);
        let near: Vec<String> = hidden.iter().map(|(d, i, m)| format!("{i} (model {m}) {d:.0} away")).collect();
        out.push_str(&format!("plants of {pack}: {on} of {} patches on; nearest off: {}
", patches.len(), near.join(", ")));
    }
    out
}

/// NEOPETS_REPLAY in a window: a report's input plays there too, so its picture can be seen again.
pub fn window_replay(p: &mut crate::play::Play, input: &mut neopets_sim::hero::HeroInput) {
    static REPLAY: std::sync::OnceLock<std::sync::Mutex<Option<Replay>>> = std::sync::OnceLock::new();
    let replay = REPLAY.get_or_init(|| std::sync::Mutex::new(Replay::from_env()));
    if let Some(r) = replay.lock().unwrap().as_mut() {
        r.apply(p, input);
    }
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
                    "hp" => p.player.hero_hp = v.parse().unwrap_or(p.player.hero_hp),
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
