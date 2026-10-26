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
pub const ROWS: usize = 5;

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
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self { mode: 0, size: (1000, 600), aspect: 0, scale: 4, smooth: true }
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
                _ => {}
            }
        }
        s
    }

    pub fn text(&self) -> String {
        format!(
            "mode = {}\nsize = {}x{}\naspect = {}\nscale = {}\nfilter = {}\n",
            self.mode,
            self.size.0,
            self.size.1,
            self.aspect,
            SCALES[self.scale],
            if self.smooth { "smooth" } else { "sharp" }
        )
    }

    pub fn load() -> Self {
        std::fs::read_to_string(SETTINGS_FILE).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(dir) = std::path::Path::new(SETTINGS_FILE).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(SETTINGS_FILE, self.text()) {
            warn!("display settings not saved: {e}");
        }
    }

    /// The rows as (name, value) for the Display page.
    pub fn rows(&self) -> [(String, String); ROWS] {
        [
            ("Window".into(), MODES[self.mode].into()),
            ("Resolution".into(), if self.mode == 1 { "Desktop".into() } else { format!("{} x {}", self.size.0, self.size.1) }),
            ("Aspect ratio".into(), ASPECTS[self.aspect].0.into()),
            ("Render scale".into(), format!("{}%", SCALES[self.scale])),
            ("Upscale filter".into(), if self.smooth { "Smooth".into() } else { "Sharp".into() }),
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
            _ => {}
        }
    }
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
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera { order: 1, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        IsDefaultUiCamera,
        WindowCamera,
    ));
    commands.spawn((
        Node { position_type: PositionType::Absolute, ..default() },
        ImageNode::default(),
        GlobalZIndex(i32::MIN + 1),
        SceneView,
    ));
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
    let Ok(mut window) = windows.single_mut() else { return };
    // The first run keeps the window as built unless a settings file asked for something; the
    // size then starts as the window's own size in physical pixels (a scaled display makes the
    // built 1000 x 600 larger).
    if last.is_none() {
        let default = DisplaySettings::default();
        if settings.mode == default.mode && settings.size == default.size {
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
    mut views: Query<(&mut Node, &mut ImageNode), With<SceneView>>,
    mut images: ResMut<Assets<Image>>,
    mut target: ResMut<SceneTarget>,
) {
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
        for (_, mut img) in &mut views {
            img.image = handle.clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_fits_the_aspect() {
        assert_eq!(viewport(1920.0, 1080.0, None), (0.0, 0.0, 1920.0, 1080.0));
        // 4:3 in a 16:9 window: pillarbox 1440 wide.
        assert_eq!(viewport(1920.0, 1080.0, Some(4.0 / 3.0)), (240.0, 0.0, 1440.0, 1080.0));
        // 21:9 in a 16:9 window: letterbox 823 high.
        assert_eq!(viewport(1920.0, 1080.0, Some(21.0 / 9.0)), (0.0, 128.0, 1920.0, 823.0));
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
        assert_eq!(DisplaySettings::parse(&s.text()), s);
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
