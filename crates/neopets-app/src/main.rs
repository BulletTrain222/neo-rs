mod archive_view;
mod character;
mod controls;
mod orbit;
mod pattern;

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

/// Placeholder rate: the original's game-logic tick rate is not specified yet (request R-0007).
const SCAFFOLD_TICK_HZ: u32 = 60;
/// At most this many ticks per rendered frame; the rest are counted as dropped.
const MAX_CATCH_UP: u32 = 8;
/// Pixel-unit scroll (touchpads) per line-unit scroll step.
const PIXELS_PER_LINE: f32 = 40.0;

#[derive(Resource)]
struct DiscStatus(String);

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

fn rgba_image(width: u32, height: u32, rgba: Vec<u8>) -> Image {
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
    let mut start_resource = 0u32;
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
        } else if arg == "--hide-hud" {
            hide_hud = true;
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
    let driver = SimDriver {
        sim: Simulation::default(),
        clock: FixedStep::new(SCAFFOLD_TICK_HZ, MAX_CATCH_UP).expect("Positive tick rate"),
        input: PlayerInput::default(),
        last_ticks: 0,
    };
    let mut app = App::new();
    if let Some(archive) = archive {
        app.insert_resource(archive);
    }
    app.insert_resource(DiscStatus(status))
        .insert_resource(HudHidden(hide_hud))
        .insert_resource(driver)
        .insert_resource(Browser {
            selected: start_resource,
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
        .add_systems(Startup, (setup_viewer, setup_hud))
        .add_systems(
            Update,
            (
                (read_input, step_sim).chain(),
                (browse_resources, cycle_textures, toggle_view, choose_action, orbit_camera, toggle_hud),
                (show_sim, show_browser),
                (apply_texture, apply_model, skin_hero).chain(),
            )
                .chain(),
        );
    if smoke_test {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.add_systems(Update, smoke_exit);
    } else if let Some(path) = screenshot {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .insert_resource(ScreenshotPath(path))
            .add_systems(Update, screenshot_and_exit);
    }
    app.run();
}

#[derive(Resource)]
struct ScreenshotPath(String);

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
    if !*taken && elapsed >= 4.0 {
        *taken = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0.clone()));
    } else if elapsed >= 6.0 {
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

fn step_sim(time: Res<Time>, mut driver: ResMut<SimDriver>, mut models: ResMut<ModelView>) {
    let elapsed = time.delta().as_nanos().min(u64::MAX as u128) as u64;
    let SimDriver {
        sim, clock, input, ..
    } = &mut *driver;
    let ticks = sim.run_for(clock, elapsed, *input);
    driver.last_ticks = ticks;
    if let Some(hero) = models.hero.as_mut() {
        for _ in 0..ticks {
            hero.tick();
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
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
) {
    let dragging = buttons.any_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_LINE,
    };
    // While a model is shown, WASD and Q/E move the point the camera orbits (a fly camera).
    let held = |k: KeyCode| keys.pressed(k) as i32 as f32;
    let fly = if models.showing() {
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

fn show_sim(driver: Res<SimDriver>, mut texts: Query<&mut Text, With<SimText>>) {
    let d = &*driver;
    for mut text in &mut texts {
        text.0 = format!(
            "SIMULATION LOOP - diagnostic scaffold, not original game timing\n\
             Tick {} at a placeholder {} Hz | {} this frame | {} dropped | alpha {:.2}\n\
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
        let model = archive_view::model(disc, index, n);
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
    mut models: ResMut<ModelView>,
    old: Query<Entity, With<ModelPart>>,
    mut grid: Query<&mut Visibility, With<DiagnosticGrid>>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    use bevy::mesh::{Indices, PrimitiveTopology};
    if !models.dirty {
        return;
    }
    models.dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let grid_visibility = if models.showing() { Visibility::Hidden } else { Visibility::Inherited };
    for mut v in &mut grid {
        *v = grid_visibility;
    }
    let mut skinned = Vec::new();
    let framing = match (&models.data, models.show_model) {
        (Some(m), true) => {
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
            for (i, part) in m.parts.iter().enumerate() {
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
                let material = part
                    .texture
                    .map_or(untextured.clone(), |t| handles[t].clone());
                let handle = meshes.add(mesh);
                if !part.weights.is_empty() {
                    skinned.push((handle.clone(), i));
                }
                commands.spawn((
                    Mesh3d(handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ModelPart,
                ));
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
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
        }
    }
}
