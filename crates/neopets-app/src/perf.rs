//! The performance overlay (a test tool of this build; no spec): F3 shows a see-through panel
//! with the frame rate and a frame-time graph, the hero's place (coordinates, facing, ground,
//! speed), the world (pack number, location name, buffer, the second world, play time) and the
//! scene (characters, enemies, shots, items, loads, entities, the camera).

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{PlayState, SimDriver, Streamer};

#[derive(Resource, Default)]
pub struct PerfOverlay {
    pub on: bool,
}

#[derive(Component)]
pub struct PerfPanel;

/// A section's body text (index into SECTIONS).
#[derive(Component)]
pub struct PerfBody(usize);

/// One bar of the frame-time graph (0 the oldest).
#[derive(Component)]
pub struct PerfBar(usize);

/// The last two seconds of frames: (frame seconds, sim ticks run).
#[derive(Default)]
pub struct Samples(VecDeque<(f32, u32)>);

/// The section headings and their colours.
const SECTIONS: [(&str, Color); 4] = [
    ("PERFORMANCE", Color::srgb(0.55, 0.95, 0.65)),
    ("HERO", Color::srgb(0.98, 0.80, 0.35)),
    ("WORLD", Color::srgb(0.55, 0.80, 1.0)),
    ("SCENE", Color::srgb(0.90, 0.65, 0.95)),
];

/// The graph: one bar per recent frame, BAR_MS milliseconds to the full height.
const BARS: usize = 60;
const GRAPH_H: f32 = 36.0;
const BAR_MS: f32 = 50.0;

const TEXT: Color = Color::srgb(0.90, 0.92, 0.95);
const DIM: Color = Color::srgb(0.60, 0.64, 0.70);

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(12),
                width: px(330),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(10)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.55)),
            BorderColor::all(Color::srgba(0.75, 0.85, 1.0, 0.25)),
            GlobalZIndex(5001),
            crate::ui_fit::FitToWindow::Right,
            PerfPanel,
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("F3 OVERLAY"), font(12.0), TextColor(DIM)));
            for (i, (title, colour)) in SECTIONS.iter().enumerate() {
                ui.spawn((Text::new(*title), font(13.0), TextColor(*colour), Node { margin: UiRect::top(px(if i == 0 { 2.0 } else { 7.0 })), ..default() }));
                ui.spawn((Text::default(), font(13.0), TextColor(TEXT), PerfBody(i)));
                if i == 0 {
                    // The frame-time graph, oldest frame on the left.
                    ui.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::FlexEnd, column_gap: px(1), height: px(GRAPH_H), margin: UiRect::top(px(4)), ..default() }).with_children(|g| {
                        for k in 0..BARS {
                            g.spawn((Node { width: px(4), height: px(0), ..default() }, BackgroundColor(Color::srgba(0.4, 0.9, 0.5, 0.85)), PerfBar(k)));
                        }
                    });
                }
            }
        });
}

/// The colour of a frame by its time: green to 60 fps, yellow to 30, red slower.
fn bar_colour(ms: f32) -> Color {
    if ms <= 17.5 {
        Color::srgba(0.40, 0.90, 0.50, 0.85)
    } else if ms <= 34.0 {
        Color::srgba(0.95, 0.80, 0.30, 0.9)
    } else {
        Color::srgba(0.95, 0.40, 0.35, 0.95)
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    mut last_pos: Local<Option<Vec3>>,
    mut panels: Query<&mut Node, (With<PerfPanel>, Without<PerfBar>)>,
    mut bodies: Query<(&mut Text, &PerfBody)>,
    mut bars: Query<(&mut Node, &mut BackgroundColor, &PerfBar), Without<PerfPanel>>,
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
    let hero = play.as_deref().map(|p| Vec3::from(p.play.player.loco.position()));
    let speed = match (hero, *last_pos) {
        (Some(h), Some(l)) if dt > 0.0 => Vec2::new(h.x - l.x, h.z - l.z).length() / dt,
        _ => 0.0,
    };
    *last_pos = hero;
    for mut n in &mut panels {
        let d = if overlay.on { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
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
    let window = windows.iter().next().map_or(String::new(), |w| format!("\nWindow {} x {}", w.physical_width(), w.physical_height()));
    let mut text = [
        format!(
            "FPS {fps:.0}   1% low {low:.0}\nFrame {:.2} ms avg, {worst:.2} ms worst\nSim {tps:.0} ticks/s, {} this frame{window}",
            1000.0 * span / s.len() as f32,
            driver.last_ticks
        ),
        String::new(),
        String::new(),
        String::new(),
    ];
    if let Some(p) = play.as_deref() {
        let p = &p.play;
        let pos = p.player.loco.position();
        let f = p.player.loco.facing;
        let yaw = f[0].atan2(f[2]).to_degrees();
        let ground = p.world.ground_at(pos[0], pos[2], pos[1] + 2.5).map_or("none".to_string(), |g| format!("{g:.2} (gap {:.2})", pos[1] - g));
        text[1] = format!(
            "X {:.2}   Y {:.2}   Z {:.2}\nFacing {yaw:.0} deg   Speed {speed:.1} u/s\nGround {ground}\n{}  state {}  HP {}/{}  MP {}/{}",
            pos[0],
            pos[1],
            pos[2],
            if p.hero_index == 0 { "Tor" } else { "Roberta" },
            p.player.loco.state,
            p.player.hero_hp,
            p.player.hero_max,
            p.player.hero_mp,
            p.player.hero_mp_max
        );
        let secs = p.ticks / 60;
        text[2] = format!(
            "World {}  {}\n{:?}, second world {}\nPlay tick {}  ({}:{:02}:{:02})",
            p.world.pack,
            String::from_utf8_lossy(&p.location_name()),
            p.world.buffer,
            p.other.as_ref().map_or("none".to_string(), |o| o.pack.to_string()),
            p.ticks,
            secs / 3600,
            secs / 60 % 60,
            secs % 60
        );
        let pack = p.world.pack;
        let here: Vec<_> = p.characters.iter().filter(|c| c.pack == pack).collect();
        let enemies = here.iter().filter(|c| c.enemy.as_ref().is_some_and(|e| !e.is_dead())).count();
        let agro = here.iter().filter(|c| c.enemy.as_ref().is_some_and(|e| !e.is_dead() && e.state == 19)).count();
        let items = p.pools.get(&pack).map_or(0, |pool| pool.slots.iter().flatten().count());
        let eye = p.player.camera.eye;
        text[3] = format!(
            "Characters {} here, {} in all\nEnemies {enemies} alive, {agro} fighting\nShots {} enemy, {} hero   Items {items}\nRegion loads {}   Entities {}\nCamera {:.1}, {:.1}, {:.1} ({:.1} away)",
            here.len(),
            p.characters.len(),
            p.shots.live(),
            p.player.hero_shots.len(),
            streamer.as_deref().map_or(0, |s| s.regions_out),
            entities.iter().count(),
            eye[0],
            eye[1],
            eye[2],
            Vec3::from(eye).distance(Vec3::from(p.player.camera.focus))
        );
    } else {
        text[3] = format!("Entities {}", entities.iter().count());
    }
    for (mut t, b) in &mut bodies {
        if t.0 != text[b.0] {
            t.0 = text[b.0].clone();
        }
    }
    // The graph: the last BARS frames, right-aligned (the newest on the right).
    let recent: Vec<f32> = s.iter().rev().take(BARS).map(|x| x.0 * 1000.0).collect();
    for (mut n, mut c, bar) in &mut bars {
        let ms = recent.get(BARS - 1 - bar.0).copied().unwrap_or(0.0);
        n.height = px((ms / BAR_MS).min(1.0) * GRAPH_H);
        c.0 = bar_colour(ms);
    }
}

/// NEOPETS_PERF (diagnostic): every 120 frames, the mean frame time, the main world's CPU time
/// per frame (First to Last), the slowest frame, and the entity, mesh and visible-mesh counts.
#[derive(Resource, Default)]
pub struct PerfLog {
    start: Option<std::time::Instant>,
    frames: u32,
    frame: f32,
    main: f32,
    worst: f32,
    /// Named spans timed inside systems (`span`), summed over the window.
    pub spans: std::collections::BTreeMap<&'static str, f32>,
}

pub fn perf_enabled() -> bool {
    std::env::var_os("NEOPETS_PERF").is_some()
}

pub fn perf_first(mut log: ResMut<PerfLog>) {
    log.start = Some(std::time::Instant::now());
}

/// NEOPETS_PERF: every 240 frames, the UI texts changed per frame and the first words of the
/// ones that changed most (a changed text is laid out and measured again).
#[allow(clippy::type_complexity)]
pub fn perf_text(changed: Query<(Entity, &Text), Changed<Text>>, nodes: Query<(Entity, Option<&Name>, Has<ImageNode>, Option<&ChildOf>), Changed<Node>>, texts: Query<&Text>, children: Query<&Children>, all_nodes: Query<(Entity, &Node, Option<&InheritedVisibility>)>, parents: Query<&ChildOf>, names: Query<&Name>, mut tally: Local<(u32, usize, usize, std::collections::HashMap<Entity, (usize, String)>)>) {
    tally.0 += 1;
    for (e, name, image, parent) in &nodes {
        tally.2 += 1;
        // A changed node, named by its name, its first child's text, or its parent's.
        let child_text = children.get(e).ok().and_then(|c| c.iter().find_map(|k| texts.get(k).ok())).map(|t| t.0.chars().take(30).collect::<String>());
        let label = name.map(|n| n.to_string()).or(child_text).unwrap_or_else(|| format!("{}{e:?} parent {:?}", if image { "image " } else { "" }, parent.map(|p| p.parent())));
        let entry = tally.3.entry(e).or_insert((0, String::new()));
        entry.0 += 1;
        entry.1 = format!("node {}", label.replace('\n', " "));
    }
    for (e, t) in &changed {
        tally.1 += 1;
        let entry = tally.3.entry(e).or_insert((0, String::new()));
        entry.0 += 1;
        entry.1 = t.0.chars().take(40).collect::<String>().replace('\n', " ");
    }
    if tally.0 % 240 != 0 {
        return;
    }
    // The UI nodes by root (its name or first text), and how many are displayed.
    let mut roots: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for (e, node, shown) in &all_nodes {
        let mut r = e;
        while let Ok(p) = parents.get(r) {
            r = p.parent();
        }
        let label = names.get(r).map(|n| n.to_string()).ok().or_else(|| children.get(r).ok().and_then(|c| c.iter().find_map(|k| texts.get(k).ok())).map(|t| t.0.chars().take(24).collect())).unwrap_or_else(|| format!("{r:?}"));
        let entry = roots.entry(label.replace('\n', " ")).or_default();
        entry.0 += 1;
        entry.1 += usize::from(node.display != Display::None && shown.is_some_and(|v| v.get()));
    }
    info!("perf: {} UI nodes; by root (nodes, shown): {:?}", all_nodes.iter().count(), roots);
    let mut top: Vec<(usize, String)> = tally.3.values().cloned().collect();
    top.sort_by(|a, b| b.0.cmp(&a.0));
    let list: Vec<String> = top.iter().take(6).map(|(n, s)| format!("{n}x {s:?}")).collect();
    info!("perf: texts changed per frame {:.2}, nodes changed per frame {:.1}; most: {}", tally.1 as f32 / 240.0, tally.2 as f32 / 240.0, list.join("; "));
    *tally = Default::default();
}

pub fn perf_last(
    mut log: ResMut<PerfLog>,
    time: Res<Time>,
    entities: Query<()>,
    meshes: Query<&ViewVisibility, With<Mesh3d>>,
    mats: Query<&MeshMaterial3d<StandardMaterial>>,
    drawn: Query<(&ViewVisibility, &Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    images: Res<Assets<Image>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    let main = log.start.map_or(0.0, |s| s.elapsed().as_secs_f32() * 1000.0);
    let dt = time.delta_secs() * 1000.0;
    log.frames += 1;
    log.frame += dt;
    log.main += main;
    log.worst = log.worst.max(dt);
    let frame_spans = take_spans();
    // A slow frame (over 20 ms) is logged on its own with its costliest spans and the main
    // world's time, to find what causes hitches.
    if dt > 20.0 && log.frames > 0 {
        let mut top: Vec<(&&str, &f32)> = frame_spans.iter().collect();
        top.sort_by(|a, b| b.1.total_cmp(a.1));
        let list: Vec<String> = top.iter().take(6).map(|(k, v)| format!("{k} {v:.1}")).collect();
        info!("perf spike: frame {dt:.1} ms, main {main:.1} ms: {}", list.join(", "));
    }
    for (k, v) in frame_spans {
        *log.spans.entry(k).or_default() += v;
    }
    if log.frames >= 120 {
        let n = log.frames as f32;
        let visible = meshes.iter().filter(|v| v.get()).count();
        let distinct: std::collections::HashSet<_> = mats.iter().map(|m| m.0.id()).collect();
        // Visible meshes by alpha mode, and the distinct (mesh, material) pairs among them.
        let mut modes: std::collections::BTreeMap<String, usize> = Default::default();
        let mut pairs = std::collections::HashSet::new();
        for (v, mesh, m) in &drawn {
            if !v.get() {
                continue;
            }
            let mode = materials.get(&m.0).map_or("none".to_string(), |x| format!("{:?}", x.alpha_mode).split('(').next().unwrap_or("").to_string());
            *modes.entry(mode).or_default() += 1;
            pairs.insert((mesh.0.id(), m.0.id()));
        }
        info!("perf: visible by alpha mode {:?}, {} distinct mesh-material pairs", modes, pairs.len());
        info!("perf: {} standard-material meshes, {} distinct materials, {} material assets, {} images", mats.iter().count(), distinct.len(), materials.len(), images.len());
        let mut sorted: Vec<(&&str, &f32)> = log.spans.iter().collect();
        sorted.sort_by(|a, b| b.1.total_cmp(a.1));
        let spans: Vec<String> = sorted.iter().map(|(k, v)| format!("{k} {:.2}", *v / n)).collect();
        info!(
            "perf: frame {:.2} ms (worst {:.1}), main {:.2} ms; entities {}, meshes {} ({} visible); spans ms/frame: {}",
            log.frame / n,
            log.worst,
            log.main / n,
            entities.iter().count(),
            meshes.iter().count(),
            visible,
            spans.join(", ")
        );
        *log = PerfLog::default();
    }
}

static SPANS: std::sync::Mutex<std::collections::BTreeMap<&'static str, f32>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// A timed span (NEOPETS_PERF): its time is added to the named total when it drops.
pub struct Span(Option<(&'static str, std::time::Instant)>);

pub fn span(name: &'static str) -> Span {
    Span((*ENABLED.get_or_init(perf_enabled)).then(|| (name, std::time::Instant::now())))
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((name, t)) = self.0 {
            if let Ok(mut m) = SPANS.lock() {
                *m.entry(name).or_default() += t.elapsed().as_secs_f32() * 1000.0;
            }
        }
    }
}

fn take_spans() -> std::collections::BTreeMap<&'static str, f32> {
    SPANS.lock().map(|mut m| std::mem::take(&mut *m)).unwrap_or_default()
}

/// NEOPETS_PERF: the render world's phases timed on the render thread (the span names start
/// with "render:"), by marks between Bevy's chained render sets.
pub fn add_render_timing(app: &mut App) {
    use bevy::render::{Render, RenderApp, RenderSystems as S};
    static MARKS: std::sync::Mutex<Vec<(&'static str, std::time::Instant)>> = std::sync::Mutex::new(Vec::new());
    fn mark(name: &'static str) -> impl FnMut() {
        move || {
            if let Ok(mut m) = MARKS.lock() {
                let now = std::time::Instant::now();
                if let Some(&(prev, t)) = m.last() {
                    if let Ok(mut s) = SPANS.lock() {
                        *s.entry(prev).or_default() += (now - t).as_secs_f32() * 1000.0;
                    }
                }
                if name == "render:end" {
                    m.clear();
                } else {
                    m.push((name, now));
                }
            }
        }
    }
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    // Once: whether the standard material is drawn bindless on this device (many materials
    // sharing one bind group) and which backend draws.
    r.add_systems(Render, (|device: Res<bevy::render::renderer::RenderDevice>, info: Res<bevy::render::renderer::RenderAdapterInfo>, mut done: Local<bool>| {
        if !*done {
            *done = true;
            info!("perf: backend {:?}, standard material bindless {}", info.backend, bevy::pbr::material_uses_bindless_resources::<StandardMaterial>(&device));
            use bevy::render::render_resource::WgpuFeatures as F;
            let f = device.features();
            let l = device.limits();
            info!("perf: binding arrays: texture {} buffer {} storage {} non-uniform {} partial {}; limits: storage buffers {} array samplers {} array textures {}", f.contains(F::TEXTURE_BINDING_ARRAY), f.contains(F::BUFFER_BINDING_ARRAY), f.contains(F::STORAGE_RESOURCE_BINDING_ARRAY), f.contains(F::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING), f.contains(F::PARTIALLY_BOUND_BINDING_ARRAY), l.max_storage_buffers_per_shader_stage, l.max_binding_array_sampler_elements_per_shader_stage, l.max_binding_array_elements_per_shader_stage);
        }
    }).before(S::ExtractCommands));
    r.add_systems(
        Render,
        (
            mark("render:extract commands").before(S::ExtractCommands),
            mark("render:prepare assets").after(S::ExtractCommands).before(S::PrepareAssets),
            mark("render:prepare meshes").after(S::PrepareAssets).before(S::PrepareMeshes),
            mark("render:create views").after(S::PrepareMeshes).before(S::CreateViews),
            mark("render:specialize").after(S::CreateViews).before(S::Specialize),
            mark("render:prepare views").after(S::Specialize).before(S::PrepareViews),
            mark("render:queue").after(S::PrepareViews).before(S::Queue),
            mark("render:phase sort").after(S::Queue).before(S::PhaseSort),
            mark("render:prepare").after(S::PhaseSort).before(S::Prepare).before(S::PrepareResources),
            mark("render:prep batch").after(S::PrepareResources).before(S::PrepareResourcesBatchPhases),
            mark("render:prep write phases").after(S::PrepareResourcesBatchPhases).before(S::PrepareResourcesWritePhaseBuffers),
            mark("render:prep collect").after(S::PrepareResourcesWritePhaseBuffers).before(S::PrepareResourcesCollectPhaseBuffers),
            mark("render:prep flush").after(S::PrepareResourcesCollectPhaseBuffers).before(S::PrepareResourcesFlush),
            mark("render:prep bind groups").after(S::PrepareResourcesFlush).before(S::PrepareBindGroups),
            mark("render:render").after(S::Prepare).before(S::Render),
            mark("render:cleanup").after(S::Render).before(S::Cleanup),
            mark("render:end").after(S::PostCleanup),
        ),
    );
}
