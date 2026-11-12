//! Display settings (a feature of this build; the original has no display options, SPEC-0035
//! Behavior 3): window mode, resolution, aspect ratio, render scale and the upscale filter,
//! changed on the Options menu's Display page and kept in a settings file.
//!
//! The 3D scene is drawn into an offscreen image the size of the game viewport (the window
//! fitted to the chosen aspect ratio) times the render scale, and a window camera shows that
//! image in the viewport, scaled with a sharp or smooth filter. Menus and the HUD draw on top at
//! the window's full resolution.

use bevy::camera::RenderTarget;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::{MonitorSelection, PrimaryWindow, VideoModeSelection, WindowMode};

/// Where the settings are kept (local, not committed).
pub const SETTINGS_FILE: &str = "private/display.txt";

/// Render scales in percent of the viewport.
pub const SCALES: [u32; 8] = [50, 67, 75, 85, 100, 125, 150, 200];
/// Aspect ratios: a name and the width over height (None fills the window).
pub const ASPECTS: [(&str, Option<f32>); 5] = [("Fill window", None), ("4:3", Some(4.0 / 3.0)), ("16:9", Some(16.0 / 9.0)), ("16:10", Some(16.0 / 10.0)), ("21:9", Some(21.0 / 9.0))];
const MODES: [&str; 3] = ["Windowed", "Borderless", "Fullscreen"];
/// Window sizes offered when the monitor lists no video modes.
const FALLBACK_SIZES: [(u32, u32); 8] = [(3840, 2160), (2560, 1440), (1920, 1200), (1920, 1080), (1600, 900), (1280, 960), (1280, 720), (1024, 768)];
/// The Display page's rows.
pub const ROWS: usize = 22;
/// The rows on the Gameplay page, which shares the Display page's window and settings file; the
/// other rows are the Display page's.
pub const GAMEPLAY_ROWS: [usize; 4] = [17, 18, 19, 20];
/// The rows of the Display page (false) or the Gameplay page (true).
pub fn page_rows(gameplay: bool) -> Vec<usize> {
    (0..ROWS).filter(|k| GAMEPLAY_ROWS.contains(k) == gameplay).collect()
}
/// Character shadows: off, the original volumes as the game draws them (blocky hulls, a drop of
/// 8 of 255), or this build's detailed ones (each character's own shape, cast by the sun).
pub const CHAR_SHADOWS: [&str; 3] = ["Off", "Original", "Detailed"];

/// The sun shadow quality (shadow_view.rs: the map size and cascades of each level).
pub const SHADOW_QUALITY: [&str; 4] = ["Very Low", "Low", "Medium", "High"];
/// How far from the camera the sun's shadows reach, in world units (shadow_view.rs).
pub const SHADOW_DISTANCES: [f32; 5] = [50.0, 100.0, 150.0, 200.0, 300.0];
/// The frame rate limits, frames per second; 0 is uncapped.
pub const FPS_LIMITS: [u32; 4] = [30, 60, 120, 0];

/// The graphics presets (`DisplaySettings::apply_preset`).
pub const PRESETS: [&str; 3] = ["Low", "Medium", "High"];

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct DisplaySettings {
    /// 0 windowed, 1 borderless fullscreen, 2 exclusive fullscreen.
    pub mode: usize,
    /// The window (or exclusive fullscreen) size in physical pixels.
    pub size: (u32, u32),
    /// Index into ASPECTS.
    pub aspect: usize,
    /// Index into SCALES.
    pub scale: usize,
    /// Smooth (linear) or sharp (nearest) upscaling.
    pub smooth: bool,
    /// Graphics options (graphics.rs): anti-aliasing (index into graphics::AA), sharper
    /// textures at a distance, bloom and colour grade (indices), ambient occlusion.
    pub aa: usize,
    pub sharp_textures: bool,
    pub bloom: usize,
    pub colour: usize,
    pub ao: bool,
    /// Distant foliage as the original draws it (SPEC-0060: far sprites, fades out by distance;
    /// SPEC-0046: instanced meshes hidden past their draw distance) instead of full models at
    /// every distance (this build's default).
    pub original_foliage: bool,
    /// Character shadows (index into CHAR_SHADOWS; SPEC-0046 Character shadow volumes).
    pub char_shadows: usize,
    /// Shadows cast by everything (trees, buildings, props): this build's option, not in the
    /// original.
    pub world_shadows: bool,
    /// The sun shadow quality (index into SHADOW_QUALITY): this build's option.
    pub shadow_quality: usize,
    /// How far the sun's shadows reach (index into SHADOW_DISTANCES): this build's option.
    pub shadow_distance: usize,
    /// The frame rate limit (index into FPS_LIMITS): this build's option.
    pub fps: usize,
    /// Movie cutscenes played (on) or passed over at once, as if skipped (this build's testing
    /// option, not in the original).
    pub movies: bool,
    /// Button pictures: the game's PlayStation icons (default) or Xbox-style A, B, X and Y
    /// (button_style.rs; this build's option).
    pub xbox_buttons: bool,
    /// The HUD's D-pad hint drawn (default) or hidden (this build's option, not in the original).
    pub dpad_hud: bool,
    /// Menu pictures carry their edges out to the window's sides (this build's option; off draws the original 512 x 448 frame).
    pub menu_edges: bool,
    /// The neighbouring world drawn in full (this build's default) or gated by the disc's zone
    /// tables as the original draws it (SPEC-0006 Zones), which hides some of its pieces from
    /// parts of the current world (user reports 112, 114 and 116).
    pub original_neighbour: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self { mode: 0, size: (1000, 600), aspect: 0, scale: 4, smooth: true, aa: 2, sharp_textures: true, bloom: 1, colour: 1, ao: true, original_foliage: false, char_shadows: 1, world_shadows: false, shadow_quality: 2, shadow_distance: 2, fps: 2, movies: true, xbox_buttons: false, dpad_hud: true, menu_edges: true, original_neighbour: false }
    }
}

impl DisplaySettings {
    /// Parse "key = value" lines; unknown keys and bad values keep the defaults.
    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let v = v.trim();
            match k.trim() {
                "mode" => s.mode = v.parse().unwrap_or(s.mode).min(MODES.len() - 1),
                "size" => {
                    if let Some((w, h)) = v.split_once('x').and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?))) {
                        s.size = (w, h);
                    }
                }
                "aspect" => s.aspect = v.parse().unwrap_or(s.aspect).min(ASPECTS.len() - 1),
                "scale" => {
                    if let Some(i) = v.parse::<u32>().ok().and_then(|p| SCALES.iter().position(|&x| x == p)) {
                        s.scale = i;
                    }
                }
                "filter" => s.smooth = v != "sharp",
                "antialiasing" => s.aa = v.parse().unwrap_or(s.aa).min(crate::graphics::AA.len() - 1),
                "sharp textures" => s.sharp_textures = v == "true",
                "bloom" => s.bloom = v.parse().unwrap_or(s.bloom).min(crate::graphics::BLOOM.len() - 1),
                "colour" => s.colour = v.parse().unwrap_or(s.colour).min(crate::graphics::COLOUR.len() - 1),
                "ambient occlusion" => s.ao = v == "true",
                "original foliage" => s.original_foliage = v == "true",
                "character shadows" => s.char_shadows = v.parse().unwrap_or(s.char_shadows).min(CHAR_SHADOWS.len() - 1),
                "world shadows" => s.world_shadows = v == "true",
                "shadow level" => s.shadow_quality = v.parse().unwrap_or(s.shadow_quality).min(SHADOW_QUALITY.len() - 1),
                // The older key counted from Low, before Very Low was added.
                "shadow quality" => s.shadow_quality = v.parse::<usize>().map_or(s.shadow_quality, |q| q + 1).min(SHADOW_QUALITY.len() - 1),
                "shadow distance" => s.shadow_distance = v.parse().unwrap_or(s.shadow_distance).min(SHADOW_DISTANCES.len() - 1),
                "frame rate" => s.fps = v.parse().unwrap_or(s.fps).min(FPS_LIMITS.len() - 1),
                "movies" => s.movies = v != "false",
                "button pictures" => s.xbox_buttons = v == "xbox",
                "dpad hud" => s.dpad_hud = v != "false",
                "menu edges" => s.menu_edges = v != "false",
                "original neighbour" => s.original_neighbour = v == "true",
                _ => {}
            }
        }
        s
    }

    pub fn text(&self) -> String {
        format!(
            "mode = {}\nsize = {}x{}\naspect = {}\nscale = {}\nfilter = {}\nantialiasing = {}\nsharp textures = {}\nbloom = {}\ncolour = {}\nambient occlusion = {}\noriginal foliage = {}\ncharacter shadows = {}\nworld shadows = {}\nshadow level = {}\nshadow distance = {}\nframe rate = {}\nmovies = {}\nbutton pictures = {}\ndpad hud = {}\nmenu edges = {}\noriginal neighbour = {}\n",
            self.mode,
            self.size.0,
            self.size.1,
            self.aspect,
            SCALES[self.scale],
            if self.smooth { "smooth" } else { "sharp" },
            self.aa,
            self.sharp_textures,
            self.bloom,
            self.colour,
            self.ao,
            self.original_foliage,
            self.char_shadows,
            self.world_shadows,
            self.shadow_quality,
            self.shadow_distance,
            self.fps,
            self.movies,
            if self.xbox_buttons { "xbox" } else { "playstation" },
            self.dpad_hud,
            self.menu_edges,
            self.original_neighbour
        )
    }

    pub fn load() -> Self {
        let mut s = std::fs::read_to_string(SETTINGS_FILE).map(|t| Self::parse(&t)).unwrap_or_default();
        // NEOPETS_GFX="aa,textures,bloom,colour,ao[,foliage]" (0/1 for the switches): the graphics options
        // for this run (diagnostic, for screenshot comparisons; not saved unless changed).
        // NEOPETS_SHADOWS="char,world[,quality]": the shadow settings for this run (diagnostic).
        if let Some(v) = std::env::var("NEOPETS_SHADOWS").ok() {
            let n: Vec<usize> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
            if let Some(&c) = n.first() {
                s.char_shadows = c.min(CHAR_SHADOWS.len() - 1);
            }
            if let Some(&w) = n.get(1) {
                s.world_shadows = w != 0;
            }
            // An optional third value: the shadow quality (0 Very Low, 1 Low, 2 Medium, 3 High);
            // a fourth: the shadow distance index.
            if let Some(&q) = n.get(2) {
                s.shadow_quality = q.min(SHADOW_QUALITY.len() - 1);
            }
            if let Some(&d) = n.get(3) {
                s.shadow_distance = d.min(SHADOW_DISTANCES.len() - 1);
            }
        }
        // NEOPETS_FPS=index: the frame rate limit for this run (diagnostic).
        if let Some(i) = std::env::var("NEOPETS_FPS").ok().and_then(|v| v.trim().parse::<usize>().ok()) {
            s.fps = i.min(FPS_LIMITS.len() - 1);
        }
        // NEOPETS_SCALE=percent: the render scale for this run (diagnostic).
        if let Some(i) = std::env::var("NEOPETS_SCALE").ok().and_then(|v| v.trim().parse::<u32>().ok()).and_then(|p| SCALES.iter().position(|&x| x == p)) {
            s.scale = i;
        }
        // NEOPETS_BUTTONS=xbox: the Xbox button pictures for this run (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_BUTTONS") {
            s.xbox_buttons = v.trim() == "xbox";
        }
        // NEOPETS_MOVIES=0 or 1: Movie cutscenes for this run (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_MOVIES") {
            s.movies = v.trim() != "0";
        }
        if let Some(v) = std::env::var("NEOPETS_GFX").ok() {
            let n: Vec<usize> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
            if let [aa, tex, bloom, colour, ao, ref rest @ ..] = n[..] {
                (s.aa, s.sharp_textures, s.bloom, s.colour, s.ao) = (aa.min(3), tex != 0, bloom.min(2), colour.min(2), ao != 0);
                // An optional sixth value: 1 for the original distant foliage.
                if let Some(&f) = rest.first() {
                    s.original_foliage = f != 0;
                }
            }
        }
        s
    }

    pub fn save(&self) {
        if let Some(dir) = std::path::Path::new(SETTINGS_FILE).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(SETTINGS_FILE, self.text()) {
            warn!("display settings not saved: {e}");
        }
    }

    /// A graphics preset's options: (anti-aliasing, bloom, ambient occlusion, world shadows,
    /// shadow quality). Low turns off the costly passes (FXAA, no bloom, no ambient occlusion,
    /// no world shadows, Low shadows); Medium is the new-player look (SMAA, bloom, ambient
    /// occlusion, Medium shadows, no world shadows); High adds world shadows at High quality.
    /// The other options (colour, textures, foliage, character shadows) are left as they are.
    fn preset_values(p: usize) -> (usize, usize, bool, bool, usize) {
        match p {
            0 => (3, 0, false, false, 1),
            1 => (2, 1, true, false, 2),
            _ => (2, 1, true, true, 3),
        }
    }

    /// The preset the options match, if any (the Display page shows Custom otherwise).
    pub fn preset(&self) -> Option<usize> {
        (0..PRESETS.len()).find(|&p| Self::preset_values(p) == (self.aa, self.bloom, self.ao, self.world_shadows, self.shadow_quality))
    }

    pub fn apply_preset(&mut self, p: usize) {
        (self.aa, self.bloom, self.ao, self.world_shadows, self.shadow_quality) = Self::preset_values(p);
    }

    /// The rows as (name, value) for the Display page.
    pub fn rows(&self) -> [(String, String); ROWS] {
        [
            ("Window".into(), MODES[self.mode].into()),
            ("Resolution".into(), if self.mode == 1 { "Desktop".into() } else { format!("{} x {}", self.size.0, self.size.1) }),
            ("Aspect ratio".into(), ASPECTS[self.aspect].0.into()),
            ("Render scale".into(), format!("{}%", SCALES[self.scale])),
            ("Upscale filter".into(), if self.smooth { "Smooth".into() } else { "Sharp".into() }),
            ("Graphics preset".into(), self.preset().map_or("Custom", |p| PRESETS[p]).into()),
            ("Anti-aliasing".into(), crate::graphics::AA[self.aa].into()),
            ("Distant textures".into(), if self.sharp_textures { "Sharp".into() } else { "Original".into() }),
            ("Bloom".into(), crate::graphics::BLOOM[self.bloom].into()),
            ("Colour".into(), crate::graphics::COLOUR[self.colour].into()),
            ("Ambient occlusion".into(), if self.ao { "On".into() } else { "Off".into() }),
            ("Distant foliage".into(), if self.original_foliage { "Original".into() } else { "Full".into() }),
            ("Character shadows".into(), CHAR_SHADOWS[self.char_shadows].into()),
            ("World shadows".into(), if self.world_shadows { "On".into() } else { "Off".into() }),
            ("Shadow quality".into(), SHADOW_QUALITY[self.shadow_quality].into()),
            ("Shadow distance".into(), format!("{}", SHADOW_DISTANCES[self.shadow_distance])),
            ("Frame rate limit".into(), match FPS_LIMITS[self.fps] { 0 => "Uncapped".into(), n => format!("{n}") }),
            ("Movie cutscenes".into(), if self.movies { "On".into() } else { "Off".into() }),
            ("Button pictures".into(), if self.xbox_buttons { "Xbox".into() } else { "PlayStation".into() }),
            ("D-pad HUD".into(), if self.dpad_hud { "On".into() } else { "Off".into() }),
            ("Menu edges".into(), if self.menu_edges { "Extended".into() } else { "Original".into() }),
            ("Neighbour world".into(), if self.original_neighbour { "Original".into() } else { "Full".into() }),
        ]
    }

    /// Step row `row` by `delta` (Left -1, Right +1), wrapping; `sizes` are the resolutions on
    /// offer, largest first.
    pub fn change(&mut self, row: usize, delta: i32, sizes: &[(u32, u32)]) {
        let step = |i: usize, n: usize| (i as i32 + delta).rem_euclid(n as i32) as usize;
        match row {
            0 => self.mode = step(self.mode, MODES.len()),
            1 if !sizes.is_empty() => {
                // Larger first in the list, so Right (+1) moves to a smaller size: flip it. A size
                // not on the list steps to the nearest smaller (Right) or larger (Left) one.
                let area = |s: (u32, u32)| s.0 as u64 * s.1 as u64;
                self.size = match sizes.iter().position(|&s| s == self.size) {
                    Some(at) => sizes[(at as i32 - delta).rem_euclid(sizes.len() as i32) as usize],
                    None if delta > 0 => sizes.iter().copied().find(|&s| area(s) < area(self.size)).unwrap_or(sizes[0]),
                    None => sizes.iter().rev().copied().find(|&s| area(s) > area(self.size)).unwrap_or(sizes[sizes.len() - 1]),
                };
            }
            2 => self.aspect = step(self.aspect, ASPECTS.len()),
            3 => self.scale = (self.scale as i32 + delta).clamp(0, SCALES.len() as i32 - 1) as usize,
            4 => self.smooth = !self.smooth,
            5 => {
                let next = match self.preset() {
                    Some(p) => (p as i32 + delta).rem_euclid(PRESETS.len() as i32) as usize,
                    None if delta > 0 => 0,
                    None => PRESETS.len() - 1,
                };
                self.apply_preset(next);
            }
            6 => self.aa = step(self.aa, crate::graphics::AA.len()),
            7 => self.sharp_textures = !self.sharp_textures,
            8 => self.bloom = step(self.bloom, crate::graphics::BLOOM.len()),
            9 => self.colour = step(self.colour, crate::graphics::COLOUR.len()),
            10 => self.ao = !self.ao,
            11 => self.original_foliage = !self.original_foliage,
            12 => self.char_shadows = step(self.char_shadows, CHAR_SHADOWS.len()),
            13 => self.world_shadows = !self.world_shadows,
            14 => self.shadow_quality = step(self.shadow_quality, SHADOW_QUALITY.len()),
            15 => self.shadow_distance = step(self.shadow_distance, SHADOW_DISTANCES.len()),
            16 => self.fps = step(self.fps, FPS_LIMITS.len()),
            17 => self.movies = !self.movies,
            18 => self.xbox_buttons = !self.xbox_buttons,
            19 => self.dpad_hud = !self.dpad_hud,
            20 => self.menu_edges = !self.menu_edges,
            21 => self.original_neighbour = !self.original_neighbour,
            _ => {}
        }
    }
}

/// The frame rate limit: the frame waits out the rest of its time (the last millisecond by
/// spinning, as a sleep can overrun). Runs last in the frame; the game's ticks are timed apart
/// from it.
pub fn limit_frame(settings: Res<DisplaySettings>, mut windows: Query<&mut Window, With<PrimaryWindow>>, mut next: Local<Option<std::time::Instant>>) {
    let limit = FPS_LIMITS[settings.fps];
    // The limit is this timer alone, vsync off: measured exact at 30, 60 and 120, where vsync held
    // this laptop at about 30 whatever the limit (with its lid closed; not tested open).
    for mut w in &mut windows {
        if w.present_mode != bevy::window::PresentMode::AutoNoVsync {
            w.present_mode = bevy::window::PresentMode::AutoNoVsync;
        }
    }
    if limit == 0 || std::env::var_os("NEOPETS_PERF_NOVSYNC").is_some() {
        *next = None;
        return;
    }
    let period = std::time::Duration::from_secs_f64(1.0 / limit as f64);
    let now = std::time::Instant::now();
    let due = next.filter(|&d| d > now);
    if let Some(d) = due {
        if let Some(rest) = d.checked_duration_since(now + std::time::Duration::from_millis(1)) {
            std::thread::sleep(rest);
        }
        while std::time::Instant::now() < d {
            std::hint::spin_loop();
        }
    }
    *next = Some(due.unwrap_or(now) + period);
}

/// The game viewport inside a window of size (w, h): the largest rectangle of the aspect ratio,
/// centred (x, y, w, h), in the window's units.
pub fn viewport(w: f32, h: f32, aspect: Option<f32>) -> (f32, f32, f32, f32) {
    match aspect {
        None => (0.0, 0.0, w, h),
        Some(a) if w / h > a => {
            let vw = (h * a).round();
            (((w - vw) / 2.0).floor(), 0.0, vw, h)
        }
        Some(a) => {
            let vh = (w / a).round();
            (0.0, ((h - vh) / 2.0).floor(), w, vh)
        }
    }
}

/// The 3D camera's offscreen image and the UI node that shows it.
#[derive(Resource, Default)]
pub struct SceneTarget {
    image: Option<Handle<Image>>,
    size: (u32, u32),
    smooth: bool,
}

#[derive(Component)]
pub struct SceneView;

/// The window camera, which draws the scene image and the UI.
#[derive(Component)]
pub struct WindowCamera;

/// Spawn the window camera (the default UI camera) and the scene view node behind every UI.
pub fn setup(mut commands: Commands, mut grades: ResMut<Assets<crate::graphics::SceneGrade>>) {
    commands.spawn((
        Camera2d,
        Camera { order: 2, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        IsDefaultUiCamera,
        WindowCamera,
    ));
    // The 3D picture, drawn with the colour grade (graphics.rs).
    let grade = grades.add(crate::graphics::SceneGrade { grade: Default::default(), image: Handle::default() });
    commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, bevy::ui_render::prelude::MaterialNode(grade), GlobalZIndex(i32::MIN + 1), SceneView));
}

/// The resolutions on offer: the primary monitor's video mode sizes (largest first, at least
/// 640 x 480), else a fixed list.
pub fn sizes(monitors: &Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = monitors
        .iter()
        .flat_map(|m| m.video_modes.iter().map(|vm| (vm.physical_size.x, vm.physical_size.y)))
        .filter(|&(w, h)| w >= 640 && h >= 480)
        .collect();
    v.sort_by(|a, b| (b.0 * b.1).cmp(&(a.0 * a.1)).then(b.0.cmp(&a.0)));
    v.dedup();
    if v.is_empty() {
        v = FALLBACK_SIZES.to_vec();
    }
    v
}

/// Apply the Display page's changes from the menu flow, and save them.
pub fn take_requests(
    menu: Option<ResMut<crate::menu_view::Menu>>,
    mut settings: ResMut<DisplaySettings>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
) {
    let _span = crate::perf::span("take_requests");
    let Some(mut menu) = menu else { return };
    if menu.flow.display_requests.is_empty() {
        return;
    }
    let sizes = sizes(&monitors);
    for (row, delta) in std::mem::take(&mut menu.flow.display_requests) {
        settings.change(row, delta, &sizes);
    }
    settings.save();
}

/// Apply the window mode and size when the settings change.
pub fn apply_window(
    mut settings: ResMut<DisplaySettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut last: Local<Option<DisplaySettings>>,
) {
    let _span = crate::perf::span("apply_window");
    let Ok(mut window) = windows.single_mut() else { return };
    // The first run keeps the window as built unless a settings file asked for something; the
    // size then starts as the window's own size in physical pixels (a scaled display makes the
    // built 1000 x 600 larger).
    if last.is_none() {
        let default = DisplaySettings::default();
        // NEOPETS_WINDOW (diagnostic) keeps the window as built over the settings file.
        if settings.mode == default.mode && settings.size == default.size || std::env::var_os("NEOPETS_WINDOW").is_some() {
            settings.mode = 0;
            settings.size = (window.physical_width(), window.physical_height());
            *last = Some(settings.clone());
            return;
        }
    }
    let changed = last.as_ref().is_none_or(|l| l.mode != settings.mode || l.size != settings.size);
    *last = Some(settings.clone());
    if !changed {
        return;
    }
    let (w, h) = settings.size;
    window.mode = match settings.mode {
        1 => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        2 => {
            // Exclusive fullscreen needs one of the monitor's own modes: the highest refresh
            // rate at this size, else the current mode.
            let mode = monitors
                .iter()
                .flat_map(|m| m.video_modes.iter())
                .filter(|vm| vm.physical_size == UVec2::new(w, h))
                .max_by_key(|vm| (vm.refresh_rate_millihertz, vm.bit_depth))
                .copied();
            WindowMode::Fullscreen(MonitorSelection::Current, mode.map_or(VideoModeSelection::Current, VideoModeSelection::Specific))
        }
        _ => WindowMode::Windowed,
    };
    if settings.mode != 1 {
        window.resolution.set_physical_resolution(w, h);
    }
}

/// Each frame: fit the viewport to the window and the aspect ratio, keep the 3D camera's
/// image at the viewport times the render scale, and place the scene view over the viewport.
#[allow(clippy::too_many_arguments)]
pub fn sync_scene(
    mut commands: Commands,
    settings: Res<DisplaySettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<Entity, With<Camera3d>>,
    mut views: Query<(&mut Node, &bevy::ui_render::prelude::MaterialNode<crate::graphics::SceneGrade>), With<SceneView>>,
    mut grades: ResMut<Assets<crate::graphics::SceneGrade>>,
    mut images: ResMut<Assets<Image>>,
    mut target: ResMut<SceneTarget>,
) {
    let _span = crate::perf::span("sync_scene");
    let Ok(window) = windows.single() else { return };
    let (pw, ph) = (window.physical_width() as f32, window.physical_height() as f32);
    if pw < 1.0 || ph < 1.0 {
        return;
    }
    let (vx, vy, vw, vh) = viewport(pw, ph, ASPECTS[settings.aspect].1);
    let s = SCALES[settings.scale] as f32 / 100.0;
    let size = (((vw * s).round() as u32).clamp(16, 8192), ((vh * s).round() as u32).clamp(16, 8192));
    if target.image.is_none() || target.size != size || target.smooth != settings.smooth {
        let mut image = Image::new_target_texture(size.0, size.1, TextureFormat::Rgba8UnormSrgb, None);
        image.sampler = if settings.smooth { ImageSampler::linear() } else { ImageSampler::nearest() };
        let handle = images.add(image);
        if let Some(old) = target.image.replace(handle.clone()) {
            images.remove(&old);
        }
        target.size = size;
        target.smooth = settings.smooth;
        for e in &cameras {
            commands.entity(e).insert(RenderTarget::from(handle.clone()));
        }
        for (_, m) in &views {
            if let Some(mut g) = grades.get_mut(&m.0) {
                g.image = handle.clone();
            }
        }
    }
    // The node is in logical pixels.
    let f = window.scale_factor();
    for (mut node, _) in &mut views {
        let want = (px(vx / f), px(vy / f), px(vw / f), px(vh / f));
        if (node.left, node.top, node.width, node.height) != want {
            (node.left, node.top, node.width, node.height) = want;
        }
    }
}

/// The Display page's window (this build), in the bindings page's look: a row per setting with
/// its value between < and > buttons, a Back row, and a line about the focused setting. The
/// front end's Display page (menus.rs) keeps the cursor and the changes; the window shows them
/// and takes the mouse.
#[derive(Component)]
pub struct DispWindow;
#[derive(Component)]
pub struct DispTitle;
#[derive(Component)]
pub struct DispRow(usize);
#[derive(Component)]
pub struct DispLabel(usize);
#[derive(Component)]
pub struct DispValue(usize);
#[derive(Component)]
pub struct DispStep(usize, i32);
#[derive(Component)]
pub struct DispHelp;

const PANEL: Color = Color::srgb(0.05, 0.06, 0.10);
const ROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
const ROW_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.22);
const ARROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
const ARROW_HOVER: Color = Color::srgba(0.95, 0.75, 0.25, 0.65);
const GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.96);
const DIM: Color = Color::srgb(0.60, 0.63, 0.70);

/// What each setting does, shown under the rows.
const HELP: [&str; ROWS + 1] = [
    "Windowed, borderless or full screen.",
    "The window size, or the screen mode in full screen.",
    "The shape of the game picture; bars fill the rest of the window.",
    "How sharp the 3D world is drawn. Higher looks better and costs speed.",
    "How the world picture is scaled to the window: Sharp keeps pixels crisp, Smooth blends them.",
    "Sets the costly options at once. Low runs best on slower computers; High adds shadows from everything. Changing an option below shows Custom.",
    "Smooths jagged edges. SMAA also smooths leaves and fences; MSAA turns to SMAA while ambient occlusion is on.",
    "Keeps ground and walls crisp at a distance and at an angle, without shimmer.",
    "A soft glow around bright light, magic and sky.",
    "Vivid and Rich lift colour and contrast a little; Original is the game's own colour.",
    "Soft shade where walls meet floors and in corners.",
    "Full keeps trees, bushes and props in 3D at any distance; Original turns far trees into sprites, fades distant plants out and hides props past their draw distance, as the PS2 game.",
    "Original is the game's own faint, blocky hero shadow; Detailed casts every character's real shape from the area's main light.",
    "Shadows cast by everything, trees and buildings too, from the area's main light. Not in the original game. On Windows it runs fastest after a restart.",
    "How sharp the shadows from the area's main light are (World shadows and Detailed character shadows). Lower is faster.",
    "How far from the camera those shadows reach. Shorter is faster; shadows end at this distance.",
    "The most frames drawn each second; Uncapped draws as fast as it can. Exclusive full screen may tear. The game's speed does not change.",
    "Off passes over every movie cutscene at once, as if skipped (for testing). Not in the original game.",
    "The button pictures in boxes and menus: the game's PlayStation ones, or Xbox A, B, X and Y.",
    "The D-pad picture on the HUD during play. Off hides it; the D-pad still works.",
    "How menus fill a wide window. Extended carries each picture's edge colours out to the sides; Original keeps the 512 x 448 frame with bars.",
    "Full always draws the whole of the next area beside you; Original hides some of its pieces from parts of this area, as the PS2 game does.",
    "Back to Options.",
];

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_window(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: px(16),
                margin: UiRect::left(px(-300)),
                width: px(600),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(14)),
                row_gap: px(3),
                display: Display::None,
                ..default()
            },
            crate::ui_fit::FitToWindow::Centre,
            BackgroundColor(PANEL),
            GlobalZIndex(5003),
            DispWindow,
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("Display"), font(22.0), TextColor(GOLD), DispTitle));
            ui.spawn((Text::new("Up and down pick, left and right or a click change. Esc or B go back."), font(12.0), TextColor(DIM), Node { margin: UiRect::bottom(px(8)), ..default() }));
            for k in 0..=ROWS {
                let back = k == ROWS;
                ui.spawn((
                    Button,
                    Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(px(10), px(3)), min_height: px(25), margin: UiRect::top(px(if back { 8.0 } else { 0.0 })), ..default() },
                    BackgroundColor(ROW),
                    DispRow(k),
                ))
                .with_children(|row| {
                    row.spawn((Text::new(if back { "Back" } else { "" }), font(15.0), TextColor(TEXT), TextLayout::no_wrap(), DispLabel(k)));
                    if back {
                        return;
                    }
                    row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|r| {
                        let arrow = |r: &mut ChildSpawnerCommands, d: i32| {
                            r.spawn((Button, Node { width: px(24), height: px(22), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(ARROW), DispStep(k, d))).with_children(|b| {
                                b.spawn((Text::new(if d < 0 { "<" } else { ">" }), font(15.0), TextColor(TEXT)));
                            });
                        };
                        arrow(r, -1);
                        r.spawn(Node { width: px(170), justify_content: JustifyContent::Center, ..default() }).with_children(|v| {
                            v.spawn((Text::default(), font(15.0), TextColor(GOLD), TextLayout::no_wrap(), DispValue(k)));
                        });
                        arrow(r, 1);
                    });
                });
            }
            ui.spawn((Text::default(), font(13.0), TextColor(TEXT), Node { margin: UiRect::top(px(10)), min_height: px(18), ..default() }, DispHelp));
        });
}

/// The mouse on the Display window: hovering a row focuses it, a click on < or > changes it (a
/// click on the row steps it forward), and Back returns to Options.
pub fn window_mouse(menu: Option<ResMut<crate::menu_view::Menu>>, rows: Query<(&Interaction, &DispRow), Changed<Interaction>>, steps: Query<(&Interaction, &DispStep), Changed<Interaction>>) {
    let _span = crate::perf::span("window_mouse");
    let Some(mut menu) = menu else { return };
    let flow = &mut menu.flow;
    if flow.current != crate::menus::DISPLAY {
        return;
    }
    let mut stepped = false;
    for (i, s) in &steps {
        if *i == Interaction::Pressed {
            flow.display_focus = s.0;
            flow.display_requests.push((s.0, s.1));
            stepped = true;
        }
    }
    for (i, r) in &rows {
        match i {
            Interaction::Hovered => flow.display_focus = r.0,
            Interaction::Pressed if !stepped => {
                flow.display_focus = r.0;
                if r.0 == ROWS {
                    flow.display_back = true;
                } else {
                    flow.display_requests.push((r.0, 1));
                }
            }
            _ => {}
        }
    }
}

/// Show the Display window while the front end is on the Display page.
#[allow(clippy::type_complexity)]
pub fn show_window(
    menu: Option<Res<crate::menu_view::Menu>>,
    settings: Res<DisplaySettings>,
    mut windows: Query<&mut Node, With<DispWindow>>,
    mut rows: Query<(&Interaction, &DispRow, &mut BackgroundColor, &mut Node), (Without<DispStep>, Without<DispWindow>)>,
    mut steps: Query<(&Interaction, &mut BackgroundColor), (With<DispStep>, Without<DispRow>)>,
    mut labels: Query<(&DispLabel, &mut Text, &mut TextColor), (Without<DispValue>, Without<DispHelp>, Without<DispTitle>)>,
    mut values: Query<(&DispValue, &mut Text), (Without<DispLabel>, Without<DispHelp>, Without<DispTitle>)>,
    mut help: Query<&mut Text, (With<DispHelp>, Without<DispLabel>, Without<DispValue>, Without<DispTitle>)>,
    mut titles: Query<&mut Text, (With<DispTitle>, Without<DispHelp>, Without<DispLabel>, Without<DispValue>)>,
) {
    let _span = crate::perf::span("show_window");
    let flow = menu.as_ref().map(|m| &m.flow);
    let open = flow.is_some_and(|f| f.current == crate::menus::DISPLAY);
    for mut n in &mut windows {
        let d = if open { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
    }
    let Some(flow) = flow.filter(|_| open) else { return };
    let focus = flow.display_focus.min(ROWS);
    let shown = settings.rows();
    let page = page_rows(flow.display_gameplay);
    for (i, r, mut bg, mut node) in &mut rows {
        bg.0 = if r.0 == focus { ROW_CURSOR } else if *i == Interaction::Hovered { ROW_HOVER } else { ROW };
        let d = if r.0 == ROWS || page.contains(&r.0) { Display::Flex } else { Display::None };
        if node.display != d {
            node.display = d;
        }
    }
    let title = if flow.display_gameplay { "Gameplay" } else { "Display" };
    for mut t in &mut titles {
        if t.0 != title {
            t.0 = title.to_string();
        }
    }
    for (i, mut bg) in &mut steps {
        bg.0 = if *i == Interaction::None { ARROW } else { ARROW_HOVER };
    }
    for (l, mut t, mut c) in &mut labels {
        let s = shown.get(l.0).map_or("Back".to_string(), |r| r.0.clone());
        if t.0 != s {
            t.0 = s;
        }
        c.0 = if l.0 == focus { GOLD } else { TEXT };
    }
    for (v, mut t) in &mut values {
        let s = shown.get(v.0).map_or(String::new(), |r| r.1.clone());
        if t.0 != s {
            t.0 = s;
        }
    }
    for mut t in &mut help {
        if t.0 != HELP[focus] {
            t.0 = HELP[focus].to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_fits_the_aspect() {
        assert_eq!(viewport(1920.0, 1080.0, None), (0.0, 0.0, 1920.0, 1080.0));
        // The two pages share the rows between them.
        assert_eq!(page_rows(true), [17, 18, 19, 20]);
        assert_eq!(page_rows(false), (0..17).chain([21]).collect::<Vec<_>>());
        // 4:3 in a 16:9 window: pillarbox 1440 wide.
        assert_eq!(viewport(1920.0, 1080.0, Some(4.0 / 3.0)), (240.0, 0.0, 1440.0, 1080.0));
        // 21:9 in a 16:9 window: letterbox 823 high.
        assert_eq!(viewport(1920.0, 1080.0, Some(21.0 / 9.0)), (0.0, 128.0, 1920.0, 823.0));
    }

    #[test]
    fn graphics_presets() {
        let sizes = [(1920, 1080)];
        let mut s = DisplaySettings::default();
        // The new-player options are the Medium preset.
        assert_eq!((s.preset(), s.rows()[5].1.as_str()), (Some(1), "Medium"));
        s.change(5, 1, &sizes);
        assert_eq!((s.preset(), s.aa, s.bloom, s.ao, s.world_shadows, s.shadow_quality), (Some(2), 2, 1, true, true, 3));
        s.change(5, 1, &sizes);
        assert_eq!((s.preset(), s.aa, s.bloom, s.ao, s.world_shadows, s.shadow_quality), (Some(0), 3, 0, false, false, 1));
        // Changing an option below shows Custom; Right from Custom goes to Low.
        s.change(10, 1, &sizes);
        assert_eq!(s.rows()[5].1, "Custom");
        s.change(5, 1, &sizes);
        assert_eq!(s.preset(), Some(0));
    }

    #[test]
    fn settings_round_trip_and_rows() {
        let mut s = DisplaySettings::default();
        let sizes = [(1920, 1080), (1280, 720), (1000, 600)];
        s.change(0, 1, &sizes);
        s.change(0, 1, &sizes);
        assert_eq!(s.mode, 2);
        s.change(1, 1, &sizes);
        assert_eq!(s.size, (1280, 720), "Right steps to the next smaller size");
        s.change(2, -1, &sizes);
        assert_eq!(ASPECTS[s.aspect].0, "21:9", "Left wraps");
        s.change(3, 1, &sizes);
        s.change(3, 1, &sizes);
        assert_eq!(SCALES[s.scale], 150);
        s.change(4, 1, &sizes);
        assert!(!s.smooth);
        s.change(18, 1, &sizes);
        assert!(s.xbox_buttons && s.rows()[18].1 == "Xbox");
        s.change(17, 1, &sizes);
        assert!(!s.movies && s.rows()[17].1 == "Off", "Movie cutscenes off");
        s.change(19, 1, &sizes);
        assert!(!s.dpad_hud && s.rows()[19].1 == "Off", "D-pad HUD off");
        assert!(!DisplaySettings::parse(&s.text()).dpad_hud, "the choice is saved");
        // The frame rate limit wraps 120, Uncapped, 30; the distance steps down from 150 to 100.
        assert_eq!(s.rows()[16].1, "120");
        s.change(16, 1, &sizes);
        assert_eq!(s.rows()[16].1, "Uncapped");
        s.change(16, 1, &sizes);
        assert_eq!(s.rows()[16].1, "30");
        assert_eq!(s.rows()[15].1, "150");
        s.change(15, -1, &sizes);
        assert_eq!(s.rows()[15].1, "100");
        assert_eq!(s.rows()[14].1, "Medium");
        s.change(14, 1, &sizes);
        s.change(14, 1, &sizes);
        assert_eq!((s.shadow_quality, s.rows()[14].1.as_str()), (0, "Very Low"), "wraps to Very Low");
        assert_eq!(DisplaySettings::parse(&s.text()), s);
        // A settings file from before Very Low keeps its level.
        assert_eq!(SHADOW_QUALITY[DisplaySettings::parse("shadow quality = 1").shadow_quality], "Medium");
        assert_eq!(SHADOW_QUALITY[DisplaySettings::parse("shadow quality = 0").shadow_quality], "Low");
        assert_eq!(s.rows()[3].1, "150%");
        assert_eq!(DisplaySettings::parse("scale = 33\nmode = 9\n").mode, 2, "bad values clamp or keep");
        // A size off the list steps to its neighbours.
        let mut t = DisplaySettings { size: (2000, 1200), ..Default::default() };
        t.change(1, 1, &sizes);
        assert_eq!(t.size, (1920, 1080));
        let mut t = DisplaySettings { size: (1100, 650), ..Default::default() };
        t.change(1, -1, &sizes);
        assert_eq!(t.size, (1280, 720));
    }
}
