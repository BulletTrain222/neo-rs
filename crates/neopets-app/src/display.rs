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
pub const ROWS: usize = 45;
/// The rows on the Gameplay page, which shares the Display page's window and settings file; the
/// other rows are the Display page's.
pub const GAMEPLAY_ROWS: [usize; 5] = [17, 18, 19, 20, ATTACK_TURN_ROW];
/// The Gameplay page's Attack turn row (this build's option).
pub const ATTACK_TURN_ROW: usize = 44;
/// The rows on the Multiplayer page (docs/MULTIPLAYER.md; this build's feature), in the order
/// shown: local co-op first, then the shared rules, then online.
pub const MULTI_ROWS: [usize; 18] = [COOP_ROW, PLAYERS_ROW, 26, PAD_ROWS[0], PAD_ROWS[1], PAD_ROWS[2], PAD_ROWS[3], 25, TOGETHER_ROW, TAGS_ROW, HOST_ROW, ADDRESS_ROW, JOIN_ROW, REMOVE_ROW, PASSWORD_ROW, UPNP_ROW, PORT_ROW, DOORS_ROW];
/// Rows no page shows any more (27, the Player 2 partner choice: every player starts as Tor and
/// switches on its own).
pub const RETIRED_ROWS: [usize; 1] = [27];
/// Local co-op on this computer: start or stop it.
pub const COOP_ROW: usize = 37;
/// How many players local co-op has (2 to 4).
pub const PLAYERS_ROW: usize = 38;
/// Each player's controller (players 1 to 4).
pub const PAD_ROWS: [usize; 4] = [39, 40, 41, 42];
/// Player tags (P1 to P4) over the other players.
pub const TAGS_ROW: usize = 43;
/// A controller choice: Auto, or controller 1 to this many (in the order they were connected).
pub const PAD_CHOICES: u8 = 4;
/// The Multiplayer page's online rows (net.rs): host, the join address, join.
pub const HOST_ROW: usize = 28;
pub const ADDRESS_ROW: usize = 29;
pub const JOIN_ROW: usize = 30;
/// The host sends player 2 away (it keeps hosting).
pub const REMOVE_ROW: usize = 31;
/// The game's password (none by default), typed with letters and digits.
pub const PASSWORD_ROW: usize = 32;
/// Ask the router to forward the port while hosting (UPnP).
pub const UPNP_ROW: usize = 33;
/// The port to host and join on (default 47817), typed with the number keys.
pub const PORT_ROW: usize = 34;
/// Co-op: player 2 kept near player 1, or free to wander the loaded area.
pub const TOGETHER_ROW: usize = 35;
/// Online: a door takes both players, or only the one who goes through it.
pub const DOORS_ROW: usize = 36;
/// Split screen layouts.
pub const SPLITS: [&str; 2] = ["Top and bottom", "Side by side"];

/// The rows of the Display page (0), the Gameplay page (1) or the Multiplayer page (2), in the
/// order shown.
pub fn page_rows(page: u8) -> Vec<usize> {
    match page {
        1 => GAMEPLAY_ROWS.to_vec(),
        2 => MULTI_ROWS.to_vec(),
        _ => (0..ROWS).filter(|k| !GAMEPLAY_ROWS.contains(k) && !MULTI_ROWS.contains(k) && !RETIRED_ROWS.contains(k)).collect(),
    }
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

/// The graphics APIs on offer (main.rs `render_settings`; Windows only, read at start).
pub const GRAPHICS_APIS: [&str; 2] = ["Vulkan", "DX12"];

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
    /// The graphics API on Windows (index into GRAPHICS_APIS), taken at the next start: this
    /// build's option. Vulkan is faster with the display on the discrete GPU; DX12 can be faster
    /// when a laptop's display hangs off its integrated GPU.
    pub graphics_api: usize,
    /// Light shafts and haze from the sun (godrays.rs): this build's option, not in the original.
    pub light_shafts: bool,
    /// Reflections on flat water (water_reflect.rs): this build's option, not in the original.
    pub water_reflections: bool,
    /// Local co-op: a player at 0 health is knocked down and can be revived (true), or the game is
    /// over at once (false).
    pub coop_revive: bool,
    /// Local co-op: the split screen's layout (`SPLITS`).
    pub coop_split: usize,
    /// Local co-op: player 2 is the partner hero when the story has one waiting (true), else
    /// always a second Tor.
    pub coop_partner: bool,
    /// Ask the router to forward the hosting port (UPnP; this build's option, off by default).
    pub upnp: bool,
    /// The UDP port to host on, and to join on when an address names none (default 47817).
    pub net_port: u16,
    /// Co-op: player 2 is brought back beside player 1 when far away (on by default).
    pub coop_together: bool,
    /// Online: a door takes only the player who goes through it (off by default: both).
    pub coop_doors_alone: bool,
    /// Local co-op: how many players (2 to 4).
    pub coop_players: usize,
    /// Each player's controller: 0 Auto, k controller k in the order they were connected.
    pub coop_pads: [u8; 4],
    /// A tag (P1 to P4) over each other player (on by default).
    pub coop_tags: bool,
    /// Steady: an attack turns the hero toward the enemy's body; Original: toward its target
    /// point (SPEC-0040 Behavior 20 item 7, attachment point 7, which on some enemies swings
    /// with their limbs). This build's option, Steady by default (user report 150).
    pub steady_turn: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self { mode: 0, size: (1000, 600), aspect: 0, scale: 4, smooth: true, aa: 2, sharp_textures: true, bloom: 1, colour: 1, ao: true, original_foliage: false, char_shadows: 1, world_shadows: false, shadow_quality: 2, shadow_distance: 2, fps: 2, movies: true, xbox_buttons: false, dpad_hud: true, menu_edges: true, original_neighbour: false, graphics_api: 0, light_shafts: false, water_reflections: false, coop_revive: true, coop_split: 0, coop_partner: true, upnp: false, net_port: crate::net::PORT, coop_together: true, coop_doors_alone: false, coop_players: 2, coop_pads: [0; 4], coop_tags: true, steady_turn: true }
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
                "light shafts" => s.light_shafts = v == "true",
                "water reflections" => s.water_reflections = v == "true",
                "coop revive" => s.coop_revive = v == "true",
                "coop split" => s.coop_split = v.parse().unwrap_or(0).min(SPLITS.len() - 1),
                "coop partner" => s.coop_partner = v == "true",
                "upnp" => s.upnp = v == "true",
                "coop together" => s.coop_together = v == "true",
                "coop doors alone" => s.coop_doors_alone = v == "true",
                "coop tags" => s.coop_tags = v != "false",
                "attack turn" => s.steady_turn = v != "original",
                "coop players" => s.coop_players = v.parse().unwrap_or(2).clamp(2, 4),
                "coop pads" => {
                    for (k, n) in v.split(',').take(4).enumerate() {
                        s.coop_pads[k] = n.trim().parse::<u8>().unwrap_or(0).min(PAD_CHOICES);
                    }
                }
                "net port" => s.net_port = v.parse().ok().filter(|&p: &u16| p > 0).unwrap_or(crate::net::PORT),
                "graphics api" => s.graphics_api = GRAPHICS_APIS.iter().position(|a| a.eq_ignore_ascii_case(v)).unwrap_or(s.graphics_api),
                _ => {}
            }
        }
        s
    }

    pub fn text(&self) -> String {
        format!(
            "mode = {}\nsize = {}x{}\naspect = {}\nscale = {}\nfilter = {}\nantialiasing = {}\nsharp textures = {}\nbloom = {}\ncolour = {}\nambient occlusion = {}\noriginal foliage = {}\ncharacter shadows = {}\nworld shadows = {}\nshadow level = {}\nshadow distance = {}\nframe rate = {}\nmovies = {}\nbutton pictures = {}\ndpad hud = {}\nmenu edges = {}\noriginal neighbour = {}\ngraphics api = {}\nlight shafts = {}\nwater reflections = {}\ncoop revive = {}\ncoop split = {}\ncoop partner = {}\nupnp = {}\nnet port = {}\ncoop together = {}\ncoop doors alone = {}\ncoop players = {}\ncoop pads = {}\ncoop tags = {}\nattack turn = {}\n",
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
            self.original_neighbour,
            GRAPHICS_APIS[self.graphics_api].to_lowercase(),
            self.light_shafts,
            self.water_reflections,
            self.coop_revive,
            self.coop_split,
            self.coop_partner,
            self.upnp,
            self.net_port,
            self.coop_together,
            self.coop_doors_alone,
            self.coop_players,
            self.coop_pads.map(|n| n.to_string()).join(","),
            self.coop_tags,
            if self.steady_turn { "steady" } else { "original" }
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
            ("Graphics API".into(), GRAPHICS_APIS[self.graphics_api].into()),
            ("Light shafts".into(), if self.light_shafts { "On".into() } else { "Off".into() }),
            ("Water reflections".into(), if self.water_reflections { "On".into() } else { "Off".into() }),
            ("Knocked out".into(), if self.coop_revive { "Revive".into() } else { "Game over".into() }),
            ("Split screen".into(), SPLITS[self.coop_split].into()),
            ("Player 2".into(), if self.coop_partner { "Partner".into() } else { "Second Tor".into() }),
            // The online rows' values come from the session (show_window).
            ("Host a game".into(), String::new()),
            ("Join address".into(), String::new()),
            ("Join a game".into(), String::new()),
            ("Remove a player".into(), String::new()),
            ("Password".into(), String::new()),
            ("Open the port (UPnP)".into(), if self.upnp { "On".into() } else { "Off".into() }),
            ("Port".into(), if self.net_port == crate::net::PORT { format!("{} (default)", self.net_port) } else { self.net_port.to_string() }),
            ("Stay together".into(), if self.coop_together { "On".into() } else { "Off".into() }),
            ("Doors take".into(), if self.coop_doors_alone { "Only you".into() } else { "Both".into() }),
            // The Local co-op row's value comes from the game (show_window).
            ("Local co-op".into(), String::new()),
            ("Players".into(), self.coop_players.to_string()),
            ("Player 1 controller".into(), pad_name(self.coop_pads[0])),
            ("Player 2 controller".into(), pad_name(self.coop_pads[1])),
            ("Player 3 controller".into(), pad_name(self.coop_pads[2])),
            ("Player 4 controller".into(), pad_name(self.coop_pads[3])),
            ("Player tags".into(), if self.coop_tags { "On".into() } else { "Off".into() }),
            ("Attack turn".into(), if self.steady_turn { "Steady".into() } else { "Original".into() }),
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
            22 => self.graphics_api = step(self.graphics_api, GRAPHICS_APIS.len()),
            23 => self.light_shafts = !self.light_shafts,
            24 => self.water_reflections = !self.water_reflections,
            25 => self.coop_revive = !self.coop_revive,
            26 => self.coop_split = step(self.coop_split, SPLITS.len()),
            27 => self.coop_partner = !self.coop_partner,
            UPNP_ROW => self.upnp = !self.upnp,
            // Left and Right step the port by one (1024 to 65535).
            PORT_ROW => self.net_port = (self.net_port as i32 + delta).clamp(1024, 65535) as u16,
            TOGETHER_ROW => self.coop_together = !self.coop_together,
            DOORS_ROW => self.coop_doors_alone = !self.coop_doors_alone,
            TAGS_ROW => self.coop_tags = !self.coop_tags,
            ATTACK_TURN_ROW => self.steady_turn = !self.steady_turn,
            PLAYERS_ROW => self.coop_players = 2 + step(self.coop_players.clamp(2, 4) - 2, 3),
            r if PAD_ROWS.contains(&r) => {
                let k = r - PAD_ROWS[0];
                self.coop_pads[k] = step(self.coop_pads[k] as usize, PAD_CHOICES as usize + 1) as u8;
            }
            _ => {}
        }
    }
}

/// A controller choice as shown: Auto, or Controller k.
pub fn pad_name(n: u8) -> String {
    if n == 0 { "Auto".into() } else { format!("Controller {n}") }
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
    pub(crate) image: Option<Handle<Image>>,
    pub(crate) size: (u32, u32),
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
    mut net: ResMut<crate::net::Net>,
    address: Res<crate::net::JoinAddress>,
    mut coop: ResMut<crate::coop::Coop>,
) {
    let _span = crate::perf::span("take_requests");
    let Some(mut menu) = menu else { return };
    if menu.flow.display_requests.is_empty() {
        return;
    }
    let sizes = sizes(&monitors);
    for (row, delta) in std::mem::take(&mut menu.flow.display_requests) {
        // The online rows act on the session (net.rs): host, join, or leave the one running.
        let leave = net.role != crate::net::Role::Off;
        match row {
            HOST_ROW => net.request = Some(if leave && net.role == crate::net::Role::Host { crate::net::Request::Leave } else { crate::net::Request::Host }),
            JOIN_ROW => {
                net.request = Some(if leave && net.role == crate::net::Role::Join { crate::net::Request::Leave } else { crate::net::Request::Join(address.0.clone()) });
                let _ = std::fs::write("private/join.txt", &address.0);
            }
            ADDRESS_ROW => {}
            PASSWORD_ROW => {}
            // Local co-op: Cross (or Left and Right) starts it, or stops it while on.
            COOP_ROW => coop.request = Some(!coop.wanted),
            REMOVE_ROW => {
                if net.role == crate::net::Role::Host && net.any_peer() {
                    net.request = Some(crate::net::Request::Remove);
                }
            }
            _ => settings.change(row, delta, &sizes),
        }
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
    cameras: Query<Entity, (With<Camera3d>, Without<crate::water_reflect::MirrorCam>)>,
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
    "Shadows cast by everything, trees and buildings too, from the area's main light. Not in the original game.",
    "How sharp the shadows from the area's main light are (World shadows and Detailed character shadows). Lower is faster.",
    "How far from the camera those shadows reach. Shorter is faster; shadows end at this distance.",
    "The most frames drawn each second; Uncapped draws as fast as it can. Exclusive full screen may tear. The game's speed does not change.",
    "Off passes over every movie cutscene at once, as if skipped (for testing). Not in the original game.",
    "The button pictures in boxes and menus: the game's PlayStation ones, or Xbox A, B, X and Y.",
    "The D-pad picture on the HUD during play. Off hides it; the D-pad still works.",
    "How menus fill a wide window. Extended carries each picture's edge colours out to the sides; Original keeps the 512 x 448 frame with bars.",
    "Full always draws the whole of the next area beside you; Original hides some of its pieces from parts of this area, as the PS2 game does.",
    "Vulkan runs fastest on most computers. DX12 can be faster on a laptop whose screen is wired to its built-in graphics (for example a gaming laptop in its hybrid or battery GPU mode). Takes effect after a restart.",
    "Beams of sunlight through trees and a soft haze toward the sun. Shafts need World shadows on. Not in the original game.",
    "Flat water mirrors the scenery and sky around it, more at low angles. Not in the original game.",
    "Revive: a fallen player gets up when the other stands beside them. Game over: any fall ends the game.",
    "How two players share the screen: one above the other, or side by side. Three or four players always have quarters.",
    "Partner: player 2 is Roberta when the story has her waiting. Second Tor: always Tor. Players 3 and 4 are always Tor.",
    "Play with a friend on your home network. Start a game first, then host; your friend joins with the address shown.",
    "The host's address (shown on their Host row). Type it with the number keys; Backspace deletes.",
    "Join the host's game. Each player needs their own copy of the game disc.",
    "Hosting: sends the last player who joined back to their title screen. You keep hosting, and they can join again.",
    "Optional. The host types one before hosting; a friend must type the same before joining. Letters and digits; Backspace deletes.",
    "On: while you host, your router is asked to let a friend on the internet in (UPnP). The way in closes when you stop hosting, or by itself after two hours. Some routers have UPnP turned off.",
    "The UDP port the game hosts on, and joins on when the address names none. 47817 unless you change it; both computers should match. Type it with the number keys (Backspace deletes) or step it with Left and Right.",
    "On: player 2 is brought back beside player 1 when more than 60 units away. Off: each player goes where they like in the area around them; a door or a load still takes both, and a fall off the world still brings player 2 back. The host's choice counts online.",
    "Online: Both, a door either player takes moves both. Only you: player 2 can go through a door alone into another world, and the two meet again when one walks into the other's world (a movie or a hero switch on the host brings player 2 back). The host's choice counts.",
    "Cross starts or stops play on this computer. Players 2 to 4 join beside player 1, each with a part of the screen. Not while online.",
    "Players on this computer: 2, 3 or 4. Three or four share the screen in quarters (with three, player 1 has the top half).",
    "Player 1's controller. Auto: the keyboard and every controller no other player has. Player 1 always has the keyboard.",
    "Player 2's controller. Auto: the next free one. Controllers are numbered in the order they were plugged in.",
    "Player 3's controller (3 or 4 players). Auto: the next free controller.",
    "Player 4's controller (4 players). Auto: the next free controller.",
    "On: a coloured tag (P1 to P4) over each other player, so you can tell who is who. Not in the original game.",
    "Steady: an attack turns your hero toward the enemy's body. Original: toward the game's aim point, which on some enemies (the Swamp Tentacle) swings with them and can turn a swing away.",
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
            let order: Vec<usize> = (0..3u8).flat_map(page_rows).chain([ROWS]).collect();
            for k in order {
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
                        r.spawn(Node { width: px(230), justify_content: JustifyContent::Center, ..default() }).with_children(|v| {
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
    (net, address, password): (Res<crate::net::Net>, Res<crate::net::JoinAddress>, Res<crate::net::NetPassword>),
    (coop, play, gamepads): (Res<crate::coop::Coop>, Option<Res<crate::PlayState>>, Query<Entity, With<Gamepad>>),
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
    let mut shown = settings.rows();
    {
        use crate::net::Role;
        shown[HOST_ROW].1 = match net.role {
            Role::Host => "Hosting (Cross stops)".into(),
            _ => "Press to host".into(),
        };
        shown[ADDRESS_ROW].1 = format!("{}{}", address.0, if focus == ADDRESS_ROW { "_" } else { "" });
        shown[JOIN_ROW].1 = match net.role {
            Role::Join if net.following() => "Joined (Cross leaves)".into(),
            Role::Join => "Joining...".into(),
            _ => "Press to join".into(),
        };
        shown[REMOVE_ROW].1 = if net.role == Role::Host && net.any_peer() { format!("Press to remove Player {}", net.peers() + 1) } else { "No one online".into() };
        if focus == PORT_ROW {
            shown[PORT_ROW].1 = format!("{}_", if settings.net_port == 0 { String::new() } else { settings.net_port.to_string() });
        }
        // Local co-op: on or off, and each controller row's controller as it is now.
        let here = play.as_ref().map_or(0, |p| if p.play.net_follow || p.play.coop_remote { 0 } else { p.play.others.len() });
        shown[COOP_ROW].1 = if net.role != Role::Off {
            "Not while online".into()
        } else if coop.wanted {
            if here == 0 { "On (starts in play)".into() } else { format!("On, {} players", here + 1) }
        } else {
            "Press to start".into()
        };
        let mut connected: Vec<Entity> = gamepads.iter().collect();
        connected.sort();
        // While on, as many as are playing (a test switch can ask for another number).
        let count = if coop.wanted && here > 0 { here + 1 } else { settings.coop_players.clamp(2, 4) };
        let pads = crate::coop::assign(&connected, settings.coop_pads, count);
        for (k, &row) in PAD_ROWS.iter().enumerate() {
            let number = pads[k].and_then(|e| connected.iter().position(|&c| c == e)).map(|n| n + 1);
            let choice = settings.coop_pads[k];
            shown[row].1 = match (k, choice, number) {
                _ if k >= count => format!("{} (not playing)", pad_name(choice)),
                (0, 0, _) => "Auto (keyboard + rest)".into(),
                (_, 0, Some(n)) => format!("Auto (Controller {n})"),
                (_, 0, None) => "Auto (none free)".into(),
                (_, c, Some(_)) => pad_name(c),
                (_, c, None) => format!("{} (not connected)", pad_name(c)),
            };
        }
        shown[PASSWORD_ROW].1 = match (password.0.is_empty(), focus == PASSWORD_ROW) {
            (true, false) => "None".into(),
            (true, true) => "_".into(),
            (false, f) => "*".repeat(password.0.len()) + if f { "_" } else { "" },
        };
    }
    let page = page_rows(flow.display_page);
    for (i, r, mut bg, mut node) in &mut rows {
        bg.0 = if r.0 == focus { ROW_CURSOR } else if *i == Interaction::Hovered { ROW_HOVER } else { ROW };
        let d = if r.0 == ROWS || page.contains(&r.0) { Display::Flex } else { Display::None };
        if node.display != d {
            node.display = d;
        }
    }
    let title = ["Display", "Gameplay", "Multiplayer"][flow.display_page.min(2) as usize];
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
    // On the Multiplayer page the session's status follows the help line.
    let line = if flow.display_page == 2 && !net.status.is_empty() { format!("{}\n{}", HELP[focus], net.status_line()) } else { HELP[focus].to_string() };
    for mut t in &mut help {
        if t.0 != line {
            t.0 = line.clone();
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
        assert_eq!(page_rows(1), [17, 18, 19, 20, 44]);
        assert_eq!(page_rows(2), [37, 38, 26, 39, 40, 41, 42, 25, 35, 43, 28, 29, 30, 31, 32, 33, 34, 36]);
        assert_eq!(page_rows(0), (0..17).chain([21, 22, 23, 24]).collect::<Vec<_>>());
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
        // Local co-op: 2 to 4 players (wrapping), each controller Auto or 1 to 4, saved.
        s.change(PLAYERS_ROW, 1, &sizes);
        s.change(PLAYERS_ROW, 1, &sizes);
        assert_eq!(s.rows()[PLAYERS_ROW].1, "4");
        s.change(PLAYERS_ROW, 1, &sizes);
        assert_eq!(s.coop_players, 2, "wraps");
        s.change(PAD_ROWS[1], -1, &sizes);
        assert_eq!(s.rows()[PAD_ROWS[1]].1, "Controller 4", "Left from Auto wraps");
        s.change(PAD_ROWS[3], 1, &sizes);
        assert_eq!(DisplaySettings::parse(&s.text()), s);
        assert_eq!(DisplaySettings::parse("coop pads = 9,x,2").coop_pads, [4, 0, 2, 0]);
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
