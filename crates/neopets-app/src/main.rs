mod archive_view;
mod character;
mod controls;
mod orbit;
mod pattern;
mod hud_view;
mod play;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use controls::{Browser, axis, movement};
use neopets_data::archive::ArchiveIndex;
use neopets_data::{Disc, iso_path};
use neopets_sim::{FixedStep, PlayerInput, Simulation};
use orbit::Orbit;

/// Play mode's world and hero: the new-game exterior (SPEC-0006 Default start) and Tor's first
/// outfit.
const PLAY_PACK: u32 = 909;
const PLAY_OUTFIT: u32 = 504;

/// Play mode was asked for, in this world pack (`--pack`, default 909).
#[derive(Resource)]
struct PlayRequest(u32, Option<[f32; 2]>, Vec<u16>);

/// Diagnostic: hold the stick forward at walk tilt in play mode (`--autowalk`), so screenshots
/// can show travel without a person at the controls; `--autojump` also presses Cross every 2 s.
#[derive(Resource)]
struct AutoWalk {
    jump: bool,
}

/// The play-mode hero entity; its children are the outfit's meshes.
#[derive(Component)]
struct HeroRoot;

#[derive(Resource)]
pub(crate) struct PlayState {
    pub(crate) play: play::Play,
    skinned: Vec<(Handle<Mesh>, usize)>,
}

/// Game logic runs a fixed 1/60 s step, one input snapshot per tick (SPEC-0007).
const GAME_TICK_HZ: u32 = 60;
/// Ticks run per rendered frame at most; the rest are dropped. The original never catches up
/// (an overlong pass slows the game), which `--original-pacing` reproduces with a cap of 1.
const MAX_CATCH_UP: u32 = 4;
/// Pixel-unit scroll (touchpads) per line-unit scroll step.
const PIXELS_PER_LINE: f32 = 40.0;

#[derive(Resource)]
struct DiscStatus(String);

/// Background loads for overworld streaming (SPEC-0006 Stream loads are asynchronous): a
/// thread with its own disc handle reads a requested pack, learns its kind, and builds its
/// scene when the buffer rule will stream it.
#[derive(Resource)]
struct Streamer {
    requests: std::sync::mpsc::Sender<(u32, bool, bool, bool)>,
    #[allow(clippy::type_complexity)]
    results: std::sync::Mutex<std::sync::mpsc::Receiver<(u32, bool, Result<(play::World, Option<neopets_data::model::ModelData>), String>)>>,
    in_flight: std::collections::HashSet<u32>,
    scenes: std::collections::HashMap<u32, neopets_data::model::ModelData>,
    regions: std::sync::mpsc::Sender<(u32, u8, u32)>,
    #[allow(clippy::type_complexity)]
    region_results: std::sync::Mutex<std::sync::mpsc::Receiver<(u32, u8, u32, Result<RegionLoad, String>)>>,
    /// Region loads sent and not yet received.
    regions_out: usize,
}

type RegionCharacter = (
    neopets_data::region::Placement,
    neopets_data::model::ModelData,
    character::HeroAnim,
    Option<neopets_data::monster::Row>,
    Option<neopets_data::monster::Tuning>,
);

/// A loaded region: its characters and its kind-8 item points.
type RegionLoad = (Vec<RegionCharacter>, Vec<neopets_data::region::ItemPoint>);

/// Read a region resource and build each placed character (summon-reserved records and records
/// with no position are left out: they are placed by a summoner).
fn load_region(disc: &mut Disc, index: &ArchiveIndex, table: &neopets_data::monster::MonsterTable, region: u32) -> Result<RegionLoad, String> {
    let file = index.read(disc, region)?;
    let chain = neopets_data::resource::walk_chunks(&file, None)?;
    // A world pack's own characters are its embedded region: the chain inside its type-32
    // chunk, with offsets relative to that chunk's payload.
    let (bytes, chunks) = match chain.chunks.iter().find(|c| c.kind == neopets_data::region::EMBEDDED) {
        Some(c) if chain.chunks.first().is_some_and(|f| f.kind == 31) => {
            let payload = file[c.payload.clone()].to_vec();
            let nested = neopets_data::resource::walk_chunks(&payload, None)?;
            (payload, nested.chunks)
        }
        _ if chain.chunks.first().is_some_and(|f| f.kind == 31) => return Ok((Vec::new(), Vec::new())),
        _ => (file, chain.chunks),
    };
    let r = neopets_data::region::Region::of(&bytes, &chunks);
    let mut out = Vec::new();
    for pl in r.records.placements.iter().filter(|p| !p.summon && p.position != [0.0; 3]) {
        match archive_view::region_character(disc, index, &bytes, &r, pl) {
            Ok((model, anim)) => out.push((pl.clone(), model, anim, table.row(pl.monster_type, pl.variant), table.tuning(pl.monster_type, pl.variant))),
            Err(e) => eprintln!("Region {region}, {:?}: {e}", pl.name),
        }
    }
    Ok((out, r.records.items))
}

impl Streamer {
    fn start(mut disc: Disc, index: neopets_data::archive::ArchiveIndex, boot: neopets_data::travel::BootTravel) -> Self {
        // A second handle for the region loader.
        let second = disc.reopen();
        let region_index = index.clone();
        let (requests, jobs) = std::sync::mpsc::channel::<(u32, bool, bool, bool)>();
        let (done, results) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // Each request: the pack, whether transitions (true) or other kinds (false) stream
            // now, whether streaming is possible at all, and whether a door job asked for it
            // (its scene is always built).
            for (pack, transitions_stream, any, job) in jobs {
                let result = play::World::load(&mut disc, &index, &boot, pack).and_then(|(world, bytes)| {
                    let streams = any && ((world.kind == neopets_data::travel::Kind::Transition) == transitions_stream)
                        && world.kind != neopets_data::travel::Kind::Interior;
                    let scene = if streams || job {
                        let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                        archive_view::world_scene(&mut disc, &index, pack, &bytes, &chain.chunks, false)?
                    } else {
                        None
                    };
                    Ok((world, scene))
                });
                if done.send((pack, job, result)).is_err() {
                    break;
                }
            }
        });
        let (regions, region_jobs) = std::sync::mpsc::channel::<(u32, u8, u32)>();
        let (region_done, region_results) = std::sync::mpsc::channel();
        if let Ok(mut disc) = second {
            std::thread::spawn(move || {
                // The monster table (SPEC-0029 section 4), read once.
                let table = match (region_index.read(&mut disc, 530), region_index.read(&mut disc, 531)) {
                    (Ok(a), Ok(b)) => neopets_data::monster::MonsterTable::parse(&a, &b).unwrap_or_default(),
                    _ => Default::default(),
                };
                for (pack, slot, region) in region_jobs {
                    let result = load_region(&mut disc, &region_index, &table, region);
                    if region_done.send((pack, slot, region, result)).is_err() {
                        break;
                    }
                }
            });
        }
        Self {
            requests,
            results: std::sync::Mutex::new(results),
            in_flight: Default::default(),
            scenes: Default::default(),
            regions,
            region_results: std::sync::Mutex::new(region_results),
            regions_out: 0,
        }
    }

    /// Send the pending step's and the door job's requests, install finished loads; returns a
    /// line per finished load.
    fn pump(&mut self, p: &mut play::Play) -> Vec<String> {
        let buffer = p.world.buffer;
        let transitions = buffer == neopets_sim::travel::Buffer::Exterior;
        let any = buffer != neopets_sim::travel::Buffer::Interior;
        if let Some(pack) = p.wanted.take() {
            if self.in_flight.insert(pack) {
                let _ = self.requests.send((pack, transitions, any, false));
            }
        }
        if let Some(pack) = p.job_pack() {
            if self.in_flight.insert(pack) {
                let _ = self.requests.send((pack, transitions, any, true));
            }
        }
        for (pack, slot, region) in p.region_wanted.drain(..) {
            if self.regions.send((pack, slot, region)).is_ok() {
                self.regions_out += 1;
            }
        }
        let mut log = Vec::new();
        let regions: Vec<_> = self.region_results.lock().map(|r| r.try_iter().collect()).unwrap_or_default();
        for (pack, slot, region, result) in regions {
            self.regions_out = self.regions_out.saturating_sub(1);
            match result {
                Ok((list, items)) => {
                    let names: Vec<String> = list.iter().map(|c| c.0.name.clone()).collect();
                    log.push(format!("region {region} for slot {slot} of {pack} on tick {}: {names:?}, {} item points", p.ticks, items.len()));
                    p.add_characters(pack, slot, region, list, items);
                }
                Err(e) => log.push(format!("region {region} failed: {e}")),
            }
        }
        let finished: Vec<_> = self.results.lock().map(|r| r.try_iter().collect()).unwrap_or_default();
        for (pack, job, result) in finished {
            self.in_flight.remove(&pack);
            match result {
                Ok((world, scene)) => {
                    log.push(format!("loaded {pack} on tick {} ({:?}, scene {}, door request {job})", p.ticks, world.kind, scene.is_some()));
                    if let Some(scene) = scene {
                        self.scenes.insert(pack, scene);
                    }
                    // A load of the job's pack finishes the job, whichever request read it.
                    if p.job_pack() == Some(pack) {
                        p.finish_job(world);
                    } else {
                        p.install(world);
                    }
                }
                Err(e) => log.push(format!("load of {pack} failed: {e}")),
            }
        }
        log
    }
}

/// Send the pending step's request to the loader and install finished loads.
fn stream_worlds(play: Option<ResMut<PlayState>>, streamer: Option<ResMut<Streamer>>) {
    let (Some(mut state), Some(mut streamer)) = (play, streamer) else { return };
    for line in streamer.pump(&mut state.play) {
        info!("{line}");
    }
}

/// The opened disc and its archive index, present only when the ISO validated.
#[derive(Resource)]
struct Archive {
    disc: Disc,
    index: ArchiveIndex,
}

/// World height of the texture quad; its width follows the texture's aspect ratio.
const QUAD_HEIGHT: f32 = 4.0;

#[derive(Component)]
struct TextureQuad;

/// The diagnostic grid and axes, hidden while a model is shown.
#[derive(Component)]
struct DiagnosticGrid;

/// Every entity spawned for the current model, despawned when the selection changes.
#[derive(Component)]
struct ModelPart;

/// The selected resource's model, and whether the viewer shows it or the texture quad.
#[derive(Resource, Default)]
struct ModelView {
    data: Option<neopets_data::model::ModelData>,
    show_model: bool,
    dirty: bool,
    /// Meshes of skinned parts (layout A) and their part index, for per-frame skinning.
    skinned: Vec<(Handle<Mesh>, usize)>,
    hero: Option<character::HeroAnim>,
}

impl ModelView {
    fn showing(&self) -> bool {
        self.show_model && self.data.is_some()
    }
}

/// Decoded textures of the selected resource, and which one the quad shows.
#[derive(Resource, Default)]
struct TextureView {
    images: Vec<(String, neopets_data::texture::Image)>,
    current: usize,
    dirty: bool,
}

pub(crate) fn rgba_image(width: u32, height: u32, rgba: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Summary text for the last resource described, so the disc is read once per selection.
#[derive(Resource, Default)]
struct BrowserView {
    shown: Option<u32>,
    text: String,
}

/// Host side of the input -> sim -> render loop. All game logic stays in neopets-sim.
#[derive(Resource)]
struct SimDriver {
    sim: Simulation,
    clock: FixedStep,
    input: PlayerInput,
    last_ticks: u32,
}

#[derive(Component)]
struct Hud;

/// Whether the HUD starts hidden (`--hide-hud`); Tab toggles it afterwards.
#[derive(Resource)]
struct HudHidden(bool);

#[derive(Component)]
struct SimText;

#[derive(Component)]
struct BrowserText;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut explicit = None;
    let mut smoke_test = false;
    let mut screenshot = None;
    let mut hide_hud = false;
    let mut catch_up = MAX_CATCH_UP;
    let mut play_mode = false;
    let mut play_pack = PLAY_PACK;
    let mut autowalk = None;
    let mut shot_at = 4.0f32;
    let mut start_resource = 0u32;
    let mut trace: Option<u64> = None;
    let mut trace_to: Option<[f32; 2]> = None;
    let mut flags: Vec<u16> = Vec::new();
    let mut look: Option<[f32; 2]> = None;
    let mut seed: Option<u32> = None;
    let mut give: Vec<u16> = Vec::new();
    // `--guard`: the fight trace holds L1 (the shield guard) while it is close to its enemy.
    let mut guard = false;
    let mut start: Option<[f32; 2]> = None;
    while let Some(arg) = args.next() {
        if arg == "--iso" {
            explicit = Some(
                args.next()
                    .unwrap_or_else(|| {
                        eprintln!("--iso requires a path");
                        std::process::exit(2)
                    })
                    .into(),
            );
        } else if arg == "--smoke-test" {
            smoke_test = true;
        } else if arg == "--resource" {
            start_resource = args
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or_else(|| {
                    eprintln!("--resource requires a resource number");
                    std::process::exit(2)
                });
        } else if arg == "--autowalk" {
            autowalk = Some(AutoWalk { jump: false });
        } else if arg == "--autojump" {
            autowalk = Some(AutoWalk { jump: true });
        } else if arg == "--play-trace" {
            trace = Some(args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--play-trace requires a tick count");
                std::process::exit(2)
            }));
        } else if arg == "--pack" {
            play_pack = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--pack requires a world pack resource number");
                std::process::exit(2)
            });
        } else if arg == "--trace-to" {
            trace_to = args.next().and_then(|v| {
                if v == "swap" {
                    return Some([f32::NAN, f32::NAN]);
                }
                if v == "new" {
                    return Some([f32::NAN, 0.0]);
                }
                if v == "door" {
                    return Some([f32::NAN, 1.0]);
                }
                if v == "herd" {
                    return Some([f32::NAN, 2.0]);
                }
                if v == "fight" {
                    return Some([f32::NAN, 3.0]);
                }
                let (x, z) = v.trim().split_once(',')?;
                Some([x.parse().ok()?, z.parse().ok()?])
            });
        } else if arg == "--look" {
            look = args.next().and_then(|v| {
                let (x, z) = v.trim().split_once(',')?;
                Some([x.parse::<f32>().ok()?, z.parse::<f32>().ok()?])
            });
        } else if arg == "--start" {
            start = args.next().and_then(|v| {
                let (x, z) = v.trim().split_once(',')?;
                Some([x.parse::<f32>().ok()?, z.parse::<f32>().ok()?])
            });
        } else if arg == "--give" {
            give.extend(args.next().and_then(|v| v.parse::<u16>().ok()));
        } else if arg == "--guard" {
            guard = true;
        } else if arg == "--seed" {
            seed = args.next().and_then(|v| v.parse::<u32>().ok());
        } else if arg == "--flag" {
            flags.extend(args.next().and_then(|v| v.parse::<u16>().ok()));
        } else if arg == "--play" {
            play_mode = true;
        } else if arg == "--original-pacing" {
            catch_up = 1;
        } else if arg == "--hide-hud" {
            hide_hud = true;
        } else if arg == "--screenshot-at" {
            shot_at = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--screenshot-at requires seconds");
                std::process::exit(2)
            });
        } else if arg == "--screenshot" {
            screenshot = Some(args.next().unwrap_or_else(|| {
                eprintln!("--screenshot requires an output .png path");
                std::process::exit(2)
            }));
        } else {
            eprintln!("Unknown argument: {arg}");
            std::process::exit(2);
        }
    }
    let root = std::env::current_dir().expect("Current directory");
    let result = iso_path(&root, explicit).and_then(|path| {
        let mut disc = Disc::open(&path)?;
        let name = disc.verify_neopets()?;
        let index = ArchiveIndex::from_disc(&mut disc)?;
        let status = format!(
            "{name}\n{} disc files indexed. Opened read-only. Boot fingerprint matches; {} archive resources.",
            disc.files().len(),
            index.len()
        );
        Ok((status, Archive { disc, index }))
    });
    let (status, archive) = match result {
        Ok((s, archive)) => (s, Some(archive)),
        Err(e) => (format!("ISO not ready\n{e}"), None),
    };
    if let Some(ticks) = trace {
        let Some(mut archive) = archive else {
            eprintln!("{status}");
            std::process::exit(1)
        };
        play_trace(&mut archive, play_pack, ticks, autowalk.is_some_and(|a| a.jump), trace_to, &flags, seed, &give, start, guard);
        return;
    }
    let driver = SimDriver {
        sim: Simulation::default(),
        clock: FixedStep::new(GAME_TICK_HZ, catch_up).expect("Positive tick rate"),
        input: PlayerInput::default(),
        last_ticks: 0,
    };
    let mut app = App::new();
    if let Some(archive) = archive {
        app.insert_resource(archive);
        if play_mode {
            app.insert_resource(PlayRequest(play_pack, look, give.clone()));
        }
        if let Some(auto) = autowalk {
            app.insert_resource(auto);
        }
    }
    app.insert_resource(DiscStatus(status))
        .insert_resource(HudHidden(hide_hud))
        .insert_resource(driver)
        .insert_resource(Browser {
            selected: if play_mode { play_pack } else { start_resource },
            typing: None,
        })
        .init_resource::<BrowserView>()
        .init_resource::<TextureView>()
        .init_resource::<ModelView>()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.06)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Neopets • Rust rewrite workspace".into(),
                resolution: (1000, 600).into(),
                visible: !smoke_test,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_viewer, setup_hud, setup_play))
        .add_systems(
            Update,
            (
                (read_input, step_sim, stream_worlds).chain(),
                (browse_resources, cycle_textures, toggle_view, choose_action, orbit_camera, toggle_hud),
                (show_sim, show_browser),
                (apply_texture, apply_model, skin_hero, sync_worlds, play_frame, sync_characters, sync_shots, sync_items, hud_view::sync_hud).chain(),
            )
                .chain(),
        );
    if smoke_test {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.add_systems(Update, smoke_exit);
    } else if let Some(path) = screenshot {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .insert_resource(ScreenshotPath(path, shot_at.max(1.0)))
            .add_systems(Update, screenshot_and_exit);
    }
    app.run();
}

#[derive(Resource)]
struct ScreenshotPath(String, f32);

/// Capture one frame once rendering has settled, then exit; for reviewing the window.
/// Waits on real time, not frames: render pipelines compile asynchronously after startup,
/// and meshes are simply not drawn until theirs are ready.
fn screenshot_and_exit(
    mut commands: Commands,
    time: Res<Time<Real>>,
    path: Res<ScreenshotPath>,
    mut taken: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let elapsed = time.elapsed_secs();
    if !*taken && elapsed >= path.1 {
        *taken = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0.clone()));
    } else if elapsed >= path.1 + 2.0 {
        // Saving runs asynchronously; exiting too soon drops the capture.
        exit.write(AppExit::Success);
    }
}

fn smoke_exit(
    frame: Res<bevy::diagnostic::FrameCount>,
    driver: Res<SimDriver>,
    mut exit: MessageWriter<AppExit>,
) {
    if frame.0 >= 5 {
        info!(
            "Startup smoke check passed after five frames; scaffold sim at tick {}.",
            driver.sim.tick
        );
        exit.write(AppExit::Success);
    }
}

/// Orbit camera over a diagnostic grid and axes (X red, Y green, Z blue). None of this is
/// game content; decoded textures, models, and areas will replace it in M3/M4/M6.
fn setup_viewer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // The quad starts with the synthetic pattern; a selected resource's textures replace it.
    let texture = match pattern::rgba() {
        Ok(rgba) => Some(images.add(rgba_image(pattern::SIZE, pattern::SIZE, rgba))),
        Err(e) => {
            error!("Synthetic test pattern failed to decode: {e}");
            None
        }
    };
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(QUAD_HEIGHT, QUAD_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: texture,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(-3.0, QUAD_HEIGHT / 2.0 + 0.2, -3.0),
        TextureQuad,
    ));
    let orbit = Orbit::default();
    // No tone mapping: decoded textures should show their stored colours.
    commands.spawn((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            near: 0.02,
            far: 400_000.0,
            ..default()
        }),
        orbit.transform(),
        orbit,
    ));
    let mut unlit = |r: f32, g: f32, b: f32| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            unlit: true,
            ..default()
        })
    };
    let grid = unlit(0.16, 0.20, 0.28);
    let axes = [
        (Vec3::new(5.0, 0.0, 0.0), Vec3::new(10.0, 0.06, 0.06), unlit(0.9, 0.25, 0.25)),
        (Vec3::new(0.0, 2.5, 0.0), Vec3::new(0.06, 5.0, 0.06), unlit(0.3, 0.85, 0.35)),
        (Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.06, 0.06, 10.0), unlit(0.3, 0.5, 0.95)),
    ];
    const HALF: i32 = 10;
    let line_x = meshes.add(Cuboid::new(2.0 * HALF as f32, 0.02, 0.02));
    let line_z = meshes.add(Cuboid::new(0.02, 0.02, 2.0 * HALF as f32));
    for i in -HALF..=HALF {
        let at = i as f32;
        commands.spawn((
            Mesh3d(line_x.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(0.0, 0.0, at),
            DiagnosticGrid,
        ));
        commands.spawn((
            Mesh3d(line_z.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(at, 0.0, 0.0),
            DiagnosticGrid,
        ));
    }
    for (center, size, material) in axes {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material),
            Transform::from_translation(center),
            DiagnosticGrid,
        ));
    }
}

fn setup_hud(mut commands: Commands, status: Res<DiscStatus>, hidden: Res<HudHidden>) {
    let font = |px: f32| TextFont {
        font_size: bevy::text::FontSize::Px(px),
        ..default()
    };
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(18),
                margin: UiRect::all(px(32)),
                max_width: percent(90),
                display: if hidden.0 { Display::None } else { Display::Flex },
                padding: UiRect::all(px(20)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.85)),
            Hud,
        ))
        .with_children(|ui| {
            ui.spawn((
                Text::new(format!(
                    "NEOPETS\nThe Darkest Faerie - rewrite workspace\n\n{}\n\nArea rendering and gameplay are not implemented yet. Original content comes from your ISO. No BIOS is used.",
                    // Bevy's built-in font has no bullet glyph.
                    status.0.replace(" • ", " | ")
                )),
                font(20.0),
                TextColor(Color::srgb(0.87, 0.91, 0.97)),
            ));
            ui.spawn((
                Text::default(),
                font(16.0),
                TextColor(Color::srgb(0.70, 0.85, 0.75)),
                SimText,
            ));
            ui.spawn((
                Text::default(),
                font(16.0),
                TextColor(Color::srgb(0.95, 0.82, 0.62)),
                BrowserText,
            ));
            ui.spawn((
                Text::new(
                    "VIEWER - diagnostic grid and axes. The quad shows the selected resource's textures, \
                     decoded from your ISO (or a synthetic pattern when there is no ISO)\n\
                     Drag with a mouse button to orbit | scroll to zoom | Tab hides this text",
                ),
                font(16.0),
                TextColor(Color::srgb(0.65, 0.72, 0.95)),
            ));
        });
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut driver: ResMut<SimDriver>,
) {
    let held = |pair: [KeyCode; 2]| keys.any_pressed(pair);
    let keyboard = Vec2::new(
        axis(
            held([KeyCode::KeyA, KeyCode::ArrowLeft]),
            held([KeyCode::KeyD, KeyCode::ArrowRight]),
        ),
        axis(
            held([KeyCode::KeyS, KeyCode::ArrowDown]),
            held([KeyCode::KeyW, KeyCode::ArrowUp]),
        ),
    );
    driver.input = movement(keyboard, gamepads.iter().map(|pad| pad.left_stick()));
}

fn step_sim(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut driver: ResMut<SimDriver>,
    mut models: ResMut<ModelView>,
    play: Option<ResMut<PlayState>>,
    autowalk: Option<Res<AutoWalk>>,
) {
    let elapsed = time.delta().as_nanos().min(u64::MAX as u128) as u64;
    let SimDriver {
        sim, clock, input, ..
    } = &mut *driver;
    let ticks = sim.run_for(clock, elapsed, *input);
    driver.last_ticks = ticks;
    if let Some(hero) = models.hero.as_mut() {
        for _ in 0..ticks {
            let _ = hero.tick();
        }
    }
    if let Some(mut play) = play {
        let i = driver.input;
        let mut input = neopets_sim::hero::HeroInput {
            side: -(i.move_x as f32) / i16::MAX as f32,
            forward: i.move_y as f32 / i16::MAX as f32,
            run: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                || gamepads.iter().any(|g| g.pressed(GamepadButton::RightTrigger)),
            jump: false,
            square: false,
            // L1 (Q or the left bumper) holds the shield guard.
            guard: keys.pressed(KeyCode::KeyQ) || gamepads.iter().any(|g| g.pressed(GamepadButton::LeftTrigger)),
            attack_held: keys.pressed(KeyCode::KeyF)
                || mouse.pressed(MouseButton::Left)
                || gamepads.iter().any(|g| g.pressed(GamepadButton::East) || g.pressed(GamepadButton::RightTrigger2)),
            attack: false,
        };
        // The attack button (F, left mouse, Circle or R2): its press edge, kept for the next tick
        // when this frame runs none.
        play.play.attack_latch |= keys.just_pressed(KeyCode::KeyF)
            || mouse.just_pressed(MouseButton::Left)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::RightTrigger2));
        // The quick-item panel (SPEC-0021): the D-pad or numpad 8/4/6/2 for its directions, T or
        // the north face button for Triangle; Cross and Circle are the jump and attack edges.
        {
            let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
            let l = &mut play.play.panel_latch;
            l.up |= keys.just_pressed(KeyCode::Numpad8) || pad(GamepadButton::DPadUp);
            l.left |= keys.just_pressed(KeyCode::Numpad4) || pad(GamepadButton::DPadLeft);
            l.right |= keys.just_pressed(KeyCode::Numpad6) || pad(GamepadButton::DPadRight);
            l.down |= keys.just_pressed(KeyCode::Numpad2) || pad(GamepadButton::DPadDown);
            l.triangle |= keys.just_pressed(KeyCode::KeyT) || pad(GamepadButton::North);
        }
        // Square (E or the west face button) confirms a prompt; a press edge like Cross.
        play.play.square_latch |= keys.just_pressed(KeyCode::KeyE)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::West));
        // Cross (Space or the south face button) is a press edge; keep it for the next tick
        // when this frame runs none.
        play.play.jump_latch |= keys.just_pressed(KeyCode::Space)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::South));
        if let Some(auto) = autowalk.as_deref() {
            input.forward = 0.7;
            if auto.jump && ticks > 0 && play.play.ticks % 120 == 0 {
                play.play.jump_latch = true;
            }
        }
        // The right stick as PS2 pad bytes (X right 255, Y up 0); J/L and I/K stand in for it
        // on the keyboard, R for R3.
        let mut right = gamepads.iter().map(|g| g.right_stick()).find(|v| v.length() > 0.2).unwrap_or(Vec2::ZERO);
        let key = |k: KeyCode| keys.pressed(k) as i32 as f32;
        right.x += key(KeyCode::KeyL) - key(KeyCode::KeyJ);
        right.y += key(KeyCode::KeyI) - key(KeyCode::KeyK);
        let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
        let r3_held = keys.pressed(KeyCode::KeyR) || gamepads.iter().any(|g| g.pressed(GamepadButton::RightThumb));
        play.play.r3_latch |= keys.just_pressed(KeyCode::KeyR)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::RightThumb));
        for _ in 0..ticks {
            input.jump = std::mem::take(&mut play.play.jump_latch);
            input.square = std::mem::take(&mut play.play.square_latch);
            input.attack = std::mem::take(&mut play.play.attack_latch);
            let pad = neopets_sim::camera::CameraPad {
                right_x: byte(right.x),
                right_y: byte(-right.y),
                r3_held,
                r3_pressed: std::mem::take(&mut play.play.r3_latch),
            };
            play.play.tick(&input, pad);
        }
    }
}

fn browse_resources(keys: Res<ButtonInput<KeyCode>>, mut browser: ResMut<Browser>) {
    const DIGITS: [(KeyCode, KeyCode, char); 10] = [
        (KeyCode::Digit0, KeyCode::Numpad0, '0'),
        (KeyCode::Digit1, KeyCode::Numpad1, '1'),
        (KeyCode::Digit2, KeyCode::Numpad2, '2'),
        (KeyCode::Digit3, KeyCode::Numpad3, '3'),
        (KeyCode::Digit4, KeyCode::Numpad4, '4'),
        (KeyCode::Digit5, KeyCode::Numpad5, '5'),
        (KeyCode::Digit6, KeyCode::Numpad6, '6'),
        (KeyCode::Digit7, KeyCode::Numpad7, '7'),
        (KeyCode::Digit8, KeyCode::Numpad8, '8'),
        (KeyCode::Digit9, KeyCode::Numpad9, '9'),
    ];
    for (main, pad, digit) in DIGITS {
        if keys.just_pressed(main) || keys.just_pressed(pad) {
            browser.type_digit(digit);
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        browser.backspace();
    }
    if keys.just_pressed(KeyCode::Escape) {
        browser.cancel();
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        browser.commit();
    }
    let step: i64 = [
        (KeyCode::BracketLeft, -1),
        (KeyCode::BracketRight, 1),
        (KeyCode::PageDown, -100),
        (KeyCode::PageUp, 100),
    ]
    .into_iter()
    .filter(|(key, _)| keys.just_pressed(*key))
    .map(|(_, d)| d)
    .sum();
    if step != 0 {
        browser.step(step);
    }
}

fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time<Real>>,
    models: Res<ModelView>,
    play: Option<Res<PlayState>>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
) {
    // In play mode the follow camera owns the view.
    if play.is_some() {
        return;
    }
    let dragging = buttons.any_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_LINE,
    };
    // While a model is shown, WASD and Q/E move the point the camera orbits (a fly camera).
    let held = |k: KeyCode| keys.pressed(k) as i32 as f32;
    let fly = if models.showing() && play.is_none() {
        Vec3::new(
            held(KeyCode::KeyD) - held(KeyCode::KeyA),
            held(KeyCode::KeyE) - held(KeyCode::KeyQ),
            held(KeyCode::KeyS) - held(KeyCode::KeyW),
        )
    } else {
        Vec3::ZERO
    };
    if !(dragging && motion.delta != Vec2::ZERO) && lines == 0.0 && fly == Vec3::ZERO {
        return;
    }
    let boost = if keys.pressed(KeyCode::ShiftLeft) { 4.0 } else { 1.0 };
    for (mut orbit, mut transform) in &mut cameras {
        if dragging {
            // Screen y grows downward; dragging up should raise the camera.
            orbit.drag(Vec2::new(motion.delta.x, -motion.delta.y));
        }
        orbit.zoom(lines);
        let step = orbit.distance * 0.75 * boost * time.delta_secs();
        orbit.fly(fly * step);
        *transform = orbit.transform();
    }
}

fn toggle_hud(keys: Res<ButtonInput<KeyCode>>, mut huds: Query<&mut Node, With<Hud>>) {
    if keys.just_pressed(KeyCode::Tab) {
        for mut node in &mut huds {
            node.display = match node.display {
                Display::None => Display::Flex,
                _ => Display::None,
            };
        }
    }
}

fn show_sim(driver: Res<SimDriver>, play: Option<Res<PlayState>>, mut texts: Query<&mut Text, With<SimText>>) {
    let d = &*driver;
    let play_line = play
        .map(|p| format!("{}\n{}\n{}\n", p.play.status(), p.play.status_camera(), p.play.status_worlds()))
        .unwrap_or_default();
    for mut text in &mut texts {
        text.0 = play_line.clone() + &format!(
            "SIMULATION LOOP - fixed 1/60 s step (SPEC-0007); no gameplay systems yet\n\
             Tick {} at {} Hz | {} this frame | {} dropped | alpha {:.2}\n\
             Input: move ({}, {}) from WASD / arrows / left stick",
            d.sim.tick,
            d.clock.rate_hz(),
            d.last_ticks,
            d.clock.dropped(),
            d.clock.alpha(),
            d.input.move_x,
            d.input.move_y,
        );
    }
}

fn show_browser(
    mut browser: ResMut<Browser>,
    archive: Option<ResMut<Archive>>,
    mut view: ResMut<BrowserView>,
    mut textures: ResMut<TextureView>,
    mut models: ResMut<ModelView>,
    mut texts: Query<&mut Text, With<BrowserText>>,
    play_request: Option<Res<PlayRequest>>,
) {
    let Some(mut archive) = archive else {
        for mut text in &mut texts {
            text.0 = "RESOURCE BROWSER\nNo validated ISO, so the archive is unavailable.".into();
        }
        return;
    };
    let Archive { disc, index } = &mut *archive;
    browser.clamp(index.len() as u32);
    let n = browser.selected;
    if view.shown != Some(n) {
        view.shown = Some(n);
        view.text = archive_view::describe(disc, index, n)
            .unwrap_or_else(|e| format!("Resource #{n}: {e}"));
        let decoded = archive_view::textures(disc, index, n);
        if let Err(e) = &decoded {
            view.text.push_str(&format!("\nTextures could not be decoded: {e}"));
        }
        *textures = TextureView {
            images: decoded.unwrap_or_default(),
            current: 0,
            dirty: true,
        };
        // In play mode the resident worlds are drawn by `sync_worlds` instead.
        let model = if play_request.is_some() { Ok(None) } else { archive_view::model(disc, index, n) };
        if let Err(e) = &model {
            view.text.push_str(&format!("\nModel could not be built: {e}"));
        }
        let data = model.ok().flatten();
        let hero = match archive_view::hero(disc, index, n) {
            Ok(h) => h,
            Err(e) => {
                view.text.push_str(&format!("\nAnimation could not be loaded: {e}"));
                None
            }
        };
        *models = ModelView {
            show_model: data.is_some(),
            data,
            dirty: true,
            skinned: Vec::new(),
            hero,
        };
    }
    let model_line = match &models.data {
        Some(m) => {
            let verts: usize = m.parts.iter().map(|p| p.positions.len()).sum();
            let tris: usize = m.parts.iter().map(|p| p.indices.len() / 3).sum();
            let mut skipped = String::new();
            if !m.skipped_leaves.is_empty() {
                skipped += &format!(", {} leaves not drawn", m.skipped_leaves.len());
            }
            if m.unresolved_placements > 0 {
                skipped += &format!(", {} stale placements skipped", m.unresolved_placements);
            }
            format!(
                "Model (SPEC-0004): {} parts, {verts} vertices, {tris} triangles, {} textures{skipped}. \
                 Unlit. V switches model/texture view; WASD/QE fly",
                m.parts.len(),
                m.textures.len()
            )
        }
        None if play_request.is_some() => "Play mode draws the resident worlds (SPEC-0006).".into(),
        None => "No models in this resource.".into(),
    };
    let model_line = match &models.hero {
        Some(h) => format!("{model_line}\n{}", h.status()),
        None => model_line,
    };
    let texture_line = match textures.images.get(textures.current) {
        Some((label, image)) => format!(
            "Texture {}/{}: {label}, {}x{} (, and . to cycle)",
            textures.current + 1,
            textures.images.len(),
            image.width,
            image.height
        ),
        None => "No textures in this resource.".into(),
    };
    let entry = match &browser.typing {
        Some(t) => format!("Go to: {t}_   (Enter to jump, Esc to cancel)"),
        None => "Type a number and press Enter | [ ] step 1 | PgUp/PgDn step 100".into(),
    };
    for mut text in &mut texts {
        text.0 = format!(
            "RESOURCE BROWSER - archive index (SPEC-0001/0002), textures (SPEC-0003)\n{}\n{model_line}\n{texture_line}\n{entry}",
            view.text
        );
    }
}

fn cycle_textures(keys: Res<ButtonInput<KeyCode>>, mut textures: ResMut<TextureView>) {
    let len = textures.images.len();
    if len < 2 {
        return;
    }
    let forward = keys.just_pressed(KeyCode::Period) as usize;
    let back = keys.just_pressed(KeyCode::Comma) as usize * (len - 1);
    let step = (forward + back) % len;
    if step != 0 {
        textures.current = (textures.current + step) % len;
        textures.dirty = true;
    }
}

/// Put the current decoded texture on the quad, sized to its aspect ratio; hide the quad when
/// the selected resource has no textures.
fn apply_texture(
    mut textures: ResMut<TextureView>,
    models: Res<ModelView>,
    mut quads: Query<
        (&MeshMaterial3d<StandardMaterial>, &mut Transform, &mut Visibility),
        With<TextureQuad>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !textures.dirty {
        return;
    }
    textures.dirty = false;
    for (material, mut transform, mut visibility) in &mut quads {
        let Some((_, image)) = textures.images.get(textures.current) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if models.showing() {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Inherited;
        transform.scale.x = image.width as f32 / image.height as f32;
        if let Some(mut m) = materials.get_mut(&material.0) {
            let handle = images.add(rgba_image(image.width, image.height, image.rgba.clone()));
            if let Some(old) = m.base_color_texture.replace(handle) {
                images.remove(&old);
            }
        }
    }
}

fn toggle_view(
    keys: Res<ButtonInput<KeyCode>>,
    mut models: ResMut<ModelView>,
    mut textures: ResMut<TextureView>,
) {
    if keys.just_pressed(KeyCode::KeyV) && models.data.is_some() {
        models.show_model = !models.show_model;
        models.dirty = true;
        textures.dirty = true;
    }
}

fn model_texture(t: &neopets_data::model::ModelTexture) -> Image {
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
    let mut image = rgba_image(t.image.width, t.image.height, t.image.rgba.clone());
    let wrap = |repeat: bool| {
        if repeat { ImageAddressMode::Repeat } else { ImageAddressMode::ClampToEdge }
    };
    // Materials on disc magnify bilinearly (TEX1 MMAG = 1, SPEC-0003).
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: wrap(t.repeat_u),
        address_mode_v: wrap(t.repeat_v),
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

/// Spawn the selected model's parts (or remove them), and frame the camera on what is shown.
fn apply_model(
    mut commands: Commands,
    play_request: Option<Res<PlayRequest>>,
    mut models: ResMut<ModelView>,
    old: Query<Entity, With<ModelPart>>,
    mut grid: Query<&mut Visibility, With<DiagnosticGrid>>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !models.dirty {
        return;
    }
    models.dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let grid_visibility = if models.showing() || play_request.is_some() { Visibility::Hidden } else { Visibility::Inherited };
    for mut v in &mut grid {
        *v = grid_visibility;
    }
    let mut skinned = Vec::new();
    let framing = match (&models.data, models.show_model) {
        (Some(m), true) => {
            for (i, mesh, material) in part_assets(m, &mut meshes, &mut materials, &mut images) {
                if !m.parts[i].weights.is_empty() {
                    skinned.push((mesh.clone(), i));
                }
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, ModelPart));
            }
            let (lo, hi) = if m.world { m.dense_box(0.6) } else { (m.min, m.max) };
            Orbit::framing(Vec3::from(lo), Vec3::from(hi))
        }
        _ => Orbit::default(),
    };
    models.skinned = skinned;
    for (mut orbit, mut transform) in &mut cameras {
        *orbit = framing.clone();
        *transform = orbit.transform();
    }
}

/// A drawn piece of a resident world in play mode: a world pack's scene or a sky dome.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum WorldPart {
    Scene(u32),
    Dome(u32),
}

/// Keep the drawn worlds in step with the resident buffers (SPEC-0006): the current world, the
/// other world when the neighbour-world test passes, and the current world's sky dome.
#[allow(clippy::too_many_arguments)]
fn sync_worlds(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut streamer: Option<ResMut<Streamer>>,
    parts: Query<(Entity, &WorldPart)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Local<Vec<WorldPart>>,
) {
    let (Some(state), Some(mut archive)) = (play, archive) else { return };
    let p = &state.play;
    let mut want = vec![WorldPart::Scene(p.world.pack)];
    if let (true, Some(o)) = (p.other_drawn(), &p.other) {
        want.push(WorldPart::Scene(o.pack));
    }
    let slot = p.boot.areas.slot_of(p.world.pack);
    want.push(WorldPart::Dome(neopets_data::area::dome_for_slot(slot)));
    if *shown == want {
        return;
    }
    for (e, part) in &parts {
        if !want.contains(part) {
            commands.entity(e).despawn();
        }
    }
    let Archive { disc, index } = &mut *archive;
    for part in &want {
        if shown.contains(part) {
            continue;
        }
        let streamed = match (*part, streamer.as_deref_mut()) {
            (WorldPart::Scene(pack), Some(s)) => s.scenes.remove(&pack),
            _ => None,
        };
        let model = match *part {
            WorldPart::Scene(_) if streamed.is_some() => Ok(streamed),
            WorldPart::Scene(pack) => index
                .read(disc, pack)
                .and_then(|bytes| {
                    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                    archive_view::world_scene(disc, index, pack, &bytes, &chain.chunks, false)
                }),
            WorldPart::Dome(d) => archive_view::model(disc, index, d),
        };
        match model {
            Ok(Some(m)) => {
                for (_, mesh, material) in part_assets(&m, &mut meshes, &mut materials, &mut images) {
                    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, *part));
                }
            }
            Ok(None) => {}
            Err(e) => error!("{part:?} could not be built: {e}"),
        }
    }
    *shown = want;
}

/// A live enemy projectile's marker, by pool slot.
#[derive(Component)]
struct ShotMarker(usize);

/// Enemy projectiles are drawn as small plain markers at their positions: the flight effects
/// of SPEC-0034 are not implemented, so this is a diagnostic stand-in, not the game's visual.
fn sync_shots(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut markers: Query<(&ShotMarker, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut made: Local<bool>,
) {
    let Some(state) = state else { return };
    if !*made {
        *made = true;
        let mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
        let material = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.85, 0.2), unlit: true, ..default() });
        for i in 0..neopets_sim::projectile::SLOTS {
            commands.spawn((ShotMarker(i), Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden));
        }
        return;
    }
    for (m, mut t, mut vis) in &mut markers {
        match state.play.shots.slots.get(m.0).copied().flatten() {
            Some(s) => {
                t.translation = Vec3::from(s.position);
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// A world item's marker, by (pack, pool slot).
#[derive(Component)]
struct ItemMarker(u32, usize);

/// World items are drawn as small plain markers (gold for coins, blue for charms, green for
/// other items), turned by their spin angle and scaled by their drawn scale: the item display
/// models of SPEC-0021 are not loaded, so this is a diagnostic stand-in.
fn sync_items(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut markers: Query<(Entity, &ItemMarker, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: Local<Option<(Handle<Mesh>, [Handle<StandardMaterial>; 3])>>,
) {
    let Some(state) = state else { return };
    let p = &state.play;
    let (mesh, mats) = assets.get_or_insert_with(|| {
        let colour = |r, g, b| StandardMaterial { base_color: Color::srgb(r, g, b), unlit: true, ..default() };
        (
            meshes.add(Cuboid::new(0.5, 0.5, 0.15)),
            [materials.add(colour(1.0, 0.8, 0.1)), materials.add(colour(0.3, 0.7, 1.0)), materials.add(colour(0.3, 0.9, 0.3))],
        )
    });
    let mut seen = std::collections::HashSet::new();
    for (e, m, mut t, mut vis) in &mut markers {
        match p.pools.get(&m.0).and_then(|pool| pool.slots.get(m.1).copied().flatten()) {
            Some(it) if p.pack_drawn(m.0) => {
                t.translation = Vec3::from(it.position);
                t.rotation = Quat::from_rotation_y(it.angle);
                t.scale = Vec3::splat(it.drawn_scale().max(0.001));
                *vis = Visibility::Visible;
                seen.insert((m.0, m.1));
            }
            Some(_) => {
                *vis = Visibility::Hidden;
                seen.insert((m.0, m.1));
            }
            None => commands.entity(e).despawn(),
        }
    }
    for (&pack, pool) in &p.pools {
        for (i, it) in pool.slots.iter().enumerate() {
            let Some(it) = it else { continue };
            if seen.contains(&(pack, i)) {
                continue;
            }
            let k = if neopets_sim::items::is_coin(it.id) {
                0
            } else if it.charm {
                1
            } else {
                2
            };
            commands.spawn((ItemMarker(pack, i), Mesh3d(mesh.clone()), MeshMaterial3d(mats[k].clone()), Transform::from_translation(Vec3::from(it.position)), Visibility::Hidden));
        }
    }
}

/// A region character's drawn root, by character id.
#[derive(Component)]
struct CharacterRoot(u64);

/// Spawn, place, skin and relight the region characters of drawn worlds; despawn the rest.
#[allow(clippy::too_many_arguments)]
fn sync_characters(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    mut roots: Query<(Entity, &CharacterRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut skinned: Local<std::collections::HashMap<u64, Vec<(Handle<Mesh>, usize)>>>,
) {
    let Some(state) = state else { return };
    let p = &state.play;
    let live: std::collections::HashSet<u64> = p.characters.iter().map(|c| c.id).collect();
    for (e, root, mut t, mut vis) in &mut roots {
        match p.characters.iter().find(|c| c.id == root.0) {
            None => {
                commands.entity(e).despawn();
                skinned.remove(&root.0);
            }
            Some(c) => {
                t.translation = Vec3::from(c.position);
                t.rotation = Quat::from_rotation_y(c.facing[0].atan2(c.facing[2]));
                *vis = if p.pack_drawn(c.pack) { Visibility::Visible } else { Visibility::Hidden };
            }
        }
    }
    skinned.retain(|id, _| live.contains(id));
    for c in &p.characters {
        let rotation = Quat::from_rotation_y(c.facing[0].atan2(c.facing[2]));
        if !skinned.contains_key(&c.id) {
            let parts = part_assets(&c.model, &mut meshes, &mut materials, &mut images);
            let mut list = Vec::new();
            commands
                .spawn((Transform::from_translation(Vec3::from(c.position)).with_rotation(rotation), Visibility::Visible, CharacterRoot(c.id)))
                .with_children(|ch| {
                    for (i, mesh, material) in parts {
                        if !c.model.parts[i].weights.is_empty() {
                            list.push((mesh.clone(), i));
                        }
                        ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    }
                });
            skinned.insert(c.id, list);
        }
        if !p.pack_drawn(c.pack) {
            continue;
        }
        let matrices = c.anim.skin_matrices();
        for (handle, i) in &skinned[&c.id] {
            let part = &c.model.parts[*i];
            let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
            let colours = relight(part, &nrm, rotation);
            if let Some(mut mesh) = meshes.get_mut(handle) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
                if let Some(col) = colours {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                }
            }
        }
    }
}

/// Bevy meshes and materials for every part of a model: (part index, mesh, material). Parts are
/// unlit and double-sided; textured parts mask alpha at 0.5.
fn part_assets(
    m: &neopets_data::model::ModelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Vec<(usize, Handle<Mesh>, Handle<StandardMaterial>)> {
    use bevy::mesh::{Indices, PrimitiveTopology};
    let handles: Vec<Handle<StandardMaterial>> = m
        .textures
        .iter()
        .map(|t| {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(model_texture(t))),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                cull_mode: None,
                ..default()
            })
        })
        .collect();
    let untextured = materials.add(StandardMaterial {
        unlit: true,
        cull_mode: None,
        ..default()
    });
    m.parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            // The GS modulates in gamma space; Bevy blends vertex colour in linear space.
            let colors: Vec<[f32; 4]> = part
                .colors
                .iter()
                .map(|c| [c[0].powf(2.2), c[1].powf(2.2), c[2].powf(2.2), c[3].min(1.0)])
                .collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, part.uvs.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_indices(Indices::U32(part.indices.clone()));
            let material = part.texture.map_or(untextured.clone(), |t| handles[t].clone());
            (i, meshes.add(mesh), material)
        })
        .collect()
}

/// Relight a lit part from its skinned normals turned into world space by `rotation`, with the
/// scene default lights (SPEC-0046), as linear vertex colours. None for unlit parts.
fn relight(part: &neopets_data::model::MeshPart, normals: &[[f32; 3]], rotation: Quat) -> Option<Vec<[f32; 4]>> {
    if !part.lit {
        return None;
    }
    let world: Vec<[f32; 3]> = normals.iter().map(|n| (rotation * Vec3::from(*n)).to_array()).collect();
    let lights = neopets_data::light::Lights::scene_default();
    Some(
        neopets_data::model::lit_colours(&part.raw_colors, &world, &lights)
            .into_iter()
            .map(|c| [c[0].powf(2.2), c[1].powf(2.2), c[2].powf(2.2), c[3].min(1.0)])
            .collect(),
    )
}

fn choose_action(keys: Res<ButtonInput<KeyCode>>, mut models: ResMut<ModelView>) {
    let Some(hero) = models.hero.as_mut() else { return };
    let a = hero.action();
    if keys.just_pressed(KeyCode::Equal) {
        hero.play((a + 1) % character::ACTIONS);
    } else if keys.just_pressed(KeyCode::Minus) {
        hero.play((a + character::ACTIONS - 1) % character::ACTIONS);
    }
}

/// Re-skin the hero's layout-A parts from its current pose (CPU skinning, every frame).
fn skin_hero(models: Res<ModelView>, mut meshes: ResMut<Assets<Mesh>>) {
    let (Some(hero), Some(data)) = (&models.hero, &models.data) else { return };
    if !models.showing() {
        return;
    }
    let matrices = hero.skin_matrices();
    for (handle, i) in &models.skinned {
        let part = &data.parts[*i];
        let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
        let colours = relight(part, &nrm, Quat::IDENTITY);
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
            if let Some(c) = colours {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
            }
        }
    }
}

/// Diagnostic (`--play-trace <ticks>`): run play mode without a window, walking forward along
/// the spawn facing (and jumping every 120 ticks with `--autojump`), and print each tick.
#[allow(clippy::too_many_arguments)]
fn play_trace(
    archive: &mut Archive,
    pack: u32,
    ticks: u64,
    jump: bool,
    to: Option<[f32; 2]>,
    flags: &[u16],
    seed: Option<u32>,
    give: &[u16],
    start: Option<[f32; 2]>,
    guard: bool,
) {
    let Archive { disc, index } = archive;
    let mut play = match play::Play::load(disc, index, pack, PLAY_OUTFIT) {
        Ok(play) => play,
        Err(e) => {
            eprintln!("Play mode could not start: {e}");
            std::process::exit(1)
        }
    };
    // `--seed N`: a fixed seed for the random stream (SPEC-0008 developer option).
    if let Some(n) = seed {
        play.rng = neopets_sim::rng::Rng::seeded(n);
    }
    // `--give N` (repeatable): a diagnostic weapon or shield equipped at the start; `--start
    // x,z` places the hero.
    for &id in give {
        play.give(id);
    }
    if let Some([x, z]) = start {
        play.place_at(x, z);
    }
    // `--flag N`: diagnostic story flags set at the start.
    for &f in flags {
        play.state.set(f, true);
    }
    let mut streamer = match disc.reopen() {
        Ok(second) => Streamer::start(second, index.clone(), play.boot.clone()),
        Err(e) => {
            eprintln!("No loader thread: {e}");
            std::process::exit(1)
        }
    };
    println!("Grid bounds: {:?}; nearest swap point {:?}", play.bounds(), play.nearest_swap_point());
    for q in [[-1840.4f32, 48.8, 1125.2], [-1854.0, 49.0, 1144.0], [-1845.75, 49.7, 1143.8]] {
        let inside: Vec<usize> = (0..play.world.navmeshes.len()).filter(|&i| play.world.navmeshes[i].locate_full(q).is_some()).collect();
        println!("Meshes containing {q:?}: {inside:?}");
    }
    for (k, pl) in play.world.planes.iter().enumerate() {
        let g = play.world.ground_at(pl.p[0] + 0.5 * pl.w * pl.d[0], pl.p[2] + 0.5 * pl.w * pl.d[2], pl.p[1] + 10.0);
        println!("Door {k}: P {:?} F {:?} D {:?} Q {:?} w {} h {}; ground below its middle {g:?}", pl.p, pl.f, pl.d, pl.q, pl.w, pl.h);
    }
    // `--trace-to swap` follows a searched route to the nearest reachable swap point.
    let mut route: Vec<[f32; 2]> = Vec::new();
    let mut rerouted = false;
    let mut last_door = String::new();
    let mut hops = 0;
    let mut last_hit = String::new();
    if to.is_some_and(|t| t[0].is_nan()) {
        let goal = match to.map(|t| t[1]) {
            Some(g) if g == 0.0 => play::Goal::NewNeighbour,
            Some(g) if g == 1.0 => play::Goal::Door,
            _ => play::Goal::Swap,
        };
        route = if to.is_some_and(|t| t[1] >= 2.0) {
            Vec::new()
        } else if goal == play::Goal::Door {
            play.stand_before_door(0).map(|m| vec![m]).unwrap_or_default()
        } else {
            play.route_to(400_000, goal).unwrap_or_default()
        };
        println!("Route to a swap point: {} steps, ending at {:?}", route.len(), route.last());
    }
    let mut last_hurt = String::new();
    let mut last_pickup = String::new();
    let mut statuses_shown: Vec<usize> = Vec::new();
    for _ in 0..ticks {
        // The trace waits for region loads (characters) before the next tick, up to a minute,
        // so that runs do not depend on the loader's speed.
        let waited = std::time::Instant::now();
        while streamer.regions_out > 0 && waited.elapsed().as_secs() < 60 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            for line in streamer.pump(&mut play) {
                println!("      {line}");
            }
        }
        // `--trace-to x,z` steers the stick toward a point through the camera's rows.
        let (mut forward, mut side) = (0.7, 0.0);
        // After a swap along a `swap` route, go on to a cell that names a new neighbour.
        if play.swaps > 0 && to.is_some_and(|t| t[1].is_nan()) && route.len() <= 1 && !rerouted {
            rerouted = true;
            route = play.route_to(400_000, play::Goal::NewNeighbour).unwrap_or_default();
            println!("      rerouted on tick {} toward a new neighbour: {} steps", play.ticks, route.len());
        }
        while route.len() > 1 && {
            let p = play.loco.position();
            (route[0][0] - p[0]).powi(2) + (route[0][1] - p[2]).powi(2) < 4.0
        } {
            route.remove(0);
        }
        let herd = to.is_some_and(|t| t[1] == 2.0);
        let fight = to.is_some_and(|t| t[1] == 3.0).then(|| play.fight_goal()).flatten();
        let goal = if herd {
            play.herd_goal()
        } else if let Some((g, close)) = fight {
            (!close).then_some(g)
        } else {
            route.first().copied().or(to.filter(|t| !t[0].is_nan()))
        };
        if let Some([tx, tz]) = goal.filter(|g| {
            let p = play.loco.position();
            route.len() > 1 || to.is_some_and(|t| t[0].is_nan()) || (g[0] - p[0]).hypot(g[1] - p[2]) > 2.0
        }) {
            let p = play.loco.position();
            let (f, l) = play.camera.rows();
            let (dx, dz) = (tx - p[0], tz - p[2]);
            let len = (dx * dx + dz * dz).sqrt().max(1e-6);
            forward = 0.7 * (dx * f[0] + dz * f[2]) / len / (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
            side = 0.7 * (dx * l[0] + dz * l[2]) / len;
        }
        let input = neopets_sim::hero::HeroInput {
            forward,
            side,
            square: play.prompt_shown.is_some() || play.talk_prompt,
            guard: guard && fight.is_some_and(|f| f.1),
            // With `--autojump` the trace also presses attack every 120 ticks for 10 ticks.
            attack_held: (jump || fight.is_some_and(|f| f.1)) && (30..40).contains(&(play.ticks % 120)),
            attack: (jump || fight.is_some_and(|f| f.1)) && play.ticks % 120 == 30,
            // On the game-over page the trace taps Cross every 60 ticks.
            jump: (jump && play.ticks % 240 == 0 && play.ticks > 0) || (play.death.page.is_some() && play.ticks % 60 == 0),
            ..Default::default()
        };
        // In a fight below half health the trace drinks: it opens the quick panel, selects the
        // first entry it owns and presses Cross (or closes the panel with Triangle).
        let mut input = input;
        if fight.is_some() && play.hero_hp > 0 && play.hero_hp < play.hero_max / 2 {
            let owned = (0..4).find(|&k| {
                let id = play.quick.entries[k];
                id >= 0 && play.inventory.get(&(id as u16)).is_some_and(|&n| n > 0)
            });
            if !play.quick.open {
                play.panel_latch.up |= play.ticks % 20 == 0;
            } else {
                match owned {
                    Some(k) if play.quick.selection == k => input.jump = true,
                    Some(0) => play.panel_latch.right = true,
                    Some(1) => play.panel_latch.up = true,
                    Some(2) => play.panel_latch.left = true,
                    Some(_) => play.panel_latch.down = true,
                    None => play.panel_latch.triangle = true,
                }
            }
        }
        play.tick(&input, neopets_sim::camera::CameraPad::default());
        // Loads go through the same background loader as the window; the trace does not wait.
        for line in streamer.pump(&mut play) {
            println!("      {line}");
        }
        if let Some((line, at)) = play.message {
            if at == play.ticks {
                let text = play.world.lines.line(line as usize).map(|l| format!("{}: {}", play.talk_partner.as_deref().unwrap_or(&l.speaker), l.text));
                println!("      LevelMsg line {line} on tick {at}: {text:?}");
            }
        }
        let now: Vec<usize> = (0..14).filter(|&i| play.statuses.active(i)).collect();
        if now != statuses_shown {
            println!("      hero statuses {now:?} on tick {} (hp {}, mp {})", play.ticks, play.hero_hp, play.hero_mp);
            statuses_shown = now;
        }
        if !play.last_pickup.is_empty() && play.last_pickup != last_pickup {
            last_pickup = play.last_pickup.clone();
            println!("      {last_pickup}");
        }
        if !play.last_hurt.is_empty() && play.last_hurt != last_hurt {
            last_hurt = play.last_hurt.clone();
            println!("      hero {last_hurt} on tick {}", play.ticks);
        }
        if !play.last_hit.is_empty() && play.last_hit != last_hit {
            last_hit = play.last_hit.clone();
            println!("      {last_hit} on tick {}", play.ticks);
        }
        if !play.last_door.is_empty() && play.last_door != last_door {
            last_door = play.last_door.clone();
            println!("      {} on tick {}", last_door, play.ticks);
            // A door trace goes on through door 0 of each world it arrives in, twice more.
            let arrived = last_door.starts_with("landed") || last_door.starts_with("entered") || last_door.contains("into the") || last_door.contains("left the");
            if arrived && to.is_some_and(|t| t[1] == 1.0) && hops < 2 {
                hops += 1;
                route = play.stand_before_door(0).map(|m| vec![m]).unwrap_or_default();
                println!("      hop {hops}: standing before door 0 of {}", play.world.pack);
            }
        }
        let (p, b) = (play.loco.position(), &play.loco.body);
        println!(
            "{:5} pos ({:9.3} {:8.3} {:9.3}) state {:2} phase {:2} action {:3} gap {:8.3} vy {:7.3} n ({:.2} {:.2} {:.2})",
            play.ticks, p[0], p[1], p[2], play.loco.state, play.loco.phase, play.loco.action(), b.gap, b.velocity[1],
            b.normal[0], b.normal[1], b.normal[2]
        );
        if play.ticks % 60 == 0 {
            println!("      {}", play.status_camera());
            println!("      {}", play.status_worlds());
        }
    }
}

fn setup_play(mut commands: Commands, request: Option<Res<PlayRequest>>, archive: Option<ResMut<Archive>>) {
    let (Some(request), Some(mut archive)) = (request, archive) else { return };
    let Archive { disc, index } = &mut *archive;
    match play::Play::load(disc, index, request.0, PLAY_OUTFIT) {
        Ok(mut play) => {
            // `--look x,z`: diagnostic start facing.
            if let Some([x, z]) = request.1 {
                play.face_point(x, z);
            }
            // `--give N`: diagnostic equipment (no inventory yet).
            for &id in &request.2 {
                play.give(id);
            }
            // The HUD's art: the front-end database and texture sheet of resource 532.
            match index.read(disc, neopets_data::frontend::RESOURCE).and_then(|b| neopets_data::frontend::Frontend::parse(&b)) {
                Ok(f) => {
                    if let Some(art) = hud_view::HudArt::new(f) {
                        commands.insert_resource(art);
                    }
                }
                Err(e) => error!("No HUD art: {e}"),
            }
            match disc.reopen() {
                Ok(second) => commands.insert_resource(Streamer::start(second, index.clone(), play.boot.clone())),
                Err(e) => error!("No loader thread, streaming is off: {e}"),
            }
            commands.insert_resource(PlayState { play, skinned: Vec::new() })
        }
        Err(e) => error!("Play mode could not start: {e}"),
    }
}

/// Spawn the hero once, then each frame place it, skin it and keep the camera on it.
#[allow(clippy::too_many_arguments)]
fn play_frame(
    mut commands: Commands,
    play: Option<ResMut<PlayState>>,
    mut heroes: Query<&mut Transform, (With<HeroRoot>, Without<Orbit>)>,
    mut cameras: Query<(&mut Orbit, &mut Transform), Without<HeroRoot>>,
    mut projections: Query<&mut Projection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut state) = play else { return };
    let p = state.play.loco.position();
    let f = state.play.loco.facing;
    // Characters face +Z in model space (SPEC-0004); turn that onto the facing.
    let rotation = Quat::from_rotation_y(f[0].atan2(f[2]));
    if !state.play.spawned {
        state.play.spawned = true;
        let parts = part_assets(&state.play.model, &mut meshes, &mut materials, &mut images);
        let mut skinned = Vec::new();
        commands
            .spawn((Transform::from_translation(Vec3::from(p)).with_rotation(rotation), Visibility::Visible, HeroRoot))
            .with_children(|c| {
                for (i, mesh, material) in parts {
                    if !state.play.model.parts[i].weights.is_empty() {
                        skinned.push((mesh.clone(), i));
                    }
                    c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                }
            });
        state.skinned = skinned;
        // SPEC-0027 Projection: 50.2 degrees vertical (S 512 on 480 lines), near 1.0. The far
        // plane is SPEC-0046's widest (10560) so the sky dome is kept.
        for mut projection in &mut projections {
            *projection = Projection::Perspective(PerspectiveProjection {
                fov: neopets_sim::camera::vertical_fov(),
                near: neopets_sim::camera::NEAR,
                far: 10_560.0,
                ..default()
            });
        }
    }
    for mut t in &mut heroes {
        t.translation = Vec3::from(p);
        t.rotation = rotation;
    }
    let matrices = state.play.hero.skin_matrices();
    for (handle, i) in &state.skinned {
        let part = &state.play.model.parts[*i];
        let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
        let colours = relight(part, &nrm, rotation);
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
            if let Some(c) = colours {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
            }
        }
    }
    // The follow camera owns the view in play mode: it looks from the eye at the focus.
    let (eye, focus) = (Vec3::from(state.play.camera.eye), Vec3::from(state.play.camera.focus));
    for (_, mut transform) in &mut cameras {
        *transform = Transform::from_translation(eye).looking_at(focus, Vec3::Y);
    }
}
